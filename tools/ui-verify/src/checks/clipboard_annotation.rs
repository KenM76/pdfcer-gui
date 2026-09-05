//! `copying_a_sticky_note_carries_the_whole_comment` — **the annotation
//! clipboard, driven end to end.**
//!
//! ## ⚠ WRITTEN 2026-09-05 AND **NOT RUN**
//!
//! Said here, in its own header, rather than left for an absent result to
//! imply. The session that wrote it was told the operator might be at his
//! keyboard, and this harness drives the real cursor and the real keyboard and
//! takes the whole desktop while it does. **No line below has been observed
//! against a running binary**, and nothing in the report that ships with it
//! claims otherwise. It is registered so the next sweep picks it up.
//!
//! ★ That is the honest state and it is worth naming what it costs: every
//! failure message here is a *prediction* of what a wrong build would print,
//! and this project has three recorded cases of an articulate, plausible
//! failure message being about nothing at all. Treat the first run as
//! calibration, not as a verdict.
//!
//! ## What this is for
//!
//! Until 2026-09-05 `Ctrl+C` over a **sticky note** — a `/Text` annotation,
//! the single most-copied comment in a review workflow — put a sentence on the
//! status row saying pdfcer does not author that kind, and copied nothing. So
//! did a stamp, a text box, a link and a file attachment. The clipboard read a
//! `MarkupSpec` out of the dictionary and `annot_author::spec_from_dict` has no
//! reader for any of them.
//!
//! The repair routes the copy through `EditSession::copy_selection`, which
//! carries an annotation pdfcer does **not** model as its own dictionary plus
//! the object closure it reaches — including its baked `/AP`.
//!
//! ## ★★ Why this cannot be a unit test
//!
//! The unit tests in `canvas::clipboard::tests` already assert the clip's
//! contents and the round trip through `ObjectClip::to_bytes`. What they cannot
//! reach is the **chord**, and the chord is where this family's defects have
//! actually lived:
//!
//! | link | what breaks it, historically |
//! |---|---|
//! | `Ctrl+C` arrives at the canvas at all | a text sweep or a focused widget owning the chord (defect O18) |
//! | the copy puts a MARKER on the OS clipboard | without one, `egui-winit` never synthesises `Event::Paste` and `Ctrl+V` vanishes — a documented trap that a NEW copy path re-earns every time, because the workaround lives at each copy site |
//! | `Ctrl+V` reaches the paste verb | the same trap, from the other end |
//! | the engine actually planted the annotation | nothing on this side can see it but the count |
//!
//! The second row is the one this check exists for. `RESUME.md` records the
//! form-field copy shipping **without** the marker, one function away from the
//! comment explaining why it was needed, and the symptom was `Ctrl+V` working
//! or not depending on what the operator had last copied in another program.
//! The annotation copy is a new copy site and inherits exactly that hazard.
//!
//! ## ★★★ It pins its own fixture and IGNORES `--pdf`
//!
//! Same posture as `ocr` and as `three_clicks_round_a_hole_measure_the_hole`,
//! and for a stronger reason than either: this check's subject is *"a
//! `/Text` annotation, which pdfcer does not model, copies whole"*, and on a
//! document whose only annotations are squares and clouds **the defect cannot
//! occur** — every one of those takes the spec route, which worked before this
//! change and works after it. An arbitrary drawing would make this check unable
//! to fail, which is this suite's own stated worst outcome.
//!
//! `fixtures/annots-with-everything.pdf` is built by
//! `tools/gen-annots-with-everything-fixture.py`, whose header argues for every
//! key on every annotation in it. The two facts this check depends on:
//!
//! * `/Annots` position **1** is a `/Text` sticky note at `/Rect [360 660 380
//!   680]`, carrying `/CA 0.4`, `/T`, `/M`, `/Contents` and an `/AP`;
//! * `/Annots` position **0** is a `/Square`, which is what a *wrong* build
//!   would copy if the click missed — and it would look like a pass, which is
//!   why the copy's own trace line is read for `annots=` rather than for mere
//!   presence.
//!
//! ## What it does NOT assert, said rather than implied
//!
//! **The pixels.** A pasted sticky note is a 20 × 20 pt icon; at fit-page zoom
//! on a 595 × 842 pt sheet that is a handful of screen pixels, below the noise
//! floor of a window capture. What is asserted instead is the engine's own
//! count of what it planted — `paste-objects-applied … annots=1` — which is a
//! number a wrong build gets wrong and a capture cannot resolve.
//!
//! **That the author, date, note text and opacity survived.** They do, on this
//! route, because the raw carrier copies the dictionary — and that is asserted
//! key by key in `canvas::clipboard::tests`, where the dictionary is readable.
//! From out here the trace carries no dictionary and inventing an oracle for
//! one would be a proxy.

