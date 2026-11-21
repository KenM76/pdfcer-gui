//! `checks::model_select_readout` — **the 3D viewer's *Face, edge, corner*
//! measure says its figures in a chosen unit, projects distances and a face's
//! area onto a chosen plane, and says corners in a coordinate system set
//! from two picked edges**
//!
//! Opens `fixtures/glow-room.prc` in the viewer, off the desktop through the
//! scripted pointer. See `docs/modules/ui-verify/checks/model_select_readout.md`.

use crate::checks::driving::{count, repo_fixture};
use crate::checks::model_inferences::open_viewer;
use crate::checks::model_light_meter::number;
use crate::checks::model_pick::Drive;
use crate::checks::model_select_measure::{
    AGAIN, ENTITY, Landed, SELECT, SWITCH, Selecting, TUBE_B,
};
use crate::checks::model_surface_measure::{Measuring, three};
use crate::checks::model_view_window::launch_with_model_file;
use crate::checks::{Check, CheckContext};
use crate::error::Result;
use crate::report::CheckReport;
use crate::trace::TraceLine;

const METHOD: &str = "python fixtures/glow-room.PROVENANCE.py <pdfcer.exe>";
const UNIT: &str = "model3d.select.unit.";
const PROJECTION: &str = "model3d.select.projection.";
const SET_FRAME: &str = "model3d.select.frame.set";
const MODEL_FRAME: &str = "model3d.select.frame.model";
const PROJECTED: &str = "model-select-projected";
const FRAME: &str = "model-select-frame";
const DISTANCE: &str = "model-select-distance";
/// Millimetres: `LengthUnit::ALL[1]`, one past the file's own choice.
const MILLIMETRES: usize = 2;
const SECOND: &str = "model3d.select.second.";
/// Inches: `LengthUnit::ALL[3]`, one past the no-second-unit choice.
const INCHES: usize = 4;
/// One metre in millimetres and inches, 1000 / 25.4, to three places, the
/// trace's spaces written `_`.
const METRE_SHOWN: &str = "1000.000_mm_(39.370_in)";
/// The projection choices' places: nothing, the screen, XY, YZ, ZX.
const NOTHING: usize = 0;
const XY: usize = 2;
const YZ: usize = 3;
/// The engine's writer states glTF's metres, so millimetres are 1000 of them.
const MM_PER_METRE: f64 = 1000.0;
/// The floor: a 4 by 4 square at height 0, so its shadow on XY is all of it
/// and on YZ is nothing.
const FLOOR_AREA: f64 = 16.0;
const FLOOR_SIDE: f64 = 4.0;
const FLOOR_HALF: f64 = 2.0;
/// The box's and `tube-b`'s nearest sides face each other along X, 0.3 apart.
const BOX_TO_TUBE: f64 = 0.3;
const SLACK: f64 = 1e-4;

/// See the module documentation.
pub struct TheViewerConvertsProjectsAndReframes;

impl Check for TheViewerConvertsProjectsAndReframes {
    fn name(&self) -> &'static str {
        "the_3d_viewer_converts_projects_and_reframes_its_measures"
    }

    fn defect(&self) -> &'static str {
        "the 3D viewer's Face, edge, corner measure offers no unit for a file that states one or \
         converts by the wrong factor, projects a face's area or a distance onto the wrong plane, \
         or sets no coordinate system from two edges, or one whose origin and axes are not where \
         the edges meet and run"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        let driven = repo_fixture("glow-room.prc", METHOD)
            .and_then(|m| {
                launch_with_model_file(
                    ctx,
                    &mut report,
                    "model-select-readout",
                    &m.to_string_lossy(),
                )
            })
            .and_then(|(session, pointer)| {
                if let Some(failure) = open_viewer(ctx, &session, &pointer)? {
                    return Ok(Some(failure));
                }
                let drive = Drive {
                    ctx,
                    session: &session,
                    pointer: &pointer,
                };
                Reading(Selecting(Measuring(&drive))).run(&mut report)
            });
        match driven {
            Ok(Some(failure)) => report.fail(failure),
            Ok(None) => report.pass(),
            Err(why) => report.from_error(&why),
        }
    }
}

fn near(value: f64, want: f64) -> bool {
    (value - want).abs() <= SLACK
}

