//! `undo_redo_round_trip` — **the check for the pair of commands that could
//! not take anything back**: author a change, undo it, redo it, and prove all
//! three states from outside the process.
//!
//! # What was wrong, and why nothing noticed
//!
//! `edit.undo` and `edit.redo` have been registered since the ribbon landed.
//! They are on the **quick-access toolbar**, which is drawn in every mode over
//! every document; they are bound to `Ctrl+Z`, `Ctrl+Y` and `Ctrl+Shift+Z`;
//! their tooltips name those chords. They had **no dispatch arm**, so every
//! press traced `command-unimplemented` and did nothing.
//!
//! That is `crate::checks::save_copy`'s defect at the other end of the same
//! day, and v0.1.0 made it worse rather than better: saving now works, so every
//! authoring feature this shell has — dimensions, seven markup kinds, text
//! marks, form fills — was reachable by an operator with no way to take any of
//! it back.
//!
//! The whole suite was green throughout, and it had to be. `pdfcer-core` tests
//! `EditSession::undo` exhaustively; `shell::commands` tests the registration
//! and the two predicates; `app::conditions` tests that the ribbon reads the
//! set it publishes. What no test in the workspace can observe is the **join** —
//! that pressing the control reaches an arm, that the arm reaches the engine,
//! that the engine's answer reaches the caches, and that the surfaces an
//! operator reads move back with it.
//!
//! # The five links, and where each is otherwise covered
//!
//! | # | Link | Its own test |
//! |---|---|---|
//! | 1 | the QAT click reports the command | `egui-shell`'s `qat::render` — yes |
//! | 2 | dispatch raises `Action::Undo` | `app::dispatch`'s unit test — yes |
//! | 3 | the apply reaches `EditSession::undo` | **nothing** |
//! | 4 | the **epoch** moves, so every epoch-keyed cache rebuilds | **nothing** |
//! | 5 | the **cached texture** is dropped, so the canvas re-rasters | **nothing** |
//!
//! Links 4 and 5 are the reason this check exists in the shape it does. They
//! are the two steps of `app::actions::apply`'s four-step protocol that have no
//! observable consequence *inside* the process a unit test could reach: a build
//! that mutated the session correctly and skipped both would satisfy every
//! count anyone could read from the engine, and show the operator the state
//! they had just taken back.
//!
//! # The six phases
//!
//! | Phase | Does | Expected |
//! |---|---|---|
//! | A | Review, Comments panel | a `comments-panel listed=N` baseline, and an `objects` line to key phase D on |
//! | B | click **Undo** with an empty log | **no** `ribbon-command-invoked`: the control is correctly greyed |
//! | C | arm Rectangle, drag on the page | `add-markup`, and `listed=N+1` |
//! | D | click **Undo** | `undo kind=AddAnnotation`, `undo-applied … epoch=`, `listed=N`, **a new `objects` line**, **a new `render-spawn`** |
//! | E | click **Redo** | `redo kind=AddAnnotation`, `listed=N+1`, and the same two invalidation signals again |
//! | F | click **Redo** again | **no** `ribbon-command-invoked`: the stack emptied and the condition followed |
//!
//! # ★ The falsifying phases, and the build each one catches
//!
//! `crate::checks`' rule for a new check is that *"it must fail against a build
//! where the wiring is absent"*. The counts alone do not satisfy that, and
//! saying why is the most useful paragraph in this file.
//!
//! ## The count is a weak oracle on its own
//!
//! `comments-panel listed=` is derived by walking the session's annotation list
//! afresh **every frame** — the panel holds no cache. So a build whose undo did
//! this:
//!
//! ```ignore
//! Arc::get_mut(&mut doc.session).map(EditSession::undo);   // and nothing else
//! ```
//!
//! would move every count in this check correctly. The annotation really is off
//! the session; the panel really does list one fewer. And the operator would be
//! looking at a page that still has the rectangle on it, with a selection
//! resolved against a revision that no longer exists, a decomposition listing
//! objects from the old one, and a page-text cache to match. **Every number
//! this check could read would already be right.**
//!
//! ## D-objects catches: *the session was mutated and the epoch never moved*
//!
//! `OpenDoc::trace_object_count` emits one `objects n=… page=…` line per
//! **`(page index, edit epoch)` pair** and suppresses the rest — that
//! suppression is what makes it an epoch oracle rather than a page count. It is
//! written by a different subsystem from the one under test
//! (`app::state`, called from `render::settle`), about a cache it owns, and it
//! is *silent* unless the epoch moved.
//!
//! So: no new `objects` line after the undo ⇒ `edit_epoch` did not change ⇒ the
//! decomposition, the page-text extraction, the font inventory and the canvas
//! selection are all still describing the revision the operator just left. The
//! planted build above passes every `listed=` assertion here and fails this one.
//!
//! Note what it is **not**: an assertion that `n` changed. An annotation is not
//! a content object, so `n` is the same before and after — which is exactly
//! why the *line's existence* is the signal and its value is not.
//!
//! ## D-render catches: *the epoch moved and the texture was kept*
//!
//! `render::worker` emits `render-spawn gen=… page=… scale=…` when a raster
//! starts. `settle_and_rasterize` keys the cached page texture on the page index
//! and the raster scale, and an undo changes **neither**, so nothing would ask
//! for a new raster unless `vector_edit`'s fourth step dropped the texture. This
//! is the one signal that speaks for the pixels an operator is actually looking
//! at, and it is independent of the epoch: a build that bumped the epoch and
//! kept the texture passes D-objects and fails here.
//!
//! It is a **one-directional** oracle and this file says so rather than
//! implying otherwise: a re-raster the harness did not cause — a resize, a
//! scroll, a strip page arriving — would also raise the count, so a spurious
//! spawn could let a non-dropping build through. Nothing here moves the window
//! or the scroll between the phases, and the failure it would produce is a
//! *false pass*, never a false failure.
//!
//! ## B and F catch: *the conditions were published unconditionally*
//!
//! `undo.available` and `redo.available` were absent from `app::conditions` for
//! the whole life of the project, with a comment saying so. The tempting way to
//! land them is to publish them beside `doc.open` and move on — which arms both
//! controls permanently, and looks identical to a working build in every phase
//! that presses one *after* an edit.
//!
//! B presses Undo when the log is empty and F presses Redo when the stack is,
//! and both read the **absence** of `ribbon-command-invoked` as the evidence.
//! That absence is admissible under `crate::checks`' rule 4 for the reason
//! `crate::checks::text_markup` states: *a greyed `egui` control does not emit
//! the event at all*, and the same control is shown to invoke, in the same run,
//! once its stack is non-empty. A run that never reached D would have proved
//! nothing by B alone.
//!
//! # Why the Comments panel is the oracle, and not a pixel
//!
//! `crate::checks::save_copy`'s argument, unchanged: a 2 pt rectangle over a CAD
//! drawing is a few hundred antialiased pixels among a page already full of thin
//! dark lines, and no threshold in this crate separates the two. A count that
//! moves in both directions is a far better oracle than a screenshot here — and
//! the two invalidation signals above are what stop the count from being the
//! *only* one.
//!
//! # Mouse only — `Ctrl+Z` is NOT driven
//!
//! Every gesture here is a real `SetCursorPos` + `mouse_event` on a QAT control
//! or the page. `Ctrl+Z`, `Ctrl+Y` and `Ctrl+Shift+Z` are **not driven here**,
//! and [`crate::checks::chords`] drives them instead.
//!
//! ★★ This block used to say they *could not* be pressed, because synthetic
//! keyboard input did not reach the window. That was false, and the two
//! sentences it went on to offer as compensation were false with it: the
//! `shell::manifest` keymap test swept only `Ctrl+<digit>`, and all three of
//! these chords were among **fourteen the manifest declared and the dispatcher
//! never dispatched**. Undo's primary route was dead, and the note reassuring
//! the reader about it was the reason nobody looked. They are covered by the
//! single `dispatch_command` every route shares — the same dispatcher this check drives
//! through the QAT — and the gap is stated rather than implied by a green
//! result.
//!
//! That matters more for this pair than for any other command in the shell,
//! because the keyboard is undo's *primary* route and the QAT is its secondary
//! one. What is proven here is the arm, the engine call and the invalidation;
//! what is not proven is that the chord reaches the arm.
//!
//! # Every way this reports SKIP, and why none of them is a pass
//!
//! * no binary, no `--pdf`, `--no-input` — the harness never began;
//! * the diagnostic switches did not reach the process;
//! * the page size could not be read and no `--page-size` was given;
//! * a mode segment, a tab or a control was never declared, or took no click;
//! * the QAT dropped controls for want of width (`ribbon-qat-controls-dropped`);
//! * the application never traced an `objects` line, so phase D has no epoch
//!   oracle to read;
//! * the fixture carries annotations the Comments panel excludes, so `listed=`
//!   would not move by exactly one.

