//! Steps shared by the multi-select checks: Shift-click, marquee, save a copy,
//! and read the saved top edges.

use super::dimdrive::{Fixture, press_on_tab};
use super::savedannots::listed_annots;
use crate::checks::CheckContext;
use crate::coords::{CanvasMapping, DocPoint, WindowPoint};
use crate::error::{Error, Result};
use crate::input::scripted::ScriptedPointer;
use crate::launch::Session;
use crate::trace::TraceLine;

const FILE_TAB: &str = "ribbon.tab.file"; // ui-text-exempt: a trace region name
const SAVE_COPY: &str = "ribbon.item.file.save_copy"; // ui-text-exempt: a trace region name
const SAVED: &str = "save-copy"; // ui-text-exempt: a trace event name

/// The last line named `event` after `mark`, or the failure sentence.
pub(super) fn after(
    session: &Session,
    event: &str,
    mark: usize,
    what: &str,
) -> Result<std::result::Result<TraceLine, String>> {
    let trace = session.trace()?;
    Ok(trace.last_after(event, mark).cloned().ok_or_else(|| {
        format!(
            "{what} and no `{event}` followed. Trace: {}.",
            session.trace_path().display()
        )
    }))
}

/// Unwrap a step's line, or return its failure sentence from the check.
macro_rules! step {
    ($e:expr) => {
        match $e? {
            Ok(line) => line,
            Err(why) => return Ok(Some(why)),
        }
    };
}
pub(crate) use step;

/// A page-0 document point of `fixture` in window coordinates.
pub(super) fn window_at(
    ctx: &CheckContext,
    session: &Session,
    fixture: &Fixture,
    at: (f64, f64),
) -> Result<WindowPoint> {
    let mapping =
        CanvasMapping::from_trace(&session.trace()?, &ctx.profile.vocab, fixture.page, 0)?;
    mapping.doc_to_window(DocPoint::new(0, at.0, at.1))
}

/// Shift-click a page-0 document point.
pub(super) fn shift_click(
    ctx: &CheckContext,
    session: &Session,
    pointer: &ScriptedPointer,
    fixture: &Fixture,
    at: (f64, f64),
) -> Result<()> {
    let p = window_at(ctx, session, fixture, at)?;
    pointer.hover(session, p)?;
    session.settle(10);
    pointer.send(
        session,
        &format!("click {:.1} {:.1} mods=shift", p.x(), p.y()),
    )?;
    session.settle(20);
    Ok(())
}

/// Drag a marquee between two page-0 document points. Left to right is a
/// window, right to left a crossing band.
pub(super) fn band(
    ctx: &CheckContext,
    session: &Session,
    pointer: &ScriptedPointer,
    fixture: &Fixture,
    (from, to): ((f64, f64), (f64, f64)),
) -> Result<()> {
    let (from, to) = (
        window_at(ctx, session, fixture, from)?,
        window_at(ctx, session, fixture, to)?,
    );
    pointer.drag(session, from, to, 8)?;
    session.settle(30);
    Ok(())
}

/// Save a copy to the next path of the save queue; the failure sentence when
/// no save followed.
pub(super) fn save(
    ctx: &CheckContext,
    session: &Session,
    pointer: &ScriptedPointer,
) -> Result<Option<String>> {
    let mark = session.trace()?.mark();
    press_on_tab(ctx, session, pointer, SAVE_COPY, FILE_TAB)?;
    session.settle(40);
    Ok(after(session, SAVED, mark, "Save a copy was pressed")?.err())
}

/// The failure sentence when an annotation's saved top edge is not as
/// wanted. Each annotation is named by a substring of its dictionary, such as
/// `/T (TextOne)` or `/NM (SQ)`, and read from its last body.
pub(super) fn tops(path: &std::path::Path, want: &[(&str, f64)]) -> Result<Option<String>> {
    let bytes = std::fs::read(path)
        .map_err(|e| Error::new(format!("cannot read {}: {e}", path.display())))?;
    let annots = listed_annots(&bytes);
    for (key, top) in want {
        let Some(a) = annots.iter().find(|a| a.body.contains(key)) else {
            return Ok(Some(format!(
                "{} lists no annotation with `{key}`.",
                path.display()
            )));
        };
        if (a.rect[3] - top).abs() > 0.5 {
            return Ok(Some(format!(
                "`{key}`'s saved /Rect is {:?}; its top should be {top}.",
                a.rect
            )));
        }
    }
    Ok(None)
}
