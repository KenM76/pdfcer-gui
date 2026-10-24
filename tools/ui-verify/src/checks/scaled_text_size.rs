//! `scaled_text_shows_and_takes_its_drawn_size` — text written `1 Tf` under
//! an 18-point text matrix, as CAD exports write it, reads 18 in Properties,
//! typing 24 draws it at 24, the run keeps its characters, and a click on it
//! with the Text tool opens all of them. With `--pdf` and `--doc-point` it
//! drives the operator's drawing instead of the built-in page.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/scaled_text_size.md`.

use crate::checks::driving::{SHELL_DIAG_ENV, click_mode_segment, declared_in};
use crate::checks::{Check, CheckContext};
use crate::coords::{CanvasMapping, DocPoint, PageGeometry, WindowPoint};
use crate::error::{Error, Result};
use crate::input::scripted::ScriptedPointer;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;
use crate::trace::TraceLine;

/// Startup commands: Edit mode with Properties mounted, Paragraphs off so a
/// Text-tool click opens the clicked line alone.
const INVOKE: &str = "mode.edit,file.properties,edit.paragraphs";
/// The Text tool on the Edit tab.
const TEXT_TOOL: &str = "ribbon.item.edit.text";
/// The tab carrying it, clicked when another tab is showing.
const EDIT_TAB: &str = "ribbon.tab.edit";
/// `text-edit-caret … len=N`: the characters a Text-tool click opened.
const CARET: &str = "text-edit-caret"; // ui-text-exempt: a trace event name, never displayed
/// Off-desktop viewport, so the check runs while the operator uses the machine.
const OFFSCREEN: &str = "-4200,-4200,1400,900";
/// The size field.
const SIZE_REGION: &str = "properties.text.size";
/// The panel's read of the clicked run; `size=` is the drawn size.
const SYNCED: &str = "text-style-synced";
/// The restyle's summary line.
const APPLIED_EVENT: &str = "text-style-applied";
/// The I-beam restyle's summary line: what it owes the operator.
const DISCLOSED_EVENT: &str = "text-style-disclosed";
/// The restyle's refusal line.
const DECLINED_EVENT: &str = "text-style-declined";
/// The text matrix's scale, which is the size the text is drawn at.
const DRAWN_PT: f64 = 18.0;
/// What the check types, unless the subject is already drawn at it.
const WANTED_PT: f64 = 24.0;
/// What the check types on a subject already drawn at [`WANTED_PT`].
const OTHER_PT: f64 = 18.0;
/// A finished page render, off the frame or inside it; `notdef=` counts
/// glyphs drawn as `.notdef`, and is `-1` when the render made no raster.
const RENDERED: [&str; 2] = ["render-async-done", "render-inline"]; // ui-text-exempt: trace event names, never displayed
/// The open line's layout in its own font; `unread=` counts glyphs whose code
/// reads back as another character.
const SHAPED: &str = "text-edit-shaped"; // ui-text-exempt: a trace event name, never displayed
/// Why the opened line is not drawn in its own font.
const FALLBACK: &str = "text-edit-preview-fallback"; // ui-text-exempt: trace event name
/// A save to the file the document came from.
const SAVED: &str = "save-in-place"; // ui-text-exempt: a trace event name, never displayed
/// Baseline origin of the fixture's one line, in PDF user space.
const ORIGIN: (f64, f64) = (72.0, 700.0);

/// See the module documentation.
pub struct ScaledTextShowsAndTakesItsDrawnSize;

impl Check for ScaledTextShowsAndTakesItsDrawnSize {
    fn name(&self) -> &'static str {
        "scaled_text_shows_and_takes_its_drawn_size"
    }

    fn defect(&self) -> &'static str {
        "CAD text drawn at 13 pt reads `1.0 pt` in Properties, and typing a size writes it as \
         the Tf operand, so the text is redrawn at that many times its scale and vanishes \
         off the page"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        match drive(ctx, &mut report) {
            Ok(Some(failure)) => report.fail(failure),
            Ok(None) => report.pass(),
            Err(why) => report.from_error(&why),
        }
    }
}

