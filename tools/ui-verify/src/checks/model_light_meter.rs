//! `checks::model_light_meter` — **the 3D viewer's Measure tab reads the
//! light arriving at a point picked on a surface, the reading falls when the
//! model's lights are switched off, and a circle averages many points**
//!
//! Opens `fixtures/glow-room.prc` in the viewer, off the desktop through the
//! scripted pointer. See `docs/modules/ui-verify/checks/model_light_meter.md`.

use crate::checks::driving::{count, declared_in, repo_fixture};
use crate::checks::model_inferences::open_viewer;
use crate::checks::model_pick::Drive;
use crate::checks::model_view_window::{launch_with_model_file, ui_rect};
use crate::checks::viewer_options::{close, reach};
use crate::checks::{Check, CheckContext};
use crate::coords::WindowPoint;
use crate::error::Result;
use crate::report::CheckReport;
use crate::trace::TraceLine;

const METHOD: &str = "python fixtures/glow-room.PROVENANCE.py <pdfcer.exe>";
const IMAGE: &str = "model3d.image";
const SWITCH: &str = "model3d.measure.on";
const LIGHT: &str = "model3d.measure.light";
const CIRCLE: &str = "model3d.light.circle";
const ALL_LIGHTS: &str = "model3d.lights.all";
const GLOW: &str = "model3d.light.glow";
const ASKED: &str = "model-light-asked";
const READING: &str = "model-light-reading";
const REFUSED: &str = "model-light-refused";
/// How far a reading's total may stray from the sum of its parts: the
/// trace's rounding, three times.
const SUM_SLACK: f64 = 4e-6;
/// Settles waited for a reading, which is worked out off the frame loop.
const WAITS: usize = 60;

/// See the module documentation.
pub struct TheViewerReadsTheLight;

impl Check for TheViewerReadsTheLight {
    fn name(&self) -> &'static str {
        "the_3d_viewer_reads_the_light_on_a_surface"
    }

    fn defect(&self) -> &'static str {
        "the 3D viewer's Measure tab has no light meter, a point picked on a surface gives no \
         reading, the reading does not fall when the model's lights are switched off, or a \
         circle round the point is not averaged over more than one point"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        let driven = repo_fixture("glow-room.prc", METHOD)
            .and_then(|m| {
                launch_with_model_file(ctx, &mut report, "model-light-meter", &m.to_string_lossy())
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
                Meter(&drive).run(&mut report)
            });
        match driven {
            Ok(Some(failure)) => report.fail(failure),
            Ok(None) => report.pass(),
            Err(why) => report.from_error(&why),
        }
    }
}

/// A reading line's number under `key`.
pub(crate) fn number(line: &TraceLine, key: &str) -> f64 {
    line.get(key)
        .and_then(|v| v.parse().ok())
        .unwrap_or(f64::NAN)
}

struct Meter<'a>(&'a Drive<'a>);

