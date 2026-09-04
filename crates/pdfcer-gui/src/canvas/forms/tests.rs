//! # `canvas::forms` tests — the focus's own assertions
//!
//! Split out of `canvas/forms.rs` on 2026-09-04 under **R2**. The parent
//! reached 1,444 of the 1,500-line ceiling and the review row A12c needed one
//! more function ([`super::live_draft`]), its documentation and its test,
//! which would not fit.
//!
//! ★ The seam is the one this crate has now taken half a dozen times
//! (`app/state`, `app/prefs`, `canvas/geometry`, `canvas/selection`): the
//! parent answers *"what does this surface do?"* and this answers *"is it
//! still doing it?"*. Nothing moved but its address — the two tests that were
//! in the parent are byte-identical here, and the third is new.
//!
//! Note that these are the tests of the parent's **impure** half. The pure
//! rules — classification, placement, the hit test, the editor's geometry and
//! `/Q` — are tested in [`super::boxes`], beside the functions they pin, and
//! must go on being tested there.

// ★ The INNER attribute, and it is load-bearing for more than the compiler.
//
// `tools/gates/check-ui-strings.sh` stops scanning a file at `#![cfg(test)]`,
// because a test assertion message is read by whoever is staring at a failing
// test and is not operator copy. Without this line the split would turn every
// assertion message below into a gate violation — which is what it did on the
// first attempt at `canvas/geometry/tests.rs`, whose header records it.
#![cfg(test)]

use super::*;
/// ★ **A focus that outlives its document is discarded; one that outlives
/// a revision is re-seeded.**
///
/// The difference from the panel, pinned. Dropping the focus on an epoch
/// change would take the caret out of a field the operator had just
/// clicked into, because clicking field B while A is focused commits A and
/// therefore moves the epoch on the very next frame.
#[test]
fn an_edit_reseeds_the_draft_and_a_different_document_discards_it() {
    let doc = crate::app::state::open_fixture(crate::app::state::FOUR_PAGES);
    let path = doc.path.clone();

    let focus = Focus {
        path: path.clone(),
        epoch: doc.edit_epoch,
        page: 0,
        field: "Name".to_owned(),
        widget: 0,
        draft: "Anna".to_owned(),
        seated: true,
    };

    // Same document, same revision: the draft survives, or typing would be
    // impossible.
    let same = focus.clone().sync(&doc, "").expect("same document");
    assert_eq!(same.draft, "Anna");

    // An edit landed: the draft is replaced by what the document holds,
    // and the FOCUS survives.
    let moved = Focus {
        epoch: doc.edit_epoch.wrapping_sub(1),
        ..focus.clone()
    };
    let reseeded = moved.sync(&doc, "Committed").expect("the focus survives");
    assert_eq!(reseeded.draft, "Committed");
    assert_eq!(reseeded.field, "Name");
    assert_eq!(reseeded.epoch, doc.edit_epoch);

    // A different document: nothing here means anything.
    let elsewhere = Focus {
        path: PathBuf::from("other.pdf"),
        ..focus
    };
    assert!(elsewhere.sync(&doc, "").is_none());
}
/// ★ **Escape is reported once, and cleared by the reading.**
///
/// Claimant 0's contract. A flag that survived its reading would spend the
/// *next* Escape as well — which the operator would experience as a press
/// that failed to ascend the selection ladder for no visible reason.
#[test]
fn escape_is_claimed_exactly_once() {
    let ctx = egui::Context::default();
    assert!(!escape_spent(&ctx), "nothing focused: the key is not ours");
    note_escape(&ctx);
    assert!(escape_spent(&ctx));
    assert!(!escape_spent(&ctx), "and it is not claimed twice");
}

/// ★★★ **The panel can read what is being typed on the page — but only while
/// the page is the thing being typed into.**
///
/// The 2026-09 review's row **A12c**, pinned from both sides. The positive
/// half is the fix: a stored [`Focus`] whose editor owns the keyboard is
/// published to [`crate::panels::forms::rows`], so the panel row beside the
/// field stops showing the value from before the gesture started.
///
/// ★★ The three negative halves are the safety argument, and each of them is
/// a way for this fix to become a worse defect than the one it replaces:
///
/// * **No `egui` focus** — a `Focus` outlives the frames the editor is
///   actually focused for, and a panel is drawn on every frame. Publishing one
///   then would let a stale canvas draft overwrite a live panel one, which is
///   A12c pointing the other way.
/// * **A different revision** — an edit landed, and the panel drops its own
///   drafts on exactly that boundary (`FormsUi::load`). Mirroring across it
///   would hand back the one value that key exists to discard.
/// * **A different document** — every field name means something else.
#[test]
fn the_panel_reads_the_pages_draft_only_while_the_page_holds_the_keyboard() {
    let doc = crate::app::state::open_fixture(crate::app::state::FOUR_PAGES);
    let ctx = egui::Context::default();
    let focus = Focus {
        path: doc.path.clone(),
        epoch: doc.edit_epoch,
        page: 0,
        field: "Name".to_owned(),
        widget: 0,
        draft: "Ann".to_owned(),
        seated: true,
    };

    // Focused on the page: the panel gets the live draft.
    store_focus(&ctx, Some(focus.clone()));
    ctx.memory_mut(|m| m.request_focus(focus.editor_id()));
    assert_eq!(
        live_draft(&ctx, &doc),
        Some(("Name".to_owned(), "Ann".to_owned())),
    );

    // The keyboard is somewhere else — the panel's own row, a dialog, nothing
    // at all. The canvas has an opinion and it is not the live one.
    ctx.memory_mut(|m| m.surrender_focus(focus.editor_id()));
    assert!(
        live_draft(&ctx, &doc).is_none(),
        "an unfocused editor's draft must not overwrite the panel's",
    );

    // A revision the draft does not describe.
    ctx.memory_mut(|m| m.request_focus(focus.editor_id()));
    store_focus(
        &ctx,
        Some(Focus {
            epoch: doc.edit_epoch.wrapping_sub(1),
            ..focus.clone()
        }),
    );
    assert!(live_draft(&ctx, &doc).is_none(), "a stale revision");

    // A document the draft does not describe.
    store_focus(
        &ctx,
        Some(Focus {
            path: PathBuf::from("other.pdf"),
            ..focus
        }),
    );
    assert!(live_draft(&ctx, &doc).is_none(), "another document");
}