/// A one-page Letter PDF with one Helvetica line set `1 Tf` under an
/// 18-point text matrix.
fn fixture() -> Vec<u8> {
    let content = format!(
        "BT /F1 1 Tf {DRAWN_PT} 0 0 {DRAWN_PT} {} {} Tm (Scaled label text) Tj ET",
        ORIGIN.0, ORIGIN.1
    );
    crate::fixture::pdf_of(&[
        b"<< /Type /Catalog /Pages 2 0 R >>".to_vec(),
        b"<< /Type /Pages /Kids [3 0 R] /Count 1 >>".to_vec(),
        b"<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] \
          /Resources << /Font << /F1 5 0 R >> >> /Contents 4 0 R >>"
            .to_vec(),
        format!(
            "<< /Length {} >>\nstream\n{content}\nendstream",
            content.len()
        )
        .into_bytes(),
        b"<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica /Encoding /WinAnsiEncoding >>"
            .to_vec(),
    ])
}

/// The panel's read of a run: its drawn size and its characters.
#[derive(Clone, Debug, PartialEq)]
struct Read {
    size: f64,
    chars: usize,
    text_hash: String,
}

/// The last resolved panel read traced after `mark`.
fn synced(session: &Session, mark: usize) -> Result<Option<Read>> {
    let trace = session.trace()?;
    Ok(trace
        .events(SYNCED)
        .filter(|l| l.lineno > mark && l.get("resolved") == Some("1"))
        .last()
        .and_then(|l| {
            Some(Read {
                size: l.get("size")?.parse().ok()?,
                chars: l.get_usize("chars")?,
                text_hash: l.get("text_hash")?.to_owned(),
            })
        }))
}

/// What is driven: the file, a point on the run, its page size, and the size
/// the run must read before it is touched (known only for the built-in page).
struct Subject {
    pdf: std::path::PathBuf,
    at: DocPoint,
    page: PageGeometry,
    drawn: Option<f64>,
}

/// The operator's drawing when `--pdf` is passed (copied first, since it is
/// his and the check edits it), else the built-in page.
fn subject(ctx: &CheckContext) -> Result<Subject> {
    let pdf = ctx.out("scaled-text-size.pdf");
    let Some(source) = ctx.pdf.clone() else {
        std::fs::write(&pdf, fixture())
            .map_err(|e| Error::new(format!("cannot write {}: {e}", pdf.display())))?;
        return Ok(Subject {
            pdf,
            at: DocPoint::new(0, ORIGIN.0 + 30.0, ORIGIN.1 + 5.0),
            page: PageGeometry {
                width_pt: 612.0,
                height_pt: 792.0,
            },
            drawn: Some(DRAWN_PT),
        });
    };
    let at = ctx.target.ok_or_else(|| {
        Error::new("--pdf without --doc-point. Pass PAGE,X,Y in PDF user space on a text run.")
    })?;
    let page = match ctx.page_size {
        Some((w, h)) => PageGeometry {
            width_pt: w,
            height_pt: h,
        },
        None => crate::fixture::page_geometry(&source).ok_or_else(|| {
            Error::new(format!(
                "cannot read a page size from {}. Pass --page-size WxH.",
                source.display()
            ))
        })?,
    };
    std::fs::copy(&source, &pdf)
        .map_err(|e| Error::new(format!("copying {}: {e}", source.display())))?;
    Ok(Subject {
        pdf,
        at,
        page,
        drawn: None,
    })
}

/// Poll up to `ceiling_ms` for `ready` to answer `Some`.
fn wait<T>(ceiling_ms: u128, mut ready: impl FnMut() -> Result<Option<T>>) -> Result<Option<T>> {
    let started = std::time::Instant::now();
    loop {
        if let Some(v) = ready()? {
            return Ok(Some(v));
        }
        if started.elapsed().as_millis() > ceiling_ms {
            return Ok(None);
        }
    }
}

