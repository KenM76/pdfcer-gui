//! # `app::actions::deskew` — one turn of File ▸ Straighten scans or Enlarge scans
//!
//! Contract: a [`DeskewStep::Page`] does its [`ScanJob`] to one picture (the
//! page's scan, or a selected image) through the edit funnel, re-rendering
//! that page only. Straighten measures first and turns only a tilt that is
//! trusted and large enough; Enlarge resamples at the run's factor. Every
//! turn logs one [`Logged`] under its run id; `dialogs::deskew` reads the log
//! with [`outcomes`] and drops it with [`forget`]. [`DeskewStep::Finish`]
//! folds the run's edits into one undo entry.
//!
//! The measurement runs outside the funnel because a measured-but-left page
//! is not an edit: it must not bump the epoch or post a disclosure.
//! `EditSession::detect_image_skew` is `&mut self` and the session is not
//! `Clone`, so a run cannot be measured off the UI thread; pacing it a page per
//! frame is the workaround, reported as engine request G171.

use std::cell::RefCell;
use std::sync::Arc;

use pdfcer_core::deskew::SkewEstimate;
use pdfcer_core::edit::{CommandKind, EditSession};
use pdfcer_core::ocr::upscale::Upscale;

use crate::app::state::OpenDoc;
use pdfcer_gui_base::deskewstep::{DeskewStep, ScanJob};

/// The detector's finest search step in `pdfcer_core::deskew`; a smaller
/// angle is inside its resolution.
pub const MIN_DEGREES: f64 = 0.05;

/// What one turn did.
#[derive(Debug, Clone, PartialEq)]
pub enum Outcome {
    /// The picture was turned.
    Straightened {
        /// The measured tilt, degrees counter-clockwise.
        degrees: f64,
        /// The engine's confidence, 0 to 1.
        confidence: f64,
        /// The picture's stream before and after, bytes.
        bytes: (usize, usize),
        /// The page also draws text, which did not turn with it.
        had_text: bool,
    },
    /// The picture was enlarged.
    Enlarged {
        /// Its size before and after, in samples.
        from: (u32, u32),
        to: (u32, u32),
        /// Its stream before and after, bytes.
        bytes: (usize, usize),
    },
    /// Measured with confidence, but under [`MIN_DEGREES`].
    Level { degrees: f64 },
    /// Measured under the run's least confidence.
    Unsure { degrees: f64, confidence: f64 },
    /// Too little ink to measure.
    Unmeasurable,
    /// The page draws no image.
    NoPicture,
    /// Skipped: the page has text and the run said to skip such pages.
    HasText,
    /// Another holder had the session.
    Busy,
    /// Another document was active when the turn arrived.
    Elsewhere,
    /// The engine declined; its sentence, verbatim.
    Refused(String),
}

/// One turn's entry in the log. `page` is 0-based.
#[derive(Debug, Clone, PartialEq)]
pub struct Logged {
    pub page: usize,
    pub outcome: Outcome,
}

/// What [`DeskewStep::Finish`] found.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Folded {
    /// Nothing or one page was changed; there was nothing to fold.
    Nothing,
    /// One undo entry now holds the run.
    One,
    /// The run is several undo entries.
    Unfolded,
}

thread_local! {
    static LOG: RefCell<Vec<(u64, Logged)>> = const { RefCell::new(Vec::new()) };
    static FOLDS: RefCell<Vec<(u64, Folded)>> = const { RefCell::new(Vec::new()) };
}

/// Every turn logged under `run`, in order.
pub fn outcomes(run: u64) -> Vec<Logged> {
    LOG.with(|l| {
        l.borrow()
            .iter()
            .filter(|(r, _)| *r == run)
            .map(|(_, e)| e.clone())
            .collect()
    })
}

/// What `run`'s [`DeskewStep::Finish`] found, once it has run.
pub fn folded(run: u64) -> Option<Folded> {
    FOLDS.with(|f| f.borrow().iter().find(|(r, _)| *r == run).map(|(_, x)| *x))
}

/// Drop `run`'s entries.
pub fn forget(run: u64) {
    LOG.with(|l| l.borrow_mut().retain(|(r, _)| *r != run));
    FOLDS.with(|f| f.borrow_mut().retain(|(r, _)| *r != run));
}

fn log(run: u64, page: usize, outcome: Outcome) {
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed
        format!(
            "deskew-page run={run} page={page} outcome={}",
            token(&outcome)
        )
    });
    LOG.with(|l| l.borrow_mut().push((run, Logged { page, outcome })));
}

