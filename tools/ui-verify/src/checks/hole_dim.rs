//! The radius/diameter fixture several ce dimension checks share: the r=30
//! hole in `hole-in-a-big-object.pdf`, and the steps that measure it and place
//! the label where a click selects it.

use super::dimdrive::{Fixture, click_on};
use crate::checks::CheckContext;
use crate::coords::{CanvasMapping, DocPoint, PageGeometry};
use crate::error::Result;
use crate::input::scripted::ScriptedPointer;
use crate::launch::Session;

pub(super) const HOLE: Fixture = Fixture {
    file: "hole-in-a-big-object.pdf",
    method: "Rebuild it with `python tools/gen-hole-in-a-big-object-fixture.py`.",
    page: PageGeometry {
        width_pt: 612.0,
        height_pt: 792.0,
    },
};
/// North, east and west of the fixture's r=30 hole at (306, 500).
pub(super) const RIM: [(f64, f64); 3] = [(306.0, 530.0), (336.0, 500.0), (276.0, 500.0)];
/// Clear of the hole and of every segment, for the placing click; the label
/// is drawn here, so a click here selects the ce dimension.
pub(super) const PLACE: (f64, f64) = (380.0, 560.0);

/// Measure the hole and place the label.
pub(super) fn measure(
    ctx: &CheckContext,
    session: &Session,
    pointer: &ScriptedPointer,
) -> Result<()> {
    for at in &RIM[..2] {
        click_on(ctx, session, pointer, &HOLE, 0, *at)?;
    }
    let mapping = CanvasMapping::from_trace(&session.trace()?, &ctx.profile.vocab, HOLE.page, 0)?;
    let last = mapping.doc_to_window(DocPoint::new(0, RIM[2].0, RIM[2].1))?;
    pointer.hover(session, last)?;
    session.settle(10);
    pointer.double_click(session, last)?;
    session.settle(25);
    click_on(ctx, session, pointer, &HOLE, 0, PLACE)?;
    Ok(())
}