use std::path::PathBuf;

use crate::checks::driving::{SHELL_DIAG_ENV, click_mode_segment};
use crate::checks::{Check, CheckContext};
use crate::coords::{CanvasMapping, DocPoint, PageGeometry};
use crate::error::{Error, Result};
use crate::input::Driver;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;
use crate::sys::vk;

/// The mode whose canvas selects annotations and may paste one.
///
/// ★ **Review, not Edit**, and the choice is an assertion in itself. A comment
/// is markup, so pasting one needs `author_markup` — which Review grants —
/// rather than `edit_content`, which only Edit does. A build that demanded the
/// content capability for an annotation clip would leave the mode whose entire
/// purpose is marking up somebody else's drawing unable to paste a comment,
/// and driving in Edit would never notice.
const MODE: &str = "review";

/// The fixture, relative to the repository root.
const FIXTURE: &str = "fixtures/annots-with-everything.pdf";

/// The sticky note's `/Rect` centre in PDF user space — `[360 660 380 680]`.
///
/// ★ Hard-coded rather than taken from `--doc-point`, which this check ignores
/// along with `--pdf`: the point and the fixture are one fact, and a suite-wide
/// coordinate aimed at a different document would put the click on blank paper
/// and report a defect about the hit test.
const NOTE_POINT: (f64, f64) = (370.0, 670.0);

/// The page, in points — `/MediaBox [0 0 595 842]` in the fixture.
const PAGE: PageGeometry = PageGeometry {
    width_pt: 595.0,
    height_pt: 842.0,
};

/// `clipboard-copy kind=selection page=… objects=… annots=… thin=… bytes=…`.
const COPY_EVENT: &str = "clipboard-copy";
/// `clipboard-paste kind=selection page=… from=… objects=… annots=… …`.
const PASTE_EVENT: &str = "clipboard-paste";
/// `paste-objects-applied page=… pasted=… annots=… resources_added=… at=[…]`.
const APPLIED_EVENT: &str = "paste-objects-applied";
/// `annot-select page=… id=… kind=… subtype=… locked=… rect=…`.
///
/// **The annotation selection's own line**, and the only one that reports one:
/// `canvas-selection` is silent for an annotation. See the precondition in
/// [`drive`] for the incident that put it here.
const ANNOT_SELECT: &str = "annot-select";
/// The `/Subtype` this check's operand must have.
const WANTED_SUBTYPE: &str = "Text";
/// `chord-not-offered id=… mode=…` — the shell's own line for a chord that
/// arrived and was refused by the mode gate. See the paste branch in [`drive`].
const CHORD_NOT_OFFERED: &str = "chord-not-offered";
/// The command `Ctrl+V` resolves to.
const PASTE_COMMAND: &str = "edit.paste";

fn fixture_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join(FIXTURE)
}

/// See the module documentation.
pub struct CopyingAStickyNoteCarriesTheWholeComment;

