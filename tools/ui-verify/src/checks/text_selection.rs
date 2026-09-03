//! `text_selection_sweeps_and_copies` — the regression test for **a gesture
//! whose entire behaviour is a drag, on a feature whose entire feedback is a
//! translucent wash.**
//!
//! # The defect class this exists for
//!
//! `HANDOFF.md` §2's defect 8 is the sharpest one this project has recorded:
//!
//! > The grid was a tint rather than a grid: a one-point minor step, ~2,450
//! > lines a frame. **A screenshot could not catch this one** — 2,450 hairlines
//! > and a wash are the same picture. It was found by printing the ladder the
//! > running app had actually chosen.
//!
//! Text selection is that trap in a purer form. Its output *is* a wash, drawn
//! at `canvas::overlay::TEXT_SELECTION_ALPHA` — deliberately low, so the
//! operator can read the text through it — over the linework of a CAD sheet. A
//! capture of a page with three words selected and a capture of the same page
//! with nothing selected are very nearly the same image, and on a dense drawing
//! they may be the same image to any threshold a pixel oracle could use.
//!
//! So the oracle is the **trace**, and the application was given a line to say
//! it with: `canvas-text-selection via=… page=… chars=… quads=…`. `chars=` is
//! the byte length of the string a copy would put on the clipboard, read from
//! the same field the copy reads — so a passing check here is a statement about
//! what would be copied, not merely about what was painted.
//!
//! # ★ The four-link chain, and which link no unit test observes
//!
//! | # | Link | Where | Its own test |
//! |---|---|---|---|
//! | 1 | a press means text exactly when the mode cannot select content | `canvas::textsel::takes_the_press` | yes |
//! | 2 | `press_kind` turns that into `DragKind::TextSelect` | `canvas::gesture::meaning` | yes |
//! | 3 | the state machine carries it across the frames of a drag | `canvas::gesture` | yes |
//! | 4 | `canvas::interact` builds a `PageContext` from the document and applies the outcome | `canvas::interact` | **no** |
//!
//! Link 4 is the one a refactor breaks silently, and it is the same shape as
//! `read_mode`'s link 3: it is a value assembled per frame out of `&OpenDoc`,
//! and every one of its parts has a plausible wrong answer that compiles. The
//! `epoch` could be read from the wrong place and stamp every selection stale on
//! arrival; the `page` could be the strip's first rather than the acting one, so
//! every canvas point would be converted against another sheet's transform; the
//! whole `if let` could be skipped on a frame where `page_text()` answered
//! `None`, leaving a gesture that silently does nothing on exactly the pages
//! whose content stream is unusual. **None of those breaks a test in the
//! workspace.**
//!
//! # The phases, and why the Edit phase is load-bearing
//!
//! The check is an assertion about a **presence** in Read and an **absence** in
//! Edit, and `crate::report`'s rule bites on the second: *never treat an absence
//! as evidence unless you have shown the thing that would have produced it was
//! working*. So the same document drag is performed in both modes:
//!
//! | Phase | Mode | Drag | Expected | If it does not hold |
//! |---|---|---|---|---|
//! | A | Read | across a band of the page | `canvas-text-selection` with `chars` > 0 and `quads` > 0 | this band had no text; try the next |
//! | B | Read | — | Escape clears it | — (not driven: keyboard, see below) |
//! | C | Edit | the **same** drag | **no** new `canvas-text-selection` line | FAIL — the text gesture escaped into the mode whose primary button is the content marquee |
//!
//! Phase A failing is **not** a failure of the application — it means the
//! harness swept blank paper, which on a drawing sheet is most of it. So the
//! band ladder is retried, and if no band has text under it the check reports
//! SKIP naming exactly that, never PASS.
//!
//! Phase C is the half that would be easy to omit and is the more dangerous
//! direction: a build where `takes_the_press` ignored its capabilities would
//! pass phase A perfectly and would have silently replaced Edit's marquee — the
//! only content-selection gesture the product has — with a text sweep.
//!
//! # ★ `quads` > 0 as well as `chars` > 0, and why both
//!
//! They are the two halves of the one-derivation promise
//! (`canvas::textsel` §5): the same pass produces the string and the boxes. A
//! build where the quad grouping silently produced nothing would still copy the
//! right text and would highlight **nothing at all** — a selection the operator
//! cannot see, which is indistinguishable from a gesture that did not work. That
//! is the single most likely way this feature ships broken, and it is one field
//! on the line.
//!
//! # Mouse only
//!
//! ★ **Escape, Ctrl+A and Ctrl+C are not driven here, and that is unwritten
//! work rather than a limitation.** This block used to say synthetic keyboard
//! input could not reach the window; it can (see [`crate::checks::add_text`]),
//! and the claim was a misreading of the dead-keymap defect.
//! **So Escape, Ctrl+A and Ctrl+C are not driven here**, and this file says so
//! rather than implying otherwise: they are covered by unit test alone
//! (`canvas::keys` for Escape, `canvas::textsel` for the other two), and the gap
//! is on the record. Nothing in the phases above needs a key.
//!
//! # Every way this reports SKIP, and why none of them is a pass
//!
//! * no binary, no `--pdf`, `--no-input` — the harness never began;
//! * the diagnostic switches did not reach the process;
//! * the page size could not be read and no `--page-size` was given — without
//!   the page height there is no y-flip;
//! * a mode segment was never declared, or took no click;
//! * the canvas is not showing page 1, so the harness's one known page size does
//!   not describe the page it would be sweeping;
//! * **no band had text under it** — phase A never succeeded, so there is
//!   nothing for phase C's silence to be measured against.

