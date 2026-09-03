//! # `app::actions::forms::paste` — putting a copied form field back
//!
//! One verb, `EditSession::paste_field`, and the reason it is a module of its
//! own is R2: `super`'s file hit the 1,500-line ceiling the moment this arrived,
//! and this is the seam — a paste is *authoring from a source*, which is a
//! different subject from `super::author`'s *authoring from choices*.
//!
//! ## ★★★ Why the shell does almost nothing here
//!
//! It used to do a great deal. Until 2026-08-29 a paste **re-authored**: it read
//! the source field into a `canvas::formfield::Draft` — mapping the field type,
//! decoding four text strings, translating eleven flags — and pushed that back
//! through `add_text_field` and its four siblings. About eighty lines on this
//! side of the boundary, every one a chance to disagree with the engine about
//! what a form field is.
//!
//! `Pass 167.0` replaced all of it with `copy_field` / `paste_field`, and the
//! difference is not a line count. `New*Field` is a **spec** — geometry plus a
//! dozen booleans — so a re-authored paste could only carry what the spec can
//! *express*, and eight properties were readable on `forms::Field` and writable
//! nowhere: `/DA`, `/Q`, `/DV`, `/AA`, `/MK`'s colours, `/BS`'s styles beyond
//! solid, the `/Ff` bits no spec names, and the baked `/AP`. `FieldClip` does
//! not express properties; it **carries** them.
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

/// **Author the form control that came off the clipboard** (`Pass 167.0`).
///
/// # ★★★ Why this is not [`author`], which it superseded within the hour
///
/// The first version of the paste **re-authored**: it built a
/// `canvas::formfield::Draft` from the source field and pushed it back through
/// `add_text_field` and its four siblings, reusing [`author`] entirely. That is
/// the right instinct — one authoring path, not two — and it was wrong here for
/// a reason that only shows up in the file rather than on the screen.
///
/// `New*Field` is a **spec**: geometry plus a dozen booleans. So a re-authored
/// paste can only carry what the spec can *express*, and eight properties are
/// readable on `forms::Field` and writable nowhere — `/DA` (the font, its size
/// and its colour), `/Q`, `/DV`, `/AA`, `/MK`'s border and background colours,
/// `/BS`'s styles beyond solid, the `/Ff` bits no spec names, and the baked
/// `/AP`. The shell disclosed the loss honestly and carried a hand-written table
/// of it.
///
/// `paste_field` does not *express* properties, it **carries** them. So the
/// table is deleted rather than maintained, at the engine's own instruction:
/// *"it rots silently every time we add an authoring key."*
///
/// ⇒ [`author`] is still the right verb for the **dialog**, where the operator
/// is choosing values and a spec is exactly what a form of controls produces.
/// The two are not duplicates; they are authoring-from-choices and
/// authoring-from-a-source.
///
/// # ★★ The disclosures are the ENGINE's and are surfaced verbatim
///
/// `FieldPasteOutcome::disclosures` is a `Vec<String>` covering a dropped
/// value, dropped actions, a carried calculation and its `/CO` registration, a
/// **renamed font resource**, an ignored rectangle size on a radio group, the
/// tab-order position, a dropped structure-tree link and a reused accessibility
/// name. Rule 4's off-canvas obligation lands there, and this function does not
/// re-derive a word of it — *one fact, one wording*.
///
/// ★ The engine's own note calls it *"not optional reading"*, so `vector_edit`
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
            // ★ A clip this shell wrote itself failing to read back is not an
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
        // ★ `FieldPastePolicy` is `#[non_exhaustive]`, so a third policy the
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
    // ★★★ NOTHING HERE READS `outcome.merged`, AND THAT IS DELIBERATE.
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
    // Measured on the release binary, 2026-08-29:
    //
    //     Ctrl+V       (a NEW independent field)    -> merged=true
    //     Ctrl+Shift+V (another widget of the SAME) -> merged=false
    //
    // Both are correct under the paste type's own definition. The trap is that
    // this module's predecessor routed through `add_*_field`, read
    // `FieldAuthorOutcome::merged`, and phrased the operator's confirmation
    // from it. Keeping that line across the migration would print exactly the
    // wrong sentence on exactly the wrong chord — and it would survive review,
    // because the field name did not change.
    //
    // ⇒ The sentence is the ENGINE's now, verbatim, through `disclosures`. If a
    // future author needs to know which of the two happened, the field is
    // `created`, not `merged`. Reported to the engine as a naming hazard.
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

    // ★ O53 — leave what was just placed selected. Guarded on the epoch, so a
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
/// ★ Re-read from the document rather than derived from the outcome's
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
