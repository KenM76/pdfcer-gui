//! `deeper_rung_delete` — **Delete removes ONE line, ONE label or ONE corner
//! point, and leaves the rest of the object standing.**
//!
//! Three checks, one file, one drive shape. They are the driven half of the
//! 2026-09-05 work that wired `EditSession::delete_subpath`,
//! `delete_text_run` and `delete_node` — three verbs whose **move** twins had
//! been wired for a fortnight, so on a CAD export a line could be entered,
//! selected and dragged, and could not be removed. Delete at the Part and Node
//! rungs traced `canvas-delete-declined … reason=no-verb-for-rung` and did
//! nothing at all, silently.
//!
//! # ★★★ THE ASSERTION THAT MATTERS IS "THE OTHERS SURVIVE"
//!
//! Read this before changing anything below.
//!
//! A check that asserted *"something was deleted"* **would pass on the exact
//! bug this feature was built to prevent.** `Pass 32.0`'s own words:
//!
//! > *"on the operator's drawing **one text object holds all 237 dimension
//! > labels**, so deleting 'a label' deleted every one of them."*
//!
//! Those 237 are **pdf dimensions** — page content pdfcer reads and must not
//! silently alter (R8b Rule 15) — and a build that routed the Part rung to
//! `delete_objects` removes every label on the sheet, reports success, drops
//! the object count by one, and traces a perfectly healthy deletion. So the
//! verdict here is a **pair**:
//!
//! | must hold | what a wrong build does |
//! |---|---|
//! | `objects_after == objects_before` | drops by one — the whole object went |
//! | `parts_after == parts_before - 1` | unchanged (nothing happened) or collapses to 0 |
//!
//! Either half alone is satisfiable by a wrong build. `parts_` alone is
//! ambiguous after a whole-object delete, because deletion **renumbers**: the
//! index the caller held then names a different object, and asking it for a run
//! count answers about whatever moved into the slot.
//!
//! # The oracle
//!
//! One line, written by `app::actions::vector` after each of the three verbs:
//!
//! ```text
//! delete-text-run-applied page=0 object=12 part=3 \
//!   objects_before=41 objects_after=41 runs_before=237 runs_after=236
//! ```
//!
//! …plus the funnel's own `delete-text-run page=0 n=1 epoch=8 disclosures=none`,
//! which is what says the **engine** accepted it. Both are required and they
//! answer different questions: the first says the page still looks right, the
//! second says an edit really landed. A build that computed the census and never
//! reached `EditSession` writes the first and not the second.
//!
//! ★ The `-applied` suffix is not decoration. `check-trace-names.py` exists
//! because a module line sharing its first token with a funnel label is the one
//! `Trace::last` returns — three recorded instances, each of which made a driven
//! check report *"the verb did nothing"* about a verb that worked.
//!
//! # ⚠ HOW TO FALSIFY THESE CHECKS — do this before believing a PASS
//!
//! A check that has never been seen to fail is not evidence. The plant, and the
//! proof that the plant landed, in order:
//!
//! 1. **Copy the file aside first.** `cp crates/pdfcer-gui/src/canvas/deleting.rs
//!    /tmp/deleting.rs.bak`. **Never `git checkout` to undo it** — this project
//!    runs parallel tracks and that discards another track's uncommitted work.
//! 2. **Plant the defect this exists to catch**, in
//!    `canvas::deleting::part_rung`: replace the `Some(PartKind::Run)` arm's
//!    body with the Object-rung verb —
//!    `Ok(DeleteSubject::Objects { page, objects: vec![object] })`. That is the
//!    pre-2026-09-05 behaviour with the decline removed, i.e. the bug
//!    `Pass 32.0` was written against.
//! 3. **Prove the plant is in the artifact, not just in the source.** Rebuild
//!    (`cargo build --release -p pdfcer-gui`), then
//!    `grep -c delete-text-run-applied target/release/pdfcer-gui.exe` — the
//!    planted build should report **0** for the text-run label, because the arm
//!    that writes it is now unreachable. `touch` the source if the build is
//!    skipped; a stale binary is the commonest cause of a falsification that
//!    "did not reproduce".
//! 4. **Require the check's own `[FAIL]` line.** The exit code is *not* the
//!    verdict — a SKIP exits the same way a PASS does, and every one of these
//!    three SKIPs on a fixture that cannot exercise it. Grep the report for
//!    `[FAIL] deleting_a_label_leaves_the_other_labels_alone`. It must say the
//!    object count fell.
//! 5. **Restore from the byte copy**, `cp /tmp/deleting.rs.bak …`, rebuild, and
//!    confirm the check passes again.
//!
//! ★ A falsification that produces a SKIP has proved nothing. If step 4 finds
//! `[SKIP]`, the fixture is wrong before the plant is wrong — see the table
//! below.
//!
//! # Fixtures, and why each check needs a particular kind of thing
//!
//! | check | fixture | `--doc-point` | why |
//! |---|---|---|---|
//! | label | `D:/Dev/pdfTests/SW41177/SW41177.pdf` | `0,1140,62` | needs a text object holding **several** runs; on a one-run object `delete_text_run` correctly deletes the object and the check cannot tell right from wrong. Measured on that point: **18 runs**, page objects **5,903** |
//! | line | `fixtures/hole-in-a-big-object.pdf` | `0,336,500` | needs a path object holding **several** subpaths. Measured: **41** — a circle and forty unrelated segments in ONE object, which is the shape of the operator's own export |
//! | point | `fixtures/polyline-nodes.pdf` | `0,150,260` | needs a subpath with **three or more** anchors — `delete_node` refuses one that would leave fewer than two, correctly. Measured: **6** |
//!
//! ★★★ **The line rung's fixture was WRONG in the first version of this table
//! and the check said so rather than passing.** It named `polyline-nodes.pdf`
//! at `0,150,260`; that page is one path object holding **one** subpath, so the
//! delete committed correctly, took the whole object with it (which is right —
//! a path with no subpaths is not a smaller object but a meaningless one), and
//! the check SKIPPED with *"the object under --doc-point held 1 line(s), and
//! this check needs at least 2"*. That is the fixture guard doing its job: the
//! discrimination this check exists for is unavailable on a one-part object,
//! and reporting a PASS there would have been reporting nothing.
//!
//! Every one of them SKIPs rather than FAILs when it does not find what it
//! needs, which is the standing rule: a check that cannot establish its
//! precondition must not report on the property beyond it.
//!
//! # ✅ DRIVEN 2026-09-05 — and the first run found two things
//!
//! All three **PASS**. What the first run found, in the order it found it:
//!
//! 1. **The two shape rungs FAILED** — `canvas-delete-declined level=Part
//!    sel=1 reason=NoObjectModel`. Not *"no verb for the rung"*: the routing
//!    was there and the frame had simply never asked for the page's
//!    decomposition, because `canvas::interact` gated it on a hand-maintained
//!    list of **gesture outcomes** and Delete is a keystroke. Fixed by
//!    `canvas::modelneed`, whose header carries all four recurrences of that
//!    defect and the 531 ms measurement behind the fix's shape.
//! 2. **The label rung SKIPPED, and the check was the thing that was wrong** —
//!    it double-clicked, and a double-click on text opens a caret by the
//!    operator's own ruling (O70). See [`Rung::arms_the_points_tool`] for the
//!    trace lines and the route that does exist.

