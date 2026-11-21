//! `checks::model_select_measure` — **the 3D viewer's *Face, edge, corner*
//! measure gives a picked face's area, an edge's length, a corner's position,
//! and the smallest and largest distances between two picks**
//!
//! Opens `fixtures/glow-room.prc` in the viewer, off the desktop through the
//! scripted pointer. See `docs/modules/ui-verify/checks/model_select_measure.md`.

use crate::checks::driving::{count, repo_fixture};
use crate::checks::model_inferences::open_viewer;
use crate::checks::model_light_meter::number;
use crate::checks::model_pick::Drive;
use crate::checks::model_surface_measure::{Measuring, Spot, three};
use crate::checks::model_view_window::{launch_with_model_file, ui_rect};
use crate::checks::viewer_options::close;
use crate::checks::{Check, CheckContext};
use crate::error::Result;
use crate::report::CheckReport;
use crate::trace::TraceLine;

const METHOD: &str = "python fixtures/glow-room.PROVENANCE.py <pdfcer.exe>";
pub(crate) const SWITCH: &str = "model3d.measure.on";
pub(crate) const SELECT: &str = "model3d.measure.select";
pub(crate) const ENTITY: &str = "model3d.select.entity.";
pub(crate) const AGAIN: &str = "model3d.measure.again";
const FIGURES: &str = "model-select-figures";
const REFUSED: &str = "model-select-refused";
const DISTANCE: &str = "model-select-distance";
/// Float error in a figure summed from single-precision corners.
const SLACK: f64 = 1e-4;
/// The fixture's geometry, from its provenance, in the viewer's frame: the
/// file's y is the viewer's height `z`, and the file's z the viewer's `-y`.
/// The floor is a 4 by 4 square at height 0.
const FLOOR_AREA: f64 = 16.0;
const FLOOR_SIDE: f64 = 4.0;
const FLOOR_HALF: f64 = 2.0;
/// `tube-b`, x -1.0 to -0.8, y -0.5 to 1.5, height 0 to 0.2, beside the box,
/// x and y -0.5 to 0.5, height 0 to 1: their nearest sides are 0.3 apart,
/// their farthest corners `sqrt(1.5^2 + 2^2 + 1^2)`.
pub(crate) const TUBE_B: (f64, f64) = (-1.0, -0.8);
const BOX_TO_TUBE_NEAREST: f64 = 0.3;
const BOX_TO_TUBE_FARTHEST: f64 = 2.692_582_404;
/// The box's six unit squares.
const BOX_SIDE_AREA: f64 = 1.0;

/// See the module documentation.
pub struct TheViewerMeasuresFacesEdgesCorners;

impl Check for TheViewerMeasuresFacesEdgesCorners {
    fn name(&self) -> &'static str {
        "the_3d_viewer_measures_faces_edges_and_corners"
    }

    fn defect(&self) -> &'static str {
        "the 3D viewer's Measure tab cannot pick a face, an edge or a corner, a picked face gives \
         the wrong area or perimeter, an edge the wrong length, a corner the wrong position, or two \
         picks the wrong smallest or largest distance"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        let driven = repo_fixture("glow-room.prc", METHOD)
            .and_then(|m| {
                launch_with_model_file(
                    ctx,
                    &mut report,
                    "model-select-measure",
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
                Selecting(Measuring(&drive)).run(&mut report)
            });
        match driven {
            Ok(Some(failure)) => report.fail(failure),
            Ok(None) => report.pass(),
            Err(why) => report.from_error(&why),
        }
    }
}

/// A spot that landed on the model, with the figures its face pick traced.
pub(crate) struct Landed {
    pub(crate) spot: Spot,
    pub(crate) at: [f64; 3],
    line: TraceLine,
}

impl Landed {
    pub(crate) fn on_floor(&self) -> bool {
        self.at[2].abs() <= SLACK
    }

    /// How far inside the floor's outline, so the floor spot nearest an edge
    /// is the one to pick edges and corners with.
    pub(crate) fn inset(&self) -> f64 {
        FLOOR_HALF - self.at[0].abs().max(self.at[1].abs())
    }

    pub(crate) fn within(&self, (lo, hi): (f64, f64)) -> bool {
        (lo - SLACK..=hi + SLACK).contains(&self.at[0]) && self.at[2] > SLACK
    }

    pub(crate) fn on_box(&self) -> bool {
        let [x, y, z] = self.at;
        x.abs() <= 0.5 + SLACK && y.abs() <= 0.5 + SLACK && z > SLACK
    }
}

fn near(line: &TraceLine, key: &str, want: f64) -> bool {
    (number(line, key) - want).abs() <= SLACK
}

