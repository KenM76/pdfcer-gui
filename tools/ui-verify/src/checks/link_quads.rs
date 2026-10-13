//! `a_link_is_moved_resized_copied_and_deleted` — a selected link moves,
//! resizes, copies and deletes, and its `/QuadPoints` (the clickable area a
//! viewer honours) follow its box through each, judged in the saved file.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/link_quads.md`.

use super::dimdrive::{Fixture, click_on, launch_with, press_on_tab};
use crate::checks::{Check, CheckContext};
use crate::coords::{CanvasMapping, DocPoint, PageGeometry};
use crate::error::{Error, Result};
use crate::input::scripted::ScriptedPointer;
use crate::launch::Session;
use crate::report::CheckReport;

const FIXTURE: Fixture = Fixture {
    file: "link-quads.pdf",
    method: "Rebuild it with `python fixtures/link-quads.PROVENANCE.py`.",
    page: PageGeometry {
        width_pt: 400.0,
        height_pt: 400.0,
    },
};
const INVOKE: &str = "mode.edit";
const SAVE_PATH_ENV: &str = "PDFCER_DIAG_SAVE_PATH"; // ui-text-exempt: an environment variable name
const CLIPBOARD_ENV: &str = "PDFCER_DIAG_CLIPBOARD_DIR"; // ui-text-exempt: an environment variable name

const EDIT_TAB: &str = "ribbon.tab.edit"; // ui-text-exempt: a trace region name
const COPY: &str = "ribbon.item.edit.copy"; // ui-text-exempt: a trace region name
const PASTE: &str = "ribbon.item.edit.paste"; // ui-text-exempt: a trace region name
const FILE_TAB: &str = "ribbon.tab.file"; // ui-text-exempt: a trace region name
const SAVE_COPY: &str = "ribbon.item.file.save_copy"; // ui-text-exempt: a trace region name

const SELECTED: &str = "annot-select"; // ui-text-exempt: a trace event name
const MOVED: &str = "move-annotation-applied"; // ui-text-exempt: a trace event name
const RESIZED: &str = "resize-annotation-applied"; // ui-text-exempt: a trace event name
const COPIED: &str = "clipboard-copy"; // ui-text-exempt: a trace event name
const PASTED: &str = "paste-objects-applied"; // ui-text-exempt: a trace event name
const DELETED: &str = "delete-annotation"; // ui-text-exempt: a trace event name
const SAVED: &str = "save-copy"; // ui-text-exempt: a trace event name

/// Inside the fixture's link, page points.
const ON_LINK: (f64, f64) = (200.0, 200.0);
/// Where the body drag lands: the link moves down 80 pt to `[100 100 300 140]`.
const MOVE_TO: (f64, f64) = (200.0, 120.0);
/// The moved link's top-right grip, and where it is dragged.
const GRIP: (f64, f64) = (300.0, 140.0);
const GRIP_TO: (f64, f64) = (340.0, 160.0);
/// Inside the moved and resized link, clear of the quad's edge.
const ON_MOVED: (f64, f64) = (150.0, 130.0);

/// The fixture's quad as fractions of its `/Rect`: inset 20 of 200 pt across
/// and 5 of 40 pt up and down, in `/QuadPoints` order.
const QUAD_IN_RECT: [(f64, f64); 4] = [(0.1, 0.875), (0.9, 0.875), (0.1, 0.125), (0.9, 0.125)];
const TOLERANCE: f64 = 0.01;

/// See the module documentation.
pub struct ALinkIsMovedResizedCopiedAndDeleted;

impl Check for ALinkIsMovedResizedCopiedAndDeleted {
    fn name(&self) -> &'static str {
        "a_link_is_moved_resized_copied_and_deleted"
    }

    fn defect(&self) -> &'static str {
        "a selected link cannot be moved, resized, copied or deleted, or its /QuadPoints — the \
         clickable area a viewer honours over /Rect — stay where the box was, so the saved link \
         answers clicks somewhere it is not drawn"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        let saved = ctx.out("link-quads.saved.pdf");
        let clip_dir = ctx.out("link-quads-clipboard");
        let _ = std::fs::remove_file(&saved);
        let env = [
            (SAVE_PATH_ENV, saved.display().to_string()),
            (CLIPBOARD_ENV, clip_dir.display().to_string()),
        ];
        let driven = launch_with(ctx, &mut report, &FIXTURE, INVOKE, "link-quads", &env).and_then(
            |(session, pointer)| {
                let outcome = drive(ctx, &mut report, &session, &pointer, &saved);
                let parked = pointer.gone(&session);
                match outcome? {
                    Some(failure) => Ok(Some(failure)),
                    None => parked.map(|_| None),
                }
            },
        );
        match driven {
            Ok(Some(failure)) => report.fail(failure),
            Ok(None) => report.pass(),
            Err(why) => report.from_error(&why),
        }
    }
}

