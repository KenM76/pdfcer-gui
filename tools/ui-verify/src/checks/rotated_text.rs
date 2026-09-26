//! `rotated_text_selects_and_copies_as_one_line` — the driven proof for the
//! operator's 2026-08-26 report about vertical text.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/rotated_text.md`.

use std::path::PathBuf;

use crate::checks::driving::{self, SHELL_DIAG_ENV};
use crate::checks::{Check, CheckContext};
use crate::coords::{CanvasMapping, DocPoint, PageGeometry};
use crate::error::{Error, Result};
use crate::input::Driver;
use crate::launch::LaunchSpec;
use crate::launch::Session;
use crate::report::CheckReport;

/// This repository's rotated-text page, relative to the workspace root.
const FIXTURE: &str = "fixtures/rotated-text.pdf";

/// The mode whose primary button sweeps text with no tool armed.
const READ: &str = "read";

/// `canvas-text-selection via=… page=… chars=… quads=…`
const SELECTION_EVENT: &str = "canvas-text-selection";

/// `cursor-custom on shape=… deg=… px=…`
const CURSOR_EVENT: &str = "cursor-custom";

/// The word the sweep runs along, and its exact byte length.
///
/// `UPWARD` is six ASCII capitals, so `chars` — which is a **byte** length, the
/// length of the string a copy would put on the clipboard — is 6 exactly.
const WORD: &str = "UPWARD";

/// The string's first glyph origin in PDF user space, from the fixture's own
/// generator: `0 1 -1 0 100 300 Tm`.
///
/// Named rather than inlined because the two sweep points below are both
/// derived from it, and a reader checking this check against the fixture should
/// find the fixture's own numbers here and not two magic constants.
const ORIGIN: (f64, f64) = (100.0, 300.0);

/// How far to the left of the baseline to aim, in points.
///
/// ★ **Left**, and it is the whole reason this sweep lands. For text turned 90°
/// anticlockwise the glyph's ascender direction is page **−x**, so the ink of a
/// letter at `x = 100` occupies roughly `x ∈ 91..103`. Aiming at the baseline
/// itself would sit on the ink's edge; aiming to the RIGHT would be off the
/// letter entirely, on the side where the extraction's own — axis-aligned, and
/// wrong — glyph box lies.
///
/// Three points is a quarter of the 12 pt size, which is the same offset
/// `canvas::textsel`'s unit tests use to find ink, derived the same way.
const ASCENDER_OFFSET_PT: f64 = 3.0;

/// Where along the string to start and stop, as points from the first origin.
///
/// The string is six capitals at 12 pt Helvetica and runs about 53 pt. Three
/// points in is inside the first letter's front half, so the caret lands
/// **before** `U`; fifty points in is past the last letter's midpoint, so it
/// lands **after** `D`. Both are the ordinary caret rule and both are inside
/// the band, which is what makes this a sweep across the word rather than a
/// sweep that relies on a nearest-line fallback.
const SWEEP_FROM_PT: f64 = 3.0;
/// The far end of the sweep. See [`SWEEP_FROM_PT`].
const SWEEP_TO_PT: f64 = 50.0;

/// The fixture, located from this crate rather than from the working directory.
///
/// `tools/ui-verify/` → up two → the workspace root, exactly as
/// [`crate::checks::ocr`] locates its own.
fn fixture() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join(FIXTURE)
}

/// See the module documentation.
pub struct RotatedTextSelectsAndCopiesAsOneLine;

impl Check for RotatedTextSelectsAndCopiesAsOneLine {
    fn name(&self) -> &'static str {
        "rotated_text_selects_and_copies_as_one_line"
    }

    fn defect(&self) -> &'static str {
        "text placed at 90 degrees on the page selects one letter per line, shades each letter as \
         a separate box in the wrong place, pastes with a newline between every character, and \
         leaves the I-beam upright over it"
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

