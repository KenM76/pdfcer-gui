//! `checks::scripteddrive` — helpers shared by checks that press through the
//! scripted pointer and seed the sandbox's preferences.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/scripteddrive.md`.

use crate::checks::CheckContext;
use crate::checks::driving::{declared_in, declared_names, list};
use crate::coords::WindowPoint;
use crate::error::{Error, Result};
use crate::input::scripted::ScriptedPointer;
use crate::launch::Session;

/// The trace event every region is declared under.
const UI_RECT: &str = "ui-rect";

/// Click the centre of region `name` in whichever viewport declared it, then
/// let the frame settle. Fails naming the regions declared beside it.
pub fn click(session: &Session, pointer: &ScriptedPointer, name: &str) -> Result<()> {
    let trace = session.trace()?;
    let (rect, viewport) = declared_in(&trace, UI_RECT, name).ok_or_else(|| {
        let prefix = name.rsplit_once('.').map_or(name, |(head, _)| head);
        Error::new(format!(
            "no `{name}` region. Declared under `{prefix}`: {}.",
            list(&declared_names(&trace, UI_RECT, prefix))
        ))
    })?;
    pointer.click_in(session, viewport.as_deref(), WindowPoint::centre_of(rect))?;
    session.settle(15);
    Ok(())
}

/// The `userdata` folder beside the binary under test, where its
/// `preferences.txt` lives.
pub fn userdata(ctx: &CheckContext) -> Result<std::path::PathBuf> {
    let exe = ctx
        .resolve_exe()
        .ok_or_else(|| Error::new("no binary to drive. Pass --exe."))?;
    Ok(exe
        .parent()
        .ok_or_else(|| Error::new("the binary has no parent directory"))?
        .join("userdata"))
}
