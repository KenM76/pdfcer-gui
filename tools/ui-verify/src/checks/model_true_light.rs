//! `checks::model_true_light` — **the 3D viewer draws at a true light level:
//! *True light level* draws the picture at an exposure fixed in lux, a
//! higher white point draws it darker, and the metered picture differs**
//!
//! Opens `fixtures/lux-room.prc` in the viewer, off the desktop through the
//! scripted pointer. See `docs/modules/ui-verify/checks/model_true_light.md`.

use crate::checks::driving::repo_fixture;
use crate::checks::model_inferences::open_viewer;
use crate::checks::model_view_window::{launch_with_model_file, ui_rect};
use crate::checks::viewer_options::{close, reach};
use crate::checks::{Check, CheckContext};
use crate::coords::WindowPoint;
use crate::error::Result;
use crate::input::scripted::ScriptedPointer;
use crate::launch::Session;
use crate::report::CheckReport;
use crate::trace::TraceLine;

const METHOD: &str = "python fixtures/lux-room.PROVENANCE.py <pdfcer.exe>";
const SWITCH: &str = "model3d.true_light";
const WHITE: &str = "model3d.white_lux";
const RENDERED: &str = "model-view-rendered";
/// The white point typed: twice the default 500 lux.
const DOUBLED: u32 = 1000;
/// How far apart, in 0–255 mean luma, two exposures must draw the model:
/// well above a frame's rounding, well below the halving a doubled white
/// point comes near.
const DARKER_BY: f64 = 4.0;
/// Settles waited for a picture drawn after a change.
const WAITS: usize = 60;

/// See the module documentation.
pub struct TheViewerDrawsATrueLightLevel;

impl Check for TheViewerDrawsATrueLightLevel {
    fn name(&self) -> &'static str {
        "the_3d_viewer_draws_at_a_true_light_level"
    }

    fn defect(&self) -> &'static str {
        "the 3D viewer has no True light level switch for a model whose lamps state their \
         strength, the switch does not draw the picture at a fixed exposure in lux, the \
         picture drawn in lux is the metered one, or doubling the lux drawn as full white \
         does not draw the model darker"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        let driven = repo_fixture("lux-room.prc", METHOD)
            .and_then(|m| {
                launch_with_model_file(ctx, &mut report, "model-true-light", &m.to_string_lossy())
            })
            .and_then(|(session, pointer)| {
                if let Some(failure) = open_viewer(ctx, &session, &pointer)? {
                    return Ok(Some(failure));
                }
                Drive {
                    ui: ui_rect(ctx)?,
                    session: &session,
                    pointer: &pointer,
                }
                .run(&mut report)
            });
        match driven {
            Ok(Some(failure)) => report.fail(failure),
            Ok(None) => report.pass(),
            Err(why) => report.from_error(&why),
        }
    }
}

struct Drive<'a> {
    ui: &'a str,
    session: &'a Session,
    pointer: &'a ScriptedPointer,
}

/// The card's bounce has arrived, or the processor drew the picture: the
/// frame a state settles on, so two states are compared settled.
fn settled(line: &TraceLine) -> bool {
    line.get("gpu-bounce") != Some("waiting")
}

/// `key`'s number on `line`; NaN, which fails every comparison, if absent.
fn number(line: &TraceLine, key: &str) -> f64 {
    line.get(key)
        .and_then(|v| v.parse().ok())
        .unwrap_or(f64::NAN)
}