use crate::checks::driving::{self, SHELL_DIAG_ENV};
use crate::checks::{Check, CheckContext};
use crate::coords::{CanvasMapping, DocPoint, PageGeometry, ScreenPoint};
use crate::error::{Error, Result};
use crate::input::Driver;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;
use crate::trace::Trace;

/// The mode whose primary button sweeps text: its tab list is `["file",
/// "view"]`, so `Capabilities::for_mode` grants it no `edit_content` and
/// `canvas::textsel::takes_the_press` answers `true`.
const READ: &str = "read";

/// The control mode: the one whose tab list contains `edit`, so the identical
/// drag is expected to be a content marquee and to trace no text selection.
///
/// Edit rather than Review, deliberately, and for the *opposite* reason
/// [`crate::checks::read_mode`] chooses it: Review would also sweep text —
/// `edit_content` is false there too — so a check that compared Read against
/// Review would be comparing two selections and would pass against a build that
/// had removed the mode gate entirely.
const EDIT: &str = "edit";

/// `canvas-text-selection via=… page=… chars=… quads=…` — `canvas::trace`'s
/// report of what the text selection just became.
const TEXT_EVENT: &str = "canvas-text-selection";

/// The byte length of the selected string. `> 0` is the whole verdict.
const CHARS_FIELD: &str = "chars";

/// How many line boxes the wash is drawn from. See the module header on why
/// this is asserted beside `chars`.
const QUADS_FIELD: &str = "quads";

/// `page-text page=… runs=… chars=… ms=… status=…` —
/// `app::cache::PageTextCache`'s report of an extraction it actually paid for.
///
/// Read here for the **cost** note rather than for a verdict: a cache that
/// worked emits one of these per `(page, epoch)` and a cache that did not emits
/// one per gesture frame, and the difference is visible by counting.
const PAGE_TEXT_EVENT: &str = "page-text";

