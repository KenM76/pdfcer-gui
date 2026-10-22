//! `a_widgets_background_is_stored_in_cmyk` — a form box's background, set in
//! Properties on the picker's CMYK tab, is written into the widget's `/MK /BG`
//! as four components. Scripted pointer, window off the desktop.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/widget_bg_cmyk.md`.

use crate::checks::driving::declared_in;
use crate::checks::properties_pane;
use crate::checks::{Check, CheckContext};
use crate::error::{Error, Result};
use crate::input::scripted::ScriptedPointer;
use crate::launch::Session;
use crate::report::CheckReport;

const FIXTURE: &str = "three-text-fields.pdf";
const METHOD: &str = "Rebuild it with `python fixtures/three-text-fields.PROVENANCE.py`.";
const FIELD: &str = "FieldOne";
const LABEL: &str = "widget-bg-cmyk";
const SWATCH: &str = "properties.widget_edit.background"; // ui-text-exempt: trace region name
const CMYK_TAB: &str = "properties.widget_edit.background.space.cmyk"; // ui-text-exempt: trace region name
const SAVED: &str = "save-in-place"; // ui-text-exempt: a trace event name

/// See the module documentation.
pub struct AWidgetsBackgroundIsStoredInCmyk;

impl Check for AWidgetsBackgroundIsStoredInCmyk {
    fn name(&self) -> &'static str {
        "a_widgets_background_is_stored_in_cmyk"
    }

    fn defect(&self) -> &'static str {
        "a form box's background offers no CMYK, or a colour chosen on its CMYK tab is not \
         written to the widget's /MK /BG as CMYK"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        let driven =
            properties_pane::launch_on_field(ctx, &mut report, (FIXTURE, METHOD), FIELD, LABEL)
                .and_then(|(session, pointer)| {
                    let outcome = steps(ctx, &mut report, &session, &pointer);
                    let parked = pointer.gone(&session);
                    match outcome? {
                        Some(failure) => Ok(Some(failure)),
                        None => parked.map(|_| None),
                    }
                });
        match driven {
            Ok(None) => report.pass(),
            Ok(Some(failure)) => report.fail(failure),
            Err(why) => report.from_error(&why),
        }
    }
}

/// The component count of every `/BG` array in a file, as written.
fn background_arity(bytes: &[u8]) -> Vec<usize> {
    let text = String::from_utf8_lossy(bytes);
    text.match_indices("/BG")
        .filter_map(|(i, _)| {
            let rest = text[i + 3..].trim_start().strip_prefix('[')?;
            Some(rest.split(']').next()?.split_whitespace().count())
        })
        .collect()
}

fn steps(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
) -> Result<Option<String>> {
    let copy = ctx.out(&format!("{LABEL}.pdf"));
    let read = |p: &std::path::Path| {
        std::fs::read(p).map_err(|e| Error::new(format!("reading {}: {e}", p.display())))
    };
    // The fixture's widgets carry no `/MK`, so a four-component `/BG` can only
    // be the edit's.
    if background_arity(&read(&copy)?).contains(&4) {
        return Ok(Some(format!(
            "the fixture already holds a CMYK `/BG`, so this check cannot tell the edit's. {METHOD}"
        )));
    }
    properties_pane::press(ctx, session, pointer, SWATCH, 20)?;
    let ui_rect = ctx
        .profile
        .vocab
        .ui_rect_event
        .ok_or_else(|| Error::new("the profile declares no ui-rect trace event."))?;
    let Some((_, vp)) = declared_in(&session.trace()?, ui_rect, CMYK_TAB) else {
        return Ok(Some(format!(
            "★★★ the background picker opened with no CMYK tab (`{CMYK_TAB}`). Trace: {}.",
            session.trace_path().display()
        )));
    };
    properties_pane::press(ctx, session, pointer, CMYK_TAB, 20)?;
    // The close is the commit.
    pointer.key(session, vp.as_deref(), "Escape", None)?;
    session.settle(30);
    pointer.key(session, None, "S", Some("ctrl"))?;
    session.settle(40);
    if !session
        .trace()?
        .events(SAVED)
        .any(|l| l.get("outcome") == Some("ok"))
    {
        return Ok(Some(format!(
            "Ctrl+S traced no `{SAVED} outcome=ok`. Trace: {}.",
            session.trace_path().display()
        )));
    }
    let arities = background_arity(&read(&copy)?);
    report.note(format!(
        "/BG component counts in the saved file: {arities:?}"
    ));
    Ok((!arities.contains(&4)).then(|| {
        format!(
            "★★★ after the CMYK tab was chosen and the picker closed, the saved file holds no \
             four-component `/BG` (component counts {arities:?}). Trace: {}.",
            session.trace_path().display()
        )
    }))
}

#[cfg(test)]
mod tests {
    use super::background_arity;

    #[test]
    fn background_arity_counts_each_bg_arrays_components() {
        let got = background_arity(
            b"<< /MK << /BG [1] /BC [0 0 0] >> >> << /MK << /BG [0.6 0.6 0.6 1] >> >>",
        );
        assert_eq!(got, vec![1, 4]);
    }
}
