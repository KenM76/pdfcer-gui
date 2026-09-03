//! `text_markup_marks_a_selection` — the regression test for **a ribbon command
//! whose operand is not the pointer**.
//!
//! Every other driving check in this crate asserts that a control *arms* a tool:
//! [`crate::checks::markup_rectangle`] and [`crate::checks::measure_linear`]
//! both end at `…-tool tool=…`, and what the operator does next is a gesture.
//! Underline, Strikeout and Squiggly are the first commands in this shell that
//! **author an annotation on the press**, from the text selection already on the
//! document — which is Acrobat's model and is argued at
//! `canvas::markup::text`'s header §1.
//!
//! That makes this the first check whose subject is a *join between two
//! features*: the text-selection gesture (`canvas::textsel`, 2026-08-14) and the
//! markup authoring path (`EditSession::add_markup`). Neither half's tests can
//! see the join, and the join is the whole feature.
//!
//! # The five-link chain, and which links no unit test observes
//!
//! | # | Link | Where | Its own test |
//! |---|---|---|---|
//! | 1 | a sweep makes a text selection on the document | `canvas::textsel` + `canvas::interact` | partly — link 4 of [`crate::checks::text_selection`] |
//! | 2 | that selection publishes `selection.text` | `app::conditions` | yes |
//! | 3 | the condition **enables the control** | `egui_shell`'s `Enable::When` | yes, on each side |
//! | 4 | the click routes to a `TextMarkKind` and builds the action | `app::dispatch` via `shell::commands::text_mark_for_command` | yes |
//! | 5 | the action reaches `add_markup` and the engine authors it | `app::actions` | yes |
//! |   | **2→3 and 1→4 joined, in one running window** | — | **no** |
//!
//! Link 3 is the interesting one and it is asserted here in a way the workspace
//! cannot: **a greyed `egui` control does not emit
//! `ribbon-command-invoked`**. So a click that produces no invoke is positive
//! evidence that the control was disabled, and a click that does produce one is
//! positive evidence that it was enabled — from outside the process, with no
//! knowledge of the condition's name. That is a *better* oracle here than pixels
//! would be: a disabled control differs from an enabled one mostly in its
//! **text** colour, which is a few dozen antialiased pixels inside a fill that
//! does not change, and `MIN_PRESSED_DELTA` is a fill measurement. Using it here
//! would have measured the wrong thing and passed.
//!
//! # The four phases, and why the first is a negative
//!
//! | Phase | State | Action | Expected | If it does not hold |
//! |---|---|---|---|---|
//! | A | nothing selected | click Underline | **no** `ribbon-command-invoked` | FAIL — a control that can only act on a selection was live without one, which is what `RIBBON_IA.md` P3 forbids |
//! | B | — | sweep a band | `canvas-text-selection chars>0 quads=N` | SKIP — this band had no text; try the next |
//! | C | text selected | click Underline | invoke **and** `text-markup-commit quads=N` **and** `add-text-markup n=1` | FAIL, with the missing line naming the link |
//! | D | selection now stale | click Underline again | **no second** `add-text-markup` | FAIL — a stale selection authored a second annotation |
//!
//! Phase A is the half that would be easy to omit, and omitting it would leave
//! the check unable to distinguish *"the condition works"* from *"the control is
//! always live and the click happened to land after a sweep"*. It is also the
//! only assertion in the suite that a control is **correctly disabled**, which
//! is the direction P3 is usually violated in.
//!
//! Phase D is the one that documents a real, deliberate consequence rather than
//! a defect: authoring a markup is an edit, `vector_edit` bumps `edit_epoch`,
//! and `canvas::textsel`'s §7 staleness rule therefore retires the selection
//! that authored it. Acrobat keeps its selection across a markup and this does
//! not. The check pins the behaviour so that a future change to the staleness
//! rule is a decision rather than an accident.
//!
//! # ★ The assertion that spans the process boundary
//!
//! `canvas-text-selection … quads=N` and `text-markup-commit … quads=N` are
//! written by two different modules about two different values — the boxes the
//! **wash** was painted from, and the boxes the `/QuadPoints` was **authored**
//! from. `canvas::textsel` §5.1 claims they are the same list from one pass.
//! Comparing the two numbers is the only way to test that claim from outside the
//! process, and a build that re-derived the authoring quads from anything else
//! would have to reproduce the line grouping exactly to pass it.
//!
//! # Mouse only
//!
//! Every gesture here is a real `SetCursorPos` + `mouse_event`, and **nothing in
//! this check needs a key**
//!
//! ★ **CORRECTED 2026-08-18.** These headers used to say synthetic keyboard
//! input does not reach the target window on this machine. It DOES — see
//! [`crate::checks::add_text`], which types real characters into a caret
//! draft and asserts they landed. The belief came from `Ctrl+E` producing no
//! trace, which was the dead-keymap defect (fourteen of twenty-one declared
//! chords were dispatched by nothing) misread as a property of the machine —
//! and while it stood nobody drove a chord, so nothing could contradict it.
//!
//! Continuing: — which is itself a consequence of the interaction
//! model chosen: select-then-press is two clicks and a drag.
//!
//! # Every way this reports SKIP, and why none of them is a pass
//!
//! * no binary, no `--pdf`, `--no-input` — the harness never began;
//! * the diagnostic switches did not reach the process;
//! * the page size could not be read and no `--page-size` was given;
//! * the mode segment, the Markup tab, or the Text markup group's controls were
//!   never declared;
//! * the canvas is not showing page 1, so the harness's one known page size does
//!   not describe the page it would be sweeping;
//! * **no band had text under it** — phase B never succeeded, so there is no
//!   selection for phase C to mark and phase A's silence would prove nothing.