use crate::checks::driving::{self, SHELL_DIAG_ENV, click_mode_segment};
use crate::checks::{Check, CheckContext};
use crate::coords::{CanvasMapping, DocPoint, PageGeometry};
use crate::error::{Error, Result};
use crate::input::Driver;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;
use crate::sys::vk;
use crate::trace::Trace;

/// The mode whose canvas may select and edit page content.
const MODE: &str = "edit";
/// `canvas-selection via=… sel=… level=… first=…`.
const SELECTION_EVENT: &str = "canvas-selection";
/// `canvas-anchors total=… selected=…` — published once the Part rung is
/// entered on a path.
const ANCHORS_EVENT: &str = "canvas-anchors";
/// How many anchor marks the overlay publishes. Mirrors
/// `canvas::overlay::PUBLISHED_ANCHORS`.
const PUBLISHED_ANCHORS: usize = 6;

/// Which of the three rungs a run of this check exercises.
///
/// One enum rather than three copies of `drive`, because the three differ in
/// exactly four things — how deep to descend, which `-applied` label to read,
/// which count field carries the parts, and what to say when the fixture cannot
/// exercise it — and everything else is one sequence. Three copies would be
/// three places for the census pair to drift.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Rung {
    /// One show operator out of a text object — `delete_text_run`.
    Label,
    /// One subpath out of a path object — `delete_subpath`.
    Line,
    /// One anchor out of a subpath — `delete_node`.
    Point,
}