use crate::checks::driving::{
    self, INVOKE_EVENT, ITEM_PREFIX, SHELL_DIAG_ENV, TAB_EVENT, UNIMPLEMENTED_EVENT, declared,
    declared_names, list, list_str, shell_trace,
};
use crate::checks::text_selection::aim;
use crate::checks::{Check, CheckContext};
use crate::coords::{DocPoint, PageGeometry};
use crate::error::{Error, Result};
use crate::geom::LRect;
use crate::input::Driver;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;
use crate::trace::Trace;

/// **Review** — the mode whose default arrangement mounts the Comments panel,
/// and whose tab list carries Markup.
///
/// `crate::checks::save_copy`'s choice, for its reason: a markup tool that works
/// in Review works in Edit, and this check's subject is not the tool. Undo
/// itself is mode-independent — it is on the QAT, which no mode hides — so the
/// mode here is entirely about reaching an *edit* to undo.
const MODE: &str = "review";

/// The tab carrying Rectangle.
const MARKUP_TAB: (&str, &str) = ("ribbon.tab.markup", "markup");

/// The tool that authors the annotation this check takes back.
///
/// Rectangle, because it is the shortest gesture that authors anything: one
/// drag, one release, no ending to press.
const RECTANGLE: (&str, &str) = ("ribbon.item.markup.rectangle", "markup.rectangle");

/// The Comments panel's control, used only as a fallback — see [`comments_count`].
const COMMENTS: (&str, &str) = ("ribbon.item.markup.comments", "markup.comments");

/// **The commands under test**, on the quick-access toolbar.
///
/// `ribbon.qat.` and not `ribbon.item.`: these two sit on **no tab**
/// (`shell::manifest`'s mode list says so in as many words), so the QAT is the
/// only surface a pointer can reach them on. That is also why this check never
/// switches tabs to press them — the QAT is drawn beside the tab strip in every
/// mode, so phases D and E press it with the Markup tab still active.
const UNDO: (&str, &str) = ("ribbon.qat.edit.undo", "edit.undo");

/// The other half of the pair.
const REDO: (&str, &str) = ("ribbon.qat.edit.redo", "edit.redo");

/// `markup-commit kind=… page=… …` — the shell decided to author one.
const COMMIT_EVENT: &str = "markup-commit";

/// `add-markup page=… n=… epoch=… …` — the engine authored it.
const APPLY_EVENT: &str = "add-markup";

/// `undo kind=… undo_depth=…` — the arm decided, naming what it is taking back.
const UNDO_EVENT: &str = "undo";

/// `redo kind=… undo_depth=…` — its twin.
const REDO_EVENT: &str = "redo";

/// `undo-applied page=… n=… epoch=… disclosures=…` — `vector_edit`'s line,
/// carrying **the epoch the undo produced**.
const UNDO_APPLIED_EVENT: &str = "undo-applied";