use crate::checks::driving::{
    self, INVOKE_EVENT, ITEM_PREFIX, SHELL_DIAG_ENV, TAB_EVENT, UNIMPLEMENTED_EVENT, declared,
    declared_names, list, list_str, shell_trace,
};
use crate::checks::text_selection::{BANDS, aim};
use crate::checks::{Check, CheckContext};
use crate::coords::{DocPoint, PageGeometry};
use crate::error::{Error, Result};
use crate::input::Driver;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;
use crate::trace::Trace;

/// ★ **The only mode in which this feature exists**, and the check is aimed at
/// it deliberately rather than for convenience.
///
/// Marking text needs two things that do not overlap the way anyone expects:
/// the ability to *select* text, which `canvas::textsel::takes_the_press`
/// grants where the mode cannot select content (Read and Review), and
/// `author_markup`, which the mode's tab list grants where it contains
/// `markup` (Review and Edit). The intersection is **Review alone** — Read has
/// no Markup tab, and in Edit the primary button is the content marquee so no
/// text selection can be made and the three controls are permanently greyed.
///
/// See `canvas::markup::text` §2. That inversion is a known gap with a known
/// fix (`CanvasTool::Text`), and it is the reason this check does not carry an
/// Edit control phase the way [`crate::checks::text_selection`] does: in Edit
/// the controls are correctly dead, and a phase asserting so would be asserting
/// the gap rather than the feature.
const MODE: &str = "review";

/// The tab that carries the Text markup group.
const TAB: &str = "ribbon.tab.markup";

/// The tab id the shell reports for [`TAB`].
const TAB_ID: &str = "markup";

/// **The control under test.**
///
/// Underline rather than Strikeout or Squiggly because it is the one an
/// operator reaches for first and because the three are the same code path with
/// one `match` arm between them — `shell::commands::text_mark_command` maps all
/// three and the mapping's own test walks `TextMarkKind::ALL`. Driving all three
/// here would cost three more clicks and three more edits to prove what that
/// test already proves, while the *join* this check exists for is per-command
/// only in its id.
const SUBJECT: &str = "ribbon.item.markup.underline";

/// The command id of [`SUBJECT`], as dispatch and the shell spell it.
const SUBJECT_ID: &str = "markup.underline";

