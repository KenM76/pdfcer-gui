//! # `app::actions::outward` — a link that leaves the document
//!
//! Decides, from Security ▸ Trust and the operator's remembered answers,
//! whether a clicked web link opens, is refused with a sentence saying where
//! to change that, or raises the Security Warning; whether a `/Launch` file
//! opens or raises the Launch File prompt; and applies either prompt's answer.
//! A `/GoToR` to a PDF opens it here at its place without asking, because
//! opening a PDF in this viewer runs nothing. Nothing is refused that the
//! operator did not refuse.
//!
//! Design and rationale: `DESIGNS.md` § O320.

use std::path::{Path, PathBuf};

use pdfcer_core::object::{Dict, Name, Object};
use pdfcer_core::outline::{Destination, DestinationReader};
use pdfcer_gui_base::outward;
use pdfcer_gui_base::outwardaction::{OutwardAction, RemoteGoTo};
use pdfcer_gui_base::prefs::trust::WebDecision;

use crate::app::PdfcerApp;
use crate::app::actions::Action;
use crate::app::state::Status;
use crate::text::outward as t;

impl PdfcerApp {
    /// Act on a link leaving the document, or on a prompt's answer.
    pub(super) fn apply_outward(&mut self, action: OutwardAction, pixels_per_point: f32) {
        match action {
            OutwardAction::WebLink { uri } => self.web_link(uri),
            OutwardAction::WebAnswer {
                uri,
                allow,
                remember,
            } => self.web_answer(&uri, allow, remember),
            OutwardAction::FileLink { file, target } => {
                self.file_link(&file, &target, pixels_per_point);
            }
            OutwardAction::Launch { file } => self.launch(file, pixels_per_point),
            OutwardAction::LaunchAnswer {
                file,
                open,
                dont_ask,
            } => self.launch_answer(&file, open, dont_ask, pixels_per_point),
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

    /// `/GoToR`: a PDF opens here at its place; any other file is a launch.
    fn file_link(&mut self, file: &Path, target: &RemoteGoTo, pixels_per_point: f32) {
        if !self.file_exists(file) {
            return;
        }
        if outward::is_pdf(file) {
            self.open_here(file, target, pixels_per_point);
        } else {
            self.launch(file.to_path_buf(), pixels_per_point);
        }
    }

    /// `/Launch`: ask first unless the operator turned asking off.
    fn launch(&mut self, file: PathBuf, pixels_per_point: f32) {
        if !self.file_exists(&file) {
            return;
        }
        let ask = self.prefs.trust.ask_before_files;
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed.
            format!(
                "launch-decision decision={} file={}",
                if ask { "ask" } else { "open" }, // ui-text-exempt: trace tokens
                file.display()
            )
        });
        if ask {
            self.dialogs.open_launch(file);
        } else {
            self.open_file(&file, pixels_per_point);
        }
    }

    fn launch_answer(&mut self, file: &Path, open: bool, dont_ask: bool, pixels_per_point: f32) {
        if !open {
            self.note(t::file_not_opened(&file.display().to_string()));
            return;
        }
        if dont_ask {
            self.prefs.trust.ask_before_files = false;
            // As `web_answer`: one decision, written now.
            let _ = self.prefs.save();
        }
        self.open_file(file, pixels_per_point);
    }

    /// Open a launched file: a PDF here, anything else in its Windows program.
    fn open_file(&mut self, file: &Path, pixels_per_point: f32) {
        if outward::is_pdf(file) {
            self.open_here(file, &RemoteGoTo::Start, pixels_per_point);
            return;
        }
        let shown = file.display().to_string();
        let said = match outward::open("file", &shown) {
            Ok(_) => t::file_opened(&shown),
            Err(why) => t::file_failed(&shown, &why),
        };
        self.note(said);
    }

    /// Open the PDF at `file` in a tab of its own, or show its tab, and arrive
    /// at `target`. A place the file does not have is said, and the file stays
    /// at its first page.
    fn open_here(&mut self, file: &Path, target: &RemoteGoTo, pixels_per_point: f32) {
        self.open_path(file.to_path_buf());
        let Status::Open(doc) = &self.status else {
            // The tab says why it did not open.
            return;
        };
        if doc.path != file {
            return;
        }
        let mut steps = Vec::new();
        let arrived = match target {
            RemoteGoTo::Start => Some(0),
            RemoteGoTo::Page(index) => (*index < doc.pages.len()).then(|| {
                steps.push(Action::GoToPage(*index));
                *index
            }),
            RemoteGoTo::Named(name) => named_page(doc, name).map(|(index, view)| {
                crate::app::actions::destination::actions_for(index, &view, &mut steps);
                index
            }),
        };
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed.
            format!(
                "filelink-open page={} path={}",
                arrived.map_or_else(|| "missing".to_owned(), |p| p.to_string()), // ui-text-exempt: trace token
                file.display(),
            )
        });
        if arrived.is_none() {
            let place = match target {
                RemoteGoTo::Page(index) => t::place_page(index.saturating_add(1)),
                RemoteGoTo::Named(name) => t::place_named(&String::from_utf8_lossy(name)),
                RemoteGoTo::Start => String::new(),
            };
            self.note(t::file_place_missing(&file.display().to_string(), &place));
        }
        self.apply_actions(steps, pixels_per_point);
    }

    /// Whether `file` is there; when it is not, say so.
    fn file_exists(&self, file: &Path) -> bool {
        let there = file.is_file();
        if !there {
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed.
                format!("filelink-missing path={}", file.display())
            });
            self.note(t::file_missing(&file.display().to_string()));
        }
        there
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

/// The page and view `name` names in `doc`'s own destinations.
///
/// Resolved by handing the engine's `DestinationReader::destination` a carrier
/// holding only `/Dest (name)`, the one reader entry point that resolves a
/// single name.
fn named_page(
    doc: &crate::app::state::OpenDoc,
    name: &[u8],
) -> Option<(usize, pdfcer_core::outline::DestView)> {
    let view = doc.session.view();
    let reader = DestinationReader::new(&view);
    let mut carrier = Dict::new();
    carrier.insert(Name::from(b"Dest"), Object::String(name.to_vec()));
    match reader.destination(&view, &carrier)? {
        Destination::Page { page_index, view } => Some((page_index, view)),
        _ => None,
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
