//! `the_print_window_opens_on_the_settings_you_last_used` — **operator request
//! O166, driven.**
//!
//! # The report
//!
//! Ken, 2026-09-10:
//!
//! > *"the printer dialogue box needs to remember our last settings."*
//!
//! Before that day `PrintDialog::open` built every field from a literal, every
//! single time, and the dialog object was dropped when the window closed — so
//! nothing about a print survived even within one session. An operator who
//! prints landscape, two-sided, at 600 dpi re-answered all four questions on
//! every print.
//!
//! Thirteen answers are now written to `userdata/preferences.txt` the moment
//! **Print** is pressed, and read back the next time the window opens. This
//! check is the evidence that the *reading* half works in a running process,
//! which is the half the operator experiences as *"it remembered."*
//!
//! # ★★★ What this check deliberately CANNOT establish, said first
//!
//! **It never presses Print, and no future edit may make it.** Committing is
//! how a print job reaches a real device, and this suite runs unattended on the
//! machine whose default printer is the operator's plotter. Four print checks
//! already state that rule in their own words rather than by reference, because
//! the day somebody adds a fifth by copying one of these files, the copied file
//! is what they will read.
//!
//! `PrintDialog::remember` is called from inside the commit block — deliberately,
//! because closing without printing is how a person says *"not this"* — so the
//! **writing** half is unreachable from here. What stands in for it:
//!
//! | The claim | What holds it |
//! |---|---|
//! | every field of `PrintPrefs` is written | `habits()` builds the struct with **no** `..Default::default()`, so a fourteenth field is a compile error |
//! | every field is read back by `open` | `every_remembered_field_is_read_back_by_the_print_dialog`, which parses the struct's fields out of its own source |
//! | the file survives a round trip | `every_preference_round_trips_through_the_file` and `the_writer_emits_no_key_the_parser_rejects` |
//! | the file the operator gets is the file this check writes | ★ **this check**, below — the seed is written in the writer's own token vocabulary |
//!
//! So: *"press Print and the file is written"* is held by the compiler and
//! three unit tests. *"the file is read and the window opens on it"* is held
//! here, by a running process. Neither half is claimed by the other, and a
//! reader who took this green as covering both would be taking more than is
//! here.
//!
//! # How it tells "read the file" from "agreed with the default"
//!
//! This is the trap `page_display_pref` names in its own header, and it is
//! sharper here because there are twelve values rather than one. A check that
//! seeded `orientation = auto` and found `auto` would pass against a build that
//! never opened the file at all.
//!
//! Two things prevent it:
//!
//! 1. **A control launch, first.** The preferences file is deleted, the program
//!    is started, the Print window is opened, and the twelve shipped defaults
//!    are read off the trace. That is measurement, not assertion — the defaults
//!    are the *application's*, and this file does not get to have an opinion
//!    about what they are.
//! 2. **Every seeded value is then required to differ from the value the
//!    control run reported.** If any one of them agrees, the check reports a
//!    SKIP naming it, rather than a pass — because that field's result would be
//!    the same whether the file was read or ignored.
//!
//! ★ The second rule is what makes the seed maintainable. When a shipped
//! default changes — and `MIN_PRINT_DPI` moved 50 → 36 the day this feature
//! landed — the check does not silently become decorative. It goes yellow and
//! says which value to change.
//!
//! # The oracle
//!
//! `print-open`, emitted once as the dialog is built:
//!
//! ```text
//! print-open printers=4 selected=1 remembered=matched unavailable=None page=0
//!            orientation=landscape duplex=long-edge paper=match-pages tray=true
//!            scale=custom percent=137 markup=document-and-markups dpi=600
//!            copies=3 collate=false subset=odd reverse=true
//! ```
//!
//! Everything after `page=` was added on 2026-09-10 **while writing this
//! check** — the fourth time in this project that sitting down to write a
//! driven check found a trace that could not tell apart the two states the
//! check existed for. Before it, `remembered=` reported the *printer* and
//! nothing else, so a build that restored the printer and silently dropped the
//! other twelve would have shown a fully green line.
//!
//! ★★★ **And then it was wrong a second time, in a way that looked
//! finished.** As first written the twelve fields were formatted from
//! `remembered.*` — the parsed `PrintPrefs` handed *into* the constructor —
//! and the dialog's own fields were assigned forty lines further down in a
//! separate expression. **A trace emitted from the INPUT to a construction
//! proves parsing, not adoption.** Ten of the twelve assertions below were
//! therefore vacuous: they would have held on a build whose struct literal
//! ignored `remembered` entirely, which is precisely the regression O166
//! exists to prevent. Only `orientation` and `duplex` were covered, and only
//! by accident — via the `print-plan` cross-read described below, which comes
//! at the values from the far end.
//!
//! The trace block now sits **after** the struct literal and reads `dialog.*`.
//! Nothing about this check changed; the thing it was pointed at did. That
//! matters more than it sounds, because a check is only ever as good as the
//! trace under it, and this one had no way to say it was reading the wrong
//! end. The defect was found by trying to make the check FAIL and being
//! unable to — see the section immediately below, which is why that section
//! exists.
//!
//! Every token is produced by the **preferences file's own** `*_key` function,
//! not by `{:?}`. That is this project's standing rule about Debug-formatting a
//! field a machine reads, and it has a second benefit here: the seed below is
//! written in the same vocabulary, so the comparison is literal, token for
//! token, rather than against a second spelling free to drift.
//!
//! # ★★★ Falsified, not merely green (2026-09-10)
//!
//! *"A check that cannot fail is not evidence"* is a standing lesson in this
//! project, written after a long-green gate was found to be aiming at
//! nothing. So this one was made to fail on purpose before it was believed,
//! and the record of that is kept here rather than in a commit message,
//! because the next person to doubt this check will be reading this file.
//!
//! **What was planted.** Not the obvious sabotage. Shadowing `remembered`
//! at the top of `open` would break the *parse* as well as the *adoption*,
//! and a check reading the wrong end of the constructor would go red on that
//! too — proving nothing about the correction described above. So the
//! narrow form was planted instead: the preferences file still parsed, the
//! trace's `remembered=` field and the printer lookup still reading it, and
//! **only the struct literal** switched to a `PrintPrefs::default()`.
//! Thirteen field reads, one line of sabotage. That is the exact regression
//! the previous oracle was blind to.
//!
//! **What came back.** `RESULT: FAIL`, naming all twelve fields with the
//! seeded value beside the default that arrived instead — and the failure
//! text's own escalation fired correctly, reporting that twelve-of-twelve
//! *"points at the whole path rather than at one field"* and naming
//! `PrintDialog::open`'s seeding block as one of the two places to look.
//! It was the right answer. The file was then restored from a copy and the
//! check re-run green.
//!
//! ⚠ If a future edit to `PrintDialog::open` makes this check awkward,
//! **re-run that falsification rather than trusting the green.** It costs
//! two release builds and about six minutes, and it is the only thing that
//! has ever caught an oracle pointed at its own input.
//!
//! # ★★ Why `print-plan` is read as well
//!
//! Because `print-open` reports what the dialog's **fields** were set to, and a
//! build that stored the preferences into fields the job planner never consults
//! would pass every assertion above while printing portrait anyway. `print-plan`
//! carries `orientation=` and `duplex=` from `effective_device()` — the values
//! that actually reach the spooler — so the two lines are required to agree.
//!
//! That is the same pairing argument `print_paper` makes about `paper=` beside
//! `sheet=`, one dimension along.
//!
//! # Every way this reports SKIP
//!
//! No binary; `--no-input`; no `--pdf` (the Print command is gated on a document
//! being open, so the ribbon control is greyed and there is no dialog to reach);
//! no ui-rect channel; the ribbon control not declared; the dialog not opening;
//! the spooler refusing on this machine; or a seeded value that turns out to
//! equal the shipped default. Each says which, and none of them is reported as
//! a pass.

