//! `checks::os_image_paste` — **a picture another program copied pastes onto
//! the page at the pointer, and a paste takes the newer of two copies**
//!
//! Drives the window off the desktop through the scripted pointer, in Edit
//! mode on a copy of the engine corpus's `four-pages.pdf`. The harness writes
//! the clipboard itself, so it snapshots it first and restores it after, and
//! only if nobody but this check and the app under test wrote it since.
//!
//! Oracles are the app's `clip-pasted source=os` lines: a 64×32 bitmap stating
//! 3780 pixels per metre is 48×24 pt and must be centred within
//! [`TOLERANCE_PT`] of the pointer; a second paste elsewhere must land
//! elsewhere; Ctrl+Z must trace `undo-applied`; text alone must paste as text
//! and add no picture. Then pdfcer copies the
//! selection: a paste must be pdfcer's own (`clipboard-paste kind=selection`),
//! and after another program's 32×16 bitmap lands, the next paste must be that
//! bitmap at 24×12 pt.

use crate::checks::driving::SHELL_DIAG_ENV;
use crate::checks::{Check, CheckContext};
use crate::coords::{CanvasMapping, DocPoint, WindowPoint};
use crate::error::{Error, Result};
use crate::input::scripted::ScriptedPointer;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;
use crate::sys;

const OFFSCREEN: &str = "-4200,-4200,1400,900";
const INVOKE: &str = "mode.edit";
const DOC: &str = "D:/Dev/pdfcer/fixtures/synthetic/pageops/four-pages.pdf";
pub(super) const PASTED: &str = "clip-pasted";
const OWN_PASTE: &str = "clipboard-paste";
const OWN_COPY: &str = "clipboard-copy";
const UNDONE: &str = "undo-applied";
const ADDED: &str = "add-image";
/// How far a pasted picture's centre may sit from the pointer, in points.
const TOLERANCE_PT: f64 = 2.0;
/// 3780 pixels per metre is 96 per inch, so a pixel is three quarters of a point.
const PPM: u32 = 3780;
pub(super) const FIRST: (f64, f64) = (200.0, 500.0);
const SECOND: (f64, f64) = (400.0, 250.0);

/// See the module documentation.
pub struct AnOsPicturePastesAtThePointer;

impl Check for AnOsPicturePastesAtThePointer {
    fn name(&self) -> &'static str {
        "a_picture_copied_in_another_program_pastes_at_the_pointer"
    }

    fn defect(&self) -> &'static str {
        "a picture copied in another program pastes nothing, or lands away from the pointer, \
         or a paste after such a copy repeats pdfcer's own older clip"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        let mut guard = ClipGuard::take();
        let driven = launch(ctx, &mut report, "os-image-paste")
            .and_then(|(session, pointer)| drive(ctx, &mut report, &session, &pointer, &mut guard));
        report.note(guard.release());
        match driven {
            Ok(Some(failure)) => report.fail(failure),
            Ok(None) => report.pass(),
            Err(why) => report.from_error(&why),
        }
    }
}

/// The clipboard as it was before the check, put back afterwards unless
/// someone other than the check or the app wrote it in between.
pub(super) struct ClipGuard {
    saved: Vec<(u32, Vec<u8>)>,
    last_ours: u32,
}

impl ClipGuard {
    pub(super) fn take() -> Self {
        Self {
            saved: sys::snapshot(),
            last_ours: sys::clipboard_sequence(),
        }
    }

    /// Write `items`, or fail the step.
    pub(super) fn set(&mut self, items: &[(u32, Vec<u8>)]) -> Result<()> {
        if !sys::set_clipboard(items) {
            return Err(Error::new("the harness could not write the clipboard."));
        }
        self.last_ours = sys::clipboard_sequence();
        Ok(())
    }

    /// Count the app's own write as ours.
    pub(super) fn adopt(&mut self) {
        self.last_ours = sys::clipboard_sequence();
    }

    pub(super) fn release(&self) -> String {
        if sys::clipboard_sequence() != self.last_ours {
            return "clipboard left alone: another program wrote it during the check".to_owned();
        }
        if sys::restore(&self.saved) {
            format!("clipboard restored ({} formats)", self.saved.len())
        } else {
            "clipboard NOT restored: it could not be opened".to_owned()
        }
    }
}

