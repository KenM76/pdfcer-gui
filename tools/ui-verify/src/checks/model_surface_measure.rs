//! `checks::model_surface_measure` — **the 3D viewer's Measure tab measures
//! between two picked surfaces: a distance for parallel ones, the angle for
//! others, and a distance on demand for any pair**
//!
//! Opens `fixtures/glow-room.prc` in the viewer, off the desktop through the
//! scripted pointer. See `docs/modules/ui-verify/checks/model_surface_measure.md`.

use crate::checks::driving::{count, declared_in, repo_fixture};
use crate::checks::model_behind::dot;
use crate::checks::model_inferences::open_viewer;
use crate::checks::model_light_meter::number;
use crate::checks::model_pick::Drive;
use crate::checks::model_view_window::{launch_with_model_file, ui_rect};
use crate::checks::viewer_options::{close, reach};
use crate::checks::{Check, CheckContext};
use crate::coords::WindowPoint;
use crate::error::Result;
use crate::geom::{LRect, Pt};
use crate::report::CheckReport;
use crate::trace::TraceLine;

const METHOD: &str = "python fixtures/glow-room.PROVENANCE.py <pdfcer.exe>";
const IMAGE: &str = "model3d.image";
const SWITCH: &str = "model3d.measure.on";
const SURFACES: &str = "model3d.measure.surfaces";
const FORCE: &str = "model3d.surface.force";
const AGAIN: &str = "model3d.measure.again";
const PICK: &str = "model-surface-pick";
const RESULT: &str = "model-surface-result";
const REFUSED: &str = "model-surface-refused";
/// How far a distance may stray from the one the two picks' points and
/// normals give: a fitted plane's float error, well under any gap measured.
const DISTANCE_SLACK: f64 = 1e-3;
/// How far an angle may stray from a right angle, in degrees.
const ANGLE_SLACK: f64 = 0.05;
/// The least gap between two parallel planes worth measuring.
const LEAST_GAP: f64 = 0.05;

/// See the module documentation.
pub struct TheViewerMeasuresBetweenSurfaces;

impl Check for TheViewerMeasuresBetweenSurfaces {
    fn name(&self) -> &'static str {
        "the_3d_viewer_measures_between_surfaces"
    }

    fn defect(&self) -> &'static str {
        "the 3D viewer's Measure tab cannot measure between two picked surfaces, two parallel \
         surfaces give no distance or the wrong one, two square surfaces give no right angle, or \
         asking for a distance always does not give one"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        let driven = repo_fixture("glow-room.prc", METHOD)
            .and_then(|m| {
                launch_with_model_file(
                    ctx,
                    &mut report,
                    "model-surface-measure",
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
                Measuring(&drive).run(&mut report)
            });
        match driven {
            Ok(Some(failure)) => report.fail(failure),
            Ok(None) => report.pass(),
            Err(why) => report.from_error(&why),
        }
    }
}

type Spot = (WindowPoint, Option<String>);

/// A spot clicked on the model, with the point and normal its pick traced.
struct Picked {
    spot: Spot,
    at: [f64; 3],
    normal: [f64; 3],
}

fn three(line: &TraceLine, key: &str) -> Option<[f64; 3]> {
    match line.get_f64_list(key)?.as_slice() {
        [x, y, z] => Some([*x, *y, *z]),
        _ => None,
    }
}

/// The gap between `b`'s point and the plane through `a` square to `a`'s normal.
fn gap(a: &Picked, b: &Picked) -> f64 {
    let d = [b.at[0] - a.at[0], b.at[1] - a.at[1], b.at[2] - a.at[2]];
    dot(d, a.normal).abs()
}

/// Two picks, the first and the second surface.
type Pair<'a> = (&'a Picked, &'a Picked);

