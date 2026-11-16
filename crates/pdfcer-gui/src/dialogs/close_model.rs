//! # `dialogs::close_model` — asking before a 3D window's document closes
//!
//! The 3D model viewer is a window of its own, beside the document it was
//! opened from. Closing that document's tab, or the program, asks first:
//! *Close both* or *Cancel* for a tab, *Close* or *Cancel* for the program.
//! No answer keeps the viewer open on its own, because it reads its saved
//! views from the document and writes into it. Closing the viewer itself
//! asks nothing and leaves the document open.
//!
//! Design: `docs/modules/pdfcer-gui/dialogs/close_model.md`.

use std::path::{Path, PathBuf};

/// The region the question's body publishes.
pub const REGION_BODY: &str = "dialog:close_model"; // ui-text-exempt: trace region name, never displayed

/// What is closing while the viewer is open.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Closing {
    /// The tab of the document at this path.
    Tab(PathBuf),
    /// The program.
    Quit,
}

impl Closing {
    /// The trace token for it.
    #[cfg_attr(not(feature = "3d"), allow(dead_code))]
    pub(crate) const fn reason(&self) -> &'static str {
        match self {
            Self::Tab(_) => "tab", // ui-text-exempt: trace token, never displayed
            Self::Quit => "quit",  // ui-text-exempt: trace token, never displayed
        }
    }
}

/// The question, while it is up.
#[cfg(feature = "3d")]
pub(crate) struct CloseModelAsk {
    closing: Closing,
    /// The sentence it asks, built once from the document's file name.
    question: String,
}

#[cfg(feature = "3d")]
impl CloseModelAsk {
    /// Ask about `closing`, for the viewer of the document at `owner`.
    pub(crate) fn new(closing: Closing, owner: &Path) -> Self {
        use crate::text::model3dclose as t;
        let question = match &closing {
            Closing::Tab(_) => t::question_tab(&owner.file_name().map_or_else(
                || t::untitled().to_owned(),
                |n| n.to_string_lossy().into_owned(),
            )),
            Closing::Quit => t::question_quit().to_owned(),
        };
        Self { closing, question }
    }

    /// What it asks about.
    pub(crate) const fn closing(&self) -> &Closing {
        &self.closing
    }

    /// Draw it. `Some(true)` once the operator chose to close, `Some(false)`
    /// on Cancel, Escape or the window's ✕.
    pub(crate) fn show(&self, ctx: &egui::Context) -> Option<bool> {
        use crate::text::model3dclose as t;
        let (accept, hover) = match self.closing {
            Closing::Tab(_) => (t::close_both(), t::close_both_hover()),
            Closing::Quit => (t::close(), t::close_hover()),
        };
        let (frame, (close, cancel)) = crate::dialogs::host::Host::new(
            "close-model", // ui-text-exempt: a viewport key, never displayed
            t::title(),
            egui::vec2(420.0, 170.0),
            egui::vec2(380.0, 150.0),
        )
        .show(ctx, |ui| {
            crate::diag::ui_rect(REGION_BODY, ui.max_rect());
            ui.label(&self.question);
            ui.add_space(4.0);
            ui.small(t::why_not_kept());
            ui.add_space(8.0);
            let (close, cancel, _) = crate::dialogs::host::Host::footer(
                ui,
                (accept, hover),
                (t::cancel(), t::cancel_hover()),
                None,
            );
            (close, cancel)
        });
        if close {
            Some(true)
        } else {
            (cancel || frame.closed).then_some(false)
        }
    }
}

impl super::DialogsState {
    /// The document the open 3D viewer belongs to: its path and serial.
    pub(crate) fn model_view_owner(&self) -> Option<(&Path, u64)> {
        #[cfg(feature = "3d")]
        return self.model3d.as_ref().map(super::model3d::ModelView::owner);
        #[cfg(not(feature = "3d"))]
        None
    }

    /// Follow the viewer's document to its new path or serial.
    pub(crate) fn rebind_model_view(&mut self, path: PathBuf, serial: u64) {
        #[cfg(feature = "3d")]
        if let Some(view) = self.model3d.as_mut() {
            view.rebind(path, serial);
        }
        #[cfg(not(feature = "3d"))]
        let _ = (path, serial);
    }

    /// Whether the close question is up.
    pub(crate) fn close_model_asking(&self) -> bool {
        #[cfg(feature = "3d")]
        return self
            .model3d
            .as_ref()
            .is_some_and(super::model3d::ModelView::asking);
        #[cfg(not(feature = "3d"))]
        false
    }

    /// Ask before `closing` goes ahead. `false` when no viewer is open, so
    /// there is nothing to ask.
    pub(crate) fn ask_close_model(&mut self, closing: Closing) -> bool {
        #[cfg(feature = "3d")]
        if let Some(view) = self.model3d.as_mut() {
            view.ask_close(closing);
            return true;
        }
        let _ = closing;
        false
    }

    /// The close question's answer, once given.
    pub(crate) fn take_close_model_answer(&mut self) -> Option<(Closing, bool)> {
        #[cfg(feature = "3d")]
        return self
            .model3d
            .as_mut()
            .and_then(super::model3d::ModelView::take_close_answer);
        #[cfg(not(feature = "3d"))]
        None
    }

    /// **Close the 3D viewer with its document**, returning the settings
    /// write it queues.
    pub(crate) fn close_model_view(
        &mut self,
        ctx: &egui::Context,
    ) -> Vec<crate::app::actions::Action> {
        #[cfg_attr(not(feature = "3d"), allow(unused_mut))]
        let mut actions = Vec::new();
        #[cfg(feature = "3d")]
        if let Some(mut view) = self.model3d.take() {
            view.shut(ctx, &mut actions);
        }
        let _ = ctx;
        actions
    }
}
