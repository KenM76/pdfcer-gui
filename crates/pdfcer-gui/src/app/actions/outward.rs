//! # `app::actions::outward` — a link that leaves the document
//!
//! Decides, from Security ▸ Trust and the operator's remembered answers,
//! whether a clicked web link opens, is refused with a sentence saying where
//! to change that, or raises the Security Warning; and applies the prompt's
//! answer. Nothing is refused that the operator did not refuse.
//!
//! Design and rationale: `DESIGNS.md` § O320.

use pdfcer_gui_base::outward;
use pdfcer_gui_base::outwardaction::OutwardAction;
use pdfcer_gui_base::prefs::trust::WebDecision;

use crate::app::PdfcerApp;
use crate::app::state::Status;
use crate::text::outward as t;

impl PdfcerApp {
    /// Act on a link leaving the document, or on the prompt's answer.
    pub(super) fn apply_outward(&mut self, action: OutwardAction) {
        match action {
            OutwardAction::WebLink { uri } => self.web_link(uri),
            OutwardAction::WebAnswer {
                uri,
                allow,
                remember,
            } => self.web_answer(&uri, allow, remember),
        }
    }

    fn web_link(&mut self, uri: String) {
        let site = outward::site(&uri);
        let decision = self.prefs.trust.web_decision(&site);
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed.
            format!(
                "weblink-decision decision={} site={site}",
                decision_word(decision)
            )
        });
        match decision {
            WebDecision::Open => self.open_web(&uri),
            WebDecision::Refuse => self.note(t::web_refused(&site)),
            WebDecision::Ask => self.dialogs.open_weblink(uri),
        }
    }

    fn web_answer(&mut self, uri: &str, allow: bool, remember: bool) {
        let site = outward::site(uri);
        if remember {
            self.prefs.trust.remember_site(&site, allow);
            // One discrete decision, written once, now; a failed write costs
            // the answer at restart and is not worth a modal.
            let _ = self.prefs.save();
        }
        if allow {
            self.open_web(uri);
        } else {
            self.note(t::web_blocked(&site));
        }
    }

    fn open_web(&mut self, uri: &str) {
        let said = match outward::open("web", uri) {
            Ok(_) => t::web_opened(uri),
            Err(why) => t::web_failed(uri, &why),
        };
        self.note(said);
    }

    /// One sentence on the status line, stamped with the active document's revision.
    fn note(&self, said: String) {
        let epoch = match &self.status {
            Status::Open(doc) => doc.edit_epoch,
            _ => 0,
        };
        crate::app::actions::record_note(epoch, said);
    }
}

/// The trace's word for a decision.
const fn decision_word(decision: WebDecision) -> &'static str {
    match decision {
        WebDecision::Open => "open",     // ui-text-exempt: trace token
        WebDecision::Refuse => "refuse", // ui-text-exempt: trace token
        WebDecision::Ask => "ask",       // ui-text-exempt: trace token
    }
}