impl Meter<'_> {
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

    /// The spots tried across the picture, the centre first.
    fn spots(&self) -> Result<Vec<(WindowPoint, Option<String>)>> {
        let trace = self.0.session.trace()?;
        let Some((rect, vp)) = declared_in(&trace, ui_rect(self.0.ctx)?, IMAGE) else {
            return Ok(Vec::new());
        };
        let mut found = Vec::new();
        for fy in [0.0, 0.15, 0.3, -0.15] {
            for fx in [0.0, -0.15, 0.15, -0.3, 0.3] {
                let (dx, dy) = (rect.width() * fx, rect.height() * fy);
                let mut r = rect;
                (r.min.x, r.max.x, r.min.y, r.max.y) =
                    (r.min.x + dx, r.max.x + dx, r.min.y + dy, r.max.y + dy);
                found.push((WindowPoint::centre_of(r), vp.clone()));
            }
        }
        Ok(found)
    }

    /// Click `spot` with the meter chosen; the reading it gives, `None` where
    /// the click is off the model, or the sentence for no answer.
    fn read_at(
        &self,
        (at, vp): &(WindowPoint, Option<String>),
    ) -> Result<std::result::Result<Option<TraceLine>, String>> {
        let session = self.0.session;
        let (asked, read) = (count(session, ASKED)?, count(session, READING)?);
        let refused = count(session, REFUSED)?;
        self.0.pointer.click_in(session, vp.as_deref(), *at)?;
        session.settle(20);
        if count(session, ASKED)? == asked {
            return Ok(Ok(None));
        }
        for _ in 0..WAITS {
            let trace = session.trace()?;
            if let Some(line) = trace.events(READING).nth(read) {
                return Ok(Ok(Some(line.clone())));
            }
            if let Some(line) = trace.events(REFUSED).nth(refused) {
                return Ok(Err(format!(
                    "★★★ the meter refused a point on the model: `{}`.",
                    line.raw
                )));
            }
            session.settle(15);
        }
        Ok(Err(
            "★★★ the meter was asked for a reading and gave neither a reading nor a refusal."
                .to_owned(),
        ))
    }

    /// The first spot a lamp shines on straight, and its reading.
    #[allow(clippy::type_complexity)]
    fn first(
        &self,
    ) -> Result<std::result::Result<((WindowPoint, Option<String>), TraceLine), String>> {
        for spot in self.spots()? {
            match self.read_at(&spot)? {
                Ok(Some(line)) if number(&line, "direct") > 0.0 => return Ok(Ok((spot, line))),
                Ok(_) => {}
                Err(why) => return Ok(Err(why)),
            }
        }
        Ok(Err(format!(
            "★★★ no point clicked across the picture reads any light straight from the lamp: {} asked, {} read.",
            count(self.0.session, ASKED)?,
            count(self.0.session, READING)?
        )))
    }

    /// Read again at `spot`, which gave a reading before.
    fn again(
        &self,
        spot: &(WindowPoint, Option<String>),
        when: &str,
    ) -> Result<std::result::Result<TraceLine, String>> {
        Ok(match self.read_at(spot)? {
            Ok(Some(line)) => Ok(line),
            Ok(None) => Err(format!(
                "★★★ {when}, the click that gave a reading before asked for none."
            )),
            Err(why) => Err(format!("{when}: {why}")),
        })
    }

    /// The next reading after the `seen`th, worked out without a click.
    fn next_reading(&self, seen: usize) -> Result<Option<TraceLine>> {
        for _ in 0..WAITS {
            if let Some(line) = self.0.session.trace()?.events(READING).nth(seen) {
                return Ok(Some(line.clone()));
            }
            self.0.session.settle(15);
        }
        Ok(None)
    }

    /// *Count glowing parts* off reads the same point again with no light
    /// from the glow, and on gives back the reading `lit` it started from.
    fn glow(&self, lit: &TraceLine) -> Result<Option<String>> {
        let counted = number(lit, "emitted");
        if counted.is_nan() || counted <= 0.0 {
            return Ok(Some(format!(
                "★★★ the reading `{}` counts no light from glow-room's glowing panel, which faces the floor the check picks: the glow is not counted by default.",
                lit.raw
            )));
        }
        let seen = count(self.0.session, READING)?;
        if !self.press(GLOW)? {
            return Ok(Some(format!(
                "★★★ the Measure tab drew no `{GLOW}` for a model with a glowing part."
            )));
        }
        close(self.0.session, self.0.pointer, ui_rect(self.0.ctx)?)?;
        let Some(out) = self.next_reading(seen)? else {
            return Ok(Some(
                "★★★ switching *Count glowing parts* off did not read the picked point again."
                    .to_owned(),
            ));
        };
        let left_out = number(&out, "emitted") == 0.0
            && number(&out, "total") < number(lit, "total")
            && number(&out, "notes") > number(lit, "notes");
        if !left_out {
            return Ok(Some(format!(
                "★★★ with the glow left out the point reads `{}`; it must count no glow, read less than `{}`, and add the note that the glow is left out.",
                out.raw, lit.raw
            )));
        }
        if !self.press(GLOW)? {
            return Ok(Some(format!("★★★ `{GLOW}` went away once switched off.")));
        }
        close(self.0.session, self.0.pointer, ui_rect(self.0.ctx)?)?;
        match self.next_reading(seen + 1)? {
            Some(back) if number(&back, "emitted") == counted => Ok(None),
            Some(back) => Ok(Some(format!(
                "★★★ switched back on, the glow reads `{}`, not the `{}` it started from.",
                back.raw, lit.raw
            ))),
            None => Ok(Some(
                "★★★ switching *Count glowing parts* back on did not read the point again."
                    .to_owned(),
            )),
        }
    }

    fn run(&self, report: &mut CheckReport) -> Result<Option<String>> {
        let Drive {
            ctx,
            session,
            pointer,
        } = self.0;
        let ui = ui_rect(ctx)?;
        if !self.press(SWITCH)? {
            return Ok(Some(format!("the viewer drew no `{SWITCH}`.")));
        }
        if !self.press(LIGHT)? {
            return Ok(Some(format!(
                "★★★ the Measure tab drew no `{LIGHT}`: there is no light meter."
            )));
        }
        close(session, pointer, ui)?;
        let (spot, lit) = match self.first()? {
            Ok(found) => found,
            Err(why) => return Ok(Some(why)),
        };
        report.note(format!("lights on: {}", lit.raw));
        let total = number(&lit, "total");
        let parts = number(&lit, "direct")
            + number(&lit, "emitted")
            + number(&lit, "bounced")
            + number(&lit, "ambient");
        // A number the line lacks is NaN, which fails every comparison.
        let sound = total > 0.0 && (total - parts).abs() <= SUM_SLACK;
        if !sound {
            return Ok(Some(format!(
                "★★★ the reading `{}` must be above zero and the sum of its four parts ({parts}).",
                lit.raw
            )));
        }
        let alone = lit.get("radius") == Some("0.000000") && lit.get("on") == Some("1");
        if !alone || lit.get("lit") != Some("1") {
            return Ok(Some(format!(
                "★★★ the reading `{}` must be of the one point picked, lit by the one lamp.",
                lit.raw
            )));
        }
        if let Some(failure) = self.glow(&lit)? {
            return Ok(Some(failure));
        }
        self.unlit_then_circle(report, &spot, total)
    }

    /// With every listed light off the lamps' share must vanish; with a
    /// circle chosen the reading must average several surface points.
    fn unlit_then_circle(
        &self,
        report: &mut CheckReport,
        spot: &(WindowPoint, Option<String>),
        total: f64,
    ) -> Result<Option<String>> {
        let Drive {
            ctx,
            session,
            pointer,
        } = self.0;
        let ui = ui_rect(ctx)?;
        self.0.click(ALL_LIGHTS, "", "model-view-lights")?;
        let dark = match self.again(spot, "with every listed light switched off")? {
            Ok(line) => line,
            Err(why) => return Ok(Some(why)),
        };
        report.note(format!("lights off: {}", dark.raw));
        // The list's switch leaves on the glowing part kept in a folder of its
        // own, and its glow is counted: what remains is that glow alone.
        let fell = number(&dark, "direct") == 0.0
            && number(&dark, "ambient") == 0.0
            && number(&dark, "total") < total;
        if !fell {
            return Ok(Some(format!(
                "★★★ with both lamps switched off a point on the model reads `{}`; it must \
                 read no lamp and no ambient light, and less than the {total} read with them on.",
                dark.raw
            )));
        }
        self.0.click(ALL_LIGHTS, "", "model-view-lights")?;
        if !self.press(CIRCLE)? {
            return Ok(Some(format!("★★★ the Measure tab drew no `{CIRCLE}`.")));
        }
        close(session, pointer, ui)?;
        let round = match self.again(spot, "with a circle chosen")? {
            Ok(line) => line,
            Err(why) => return Ok(Some(why)),
        };
        report.note(format!("circle: {}", round.raw));
        let on: usize = round.get("on").and_then(|n| n.parse().ok()).unwrap_or(0);
        let averaged = number(&round, "radius") > 0.0 && on >= 2 && number(&round, "total") > 0.0;
        if !averaged {
            return Ok(Some(format!(
                "★★★ the circle's reading `{}` must have a radius, more than one point on the \
                 surface and a total above zero.",
                round.raw
            )));
        }
        // A picture for the report; the verdict is the trace above.
        let png = ctx.out("model-light-meter.circle.png");
        let _ = pointer.screenshot_in(session, spot.1.as_deref(), &png);
        Ok(None)
    }
}
