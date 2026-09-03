//! `restyling_selected_text_reaches_the_document` — **sweep text, press Bold,
//! and the file changes.**
//!
//! # What this is for
//!
//! O37 — *"We should also have all the font tools available that Word does"* —
//! shipped on 2026-08-27, and every claim about it up to this check came from
//! unit tests over the release engine. R1 is explicit that this is not a report
//! of working software, and it is the founding rule of this project because two
//! of the old shell's worst defects were invisible to a green suite and obvious
//! within thirty seconds of using the program.
//!
//! ## ★★ The specific way this feature can pass its unit tests and be dead
//!
//! `app::actions::textstyle` is tested against a document. It calls
//! `format_text`, reads the file back, and asserts the size changed. **Every
//! one of those tests would still pass on a build where the panel never draws**
//! — where the section's guard is wrong, where the operator's sweep produces no
//! `text_selection`, where the panel is not in the dock, or where the Bold
//! button is below the fold of a scroll area.
//!
//! Each of those is a state a running window shows in two seconds and no unit
//! test can construct, because what is under test is *the chain of surfaces*
//! rather than the function at the end of it. That chain is:
//!
//! | # | link | its own test |
//! |---|---|---|
//! | 1 | a sweep on the canvas produces a `TextSelection` | shared with `clipboard_text` |
//! | 2 | `TextSelection::runs` answers with run ordinals | unit-tested |
//! | 3 | the Properties panel draws a Text section for it | ★ **nothing** |
//! | 4 | the section's read-back stamp does not wipe itself every frame | ★ **nothing** — a per-frame `sync`, and a stale stamp is invisible to a unit test |
//! | 5 | Bold is on screen and clickable | ★ **nothing** |
//! | 6 | the press reaches `format_text` | unit-tested, eight ways |
//!
//! Links 3, 4 and 5 have no other instrument. Link 4 is the one that would fail
//! most plausibly: `sync` runs every frame, and a stamp recomputed too eagerly
//! re-reads the document between the operator's press and the button's read.
//!
//! ## ★★ There is a link 0, and this check spent a red run learning it
//!
//! **The panel has to be on screen before any of the six can be observed**, and
//! the dock arrangement is *persisted per machine* — so it is not a constant
//! this check may assume. Until 2026-08-29 it assumed it, found no
//! `properties.text` region, and reported the panel as saying nothing about 266
//! selected characters. The panel was not saying nothing; it was not there.
//! [`INVOKE`] carries the fix and the evidence.
//!
//! ⇒ The general form, and it is worth stating because it will apply again:
//! **a driven check must put the surface it is about on screen itself.** An
//! arrangement that happens to be right on the machine the check was written on
//! is not a precondition, it is a coincidence, and the failure it produces
//! accuses the program of exactly the defect the check exists to find.
//!
//! # ★ Why Bold and not the size field
//!
//! Because it is **one click on a button** and the size field is an
//! `egui::DragValue`, which takes a number by double-click-then-type or by
//! scrub. `geometry_fields` scrubs, and its own header gives the reason to
//! prefer that over typing; but a scrub also has to be arithmetically
//! reconciled against a speed constant, and a check whose failure could be
//! either the program or its own arithmetic is a worse check.
//!
//! Bold has no such ambiguity: pressed or not pressed. It is also the control
//! with the most machinery behind it — the two-verb retry — so it is the one
//! whose silence would be most expensive.
//!
//! # The oracle, and why it is two lines rather than one
//!
//! `text-style … applied=N of=N`, **plus** the `format-text` label that
//! `vector_edit` writes when the edit reached the engine.
//!
//! One alone is not enough in either direction. The first without the second is
//! a module that decided to act and whose action never landed — the exact shape
//! of the grips' year-long defect. The second without the first cannot happen,
//! and asserting it alone would let a check pass on a build where some *other*
//! verb wrote that label in the same window.
//!
//! ★ A `text-style-declined` is reported as a **failure with its reason**, not
//! as a skip. A refusal here is the program answering, and the sentence it
//! answers with is the thing worth reading — a `FaceLacksCharacters` on this
//! fixture would mean the two-verb retry took the offer and the offer was bad,
//! which is a real finding about the engine and not a reason to stay quiet.

use crate::checks::driving::{self, SHELL_DIAG_ENV, click_mode_segment};
use crate::checks::{Check, CheckContext};
use crate::coords::{CanvasMapping, DocPoint, PageGeometry};
use crate::error::{Error, Result};
use crate::input::Driver;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;

