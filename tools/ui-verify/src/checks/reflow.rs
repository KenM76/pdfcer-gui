//! `reflowing_a_paragraph_rewraps_it` — **put the caret in a paragraph, press
//! Reflow, and the block loses a line.**
//!
//! # What this is for
//!
//! `OPERATOR_REQUESTS.md` **O54(b)**, 2026-08-28:
//!
//! > *"I think the paragraph reflow was implemented ages ago in the pdfcer core,
//! > so we should have that option too."*
//!
//! He was right. `EditSession::reflow_block` shipped in `Pass 91`, was tested,
//! was documented, and **no control in this shell had ever raised it**. That is
//! the failure mode this check exists at: not a broken verb, an unreachable
//! one.
//!
//! ## ★★★ Why a unit test cannot see what this sees
//!
//! Because every link in front of the verb is wiring, and wiring is what this
//! project keeps shipping broken with a green suite:
//!
//! | # | link | a unit test can see it? |
//! |---|---|---|
//! | 1 | `edit.reflow_block` is registered and drawn on the Edit tab | yes — a registry test |
//! | 2 | a click on it reaches `dispatch_command` | **no** |
//! | 3 | the arm finds the **caret's draft** in egui memory | **no** — the draft is frame state |
//! | 4 | the draft's `Anchor::Run` resolves to a **block index** | partly |
//! | 5 | the action reaches `reflow_block` and the engine re-wraps | yes |
//!
//! ★ Link 3 is the one with no other instrument. The operand is not a
//! selection the application holds — it is a caret in `egui`'s temporary data,
//! written by a click and read by a command, and nothing but a driven run puts
//! a real one there.
//!
//! ## ★★ The oracle is the LINE COUNT, and it is a real one
//!
//! `reflow-block-applied … lines=6->5`. The fixture is built so a correct
//! reflow **must** change that number: six deliberately short ragged lines,
//! wrapping to the widest of them, packs to five. A build that raised the
//! action, reached the engine and re-wrapped nothing would report `6->6`, and
//! that is asserted as a **failure** rather than accepted as "it ran".
//!
//! ⇒ `tools/gen-reflow-fixture.py` and this file are one instrument. Its header
//! records why no existing fixture would do: a CAD title block has no
//! paragraph at all, and `tail-alignment.pdf`'s blocks are placed flush by
//! measurement, so re-wrapping them has nothing to do. **A check driven against
//! either would report the feature broken about a build whose reflow works.**
//!
//! ## ★★★ What this check deliberately does NOT do
//!
//! It never types. `reflow_block` is planned against the **base** document and
//! refuses a page this session has already rewritten — so a check that typed
//! one character first would exercise the refusal, not the reflow, and would
//! read as a failure. The refusal has its own sentence in `text::textedit` and
//! its own arm; this check is about the path that works.
//!
//! ★ That constraint is a fact about the feature and is stated here rather
//! than discovered by whoever extends this file. It is the reason the sequence
//! is *click, press* and not *click, type, press*.

use crate::checks::driving::{self, SHELL_DIAG_ENV, declared, declared_names, list};
use crate::checks::text_selection::aim;
use crate::checks::{Check, CheckContext};
use crate::coords::{DocPoint, PageGeometry};
use crate::error::{Error, Result};
use crate::geom::LRect;
use crate::input::Driver;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;

