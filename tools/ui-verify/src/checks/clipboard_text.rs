//! `ctrl_c_copies_text_to_the_os_clipboard` — the one assertion no unit test in
//! this workspace can make.
//!
//! # Why this check exists, and why nothing else could have caught the defect
//!
//! `OPERATOR_REQUESTS.md` O18, reported 2026-08-21:
//!
//! > *"if I select text in read mode … and press ctrl+c to copy, then try to
//! > paste in notepad, it doesn't work. I get a notice to paste it back into
//! > pdfc to place it."*
//!
//! Sweeping text on the page and pressing `Ctrl+C` had **never once copied it**,
//! in any mode, since the day that code was written. `egui-winit` intercepts the
//! three clipboard chords and pushes `Event::Copy` *instead of* an `Event::Key`,
//! and the handler was looking for the key. The object clipboard answered
//! instead and left its own marker sentence on the clipboard.
//!
//! **1,628 unit tests were green throughout**, and no number of them would have
//! helped, because the failure is not in a function's return value — it is in
//! *which of two handlers reached the operating system last*. Even a trace is
//! not enough: `text-copy source=selection` can be emitted, truthfully, by a
//! frame in which a later handler then overwrites the clipboard. The trace says
//! what a function did; it cannot say what the operator would get.
//!
//! **The only oracle for "what does the operator get when they paste" is the
//! operating system's clipboard, read from outside the process.** That is what
//! this check does, and it is the entire justification for
//! [`crate::sys::clipboard_text`] existing.
//!
//! # ★★ It clears the clipboard first, and that is not hygiene
//!
//! It is the difference between a check and a coin toss. Three outcomes are
//! otherwise indistinguishable:
//!
//! | the application | the clipboard afterwards | without clearing first |
//! |---|---|---|
//! | copies the text correctly | the swept text | passes |
//! | writes the object marker (the defect) | `"…copied from pdfcer…"` | fails |
//! | **does nothing at all** | whatever was there before | **passes, if a previous run left the right text** |
//!
//! Row three is the one that matters. A check that passes when the application
//! does nothing is worse than no check, because it certifies the silence. So:
//! clear, drive, read — and if the clear itself fails, report SKIPPED rather
//! than asserting against a clipboard this process never controlled.
//!
//! # What it asserts, in both directions
//!
//! A positive and a negative, and neither alone is sufficient:
//!
//! * the clipboard holds **something**, and
//! * what it holds is **not the object clipboard's marker sentence**.
//!
//! The negative is the one that names the defect. Without it, a build that
//! wrote *"1 object copied from pdfcer"* would satisfy "the clipboard is not
//! empty" and pass while doing exactly the wrong thing.
//!
//! It deliberately does **not** assert the exact string. What a sweep across a
//! CAD drawing's title block yields depends on the fixture, on where
//! `--doc-point` aims and on how the producer split its show operators; pinning
//! it would make the check a statement about the file rather than about the
//! program, and it would be re-pinned rather than believed the first time it
//! failed.
//!
//! # Read mode, deliberately
//!
//! The operator reported it there first, and it is the sharper case: Read has no
//! content selection at all, so the *only* thing a click can produce is a text
//! sweep. If the clipboard comes back holding an object marker in Read, an
//! object clipboard answered a chord in a mode that cannot select an object —
//! which is a second defect wearing the first one's clothes.

use crate::checks::driving::{SHELL_DIAG_ENV, click_mode_segment};
use crate::checks::{Check, CheckContext, CheckReport};
use crate::coords::{CanvasMapping, DocPoint, PageGeometry};
use crate::error::{Error, Result};
use crate::input::Driver;
use crate::launch::{LaunchSpec, Session};

/// The mode this drives. See the module docs for why Read rather than Edit.
const MODE: &str = "read";

/// The substring that means the **object** clipboard answered.
///
/// Matched as a fragment rather than as the whole sentence because
/// `text::clipboard::os_marker` spells singular and plural differently and both
/// mean the same failure. Kept in sync by nothing but this comment, which is
/// acceptable precisely because a *false negative* here is harmless: if the
/// wording changes and this stops matching, the positive assertion still fails
/// on a build that copies no text.
const OBJECT_MARKER: &str = "copied from pdfcer";