/// The mode whose canvas may select page content and whose panels carry
/// Properties.
const MODE: &str = "edit";
/// The commands run at startup, in order, before this check touches anything.
///
/// # ★★★ The Properties panel has to be ASKED FOR, and not asking for it cost
/// a red run
///
/// `file.properties` is the command that mounts and activates the Properties
/// panel from **any** arrangement (`app::panels::show_panel` — it activates the
/// panel if it is already mounted and otherwise looks its home up in the `edit`
/// default, and it is idempotent rather than a toggle). Two sibling checks —
/// `field_delete_gate` and `annot_delete_gate` — already launch this way, and
/// the reason they give is the one that applies here:
///
/// > so the check does not have to know what dock layout the machine it runs
/// > on happens to have persisted.
///
/// This check did not, and on 2026-08-29 it failed with *"266 character(s) are
/// selected and the Properties panel says nothing about them"*. Its own failure
/// text named three candidates and **candidate (1) was the right one**. The
/// trace settles it without ambiguity:
///
/// | evidence | in `restyle-text.trace.txt` |
/// |---|---|
/// | six panel **bodies** were on screen | `dock.body.view.panel_tool`, `.panel_objects`, `.panel_pages`, `.panel_layers`, `.panel_forms`, `dock.body.measure.manage_groups` |
/// | ten panel **tabs** were declared | none of them `dock.tab.file.properties` |
/// | the Properties panel | **no tab, no body, and no `panel-shown` line** |
///
/// So the section was not guarded out and the run did not fail to pin — there
/// was no panel for `panels::properties::text::section` to draw into at all.
/// Candidates (2) and (3) could not have been reached, and neither can be
/// judged until this line exists.
///
/// ★ `mode.edit` is first and is load-bearing: `dispatch`'s mode arm sets the
/// ribbon mode and *"the dock follows on the same frame"*, so a panel mounted
/// before the mode moved would be mounted into the workspace the check is about
/// to leave. Ordering them here rather than clicking afterwards is what makes
/// that impossible.
///
/// ★★ The Edit-mode segment is still **clicked** below, and that is not
/// redundant: `docks` compares `ribbon.mode()` against `modes.active()` and
/// does nothing when they agree, so the click cannot disturb the panel, and it
/// keeps the check driving the operator's own gesture rather than trusting an
/// environment variable to have taken.
const INVOKE: &str = "mode.edit,file.properties";
/// The Text section's own region — its presence is the whole of link 3.
const SECTION_REGION: &str = "properties.text";
/// The Bold button.
const BOLD_REGION: &str = "properties.text.bold";
/// The `text-style-applied page=… change=… applied=… of=…` line — this
/// module's own summary of the whole gesture.
///
/// ★ Named `-applied` rather than plain `text-style` because `vector_edit`'s
/// label for the same edit is a sibling event, and trace matching is on the
/// exact event name. Two lines sharing a name is how a check reads the wrong
/// one and then reports `applied=0` about a gesture that worked.
const STYLE_EVENT: &str = "text-style-applied";
/// The `text-style-declined applied=…` line.
const DECLINED_EVENT: &str = "text-style-declined";
/// The label `vector_edit` writes when the restyle reached the engine — named
/// after the ENGINE verb, so it says which crate did the work.
const APPLIED: &str = "format-text";
/// The sweep's own oracle, shared with `clipboard_text`.
const SELECTION_EVENT: &str = "canvas-text-selection";
/// How far to sweep along the baseline, in PDF points.
///
/// Sixty. The fixture's title-block labels are six-point text a few tens of
/// points wide, so this crosses a whole label and a little blank paper. It does
/// **not** need to stop inside the run: a sweep that overshoots selects the run
/// and stops, which is the operator's own gesture.
const SWEEP_PT: f64 = 60.0;
/// How many scroll notches to spend looking for Bold below the panel's fold.
const SCROLL_ATTEMPTS: usize = 6;
/// `T`, as a Windows virtual key — the text-sweep tool.
///
/// ★★ Pressing it is not optional, and the first run of this check is why.
///
/// `textsel::gate::takes_the_press` reads
/// `tool.is_text() || (Select && !caps.edit_content)`. In **Edit** the second
/// disjunct is false, so a drag with the Select tool is an object marquee — the
/// first run of this check produced `sel=1` on the trace and reported the panel
/// as broken, when what had actually happened is that it never swept any text.
///
/// ★ That is a discoverability finding about the product, not only about the
/// harness, and it is written up rather than absorbed: an operator who wants to
/// restyle text in Edit mode has to know to arm a tool first, and nothing on
/// screen tells them so.
const VK_T: u16 = 0x54;

