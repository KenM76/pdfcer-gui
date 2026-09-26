//! # `dialogs::import_text` tests — the window's own decisions
//!
//! ## What these can and cannot prove
//!
//! They cannot prove the operator can import a text file. Every test here calls
//! a function directly; the picker, the dispatch, the dialog host, the apply
//! arm and the engine are all between this and a working feature, and
//! `tools/ui-verify` is the instrument for that. R1 is not relaxed.
//!
//! What they prove is the part this window **decides**, and it is a short list
//! because that is the design: the window chooses four things and takes the
//! engine's answer for everything else.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/dialogs/import_text/tests.md`.

#![cfg(test)]

use super::*;

/// **The window opens on A4, found by NAME rather than by position.**
#[test]
fn the_window_opens_on_a4_whatever_order_the_engine_lists_its_sheets_in() {
    let dialog = ImportTextDialog::open(std::path::PathBuf::from("register.txt"), 0);
    assert_eq!(
        sheet_of(dialog.sheet).id(),
        "a4",
        "the import window must open on A4 — this operator's world is metric, and every other \
         sheet chooser in this program opens on a metric size"
    );
}

/// **The template takes the engine's defaults for everything the window does
/// not draw a control for.**
#[test]
fn the_template_keeps_every_engine_default_the_window_does_not_control() {
    let dialog = ImportTextDialog::open(std::path::PathBuf::from("register.txt"), 0);
    let built = dialog.template();
    let engine = pdfcer_core::text_edit::PageTemplate::new();

    assert_eq!(
        built.alignment, engine.alignment,
        "alignment is not a control in this window and must be the engine's"
    );
    assert_eq!(
        built.leading, engine.leading,
        "leading is not a control in this window and must be the engine's derived default"
    );
    assert!(
        matches!(built.unmappable, pdfcer_core::text_edit::Unmappable::Refuse),
        "the unmappable policy must stay `Refuse`: it is what makes a character the face cannot \
         write stop the import instead of arriving as a silent gap, and this window offers no \
         control to change it"
    );
}

/// **The four controls the window DOES draw reach the template.**
#[test]
fn the_four_controls_reach_the_template_and_the_margin_reaches_all_four_sides() {
    let mut dialog = ImportTextDialog::open(std::path::PathBuf::from("register.txt"), 0);
    dialog.margin = 36.0;
    dialog.size = 9.5;
    dialog.face = 4; // Courier — see `FACES`.
    let built = dialog.template();

    assert!((built.margin_left - 36.0).abs() < 1e-9);
    assert!((built.margin_right - 36.0).abs() < 1e-9);
    assert!((built.margin_top - 36.0).abs() < 1e-9);
    assert!((built.margin_bottom - 36.0).abs() < 1e-9);
    assert!((built.size - 9.5).abs() < 1e-9);
    assert_eq!(
        built.face,
        pdfcer_core::fontdata::Std14::Courier,
        "the face chooser must reach the template — Courier is the one choice that changes \
         whether a space-aligned register is readable"
    );
}

/// **A stored sheet index past the end of the engine's list does not
/// panic.**
#[test]
fn a_sheet_index_past_the_end_of_the_engines_list_clamps_rather_than_panicking() {
    let mut dialog = ImportTextDialog::open(std::path::PathBuf::from("register.txt"), 0);
    dialog.sheet = usize::MAX;
    let built = dialog.template();
    assert!(
        built.media_box.width() > 0.0 && built.media_box.height() > 0.0,
        "a stored index past the end of `PaperSize::ALL` must clamp to a real sheet"
    );
    // And the face index, which has the same shape and the same hazard.
    dialog.face = usize::MAX;
    let _ = dialog.template();
    let _ = face_name(usize::MAX);
}

/// **The four radios convert to the engine's four positions**, and two of
/// them carry the page the dialog froze.
#[test]
fn the_radios_carry_the_frozen_page_into_the_engines_position() {
    use pdfcer_core::pageops::InsertPosition;
    let mut dialog = ImportTextDialog::open(std::path::PathBuf::from("register.txt"), 7);

    dialog.position = Where::BeforeCurrent;
    assert_eq!(dialog.insert_position(), InsertPosition::Before(7));
    dialog.position = Where::AfterCurrent;
    assert_eq!(dialog.insert_position(), InsertPosition::After(7));
    dialog.position = Where::Start;
    assert_eq!(dialog.insert_position(), InsertPosition::Start);
    dialog.position = Where::End;
    assert_eq!(dialog.insert_position(), InsertPosition::End);
}

/// **Every face the chooser offers has a label**, and no label is the
/// fallback by accident.
#[test]
fn every_offered_face_has_its_own_label() {
    let mut seen = std::collections::BTreeSet::new();
    for index in 0..FACES.len() {
        assert!(
            seen.insert(face_name(index)),
            "two faces in `FACES` share the label {:?} — one of them is falling through \
             `face_name`'s catch-all arm, and the chooser shows the same word twice",
            face_name(index)
        );
    }
    assert_eq!(seen.len(), FACES.len());
}
