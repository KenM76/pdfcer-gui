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
