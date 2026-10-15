//! `edit.duplicate`, Ctrl+D: a second copy of everything selected — page
//! objects, parts of a placed drawing, annotations — offset so it is visible,
//! as one undoable paste, without touching the clipboard.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/canvas/duplicate.md`.

use super::clipboard::{Clipped, PASTE_OFFSET_PT, Refusal};
use crate::app::actions::{Action, VectorAction};
use crate::app::state::OpenDoc;

/// Lift the selection and raise one `PasteObjects` of it at the paste offset.
/// The returned record carries what the lift left out, for the disclosure.
pub fn duplicate(doc: &mut OpenDoc, actions: &mut Vec<Action>) -> Result<Clipped, Refusal> {
    let taken = super::take::selected(doc)?;
    let (dx, dy) = (PASTE_OFFSET_PT, -PASTE_OFFSET_PT);
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed.
        format!(
            "selection-duplicate kind={} page={} objects={} items={} annots={} thin={} \
             left_behind={} in_form_left={} dx={dx:.1} dy={dy:.1}",
            taken.kind.name(),
            taken.page,
            taken.objects,
            taken.clip.len(),
            taken.plan.carried(),
            taken.plan.thin,
            taken.plan.refused.len(),
            taken.in_form_left,
        )
    });
    actions.push(
        VectorAction::PasteObjects {
            page: taken.page,
            clip: taken.clip.to_bytes(),
            at: pdfcer_core::vector::Matrix::translate(dx, dy),
        }
        .into(),
    );
    Ok(taken.clipped())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::canvas::target::TargetId;

    /// The one `PasteObjects` a duplicate must raise, or a panic naming what
    /// it raised instead.
    fn the_paste(actions: &[Action]) -> (usize, pdfcer_core::vector::Matrix) {
        let [Action::Vector(VectorAction::PasteObjects { page, at, .. })] = actions else {
            panic!("a duplicate raises exactly one PasteObjects: {actions:?}");
        };
        (*page, *at)
    }

    #[test]
    fn a_page_object_duplicates_as_one_paste_at_the_offset() {
        let mut doc = crate::app::state::open_local_fixture("overlapping-boxes.pdf");
        doc.selection.select_only(0, TargetId::Object(0), "test");
        let mut actions = Vec::new();
        let clipped = duplicate(&mut doc, &mut actions).expect("a page object duplicates");
        let (page, at) = the_paste(&actions);
        assert_eq!(page, 0);
        assert_eq!(
            at,
            pdfcer_core::vector::Matrix::translate(PASTE_OFFSET_PT, -PASTE_OFFSET_PT),
            "right and down the page, as a paste onto the same page lands"
        );
        assert!(matches!(clipped, Clipped::Selection { count: 1, .. }));
    }

    #[test]
    fn a_part_of_a_placed_drawing_duplicates_as_page_content() {
        let mut doc = crate::app::state::open_local_fixture("half-scale-form.pdf");
        doc.selection.select_only(0, TargetId::Leaf(0), "test");
        let mut actions = Vec::new();
        duplicate(&mut doc, &mut actions).expect("a drawing part duplicates");
        assert_eq!(the_paste(&actions).0, 0);
    }

    #[test]
    fn nothing_selected_refuses_and_raises_nothing() {
        let mut doc = crate::app::state::open_local_fixture("overlapping-boxes.pdf");
        let mut actions = Vec::new();
        assert_eq!(
            duplicate(&mut doc, &mut actions),
            Err(Refusal::NothingSelected)
        );
        assert!(actions.is_empty(), "a refusal raises nothing: {actions:?}");
    }

    /// The signature takes no `egui::Context`, so the duplicate cannot reach
    /// the clipboard: that is why the command exists beside Copy and Paste.
    #[test]
    fn the_duplicate_cannot_reach_the_clipboard() {
        let _: fn(&mut OpenDoc, &mut Vec<Action>) -> Result<Clipped, Refusal> = duplicate;
    }
}