use std::path::{Path, PathBuf};

use crate::checks::driving::{
    ITEM_PREFIX, SHELL_DIAG_ENV, TAB_EVENT, declared, declared_names, list, shell_trace,
};
use crate::checks::{Check, CheckContext};
use crate::error::{Error, Result};
use crate::input::Driver;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;

/// The ribbon control that opens the dialog, and the tab it lives on.
const SUBJECT: &str = "ribbon.item.file.print";
const TAB_ID: &str = "file";
const TAB: &str = "ribbon.tab.file";

/// The line the dialog emits once, when it is built. The whole oracle.
const OPEN_EVENT: &str = "print-open";

/// The per-frame line carrying what the *job* was planned with.
const PLAN_EVENT: &str = "print-plan";

/// The preferences file, beside the executable under test.
///
/// ★ Deleted before the control run and rewritten before the second. Safe only
/// because the suite is **never** pointed at a published build — that is the
/// standing rule, and this check is one of the reasons for it. Pointed at the
/// operator's own install it would overwrite his real print settings.
const PREFS_FILE: &str = "preferences.txt";

/// **The seed: twelve `key = value` pairs, and the token each must come back
/// as.**
///
/// The two columns are deliberately the same string. The trace spells every
/// remembered value with the preferences file's own `*_key` function, so the
/// value written into the file and the token read out of the trace are
/// identical by construction — and a build that started translating one of them
/// on the way through would be caught by that identity rather than by a second
/// table here that could be edited to agree with the defect.
///
/// The third column is the trace field the token appears under, which is *not*
/// always the file's key: the file says `print_markup` and the trace says
/// `markup=`, the file says `print_tray_by_page_size` and the trace says
/// `tray=`. Those are two vocabularies with different constraints — a
/// hand-editable file wants a self-describing key, a whitespace-split trace
/// wants a short one — and this table is the seam.
///
/// ⚠ **`print_printer` is not here.** A printer name is machine-specific and a
/// name that does not resolve falls silently back to the Windows default, which
/// is correct behaviour and would make this row unassertable. `remembered=` is
/// checked instead: `none` on the control run, and a *seeded* run leaves it
/// `none` too because no name is seeded. The printer-by-name path is exercised
/// by `print_dialog`'s own selection assertions.
const SEED: [(&str, &str, &str); 12] = [
    ("print_orientation", "landscape", "orientation"),
    ("print_duplex", "long-edge", "duplex"),
    ("print_paper", "match-pages", "paper"),
    ("print_tray_by_page_size", "true", "tray"),
    ("print_scale", "custom", "scale"),
    ("print_custom_percent", "137", "percent"),
    ("print_markup", "document-and-markups", "markup"),
    ("print_max_dpi", "600", "dpi"),
    ("print_copies", "3", "copies"),
    ("print_collate", "false", "collate"),
    ("print_subset", "odd", "subset"),
    ("print_reverse", "true", "reverse"),
];

