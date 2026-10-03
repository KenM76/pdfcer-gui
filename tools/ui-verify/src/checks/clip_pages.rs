//! `checks::clip_pages` — **what another program copied becomes a new PDF
//! (File ▸ New from clipboard) or pages after the one on screen (Pages ▸
//! Insert from clipboard)**
//!
//! Drives the window off the desktop through the scripted pointer on a copy of
//! the engine corpus's `four-pages.pdf`, in Edit, with the clipboard
//! snapshotted and restored as in [`super::os_image_paste`]. The picture is a
//! 64×32 bitmap at 96 pixels per inch, 48×24 pt; the text is two lines split
//! by a form feed, which Import text as pages sets as two pages.
//!
//! Oracles: Insert from clipboard traces `insert-pages page=1 n=1`, receipt "Inserted 1 page after page 1.", for the
//! picture and `import-text-applied pages=2 first=1` for the text, both after
//! page 0, the page on screen; New from clipboard traces
//! `pages-from-clipboard to=new` with `kind=image pages=1 w=48.00 h=24.00` for
//! the picture and `kind=text pages=2` for the text, read from the new
//! document's own pages.

use super::os_image_paste::{self as osp, ClipGuard};
use crate::checks::driving::{declared_in, declared_names, list};
use crate::checks::{Check, CheckContext};
use crate::coords::WindowPoint;
use crate::error::{Error, Result};
use crate::input::scripted::ScriptedPointer;
use crate::launch::Session;
use crate::report::CheckReport;
use crate::sys;

const STEM: &str = "clip-pages";
const INSERTED: &str = "insert-pages";
const IMPORTED: &str = "import-text-applied";
const MADE: &str = "pages-from-clipboard";
const UI_RECT: &str = "ui-rect";
/// Two pages' worth of text: a form feed is a page break.
const TEXT: &str = "first page\u{c}second page";
/// The bitmap's size on the page, in points.
const SIZE: (f64, f64) = (48.0, 24.0);
/// How far a size may differ from [`SIZE`], in points.
const TOLERANCE_PT: f64 = 0.5;

/// See the module documentation.
pub struct TheClipboardBecomesANewPdfOrPagesAfterThisOne;

impl Check for TheClipboardBecomesANewPdfOrPagesAfterThisOne {
    fn name(&self) -> &'static str {
        "the_clipboard_becomes_a_new_pdf_or_pages_after_this_one"
    }

    fn defect(&self) -> &'static str {
        "a picture or text copied in another program cannot be made into a new PDF or added as \
         pages, as Acrobat's Create from Clipboard and Insert from Clipboard do"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        let mut guard = ClipGuard::take();
        let driven = osp::launch(ctx, &mut report, STEM)
            .and_then(|(session, pointer)| drive(&mut report, &session, &pointer, &mut guard));
        report.note(guard.release());
        match driven {
            Ok(Some(failure)) => report.fail(failure),
            Ok(None) => report.pass(),
            Err(why) => report.from_error(&why),
        }
    }
}

fn drive(
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
    guard: &mut ClipGuard,
) -> Result<Option<String>> {
    pointer.key(session, None, "3", Some("ctrl"))?;
    session.settle(20);
    let mut failure = insert_picture(session, pointer, guard)?;
    if failure.is_none() {
        failure = insert_text(session, pointer, guard)?;
    }
    if failure.is_none() {
        failure = new_from(report, session, pointer, guard, Picture)?;
    }
    if failure.is_none() {
        failure = new_from(report, session, pointer, guard, Words)?;
    }
    let parked = pointer.gone(session);
    match failure {
        Some(failure) => Ok(Some(failure)),
        None => parked.map(|_| None),
    }
}

/// Click the declared `region`.
fn press(session: &Session, pointer: &ScriptedPointer, region: &str) -> Result<()> {
    let trace = session.trace()?;
    let (rect, viewport) = declared_in(&trace, UI_RECT, region).ok_or_else(|| {
        Error::new(format!(
            "no `{region}` region. Ribbon items declared: {}.",
            list(&declared_names(&trace, UI_RECT, "ribbon.item."))
        ))
    })?;
    pointer.click_in(session, viewport.as_deref(), WindowPoint::centre_of(rect))?;
    session.settle(30);
    Ok(())
}