/// Edit mode, then arm the caret — both through the harness seam.
///
/// ★ `edit.text` is rung here rather than clicked because arming the caret is
/// **not what this check is about**, and it already has its own driven check.
/// A check that re-verifies its own preconditions through the slowest possible
/// route fails for reasons that are not its subject.
const INVOKE: &str = "mode.edit,edit.text";
/// `text-edit-caret page=… run=… len=…` — a click resolved a run.
const CARET_EVENT: &str = "text-edit-caret";
/// `text-edit-declined reason=…` — a click did not.
const DECLINED_EVENT: &str = "text-edit-declined";
/// The Reflow control, as (region, command id).
///
/// ★ The pair is written once so the region and the id cannot drift apart: a
/// check that clicked one control and asserted about another would pass or fail
/// for reasons unrelated to either.
const REFLOW_ITEM: (&str, &str) = ("ribbon.item.edit.reflow_block", "edit.reflow_block");
/// The ribbon tab that carries [`REFLOW_ITEM`]. A mode is not a tab — see the
/// click in [`drive`].
const EDIT_TAB: &str = "ribbon.tab.edit";
/// `reflow-resolved page=… run=… block=…` — the caret became a block index.
const RESOLVED_EVENT: &str = "reflow-resolved";
/// `reflow-declined reason=…` — it did not, and the shell said which way.
const DECLINE_EVENT: &str = "reflow-declined";
/// `reflow-block-applied page=… block=… lines=A->B …` — the engine re-wrapped.
///
/// ★ `-applied`, per the convention this project adopted after making the
/// same-name mistake twice: `vector_edit` writes its own bare `reflow-block …`
/// line for the identical edit, and `.last()` on the bare name reads that one.
const APPLIED_EVENT: &str = "reflow-block-applied";
/// The page's own region, so a failure can say whether a sheet was drawn.
const PAGE_REGION: &str = "page";

/// Where to put the caret, in PDF user space on the fixture.
///
/// ★★ Inside **line 2** of the block — `x = 120`, baseline `y = 668` — rather
/// than in the first or last line. A first-line caret would pass on a build
/// whose block lookup returned 0 unconditionally, and the last line is the one
/// most likely to be split off by a recogniser that disagrees about the block's
/// extent. The middle is the honest place to ask.
///
/// ★ The numbers come from `tools/gen-reflow-fixture.py`, which prints the
/// geometry it computed for exactly this reason. They are quoted, not derived
/// twice.
const CARET_AT: (f64, f64) = (120.0, 668.0);
/// The line count the fixture starts with.
const LINES_BEFORE: u32 = 6;

/// See the module documentation.
pub struct ReflowingAParagraphRewrapsIt;

impl Check for ReflowingAParagraphRewrapsIt {
    fn name(&self) -> &'static str {
        "reflowing_a_paragraph_rewraps_it"
    }

    fn defect(&self) -> &'static str {
        "paragraph reflow shipped in the engine and no control in the shell raised it — the \
         operator asked for a capability he already owned, which is what an unreachable verb \
         looks like from outside"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        match drive(ctx, &mut report) {
            Ok(Some(failure)) => report.fail(failure),
            Ok(None) => report.pass(),
            Err(why) => report.from_error(&why),
        }
    }
}

