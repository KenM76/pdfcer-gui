//! # `dialogs::link` — the Add-link window
//!
//! Opened when the operator has drawn the link's box on the canvas, so the
//! box is settled before any question is asked — the order Acrobat's Link
//! tool and Bluebeam's Hyperlink tool use. Says where the link goes and how
//! it looks, and raises `LinkAction::Add`.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/dialogs/link.md`.

use egui::Ui;
use pdfcer_core::page_tree::Rect;
use pdfcer_gui_base::linkaction::LinkAction;
use pdfcer_gui_base::linkform::{LinkForm, Regions};

use crate::app::actions::Action;
use crate::text::linkedit as t;

/// The region this dialog publishes for its body.
pub const REGION_BODY: &str = "dialog:add-link"; // ui-text-exempt: trace region name, never displayed
/// The Add button.
pub const REGION_ADD: &str = "add-link.add"; // ui-text-exempt: trace region name, never displayed

/// The regions the form publishes inside this window.
pub const REGIONS: Regions = Regions {
    page: "add-link.page",     // ui-text-exempt: trace region name, never displayed
    web: "add-link.web",       // ui-text-exempt: trace region name, never displayed
    named: "add-link.named",   // ui-text-exempt: trace region name, never displayed
    field: "add-link.field",   // ui-text-exempt: trace region name, never displayed
    border: "add-link.border", // ui-text-exempt: trace region name, never displayed
    colour: "add-link.colour", // ui-text-exempt: trace region name, never displayed
};

/// A click places a box this size, in points: a line of 12-point text
/// about six characters long.
pub const CLICK_SIZE_PT: (f64, f64) = (72.0, 18.0);

/// The Add-link window.
#[derive(Debug)]
pub struct LinkDialog {
    page_index: usize,
    page_count: usize,
    rect: Rect,
    form: LinkForm,
    add_requested: bool,
    close_requested: bool,
}

impl LinkDialog {
    /// The window for a box drawn on page `page_index` of `page_count`. A
    /// click's zero-area box becomes [`CLICK_SIZE_PT`] with its lower-left
    /// corner at the click.
    #[must_use]
    pub fn open(page_index: usize, page_count: usize, rect: Rect) -> Self {
        let rect = if rect.urx - rect.llx <= 0.0 || rect.ury - rect.lly <= 0.0 {
            Rect {
                llx: rect.llx,
                lly: rect.lly,
                urx: rect.llx + CLICK_SIZE_PT.0,
                ury: rect.lly + CLICK_SIZE_PT.1,
            }
        } else {
            rect
        };
        Self {
            page_index,
            page_count,
            rect,
            form: LinkForm::default(),
            add_requested: false,
            close_requested: false,
        }
    }

    /// Draw the window. Returns `false` once it is done.
    pub fn show(&mut self, ctx: &egui::Context, actions: &mut Vec<Action>) -> bool {
        let (frame, ()) = crate::dialogs::host::Host::new(
            "add-link", // ui-text-exempt: a viewport key, never displayed.
            t::window_title(),
            egui::vec2(420.0, 380.0),
            egui::vec2(340.0, 280.0),
        )
        .show(ctx, |ui| {
            crate::diag::ui_rect(REGION_BODY, ui.max_rect());
            self.body(ui);
        });
        let open = !frame.closed;
        if std::mem::take(&mut self.add_requested)
            && let Ok(target) = self.form.target(self.page_count)
        {
            let border = self.form.border();
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed
                format!(
                    "add-link-requested page={} llx={:.1} lly={:.1} urx={:.1} ury={:.1} \
                         target={target:?} border={}",
                    self.page_index,
                    self.rect.llx,
                    self.rect.lly,
                    self.rect.urx,
                    self.rect.ury,
                    border.is_some()
                )
            });
            actions.push(Action::Link(LinkAction::Add {
                page: self.page_index,
                rect: self.rect,
                target,
                border,
            }));
            return false;
        }
        open && !std::mem::take(&mut self.close_requested)
    }

    fn body(&mut self, ui: &mut Ui) {
        ui.label(
            egui::RichText::new(t::sits_on(self.page_index + 1))
                .small()
                .weak(),
        );
        ui.add_space(6.0);
        self.form.goes_ui(ui, self.page_count, REGIONS);
        ui.separator();
        self.form.border_ui(ui, "add-link-colour", REGIONS); // ui-text-exempt: an egui id salt, never displayed
        ui.separator();
        ui.horizontal(|ui| {
            // Add is ABSENT while the destination cannot be written, and
            // the sentence beside it says what to fix (R9).
            match self.form.target(self.page_count) {
                Err(reason) => {
                    ui.colored_label(ui.visuals().error_fg_color, reason);
                }
                Ok(_) => {
                    let response = ui.button(t::add_button());
                    crate::diag::ui_rect(REGION_ADD, response.rect);
                    if response.clicked() {
                        self.add_requested = true;
                    }
                }
            }
            if ui.button(t::cancel_button()).clicked() {
                self.close_requested = true;
            }
        });
    }
}

