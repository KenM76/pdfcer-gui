//! `measure_linear_places_a_dimension` — the regression test for **a feature
//! whose every link has a passing unit test and whose end-to-end effect on the
//! document nobody had ever observed**.
//!
//! # The defect class this exists for
//!
//! `SALVAGE.md`'s salvage procedure has six steps, and step 5 is the one this
//! file discharges:
//!
//! > **Assert it in `ui-verify` before calling it done (R1). A green unit test
//! > is the floor.**
//!
//! The measure salvage landed with 36 carried tests, all green, including two
//! that prove a canvas-authored `DimensionKind` is byte-for-byte the one
//! `pdfcer dimension-add` builds. Every one of them runs without a window.
//! None of them can state that a ribbon click arms the tool, that a click on
//! the page becomes a pick, that the third pick raises an action, or that the
//! action reaches the engine — because each of those is a property of a **call
//! site**, and a call site's effect is observable only in a running process.
//!
//! `SALVAGE.md`'s own correction of 2026-08-14 is the cautionary half of the
//! same story: five documents said the two-line gesture "has no caller" while
//! it had had one for two days, because *a status word in a table is a claim,
//! and it decays*. This check is the opposite of a status word. It is six
//! links, driven in order, through the operating system:
//!
//! | # | Link | Where | Its own test |
//! |---|---|---|---|
//! | 1 | the ribbon click reports the command | `egui-shell`'s `band::render_command` | yes |
//! | 2 | dispatch routes the id to a `MeasureKind` | `app/dispatch.rs`, via `shell::commands::measure_for_command` | yes |
//! | 3 | the kind arms the canvas tool | `canvas::tool::arm_measure` | yes |
//! | 4 | the armed tool renders the control **pressed** | `app/conditions.rs` publishing `selected:measure.linear`, read by `band::render_command` | yes |
//! | 5 | a canvas click becomes a **pick** | `canvas::interact`'s `Click` arm → `canvas::measure::click` | yes |
//! | 6 | the third pick's action is **accepted by the engine** | `app/actions.rs`'s `CommitDimension` arm → `vector_edit` → `EditSession::add_dimension` | yes |
//!
//! Six passing tests, six joins, and no test anywhere observes two adjacent
//! links being connected.
//!
//! # ★ Link 6 is the assertion this check exists for, and it is not link 5
//!
//! A `measure-pick … committed=true` line proves the **shell** raised
//! `Action::CommitDimension`. It says nothing whatever about whether the
//! engine accepted it. `app/actions.rs`'s `vector_edit` is explicit about the
//! difference, because it has two exits and only one of them changes the
//! document:
//!
//! ```text
//! Ok  → doc.edit_epoch += 1; doc.page_texture = None
//!       pdfcer-diag add-dimension page=P n=1 epoch=E disclosures=…
//! Err → the document is left ALONE
//!       pdfcer-diag add-dimension-refused page=P n=1 detail=…
//! ```
//!
//! A build in which `add_dimension` refuses every kind — a bad group id, a
//! degenerate length, a borrowed session — emits an **identical**
//! `measure-pick committed=true` and places nothing. That is precisely the
//! class of defect this harness exists for, so the check asserts on
//! `add-dimension` (the success event, carrying the bumped `epoch=`) and reads
//! `add-dimension-refused` only to say *why* in a failure message.
//!
//! The epoch is the honest signal for a second reason: it is the same counter
//! `canvas::interact` re-resolves the selection against and
//! `render::worker`'s key rebuilds the raster on, so a bumped epoch is not a
//! label the edit path prints about itself — it is the value the rest of the
//! application reacts to.
//!
//! # What it does, through the operating system
//!
//! Mouse only, and nothing below needs a key:
//!
//! ★ **CORRECTED 2026-08-18.** These headers used to say synthetic keyboard
//! input does not reach the target window on this machine. It DOES — see
//! [`crate::checks::add_text`], which types real characters into a caret
//! draft and asserts they landed. The belief came from `Ctrl+E` producing no
//! trace, which was the dead-keymap defect (fourteen of twenty-one declared
//! chords were dispatched by nothing) misread as a property of the machine —
//! and while it stood nobody drove a chord, so nothing could contradict it.
//!
//! To restate the reason this check is mouse-only: a linear dimension is three
//! clicks, and *that is the feature*.
//!
//! 1. Click the **Review** mode segment. Measure is in Review's and Edit's tab
//!    lists and not in Read's, and Read is the default, so without this step
//!    there is no Measure tab to activate.
//! 2. Click the **Measure** tab.
//! 3. Capture the window — the *before* picture.
//! 4. Click **Linear** in the Dimension group.
//! 5. Capture the window again — the *after* picture.
//! 6. Click **three points on the page**: A, B, and where the dimension sits —
//!    clicking a second time at the same point whenever the application says
//!    the pick it found needs confirming (see the rule-4 section below).
//!
//! # ★ The assertions, split by oracle
//!
//! ## Trace evidence — that the arm happened
//!
//! | Assertion | Line | What its absence means |
//! |---|---|---|
//! | the click reached the control | `ribbon-command-invoked id=measure.linear` | the click missed, or the control is disabled |
//! | the tool was armed | `measure-tool tool=Measure(Linear)` | **link 2 or 3** |
//!
//! The second is genuinely necessary: an armed measure tool is invisible from
//! outside the process. A crosshair is a cursor, and a screenshot of an armed
//! canvas and an unarmed one are the same picture — `HANDOFF.md`'s defect 8
//! exactly, the grid that was a wash, found by printing the ladder the running
//! program had chosen rather than by looking at it. `canvas::measure`'s own
//! comment above the `measure-pick` line makes the same point about picks:
//! *"a first pick and a second are the same screenshot."*
//!
//! ## Pixel evidence — that the control renders pressed
//!
//! A trace line is written by the code under test, about itself. `arm_measure`
//! traces unconditionally the moment it is called, so `measure-tool` proves
//! links 2 and 3 and says nothing about link 4: a build whose ribbon never
//! renders a pressed state emits an identical trace and looks identical to a
//! reader of that trace. So the pressed state is asserted from the captured
//! window, three ways, exactly as [`crate::checks::markup_rectangle`] does:
//!
//! | # | Comparison | What it rules out |
//! |---|---|---|
//! | P1 | Linear after ≠ Linear before | the control never changed |
//! | P2 | **Linear after ≠ Two-line after**, in one capture | a *global* repaint — a theme change, a hover, a resize — masquerading as a pressed state |
//! | P3 | Two-line after = Two-line before | the whole band changing, i.e. P1 passing for a reason unrelated to the click |
//!
//! P2 is the load-bearing one: a differential inside a single frame, which
//! nothing that happens to *both* controls can satisfy.
//!
//! ## Gesture evidence — that three picks are taken, and the third commits
//!
//! ```text
//! pdfcer-diag measure-pick kind=Linear in_progress=true  committed=false   ← A
//! pdfcer-diag measure-pick kind=Linear in_progress=true  committed=false   ← B
//! pdfcer-diag measure-pick kind=Linear in_progress=false committed=true    ← where
//! ```
//!
//! The shape of that sequence *is* the feature. `canvas::measure::click`'s
//! header states the rule it encodes — **the third pick is the commit, and
//! there is no accept box** — and records why: decision 024 and
//! `shell-redesign.md` §2.4 exist because the operator disliked *"a separate
//! accept / reject box somewhere on the screen"*, and `MODES_AND_PANELS.md`
//! now makes application-initiated floating surfaces default to Never. A build
//! that quietly restored the old two-click commit would emit `committed=true`
//! on the **second** line, and this check is what would say so.
//!
//! `committed=false` on the first two is asserted as strictly as
//! `committed=true` on the third, and it is the half that catches the reverse
//! regression: a tool that committed a zero-length dimension on pick A would
//! otherwise satisfy "a dimension was placed" perfectly.
//!
//! ### ★ A pick is not always one click, and that is rule 4 rather than a
//! wobble
//!
//! Snapping landed after this check was first written, and it changes the
//! click-to-pick arithmetic in a way that has to be modelled rather than
//! papered over. `canvas::measure::snapped` resolves every pick through
//! `pdfcer_core::vector::snap::snap_candidates`, and when the winning candidate
//! is **derived** — a centreline pdfcer *inferred* rather than one the file
//! states — `MeasureState::resolve_click` refuses to commit it on the click
//! that found it:
//!
//! ```text
//! pdfcer-diag measure-pick outcome=Promoted reason=derived-candidate-needs-confirm
//! ```
//!
//! That is `pdfce_FeatureRequests/README.md` rule 4's fuzzy-never-sneaky gate:
//! an inference is announced before it is acted on, and a second click on the
//! same point confirms it. It is deliberate, it is what
//! `canvas::snap::snap_commit_clicks` exists to encode, and a check that
//! treated it as a failure would be filing a defect against the feature.
//!
//! So a pick is **one or two clicks**, and this check clicks again when it is
//! told to — modelling the operator, who does exactly that. Three other
//! responses were available and each was rejected:
//!
//! | Response | Why not |
//! |---|---|
//! | aim the picks at open paper so no candidate is found | lucky rather than honest: nothing about a fixture guarantees a point is far from every endpoint, midpoint and axis, and a check that silently depended on that would start failing the day somebody changed `--pdf` |
//! | drive with **Alt** held, which refuses the snap | it would assert about a *non-default* configuration. Snapping on is the shipped behaviour, so a check that only ever exercised snapping off would stop covering the path the operator uses |
//! | loosen to "at least one `measure-pick` line" | it would pass against a build where nothing ever commits, which is the entire failure this check exists to catch |
//!
//! The strictness is kept exactly where it was. Every click produces **one**
//! trace line, a `Promoted` line is followed by a click at the *same* point
//! which must then resolve (`resolve_click` compares the promoted point against
//! the new one, and the same screen pixel yields the same candidate, so it
//! converges or the two-click confirm is broken and this check says so), and
//! the three resolved picks must still read `committed=false, false, true`.
//! [`MAX_CLICKS_PER_PICK`] is the bound.
//!
//! ### What is deliberately not asserted here: the Tab cycle
//!
//! `measure-snap-cycle index=N` reports the operator choosing *"the other
//! candidate"* between an endpoint and the midpoint a few pixels from it. It is
//! driven by <kbd>Tab</kbd>, and this check has never asserted it. It is named
//! rather than omitted so the gap is on the record: **the snap cycle is covered
//! by unit test alone.** ★ The reason given here used to be that keyboard input
//! could not be sent; that was false (see the correction below), so this gap is
//! now simply unwritten work rather than a limitation.
//!
//! ## Document evidence — that the engine accepted it
//!
//! `add-dimension page=… n=1 epoch=… disclosures=…`. See §"link 6" above.
//!
//! # Where the six clicks are aimed
//!
//! The three ribbon clicks go to rectangles **the application itself declared
//! on the frame it drew them** ([`crate::coords::WindowFrame::declared_center`]).
//! The three canvas clicks go through [`crate::coords::CanvasMapping`], built
//! from the `canvas rect=`/`zoom=`/`page=` the application traced this run, so
//! the only spatial literals in this file are the three
//! [`DocPoint`](crate::coords::DocPoint) fractions in [`PICKS`] — which is the
//! one literal [`crate::checks`] rule 2 permits, because a document coordinate
//! is stable under every layout change the roadmap contemplates.
//!
//! # Every way this reports SKIP, and why none of them is a pass
//!
//! * no binary, no `--pdf`, `--no-input` — the harness never began;
//! * the diagnostic switches did not reach the process;
//! * the page size could not be read from the fixture and no `--page-size` was
//!   given — without the page height there is no y-flip, and a wrong page
//!   height mirrors every click about the page centre, landing on the page and
//!   hit-testing something plausible;
//! * the mode segment, the Measure tab, or the Dimension group's controls were
//!   never declared — each names the specific surface that is missing, and the
//!   `ribbon.item.*` case names `report::band_item` and its call site;
//! * a measure tool was already armed before the click — `arm_measure`
//!   **toggles** on the same kind, so a click on an already-armed Linear
//!   correctly *disarms* it, and a check that did not notice would report the
//!   feature broken;
//! * the two controls already looked different before the click, so a
//!   difference afterwards could not be attributed to it;
//! * the canvas is not showing page 1, so the harness's one known page size
//!   does not describe the page it would be clicking on;
//! * a pick point does not map onto the canvas as currently laid out.

