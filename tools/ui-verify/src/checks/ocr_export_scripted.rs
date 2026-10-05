//! `ocr_layer_exports_to_word` and `ocr_layer_exports_to_text` — File ▸
//! Export ▸ Word document… and Text… on `fixtures/ocr-layer.pdf`, whose page
//! carries an OCR layer (invisible text over a picture) beside visible text,
//! write the layer's words into the exported file. Run with the scripted
//! pointer in a window placed off the desktop.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/ocr_export_scripted.md`.

use crate::checks::driving::{SHELL_DIAG_ENV, declared, declared_in, declared_names, list};
use crate::checks::{Check, CheckContext};
use crate::coords::WindowPoint;
use crate::error::{Error, Result};
use crate::input::scripted::ScriptedPointer;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;

const FIXTURE: &str = "ocr-layer.pdf";
const MODE: &str = "ribbon.mode.read"; // ui-text-exempt: a trace region name, never displayed
const TAB: &str = "ribbon.tab.file"; // ui-text-exempt: a trace region name, never displayed
/// The Export group when the band is too narrow to show it open.
const COLLAPSED: &str = "ribbon.group.file.export.collapsed"; // ui-text-exempt: a trace region name, never displayed
const SAVE_PATH_ENV: &str = "PDFCER_DIAG_SAVE_PATH"; // ui-text-exempt: an environment variable name
/// Off the desktop, so no OS input can reach it and none of his is taken.
const OFFSCREEN: &str = "-4200,-4200,1400,900";
/// Words only the fixture's OCR layer (object 6, rendering mode 3) spells.
const OCR_WORDS: [&str; 4] = ["SITE", "PLAN", "REVISION", "DRAWING"];
/// The fixture's visible stamp: present in any export that read the page.
const VISIBLE: &str = "VISIBLE CONTROL STAMP";
/// The Word package's main part, Deflate-compressed inside the zip.
const MAIN_PART: &str = "word/document.xml";

/// One export format.
#[derive(Clone, Copy)]
pub enum Format {
    Word,
    Text,
}

impl Format {
    const fn item(self) -> &'static str {
        match self {
            Self::Word => "ribbon.item.file.export_word", // ui-text-exempt: a trace region name
            Self::Text => "ribbon.item.file.export_text", // ui-text-exempt: a trace region name
        }
    }

    const fn button(self) -> &'static str {
        match self {
            Self::Word => "export-word.export", // ui-text-exempt: a trace region name
            Self::Text => "export-text.export", // ui-text-exempt: a trace region name
        }
    }

    /// The trace line a successful write emits.
    const fn wrote(self) -> &'static str {
        match self {
            Self::Word => "export-word", // ui-text-exempt: a trace event name
            Self::Text => "export-text", // ui-text-exempt: a trace event name
        }
    }

    const fn extension(self) -> &'static str {
        match self {
            Self::Word => "docx", // ui-text-exempt: a file extension
            Self::Text => "txt",  // ui-text-exempt: a file extension
        }
    }
}

pub struct OcrLayerExports {
    pub format: Format,
}

impl Check for OcrLayerExports {
    fn name(&self) -> &'static str {
        match self.format {
            Format::Word => "ocr_layer_exports_to_word",
            Format::Text => "ocr_layer_exports_to_text",
        }
    }

    fn defect(&self) -> &'static str {
        "a recognised page exports without its recognised words: the OCR layer is invisible \
         text, and an export that keeps only what is drawn writes a scan's export empty"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        match drive(ctx, &mut report, self.format) {
            Ok(Some(failure)) => report.fail(failure),
            Ok(None) => report.pass(),
            Err(why) => report.from_error(&why),
        }
    }
}

fn drive(ctx: &CheckContext, report: &mut CheckReport, format: Format) -> Result<Option<String>> {
    let pdf = crate::fixture::workspace_root()
        .join("fixtures")
        .join(FIXTURE);
    if !pdf.is_file() {
        return Ok(Some(format!(
            "the fixture is not at {}; it is committed, so this is a broken checkout.",
            pdf.display()
        )));
    }
    let stem = format!("ocr-export-{}", format.extension());
    let target = ctx.out(&format!("{stem}.{}", format.extension()));
    let _ = std::fs::remove_file(&target);
    if target.exists() {
        return Err(Error::new(format!(
            "cannot clear {} before the export.",
            target.display()
        )));
    }
    let (trace, path) = export(ctx, report, &pdf, &target, &stem, format)?;
    let Some(line) = trace.last(format.wrote()) else {
        return Ok(Some(format!(
            "Export was pressed and no `{}` line followed. Refused: {:?}. Failed: {:?}. Trace: \
             {path}.",
            format.wrote(),
            trace
                .last(&format!("{}-refused", format.wrote()))
                .map(|l| l.raw.clone()),
            trace
                .last(&format!("{}-failed", format.wrote()))
                .map(|l| l.raw.clone()),
        )));
    };
    report.note(format!("wrote: `{}`", line.raw));
    let bytes = std::fs::read(&target).map_err(|e| {
        Error::new(format!(
            "the export traced success and {} cannot be read: {e}",
            target.display()
        ))
    })?;
    let text = match format {
        Format::Text => String::from_utf8_lossy(&bytes).into_owned(),
        Format::Word => match zip_part(&bytes, MAIN_PART) {
            Ok(xml) => String::from_utf8_lossy(&xml).into_owned(),
            Err(why) => return Ok(Some(format!("{}: {why}", target.display()))),
        },
    };
    Ok(judge(&text, &target.display().to_string()))
}

