//! The viewer's side of closing with its document: which document it belongs
//! to, the question asked before that document or the program closes, the
//! writes it refuses while another document is on screen, and its window's
//! real owner traced for `ui-verify`.

use super::{ModelView, PictureFor};
use crate::app::actions::Action;
use crate::dialogs::close_model::{CloseModelAsk, Closing};
use crate::text::model3dclose as t;

impl ModelView {
    /// The document it was opened from: its path and serial as last seen.
    pub(crate) fn owner(&self) -> (&std::path::Path, u64) {
        (&self.owner.0, self.owner.1)
    }

    /// Whether the document with this path and serial is the one it belongs to.
    pub(crate) fn owned_by(&self, path: &std::path::Path, serial: u64) -> bool {
        self.owner.0 == path || self.owner.1 == serial
    }

    /// Follow its document to a new path or serial.
    pub(crate) fn rebind(&mut self, path: std::path::PathBuf, serial: u64) {
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed
            format!("model-view-owner-moved path={path:?} serial={serial}")
        });
        self.owner = (path, serial);
    }

    /// Ask before `closing` goes ahead, replacing any question already up.
    pub(crate) fn ask_close(&mut self, closing: Closing) {
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed
            format!(
                "model-close-asked reason={} owner={:?}",
                closing.reason(),
                self.owner.0
            )
        });
        self.answered = None;
        self.asking = Some(CloseModelAsk::new(closing, &self.owner.0));
    }

    /// Whether the question is up.
    pub(crate) const fn asking(&self) -> bool {
        self.asking.is_some()
    }

    /// The question's answer once given: what was closing, and `true` to
    /// close.
    pub(crate) fn take_close_answer(&mut self) -> Option<(Closing, bool)> {
        self.answered.take()
    }

    /// **Close the viewer because its document, or the program, is closing**:
    /// its settings kept now rather than after their usual pause.
    pub(crate) fn shut(&mut self, ctx: &egui::Context, actions: &mut Vec<Action>) {
        self.keep_settings(ctx, true, actions);
        // ui-text-exempt: diagnostic trace, never displayed
        crate::diag::trace(|| "model-view-closed how=document".to_owned());
    }

    /// Draw the question while it is up and hold its answer.
    pub(super) fn show_close_ask(&mut self, ctx: &egui::Context) {
        let Some(ask) = self.asking.as_mut() else {
            return;
        };
        if let Some(close) = ask.show(ctx) {
            let closing = ask.closing().clone();
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed
                format!(
                    "model-close-answered reason={} choice={}",
                    closing.reason(),
                    if close { "close" } else { "cancel" }
                )
            });
            self.asking = None;
            self.answered = Some((closing, close));
        }
    }

    /// Drop this frame's writes into the document, which is not the one on
    /// screen, and say why. A picture saved to a file of its own still goes.
    pub(super) fn refuse_writes(&mut self) {
        let page = matches!(self.picture, Some((PictureFor::Page, _)));
        if page {
            self.picture = None;
        }
        let views = self.views.take().is_some();
        let measured = self.measure.writing.take().is_some();
        if page || views || measured {
            self.failed = Some(t::other_document().to_owned());
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed
                format!(
                    "model-view-write-refused reason=other-document picture={page} views={views} measurements={measured}"
                )
            });
        }
    }

    /// The window's owner as the operating system reports it, traced when
    /// it changes: `owned=0` is a window of its own.
    pub(super) fn trace_owner(&self) {
        if !self.native {
            return;
        }
        if let Some(owner) = native_window::owner_of(&self.title) {
            crate::diag::trace_on_change("model-view-owner", || {
                // ui-text-exempt: diagnostic trace, never displayed
                format!("owned={}", u8::from(owner != 0))
            });
        }
    }
}