/// Launch the binary on the subject's file.
fn launch(
    ctx: &CheckContext,
    report: &mut CheckReport,
    subject: &Subject,
) -> Result<(Session, ScriptedPointer)> {
    let exe = ctx.resolve_exe().ok_or_else(|| {
        Error::new(format!(
            "no binary to drive. Pass --exe, or build the profile's default at {}.",
            ctx.profile.default_exe
        ))
    })?;
    let viewport_env = ctx.profile.viewport_env.ok_or_else(|| {
        Error::new("the profile has no viewport variable to place the window off the desktop.")
    })?;
    let mut spec = LaunchSpec::new(&exe, ctx.out("scaled-text-size.trace.txt"));
    spec.pdf = Some(subject.pdf.clone());
    spec.env.push((
        ctx.profile.diag_env.0.to_owned(),
        ctx.profile.diag_env.1.to_owned(),
    ));
    spec.env
        .push((SHELL_DIAG_ENV.0.to_owned(), SHELL_DIAG_ENV.1.to_owned()));
    spec.env
        .push(("PDFCER_DIAG_INVOKE".to_owned(), INVOKE.to_owned()));
    spec.env
        .push((viewport_env.to_owned(), OFFSCREEN.to_owned()));
    spec.place = false;
    spec.allow_stale = ctx.allow_stale;
    spec.source_root = ctx.source_root.clone();
    let pointer = ScriptedPointer::attach(&mut spec, ctx.out("scaled-text-size.pointer.txt"))?;
    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    report.artifact(session.trace_path().to_path_buf());
    report.artifact(pointer.path().to_path_buf());
    session.settle(40);
    Ok((session, pointer))
}

/// Click the line as an object and answer the panel's read.
fn click_text(
    ctx: &CheckContext,
    session: &Session,
    pointer: &ScriptedPointer,
    subject: &Subject,
) -> Result<Option<Read>> {
    let vocab = &ctx.profile.vocab;
    let ui_rect = vocab
        .ui_rect_event
        .ok_or_else(|| Error::new("the profile declares no ui-rect trace event."))?;
    click_mode_segment(session, pointer, ui_rect, "edit")?;
    session.settle(20);
    arm_text_tool(ctx, session, pointer)?;
    let mapping =
        CanvasMapping::from_trace(&session.trace()?, vocab, subject.page, subject.at.page)?;
    let mark = session.trace()?.mark();
    pointer.click(session, mapping.doc_to_window(subject.at)?)?;
    wait(10_000, || {
        session.settle(4);
        synced(session, mark)
    })
}

fn drive(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    let subject = subject(ctx)?;
    let (session, pointer) = launch(ctx, report, &subject)?;
    let Some(read) = click_text(ctx, &session, &pointer, &subject)? else {
        return Err(Error::new(format!(
            "a click on the line produced no resolved `{SYNCED}` line, so the panel never \
             read the run. SKIPPED: the aim or the selection failed before this check's \
             subject. Trace: {}.",
            session.trace_path().display()
        )));
    };
    if let Some(drawn) = subject.drawn.filter(|d| (read.size - d).abs() > 0.05) {
        return Ok(Some(format!(
            "★ Properties reads `{:.2}` pt for text drawn at {drawn} pt (`1 Tf` under a \
             {drawn}-point text matrix). `canvas::textedit::pin::inspect` must report \
             `drawn_size`, the Tf operand times `em_scale`. Trace: {}.",
            read.size,
            session.trace_path().display()
        )));
    }
    report.note(format!(
        "★ Properties read {:.2} pt and {} characters",
        read.size, read.chars
    ));
    let wanted = if (read.size - WANTED_PT).abs() < 0.5 {
        OTHER_PT
    } else {
        WANTED_PT
    };
    let baseline = squares(&session.trace()?, 0).ok_or_else(|| {
        Error::new(format!(
            "no finished page render before the size change, so the page's own square count is unknown. Trace: {}.",
            session.trace_path().display()
        ))
    })?;
    let mark = session.trace()?.mark();
    let failure = type_size(ctx, &session, &pointer, report, &read, wanted)?;
    let failure = match failure {
        Some(f) => Some(f),
        None => redrawn(&session, report, mark, baseline)?,
    };
    let failure = match failure {
        Some(f) => Some(f),
        None => reopen_as_text(ctx, &session, &pointer, &subject, report, &read, baseline)?,
    };
    save(&session, &pointer, report)?;
    Ok(failure)
}