impl Drive<'_> {
    /// The first picture drawn after the `seen`th that `done` accepts.
    fn picture_after(
        &self,
        seen: usize,
        done: impl Fn(&TraceLine) -> bool,
    ) -> Result<Option<TraceLine>> {
        for tries in 0..WAITS {
            let trace = self.session.trace()?;
            let found: Vec<&TraceLine> = trace
                .events(RENDERED)
                .skip(seen)
                .filter(|l| done(l))
                .collect();
            if let Some(line) = found.iter().find(|l| settled(l)) {
                return Ok(Some((*line).clone()));
            }
            if let Some(line) = found.last().filter(|_| tries + 1 == WAITS) {
                return Ok(Some((*line).clone()));
            }
            self.session.settle(15);
        }
        Ok(None)
    }

    fn drawn(&self) -> Result<usize> {
        Ok(self.session.trace()?.events(RENDERED).count())
    }

    fn run(&self, report: &mut CheckReport) -> Result<Option<String>> {
        let Some(metered) = self.picture_after(0, |l| l.get("luma").is_some())? else {
            return Ok(Some(format!(
                "the viewer drew no `{RENDERED}` line carrying `luma=`."
            )));
        };
        report.note(format!("metered: {}", metered.raw));
        if metered.get("exposure") != Some("metered") {
            return Ok(Some(format!(
                "★★★ the opening picture `{}` is not drawn metered: the switch must be off by default.",
                metered.raw
            )));
        }
        let seen = self.drawn()?;
        let Some((rect, vp)) = reach(self.session, self.pointer, self.ui, SWITCH)? else {
            return Ok(Some(format!(
                "★★★ the viewer drew no `{SWITCH}` for a model whose lamps state their candela."
            )));
        };
        self.pointer
            .click_in(self.session, vp.as_deref(), WindowPoint::centre_of(rect))?;
        self.session.settle(20);
        close(self.session, self.pointer, self.ui)?;
        let fixed = |want: f64| {
            move |l: &TraceLine| {
                l.get("exposure") == Some("fixed") && number(l, "white-lux") == want
            }
        };
        let Some(at_500) = self.picture_after(seen, fixed(500.0))? else {
            return Ok(Some(self.not_fixed(seen, "switched on")?));
        };
        report.note(format!("fixed at 500 lux: {}", at_500.raw));
        // The picture's height changes with the note said under it, so a
        // changed hash alone would pass on the switch's own wording.
        let (metered_luma, luma_500) = (number(&metered, "luma"), number(&at_500, "luma"));
        // A missing luma is NaN, which fails both comparisons.
        let differs = (luma_500 - metered_luma).abs() >= DARKER_BY;
        if !differs {
            return Ok(Some(format!(
                "★★★ the picture drawn at 500 lux must differ in brightness from the metered one \
                 by at least {DARKER_BY}: luma {luma_500} against {metered_luma}."
            )));
        }
        let seen = self.drawn()?;
        if !self.type_white(DOUBLED)? {
            return Ok(Some(format!(
                "★★★ with the switch on the viewer drew no `{WHITE}` field."
            )));
        }
        let Some(at_1000) = self.picture_after(seen, fixed(f64::from(DOUBLED)))? else {
            return Ok(Some(self.not_fixed(seen, "with 1000 lux typed")?));
        };
        report.note(format!("fixed at 1000 lux: {}", at_1000.raw));
        let (bright, dark) = (number(&at_500, "luma"), number(&at_1000, "luma"));
        let darker = dark + DARKER_BY <= bright;
        if !darker {
            return Ok(Some(format!(
                "★★★ doubling the lux drawn as full white must draw the model at least {DARKER_BY} \
                 darker: luma {bright} at 500 lux, {dark} at 1000."
            )));
        }
        Ok(None)
    }

    /// Select the white point field's text, type `lux` and press Enter;
    /// `false` when the field is drawn nowhere.
    fn type_white(&self, lux: u32) -> Result<bool> {
        let Some((rect, vp)) = reach(self.session, self.pointer, self.ui, WHITE)? else {
            return Ok(false);
        };
        let vp = vp.as_deref();
        self.pointer
            .click_in(self.session, vp, WindowPoint::centre_of(rect))?;
        self.session.settle(10);
        self.pointer.key(self.session, vp, "A", Some("ctrl"))?;
        self.pointer.type_text(self.session, vp, &lux.to_string())?;
        self.pointer.key(self.session, vp, "Enter", None)?;
        self.session.settle(15);
        close(self.session, self.pointer, self.ui)?;
        Ok(true)
    }

    /// Why no picture drawn in lux arrived after the `seen`th, `when`.
    fn not_fixed(&self, seen: usize, when: &str) -> Result<String> {
        let trace = self.session.trace()?;
        let last = trace
            .events(RENDERED)
            .skip(seen)
            .last()
            .map(|l| l.raw.clone());
        let toggled = trace
            .events("model-view-true-light")
            .last()
            .map(|l| l.raw.clone());
        Ok(format!(
            "★★★ {when}, no picture was drawn at the asked exposure in lux: newest picture {last:?}, \
             newest switch line {toggled:?}. `exposure=refused` names an engine refusal, said under \
             the switch."
        ))
    }
}