/// See the module documentation.
pub struct ThePrintWindowOpensOnTheSettingsYouLastUsed;

impl Check for ThePrintWindowOpensOnTheSettingsYouLastUsed {
    fn name(&self) -> &'static str {
        "the_print_window_opens_on_the_settings_you_last_used"
    }

    fn defect(&self) -> &'static str {
        "the Print window opens on pdfcer's defaults rather than on the settings the operator \
         last used, even though they are in the preferences file"
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

/// The `userdata` folder belonging to the binary under test.
fn userdata(exe: &Path) -> Option<PathBuf> {
    exe.parent().map(|dir| dir.join("userdata"))
}

/// Build a launch spec that opens `pdf` with both diagnostic channels on.
fn spec_for(ctx: &CheckContext, exe: &Path, pdf: &Path, trace: &str) -> LaunchSpec {
    let mut spec = LaunchSpec::new(exe, ctx.out(trace));
    spec.pdf = Some(pdf.to_path_buf());
    spec.env.push((
        ctx.profile.diag_env.0.to_owned(),
        ctx.profile.diag_env.1.to_owned(),
    ));
    spec.env
        .push((SHELL_DIAG_ENV.0.to_owned(), SHELL_DIAG_ENV.1.to_owned()));
    spec.allow_stale = ctx.allow_stale;
    spec.source_root = ctx.source_root.clone();
    spec
}

