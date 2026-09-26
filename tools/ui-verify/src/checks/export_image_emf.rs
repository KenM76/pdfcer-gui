//! `export_image_writes_a_metafile` — the EMF radio is reachable, the press
//! writes a file, and the bytes are a metafile that agrees with itself.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/export_image_emf.md`.

use crate::checks::driving::{
    SHELL_DIAG_ENV, declared, declared_names, declared_or_in_overflow, frame_of, list,
};
use crate::checks::{Check, CheckContext};
use crate::error::{Error, Result};
use crate::input::Driver;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;

/// The mode this runs in.
///
/// **Read**, as `export_dxf` runs in, and for the reason that check states: an
/// export reads the document and writes elsewhere, so there is no mode in which
/// it should be refused. If a capability gate ever creeps onto this command,
/// this is where it shows up.
const MODE: &str = "read";
/// The window's own region.
const WINDOW: &str = "dialog:export-image";
/// The EMF radio's region — `dialogs::export_image::region_for_format`.
const EMF_RADIO: &str = "export-image.format.emf";
/// The Export button.
const EXPORT: &str = "export-image.export";
/// The trace the window emits when it opens.
const OPENED: &str = "export-image-open";
/// The trace the window emits on the press, carrying the plan.
const REQUESTED: &str = "export-image-requested";
/// The value [`REQUESTED`] carries in its `format=` field when the EMF radio
/// is the one selected.
///
/// ★★★ **Lowercase, and sourced rather than chosen.** It is the FILE TOKEN,
/// `pdfcer_gui::app::prefs::exporting::image_format_key(ImageFormat::Emf)` —
/// the same string the preferences file stores, which is exactly why the
/// emitter uses it: a check reading this trace and a check reading
/// `preferences.txt` then cannot disagree about what EMF is called.
///
/// ⚠ **This constant was `"Emf"` until 2026-09-14 and the check was RIGHT to
/// say so when it was written.** On 2026-09-04 the emitter read
/// `format={:?}` — `Debug` on `ImageFormat`, which prints `Emf`. On
/// 2026-09-13 commit `28f5389` replaced that with the token, correctly and
/// for this project's own standing reason (*never `Debug`-format a field a
/// machine reads*), and **broke the only machine reading the field without
/// anything going red** — because this check had never been run. The first
/// sweep to run it, on 2026-09-14, duly reported *"the radio drew and did not
/// bind"* while quoting `format=emf` in the same sentence.
///
/// ★ The lesson generalises past this file: **changing a trace field's
/// spelling is an edit to every reader of that field**, and an unrun check is
/// a reader that cannot object to it.
const EMF_KEY: &str = "emf";
/// What the `format=` field held *before* 2026-09-13, i.e. `Debug` on the
/// enum.
///
/// ★★ Kept as a SEPARATE diagnosis rather than folded into the comparison.
/// Seeing this spelling again would mean the emitter had regressed to
/// `{:?}` — a real defect, with a real one-line remedy, and **not remotely
/// the same thing as a radio that failed to bind**. An assertion that lumps
/// the two together produces the message this check produced today: a
/// confident accusation against the program, quoting the evidence that
/// clears it.
const EMF_KEY_DEBUG_SPELLING: &str = "Emf";
/// The trace the apply arm emits per file written.
const WROTE: &str = "export-image";
/// The environment seam that answers the save dialog.
const SAVE_PATH_ENV: &str = "PDFCER_DIAG_SAVE_PATH"; // ui-text-exempt: an environment variable name

/// `ENHMETA_SIGNATURE` — the four bytes `' '`, `'E'`, `'M'`, `'F'` read as a
/// little-endian `u32`, at offset 40 of every enhanced metafile.
const ENHMETA_SIGNATURE: u32 = 0x464D_4520;
/// `EMR_HEADER` — the record type of the first record, at offset 0.
const EMR_HEADER: u32 = 1;
/// Offset of `dSignature` in `ENHMETAHEADER`.
const OFF_SIGNATURE: usize = 40;
/// Offset of `nBytes` — the metafile's total size, back-patched by the writer.
const OFF_BYTES: usize = 48;
/// Offset of `nRecords`.
const OFF_RECORDS: usize = 52;
/// `sizeof(ENHMETAHEADER)` as this writer emits it; a file shorter than this
/// has no header to read at all.
const HEADER_LEN: usize = 108;

/// See the module documentation.
pub struct ExportImageWritesAMetafile;