/// **Where to sweep, as fractions of the page box** — the two ends of each
/// candidate band.
///
/// # Why bands rather than points
///
/// [`crate::checks::read_mode`]'s ladder looks for a *point* with an object
/// under it. This needs a **horizontal run** with glyphs along it, which is a
/// different target and a more forgiving one: a sweep that starts on blank paper
/// and ends on a word still selects, because
/// `EditableTextModel::hit_test` resolves an off-text point to the nearest line
/// — which is Acrobat's behaviour and is documented as such in
/// `canvas::textsel::hit`.
///
/// # Why these, in this order
///
/// Ordered for the drawing fixtures this project uses (`HANDOFF.md` §2's
/// table). A SolidWorks sheet keeps its dense text in the **title block, bottom
/// right**, and its sparse text in view labels across the middle — so the title
/// block is tried first here, where `read_mode`'s object ladder tries the middle
/// first. Each band is a wide sweep, because a narrow one on a sparse sheet is
/// a coin toss.
///
/// ★ `pub(crate)` because [`crate::checks::text_markup`] sweeps the same
/// ladder for the same reason — it needs a selection before it can mark one —
/// and a second copy of a *calibration* is the thing this crate's
/// [`crate::profile`] module exists to prevent: the numbers are tuned to this
/// project's two fixtures, and two tunings drift apart silently, each check
/// SKIPping on a different sheet.
pub(crate) const BANDS: [((f64, f64), (f64, f64)); 8] = [
    // The title block, bottom right — three sweeps at different heights,
    // because its rows are close together and a single y can fall between them.
    ((0.62, 0.06), (0.97, 0.06)),
    ((0.62, 0.12), (0.97, 0.12)),
    ((0.62, 0.18), (0.97, 0.18)),
    // Across the middle of the sheet, where view labels and notes sit.
    ((0.10, 0.50), (0.90, 0.50)),
    ((0.10, 0.35), (0.90, 0.35)),
    ((0.10, 0.65), (0.90, 0.65)),
    // The top band, for documents that are prose rather than drawings.
    ((0.10, 0.88), (0.90, 0.88)),
    ((0.10, 0.78), (0.90, 0.78)),
];

/// See the module documentation.
pub struct TextSelectionSweepsAndCopies;

impl Check for TextSelectionSweepsAndCopies {
    fn name(&self) -> &'static str {
        "text_selection_sweeps_and_copies"
    }

    fn defect(&self) -> &'static str {
        "a drag across text on the page selects nothing, selects text but highlights nothing, or \
         sweeps text in a mode whose primary button is the content marquee — the canvas wiring \
         that assembles a page's extraction per frame, which is the one link in that chain no \
         unit test observes"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        match drive(ctx, &mut report) {
            Ok(Some(failure)) => report.fail(failure),
            Ok(None) => report.pass(),
            Err(skip) => report.skip(skip.to_string()),
        }
    }
}

/// Every `canvas-text-selection` line that reports a **non-empty** selection.
///
/// Filtered on `chars > 0` rather than counting the event, because a *clear* is
/// traced too — `chars=0` — and a check that counted lines would be satisfied by
/// the gesture that ends a selection as readily as by the one that makes it.
fn selections(trace: &Trace) -> Vec<&crate::trace::TraceLine> {
    trace
        .events(TEXT_EVENT)
        .filter(|l| l.get_usize(CHARS_FIELD).unwrap_or(0) > 0)
        .collect()
}

/// Aim a document point at the canvas as it is laid out **right now**.
///
/// Re-derived per use rather than cached, and that is required rather than
/// careful: Read defaults to a continuous strip and Edit to a single page
/// (`viewer::display::default_for_mode`), so the same `DocPoint` is a different
/// screen pixel in the two modes. That is exactly why this crate writes document
/// coordinates and never screen ones.
///
/// `pub(crate)` for [`crate::checks::text_markup`], which sweeps in a third mode
/// and would otherwise carry a fourth copy of the same three lines. The mode
/// sensitivity above is precisely why it must be a *function* rather than a
/// value either check could cache.
pub(crate) fn aim(
    ctx: &CheckContext,
    session: &Session,
    page: PageGeometry,
    at: DocPoint,
) -> Result<ScreenPoint> {
    let trace = session.trace()?;
    let shown = trace
        .last(ctx.profile.vocab.canvas_event)
        .and_then(|l| l.get_usize("page"));
    if shown != Some(at.page) {
        return Err(Error::new(format!(
            "the canvas is showing page {}, and this check's point is on page {}. Converting a \
             document point against another page's rect would put it somewhere plausible and \
             wrong.",
            shown.map_or_else(|| "an unreported index".to_owned(), |p| (p + 1).to_string()),
            at.page + 1
        )));
    }
    let mapping = CanvasMapping::from_trace(&trace, &ctx.profile.vocab, page, at.page)?;
    Ok(session.frame()?.to_screen(mapping.doc_to_window(at)?))
}