#[allow(clippy::too_many_lines)]
fn drive(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    if !ctx.allow_input {
        return Err(Error::new(
            "input is disabled (--no-input). This check clicks into a paragraph and then clicks \
             a ribbon control. Both are real pointer gestures, and the caret it needs exists \
             only because a click put it there.",
        ));
    }
    let exe = ctx.resolve_exe().ok_or_else(|| {
        Error::new(format!(
            "no binary to drive. Pass --exe, or build the profile's default at {}.",
            ctx.profile.default_exe
        ))
    })?;
    // ★★★ The fixture is NOT overridable by `--pdf`, and that is deliberate
    // where every neighbouring check accepts one. The oracle is a line count
    // this specific document produces; run against another file the check
    // would still report `6->5` as its expectation and would fail on a healthy
    // build. A check whose expectation is bound to its fixture must own it.
    let pdf =
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/paragraph.pdf");
    if !pdf.exists() {
        return Err(Error::new(format!(
            "the fixture is missing at {}. Regenerate it: python tools/gen-reflow-fixture.py \
             — the oracle here is a line count only that document produces.",
            pdf.display()
        )));
    }
    let page = PageGeometry {
        // The fixture is US Letter by construction — see its generator. Read
        // from the file rather than assumed, so a regenerated fixture with a
        // different size fails loudly instead of aiming at blank paper.
        width_pt: 612.0,
        height_pt: 792.0,
    };
    let measured = crate::fixture::page_geometry(&pdf);
    if let Some(actual) = measured
        && (actual.width_pt - page.width_pt).abs() > 1.0
    {
        return Err(Error::new(format!(
            "fixtures/paragraph.pdf measures {}x{} pt and this check's caret point is quoted in \
             the US Letter geometry its generator prints. Regenerate the fixture or update \
             CARET_AT — aiming the old point at a new sheet lands on blank paper, which is \
             symptom-identical to a broken hit test.",
            actual.width_pt, actual.height_pt
        )));
    }
    let ui_rect = ctx.profile.vocab.ui_rect_event.ok_or_else(|| {
        Error::new(format!(
            "the `{}` profile declares no ui-rect trace event, so the application cannot say \
             where its controls are.",
            ctx.profile.name
        ))
    })?;

    let mut spec = LaunchSpec::new(&exe, ctx.out("reflow.trace.txt"));
    spec.pdf = Some(pdf.clone());
    spec.env.push((
        ctx.profile.diag_env.0.to_owned(),
        ctx.profile.diag_env.1.to_owned(),
    ));
    spec.env
        .push((SHELL_DIAG_ENV.0.to_owned(), SHELL_DIAG_ENV.1.to_owned()));
    spec.env
        .push(("PDFCER_DIAG_INVOKE".to_owned(), INVOKE.to_owned()));
    spec.allow_stale = ctx.allow_stale;
    spec.source_root = ctx.source_root.clone();

    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    report.artifact(session.trace_path().to_path_buf());
    report.note(format!(
        "launched {} on fixtures/paragraph.pdf as pid {} with PDFCER_DIAG_INVOKE={INVOKE}",
        exe.display(),
        session.pid()
    ));
    session.settle(40);
    let driver = Driver::new(session.window());

    // ★★★ **SELECT THE EDIT TAB — added 2026-09-05, on the first run this check
    // ever had.**
    //
    // `PDFCER_DIAG_INVOKE=mode.edit,edit.text` sets the MODE and arms the tool,
    // and neither of those selects a ribbon TAB. The band was still showing
    // File, so `ribbon.item.edit.reflow_block` was not declared and this check
    // SKIPPED with *"the Reflow control is not on the Edit tab"* — a sentence
    // about a control that is on the Edit tab, from a run that was looking at
    // the File one. Its own reason listed what it saw and every entry began
    // `ribbon.item.file.`, which is the tell.
    //
    // ⇒ A mode is not a tab. Same seam `add_text` and `adopt_widget` already
    // click through; this check was written without it.
    let trace = session.trace()?;
    let tab = declared(&trace, ui_rect, EDIT_TAB).ok_or_else(|| {
        Error::new(format!(
            "no `{EDIT_TAB}` region, so the ribbon cannot be put on the tab that carries Reflow. Tabs declared: {}.",
            list(&declared_names(&trace, ui_rect, "ribbon.tab."))
        ))
    })?;
    driver.click_at(session.frame()?.declared_center(tab))?;
    session.settle(20);

    if declared(&session.trace()?, ui_rect, PAGE_REGION).is_none() {
        return Err(Error::new(format!(
            "the application declared no `{PAGE_REGION}` region, so no sheet is on screen and \
             there is no paragraph to click into. Regions beginning `page`: {}.",
            list(&declared_names(&session.trace()?, ui_rect, "page"))
        )));
    }

    // --- 1: put the caret in the middle line of the paragraph --------------
    let at = aim(
        ctx,
        &session,
        page,
        DocPoint::new(0, CARET_AT.0, CARET_AT.1),
    )?;
    driver.click_at(at)?;
    session.settle(24);

    let trace = session.trace()?;
    let Some(caret) = trace.events(CARET_EVENT).last() else {
        let declined = trace
            .events(DECLINED_EVENT)
            .filter_map(|l| l.get("reason").map(str::to_owned))
            .last();
        return Ok(Some(match declined {
            Some(reason) => format!(
                "THE CARET WAS REFUSED WITH reason={reason} on a plain left-aligned paragraph. \
                 This is the step BEFORE the one under test and there is nothing about this \
                 fixture that should refuse — it is upright, single-font, one text object. \
                 Trace: {}.",
                session.trace_path().display()
            ),
            None => format!(
                "THE CLICK PRODUCED NEITHER A CARET NOR A DECLINE: no `{CARET_EVENT}`, no \
                 `{DECLINED_EVENT}`. Either `edit.text` did not arm — check for \
                 `command-declined id=edit.text` — or the click missed the paragraph. It aimed \
                 at ({}, {}) in page points, which `tools/gen-reflow-fixture.py` prints as \
                 inside line 2. Trace: {}.",
                CARET_AT.0,
                CARET_AT.1,
                session.trace_path().display()
            ),
        }));
    };
    report.note(format!(
        "★ the caret landed in the paragraph: `{}`",
        caret.raw
    ));

    // --- 2: press Reflow ---------------------------------------------------
    let rect = control(&session.trace()?, ui_rect, REFLOW_ITEM.0)?;
    driver.click_at(session.frame()?.declared_center(rect))?;
    session.settle(40);

    let trace = session.trace()?;
    let Some(resolved) = trace.events(RESOLVED_EVENT).last() else {
        // ★★★ The three-way message, because the three ways this fails are
        // three different repairs and a single "it did not work" would send
        // somebody to the wrong one.
        let declined = trace
            .events(DECLINE_EVENT)
            .filter_map(|l| l.get("reason").map(str::to_owned))
            .last();
        return Ok(Some(match declined.as_deref() {
            Some("no-caret") => format!(
                "★★★ THE COMMAND COULD NOT SEE THE CARET: `{DECLINE_EVENT} reason=no-caret`, \
                 while `{}` says one exists.\n\
                 The draft lives in egui's temporary memory and the dispatch reads it with \
                 `canvas::textedit::read`. If a ribbon click CLEARS the draft before the \
                 command runs — a focus change, an Escape rung by the tool row — the operand \
                 is gone by the time it is asked for, and this is the only instrument that can \
                 see that. Trace: {}.",
                caret.raw,
                session.trace_path().display()
            ),
            Some("run-not-in-a-block") => format!(
                "★★ THE RUN IS NOT IN A RECOGNISED BLOCK: `{DECLINE_EVENT} \
                 reason=run-not-in-a-block` on a six-line left-aligned paragraph, which is the \
                 shape block recognition is FOR.\n\
                 `canvas::textedit::reflow::block_of_run` recognises with \
                 `BlockRecognitionOptions::default()` and asks `block_at`. A default that no \
                 longer groups 16 pt-leaded 12 pt lines would produce exactly this. Check the \
                 engine's recogniser before the shell's lookup. Trace: {}.",
                session.trace_path().display()
            ),
            Some(other) => format!(
                "THE COMMAND DECLINED WITH reason={other}, which cannot be right here: this \
                 session has typed nothing and the caret is on existing page text. Trace: {}.",
                session.trace_path().display()
            ),
            None => format!(
                "★★★ THE CONTROL WAS PRESSED AND NOTHING HAPPENED: no `{RESOLVED_EVENT}`, no \
                 `{DECLINE_EVENT}`.\n\
                 **This is the exact state the feature was in before 2026-08-28** — the verb \
                 existed in the engine and no control raised it. The dispatch arm lives in \
                 `app::dispatch::text`; if `handles` no longer names `{}`, the command falls \
                 through to `command-unimplemented` and traces nothing else. Trace: {}.",
                REFLOW_ITEM.1,
                session.trace_path().display()
            ),
        }));
    };
    report.note(format!(
        "★★ the caret resolved to a block: `{}`",
        resolved.raw
    ));

    let Some(applied) = trace.events(APPLIED_EVENT).last() else {
        return Ok(Some(format!(
            "★★ THE BLOCK WAS RESOLVED AND NOTHING REACHED THE DOCUMENT: `{}` and no \
             `{APPLIED_EVENT}` line.\n\
             The action was raised and its apply arm never ran, or `reflow_block` refused. It \
             refuses a composite (Type0) block, a rotated or skewed one, and a block sharing a \
             text object with other content — this fixture is none of those by construction, so \
             a refusal here is itself the finding. A refused `vector_edit` traces \
             `reflow-block-refused`. Trace: {}.",
            resolved.raw,
            session.trace_path().display()
        )));
    };
    report.note(format!("★★★ the engine re-wrapped it: `{}`", applied.raw));

    // --- the oracle: the block actually lost a line ------------------------
    let lines = applied.get("lines").unwrap_or_default();
    let (before, after) = parse_lines(lines).ok_or_else(|| {
        Error::new(format!(
            "`{APPLIED_EVENT}` reported lines={lines:?}, which this check cannot read. It \
             expects `A->B`. Reported as SKIPPED rather than failed: a check that cannot read \
             its own oracle has learned nothing about the build."
        ))
    })?;
    if before != LINES_BEFORE {
        return Err(Error::new(format!(
            "the block was recognised as {before} lines and the fixture is written with \
             {LINES_BEFORE}. The recogniser and the generator disagree about what one paragraph \
             is, so the line-count oracle below would be measuring something other than the \
             re-wrap. Regenerate with `python tools/gen-reflow-fixture.py` and re-read its \
             printed geometry."
        )));
    }
    if after >= before {
        return Ok(Some(format!(
            "★★★ THE REFLOW RAN AND CHANGED NOTHING: `{}` reports lines={before}->{after}.\n\
             This fixture's six lines are deliberately short and ragged and wrap to the widest \
             of them, so a correct re-wrap MUST pack them into fewer — `pdfcer reflow --page \
             1 --block 0` on this file reports 6->5. Equal counts mean the wrap width was taken \
             as something other than the block box, or the request reached the engine with an \
             override this shell does not set. Trace: {}.",
            applied.raw,
            session.trace_path().display()
        )));
    }
    report.note(format!(
        "★ the paragraph packed from {before} lines to {after}, height_delta={}",
        applied.get("height_delta").unwrap_or("?")
    ));
    Ok(None)
}

/// `A->B` from the trace's `lines=` field.
///
/// ★ A parse rather than a `split` at the call site, so a malformed field is a
/// SKIP with a sentence instead of a silent `0->0` that would pass the
/// `after >= before` test by arithmetic accident.
fn parse_lines(field: &str) -> Option<(u32, u32)> {
    let (before, after) = field.split_once("->")?;
    Some((before.trim().parse().ok()?, after.trim().parse().ok()?))
}

/// One declared control rect, or a SKIP naming what was declared instead.
fn control(trace: &crate::trace::Trace, ui_rect: &str, region: &str) -> Result<LRect> {
    let rect = declared(trace, ui_rect, region).ok_or_else(|| {
        Error::new(format!(
            "the application declared no `{region}` region, so the Reflow control is not on the \
             Edit tab and there is nothing to press. Band controls it did declare: {}.",
            list(&declared_names(trace, ui_rect, driving::ITEM_PREFIX))
        ))
    })?;
    if rect.is_substantial() {
        Ok(rect)
    } else {
        Err(Error::new(format!(
            "`{region}` was declared at {rect:?}, which has no usable area to click."
        )))
    }
}
