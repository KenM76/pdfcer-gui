//! Properties comes to the front of its stack when it has something new to
//! show — a tool with options armed, or an object picked on the page — and
//! gives the front back to the tab it covered once it has nothing. Design:
//! `DESIGNS.md` § O414.
//!
//! It never mounts the panel: a Properties the operator closed, or a mode
//! without one, stays as it is. A pick while the Objects tree is in front of
//! Properties' stack leaves the tree there, because the tree shows the pick.

use egui_shell::dock::PanelId;
use pdfcer_gui_base::selectionidentity::Selection;

use crate::app::PdfcerApp;
use crate::app::state::Status;
use crate::panels::Panel;
use crate::panels::properties::tool::{Slot, block_for, slot_of};

/// What Properties last had to show, and what it covered.
#[derive(Default)]
pub struct Forward {
    /// Whether the armed tool had options last frame.
    tool: bool,
    /// The selection last frame.
    picked: Vec<Selection>,
    /// The tab Properties was brought in front of, and whether its side was
    /// shown before.
    covered: Option<(PanelId, bool)>,
}

impl PdfcerApp {
    /// Bring Properties forward or give the front back, once per frame
    /// after the frame's actions apply.
    pub(super) fn properties_forward(&mut self, ctx: &egui::Context) {
        let armed = crate::canvas::tool::selected(ctx);
        let tool = block_for(armed).is_some_and(|b| slot_of(b) == Slot::AboveTheSelection);
        let picked = match &self.status {
            Status::Open(doc) => doc.selection.entries().to_vec(),
            _ => Vec::new(),
        };
        let armed_now = tool && !self.forward.tool;
        let picked_now = !picked.is_empty() && picked != self.forward.picked;
        let nothing = !tool && picked.is_empty();
        self.forward.tool = tool;
        self.forward.picked = picked;
        if armed_now || picked_now {
            self.bring_forward(armed_now);
        } else if nothing {
            self.give_back();
        }
    }

    /// Properties to the front of its stack, remembering what it covers;
    /// `tool` when an armed tool asks, which the Objects tree does not stop.
    fn bring_forward(&mut self, tool: bool) {
        let id = PanelId::new(Panel::Properties.command_id());
        let layout = self.dock.layout();
        let Some(at) = layout.find(&id) else {
            return;
        };
        if layout.is_on_screen(&id) {
            return;
        }
        let side = layout.side(at.side);
        let front = side.columns[at.column].stacks[at.stack]
            .active_panel()
            .cloned();
        let objects = PanelId::new(Panel::Objects.command_id());
        if !tool && front.as_ref() == Some(&objects) && side.visible {
            return;
        }
        let shown = side.visible;
        if self.forward.covered.is_none() {
            self.forward.covered = front.map(|f| (f, shown));
        }
        self.dock.activate(&id);
        self.modes
            .record_layout(self.dock.layout(), &mut self.layout);
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed
            format!("properties-forward tool={tool} side={}", at.side.key())
        });
    }

    /// The covered tab back to the front, while Properties is still the one
    /// in front of it; the side hidden again if it was.
    fn give_back(&mut self) {
        let Some((front, shown)) = self.forward.covered.take() else {
            return;
        };
        let id = PanelId::new(Panel::Properties.command_id());
        let layout = self.dock.layout();
        if !layout.is_active(&id) || !layout.contains(&front) {
            return;
        }
        self.dock.activate(&front);
        if let Some(at) = self.dock.layout().find(&front) {
            self.dock.layout_mut().side_mut(at.side).visible = shown;
        }
        self.modes
            .record_layout(self.dock.layout(), &mut self.layout);
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed
            format!("properties-back to={}", front.as_str())
        });
    }
}