/// `redo-applied …` — its twin.
const REDO_APPLIED_EVENT: &str = "redo-applied";

/// `undo-declined reason=empty-stack` — the arm found nothing on the log.
///
/// Read only to improve a failure message. Its presence in phases D or E would
/// mean the click arrived, reached the arm, and the arm disagreed with the
/// condition that armed the control — a different fix from a click that never
/// arrived.
const UNDO_DECLINED_EVENT: &str = "undo-declined";

/// `redo-declined reason=empty-stack`.
const REDO_DECLINED_EVENT: &str = "redo-declined";

/// `comments-panel pages=… listed=N …` — the count oracle.
const COMMENTS_EVENT: &str = "comments-panel";

/// The field on [`COMMENTS_EVENT`] that counts the rows the panel drew.
const LISTED_FIELD: &str = "listed";

/// The field on [`COMMENTS_EVENT`] that counts annotations the panel left out.
const EXCLUDED_FIELD: &str = "excluded_total";

/// ★ `objects n=… page=… …` — **the epoch oracle**.
///
/// Emitted by `OpenDoc::trace_object_count` once per `(page index, edit epoch)`
/// pair and suppressed for every repeat, so a *new* line means the epoch moved.
/// See the module header for the build this catches and why the count `n` is
/// deliberately not the thing asserted on.
const OBJECTS_EVENT: &str = "objects";

/// The event the decomposition traces when a page will not decode.
///
/// Read for a SKIP reason: a fixture whose page cannot be decomposed emits this
/// instead of [`OBJECTS_EVENT`], and the epoch oracle is then unavailable for a
/// reason that has nothing to do with undo.
const OBJECTS_UNAVAILABLE_EVENT: &str = "objects-unavailable";

/// ★ `render-spawn gen=… page=… scale=…` — **the texture oracle**.
///
/// A raster starting. Nothing else asks for one after an undo, because the
/// texture's key carries only the page index and the raster scale and an undo
/// changes neither — so a new spawn means `vector_edit`'s fourth step ran.
const RENDER_SPAWN_EVENT: &str = "render-spawn";

/// `ribbon-qat-controls-dropped dropped=… of=…` — the QAT ran out of width.
///
/// The shell's own disclosure, read for a SKIP: a dropped control declares no
/// rect, and "there is nothing to aim at" would otherwise be reported as though
/// the command were missing.
const QAT_DROPPED_EVENT: &str = "ribbon-qat-controls-dropped";

/// The rectangle drag, in page fractions: `((x0, y0), (x1, y1))`, PDF user
/// space with y measured from the bottom.
///
/// `crate::checks::save_copy`'s drag, deliberately: two checks that author the
/// same annotation the same way are two independent readings of one gesture, and
/// a different rectangle here would add a variable neither of them is about.
const DRAG: ((f64, f64), (f64, f64)) = ((0.24, 0.28), (0.58, 0.46));

/// See the module documentation.
pub struct UndoRedoRoundTrip;

impl Check for UndoRedoRoundTrip {
    fn name(&self) -> &'static str {
        "undo_redo_round_trip"
    }

    fn defect(&self) -> &'static str {
        "Undo and Redo do nothing, or take the change off the session without invalidating what \
         the operator is looking at — the five-link chain from a quick-access click to a \
         re-rasterized page, three links of which no test in the workspace can observe"
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

/// How many times the shell has reported `id` invoked.
///
/// A **count**, never a presence: this check clicks the same two controls more
/// than once, and "has it ever been invoked?" would be answered `true` by a
/// click made ten seconds earlier — which is precisely the question phases B and
/// F must not ask.
fn invokes(session: &Session, id: &str) -> Result<usize> {
    Ok(shell_trace(session)?
        .events(INVOKE_EVENT)
        .filter(|l| l.get("id") == Some(id))
        .count())
}

/// The most recent `comments-panel` line's `listed=` count.
fn listed(trace: &Trace) -> Option<usize> {
    trace.last(COMMENTS_EVENT)?.get_usize(LISTED_FIELD)
}

/// The two invalidation signals, as a pair, at one moment.
///
/// Read together and compared together, because the failure they exist to catch
/// is *one of the two steps was omitted* and a check that read them at different
/// moments could not attribute a change to the phase that caused it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Invalidation {
    /// How many `objects` lines the application has traced — one per
    /// `(page, epoch)`.
    objects: usize,
    /// How many rasters it has started.
    rasters: usize,
}

impl Invalidation {
    /// Take a reading.
    fn of(trace: &Trace) -> Self {
        Self {
            objects: trace.events(OBJECTS_EVENT).count(),
            rasters: trace.events(RENDER_SPAWN_EVENT).count(),
        }
    }
}

/// Click a ribbon tab and confirm the shell reported it.
fn click_tab(
    session: &Session,
    driver: &Driver,
    ui_rect: &str,
    (region, id): (&str, &str),
) -> Result<()> {
    let trace = session.trace()?;
    let rect = declared(&trace, ui_rect, region).ok_or_else(|| {
        Error::new(format!(
            "the application declared no `{region}` region in `{MODE}`. Either this build does \
             not show that tab in this mode, or the tab strip is too narrow and it has moved into \
             the overflow menu — which this check cannot open. Tabs declared: {}.",
            list(&declared_names(&trace, ui_rect, "ribbon.tab."))
        ))
    })?;
    let before = shell_trace(session)?
        .events(TAB_EVENT)
        .filter(|l| l.get("tab") == Some(id))
        .count();
    driver.click_at(session.frame()?.declared_center(rect))?;
    session.settle(14);
    if shell_trace(session)?
        .events(TAB_EVENT)
        .filter(|l| l.get("tab") == Some(id))
        .count()
        <= before
    {
        return Err(Error::new(format!(
            "the click on `{region}` produced no new `{TAB_EVENT} tab={id}` line. The mode click \
             DID land, so pointer input works and this is not the input channel."
        )));
    }
    Ok(())
}