/// See the module documentation.
pub struct CtrlCCopiesTextToTheOsClipboard;

impl Check for CtrlCCopiesTextToTheOsClipboard {
    fn name(&self) -> &'static str {
        "ctrl_c_copies_text_to_the_os_clipboard"
    }

    fn defect(&self) -> &'static str {
        "sweeping text on the page and pressing Ctrl+C puts \"1 object copied from pdfcer\" on the \
         clipboard instead of the text. Pasting into any other program gives that sentence"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        match drive(ctx, &mut report) {
            Ok(Some(failure)) => report.fail(failure),
            Ok(None) => report.pass(),
            Err(skip) => report.skip(skip.to_string()),
        }
    }
}

fn drive(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    let vocab = &ctx.profile.vocab;
    let exe = ctx.resolve_exe().ok_or_else(|| {
        Error::new(format!(
            "no binary to drive. Pass --exe, or build the profile's default at {}.",
            ctx.profile.default_exe
        ))
    })?;
    let pdf = ctx
        .pdf
        .clone()
        .ok_or_else(|| Error::new("no --pdf. This check needs a document with text in it."))?;
    let target = ctx.target.ok_or_else(|| {
        Error::new(
            "no --doc-point. Pass PAGE,X,Y in PDF user space naming a point that sits ON TEXT — \
             a sweep that starts on blank paper selects nothing, and an empty clipboard is not a \
             statement about the copy path.",
        )
    })?;
    if !ctx.allow_input {
        return Err(Error::new(
            "input is disabled (--no-input). This check sweeps the pointer across text and \
             presses Ctrl+C. Reported as SKIPPED rather than passed.",
        ));
    }
    let ui_rect = vocab.ui_rect_event.ok_or_else(|| {
        Error::new(format!(
            "the `{}` profile declares no ui-rect trace event.",
            ctx.profile.name
        ))
    })?;
    let page: PageGeometry = match ctx.page_size {
        Some((w, h)) => PageGeometry {
            width_pt: w,
            height_pt: h,
        },
        None => crate::fixture::page_geometry(&pdf).ok_or_else(|| {
            Error::new(format!(
                "cannot read a page size from {}. Pass --page-size WxH.",
                pdf.display()
            ))
        })?,
    };

    // --- 0: OWN THE CLIPBOARD BEFORE ANYTHING ELSE --------------------------
    //
    // See the module docs' table. Everything after this is only a statement
    // about this run because of this line.
    if !crate::sys::clear_clipboard() {
        return Err(Error::new(
            "could not clear the OS clipboard — another process is holding it, or this is not \
             Windows. SKIPPED rather than run: an assertion made against a clipboard this \
             process never controlled would pass on stale content from an earlier run, which is \
             precisely the failure this check exists to make impossible.",
        ));
    }
    if let Some(stale) = crate::sys::clipboard_text() {
        return Err(Error::new(format!(
            "the clipboard still holds {} character(s) after being cleared, so something is \
             writing to it concurrently (a clipboard manager, usually). SKIPPED: the read at the \
             end could not be attributed to the application.",
            stale.chars().count()
        )));
    }

    let mut spec = LaunchSpec::new(&exe, ctx.out("clipboard-text.trace.txt"));
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
        "launched {} as pid {}",
        exe.display(),
        session.pid()
    ));
    session.settle(40);
    let driver = Driver::new(session.window());

    // --- 1: Read mode -------------------------------------------------------
    click_mode_segment(&session, &driver, ui_rect, MODE)?;
    session.settle(20);

    // --- 2: sweep across the text ------------------------------------------
    let trace = session.trace()?;
    let mapping = CanvasMapping::from_trace(&trace, vocab, page, target.page)?;
    let frame = session.frame()?;
    let start =
        frame.to_screen(mapping.doc_to_window(DocPoint::new(target.page, target.x, target.y))?);
    // ★ Sweep to the RIGHT along the same baseline. A diagonal drag would also
    // select, and would make a failure ambiguous between "the sweep missed the
    // line" and "the copy did not fire". Same y, a generous run in x.
    let end = frame.to_screen(mapping.doc_to_window(DocPoint::new(
        target.page,
        target.x + SWEEP_PT,
        target.y,
    ))?);
    driver.drag(start, end)?;
    session.settle(24);

    // The sweep is reported, so a failure can say WHICH half went wrong.
    let trace = session.trace()?;
    let swept = trace
        .events(SELECTION_EVENT)
        .last()
        .and_then(|l| l.get("chars"))
        .and_then(|n| n.parse::<usize>().ok())
        .unwrap_or(0);
    if swept == 0 {
        return Err(Error::new(format!(
            "the drag from (page {}, {:.1}, {:.1}) rightwards {SWEEP_PT} pt selected no text, so \
             there is nothing for Ctrl+C to copy. That is a fact about the fixture and the point \
             rather than about the build — aim --doc-point at a run of text. SKIPPED.",
            target.page + 1,
            target.x,
            target.y,
        )));
    }

    // --- 3: the chord -------------------------------------------------------
    driver.press_chord(&[crate::sys::vk::CONTROL], crate::sys::vk::C)?;
    session.settle(30);

    // --- 4: the only oracle that matters ------------------------------------
    // clipboard-chord-exempt: the failure message below QUOTES the broken form
    // in order to name the defect for whoever reads the report. It is prose
    // inside a string literal, not a call. The gate skips comment lines but
    // cannot see inside a multi-line string, and a gate that made this check
    // delete its own explanation would be working against itself.
    let Some(text) = crate::sys::clipboard_text() else {
        return Ok(Some(
            "★ THE CLIPBOARD IS EMPTY after sweeping text and pressing Ctrl+C. The chord reached \
             nothing that writes text. This is defect O18's shape: `egui-winit` pushes \
             `Event::Copy` and NO key event, so a handler asking `key_pressed(Key::C)` can never \
             fire. Check `canvas::textsel::clipboard::pending_key`."
                .to_owned(),
        ));
    };

    report.note(format!(
        "the sweep selected {swept} character(s); the clipboard holds {} after Ctrl+C",
        text.chars().count()
    ));

    if text.contains(OBJECT_MARKER) {
        return Ok(Some(format!(
            "★★ THE OBJECT CLIPBOARD ANSWERED A TEXT COPY — this is exactly defect O18 as the \
             operator reported it. Pasting into another program yields {text:?} instead of the \
             swept text. `canvas::clipboard::text_owns_the_chord` is the guard that must stand \
             the object path down; it is either not being called or answering false while a text \
             selection is live. ★ Note this happened in READ mode, which cannot select an object \
             at all."
        )));
    }

    if text.trim().is_empty() {
        return Ok(Some(format!(
            "the clipboard holds {} character(s) but all of them are whitespace, which is not a \
             copy of anything the operator can see. Either the sweep crossed a gap between runs \
             or the extraction returned nothing for them.",
            text.chars().count()
        )));
    }

    report.note(format!(
        "clipboard begins {:?}",
        text.chars().take(40).collect::<String>()
    ));
    Ok(None)
}

/// How far to sweep, in PDF points, along the baseline.
///
/// Wide enough to cross several glyphs on a drawing's title block at any
/// reasonable font size, and short enough that it stays inside one line rather
/// than wrapping into whatever is beside it.
const SWEEP_PT: f64 = 60.0;

/// The trace line that says a sweep produced a selection, and **how many
/// characters** it took.
///
/// ★ The first version of this check watched `text-selection`, which no build
/// has ever emitted — the event is `canvas-text-selection`. The sweep worked
/// perfectly and the check reported SKIPPED with a message blaming the fixture
/// and `--doc-point`. A wrong event name and a genuinely missing feature are
/// the same silence, and the harness cannot tell them apart; only reading the
/// trace can.
///
/// It reads `chars=` rather than merely counting the line, so a sweep that
/// produced an EMPTY selection is distinguishable from one that produced text.
const SELECTION_EVENT: &str = "canvas-text-selection";