impl Rung {
    /// The trace label `app::actions::vector` writes the census under.
    const fn applied(self) -> &'static str {
        match self {
            Self::Label => "delete-text-run-applied",
            Self::Line => "delete-subpath-applied",
            Self::Point => "delete-node-applied",
        }
    }

    /// The funnel's own label — the line that says the **engine** accepted it.
    const fn funnel(self) -> &'static str {
        match self {
            Self::Label => "delete-text-run",
            Self::Line => "delete-subpath",
            Self::Point => "delete-node",
        }
    }

    /// The census field prefix: `runs`, `lines` or `points`.
    const fn unit(self) -> &'static str {
        match self {
            Self::Label => "runs",
            Self::Line => "lines",
            Self::Point => "points",
        }
    }

    /// The operator's word for the thing being removed, for failure prose.
    const fn thing(self) -> &'static str {
        match self {
            Self::Label => "label",
            Self::Line => "line",
            Self::Point => "corner point",
        }
    }

    /// How many descents past the first click reach the rung.
    ///
    /// One double-click enters the Part rung; a second descends to the Node
    /// rung. `canvas::selection::descend` is the rule.
    ///
    /// ★★★ **Zero for the label, and that is a fact about the program rather
    /// than a shortcut.** See [`Self::arms_the_points_tool`].
    const fn descents(self) -> usize {
        match self {
            Self::Label => 0,
            Self::Line => 1,
            Self::Point => 2,
        }
    }

    /// ★★★ **Whether this rung is reached with the Points tool rather than
    /// with a double-click**, and the answer is *only the label*.
    ///
    /// # The measurement that put this here
    ///
    /// This check was written assuming one double-click enters the Part rung
    /// on any object, as it does on a path. Driven for the first time on
    /// 2026-09-05 against `SW41177.pdf` at `0,1140,62`, it reported
    ///
    /// ```text
    /// [SKIP] the ladder is at `Object` and this check needs `Part`
    /// ```
    ///
    /// …and the trace said why, in the application's own words:
    ///
    /// ```text
    /// pdfcer-diag text-edit-caret kind=Edit page=0 run=424 len=29
    /// pdfcer-diag canvas-double-click-text via=descend
    /// ```
    ///
    /// **A double-click on a text object opens a caret and returns**, before
    /// the ladder is touched at all — `canvas::clicking`'s O70 arm, which is
    /// the operator's own ruling: *"double-clicking inside the bounding box
    /// should edit the text."* So the Part rung on text is not reachable by
    /// double-click, by design, and a check that keeps trying is measuring a
    /// gesture the program deliberately does not have.
    ///
    /// # The route that does exist
    ///
    /// The **Points** tool (`view.tool_node`, chord `A`, labelled *Points*
    /// because a draughtsman says point). `canvas::clicking`'s node-tool
    /// branch takes a click before every other claimant and calls
    /// `SelectionState::click_direct`, which lands on the Part rung whenever
    /// the probe found a part — and `provider::part_hits` dispatches on the
    /// object's kind, so on a text object a "part" **is** a show operator.
    /// One click, one label selected, no double-click anywhere near the caret.
    ///
    /// ⚠ So this is not the check being lenient: it is the check driving the
    /// gesture the program actually offers, which is the whole point of
    /// driving rather than unit-testing. The path rungs keep the double-click
    /// because that is *their* real route.
    const fn arms_the_points_tool(self) -> bool {
        matches!(self, Self::Label)
    }

    /// The rung name the application publishes on `canvas-selection level=`.
    const fn level(self) -> &'static str {
        match self {
            Self::Label | Self::Line => "Part",
            Self::Point => "Node",
        }
    }

    /// The smallest part count at which the check can tell a right build from a
    /// wrong one, and why.
    ///
    /// ★ **Two, not one, and for the Point rung three.** On a one-part object
    /// every one of these verbs correctly deletes the whole object — a painting
    /// operator with no path, or a `BT`…`ET` that shows nothing, is not a
    /// smaller object but a meaningless one — so a right build and a wrong build
    /// produce the *identical* census and the check has no discrimination at
    /// all. `delete_node` additionally refuses a subpath that would be left with
    /// fewer than two anchors, so its floor is three.
    const fn needs_parts(self) -> usize {
        match self {
            Self::Label | Self::Line => 2,
            Self::Point => 3,
        }
    }
}