/// The last line named `event` after `mark`, or the failure sentence.
fn after(
    session: &Session,
    event: &str,
    mark: usize,
    what: &str,
) -> Result<std::result::Result<crate::trace::TraceLine, String>> {
    let trace = session.trace()?;
    Ok(trace.last_after(event, mark).cloned().ok_or_else(|| {
        format!(
            "{what} and no `{event}` followed. Trace: {}.",
            session.trace_path().display()
        )
    }))
}

fn drag_page(
    ctx: &CheckContext,
    session: &Session,
    pointer: &ScriptedPointer,
    from: (f64, f64),
    to: (f64, f64),
) -> Result<()> {
    let mapping =
        CanvasMapping::from_trace(&session.trace()?, &ctx.profile.vocab, FIXTURE.page, 0)?;
    let a = mapping.doc_to_window(DocPoint::new(0, from.0, from.1))?;
    let b = mapping.doc_to_window(DocPoint::new(0, to.0, to.1))?;
    pointer.hover(session, a)?;
    session.settle(10);
    pointer.drag(session, a, b, 10)?;
    session.settle(30);
    Ok(())
}

/// Click inside a link and require it selected.
fn select_link(
    ctx: &CheckContext,
    session: &Session,
    pointer: &ScriptedPointer,
    at: (f64, f64),
) -> Result<Option<String>> {
    let mark = session.trace()?.mark();
    click_on(ctx, session, pointer, &FIXTURE, 0, at)?;
    let trace = session.trace()?;
    Ok((!trace
        .last_after(SELECTED, mark)
        .is_some_and(|l| l.get("subtype") == Some("Link")))
    .then(|| {
        format!(
            "a click at {at:?} inside a link traced no `{SELECTED}` with subtype=Link. Trace: {}.",
            session.trace_path().display()
        )
    }))
}

/// Save a copy and return the links it holds, or the failure sentence.
fn save_and_read(
    ctx: &CheckContext,
    session: &Session,
    pointer: &ScriptedPointer,
    saved: &std::path::Path,
) -> Result<std::result::Result<Vec<Link>, String>> {
    let mark = session.trace()?.mark();
    press_on_tab(ctx, session, pointer, SAVE_COPY, FILE_TAB)?;
    session.settle(40);
    if let Err(why) = after(session, SAVED, mark, "Save a copy was pressed")? {
        return Ok(Err(why));
    }
    let bytes = std::fs::read(saved)
        .map_err(|e| Error::new(format!("cannot read {}: {e}", saved.display())))?;
    Ok(Ok(links_in(&bytes)))
}