/// Launch, open the Print window, and return the session with its `print-open`
/// line already traced.
///
/// Factored out because this check does it twice with different files on disk,
/// and the two runs must reach the dialog by **identical** means — a control
/// that arrived through a different route would not be a control.
fn launch_and_open(
    ctx: &CheckContext,
    exe: &Path,
    pdf: &Path,
    trace_name: &str,
    ui_rect: &str,
    report: &mut CheckReport,
) -> Result<Session> {
    let session = Session::launch(
        &spec_for(ctx, exe, pdf, trace_name),
        ctx.profile.trace_prefix,
    )?;
    report.artifact(session.trace_path().to_path_buf());
    // Long: the process opens a document before it draws a ribbon.
    session.settle(40);
    session.maximize();
    session.settle(12);

    let trace = session.trace()?;
    if !trace.started(ctx.profile.vocab.start_event) {
        return Err(Error::new(format!(
            "the trace has no `{}` line, so {}={} did not reach the process. Captured stderr is \
             at {}.",
            ctx.profile.vocab.start_event,
            ctx.profile.diag_env.0,
            ctx.profile.diag_env.1,
            session.trace_path().display()
        )));
    }

    let driver = Driver::new(session.window());
    let tab = declared(&trace, ui_rect, TAB).ok_or_else(|| {
        Error::new(format!(
            "the application declared no `{TAB}` region. Tabs declared: {}.",
            list(&declared_names(&trace, ui_rect, "ribbon.tab."))
        ))
    })?;
    driver.click_at(session.frame()?.declared_center(tab))?;
    session.settle(12);
    if !shell_trace(&session)?
        .events(TAB_EVENT)
        .any(|l| l.get("tab") == Some(TAB_ID))
    {
        return Err(Error::new(format!(
            "the click on `{TAB}` produced no `{TAB_EVENT} tab={TAB_ID}` line, so no click \
             reached the ribbon."
        )));
    }

    // ★ Through the overflow when the ribbon has folded it there. At the
    // harness's window width the File tab correctly folds its rightmost groups
    // — Print among them — into the overflow menu, and a lookup that read only
    // the tab surface reports "Print is missing", which is false. The same
    // mistake stood as `print_paper`'s FAIL for days.
    let Some(control) =
        crate::checks::driving::declared_or_in_overflow(&session, &driver, ui_rect, SUBJECT)?
    else {
        let trace = session.trace()?;
        return Err(Error::new(format!(
            "the File tab is active and neither it nor its overflow declares `{SUBJECT}`. \
             Controls declared: {}. That is `print_dialog`'s defect, not this one — it is \
             reported there.",
            list(&declared_names(&trace, ui_rect, ITEM_PREFIX))
        )));
    };
    driver.click_at(session.frame()?.declared_center(control))?;
    // Enumerating printers touches the spooler, which BLOCKS on a network
    // printer, and the dialog also enumerates the selected device's forms. Same
    // settle as `print_paper`'s, for the same reason.
    session.settle(40);

    Ok(session)
}

/// The `print-open` line, or a message saying which of the three causes applies.
fn open_line(session: &Session) -> Result<crate::trace::TraceLine> {
    let trace = session.trace()?;
    let line = trace.events(OPEN_EVENT).next().cloned().ok_or_else(|| {
        Error::new(format!(
            "the click on `{SUBJECT}` produced no `{OPEN_EVENT}` line, so the dialog never \
             opened. That is `print_dialog`'s subject and it reports the three causes apart; \
             nothing about remembered settings can be learned here."
        ))
    })?;
    if line.get("unavailable").unwrap_or("<absent>") != "None" {
        return Err(Error::new(
            "the spooler refused on this machine, so the dialog has no device and several of \
             its controls are not drawn. Reported as SKIPPED: a refused enumeration proves \
             nothing either way about what was remembered.",
        ));
    }
    Ok(line)
}

/// Read the twelve tokens off a `print-open` line, in `SEED` order.
fn tokens(line: &crate::trace::TraceLine) -> Vec<String> {
    SEED.iter()
        .map(|(_, _, field)| line.get(field).unwrap_or("<absent>").to_owned())
        .collect()
}