/// See the module documentation.
pub struct RestylingSelectedTextReachesTheDocument;

impl Check for RestylingSelectedTextReachesTheDocument {
    fn name(&self) -> &'static str {
        "restyling_selected_text_reaches_the_document"
    }

    fn defect(&self) -> &'static str {
        "text can be swept on the page and the Properties panel offers nothing to change about \
         it — or offers a Bold button that commits nothing, which is the same defect shape the \
         eight resize grips had for the whole life of the project"
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

/// Poll until the restyle reports one way or the other, and answer how long it
/// took.
///
/// ★ A bounded poll rather than a fixed sleep, for the reason `CONTINUE.md` §7
/// gives about harness-owned failure modes: a fixed sleep long enough for the
/// worst case makes every run slow, and one short enough to be pleasant fails
/// on the operator's own drawings — which is where this check is aimed.
///
/// Returns the elapsed milliseconds. Reaching the ceiling is **not** an error
/// here: the caller then reads a trace with neither line in it and reports the
/// "nothing happened" verdict, which is the right answer if a restyle really
/// can take twenty seconds.
fn wait_for_verdict(session: &Session) -> Result<u128> {
    const CEILING_MS: u128 = 20_000;
    let started = std::time::Instant::now();
    loop {
        session.settle(4);
        let trace = session.trace()?;
        if trace.last(STYLE_EVENT).is_some() || trace.last(DECLINED_EVENT).is_some() {
            return Ok(started.elapsed().as_millis());
        }
        if started.elapsed().as_millis() > CEILING_MS {
            return Ok(started.elapsed().as_millis());
        }
    }
}

