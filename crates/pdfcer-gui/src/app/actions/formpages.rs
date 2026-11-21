//! Form preview ▸ Use these pages: a dynamic XFA form's computed pages become
//! the document's own, through `EditSession::materialize_merged_xfa`.
//!
//! The pages are laid out exactly as the preview laid them —
//! `dialogs::xfa_preview::lay_out` with the operator's reading choices and the
//! render settings' faces — so what is written is what was shown. With *Keep
//! the fields fillable* on, each field the engine can carry becomes an
//! AcroForm field holding its merged value; the merged form, not its bare
//! template, is passed so fields bound to one data value share one field.
//! Every page is inferred content, so the status row says so, with what was
//! replaced, how many fields stayed fillable and what they do not carry, the
//! faces drawn in a standard face or left unembedded, and the usage-rights
//! signature removed. A signed form is refused, never stripped.
//!
//! A value given to a field in the preview goes into the form's data through
//! `EditSession::fill_xfa_field`, merged with the same reading choices, so the
//! preview redrawn after it shows the value where it was given.

use pdfcer_core::edit::{EditError, XfaMaterializeOutcome, XfaSignatures};
use pdfcer_core::xfa::{
    FieldOutput, FontEmbedding, PaintNoteKind, Template, XfaFieldNoteKind, XfaFillError, XfaFonts,
};

use crate::app::state::OpenDoc;
use crate::app::status::decline;
use crate::app::unlock::Refusal;
use crate::dialogs::xfa_preview::{Choices, lay_out};
use crate::text::xfapreview::{self as t, FormFillRefusal, FormPagesRefusal};

/// How many field names one field-note line lists before "and N more".
const NAMED_PER_NOTE: usize = 5;

/// Why nothing was written.
enum UseError {
    Edit(EditError),
    Layout(String),
}

impl std::fmt::Display for UseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Edit(e) => write!(f, "{e}"),
            Self::Layout(why) => f.write_str(why),
        }
    }
}

impl Refusal for UseError {
    fn encrypted(&self) -> bool {
        matches!(self, Self::Edit(e) if e.encrypted())
    }
}

/// Lay the form out with `choices` and write its pages over the stand-ins,
/// keeping its fields fillable when `fillable`.
pub(super) fn apply(
    doc: &mut OpenDoc,
    choices: pdfcer_gui_base::formpreview::ReadingChoices,
    fillable: bool,
) {
    let env = crate::app::settings::SettingsExt::render_options(&doc.settings).fonts;
    let fonts = pdfcer_render::font::xfa_fonts(&env);
    let choices = Choices(choices);
    super::apply::vector_edit(doc, "form-pages", 0, 0, |session| {
        let (merged, layout) = lay_out(&session.view(), &fonts, choices).map_err(|why| {
            decline::record_form_pages(FormPagesRefusal::NotLaidOut);
            UseError::Layout(why)
        })?;
        let fields = if fillable {
            FieldOutput::Fillable
        } else {
            FieldOutput::Flat
        };
        let paint = choices
            .paint()
            .with_font_embedding(FontEmbedding::Embed)
            .with_fields(fields);
        let out = session
            .materialize_merged_xfa(
                &merged,
                &layout.pages,
                &paint,
                &fonts,
                XfaSignatures::Refuse,
            )
            .map_err(refused)?;
        Ok::<_, UseError>(disclose(&out, &fonts, &merged.template, fillable))
    });
}

/// Word the engine's refusal on the status row, where it has a sentence of
/// its own; the funnel's floor words the rest.
fn refused(e: EditError) -> UseError {
    match &e {
        EditError::XfaNotDynamic => decline::record_form_pages(FormPagesRefusal::NotDynamic),
        EditError::XfaSigned { signatures } => {
            decline::record_form_pages(FormPagesRefusal::Signed(*signatures));
        }
        EditError::CertificationForbidsChange { .. } => {
            decline::record_form_pages(FormPagesRefusal::Certified);
        }
        _ => {}
    }
    UseError::Edit(e)
}

