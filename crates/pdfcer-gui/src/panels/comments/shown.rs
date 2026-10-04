//! # `panels::comments::shown` — *Show on screen* on a hidden comment's row
//!
//! A mark taken off screen cannot be clicked on the canvas, so the Comments
//! list, which still lists it, is the way back. Raises the same
//! `AnnotAction::SetFlag` the Properties panel's switch does.

use pdfcer_gui_base::annotflagswitch::FlagSwitch;

use super::RowSink;
use super::model::CommentRow;
use crate::app::actions::annot::AnnotAction;
use crate::text::panels::annotflags as t;

/// The control's rect, for `ui-verify`.
// ui-text-exempt: trace region name, never displayed
pub const REGION: &str = "comments.show_again";

/// The control, on a hidden row in a mode that authors markup.
pub(super) fn control(ui: &mut egui::Ui, comment: &CommentRow, sink: &mut RowSink<'_>) {
    let Some(id) = comment
        .id
        .filter(|_| comment.suppressed && sink.deletable_stance)
    else {
        return;
    };
    let button = ui.button(t::show_again());
    *sink.writing_controls_drawn += 1;
    crate::diag::ui_rect_visible(REGION, button.rect, ui.clip_rect());
    if button.clicked() {
        *sink.verb = Some(AnnotAction::SetFlag {
            page: comment.page_index,
            id,
            switch: FlagSwitch::OnScreen,
            on: true,
        });
    }
}