/// Locate a declared control and refuse a degenerate rectangle.
fn control(trace: &Trace, ui_rect: &str, name: &str, prefix: &str) -> Result<LRect> {
    let rect = declared(trace, ui_rect, name).ok_or_else(|| {
        let dropped = trace
            .last(QAT_DROPPED_EVENT)
            .map(|l| format!(" The shell reported `{}`.", l.raw))
            .unwrap_or_default();
        Error::new(format!(
            "no `{name}` rect was declared, so there is nothing to aim at.{dropped} Regions \
             declared under `{prefix}`: {}.",
            list(&declared_names(trace, ui_rect, prefix))
        ))
    })?;
    if !rect.is_substantial() {
        return Err(Error::new(format!(
            "`{name}` was declared at {rect:?}, which has no usable area. A click aimed at a \
             degenerate rectangle proves nothing."
        )));
    }
    Ok(rect)
}

/// Click a band control and confirm the shell reported the invoke.
fn click_command(
    session: &Session,
    driver: &Driver,
    ui_rect: &str,
    (region, id): (&str, &str),
    settle: u32,
) -> Result<()> {
    let rect = control(&session.trace()?, ui_rect, region, ITEM_PREFIX)?;
    let before = invokes(session, id)?;
    driver.click_at(session.frame()?.declared_center(rect))?;
    session.settle(settle);
    if invokes(session, id)? <= before {
        return Err(Error::new(format!(
            "`{region}` DID NOT TAKE THE CLICK: it was declared at {rect:?} and the click \
             produced no `{INVOKE_EVENT} id={id}`. Commands the shell reported invoked this run: \
             {}.",
            list_str(
                &shell_trace(session)?
                    .events(INVOKE_EVENT)
                    .filter_map(|l| l.get("id"))
                    .collect::<Vec<&str>>()
            )
        )));
    }
    Ok(())
}

/// Click a **quick-access** control and report whether the shell saw an invoke.
///
/// Returns `true` when the click produced a new `ribbon-command-invoked`, and
/// `false` when it did not — which is the answer phases B and F are asking for
/// and the reason this is not [`click_command`]. A greyed `egui` control takes
/// the click and emits nothing, so the two outcomes are both *expected results*
/// here rather than one being a failure to be raised from inside.
///
/// The rect is still required, and its absence is still a SKIP: a control that
/// was never drawn cannot have been clicked, and reading *that* silence as
/// "correctly greyed" would be the vacuous pass this crate's rule 4 exists to
/// forbid.
fn click_qat(
    session: &Session,
    driver: &Driver,
    ui_rect: &str,
    (region, id): (&str, &str),
    settle: u32,
) -> Result<bool> {
    let rect = control(&session.trace()?, ui_rect, region, "ribbon.qat.")?;
    let before = invokes(session, id)?;
    driver.click_at(session.frame()?.declared_center(rect))?;
    session.settle(settle);
    Ok(invokes(session, id)? > before)
}

/// Put the session into Review with the Comments panel showing, and report how
/// many annotations the panel can see.
///
/// `crate::checks::save_copy`'s function, and its finding is carried across
/// rather than rediscovered: the mode's own arrangement mounts the panel, and
/// the ribbon control is a **fallback** only, because on a wide sheet the Markup
/// tab's bands overflow and the Comments group is not drawn at all.
fn comments_count(
    session: &Session,
    driver: &Driver,
    ui_rect: &str,
    report: &mut CheckReport,
) -> Result<usize> {
    driving::click_mode_segment(session, driver, ui_rect, MODE)?;
    session.settle(16);

    if listed(&session.trace()?).is_none() {
        report.note(
            "the `review` arrangement did not mount the Comments panel; trying its ribbon control"
                .to_owned(),
        );
        click_tab(session, driver, ui_rect, MARKUP_TAB)?;
        click_command(session, driver, ui_rect, COMMENTS, 18)?;
    }

    let trace = session.trace()?;
    let count = listed(&trace).ok_or_else(|| {
        Error::new(format!(
            "the Comments panel never traced `{COMMENTS_EVENT}`, so this check has no oracle. \
             Either the panel is not mounted (check `app::modes::defaults`' `{MODE}` arrangement) \
             or it is drawing without tracing. Trace: {}.",
            session.trace_path().display()
        ))
    })?;
    let excluded = trace
        .last(COMMENTS_EVENT)
        .and_then(|l| l.get_usize(EXCLUDED_FIELD))
        .unwrap_or(0);
    if excluded > 0 {
        return Err(Error::new(format!(
            "the Comments panel excluded {excluded} annotation(s) — widgets, popups or trap nets, \
             which it leaves out by editorial rule. This check's verdict is that `{LISTED_FIELD}` \
             moves by exactly one in each direction, and on a document with excluded annotations \
             that arithmetic is measuring the panel's rules rather than the undo. Point --pdf at a \
             drawing without form fields."
        )));
    }
    report.note(format!(
        "baseline: the Comments panel lists {count} annotation(s) — `{}`",
        trace
            .last(COMMENTS_EVENT)
            .map_or_else(String::new, |l| l.raw.clone())
    ));
    Ok(count)
}

/// Launch one process with both diagnostic channels armed.
fn launch(ctx: &CheckContext, report: &mut CheckReport, pdf: &std::path::Path) -> Result<Session> {
    let mut spec = LaunchSpec::new(
        ctx.resolve_exe().ok_or_else(|| {
            Error::new(format!(
                "no binary to drive. Pass --exe, or build the profile's default at {}.",
                ctx.profile.default_exe
            ))
        })?,
        ctx.out("undo_redo.trace.txt"),
    );
    spec.pdf = Some(pdf.to_path_buf());
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
        "launched {} on {} as pid {}",
        spec.exe.display(),
        pdf.display(),
        session.pid()
    ));
    report.artifact(session.trace_path().to_path_buf());
    session.settle(40);

    if !session.trace()?.started(ctx.profile.vocab.start_event) {
        return Err(Error::new(format!(
            "the trace has no `{}` line, so the diagnostic switch {}={} did not reach the process \
             and this check has no oracle. Captured stderr is at {}.",
            ctx.profile.vocab.start_event,
            ctx.profile.diag_env.0,
            ctx.profile.diag_env.1,
            session.trace_path().display()
        )));
    }
    Ok(session)
}