/// The most glyphs any finished render after `mark` drew as `.notdef`.
fn squares(trace: &crate::trace::Trace, mark: usize) -> Option<usize> {
    trace
        .lines
        .iter()
        .filter(|l| l.lineno > mark && RENDERED.contains(&l.event.as_str()))
        .filter_map(|l| l.get_usize("notdef"))
        .max()
}

/// The page re-rendered after the size change with no more `.notdef` glyphs
/// than it had before it.
fn redrawn(
    session: &Session,
    report: &mut CheckReport,
    mark: usize,
    baseline: usize,
) -> Result<Option<String>> {
    let seen = wait(20_000, || {
        session.settle(4);
        Ok(squares(&session.trace()?, mark))
    })?;
    // A finer tier may follow the first raster; read every one that landed.
    session.settle(30);
    let Some(after) = seen.and(squares(&session.trace()?, mark)) else {
        return Err(Error::new(format!(
            "no finished page render after the size change, so the redraw is unmeasured. SKIPPED. Trace: {}.",
            session.trace_path().display()
        )));
    };
    if after > baseline {
        return Ok(Some(format!(
            "★★ the page redrew with {after} glyphs drawn as `.notdef` squares where it had {baseline} before the size change. Trace: {}.",
            session.trace_path().display()
        )));
    }
    report.note(format!(
        "★★ the page redrew after the size change with {after} `.notdef` glyphs ({baseline} before)"
    ));
    Ok(None)
}

/// Save to the check's copy with Ctrl+S, so the edited file can be rendered
/// outside the program; a failed save is noted, not judged.
fn save(session: &Session, pointer: &ScriptedPointer, report: &mut CheckReport) -> Result<()> {
    let mark = session.trace()?.mark();
    pointer.key(session, None, "Escape", None)?;
    session.settle(8);
    pointer.key(session, None, "S", Some("ctrl"))?;
    let saved = wait(20_000, || {
        session.settle(4);
        Ok(session.trace()?.last_after(SAVED, mark).cloned())
    })?;
    match saved {
        Some(line) => report.note(format!("saved: `{}`", line.raw)),
        None => report.note(format!("Ctrl+S wrote no `{SAVED}` line")),
    };
    Ok(())
}

/// Arm the Text tool from the ribbon and click the run again: the caret must
/// open every character the run had before its size changed. A run whose codes
/// were re-encoded opens other characters, drawn as squares.
fn reopen_as_text(
    ctx: &CheckContext,
    session: &Session,
    pointer: &ScriptedPointer,
    subject: &Subject,
    report: &mut CheckReport,
    before: &Read,
    baseline: usize,
) -> Result<Option<String>> {
    // The restyle closed the open line and left the Text tool armed, so this
    // click opens the line afresh from the restyled page.
    let mapping = CanvasMapping::from_trace(
        &session.trace()?,
        &ctx.profile.vocab,
        subject.page,
        subject.at.page,
    )?;
    let mark = session.trace()?.mark();
    pointer.click(session, mapping.doc_to_window(subject.at)?)?;
    let caret: Option<TraceLine> = wait(10_000, || {
        session.settle(4);
        Ok(session.trace()?.last_after(CARET, mark).cloned())
    })?;
    let Some(caret) = caret else {
        return Err(Error::new(format!(
            "a Text-tool click on the restyled run opened no `{CARET}` line. SKIPPED: the \
             tool or the aim failed. Trace: {}.",
            session.trace_path().display()
        )));
    };
    match caret.get_usize("len") {
        Some(len) if len == before.chars => {
            report.note(format!(
                "★★★ the Text tool opened the restyled run's {len} characters: `{}`",
                caret.raw
            ));
            shaped_whole(session, report, mark, baseline)
        }
        other => Ok(Some(format!(
            "★ after the size change the Text tool opened `{}` characters where the run had \
             {}: `{}`. Its codes were re-encoded, which draws as squares. Trace: {}.",
            other.map_or_else(|| "no len".to_owned(), |l| l.to_string()),
            before.chars,
            caret.raw,
            session.trace_path().display()
        ))),
    }
}

