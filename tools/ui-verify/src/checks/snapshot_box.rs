//! `a_snapshot_box_stays_on_the_page_through_a_zoom` — View ▸ Snapshot arms
//! from the ribbon, a drag lays a box at the page points dragged over, and
//! after a zoom the box is drawn over the same page area. Driven through the
//! scripted pointer on a window placed off the desktop.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/snapshot_box.md`.

use crate::checks::driving::{SHELL_DIAG_ENV, declared, declared_names, list};
use crate::checks::{Check, CheckContext};
use crate::coords::{CanvasMapping, DocPoint, PageGeometry, WindowPoint};
use crate::error::{Error, Result};
use crate::geom::{LRect, Pt};
use crate::input::scripted::ScriptedPointer;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;

pub(super) const TAB: &str = "ribbon.tab.view"; // ui-text-exempt: a trace region name, never displayed
pub(super) const ITEM: &str = "ribbon.item.view.tool_snapshot"; // ui-text-exempt: a trace region name, never displayed
/// Prefix of the View tab's groups, for finding a collapsed one.
pub(super) const GROUPS: &str = "ribbon.group.view."; // ui-text-exempt: a trace region name, never displayed
const ZOOM_GROUP: &str = "status-group:zoom"; // ui-text-exempt: a trace region name, never displayed
pub(super) const BOX_REGION: &str = "canvas.snapshot"; // ui-text-exempt: a trace region name, never displayed
pub(super) const ARMED_EVENT: &str = "snapshot-tool"; // ui-text-exempt: a trace event name, never displayed
pub(super) const BOX_EVENT: &str = "snapshot-box"; // ui-text-exempt: a trace event name, never displayed
/// Off the desktop, so no OS input can reach it and none of his is taken.
const OFFSCREEN: &str = "-4200,-4200,1400,900";
/// The dragged corners as fractions of the page box (y up), about the page's
/// middle so a zoom about the viewport keeps them in view.
pub(super) const FROM: (f64, f64) = (0.42, 0.44);
pub(super) const TO: (f64, f64) = (0.58, 0.56);
/// How far a laid corner may sit from the dragged one, in points: the
/// scripted pointer lands on whole logical points.
const CORNER_TOLERANCE_PT: f64 = 1.5;
/// How far the drawn box may sit from where the mapping puts it, in logical
/// points: the stroke is a pixel wide and the trace rounds.
const DRAWN_TOLERANCE: f32 = 3.0;
/// The `+` sits at the zoom group's right end; the middle is the readout.
const PLUS_AT: f32 = 0.93;
/// Zoom-in presses: enough to move every corner well past the tolerance.
const PRESSES: usize = 2;

/// A failure sentence, or the value a step read.
pub(super) type Step<T> = std::result::Result<T, String>;

/// See the module documentation.
pub struct ASnapshotBoxStaysOnThePageThroughAZoom;

impl Check for ASnapshotBoxStaysOnThePageThroughAZoom {
    fn name(&self) -> &'static str {
        "a_snapshot_box_stays_on_the_page_through_a_zoom"
    }

    fn defect(&self) -> &'static str {
        "View ▸ Snapshot does not arm, a drag lays no box or lays it away from the page points \
         dragged over, or the box stays put on the screen while the page zooms under it"
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

/// The launched window and the pointer that drives it.
pub(super) struct Rig {
    pub(super) session: Session,
    pub(super) pointer: ScriptedPointer,
    pub(super) ui_rect: &'static str,
}

impl Rig {
    pub(super) fn region(&self, name: &str) -> Result<LRect> {
        let trace = self.session.trace()?;
        declared(&trace, self.ui_rect, name).ok_or_else(|| {
            let prefix = name.rsplit_once('.').map_or(name, |(head, _)| head);
            Error::new(format!(
                "no `{name}` region. Declared under `{prefix}`: {}.",
                list(&declared_names(&trace, self.ui_rect, prefix))
            ))
        })
    }

    pub(super) fn click(&self, region: &str) -> Result<()> {
        let rect = self.region(region)?;
        self.pointer
            .click(&self.session, WindowPoint::centre_of(rect))?;
        self.session.settle(15);
        Ok(())
    }

    /// Click `item`, opening each collapsed group under `groups` until it shows.
    pub(super) fn click_item(&self, item: &str, groups: &str) -> Result<()> {
        if declared(&self.session.trace()?, self.ui_rect, item).is_none() {
            let collapsed: Vec<String> =
                declared_names(&self.session.trace()?, self.ui_rect, groups)
                    .into_iter()
                    .filter(|n| n.ends_with(".collapsed"))
                    .collect();
            for group in collapsed {
                self.click(&group)?;
                if declared(&self.session.trace()?, self.ui_rect, item).is_some() {
                    break;
                }
            }
        }
        self.click(item)
    }
}