/// The `Debug` spelling of the kind [`SUBJECT_ID`] names, as the application
/// traces it.
const SUBJECT_KIND: &str = "Underline";

/// The sibling named only in notes and SKIP reasons — the control that shares
/// the band and is **not** clicked.
///
/// Not a pixel differential here (see the module header on why the invoke is
/// the better oracle for enablement), but its presence is still worth
/// asserting: a build that registered one of the three and not the others would
/// otherwise pass this check completely.
const SIBLING: &str = "ribbon.item.markup.strikeout";

/// The third of the family, likewise checked for presence only.
const THIRD: &str = "ribbon.item.markup.squiggly";

/// `canvas-text-selection via=… page=… chars=… quads=…` — the selection that
/// will be the operand.
const TEXT_EVENT: &str = "canvas-text-selection";

/// `text-markup-commit kind=… page=… quads=…` — `canvas::markup::text`'s report
/// that a command turned a selection into an action.
const COMMIT_EVENT: &str = "text-markup-commit";

/// `text-markup-declined kind=… reason=…` — the same module's refusal line.
///
/// Read to *improve failure messages*: `reason=Stale` and `reason=NoSelection`
/// send a reader to two different places, and both are different again from a
/// command that never reached dispatch at all.
const DECLINE_EVENT: &str = "text-markup-declined";

/// `add-text-markup page=… n=… epoch=… disclosures=…` — `app::actions`'
/// `vector_edit` reporting that the **engine** authored the annotation.
///
/// The line that makes this check about a document rather than about an intent.
/// [`COMMIT_EVENT`] says the shell decided to author one; this says
/// `EditSession::add_markup` returned `Ok` and the revision moved.
const APPLY_EVENT: &str = "add-text-markup";

/// How many characters the selection carries. `> 0` gates phase C.
const CHARS_FIELD: &str = "chars";

/// How many line boxes. Compared **across** the two events — see the module
/// header's boundary-spanning assertion.
const QUADS_FIELD: &str = "quads";

/// See the module documentation.
pub struct TextMarkupMarksASelection;

impl Check for TextMarkupMarksASelection {
    fn name(&self) -> &'static str {
        "text_markup_marks_a_selection"
    }

    fn defect(&self) -> &'static str {
        "Markup ▸ Text markup ▸ Underline is live with nothing selected, does nothing when text \
         IS selected, authors an annotation over different boxes from the ones highlighted, or \
         marks a selection that a later edit has invalidated — the join between the \
         text-selection gesture and the markup authoring path, which neither feature's tests can \
         observe"
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

/// Every `canvas-text-selection` line reporting a **non-empty** selection.
///
/// Filtered on `chars > 0` for the reason [`crate::checks::text_selection`]
/// records: a *clear* is traced too, with `chars=0`, and a count of the event
/// would be satisfied by the gesture that ends a selection.
fn selections(trace: &Trace) -> Vec<&crate::trace::TraceLine> {
    trace
        .events(TEXT_EVENT)
        .filter(|l| l.get_usize(CHARS_FIELD).unwrap_or(0) > 0)
        .collect()
}