impl Check for CopyingAStickyNoteCarriesTheWholeComment {
    fn name(&self) -> &'static str {
        "copying_a_sticky_note_carries_the_whole_comment"
    }

    fn defect(&self) -> &'static str {
        "Ctrl+C over a sticky note, a stamp, a text box, a link or a file attachment copies \
         NOTHING and says pdfcer does not author that kind of annotation — so a standard note \
         cannot be carried from one drawing to another. Or it copies, Ctrl+V arrives, and the \
         engine plants no annotation at all, which traces identically to a paste of empty page \
         content"
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
            "input is disabled (--no-input). This check clicks a mode segment, clicks an \
             annotation and presses Ctrl+C and Ctrl+V. Reported as SKIPPED rather than passed.",
        ));
    }
    let ui_rect = vocab.ui_rect_event.ok_or_else(|| {
        Error::new(format!(
            "the `{}` profile declares no ui-rect trace event.",
            ctx.profile.name
        ))
    })?;
    let fixture = fixture_path();
    if !fixture.is_file() {
        return Err(Error::new(format!(
            "the annotation fixture is not at {}. Generate it:\n  python \
             tools/gen-annots-with-everything-fixture.py",
            fixture.display()
        )));
    }
    // ★ A sweep that supplied `--pdf` and had it thrown away must be told so.
    // A run that silently ignored a flag is indistinguishable from one that
    // honoured it, which is the finding `RESUME.md` records about the two other
    // fixture-pinning checks in this suite.
    if ctx.pdf.is_some() {
        report.note(
            "★ this check IGNORES --pdf and pins fixtures/annots-with-everything.pdf: its \
             subject is an annotation pdfcer does NOT model, and on a drawing whose comments \
             are all squares and clouds the defect cannot occur",
        );
    }
    report.note(format!("fixture {}", fixture.display()));

    let mut spec = LaunchSpec::new(&exe, ctx.out("clipboard-annotation.trace.txt"));
    spec.pdf = Some(fixture.clone());
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

    // --- 1: Review, the mode that may author and paste markup ---------------
    click_mode_segment(&session, &driver, ui_rect, MODE)?;
    session.settle(20);

    // --- 2: click the sticky note ------------------------------------------
    //
    // ★ No tool is armed first and none should be: with a markup or measure
    // tool armed a click on the page is a PICK, not a selection, and the check
    // would report "the note could not be selected" about a build whose
    // selection is fine. `sys::vk::V`'s doc comment carries the general rule;
    // here the mode was just entered, so nothing is armed.
    let trace = session.trace()?;
    let mapping = CanvasMapping::from_trace(&trace, vocab, PAGE, 0)?;
    let window_point = mapping.doc_to_window(DocPoint::new(0, NOTE_POINT.0, NOTE_POINT.1))?;
    let frame = session.frame()?;
    driver.click_at(frame.to_screen(window_point))?;
    session.settle(16);

    // ★★★ THE PRECONDITION READS `annot-select`, NOT `canvas-selection` —
    // corrected 2026-09-05, on the first run this check ever had.
    //
    // It read `canvas-selection sel=` and SKIPPED with *"the click at PDF (370,
    // 670) selected nothing … either the fixture was regenerated with different
    // geometry or an annotation hit test regressed."* Both suggestions were
    // wrong and the message was confident. The trace of that very run carried:
    //
    // ```text
    // pdfcer-diag annot-select page=0 id=ObjId { num: 6, generation: 0 }
    //     kind=Markup subtype=Text locked=false rect=[[360.0 162.0] - [380.0 182.0]]
    // ```
    //
    // The sticky note was selected, first time, exactly where this check aimed.
    // `canvas::clicking` publishes `canvas-selection` for an **object**
    // selection and `annot-select` for an **annotation** one — they are
    // different selections in different fields of the model — so a check whose
    // whole subject is an annotation was reading the line that is silent for
    // annotations by construction. It could never have passed, and a SKIP is
    // not red, so it would have sat there looking like an ordinary aim problem.
    //
    // ⇒ **Ask what the check SAMPLED before asking what is broken.**
    //
    // ★ Reading `subtype=` is a strength the old oracle did not have: it proves
    // the click took the **/Text** annotation and not the /Square at `/Annots`
    // 0, which is the aim error this fixture was built to expose. `sel=1` would
    // have been equally happy with either.
    let trace = session.trace()?;
    let selection = trace.events(ANNOT_SELECT).last();
    let subtype = selection.as_ref().and_then(|l| l.get("subtype"));
    if subtype != Some(WANTED_SUBTYPE) {
        return Err(Error::new(format!(
            "the click at PDF ({:.0}, {:.0}) did not select the sticky note. That point is the \
             centre of the fixture's /Text annotation at /Rect [360 660 380 680]. The last \
             `{ANNOT_SELECT}` line was {}; this check needs `subtype={WANTED_SUBTYPE}`. If it \
             names Square the aim is off by the /Annots-0 rectangle; if there is no line at \
             all, either the fixture was regenerated with different geometry or an annotation \
             hit test regressed. SKIPPED rather than failed, because a check that cannot get \
             its operand selected is not judging its own subject. Trace: {}.",
            NOTE_POINT.0,
            NOTE_POINT.1,
            selection
                .as_ref()
                .map_or_else(|| "absent".to_owned(), |l| format!("`{}`", l.raw)),
            session.trace_path().display()
        )));
    }
    report.note("the click selected the sticky note (`annot-select … subtype=Text`)");

    // --- 3: Ctrl+C ----------------------------------------------------------
    driver.press_chord(&[vk::CONTROL], vk::C)?;
    session.settle(20);

    let trace = session.trace()?;
    let copy = trace.events(COPY_EVENT).last();
    let Some(copy) = copy else {
        return Ok(Some(format!(
            "★ CTRL+C OVER A STICKY NOTE COPIED NOTHING — no `{COPY_EVENT}` line at all.\n\
             Before 2026-09-05 this was the shipped behaviour: `canvas::clipboard::copy` read a \
             MarkupSpec out of the dictionary, `annot_author::spec_from_dict` has no reader for \
             a /Text annotation, and the copy refused with `Refusal::Unreadable`. If the line is \
             absent, either that route is back or the chord never reached the canvas — check \
             for a `command-declined … reason=text-owns-the-clipboard` line first, which is a \
             different finding and sends you somewhere else. Trace: {}.",
            session.trace_path().display()
        )));
    };
    report.note(format!("★ the copy ran: `{}`", copy.raw));

    // ★★★ THE LINE THAT MAKES THIS CHECK ABLE TO FAIL.
    //
    // `kind=selection` alone would be satisfied by a build that copied the
    // PAGE CONTENT under the note and no annotation — same event, same page,
    // and `objects=1 annots=0` versus `objects=0 annots=1` is the whole
    // difference. Reading `annots=` is what tells a copy of the comment from a
    // copy of the rectangle it sits on.
    let annots = copy.get_usize("annots").unwrap_or(0);
    if copy.get("kind") == Some("markup") {
        return Ok(Some(format!(
            "★ the copy took the SPEC route (`{}`), which means `spec_from_dict` read the sticky \
             note — so either the engine learned to model /Text and \
             `canvas::annotclip::carried_options` has not been taught its keys, or the click \
             landed on the /Square at /Annots 0 instead. The first is a silent data loss; the \
             second is an aim problem. `canvas::annotclip`'s \
             `the_engine_models_a_square_and_carries_a_sticky_note_whole` distinguishes them \
             without driving. Trace: {}.",
            copy.raw,
            session.trace_path().display()
        )));
    }
    if annots != 1 {
        return Ok(Some(format!(
            "★ THE COPY CARRIED {annots} ANNOTATIONS, not 1: `{}`.\n\
             `EditSession::copy_selection` returns Ok with an EMPTY annotation payload when the \
             index list never reached it, so a clip that carried nothing is not an error and \
             looks exactly like a clip that carried something. `annots=0` here means \
             `canvas::annotclip::selected` resolved no /Annots position for the selected ObjId. \
             Trace: {}.",
            copy.raw,
            session.trace_path().display()
        )));
    }
    let bytes = copy.get_usize("bytes").unwrap_or(0);
    if bytes == 0 {
        return Ok(Some(format!(
            "the clip serialised to ZERO bytes: `{}`. `ObjectClip::to_bytes` writes a magic \
             prefix and a version before anything else, so an empty payload is not a clip with \
             nothing in it — it is a clip that was never assembled. Trace: {}.",
            copy.raw,
            session.trace_path().display()
        )));
    }

    // --- 4: Ctrl+V ----------------------------------------------------------
    //
    // ★★ This is the step the whole check is worth writing for. `egui-winit`
    // raises `Event::Paste` only when the OS clipboard holds non-empty text and
    // swallows the raw key either way, so a copy path that forgot the marker
    // leaves this keystroke producing NO EVENT OF ANY KIND — and whether it
    // works then depends on what the operator last copied in another program.
    // A unit test cannot see this; nothing but a real chord can.
    driver.press_chord(&[vk::CONTROL], vk::V)?;
    session.settle(30);

    let trace = session.trace()?;
    let Some(paste) = trace.events(PASTE_EVENT).last() else {
        // ★★★ **ASK THE MODE GATE FIRST — added 2026-09-05 after this branch's
        // first real run blamed the wrong mechanism.**
        //
        // The message below suspects the OS-clipboard marker, which is the
        // right first suspect for a *new copy site*. On the run this check
        // finally got, it was wrong in a way that would have cost a session:
        // the marker was fine, `Event::Paste` was synthesised, `Ctrl+V` reached
        // the shell — and the shell REFUSED it:
        //
        // ```text
        // chord-command chord="Ctrl+V" id=edit.paste via=clipboard-event
        // chord-not-offered id=edit.paste mode=review
        // ```
        //
        // `edit.paste` is not offered in **Review**, while `edit.copy` is. So
        // the mode whose whole purpose is marking up somebody else's drawing
        // can copy a comment and cannot paste it — which is precisely the
        // failure this check's own header predicted and named as its reason for
        // driving Review rather than Edit.
        //
        // ⇒ A failure message that names a mechanism the trace can rule out is
        // a confident failure about the wrong subject. Read the gate line, and
        // say so when it is there.
        //
        // ⚠ **THE GATE WAS OPENED LATER THE SAME DAY.** `edit.paste` now
        // escapes its tab with the rest of the Clipboard group, so this branch
        // should no longer fire in Review — and it is kept because it is one of
        // only two witnesses that would notice the escape list being narrowed
        // again. `a_paste_review_may_not_do_says_so` is the other and owns the
        // refusal side. ★ Neither has been re-run against the fixed build: the
        // session that made the fix worked headlessly.
        let refusal = trace
            .events(CHORD_NOT_OFFERED)
            .find(|l| l.get("id") == Some(PASTE_COMMAND));
        if let Some(refusal) = refusal {
            return Ok(Some(format!(
                "★★★ CTRL+V REACHED THE SHELL AND THE MODE GATE REFUSED IT: `{}`.\n\
                 The copy worked — `{}` — so the clipboard, the marker and the chord are all \
                 fine. `{PASTE_COMMAND}` is simply not offered in this mode, and `edit.copy` \
                 is, so an operator in the mode whose entire purpose is marking up somebody \
                 else's drawing can copy a comment and has nowhere to put it. This is an \
                 APPLICATION finding, not a clipboard one: look at the command's \
                 `visible_when`/capability gate for the mode, not at `canvas::clipboard`. \
                 Trace: {}.",
                refusal.raw,
                copy.raw,
                session.trace_path().display()
            )));
        }
        return Ok(Some(format!(
            "the copy happened and CTRL+V RAISED NOTHING: no `{PASTE_EVENT}` line, and no \
             `{CHORD_NOT_OFFERED}` line for `{PASTE_COMMAND}` either.\n\
             ★ Suspect the OS-CLIPBOARD MARKER next. `egui-winit` synthesises \
             `Event::Paste` only when the OS clipboard holds non-empty text and returns before \
             pushing a key event either way, so a copy site that did not call `ctx.copy_text` \
             makes this chord vanish completely. That is a documented trap this project has \
             already shipped once, on the form-field copy, one function away from the comment \
             explaining it — and the annotation copy is a NEW copy site. Look for \
             `clipboard-image-declined` and then for whether anything reached the OS clipboard \
             at all. Trace: {}.",
            session.trace_path().display()
        )));
    };
    report.note(format!("★ the paste was raised: `{}`", paste.raw));

    // --- 5: and the ENGINE planted it ---------------------------------------
    let Some(applied) = trace.events(APPLIED_EVENT).last() else {
        return Ok(Some(format!(
            "the paste was raised and NEVER REACHED THE ENGINE: no `{APPLIED_EVENT}` line. The \
             action goes through `app::actions::vector`'s `vector_edit_on_page`, which refuses \
             on an encrypted or certified document and traces the refusal — look for that \
             before suspecting the clipboard. Trace: {}.",
            session.trace_path().display()
        )));
    };
    let planted = applied.get_usize("annots").unwrap_or(0);
    if planted != 1 {
        return Ok(Some(format!(
            "★ THE ENGINE PLANTED {planted} ANNOTATIONS: `{}`.\n\
             This is the assertion nothing else in the suite can make. `pasted=` counts CONTENT \
             objects only, so an annotation-only paste traces `pasted=0` whether it worked or \
             not — the two builds are indistinguishable without this field, which is why it was \
             added to the trace in the same change. A zero here with a non-zero `annots=` on the \
             copy line means the payload was lost between `to_bytes` and `from_bytes`, or \
             `paste_clip_annotations` refused every member. Trace: {}.",
            applied.raw,
            session.trace_path().display()
        )));
    }
    report.note(format!("★ the engine planted it: `{}`", applied.raw));
    Ok(None)
}