/// **The Add-link round trip**: requested by `edit.insert_link`, waiting on
/// the canvas for a box, then the window.
#[derive(Debug, Default)]
pub struct LinkSlot {
    /// `edit.insert_link` ran; the frame has not armed the canvas yet. The
    /// document's page count, for the window's page field.
    requested: Option<usize>,
    /// The canvas is armed for a box; the page count travels on.
    waiting: Option<usize>,
    /// The window, once the box is drawn.
    dialog: Option<LinkDialog>,
}

impl LinkSlot {
    /// Ask for a box on a document of `page_count` pages.
    pub fn request(&mut self, page_count: usize) {
        self.requested = Some(page_count);
    }

    /// Read-and-clear the request, marking the slot as waiting for a box.
    pub fn take_request(&mut self) -> bool {
        let Some(count) = self.requested.take() else {
            return false;
        };
        self.waiting = Some(count);
        true
    }

    /// Whether a box is still wanted.
    #[must_use]
    pub const fn waiting(&self) -> bool {
        self.waiting.is_some()
    }

    /// The canvas placement was cancelled.
    pub fn cancelled(&mut self) {
        self.waiting = None;
    }

    /// The box was drawn on page `page_index`: open the window.
    pub fn deliver(&mut self, page_index: usize, rect: Rect) {
        if let Some(count) = self.waiting.take() {
            self.dialog = Some(LinkDialog::open(page_index, count, rect));
        }
    }

    /// Draw the window, if open.
    pub fn show(&mut self, ctx: &egui::Context, actions: &mut Vec<Action>) {
        if self.dialog.as_mut().map(|d| d.show(ctx, actions)) == Some(false) {
            self.dialog = None;
        }
    }

    /// Forget everything: the document closed.
    pub fn reset(&mut self) {
        *self = Self::default();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_click_becomes_a_standard_box_at_its_corner() {
        let d = LinkDialog::open(
            0,
            1,
            Rect {
                llx: 100.0,
                lly: 200.0,
                urx: 100.0,
                ury: 200.0,
            },
        );
        assert_eq!(
            (d.rect.llx, d.rect.lly, d.rect.urx, d.rect.ury),
            (100.0, 200.0, 172.0, 218.0)
        );
    }

    #[test]
    fn the_slot_opens_the_window_only_for_a_box_it_asked_for() {
        let mut slot = LinkSlot::default();
        let r = Rect {
            llx: 10.0,
            lly: 20.0,
            urx: 50.0,
            ury: 40.0,
        };
        slot.deliver(0, r);
        assert!(slot.dialog.is_none(), "nothing was asked for");
        slot.request(3);
        assert!(slot.take_request() && slot.waiting());
        assert!(!slot.take_request(), "the request is read-and-clear");
        slot.deliver(1, r);
        assert!(!slot.waiting());
        let d = slot.dialog.as_ref().expect("the window opens");
        assert_eq!((d.page_index, d.page_count), (1, 3));
    }

    #[test]
    fn a_dragged_box_is_kept() {
        let r = Rect {
            llx: 10.0,
            lly: 20.0,
            urx: 50.0,
            ury: 40.0,
        };
        assert_eq!(LinkDialog::open(0, 1, r).rect, r);
    }
}
