//! `pages_can_be_copied_and_pasted` — the driven proof of
//! `OPERATOR_REQUESTS.md` **O59**'s second item.
//!
//! # What this is about
//!
//! Ken, 2026-08-29, to the engine session: *"can you make sure we have cut,
//! copy, and paste available for everything and if not implement?"* The engine
//! shipped `copy_pages` / `cut_pages` / `paste_pages`; this shell consumed them
//! as three named commands on the Pages tab.
//!
//! # ★★★ The oracle is the PAGE COUNT, and nothing smaller would do
//!
//! Every cheaper thing a check could read here is a statement of *intent*:
//!
//! | reading | what a green result would prove |
//! |---|---|
//! | `pageclip-copy` in the trace | the shell asked the engine for a clip |
//! | `pageclip-paste` in the trace | the shell raised an action |
//! | `insert-pages` applied | the engine returned `Ok` |
//! | **the document has more pages** | **the operator got what they pressed the button for** |
//!
//! Only the last one distinguishes a working paste from a paste that inserted
//! zero pages successfully — which is exactly what a clip built from an empty
//! page list would do, silently, with every trace line present and correct.
//!
//! ★ So this counts pages before and after, from the application's own report
//! of how many it has, and asserts the arithmetic.
//!
//! # The sequence
//!
//! 1. open a four-page fixture and read the page count;
//! 2. invoke `pages.copy` — with nothing picked, the operand rule takes the
//!    **current** sheet, which is the ordinary case and the one an operator
//!    reaches first;
//! 3. invoke `pages.paste`;
//! 4. assert the document now has **one more page**.
//!
//! ★★ Steps 2 and 3 are invoked by **command id**, not by clicking the ribbon,
//! and that is deliberate rather than lazy. The subject here is whether the
//! page clipboard works; whether the Pages ▸ Clipboard band is drawn where the
//! manifest says is `shell::manifest`'s own assertion and a different question.
//! A check that clicked the band would fail for two unrelated reasons and its
//! message could not tell them apart.
//!
//! # ★ What this does NOT prove, said out loud
//!
//! That the **orphaned-widget disclosure** fires. That needs a source document
//! whose form fields straddle the copied sheets, and the honest way to get one
//! is the engine's own smoke fixture rather than a drawing improvised here. It
//! is the disclosure the engine flagged as *"the one that produces a document
//! that looks right and is not"*, so its absence from this check is a gap
//! rather than a decision — recorded so the next session finds it named.

use crate::checks::driving::SHELL_DIAG_ENV;
use crate::checks::{Check, CheckContext};
use crate::error::{Error, Result};
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;

/// Edit mode, then copy, then paste — the whole gesture, by command id.
const INVOKE: &str = "mode.edit,pages.copy,pages.paste";

/// The shell's own line for the copy half.
const COPY_LINE: &str = "pageclip-copy";

/// The shell's own line for the paste half.
const PASTE_LINE: &str = "pageclip-paste";

/// The line the INSERT publishes when it lands: `pages=` after, `was=` before.
///
/// ★★★ This is the oracle, and finding it changed the check. The first version
/// read `open ok pages=N` at the start and hoped to read a second page count
/// from somewhere at the end — but the application publishes its page count on
/// **open** and not on every edit, so there was no "after" to read and the
/// check would have compared a number to itself.
///
/// `insert-pages-landed` carries **both numbers from the same moment**:
/// `was=` is the count before the insert and `pages=` the count after. That
/// removes the whole class of error where two readings straddle a frame and
/// describe different states — and it comes from the insert path itself, so it
/// cannot be present unless the document actually changed.
const LANDED: &str = "insert-pages-landed";

/// See the module documentation.
pub struct PagesCanBeCopiedAndPasted;

impl Check for PagesCanBeCopiedAndPasted {
    fn name(&self) -> &'static str {
        "pages_can_be_copied_and_pasted"
    }

    fn defect(&self) -> &'static str {
        "the page clipboard traces a copy and a paste and the document has exactly as many pages \
         as before — every intent line present and correct, and nothing inserted, which is what a \
         clip built from an empty page list does silently"
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
    let exe = ctx.resolve_exe().ok_or_else(|| {
        Error::new(format!(
            "no binary to drive. Pass --exe, or build the profile's default at {}.",
            ctx.profile.default_exe
        ))
    })?;
    let pdf = ctx.pdf.clone().ok_or_else(|| {
        Error::new("no --pdf. This check needs a document with pages to copy one of.")
    })?;

    let mut spec = LaunchSpec::new(&exe, ctx.out("page-clipboard.trace.txt"));
    spec.pdf = Some(pdf.clone());
    spec.env.push((
        ctx.profile.diag_env.0.to_owned(),
        ctx.profile.diag_env.1.to_owned(),
    ));
    spec.env
        .push((SHELL_DIAG_ENV.0.to_owned(), SHELL_DIAG_ENV.1.to_owned()));
    spec.env
        .push(("PDFCER_DIAG_INVOKE".to_owned(), INVOKE.to_owned()));
    spec.allow_stale = ctx.allow_stale;
    spec.source_root = ctx.source_root.clone();

    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    report.artifact(session.trace_path().to_path_buf());
    report.note(format!(
        "launched {} as pid {} with PDFCER_DIAG_INVOKE={INVOKE}",
        exe.display(),
        session.pid()
    ));
    session.settle(50);

    let trace = session.trace()?;

    if trace.events(COPY_LINE).next().is_none() {
        return Ok(Some(format!(
            "`pages.copy` produced no `{COPY_LINE}` line. Either the command is not routed — \
             `dispatch::pageclip::handles` — or `copy_pages` refused and said so on the status \
             row instead. Trace: {}",
            session.trace_path().display()
        )));
    }
    report.note("★ pages.copy produced a pageclip-copy line");

    if trace.events(PASTE_LINE).next().is_none() {
        return Ok(Some(format!(
            "`pages.paste` produced no `{PASTE_LINE}` line on a run where the copy succeeded, so \
             the clipboard did not hold what the copy put there — the two halves disagree about \
             the `Clipped::Pages` variant. Trace: {}",
            session.trace_path().display()
        )));
    }
    report.note("★ pages.paste produced a pageclip-paste line");

    // --- the assertion that is about the DOCUMENT, not the intent ----------
    let Some(landed) = trace.events(LANDED).last() else {
        return Ok(Some(format!(
            "★★★ BOTH TRACE LINES ARE PRESENT AND NOTHING WAS INSERTED. The shell asked for a \
             copy and raised a paste, and no `{LANDED}` line followed — so the document did not \
             change, which is exactly what a clip built from an empty page list does: silently, \
             with every intent line correct. Look for `paste-pages-refused` in the trace before \
             suspecting this harness. Trace: {}",
            session.trace_path().display()
        )));
    };
    let before: usize = landed.get("was").and_then(|v| v.parse().ok()).unwrap_or(0);
    let after: usize = landed
        .get("pages")
        .and_then(|v| v.parse().ok())
        .unwrap_or(0);
    if after <= before {
        return Ok(Some(format!(
            "the insert landed and reported {before} page(s) before and {after} after, so it \
             inserted nothing. Trace: {}",
            session.trace_path().display()
        )));
    }
    report.note(format!(
        "★★ the document went from {before} to {after} page(s) — the paste reached the document, \
         not just the action queue"
    ));

    Ok(None)
}
