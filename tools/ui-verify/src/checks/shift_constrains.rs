//! `shift_constrains_a_resize` — **Shift preserves aspect**, driven end to end,
//! and proved by the *difference* between two drags in one process.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/shift_constrains.md`.

use crate::checks::driving::{self, SHELL_DIAG_ENV, click_mode_segment};
use crate::checks::{Check, CheckContext};
use crate::coords::{CanvasMapping, DocPoint, PageGeometry};
use crate::error::{Error, Result};
use crate::input::{Driver, Key};
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;

/// The mode whose canvas may select page content.
const MODE: &str = "edit";
/// `resize-commit grip=… sx=… sy=… ax=… ay=…` — the shell's own report.
const COMMIT_EVENT: &str = "resize-commit";
/// `resize-declined reason=…` — the six worded refusals.
const DECLINED_EVENT: &str = "resize-declined";
/// `constrain lock=…` — traced once per transition by `canvas::constrain`.
const CONSTRAIN_EVENT: &str = "constrain";
/// The region the selection outline publishes.
const OUTLINE_REGION: &str = "canvas.selection-outline";

/// How far the south-east grip travels on x, as a fraction of **the selection
/// box's own width**.
const DRAG_X_OF_SHAPE: f32 = 0.25;
/// How far the south-east grip travels on y, as a fraction of **the selection
/// box's own height**.
const DRAG_Y_OF_SHAPE: f32 = 0.04;

/// How different the two unconstrained factors must be before this check will
/// claim to have measured anything.
const MIN_DISCRIMINATION: f64 = 0.05;

/// See the module documentation.
pub struct ShiftConstrainsAResize;

impl Check for ShiftConstrainsAResize {
    fn name(&self) -> &'static str {
        "shift_constrains_a_resize"
    }

    fn defect(&self) -> &'static str {
        "Shift does not preserve aspect on a resize — *the* resize convention, present in every \
         program in the class, and absent from every drag in this shell until 2026-08-20. An \
         operator holding Shift gets a free-form resize and cannot tell whether the key did \
         anything"
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

