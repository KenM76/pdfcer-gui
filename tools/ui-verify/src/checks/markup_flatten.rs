//! `a_markup_can_be_made_part_of_the_page` — a drawn markup's right-click
//! "Make part of the page" burns it into the page: it still shows, it is no
//! longer a markup, and Ctrl+Z makes it one again.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/markup_flatten.md`.

use std::path::PathBuf;

use crate::checks::driving::{
    SHELL_DIAG_ENV, click_mode_segment, declared, declared_names, declared_or_in_overflow, list,
};
use crate::checks::{Check, CheckContext};
use crate::coords::{CanvasMapping, DocPoint, PageGeometry, ScreenPoint};
use crate::error::{Error, Result};
use crate::input::Driver;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;
use crate::sys::vk;

const FIXTURE: &str = "fixtures/four-pages.pdf";
const MODE: &str = "review";
const TAB: &str = "ribbon.tab.markup";
const ITEM: &str = "ribbon.item.markup.polygon";
const COMMIT_EVENT: &str = "add-markup";
/// A selected markup shape publishes one anchor per node; node 1 witnesses
/// "a markup is selected here".
const NODE_ONE: &str = "canvas.markup-node.1";
const ROW: &str = "menu.item.canvas.markup.markup.flatten";
const FLATTENED: &str = "annotation-flattened";

const CORNERS: [(f64, f64); 4] = [(0.25, 0.25), (0.55, 0.25), (0.55, 0.55), (0.25, 0.55)];
/// The top edge's midpoint: on the shape's ink, clear of every node anchor.
const EDGE: (f64, f64) = (0.40, 0.25);
/// Where the ink is counted: around corner 1.
const SAMPLE: (f64, f64) = (0.55, 0.25);
const SAMPLE_HALF_FRACTION: f64 = 0.03;
const INK_FLOOR: usize = 4;

const VK_CONTROL: u16 = 0x11;
const VK_Z: u16 = 0x5A;

fn fixture_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join(FIXTURE)
}

/// See the module documentation.
pub struct AMarkupCanBeMadePartOfThePage;

impl Check for AMarkupCanBeMadePartOfThePage {
    fn name(&self) -> &'static str {
        "a_markup_can_be_made_part_of_the_page"
    }

    fn defect(&self) -> &'static str {
        "a markup cannot be burned into the page from its right-click menu, or the burn changes \
         how it looks, or leaves it selectable as a markup, or cannot be undone"
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

fn at(
    session: &Session,
    mapping: &CanvasMapping,
    page: PageGeometry,
    f: (f64, f64),
) -> Result<ScreenPoint> {
    Ok(session
        .frame()?
        .to_screen(mapping.doc_to_window(DocPoint::new(
            0,
            f.0 * page.width_pt,
            f.1 * page.height_pt,
        ))?))
}

/// Ink around [`SAMPLE`] in a fresh capture.
fn ink(
    session: &Session,
    mapping: &CanvasMapping,
    page: PageGeometry,
    out: &std::path::Path,
) -> Result<crate::pixels::InkReport> {
    let image = crate::capture::window_to_png(session, out)?;
    let frame = session.frame()?;
    let (x, y) = (SAMPLE.0 * page.width_pt, SAMPLE.1 * page.height_pt);
    let half = SAMPLE_HALF_FRACTION * page.width_pt;
    let a = mapping.doc_to_window(DocPoint::new(0, x - half, y - half))?;
    let b = mapping.doc_to_window(DocPoint::new(0, x + half, y + half))?;
    let rect = crate::geom::LRect::new(
        crate::geom::Pt::new(a.x().min(b.x()), a.y().min(b.y())),
        crate::geom::Pt::new(a.x().max(b.x()), a.y().max(b.y())),
    );
    Ok(crate::pixels::ink_run_into(
        &image,
        frame.logical_to_capture_pixels(rect),
    ))
}

