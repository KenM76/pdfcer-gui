//! `a_click_selects_the_whole_drawing_and_a_double_click_goes_inside` —
//! **Smart-Selector, driven on a real wrapped CAD sheet.**
//!
//! # The request
//!
//! `OPERATOR_REQUESTS.md` **O70**, 2026-08-31:
//!
//! > *"if a click selects an object that is made of multiple objects (group,
//! > form, etc) a double click should bring me further down the chain … If I
//! > recall this is similar to how Inscape does things and we should follow
//! > that convention."*
//!
//! ## ★★★ The trace field this whole check turns on
//!
//! `canvas-selection … first=` says **which of two index spaces** the selection
//! landed in — `object:N` for a page object, `leaf:N` for something painted
//! inside a form XObject. It exists because `sel=` and `level=` cannot tell
//! them apart:
//!
//! ```text
//! canvas-selection via=click mod=false sel=1 level=Object first=object:12   ← the container
//! canvas-selection via=enter-form mod=true sel=1 level=Object first=leaf:1180 ← inside it
//! ```
//!
//! Both are `sel=1 level=Object`. A check reading only those would pass on the
//! build this feature replaces **and** on the one it introduces, which is this
//! project's own definition of measuring nothing.
//!
//! ## What the two clicks must produce, and why that order is the feature
//!
//! | # | gesture | oracle |
//! |---|---|---|
//! | A | one click on drawing content | `first=object:N` — the **container** |
//! | B | double-click at the same point | `smart-enter`, then `first=leaf:M` |
//! | C | Escape, Escape | `canvas-escape outcome=LeftContainer` |
//!
//! ★★ Step A is the half that sounds backwards and is the actual change. Before
//! this feature a click selected the **leaf** — the engine excludes forms from a
//! deep hit test, so the interior was all a click could reach and the wrapped
//! drawing itself was unselectable except through a Format-tab command. So a
//! build with the feature missing fails step A, not step B: it goes straight to
//! `first=leaf:…` on the first click.
//!
//! ★ Step C is two presses, not one, and the count is the assertion. `canvas::keys`
//! puts the container **below** the selection on the Escape ladder — one press
//! clears what is selected, a second steps out — because the selection is the
//! more transient of the two. A build that leaves on the first press would strand
//! an operator who pressed Escape to drop a selection outside the container they
//! were working in.
//!
//! ## ★★★ Why it opens its OWN fixture and ignores `--pdf`
//!
//! Because the subject needs a page whose visible content is painted from
//! inside a form, and **neither real drawing can provide one at fit zoom**.
//! Measured, 2026-08-31:
//!
//! | document | forms | what a driven click selected |
//! |---|---|---|
//! | `SW41177.pdf` | **none** — `/Subtype /Form` appears zero times | a page object, correctly; nothing to enter, ever |
//! | `ncored-benchmark-cad-drawing.pdf` | one, over **10,256 leaves** and 129,758 page objects | a page object at all three points tried: 119703, 1528, 64850 |
//!
//! The second is the instructive one. Points were chosen by asking
//! `hit_test_point_deep` directly, with a 3 pt tolerance, and every one of them
//! still selected a page object when driven — because the shell asks with
//! `SELECT_SCREEN_TOLERANCE_PX` converted **at the current zoom**, and that
//! sheet opens at about 0.39×, making six screen pixels roughly fifteen points
//! of page. At that radius the big page objects win everywhere. The leaves that
//! do survive a tight probe are 4 × 6 pt glyph strokes — a pixel and a half.
//!
//! ⇒ So the feature is reachable by an operator who has zoomed in and
//! unreachable by a harness aiming at a page opened to fit. That makes the
//! DOCUMENT the wrong instrument, not the feature wrong, and the answer is
//! `tools/gen-form-xobject-fixture.py`: a 400 × 300 pt page whose entire
//! content is one `Do` on a form holding three fat crossing strokes. Its header
//! carries the measurements above and why each dimension is what it is.
//!
//! ★ The real drawings keep their checks — this suite drives them for
//! everything whose subject IS a real drawing. This one's subject is a
//! containment relationship, and a fixture states it exactly.

use crate::checks::driving::{SHELL_DIAG_ENV, click_mode_segment, declared, declared_names, list};
use crate::checks::text_selection::aim;
use crate::checks::{Check, CheckContext};
use crate::coords::{DocPoint, PageGeometry};
use crate::error::{Error, Result};
use crate::input::Driver;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;

/// A fixture in **this** repository, or `None` if it has not been generated.
///
/// `CARGO_MANIFEST_DIR` is `tools/ui-verify`, so callers pass a path two levels
/// up. Its own constant carries the trap that makes that worth stating.
fn local_fixture(rel: &str) -> Option<std::path::PathBuf> {
    let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(rel);
    path.is_file().then_some(path)
}