/// Launch on the fixture off the desktop; `stem` names the artifacts.
pub(super) fn launch(
    ctx: &CheckContext,
    report: &mut CheckReport,
    stem: &str,
) -> Result<(Rig, PageGeometry)> {
    launch_with(ctx, report, stem, &[])
}

/// [`launch`], with `env` added to the app's environment.
pub(super) fn launch_with(
    ctx: &CheckContext,
    report: &mut CheckReport,
    stem: &str,
    env: &[(String, String)],
) -> Result<(Rig, PageGeometry)> {
    let exe = ctx.resolve_exe().ok_or_else(|| {
        Error::new(format!(
            "no binary to drive. Pass --exe, or build the profile's default at {}.",
            ctx.profile.default_exe
        ))
    })?;
    let viewport_env = ctx.profile.viewport_env.ok_or_else(|| {
        Error::new("the profile has no viewport variable to place the window off the desktop.")
    })?;
    let ui_rect = ctx
        .profile
        .vocab
        .ui_rect_event
        .ok_or_else(|| Error::new("the profile declares no ui-rect trace event."))?;
    let pdf = crate::fixture::workspace_root()
        .join("fixtures")
        .join("blank-overhang.pdf");
    let page = crate::fixture::page_geometry(&pdf)
        .ok_or_else(|| Error::new(format!("cannot read a page size from {}.", pdf.display())))?;
    let mut spec = LaunchSpec::new(&exe, ctx.out(&format!("{stem}.trace.txt")));
    spec.pdf = Some(pdf);
    spec.env.push((
        ctx.profile.diag_env.0.to_owned(),
        ctx.profile.diag_env.1.to_owned(),
    ));
    spec.env
        .push((SHELL_DIAG_ENV.0.to_owned(), SHELL_DIAG_ENV.1.to_owned()));
    spec.env
        .push((viewport_env.to_owned(), OFFSCREEN.to_owned()));
    spec.env.extend_from_slice(env);
    spec.place = false;
    spec.allow_stale = ctx.allow_stale;
    spec.source_root = ctx.source_root.clone();
    let pointer = ScriptedPointer::attach(&mut spec, ctx.out(&format!("{stem}.pointer.txt")))?;
    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    report.artifact(session.trace_path().to_path_buf());
    report.artifact(pointer.path().to_path_buf());
    session.settle(45);
    Ok((
        Rig {
            session,
            pointer,
            ui_rect,
        },
        page,
    ))
}

fn drive(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    let (rig, page) = launch(ctx, report, "snapshot-box")?;
    rig.click(TAB)?;
    rig.click_item(ITEM, GROUPS)?;
    match rig.session.trace()?.last(ARMED_EVENT) {
        Some(l) if l.get("armed") == Some("true") => {}
        other => {
            return Ok(Some(format!(
                "★ the Snapshot button traced `{}`, not an armed tool.",
                other.map_or("nothing", |l| l.raw.as_str())
            )));
        }
    }
    let corners = (
        (FROM.0 * page.width_pt, FROM.1 * page.height_pt),
        (TO.0 * page.width_pt, TO.1 * page.height_pt),
    );
    let mapping = CanvasMapping::from_trace(&rig.session.trace()?, &ctx.profile.vocab, page, 0)?;
    let from = mapping.doc_to_window(DocPoint::new(0, corners.0.0, corners.0.1))?;
    let to = mapping.doc_to_window(DocPoint::new(0, corners.1.0, corners.1.1))?;
    rig.pointer.drag(&rig.session, from, to, 12)?;
    rig.session.settle(20);

    let laid = match laid_box(&rig, corners)? {
        Ok(raw) => raw,
        Err(why) => return Ok(Some(why)),
    };
    report.note(format!("★ the drag laid `{laid}`"));
    if let Err(why) = drawn_where_mapped(&rig, ctx, page, corners, "before the zoom")? {
        return Ok(Some(why));
    }
    let before = rig.region(BOX_REGION)?;

    let zoom = rig.region(ZOOM_GROUP)?;
    let x = zoom.min.x + PLUS_AT * zoom.width();
    let y = (zoom.min.y + zoom.max.y) / 2.0;
    let plus = WindowPoint::centre_of(LRect::new(Pt::new(x, y), Pt::new(x, y)));
    for _ in 0..PRESSES {
        rig.pointer.click(&rig.session, plus)?;
        rig.session.settle(15);
    }
    rig.session.settle(30);
    let after = rig.region(BOX_REGION)?;
    if (after.width() - before.width()).abs() < DRAWN_TOLERANCE {
        return Ok(Some(format!(
            "★★ two zoom-in presses left the box {:.1} wide, as before; the zoom never \
             happened, so the check proves nothing.",
            after.width()
        )));
    }
    if let Err(why) = drawn_where_mapped(&rig, ctx, page, corners, "after the zoom")? {
        return Ok(Some(why));
    }
    match rig.session.trace()?.last(BOX_EVENT) {
        Some(l) if box_fields(l) == laid => {}
        other => {
            return Ok(Some(format!(
                "★★ the zoom changed the box from `{laid}` to `{}`.",
                other.map_or("nothing", |l| l.raw.as_str())
            )));
        }
    }
    report.note(format!(
        "★★ the box grew from {:.0} to {:.0} wide with the page and stayed on it",
        before.width(),
        after.width()
    ));
    Ok(None)
}

