//! # `app::actions::forms::paste` — putting a copied form field back
//!
//! One verb, `EditSession::paste_field`, and the reason it is a module of its
//! own is R2: `super`'s file hit the 1,500-line ceiling the moment this arrived,
//! and this is the seam — a paste is *authoring from a source*, which is a
//! different subject from `super::author`'s *authoring from choices*.
//!
//! ## Why the shell does almost nothing here
//!
//! A paste routed through `add_text_field` and its siblings would have to
//! **re-author**: read the source field into a `canvas::formfield::Draft`,
//! mapping the field type, decoding the text strings, translating the flags.
//! `New*Field` is a **spec** — geometry plus a dozen booleans — so such a paste
//! could carry only what the spec can *express*, and these are readable on
//! `forms::Field` and writable nowhere: `/DA`, `/Q`, `/DV`, `/AA`, `/MK`'s
//! colours, `/BS`'s styles beyond solid, the `/Ff` bits no spec names, and the
//! baked `/AP`. `FieldClip` does not express properties; it **carries** them,
//! which is why `copy_field` / `paste_field` is the pair used here.
//!
//! ⇒ What is left here is the three things the *shell* owns and the engine
//! cannot know: which policy the operator's chord meant, what to call a new
//! field, and what to leave selected afterwards.
//!
//! ## Rule 4 — the disclosure is the engine's, verbatim
//!
//! `FieldPasteOutcome::disclosures` reaches the status row through
//! `vector_edit`, like every other verb's. Nothing here paraphrases it: *one
//! fact, one wording*, and the engine's version is the authoritative one because
//! it reports what the operation **did** rather than what the shell intended.

use crate::app::state::OpenDoc;