/// One half of the round trip: press a history control and assert everything
/// that must follow from it.
///
/// Factored because phases D and E are the **same** six assertions in opposite
/// directions, and two hand-written copies would be two chances for one of them
/// to lose the invalidation half — which is the half no other test in the
/// workspace makes.
struct Step {
    /// The control to press.
    control: (&'static str, &'static str),
    /// The arm's own event: `undo` / `redo`.
    event: &'static str,
    /// `vector_edit`'s event, carrying the epoch: `undo-applied` / `redo-applied`.
    applied: &'static str,
    /// The empty-stack decline, read only to sharpen a failure message.
    declined: &'static str,
    /// What `listed=` must read afterwards.
    expect_listed: usize,
    /// Prose for the report.
    what: &'static str,
}

/// Drive one [`Step`]. `Ok(Some(_))` is a failure verdict.
fn history_step(
    session: &Session,
    driver: &Driver,
    ui_rect: &str,
    report: &mut CheckReport,
    step: &Step,
) -> Result<Option<String>> {
    let before = Invalidation::of(&session.trace()?);

    if !click_qat(session, driver, ui_rect, step.control, 30)? {
        return Ok(Some(format!(
            "★ `{}` TOOK NO CLICK AT {}. The control was declared, the click was delivered to its \
             centre, and the shell traced no `{INVOKE_EVENT} id={}` — which for a QAT control \
             means it was GREYED. Something is on the stack (the previous phase proved it), so \
             the predicate the ribbon evaluated disagrees with the session: look at \
             `app::conditions`, which must publish `undo.available` from \
             `EditSession::can_undo` and `redo.available` from `can_redo`.",
            step.control.0, step.what, step.control.1
        )));
    }

    let trace = session.trace()?;

    // --- the arm ran -------------------------------------------------------
    let Some(decided) = trace.last(step.event) else {
        return Ok(Some(format!(
            "★ `{}` WAS INVOKED AND NOTHING HAPPENED. The shell traced \
             `{INVOKE_EVENT} id={}` and there is no `{}` line. {}\nTrace: {}.",
            step.control.0,
            step.control.1,
            step.event,
            if trace
                .events(UNIMPLEMENTED_EVENT)
                .any(|l| l.get("id") == Some(step.control.1))
            {
                format!(
                    "★ The application traced `{UNIMPLEMENTED_EVENT} id={}`, which is \
                     `dispatch_command`'s fall-through: the command arrived and dispatch had NO \
                     ARM for it. That is the exact state this check was written against — look at \
                     `app/dispatch.rs`.",
                    step.control.1
                )
            } else {
                match trace.last(step.declined) {
                    Some(l) => format!(
                        "The arm ran and DECLINED: `{}`. The control was enabled and the arm found \
                         an empty stack, so `app::conditions` and \
                         `app::actions::apply`'s history arm are asking different questions.",
                        l.raw
                    ),
                    None => format!(
                        "There is no `{UNIMPLEMENTED_EVENT}` and no `{}` either, so the command \
                         reached an arm and the arm raised no action — look at whether \
                         `Action::Undo`/`Action::Redo` are matched in `PdfcerApp::apply`.",
                        step.declined
                    ),
                }
            },
            session.trace_path().display()
        )));
    };
    report.note(format!(
        "{}: the arm decided — `{}`",
        step.what, decided.raw
    ));

    // --- the engine ran, and said which revision it produced ---------------
    let Some(applied) = trace.last(step.applied) else {
        return Ok(Some(format!(
            "★ THE ARM DECIDED AND THE ENGINE NEVER RAN. `{}` is in the trace and there is no \
             `{}` line, so `vector_edit` was not reached — or it refused. A refusal traces \
             `{}-refused … reason=session-borrowed`, which means the render worker was not \
             cancelled before `Arc::get_mut`.",
            decided.raw, step.applied, step.applied
        )));
    };
    report.note(format!("{}: the engine ran — `{}`", step.what, applied.raw));

    // --- ★ the two invalidation signals ------------------------------------
    let after = Invalidation::of(&session.trace()?);
    if after.objects <= before.objects {
        return Ok(Some(format!(
            "★★ THE SESSION WAS CHANGED AND THE EPOCH DID NOT MOVE. `{}` ran, and \
             `OpenDoc::trace_object_count` — which emits one `{OBJECTS_EVENT}` line per \
             (page, edit epoch) pair and suppresses every repeat — traced no new one: {} lines \
             before the click and {} after.\n\n\
             THIS IS THE ASSERTION NO COUNT IN THIS CHECK CAN MAKE. The Comments panel walks the \
             session's annotations afresh every frame, so a build that mutated the session and \
             skipped `vector_edit`'s epoch bump satisfies every `{LISTED_FIELD}=` reading here — \
             while the operator's decomposition, page-text cache, font inventory and canvas \
             selection all go on describing the revision they just left.\n\
             Look at `app::actions::apply`'s `vector_edit`, step 3.",
            applied.raw, before.objects, after.objects
        )));
    }
    if after.rasters <= before.rasters {
        return Ok(Some(format!(
            "★★ THE PAGE WAS NOT RE-RASTERIZED. `{}` ran and `render::worker` started no new \
             raster: {} `{RENDER_SPAWN_EVENT}` lines before the click and {} after.\n\n\
             `settle_and_rasterize` keys the cached page texture on the page index and the raster \
             scale, and neither changes across a history step — so unless `vector_edit`'s fourth \
             step drops the texture, the page KEEPS DRAWING THE STATE THAT WAS JUST TAKEN BACK. \
             Every count in this check would still be correct; the pixels would not.\n\
             Look at `app::actions::apply`'s `vector_edit`, step 4.",
            applied.raw, before.rasters, after.rasters
        )));
    }
    report.note(format!(
        "{}: the epoch moved ({} → {} `{OBJECTS_EVENT}` lines) and the page re-rasterized \
         ({} → {} `{RENDER_SPAWN_EVENT}` lines)",
        step.what, before.objects, after.objects, before.rasters, after.rasters
    ));

    // --- the surface the operator reads moved with it ----------------------
    let now = listed(&session.trace()?);
    if now != Some(step.expect_listed) {
        return Ok(Some(format!(
            "★ THE COMMENTS PANEL DOES NOT AGREE. After {}, `{LISTED_FIELD}` should read {} and \
             it reads {:?}. The engine traced `{}`, so the command DID run — which makes this a \
             disagreement between the session and the panel rather than a history step that did \
             not happen.",
            step.what, step.expect_listed, now, applied.raw
        )));
    }
    report.note(format!(
        "{}: the Comments panel now lists {} annotation(s)",
        step.what, step.expect_listed
    ));
    Ok(None)
}

