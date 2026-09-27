//! `markup_draws_into_page_content` — a Review shape drawn with the pen's
//! *On page* switch lands in the page's own content, not as a comment.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/markup_on_page.md`.

use crate::checks::driving::{self, SHELL_DIAG_ENV, declared, declared_names, list};
use crate::checks::text_selection::aim;
use crate::checks::{Check, CheckContext};
use crate::coords::{DocPoint, PageGeometry};
use crate::error::{Error, Result};
use crate::geom::LRect;
use crate::input::Driver;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;

/// Review, whose tab list carries Markup.
const MODE: &str = "review";
/// The tab carrying the Style group and the rectangle tool.
const TAB: &str = "ribbon.tab.markup";
/// The pen's destination switch.
const TARGET: &str = "markup.style.target";
/// The rectangle tool.
const RECTANGLE: &str = "ribbon.item.markup.rectangle";
/// `markup-pen … on_page=…` — one line per pen change.
const PEN_EVENT: &str = "markup-pen";
/// The funnel's line for an annotation commit.
const ANNOT_EVENT: &str = "add-markup";
/// The funnel's line for a page-content commit.
const CONTENT_EVENT: &str = "add-markup-as-content";
/// `markup-as-content-applied page=… objects=S..E resources_added=…`.
const APPLIED_EVENT: &str = "markup-as-content-applied";
/// The drag, in page fractions, PDF y up.
const DRAG: ((f64, f64), (f64, f64)) = ((0.30, 0.30), (0.55, 0.45));

/// See the module documentation.
pub struct MarkupDrawsIntoPageContent;

impl Check for MarkupDrawsIntoPageContent {
    fn name(&self) -> &'static str {
        "markup_draws_into_page_content"
    }

    fn defect(&self) -> &'static str {
        "a Review shape drawn with the pen set to draw on the page is still authored as a \
         comment, or the pen offers no way to choose"
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

/// Find a declared region or fail with the names that were declared.
fn region(
    session: &Session,
    ui_rect: &str,
    name: &str,
) -> Result<std::result::Result<LRect, String>> {
    let trace = session.trace()?;
    Ok(declared(&trace, ui_rect, name).ok_or_else(|| {
        format!(
            "`{name}` was never declared, so there is nothing to click. Markup regions this run: \
             {}.",
            list(&declared_names(&trace, ui_rect, "markup."))
        )
    }))
}

/// Run the sequence. `Err` is SKIP, `Ok(Some(_))` is FAIL, `Ok(None)` is a pass.
fn drive(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    let exe = ctx
        .resolve_exe()
        .ok_or_else(|| Error::new("no binary to drive; pass --exe."))?;
    let pdf = ctx
        .pdf
        .clone()
        .ok_or_else(|| Error::new("no --pdf; every markup tool is gated on an open page."))?;
    if !ctx.allow_input {
        return Err(Error::new(
            "input is disabled (--no-input); this check is clicks and a drag.",
        ));
    }
    let ui_rect = ctx.profile.vocab.ui_rect_event.ok_or_else(|| {
        Error::new("the profile declares no ui-rect event, so nothing can be aimed at.")
    })?;
    let page: PageGeometry = match ctx.page_size {
        Some((w, h)) => PageGeometry {
            width_pt: w,
            height_pt: h,
        },
        None => crate::fixture::page_geometry(&pdf).ok_or_else(|| {
            Error::new(format!(
                "cannot read a page size from {}; pass --page-size.",
                pdf.display()
            ))
        })?,
    };

    let mut spec = LaunchSpec::new(&exe, ctx.out("markup_on_page.trace.txt"));
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
    session.settle(40);
    if !session.trace()?.started(ctx.profile.vocab.start_event) {
        return Err(Error::new(
            "the trace has no start line, so this check has no oracle.",
        ));
    }
    let driver = Driver::new(session.window());

    driving::click_mode_segment(&session, &driver, ui_rect, MODE)?;
    let tab = match region(&session, ui_rect, TAB)? {
        Ok(r) => r,
        Err(why) => return Err(Error::new(why)),
    };
    driver.click_at(session.frame()?.declared_center(tab))?;
    session.settle(16);

    // 1. The switch exists and turns on.
    let target = match region(&session, ui_rect, TARGET)? {
        Ok(r) => r,
        Err(why) => {
            return Ok(Some(format!(
                "THE PEN OFFERS NO WAY TO DRAW ON THE PAGE. {why}"
            )));
        }
    };
    driver.click_at(session.frame()?.declared_center(target))?;
    session.settle(12);
    let on = session
        .trace()?
        .events(PEN_EVENT)
        .last()
        .and_then(|l| l.get("on_page"))
        .map(str::to_owned);
    if on.as_deref() != Some("true") {
        return Ok(Some(format!(
            "clicking `{TARGET}` did not set the pen to draw on the page: the last `{PEN_EVENT}` \
             line has on_page={on:?}."
        )));
    }
    report.note("the pen's On page switch is on");

    // 2. A rectangle drawn now becomes page content.
    let tool = match region(&session, ui_rect, RECTANGLE)? {
        Ok(r) => r,
        Err(why) => return Err(Error::new(why)),
    };
    driver.click_at(session.frame()?.declared_center(tool))?;
    session.settle(12);
    let at = |(x, y): (f64, f64)| {
        aim(
            ctx,
            &session,
            page,
            DocPoint::new(0, x * page.width_pt, y * page.height_pt),
        )
    };
    let (from, to) = (at(DRAG.0)?, at(DRAG.1)?);
    let before = session.trace()?.events(ANNOT_EVENT).count();
    driver.drag(from, to)?;
    session.settle(24);

    let trace = session.trace()?;
    if trace.events(ANNOT_EVENT).count() > before {
        return Ok(Some(format!(
            "the rectangle was authored as a COMMENT (`{ANNOT_EVENT}`) with the pen set to draw \
             on the page. Trace: {}.",
            session.trace_path().display()
        )));
    }
    let Some(applied) = trace.events(APPLIED_EVENT).last() else {
        return Ok(Some(format!(
            "the drag authored nothing: no `{APPLIED_EVENT}` line{}. Trace: {}.",
            if trace.events(CONTENT_EVENT).next().is_some() {
                format!(", although `{CONTENT_EVENT}` ran — the engine refused it")
            } else {
                String::new()
            },
            session.trace_path().display()
        )));
    };
    let objects = applied.get("objects").unwrap_or("");
    let span = objects
        .split_once("..")
        .and_then(|(s, e)| Some(e.parse::<usize>().ok()? - s.parse::<usize>().ok()?));
    if !matches!(span, Some(n) if n > 0) {
        return Ok(Some(format!(
            "`{APPLIED_EVENT}` reported objects={objects:?}: the page gained no drawing objects."
        )));
    }
    report.note(format!(
        "★ the rectangle became page objects {objects}, and no comment was authored"
    ));
    Ok(None)
}