/// The trace's word for an outcome, with its numbers.
fn token(outcome: &Outcome) -> String {
    // ui-text-exempt: diagnostic trace vocabulary, never displayed
    match outcome {
        Outcome::Straightened {
            degrees,
            confidence,
            bytes,
            had_text,
        } => format!(
            "straightened degrees={degrees:.3} confidence={confidence:.3} old={} new={} had-text={had_text}", // ui-text-exempt: diagnostic trace, never displayed
            bytes.0, bytes.1
        ),
        Outcome::Enlarged { from, to, bytes } => format!(
            "enlarged from={}x{} to={}x{} old={} new={}", // ui-text-exempt: diagnostic trace, never displayed
            from.0, from.1, to.0, to.1, bytes.0, bytes.1
        ),
        Outcome::Level { degrees } => format!("level degrees={degrees:.3}"), // ui-text-exempt: diagnostic trace, never displayed
        Outcome::Unsure {
            degrees,
            confidence,
        } => format!("unsure degrees={degrees:.3} confidence={confidence:.3}"), // ui-text-exempt: diagnostic trace, never displayed
        Outcome::Unmeasurable => "unmeasurable".to_owned(),
        Outcome::NoPicture => "no-picture".to_owned(),
        Outcome::HasText => "has-text".to_owned(),
        Outcome::Busy => "busy".to_owned(),
        Outcome::Elsewhere => "elsewhere".to_owned(),
        Outcome::Refused(_) => "refused".to_owned(),
    }
}

/// Route one turn. A turn for a document other than the active one does
/// nothing: the dialog pauses while its document is off screen, so this is
/// the guard behind that, not the mechanism.
pub(super) fn apply(doc: &mut OpenDoc, step: &DeskewStep) {
    match *step {
        DeskewStep::Page {
            run,
            doc: serial,
            page,
            object,
            job,
        } => {
            let outcome = if serial == doc.serial {
                turn(doc, page, object, job)
            } else {
                Outcome::Elsewhere
            };
            log(run, page, outcome);
        }
        DeskewStep::Finish { run, doc: serial } => {
            if serial == doc.serial {
                finish(doc, run);
            } else {
                FOLDS.with(|f| f.borrow_mut().push((run, Folded::Unfolded)));
            }
        }
    }
}

/// Do `job` to one picture.
fn turn(doc: &mut OpenDoc, page: usize, object: Option<usize>, job: ScanJob) -> Outcome {
    match job {
        ScanJob::Straighten {
            skip_text,
            min_confidence_percent,
        } => {
            let least = f64::from(min_confidence_percent) / 100.0;
            page_turn(doc, page, object, skip_text, least)
        }
        ScanJob::Enlarge {
            factor_halves,
            filter,
        } => match Upscale::new(f32::from(factor_halves) / 2.0, filter) {
            Ok(up) => enlarge(doc, page, object, up),
            Err(e) => Outcome::Refused(e.to_string()),
        },
    }
}

/// Measure, then straighten when the measurement says to.
fn page_turn(
    doc: &mut OpenDoc,
    page: usize,
    object: Option<usize>,
    skip_text: bool,
    least: f64,
) -> Outcome {
    let had_text = page_has_text(doc, page);
    if had_text && skip_text {
        return Outcome::HasText;
    }
    let (index, estimate) = match measure(doc, page, object) {
        Ok(found) => found,
        Err(outcome) => return outcome,
    };
    let degrees = estimate.angle_degrees;
    let confidence = estimate.confidence;
    if confidence < least {
        return Outcome::Unsure {
            degrees,
            confidence,
        };
    }
    if degrees.abs() < MIN_DEGREES {
        return Outcome::Level { degrees };
    }
    let mut done = None;
    let mut refusal = None;
    super::funnel::vector_edit_on_page(doc, "deskew", page, 1, |session| {
        match session.deskew_image(page, index, degrees) {
            Ok(d) => {
                let notes = d.notes.clone();
                done = Some(d);
                Ok(notes)
            }
            Err(e) => {
                refusal = Some(e.to_string());
                Err(e)
            }
        }
    });
    match (done, refusal) {
        (Some(d), _) => Outcome::Straightened {
            degrees,
            confidence,
            bytes: (d.old_stream_bytes, d.new_stream_bytes),
            had_text,
        },
        (None, Some(reason)) => Outcome::Refused(reason),
        (None, None) => Outcome::Busy,
    }
}