fn drive(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
    saved: &std::path::Path,
) -> Result<Option<String>> {
    if let Some(failure) = select_link(ctx, session, pointer, ON_LINK)? {
        return Ok(Some(failure));
    }
    // --- move, then resize: the engine reports each geometry key it carried.
    let mark = session.trace()?.mark();
    drag_page(ctx, session, pointer, ON_LINK, MOVE_TO)?;
    let moved = match after(session, MOVED, mark, "the selected link's body was dragged")? {
        Ok(line) => line,
        Err(why) => return Ok(Some(why)),
    };
    report.note(format!("moved: `{}`", moved.raw));
    let mark = session.trace()?.mark();
    drag_page(ctx, session, pointer, GRIP, GRIP_TO)?;
    let resized = match after(
        session,
        RESIZED,
        mark,
        "the moved link's top-right grip was dragged",
    )? {
        Ok(line) => line,
        Err(why) => return Ok(Some(why)),
    };
    report.note(format!("resized: `{}`", resized.raw));

    // --- copy and paste from the ribbon, the clipboard captured to a folder.
    let mark = session.trace()?.mark();
    press_on_tab(ctx, session, pointer, COPY, EDIT_TAB)?;
    let copied = match after(
        session,
        COPIED,
        mark,
        "Copy was pressed with the link selected",
    )? {
        Ok(line) => line,
        Err(why) => return Ok(Some(why)),
    };
    report.note(format!("copied: `{}`", copied.raw));
    if copied.get("annots") != Some("1") {
        return Ok(Some(format!(
            "`{}` must carry the one selected link (annots=1).",
            copied.raw
        )));
    }
    let mark = session.trace()?.mark();
    press_on_tab(ctx, session, pointer, PASTE, EDIT_TAB)?;
    session.settle(30);
    let pasted = match after(
        session,
        PASTED,
        mark,
        "Paste was pressed after copying the link",
    )? {
        Ok(line) => line,
        Err(why) => return Ok(Some(why)),
    };
    report.note(format!("pasted: `{}`", pasted.raw));
    if pasted.get("annots") != Some("1") {
        return Ok(Some(format!(
            "`{}` must plant one link (annots=1).",
            pasted.raw
        )));
    }

    // --- the file: two links, each quad where its box is.
    let links = match save_and_read(ctx, session, pointer, saved)? {
        Ok(links) => links,
        Err(why) => return Ok(Some(why)),
    };
    report.artifact(saved.to_path_buf());
    for link in &links {
        report.note(format!(
            "saved link {}: rect={:?} quads={:?}",
            link.num, link.rect, link.quads
        ));
    }
    if links.len() != 2 {
        return Ok(Some(format!(
            "after a copy and paste the saved file holds {} links, not 2: {links:?}.",
            links.len()
        )));
    }
    if let Some(failure) = links.iter().find_map(quad_off_its_box) {
        return Ok(Some(failure));
    }

    // --- delete one, and the file holds the other.
    if let Some(failure) = select_link(ctx, session, pointer, ON_MOVED)? {
        return Ok(Some(failure));
    }
    let mark = session.trace()?.mark();
    pointer.key(session, None, "Delete", None)?;
    session.settle(30);
    let deleted = match after(
        session,
        DELETED,
        mark,
        "Delete was pressed with a link selected",
    )? {
        Ok(line) => line,
        Err(why) => return Ok(Some(why)),
    };
    report.note(format!("deleted: `{}`", deleted.raw));
    let links = match save_and_read(ctx, session, pointer, saved)? {
        Ok(links) => links,
        Err(why) => return Ok(Some(why)),
    };
    if links.len() != 1 {
        return Ok(Some(format!(
            "one of two links was deleted and the saved file holds {}: {links:?}.",
            links.len()
        )));
    }
    Ok(None)
}

/// A `/Link` as the saved file finally states it.
#[derive(Debug)]
struct Link {
    num: u32,
    rect: [f64; 4],
    quads: Vec<f64>,
}

/// The failure sentence when `link`'s quad is not the fixture's quad mapped
/// into its current `/Rect`.
fn quad_off_its_box(link: &Link) -> Option<String> {
    let [x0, y0, x1, y1] = link.rect;
    let (w, h) = (x1 - x0, y1 - y0);
    let want: Vec<f64> = QUAD_IN_RECT
        .iter()
        .flat_map(|&(fx, fy)| [x0 + fx * w, y0 + fy * h])
        .collect();
    let off = link.quads.len() != 8
        || link
            .quads
            .iter()
            .zip(&want)
            .enumerate()
            .any(|(i, (got, want))| {
                (got - want).abs() > TOLERANCE * if i % 2 == 0 { w } else { h }
            });
    off.then(|| {
        format!(
            "link {} has /Rect {:?} and /QuadPoints {:?}; its quad should be {want:?}, where the \
             box now is. A viewer that honours /QuadPoints answers clicks on this link somewhere \
             it is not drawn.",
            link.num, link.rect, link.quads
        )
    })
}