/// The opened line's own-font layout pairs every glyph with its character;
/// a glyph whose code reads back as another draws as a square. A line the
/// engine would not lay out is drawn in the shell's stand-in font, so its
/// squares can only come from the page render beneath it.
fn shaped_whole(
    session: &Session,
    report: &mut CheckReport,
    mark: usize,
    baseline: usize,
) -> Result<Option<String>> {
    let shaped: Option<TraceLine> = wait(10_000, || {
        session.settle(4);
        Ok(session.trace()?.last_after(SHAPED, mark).cloned())
    })?;
    let Some(shaped) = shaped else {
        return Err(Error::new(format!(
            "the opened line was never laid out (no `{SHAPED}`). SKIPPED. Trace: {}.",
            session.trace_path().display()
        )));
    };
    if shaped.get_usize("shaped") == Some(0) {
        return stand_in(session, report, mark, baseline, &shaped.raw);
    }
    match shaped.get_usize("unread") {
        Some(0) => {
            report.note(format!(
                "★★★ every glyph of the opened line reads back as its character: `{}`",
                shaped.raw
            ));
            Ok(None)
        }
        n => Ok(Some(format!(
            "★★★ the opened line draws {} glyphs whose codes read back as other characters — squares on the page: `{}`. Trace: {}.",
            n.map_or_else(|| "an unknown number of".to_owned(), |n| n.to_string()),
            shaped.raw,
            session.trace_path().display()
        ))),
    }
}

/// The opened line fell back to the stand-in font: say why, and require every
/// page render so far to draw no more `.notdef` glyphs than the page did.
fn stand_in(
    session: &Session,
    report: &mut CheckReport,
    mark: usize,
    baseline: usize,
    shaped: &str,
) -> Result<Option<String>> {
    let trace = session.trace()?;
    let why = trace
        .last_after(FALLBACK, mark)
        .map_or_else(|| "no fallback line".to_owned(), |l| l.raw.clone());
    let worst = squares(&trace, 0).unwrap_or(0);
    if worst > baseline {
        return Ok(Some(format!(
            "★★★ the opened line is in the stand-in font and the page beneath it drew {worst} `.notdef` glyphs where it had {baseline}: `{shaped}`. Trace: {}.",
            session.trace_path().display()
        )));
    }
    report.note(format!(
        "★★★ the opened line is not laid out in its own font (`{shaped}`; `{why}`), so it draws in the stand-in font; every page render drew at most {worst} `.notdef` glyphs ({baseline} before)"
    ));
    Ok(None)
}

/// Click the Edit tab when the Text tool is not on the band, then the tool.
fn arm_text_tool(ctx: &CheckContext, session: &Session, pointer: &ScriptedPointer) -> Result<()> {
    let ui_rect = ctx.profile.vocab.ui_rect_event.unwrap_or_default();
    if declared_in(&session.trace()?, ui_rect, TEXT_TOOL).is_none()
        && let Some((tab, viewport)) = declared_in(&session.trace()?, ui_rect, EDIT_TAB)
    {
        pointer.click_in(session, viewport.as_deref(), WindowPoint::centre_of(tab))?;
        session.settle(16);
    }
    let Some((tool, viewport)) = declared_in(&session.trace()?, ui_rect, TEXT_TOOL) else {
        return Err(Error::new(format!(
            "no `{TEXT_TOOL}` region on screen, so the Text tool cannot be armed. Trace: {}.",
            session.trace_path().display()
        )));
    };
    pointer.click_in(session, viewport.as_deref(), WindowPoint::centre_of(tool))?;
    session.settle(16);
    Ok(())
}

