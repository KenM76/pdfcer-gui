//! Press one ribbon or shell region through the scripted pointer, reaching a
//! ribbon item wherever the responsive band has put it.
//!
//! A narrow window folds a group into its collapsed button; the item is then
//! drawn only inside that group's popup. [`press`] opens the popup (through
//! [`declared_or_in_overflow`]) before it judges the item absent, so a check
//! fails only when the build truly lacks the item.

use crate::checks::driving::{declared_in, declared_names, declared_or_in_overflow, list};
use crate::coords::WindowPoint;
use crate::error::Result;
use crate::input::scripted::ScriptedPointer;
use crate::launch::Session;

/// Click `region`, or the failure naming the regions under `family`.
///
/// A `ribbon.item.` region not on the band is searched for first, scrolling
/// the band and opening collapsed groups; the popup that holds it stays open
/// for the click.
pub fn press(
    session: &Session,
    pointer: &ScriptedPointer,
    ui_rect: &str,
    region: &str,
    family: &str,
) -> Result<std::result::Result<(), String>> {
    if region.starts_with("ribbon.item.") {
        declared_or_in_overflow(session, pointer, ui_rect, region)?;
    }
    let trace = session.trace()?;
    let Some((r, viewport)) = declared_in(&trace, ui_rect, region) else {
        return Ok(Err(format!(
            "no `{region}` region. Regions beginning `{family}`: {}. Trace: {}.",
            list(&declared_names(&trace, ui_rect, family)),
            session.trace_path().display()
        )));
    };
    pointer.click_in(session, viewport.as_deref(), WindowPoint::centre_of(r))?;
    session.settle(20);
    Ok(Ok(()))
}