pub(super) fn launch(
    ctx: &CheckContext,
    report: &mut CheckReport,
    stem: &str,
) -> Result<(Session, ScriptedPointer)> {
    let exe = ctx
        .resolve_exe()
        .ok_or_else(|| Error::new("no binary to drive. Pass --exe, or build the default."))?;
    let viewport_env = ctx.profile.viewport_env.ok_or_else(|| {
        Error::new("the profile has no viewport variable to place the window off the desktop.")
    })?;
    let doc = ctx.out(&format!("{stem}-source.pdf"));
    std::fs::copy(DOC, &doc).map_err(|e| Error::new(format!("copying {DOC}: {e}")))?;
    let mut spec = LaunchSpec::new(&exe, ctx.out(&format!("{stem}.trace.txt")));
    spec.pdf = Some(doc);
    for (k, v) in [
        (ctx.profile.diag_env.0, ctx.profile.diag_env.1),
        SHELL_DIAG_ENV,
        (viewport_env, OFFSCREEN),
        ("PDFCER_DIAG_INVOKE", INVOKE),
    ] {
        spec.env.push((k.to_owned(), v.to_owned()));
    }
    spec.place = false;
    spec.allow_stale = ctx.allow_stale;
    spec.source_root = ctx.source_root.clone();
    let pointer = ScriptedPointer::attach(&mut spec, ctx.out(&format!("{stem}.pointer.txt")))?;
    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    report.artifact(session.trace_path().to_path_buf());
    report.artifact(pointer.path().to_path_buf());
    session.settle(45);
    Ok((session, pointer))
}

/// A `width`×`height` 32-bit `BI_BITFIELDS` bitmap at [`PPM`], opaque teal.
pub(super) fn dib(width: u32, height: u32) -> Vec<u8> {
    let mut b = Vec::new();
    for v in [40u32, width, height] {
        b.extend(v.to_le_bytes());
    }
    b.extend(1u16.to_le_bytes());
    b.extend(32u16.to_le_bytes());
    for v in [3u32, width * height * 4, PPM, PPM, 0, 0] {
        b.extend(v.to_le_bytes());
    }
    for mask in [0x00FF_0000u32, 0x0000_FF00, 0x0000_00FF] {
        b.extend(mask.to_le_bytes());
    }
    for _ in 0..width * height {
        b.extend([0x80, 0x80, 0x00, 0xFF]);
    }
    b
}

pub(super) fn utf16(text: &str) -> Vec<u8> {
    text.encode_utf16()
        .chain(std::iter::once(0))
        .flat_map(u16::to_le_bytes)
        .collect()
}

pub(super) fn count(session: &Session, name: &str) -> Result<usize> {
    Ok(session.trace()?.events(name).count())
}

/// Where `p` on page 0 is in the window, measured from the latest frame.
pub(super) fn at(ctx: &CheckContext, session: &Session, p: (f64, f64)) -> Result<WindowPoint> {
    let pdf = std::path::Path::new(DOC);
    let geom = crate::fixture::page_geometry(pdf)
        .ok_or_else(|| Error::new(format!("{DOC} has no page geometry the harness reads.")))?;
    let mapping = CanvasMapping::from_trace(&session.trace()?, &ctx.profile.vocab, geom, 0)?;
    mapping.doc_to_window(DocPoint::new(0, p.0, p.1))
}

/// Hover `p`, paste, and return the newest `clip-pasted` line's rectangle if
/// the paste added one.
fn paste_at(
    ctx: &CheckContext,
    session: &Session,
    pointer: &ScriptedPointer,
    p: (f64, f64),
) -> Result<Option<[f64; 4]>> {
    let before = count(session, PASTED)?;
    pointer.hover(session, at(ctx, session, p)?)?;
    pointer.paste(session, None, "x")?;
    session.settle(20);
    let trace = session.trace()?;
    Ok(trace.events(PASTED).nth(before).and_then(rect))
}

/// A `clip-pasted` line's rectangle as `[llx, lly, urx, ury]`.
pub(super) fn rect(line: &crate::trace::TraceLine) -> Option<[f64; 4]> {
    let field = |k| line.get(k).and_then(|v| v.parse::<f64>().ok());
    match (field("llx"), field("lly"), field("urx"), field("ury")) {
        (Some(a), Some(b), Some(c), Some(d)) => Some([a, b, c, d]),
        _ => None,
    }
}

/// A rectangle as `llx lly urx ury`.
pub(super) fn show(r: [f64; 4]) -> String {
    format!("{:.2} {:.2} {:.2} {:.2}", r[0], r[1], r[2], r[3])
}

/// `r` is `size` points and centred within the tolerance of `p`.
pub(super) fn lands(r: [f64; 4], p: (f64, f64), size: (f64, f64)) -> Option<String> {
    let (w, h) = (r[2] - r[0], r[3] - r[1]);
    let (cx, cy) = ((r[0] + r[2]) / 2.0, (r[1] + r[3]) / 2.0);
    if (w - size.0).abs() > 0.5 || (h - size.1).abs() > 0.5 {
        return Some(format!(
            "the picture landed {w:.2}×{h:.2} pt, not {:.0}×{:.0} pt.",
            size.0, size.1
        ));
    }
    ((cx - p.0).abs() > TOLERANCE_PT || (cy - p.1).abs() > TOLERANCE_PT).then(|| {
        format!(
            "the picture is centred at ({cx:.2}, {cy:.2}), not at the pointer ({:.0}, {:.0}).",
            p.0, p.1
        )
    })
}