/// The visible stamp proves the page was read; then every OCR word must be
/// there too.
fn judge(text: &str, file: &str) -> Option<String> {
    if !text.contains(VISIBLE) {
        return Some(format!(
            "{file} does not hold the page's visible stamp `{VISIBLE}`, so it is no export of \
             this page and says nothing about the OCR layer."
        ));
    }
    let missing: Vec<&str> = OCR_WORDS
        .iter()
        .copied()
        .filter(|w| !text.contains(w))
        .collect();
    if missing.is_empty() {
        return None;
    }
    Some(format!(
        "★ {file} holds the visible stamp and not the OCR layer's words {missing:?}: the export \
         dropped the recognised (invisible) text."
    ))
}

/// The bytes of `name` in the zip `bytes`, inflated. Reads local file headers
/// in order; the engine's writer puts the sizes in each header.
fn zip_part(bytes: &[u8], name: &str) -> std::result::Result<Vec<u8>, String> {
    let u16_at = |i: usize| {
        bytes
            .get(i..i + 2)
            .map(|b| u16::from_le_bytes([b[0], b[1]]))
    };
    let u32_at = |i: usize| {
        bytes
            .get(i..i + 4)
            .map(|b| u32::from_le_bytes([b[0], b[1], b[2], b[3]]))
    };
    let mut at = 0usize;
    while u32_at(at) == Some(0x0403_4b50) {
        let fields = (
            u16_at(at + 8),
            u32_at(at + 18),
            u16_at(at + 26),
            u16_at(at + 28),
        );
        let (Some(method), Some(size), Some(name_len), Some(extra)) = fields else {
            break;
        };
        let start = at + 30;
        let data = start + usize::from(name_len) + usize::from(extra);
        let end = data + size as usize;
        let (Some(here), Some(body)) = (
            bytes.get(start..start + usize::from(name_len)),
            bytes.get(data..end),
        ) else {
            break;
        };
        if here == name.as_bytes() {
            return match method {
                0 => Ok(body.to_vec()),
                8 => miniz_oxide::inflate::decompress_to_vec(body)
                    .map_err(|e| format!("`{name}` does not inflate: {e:?}")),
                other => Err(format!("`{name}` is stored with method {other}")),
            };
        }
        at = end;
    }
    Err(format!("no `{name}` entry in the package"))
}

/// Launch on `pdf`, click Read mode, the File tab, the export item and its
/// window's Export button, with the save dialog answered by `target`.
fn export(
    ctx: &CheckContext,
    report: &mut CheckReport,
    pdf: &std::path::Path,
    target: &std::path::Path,
    stem: &str,
    format: Format,
) -> Result<(crate::trace::Trace, String)> {
    let exe = ctx.resolve_exe().ok_or_else(|| {
        Error::new(format!(
            "no binary to drive. Pass --exe, or build the profile's default at {}.",
            ctx.profile.default_exe
        ))
    })?;
    let viewport_env = ctx.profile.viewport_env.ok_or_else(|| {
        Error::new("the profile has no viewport variable to place the window off the desktop.")
    })?;
    let ui_rect = ctx
        .profile
        .vocab
        .ui_rect_event
        .ok_or_else(|| Error::new("the profile declares no ui-rect trace event."))?;
    let mut spec = LaunchSpec::new(&exe, ctx.out(&format!("{stem}.trace.txt")));
    spec.pdf = Some(pdf.to_path_buf());
    spec.env.push((
        ctx.profile.diag_env.0.to_owned(),
        ctx.profile.diag_env.1.to_owned(),
    ));
    spec.env
        .push((SHELL_DIAG_ENV.0.to_owned(), SHELL_DIAG_ENV.1.to_owned()));
    spec.env
        .push((viewport_env.to_owned(), OFFSCREEN.to_owned()));
    spec.env
        .push((SAVE_PATH_ENV.to_owned(), target.display().to_string()));
    spec.place = false;
    spec.allow_stale = ctx.allow_stale;
    spec.source_root = ctx.source_root.clone();
    let pointer = ScriptedPointer::attach(&mut spec, ctx.out(&format!("{stem}.pointer.txt")))?;
    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    let path = session.trace_path().display().to_string();
    report.artifact(session.trace_path().to_path_buf());
    report.artifact(pointer.path().to_path_buf());
    session.settle(45);

    let click = |region: &str| -> Result<()> {
        let trace = session.trace()?;
        let (rect, viewport) = declared_in(&trace, ui_rect, region).ok_or_else(|| {
            let prefix = region.rsplit_once('.').map_or(region, |(head, _)| head);
            Error::new(format!(
                "no `{region}` region. Declared under `{prefix}`: {}.",
                list(&declared_names(&trace, ui_rect, prefix))
            ))
        })?;
        pointer.click_in(&session, viewport.as_deref(), WindowPoint::centre_of(rect))?;
        session.settle(15);
        Ok(())
    };
    click(MODE)?;
    click(TAB)?;
    let trace = session.trace()?;
    if declared(&trace, ui_rect, format.item()).is_none()
        && declared(&trace, ui_rect, COLLAPSED).is_some()
    {
        click(COLLAPSED)?;
    }
    click(format.item())?;
    session.settle(20);
    click(format.button())?;
    session.settle(30);
    pointer.gone(&session)?;
    Ok((session.trace()?, path))
}
