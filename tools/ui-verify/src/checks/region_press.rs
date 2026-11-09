//! A click on a declared `ui-rect` region, in whichever window declared it.

use crate::checks::driving::{declared_in, declared_names, list};
use crate::coords::WindowPoint;
use crate::error::{Error, Result};
use crate::input::scripted::ScriptedPointer;
use crate::launch::Session;

const UI_RECT: &str = "ui-rect";

/// Click the centre of region `name` and settle. When it is not declared,
/// the error lists what is declared under `family` instead.
pub fn press_region(
    session: &Session,
    pointer: &ScriptedPointer,
    name: &str,
    family: &str,
) -> Result<()> {
    let trace = session.trace()?;
    let (rect, viewport) = declared_in(&trace, UI_RECT, name).ok_or_else(|| {
        Error::new(format!(
            "no `{name}` region. Declared under `{family}`: {}.",
            list(&declared_names(&trace, UI_RECT, family))
        ))
    })?;
    pointer.click_in(session, viewport.as_deref(), WindowPoint::centre_of(rect))?;
    session.settle(30);
    Ok(())
}
