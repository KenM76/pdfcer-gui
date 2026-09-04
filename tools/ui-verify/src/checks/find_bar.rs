//! `find_opens_and_finds` — Ctrl+F reaches Find, and Find finds something.
//!
//! # What this is for
//!
//! Find is the command that decides whether pdfcer can replace a PDF reader,
//! and its chord is the most reflexive one in any application. It was built
//! with 38 unit tests and driven once by hand; this is the part that runs
//! every time.
//!
//! It could not be written when Find landed. `Driver::press` sends a bare
//! virtual key with no modifiers, so a command bound to `Ctrl+F` was simply
//! unreachable from this harness — the check was filed rather than written,
//! and `Driver::press_chord` was added to unblock it.
//!
//! # What it asserts, in order, and why each step is separate
//!
//! 1. **The chord dispatches the command.** `chord-command chord=Ctrl+F
//!    id=edit.find`. This is the one that would have caught the defect the
//!    Open work found in `Ctrl+O`: a chord printed in a tooltip, present in
//!    the keymap, and bound to nothing, because the key table could not spell
//!    a letter chord. That state is invisible in every unit test — the keymap
//!    was right, the command was right, and the two were never introduced.
//! 2. **The bar opens.** `find-toggled open=true`. Separate from step 1
//!    because a command that dispatches and does nothing is exactly the
//!    `command-unimplemented` shape this project keeps finding, and merging
//!    the two assertions would let a dispatch with no effect pass as a
//!    working Find.
//! 3. **The bar is on screen.** A `find-bar` region with a non-zero area.
//!    Separate again: the Find state can be open while the widget is clipped
//!    to nothing, which is a real failure mode this project has hit — three
//!    panels shipped with a body, a rail entry and no control anyone could
//!    click, and every verification passed for the whole of their shipped
//!    life.
//! 4. **A search runs and reports hits.** `find needle=… hits=N`. With
//!    `hits=0` the check still PASSES but says so loudly in a note: whether a
//!    given PDF contains a given word is a property of the fixture, not of
//!    the application, and failing on it would make this check fail whenever
//!    somebody pointed it at a drawing instead of a report.
//!
//! # Why the needle is typed rather than injected
//!
//! There is no `PDFCER_DIAG_FIND` seam and this check deliberately does not
//! ask for one. The native-file-dialog seam exists because a native dialog is
//! **outside egui's event loop** and cannot be driven at all; the find field
//! is an ordinary egui text field inside the window, so typing into it
//! exercises the focus handling, the text field, the Enter binding and the
//! search in one gesture. Substituting the answer there would skip the parts
//! most likely to be wrong.
//!
//! # It types into a real window
//!
//! Every keystroke goes to the foreground window, so this check needs
//! `--no-input` to be off and refuses rather than degrades otherwise. See
//! `Driver::press_chord`, which will not send a chord at all without a target
//! window — a bare keystroke into the operator's editor types a character,
//! but a chord runs a command.

use crate::checks::{Check, CheckContext, CheckReport};
use crate::error::{Error, Result};
use crate::input::Driver;
use crate::launch::{LaunchSpec, Session};
use crate::sys::vk;

/// What to search for.
///
/// `e` rather than a word, and the reason is the fixture problem: this check
/// is pointed at whatever PDF the operator passes, and any *word* is a bet
/// about that document's contents. A single common letter is the closest
/// thing to a needle that a page of English or of a CAD title block will
/// contain — and when it does not, the check still passes and says so, which
/// is why the choice is a note rather than a load-bearing assumption.
const NEEDLE: char = 'e';

/// `E`. Typed by virtual key, so the letter above and this must agree.
const NEEDLE_VK: u16 = 0x45;

pub struct FindOpensAndFinds;

impl Check for FindOpensAndFinds {
    fn name(&self) -> &'static str {
        "find_opens_and_finds"
    }

    fn defect(&self) -> &'static str {
        "Ctrl+F does not reach Find, or Find opens without finding anything"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        match assess(ctx, &mut report) {
            Ok(Some(failure)) => report.fail(failure),
            Ok(None) => report.pass(),
            Err(why) => report.from_error(&why),
        }
    }
}