use crate::checks::driving::{
    self, INVOKE_EVENT, ITEM_PREFIX, MIN_PRESSED_DELTA, SHELL_DIAG_ENV, TAB_EVENT,
    UNIMPLEMENTED_EVENT, declared, declared_names, delta, fill_of, list, list_str, shell_trace,
};
use crate::checks::{Check, CheckContext};
use crate::coords::{CanvasMapping, DocPoint, PageGeometry};
use crate::error::{Error, Result};
use crate::input::Driver;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;

/// The mode whose tab list contains Measure, and the segment to click.
///
/// Read is the default and its tabs are `["file", "view"]`; Measure is in
/// Review's list and in Edit's. Review rather than Edit because it is the
/// weaker claim — a dimension that places in Review places in Edit — and
/// because Review is the row of `MODES_AND_PANELS.md`'s gesture table that
/// proves the mode gate is per-capability rather than a single on/off: a
/// reviewer places dimensions and does **not** select page content.
const MODE: &str = "review";

/// The tab that carries the Dimension group.
const TAB: &str = "ribbon.tab.measure";

/// The tab's id, as the shell reports it.
const TAB_ID: &str = "measure";

/// **The control under test.**
const SUBJECT: &str = "ribbon.item.measure.linear";

/// The command id of [`SUBJECT`], as dispatch and the shell spell it.
const SUBJECT_ID: &str = "measure.linear";

/// **The control that must NOT change** — the sibling in the same group,
/// drawn by the same code, in the same capture, on the same frame.
///
/// `measure.two_line` rather than `measure.radius_diameter` for a reason worth
/// stating: two-line is a *registered and dispatching* command (`SALVAGE.md`'s
/// Phase 7 entry), so a build where it vanished from the band would be news in
/// its own right, whereas radius/diameter is one of the three decisions
/// `HANDOFF.md` §8 records as still open and could legitimately move.
const SIBLING: &str = "ribbon.item.measure.two_line";