/// The mode this is about: content selection needs it.
const MODE: &str = "edit";
/// The selection line, and the only oracle that distinguishes the two spaces.
const SELECTION: &str = "canvas-selection"; // ui-text-exempt: a trace event name, never displayed
/// The line `canvas::smart::enter` writes.
const ENTER: &str = "smart-enter"; // ui-text-exempt: a trace event name, never displayed
/// The line the Escape ladder writes when it steps back out.
const ESCAPE: &str = "canvas-escape"; // ui-text-exempt: a trace event name, never displayed
/// The page region, so a failure can say whether a sheet was drawn at all.
const PAGE_REGION: &str = "page"; // ui-text-exempt: a trace region name, never displayed
/// **The fixture this check opens**, relative to `CARGO_MANIFEST_DIR` — which
/// is `tools/ui-verify`, so **two** levels up and not three.
///
/// `form_groups` records that trap: written with a third `../` it resolves to a
/// `D:/Dev/fixtures/` that does not exist, and the check SKIPs on every run
/// while telling the reader to run a generator that writes somewhere else.
const FIXTURE: &str = "../../fixtures/form-xobject.pdf";
/// The page, in points, as the generator writes it.
const FIXTURE_PAGE: PageGeometry = PageGeometry {
    width_pt: 400.0,
    height_pt: 300.0,
};
/// Where to click: the middle of the horizontal bar inside the form.
///
/// The form is placed at `(40, 40)` and the bar runs at `y = 110` in the form's
/// own space, so it is at `y = 150` on the page. `x = 100` is well clear of the
/// vertical bar at `x = 200` and of the diagonal, so a click here can only mean
/// one of the three strokes.
const POINT: (f64, f64) = (100.0, 150.0);

pub struct AClickSelectsTheWholeDrawing;

impl Check for AClickSelectsTheWholeDrawing {
    fn name(&self) -> &'static str {
        "a_click_selects_the_whole_drawing_and_a_double_click_goes_inside"
    }

    fn defect(&self) -> &'static str {
        "a click on a CAD sheet selects one line inside the wrapped drawing rather than the \
         drawing, so the container is unreachable by pointer at all — and there is no gesture \
         that goes deeper on purpose, only one that has already gone as deep as it can"
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

