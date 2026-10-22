//! `app::dispatch::trust` — Security ▸ Trust's commands
//!
//! Two switches flipped and saved at once, and the Web sites… window. All three
//! are preferences, so none needs a document open.
//!
//! Design and rationale: `DESIGNS.md` § O320.

use crate::app::PdfcerApp;

const WEB_SITES: &str = "file.trust_web_sites";
const ASK_FILES: &str = "file.trust_ask_files";
const PLAY_3D: &str = "file.trust_play_3d";

/// Whether `id` is one of Security ▸ Trust's commands.
#[must_use]
pub(crate) fn claims(id: &str) -> bool {
    matches!(id, WEB_SITES | ASK_FILES | PLAY_3D)
}

impl PdfcerApp {
    /// Route one Security ▸ Trust command.
    pub(in crate::app) fn dispatch_trust(&mut self, id: &str) {
        let on = match id {
            WEB_SITES => {
                self.dialogs.open_trust_sites(&self.prefs.trust);
                return;
            }
            ASK_FILES => {
                self.prefs.trust.ask_before_files = !self.prefs.trust.ask_before_files;
                self.prefs.trust.ask_before_files
            }
            PLAY_3D => {
                self.prefs.trust.play_3d = !self.prefs.trust.play_3d;
                self.prefs.trust.play_3d
            }
            _ => return,
        };
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed.
            format!("trust-switch id={id} on={on}")
        });
        // One discrete decision, written now, as `view.smart_select` writes.
        let _ = self.prefs.save();
    }
}
