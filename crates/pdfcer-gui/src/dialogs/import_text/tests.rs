//! # `dialogs::import_text` tests — the window's own decisions
//!
//! ## ★★★ What these can and cannot prove
//!
//! They cannot prove the operator can import a text file. Every test here calls
//! a function directly; the picker, the dispatch, the dialog host, the apply
//! arm and the engine are all between this and a working feature, and
//! `tools/ui-verify` is the instrument for that. R1 is not relaxed.
//!
//! What they prove is the part this window **decides**, and it is a short list
//! because that is the design: the window chooses four things and takes the
//! engine's answer for everything else.

#![cfg(test)]

use super::*;

/// ★★★ **The window opens on A4, found by NAME rather than by position.**
///
/// `PaperSize::ALL`'s order is the engine's business and it has said the table
/// will grow. A hard-coded index would silently open on a different sheet the
/// day one is inserted before A4 — and a window that opens on the wrong paper
/// is a defect an operator only notices *after* importing, when the pages are
/// already in his document.
///
/// ★ The test asserts the **id**, not the index, for the same reason the
/// implementation looks it up by id: an index asserted here would go green
/// against a reordered table while the window opened on Letter.
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

/// ★★ **The template takes the engine's defaults for everything the window does
/// not draw a control for.**
///
/// `PageTemplate` has ten fields and this window offers four. The other six —
/// `leading`, `alignment`, `color`, `unmappable` and the two margins that are
/// not separately controlled — must arrive as `PageTemplate::new()` set them,
/// so that a field the engine adds tomorrow comes with the engine's default
/// rather than a zero this shell invented.
///
/// ★ `unmappable` is the one that matters most: its default is `Refuse`, which
/// is what makes a text file full of characters the face cannot write **stop**
/// rather than arrive with silent gaps. A window that reconstructed the
/// template field by field could drop that without any test noticing.
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

/// ★★ **The four controls the window DOES draw reach the template.**
///
/// The other half of the test above, and it needs saying separately: a build
/// that returned `PageTemplate::new()` unchanged would pass every assertion
/// there and ignore every choice the operator made.
///
/// ★ The margin is asserted on **all four** sides. The window offers one
/// spinner and the engine has four fields; a build that set only `margin_left`
/// would produce a page with text running off three edges, and it would look
/// like a rendering fault.
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

/// ★★★ **A stored sheet index past the end of the engine's list does not
/// panic.**
///
/// `PaperSize::ALL` can **shrink** between builds as well as grow — the engine
/// says the table moves — and this window stores an index. Clamping rather than
/// indexing is what stops a window failing to open, which is a far worse
/// outcome than opening on the wrong sheet with the chooser right there.
///
/// ★ Asserted at `usize::MAX` rather than `len()`, because the interesting
/// failure is not off-by-one — it is a stored value from a completely different
/// table.
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

/// ★★ **The four radios convert to the engine's four positions**, and two of
/// them carry the page the dialog froze.
///
/// The conversion is the reason `Where` exists as a local enum at all —
/// `dialogs::insert_pages` states it: two of the four need the current page
/// index, which the radio does not carry and the dialog does.
///
/// ★ It asserts against page **7** rather than 0, because `Before(0)` and
/// `Start` are the same position and a test using the first page could not tell
/// a build that confused them from a correct one.
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

/// ★ **Every face the chooser offers has a label**, and no label is the
/// fallback by accident.
///
/// `face_name`'s `_` arm answers `"Helvetica"`, which is correct for
/// `Std14::Helvetica` and would be *silently wrong* for any face added to
/// [`FACES`] without a matching arm. This is what notices.
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