#[allow(clippy::too_many_lines)]
fn drive(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    let exe = ctx.resolve_exe().ok_or_else(|| {
        Error::new(format!(
            "no binary to drive. Pass --exe, or build the profile's default at {}.",
            ctx.profile.default_exe
        ))
    })?;
    if !ctx.allow_input {
        return Err(Error::new(
            "input is disabled (--no-input). This check draws a markup and right-clicks it.",
        ));
    }
    let pdf = fixture_path();
    if !pdf.is_file() {
        return Err(Error::new(format!(
            "the pinned fixture is missing: {}",
            pdf.display()
        )));
    }
    if let Some(supplied) = ctx.pdf.as_ref()
        && supplied.file_name() != pdf.file_name()
    {
        report.note(format!(
            "--pdf {} was IGNORED; this check pins {FIXTURE}, whose page is blank where the \
             markup is drawn",
            supplied.display()
        ));
    }
    let ui_rect = ctx.profile.vocab.ui_rect_event.ok_or_else(|| {
        Error::new(format!(
            "the `{}` profile declares no ui-rect trace event.",
            ctx.profile.name
        ))
    })?;
    let page = crate::fixture::page_geometry(&pdf)
        .ok_or_else(|| Error::new(format!("cannot read a page size from {}", pdf.display())))?;

    let mut spec = LaunchSpec::new(&exe, ctx.out("markup_flatten.trace.txt"));
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
    report.note(format!("launched as pid {}", session.pid()));
    session.settle(40);
    let driver = Driver::new(session.window());

    // --- Draw a polygon markup ----------------------------------------------
    click_mode_segment(&session, &driver, ui_rect, MODE)?;
    let Some(tab) = declared(&session.trace()?, ui_rect, TAB) else {
        return Ok(Some(format!("`{MODE}` declares no `{TAB}` region.")));
    };
    driver.click_at(session.frame()?.declared_center(tab))?;
    session.settle(14);
    let Some(item) = declared_or_in_overflow(&session, &driver, ui_rect, ITEM)? else {
        return Ok(Some(format!("the Markup tab declares no `{ITEM}`.")));
    };
    driver.click_at(session.frame()?.declared_center(item))?;
    session.settle(16);
    let trace = session.trace()?;
    let mapping = CanvasMapping::from_trace(&trace, &ctx.profile.vocab, page, 0)?;
    // Three clicks and a double-click: the double-click's first press is the
    // fourth vertex.
    for c in &CORNERS[..3] {
        driver.click_at(at(&session, &mapping, page, *c)?)?;
        session.settle(12);
    }
    driver.double_click_at(at(&session, &mapping, page, CORNERS[3])?)?;
    session.settle(30);
    if session.trace()?.events(COMMIT_EVENT).count() == 0 {
        return Ok(Some(format!(
            "the polygon did not commit (no `{COMMIT_EVENT}`), so there is no markup to flatten."
        )));
    }
    driver.press(vk::V)?;
    session.settle(12);
    let before = ink(
        &session,
        &mapping,
        page,
        &ctx.out("markup_flatten_before.png"),
    )?;
    report.note(format!("before: {}", before.summary()));
    if before.ink < INK_FLOOR {
        return Err(Error::new(format!(
            "the polygon's corner shows no ink before the flatten ({}); the harness is aiming \
             at the wrong place, so nothing below can mean anything.",
            before.summary()
        )));
    }

    // --- Select, right-click, Make part of the page -------------------------
    let edge = at(&session, &mapping, page, EDGE)?;
    driver.click_at(edge)?;
    session.settle(18);
    if declared(&session.trace()?, ui_rect, NODE_ONE).is_none() {
        return Ok(Some(format!(
            "clicking the polygon's edge selected no markup (no `{NODE_ONE}`)."
        )));
    }
    driver.right_click_at(edge)?;
    session.settle(20);
    let trace = session.trace()?;
    let Some(row) = declared(&trace, ui_rect, ROW) else {
        return Ok(Some(format!(
            "the markup's right-click menu offers no Make part of the page (`{ROW}`). Rows: {}.",
            list(&declared_names(&trace, ui_rect, "menu.item.canvas.markup."))
        )));
    };
    let mark = trace.mark();
    driver.click_at(session.frame()?.declared_center(row))?;
    session.settle(40);
    let trace = session.trace()?;
    let Some(done) = trace.last_after(FLATTENED, mark) else {
        return Ok(Some(format!(
            "Make part of the page was clicked and no `{FLATTENED}` line followed: \
             `flatten_annotations` never ran. Trace: {}.",
            session.trace_path().display()
        )));
    };
    report.note(format!("flattened: `{}`", done.raw));
    if done.get("changed") != Some("true") || done.get("flattened") != Some("1") {
        return Ok(Some(format!(
            "the engine did not burn exactly one markup: `{}`.",
            done.raw
        )));
    }

    // --- It looks the same and is no longer a markup ------------------------
    let after = ink(
        &session,
        &mapping,
        page,
        &ctx.out("markup_flatten_after.png"),
    )?;
    report.note(format!("after: {}", after.summary()));
    if after.ink < INK_FLOOR {
        return Ok(Some(format!(
            "the markup vanished from the page when it was made part of it: {} before, {} after.",
            before.summary(),
            after.summary()
        )));
    }
    driver.click_at(edge)?;
    session.settle(18);
    if declared(&session.trace()?, ui_rect, NODE_ONE).is_some() {
        return Ok(Some(format!(
            "after the flatten, clicking the same edge still selects a markup (`{NODE_ONE}`)."
        )));
    }

    // --- Ctrl+Z makes it a markup again -------------------------------------
    driver.press_chord(&[VK_CONTROL], VK_Z)?;
    session.settle(30);
    driver.click_at(edge)?;
    session.settle(18);
    if declared(&session.trace()?, ui_rect, NODE_ONE).is_none() {
        return Ok(Some(format!(
            "Ctrl+Z after the flatten did not bring the markup back: clicking its edge selects \
             nothing (no `{NODE_ONE}`). Trace: {}.",
            session.trace_path().display()
        )));
    }
    report.note("Ctrl+Z made it a selectable markup again");
    Ok(None)
}