/// The status-row lines for a written form: what replaced what, how many
/// fields stayed fillable and what they leave out, what the pages leave out,
/// the faces substituted or left unembedded, and the usage-rights signature
/// removed.
fn disclose(
    out: &XfaMaterializeOutcome,
    fonts: &XfaFonts,
    template: &Template,
    fillable: bool,
) -> Vec<String> {
    let fallbacks = fonts.fallbacks();
    let unembedded: Vec<String> = out
        .fonts
        .iter()
        .filter(|f| f.supplied && f.program.is_none())
        .map(|f| f.base_font.clone())
        .collect();
    crate::diag::trace(|| {
        format!(
            // ui-text-exempt: diagnostic trace, never displayed in the UI
            "form-pages-applied written={} replaced={} fields={} fallbacks={} unembedded={} notes={} fillable={fillable} fields_written={} field_notes={}",
            out.pages_written,
            out.pages_replaced,
            out.fields_dropped,
            fallbacks.len(),
            unembedded.len(),
            out.notes.len(),
            out.fields_written,
            out.field_notes.len()
        )
    });
    let mut lines = vec![t::pages_used(
        out.pages_written,
        out.pages_replaced,
        out.fields_dropped,
        fillable.then_some(out.fields_written),
    )];
    lines.extend(field_notes(out, template));
    let mut counts: Vec<(PaintNoteKind, usize)> = Vec::new();
    for (_, note) in &out.notes {
        match counts.iter_mut().find(|(k, _)| *k == note.kind) {
            Some((_, n)) => *n += 1,
            None => counts.push((note.kind, 1)),
        }
    }
    if !counts.is_empty() {
        let notes: Vec<String> = counts
            .into_iter()
            .map(|(k, n)| t::paint_note(k, n))
            .collect();
        lines.push(t::pages_notes(&notes));
    }
    if !fallbacks.is_empty() {
        lines.push(t::pages_fallbacks(&fallbacks));
    }
    if !unembedded.is_empty() {
        lines.push(t::pages_not_embedded(&unembedded));
    }
    if out.usage_rights_removed {
        lines.push(t::pages_usage_rights_removed().to_owned());
    }
    lines
}

/// One line per kind of field note, in the order the kinds first occur,
/// naming the first few fields by their template `name`.
fn field_notes(out: &XfaMaterializeOutcome, template: &Template) -> Vec<String> {
    let mut kinds: Vec<(&XfaFieldNoteKind, usize, Vec<String>)> = Vec::new();
    for note in &out.field_notes {
        let at = match kinds.iter().position(|(k, ..)| *k == &note.kind) {
            Some(at) => at,
            None => {
                kinds.push((&note.kind, 0, Vec::new()));
                kinds.len() - 1
            }
        };
        let (_, count, named) = &mut kinds[at];
        *count += 1;
        if named.len() < NAMED_PER_NOTE
            && let Some(name) = template.node_at(&note.path).and_then(|n| n.name())
        {
            named.push(name.to_owned());
        }
    }
    kinds
        .into_iter()
        .map(|(kind, count, named)| t::field_note(kind, count, &named))
        .collect()
}

/// Write `value` into the field at template `path`, as one undo step.
pub(super) fn fill(
    doc: &mut OpenDoc,
    path: &[usize],
    value: &str,
    name: Option<&str>,
    choices: pdfcer_gui_base::formpreview::ReadingChoices,
) {
    let opts = Choices(choices).merge();
    super::apply::vector_edit(doc, "form-fill", 0, 0, |session| {
        let out = session
            .fill_xfa_field(path, value, &opts)
            .map_err(fill_refused)?;
        crate::diag::trace(|| {
            format!(
                // ui-text-exempt: diagnostic trace, never displayed in the UI
                "form-fill-applied path={path:?} chars={} shared={} created={}",
                value.chars().count(),
                out.shared_with.len(),
                out.created.len()
            )
        });
        Ok::<_, EditError>(vec![t::filled(name, out.shared_with.len(), &out.created)])
    });
}

/// Word a fill the engine refused for want of a data value; the funnel's
/// floor words the rest.
fn fill_refused(e: EditError) -> EditError {
    match &e {
        EditError::XfaFill(XfaFillError::DataRefAbsent { reference }) => {
            decline::record_form_fill(FormFillRefusal::DataRefAbsent(reference.clone()));
        }
        EditError::XfaFill(XfaFillError::Unbound) => {
            decline::record_form_fill(FormFillRefusal::Unbound);
        }
        _ => {}
    }
    e
}
