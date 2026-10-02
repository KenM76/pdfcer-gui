//! `checks::paste_chord` — **Ctrl+V pastes another program's picture when
//! the clipboard holds no text**
//!
//! The toolkit raises a paste for `Ctrl+V` only when the clipboard holds
//! text, so this is the one check that must press the real chord rather than
//! the scripted pointer's paste. It posts the key messages to the launched
//! window alone, which leaves the operator's keyboard untouched, with a
//! picture and no text on the clipboard (snapshotted and restored as in
//! [`super::os_image_paste`]).
//!
//! Oracles: after `Ctrl+V` a `chord-command ... via=paste-hook` line and a
//! `clip-pasted source=os` rectangle centred on the pointer; after a plain
//! `V`, no new `clip-pasted`.

use super::os_image_paste::{self as osp, ClipGuard};
use crate::checks::{Check, CheckContext};
use crate::error::{Error, Result};
use crate::input::scripted::ScriptedPointer;
use crate::launch::Session;
use crate::report::CheckReport;
use crate::sys;

const STEM: &str = "paste-chord";
const HOOKED: &str = "via=paste-hook";
const VK_CONTROL: u16 = 0x11;
const VK_V: u16 = 0x56;

/// See the module documentation.
pub struct CtrlVPastesAPicture;

impl Check for CtrlVPastesAPicture {
    fn name(&self) -> &'static str {
        "ctrl_v_pastes_a_picture_when_the_clipboard_holds_no_text"
    }

    fn defect(&self) -> &'static str {
        "Ctrl+V does nothing when another program copied a picture and no text, because the \
         toolkit drops the chord"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        let mut guard = ClipGuard::take();
        let driven = osp::launch(ctx, &mut report, STEM)
            .and_then(|(session, pointer)| drive(ctx, &mut report, &session, &pointer, &mut guard));
        report.note(guard.release());
        match driven {
            Ok(Some(failure)) => report.fail(failure),
            Ok(None) => report.pass(),
            Err(why) => report.from_error(&why),
        }
    }
}

fn hooked_chords(session: &Session) -> Result<usize> {
    Ok(session
        .trace()?
        .events("chord-command")
        .filter(|l| l.get("via") == Some("paste-hook"))
        .count())
}

fn drive(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
    guard: &mut ClipGuard,
) -> Result<Option<String>> {
    let window = session
        .window()
        .ok_or_else(|| Error::new("the launched app has no window to post keys to."))?;
    guard.set(&[(sys::CF_DIB, osp::dib(64, 32))])?;
    pointer.hover(session, osp::at(ctx, session, osp::FIRST)?)?;
    let (pasted, chords) = (osp::count(session, osp::PASTED)?, hooked_chords(session)?);
    if !sys::post_chord(window, &[], VK_V) {
        return Err(Error::new("the app's window refused a posted key."));
    }
    session.settle(20);
    if osp::count(session, osp::PASTED)? != pasted {
        return Ok(Some("a plain V pasted the clipboard.".to_owned()));
    }
    sys::post_chord(window, &[VK_CONTROL], VK_V);
    session.settle(20);
    let failure = if hooked_chords(session)? == chords {
        Some(format!(
            "Ctrl+V with a picture and no text traced no `chord-command {HOOKED}` line."
        ))
    } else {
        match session.trace()?.events(osp::PASTED).nth(pasted) {
            None => Some(format!(
                "Ctrl+V reached the app but traced no `{}` line.",
                osp::PASTED
            )),
            Some(line) => {
                let r = osp::rect(line).ok_or_else(|| {
                    Error::new(format!("a `{}` line without its rectangle.", osp::PASTED))
                })?;
                report.note(format!("Ctrl+V paste {}", osp::show(r)));
                osp::lands(r, osp::FIRST, (48.0, 24.0))
            }
        }
    };
    let parked = pointer.gone(session);
    match failure {
        Some(failure) => Ok(Some(failure)),
        None => parked.map(|_| None),
    }
}