/// See the module documentation.
pub struct DeletingALabelLeavesTheOtherLabelsAlone;

impl Check for DeletingALabelLeavesTheOtherLabelsAlone {
    fn name(&self) -> &'static str {
        "deleting_a_label_leaves_the_other_labels_alone"
    }

    fn defect(&self) -> &'static str {
        "Delete on a selected text run reaches no verb at all and silently does nothing — or, \
         worse, reaches `delete_objects` and removes every label on the sheet, because one \
         SolidWorks text object holds all 237 of them"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        report_for(self.name(), self.defect(), Rung::Label, ctx)
    }
}

/// See the module documentation.
pub struct DeletingALineLeavesTheRestOfTheShapeAlone;

impl Check for DeletingALineLeavesTheRestOfTheShapeAlone {
    fn name(&self) -> &'static str {
        "deleting_a_line_leaves_the_rest_of_the_shape_alone"
    }

    fn defect(&self) -> &'static str {
        "Delete on a selected subpath reaches no verb — its move twin `move_subpath` has been \
         wired since Pass 28.0, so the line can be dragged and not removed — and the only \
         Delete offered at that moment takes the whole drawing view, which on the measured \
         export is 1,194 subpaths"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        report_for(self.name(), self.defect(), Rung::Line, ctx)
    }
}

/// See the module documentation.
pub struct DeletingAPointLeavesTheRestOfTheLineAlone;

impl Check for DeletingAPointLeavesTheRestOfTheLineAlone {
    fn name(&self) -> &'static str {
        "deleting_a_point_leaves_the_rest_of_the_line_alone"
    }

    fn defect(&self) -> &'static str {
        "Delete on a selected anchor reaches no verb, so a point of a polyline can be nudged \
         (`move_node`, wired) and not removed (`delete_node`, not wired) — and when it is \
         wired, the disclosure it owes when a curve is discarded goes unsaid"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        report_for(self.name(), self.defect(), Rung::Point, ctx)
    }
}

/// The shared `run` body — build the report, drive, translate the three-way
/// return.
fn report_for(
    name: &'static str,
    defect: &'static str,
    rung: Rung,
    ctx: &CheckContext,
) -> CheckReport {
    let mut report = CheckReport::new(name, defect);
    match drive(ctx, &mut report, rung) {
        Ok(Some(failure)) => report.fail(failure),
        Ok(None) => report.pass(),
        Err(why) => report.from_error(&why),
    }
}