/// One completed resize's two factors, read off `resize-commit`.
#[derive(Debug, Clone, Copy)]
struct Factors {
    sx: f64,
    sy: f64,
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
    // PINNED: `--pdf` and `--doc-point` are read and IGNORED here.
    //
    //
    // ⇒ `fixture::grip_gesture_target` holds the point and the reason. A
    // check whose subject cannot exist under an arbitrary aim must not be
    // steerable into a place where its subject does not exist.
    let (pdf, target) = crate::fixture::grip_gesture_target();
    if !pdf.is_file() {
        return Err(Error::new(format!(
            "the grip-gesture fixture is not at {}. Every check that drags a selection grip \
             needs it; `fixture::grip_gesture_target` says which point on it is the measurable \
             one, and what it cost to find out.",
            pdf.display()
        )));
    }
    report.note(format!(
        "--pdf and --doc-point are IGNORED: this check pins {} at page 0, 300, 500 — the \
         one place on that sheet where a grip drag can be measured",
        pdf.display()
    ));
    if !ctx.allow_input {
        return Err(Error::new(
            "input is disabled (--no-input). This check clicks a mode segment, clicks page \
             content and performs two grip drags, one of them with Shift held. Reported as \
             SKIPPED rather than passed.",
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

    let mut spec = LaunchSpec::new(&exe, ctx.out("shift-constrains.trace.txt"));
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

    // --- 1: Edit, the one mode whose canvas selects content ----------------
    click_mode_segment(&session, &driver, ui_rect, MODE)?;
    session.settle(20);

    // --- 2: select the shape -----------------------------------------------
    let trace = session.trace()?;
    let mapping = CanvasMapping::from_trace(&trace, vocab, page, target.page)?;
    let window_point = mapping.doc_to_window(DocPoint::new(target.page, target.x, target.y))?;
    let frame = session.frame()?;
    let at = frame.to_screen(window_point);
    driver.click_at(at)?;
    session.settle(16);

    let trace = session.trace()?;
    let selected = trace
        .last(vocab.click_event)
        .and_then(|l| l.get_usize(vocab.click_selection_field))
        .or_else(|| {
            trace
                .last(vocab.canvas_event)
                .and_then(|l| l.get_usize(vocab.canvas_selection_field))
        });
    if selected == Some(0) {
        return Err(Error::new(format!(
            "the click at (page {}, {:.1}, {:.1}) selected nothing, so there are no grips to \
             drag. A fact about the fixture and the point, not about the constraint — aim at a \
             shape. SKIPPED rather than FAILED for exactly that reason.",
            target.page + 1,
            target.x,
            target.y
        )));
    }
    report.note("the click selected a shape, so the outline and its grips are drawn");

    // --- 3: the unconstrained drag, which is the CONTROL --------------------
    let free = match one_drag(&session, &driver, ui_rect, None)? {
        Ok(f) => f,
        Err(why) => return Ok(Some(why)),
    };
    report.note(format!(
        "unmodified: sx={:.4} sy={:.4} — the control",
        free.sx, free.sy
    ));
    let spread = (free.sx - free.sy).abs();
    if spread < MIN_DISCRIMINATION {
        return Err(Error::new(format!(
            "the unconstrained drag produced sx={:.4} and sy={:.4}, which differ by only \
             {spread:.4}. This check proves a constraint by the DIFFERENCE between a free drag \
             and a locked one, so a control run that is already square cannot discriminate: a \
             build that ignored Shift entirely would pass. The travel is \
             {DRAG_X_OF_SHAPE}×{DRAG_Y_OF_SHAPE} of the box's OWN extents, so a square pair \
             means the drag never reached the grip, not that the shape swallowed the \
             lopsidedness — a fact about the fixture and the aim, not about the \
             build, so it is SKIPPED rather than passed. The box's aspect ratio is no longer \
             one of the candidates: since 2026-08-29 the travel is expressed in the box's own \
             space, so a tall box and a wide one produce the same pair of factors.",
            free.sx, free.sy
        )));
    }

    // Put the shape back before measuring again. Without this the second drag
    // starts from the *resized* box, so its factors are relative to different
    // extents and the two runs are not comparable — the check would then be
    // asserting something true about two different objects.
    driver.press_chord(&[crate::sys::vk::CONTROL], crate::sys::vk::Z)?;
    session.settle(20);

    // --- 4: the same travel, with Shift held throughout ---------------------
    let locked = match one_drag(&session, &driver, ui_rect, Some(Key::Shift))? {
        Ok(f) => f,
        Err(why) => return Ok(Some(why)),
    };
    report.note(format!(
        "★ with Shift: sx={:.4} sy={:.4}",
        locked.sx, locked.sy
    ));

    // --- 5: assertion 2 — the two factors are the same ----------------------
    //
    // Compared against a tolerance rather than for exact equality: both numbers
    // arrive through a `{:.4}` format in the trace, so two values that ARE the
    // same `f32` can print a unit apart in the last place.
    if (locked.sx - locked.sy).abs() > 1e-3 {
        let shot = ctx.out("shift-constrains-aspect.png");
        if crate::capture::window_to_png(&session, &shot).is_ok() {
            report.artifact(shot);
        }
        return Ok(Some(format!(
            "★ SHIFT DID NOT PRESERVE ASPECT. The same drag — {DRAG_X_OF_SHAPE}×\
             {DRAG_Y_OF_SHAPE} of the selection box's own extents — \
             gave sx={:.4} sy={:.4} unmodified and sx={:.4} sy={:.4} with Shift held — the two \
             are the same shape of answer, so the modifier changed nothing.\n\
             Look at `canvas::interact`'s `GestureOutcome::Resize` arm: the flag reaches \
             `resizing::Frame::constrain`, and `resizing::drag` applies \
             `constrain::aspect` between `factors` and the in-flight return. If the lock were \
             applied at the CALL SITE instead, the ghost would be constrained and the commit \
             would not — which is this failure exactly. Trace: {}.",
            free.sx,
            free.sy,
            locked.sx,
            locked.sy,
            session.trace_path().display()
        )));
    }

    // --- 6: assertion 3 — it kept the DOMINANT factor -----------------------
    //
    // The travel is x-dominant, so the kept factor must be the control run's
    // `sx`. A build that averaged the pair, or that took `sy`, or that took the
    // factor closer to unity, satisfies assertion 2 and fails here.
    if (locked.sx - free.sx).abs() > 0.02 {
        return Ok(Some(format!(
            "★ SHIFT LOCKED THE PROPORTION TO THE WRONG FACTOR: it kept {:.4}, and the drag's \
             dominant axis produced {:.4} unmodified.\n\
             `constrain::aspect` keeps the factor FURTHER FROM UNITY, which is the same thing \
             as the axis the pointer travelled furthest along relative to the box's own \
             extent. Keeping the other one shrinks a shape the operator was enlarging. \
             Averaging the pair — the other plausible wrong answer — lands between the two and \
             fails this same assertion. Trace: {}.",
            locked.sx,
            free.sx,
            session.trace_path().display()
        )));
    }
    report.note(format!(
        "★★ the locked resize kept the dominant factor: {:.4}, the same the free drag produced \
         on x",
        locked.sx
    ));

    // --- 7: assertion 4 — and the operator was TOLD -------------------------
    let trace = session.trace()?;
    let announced = trace
        .events(CONSTRAIN_EVENT)
        .filter_map(|l| l.get("lock").map(str::to_owned))
        .any(|l| l == "Aspect");
    if !announced {
        let shot = ctx.out("shift-constrains-caption.png");
        if crate::capture::window_to_png(&session, &shot).is_ok() {
            report.artifact(shot);
        }
        return Ok(Some(format!(
            "the aspect lock WORKED and was never announced: no `{CONSTRAIN_EVENT} lock=Aspect` \
             line was traced.\n\
             `drag-moves` D5 has two clauses and this is the second — *the affordance shows the \
             constraint while it is active* — whose stated failure mode is an operator who \
             holds Shift, gets a result they did not expect, and cannot tell whether the \
             modifier did anything. `constrain::resize` is what announces; a caller that passed \
             the raw modifier straight to `Frame::constrain` would behave correctly and say \
             nothing. Trace: {}.",
            session.trace_path().display()
        )));
    }
    report.note("★ and it was announced — `constrain lock=Aspect` reached the status row");
    Ok(None)
}

/// Perform one south-east grip drag and return the factors it committed.
fn one_drag(
    session: &Session,
    driver: &Driver,
    ui_rect: &'static str,
    modifier: Option<Key>,
) -> Result<std::result::Result<Factors, String>> {
    let trace = session.trace()?;
    let outline = driving::declared(&trace, ui_rect, OUTLINE_REGION).ok_or_else(|| {
        Error::new(format!(
            "the application declared no `{OUTLINE_REGION}` region, so the harness does not \
             know where the grips are. It refuses to guess: a guessed grip position lands \
             inside the object, which is a MOVE drag, and this check would then measure the \
             wrong gesture."
        ))
    })?;
    let frame = session.frame()?;
    // `declared_at(1.0, 1.0)` — the bottom-right corner, where `handles` centres
    // the south-east grip. Not the centre: that is `Grip::Move`.
    let from = frame.declared_at(outline, 1.0, 1.0);
    let to = frame.declared_at(outline, 1.0 + DRAG_X_OF_SHAPE, 1.0 + DRAG_Y_OF_SHAPE);
    // A mid-point so the drag passes through frames where the constraint is
    // live rather than teleporting from press to release. `drag_via`'s own
    // header makes the same argument for holding the modifier throughout.
    let via = frame.declared_at(
        outline,
        1.0 + DRAG_X_OF_SHAPE / 2.0,
        1.0 + DRAG_Y_OF_SHAPE / 2.0,
    );
    // The grip must still be reachable with a real cursor. A fraction of a
    // very wide box can put the release point off the window, where the driver
    // clamps and the drag measured is not the drag asked for — a harness fault
    // that would read as a program one. `client_logical` is the same space
    // `ui-rect` publishes in, so this compares like with like.
    let window = frame.client_logical();
    let release_x = outline.max.x + DRAG_X_OF_SHAPE * (outline.max.x - outline.min.x);
    let release_y = outline.max.y + DRAG_Y_OF_SHAPE * (outline.max.y - outline.min.y);
    if release_x > window.max.x || release_y > window.max.y {
        return Err(Error::new(format!(
            "a {DRAG_X_OF_SHAPE} × {DRAG_Y_OF_SHAPE} fraction of this selection box releases at \
             ({release_x:.0}, {release_y:.0}), which is outside the {:.0} × {:.0} pt window. \
             The driver would clamp the cursor and the factors committed would be smaller than \
             the ones asked for — a fact about the harness, so it is SKIPPED rather than \
             reported as a constraint that did not hold. Aim at a smaller shape, or give the \
             window more room.",
            window.max.x, window.max.y
        )));
    }

    let before = session.trace()?.events(COMMIT_EVENT).count();
    driver.drag_via(
        from,
        via,
        std::time::Duration::from_millis(60),
        to,
        modifier,
    )?;
    session.settle(30);

    let trace = session.trace()?;
    let Some(commit) = trace.events(COMMIT_EVENT).nth(before) else {
        let declined = trace
            .events(DECLINED_EVENT)
            .filter_map(|l| l.get("reason").map(str::to_owned))
            .last();
        return Ok(Err(match declined {
            Some(reason) => format!(
                "the grip drag was DECLINED: reason={reason}. `NotAPath` means the point aimed \
                 at text or a picture; `ManyObjects` means the click selected more than one. \
                 Both are honest refusals and neither is what this check is for. Trace: {}.",
                session.trace_path().display()
            ),
            None => format!(
                "the grip drag committed nothing and declined nothing — the state the whole \
                 resize feature is a fix for. Before asking about the constraint, look at \
                 `resize_scales_a_shape`, which measures that alone. Trace: {}.",
                session.trace_path().display()
            ),
        }));
    };
    // A missing or unparsable field answers 0.0, which fails every assertion
    // downstream — the safe direction: a check that could not read a number must
    // not report that the number was right.
    Ok(Ok(Factors {
        sx: commit.get("sx").and_then(|v| v.parse().ok()).unwrap_or(0.0),
        sy: commit.get("sy").and_then(|v| v.parse().ok()).unwrap_or(0.0),
    }))
}