/// Run the sequence. `Err` is SKIP, `Ok(Some(_))` is FAIL, `Ok(None)` is a pass.
#[allow(clippy::too_many_lines)]
fn drive(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    // --- preconditions -----------------------------------------------------
    let pdf = ctx.pdf.clone().ok_or_else(|| {
        Error::new(
            "no --pdf. This check authors an annotation on a real page and then takes it back, so \
             it needs a document — and with nothing open both controls are correctly greyed and \
             this check would be measuring the gate rather than the feature.",
        )
    })?;
    if !ctx.allow_input {
        return Err(Error::new(
            "input is disabled (--no-input). This check is six clicks and a drag. Reported as \
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
    let page: PageGeometry = match ctx.page_size {
        Some((w, h)) => PageGeometry {
            width_pt: w,
            height_pt: h,
        },
        None => crate::fixture::page_geometry(&pdf).ok_or_else(|| {
            Error::new(format!(
                "cannot read a page size from {}. The harness needs the page box to turn this \
                 check's fractions into points. Pass --page-size WxH.",
                pdf.display()
            ))
        })?,
    };

    let session = launch(ctx, report, &pdf)?;
    let driver = Driver::new(session.window());

    // --- PHASE A: the baseline every later count is measured against -------
    let listed_before = comments_count(&session, &driver, ui_rect, report)?;

    // …and the epoch oracle has to exist before phases D and E can rest on it.
    // A fixture whose page will not decompose traces `objects-unavailable`
    // instead, and reading that silence as "the epoch did not move" would
    // report a fixture problem as an application defect.
    let trace = session.trace()?;
    if trace.events(OBJECTS_EVENT).count() == 0 {
        return Err(Error::new(format!(
            "the application has traced no `{OBJECTS_EVENT}` line, so phases D and E have no way \
             to see whether `edit_epoch` moved. {} Without it this check could only assert the \
             Comments count, which a build that skipped every cache invalidation would satisfy — \
             see this check's header. Trace: {}.",
            match trace.last(OBJECTS_UNAVAILABLE_EVENT) {
                Some(l) => format!(
                    "The decomposition reported it could not read the page: `{}`. Point --pdf at a \
                     document whose first page decodes.",
                    l.raw
                ),
                None => "There is no `objects-unavailable` line either, so the count was never \
                         attempted — check that `settle_and_rasterize` still calls \
                         `trace_object_count`."
                    .to_owned(),
            },
            session.trace_path().display()
        )));
    }

    // --- ★ PHASE B: Undo is GREYED with an empty command log ---------------
    //
    // Nothing has been edited, so `undo.available` must be unset and the
    // control must take the click and emit nothing. The absence is admissible
    // because phase D presses the same control again and requires the opposite;
    // a run that stopped here would have proved nothing.
    if click_qat(&session, &driver, ui_rect, UNDO, 12)? {
        return Ok(Some(format!(
            "★ UNDO IS LIVE WITH AN EMPTY COMMAND LOG. Nothing has been edited in this session, \
             and clicking `{}` produced `{INVOKE_EVENT} id={}` anyway — which a greyed `egui` \
             control cannot do.\n\n\
             The likely build: `app::conditions` publishes `undo.available` unconditionally \
             (beside `doc.open`, say) rather than from `EditSession::can_undo`. That build passes \
             every later phase of this check, because by then there really is something to undo. \
             {}",
            UNDO.0,
            UNDO.1,
            match session.trace()?.last(UNDO_DECLINED_EVENT) {
                Some(l) => format!(
                    "The arm then declined, as it must: `{}` — so the verb is right and the \
                     predicate is wrong.",
                    l.raw
                ),
                None => format!(
                    "The arm traced no `{UNDO_DECLINED_EVENT}`, so an empty log did not even \
                     decline — look at `app::actions::apply`'s history arm."
                ),
            }
        )));
    }
    // ★ **The note is deliberately NOT emitted here.** A silence is only
    // evidence once the channel that would have broken it is shown to work, and
    // at this point in the run nothing has proved that a *band* click lands —
    // only that a mode segment does. A run whose pointer stops reaching the
    // window after the mode click would otherwise print "correctly greyed"
    // about a control nothing was delivered to, which is `crate::checks` rule 4
    // broken in its own words. Observed once, on 2026-08-14, against a planted
    // build: every click after the mode segment was lost, and this line claimed
    // the greying as a finding on the way past.
    //
    // The verdict is recorded and worded at the END, after phase D has pressed
    // the same control and required the opposite.

    // --- PHASE C: author a rectangle on the page ---------------------------
    click_tab(&session, &driver, ui_rect, MARKUP_TAB)?;
    click_command(&session, &driver, ui_rect, RECTANGLE, 16)?;

    let from = aim(
        ctx,
        &session,
        page,
        DocPoint::new(0, DRAG.0.0 * page.width_pt, DRAG.0.1 * page.height_pt),
    )?;
    let to = aim(
        ctx,
        &session,
        page,
        DocPoint::new(0, DRAG.1.0 * page.width_pt, DRAG.1.1 * page.height_pt),
    )?;
    let applies_before = session.trace()?.events(APPLY_EVENT).count();
    driver.drag(from, to)?;
    session.settle(24);

    let trace = session.trace()?;
    let Some(commit) = trace
        .events(COMMIT_EVENT)
        .filter(|l| l.get("kind") == Some("Rectangle"))
        .last()
    else {
        return Ok(Some(format!(
            "THE DRAG AUTHORED NOTHING, so there is no change for this check to take back. That \
             is `markup_rectangle`'s subject rather than this one's — read its verdict first. \
             Trace: {}.",
            session.trace_path().display()
        )));
    };
    if trace.events(APPLY_EVENT).count() <= applies_before {
        return Ok(Some(format!(
            "THE ENGINE NEVER AUTHORED THE RECTANGLE. The application decided to author one — \
             `{}` — and no `{APPLY_EVENT}` line followed.",
            commit.raw
        )));
    }
    session.settle(10);
    let listed_after = listed(&session.trace()?).unwrap_or(listed_before);
    if listed_after != listed_before + 1 {
        return Ok(Some(format!(
            "THE COMMENTS PANEL DOES NOT SEE THE ANNOTATION THAT WAS JUST AUTHORED: it listed \
             {listed_before} before the drag and {listed_after} after it. The engine traced \
             `{APPLY_EVENT}`, so the annotation IS on the session — which makes this the panel's \
             finding rather than undo's, and it also means every later phase would be measuring \
             the panel."
        )));
    }
    report.note(format!(
        "the rectangle was authored (`{}`) and the Comments panel lists {listed_after}, one more \
         than the baseline — there is now exactly one change to take back",
        commit.raw
    ));

    // --- ★ PHASE D: UNDO ---------------------------------------------------
    if let Some(failure) = history_step(
        &session,
        &driver,
        ui_rect,
        report,
        &Step {
            control: UNDO,
            event: UNDO_EVENT,
            applied: UNDO_APPLIED_EVENT,
            declined: UNDO_DECLINED_EVENT,
            expect_listed: listed_before,
            what: "the undo",
        },
    )? {
        return Ok(Some(failure));
    }

    // --- ★ PHASE E: REDO ---------------------------------------------------
    if let Some(failure) = history_step(
        &session,
        &driver,
        ui_rect,
        report,
        &Step {
            control: REDO,
            event: REDO_EVENT,
            applied: REDO_APPLIED_EVENT,
            declined: REDO_DECLINED_EVENT,
            expect_listed: listed_before + 1,
            what: "the redo",
        },
    )? {
        return Ok(Some(failure));
    }

    // --- ★ PHASE F: the redo stack emptied, and the control followed -------
    //
    // Phase E consumed the only entry, so `redo.available` must now be unset.
    // The same control that invoked one press ago must now take a click and
    // emit nothing — which is the strongest form this suite has of "the
    // condition tracks the session", because the two readings are one click
    // apart on one control in one run.
    if click_qat(&session, &driver, ui_rect, REDO, 12)? {
        return Ok(Some(format!(
            "★ REDO IS STILL LIVE WITH AN EMPTY REDO STACK. The previous phase redid the only \
             entry there was, and clicking `{}` produced `{INVOKE_EVENT} id={}` again.\n\n\
             `redo.available` is therefore not being re-read from `EditSession::can_redo` each \
             frame — either it is published unconditionally, or it is latched. {}",
            REDO.0,
            REDO.1,
            match session.trace()?.last(REDO_DECLINED_EVENT) {
                Some(l) => format!("The arm declined, as it must: `{}`.", l.raw),
                None => format!(
                    "The arm traced no `{REDO_DECLINED_EVENT}` either, so an empty stack did not \
                     decline — which means something WAS redone twice."
                ),
            }
        )));
    }
    report.note(format!(
        "★★ ROUND TRIP PROVEN: {listed_before} → {} authored → {listed_before} undone → {} redone, \
         with a fresh `{OBJECTS_EVENT}` line and a fresh `{RENDER_SPAWN_EVENT}` at each step — so \
         the epoch moved and the page was re-rasterized, not merely the session mutated. `{}` was \
         greyed before the edit and `{}` is greyed after the redo",
        listed_before + 1,
        listed_before + 1,
        UNDO.0,
        REDO.0
    ));
    report.note(format!(
        "★ …and phase B's silence is evidence rather than an absence: `{}` was clicked at \
         its centre with an empty command log and the shell traced no `{INVOKE_EVENT}`, while \
         the SAME control clicked in phase D did — so the pointer reaches it, and a greyed \
         `egui` control is what swallowed the first press",
        UNDO.0
    ));
    report.note(
        "NOT covered here: `Ctrl+Z`, `Ctrl+Y`, `Ctrl+Shift+Z`. `checks::chords` presses all three and asserts each dispatches. Until 2026-08-18 this note said they COULD NOT be driven and were covered by the manifest keymap test; both halves were false, and all three were among fourteen declared chords the dispatcher ignored.",
    );
    report.note(format!(
        "NOT covered here: that the page's PIXELS show the undone state. `{RENDER_SPAWN_EVENT}` \
         proves a raster was asked for, which is the step that was in doubt; it does not prove \
         what came out. A pixel oracle was rejected for `save_copy`'s reason — a 2 pt rectangle \
         over a CAD drawing is a few hundred antialiased pixels among a page full of thin dark \
         lines, and no threshold in this crate separates the two"
    ));
    Ok(None)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The names this check greps for are the ones `egui-shell` builds, and the
    /// ids are the ones the application registers.
    ///
    /// Pinned for the reason every sibling check pins its own: the two crates
    /// are joined by a **string** and nothing else, so a rename would leave both
    /// sides compiling while every assertion here quietly stopped matching — and
    /// a check that matches nothing passes vacuously.
    #[test]
    fn the_selectors_match_the_shells_own_spelling() {
        for (region, id) in [RECTANGLE, COMMENTS] {
            assert_eq!(region, format!("ribbon.item.{id}"));
            assert!(region.starts_with(ITEM_PREFIX), "{region}");
        }
        // ★ The two commands under test are on the QAT and on NO tab, which is
        // why they are addressed under a different prefix. A build that moved
        // either onto a tab would break this — and it would also silently take
        // undo away from Read, which is what
        // `app::modes::capability`'s `a_command_on_no_tab_is_offered_by_every_mode`
        // exists to prevent from the other side.
        for (region, id) in [UNDO, REDO] {
            assert_eq!(region, format!("ribbon.qat.{id}"));
            assert!(!region.starts_with(ITEM_PREFIX), "{region}");
        }
        assert_eq!(MARKUP_TAB.0, format!("ribbon.tab.{}", MARKUP_TAB.1));
        assert_eq!(MODE, "review");
    }

    /// ★ **The two invalidation oracles are read as counts, and a count that
    /// did not move is a failure.**
    ///
    /// [`Invalidation`] is three lines of arithmetic and exactly the kind of
    /// thing that gets "simplified" into a presence test — which would pass
    /// against the build this check exists to catch, because an `objects` line
    /// from the document's *first* frame is present whether or not the undo
    /// produced a second one.
    #[test]
    fn the_invalidation_reading_counts_lines_rather_than_finding_one() {
        let text = "\
pdfcer-diag start argv1=None\n\
pdfcer-diag objects n=812 page=0 paths=700 text=100 images=12 forms=0\n\
pdfcer-diag render-spawn gen=1 page=0 scale=2\n\
pdfcer-diag add-markup page=0 n=1 epoch=1 disclosures=none\n\
pdfcer-diag objects n=812 page=0 paths=700 text=100 images=12 forms=0\n\
pdfcer-diag render-spawn gen=2 page=0 scale=2\n";
        let before = Invalidation {
            objects: 1,
            rasters: 1,
        };
        let after = Invalidation::of(&Trace::parse(text, "pdfcer-diag"));
        assert_eq!(
            after,
            Invalidation {
                objects: 2,
                rasters: 2
            }
        );
        assert!(after.objects > before.objects, "the epoch moved");
        assert!(after.rasters > before.rasters, "the texture was dropped");

        // ★ And the fixture is the hostile case on purpose: `n=812` is
        // IDENTICAL in both lines, because an annotation is not a content
        // object. A check that asserted on the count rather than on the line
        // would see nothing happen at all.
        let parsed = Trace::parse(text, "pdfcer-diag");
        let objects: Vec<&str> = parsed
            .events(OBJECTS_EVENT)
            .filter_map(|l| l.get("n"))
            .collect();
        assert_eq!(objects, ["812", "812"]);
    }

    /// The two application streams do not contaminate each other, and every
    /// field this check reads is read from the line the application really
    /// writes.
    #[test]
    fn the_application_and_shell_streams_do_not_contaminate_each_other() {
        let text = "\
pdfcer-diag start argv1=None\n\
egui-shell-diag ribbon-command-invoked id=edit.undo handler=490 surface=qat\n\
pdfcer-diag undo kind=AddAnnotation undo_depth=1\n\
pdfcer-diag undo-applied page=0 n=1 epoch=2 disclosures=none\n\
pdfcer-diag comments-panel pages=1 listed=12 with_note=0 excluded_total=0\n\
pdfcer-diag objects-unavailable page=0 reason=decompose-failed detail=\"bad stream\"\n";
        let app = Trace::parse(text, "pdfcer-diag");
        let shell = Trace::parse(text, driving::SHELL_TRACE_PREFIX);

        assert!(app.started("start"));
        assert!(
            app.events(INVOKE_EVENT).next().is_none(),
            "the shell's line must not be read as the application's"
        );
        assert!(
            shell
                .events(INVOKE_EVENT)
                .any(|l| l.get("id") == Some(UNDO.1))
        );
        assert_eq!(
            app.last(UNDO_EVENT).and_then(|l| l.get("kind")),
            Some("AddAnnotation"),
            "the arm's line is what names the operation the operator took back"
        );
        assert_eq!(
            app.last(UNDO_APPLIED_EVENT).and_then(|l| l.get("epoch")),
            Some("2"),
            "the engine's line is what carries the epoch the undo produced"
        );
        assert_eq!(listed(&app), Some(12));

        // ★ `objects` and `objects-unavailable` are two events, and the epoch
        // oracle must not count the second as the first. `Trace::events`
        // matches the event name exactly, and this pins that it stays exact —
        // a prefix match would make a page that will not decompose look like an
        // epoch that moved on every frame.
        assert_eq!(app.events(OBJECTS_EVENT).count(), 0);
        assert_eq!(app.events(OBJECTS_UNAVAILABLE_EVENT).count(), 1);
    }

    /// The drag is a real rectangle, well inside the page.
    ///
    /// Shared with `save_copy` on purpose — see [`DRAG`] — so this assertion is
    /// the same one, made twice, about one gesture.
    #[test]
    fn the_drag_is_a_real_rectangle_inside_the_page() {
        assert!((DRAG.0.0 - DRAG.1.0).abs() > 0.1);
        assert!((DRAG.0.1 - DRAG.1.1).abs() > 0.1);
        for (x, y) in [DRAG.0, DRAG.1] {
            assert!(
                (0.05..=0.95).contains(&x) && (0.05..=0.95).contains(&y),
                "({x}, {y}) is too close to the page edge to survive a margin"
            );
        }
    }
}