/// Press `item` on `tab` and return the first `name` line it added.
fn run(
    session: &Session,
    pointer: &ScriptedPointer,
    tab: &str,
    item: &str,
    name: &str,
) -> Result<Option<crate::trace::TraceLine>> {
    press(session, pointer, tab)?;
    let before = osp::count(session, name)?;
    press(session, pointer, item)?;
    Ok(session.trace()?.events(name).nth(before).cloned())
}

const PAGES_TAB: &str = "ribbon.tab.pages";
const INSERT: &str = "ribbon.item.pages.insert_from_clipboard";
const FILE_TAB: &str = "ribbon.tab.file";
const NEW: &str = "ribbon.item.file.new_from_clipboard";

/// A picture inserted from the clipboard is one page, after page 0.
fn insert_picture(
    session: &Session,
    pointer: &ScriptedPointer,
    guard: &mut ClipGuard,
) -> Result<Option<String>> {
    guard.set(&[(sys::CF_DIB, osp::dib(64, 32))])?;
    let Some(line) = run(session, pointer, PAGES_TAB, INSERT, INSERTED)? else {
        return Ok(Some(format!(
            "Insert from clipboard with a picture traced no `{INSERTED}` line."
        )));
    };
    let (page, n) = (line.get("page"), line.get("n"));
    if page != Some("1") || n != Some("1") {
        return Ok(Some(format!(
            "the picture went in as page={} n={}, not one page after page 0.",
            page.unwrap_or("-"),
            n.unwrap_or("-")
        )));
    }
    // The receipt names the page the insert follows, 1-based: page 0 is "page 1".
    Ok((!line.raw.contains(RECEIPT))
        .then(|| format!("the receipt does not say `{RECEIPT}`: {}", line.raw)))
}

/// The status receipt for one page inserted after the first.
const RECEIPT: &str = "Inserted 1 page after page 1.";

/// Text inserted from the clipboard is set as pages, after page 0.
fn insert_text(
    session: &Session,
    pointer: &ScriptedPointer,
    guard: &mut ClipGuard,
) -> Result<Option<String>> {
    guard.set(&[(sys::CF_UNICODETEXT, osp::utf16(TEXT))])?;
    let Some(line) = run(session, pointer, PAGES_TAB, INSERT, IMPORTED)? else {
        return Ok(Some(format!(
            "Insert from clipboard with text traced no `{IMPORTED}` line."
        )));
    };
    let (pages, first) = (line.get("pages"), line.get("first"));
    Ok((pages != Some("2") || first != Some("1")).then(|| {
        format!(
            "the text went in as pages={} first={}, not two pages after page 0.",
            pages.unwrap_or("-"),
            first.unwrap_or("-")
        )
    }))
}

/// What New from clipboard is given.
#[derive(Clone, Copy)]
enum Source {
    Picture,
    Words,
}
use Source::{Picture, Words};

/// New from clipboard makes a document of the picture's size, or of the text's
/// two pages.
fn new_from(
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
    guard: &mut ClipGuard,
    source: Source,
) -> Result<Option<String>> {
    let (kind, pages) = match source {
        Picture => {
            guard.set(&[(sys::CF_DIB, osp::dib(64, 32))])?;
            ("image", "1")
        }
        Words => {
            guard.set(&[(sys::CF_UNICODETEXT, osp::utf16(TEXT))])?;
            ("text", "2")
        }
    };
    let Some(line) = run(session, pointer, FILE_TAB, NEW, MADE)? else {
        return Ok(Some(format!(
            "New from clipboard with {kind} traced no `{MADE}` line."
        )));
    };
    let field = |k| line.get(k).unwrap_or("-");
    report.note(format!(
        "new from {kind}: pages={} w={} h={}",
        field("pages"),
        field("w"),
        field("h")
    ));
    if line.get("kind") != Some(kind) || line.get("pages") != Some(pages) {
        return Ok(Some(format!(
            "New from clipboard with {kind} made kind={} pages={}, not {pages} page(s) of {kind}.",
            field("kind"),
            field("pages")
        )));
    }
    if matches!(source, Words) {
        return Ok(None);
    }
    let size = |k| {
        line.get(k)
            .and_then(|v| v.parse::<f64>().ok())
            .unwrap_or(0.0)
    };
    let (w, h) = (size("w"), size("h"));
    Ok(
        ((w - SIZE.0).abs() > TOLERANCE_PT || (h - SIZE.1).abs() > TOLERANCE_PT)
            .then(|| format!("the new page is {w:.2}×{h:.2} pt, not the picture's 48×24.")),
    )
}