#[allow(clippy::too_many_lines)]
fn drive(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    let exe = ctx.resolve_exe().ok_or_else(|| {
        Error::new(format!(
            "no binary to drive. Pass --exe, or build the profile's default at {}.",
            ctx.profile.default_exe
        ))
    })?;
    let pdf = ctx.pdf.clone().ok_or_else(|| {
        Error::new(
            "no --pdf. `file.print` is gated on `doc.open`, so with nothing open the control is \
             greyed and there is no dialog to reach.",
        )
    })?;
    if !ctx.allow_input {
        return Err(Error::new(
            "input is disabled (--no-input), and this check is two launches and four clicks. \
             Reported as SKIPPED rather than passed — a check that did not run has learned \
             nothing.",
        ));
    }
    let ui_rect = ctx.profile.vocab.ui_rect_event.ok_or_else(|| {
        Error::new(format!(
            "the `{}` profile declares no ui-rect trace event, so the ribbon's Print control \
             cannot be found.",
            ctx.profile.name
        ))
    })?;
    let Some(dir) = userdata(&exe) else {
        return Err(Error::new(
            "the executable under test has no parent directory, so its `userdata` folder cannot \
             be located.",
        ));
    };
    let prefs_path = dir.join(PREFS_FILE);

    // ★★★ **Delete the seeded preferences file on EVERY path out of this
    // check** — and the honest account of what that is worth is worth more
    // than the rule it enacts, because the first draft of this comment claimed
    // a danger that **does not exist under the way the suite actually runs**.
    //
    // ★ What was MEASURED (`sandbox.rs`, 2026-09-10). Isolation is ON by
    // default. Before each check runs, `Sandbox::for_check` makes a private
    // directory beside the binary, hard-links the binary into it, copies in
    // `models/` — and **`userdata/` is not among the sibling directories it
    // brings**, so every check begins with no preferences file of any kind.
    // `ctx.exe` is then rewritten to the sandboxed path, which is what
    // `userdata()` below resolves against, and `drop(sandbox)` removes the
    // whole directory afterwards. Under a default `run-all`, therefore, this
    // guard deletes a file inside a directory that is about to be deleted
    // anyway, and the delete at the top of this function always finds nothing.
    //
    // ★★ So why keep it. Because the two runs where it is NOT redundant are
    // exactly the two where losing the file would cost the most:
    //
    //   1. `--shared-profile`, where every check writes to ONE `userdata/`
    //      beside the binary. That flag exists to reproduce a run made before
    //      isolation existed, and a person reaching for it is already
    //      debugging something confusing. `print_orientation = landscape` and
    //      `print_copies = 3` left behind for the next hundred checks is not
    //      what they should find.
    //   2. A hand run of the form `--check the_print_window_opens_… --exe
    //      <somewhere real>`, which is how a defect in this check will actually
    //      be investigated. If isolation fails for any reason — a read-only
    //      directory, a link that cannot be made — the harness reports it and
    //      skips, but a `--shared-profile` retry against a real install is the
    //      obvious next move, and that install's `userdata/` holds settings a
    //      person chose.
    //
    // ★ **Deleted rather than restored**, which is the opposite of what
    // `ui_scale` does with the same file, and the difference is deliberate.
    // `ui_scale` writes back `1.0` because that is a real, safe, non-absent
    // value of the one key it owns. Here the neutral state is *no print
    // preferences at all*: that is what a fresh `userdata` folder has, it is
    // the state `PrintPrefs::default` is specified against, and there is no
    // non-default print value that would be safe to leave behind. Rewriting
    // the shipped defaults into the file would also be a lie of a different
    // kind — it would make a later reader think the operator had chosen them.
    //
    // ★ A guard rather than a line at the end, because there is more than one
    // way out below. **Measured 2026-09-10: eight explicit exits — four FAILs
    // (`Ok(Some(…))`), three SKIPs (`Err(Error::new(…))`) and the tail
    // `Ok(None)` — plus five `?` operators, each an exit too and none of them
    // visible as one when reading down the page.** A tidy-up line at the end
    // is correct on exactly one of those thirteen.
    //
    // (The exits ABOVE this point deliberately have no guard, and that is not
    // an oversight: everything above resolves arguments — the exe, the PDF,
    // `--no-input`, the ui-rect event, the `userdata` folder — and not one of
    // them has touched the disk yet. A guard placed earlier would delete a
    // preferences file this check never wrote.)
    //
    // Failure to delete is REPORTED and does not change the verdict: this
    // check's assertions are about the application, and a harness that
    // downgraded a real pass because it could not remove a file would be
    // reporting its own housekeeping as a defect in the program.
    struct Neutral<'a>(&'a Path);
    impl Drop for Neutral<'_> {
        fn drop(&mut self) {
            if self.0.exists()
                && let Err(why) = std::fs::remove_file(self.0)
            {
                eprintln!(
                    "ui-verify: WARNING — could not delete {} ({why}). It holds the print \
                     settings this check seeded, so a later --shared-profile run will \
                     not be starting from the shipped defaults. Delete it by hand.",
                    self.0.display()
                );
            }
        }
    }
    let _neutral = Neutral(&prefs_path);

    // --- process 1: the CONTROL, with no preferences file at all -------------
    //
    // What the shipped defaults are is measured here rather than asserted in
    // this file. They belong to the application, and a check that hard-coded
    // them would go quietly wrong the day one moved — `MIN_PRINT_DPI` moved
    // 50 → 36 on the day this feature landed.
    match std::fs::remove_file(&prefs_path) {
        Ok(()) => report.note(format!(
            "deleted {} so the control run starts from the shipped defaults",
            prefs_path.display()
        )),
        Err(_) => report.note(format!(
            "{} did not exist; the control run starts from the shipped defaults anyway",
            prefs_path.display()
        )),
    };

    let session = launch_and_open(
        ctx,
        &exe,
        &pdf,
        "print-remembered-1.trace.txt",
        ui_rect,
        report,
    )?;
    report.note(format!("control process: pid {}", session.pid()));
    let control = open_line(&session)?;
    report.note(format!("control: `{}`", control.raw));

    if control.get("remembered") != Some("none") {
        return Ok(Some(format!(
            "★★ the preferences file was deleted and the dialog still reported \
             `remembered={}` — `{}`.\n\n\
             `none` is the only honest answer with no file on disk. Anything else means the \
             dialog is being handed a `PrintPrefs` that came from somewhere this check cannot \
             see, and every measurement below it would be against an unknown baseline.",
            control.get("remembered").unwrap_or("<absent>"),
            control.raw
        )));
    }
    let defaults = tokens(&control);
    drop(session);

    // --- the seed ------------------------------------------------------------
    //
    // ★ Written in the writer's own vocabulary, and every value required to
    // DIFFER from what the control run just reported. A seeded value that
    // happened to equal the default would come back correct whether the file
    // was read or ignored, so it is a skip rather than a pass — see the module
    // header.
    let mut agreed: Vec<String> = Vec::new();
    for (index, (key, value, field)) in SEED.iter().enumerate() {
        if defaults.get(index).map(String::as_str) == Some(*value) {
            agreed.push(format!("{key} = {value} (trace `{field}=`)"));
        }
    }
    if !agreed.is_empty() {
        return Err(Error::new(format!(
            "the seed is no longer a seed: {} of the 12 values this check writes already equal \
             the shipped default, so those fields would read back correctly whether or not the \
             file was consulted. Reported as SKIPPED rather than passed. Change the value(s) in \
             `SEED` to something the application does not ship: {}.",
            agreed.len(),
            list(&agreed)
        )));
    }

    let mut text = String::from(
        "# written by tools/ui-verify, check the_print_window_opens_on_the_settings_you_last_used\n",
    );
    for (key, value, _) in SEED {
        text.push_str(key);
        text.push_str(" = ");
        text.push_str(value);
        text.push('\n');
    }
    // ★ Through `sandbox::write_prefs`, which carries the O173 suppression as a
    // header. Writing this file directly used to drop it, which opened the
    // default-app offer in front of this check's own window — see that
    // function for the full account.
    if let Err(why) = crate::sandbox::write_prefs(&dir, &text) {
        return Err(Error::new(format!(
            "could not write {}: {why}",
            prefs_path.display()
        )));
    }
    report.note(format!(
        "seeded {} with 12 print settings, none of which is this build's default",
        prefs_path.display()
    ));

    // --- process 2: the same route, over the seeded file ---------------------
    let session = launch_and_open(
        ctx,
        &exe,
        &pdf,
        "print-remembered-2.trace.txt",
        ui_rect,
        report,
    )?;
    report.note(format!("seeded process: pid {}", session.pid()));
    let line = open_line(&session)?;
    report.note(format!("seeded: `{}`", line.raw));

    let mut wrong: Vec<String> = Vec::new();
    for (index, (key, value, field)) in SEED.iter().enumerate() {
        let got = line.get(field).unwrap_or("<absent>");
        if got != *value {
            wrong.push(format!(
                "{field}={got} (the file says `{key} = {value}`, this build's default is `{}`)",
                defaults.get(index).map_or("<unmeasured>", String::as_str)
            ));
        }
    }

    if !wrong.is_empty() {
        let all = wrong.len() == SEED.len();
        return Ok(Some(format!(
            "★★★ {} OF THE 12 REMEMBERED SETTINGS DID NOT COME BACK.\n\n{}\n\n\
             {}\n\n\
             The file on disk is {} and it was written by this check, in the same token \
             vocabulary `Prefs::write_to_string` uses, so a value that did not arrive was \
             either not parsed (`Prefs::parse`'s `print_*` arms) or parsed and not adopted \
             (`PrintDialog::open`'s seeding block). Trace: {}.",
            wrong.len(),
            wrong
                .iter()
                .map(|w| format!("  • {w}"))
                .collect::<Vec<_>>()
                .join("\n"),
            if all {
                "★ ALL TWELVE, which points at the whole path rather than at one field: either \
                 the file is not being read at this point in start-up, or `open_print` is being \
                 handed a `PrintPrefs::default()` instead of the application's. Check \
                 `app::dispatch`'s `file.print` arm — it must pass `&self.prefs.print`."
            } else {
                "★ Some but not all, which points at individual arms rather than at the path: \
                 the fields that DID arrive prove the file was read and adopted. Look at each \
                 named key in `Prefs::parse` and at the matching field in `PrintDialog::open`; \
                 a key written and never parsed is exactly the gap \
                 `the_writer_emits_no_key_the_parser_rejects` exists to prevent, so if that \
                 test is green the fault is more likely in the adoption."
            },
            prefs_path.display(),
            session.trace_path().display()
        )));
    }

    report.note("all 12 remembered settings came back into the dialog's own fields");

    // --- and they reached the PLAN, not just the fields -----------------------
    //
    // ★★ `print-open` reports what the dialog's fields were set to. A build that
    // stored the preferences into fields the job planner never consults would
    // pass every assertion above and print portrait anyway. `print-plan` carries
    // `orientation=` and `duplex=` from `effective_device()` — the values that
    // actually reach the spooler.
    //
    // ⚠ Only these two of the twelve are on `print-plan`, so this is a spot
    // check rather than a second full pass, and it is reported as such. The
    // others have no per-frame line to compare against; adding one would be a
    // trace change made to satisfy a check rather than to disclose something,
    // which is the wrong direction.
    let trace = session.trace()?;
    let Some(plan) = trace.events(PLAN_EVENT).last() else {
        return Ok(Some(format!(
            "the dialog is open, reported all 12 settings restored, and traced no \
             `{PLAN_EVENT}` line — so nothing says what the job was actually planned with. \
             Trace: {}.",
            session.trace_path().display()
        )));
    };
    // `print-plan` spells these with `{:?}` on the dialog's own enums, so the
    // words differ from the file's tokens by design: the file is hand-editable
    // and the plan line is a dump of the type. Mapped here rather than
    // "fixed", because `print-plan` predates the preferences file by weeks and
    // three other checks read it.
    const PLAN_WORDS: [(&str, &str, &str); 2] = [
        ("orientation", "landscape", "Landscape"),
        ("duplex", "long-edge", "LongEdge"),
    ];
    let mut unplanned: Vec<String> = Vec::new();
    for (field, file_token, plan_word) in PLAN_WORDS {
        let got = plan.get(field).unwrap_or("<absent>");
        if got != plan_word {
            unplanned.push(format!(
                "{field}={got}, where the restored setting is `{file_token}` (`{plan_word}`)"
            ));
        }
    }
    if !unplanned.is_empty() {
        return Ok(Some(format!(
            "★★★ THE SETTINGS WERE RESTORED AND THE JOB WAS NOT PLANNED WITH THEM: {}.\n\n\
             `{OPEN_EVENT}` reports all 12 values came back into the dialog's fields, and \
             `{PLAN_EVENT}` reports the job being laid out with something else. That is the \
             worse half of this defect, because the window looks entirely correct: the radios \
             are where the operator left them and the paper comes out portrait.\n\n\
             `PrintDialog::effective_device` is what the plan reads; the restored values go \
             into `self.device`. Trace: {}.",
            list(&unplanned),
            session.trace_path().display()
        )));
    }

    report.note(format!(
        "★★★ the Print window opened on all 12 remembered settings, and the job was planned \
         with them — `{}`",
        plan.raw
    ));
    Ok(None)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// ★★★ **Every seeded value must be spelled the way the file spells it.**
    ///
    /// The seed is written straight into `preferences.txt` and compared straight
    /// against the trace, and both sides use the preferences module's `*_key`
    /// vocabulary. A value here that the parser does not recognise would be
    /// dropped with a `PrefNote::BadValue`, the field would come back as the
    /// default, and this check would report a defect in the application that is
    /// really a typo in this file.
    ///
    /// This test cannot call `pdfcer-gui`'s parser — `ui-verify` does not depend
    /// on it — so it pins the shape instead: every token is non-empty, lower
    /// case, and free of the whitespace that would split it into two trace
    /// fields.
    #[test]
    fn every_seeded_value_is_a_single_file_token() {
        for (key, value, field) in SEED {
            assert!(!value.is_empty(), "{key} has an empty value");
            assert!(
                !value.contains(char::is_whitespace),
                "{key} = {value} contains whitespace, so the trace would split it into two \
                 fields and `get(\"{field}\")` would return only the first"
            );
            assert_eq!(
                value.to_ascii_lowercase(),
                value,
                "{key} = {value} is not lower case; every file token in \
                 `app::prefs::printing` is"
            );
        }
    }

    /// **Every seeded key is a `print_` key.**
    ///
    /// The file this check writes replaces the operator's whole preferences
    /// file. Seeding a non-print key would be this check quietly changing
    /// something outside its subject — and, worse, changing it for every check
    /// that runs after it in the suite, since `userdata` is shared.
    #[test]
    fn the_seed_touches_only_printing() {
        for (key, _, _) in SEED {
            assert!(
                key.starts_with("print_"),
                "{key} is not a printing preference; this check must not write one"
            );
        }
    }

    /// **The trace field names are distinct.**
    ///
    /// Two rows naming the same trace field would make one of them unassertable
    /// and the other one duplicated, and the count in the failure message would
    /// still read as 12.
    #[test]
    fn each_seeded_row_reads_a_different_trace_field() {
        let mut seen: Vec<&str> = Vec::new();
        for (_, _, field) in SEED {
            assert!(
                !seen.contains(&field),
                "trace field `{field}` appears twice"
            );
            seen.push(field);
        }
        assert_eq!(
            seen.len(),
            12,
            "the seed covers 12 of the 13 remembered settings; the \
             thirteenth is the printer, which is machine-specific — see the module header"
        );
    }

    /// **No key is seeded twice.**
    #[test]
    fn each_seeded_row_writes_a_different_key() {
        let mut seen: Vec<&str> = Vec::new();
        for (key, _, _) in SEED {
            assert!(!seen.contains(&key), "preference key `{key}` appears twice");
            seen.push(key);
        }
    }
}