/// Run the sequence. `Err` is SKIP, `Ok(Some(_))` is FAIL, `Ok(None)` is a pass.
#[allow(clippy::too_many_lines)]
fn drive(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    // --- preconditions -----------------------------------------------------
    let exe = ctx.resolve_exe().ok_or_else(|| {
        Error::new(format!(
            "no binary to drive. Pass --exe, or build the profile's default at {}.",
            ctx.profile.default_exe
        ))
    })?;
    let pdf = ctx.pdf.clone().ok_or_else(|| {
        Error::new(
            "no --pdf. This check sweeps the pointer across page text and asserts on what the \
             selection did about it, so it needs a document with readable text on its first page.",
        )
    })?;
    if !ctx.allow_input {
        return Err(Error::new(
            "input is disabled (--no-input). This check is a real drag across a real canvas and \
             it needs the pointer and the foreground. Reported as SKIPPED rather than passed: a \
             check that did not run has learned nothing.",
        ));
    }
    let ui_rect = ctx.profile.vocab.ui_rect_event.ok_or_else(|| {
        Error::new(format!(
            "the `{}` profile declares no ui-rect trace event, so the application cannot state \
             where its mode segments are and this check has nothing to aim at.",
            ctx.profile.name
        ))
    })?;
    let page: PageGeometry = match ctx.page_size {
        Some((w, h)) => PageGeometry {
            width_pt: w,
            height_pt: h,
        },
        None => crate::fixture::page_geometry(&pdf).ok_or_else(|| {
            Error::new(format!(
                "cannot read a page size from {}. The harness needs the page box to turn this \
                 check's fractions into points, and the page height to flip PDF y (up) into \
                 window y (down). Pass --page-size WxH. It refuses to guess: a wrong page height \
                 mirrors every sweep about the page centre, which lands on the page and selects \
                 something plausible.",
                pdf.display()
            ))
        })?,
    };
    report.note(format!(
        "fixture {} — page 1 is {:.0}x{:.0} pt",
        pdf.display(),
        page.width_pt,
        page.height_pt
    ));

    // --- launch, with BOTH diagnostic channels armed -----------------------
    let mut spec = LaunchSpec::new(&exe, ctx.out("text_selection.trace.txt"));
    spec.pdf = Some(pdf.clone());
    spec.env.push((
        ctx.profile.diag_env.0.to_owned(),
        ctx.profile.diag_env.1.to_owned(),
    ));
    spec.env
        .push((SHELL_DIAG_ENV.0.to_owned(), SHELL_DIAG_ENV.1.to_owned()));
    spec.allow_stale = ctx.allow_stale;
    spec.source_root = ctx.source_root.clone();

    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    report.note(format!(
        "launched {} as pid {} with {}={} and {}={}",
        exe.display(),
        session.pid(),
        ctx.profile.diag_env.0,
        ctx.profile.diag_env.1,
        SHELL_DIAG_ENV.0,
        SHELL_DIAG_ENV.1
    ));
    report.artifact(session.trace_path().to_path_buf());
    session.settle(40);

    let trace = session.trace()?;
    if !trace.started(ctx.profile.vocab.start_event) {
        return Err(Error::new(format!(
            "the trace has no `{}` line, so the diagnostic switch {}={} did not reach the \
             process and this check has no oracle. Captured stderr is at {}.",
            ctx.profile.vocab.start_event,
            ctx.profile.diag_env.0,
            ctx.profile.diag_env.1,
            session.trace_path().display()
        )));
    }
    let driver = Driver::new(session.window());

    // --- phase A: sweep in Read --------------------------------------------
    let mut unreachable: Vec<String> = Vec::new();
    let mut found: Option<(usize, String)> = None;

    for (n, (start, end)) in BANDS.iter().enumerate() {
        driving::click_mode_segment(&session, &driver, ui_rect, READ)?;
        let from = DocPoint::new(0, start.0 * page.width_pt, start.1 * page.height_pt);
        let to = DocPoint::new(0, end.0 * page.width_pt, end.1 * page.height_pt);
        let (from, to) = match (aim(ctx, &session, page, from), aim(ctx, &session, page, to)) {
            (Ok(a), Ok(b)) => (a, b),
            (Err(e), _) | (_, Err(e)) => {
                // Not fatal: this band cannot be reached in this mode's layout,
                // which says nothing about the gesture. Recorded so a run where
                // NO band was reachable can say so rather than reporting
                // "no text".
                unreachable.push(format!("band {}: {}", n + 1, e.message()));
                continue;
            }
        };

        let before = selections(&session.trace()?).len();
        driver.drag(from, to)?;
        session.settle(16);
        let after = session.trace()?;
        let lines = selections(&after);
        // ★ The **last** new line, not the first.
        //
        // A sweep traces every distinct state it passes through — measured on
        // `SW41177.pdf`: `chars=1`, then `9`, then `17`, then the settled
        // `via=drag chars=17`. The first is a real selection and a poor verdict:
        // it is one character, taken on the frame egui first called the press a
        // drag, and a build whose range stopped growing after the first frame
        // would pass a check that read it. The last line is the selection the
        // operator is actually left holding, which is what the assertion is
        // about and what the report should print.
        if let Some(line) = lines.last().filter(|_| lines.len() > before) {
            let quads = line.get_usize(QUADS_FIELD).unwrap_or(0);
            if quads == 0 {
                return Ok(Some(format!(
                    "THE SELECTION HIGHLIGHTS NOTHING. A sweep across band {} traced `{}` — so \
                     `canvas::textsel::resolve` produced a non-empty string and no line boxes. \
                     Those two are supposed to be one derivation, from one pass over one byte \
                     range (`canvas::textsel` header section 5): the string is sliced from the \
                     covered runs and the boxes are accumulated from the glyphs inside the same \
                     windows. `quads=0` with `chars>0` means the box half fell out — look at the \
                     line grouping in `resolve`, at `find::reveal::quad_to_canvas` declining the \
                     page's transform, and at whether `EditableTextModel::lines()` claimed the \
                     glyphs at all. The operator sees a gesture that copies text and shows \
                     nothing, which is indistinguishable from one that did not work.",
                    n + 1,
                    line.raw
                )));
            }
            report.note(format!(
                "band {}: the sweep traced `{}` — so the drag reached the canvas, the press meant \
                 text, and the range covers glyphs",
                n + 1,
                line.raw
            ));
            found = Some((n + 1, line.raw.clone()));
            break;
        }
        report.note(format!(
            "band {}: no text under the sweep from ({:.0}, {:.0}) to ({:.0}, {:.0}); trying the \
             next",
            n + 1,
            start.0 * page.width_pt,
            start.1 * page.height_pt,
            end.0 * page.width_pt,
            end.1 * page.height_pt
        ));
    }

    let Some((band, read_line)) = found else {
        return Err(Error::new(format!(
            "no band had text under it: {} sweeps were performed in `{READ}` and none of them \
             selected a character. This check declines to call that a pass — with nothing \
             established to have been selectable, {EDIT}'s silence in phase C would prove \
             nothing. {}Trace: {}.",
            BANDS.len(),
            if unreachable.is_empty() {
                String::new()
            } else {
                format!(
                    "Bands that could not be aimed at all, which is a different problem and may \
                     be the whole of this one: {}. ",
                    driving::list(&unreachable)
                )
            },
            session.trace_path().display()
        )));
    };

    // --- the picture, saved as evidence rather than asserted on -------------
    //
    // ★ Captured, and deliberately **not** used as an oracle.
    //
    // This whole file exists because the wash cannot be an oracle: at
    // `TEXT_SELECTION_ALPHA` over a drawing sheet, selected and unselected are
    // the same picture to any threshold (the module header's first section). So
    // the verdict is the trace, and this is evidence — the thing a reader needs
    // when the trace says `chars=17 quads=1` and they want to know whether the
    // band landed on the words they expected, or whether the highlight covers
    // its own text the way Find's current hit once did (`HANDOFF.md` §2's
    // defect 3, and `canvas::overlay::CURRENT_ALPHA`'s screenshot-derived
    // bound).
    //
    // `crate::capture`'s own rule: *every check that looks at pixels saves its
    // evidence, pass or fail — on a failure it is what the reader needs; on a
    // pass it is what makes the next failure diagnosable.* A capture failure is
    // noted and does not fail the check: the assertion above has already held,
    // and refusing a verdict because a screenshot could not be taken would be
    // the harness failing the application for its own limitation.
    let shot = ctx.out("text_selection.png");
    match crate::capture::window_to_png(&session, &shot) {
        Ok(_) => {
            report.artifact(shot);
            report.note(
                "the window with the selection on it is saved beside the trace. It is evidence, \
                 not the oracle: at the wash's alpha over a drawing sheet, selected and \
                 unselected text are the same picture to any threshold, which is why this check \
                 asserts on `chars` and `quads` instead",
            );
        }
        Err(e) => {
            report.note(format!(
                "could not capture the window ({e}); the trace assertion above still stands, and \
                 it is the one this check's verdict rests on"
            ));
        }
    }

    // --- the cost note, from the application's own measurement --------------
    //
    // Not a verdict — there is no threshold here that would not be a claim about
    // one machine — but the count IS the evidence for the cache, and a run that
    // did not report it would leave the affordability argument unmeasured.
    let extractions = session.trace()?.events(PAGE_TEXT_EVENT).count();
    report.note(format!(
        "{extractions} `{PAGE_TEXT_EVENT}` line(s) so far — one per (page, edit epoch) actually \
         extracted. A sweep is many frames; anything near the frame count means the cache in \
         `app::cache::PageTextCache` is not being hit"
    ));
    if let Some(line) = session.trace()?.last(PAGE_TEXT_EVENT) {
        report.note(format!("most recent extraction: `{}`", line.raw));
    }

    // --- phase C: the same sweep in Edit ------------------------------------
    driving::click_mode_segment(&session, &driver, ui_rect, EDIT)?;
    let (start, end) = BANDS[band - 1];
    let from = aim(
        ctx,
        &session,
        page,
        DocPoint::new(0, start.0 * page.width_pt, start.1 * page.height_pt),
    )?;
    let to = aim(
        ctx,
        &session,
        page,
        DocPoint::new(0, end.0 * page.width_pt, end.1 * page.height_pt),
    )?;
    let before = selections(&session.trace()?).len();
    driver.drag(from, to)?;
    session.settle(16);
    let after = session.trace()?;
    let lines = selections(&after);
    if lines.len() > before {
        let line = lines.last().map_or("", |l| l.raw.as_str());
        return Ok(Some(format!(
            "THE TEXT GESTURE ESCAPED INTO `{EDIT}`. The same document band that selected text in \
             `{READ}` — `{read_line}` — was swept again with the mode selector on `{EDIT}`, whose \
             tab list contains `edit`, and the application traced `{line}`. In a mode with \
             `edit_content` there is nothing that may write that line: \
             `canvas::textsel::takes_the_press` requires `!caps.edit_content`, \
             `canvas::gesture::press_kind` therefore yields `Marquee(Select)` rather than \
             `TextSelect`, and `canvas::interact` asks the same predicate again before routing a \
             click. So the capabilities the canvas was handed this frame were not `{EDIT}`'s — or \
             the predicate stopped consulting them. Note that this build has silently replaced \
             the only content-selection gesture the product has."
        )));
    }
    report.note(format!(
        "the same band in {EDIT} traced no new `{TEXT_EVENT}` — the primary button is the content \
         marquee there, as it was before this feature existed"
    ));
    report.note(format!(
        "verdict established on band {band}: {READ} swept text the same drag did not sweep in \
         {EDIT}, and the selection it made both copied characters and drew boxes"
    ));
    // ★ Escape, Ctrl+A and Ctrl+C are NOT driven — see the module header's
    // "Mouse only" section. Said in the report rather than only in the source,
    // so a reader of a PASS knows exactly what it does and does not cover.
    report.note(
        "not covered here: Escape clears the selection, Ctrl+A selects the page and Ctrl+C copies \
         it. This check does not drive them; keystrokes DO reach the window (see \
         find_bar), so all three are covered by unit test alone and the gap is on the record \
         rather than implied by a green result",
    );
    Ok(None)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The two modes are the two the argument needs, and they are not the same
    /// one.
    ///
    /// `EDIT` in particular: comparing Read against **Review** would compare two
    /// modes that both sweep text, and would pass against a build that had
    /// deleted the mode gate outright. See [`EDIT`]'s own documentation.
    #[test]
    fn the_control_mode_is_the_one_that_does_not_sweep() {
        assert_eq!(READ, "read");
        assert_eq!(EDIT, "edit");
        assert_ne!(READ, EDIT);
    }

    /// Every band is well inside the page box, so both ends land on paper
    /// rather than on the grey surround whatever size the fixture is — and
    /// every band is a real sweep rather than a click.
    #[test]
    fn every_band_is_inside_the_page_and_actually_travels() {
        for ((x0, y0), (x1, y1)) in BANDS {
            for f in [x0, y0, x1, y1] {
                assert!((0.05..=0.98).contains(&f), "fraction {f} is off the page");
            }
            assert!(
                (x1 - x0).abs() >= 0.2,
                "a band that travels {:.2} of the page width is a click, not a sweep",
                (x1 - x0).abs()
            );
        }
        assert!(
            BANDS.len() >= 6,
            "a ladder short enough to miss every text run on a drawing sheet would turn a \
             working gesture into a SKIP more often than it would find text"
        );
    }

    /// ★ **A cleared selection is not a selection.**
    ///
    /// `canvas::trace` emits `chars=0` for a clear — deliberately, because a
    /// clear is a real event with a real cause — so a check that counted
    /// `canvas-text-selection` lines would be satisfied by the gesture that
    /// *ends* a selection as readily as by the one that makes it. Phase C's
    /// whole verdict turns on this filter.
    #[test]
    fn only_a_non_empty_selection_counts() {
        let trace = Trace::parse(
            "pdfcer-diag canvas-text-selection via=clear page=0 chars=0 quads=0\n\
             pdfcer-diag canvas-text-selection via=drag page=0 chars=27 quads=2",
            "pdfcer-diag",
        );
        let found = selections(&trace);
        assert_eq!(found.len(), 1, "the clear must not be counted");
        assert_eq!(found[0].get_usize(CHARS_FIELD), Some(27));
        assert_eq!(found[0].get_usize(QUADS_FIELD), Some(2));
    }

    /// …and a selection that copies text while highlighting nothing is read as
    /// the distinct failure it is, rather than as a pass.
    ///
    /// This is the one-derivation promise's failure mode, and it is a FAIL and
    /// not a SKIP: unlike "no text under the sweep", it is evidence the gesture
    /// ran and produced half an answer.
    #[test]
    fn a_selection_with_no_boxes_is_visible_to_the_filter() {
        let trace = Trace::parse(
            "pdfcer-diag canvas-text-selection via=drag page=0 chars=27 quads=0",
            "pdfcer-diag",
        );
        let found = selections(&trace);
        assert_eq!(found.len(), 1, "it is still a selection");
        assert_eq!(
            found[0].get_usize(QUADS_FIELD),
            Some(0),
            "…and the check reads the box count separately, which is what lets it FAIL here"
        );
    }
}