impl Check for ExportImageWritesAMetafile {
    fn name(&self) -> &'static str {
        "export_image_writes_a_metafile"
    }

    fn defect(&self) -> &'static str {
        "File > Export image offers EMF and the radio does nothing — or the press writes no \
         file, or writes one that is not a metafile, or writes a metafile whose own header \
         disagrees with its length, so an operator hands a drawing to LibreOffice or to \
         Word's Paste Special and gets an empty frame"
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

/// Read a little-endian `u32` at `offset`, or `None` if the slice is short.
fn u32_at(bytes: &[u8], offset: usize) -> Option<u32> {
    bytes
        .get(offset..offset + 4)
        .map(|b| u32::from_le_bytes([b[0], b[1], b[2], b[3]]))
}

#[allow(clippy::too_many_lines)]
fn drive(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    let exe = ctx.resolve_exe().ok_or_else(|| {
        Error::new(format!(
            "no binary to drive. Pass --exe, or build the profile's default at {}.",
            ctx.profile.default_exe
        ))
    })?;
    let pdf = ctx
        .pdf
        .clone()
        .ok_or_else(|| Error::new("no fixture document. Pass --pdf."))?;
    if !ctx.allow_input {
        return Err(Error::new(
            "input is disabled (--no-input). This check clicks a mode segment, a ribbon tab, \
             a ribbon control, a format radio and a button. Reported as SKIPPED rather than \
             passed: a check that did not run has learned nothing.",
        ));
    }
    let ui_rect = ctx.profile.vocab.ui_rect_event.ok_or_else(|| {
        Error::new(format!(
            "the `{}` profile declares no ui-rect trace event, so the application cannot say \
             where its controls are.",
            ctx.profile.name
        ))
    })?;

    // ★ Removed before the run rather than merely named. A metafile left by an
    // earlier run would let a build that writes NOTHING pass every assertion
    // below — `a_driven_check_that_does_not_establish_its_preconditions_measures_the_previous_run`
    // in the Rust RAG, and `export_dxf` clears its own target for the same
    // reason.
    let target = ctx.out("export_image.emf");
    let _ = std::fs::remove_file(&target);
    if target.exists() {
        return Err(Error::new(format!(
            "cannot clear {} before the run, so a file written by an earlier run could be \
             mistaken for this one's.",
            target.display()
        )));
    }

    let mut spec = LaunchSpec::new(&exe, ctx.out("export_image_emf.trace.txt"));
    spec.pdf = Some(pdf);
    spec.env.push((
        ctx.profile.diag_env.0.to_owned(),
        ctx.profile.diag_env.1.to_owned(),
    ));
    spec.env
        .push((SHELL_DIAG_ENV.0.to_owned(), SHELL_DIAG_ENV.1.to_owned()));
    spec.env
        .push((SAVE_PATH_ENV.to_owned(), target.display().to_string()));
    spec.allow_stale = ctx.allow_stale;
    spec.source_root = ctx.source_root.clone();

    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    report.note(format!(
        "launched {} as pid {}",
        exe.display(),
        session.pid()
    ));
    report.artifact(session.trace_path().to_path_buf());
    session.settle(40);
    let driver = Driver::new(session.window());

    crate::checks::driving::click_mode_segment(&session, &driver, ui_rect, MODE)?;

    // --- 1: the File tab ---------------------------------------------------
    let trace = session.trace()?;
    let tab = declared(&trace, ui_rect, "ribbon.tab.file").ok_or_else(|| {
        Error::new(format!(
            "no `ribbon.tab.file` region in {MODE}. Tabs declared: {}.",
            list(&declared_names(&trace, ui_rect, "ribbon.tab."))
        ))
    })?;
    driver.click_at(session.frame()?.declared_center(tab))?;
    session.settle(14);

    // --- 2: open the window ------------------------------------------------
    let Some(item) =
        declared_or_in_overflow(&session, &driver, ui_rect, "ribbon.item.file.export_image")?
    else {
        return Ok(Some(format!(
            "the File tab declares no `ribbon.item.file.export_image`, on the band or in the \
             overflow. Items declared: {}.",
            list(&declared_names(
                &session.trace()?,
                ui_rect,
                "ribbon.item.file."
            ))
        )));
    };
    driver.click_at(session.frame()?.declared_center(item))?;
    session.settle(20);

    let trace = session.trace()?;
    if declared(&trace, ui_rect, WINDOW).is_none() {
        let unimplemented = trace
            .events("command-unimplemented")
            .any(|l| l.get("id") == Some("file.export_image"));
        return Ok(Some(if unimplemented {
            "`file.export_image` was clicked and traced `command-unimplemented` — it is still \
             scaffolded."
                .to_owned()
        } else {
            format!(
                "`file.export_image` was clicked and no `{WINDOW}` region appeared. The arm \
                 ran and built no window — or declined it. In {MODE} it must not decline: an \
                 export reads the document and writes elsewhere."
            )
        }));
    }
    if let Some(opened) = trace.last(OPENED) {
        report.note(format!("window opened: `{}`", opened.raw));
    }

    // --- 3: ★★ the EMF radio EXISTS ----------------------------------------
    //
    // The first thing this check is for. `ImageFormat::ALL` is four long in a
    // unit test whatever the window draws; whether a fourth radio reached the
    // screen — inside the window's height, not clipped — is a question only a
    // laid-out frame answers.
    let Some(radio) = declared(&trace, ui_rect, EMF_RADIO) else {
        return Ok(Some(format!(
            "the window declares no `{EMF_RADIO}` region, so EMF is not on offer. Format \
             regions declared: {}. LibreOffice 24 and Word's Paste Special have no other \
             vector route, so without this radio the operator's only vector answer is one \
             those programs refuse.",
            list(&declared_names(&trace, ui_rect, "export-image.format"))
        )));
    };
    driver.click_at(frame_of(&session, &trace, ui_rect, EMF_RADIO)?.declared_center(radio))?;
    session.settle(14);

    // --- 4: export ---------------------------------------------------------
    let trace = session.trace()?;
    let Some(button) = declared(&trace, ui_rect, EXPORT) else {
        return Ok(Some(format!(
            "the window declares no `{EXPORT}` region, so there is nothing to press."
        )));
    };
    driver.click_at(frame_of(&session, &trace, ui_rect, EXPORT)?.declared_center(button))?;
    session.settle(40);

    // --- 5: ★★ the PRESS carried the format the radio selected -------------
    //
    // Link 2 of the module header. A build whose radio drew and did not bind
    // its value would trace `format=png` here, write PNG bytes, and name the
    // file `.emf` — which opens in nothing and looks like a broken metafile
    // rather than like a broken radio.
    //
    // ⚠ The field is compared against [`EMF_KEY`], the token the emitter
    // reduces the enum to, NOT against the enum's `Debug` spelling. Read that
    // constant's documentation before touching this: getting it backwards is
    // how this check spent its first-ever run accusing a radio that worked.
    let trace = session.trace()?;
    let Some(requested) = trace.last(REQUESTED) else {
        return Ok(Some(format!(
            "Export was pressed and no `{REQUESTED}` line followed, so the window never \
             raised an action. The button drew and did nothing."
        )));
    };
    report.note(format!("requested: `{}`", requested.raw));
    let chosen = requested.get("format");
    if chosen == Some(EMF_KEY_DEBUG_SPELLING) {
        return Ok(Some(format!(
            "the plan says format={EMF_KEY_DEBUG_SPELLING} — the enum's `Debug` spelling, \
             which means the emitter in `dialogs::export_image` has gone back to \
             `format={{:?}}` instead of `prefs::exporting::image_format_key`. The RADIO IS \
             FINE and the export will be a metafile; what is broken is that the trace and \
             `preferences.txt` now name the same format two different ways, so a reader of \
             one cannot be checked against a reader of the other. This is the state that \
             existed until 2026-09-13 and it is worth one line to restore: `{}`",
            requested.raw
        )));
    }
    if chosen != Some(EMF_KEY) {
        return Ok(Some(format!(
            "★ the EMF radio was clicked and the plan says format={}. The radio drew and did \
             not bind, so the file about to be written is not the format that was chosen — \
             and it will be named `.emf` regardless: `{}`",
            chosen.unwrap_or("<absent>"),
            requested.raw
        )));
    }

    let Some(wrote) = trace.last(WROTE) else {
        let refused = trace.last("export-image-refused");
        let declined = trace.last("export-image-declined");
        let cancelled = trace.last("export-image-cancelled");
        let render_failed = trace.last("export-image-render-failed");
        let encode_failed = trace.last("export-image-encode-failed");
        let write_failed = trace.last("export-image-write-failed");
        let show = |l: Option<&crate::trace::TraceLine>| {
            l.map_or_else(|| "none".to_owned(), |l| l.raw.clone())
        };
        return Ok(Some(format!(
            "Export was pressed and no `{WROTE}` line followed. refused={} declined={} \
             cancelled={} render-failed={} encode-failed={} write-failed={}",
            show(refused),
            show(declined),
            show(cancelled),
            show(render_failed),
            show(encode_failed),
            show(write_failed),
        )));
    };
    report.note(format!("wrote: `{}`", wrote.raw));

    // --- 6: ★ the file is on disk ------------------------------------------
    if !target.exists() {
        return Ok(Some(format!(
            "the shell traced a successful export and {} does not exist. The disclosure and \
             the disk disagree, which is the worst of the failures here: the operator has \
             been told a file was written.",
            target.display()
        )));
    }
    let bytes = std::fs::read(&target).map_err(|e| {
        Error::new(format!(
            "the export wrote {} and it cannot be read back: {e}",
            target.display()
        ))
    })?;
    if bytes.len() < HEADER_LEN {
        return Ok(Some(format!(
            "{} is {} bytes — shorter than an enhanced metafile's {HEADER_LEN}-byte header, \
             so there is nothing for a reader to parse.",
            target.display(),
            bytes.len()
        )));
    }

    // --- 7: ★★★ the bytes ARE a metafile ------------------------------------
    let record_type = u32_at(&bytes, 0).unwrap_or_default();
    let signature = u32_at(&bytes, OFF_SIGNATURE).unwrap_or_default();
    if record_type != EMR_HEADER || signature != ENHMETA_SIGNATURE {
        let looks_like_png = bytes.starts_with(b"\x89PNG");
        let looks_like_svg = bytes.starts_with(b"<") || bytes.starts_with(b"<?xml");
        return Ok(Some(format!(
            "★ {} is not an enhanced metafile: the first record type is {record_type} (want \
             {EMR_HEADER}) and the signature at offset {OFF_SIGNATURE} is {signature:#010x} \
             (want {ENHMETA_SIGNATURE:#010x}, which is ' EMF'). It looks like a PNG: {}. It \
             looks like an SVG: {}. Either the radio did not bind or the writer was routed \
             through the wrong encoder — and the file has the right extension either way, so \
             nothing but a reader would have noticed.",
            target.display(),
            looks_like_png,
            looks_like_svg
        )));
    }

    // --- 8: ★★★ the metafile agrees with ITSELF -----------------------------
    //
    // `nBytes` is a placeholder back-patched after every record is written. A
    // value that disagrees with the file's length means the header and the body
    // came from different states — a truncated write, or a header copied from
    // the wrong buffer. GDI refuses such a file, so the operator's paste is
    // empty and the file on disk looks perfectly plausible.
    let declared_bytes = u32_at(&bytes, OFF_BYTES).unwrap_or_default() as usize;
    if declared_bytes != bytes.len() {
        return Ok(Some(format!(
            "★ the metafile's own header says it is {declared_bytes} bytes and the file is \
             {} bytes. The header was back-patched from a different state than the body, so \
             GDI will refuse it and the paste will be empty — while the file keeps its \
             extension, its signature and a plausible size.",
            bytes.len()
        )));
    }
    let records = u32_at(&bytes, OFF_RECORDS).unwrap_or_default();
    if records < 2 {
        return Ok(Some(format!(
            "the metafile declares {records} record(s). Even an empty page is a header plus \
             an EOF record, so a count below two means the writer emitted a header over \
             nothing."
        )));
    }
    report.note(format!(
        "{} bytes on disk, self-consistent header, {records} records",
        bytes.len()
    ));

    // --- 9: ★★ the disk agrees with what the shell SAID --------------------
    //
    // The cross-check that makes this worth more than a smoke test, and the
    // same shape as `export_dxf`'s count comparison: two values that are
    // supposed to describe one thing are exactly the pair a refactor separates.
    let Some(reported) = wrote.get("bytes").and_then(|v| v.parse::<usize>().ok()) else {
        return Err(Error::new(format!(
            "the `{WROTE}` line carries no readable `bytes=` count, so the file cannot be \
             checked against it: `{}`",
            wrote.raw
        )));
    };
    if reported != bytes.len() {
        return Ok(Some(format!(
            "★ the shell reported writing {reported} bytes and {} holds {}. The disclosure \
             describes an export the disk does not — which an operator has no way to notice, \
             because both numbers look reasonable and only one of them is in the file they \
             hand to somebody else.",
            target.display(),
            bytes.len()
        )));
    }
    report.note(format!("{reported} bytes reported and {reported} on disk"));
    Ok(None)
}
