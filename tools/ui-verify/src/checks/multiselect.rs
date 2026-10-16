//! Steps shared by the multi-select checks: Shift-click, marquee, save a copy,
//! and read the saved top edges.

use super::dimdrive::{Fixture, launch_with, press_on_tab};
use super::savedannots::listed_annots;
use crate::checks::{Check, CheckContext};
use crate::coords::{CanvasMapping, DocPoint, WindowPoint};
use crate::error::{Error, Result};
use crate::input::scripted::ScriptedPointer;
use crate::launch::Session;
use crate::report::CheckReport;
use crate::trace::TraceLine;

const FILE_TAB: &str = "ribbon.tab.file"; // ui-text-exempt: a trace region name
const SAVE_COPY: &str = "ribbon.item.file.save_copy"; // ui-text-exempt: a trace region name
const SAVED: &str = "save-copy"; // ui-text-exempt: a trace event name
const SAVE_PATH_ENV: &str = "PDFCER_DIAG_SAVE_PATH"; // ui-text-exempt: an environment variable name

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
    corners: ((f64, f64), (f64, f64)),
) -> Result<()> {
    band_with(ctx, session, pointer, fixture, corners, None)
}

/// [`band`] with `mods` (`shift` adds, `ctrl` subtracts) held through it.
pub(super) fn band_with(
    ctx: &CheckContext,
    session: &Session,
    pointer: &ScriptedPointer,
    fixture: &Fixture,
    (from, to): ((f64, f64), (f64, f64)),
    mods: Option<&str>,
) -> Result<()> {
    let (from, to) = (
        window_at(ctx, session, fixture, from)?,
        window_at(ctx, session, fixture, to)?,
    );
    match mods {
        None => {
            pointer.drag(session, from, to, 8)?;
        }
        Some(m) => {
            pointer.send(
                session,
                &format!(
                    "drag {:.1} {:.1} {:.1} {:.1} steps=8 mods={m}",
                    from.x(),
                    from.y(),
                    to.x(),
                    to.y()
                ),
            )?;
        }
    }
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

/// The two saved copies a move-then-Undo check judges: after the move, and
/// after one Undo.
pub(super) struct Saves {
    pub moved: std::path::PathBuf,
    pub undone: std::path::PathBuf,
}

/// A body driven between launch and park, writing [`Saves`] by Save a copy.
pub(super) type SavingBody = fn(
    &CheckContext,
    &mut CheckReport,
    &Session,
    &ScriptedPointer,
    &Saves,
) -> Result<Option<String>>;

/// Launch `fixture` off-screen in `invoke` with `PDFCER_DIAG_SAVE_PATH`
/// pointing its first two saves at `<stem>.moved.pdf` and `<stem>.undone.pdf`,
/// drive `body`, park the pointer, and report.
pub(super) fn run_saving(
    check: &dyn Check,
    ctx: &CheckContext,
    fixture: &Fixture,
    invoke: &str,
    stem: &str,
    body: SavingBody,
) -> CheckReport {
    let mut report = CheckReport::new(check.name(), check.defect());
    let saves = Saves {
        moved: ctx.out(&format!("{stem}.moved.pdf")),
        undone: ctx.out(&format!("{stem}.undone.pdf")),
    };
    for p in [&saves.moved, &saves.undone] {
        let _ = std::fs::remove_file(p);
    }
    let env = [(
        SAVE_PATH_ENV,
        format!("{};{}", saves.moved.display(), saves.undone.display()),
    )];
    let outcome = launch_with(ctx, &mut report, fixture, invoke, stem, &env).and_then(
        |(session, pointer)| {
            let outcome = body(ctx, &mut report, &session, &pointer, &saves);
            let parked = pointer.gone(&session);
            match outcome? {
                Some(failure) => Ok(Some(failure)),
                None => parked.map(|_| None),
            }
        },
    );
    match outcome {
        Ok(Some(failure)) => report.fail(failure),
        Ok(None) => report.pass(),
        Err(why) => report.from_error(&why),
    }
}

/// Save a copy, then compare its saved top edges with `want`; the failure
/// sentence names `when`.
pub(super) fn saved_tops(
    ctx: &CheckContext,
    session: &Session,
    pointer: &ScriptedPointer,
    path: &std::path::Path,
    want: &[(&str, f64)],
    when: &str,
) -> Result<Option<String>> {
    if let Some(why) = save(ctx, session, pointer)? {
        return Ok(Some(why));
    }
    Ok(tops(path, want)?.map(|why| format!("{when}, {why}")))
}