/// A `toward/away` pair of a projected face, or `None` for `-` or a refusal.
fn pair(line: &TraceLine, key: &str) -> Option<(f64, f64)> {
    let (a, b) = line.get(key)?.split_once('/')?;
    Some((a.parse().ok()?, b.parse().ok()?))
}

struct Reading<'a>(Selecting<'a>);

impl Reading<'_> {
    fn drive(&self) -> &Drive<'_> {
        self.0.drive()
    }

    /// Press `region` and return the first `event` line it traced.
    fn traced(&self, region: &str, event: &str) -> Result<Option<TraceLine>> {
        let session = self.drive().session;
        let before = count(session, event)?;
        if !self.0.choose(region)? {
            return Ok(None);
        }
        session.settle(10);
        Ok(session.trace()?.events(event).nth(before).cloned())
    }

    /// Clear, then pick `spots` in turn; whether each picked something.
    fn pick(&self, spots: &[&Landed]) -> Result<bool> {
        self.drive().click(AGAIN, "", "model-select-cleared")?;
        for l in spots {
            if self.0.click(&l.spot)?.is_none() {
                return Ok(false);
            }
        }
        Ok(true)
    }

    /// The newest projection line, if any was traced after the first `before`.
    fn projected_since(&self, before: usize) -> Result<Option<TraceLine>> {
        let session = self.drive().session;
        session.settle(10);
        let trace = session.trace()?;
        Ok((trace.events(PROJECTED).count() > before)
            .then(|| trace.events(PROJECTED).last().cloned())
            .flatten())
    }

    /// Pick `spots` afresh; the newest projection line they made.
    fn pick_projected(&self, spots: &[&Landed]) -> Result<Option<TraceLine>> {
        let before = count(self.drive().session, PROJECTED)?;
        if !self.pick(spots)? {
            return Ok(None);
        }
        self.projected_since(before)
    }

    fn run(&self, report: &mut CheckReport) -> Result<Option<String>> {
        let m = &self.0.0;
        if !m.turn()? {
            return Ok(Some("the viewer drew no picture to turn.".to_owned()));
        }
        if !m.press(SWITCH)? || !self.0.choose(SELECT)? {
            return Ok(Some(format!(
                "the viewer drew no `{SWITCH}` or `{SELECT}`."
            )));
        }
        let landed = self.0.survey()?;
        report.note(format!("{} spots landed on the model", landed.len()));
        if let Some(failure) = self.unit(report)? {
            return Ok(Some(failure));
        }
        if let Some(failure) = self.floor_projected(report, &landed)? {
            return Ok(Some(failure));
        }
        if let Some(failure) = self.distance_projected(report, &landed)? {
            return Ok(Some(failure));
        }
        self.frame(report, &landed)
    }

    /// Millimetres of a file in metres are a thousand to its unit.
    fn unit(&self, report: &mut CheckReport) -> Result<Option<String>> {
        let Some(line) = self.traced(&format!("{UNIT}{MILLIMETRES}"), "model-select-unit")? else {
            return Ok(Some(format!(
                "★★★ the Measure tab drew no `{UNIT}{MILLIMETRES}`, or choosing it traced no \
                 `model-select-unit`: a file stating its unit offers no other."
            )));
        };
        report.note(format!("millimetres: {}", line.raw));
        let right = line.get("unit") == Some("mm") && near(number(&line, "factor"), MM_PER_METRE);
        if !right {
            return Ok(Some(format!(
                "★★★ choosing millimetres for a file in metres traced `{}`; it must be unit=mm \
                 factor={MM_PER_METRE}.",
                line.raw
            )));
        }
        let Some(line) = self.traced(&format!("{SECOND}{INCHES}"), "model-select-unit")? else {
            return Ok(Some(format!(
                "★★★ the Measure tab drew no `{SECOND}{INCHES}`, or choosing it traced no \
                 `model-select-unit`: there is no second unit."
            )));
        };
        report.note(format!("and inches: {}", line.raw));
        let right = line.get("second") == Some("in") && line.get("shown") == Some(METRE_SHOWN);
        Ok((!right).then(|| {
            format!(
                "★★★ millimetres and again inches traced `{}`; one metre must be shown \
                 `{METRE_SHOWN}`.",
                line.raw
            )
        }))
    }

    /// The floor's area flattened onto XY is all of it, onto YZ nothing.
    fn floor_projected(
        &self,
        report: &mut CheckReport,
        landed: &[Landed],
    ) -> Result<Option<String>> {
        let Some(floor) = landed.iter().find(|l| l.on_floor()) else {
            return Ok(Some("no spot of the grid landed on the floor.".to_owned()));
        };
        if !self.0.choose(&format!("{PROJECTION}{XY}"))? {
            return Ok(Some(format!(
                "★★★ the Measure tab drew no `{PROJECTION}{XY}`."
            )));
        }
        let on_xy = self.pick_projected(&[floor])?;
        let on_yz = self.traced(&format!("{PROJECTION}{YZ}"), PROJECTED)?;
        let (Some(on_xy), Some(on_yz)) = (on_xy, on_yz) else {
            return Ok(Some(format!(
                "★★★ a face picked with a projection chosen traced no `{PROJECTED}`, or choosing \
                 `{PROJECTION}{YZ}` did not remake it."
            )));
        };
        report.note(format!("floor on XY: {}", on_xy.raw));
        report.note(format!("floor on YZ: {}", on_yz.raw));
        let xy = pair(&on_xy, "face1");
        let yz = pair(&on_yz, "face1");
        let right = on_xy.get("plane") == Some("xy")
            && on_yz.get("plane") == Some("yz")
            && xy.is_some_and(|(t, a)| near(t + a, FLOOR_AREA) && near(t.min(a), 0.0))
            && yz.is_some_and(|(t, a)| near(t + a, 0.0));
        Ok((!right).then(|| {
            format!(
                "★★★ the floor, a {FLOOR_AREA} square lying in XY, projected `{}` on XY and `{}` \
                 on YZ; it must be {FLOOR_AREA} on one side on XY and nothing on YZ.",
                on_xy.raw, on_yz.raw
            )
        }))
    }

    /// The box and `tube-b` are 0.3 apart along X: all of it on XY, none of
    /// it on YZ, where all 0.3 is square to the plane.
    fn distance_projected(
        &self,
        report: &mut CheckReport,
        landed: &[Landed],
    ) -> Result<Option<String>> {
        let (Some(a), Some(b)) = (
            landed.iter().find(|l| l.on_box()),
            landed.iter().find(|l| l.within(TUBE_B)),
        ) else {
            return Ok(Some("no spots landed on the box and on tube-b.".to_owned()));
        };
        let before = count(self.drive().session, DISTANCE)?;
        let on_yz = self.pick_projected(&[a, b])?;
        if count(self.drive().session, DISTANCE)? == before {
            return Ok(Some("★★★ two picks measured no distance.".to_owned()));
        }
        let on_xy = self.traced(&format!("{PROJECTION}{XY}"), PROJECTED)?;
        let (Some(on_xy), Some(on_yz)) = (on_xy, on_yz) else {
            return Ok(Some(format!(
                "★★★ two picks with a projection chosen traced no `{PROJECTED}`."
            )));
        };
        report.note(format!("box to tube-b on YZ: {}", on_yz.raw));
        report.note(format!("box to tube-b on XY: {}", on_xy.raw));
        // Each side is measured from its face's triangles (grown or whole),
        // so the span is the distance line's own delta; the projection must
        // split that delta into the plane's part and the part square to it.
        let Some(span) = self
            .drive()
            .session
            .trace()?
            .events(DISTANCE)
            .last()
            .cloned()
        else {
            return Ok(Some("★★★ two picks measured no distance.".to_owned()));
        };
        report.note(format!("box to tube-b span: {}", span.raw));
        let [dx, dy, dz] = ["dx", "dy", "dz"].map(|k| number(&span, k));
        let whole = span.get("a_source") != Some("grown") && span.get("b_source") != Some("grown");
        let whole_right = !whole || (near(dx.abs(), BOX_TO_TUBE) && near(dy, 0.0) && near(dz, 0.0));
        let right = whole_right
            && near(number(&on_xy, "minimum"), dx.hypot(dy))
            && near(number(&on_xy, "out"), dz)
            && near(number(&on_yz, "minimum"), dy.hypot(dz))
            && near(number(&on_yz, "out"), dx);
        Ok((!right).then(|| {
            format!(
                "★★★ the box and tube-b spanned `{}` and projected `{}` on XY and `{}` on YZ; \
                 XY must keep the X-Y part with Z out, YZ the Y-Z part with X out (whole parts \
                 are {BOX_TO_TUBE} apart along X).",
                span.raw, on_xy.raw, on_yz.raw
            )
        }))
    }

    /// Two floor edges meeting at a corner set a system there whose Z is the
    /// floor's normal, and every floor corner is then 0 or 4 along its X and
    /// Y and 0 along its Z.
    fn frame(&self, report: &mut CheckReport, landed: &[Landed]) -> Result<Option<String>> {
        self.0.choose(&format!("{PROJECTION}{NOTHING}"))?;
        // The floor spot nearest a side running along Y, and the one nearest
        // a side running along X: their edges meet at a floor corner.
        let nearest = |along_y: bool| {
            landed
                .iter()
                .filter(|l| l.on_floor() && (l.at[0].abs() > l.at[1].abs()) == along_y)
                .min_by(|a, b| a.inset().total_cmp(&b.inset()))
        };
        let (Some(by_x), Some(by_y)) = (nearest(true), nearest(false)) else {
            return Ok(Some("no spot of the grid landed on the floor.".to_owned()));
        };
        if !self.0.choose(&format!("{ENTITY}1"))? || !self.pick(&[by_x, by_y])? {
            return Ok(Some(
                "★★★ two edge picks on the floor picked nothing.".to_owned(),
            ));
        }
        let Some(set) = self.traced(SET_FRAME, FRAME)? else {
            return Ok(Some(format!(
                "★★★ two straight floor edges, one along X and one along Y, gave no `{SET_FRAME}` \
                 or pressing it traced no `{FRAME}`."
            )));
        };
        report.note(format!("system: {}", set.raw));
        let origin = three(&set, "origin").unwrap_or([f64::NAN; 3]);
        let z = three(&set, "z").unwrap_or([f64::NAN; 3]);
        let right = set.get("set") == Some("1")
            && near(origin[0].abs(), FLOOR_HALF)
            && near(origin[1].abs(), FLOOR_HALF)
            && near(origin[2], 0.0)
            && near(z[2].abs(), 1.0);
        if !right {
            return Ok(Some(format!(
                "★★★ two floor edges set the system `{}`; its origin must be a floor corner \
                 (±{FLOOR_HALF}, ±{FLOOR_HALF}, 0) and its Z the floor's normal.",
                set.raw
            )));
        }
        if let Some(failure) = self.corner_in_frame(report, by_x)? {
            return Ok(Some(failure));
        }
        let back = self.traced(MODEL_FRAME, FRAME)?;
        let right = back.as_ref().is_some_and(|l| l.get("set") == Some("0"));
        Ok((!right).then(|| {
            format!(
                "★★★ `{MODEL_FRAME}` traced `{}`; it must go back to the model's system, set=0.",
                back.as_ref().map_or("nothing", |l| l.raw.as_str())
            )
        }))
    }

    /// A floor corner picked in the system: 0 or 4 along X and Y, 0 along Z.
    fn corner_in_frame(&self, report: &mut CheckReport, spot: &Landed) -> Result<Option<String>> {
        if !self.0.choose(&format!("{ENTITY}2"))? {
            return Ok(Some(format!("★★★ the Measure tab drew no `{ENTITY}2`.")));
        }
        self.drive().click(AGAIN, "", "model-select-cleared")?;
        let Some(corner) = self.0.click(&spot.spot)? else {
            return Ok(Some(
                "★★★ a corner pick on the floor traced nothing.".to_owned(),
            ));
        };
        report.note(format!("corner in the system: {}", corner.raw));
        let local = three(&corner, "local").unwrap_or([f64::NAN; 3]);
        let on_side = |v: f64| near(v, 0.0) || near(v, FLOOR_SIDE);
        let right = on_side(local[0]) && on_side(local[1]) && near(local[2], 0.0);
        Ok((!right).then(|| {
            format!(
                "★★★ a floor corner in the system the floor's edges set traced `{}`; its local \
                 X and Y must each be 0 or {FLOOR_SIDE} and its Z 0.",
                corner.raw
            )
        }))
    }
}