/// Run the sequence.
///
/// The three-way return is the SKIP/FAIL/PASS rule made structural: `Err` is a
/// precondition that was absent (SKIP), `Ok(Some(_))` is an assertion that did
/// not hold (FAIL), `Ok(None)` is a pass.
#[allow(clippy::too_many_lines)]
fn drive(ctx: &CheckContext, report: &mut CheckReport, rung: Rung) -> Result<Option<String>> {
    let vocab = &ctx.profile.vocab;
    let exe = ctx.resolve_exe().ok_or_else(|| {
        Error::new(format!(
            "no binary to drive. Pass --exe, or build the profile's default at {}.",
            ctx.profile.default_exe
        ))
    })?;
    let pdf = ctx.pdf.clone().ok_or_else(|| {
        Error::new(format!(
            "no --pdf. This check needs a document whose page 1 holds an object with at least \
             {} {}s in it. See the fixture table in this module's header.",
            rung.needs_parts(),
            rung.thing()
        ))
    })?;
    let target = ctx.target.ok_or_else(|| {
        Error::new(
            "no --doc-point. Pass PAGE,X,Y in PDF user space naming a point ON the object \
             whose part is to be removed. The harness deliberately has no default: a click on \
             blank paper is symptom-identical to a broken hit test.",
        )
    })?;
    if !ctx.allow_input {
        return Err(Error::new(
            "input is disabled (--no-input). This check descends the selection ladder with \
             real double-clicks and presses Delete as a real keystroke. Reported as SKIPPED \
             rather than passed: a check that did not run has learned nothing.",
        ));
    }
    let ui_rect = vocab.ui_rect_event.ok_or_else(|| {
        Error::new(format!(
            "the `{}` profile declares no ui-rect trace event, so this check cannot leave \
             Read mode — where a canvas click on content is refused by design.",
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

    // --- launch -------------------------------------------------------------
    let mut spec = LaunchSpec::new(&exe, ctx.out("deeper_rung_delete.trace.txt"));
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
    report.artifact(session.trace_path().to_path_buf());
    report.note(format!(
        "launched {} as pid {}",
        exe.display(),
        session.pid()
    ));
    session.settle(40);
    let driver = Driver::new(session.window());

    // --- 1: Edit ------------------------------------------------------------
    //
    // The shell's default mode is Read, where a canvas click on content is
    // refused BY DESIGN. A check that skipped this step would report the mode
    // gate as a selection defect — which `delete_key`'s own header records as
    // having happened.
    click_mode_segment(&session, &driver, ui_rect, MODE)?;
    session.settle(20);

    // --- 2: select the object, then descend to the rung ---------------------
    //
    // ★★★ The label rung arms the **Points** tool first, because a
    // double-click on text opens a caret and never touches the ladder. The
    // measurement that established that, and the trace lines it was read out
    // of, are on `Rung::arms_the_points_tool`.
    //
    // ★ Pressed BEFORE the click rather than after it: with the arrow armed,
    // the first click would select the whole text object and the Points tool's
    // own branch would then be entering an object it did not pick.
    if rung.arms_the_points_tool() {
        driver.press(vk::A)?;
        session.settle(10);
    }
    let trace = session.trace()?;
    let mapping = CanvasMapping::from_trace(&trace, vocab, page, target.page)?;
    let window_point = mapping.doc_to_window(DocPoint::new(target.page, target.x, target.y))?;
    let frame = session.frame()?;
    let at = frame.to_screen(window_point);
    driver.click_at(at)?;
    session.settle(12);

    // The precondition, asserted rather than assumed: without a selection, a
    // Delete that removes nothing is a mystery rather than a defect.
    let trace = session.trace()?;
    let selected = trace
        .last(SELECTION_EVENT)
        .and_then(|l| l.get_usize("sel"))
        .unwrap_or(0);
    if selected == 0 {
        return Err(Error::new(format!(
            "the click at document point ({:.1}, {:.1}) on page {} selected nothing, so the \
             ladder cannot be descended. That is either a --doc-point that is not on an \
             object or a broken hit test, and this harness cannot tell them apart — so it \
             declines to file either. Trace: {}.",
            target.x,
            target.y,
            target.page,
            session.trace_path().display()
        )));
    }

    for _ in 0..rung.descents() {
        driver.double_click_at(at)?;
        session.settle(16);
    }

    // ★ The Point rung needs an ANCHOR under the pointer, and the second
    // double-click above only guarantees the rung, not the anchor —
    // *"inside this part, nothing picked yet"* is a real state the ladder can
    // be in. So aim the last descent at a published anchor mark instead of at
    // the original point.
    if rung == Rung::Point {
        let trace = session.trace()?;
        if trace.last(ANCHORS_EVENT).is_none() {
            return Err(Error::new(format!(
                "no `{ANCHORS_EVENT}` line after two descents, so no anchors were drawn and \
                 the Node rung has nothing to pick. Aim --doc-point at a path object — \
                 `fixtures/polyline-nodes.pdf` at `0,150,260` is built for it. Trace: {}.",
                session.trace_path().display()
            )));
        }
        let mut landed = false;
        for n in 0..PUBLISHED_ANCHORS {
            // Re-read every iteration: descending re-lays the marks out, and a
            // rect read before the previous attempt names a place the anchor
            // has left. Third distinct cause of the same read-then-act bug in
            // this suite.
            let trace = session.trace()?;
            let Some(rect) = driving::declared(&trace, ui_rect, anchor_region(n)) else {
                continue;
            };
            let frame = session.frame()?;
            driver.click_at(frame.declared_center(rect))?;
            session.settle(14);
            if level_is(&session.trace()?, rung.level()) {
                report.note(format!("picked published anchor {n}"));
                landed = true;
                break;
            }
        }
        if !landed {
            return Err(Error::new(format!(
                "none of the {PUBLISHED_ANCHORS} published anchor marks could be clicked into \
                 the Node rung. Trace: {}.",
                session.trace_path().display()
            )));
        }
    }

    let trace = session.trace()?;
    if !level_is(&trace, rung.level()) {
        let seen = trace
            .last(SELECTION_EVENT)
            .and_then(|l| l.get("level").map(str::to_owned))
            .unwrap_or_else(|| "none".to_owned());
        return Err(Error::new(format!(
            "the ladder is at `{seen}` and this check needs `{}`, so the {} was never \
             selected and the Delete below would be testing the Object rung instead. On a \
             text object the Part rung is a show operator and on a path it is a subpath; a \
             point that lands on neither cannot descend. Trace: {}.",
            rung.level(),
            rung.thing(),
            session.trace_path().display()
        )));
    }
    report.note(format!(
        "the selection ladder is at the {} rung ({} selected)",
        rung.level(),
        selected
    ));

    // --- 3: press Delete, through the OS ------------------------------------
    let applied_before = trace.events(rung.applied()).count();
    let whole_object_before = trace.events("delete-objects").count();
    driver.press(vk::DELETE)?;
    session.settle(24);
    let after = session.trace()?;

    // --- 4: the verdict -----------------------------------------------------
    let Some(line) = after.events(rung.applied()).nth(applied_before) else {
        // ★ Two very different silences, and they must not wear the same
        // sentence. The pre-2026-09-05 build declined by name and did nothing;
        // a build that reached the Object rung's verb instead removed the whole
        // object and looks, from a count alone, like a working delete.
        let declined = after
            .events("canvas-delete-declined")
            .last()
            .map(|l| l.raw.clone());
        let whole = after.events("delete-objects").count() > whole_object_before;
        return Ok(Some(if whole {
            format!(
                "★★★ THE DEFECT: Delete at the {} rung reached `delete_objects` and removed \
                 the WHOLE object. This is the bug `Pass 32.0` was written against — on the \
                 operator's drawing one text object holds all 237 labels, so 'delete a label' \
                 deletes every one of them. The Part and Node rungs must route to \
                 `delete_subpath` / `delete_text_run` / `delete_node`; see \
                 `canvas::deleting::subject`.",
                rung.level()
            )
        } else {
            match declined {
                Some(raw) => format!(
                    "Delete removed nothing and the application said why: `{raw}`. If the \
                     reason is `no-verb-for-rung` this is the pre-2026-09-05 build, in which \
                     the deeper rungs reached no verb at all. Any other reason is a real \
                     refusal — read `canvas::deleting::Refusal` for what it means and whether \
                     the fixture can exercise this case."
                ),
                None => format!(
                    "Delete raised nothing and declined nothing: no `{}` line and no \
                     `canvas-delete-declined` line at all. The key never reached the ladder. \
                     Look at `canvas::keys`' guards — a focused widget, a mode without \
                     `edit_content`, or a text draft in flight all swallow it. Trace: {}.",
                    rung.applied(),
                    session.trace_path().display()
                ),
            }
        }));
    };
    report.note(format!("★ the delete committed: `{}`", line.raw));

    let field = |suffix: &str| {
        line.get_usize(&format!("{}_{suffix}", rung.unit()))
            .ok_or_else(|| {
                Error::new(format!(
                    "the `{}` line carries no `{}_{suffix}=` field, so the census this check \
                     is built on cannot be read. The line is: `{}`.",
                    rung.applied(),
                    rung.unit(),
                    line.raw
                ))
            })
    };
    let parts_before = field("before")?;
    let parts_after = field("after")?;
    let objects_before = line.get_usize("objects_before").unwrap_or(0);
    let objects_after = line.get_usize("objects_after").unwrap_or(0);

    // ★ The fixture check comes AFTER the delete, because the count that
    // decides it is on the applied line. A SKIP here means the document could
    // not exercise the case, not that the build is wrong — on a one-part object
    // every one of these verbs correctly deletes the whole object, so a right
    // build and a wrong build produce the identical census.
    if parts_before < rung.needs_parts() {
        return Err(Error::new(format!(
            "the object under --doc-point held {parts_before} {}(s), and this check needs at \
             least {} to tell a right build from a wrong one: on a one-part object every one \
             of these verbs CORRECTLY deletes the whole object, so both builds look the same. \
             Pick a point on an object with more parts — see the fixture table in this \
             module's header.",
            rung.thing(),
            rung.needs_parts()
        )));
    }

    // --- the pair. Both halves, and the object half first ------------------
    if objects_after != objects_before {
        return Ok(Some(format!(
            "★★★ THE OTHERS DID NOT SURVIVE. Removing one {} changed the page's object count \
             from {objects_before} to {objects_after} — the enclosing object went with it. \
             That is the defect this check exists for and it is the reason 'something was \
             deleted' is not an acceptable assertion: on the operator's drawing one text \
             object holds all 237 labels and one path object holds 1,194 subpaths. The line \
             was: `{}`.",
            rung.thing(),
            line.raw
        )));
    }
    if parts_after + 1 != parts_before {
        return Ok(Some(format!(
            "the {} count went {parts_before} → {parts_after}, and exactly one should have \
             gone. Equal means the engine refused and the funnel's `{}-refused` line carries \
             its reason; a drop of more than one means the verb removed more than it was \
             asked for. The line was: `{}`.",
            rung.unit(),
            rung.funnel(),
            line.raw
        )));
    }
    report.note(format!(
        "★★ one {} removed and the rest survived: {} {parts_before} → {parts_after}, page \
         objects {objects_before} → {objects_after} (unchanged)",
        rung.thing(),
        rung.unit()
    ));

    // --- and it reached the engine -----------------------------------------
    //
    // The census alone could in principle be produced by a shell that computed
    // the right numbers and never called `EditSession`. The funnel's line is
    // what says an edit landed: it is written only inside `vector_edit`'s `Ok`
    // arm, after the epoch is bumped.
    let Some(funnel) = after.last(rung.funnel()) else {
        return Ok(Some(format!(
            "the census says a {} was removed and no `{}` line followed, so the shell's own \
             numbers moved without an edit reaching `EditSession`. Trace: {}.",
            rung.thing(),
            rung.funnel(),
            session.trace_path().display()
        )));
    };
    report.note(format!("★★ the engine accepted it: `{}`", funnel.raw));

    // --- the disclosure it owes --------------------------------------------
    //
    // ★★ NOT an assertion, and deliberately. `delete_node`'s disclosure list is
    // non-empty **only when a curve was discarded with the point** — the
    // §8.5.2.2 `c`/`v`/`y` case — and whether the anchor this run happened to
    // pick sits on a curve is a fact about the fixture's content stream, not
    // about the build. Asserting it would make the check fail on a polygonal
    // drawing, which is most of them.
    //
    // What IS asserted is that the field exists at all: `vector_edit` writes
    // `disclosures=none` or the joined sentences, always, so its absence would
    // mean the funnel line was not the funnel's.
    match funnel.get("disclosures") {
        Some("none") => {
            report.note(format!(
                "the engine disclosed nothing, which for a {} means no curve was discarded",
                rung.thing()
            ));
        }
        Some(said) => {
            report.note(format!(
                "★★★ the engine disclosed a shape change and the status row carries it: \
                 `{said}`. This is the sentence rule 4 owes — re-adding a point does not \
                 bring the curve back."
            ));
        }
        None => {
            return Ok(Some(format!(
                "the `{}` line carries no `disclosures=` field: `{}`. Every edit through \
                 `app::actions::funnel::vector_edit` writes one, so this is not the funnel's \
                 line — most likely a module summary sharing its first token with the funnel \
                 label, which is the exact failure `check-trace-names.py` exists to prevent.",
                rung.funnel(),
                funnel.raw
            )));
        }
    }

    Ok(None)
}

/// Whether the most recent selection line reports `level=<name>`.
fn level_is(trace: &Trace, level: &str) -> bool {
    trace
        .last(SELECTION_EVENT)
        .and_then(|l| l.get("level"))
        .is_some_and(|seen| seen == level)
}

/// The region name for the `n`th drawn anchor mark. Mirrors
/// `canvas::overlay`'s published names, exactly as `bezier_handle` does.
fn anchor_region(n: usize) -> &'static str {
    const NAMES: [&str; PUBLISHED_ANCHORS] = [
        "canvas.anchor.0",
        "canvas.anchor.1",
        "canvas.anchor.2",
        "canvas.anchor.3",
        "canvas.anchor.4",
        "canvas.anchor.5",
    ];
    NAMES[n.min(PUBLISHED_ANCHORS - 1)]
}
