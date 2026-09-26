//! `the_print_window_opens_on_the_settings_you_last_used` — **operator request
//! O166, driven.**
//!
//! Design and rationale: `docs/modules/ui-verify/checks/print_remembered.md`.

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
/// **Reset to the bare sandbox seed** before the control run and rewritten
/// before the second — never deleted. Those are not the same act, and the
/// difference cost this check two sweeps: deletion takes `ask_default_app = false`
/// with it, and the symptom is the O173 offer opening a real OS window in front of
/// the very click this check is about to make. See [`sandbox::reset_prefs`].
///
/// Safe only because the suite is **never** pointed at a published build — that
/// is the standing rule, and this check is one of the reasons for it. Pointed at the
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

    // Through the overflow when the ribbon has folded it there. At the
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

    //
    //
    // **What this paragraph used to say, and what its one wrong clause
    // cost.** It said *"`userdata/` is not among the sibling directories it brings,
    // so every check begins with no preferences file of any kind"*, and concluded
    // from that that the delete at the top of this function always finds nothing —
    // which is what made the delete look free. The clause was **true on the day it
    // was written**. `sandbox::seed_prefs` then began writing a
    // `userdata/preferences.txt` holding exactly one key — `ask_default_app =
    // false`, suppressing the O173 startup offer — and **no signature anywhere
    // changed**, so neither the compiler nor any test could see that this comment
    // had become the opposite of the truth. Every check now begins with a
    // preferences file; the delete always found it; and what it removed was the
    // suppression.
    //
    // So why keep it. Because the two runs where it is NOT redundant are
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
    // **RESET to the bare seed, never deleted — and this paragraph used
    // to argue the opposite, at length and persuasively.** The argument it made
    // was: `ui_scale` writes back `1.0` because that is a real, safe, non-absent
    // value of the one key it owns, whereas here the neutral state is *no print
    // preferences at all* — which is what a fresh `userdata` folder has, is the
    // state `PrintPrefs::default` is specified against, and is not something any
    // non-default print value could be left behind to represent. Writing the
    // shipped defaults into the file would additionally be a lie of a different
    // kind, making a later reader think the operator had chosen them.
    //
    // Every clause of that is still true, and it still reached the wrong
    // conclusion, because **the file holds one key that is not a print
    // preference.** `sandbox::reset_prefs` writes the header and nothing else:
    // `ask_default_app = false`, and no print keys whatever. So it delivers
    // *exactly* the neutral state the old argument was reaching for — every print
    // key absent, every one taking its compiled-in default, nothing pinned, no
    // chosen-looking value left behind — while keeping the O173 offer shut.
    // Deletion does not, because an absent `ask_default_app` takes its own
    // compiled-in default, and that one is `true`.
    //
    // **What it cost, measured 2026-09-13 by driving the check.** The
    // control launch's trace carries `dialog-owned title="Open PDFs with pdfcer"
    // owned=true` and `dialog-focus — focused=Some(true)` forty lines ahead of the
    // File-tab click, and the click then produced no `ribbon-tab-activated` line at
    // all: it went to the offer's window, which had the foreground. The check
    // skipped saying *"the click on `ribbon.tab.file` produced no
    // `ribbon-tab-activated tab=file` line, so no click reached the ribbon"*, and
    // **five documents in this repository then recorded that as a ribbon defect** —
    // "the File-tab route", promoted to a suite-wide blocker on the strength of a
    // second check reporting the same sentence. The ribbon was never involved. An
    // absence reported by a check is first a question about the check.
    //
    // ⇒ `sandbox::write_prefs` exists precisely to close this class, and its own
    // doc table names THIS CHECK as one of the three that lost the seed. The repair
    // made then covered the **write** path — the seeding call below goes through
    // `write_prefs` — and not the **delete** path sixty lines above it. *A fix that
    // names its victims can still miss one*, and the one it misses is the one
    // spelled with a different verb.
    //
    //
    // (The exits ABOVE this point deliberately have no guard, and that is not
    // an oversight: everything above resolves arguments — the exe, the PDF,
    // `--no-input`, the ui-rect event, the `userdata` folder — and not one of
    // them has touched the disk yet. A guard placed earlier would rewrite a
    // preferences file this check never wrote — byte-identical to the seed inside
    // a sandbox, and a loss of the operator's own print settings on the two paths
    // above where it is not.)
    //
    // Failure to reset **in the guard** is REPORTED and does not change the
    // verdict: this check's assertions are about the application, and a harness
    // that downgraded a real pass because it could not rewrite a file on its way
    // out would be reporting its own housekeeping as a defect in the program.
    // Failure to reset **before the control run** is a SKIP, which is the opposite
    // treatment and deliberately so — see there.
    struct Neutral<'a>(&'a Path);
    impl Drop for Neutral<'_> {
        fn drop(&mut self) {
            // Reset, not removed. The two paths where this guard is not
            // redundant — `--shared-profile`, and a hand run against a real
            // `--exe` — are exactly the paths where removing the O173 suppression
            // would hand the offer to the NEXT check's window.
            //
            // `exists()` is still the gate, so a file this check never created is
            // never created by its cleanup either.
            if self.0.exists()
                && let Some(userdata) = self.0.parent()
                && let Err(why) = crate::sandbox::reset_prefs(userdata)
            {
                eprintln!(
                    "ui-verify: WARNING — could not reset {} ({why}). It holds the print \
                     settings this check seeded, so a later --shared-profile run will \
                     not be starting from the shipped defaults. Reset it by hand to a \
                     file holding `ask_default_app = false` and nothing else.",
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
    match crate::sandbox::reset_prefs(&dir) {
        Ok(()) => report.note(format!(
            "reset {} to the bare seed, so the control run starts from the shipped \
             print defaults with the O173 offer still suppressed",
            prefs_path.display()
        )),
        // A SKIP, where the old delete treated its own failure as a note and
        // carried on. The asymmetry is the point: a delete that failed left a file
        // whose print keys this check knows nothing about, and the twelve values
        // measured from the launch below would then be somebody else's settings
        // recorded as "the shipped defaults" — a baseline that is wrong without
        // being empty. `write_prefs`' own contract says a caller in a check reports
        // this as a SKIP, because a preference that could not be written means the
        // check never began.
        Err(why) => {
            return Err(Error::new(format!(
                "could not reset {} to the bare seed ({why}), so the control run would \
                 measure whatever that file happens to hold and call it the shipped \
                 defaults.",
                prefs_path.display()
            )));
        }
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
            "★★ the preferences file held no print keys and the dialog still \
             reported `remembered={}` — `{}`.\n\n\
             `none` is the only honest answer when every print key is absent from the \
             file. Anything else means the dialog is being handed a `PrintPrefs` that came \
             from somewhere this check cannot see, and every measurement below it would be \
             against an unknown baseline.",
            control.get("remembered").unwrap_or("<absent>"),
            control.raw
        )));
    }
    let defaults = tokens(&control);
    drop(session);

    // --- the seed ------------------------------------------------------------
    //
    // Written in the writer's own vocabulary, and every value required to
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
    // `print-open` reports what the dialog's fields were set to. A build that
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

    /// **Every seeded value must be spelled the way the file spells it.**
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
