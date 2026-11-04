//! # `app::conditions::offered` — which selection-dependent right-click rows are offered
//!
//! The `*_OFFERED`, `*_ALLOWED` and `LAYER_ASSIGNABLE` names of `shell::menus`,
//! each asked on the same test the row's dispatcher arm routes on, so a row is
//! drawn exactly when its command can act.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/app/conditions/offered.md`.

use egui_shell::commands::ConditionSet;

use crate::app::state::OpenDoc;

impl crate::app::PdfcerApp {
    /// Publish the offered and allowed states of the selection's menu rows.
    pub(super) fn offered_conditions(&self, doc: &OpenDoc, set: &mut ConditionSet) {
        // One circular ce dimension selected: the radius / diameter switch
        // offers whichever measure it does not show.
        match crate::app::dispatch::dimdisplay::circular(doc, &doc.selection) {
            Some((_, false)) => set.set(crate::shell::menus::DIMENSION_DIAMETER_OFFERED),
            Some((_, true)) => set.set(crate::shell::menus::DIMENSION_RADIUS_OFFERED),
            None => {}
        }
        // One closed perimeter ce dimension: the area / perimeter switch.
        match crate::app::dispatch::dimdisplay::perimeter(doc, &doc.selection) {
            Some((_, false)) => set.set(crate::shell::menus::DIMENSION_AREA_OFFERED),
            Some((_, true)) => set.set(crate::shell::menus::DIMENSION_PERIMETER_OFFERED),
            None => {}
        }
        // Two or more runs of one page text object: `format.merge_text_runs`
        // is offered, and enabled when the engine's preflight agrees.
        if !doc.selection.is_empty() {
            let merge = crate::canvas::runmerge::operand(
                doc.page_objects().as_deref(),
                &doc.selection,
                doc.view.page_index,
            );
            if let Some(merge) = merge {
                set.set(crate::shell::menus::TEXT_MERGE_OFFERED);
                if merge.allowed() {
                    set.set(crate::shell::menus::TEXT_MERGE_ALLOWED);
                }
            }
            // One text object of several lines: `format.split_text_lines`.
            let split = crate::canvas::runsplit::operand_of(doc);
            if let Some(split) = split {
                set.set(crate::shell::menus::TEXT_SPLIT_OFFERED);
                if split.allowed() {
                    set.set(crate::shell::menus::TEXT_SPLIT_ALLOWED);
                }
            }
        }
        // A movable selection in a document with layers:
        // `format.move_to_layer`.
        if self.capabilities().edit_content
            && crate::app::actions::layerassign::offered(doc, &doc.selection)
        {
            set.set(crate::shell::menus::LAYER_ASSIGNABLE);
        }
    }
}