/// Type `wanted` into the size field; the restyle's verdict line, or a
/// failure sentence.
fn commit_typed_size(
    ctx: &CheckContext,
    session: &Session,
    pointer: &ScriptedPointer,
    mark: usize,
    wanted: f64,
) -> Result<std::result::Result<String, String>> {
    let ui_rect = ctx.profile.vocab.ui_rect_event.unwrap_or_default();
    let Some((field, viewport)) = declared_in(&session.trace()?, ui_rect, SIZE_REGION) else {
        return Err(Error::new(format!(
            "no `{SIZE_REGION}` region on screen, so there is nothing to type into. \
             Trace: {}.",
            session.trace_path().display()
        )));
    };
    let vp = viewport.as_deref();
    pointer.click_in(session, vp, WindowPoint::centre_of(field))?;
    session.settle(12);
    pointer.key(session, vp, "A", Some("ctrl"))?;
    pointer.type_text(session, vp, &format!("{wanted:.0}"))?;
    session.settle(8);
    pointer.key(session, vp, "Enter", None)?;
    let verdict = wait(30_000, || {
        session.settle(4);
        let t = session.trace()?;
        Ok(t.last_after(DECLINED_EVENT, mark)
            .or_else(|| t.last_after(APPLIED_EVENT, mark))
            .or_else(|| t.last_after(DISCLOSED_EVENT, mark))
            .map(|l| l.raw.clone()))
    })?;
    Ok(match verdict {
        Some(v) if v.contains(APPLIED_EVENT) || v.contains(DISCLOSED_EVENT) => Ok(v),
        Some(v) => Err(format!("the size change was declined: `{v}`")),
        None => Err(format!(
            "{wanted:.0} was typed and Enter pressed, and no `{APPLIED_EVENT}` or \
             `{DECLINED_EVENT}` line followed. Trace: {}.",
            session.trace_path().display()
        )),
    })
}

/// Type `wanted` and require the panel's re-read to match it and to hold
/// the same characters as `before`.
fn type_size(
    ctx: &CheckContext,
    session: &Session,
    pointer: &ScriptedPointer,
    report: &mut CheckReport,
    before: &Read,
    wanted: f64,
) -> Result<Option<String>> {
    let mark = session.trace()?.mark();
    let verdict = match commit_typed_size(ctx, session, pointer, mark, wanted)? {
        Ok(v) => v,
        Err(failure) => return Ok(Some(failure)),
    };
    let after = wait(15_000, || {
        session.settle(4);
        synced(session, mark)
    })?;
    let Some(after) = after else {
        return Ok(Some(format!(
            "the size applied (`{verdict}`) and the panel never re-read the run, so the \
             result is unmeasured. Trace: {}.",
            session.trace_path().display()
        )));
    };
    // The I-beam restyles the word at the caret, and the panel re-reads the
    // run's first glyph, which on a multi-word line is another word. The
    // engine's own `Tf size was changed from A to B` note measures the restyled word.
    let written = session
        .trace()?
        .last_after(DISCLOSED_EVENT, mark)
        .and_then(|l| tf_change(&l.raw));
    let drawn_now = written.map_or(after.size, |(from, to)| before.size * to / from);
    if (drawn_now - wanted).abs() > 0.05 {
        return Ok(Some(format!(
            "★ {wanted:.0} was typed and the restyled text now draws at `{:.2}` pt. A reading of {:.0} is \
             the typed size written as the Tf operand under the {:.0}-point matrix, which \
             redraws the text off the page. `StyleChange::stamp` must call `FormatRequest::drawn_size`, which divides by the \
             run's `em_scale`. Trace: {}.",
            drawn_now,
            wanted * before.size,
            before.size,
            session.trace_path().display()
        )));
    }
    if (after.chars, &after.text_hash) != (before.chars, &before.text_hash) {
        return Ok(Some(format!(
            "★★ the run reads {wanted:.0} pt and its characters changed: {} characters hashed \
             `{}` before, {} hashed `{}` after. A size change must not re-encode the run; \
             re-encoded codes draw as squares. Trace: {}.",
            before.chars,
            before.text_hash,
            after.chars,
            after.text_hash,
            session.trace_path().display()
        )));
    }
    report.note(format!(
        "★★ typed {wanted:.0}; the restyled text now draws at {drawn_now:.2} pt ({}) and the run keeps its {} characters",
        written.map_or("the panel's re-read", |_| "the engine's Tf note"),
        after.chars
    ));
    Ok(None)
}

/// The `(from, to)` operands of the engine's `Tf size was changed from A to B pt`
/// disclosure.
fn tf_change(raw: &str) -> Option<(f64, f64)> {
    let rest = raw.split("Tf").nth(1)?.split("changed from ").nth(1)?;
    let mut words = rest.split_whitespace();
    let from = words.next()?.parse::<f64>().ok()?;
    let to = words
        .nth(1)?
        .trim_end_matches(|c: char| !c.is_ascii_digit())
        .parse::<f64>()
        .ok()?;
    (from > 0.0).then_some((from, to))
}