/// **Author the form control that came off the clipboard.**
///
/// # Why this is not [`author`]
///
/// Reusing [`author`] would be the right instinct — one authoring path, not two
/// — and it is wrong here for a reason that shows up in the file rather than on
/// the screen. `New*Field` is a **spec**: geometry plus a dozen booleans. So a
/// re-authored paste can carry only what the spec can *express*, and these are
/// readable on `forms::Field` and writable nowhere — `/DA` (the font, its size
/// and its colour), `/Q`, `/DV`, `/AA`, `/MK`'s border and background colours,
/// `/BS`'s styles beyond solid, the `/Ff` bits no spec names, and the baked
/// `/AP`. A shell that re-authored would have to disclose that loss and carry a
/// hand-written table of it, and such a table *"rots silently every time we add
/// an authoring key."*
///
/// `paste_field` does not *express* properties, it **carries** them, so there is
/// no table to maintain.
///
/// ⇒ [`author`] is still the right verb for the **dialog**, where the operator
/// is choosing values and a spec is exactly what a form of controls produces.
/// The two are not duplicates; they are authoring-from-choices and
/// authoring-from-a-source.
///
/// # The disclosures are the ENGINE's and are surfaced verbatim
///
/// `FieldPasteOutcome::disclosures` is a `Vec<String>` covering a dropped
/// value, dropped actions, a carried calculation and its `/CO` registration, a
/// **renamed font resource**, an ignored rectangle size on a radio group, the
/// tab-order position, a dropped structure-tree link and a reused accessibility
/// name. Rule 4's off-canvas obligation lands there, and this function does not
/// re-derive a word of it — *one fact, one wording*.
///
/// The engine's own note calls it *"not optional reading"*, so `vector_edit`
/// carries it to the status row exactly as every other verb's disclosures.
///
/// # Selecting what landed
///
/// [`author`]'s O53 behaviour, kept: a newly placed field is left selected so
/// the grips are already there and the next drag is already live. Widget 0 for a
/// new field, because that is the one the engine placed first; for a duplicate
/// the operator's own widget index is unknown until the outcome comes back, so
/// the **last** widget of the field is the one that just arrived.
pub(super) fn paste(
    doc: &mut OpenDoc,
    page: usize,
    rect: pdfcer_core::page_tree::Rect,
    clip: &[u8],
    policy: &pdfcer_core::formclip::FieldPastePolicy,
) {
    use pdfcer_core::formclip::{FieldClip, FieldPastePolicy};

    let clip = match FieldClip::from_bytes(clip) {
        Ok(c) => c,
        Err(e) => {
            // A clip this shell wrote itself failing to read back is not an
            // operator error and not a document error — it is a version skew
            // between the writer and the reader, which is exactly what the
            // format's magic and version word exist to catch. Traced and
            // reported, never silent.
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed.
                format!("paste-field-refused page={page} reason=unreadable-clip err={e}")
            });
            crate::app::actions::record_note(
                doc.edit_epoch,
                crate::text::fieldclip::refusal(&crate::canvas::fieldclip::Refusal::EngineRefused(
                    e.to_string(),
                )),
            );
            return;
        }
    };

    // The name to select afterwards, taken BEFORE the move into the closure.
    let wanted = match policy {
        FieldPastePolicy::NewField { name, .. } => name.clone(),
        FieldPastePolicy::AdditionalWidget { existing } => existing.clone(),
        // `FieldPastePolicy` is `#[non_exhaustive]`, so a third policy the
        // engine adds later compiles here rather than breaking the build. It
        // reaches this arm with no name to select afterwards, which costs the
        // O53 re-selection and nothing else -- the paste itself still happens.
        // Traced, so a build that started taking this arm says so instead of
        // quietly losing the selection.
        _ => {
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed.
                "paste-field-unknown-policy reason=engine-added-a-variant".to_owned()
            });
            String::new()
        }
    };
    // NOTHING HERE READS `outcome.merged`, AND THAT IS DELIBERATE.
    //
    // `FieldPasteOutcome::merged` does **not** mean what
    // `FieldAuthorOutcome::merged` means, and on the two cases this shell has,
    // the two are INVERTED:
    //
    // | type | `merged` means |
    // |---|---|
    // | `FieldAuthorOutcome` (`add_*_field`) | this widget joined an EXISTING field |
    // | `FieldPasteOutcome` (`paste_field`)  | the field is in Shape A — one dict that is both field and widget (12.5.6.19) |
    //
    // Measured on the release binary:
    //
    //     Ctrl+V       (a NEW independent field)    -> merged=true
    //     Ctrl+Shift+V (another widget of the SAME) -> merged=false
    //
    // Both are correct under their own type's definition. The trap is that a
    // confirmation phrased from `FieldAuthorOutcome::merged` reads correctly
    // until it is pointed at a paste, where it prints exactly the wrong
    // sentence on exactly the wrong chord — and survives review, because the
    // field name is the same.
    //
    // ⇒ The sentence here is the ENGINE's, verbatim, through `disclosures`. The
    // field that says which of the two happened is `created`, not `merged`.
    let before = doc.edit_epoch;
    let widgets = std::cell::Cell::new(0usize);

    crate::app::actions::apply::vector_edit(doc, "paste-field", page, 1, |session| {
        session
            .paste_field(&clip, page, rect, policy)
            .map(|outcome| {
                widgets.set(outcome.widget_ids.len());
                crate::diag::trace(|| {
                    // ui-text-exempt: diagnostic trace, never displayed.
                    format!(
                        "paste-field-applied page={page} merged={} created={} widgets={}",
                        outcome.merged,
                        outcome.created,
                        outcome.widget_ids.len()
                    )
                });
                outcome.disclosures
            })
    });

    // O53 — leave what was just placed selected. Guarded on the epoch, so a
    // refusal leaves the previous selection alone rather than pointing at a
    // field that was never created.
    if doc.edit_epoch != before && !wanted.is_empty() {
        let index = widget_index_after_paste(doc, &wanted);
        doc.selected_field = Some(crate::app::state::SelectedField {
            field: wanted,
            widget: index,
            page,
        });
    }
}

/// Which widget of `fqn` the paste just added — the last one.
///
/// Re-read from the document rather than derived from the outcome's
/// `widget_ids`, because the two address spaces differ: the outcome names
/// `ObjId`s and `SelectedField` wants an **index within the field**. Reading
/// the field back is the only thing that knows both.
///
/// Zero when the field cannot be found, which is unreachable on the success
/// path and is a defensible index rather than a panic if it ever is not.
fn widget_index_after_paste(doc: &OpenDoc, fqn: &str) -> usize {
    let view = doc.session.view();
    pdfcer_core::forms::parse_acroform(&view)
        .and_then(|form| {
            form.fields_named(fqn)
                .next()
                .map(|f| f.widgets.len().saturating_sub(1))
        })
        .unwrap_or(0)
}