/// Every live `/Link` in `bytes`: the last definition of each object number,
/// as an appended save leaves them, kept when a page's last definition lists
/// it in `/Annots`. A deleted annotation's original body stays in an appended
/// file, so being defined is not being on the page. Objects inside object
/// streams are not seen; the save writes its appended objects plainly.
fn links_in(bytes: &[u8]) -> Vec<Link> {
    let text = String::from_utf8_lossy(bytes);
    let mut last: std::collections::BTreeMap<u32, &str> = std::collections::BTreeMap::new();
    let mut rest = text.as_ref();
    while let Some(at) = rest.find(" 0 obj") {
        let head = &rest[..at];
        let digits = head.len() - head.trim_end_matches(|c: char| c.is_ascii_digit()).len();
        let body_start = at + " 0 obj".len();
        let Some(end) = rest[body_start..].find("endobj") else {
            break;
        };
        if digits > 0
            && let Ok(num) = head[head.len() - digits..].parse::<u32>()
        {
            last.insert(num, &rest[body_start..body_start + end]);
        }
        rest = &rest[body_start + end..];
    }
    let listed: std::collections::BTreeSet<u32> = last
        .values()
        .filter(|body| is_page(body))
        .filter_map(|body| references_after(body, "/Annots"))
        .flatten()
        .collect();
    last.into_iter()
        .filter(|(num, _)| listed.contains(num))
        .filter(|(_, body)| body.contains("/Subtype /Link") || body.contains("/Subtype/Link"))
        .filter_map(|(num, body)| {
            let rect = numbers_after(body, "/Rect")?;
            let rect: [f64; 4] = rect.try_into().ok()?;
            let quads = numbers_after(body, "/QuadPoints").unwrap_or_default();
            Some(Link { num, rect, quads })
        })
        .collect()
}

/// Whether `body` is a `/Page` dictionary (not `/Pages`).
fn is_page(body: &str) -> bool {
    ["/Type /Page", "/Type/Page"].iter().any(|t| {
        body.match_indices(t)
            .any(|(i, _)| !body[i + t.len()..].starts_with('s'))
    })
}

/// The object numbers referenced (`n g R`) in the array following `key`.
fn references_after(body: &str, key: &str) -> Option<Vec<u32>> {
    let from = body.find(key)? + key.len();
    let open = body[from..].find('[')? + from + 1;
    let close = body[open..].find(']')? + open;
    let words: Vec<&str> = body[open..close].split_whitespace().collect();
    Some(
        words
            .windows(3)
            .filter(|w| w[2] == "R")
            .filter_map(|w| w[0].parse().ok())
            .collect(),
    )
}

/// The numbers in the array following `key` in `body`.
fn numbers_after(body: &str, key: &str) -> Option<Vec<f64>> {
    let from = body.find(key)? + key.len();
    let open = body[from..].find('[')? + from + 1;
    let close = body[open..].find(']')? + open;
    body[open..close]
        .split_whitespace()
        .map(|n| n.parse::<f64>().ok())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_fixtures_own_link_is_on_its_box() {
        let bytes = std::fs::read(
            crate::fixture::workspace_root()
                .join("fixtures")
                .join(FIXTURE.file),
        )
        .expect("the fixture is committed");
        let links = links_in(&bytes);
        assert_eq!(links.len(), 1);
        assert_eq!(quad_off_its_box(&links[0]), None);
    }

    #[test]
    fn a_quad_left_behind_by_a_moved_box_is_named() {
        let link = Link {
            num: 9,
            rect: [100.0, 100.0, 300.0, 140.0],
            quads: vec![120.0, 215.0, 280.0, 215.0, 120.0, 185.0, 280.0, 185.0],
        };
        assert!(quad_off_its_box(&link).is_some());
    }

    #[test]
    fn the_last_definition_of_an_object_wins() {
        let bytes = b"3 0 obj << /Type /Page /Annots [5 0 R] >> endobj\n\
                      5 0 obj << /Subtype /Link /Rect [0 0 1 1] >> endobj\n\
                      5 0 obj << /Subtype /Link /Rect [0 0 2 2] >> endobj\n";
        let links = links_in(bytes);
        assert_eq!(links.len(), 1);
        assert_eq!(links[0].rect, [0.0, 0.0, 2.0, 2.0]);
    }

    #[test]
    fn a_link_the_page_no_longer_lists_is_gone() {
        let bytes = b"3 0 obj << /Type /Page /Annots [5 0 R] >> endobj\n\
                      5 0 obj << /Subtype /Link /Rect [0 0 1 1] >> endobj\n\
                      6 0 obj << /Subtype /Link /Rect [0 0 2 2] >> endobj\n\
                      3 0 obj << /Type /Page /Annots [6 0 R] >> endobj\n";
        let links = links_in(bytes);
        assert_eq!(links.len(), 1);
        assert_eq!(links[0].num, 6);
    }
}