/// Where the pointer is parked before each capture.
///
/// The Dimension group's caption, which is an `egui::Label` and therefore has
/// no hover styling of its own, sitting directly beneath the controls being
/// measured. Parking matters: after a click the pointer is *on* the control,
/// `egui` paints it in its hovered visuals, and a before/after comparison
/// would then be measuring a hover as well as a selection.
const PARK: &str = "ribbon.group.measure.dimension.caption";

/// `measure-tool tool=…` — the application reporting which measure tool the
/// canvas is now armed with. Emitted by `canvas::tool::arm_measure`,
/// unconditionally, every time it is called.
const ARM_EVENT: &str = "measure-tool";

/// The `Debug` spelling of `CanvasTool::Measure(MeasureKind::Linear)`.
const ARM_VALUE: &str = "Measure(Linear)";

/// `measure-pick …` — **one line per click** the armed tool resolved, from
/// `canvas::measure::click`.
///
/// One event name, two shapes, and telling them apart is the whole of the
/// rule-4 handling in this file:
///
/// ```text
/// measure-pick kind=Linear in_progress=true committed=false          ← a resolved pick
/// measure-pick outcome=Promoted reason=derived-candidate-needs-confirm  ← click again
/// ```
const PICK_EVENT: &str = "measure-pick";

/// The `Debug` spelling of `MeasureKind::Linear`, as a resolved
/// [`PICK_EVENT`] line prints it.
const PICK_KIND: &str = "Linear";

/// The `outcome=` value that means *the click found an inference and is asking
/// before acting on it* — `ClickOutcome::Promoted`.
///
/// Matched on the field rather than on the raw line so a future addition to the
/// message cannot silently stop this check recognising a promotion; a
/// promotion it failed to recognise would be counted as a resolved pick with no
/// `committed=` field, and the sequence assertion would then fail against a
/// working build.
const PICK_PROMOTED: &str = "Promoted";

/// How many clicks one pick may take before this check calls the two-click
/// confirm broken.
///
/// **Two**, and the number is `snap_commit_clicks`'s own: a routine candidate
/// commits on the first click, a derived one on the second. A third would mean
/// `MeasureState::resolve_click` is not converging — its promote branch
/// compares `derived_promoted != Some(point)`, so a click at the same screen
/// pixel that promoted again would mean the resolved point is *moving* between
/// two clicks of a stationary pointer, which is a real finding and not a reason
/// to keep clicking.
const MAX_CLICKS_PER_PICK: usize = 2;

/// `add-dimension page=… n=… epoch=… disclosures=…` — **the engine accepted
/// the dimension and the document changed.**
///
/// Built by `app/actions.rs`'s `vector_edit` from the label its
/// `Action::CommitDimension` arm passes (`"add-dimension"`), on the `Ok` path
/// only. See the module header's §"link 6".
const COMMIT_EVENT: &str = "add-dimension";

/// `add-dimension-refused page=… …` — the same funnel's `Err` path, where the
/// engine declined and the document was left alone.
///
/// Read only to improve a failure message. Two shapes reach it: a structured
/// `EditError` from `add_dimension` (`detail=`), and the borrow guard
/// (`reason=session-borrowed`), which means another holder of the
/// `Arc<EditSession>` was alive when the action was applied.
const REFUSED_EVENT: &str = "add-dimension-refused";

/// **The three clicks that place a linear dimension**, as fractions of the
/// page box: *what*, *to what*, and *where it sits*.
///
/// # Why fractions rather than absolute points
///
/// [`crate::checks`] rule 2 permits a [`DocPoint`] literal, and this is one —
/// resolved against the fixture's own `/MediaBox` at run time rather than
/// against a page size written down here. That makes the check fixture-
/// agnostic in the one way that matters: a dimension needs no *content* under
/// it, only somewhere on the page to put it, so any fixture large enough to
/// have a middle will do. Absolute points would silently move off the page the
/// first time somebody pointed `--pdf` at a letter-size document.
///
/// # These are where the pointer goes, not necessarily what is committed
///
/// Since snapping landed, `canvas::measure::snapped` resolves each click to the
/// nearest snap candidate within `PageMapping::snap_tolerance` and commits
/// *that*, which is what makes a dimension measure a line rather than *near*
/// one. So these fractions are the aim, and the committed geometry is the
/// application's answer to it. Nothing in this check asserts on the committed
/// coordinates, deliberately: that is `pdfcer-core`'s snap query, it has its own
/// tests, and re-deriving the expected snap here would be this harness
/// reimplementing the thing it is supposed to be observing.
///
/// # Why these three
///
/// A and B are 35 % of the page width apart on the same horizontal line, which
/// is comfortably longer than any degeneracy threshold and gives the third
/// click an unambiguous perpendicular to resolve a standoff against. The third
/// sits above the pair and off the midpoint, so `placement_from_point` returns
/// a non-zero **offset** and a non-zero **text_along** — the two components
/// `LinearPick::placing_kind` computes, and the two that would both read zero
/// if the third click were being ignored and the dimension committed on the
/// second.
///
/// The y values are PDF user space: origin bottom-left, y growing **up**. The
/// one flip in this crate happens inside
/// [`CanvasMapping::doc_to_window`](crate::coords::CanvasMapping::doc_to_window).
const PICKS: [(f64, f64); 3] = [
    (0.30, 0.45), // A — what
    (0.65, 0.45), // B — to what
    // Where the dimension sits: above the pair (a non-zero standoff) and
    // deliberately NOT at the A–B midpoint (a non-zero `text_along`). The
    // midpoint is 0.475, and this being 0.475 is exactly the mistake
    // `a_linear_dimension_is_exactly_three_clicks` caught — a placement click
    // on the midpoint makes one of the two components `placement_from_point`
    // returns zero, so half of what the third click is *for* would go
    // unexercised while the check still went green.
    (0.55, 0.62),
];

/// See the module documentation.
pub struct MeasureLinearPlacesADimension;

impl Check for MeasureLinearPlacesADimension {
    fn name(&self) -> &'static str {
        "measure_linear_places_a_dimension"
    }

    fn defect(&self) -> &'static str {
        "clicking Measure ▸ Dimension ▸ Linear and then three points on the page does not \
         place a dimension — the control does not arm, or arms without rendering pressed, or \
         the picks never reach the tool, or the third pick's action is refused by the engine \
         and the document is left unchanged while the shell reports a commit"
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

