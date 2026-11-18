//! Form preview ▸ Use these pages: a dynamic XFA form's computed pages become
//! the document's own, through `EditSession::materialize_xfa`.
//!
//! The pages are laid out exactly as the preview laid them —
//! `dialogs::xfa_preview::lay_out` with the operator's reading choices and the
//! render settings' faces — so what is written is what was shown. Every page
//! is inferred content, so the status row says so, with what was replaced, the
//! faces drawn in a standard face or left unembedded, and the usage-rights
//! signature removed. A signed form is refused, never stripped.
//!
//! A value typed into a field in the preview goes into the form's data through
//! `EditSession::fill_xfa_field`, merged with the same reading choices, so the
//! preview redrawn after it shows the value where it was typed.

use pdfcer_core::edit::{EditError, XfaMaterializeOutcome, XfaSignatures};
use pdfcer_core::xfa::{FontEmbedding, PaintNoteKind, XfaFonts};

use crate::app::state::OpenDoc;
use crate::app::status::decline;
use crate::app::unlock::Refusal;
use crate::dialogs::xfa_preview::{Choices, lay_out};
use crate::text::xfapreview::{self as t, FormPagesRefusal};

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

/// Lay the form out with `choices` and write its pages over the stand-ins.
pub(super) fn apply(doc: &mut OpenDoc, choices: [bool; 5]) {
    let env = crate::app::settings::SettingsExt::render_options(&doc.settings).fonts;
    let fonts = pdfcer_render::font::xfa_fonts(&env);
    let choices = Choices(choices);
    super::apply::vector_edit(doc, "form-pages", 0, 0, |session| {
        let (merged, layout) = lay_out(&session.view(), &fonts, choices).map_err(|why| {
            decline::record_form_pages(FormPagesRefusal::NotLaidOut);
            UseError::Layout(why)
        })?;
        let paint = choices.paint().with_font_embedding(FontEmbedding::Embed);
        let out = session
            .materialize_xfa(
                &merged.template,
                &layout.pages,
                &paint,
                &fonts,
                XfaSignatures::Refuse,
            )
            .map_err(refused)?;
        Ok::<_, UseError>(disclose(&out, &fonts))
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

/// The status-row lines for a written form: what replaced what, what the
/// pages leave out, the faces substituted or left unembedded, and the
/// usage-rights signature removed.
fn disclose(out: &XfaMaterializeOutcome, fonts: &XfaFonts) -> Vec<String> {
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
            "form-pages-applied written={} replaced={} fields={} fallbacks={} unembedded={} notes={}",
            out.pages_written,
            out.pages_replaced,
            out.fields_dropped,
            fallbacks.len(),
            unembedded.len(),
            out.notes.len()
        )
    });
    let mut lines = vec![t::pages_used(
        out.pages_written,
        out.pages_replaced,
        out.fields_dropped,
    )];
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

/// Write `value` into the field at template `path`, as one undo step.
pub(super) fn fill(
    doc: &mut OpenDoc,
    path: &[usize],
    value: &str,
    name: Option<&str>,
    choices: [bool; 5],
) {
    let opts = Choices(choices).merge();
    super::apply::vector_edit(doc, "form-fill", 0, 0, |session| {
        let out = session.fill_xfa_field(path, value, &opts)?;
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
