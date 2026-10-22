//! # `dialogs::trust_sites` — Security ▸ Trust ▸ Web sites…
//!
//! Acrobat's Trust Manager ▸ Internet Access as a window: the web-link mode
//! (ask, allow all, block all) and every remembered site, each Allow or Block
//! and removable. OK raises [`OutwardAction::SetWebTrust`] with the edited
//! copy; Cancel, Escape and the close button change nothing.
//!
//! Design and rationale: `DESIGNS.md` § O320.

use pdfcer_gui_base::outwardaction::OutwardAction;
use pdfcer_gui_base::prefs::trust::{SiteAnswer, TrustPrefs, WebLinks};

use crate::app::actions::Action;
use crate::text::outward as t;

/// The window body's published region, for `ui-verify`.
const REGION_BODY: &str = "trust_sites.body";
/// The prefix of each site row's Forget button region; the row index follows.
pub const REGION_FORGET: &str = "trust_sites.forget";

/// The editor, holding its own copy until OK.
pub struct TrustSitesDialog {
    mode: WebLinks,
    sites: Vec<SiteAnswer>,
    answer: Option<bool>,
}

impl TrustSitesDialog {
    /// The editor over `trust`'s current mode and sites.
    #[must_use]
    pub fn new(trust: &TrustPrefs) -> Self {
        Self {
            mode: trust.web_links,
            sites: trust.sites().to_vec(),
            answer: None,
        }
    }

    /// Draw it. Returns `false` when it should close.
    pub fn show(&mut self, ctx: &egui::Context, actions: &mut Vec<Action>) -> bool {
        let (frame, ()) = pdfcer_gui_base::dialoghost::Host::new(
            "trust_sites", // ui-text-exempt: a viewport key, never displayed.
            t::sites_title(),
            egui::vec2(480.0, 360.0),
            egui::vec2(380.0, 260.0),
        )
        .show(ctx, |ui| {
            crate::diag::ui_rect(REGION_BODY, ui.max_rect());
            self.body(ui);
        });
        let accepted = match self.answer {
            Some(accepted) => accepted,
            None if frame.closed => false,
            None => return true,
        };
        if accepted {
            actions.push(self.accepted());
        }
        false
    }

    /// The action OK raises.
    fn accepted(&self) -> Action {
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed.
            format!(
                "trust-sites-set mode={} sites={}",
                mode_word(self.mode),
                self.sites.len()
            )
        });
        Action::Outward(OutwardAction::SetWebTrust {
            mode: self.mode,
            sites: self.sites.clone(),
        })
    }

    fn body(&mut self, ui: &mut egui::Ui) {
        ui.radio_value(&mut self.mode, WebLinks::Ask, t::sites_mode_ask());
        ui.radio_value(&mut self.mode, WebLinks::Allow, t::sites_mode_allow());
        ui.radio_value(&mut self.mode, WebLinks::Block, t::sites_mode_block());
        ui.separator();
        ui.label(t::sites_heading());
        if self.mode != WebLinks::Ask {
            ui.label(t::sites_unused());
        }
        self.rows(ui);
        ui.separator();
        let (ok, cancel, _) = pdfcer_gui_base::dialoghost::Host::footer(
            ui,
            (t::sites_ok(), ""),
            (t::sites_cancel(), ""),
            None,
        );
        if ok {
            self.answer = Some(true);
        } else if cancel {
            self.answer = Some(false);
        }
    }

    /// One row per remembered site: the host, Allow or Block, and Forget.
    fn rows(&mut self, ui: &mut egui::Ui) {
        if self.sites.is_empty() {
            ui.label(t::sites_none());
            return;
        }
        let mut forget = None;
        egui::ScrollArea::vertical()
            .max_height(180.0)
            .show(ui, |ui| {
                for (index, site) in self.sites.iter_mut().enumerate() {
                    ui.horizontal(|ui| {
                        ui.label(&site.host);
                        ui.radio_value(&mut site.allow, true, t::sites_allow());
                        ui.radio_value(&mut site.allow, false, t::sites_block());
                        let button = ui.button(t::sites_forget());
                        crate::diag::ui_rect(&format!("{REGION_FORGET}.{index}"), button.rect);
                        if button.clicked() {
                            forget = Some(index);
                        }
                    });
                }
            });
        if let Some(index) = forget {
            self.sites.remove(index);
        }
    }
}

/// The trace's word for a mode.
const fn mode_word(mode: WebLinks) -> &'static str {
    match mode {
        WebLinks::Ask => "ask",     // ui-text-exempt: trace token
        WebLinks::Allow => "allow", // ui-text-exempt: trace token
        WebLinks::Block => "block", // ui-text-exempt: trace token
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// OK carries the edited copy, not the preferences it started from.
    #[test]
    fn ok_carries_the_edited_copy() {
        let mut trust = TrustPrefs::default();
        trust.remember_site("a.example", true);
        trust.remember_site("b.example", false);
        let mut dialog = TrustSitesDialog::new(&trust);
        dialog.sites.remove(0);
        dialog.mode = WebLinks::Block;
        assert_eq!(
            dialog.accepted(),
            Action::Outward(OutwardAction::SetWebTrust {
                mode: WebLinks::Block,
                sites: vec![SiteAnswer {
                    host: "b.example".to_owned(),
                    allow: false,
                }],
            })
        );
    }

    /// The words `ui-verify` matches on; `mode_word` is an exhaustive match.
    #[test]
    fn the_modes_trace_the_words_the_harness_reads() {
        let words = [WebLinks::Ask, WebLinks::Allow, WebLinks::Block].map(mode_word);
        assert_eq!(words, ["ask", "allow", "block"]);
    }
}