/// Enlarge the picture by `up`.
fn enlarge(doc: &mut OpenDoc, page: usize, object: Option<usize>, up: Upscale) -> Outcome {
    doc.render_worker.cancel_and_wait();
    let index = match Arc::get_mut(&mut doc.session) {
        Some(session) => match picture(session, page, object) {
            Ok(i) => i,
            Err(outcome) => return outcome,
        },
        None => return Outcome::Busy,
    };
    let mut done = None;
    let mut refusal = None;
    super::funnel::vector_edit_on_page(doc, "upscale", page, 1, |session| {
        match session.upscale_image(page, index, up) {
            Ok(u) => {
                let notes = u.notes.clone();
                done = Some(u);
                Ok(notes)
            }
            Err(e) => {
                refusal = Some(e.to_string());
                Err(e)
            }
        }
    });
    match (done, refusal) {
        (Some(u), _) => Outcome::Enlarged {
            from: u.old_size,
            to: u.new_size,
            bytes: (u.old_stream_bytes, u.new_stream_bytes),
        },
        (None, Some(reason)) => Outcome::Refused(reason),
        (None, None) => Outcome::Busy,
    }
}

/// The picture a turn works on: `object`, or the page's scan.
fn picture(
    session: &mut EditSession,
    page: usize,
    object: Option<usize>,
) -> Result<usize, Outcome> {
    match object {
        Some(i) => Ok(i),
        None => match session.page_scan_image(page) {
            Ok(Some(i)) => Ok(i),
            Ok(None) => Err(Outcome::NoPicture),
            Err(e) => Err(Outcome::Refused(e.to_string())),
        },
    }
}

/// The picture to measure and its skew, or the outcome that ends the turn.
fn measure(
    doc: &mut OpenDoc,
    page: usize,
    object: Option<usize>,
) -> Result<(usize, SkewEstimate), Outcome> {
    doc.render_worker.cancel_and_wait();
    let Some(session) = Arc::get_mut(&mut doc.session) else {
        return Err(Outcome::Busy);
    };
    let index = picture(session, page, object)?;
    match session.detect_image_skew(page, index) {
        Ok(Some(estimate)) => Ok((index, estimate)),
        Ok(None) => Err(Outcome::Unmeasurable),
        Err(e) => Err(Outcome::Refused(e.to_string())),
    }
}

/// Whether `page` draws any extractable text. An extraction failure is not
/// text: a page whose content will not parse is a scan's likeliest shape.
fn page_has_text(doc: &OpenDoc, page: usize) -> bool {
    use crate::app::settings::SettingsExt as _;
    let Some(tree_page) = doc.pages.get(page) else {
        return false;
    };
    let view = doc.session.view();
    pdfcer_core::text_extract::extract_page_view(
        &view,
        tree_page,
        page,
        &doc.settings.extract_options(),
    )
    .is_ok_and(|text| !text.plain_text().trim().is_empty())
}

/// The undo kind an outcome left on the stack, when it changed the page.
fn edit_kind(outcome: &Outcome) -> Option<CommandKind> {
    match outcome {
        Outcome::Straightened { .. } => Some(CommandKind::DeskewImage),
        Outcome::Enlarged { .. } => Some(CommandKind::UpscaleImage),
        _ => None,
    }
}

/// Fold the run's edits into one undo entry, when they are still the
/// newest entries on the stack. `coalesce_last` cannot tell whose commands
/// it folds, so the kinds are read first: an edit the operator made mid-run
/// sits among them and the fold is declined rather than swallowing it.
fn finish(doc: &mut OpenDoc, run: u64) {
    let kinds: Vec<CommandKind> = outcomes(run)
        .iter()
        .filter_map(|e| edit_kind(&e.outcome))
        .collect();
    let corrected = kinds.len();
    let folded = if corrected < 2 {
        Folded::Nothing
    } else {
        doc.render_worker.cancel_and_wait();
        if Arc::get_mut(&mut doc.session).is_some_and(|s| folds(s, corrected, kinds[0])) {
            Folded::One
        } else {
            Folded::Unfolded
        }
    };
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed
        let word = match folded {
            Folded::Nothing => "nothing",
            Folded::One => "one",
            Folded::Unfolded => "unfolded",
        };
        format!("deskew-finished run={run} corrected={corrected} folded={word}")
    });
    FOLDS.with(|f| f.borrow_mut().push((run, folded)));
}

/// Fold the newest `n` entries into one, only when every one is `kind`.
fn folds(session: &mut EditSession, n: usize, kind: CommandKind) -> bool {
    session.undo_depth() >= n
        && session.undo_kinds().take(n).all(|k| k == kind)
        && session.coalesce_last(n, kind)
}
