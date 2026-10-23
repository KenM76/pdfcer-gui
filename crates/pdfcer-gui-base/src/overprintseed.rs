//! # `overprintseed` — the document reading the overprint default waits on
//!
//! Settings ▸ Display ▸ *Use overprint preview* decides
//! `view.overprint_preview` as a document opens
//! ([`crate::prefs::overprint`]). *Only for PDF/X files* reads the file's
//! claim, which is cheap and is read at once. *Automatic* asks whether
//! anything drawn selects an overprinting graphics state
//! (`pdfcer_core::prepress::overprint_use`), and a "no" parses every content
//! stream, so that walk runs on a thread.
//!
//! The thread parses its own copy of the bytes the tab parsed, with the
//! tab's password and [`LoadOptions`], never [`crate::opendoc::OpenDoc::session`]
//! itself: a thread holding the session's `Arc` makes every edit's
//! `Arc::get_mut` fail until it finishes. The bytes, not the path, because
//! a tab can be read from a file other than the one it is bound to.
//!
//! Until the answer arrives, and when the walk cannot reach every drawn
//! stream, the answer is unknown and *Automatic* shows the preview, which is
//! the page as it prints. An unknown that stays unknown is said on the
//! status line.
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/overprintseed.md`.

use std::sync::mpsc::{Receiver, TryRecvError};

use pdfcer_core::document::{Document, LoadOptions};
use pdfcer_core::prepress::{OverprintUse, overprint_use, pdfx_claim, pdfx_output_intents};
use pdfcer_core::view::DocumentView;

use crate::prefs::overprint::PrintClaims;
use crate::secret::Secret;

/// What the overprint walk found.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Walked {
    /// Whether anything drawn asks for overprint; `None` when the walk could
    /// not reach every drawn stream and found no request.
    pub asks: Option<bool>,
    /// Why the walk is incomplete, one entry per stopping point, empty when
    /// it is complete.
    pub unwalked: Vec<String>,
}

impl Walked {
    /// The engine's answer, with an incomplete "no" read as unknown.
    #[must_use]
    pub fn of(found: &OverprintUse) -> Self {
        Self {
            asks: answer(found.asks(), found.is_complete()),
            unwalked: found.unwalked.clone(),
        }
    }

    /// No walk happened: the file could not be read again, or the thread
    /// could not start or ended without an answer.
    #[must_use]
    pub fn unread(why: String) -> Self {
        Self {
            asks: None,
            unwalked: vec![why],
        }
    }
}

/// A request found is yes whether or not the walk finished; a "no" is only
/// a no when it did.
const fn answer(asks: bool, complete: bool) -> Option<bool> {
    if asks {
        Some(true)
    } else if complete {
        Some(false)
    } else {
        None
    }
}

/// Where a document's overprint default stands.
#[derive(Debug, Default)]
pub enum OverprintSeed {
    /// Decided: nothing is waiting and nothing is owed.
    #[default]
    Settled,
    /// The walk is running. `shown` is what the preview was set to while it
    /// does, so an operator's own switch is not overridden by a late answer.
    Walking {
        /// The thread's one answer.
        answer: Receiver<Walked>,
        /// The preview as seeded.
        shown: bool,
    },
    /// The walk ended without an answer; the preview was left on.
    Unknown {
        /// What stopped it.
        unwalked: Vec<String>,
    },
}

/// The file claims PDF/X: a registered or unregistered `GTS_PDFXVersion` in
/// its XMP or Info dictionary, or a `GTS_PDFX` output intent. An intent
/// alone counts because every PDF/X part requires one and only a file made
/// for a press carries one.
#[must_use]
pub fn claims_pdfx(view: &DocumentView<'_>) -> bool {
    pdfx_claim(view).is_some() || !pdfx_output_intents(view).is_empty()
}

/// The claims a document makes, with the overprint answer `asks`.
#[must_use]
pub fn claims(view: &DocumentView<'_>, asks: Option<bool>) -> PrintClaims {
    PrintClaims {
        pdfx: claims_pdfx(view),
        overprint: asks,
    }
}

/// Walk `view` here, on the calling thread. For a document with no file
/// behind it, which is small by construction.
#[must_use]
pub fn walk_here(view: &DocumentView<'_>) -> Walked {
    Walked::of(&overprint_use(view))
}

/// Start the walk over `bytes`, parsed as the tab parsed them. The answer
/// arrives once on the returned receiver.
///
/// # Errors
///
/// The thread could not be started.
pub fn walk_bytes(
    bytes: Vec<u8>,
    password: Option<Secret>,
    options: LoadOptions,
) -> std::io::Result<Receiver<Walked>> {
    let (tx, rx) = std::sync::mpsc::channel();
    std::thread::Builder::new()
        .name("overprint-walk".into())
        .spawn(move || {
            let walked = match Document::from_bytes_with_options(
                bytes,
                password.as_ref().map(Secret::expose),
                options,
            ) {
                Ok(doc) => walk_here(&doc.view()),
                Err(why) => Walked::unread(why.to_string()),
            };
            let _ = tx.send(walked);
        })?;
    Ok(rx)
}

/// The walk's answer, if it has arrived. A thread that ended without
/// sending reads as an unknown answer.
#[must_use]
pub fn take(answer: &Receiver<Walked>) -> Option<Walked> {
    match answer.try_recv() {
        Ok(walked) => Some(walked),
        Err(TryRecvError::Empty) => None,
        Err(TryRecvError::Disconnected) => Some(Walked::unread(
            "the reading thread stopped without an answer".to_owned(), // ui-text-exempt: an engine-style reason, shown inside a translated sentence
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_incomplete_no_is_unknown_and_a_found_request_is_yes() {
        assert_eq!(answer(false, true), Some(false));
        assert_eq!(answer(false, false), None);
        assert_eq!(answer(true, false), Some(true));
        assert_eq!(answer(true, true), Some(true));
    }
}