/// Run the sequence. `Err` is SKIP, `Ok(Some(_))` is FAIL, `Ok(None)` is a pass.
#[allow(clippy::too_many_lines)]
fn drive(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    let vocab = &ctx.profile.vocab;
    let exe = ctx.resolve_exe().ok_or_else(|| {
        Error::new(format!(
            "no binary to drive. Pass --exe, or build the profile's default at {}.",
            ctx.profile.default_exe
        ))
    })?;
    if !ctx.allow_input {
        return Err(Error::new(
            "input is disabled (--no-input). This check is a real drag across a real canvas \
             followed by a real Ctrl+C. Reported as SKIPPED rather than passed: a check that did \
             not run has learned nothing.",
        ));
    }
    let ui_rect = vocab.ui_rect_event.ok_or_else(|| {
        Error::new(format!(
            "the `{}` profile declares no ui-rect trace event, so the application cannot state \
             where its mode segments are and this check has nothing to aim at.",
            ctx.profile.name
        ))
    })?;
    let pdf = fixture();
    if !pdf.exists() {
        return Err(Error::new(format!(
            "the rotated-text fixture is missing at {}. It is committed; regenerate it with \
             `cargo test -p pdfcer-gui regenerate_the_rotated_text_fixture -- --ignored`. This \
             check asserts EXACT character and box counts, so it cannot fall back to --pdf: on an \
             arbitrary drawing there is no `chars={}` to assert.",
            pdf.display(),
            WORD.len()
        )));
    }
    // US Letter, from the fixture's own `/MediaBox [0 0 612 792]`. Read rather
    // than assumed, so a regenerated fixture at another size makes this SKIP
    // instead of mirroring every sweep about the page centre.
    let page: PageGeometry = crate::fixture::page_geometry(&pdf).ok_or_else(|| {
        Error::new(format!(
            "cannot read a page size from {}. Without the page height there is no PDF-y-up to \
             window-y-down flip, and this check refuses to guess one.",
            pdf.display()
        ))
    })?;

    // --- own the clipboard before anything else ----------------------------
    //
    // The same rule `clipboard_text` sets out and for the same reason:
    // assertion 4 is only a statement about THIS run if this process controlled
    // the clipboard going into it. A previous run of this very check leaves
    // `UPWARD` there, which is the one stale value that would make a broken
    // build pass.
    if !crate::sys::clear_clipboard() {
        return Err(Error::new(
            "could not clear the OS clipboard — another process is holding it, or this is not \
             Windows. SKIPPED rather than run: the read at the end could not be attributed to \
             the application, and a previous run of this check leaves the very word it asserts.",
        ));
    }
    if let Some(stale) = crate::sys::clipboard_text() {
        return Err(Error::new(format!(
            "the clipboard still holds {} character(s) after being cleared, so something is \
             writing to it concurrently (a clipboard manager, usually).",
            stale.chars().count()
        )));
    }

    // --- launch -------------------------------------------------------------
    let mut spec = LaunchSpec::new(&exe, ctx.out("rotated-text.trace.txt"));
    spec.pdf = Some(pdf.clone());
    spec.env.push((
        ctx.profile.diag_env.0.to_owned(),
        ctx.profile.diag_env.1.to_owned(),
    ));
    spec.env
        .push((SHELL_DIAG_ENV.0.to_owned(), SHELL_DIAG_ENV.1.to_owned()));
    spec.allow_stale = ctx.allow_stale;
    spec.source_root = ctx.source_root.clone();

    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    report.artifact(session.trace_path().to_path_buf());
    report.note(format!(
        "launched {} as pid {} on {} ({:.0}x{:.0} pt)",
        exe.display(),
        session.pid(),
        pdf.display(),
        page.width_pt,
        page.height_pt
    ));
    session.settle(40);

    let trace = session.trace()?;
    if !trace.started(vocab.start_event) {
        return Err(Error::new(format!(
            "the trace has no `{}` line, so the diagnostic switch {}={} did not reach the \
             process and this check has no oracle. Captured stderr is at {}.",
            vocab.start_event,
            ctx.profile.diag_env.0,
            ctx.profile.diag_env.1,
            session.trace_path().display()
        )));
    }
    let driver = Driver::new(session.window());

    // --- Read mode ----------------------------------------------------------
    driving::click_mode_segment(&session, &driver, ui_rect, READ)?;
    session.settle(20);

    // --- the sweep, UP the string -------------------------------------------
    let trace = session.trace()?;
    let mapping = CanvasMapping::from_trace(&trace, vocab, page, 0)?;
    let frame = session.frame()?;
    let at = |offset: f64| DocPoint::new(0, ORIGIN.0 - ASCENDER_OFFSET_PT, ORIGIN.1 + offset);
    let start = frame.to_screen(mapping.doc_to_window(at(SWEEP_FROM_PT))?);
    let end = frame.to_screen(mapping.doc_to_window(at(SWEEP_TO_PT))?);
    report.note(format!(
        "sweeping the 90° string from ({:.0}, {:.0}) to ({:.0}, {:.0}) in PDF user space — up the \
         page, three points to the LEFT of the baseline, which is where the ink of \
         anticlockwise-turned text sits",
        ORIGIN.0 - ASCENDER_OFFSET_PT,
        ORIGIN.1 + SWEEP_FROM_PT,
        ORIGIN.0 - ASCENDER_OFFSET_PT,
        ORIGIN.1 + SWEEP_TO_PT
    ));
    driver.drag(start, end)?;
    session.settle(24);

    // --- assertions 1 and 2 -------------------------------------------------
    let trace = session.trace()?;
    // ★ The **last** non-empty selection, not the first. A sweep traces every
    // distinct state it passes through — `chars=1`, then `3`, then the settled
    // value — and the first is a real selection and a poor verdict: it is
    // whatever was covered on the frame egui first called the press a drag. The
    // last line is the selection the operator is left holding. Filtered on
    // `chars > 0` because a *clear* is traced too, as `chars=0`.
    let selections: Vec<_> = trace
        .events(SELECTION_EVENT)
        .filter(|l| l.get_usize("chars").unwrap_or(0) > 0)
        .collect();
    let Some(line) = selections.last() else {
        return Err(Error::new(format!(
            "the sweep selected nothing at all, so none of this check's four assertions can be \
             made. That is a statement about the aim or the layout rather than about rotated \
             text — the canvas may be showing the page at a zoom that puts the string off \
             screen. SKIPPED. Trace: {}.",
            session.trace_path().display()
        )));
    };
    let chars = line.get_usize("chars").unwrap_or(0);
    let quads = line.get_usize("quads").unwrap_or(0);
    report.note(format!("the sweep traced `{}`", line.raw));

    if chars != WORD.len() || quads != 1 {
        return Ok(Some(format!(
            "★ THE 90° STRING DID NOT SELECT AS ONE LINE. The sweep traced `{}` — expected \
             `chars={} quads=1`, the six letters of {WORD} as one band.\n\n\
             `chars={}` with `quads={}` is the reported defect exactly: `pdfcer-core` publishes a \
             glyph's advance as a LENGTH and never publishes its direction, so \
             `text_extract::layout::classify` — which breaks a line whenever the baseline y moves \
             — puts every letter of a 90° string on a line of its own. The shell recovers the \
             direction in `canvas::textsel::writing` and bands by it.\n\n\
             Where to look, in order: is `writing::census` returning the direction at all (it \
             needs a chain of three consecutive same-direction steps); is \
             `textsel::PageContext::opts` the settings funnel's options at all three call sites, \
             or has one reverted to `ExtractOptions::default()`; and is `textsel::hit` still \
             asking `writing::Rotated::position_at` BEFORE the engine's `hit_test`, whose line \
             boxes are hung off the wrong corner for rotated glyphs.",
            line.raw,
            WORD.len(),
            chars,
            quads
        )));
    }

    // --- assertion 3: the cursor turned -------------------------------------
    //
    // Read from the trace and nowhere else. See the module header: Windows
    // composites the pointer separately from window contents, so no capture
    // this harness can take contains a cursor.
    let turned = trace
        .events(CURSOR_EVENT)
        .filter(|l| l.get("shape") == Some("ibeam"))
        .filter_map(|l| l.get_usize("deg"))
        .collect::<Vec<_>>();
    if !turned.contains(&90) {
        return Ok(Some(format!(
            "★ THE I-BEAM DID NOT TURN. The sweep selected the string correctly, so the pointer \
             was over 90° text with `DragKind::TextSelect` in flight — which is exactly when \
             `tool::cursor_for` answers `CursorIcon::Text` and `canvas::interact` computes the \
             tilt. The I-beam angles traced were {turned:?}, and none of them is 90.\n\n\
             This is the operator's *\"the I cursor doesn't reorient\"* half, and it is the half \
             NO screenshot can catch: Windows composites the pointer separately, so every window \
             capture this harness takes contains no cursor at any price. The trace is the only \
             oracle there is.\n\n\
             Where to look: `canvas::interact`'s tilt block (is it still there, and is it \
             mapping the shape rather than passing `Shape::of`'s upright answer straight \
             through); `textsel::tilt_at` (does it still project the direction through \
             `viewer::pdf_space_to_canvas` rather than using the PDF-space angle); and \
             `cursor::Tilt::nearest` (5° quantisation, folded into 0..180)."
        )));
    }
    report.note(format!(
        "the I-beam turned: angles traced during the drag were {turned:?}"
    ));

    // --- assertion 4: the clipboard -----------------------------------------
    driver.press_chord(&[crate::sys::vk::CONTROL], crate::sys::vk::C)?;
    session.settle(30);
    // clipboard-chord-exempt: the failure message below QUOTES the broken form
    // in order to name defect O18 for whoever reads the report. It is prose
    // inside a string literal, not a call. The gate skips comment lines but
    // cannot see inside a multi-line string, and a gate that made this check
    // delete its own explanation would be working against itself —
    // `clipboard_text.rs` carries the identical exemption for the identical
    // sentence.
    let Some(text) = crate::sys::clipboard_text() else {
        return Ok(Some(
            "★ THE CLIPBOARD IS EMPTY after selecting the 90° string and pressing Ctrl+C. The \
             selection was correct — assertions 1 and 2 held — so this is the copy path rather \
             than the rotated-text work. That is defect O18's shape: `egui-winit` pushes \
             `Event::Copy` and no key event, so a handler asking `key_pressed(Key::C)` can never \
             fire. Check `canvas::textsel::clipboard::pending_key`."
                .to_owned(),
        ));
    };
    let text = text.trim_end_matches(['\r', '\n']).to_owned();
    if text != WORD {
        return Ok(Some(format!(
            "★ THE CLIPBOARD DOES NOT HOLD THE STRING. Expected {WORD:?}; got {text:?}.\n\n\
             This is the assertion the operator actually made — *\"when I copy and paste into \
             notepad, I get the text on one line as expected\"* — and it is the one no field of \
             the trace can stand in for: the trace reports what the SELECTION holds, and this \
             reports what another process would receive. If the escapes in the value above are \
             newlines, the derived line breaks between the letters were not suppressed and the \
             `chars` count that passed above is measuring something else."
        )));
    }
    report.note(format!(
        "the OS clipboard holds {text:?} — read from outside the process, which is the only \
         place the operator's complaint can be answered"
    ));

    Ok(None)
}