#[allow(clippy::too_many_lines)]
fn drive(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    let vocab = &ctx.profile.vocab;
    let exe = ctx.resolve_exe().ok_or_else(|| {
        Error::new(format!(
            "no binary to drive. Pass --exe, or build the profile's default at {}.",
            ctx.profile.default_exe
        ))
    })?;
    let pdf = ctx
        .pdf
        .clone()
        .ok_or_else(|| Error::new("no --pdf. This check needs a page carrying real text."))?;
    let target = ctx.target.ok_or_else(|| {
        Error::new(
            "no --doc-point. Pass PAGE,X,Y in PDF user space naming the LEFT END of a piece of \
             text's baseline. `pdfcer extract-text --json` gives the first glyph's x and y of \
             every run; use those. A point on blank paper sweeps nothing and the check would \
             report the panel as broken.",
        )
    })?;
    if !ctx.allow_input {
        return Err(Error::new(
            "input is disabled (--no-input). This check sweeps the pointer across text and \
             presses a button, and neither can be simulated from the trace.",
        ));
    }
    let ui_rect = vocab.ui_rect_event.ok_or_else(|| {
        Error::new(format!(
            "the `{}` profile declares no ui-rect trace event.",
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
                "cannot read a page size from {}. Pass --page-size WxH.",
                pdf.display()
            ))
        })?,
    };

    let mut spec = LaunchSpec::new(&exe, ctx.out("restyle-text.trace.txt"));
    spec.pdf = Some(pdf.clone());
    spec.env.push((
        ctx.profile.diag_env.0.to_owned(),
        ctx.profile.diag_env.1.to_owned(),
    ));
    spec.env
        .push((SHELL_DIAG_ENV.0.to_owned(), SHELL_DIAG_ENV.1.to_owned()));
    // ★ See [`INVOKE`]: without this the Properties panel is only on screen if
    // the machine happens to have persisted an arrangement containing it, and
    // on 2026-08-29 it had not.
    spec.env
        .push(("PDFCER_DIAG_INVOKE".to_owned(), INVOKE.to_owned()));
    spec.allow_stale = ctx.allow_stale;
    spec.source_root = ctx.source_root.clone();

    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    report.artifact(session.trace_path().to_path_buf());
    report.note(format!(
        "launched {} as pid {} with PDFCER_DIAG_INVOKE={INVOKE}",
        exe.display(),
        session.pid()
    ));
    session.settle(40);
    let driver = Driver::new(session.window());

    // --- 1: Edit mode -------------------------------------------------------
    click_mode_segment(&session, &driver, ui_rect, MODE)?;
    session.settle(20);

    // --- 2: sweep across the text ------------------------------------------
    let trace = session.trace()?;
    let mapping = CanvasMapping::from_trace(&trace, vocab, page, target.page)?;
    let frame = session.frame()?;
    let start =
        frame.to_screen(mapping.doc_to_window(DocPoint::new(target.page, target.x, target.y))?);
    // Along the baseline, same y — a diagonal drag would also select and would
    // make a failure ambiguous between "the sweep missed the line" and "the
    // panel did not follow".
    let end = frame.to_screen(mapping.doc_to_window(DocPoint::new(
        target.page,
        target.x + SWEEP_PT,
        target.y,
    ))?);
    // ★ Arm the text-sweep tool FIRST — see `VK_T`'s own note. Without this the
    // drag below is an object marquee and the check reports a working panel as
    // broken.
    driver.press(VK_T)?;
    session.settle(16);
    driver.drag(start, end)?;
    session.settle(24);

    let trace = session.trace()?;
    let swept = trace
        .events(SELECTION_EVENT)
        .last()
        .and_then(|l| l.get("chars"))
        .and_then(|n| n.parse::<usize>().ok())
        .unwrap_or(0);
    if swept == 0 {
        return Err(Error::new(format!(
            "the drag from (page {}, {:.1}, {:.1}) rightwards {SWEEP_PT} pt selected no text, \
             so there was nothing for the panel to describe. SKIPPED rather than failed: this \
             says the --doc-point is not on text, which is the harness's aim and not the \
             program's behaviour. Trace: {}.",
            target.page,
            target.x,
            target.y,
            session.trace_path().display()
        )));
    }
    report.note(format!("the sweep selected {swept} character(s)"));

    // --- 3: the section must be on screen ----------------------------------
    //
    // ★ Link 3, and the first thing with no other instrument. `properties.text`
    // is published with `ui_rect_visible`, so its absence means the section is
    // not merely un-drawn but not VISIBLE — which is the operator's own test.
    let trace = session.trace()?;
    if driving::declared(&trace, ui_rect, SECTION_REGION).is_none() {
        // Evidence before the verdict: a section that is not on screen is a
        // layout question, and a layout question has exactly one oracle.
        let shot = ctx.out("restyle_text.no-section.png");
        if crate::capture::window_to_png(&session, &shot).is_ok() {
            report.artifact(shot);
        }
        return Ok(Some(format!(
            "★ {swept} CHARACTER(S) ARE SELECTED AND THE PROPERTIES PANEL SAYS NOTHING ABOUT \
             THEM: no `{SECTION_REGION}` region.\n\
             ★★ **The panel being absent is no longer one of the candidates.** This check now \
             launches with `PDFCER_DIAG_INVOKE={INVOKE}`, so the panel is mounted and active \
             before anything else happens — that was the 2026-08-29 failure and it is closed. \
             Confirm it in the trace before reading further: a `dock.body.file.properties` \
             region says the panel drew. If it is missing, the finding is that \
             `file.properties` no longer mounts a panel, which is a defect in \
             `app::panels::show_panel` and not in this section.\n\
             Two candidates remain. (1) **The section's guard is wrong** — \
             `panels::properties::text::section` reads `doc.text_selection` and the staleness \
             gate inside `TextSelection::runs`, which answers an empty vec whenever \
             `selection.epoch != doc.edit_epoch`; an empty vec returns to `route` before \
             drawing. (2) **The run would not pin** — `TextStyleDraft::sync` answers `false` \
             when `textedit::pin::inspect` finds nothing or the `/BaseFont` join fails — in \
             which case the section should still draw its heading and one sentence and publish \
             this very region, so an absent region rules this out rather than in. The \
             screenshot beside this report shows which. Trace: {}.",
            session.trace_path().display()
        )));
    }
    report.note("★ the Properties panel drew a Text section for the swept text");

    // --- 4: find Bold, scrolling the panel the way an operator would --------
    let mut bold = None;
    for attempt in 0..SCROLL_ATTEMPTS {
        let trace = session.trace()?;
        if let Some(rect) = driving::declared(&trace, ui_rect, BOLD_REGION) {
            bold = Some(rect);
            if attempt > 0 {
                report.note(format!(
                    "Bold was below the panel's fold; {attempt} scroll notch(es) brought it \
                     into view — the Properties panel is a scroll area and its slot is shorter \
                     than its content"
                ));
            }
            break;
        }
        let Some(section) = driving::declared(&trace, ui_rect, SECTION_REGION) else {
            return Err(Error::new(format!(
                "the Text section stopped being visible while scrolling for Bold, so there is \
                 nothing left to aim at. Trace: {}.",
                session.trace_path().display()
            )));
        };
        driver.scroll_at(session.frame()?.declared_center(section), -1)?;
        session.settle(12);
    }
    let bold = bold.ok_or_else(|| {
        Error::new(format!(
            "no `{BOLD_REGION}` region after scrolling the Properties panel {SCROLL_ATTEMPTS} \
             times. SKIPPED rather than failed: a button that was never pressed proves nothing \
             about pressing it. Trace: {}.",
            session.trace_path().display()
        ))
    })?;

    // --- 5: press it -------------------------------------------------------
    let frame = session.frame()?;
    driver.click_at(frame.declared_center(bold))?;

    // ★★ WAIT FOR THE GESTURE, do not guess at it. `settle(30)` — 750 ms — was
    // the first shape here and it read the trace mid-restyle: the trace held
    // ELEVEN completed `format-text` lines and neither of this module's own
    // summary lines, so the check reported "Bold was pressed and nothing
    // happened" over a gesture that was still running.
    //
    // The cost is structural rather than accidental. A restyle re-resolves the
    // pin per run from a fresh provenance extraction, and an extraction is the
    // expensive thing this shell does. A sweep across a title-block label is a
    // dozen runs, so a dozen extractions. That is a real limit, measured here
    // and recorded in the module's own docs; it is not this check's business to
    // hide it behind a longer sleep, so the poll below REPORTS the elapsed time
    // whatever the verdict.
    let waited = wait_for_verdict(&session)?;
    report.note(format!(
        "the restyle took {waited} ms of wall clock — one provenance extraction per run"
    ));

    // --- 6: the verdict ----------------------------------------------------
    let trace = session.trace()?;
    if let Some(declined) = trace.events(DECLINED_EVENT).last() {
        return Ok(Some(format!(
            "Bold was pressed and the restyle DECLINED: `{}`.\n\
             That is the program answering rather than staying silent, so the chain works — but \
             the answer is worth reading. On a page with no bold face, synthesis should apply; \
             on a page with one, the retry should use it. A refusal means neither route took, \
             which is either an unpinnable run or the engine naming a real face that then \
             failed coverage (filed, 2026-08-27). Trace: {}.",
            declined.raw,
            session.trace_path().display()
        )));
    }
    let Some(applied) = trace.events(STYLE_EVENT).last() else {
        let shot = ctx.out("restyle_text.no-effect.png");
        if crate::capture::window_to_png(&session, &shot).is_ok() {
            report.artifact(shot);
        }
        return Ok(Some(format!(
            "★ BOLD WAS PRESSED AND NOTHING HAPPENED AND NOTHING WAS DECLINED: no \
             `{STYLE_EVENT}` line and no `{DECLINED_EVENT}` line.\n\
             Two candidates. (1) **The click missed the button** — the region was declared, so \
             this is a coordinate problem and the screenshot beside this report settles it. (2) \
             **The press raised no action**, which on a plain `ui.button` means the section \
             re-drew between the press and the read; the read-back `sync` runs every frame, and \
             a stamp recomputed too eagerly is exactly this symptom. The second is a real \
             defect and the operator would report it as 'the Bold button does nothing'. Trace: \
             {}.",
            session.trace_path().display()
        )));
    };
    report.note(format!("★ Bold committed a restyle: `{}`", applied.raw));

    let n: usize = applied
        .get("applied")
        .and_then(|v| v.parse().ok())
        .unwrap_or(0);
    if n == 0 {
        return Ok(Some(format!(
            "the restyle reported `applied=0`, so the module decided to act and every run it \
             tried refused without saying so. That is worse than a decline: `{}`. Trace: {}.",
            applied.raw,
            session.trace_path().display()
        )));
    }
    if trace.last(APPLIED).is_none() {
        return Ok(Some(format!(
            "Bold computed `{}` and no `{APPLIED}` line followed, so the action was raised and \
             its apply arm never ran. Nothing reached the document, which from a chair is \
             indistinguishable from the button doing nothing. Trace: {}.",
            applied.raw,
            session.trace_path().display()
        )));
    }
    report.note("★★ the swept text was restyled in the open document, through `format_text`");
    Ok(None)
}