/// Run the sequence.
///
/// The three-way return is [`crate::report`]'s rule made structural: `Err` is
/// a precondition that was absent (SKIP), `Ok(Some(_))` is an assertion that
/// did not hold (FAIL), `Ok(None)` is a pass. Reaching for `?` therefore
/// yields a SKIP, which is the safe default; the unsafe default would be a
/// pass.
#[allow(clippy::too_many_lines)]
fn drive(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    // --- preconditions -----------------------------------------------------
    let exe = ctx.resolve_exe().ok_or_else(|| {
        Error::new(format!(
            "no binary to drive. Pass --exe, or build the profile's default at {}.",
            ctx.profile.default_exe
        ))
    })?;
    // Not optional. Every `measure.*` command is registered
    // `.enabled_when("doc.pages")`, so with nothing open the control is
    // correctly greyed — and a check that drove that would be asserting the
    // enable predicate works while claiming to assert that Linear does.
    let pdf = ctx.pdf.clone().ok_or_else(|| {
        Error::new(
            "no --pdf. Every measure command is gated on `doc.pages`, so with no document open \
             the Linear control is correctly disabled and this check would be measuring the \
             gate rather than the feature. A dimension also has to be placed *on* a page.",
        )
    })?;
    if !ctx.allow_input {
        return Err(Error::new(
            "input is disabled (--no-input). This check is six real clicks and two window \
             captures, and every one of them needs the pointer and the foreground. Reported as \
             SKIPPED rather than passed: a check that did not run has learned nothing.",
        ));
    }
    let ui_rect = ctx.profile.vocab.ui_rect_event.ok_or_else(|| {
        Error::new(format!(
            "the `{}` profile declares no ui-rect trace event, so the application cannot state \
             where its controls are and this check has nothing to aim at.",
            ctx.profile.name
        ))
    })?;

    // The page height, for the y-flip, and the width, for the fractions in
    // `PICKS`. Refused rather than guessed: `crate::fixture`'s header explains
    // that a wrong page height produces a click mirrored about the page
    // centre, which lands on the page and looks like a working conversion.
    let page: PageGeometry = match ctx.page_size {
        Some((w, h)) => PageGeometry {
            width_pt: w,
            height_pt: h,
        },
        None => crate::fixture::page_geometry(&pdf).ok_or_else(|| {
            Error::new(format!(
                "cannot read a page size from {}. The harness needs the page box to turn this \
                 check's three document-space fractions into points, and the page height to \
                 flip PDF y (up) into window y (down). Pass --page-size WxH.",
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
    let mut spec = LaunchSpec::new(&exe, ctx.out("measure_linear.trace.txt"));
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
    // Generous: the ribbon is chrome and is laid out on the first frame, but
    // the fixture still has to parse and raster, and a window captured
    // mid-raster is a window whose controls are drawn over a placeholder.
    session.settle(40);

    let trace = session.trace()?;
    if !trace.started(ctx.profile.vocab.start_event) {
        return Err(Error::new(format!(
            "the trace has no `{}` line, so the diagnostic switch {}={} did not reach the \
             process and this check has no way to learn where any control is. Captured stderr \
             is at {}.",
            ctx.profile.vocab.start_event,
            ctx.profile.diag_env.0,
            ctx.profile.diag_env.1,
            session.trace_path().display()
        )));
    }
    for reject in trace.rejected_steps() {
        report.note(format!(
            "the application REJECTED a script step: {}",
            reject.raw
        ));
    }

    let frame = session.frame()?;
    report.note(format!(
        "window client area {}x{} px at desktop ({}, {}), DPI scale {:.2}",
        frame.client_size.0,
        frame.client_size.1,
        frame.client_origin.0,
        frame.client_origin.1,
        frame.scale
    ));
    let driver = Driver::new(session.window());

    // --- step 1: switch to Review -----------------------------------------
    //
    // A failure here is a SKIP, not a FAIL: see `driving::click_mode_segment`.
    driving::click_mode_segment(&session, &driver, ui_rect, MODE)?;
    report.note(format!(
        "the {MODE} segment reported the click, so pointer input reaches the ribbon"
    ));

    // --- step 2: activate the Measure tab ----------------------------------
    let trace = session.trace()?;
    let tab = declared(&trace, ui_rect, TAB).ok_or_else(|| {
        Error::new(format!(
            "the application declared no `{TAB}` region after switching to {MODE}. Either this \
             build has no Measure tab, or the tab strip is too narrow and the tab has moved \
             into the strip's overflow menu — which this check cannot open, because the menu's \
             contents are not published as regions. Tabs declared: {}. Strip affordance \
             declared: {}.",
            list(&declared_names(&trace, ui_rect, "ribbon.tab.")),
            declared(&trace, ui_rect, "ribbon.tabs.overflow")
                .map_or_else(|| "no".to_owned(), |r| format!("yes, at {r:?}")),
        ))
    })?;
    report.note(format!("{TAB} declared at {tab:?}; clicking its centre"));
    driver.click_at(frame.declared_center(tab))?;
    session.settle(12);

    let shell = shell_trace(&session)?;
    if !shell
        .events(TAB_EVENT)
        .any(|l| l.get("tab") == Some(TAB_ID))
    {
        return Err(Error::new(format!(
            "the click on `{TAB}` produced no `{TAB_EVENT} tab={TAB_ID}` line. The {MODE} click \
             DID land, so pointer input works and this is not the input channel; the likely \
             cause is that the tab moved between the frame that declared its rect and the frame \
             that received the click. Re-run; if it persists, the tab strip is reflowing every \
             frame, which is itself the finding."
        )));
    }
    report.note("the Measure tab reported the click");

    // --- step 3: locate the two controls ----------------------------------
    let trace = session.trace()?;
    let items = declared_names(&trace, ui_rect, ITEM_PREFIX);
    if items.is_empty() {
        return Err(Error::new(format!(
            "the application declared no `{ITEM_PREFIX}*` regions at all, so no individual \
             ribbon command can be located from outside the process and there is nothing to \
             click. This is not a defect in Measure: it is a build whose ribbon publishes rects \
             for its group captions, its tabs and its mode segments but not for its command \
             controls. The publisher is `egui_shell::ribbon::report::band_item`, called from \
             `band::render_command`. Regions declared under `ribbon.`: {}.",
            list(&declared_names(&trace, ui_rect, "ribbon."))
        )));
    }
    report.note(format!(
        "{} command control(s) declared their rects on the Measure tab",
        items.len()
    ));

    let subject = declared(&trace, ui_rect, SUBJECT).ok_or_else(|| {
        Error::new(format!(
            "the Measure tab is active and its controls publish their rects, but none of them \
             is `{SUBJECT}`. Controls declared: {}.",
            list(&items)
        ))
    })?;
    let sibling = declared(&trace, ui_rect, SIBLING).ok_or_else(|| {
        Error::new(format!(
            "`{SUBJECT}` is declared but `{SIBLING}` is not, and this check needs both: the \
             pressed state is asserted as a difference between the control that was clicked and \
             a sibling in the same group that was not, measured in one capture. Without the \
             sibling the only available evidence is before-and-after, which any repaint of the \
             whole band would satisfy. Controls declared: {}.",
            list(&items)
        ))
    })?;
    let park = declared(&trace, ui_rect, PARK).ok_or_else(|| {
        Error::new(format!(
            "the Dimension group declared no `{PARK}` region, and this check parks the pointer \
             there before each capture so that a hover is not mistaken for a selection. \
             Captions declared: {}.",
            list(&declared_names(&trace, ui_rect, "ribbon.group.measure."))
        ))
    })?;
    if !subject.is_substantial() || !sibling.is_substantial() {
        return Err(Error::new(format!(
            "a control was declared at no usable size — `{SUBJECT}` at {subject:?}, `{SIBLING}` \
             at {sibling:?}. A click aimed at a degenerate rectangle proves nothing, so this is \
             reported rather than driven."
        )));
    }
    report.note(format!(
        "{SUBJECT} at {subject:?}, {SIBLING} at {sibling:?}, parking on {PARK} at {park:?}"
    ));

    // A tool armed BEFORE the click would make the click a *disarm*:
    // `arm_measure` toggles on the same kind. Nothing persists it across a
    // launch today — the tool lives in `egui::Memory`'s temporary data — but
    // "today" is not an assertion, and a check that mis-read a disarm as a
    // failure would blame working code.
    if let Some(line) = trace.last(ARM_EVENT) {
        return Err(Error::new(format!(
            "a measure tool was already armed before this check clicked anything: `{}`. \
             `canvas::tool::arm_measure` TOGGLES on the same kind, so a click on an \
             already-armed Linear correctly retires it — and this check would then be asserting \
             that a working disarm is a broken arm.",
            line.raw
        )));
    }
    // Likewise a pick already taken: the pick sequence asserted below counts
    // from zero, and a stray pick would shift every line by one.
    if let Some(line) = trace.last(PICK_EVENT) {
        return Err(Error::new(format!(
            "a measure pick had already been taken before this check clicked anything: `{}`. \
             The three-line pick sequence below is asserted positionally, so a pre-existing \
             pick would make every assertion about the wrong click.",
            line.raw
        )));
    }

    // --- step 4: the BEFORE capture ---------------------------------------
    driver.move_to(frame.declared_center(park))?;
    session.settle(8);
    let before_path = ctx.out("measure_linear.before.png");
    let before = crate::capture::window_to_png(&session, &before_path)?;
    report.artifact(before_path);

    let subject_before = fill_of(&before, &frame, subject).ok_or_else(|| {
        Error::new(format!(
            "`{SUBJECT}` was declared at {subject:?}, which resolves to no pixels of the \
             captured client area — the application declared a control outside its own window."
        ))
    })?;
    let sibling_before = fill_of(&before, &frame, sibling).ok_or_else(|| {
        Error::new(format!(
            "`{SIBLING}` was declared at {sibling:?}, which resolves to no pixels of the capture."
        ))
    })?;
    let before_gap = delta(subject_before, sibling_before);
    report.note(format!(
        "before the click: Linear fills {subject_before}, Two-line fills {sibling_before} \
         (max channel gap {before_gap})"
    ));
    if before_gap >= MIN_PRESSED_DELTA {
        return Err(Error::new(format!(
            "Linear and Two-line already differ by {before_gap} (>= {MIN_PRESSED_DELTA}) BEFORE \
             anything was clicked, so a difference afterwards could not be attributed to the \
             click. Two controls in one group, neither selected, should be drawn in the same \
             fill. Look at the before capture."
        )));
    }

    // --- step 5: click Linear ---------------------------------------------
    driver.click_at(frame.declared_center(subject))?;
    session.settle(16);

    // --- step 6: the AFTER capture ----------------------------------------
    //
    // Park FIRST. The pointer is sitting on Linear after the click, so a
    // capture taken now would show it hovered as well as selected, and the
    // difference this check measures would be partly a hover — which the
    // sibling, never hovered, would not share. That would make P2 pass for the
    // wrong reason, which is the worst outcome available: a green check
    // measuring the wrong thing.
    driver.move_to(frame.declared_center(park))?;
    session.settle(8);
    let after_path = ctx.out("measure_linear.after.png");
    let after = crate::capture::window_to_png(&session, &after_path)?;
    report.artifact(after_path);

    // --- the TRACE assertions for the arm ----------------------------------
    let shell = shell_trace(&session)?;
    if !shell
        .events(INVOKE_EVENT)
        .any(|l| l.get("id") == Some(SUBJECT_ID))
    {
        let seen: Vec<&str> = shell
            .events(INVOKE_EVENT)
            .filter_map(|l| l.get("id"))
            .collect();
        return Ok(Some(format!(
            "TRACE: the click did not reach the control. The harness clicked the centre of \
             {subject:?} — the rectangle the application itself published for `{SUBJECT}` on \
             the frame it drew it — and the shell traced no `{INVOKE_EVENT} id={SUBJECT_ID}`. \
             Commands the shell reported as invoked this run: {}. Two readings: the control was \
             disabled, which for a `doc.pages`-gated command means the fixture opened no pages; \
             or the band reported a rectangle it did not draw the control in. Both are findings.",
            list_str(&seen)
        )));
    }
    report.note(format!(
        "the shell traced `{INVOKE_EVENT} id={SUBJECT_ID}`, so the click reached the control"
    ));

    let trace = session.trace()?;
    if !trace
        .events(ARM_EVENT)
        .any(|l| l.get("tool") == Some(ARM_VALUE))
    {
        let unimplemented = trace
            .events(UNIMPLEMENTED_EVENT)
            .any(|l| l.get("id") == Some(SUBJECT_ID));
        let tools: Vec<&str> = trace
            .events(ARM_EVENT)
            .filter_map(|l| l.get("tool"))
            .collect();
        return Ok(Some(format!(
            "TRACE: the click reached the control and armed nothing. The shell traced \
             `{INVOKE_EVENT} id={SUBJECT_ID}`, so the command was invoked and its token was \
             handed to the application — and the application traced no `{ARM_EVENT} \
             tool={ARM_VALUE}`. `canvas::tool::arm_measure` traces unconditionally the moment \
             it is called, so its silence means it was never called. Tools reported this run: \
             {}. {} Look at `app/dispatch.rs`'s guard arm — the one matching on \
             `shell::commands::measure_for_command(id).is_some()` — and at \
             `measure_for_command` itself, which is the single binding between a command id and \
             a `MeasureKind`. Note also that `app::modes::capability` gates arming: a build \
             whose {MODE} mode had lost the `measure` tab from its tab list would decline this \
             command and trace nothing, and the fix would be in the manifest rather than in \
             dispatch.",
            list_str(&tools),
            if unimplemented {
                format!(
                    "The application DID trace `{UNIMPLEMENTED_EVENT} id={SUBJECT_ID}`, which \
                     is `dispatch_command`'s fall-through arm: the command arrived at dispatch \
                     and dispatch had no arm for it."
                )
            } else {
                format!(
                    "No `{UNIMPLEMENTED_EVENT} id={SUBJECT_ID}` either, so the command did not \
                     reach `dispatch_command`'s fall-through — check `dispatch_token`'s \
                     token-to-id lookup."
                )
            }
        )));
    }
    report.note(format!(
        "the application traced `{ARM_EVENT} tool={ARM_VALUE}`, so the canvas tool is armed"
    ));

    // --- the PIXEL assertions ---------------------------------------------
    //
    // Reached only once the trace has established that the tool really is
    // armed, so anything that fails from here on is link 4 — the ribbon not
    // showing a state the application is genuinely in — and the failure text
    // can say so without hedging.
    let subject_after = fill_of(&after, &frame, subject).ok_or_else(|| {
        Error::new(format!(
            "`{SUBJECT}` resolved to no pixels in the after capture, though it did in the \
             before capture — the control moved off the client area across the click."
        ))
    })?;
    let sibling_after = fill_of(&after, &frame, sibling).ok_or_else(|| {
        Error::new(format!(
            "`{SIBLING}` resolved to no pixels in the after capture, though it did in the \
             before capture."
        ))
    })?;
    let p1 = delta(subject_after, subject_before);
    let p2 = delta(subject_after, sibling_after);
    let p3 = delta(sibling_after, sibling_before);
    report.note(format!(
        "after the click: Linear fills {subject_after}, Two-line fills {sibling_after}"
    ));
    report.note(format!(
        "P1 Linear across the click: {p1}; P2 Linear vs Two-line in one capture: {p2}; \
         P3 Two-line across the click: {p3}; threshold {MIN_PRESSED_DELTA}"
    ));
    // ★ From here on, a failed assertion is COLLECTED rather than returned.
    //
    // Everything above this line is a precondition for everything below it —
    // no arm, no picks — so those exits are early and stay early. The three
    // groups that follow are independent facts about one feature, and a check
    // that stopped at the first would answer "does the control look pressed?"
    // and leave "does a dimension get placed?" unanswered on the run where
    // that is the more interesting question. One run, the whole story.
    let mut failures: Vec<String> = Vec::new();
    if p2 < MIN_PRESSED_DELTA {
        failures.push(format!(
            "PIXELS: the tool is armed and the control does not look it. In the SAME capture, \
             Linear's fill is {subject_after} and Two-line's is {sibling_after} — a maximum \
             channel difference of {p2}, under the {MIN_PRESSED_DELTA} floor — so the control \
             that was clicked is drawn exactly like the one that was not. The trace already \
             proved `{ARM_EVENT} tool={ARM_VALUE}`, which is why this is a rendering finding \
             and not a dispatch one: look at `app/conditions.rs` publishing \
             `selected:{SUBJECT_ID}` and at `egui_shell::ribbon::band::render_command` reading \
             it. THIS IS THE ASSERTION A TRACE LINE CANNOT MAKE — a build whose ribbon never \
             shows a pressed state emits an identical trace."
        ));
    }
    if p2 >= MIN_PRESSED_DELTA && p1 < MIN_PRESSED_DELTA {
        failures.push(format!(
            "PIXELS: Linear differs from Two-line ({p2}) but did not change across the click \
             ({p1} < {MIN_PRESSED_DELTA}), so it looked that way already and the click is not \
             what made it."
        ));
    }
    if p3 >= MIN_PRESSED_DELTA {
        failures.push(format!(
            "PIXELS: Two-line ALSO changed across the click ({p3} >= {MIN_PRESSED_DELTA}), so \
             what was measured is the whole band repainting rather than one control being \
             pressed. A pressed state that spreads to its neighbours is not a pressed state; \
             compare the two captures."
        ));
    }
    if failures.is_empty() {
        report.note(
            "pressed rendering confirmed from the pixels: the clicked control changed, its \
             un-clicked sibling did not, and the two differ in one capture",
        );
    }

    // --- step 7: aim the three picks, in DOCUMENT space --------------------
    //
    // Rebuilt here rather than earlier, because the mode switch and the tab
    // activation both relaid the canvas out: Read defaults to a continuous
    // strip and Review to a single page (`viewer::display::default_for_mode`),
    // so a mapping taken before step 1 would describe a canvas that no longer
    // exists. The application re-declares its `canvas` line on every layout
    // change, so the freshest one is the right one — `CanvasMapping::from_trace`
    // takes the last.
    let trace = session.trace()?;
    let canvas_page = trace
        .last(ctx.profile.vocab.canvas_event)
        .and_then(|l| l.get_usize("page"));
    if canvas_page != Some(0) {
        return Err(Error::new(format!(
            "the canvas is showing page {}, not page 1, and this check knows only the size of \
             the page it read a `/MediaBox` for. Converting its document-space picks against a \
             page of a different size would put them somewhere plausible and wrong — a whole \
             class of confidently-wrong click that `crate::coords` exists to refuse. Trace: {}.",
            canvas_page.map_or_else(|| "an unreported index".to_owned(), |p| (p + 1).to_string()),
            session.trace_path().display()
        )));
    }
    let mapping = CanvasMapping::from_trace(&trace, &ctx.profile.vocab, page, 0)?;
    report.note(format!(
        "canvas rect {:?} at zoom {:.3}, showing page 1",
        mapping.image_rect, mapping.zoom
    ));

    let frame = session.frame()?;
    let mut aimed = Vec::with_capacity(PICKS.len());
    for (fx, fy) in PICKS {
        let point = DocPoint::new(0, fx * page.width_pt, fy * page.height_pt);
        let window = mapping.doc_to_window(point)?;
        aimed.push((point, frame.to_screen(window)));
    }
    report.note(format!(
        "picks: A ({:.0}, {:.0}), B ({:.0}, {:.0}), placement ({:.0}, {:.0}) in PDF user space",
        aimed[0].0.x, aimed[0].0.y, aimed[1].0.x, aimed[1].0.y, aimed[2].0.x, aimed[2].0.y
    ));

    // --- step 8: three picks on the page ----------------------------------
    //
    // ★ One pick is one OR TWO clicks — see the module header's rule-4
    // section. Each click must produce exactly one `measure-pick` line; a
    // `Promoted` line means the application found an inference and is asking
    // before acting on it, so the same point is clicked again, at most
    // `MAX_CLICKS_PER_PICK` times in total.
    let mut resolved: Vec<String> = Vec::with_capacity(PICKS.len());
    let mut promotions = 0usize;
    for (n, (point, screen)) in aimed.iter().enumerate() {
        let mut clicks = 0usize;
        loop {
            let before = session.trace()?.events(PICK_EVENT).count();
            clicks += 1;
            report.note(format!(
                "pick {} of {}: click {clicks} at document ({:.0}, {:.0}) → screen ({}, {})",
                n + 1,
                PICKS.len(),
                point.x,
                point.y,
                screen.x(),
                screen.y()
            ));
            driver.click_at(*screen)?;
            session.settle(12);

            let trace = session.trace()?;
            let lines: Vec<&crate::trace::TraceLine> = trace.events(PICK_EVENT).collect();
            let new: Vec<&crate::trace::TraceLine> = lines.iter().skip(before).copied().collect();
            if new.len() != 1 {
                failures.push(format!(
                    "GESTURE: one click on the page produced {} `{PICK_EVENT}` line(s), not 1. \
                     `canvas::measure::click` traces exactly once per click it is handed — on \
                     the promote path and on the resolve path alike — so zero means the click \
                     never became a pick and more than one means it was delivered twice. Zero \
                     is the interesting case and has three readings: the gesture machine \
                     swallowed it (`canvas::gesture::press_kind` returns \
                     `click: caps.author_measure`, so a `{MODE}` mode that had lost the \
                     `measure` tab from its tab list would swallow every one), the click landed \
                     outside the page rect, or `canvas::interact`'s `Click` arm no longer \
                     branches on `active_tool.measure_kind()`. New lines: {}.",
                    new.len(),
                    list_str(&new.iter().map(|l| l.raw.as_str()).collect::<Vec<_>>())
                ));
                return Ok(verdict(failures));
            }
            let line = new[0];

            // Rule 4: an inferred candidate is announced, not committed.
            if line.get("outcome") == Some(PICK_PROMOTED) {
                promotions += 1;
                report.note(format!(
                    "pick {}: the application promoted a derived snap candidate rather than \
                     committing it (`{}`) — rule 4's fuzzy-never-sneaky gate. Clicking the same \
                     point again to confirm, exactly as an operator would.",
                    n + 1,
                    line.raw
                ));
                if clicks >= MAX_CLICKS_PER_PICK {
                    failures.push(format!(
                        "GESTURE: the two-click confirm did not converge. Pick {} was promoted \
                         on click 1 and promoted AGAIN on click {clicks}, at the same screen \
                         pixel. `MeasureState::resolve_click` promotes only when \
                         `derived_promoted != Some(point)`, so a second promotion means the \
                         point the snap query resolved to MOVED between two clicks of a \
                         stationary pointer — and a derived candidate that can never be \
                         confirmed is a candidate the operator can never pick. Look at \
                         `canvas::measure::snapped` and at whether \
                         `snap::active_snap_candidate` is being handed a stable `snap_cycle`. \
                         Line: `{}`.",
                        n + 1,
                        line.raw
                    ));
                    return Ok(verdict(failures));
                }
                continue;
            }

            if line.get("kind") != Some(PICK_KIND) {
                failures.push(format!(
                    "GESTURE: pick {} was resolved by the `{}` tool rather than `{PICK_KIND}`. \
                     The armed kind changed between the ribbon click and the canvas click, \
                     which `MeasureState::set_kind` would have discarded the gesture over. \
                     Line: `{}`.",
                    n + 1,
                    line.get("kind").unwrap_or("?"),
                    line.raw
                ));
                return Ok(verdict(failures));
            }
            resolved.push(line.get("committed").unwrap_or("?").to_owned());
            break;
        }
    }
    if promotions > 0 {
        report.note(format!(
            "{promotions} of the picks needed a confirming second click, which is rule 4 \
             working rather than a wobble: a derived snap candidate is pdfcer's inference, and \
             `snap::snap_commit_clicks` requires two clicks for one"
        ));
    }

    // ★ The shape of the sequence IS the feature: two picks that take, and a
    // third that commits. Both halves are asserted, because a tool that
    // committed on pick A would satisfy "a dimension was placed" perfectly.
    let trace = session.trace()?;
    if resolved != ["false", "false", "true"] {
        let all: Vec<&str> = trace.events(PICK_EVENT).map(|l| l.raw.as_str()).collect();
        failures.push(format!(
            "GESTURE: the three picks reported committed={resolved:?}, and a linear dimension \
             is what, to what, and WHERE — so the expected sequence is [false, false, true]. A \
             `true` on the second is the old two-click commit with a zero standoff returning, \
             which `LinearPick::second`'s own documentation records as the behaviour the third \
             pick exists to replace; a `false` on the third means \
             `LinearPick::commit_point`'s placing arm returned `None` and no \
             `Action::CommitDimension` was raised at all. Every `{PICK_EVENT}` line this run: \
             {}.",
            list_str(&all)
        ));
        return Ok(verdict(failures));
    }
    report.note(
        "the pick sequence is committed=false, false, true — the third pick is the commit, \
         and there is no accept box",
    );

    // --- the DOCUMENT assertion -------------------------------------------
    //
    // ★ The one that a `committed=true` cannot make. See the module header's
    // §"link 6": the shell raising an action and the engine accepting it are
    // two different facts, and only the second one changes the document.
    let commits: Vec<&crate::trace::TraceLine> = trace.events(COMMIT_EVENT).collect();
    let Some(commit) = commits.last() else {
        let refusals: Vec<&str> = trace
            .events(REFUSED_EVENT)
            .map(|l| l.raw.as_str())
            .collect();
        failures.push(format!(
            "DOCUMENT: the shell committed and the engine did not. The third pick traced \
             `{PICK_EVENT} … committed=true`, so `Action::CommitDimension` was raised — and \
             `app/actions.rs`'s `vector_edit` traced no `{COMMIT_EVENT}` line, which it writes \
             on its Ok path only, after bumping `doc.edit_epoch` and dropping the page texture. \
             So no dimension is on the page and the undo log has nothing in it. {} This is \
             exactly the gap a `committed=true` cannot see: the action was raised, and the \
             document did not change.",
            if refusals.is_empty() {
                format!(
                    "There is no `{REFUSED_EVENT}` line either, so `vector_edit` was never \
                     reached — look at `app/dispatch.rs`'s `Action::CommitDimension` arm and at \
                     whether the frame's actions are being applied at all."
                )
            } else {
                format!(
                    "The application DID trace a refusal, and it names the cause: {}. A \
                     `reason=session-borrowed` means another holder of the `Arc<EditSession>` \
                     was alive when the action was applied; a `detail=` is the engine's own \
                     structured `EditError` from `EditSession::add_dimension`.",
                    list_str(&refusals)
                )
            }
        ));
        return Ok(verdict(failures));
    };
    let epoch = commit.get_usize("epoch");
    let placed = commit.get_usize("n");
    report.note(format!(
        "the engine accepted it: `{}` — the edit epoch is now {}, which is the counter the \
         selection re-resolve and the raster key both read",
        commit.raw,
        epoch.map_or_else(|| "unreported".to_owned(), |e| e.to_string())
    ));
    if commits.len() != 1 {
        failures.push(format!(
            "DOCUMENT: three clicks placed {} dimensions, not one. `app/actions.rs`'s \
             `CommitDimension` arm holds a one-`add_dimension`-one-undo-entry contract, and a \
             second commit means a pick was resolved twice — the most likely cause is a click \
             being reported on two consecutive frames. Lines: {}.",
            commits.len(),
            list_str(&commits.iter().map(|l| l.raw.as_str()).collect::<Vec<_>>())
        ));
    }
    if placed != Some(1) {
        failures.push(format!(
            "DOCUMENT: the commit reported n={placed:?} operands rather than 1. `add-dimension` \
             is raised with a literal 1 in the `CommitDimension` arm, so anything else means \
             the trace's shape has changed and this check is reading the wrong field. Line: \
             `{}`.",
            commit.raw
        ));
    }
    if epoch.is_none_or(|e| e == 0) {
        failures.push(format!(
            "DOCUMENT: the commit line carries epoch={epoch:?}. `vector_edit` bumps \
             `doc.edit_epoch` immediately before writing this line, so a zero or absent epoch \
             means the bump did not happen — and the epoch is what `canvas::interact` \
             re-resolves the selection against and what `render::worker`'s key rebuilds the \
             raster on, so a dimension added without it is a dimension nothing will redraw. \
             Line: `{}`.",
            commit.raw
        ));
    }
    Ok(verdict(failures))
}

/// Turn the collected assertion failures into one verdict.
///
/// # Why the failures are collected rather than returned one at a time
///
/// A [`CheckReport`] carries one outcome, so a check that returned at its first
/// failed assertion would answer only the first question it happened to ask. On
/// this check that is a real loss: the pressed rendering and the placed
/// dimension are independent facts about one feature, and a run that stopped at
/// "the control does not look pressed" would leave "and does a dimension get
/// placed?" unanswered — which is the more interesting half, and the half a
/// reader would then have to go and drive by hand.
///
/// Numbered when there is more than one, because a wall of prose with two
/// distinct findings in it reads as one long finding.
fn verdict(failures: Vec<String>) -> Option<String> {
    match failures.len() {
        0 => None,
        1 => failures.into_iter().next(),
        n => Some(
            failures
                .iter()
                .enumerate()
                .map(|(i, f)| format!("[{} of {n}] {f}", i + 1))
                .collect::<Vec<_>>()
                .join("  ——  "),
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The names this check greps for are the ones the two crates build.
    ///
    /// Pinned here as well as in `egui-shell`'s own
    /// `the_reported_names_are_a_stability_contract`, because the crates are
    /// joined by a **string** and nothing else: this crate drives a process,
    /// so it cannot import the constant, and a rename would leave both sides
    /// compiling while every assertion here quietly stopped matching. A check
    /// that matches nothing passes vacuously, and that is the failure this
    /// test exists to make impossible.
    #[test]
    fn the_selectors_match_the_shells_own_spelling() {
        assert_eq!(SUBJECT, format!("ribbon.item.{SUBJECT_ID}"));
        assert!(SUBJECT.starts_with(ITEM_PREFIX));
        assert!(SIBLING.starts_with(ITEM_PREFIX));
        assert_eq!(TAB, format!("ribbon.tab.{TAB_ID}"));
        assert_eq!(PARK, "ribbon.group.measure.dimension.caption");
        // The parking spot must not be one of the controls being measured, or
        // the pointer would be hovering the thing under test.
        assert!(!PARK.starts_with(ITEM_PREFIX));
        assert_ne!(SUBJECT, SIBLING);
        // Review, not Read: Read's tab list is file+view, so the Measure tab
        // this check clicks does not exist there at all.
        assert_eq!(MODE, "review");
    }

    /// **Three picks, and the third is the only commit.**
    ///
    /// The sequence is the feature, so it is pinned as data here as well as
    /// asserted against the running binary: someone re-ordering [`PICKS`] or
    /// adding a fourth entry has to come past this test and decide what the
    /// expected `committed=` sequence now is.
    #[test]
    fn a_linear_dimension_is_exactly_three_clicks() {
        assert_eq!(PICKS.len(), 3, "what, to what, and where");
        // A and B share a y, so the pair has an unambiguous perpendicular for
        // the third click to resolve a standoff against.
        assert!((PICKS[0].1 - PICKS[1].1).abs() < f64::EPSILON);
        // …and they are far enough apart to be a real length rather than a
        // degeneracy the engine could legitimately refuse.
        assert!(PICKS[1].0 - PICKS[0].0 > 0.25);
        // The placement click is off the A–B line, so `placement_from_point`
        // returns a non-zero offset...
        assert!(PICKS[2].1 > PICKS[0].1);
        // ...and off its midpoint, so it returns a non-zero text_along too.
        let midpoint = f64::midpoint(PICKS[0].0, PICKS[1].0);
        assert!((PICKS[2].0 - midpoint).abs() > 0.001);
        // Every pick is well inside the page box, so a fixture of any size
        // maps all three onto paper rather than onto the grey surround.
        for (fx, fy) in PICKS {
            assert!((0.05..=0.95).contains(&fx), "x fraction {fx}");
            assert!((0.05..=0.95).contains(&fy), "y fraction {fy}");
        }
    }

    /// **The success and refusal events are two different event names**, which
    /// is what lets `Trace::events` tell them apart.
    ///
    /// `vector_edit` builds both from one label, so `add-dimension-refused`
    /// begins with `add-dimension` as a *string* — and a check that matched on
    /// a prefix would read every refusal as a success and report a placed
    /// dimension for a document nothing was written to. `Trace::parse` splits
    /// the event at the first space and compares it whole, so the two are
    /// distinct; this test is what says so out loud.
    #[test]
    fn a_refusal_is_not_read_as_a_commit() {
        let trace = crate::trace::Trace::parse(
            "pdfcer-diag add-dimension-refused page=0 n=1 detail=DegenerateLength\n\
             pdfcer-diag measure-pick kind=Linear in_progress=false committed=true",
            "pdfcer-diag",
        );
        assert_eq!(
            trace.events(COMMIT_EVENT).count(),
            0,
            "a refusal must not satisfy the document assertion, even though its event name \
             starts with the commit's"
        );
        assert_eq!(trace.events(REFUSED_EVENT).count(), 1);
        assert_eq!(
            trace.last(PICK_EVENT).and_then(|l| l.get("committed")),
            Some("true"),
            "…and this is the line that would have called it a success"
        );
    }

    /// The pick sequence is read positionally, and the fields are read by
    /// name — so a line whose `committed=` is missing reads as `"?"` rather
    /// than as `false`, and fails loudly instead of quietly.
    #[test]
    fn a_pick_line_without_a_committed_field_is_not_read_as_false() {
        let trace = crate::trace::Trace::parse(
            "pdfcer-diag measure-pick kind=Linear in_progress=true",
            "pdfcer-diag",
        );
        let line = trace.last(PICK_EVENT).expect("the pick line");
        assert_eq!(line.get("committed"), None);
        assert_eq!(line.get("kind"), Some(PICK_KIND));
    }

    /// ★ **A promotion is not a pick, and it is not a failure either.**
    ///
    /// The two shapes share one event name, so the classification is a field
    /// read. Getting it wrong in either direction is a real hazard: a
    /// promotion counted as a resolved pick has no `committed=` field and
    /// would read as `"?"`, failing the sequence assertion against a build
    /// that is doing exactly what `pdfce_FeatureRequests/README.md` rule 4
    /// asks of it; and a resolved pick mistaken for a promotion would make the
    /// check click again and lose count.
    #[test]
    fn a_promotion_is_told_apart_from_a_resolved_pick() {
        let trace = crate::trace::Trace::parse(
            "pdfcer-diag measure-pick outcome=Promoted reason=derived-candidate-needs-confirm\n\
             pdfcer-diag measure-pick kind=Linear in_progress=true committed=false",
            "pdfcer-diag",
        );
        let lines: Vec<_> = trace.events(PICK_EVENT).collect();
        assert_eq!(lines.len(), 2, "both shapes carry the same event name");
        assert_eq!(lines[0].get("outcome"), Some(PICK_PROMOTED));
        assert_eq!(
            lines[0].get("committed"),
            None,
            "a promotion commits nothing, so it carries no committed field"
        );
        assert_eq!(
            lines[1].get("outcome"),
            None,
            "a resolved pick must not be mistaken for a promotion"
        );
        assert_eq!(lines[1].get("committed"), Some("false"));
    }

    /// The confirm bound is the engine's own, not a number picked to make a
    /// flaky check stop failing.
    #[test]
    fn a_pick_takes_at_most_two_clicks() {
        assert_eq!(
            MAX_CLICKS_PER_PICK, 2,
            "`canvas::snap::snap_commit_clicks` returns 2 for a derived candidate and 1 for \
             every other kind; a third click would mean the confirm is not converging, which is \
             a finding rather than a reason to keep clicking"
        );
    }
}