/// The `snapshot-box` line, required on page 0 at the dragged corners.
pub(super) fn laid_box(rig: &Rig, corners: ((f64, f64), (f64, f64))) -> Result<Step<String>> {
    let trace = rig.session.trace()?;
    let Some(line) = trace.last(BOX_EVENT) else {
        return Ok(Err(format!(
            "★ the drag laid no box: no `{BOX_EVENT}` line. Trace: {}.",
            rig.session.trace_path().display()
        )));
    };
    let field = |k: &str| line.get(k).and_then(|v| v.parse::<f64>().ok());
    let want = [
        ("llx", corners.0.0.min(corners.1.0)),
        ("lly", corners.0.1.min(corners.1.1)),
        ("urx", corners.0.0.max(corners.1.0)),
        ("ury", corners.0.1.max(corners.1.1)),
    ];
    let off = want
        .iter()
        .any(|(k, v)| field(k).is_none_or(|got| (got - v).abs() > CORNER_TOLERANCE_PT));
    if line.get("page") != Some("0") || off {
        return Ok(Err(format!(
            "★ the drag over page 0 from ({:.1}, {:.1}) to ({:.1}, {:.1}) laid `{}`.",
            corners.0.0, corners.0.1, corners.1.0, corners.1.1, line.raw
        )));
    }
    Ok(Ok(box_fields(line)))
}

/// A `snapshot-box` line's page and corners, without the trace's framing.
pub(super) fn box_fields(line: &crate::trace::TraceLine) -> String {
    ["page", "llx", "lly", "urx", "ury"]
        .iter()
        .map(|k| format!("{k}={}", line.get(k).unwrap_or("?")))
        .collect::<Vec<_>>()
        .join(" ")
}

/// The `canvas.snapshot` region, required where the current mapping puts the
/// dragged corners.
fn drawn_where_mapped(
    rig: &Rig,
    ctx: &CheckContext,
    page: PageGeometry,
    corners: ((f64, f64), (f64, f64)),
    when: &str,
) -> Result<Step<()>> {
    let trace = rig.session.trace()?;
    let mapping = CanvasMapping::from_trace(&trace, &ctx.profile.vocab, page, 0)?;
    // Unbounded: after a zoom a corner may be scrolled out of view and is
    // still where the box must be drawn.
    let anywhere = LRect::new(Pt::new(-1.0e6, -1.0e6), Pt::new(1.0e6, 1.0e6));
    let a = mapping.doc_to_window_off_page(DocPoint::new(0, corners.0.0, corners.0.1), anywhere)?;
    let b = mapping.doc_to_window_off_page(DocPoint::new(0, corners.1.0, corners.1.1), anywhere)?;
    let want = [
        a.x().min(b.x()),
        a.y().min(b.y()),
        a.x().max(b.x()),
        a.y().max(b.y()),
    ];
    let Some(drawn) = declared(&trace, rig.ui_rect, BOX_REGION) else {
        return Ok(Err(format!(
            "★ {when}, no `{BOX_REGION}` region: the box is not drawn."
        )));
    };
    let got = [drawn.min.x, drawn.min.y, drawn.max.x, drawn.max.y];
    if want
        .iter()
        .zip(got)
        .any(|(w, g)| (w - g).abs() > DRAWN_TOLERANCE)
    {
        return Ok(Err(format!(
            "★★ {when}, the box is drawn at ({:.1}, {:.1})-({:.1}, {:.1}) and the page points \
             it holds map to ({:.1}, {:.1})-({:.1}, {:.1}).",
            got[0], got[1], got[2], got[3], want[0], want[1], want[2], want[3]
        )));
    }
    Ok(Ok(()))
}
