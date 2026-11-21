//! `word_lists_reach_the_program_that_reads_them` — in File ▸ Recognise
//! text…, with a Tesseract program add-on chosen, *No word lists* and an added
//! word file reach the program: the engine's stand-in answers one extra word
//! for each (`nodawg` when both built-in lists are off, `words:` plus the
//! file's lines for `--user-words`), so the run applies two more words than
//! the plain run does. The word file is answered through
//! `PDFCER_DIAG_WORDS_PATH`. Run with the scripted pointer in a window placed
//! off the desktop.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/ocr_word_lists.md`.

use super::ocr_extra_folder::{click, raw};
use super::ocr_program_addon::{ADDON, STAND_IN_WORDS, launch, open_dialog, plant, stand_in};
use crate::checks::{Check, CheckContext, CheckReport};
use crate::error::{Error, Result};

/// Regions.
const NO_LISTS: &str = "ocr-no-word-lists"; // ui-text-exempt: a trace region name, never displayed
const ADD_WORDS: &str = "ocr-add-words"; // ui-text-exempt: a trace region name, never displayed
const LAYOUT: &str = "ocr-by-layout"; // ui-text-exempt: a trace region name, never displayed
/// The engine's answer for the chosen model.
const OFFER: &str = "ocr-reading-offer"; // ui-text-exempt: a trace event name, never displayed
const RUN: &str = "ocr-run"; // ui-text-exempt: a trace region name, never displayed
/// The seam answering the word-file picker.
const WORDS_PATH: &str = "PDFCER_DIAG_WORDS_PATH";
/// How many 20-frame waits recognition gets.
const RECOGNITION_POLLS: u32 = 40;

/// See the module documentation.
pub struct WordListsReachTheProgramThatReadsThem;

impl Check for WordListsReachTheProgramThatReadsThem {
    fn name(&self) -> &'static str {
        "word_lists_reach_the_program_that_reads_them"
    }

    fn defect(&self) -> &'static str {
        "the Recognise text window offers no word-list choice for Tesseract, or the choice \
         made there never reaches the program"
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
    let exe = ctx
        .resolve_exe()
        .ok_or_else(|| Error::new("no binary to drive. Pass --exe."))?;
    let root = std::path::absolute(ctx.out("ocr-word-lists"))
        .map_err(|e| Error::new(format!("cannot make the folder absolute: {e}")))?;
    plant(&root, &stand_in(ctx)?)?;
    let words = root.join("project-words.txt");
    std::fs::write(&words, "alpha\nbeta\n")
        .map_err(|e| Error::new(format!("cannot write the word file: {e}")))?;
    let words = words.display().to_string();
    let prefs = format!("ocr_folder = {}\nocr_model = {ADDON}\n", root.display());
    let (session, pointer) = launch(
        ctx,
        &exe,
        &prefs,
        "word-lists",
        &[(WORDS_PATH, words.as_str())],
        report,
    )?;
    let ui_rect = ctx.profile.vocab.ui_rect_event.unwrap_or("ui-rect");
    if open_dialog(&session, &pointer, ctx)?.is_none() {
        return Ok(Some(format!(
            "the add-on is in an extra folder and Recognise text does not list `{ADDON}`."
        )));
    }
    let trace = session.trace()?;
    if let Some(failure) = drawn_as_offered(&trace, ui_rect, "true", "true", "false") {
        return Ok(Some(failure));
    }
    for region in [NO_LISTS, ADD_WORDS] {
        if crate::checks::driving::declared(&trace, ui_rect, region).is_none() {
            return Ok(Some(format!(
                "★ Tesseract takes word lists and the window drew no `{region}` for it."
            )));
        }
    }
    click(&session, &pointer, ui_rect, NO_LISTS)?;
    click(&session, &pointer, ui_rect, ADD_WORDS)?;
    session.settle(10);
    click(&session, &pointer, ui_rect, RUN)?;
    let mut trace = session.trace()?;
    for _ in 0..RECOGNITION_POLLS {
        if trace.last("ocr-applied").is_some() || trace.last("ocr-refused").is_some() {
            break;
        }
        session.settle(20);
        trace = session.trace()?;
    }
    let _ = pointer.gone(&session);
    let started = raw(&trace, "ocr-started");
    report.note(format!("`{started}`"));
    let asked = trace.last("ocr-started").is_some_and(|l| {
        l.get("builtin-lists") == Some("false") && l.get("user-words") == Some("1")
    });
    if !asked {
        return Ok(Some(format!(
            "★★ *No word lists* and one word file were chosen, and the run asked for `{started}`."
        )));
    }
    let Some(applied) = trace.last("ocr-applied") else {
        return Ok(Some(format!(
            "★★ the run applied nothing: `{}`.",
            raw(&trace, "ocr-refused")
        )));
    };
    report.note(format!("`{}`", applied.raw));
    let want = STAND_IN_WORDS + 2;
    if applied.get_usize("words") != Some(want) {
        return Ok(Some(format!(
            "★★★ the program answers {STAND_IN_WORDS} words plus one for lists off and one for \
             a word file, so {want}; the run applied `{}`. The choice did not reach it.",
            applied.raw
        )));
    }
    Ok(None)
}

/// The engine's answer for the chosen model, `ocr-reading-offer`, names the
/// choices `check_options` accepts; the window must draw exactly those.
/// `None` when it does and the answer is the one wanted.
pub(super) fn drawn_as_offered(
    trace: &crate::trace::Trace,
    ui_rect: &str,
    drop_lists: &str,
    word_files: &str,
    layout: &str,
) -> Option<String> {
    let Some(offer) = trace.last(OFFER) else {
        return Some(format!(
            "the window traced no `{OFFER}`: the reading choices were not asked of the engine."
        ));
    };
    let want = [
        ("drop-lists", drop_lists),
        ("word-files", word_files),
        ("layout", layout),
    ];
    if offer.get("source") != Some("check_options")
        || want.iter().any(|(k, v)| offer.get(k) != Some(*v))
    {
        return Some(format!(
            "★ the engine's answer for this model should be {want:?} from `check_options`;              the window traced `{}`.",
            offer.raw
        ));
    }
    // Ever drawn: the window closes on Run, which retires its regions.
    let drawn = |r: &str| {
        trace
            .lines
            .iter()
            .any(|l| l.event == ui_rect && l.get("name") == Some(r))
    };
    let pairs = [
        (NO_LISTS, drop_lists),
        (ADD_WORDS, word_files),
        (LAYOUT, layout),
    ];
    pairs
        .iter()
        .find(|(r, v)| drawn(r) != (*v == "true"))
        .map(|(r, v)| {
            format!("★ `check_options` answered {v} for `{r}`, and the window drew otherwise.")
        })
}
