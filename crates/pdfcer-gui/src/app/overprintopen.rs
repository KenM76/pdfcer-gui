//! # `app::overprintopen` — the overprint default, applied as a document opens
//!
//! [`seed`] decides `view.overprint_preview` from Settings ▸ Display ▸ *Use
//! overprint preview* when a document is adopted; *Automatic*'s walk then
//! runs on a thread and [`PdfcerApp::overprint_seed_poll`] applies its
//! answer when it arrives. The readings are
//! [`pdfcer_gui_base::overprintseed`]'s.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/app/overprintopen.md`.

use std::time::Duration;

use pdfcer_gui_base::overprintseed::{self as seed, OverprintSeed, Walked};
use pdfcer_gui_base::prefs::overprint::PrintClaims;

use crate::app::prefs::OverprintDefault;
use crate::app::state::OpenDoc;
use crate::app::{PdfcerApp, Status};

/// How soon a frame looks again for the walk's answer while it runs.
const POLL: Duration = Duration::from_millis(100);

/// Decide `doc`'s overprint preview under `choice`, starting *Automatic*'s
/// walk when the document has a file behind it.
pub(super) fn seed(doc: &mut OpenDoc, choice: OverprintDefault) {
    let claims = match choice {
        OverprintDefault::PdfxOnly => PrintClaims {
            pdfx: seed::claims_pdfx(&doc.session.view()),
            overprint: None,
        },
        OverprintDefault::Automatic => PrintClaims {
            pdfx: false,
            overprint: start_walk(doc),
        },
        OverprintDefault::Always | OverprintDefault::Never => PrintClaims::default(),
    };
    let on = choice.resolve(claims);
    doc.view.overprint_preview = on;
    if let OverprintSeed::Walking { shown, .. } = &mut doc.overprint_seed {
        *shown = on;
    }
    crate::diag::trace(|| {
        format!(
            // ui-text-exempt: diagnostic trace, never displayed in the UI
            "overprint-default choice={} pdfx={} asks={} on={}",
            choice.id(),
            u8::from(claims.pdfx),
            asks_token(choice, claims.overprint),
            u8::from(on)
        )
    });
}

/// `0`/`1` for a known answer, `?` for one the walk has not given, `-` when
/// the choice does not read the document.
fn asks_token(choice: OverprintDefault, asks: Option<bool>) -> &'static str {
    match (choice, asks) {
        (OverprintDefault::Automatic, Some(true)) => "1",
        (OverprintDefault::Automatic, Some(false)) => "0",
        (OverprintDefault::Automatic, None) => "?",
        _ => "-",
    }
}

/// Start *Automatic*'s walk. A document made here, with no file behind it,
/// is walked at once; otherwise the answer is `None` until the thread's
/// arrives.
fn start_walk(doc: &mut OpenDoc) -> Option<bool> {
    if doc.stored_under().is_none() {
        let walked = seed::walk_here(&doc.session.view());
        return settle(doc, walked);
    }
    let bytes = doc.session.document().bytes().to_vec();
    match seed::walk_bytes(bytes, doc.opened_with.clone(), doc.load_options) {
        Ok(answer) => {
            doc.overprint_seed = OverprintSeed::Walking {
                answer,
                shown: true,
            };
            None
        }
        Err(why) => settle(doc, Walked::unread(why.to_string())),
    }
}

/// Record a finished walk on `doc` and return its answer.
fn settle(doc: &mut OpenDoc, walked: Walked) -> Option<bool> {
    doc.overprint_seed = match walked.asks {
        Some(_) => OverprintSeed::Settled,
        None => OverprintSeed::Unknown {
            unwalked: walked.unwalked,
        },
    };
    walked.asks
}

impl PdfcerApp {
    /// Apply *Automatic*'s answer to the document on screen when it has
    /// arrived. The preview is left alone if the operator has switched it
    /// since it was seeded.
    pub(super) fn overprint_seed_poll(&mut self, ctx: &egui::Context) {
        let Status::Open(doc) = &mut self.status else {
            return;
        };
        let OverprintSeed::Walking { answer, shown } = &doc.overprint_seed else {
            return;
        };
        let shown = *shown;
        let Some(walked) = seed::take(answer) else {
            ctx.request_repaint_after(POLL);
            return;
        };
        let unwalked = walked.unwalked.len();
        let asks = settle(doc, walked);
        let switched = doc.view.overprint_preview != shown;
        if !switched {
            doc.view.overprint_preview = asks.unwrap_or(true);
        }
        let on = doc.view.overprint_preview;
        crate::diag::trace(|| {
            format!(
                // ui-text-exempt: diagnostic trace, never displayed in the UI
                "overprint-default choice=automatic pdfx=0 asks={} on={} unwalked={unwalked} \
                 switched={}",
                asks_token(OverprintDefault::Automatic, asks),
                u8::from(on),
                u8::from(switched)
            )
        });
    }
}
