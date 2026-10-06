//! `ctrl_u_underlines_the_text_itself` — **Ctrl+U marks the characters in the
//! page content, shows Underline pressed from that mark, and a second Ctrl+U
//! takes the line off**
//!
//! Design and rationale: `docs/modules/ui-verify/checks/tied_underline.md`.

use crate::checks::driving::shell_trace;
use crate::checks::word_styles::{Driven, launch_on};
use crate::checks::{Check, CheckContext};
use crate::error::{Error, Result};
use crate::report::CheckReport;

const FIXTURE: &str = "repeated-word.pdf";
/// Inside the second `M10`, which starts at x = 108.
const IN_SECOND: (f64, f64) = (118.0, 703.0);
const FORMAT_TAB: &str = "ribbon.tab.format"; // ui-text-exempt: a trace region name, never displayed
const DECORATED: &str = "text-decorate-applied"; // ui-text-exempt: a trace event name, never displayed
const SELECTED: &str = "ribbon-item-selected"; // ui-text-exempt: a trace event name, never displayed
const SAVED: &str = "save-in-place"; // ui-text-exempt: a trace event name, never displayed
const UNDERLINE: &str = "format.underline";
/// The second copy's show operator inside the engine's decoration marker. A
/// line drawn beside the text as separate content or as an annotation carries
/// no marker round the characters.
const TIED: &str = "/Line /Underline /Id 1>> BDC (M10) Tj EMC";
/// The rule the engine derives from that marker.
const RULE: &str = "/pdfc_Deco <</Rule 1>> BDC";

/// See the module documentation.
pub struct CtrlUUnderlinesTheTextItself;

impl Check for CtrlUUnderlinesTheTextItself {
    fn name(&self) -> &'static str {
        "ctrl_u_underlines_the_text_itself"
    }

    fn defect(&self) -> &'static str {
        "Ctrl+U drew an underline that is not tied to the text (a line beside it that stays put \
         when the text moves), or Underline did not read back as pressed, or a second \
         Ctrl+U did not take it off"
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

/// The ribbon's Underline as last drawn after shell-trace line `after`.
fn pressed(session: &crate::launch::Session, after: usize) -> Result<Option<bool>> {
    Ok(shell_trace(session)?
        .events(SELECTED)
        .filter(|l| l.lineno > after && l.get("id") == Some(UNDERLINE))
        .last()
        .map(|l| l.get("selected") == Some("1")))
}

/// Press Ctrl+U; the `on=` of the decoration it applied, and Underline's
/// drawn state afterwards.
fn press(d: &Driven) -> Result<(Option<String>, Option<bool>)> {
    let mark = d.session.trace()?.mark();
    let shell_mark = shell_trace(&d.session)?.mark();
    d.pointer.key(&d.session, None, "U", Some("ctrl"))?;
    d.session.settle(40);
    let applied = d
        .session
        .trace()?
        .last_after(DECORATED, mark)
        .filter(|l| l.get("applied") == Some("1"))
        .and_then(|l| l.get("on").map(str::to_owned));
    Ok((applied, pressed(&d.session, shell_mark)?))
}

fn drive(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    let (d, doc) = launch_on(ctx, report, "tied-underline", FIXTURE)?;
    let source_len = std::fs::metadata(&doc).map_or(0, |m| m.len());
    if let Err(why) = d.caret_at(IN_SECOND)? {
        return Ok(Some(why));
    }
    d.click(FORMAT_TAB)?;
    d.session.settle(20);
    let (on, after_on) = press(&d)?;
    d.pointer.key(&d.session, None, "S", Some("ctrl"))?;
    d.session.settle(40);
    let saved = d
        .session
        .trace()?
        .events(SAVED)
        .last()
        .is_some_and(|l| l.raw.contains("outcome=ok"));
    let (off, after_off) = press(&d)?;
    d.pointer.gone(&d.session)?;
    let bytes =
        std::fs::read(&doc).map_err(|e| Error::new(format!("reading the saved copy: {e}")))?;
    let tail = &bytes[usize::try_from(source_len).unwrap_or(0).min(bytes.len())..];
    let has = |needle: &str| tail.windows(needle.len()).any(|w| w == needle.as_bytes());
    let (tied, rule) = (has(TIED), has(RULE));
    report.note(format!(
        "first Ctrl+U on={on:?}, Underline drawn {after_on:?}; saved {saved}, `{TIED}` {tied}, \
         `{RULE}` {rule}; second Ctrl+U on={off:?}, Underline drawn {after_off:?}"
    ));
    let mut findings = Vec::new();
    if on.as_deref() != Some("true") || after_on != Some(true) {
        findings.push(format!(
            "the first Ctrl+U applied on={on:?} and left Underline drawn {after_on:?}; on=true \
             and pressed were owed."
        ));
    }
    if !saved || !tied || !rule {
        findings.push(format!(
            "the saved revision (saved {saved}) carries `{TIED}` {tied} and `{RULE}` {rule}; \
             both were owed."
        ));
    }
    if off.as_deref() != Some("false") || after_off != Some(false) {
        findings.push(format!(
            "the second Ctrl+U applied on={off:?} and left Underline drawn {after_off:?}; \
             on=false and released were owed."
        ));
    }
    Ok((!findings.is_empty()).then(|| format!("{} Trace: {}.", findings.join(" "), d.path())))
}
