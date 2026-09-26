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
//!
//! Design and rationale: `docs/modules/pdfcer-gui/app/actions/forms/paste.md`.

use crate::app::state::OpenDoc;

/// **Author the form control that came off the clipboard.**
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
