//! `a_formatted_comment_says_it_is_formatted` — a sticky note whose body is
//! also stored as rich text (`/RC`) shows a line in the Comments panel naming
//! the formatting, for both forms the spec permits (a string and a stream).
//!
//! Design and rationale: `docs/modules/ui-verify/checks/rich_comment.md`.

use crate::checks::word_styles::launch_invoking;
use crate::checks::{Check, CheckContext};
use crate::error::Result;
use crate::report::CheckReport;

const FIXTURE: &str = "rich-comment.pdf";
/// Review mode, then the Comments panel's show command.
const INVOKE: &str = "mode.review,markup.comments";
const EVENT: &str = "comment-rich-note"; // ui-text-exempt: a trace event name, never displayed
/// The panel's census, whose `rich=` counts rows carrying `/RC`.
const CENSUS: &str = "comments-panel"; // ui-text-exempt: a trace event name, never displayed

/// One note the fixture carries: its object number, and the words its
/// formatted copy must be named by.
struct Expect {
    id: &'static str,
    form: &'static str,
    words: &'static [&'static str],
}

/// Note A: `/RC` as a string with a `/DS`; note B: `/RC` as a stream.
const NOTES: [Expect; 2] = [
    Expect {
        id: "5",
        form: "string",
        words: &["bold", "12 pt", "Helvetica", "#FF0000"],
    },
    Expect {
        id: "6",
        form: "stream",
        words: &["italic"],
    },
];

/// See the module documentation.
pub struct AFormattedCommentSaysItIsFormatted;

impl Check for AFormattedCommentSaysItIsFormatted {
    fn name(&self) -> &'static str {
        "a_formatted_comment_says_it_is_formatted"
    }

    fn defect(&self) -> &'static str {
        "a comment whose note carries formatting shows as plain text and nothing says the \
         formatting exists or that editing the note drops it"
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

fn drive(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    let (d, _doc) = launch_invoking(ctx, report, "rich-comment", FIXTURE, INVOKE)?;
    d.session.settle(20);
    let trace = d.session.trace()?;
    d.pointer.gone(&d.session)?;
    let census = trace.events(CENSUS).last().map(|l| l.raw.clone());
    let rich = trace
        .events(CENSUS)
        .last()
        .and_then(|l| l.get_usize("rich"));
    report.note(format!("census: {census:?}"));
    if rich != Some(2) {
        return Ok(Some(format!(
            "the Comments panel counted {rich:?} rows carrying formatted text on a page with two \
             (`{CENSUS}` rich=). Trace: {}.",
            d.path()
        )));
    }
    for note in &NOTES {
        let line = trace
            .events(EVENT)
            .filter(|l| l.get("id") == Some(note.id))
            .last();
        let Some(line) = line else {
            return Ok(Some(format!(
                "note {} (`/RC` as a {}) drew no `{EVENT}` line. Trace: {}.",
                note.id,
                note.form,
                d.path()
            )));
        };
        report.note(format!("note {}: {}", note.id, line.raw));
        let words = line.get("words").unwrap_or("");
        let missing: Vec<&str> = note
            .words
            .iter()
            .copied()
            .filter(|w| !words.split(", ").any(|got| got == *w))
            .collect();
        if line.get("state") != Some("formatted") || !missing.is_empty() {
            return Ok(Some(format!(
                "note {} (`/RC` as a {}) was described as state={:?} words=\"{words}\"; it holds \
                 {:?}, and {missing:?} went unnamed. Trace: {}.",
                note.id,
                note.form,
                line.get("state"),
                note.words,
                d.path()
            )));
        }
    }
    Ok(None)
}
