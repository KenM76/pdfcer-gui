//! `a_copied_paragraph_pastes_as_one_formatted_paragraph` — four clicks on a
//! six-line paragraph in Read mode, then Copy: the clipboard receives
//! `HTML Format`, `Rich Text Format` and `CF_UNICODETEXT` in one placement,
//! and the plain text joins the wrapped lines with a space, so a word
//! processor flows them as one paragraph. Driven through the scripted pointer
//! on a window off the desktop, with the payload sent to a folder so the
//! operator's clipboard is untouched.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/rich_copy.md`.

use super::dimdrive::{Fixture, run_on_with};
use crate::checks::{Check, CheckContext};
use crate::coords::{CanvasMapping, DocPoint, PageGeometry};
use crate::error::{Error, Result};
use crate::input::scripted::ScriptedPointer;
use crate::launch::Session;
use crate::report::CheckReport;

const FIXTURE: Fixture = Fixture {
    file: "paragraph.pdf",
    method: "Regenerate it: python tools/gen-reflow-fixture.py.",
    page: PageGeometry {
        width_pt: 612.0,
        height_pt: 792.0,
    },
};
const CAPTURE_ENV: &str = "PDFCER_DIAG_CLIPBOARD_DIR"; // ui-text-exempt: an environment variable name
const SELECTION: &str = "canvas-text-selection"; // ui-text-exempt: a trace event name, never displayed
const COPIED: &str = "text-copied"; // ui-text-exempt: a trace event name, never displayed
const DECLINED: &str = "text-copy-declined"; // ui-text-exempt: a trace event name, never displayed
/// The placement, in order, with the file each is captured to.
const HTML_FILE: &str = "1-HTML_Format.bin"; // ui-text-exempt: a capture file name
const RTF_FILE: &str = "2-Rich_Text_Format.bin"; // ui-text-exempt: a capture file name
const TEXT_FILE: &str = "3-CF_UNICODETEXT.bin"; // ui-text-exempt: a capture file name
/// Inside line 2 of the fixture's six-line block.
const ON_LINE: (f64, f64) = (120.0, 668.0);
/// The end of line 1 and the start of line 2 joined by one space: a copy that
/// keeps the drawn line break cannot contain it.
const JOINED: &str = "of a sheet in one file"; // ui-text-exempt: fixture text the check searches for

/// See the module documentation.
pub struct ACopiedParagraphPastesAsOneFormattedParagraph;

impl Check for ACopiedParagraphPastesAsOneFormattedParagraph {
    fn name(&self) -> &'static str {
        "a_copied_paragraph_pastes_as_one_formatted_paragraph"
    }

    fn defect(&self) -> &'static str {
        "copying selected text puts plain text only on the clipboard, with a line break at every \
         drawn line, so a word processor loses the font and the paragraph"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let dir = ctx.out("rich-copy-capture");
        let _ = std::fs::remove_dir_all(&dir);
        let env = [(CAPTURE_ENV, dir.display().to_string())];
        run_on_with(self, ctx, &FIXTURE, "mode.read", "rich-copy", &env, drive)
    }
}

fn drive(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
) -> Result<Option<String>> {
    let dir = ctx.out("rich-copy-capture");
    if dir.exists() {
        return Err(Error::new(format!(
            "cannot clear {} before the run.",
            dir.display()
        )));
    }
    session.settle(45);
    let mapping =
        CanvasMapping::from_trace(&session.trace()?, &ctx.profile.vocab, FIXTURE.page, 0)?;
    let at = mapping.doc_to_window(DocPoint::new(0, ON_LINE.0, ON_LINE.1))?;
    pointer.quadruple_click(session, at)?;
    session.settle(40);
    let trace = session.trace()?;
    match trace.last(SELECTION) {
        Some(l) if l.get("via") == Some("paragraph") => {}
        other => {
            return Err(Error::new(format!(
                "four clicks selected `{}`, not the paragraph, so there is nothing to copy. \
                 `a_fourth_click_selects_the_paragraph` owns that defect.",
                other.map_or("nothing", |l| l.raw.as_str())
            )));
        }
    }
    let mark = trace.mark();
    pointer.copy(session, None)?;
    session.settle(40);
    let trace = session.trace()?;
    if let Some(declined) = trace.last_after(DECLINED, mark) {
        return Ok(Some(format!("★ Copy declined: `{}`.", declined.raw)));
    }
    let Some(line) = trace.last_after(COPIED, mark) else {
        return Ok(Some(format!("★ Copy traced no `{COPIED}` line.")));
    };
    report.note(format!("copy: `{}`", line.raw));
    report.artifact(dir.clone());
    judge(&dir, line.get_usize("paragraphs"))
}

/// The three captured formats, and one paragraph whose lines are joined.
fn judge(dir: &std::path::Path, paragraphs: Option<usize>) -> Result<Option<String>> {
    let read = |name: &str| std::fs::read(dir.join(name)).ok();
    let (Some(html), Some(rtf), Some(text)) = (read(HTML_FILE), read(RTF_FILE), read(TEXT_FILE))
    else {
        let found: Vec<String> = std::fs::read_dir(dir)
            .map(|d| {
                d.filter_map(|e| e.ok())
                    .map(|e| e.file_name().to_string_lossy().into_owned())
                    .collect()
            })
            .unwrap_or_default();
        return Ok(Some(format!(
            "★★ the copy did not place HTML, RTF and text in that order: captured {found:?}."
        )));
    };
    if !html.starts_with(b"Version:") || !rtf.starts_with(br"{\rtf1") {
        return Ok(Some(
            "★★ the HTML capture has no CF_HTML header or the RTF capture is not RTF.".to_owned(),
        ));
    }
    let units: Vec<u16> = text
        .chunks_exact(2)
        .map(|p| u16::from_le_bytes([p[0], p[1]]))
        .take_while(|&u| u != 0)
        .collect();
    let plain = String::from_utf16_lossy(&units);
    if !plain.contains(JOINED) || paragraphs != Some(1) {
        return Ok(Some(format!(
            "★★★ the copy is not one flowed paragraph: `paragraphs={paragraphs:?}`, and the text \
             does not join line 1 to line 2 as `{JOINED}`: {plain:?}"
        )));
    }
    if !contains(&rtf, b"Helvetica") || !contains(&html, b"Helvetica") {
        return Ok(Some(
            "★★★ the formatted copy does not name the page's font, Helvetica.".to_owned(),
        ));
    }
    Ok(None)
}

fn contains(hay: &[u8], needle: &[u8]) -> bool {
    hay.windows(needle.len()).any(|w| w == needle)
}
