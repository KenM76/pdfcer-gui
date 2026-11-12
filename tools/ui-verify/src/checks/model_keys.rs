//! `the_3d_viewers_controls_tab_sets_layouts_swaps_sides_and_takes_a_key` —
//! the options menu's Controls tab: a ready layout's card, the swap of the
//! turning and strafing sides, and a key chip that takes the next key.
//!
//! Places the engine corpus's `assembly.prc` and opens it with *View…*. On
//! the Controls tab, presses the Shooter card, the swap, the Default card,
//! then the first Walk forward chip followed by the U key. Oracles, each a
//! new `model-nav-prefs` line: `bind=shooter`; `swap=true`; `bind=default`
//! (a card leaves the swap as it is, and a swapped Default still matches
//! Default); `bind=custom` once U replaces W. Design: `DESIGNS.md` § O411 /
//! O412.

use crate::checks::model_inferences::open_viewer;
use crate::checks::model_view_window::{launch_with_model_file, ui_rect};
use crate::checks::viewer_options::{close, reach};
use crate::checks::{Check, CheckContext};
use crate::coords::WindowPoint;
use crate::error::Result;
use crate::input::scripted::ScriptedPointer;
use crate::launch::Session;
use crate::report::CheckReport;

const MODEL: &str = "D:/Dev/pdfcer/fixtures/synthetic/prc/assembly.prc";
/// `dialogs::model3d::navkeys`'s regions.
const SHOOTER: &str = "model3d.controls.preset.shooter";
const DEFAULT: &str = "model3d.controls.preset.default";
const SWAP: &str = "model3d.controls.swap";
const CHIP: &str = "model3d.controls.key.forward_0";
const PREFS: &str = "model-nav-prefs"; // ui-text-exempt: a trace event name, never displayed

/// See the module documentation.
pub struct TheControlsTabWorks;

impl Check for TheControlsTabWorks {
    fn name(&self) -> &'static str {
        "the_3d_viewers_controls_tab_sets_layouts_swaps_sides_and_takes_a_key"
    }

    fn defect(&self) -> &'static str {
        "the 3D viewer's Controls tab is missing, a ready layout's card does not set its \
         keys, the swap of the turning and strafing sides does nothing, or a key chip does \
         not take the key pressed next"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        let driven = launch_with_model_file(ctx, &mut report, "model-keys", MODEL)
            .and_then(|(session, pointer)| drive(ctx, &mut report, &session, &pointer));
        match driven {
            Ok(Some(failure)) => report.fail(failure),
            Ok(None) => report.pass(),
            Err(why) => report.from_error(&why),
        }
    }
}

/// Press `name` on the Controls tab, then `key` when given; whether the
/// newest `model-nav-prefs` line after it has `field=want`.
fn step(
    session: &Session,
    pointer: &ScriptedPointer,
    report: &mut CheckReport,
    (ui, name, key): (&str, &str, Option<&str>),
    (field, want): (&str, &str),
) -> Result<Option<String>> {
    let Some((rect, vp)) = reach(session, pointer, ui, name)? else {
        return Ok(Some(format!("no `{name}` on the options menu's tabs.")));
    };
    let mark = session.trace()?.mark();
    pointer.click_in(session, vp.as_deref(), WindowPoint::centre_of(rect))?;
    session.settle(20);
    if let Some(key) = key {
        pointer.key(session, vp.as_deref(), key, None)?;
        session.settle(20);
    }
    let trace = session.trace()?;
    let line = trace.last_after(PREFS, mark);
    report.note(format!(
        "{name}: `{}`",
        line.map_or("nothing", |l| l.raw.as_str())
    ));
    if line.and_then(|l| l.get(field)) == Some(want) {
        return Ok(None);
    }
    Ok(Some(format!(
        "pressing `{name}` must trace `{PREFS} … {field}={want}`; it traced `{}`.",
        line.map_or("nothing", |l| l.raw.as_str())
    )))
}

fn drive(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
) -> Result<Option<String>> {
    if let Some(failure) = open_viewer(ctx, session, pointer)? {
        return Ok(Some(failure));
    }
    let ui = ui_rect(ctx)?;
    let steps = [
        ((ui, SHOOTER, None), ("bind", "shooter")),
        ((ui, SWAP, None), ("swap", "true")),
        ((ui, DEFAULT, None), ("bind", "default")),
        ((ui, CHIP, Some("U")), ("bind", "custom")),
    ];
    for (press, want) in steps {
        if let Some(failure) = step(session, pointer, report, press, want)? {
            return Ok(Some(failure));
        }
    }
    close(session, pointer, ui)?;
    Ok(None)
}