fn drive(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    if !ctx.allow_input {
        return Err(Error::new(
            "input is disabled (--no-input). This check is two clicks and two keystrokes on the \
             page.",
        ));
    }
    let exe = ctx.resolve_exe().ok_or_else(|| {
        Error::new(format!(
            "no binary to drive. Pass --exe, or build the profile's default at {}.",
            ctx.profile.default_exe
        ))
    })?;
    // ★★★ Its own fixture, and `--pdf` is deliberately ignored. See the
    // header: two real drawings were driven first, and the measurement that
    // ruled both out is what this fixture exists to answer.
    let pdf = local_fixture(FIXTURE).ok_or_else(|| {
        Error::new(format!(
            "the fixture {FIXTURE} is missing. Run `python tools/gen-form-xobject-fixture.py`; \
             its header says what the file must contain and why neither real drawing can stand \
             in for it."
        ))
    })?;
    let ui_rect = ctx
        .profile
        .vocab
        .ui_rect_event
        .ok_or_else(|| Error::new("the profile declares no ui-rect trace event."))?;
    // ★ Read from the FILE rather than taken from the constant above, so a
    // regenerated fixture at another size makes this check aim correctly rather
    // than silently at the wrong place. The constant is the fallback, and the
    // two disagreeing is a fact worth surfacing in the note below.
    let page = crate::fixture::page_geometry(&pdf).unwrap_or(FIXTURE_PAGE);

    let mut spec = LaunchSpec::new(&exe, ctx.out("smart-select.trace.txt"));
    spec.pdf = Some(pdf);
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
    session.settle(45);
    let driver = Driver::new(session.window());

    click_mode_segment(&session, &driver, ui_rect, MODE)?;
    if declared(&session.trace()?, ui_rect, PAGE_REGION).is_none() {
        return Err(Error::new(format!(
            "no `{PAGE_REGION}` region, so no sheet is on screen. Regions beginning `page`: {}.",
            list(&declared_names(&session.trace()?, ui_rect, "page"))
        )));
    }

    // --- A: one click must select the CONTAINER ----------------------------
    let at = aim(ctx, &session, page, DocPoint::new(0, POINT.0, POINT.1))?;
    driver.click_at(at)?;
    session.settle(30);

    let trace = session.trace()?;
    let Some(first) = trace
        .last(SELECTION)
        .and_then(|l| l.get("first").map(str::to_owned))
    else {
        return Ok(Some(format!(
            "THE CLICK SELECTED NOTHING: no `{SELECTION}` line after clicking the point this run \
             was given. The point may be on blank paper — which this check cannot tell from a \
             broken hit test, and is why `--doc-point` has no default. Trace: {}.",
            session.trace_path().display()
        )));
    };
    if first == "none" {
        return Err(Error::new(format!(
            "the click at ({:.0}, {:.0}) selected nothing (`first=none`), so this fixture has no \
             stroke there. SKIPPED rather than failed: that is a fact about the fixture, not \
             about the feature. Regenerate it with `python tools/gen-form-xobject-fixture.py`, \
             whose header states where the three strokes are.",
            POINT.0, POINT.1
        )));
    }
    if first.starts_with("leaf:") {
        return Ok(Some(format!(
            "★★★ THE CLICK WENT STRAIGHT INSIDE: `{SELECTION} … first={first}`.\n\
             A single click selected an object painted INSIDE a form XObject rather than the \
             form itself, which is the behaviour O70 replaces — the container is then \
             unreachable by pointer at all, because the engine excludes forms from a deep hit \
             test and nothing substitutes them back. Look at `canvas::smart::Scope::resolve` and \
             at whether `canvas::input::allowed_candidates` is calling it. Trace: {}.",
            session.trace_path().display()
        )));
    }
    report.note(format!(
        "★ one click selected the container: `first={first}`"
    ));

    // --- B: a double-click must go inside ----------------------------------
    driver.double_click_at(at)?;
    session.settle(30);

    let trace = session.trace()?;
    if trace.events(ENTER).count() == 0 {
        return Ok(Some(format!(
            "★★★ THE DOUBLE-CLICK DID NOT GO INSIDE: no `{ENTER}` line. The container was \
             selected and a double-click on it changed no scope, so there is no way to reach \
             what is drawn inside it except the Format-tab command an operator has to know \
             exists. `canvas::clicking`'s enter arm is the code; its guard asks whether the hit \
             is classified `FormXObject`, which is where a build that reached this state and \
             stopped would be failing. Trace: {}.",
            session.trace_path().display()
        )));
    }
    let inside = trace
        .last(SELECTION)
        .and_then(|l| l.get("first").map(str::to_owned))
        .unwrap_or_else(|| "none".to_owned());
    if !inside.starts_with("leaf:") {
        return Ok(Some(format!(
            "★★ ENTERED, AND SELECTED NOTHING INSIDE: `{ENTER}` was traced and the selection is \
             `first={inside}` rather than a leaf. The scope changed and the re-probe under it \
             found nothing, which leaves the operator inside a container with the container \
             still selected — a state with no visible difference from before the gesture. Trace: \
             {}.",
            session.trace_path().display()
        )));
    }
    report.note(format!(
        "★★ the double-click entered it and selected what is inside: `first={inside}`"
    ));

    // --- C: two Escapes step back out --------------------------------------
    //
    // ★ The FIRST press must not leave. `canvas::keys` puts the container below
    // the selection on the ladder, so this asserts the order as well as the
    // effect: if one press did both, an operator who pressed Escape to drop a
    // selection would silently lose the container they were working inside.
    driver.press(crate::sys::vk::ESCAPE)?;
    session.settle(15);
    if session
        .trace()?
        .events(ESCAPE)
        .any(|l| l.get("outcome") == Some("LeftContainer"))
    {
        return Ok(Some(format!(
            "★★ ONE ESCAPE LEFT THE CONTAINER, and it should have taken two. The first press \
             belongs to the selection — `canvas::keys`' ladder retires the most transient thing \
             first, and a selection inside a title block is remade by every click while the fact \
             that the operator is working inside it survives them all. A build that leaves on \
             the first press loses the scope every time somebody deselects. Trace: {}.",
            session.trace_path().display()
        )));
    }
    driver.press(crate::sys::vk::ESCAPE)?;
    session.settle(15);
    if !session
        .trace()?
        .events(ESCAPE)
        .any(|l| l.get("outcome") == Some("LeftContainer"))
    {
        return Ok(Some(format!(
            "★★★ THERE IS NO WAY BACK OUT: two Escapes and no `{ESCAPE} outcome=LeftContainer`. \
             The operator is scoped to a container with no keyboard route out of it — every \
             click still resolves inside, and nothing on screen says why. That is the stranding \
             this arm was designed to make unrepresentable. Trace: {}.",
            session.trace_path().display()
        )));
    }
    report.note("★★★ …and two Escapes stepped back out, in that order");
    Ok(None)
}
