//! `checks::model_lights` — **the 3D viewer switches the model's lights off
//! and on: a glowing part by its second box and by the menu, a folder of
//! them, a lamp, and all the gathered ones at once**
//!
//! Opens `fixtures/glow-room.prc` in the viewer, off the desktop through the
//! scripted pointer. Each switch must trace a `model-view-lights` line naming
//! what is off and must change the picture; switching back must return the
//! picture it had. See `docs/modules/ui-verify/checks/model_lights.md`.

use std::collections::BTreeSet;

use crate::checks::driving::repo_fixture;
use crate::checks::model_inferences::open_viewer;
use crate::checks::model_pick::{Drive, set};
use crate::checks::model_shadow_sun::rendered;
use crate::checks::model_view_window::launch_with_model_file;
use crate::checks::{Check, CheckContext};
use crate::error::Result;
use crate::report::CheckReport;
use crate::trace::TraceLine;

const METHOD: &str = "python fixtures/glow-room.PROVENANCE.py <pdfcer.exe>";
const LIGHTS: &str = "model-view-lights";
const STATE: &str = "model-view-parts-state";
/// `bulb`: glows, beside parts that do not, so it is gathered.
const BULB: usize = 3;
/// `Fittings`: a folder of the model's own holding only glowing parts.
const FITTINGS: usize = 4;
/// The root, on which the file stores both lamps.
const ROOT: usize = 0;

/// See the module documentation.
pub struct TheViewerSwitchesLights;

impl Check for TheViewerSwitchesLights {
    fn name(&self) -> &'static str {
        "the_3d_viewer_switches_lights_off_and_on"
    }

    fn defect(&self) -> &'static str {
        "a glowing part, a folder of them or a lamp in a 3D model cannot be switched off and \
         back on from the part list or the right-click menu, or switching changes no picture"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        let driven = repo_fixture("glow-room.prc", METHOD)
            .and_then(|m| {
                launch_with_model_file(ctx, &mut report, "model-lights", &m.to_string_lossy())
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
                Lights(&drive).run(&mut report)
            });
        match driven {
            Ok(Some(failure)) => report.fail(failure),
            Ok(None) => report.pass(),
            Err(why) => report.from_error(&why),
        }
    }
}

struct Lights<'a>(&'a Drive<'a>);