fn drive(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
    guard: &mut ClipGuard,
) -> Result<Option<String>> {
    let failure = match place_twice(ctx, report, session, pointer, guard)? {
        Some(failure) => Some(failure),
        None => match text_is_no_picture(ctx, session, pointer, guard)? {
            Some(failure) => Some(failure),
            None => newer_wins(ctx, report, session, pointer, guard)?,
        },
    };
    let parked = pointer.gone(session);
    match failure {
        Some(failure) => Ok(Some(failure)),
        None => parked.map(|_| None),
    }
}

/// A bitmap pastes at the pointer, at another point elsewhere, and undoes.
fn place_twice(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
    guard: &mut ClipGuard,
) -> Result<Option<String>> {
    guard.set(&[(sys::CF_DIB, dib(64, 32))])?;
    let Some(first) = paste_at(ctx, session, pointer, FIRST)? else {
        return Ok(Some(format!(
            "pasting another program's picture traced no `{PASTED}` line."
        )));
    };
    report.note(format!("first paste {}", show(first)));
    if let Some(failure) = lands(first, FIRST, (48.0, 24.0)) {
        return Ok(Some(failure));
    }
    if count(session, ADDED)? == 0 {
        return Ok(Some(format!(
            "the paste placed nothing (no `{ADDED}` line)."
        )));
    }
    let Some(second) = paste_at(ctx, session, pointer, SECOND)? else {
        return Ok(Some("a second paste traced nothing.".to_owned()));
    };
    if let Some(failure) = lands(second, SECOND, (48.0, 24.0)) {
        return Ok(Some(format!("second paste: {failure}")));
    }
    let undos = count(session, UNDONE)?;
    pointer.key(session, None, "Z", Some("ctrl"))?;
    session.settle(20);
    Ok((count(session, UNDONE)? == undos).then(|| "Ctrl+Z did not undo the paste.".to_owned()))
}

/// Text alone on the clipboard pastes as text, never as a picture.
fn text_is_no_picture(
    ctx: &CheckContext,
    session: &Session,
    pointer: &ScriptedPointer,
    guard: &mut ClipGuard,
) -> Result<Option<String>> {
    guard.set(&[(sys::CF_UNICODETEXT, utf16("plain words"))])?;
    let (pasted, added) = (count(session, PASTED)?, count(session, ADDED)?);
    pointer.hover(session, at(ctx, session, FIRST)?)?;
    pointer.paste(session, None, "plain words")?;
    session.settle(20);
    if count(session, ADDED)? != added {
        return Ok(Some(
            "text on the clipboard was placed as a picture.".to_owned(),
        ));
    }
    let trace = session.trace()?;
    let as_text = trace
        .events(PASTED)
        .nth(pasted)
        .is_some_and(|l| l.get("kind") == Some("text"));
    Ok((!as_text).then(|| format!("pasting text traced no `{PASTED} kind=text` line.")))
}

/// pdfcer's clip pastes until another program copies; then that copy does.
fn newer_wins(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
    guard: &mut ClipGuard,
) -> Result<Option<String>> {
    guard.set(&[(sys::CF_DIB, dib(64, 32))])?;
    if paste_at(ctx, session, pointer, FIRST)?.is_none() {
        return Ok(Some("the picture to copy was not placed.".to_owned()));
    }
    let copies = count(session, OWN_COPY)?;
    pointer.copy(session, None)?;
    session.settle(20);
    guard.adopt();
    if count(session, OWN_COPY)? == copies {
        return Ok(Some(format!(
            "copying the pasted picture traced no `{OWN_COPY}`."
        )));
    }
    let own = count(session, OWN_PASTE)?;
    if paste_at(ctx, session, pointer, SECOND)?.is_some() {
        return Ok(Some(
            "after pdfcer copied, a paste read the OS clipboard instead of pdfcer's clip."
                .to_owned(),
        ));
    }
    if count(session, OWN_PASTE)? == own {
        return Ok(Some(format!(
            "pdfcer's own clip did not paste (no `{OWN_PASTE}`)."
        )));
    }
    guard.set(&[(sys::CF_DIB, dib(32, 16))])?;
    let Some(newer) = paste_at(ctx, session, pointer, SECOND)? else {
        return Ok(Some(
            "★★★ after another program copied, the paste repeated pdfcer's older clip.".to_owned(),
        ));
    };
    report.note(format!("newer paste {}", show(newer)));
    Ok(lands(newer, SECOND, (24.0, 12.0)).map(|f| format!("the newer copy: {f}")))
}
