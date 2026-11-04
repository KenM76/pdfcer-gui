//! `the_3d_viewer_remembers_its_settings` — the 3D viewer opens with the
//! settings the preferences file holds (O387) and writes a setting changed
//! back to it, keeping the others.
//!
//! Planted: stairs on, speed 2, the Look tab. The viewer must trace those
//! values as recalled; then the Picture tab is chosen and the menu closed,
//! and the file must hold `model3d_tab = 3` with the planted stairs and
//! speed still there.

use crate::checks::model_axes::{launch, opened_from_page};
use crate::checks::model_eye::value_of;
use crate::checks::{Check, CheckContext};
use crate::coords::WindowPoint;
use crate::error::{Error, Result};
use crate::report::CheckReport;

/// `dialogs::model3d::options::REGION_TAB` for the Picture tab.
const PICTURE_TAB: &str = "model3d.options.tab.picture";
const PLANTED: &str = "model3d_stairs = 1\nmodel3d_speed = 2.000\nmodel3d_tab = 1\n";

/// See the module documentation.
pub struct TheViewerRemembersItsSettings;

impl Check for TheViewerRemembersItsSettings {
    fn name(&self) -> &'static str {
        "the_3d_viewer_remembers_its_settings"
    }

    fn defect(&self) -> &'static str {
        "The 3D viewer opens with its defaults whatever the preferences file holds, or a \
         setting changed in it is not written back, or writing one loses the others"
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

fn drive(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    let ui_rect = ctx
        .profile
        .vocab
        .ui_rect_event
        .ok_or_else(|| Error::new("the profile declares no ui-rect trace event."))?;
    let exe = ctx
        .resolve_exe()
        .ok_or_else(|| Error::new("no binary to drive."))?;
    let userdata = exe
        .parent()
        .ok_or_else(|| Error::new("the binary has no parent directory"))?
        .join("userdata");
    crate::sandbox::write_prefs(&userdata, PLANTED)
        .map_err(|e| Error::new(format!("could not plant the preferences file: {e}")))?;
    let (session, pointer) = launch(ctx, "model-memory")?;
    report.artifact(session.trace_path().to_path_buf());
    report.artifact(pointer.path().to_path_buf());
    session.settle(45);
    let outcome = (|| -> Result<Option<String>> {
        if let Some(failure) = opened_from_page(ctx, report, &session, &pointer, ui_rect)? {
            return Ok(Some(failure));
        }
        let trace = session.trace()?;
        let recalled = trace
            .events("model-view-settings-recalled")
            .last()
            .map(|l| ["stairs", "speed", "tab"].map(|k| l.get(k).map(str::to_owned)));
        report.note(format!("recalled stairs, speed, tab: {recalled:?}"));
        let want = ["true", "2.00", "1"].map(|v| Some(v.to_owned()));
        if recalled != Some(want) {
            return Ok(Some(format!(
                "the viewer recalled {recalled:?} from a file planting stairs on, speed 2 \
                 and the Look tab."
            )));
        }
        let Some((tab, vp)) =
            crate::checks::viewer_options::reach(&session, &pointer, ui_rect, PICTURE_TAB)?
        else {
            return Ok(Some(format!(
                "no `{PICTURE_TAB}` region in the options menu."
            )));
        };
        pointer.click_in(&session, vp.as_deref(), WindowPoint::centre_of(tab))?;
        session.settle(20);
        crate::checks::viewer_options::close(&session, &pointer, ui_rect)?;
        // The viewer writes a second after the last write, once no pointer is held.
        session.settle(90);
        let persisted = session
            .trace()?
            .events("model-view-settings-persisted")
            .last()
            .and_then(|l| l.get("keys").map(str::to_owned));
        let text = std::fs::read_to_string(userdata.join("preferences.txt")).unwrap_or_default();
        let kept = ["model3d_tab", "model3d_stairs", "model3d_speed"].map(|k| value_of(&text, k));
        report.note(format!(
            "persisted keys {persisted:?}; file tab, stairs, speed {kept:?}"
        ));
        let want = ["3", "1", "2.000"].map(|v| Some(v.to_owned()));
        if kept != want {
            return Ok(Some(format!(
                "after choosing the Picture tab the file holds tab, stairs, speed {kept:?}; \
                 wanted {want:?} (persisted keys {persisted:?})."
            )));
        }
        Ok(None)
    })();
    pointer.gone(&session)?;
    outcome
}