/// How many times the shell has reported [`SUBJECT_ID`] invoked.
///
/// A **count**, not a presence, and for the reason
/// `driving::click_mode_segment` counts its mode events: this check clicks the
/// same control three times, and "has it ever been invoked?" would be answered
/// `true` by a click made ten seconds earlier.
fn invokes(session: &Session) -> Result<usize> {
    Ok(shell_trace(session)?
        .events(INVOKE_EVENT)
        .filter(|l| l.get("id") == Some(SUBJECT_ID))
        .count())
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
            "no --pdf. This check sweeps text on a page and then marks it, so it needs a \
             document with readable text on its first page. With nothing open the control is \
             correctly greyed and the check would be measuring the gate rather than the feature.",
        )
    })?;
    if !ctx.allow_input {
        return Err(Error::new(
            "input is disabled (--no-input). This check is three clicks and a drag on a real \
             canvas. Reported as SKIPPED rather than passed: a check that did not run has \
             learned nothing.",
        ));
    }
    let ui_rect = ctx.profile.vocab.ui_rect_event.ok_or_else(|| {
        Error::new(format!(
            "the `{}` profile declares no ui-rect trace event, so the application cannot state \
             where its controls are and this check has nothing to aim at.",
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
    let mut spec = LaunchSpec::new(&exe, ctx.out("text_markup.trace.txt"));
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

    // --- step 1: Review, which is the only mode this feature exists in ------
    driving::click_mode_segment(&session, &driver, ui_rect, MODE)?;
    report.note(format!(
        "the {MODE} segment reported the click — the one mode that both selects text and authors \
         markup (see this check's MODE constant)"
    ));

    // --- step 2: the Markup tab --------------------------------------------
    let frame = session.frame()?;
    let trace = session.trace()?;
    let tab = declared(&trace, ui_rect, TAB).ok_or_else(|| {
        Error::new(format!(
            "the application declared no `{TAB}` region after switching to {MODE}. Either this \
             build has no Markup tab, or the tab strip is too narrow and the tab has moved into \
             its overflow menu — which this check cannot open, because the menu's contents are \
             not published as regions. Tabs declared: {}.",
            list(&declared_names(&trace, ui_rect, "ribbon.tab."))
        ))
    })?;
    driver.click_at(frame.declared_center(tab))?;
    session.settle(12);
    if !shell_trace(&session)?
        .events(TAB_EVENT)
        .any(|l| l.get("tab") == Some(TAB_ID))
    {
        return Err(Error::new(format!(
            "the click on `{TAB}` produced no `{TAB_EVENT} tab={TAB_ID}` line. The {MODE} click \
             DID land, so pointer input works and this is not the input channel; the likely \
             cause is that the tab moved between the frame that declared its rect and the frame \
             that received the click."
        )));
    }
    report.note("the Markup tab reported the click");

    // --- step 3: locate the control, and confirm its two siblings exist -----
    let trace = session.trace()?;
    let items = declared_names(&trace, ui_rect, ITEM_PREFIX);
    let subject = declared(&trace, ui_rect, SUBJECT).ok_or_else(|| {
        Error::new(format!(
            "the Markup tab is active and its controls publish their rects, but none of them is \
             `{SUBJECT}`. This build has no Underline control. Controls declared: {}.",
            list(&items)
        ))
    })?;
    for name in [SIBLING, THIRD] {
        if declared(&trace, ui_rect, name).is_none() {
            return Err(Error::new(format!(
                "`{SUBJECT}` is declared and `{name}` is not. The three text-markup kinds are one \
                 code path with one `match` arm between them, so a build carrying one of them and \
                 not the others is a registration that was half done — and this check drives only \
                 Underline, so it would pass without noticing. Controls declared: {}.",
                list(&items)
            )));
        }
    }
    if !subject.is_substantial() {
        return Err(Error::new(format!(
            "`{SUBJECT}` was declared at {subject:?}, which has no usable area. A click aimed at \
             a degenerate rectangle proves nothing."
        )));
    }
    report.note(format!(
        "{SUBJECT} at {subject:?}, with {SIBLING} and {THIRD} beside it"
    ));

    // ==================================================================
    // PHASE A — the control must be DEAD with nothing selected
    // ==================================================================
    //
    // The only assertion in this suite that a control is *correctly disabled*,
    // and the reason it can be made from outside the process is that a greyed
    // `egui` control never reports itself invoked. See the module header on why
    // this is a better oracle here than a pixel differential.
    let before = invokes(&session)?;
    driver.click_at(frame.declared_center(subject))?;
    session.settle(12);
    let after = invokes(&session)?;
    if after > before {
        return Ok(Some(format!(
            "P3: THE CONTROL IS LIVE WITH NOTHING SELECTED. Nothing has been swept on the page \
             this run, so there is no text selection — and clicking `{SUBJECT}` produced \
             `{INVOKE_EVENT} id={SUBJECT_ID}` anyway, which a disabled control cannot do. \
             `RIBBON_IA.md` P3 forbids a control that is always live and does nothing on almost \
             every press; the command is registered `enabled_when(\"selection.text\")` and \
             `app::conditions` publishes that name only for a **live** text selection on the \
             open document. So either the registration lost its predicate, or the condition is \
             being published unconditionally. {}",
            match session.trace()?.last(DECLINE_EVENT) {
                Some(line) => format!(
                    "The application then declined it — `{}` — which is the belt working and the \
                     braces missing: nothing was authored, and the operator still pressed a live \
                     button that did nothing.",
                    line.raw
                ),
                None => format!(
                    "No `{DECLINE_EVENT}` either, so the press may have authored something: check \
                     the trace for `{APPLY_EVENT}`."
                ),
            }
        )));
    }
    report.note(format!(
        "with nothing selected, a click on {SUBJECT} produced no `{INVOKE_EVENT}` — the control \
         is greyed, which is what P3 requires of a command that can only act on a selection"
    ));

    // ==================================================================
    // PHASE B — make the operand
    // ==================================================================
    let mut unreachable: Vec<String> = Vec::new();
    let mut found: Option<(usize, usize, String)> = None;

    for (n, (start, end)) in BANDS.iter().enumerate() {
        let from = DocPoint::new(0, start.0 * page.width_pt, start.1 * page.height_pt);
        let to = DocPoint::new(0, end.0 * page.width_pt, end.1 * page.height_pt);
        let (from, to) = match (aim(ctx, &session, page, from), aim(ctx, &session, page, to)) {
            (Ok(a), Ok(b)) => (a, b),
            (Err(e), _) | (_, Err(e)) => {
                unreachable.push(format!("band {}: {}", n + 1, e.message()));
                continue;
            }
        };
        let before = selections(&session.trace()?).len();
        driver.drag(from, to)?;
        session.settle(16);
        let after = session.trace()?;
        let lines = selections(&after);
        // The **last** new line: a sweep traces every distinct state it passes
        // through, and the settled one is the selection the operator is left
        // holding — which is the one the next click will mark. See
        // `text_selection`'s own note on this.
        if let Some(line) = lines.last().filter(|_| lines.len() > before) {
            let quads = line.get_usize(QUADS_FIELD).unwrap_or(0);
            if quads == 0 {
                return Ok(Some(format!(
                    "THE SELECTION HAS NO BOXES. Band {} traced `{}` — text was selected and no \
                     line boxes were produced. There is therefore nothing to author a \
                     `/QuadPoints` from, and `canvas::textsel`'s one-derivation promise (its \
                     header section 5.1) has failed on the half this feature consumes.",
                    n + 1,
                    line.raw
                )));
            }
            report.note(format!(
                "band {}: the sweep traced `{}` — there is now a selection to mark",
                n + 1,
                line.raw
            ));
            found = Some((n + 1, quads, line.raw.clone()));
            break;
        }
        report.note(format!(
            "band {}: no text under the sweep; trying the next",
            n + 1
        ));
    }

    let Some((band, selection_quads, selection_line)) = found else {
        return Err(Error::new(format!(
            "no band had text under it: {} sweeps were performed in `{MODE}` and none of them \
             selected a character. This check declines to call that a pass — with no selection \
             established, phase A's silence proves nothing (a control greyed for want of a \
             selection and a control greyed because the feature is missing look identical). \
             {}Trace: {}.",
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

    // ==================================================================
    // PHASE C — mark it
    // ==================================================================
    let invokes_before = invokes(&session)?;
    let applies_before = session.trace()?.events(APPLY_EVENT).count();
    driver.click_at(frame.declared_center(subject))?;
    session.settle(24);

    if invokes(&session)? <= invokes_before {
        // Bound to a `let` before it is borrowed: `shell_trace` builds a whole
        // `Trace`, and collecting `&str` out of a temporary would leave the
        // strings pointing at a value dropped at the end of the statement.
        let shell = shell_trace(&session)?;
        let seen: Vec<&str> = shell
            .events(INVOKE_EVENT)
            .filter_map(|l| l.get("id"))
            .collect();
        return Ok(Some(format!(
            "THE CONTROL DID NOT COME ALIVE. Band {band} was swept and the application traced \
             `{selection_line}`, so a live text selection exists — and clicking `{SUBJECT}` \
             produced no `{INVOKE_EVENT} id={SUBJECT_ID}`, which means the control is still \
             disabled. The chain is `canvas::interact` storing the selection → \
             `app::conditions` publishing `selection.text` from a **live** one → the command's \
             `enabled_when`. The middle link is the one with no test that observes it in a \
             window: look first at whether the condition asks `live(doc.edit_epoch)` against the \
             same epoch the selection was stamped with. Commands the shell reported invoked this \
             run: {}.",
            list_str(&seen)
        )));
    }
    report.note(format!(
        "with text selected, the same control reported `{INVOKE_EVENT} id={SUBJECT_ID}` — so the \
         selection is what enabled it"
    ));

    let trace = session.trace()?;
    // `.last()` rather than `.next_back()`: `Trace::events` returns an opaque
    // `impl Iterator`, not a `DoubleEndedIterator`, so walking to the end is the
    // only way to take the most recent line — and the most recent is what this
    // check is about, because the control has been clicked more than once.
    let Some(commit) = trace
        .events(COMMIT_EVENT)
        .filter(|l| l.get("kind") == Some(SUBJECT_KIND))
        .last()
    else {
        let declined = trace.last(DECLINE_EVENT);
        let unimplemented = trace
            .events(UNIMPLEMENTED_EVENT)
            .any(|l| l.get("id") == Some(SUBJECT_ID));
        return Ok(Some(format!(
            "THE CLICK REACHED THE CONTROL AND AUTHORED NOTHING. The shell traced \
             `{INVOKE_EVENT} id={SUBJECT_ID}`, so the command was invoked and its token handed to \
             the application — and there is no `{COMMIT_EVENT} kind={SUBJECT_KIND}`. {} Look at \
             `app/dispatch.rs`'s guard arm matching \
             `shell::commands::text_mark_for_command(id).is_some()`, and at that function itself: \
             it is the single binding between the id and a `TextMarkKind`, and it must not \
             overlap `markup_for_command`, whose arm would otherwise swallow this command whole.",
            match (declined, unimplemented) {
                (Some(line), _) => format!(
                    "The application DID trace `{}`, so the arm ran and refused: that names the \
                     rule that rejected the selection rather than the routing.",
                    line.raw
                ),
                (None, true) => format!(
                    "The application traced `{UNIMPLEMENTED_EVENT} id={SUBJECT_ID}`, which is \
                     `dispatch_command`'s fall-through: the command arrived and dispatch had no \
                     arm for it."
                ),
                (None, false) => format!(
                    "No `{DECLINE_EVENT}` and no `{UNIMPLEMENTED_EVENT}` either, so the command \
                     did not reach `dispatch_command`'s fall-through — check `dispatch_token`'s \
                     token-to-id lookup."
                ),
            }
        )));
    };
    report.note(format!("the application traced `{}`", commit.raw));

    // ★ The boundary-spanning assertion — see the module header.
    let commit_quads = commit.get_usize(QUADS_FIELD).unwrap_or(0);
    if commit_quads != selection_quads {
        return Ok(Some(format!(
            "THE MARK AND THE WASH DESCRIBE DIFFERENT BOXES. The selection was traced with \
             `{QUADS_FIELD}={selection_quads}` — `{selection_line}` — and the annotation was \
             authored from `{QUADS_FIELD}={commit_quads}`: `{}`. `canvas::textsel` section 5.1 \
             claims both lists are the same accumulation from one walk over the same glyphs, kept \
             in two spaces and pushed in one iteration. A mismatch means something re-derived the \
             authoring quads — and the operator would be marking glyphs they never saw \
             highlighted, which is only discoverable after saving.",
            commit.raw
        )));
    }
    report.note(format!(
        "the wash and the mark agree on {selection_quads} box(es) — the one-derivation promise, \
         asserted across two modules and two trace lines"
    ));

    let applies: Vec<&crate::trace::TraceLine> = trace.events(APPLY_EVENT).collect();
    if applies.len() <= applies_before {
        return Ok(Some(format!(
            "THE ENGINE NEVER AUTHORED IT. The application decided to author an annotation — \
             `{}` — and no `{APPLY_EVENT}` line followed, so `app::actions`' apply arm never ran \
             or `EditSession::add_markup` refused. The action funnel is the suspect: a \
             `CommitTextMarkup` raised into `actions` is applied after the frame by \
             `PdfcerApp::apply`, and an arm that is missing there compiles (the `match` would not) \
             — but a `vector_edit` that returned `Err` traces `add-text-markup-refused` instead, \
             so check for that line first.",
            commit.raw
        )));
    }
    let applied = applies.last().map_or("", |l| l.raw.as_str());
    report.note(format!(
        "the engine authored it: `{applied}` — one annotation, one undo entry, the revision moved"
    ));

    // ==================================================================
    // PHASE D — the selection is spent, and that is deliberate
    // ==================================================================
    //
    // Authoring is an edit; an edit bumps `edit_epoch`; a selection stamped
    // with the old one is stale (`canvas::textsel` section 7). So the control
    // greys itself and a second press must author nothing. Pinned here because
    // it is a behaviour difference from Acrobat — which keeps its selection —
    // and a change to it should be a decision rather than a side effect.
    let applies_before = applies.len();
    driver.click_at(frame.declared_center(subject))?;
    session.settle(16);
    let trace = session.trace()?;
    if trace.events(APPLY_EVENT).count() > applies_before {
        return Ok(Some(format!(
            "A STALE SELECTION AUTHORED A SECOND ANNOTATION. The first press marked the \
             selection and bumped the document's revision, which makes that selection stale — its \
             recorded boxes were resolved against the previous one and may now be over different \
             glyphs. A second press produced another `{APPLY_EVENT}`, so either \
             `app::conditions` is publishing `selection.text` for a selection that is not live, \
             or `canvas::markup::text::mark` stopped asking. `canvas::textsel` section 7 calls \
             painting stale geometry the one thing rule 4 forbids outright; authoring it into the \
             file is worse, because it survives the frame."
        )));
    }
    report.note(
        "a second press on the same control authored nothing: marking is an edit, the edit \
         retires the selection that authored it, and the control greys itself. That is a \
         deliberate difference from Acrobat (which keeps the selection) and is recorded at \
         `canvas::textsel` section 7",
    );

    // --- the picture, saved as evidence rather than asserted on -------------
    //
    // The annotation IS visible — an underline is a red line where there was
    // none — so unlike the selection wash this could in principle be a pixel
    // oracle. It is not one, for the reason `crate::capture`'s rule gives about
    // what evidence is for: the quantity that matters is *where* the line
    // landed, and the harness has no independent account of where the glyphs
    // are. Asserting "some pixels changed near the band" would pass for a mark
    // in the wrong place, which is the failure this feature can actually
    // produce.
    let shot = ctx.out("text_markup.png");
    match crate::capture::window_to_png(&session, &shot) {
        Ok(_) => {
            report.artifact(shot);
            report.note(
                "the window with the authored markup on it is saved beside the trace. It is \
                 evidence rather than the oracle: what a reader needs from it is whether the line \
                 landed under the words that were swept, and the harness has no independent \
                 account of where those glyphs are",
            );
        }
        Err(e) => {
            report.note(format!(
                "could not capture the window ({e}); every assertion above still stands, and they \
                 are what this check's verdict rests on"
            ));
        }
    }

    report.note(format!(
        "verdict established on band {band}: greyed with nothing selected, live with a selection, \
         authored one annotation over the boxes that were highlighted, and greyed again \
         afterwards"
    ));
    report.note(format!(
        "not covered here: Strikeout and Squiggly are not driven — they are the same dispatch arm \
         with one `match` arm between them, and `shell::commands::mapping` walks all three. Both \
         controls' presence IS asserted above, so a half-done registration still fails. Nor is \
         `{MODE}` compared against Edit: in Edit these controls are correctly dead for want of a \
         text selection, and a phase asserting that would be pinning the known gap rather than \
         the feature (see this check's MODE constant)"
    ));
    Ok(None)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The names this check greps for are the ones `egui-shell` builds, and the
    /// ids are the ones the application registers.
    ///
    /// Pinned here for the reason [`crate::checks::markup_rectangle`]'s twin
    /// test states: the two crates are joined by a **string** and nothing else,
    /// so a rename would leave both sides compiling while every assertion here
    /// quietly stopped matching — and a check that matches nothing passes
    /// vacuously.
    #[test]
    fn the_selectors_match_the_shells_own_spelling() {
        assert_eq!(SUBJECT, format!("ribbon.item.{SUBJECT_ID}"));
        assert_eq!(TAB, format!("ribbon.tab.{TAB_ID}"));
        for name in [SUBJECT, SIBLING, THIRD] {
            assert!(name.starts_with(ITEM_PREFIX), "{name}");
        }
        assert_ne!(SUBJECT, SIBLING);
        assert_ne!(SUBJECT, THIRD);
        assert_ne!(SIBLING, THIRD);
        // ★ Review, not Read and not Edit. The whole feature exists in exactly
        // one mode and this constant is where that finding is enforced: a
        // check aimed at Read would find no Markup tab, and one aimed at Edit
        // would find three permanently greyed controls and report phase C as a
        // failure of the feature rather than of the mode.
        assert_eq!(MODE, "review");
    }

    /// ★ **The quad counts really are compared, and a mismatch is visible to
    /// the parser** — the boundary-spanning assertion, tested on synthetic
    /// lines so that its arithmetic cannot be wrong in the one run that matters.
    #[test]
    fn the_two_quad_counts_are_read_from_their_own_lines() {
        let trace = Trace::parse(
            "pdfcer-diag canvas-text-selection via=drag page=0 chars=27 quads=2\n\
             pdfcer-diag text-markup-commit kind=Underline page=0 quads=2\n\
             pdfcer-diag add-text-markup page=0 n=1 epoch=1 disclosures=none",
            "pdfcer-diag",
        );
        let selection = selections(&trace);
        assert_eq!(selection.len(), 1);
        assert_eq!(selection[0].get_usize(QUADS_FIELD), Some(2));
        let commit = trace
            .events(COMMIT_EVENT)
            .filter(|l| l.get("kind") == Some(SUBJECT_KIND))
            .last()
            .expect("the commit line");
        assert_eq!(commit.get_usize(QUADS_FIELD), Some(2));
        assert_eq!(trace.events(APPLY_EVENT).count(), 1);

        // …and a build whose authoring quads came from somewhere else shows up
        // as two different numbers on two lines, which is the whole assertion.
        let diverged = Trace::parse(
            "pdfcer-diag canvas-text-selection via=drag page=0 chars=27 quads=2\n\
             pdfcer-diag text-markup-commit kind=Underline page=0 quads=27",
            "pdfcer-diag",
        );
        assert_ne!(
            selections(&diverged)[0].get_usize(QUADS_FIELD),
            diverged
                .events(COMMIT_EVENT)
                .last()
                .and_then(|l| l.get_usize(QUADS_FIELD))
        );
    }

    /// A cleared selection is not a selection — the filter phase B's ladder
    /// depends on, and the same one `text_selection` documents.
    #[test]
    fn only_a_non_empty_selection_counts() {
        let trace = Trace::parse(
            "pdfcer-diag canvas-text-selection via=clear page=0 chars=0 quads=0\n\
             pdfcer-diag canvas-text-selection via=drag page=0 chars=27 quads=2",
            "pdfcer-diag",
        );
        assert_eq!(selections(&trace).len(), 1, "the clear must not be counted");
    }
}