pub(crate) struct Selecting<'a>(pub(crate) Measuring<'a>);

impl Selecting<'_> {
    pub(crate) fn drive(&self) -> &Drive<'_> {
        self.0.0
    }

    /// Press `region` on the options menu and close the menu again.
    pub(crate) fn choose(&self, region: &str) -> Result<bool> {
        let pressed = self.0.press(region)?;
        close(
            self.drive().session,
            self.drive().pointer,
            ui_rect(self.drive().ctx)?,
        )?;
        Ok(pressed)
    }

    /// Click `spot`; the figures or refusal line it traced, if it landed.
    pub(crate) fn click(&self, (at, vp): &Spot) -> Result<Option<TraceLine>> {
        let session = self.drive().session;
        let (figures, refused) = (count(session, FIGURES)?, count(session, REFUSED)?);
        self.drive().pointer.click_in(session, vp.as_deref(), *at)?;
        session.settle(20);
        let trace = session.trace()?;
        Ok(trace
            .events(FIGURES)
            .nth(figures)
            .or_else(|| trace.events(REFUSED).nth(refused))
            .cloned())
    }

    /// Every spot of the grid that lands on the model, each picked alone.
    pub(crate) fn survey(&self) -> Result<Vec<Landed>> {
        let mut landed = Vec::new();
        for spot in self.0.spots()? {
            self.drive().click(AGAIN, "", "model-select-cleared")?;
            let Some(line) = self.click(&spot)? else {
                continue;
            };
            if let Some(at) = three(&line, "at") {
                landed.push(Landed { spot, at, line });
            }
        }
        Ok(landed)
    }

    fn run(&self, report: &mut CheckReport) -> Result<Option<String>> {
        if !self.0.turn()? {
            return Ok(Some("the viewer drew no picture to turn.".to_owned()));
        }
        if !self.0.press(SWITCH)? {
            return Ok(Some(format!("the viewer drew no `{SWITCH}`.")));
        }
        if !self.choose(SELECT)? {
            return Ok(Some(format!(
                "★★★ the Measure tab drew no `{SELECT}`: there is no face, edge and corner measure."
            )));
        }
        let landed = self.survey()?;
        report.note(format!(
            "{} spots of the grid landed on the model",
            landed.len()
        ));
        for failure in [self.faces(report, &landed), self.between(report, &landed)?] {
            if failure.is_some() {
                return Ok(failure);
            }
        }
        let floor = landed
            .iter()
            .filter(|l| l.on_floor())
            .min_by(|a, b| a.inset().total_cmp(&b.inset()));
        let Some(floor) = floor else {
            return Ok(Some("no spot of the grid landed on the floor.".to_owned()));
        };
        self.edge_and_corner(report, floor)
    }

    /// The floor's face is its 4 by 4 square; a box face, which the file gives
    /// only as the whole box, is the unit square of flat patch at the click.
    fn faces(&self, report: &mut CheckReport, landed: &[Landed]) -> Option<String> {
        let floor = landed.iter().find(|l| l.on_floor())?;
        report.note(format!("floor face: {}", floor.line.raw));
        let right = floor.line.event == FIGURES
            && near(&floor.line, "area", FLOOR_AREA)
            && near(&floor.line, "perimeter", FLOOR_AREA)
            && floor.line.get("shape") == Some("plane");
        if !right {
            return Some(format!(
                "★★★ a face pick on the floor traced `{}`; it must be a plane of area \
                 {FLOOR_AREA} and perimeter {FLOOR_AREA}.",
                floor.line.raw
            ));
        }
        let boxed = landed.iter().find(|l| l.on_box())?;
        report.note(format!("box face: {}", boxed.line.raw));
        let right =
            near(&boxed.line, "area", BOX_SIDE_AREA) && boxed.line.get("source") == Some("grown");
        (!right).then(|| {
            format!(
                "★★★ a face pick on the box traced `{}`; it must be the grown flat patch, source=grown, of area {BOX_SIDE_AREA}.",
                boxed.line.raw
            )
        })
    }

    /// The box and `tube-b`, picked as faces of a file that separates none,
    /// are the whole parts, 0.3 apart at their nearest.
    fn between(&self, report: &mut CheckReport, landed: &[Landed]) -> Result<Option<String>> {
        let (Some(a), Some(b)) = (
            landed.iter().find(|l| l.on_box()),
            landed.iter().find(|l| l.within(TUBE_B)),
        ) else {
            return Ok(Some(format!(
                "no two spots of the grid landed on the box and on tube-b: {:?}.",
                landed.iter().map(|l| l.at).collect::<Vec<_>>()
            )));
        };
        let session = self.drive().session;
        self.drive().click(AGAIN, "", "model-select-cleared")?;
        let before = count(session, DISTANCE)?;
        for l in [a, b] {
            if self.click(&l.spot)?.is_none() {
                return Ok(Some(format!(
                    "★★★ a spot that picked before picked nothing: {:?}.",
                    l.at
                )));
            }
        }
        let Some(line) = session.trace()?.events(DISTANCE).nth(before).cloned() else {
            return Ok(Some(
                "★★★ a second pick measured no distance from the first.".to_owned(),
            ));
        };
        report.note(format!("box to tube-b: {}", line.raw));
        // Each side is measured from the triangles its figures describe, and
        // says which: the pick line's `source` and the distance's per-side
        // source must agree.
        for (l, key) in [(a, "a_source"), (b, "b_source")] {
            if l.line.get("source") != line.get(key) {
                return Ok(Some(format!(
                    "★★★ a pick traced source `{}` but its distance side `{key}` is `{}`. `{}`",
                    l.line.get("source").unwrap_or("-"),
                    line.get(key).unwrap_or("-"),
                    line.raw
                )));
            }
        }
        let grown = [a, b].iter().any(|l| l.line.get("source") == Some("grown"));
        let whole_pair = near(&line, "minimum", BOX_TO_TUBE_NEAREST)
            && near(&line, "maximum", BOX_TO_TUBE_FARTHEST);
        if !grown {
            return Ok((!whole_pair).then(|| {
                format!(
                    "★★★ the box and tube-b measured `{}` as whole parts; the smallest distance \
                     must be {BOX_TO_TUBE_NEAREST} and the largest {BOX_TO_TUBE_FARTHEST}.",
                    line.raw
                )
            }));
        }
        // A grown patch is a subset of its part, so its span lies inside the
        // whole parts' span. On this fixture the patches exclude the parts'
        // nearest pair, so measuring the whole parts is the defect.
        // A missing number is NaN, which fails both comparisons.
        let (min, max) = (number(&line, "minimum"), number(&line, "maximum"));
        let inside = min >= BOX_TO_TUBE_NEAREST - 1e-6 && max <= BOX_TO_TUBE_FARTHEST + 1e-6;
        Ok((!inside || whole_pair).then(|| {
            format!(
                "★★★ grown faces measured `{}`: the span must lie inside the whole parts' \
                 {BOX_TO_TUBE_NEAREST}..{BOX_TO_TUBE_FARTHEST} and not be that pair.",
                line.raw
            )
        }))
    }

    /// An edge picked on the floor is one 4-long side; a corner is one of its
    /// four corners.
    fn edge_and_corner(&self, report: &mut CheckReport, floor: &Landed) -> Result<Option<String>> {
        if !self.choose(&format!("{ENTITY}1"))? {
            return Ok(Some(format!("★★★ the Measure tab drew no `{ENTITY}1`.")));
        }
        self.drive().click(AGAIN, "", "model-select-cleared")?;
        let Some(edge) = self.click(&floor.spot)? else {
            return Ok(Some(
                "★★★ an edge pick on the floor traced nothing.".to_owned(),
            ));
        };
        report.note(format!("floor edge: {}", edge.raw));
        let right = edge.event == FIGURES
            && edge.get("entity") == Some("edge")
            && near(&edge, "length", FLOOR_SIDE)
            && edge.get("shape") == Some("line");
        if !right {
            return Ok(Some(format!(
                "★★★ an edge pick on the floor traced `{}`; it must be a straight edge {FLOOR_SIDE} long.",
                edge.raw
            )));
        }
        if !self.choose(&format!("{ENTITY}2"))? {
            return Ok(Some(format!("★★★ the Measure tab drew no `{ENTITY}2`.")));
        }
        self.drive().click(AGAIN, "", "model-select-cleared")?;
        let Some(corner) = self.click(&floor.spot)? else {
            return Ok(Some(
                "★★★ a corner pick on the floor traced nothing.".to_owned(),
            ));
        };
        report.note(format!("floor corner: {}", corner.raw));
        let at = three(&corner, "corner").unwrap_or([f64::NAN; 3]);
        let right = corner.get("entity") == Some("corner")
            && (at[0].abs() - FLOOR_HALF).abs() <= SLACK
            && (at[1].abs() - FLOOR_HALF).abs() <= SLACK
            && at[2].abs() <= SLACK;
        Ok((!right).then(|| {
            format!(
                "★★★ a corner pick on the floor traced `{}`; it must be one of the floor's corners \
                 (±{FLOOR_HALF}, ±{FLOOR_HALF}, 0).",
                corner.raw
            )
        }))
    }
}