fn assess(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    let exe = ctx.resolve_exe().ok_or_else(|| {
        Error::new(format!(
            "no binary to drive. Pass --exe, or build the profile's default at {}.",
            ctx.profile.default_exe
        ))
    })?;

    // A document is not optional here, unlike the chrome checks. `edit.find`
    // is gated on `doc.pages`, so with nothing open the chord correctly does
    // nothing — and a check that drove that would be asserting the gate works
    // while claiming to assert that Find does.
    let pdf = ctx.pdf.clone().ok_or_else(|| {
        Error::new(
            "no --pdf. Find is gated on a document having pages, so with nothing open the chord \
             correctly does nothing and this check would be measuring the gate rather than the \
             feature.",
        )
    })?;

    // Checked before launching. A run that cannot type should not leave a
    // window on the operator's desktop to find that out.
    if !ctx.allow_input {
        return Err(Error::new(
            "input is disabled (--no-input), and this check is entirely input: it presses \
             Ctrl+F, types a needle and presses Enter. Reported as SKIPPED rather than passed — \
             a check that did not run has learned nothing.",
        ));
    }

    let mut spec = LaunchSpec::new(&exe, ctx.out("find_bar.trace.txt"));
    spec.pdf = Some(pdf);
    spec.env.push((
        ctx.profile.diag_env.0.to_owned(),
        ctx.profile.diag_env.1.to_owned(),
    ));
    spec.allow_stale = ctx.allow_stale;
    spec.source_root = ctx.source_root.clone();

    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    report.note(format!(
        "launched {} as pid {}",
        exe.display(),
        session.pid()
    ));
    report.artifact(session.trace_path().to_path_buf());
    // Long enough for the first page to raster. A search on a document still
    // loading would report zero hits for a reason that is not Find's.
    session.settle(48);

    let driver = Driver::new(session.window());

    // --- 0. prove the input channel works, BEFORE testing the feature ------
    //
    // ★ Without this the check cannot tell "Find is broken" from "nothing was
    // ever typed at it", and it will confidently report the first. That is
    // not hypothetical: this check's own first run reported "Ctrl+F did not
    // dispatch `edit.find`" against a build in which Ctrl+F works, and the
    // real answer was that the chord never arrived.
    //
    // `Ctrl+2` is the control. It is bound to `mode.review`, and digit chords
    // are the ones the application's key table could always spell — the
    // letter chords were the late addition. So a `Ctrl+2` that produces
    // nothing is evidence about the HARNESS, and it is reported as a SKIP,
    // because a check that could not deliver a keystroke has learned nothing
    // about the application.
    driver.press_chord(&[vk::CONTROL], vk::DIGIT_2)?;
    session.settle(12);
    let probe = session.trace()?;
    if !probe
        .events("chord-command")
        .any(|l| l.get("id") == Some("mode.review"))
    {
        return Err(Error::new(format!(
            "the control chord Ctrl+2 (`mode.review`) produced no `chord-command` line, so no keystroke reached the application and nothing below would mean anything. The window reported itself foreground, so this is not the focus guard — the likely causes are that synthetic `keybd_event` input is not reaching this window at all, or that the process needs longer than {} frames before it reads the keyboard. Reported as SKIPPED rather than as a Find failure: a check that types into nothing must never name a feature as the culprit.",
            48
        )));
    }
    report.note("control chord Ctrl+2 arrived, so the input channel works");

    // --- the gesture -------------------------------------------------------
    driver.press_chord(&[vk::CONTROL], vk::F)?;
    // The field takes focus on the frame the bar opens, so the needle cannot
    // be typed in the same breath.
    session.settle(12);
    driver.press(NEEDLE_VK)?;
    session.settle(6);
    driver.press(vk::ENTER)?;
    session.settle(24);

    let trace = session.trace()?;
    if !trace.started(ctx.profile.vocab.start_event) {
        return Err(Error::new(format!(
            "the trace has no `{}` line, so the diagnostic switch {}={} did not reach the \
             process and nothing below could be observed. Captured stderr is at {}.",
            ctx.profile.vocab.start_event,
            ctx.profile.diag_env.0,
            ctx.profile.diag_env.1,
            session.trace_path().display()
        )));
    }

    // --- 1. did the chord dispatch the command? ----------------------------
    let dispatched = trace
        .events("chord-command")
        .any(|l| l.get("id") == Some("edit.find"));
    if !dispatched {
        let chords: Vec<&str> = trace
            .events("chord-command")
            .filter_map(|l| l.get("chord"))
            .collect();
        return Ok(Some(format!(
            "Ctrl+F did not dispatch `edit.find`. Chords the application reported this run: {}. \
             A chord that is in the keymap and reaches nothing is this project's `Ctrl+O` \
             defect — the keymap was right, the command was right, and the key table could not \
             spell a letter chord, so nothing ever looked it up.",
            if chords.is_empty() {
                "none".to_owned()
            } else {
                chords.join(", ")
            }
        )));
    }
    report.note("Ctrl+F dispatched `edit.find`");

    // --- 2. did the bar open? ----------------------------------------------
    let opened = trace
        .events("find-toggled")
        .any(|l| l.get("open") == Some("true"));
    if !opened {
        return Ok(Some(
            "`edit.find` dispatched but the find bar never reported opening. A command that \
             dispatches and does nothing is the `command-unimplemented` shape, and it is why \
             this is asserted separately from the dispatch."
                .to_owned(),
        ));
    }
    report.note("the find bar reported open");

    // --- 3. is it actually on screen? --------------------------------------
    let ui_rect = ctx.profile.vocab.ui_rect_event.unwrap_or("ui-rect");
    let bar = trace
        .events(ui_rect)
        .filter(|l| l.get("name") == Some("find-bar"))
        .filter_map(|l| l.get_rect("rect"))
        // `.last()`, not `.next_back()`: `Trace::events` is forward-only.
        // The LAST is wanted for the reason the QAT check documents — an
        // early frame can carry a rect from before the layout settled, which
        // is exactly the find bar's own one-frame misplacement.
        .last();
    match bar {
        Some(r) if r.width() > 0.0 && r.height() > 0.0 => {
            report.note(format!(
                "find-bar occupies {:.1} x {:.1} pt",
                r.width(),
                r.height()
            ));
        }
        Some(r) => {
            return Ok(Some(format!(
                "the find bar is open and declares a region of {:.1} x {:.1} pt — it is open and \
                 not on screen. Three panels in the old shell shipped with a body and no control \
                 anyone could click, and passed every verification for their whole shipped life.",
                r.width(),
                r.height()
            )));
        }
        None => {
            return Ok(Some(format!(
                "the find bar is open but declared no `{ui_rect} name=find-bar` region, so there \
                 is no evidence it was laid out. Open-but-not-drawn is a state this project has \
                 shipped before."
            )));
        }
    }

    // --- 4. did a search run? ----------------------------------------------
    let Some(search) = trace.events("find").last() else {
        return Ok(Some(
            "the bar opened and took a needle, but no `find` line was traced — so pressing \
             Enter did not run a search. Find deliberately does NOT search on a keystroke \
             (measured at 331-449 ms per search on a dense sheet), which makes Enter the only \
             way in, and therefore the one that has to work."
                .to_owned(),
        ));
    };
    let hits = search.get_usize("hits").unwrap_or(0);
    let needle = search.get("needle").unwrap_or("?");
    report.note(format!("searched {needle} and reported {hits} hit(s)"));

    if hits == 0 {
        // A PASS, and the note says why. Whether a given PDF contains `e` is
        // a property of the fixture, not of the application; failing here
        // would make the check fail whenever it was pointed at a drawing with
        // no extractable text, which is a document type this product exists
        // to open.
        report.note(format!(
            "0 hits: the search RAN, which is what this check is about. Whether `{NEEDLE}` \
             appears in this document is the fixture's business — a scanned or purely vector \
             sheet legitimately has no extractable text. Point the check at a text document to \
             exercise the hit path."
        ));
    }

    Ok(None)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The letter and its virtual key agree.
    ///
    /// Two constants describing one keystroke, so they can disagree — and if
    /// they did, the check would type one character and report having
    /// searched for another, which reads as a Find defect rather than as a
    /// harness bug.
    #[test]
    fn the_needle_and_its_virtual_key_are_the_same_letter() {
        assert_eq!(
            u32::from(NEEDLE_VK),
            NEEDLE.to_ascii_uppercase() as u32,
            "on Windows a letter's virtual key IS its ASCII uppercase code point"
        );
    }

    /// `Ctrl+F`'s parts are the codes Windows uses.
    ///
    /// Pinned because a wrong modifier code does not fail loudly: it sends a
    /// chord nobody bound, the application does nothing, and the check
    /// reports that Find is broken.
    #[test]
    fn the_chord_is_control_plus_f() {
        assert_eq!(vk::CONTROL, 0x11, "VK_CONTROL");
        assert_eq!(vk::F, 0x46, "VK_F is ASCII 'F'");
    }
}