impl Lights<'_> {
    /// The picture's hash once it has settled.
    fn picture(&self) -> Result<String> {
        self.0.session.settle(60);
        Ok(rendered(self.0.session, ["hash"])?.map_or_else(|| "-".to_owned(), |[h]| h))
    }

    /// Click `region` and judge the lights line it traces.
    fn step(
        &self,
        report: &mut CheckReport,
        region: &str,
        off: (&[usize], &[usize]),
        start: &str,
    ) -> Result<Option<String>> {
        let line = self.0.click(region, "", LIGHTS)?;
        self.judge(report, region, line, off, start)
    }

    /// Require `line` to leave the parts and lamps of `off` switched off, and
    /// the picture to differ from `start` exactly when something is off.
    fn judge(
        &self,
        report: &mut CheckReport,
        what: &str,
        line: Option<TraceLine>,
        (dark, lamps_off): (&[usize], &[usize]),
        start: &str,
    ) -> Result<Option<String>> {
        let Some(line) = line else {
            return Ok(Some(format!(
                "★★★ `{what}` traced no `{LIGHTS}` line: it switches nothing."
            )));
        };
        let now = self.picture()?;
        report.note(format!("{what}: {} picture={now}", line.raw));
        let want = |all: &[usize]| all.iter().copied().collect::<BTreeSet<usize>>();
        if set(&line, "dark") != want(dark) || set(&line, "lamps-off") != want(lamps_off) {
            return Ok(Some(format!(
                "★★★ `{what}` traced `{}`; it must leave parts {dark:?} and lamps {lamps_off:?} off.",
                line.raw
            )));
        }
        let all_on = dark.is_empty() && lamps_off.is_empty();
        if all_on != (now == start) {
            return Ok(Some(format!(
                "★★★ after `{what}` the picture is {now} and it opened as {start}: a light \
                 switched off must change the picture and switched back on must return it."
            )));
        }
        Ok(None)
    }

    /// The root's light box switches the lamps stored on it with the parts
    /// under it, and the lamps' own rows stay on.
    fn root(&self, report: &mut CheckReport, start: &str) -> Result<Option<String>> {
        let region = format!("model3d.part.glow.{ROOT}");
        for (dark, drawn) in [(&[ROOT][..], "0"), (&[][..], "2")] {
            let line = self.0.click(&region, "", LIGHTS)?;
            if let Some(line) = &line
                && line.get("lamps-drawn") != Some(drawn)
            {
                return Ok(Some(format!(
                    "★★★ `{region}` traced `{}`; the root carries both lamps, so {drawn} must \
                     be drawn.",
                    line.raw
                )));
            }
            let failure = self.judge(report, &region, line, (dark, &[]), start)?;
            if failure.is_some() {
                return Ok(failure);
            }
        }
        Ok(None)
    }

    /// Choose `key` on the menu of the bulb's row.
    fn menu(&self, key: &str) -> Result<Option<TraceLine>> {
        let row = format!("model3d.part.name.{BULB}");
        self.0.click(&row, " btn=r", STATE)?;
        self.0.click(&format!("model3d.menu.{key}"), "", LIGHTS)
    }

    fn run(&self, report: &mut CheckReport) -> Result<Option<String>> {
        let start = self.picture()?;
        let trace = self.0.session.trace()?;
        let Some(open) = trace.events(LIGHTS).last().cloned() else {
            return Ok(Some(format!(
                "★★★ the viewer traced no `{LIGHTS}` line on opening: it knows no lights."
            )));
        };
        report.note(format!("open: {} picture={start}", open.raw));
        let carried = open.get("lamp-nodes") == Some("0|0");
        if set(&open, "gathered") != BTreeSet::from([BULB])
            || open.get("lamps") == Some("0")
            || !carried
        {
            return Ok(Some(format!(
                "the model opened as `{}`; it must carry two lamps on node {ROOT} and gather part \
                 {BULB} alone.",
                open.raw
            )));
        }
        let glow = format!("model3d.part.glow.{BULB}");
        let folder = format!("model3d.part.glow.{FITTINGS}");
        let gathered = format!("model3d.lights.part.{BULB}");
        let all = "model3d.lights.all";
        let steps: [(&str, &[usize], &[usize]); 10] = [
            (&glow, &[BULB], &[]),
            (&glow, &[], &[]),
            (&folder, &[FITTINGS], &[]),
            (&folder, &[], &[]),
            (&gathered, &[BULB], &[]),
            (&gathered, &[], &[]),
            ("model3d.lights.lamp.0", &[], &[0]),
            (all, &[], &[]),
            (all, &[BULB], &[0, 1]),
            (all, &[], &[]),
        ];
        for (region, dark, lamps) in steps {
            let failure = self.step(report, region, (dark, lamps), &start)?;
            if failure.is_some() {
                return Ok(failure);
            }
        }
        let failure = self.root(report, &start)?;
        if failure.is_some() {
            return Ok(failure);
        }
        self.0
            .click(&format!("model3d.part.name.{BULB}"), "", STATE)?;
        // The selection is drawn on the picture, so the menu is judged
        // against the picture with the bulb selected.
        let chosen = self.picture()?;
        let off = self.menu("light_off")?;
        let failure = self.judge(report, "Turn light off", off, (&[BULB], &[]), &chosen)?;
        if failure.is_some() {
            return Ok(failure);
        }
        let on = self.menu("light_on")?;
        self.judge(report, "Turn light on", on, (&[], &[]), &chosen)
    }
}