/// Two picks on parallel planes a gap apart, and two on square ones.
fn pairs(picked: &[Picked]) -> (Option<Pair<'_>>, Option<Pair<'_>>) {
    let mut parallel = None;
    let mut square = None;
    for (i, a) in picked.iter().enumerate() {
        for b in &picked[i + 1..] {
            let along = dot(a.normal, b.normal).abs();
            if parallel.is_none() && along > 0.9999 && gap(a, b) > LEAST_GAP {
                parallel = Some((a, b));
            }
            if square.is_none() && along < 1e-4 {
                square = Some((a, b));
            }
        }
    }
    (parallel, square)
}

struct Measuring<'a>(&'a Drive<'a>);

impl Measuring<'_> {
    /// Press `region` on the options menu, opening it and its tab as needed;
    /// whether it was there to press.
    fn press(&self, region: &str) -> Result<bool> {
        let Drive {
            ctx,
            session,
            pointer,
        } = self.0;
        let Some((rect, vp)) = reach(session, pointer, ui_rect(ctx)?, region)? else {
            return Ok(false);
        };
        pointer.click_in(session, vp.as_deref(), WindowPoint::centre_of(rect))?;
        session.settle(20);
        Ok(true)
    }

    /// Turn the model off its opening view, which looks square at one face of
    /// everything, so sides at right angles to it come into sight.
    fn turn(&self) -> Result<bool> {
        let session = self.0.session;
        let Some((rect, vp)) = declared_in(&session.trace()?, ui_rect(self.0.ctx)?, IMAGE) else {
            return Ok(false);
        };
        let from = WindowPoint::centre_of(rect);
        let aside = Pt {
            x: from.x() + 70.0,
            y: from.y() + 70.0,
        };
        let to = WindowPoint::centre_of(LRect {
            min: aside,
            max: aside,
        });
        self.0
            .pointer
            .drag_in(session, vp.as_deref(), from, to, 8, "l")?;
        session.settle(40);
        Ok(true)
    }

    /// A grid of spots across the picture.
    fn spots(&self) -> Result<Vec<Spot>> {
        let trace = self.0.session.trace()?;
        let Some((rect, vp)) = declared_in(&trace, ui_rect(self.0.ctx)?, IMAGE) else {
            return Ok(Vec::new());
        };
        let steps = [-0.36, -0.24, -0.12, 0.0, 0.12, 0.24, 0.36];
        let mut found = Vec::new();
        for fy in steps {
            for fx in steps {
                let (dx, dy) = (rect.width() * fx, rect.height() * fy);
                let mut r = rect;
                (r.min.x, r.max.x, r.min.y, r.max.y) =
                    (r.min.x + dx, r.max.x + dx, r.min.y + dy, r.max.y + dy);
                found.push((WindowPoint::centre_of(r), vp.clone()));
            }
        }
        Ok(found)
    }

    /// Click `spot`; the pick line it traced, if it landed on the model.
    fn click(&self, (at, vp): &Spot) -> Result<Option<TraceLine>> {
        let session = self.0.session;
        let before = count(session, PICK)?;
        self.0.pointer.click_in(session, vp.as_deref(), *at)?;
        session.settle(20);
        Ok(session.trace()?.events(PICK).nth(before).cloned())
    }

    /// Every spot of the grid that lands on the model, with its pick.
    fn survey(&self) -> Result<Vec<Picked>> {
        let mut picked = Vec::new();
        for spot in self.spots()? {
            let Some(line) = self.click(&spot)? else {
                continue;
            };
            if let (Some(at), Some(normal)) = (three(&line, "at"), three(&line, "normal")) {
                picked.push(Picked { spot, at, normal });
            }
        }
        Ok(picked)
    }

    /// Start again, pick `a` then `b`, and return the result line.
    fn measure(&self, a: &Picked, b: &Picked) -> Result<std::result::Result<TraceLine, String>> {
        let session = self.0.session;
        self.0.click(AGAIN, "", "model-surface-cleared")?;
        let (results, refused) = (count(session, RESULT)?, count(session, REFUSED)?);
        for p in [a, b] {
            if self.click(&p.spot)?.is_none() {
                return Ok(Err(format!(
                    "★★★ a spot that picked a surface before picked none: {:?}.",
                    p.at
                )));
            }
        }
        let trace = session.trace()?;
        if let Some(line) = trace.events(RESULT).nth(results) {
            return Ok(Ok(line.clone()));
        }
        Ok(Err(match trace.events(REFUSED).nth(refused) {
            Some(line) => format!("★★★ two flat surfaces were refused: `{}`.", line.raw),
            None => "★★★ the second surface picked gave no measurement and no refusal.".to_owned(),
        }))
    }

    fn run(&self, report: &mut CheckReport) -> Result<Option<String>> {
        let ui = ui_rect(self.0.ctx)?;
        if !self.turn()? {
            return Ok(Some(format!("the viewer drew no `{IMAGE}`.")));
        }
        if !self.press(SWITCH)? {
            return Ok(Some(format!("the viewer drew no `{SWITCH}`.")));
        }
        if !self.press(SURFACES)? {
            return Ok(Some(format!(
                "★★★ the Measure tab drew no `{SURFACES}`: there is no surface measure."
            )));
        }
        close(self.0.session, self.0.pointer, ui)?;
        let picked = self.survey()?;
        report.note(format!(
            "{} spots of the grid landed on the model",
            picked.len()
        ));
        let (Some(parallel), Some(square)) = pairs(&picked) else {
            return Ok(Some(format!(
                "no two spots across the picture lie on parallel planes apart and on square \
                 ones: {} picks with normals {:?}.",
                picked.len(),
                picked.iter().map(|p| p.normal).collect::<Vec<_>>()
            )));
        };
        let failure = self.parallel(report, parallel)?;
        if failure.is_some() {
            return Ok(failure);
        }
        self.square(report, square)
    }

    /// Two parallel planes give their gap.
    fn parallel(&self, report: &mut CheckReport, (a, b): Pair<'_>) -> Result<Option<String>> {
        let line = match self.measure(a, b)? {
            Ok(line) => line,
            Err(why) => return Ok(Some(why)),
        };
        report.note(format!("parallel: {}", line.raw));
        let want = gap(a, b);
        let right = line.get("relation") == Some("parallel")
            && line.get("forced") == Some("0")
            && (number(&line, "distance") - want).abs() <= DISTANCE_SLACK
            && number(&line, "angle") <= 0.5;
        Ok((!right).then(|| {
            format!(
                "★★★ two parallel surfaces {want:.6} apart measured `{}`; it must be a distance \
                 of {want:.6}, not forced, at an angle of 0.",
                line.raw
            )
        }))
    }

    /// Two square planes give a right angle, and a distance once forced.
    fn square(&self, report: &mut CheckReport, (a, b): Pair<'_>) -> Result<Option<String>> {
        let line = match self.measure(a, b)? {
            Ok(line) => line,
            Err(why) => return Ok(Some(why)),
        };
        report.note(format!("square: {}", line.raw));
        let right_angle = (number(&line, "angle") - 90.0).abs() <= ANGLE_SLACK;
        if line.get("relation") != Some("angled") || !right_angle {
            return Ok(Some(format!(
                "★★★ two square surfaces measured `{}`; it must be an angle of 90.",
                line.raw
            )));
        }
        let before = count(self.0.session, RESULT)?;
        if !self.press(FORCE)? {
            return Ok(Some(format!("★★★ the Measure tab drew no `{FORCE}`.")));
        }
        close(self.0.session, self.0.pointer, ui_rect(self.0.ctx)?)?;
        let Some(forced) = self.0.session.trace()?.events(RESULT).nth(before).cloned() else {
            return Ok(Some(format!(
                "★★★ pressing `{FORCE}` with two surfaces picked measured them not again."
            )));
        };
        report.note(format!("forced: {}", forced.raw));
        let right = forced.get("relation") == Some("parallel")
            && forced.get("forced") == Some("1")
            && number(&forced, "distance") >= 0.0
            && (number(&forced, "angle") - 90.0).abs() <= ANGLE_SLACK;
        Ok((!right).then(|| {
            format!(
                "★★★ forced, two square surfaces measured `{}`; it must be a forced distance with \
                 the angle of 90 still said.",
                forced.raw
            )
        }))
    }
}
