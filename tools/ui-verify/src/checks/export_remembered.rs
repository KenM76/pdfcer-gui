//! `the_export_windows_open_on_the_settings_you_last_used` — **operator
//! request O196, driven.**
//!
//! # The report
//!
//!
//! > *"the export windows forget everything. every time I export a dxf I have
//! > to set it up again."*
//!
//! Twelve answers across three windows — Export image, Export text, Export to
//! DXF — were literals in three constructors until that day. An operator who
//! exports millimetre DXFs with the text omitted answered both questions again
//! on every export, on every drawing, for the whole life of the program.
//!
//! The twelve are now written to `userdata/preferences.txt` when Export is
//! pressed, and read back the next time each window opens. **This check is the
//! evidence that the reading half works in a running process**, which is the
//! half the operator experiences as *"it remembered"*.
//!
//! # ★★★ What this check deliberately CANNOT establish, said first
//!
//! **It never presses Export, and no future edit may make it.** Committing
//! opens a native save picker — a hard wall for synthetic input — and then
//! writes a file into a directory this harness has no business choosing. The
//! producing half (`habits()` on each of the three dialogs) is unreachable
//! from here, exactly as the Print window's committing half is unreachable
//! from `print_remembered`, and for a related reason: closing without
//! exporting is how a person says *"not this"*, so the write is inside the
//! commit block where it belongs.
//!
//! What stands in for it:
//!
//! | The claim | What holds it |
//! |---|---|
//! | every field of each `Export*Prefs` is written | each `habits()` is one struct literal with **no** `..Default::default()`, so a field added to the prefs type is a compile error there |
//! | every field is read back by `open` | the three dialogs' own unit tests, plus the trace this check reads, which is formatted from `dialog.*` and not from `remembered.*` |
//! | the file survives a round trip | `app::prefs`' round-trip and writer/parser-agreement tests |
//! | the file the operator gets is the file this check writes | ★ **this check**, below — the seed is written in the writer's own token vocabulary |
//!
//! # ★★ Why this one needs no mouse, and what that buys
//!
//! `print_remembered` reaches its window by clicking a ribbon tab and then a
//! ribbon control. That costs it two skip reasons it cannot avoid —
//! `--no-input`, and the control having folded into the ribbon overflow — and
//! makes it unrunnable on a machine whose desktop is in use.
//!
//! This check reaches all three windows through **`PDFCER_DIAG_INVOKE`**,
//! which takes a comma-separated list of command ids, **one rung per frame**,
//! each dispatched through the same `dispatch_command` choke point a keystroke
//! reaches. One launch therefore opens all three windows, and each emits its
//! own `-open` line as it is built. No pointer is moved, no key is pressed,
//! and the check runs identically under `--no-input`.
//!
//! ⚠ **This is why `frame_of` is absent from this file**, and the absence is
//! deliberate rather than an oversight. The standing rule — *inside a
//! dialogue, use `frame_of(&session, &trace, ui_rect, NAME)` and never
//! `session.frame()`* — exists because a dialogue is a second OS viewport, so
//! a rect measured against the main window's frame is measured from the wrong
//! origin. It does not bite here because **this check measures no rect at
//! all**: its entire oracle is three trace lines. An edit that starts
//! measuring a rect inherits that rule the moment it does.
//!
//! # How it tells "read the file" from "agreed with the default"
//!
//! The same trap `print_remembered`'s header names, and it is sharper here for
//! a reason worth stating: **`ExportDxfPrefs::default()` is field-for-field
//! identical to `DxfOptions::default()`.** A check that hard-coded its
//! expectations and seeded `units = inches` would pass against a build that
//! never opened the preferences file, never parsed a key, and ignored
//! `remembered` entirely — which is the exact regression O196 exists to
//! prevent.
//!
//! Two things prevent it:
//!
//! 1. **A control launch, first.** The preferences file is reset to the bare
//!    sandbox seed — which carries no export keys at all, so every one of the
//!    twelve takes its compiled-in default — the three windows are opened, and
//!    the twelve shipped defaults are read off the trace. That is measurement,
//!    not assertion: the defaults belong to the application, and this file
//!    does not get to have an opinion about what they are.
//! 2. **Every seeded value is then required to differ from the value the
//!    control run reported.** If any one agrees, the check reports a SKIP
//!    naming it rather than a pass, because that field would read back
//!    correctly whether the file was consulted or ignored.
//!
//! ★ The second rule is what keeps the seed maintainable. When a shipped
//! default moves, this check does not silently become decorative: it goes
//! yellow and says which value to change.
//!
//! # The oracle: three lines, twelve tokens
//!
//! ```text
//! export-image-open page=0 pages=9 format=emf scope=all dpi=150 transparent=0 quality=72
//! export-text-open page=0 pages=9 scope=current separator=marker endings=windows bom=1
//! export-dxf-open page=0 groups=0 suggestion=uncalibrated scale=1 units=millimetres arcs=0 text=omit
//! ```
//!
//!
//! # ★★ Why the seed table has five columns where `print_remembered`'s has three
//!
//! Its table can make the file value and the trace token the same string by
//! construction, because both sides are spelled by a `*_key` function. Three
//! of these twelve are not:
//!
//! | key | file | trace |
//! |---|---|---|
//! | `export_image_transparent` | `true` / `false` | `1` / `0` |
//! | `export_text_byte_order_mark` | `true` / `false` | `1` / `0` |
//! | `export_dxf_fit_arcs` | `true` / `false` | `1` / `0` |
//!
//! The file spells a bool with `opening::bool_key`; the three traces spell it
//! with `u8::from`. Neither is wrong — a hand-editable file wants the word,
//! and a whitespace-split trace field is happier with a digit — but a table
//! that assumed they agreed would report three defects that do not exist.
//! Hence the fifth column, and
//! `only_the_boolean_rows_translate_between_the_file_and_the_trace`, which
//! pins the translation so a fourth spelling cannot arrive by hand.
//!
//! ⚠ And `scope` appears on **two different events** — the image window's and
//! the text window's — with different shipped defaults (`current` and `all`).
//! A row is therefore identified by `(event, field)` and never by `field`
//! alone; `each_seeded_row_reads_a_different_event_and_field` is the guard,
//! and it asserts the duplication rather than merely tolerating it.
//!
//! # ★★★ The DXF calibration trap, which is a feature and not a defect
//!
//! `dialogs::export_dxf::seeded_options` does two things in order: ① it takes
//! the operator's habit from the file, then ② **a calibrated ce dimension
//! group on the page overrules the remembered units**, because a metric page
//! exported in inches is a DXF that opens cleanly, measures consistently, and
//! is wrong by 25.4× — discovered by whoever cuts from it.
//!
//! So on a document whose page carries a calibrated group, `units=` reports
//! the page's units no matter what the file says, and a check that asserted
//! its seed would report a defect where the application is doing the single
//! most important thing that window does.
//!
//! This check reads `suggestion=` off the control run's `export-dxf-open`
//! line. When it reads `calibrated`, the `export_dxf_units` row is dropped
//! from both the seed-differs test and the comparison, and the reason is
//! reported as a note naming the document. The other eleven are unaffected,
//! and the count in the pass note says eleven so that nobody reads the green
//! as covering twelve.
//!
//! # Every way this reports SKIP
//!
//! No binary; no `--pdf` (all three commands decline with nothing open, so
//! there would be no window and no line); the preferences file could not be
//! reset; the process never saw `PDFCER_DIAG`; one of the three `-open` lines
//! absent; a seeded value that turns out to equal the shipped default; or the
//! two runs disagreeing about `suggestion=`, which would mean the control and
//! the seeded measurement were taken against different pages. Each says which,
//! and none of them is reported as a pass.

//! # Driven, and falsified, on 2026-09-14
//!
//! Against `target/release/pdfcer-gui.exe` at `1ee4d7a`, over
//! `fixtures/a1-titleblock.pdf` (one page, no calibrated ce dimension group,
//! so `suggestion=uncalibrated` and all twelve were asserted), under
//! `--no-input`.
//!
//! **The control run measured these defaults, and they are recorded here as
//! evidence rather than as an expectation** — the check re-measures them every
//! time and this paragraph binds nothing:
//!
//! ```text
//! export-image-open page=0 pages=1 format=png scope=current dpi=300 transparent=1 quality=90
//! export-text-open  page=0 pages=1 scope=all separator=form-feed endings=as-extracted bom=0
//! export-dxf-open   page=0 groups=0 suggestion=uncalibrated scale=1 units=inches arcs=1 text=entities
//! ```
//!
//! Every one of the twelve seeded values differs from its measured default, so
//! no row was skipped, and the seeded run returned all twelve.
//!
//! ## The sabotage, because a green first run is not evidence
//!
//! ★ The NARROW one, deliberately: **only `export_text.rs`'s struct literal**
//! was switched to take `ExportTextPrefs::default()` for its four fields, the
//! release binary was rebuilt, and the check was re-driven. It returned
//! `RESULT: FAIL` naming **4 of the 12** — the four text rows, each with its
//! file value, its expected token, and the measured default it had fallen back
//! to — and selected the middle escalation:
//!
//! > *"Every failure is on ONE window's line and the other two windows
//! > restored correctly, which proves the file was read, parsed and adopted.
//! > The fault is in that window's constructor."*
//!
//! Which is exactly where the sabotage was. The other two windows stayed
//! green throughout, which is the part that matters: a check that failed *all
//! twelve* on a one-window fault would have sent a reader to the dispatcher or
//! to the preferences file and wasted the trip.
//!
//! ⚠ **The all-twelve and the scattered escalations have NOT been driven**, and
//! saying so is the point of this paragraph rather than leaving two untested
//! branches to be inferred as tested. They are string selection over the same
//! measured data as the branch that did fire.
//!

use std::path::{Path, PathBuf};

use crate::checks::driving::list;
use crate::checks::{Check, CheckContext};
use crate::error::{Error, Result};
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;
use crate::trace::TraceLine;

/// The environment variable that rings a command per frame.
///
/// ★ The whole reason this check needs no input. See the module header.
const INVOKE_ENV: &str = "PDFCER_DIAG_INVOKE";

/// The three commands, in the order they are rung — one per frame.
///
/// Order is not significant here (unlike a mode-then-tool pair, where it is
/// everything), because the three windows are independent `Option` fields on
/// the dialog host and all three can be open at once.
const INVOKE_LIST: &str = "file.export_image,file.export_text,file.export_dxf";

/// The line the Export-image window emits once, when it is built.
const IMAGE_EVENT: &str = "export-image-open";

/// The line the Export-text window emits once, when it is built.
const TEXT_EVENT: &str = "export-text-open";

/// The line the Export-to-DXF window emits once, when it is built.
const DXF_EVENT: &str = "export-dxf-open";

/// Each `-open` line beside the command that produces it.
///
/// Kept as a pair so a missing line can name the command that should have
/// produced it. *"No `export-text-open`"* sends a reader to the dialog;
/// *"no `export-text-open`, from `file.export_text`"* sends them to the
/// dispatcher first, which is where a rung that never fired actually shows.
const EVENTS: [(&str, &str); 3] = [
    (IMAGE_EVENT, "file.export_image"),
    (TEXT_EVENT, "file.export_text"),
    (DXF_EVENT, "file.export_dxf"),
];

/// The preferences file, beside the executable under test.
///
/// ★ **Reset to the bare sandbox seed** before the control run and rewritten
/// before the second — never deleted. Those are not the same act: deletion
/// takes `ask_default_app = false` with it, and the symptom is the O173 offer
/// opening a real OS window in front of this check's own process. See
/// [`crate::sandbox::reset_prefs`], and `print_remembered`'s account of what
/// that cost when it was learned.
const PREFS_FILE: &str = "preferences.txt";

/// The `suggestion=` token that means the page overrules the remembered units.
const CALIBRATED: &str = "calibrated";

/// The one seeded key a calibrated page is allowed to overrule.
const UNITS_KEY: &str = "export_dxf_units";

/// One remembered answer, and everything needed to assert it.
///
/// Five columns rather than `print_remembered`'s three, for the two reasons in
/// the module header: three of the twelve are spelled differently in the file
/// and in the trace, and one field name (`scope`) appears on two events.
struct Seed {
    /// The key as `Prefs::write_to_string` spells it in `preferences.txt`.
    key: &'static str,
    /// The value this check writes into the file, in the writer's vocabulary.
    file: &'static str,
    /// Which of the three `-open` lines carries it.
    event: &'static str,
    /// The field on that line.
    field: &'static str,
    /// The token that field must read after the seeded run.
    ///
    /// Equal to [`Seed::file`] for every row except the three bools, where the
    /// file says `true`/`false` and the trace says `1`/`0`.
    traced: &'static str,
}

/// **The seed: twelve `key = value` pairs, and the token each must come back
/// as.**
///
/// Every value is chosen to differ from this build's shipped default — and
/// that is *checked at run time* against the control launch rather than
/// trusted here, because a default that moves would otherwise turn this table
/// into decoration without anything going red.
///
/// ⚠ `export_image_dpi` is written `150` and read back `150` because the
/// dialog's `dpi` is an `f32` printed with `Display`, which drops a trailing
/// `.0`. A seed of `150.5` would read back `150.5` and would also be fine; a
/// seed of `150.0` would read back `150` and would report a defect that does
/// not exist. That is the one row where the two vocabularies could drift
/// without either side being wrong, so it is spelled the way the trace spells
/// it.
const SEED: [Seed; 13] = [
    Seed {
        key: "export_image_format",
        file: "emf",
        event: IMAGE_EVENT,
        field: "format",
        traced: "emf",
    },
    Seed {
        key: "export_image_pages",
        file: "all",
        event: IMAGE_EVENT,
        field: "scope",
        traced: "all",
    },
    Seed {
        key: "export_image_dpi",
        file: "150",
        event: IMAGE_EVENT,
        field: "dpi",
        traced: "150",
    },
    Seed {
        key: "export_image_transparent",
        file: "false",
        event: IMAGE_EVENT,
        field: "transparent",
        traced: "0",
    },
    Seed {
        key: "export_image_quality",
        file: "72",
        event: IMAGE_EVENT,
        field: "quality",
        traced: "72",
    },
    Seed {
        key: "export_image_keep_text",
        file: "true",
        event: IMAGE_EVENT,
        field: "keep_text",
        traced: "1",
    },
    Seed {
        key: "export_text_pages",
        file: "current",
        event: TEXT_EVENT,
        field: "scope",
        traced: "current",
    },
    Seed {
        key: "export_text_separator",
        file: "marker",
        event: TEXT_EVENT,
        field: "separator",
        traced: "marker",
    },
    Seed {
        key: "export_text_line_endings",
        file: "windows",
        event: TEXT_EVENT,
        field: "endings",
        traced: "windows",
    },
    Seed {
        key: "export_text_byte_order_mark",
        file: "true",
        event: TEXT_EVENT,
        field: "bom",
        traced: "1",
    },
    Seed {
        key: UNITS_KEY,
        file: "millimetres",
        event: DXF_EVENT,
        field: "units",
        traced: "millimetres",
    },
    Seed {
        key: "export_dxf_fit_arcs",
        file: "false",
        event: DXF_EVENT,
        field: "arcs",
        traced: "0",
    },
    Seed {
        key: "export_dxf_text",
        file: "omit",
        event: DXF_EVENT,
        field: "text",
        traced: "omit",
    },
];

/// See the module documentation.
pub struct TheExportWindowsOpenOnTheSettingsYouLastUsed;

impl Check for TheExportWindowsOpenOnTheSettingsYouLastUsed {
    fn name(&self) -> &'static str {
        "the_export_windows_open_on_the_settings_you_last_used"
    }

    fn defect(&self) -> &'static str {
        "an Export window opens on pdfcer's defaults rather than on the settings the operator \
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

/// Launch, ring all three export commands, and return the settled session.
///
/// Factored out because this check does it twice with different files on disk,
/// and the two runs must reach the windows by **identical** means — a control
/// that arrived by a different route would not be a control.
fn launch_all_three(
    ctx: &CheckContext,
    exe: &Path,
    pdf: &Path,
    trace_name: &str,
    report: &mut CheckReport,
) -> Result<Session> {
    let mut spec = LaunchSpec::new(exe, ctx.out(trace_name));
    spec.pdf = Some(pdf.to_path_buf());
    spec.env.push((
        ctx.profile.diag_env.0.to_owned(),
        ctx.profile.diag_env.1.to_owned(),
    ));
    spec.env
        .push((INVOKE_ENV.to_owned(), INVOKE_LIST.to_owned()));
    spec.allow_stale = ctx.allow_stale;
    spec.source_root = ctx.source_root.clone();

    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    report.artifact(session.trace_path().to_path_buf());
    // Long: the process opens a document before it draws anything, then rings
    // one command per frame, and each window lays itself out — the image
    // window measures every page's extent as it is built.
    session.settle(48);

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
    Ok(session)
}

/// The three `-open` lines, or a message naming which are absent.
///
/// ★ The FIRST of each, not the last. Each window emits its line once, as it
/// is built; a second would mean the window was opened twice, and the first is
/// the one produced by the preferences file this check just wrote.
fn open_lines(session: &Session) -> Result<Vec<(&'static str, TraceLine)>> {
    let trace = session.trace()?;
    let mut lines: Vec<(&'static str, TraceLine)> = Vec::new();
    let mut missing: Vec<String> = Vec::new();
    for (event, command) in EVENTS {
        match trace.events(event).next().cloned() {
            Some(line) => lines.push((event, line)),
            None => missing.push(format!("`{event}` (from `{command}`)")),
        }
    }
    if !missing.is_empty() {
        return Err(Error::new(format!(
            "{} of the 3 export windows produced no `-open` line: {}. Every one of them is \
             gated on a document being open, so the likeliest cause is that `{INVOKE_ENV}` \
             rang its command before the document was; the next is that the command is no \
             longer registered, which `command_ids` reports on. Nothing about remembered \
             settings can be learned from a window that did not open. Trace: {}.",
            missing.len(),
            list(&missing),
            session.trace_path().display()
        )));
    }
    Ok(lines)
}

/// One field off one of the three lines, or `<absent>`.
fn token<'a>(lines: &'a [(&'static str, TraceLine)], event: &str, field: &str) -> &'a str {
    lines
        .iter()
        .find(|(name, _)| *name == event)
        .and_then(|(_, line)| line.get(field))
        .unwrap_or("<absent>")
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
            "no --pdf. All three export commands decline with nothing open, so there would be \
             no window and no line to read.",
        )
    })?;
    let Some(dir) = userdata(&exe) else {
        return Err(Error::new(
            "the executable under test has no parent directory, so its `userdata` folder cannot \
             be located.",
        ));
    };
    let prefs_path = dir.join(PREFS_FILE);

    // ★ A guard rather than a line at the end, because there is more than one
    // way out below — several FAILs, several SKIPs, and a `?` on every launch
    // and every trace read, none of which looks like an exit when reading down
    // the page.
    //
    // The exits ABOVE this point deliberately have no guard, and that is not an
    // oversight: everything above resolves arguments and nothing has touched the
    // disk yet. A guard placed earlier would rewrite a preferences file this
    // check never wrote.
    //
    // RESET, never removed — the two paths where this guard is not redundant
    // (`--shared-profile`, and a hand run against a real `--exe`) are exactly
    // the paths where removing the O173 suppression would hand the
    // default-app offer to the NEXT check's window. `print_remembered` carries
    // the full account of what that cost; this is the rule it arrived at.
    //
    // Failure to reset **in the guard** is reported and does not change the
    // verdict: a harness that downgraded a real pass because it could not
    // rewrite a file on its way out would be reporting its own housekeeping as
    // a defect in the program. Failure to reset **before the control run** is a
    // SKIP, which is the opposite treatment and deliberately so — see there.
    struct Neutral<'a>(&'a Path);
    impl Drop for Neutral<'_> {
        fn drop(&mut self) {
            if self.0.exists()
                && let Some(userdata) = self.0.parent()
                && let Err(why) = crate::sandbox::reset_prefs(userdata)
            {
                eprintln!(
                    "ui-verify: WARNING — could not reset {} ({why}). It holds the export \
                     settings this check seeded, so a later --shared-profile run will not be \
                     starting from the shipped defaults. Reset it by hand to a file holding \
                     `ask_default_app = false` and nothing else.",
                    self.0.display()
                );
            }
        }
    }
    let _neutral = Neutral(&prefs_path);

    // --- process 1: the CONTROL, over a file with no export keys -------------
    //
    // What the shipped defaults are is MEASURED here rather than asserted in
    // this file. They belong to the application, and a check that hard-coded
    // them would go quietly wrong the day one moved.
    match crate::sandbox::reset_prefs(&dir) {
        Ok(()) => report.note(format!(
            "reset {} to the bare seed, so the control run starts from the shipped export \
             defaults with the O173 offer still suppressed",
            prefs_path.display()
        )),
        Err(why) => {
            return Err(Error::new(format!(
                "could not reset {} to the bare seed ({why}), so the control run would measure \
                 whatever that file happens to hold and call it the shipped defaults.",
                prefs_path.display()
            )));
        }
    };

    let session = launch_all_three(ctx, &exe, &pdf, "export-remembered-1.trace.txt", report)?;
    report.note(format!("control process: pid {}", session.pid()));
    let control = open_lines(&session)?;
    for (event, line) in &control {
        report.note(format!("control `{event}`: `{}`", line.raw));
    }

    // ★★★ The calibration trap. See the module header: a calibrated ce
    // dimension group on the page OVERRULES the remembered DXF units, by
    // design and for a reason worth more than this assertion.
    let suggestion = token(&control, DXF_EVENT, "suggestion").to_owned();
    let calibrated = suggestion == CALIBRATED;
    if calibrated {
        report.note(format!(
            "★ this page carries a calibrated ce dimension group (`suggestion={suggestion}`), \
             which overrules the remembered DXF units by design — `seeded_options` step ②. \
             `{UNITS_KEY}` is dropped from this run and 12 of the 13 are asserted. Point --pdf \
             at a drawing with no calibrated group to assert the thirteenth."
        ));
    }

    let defaults: Vec<String> = SEED
        .iter()
        .map(|seed| token(&control, seed.event, seed.field).to_owned())
        .collect();
    drop(session);

    // --- the seed ------------------------------------------------------------
    //
    // ★ Written in the writer's own vocabulary, and every value required to
    // DIFFER from what the control run just reported. A seeded value that
    // happened to equal the default would come back correct whether the file
    // was read or ignored, so it is a SKIP rather than a pass.
    let mut agreed: Vec<String> = Vec::new();
    for (index, seed) in SEED.iter().enumerate() {
        if calibrated && seed.key == UNITS_KEY {
            continue;
        }
        if defaults.get(index).map(String::as_str) == Some(seed.traced) {
            agreed.push(format!(
                "{} = {} (reads `{}={}` on `{}`)",
                seed.key, seed.file, seed.field, seed.traced, seed.event
            ));
        }
    }
    if !agreed.is_empty() {
        return Err(Error::new(format!(
            "the seed is no longer a seed: {} of the values this check writes already equal the \
             shipped default, so those fields would read back correctly whether or not the file \
             was consulted. Reported as SKIPPED rather than passed. Change the value(s) in \
             `SEED` to something the application does not ship: {}.",
            agreed.len(),
            list(&agreed)
        )));
    }

    let mut text = String::from(
        "# written by tools/ui-verify, check the_export_windows_open_on_the_settings_you_last_used\n",
    );
    for seed in &SEED {
        text.push_str(seed.key);
        text.push_str(" = ");
        text.push_str(seed.file);
        text.push('\n');
    }
    // ★ Through `sandbox::write_prefs`, which carries the O173 suppression as a
    // header. Writing this file directly would drop it.
    if let Err(why) = crate::sandbox::write_prefs(&dir, &text) {
        return Err(Error::new(format!(
            "could not write {}: {why}",
            prefs_path.display()
        )));
    }
    report.note(format!(
        "seeded {} with 13 export settings, none of which is this build's default",
        prefs_path.display()
    ));

    // --- process 2: the same route, over the seeded file ---------------------
    let session = launch_all_three(ctx, &exe, &pdf, "export-remembered-2.trace.txt", report)?;
    report.note(format!("seeded process: pid {}", session.pid()));
    let seeded = open_lines(&session)?;
    for (event, line) in &seeded {
        report.note(format!("seeded `{event}`: `{}`", line.raw));
    }

    // ★ The two runs must have been measuring the same page. `suggestion=` is a
    // property of the document and of nothing this check writes, so a change
    // means the control and the comparison are not about the same thing — and
    // the units row's treatment was decided on the control's answer.
    let suggestion_now = token(&seeded, DXF_EVENT, "suggestion");
    if suggestion_now != suggestion {
        return Err(Error::new(format!(
            "the control run reported `suggestion={suggestion}` and the seeded run reported \
             `suggestion={suggestion_now}` for the same document, so the two measurements are \
             not about the same page. Reported as SKIPPED: the shipped defaults measured by the \
             first run cannot be compared against the second."
        )));
    }

    let mut wrong: Vec<String> = Vec::new();
    let mut wrong_events: Vec<&'static str> = Vec::new();
    let mut asserted = 0_usize;
    for (index, seed) in SEED.iter().enumerate() {
        if calibrated && seed.key == UNITS_KEY {
            continue;
        }
        asserted += 1;
        let got = token(&seeded, seed.event, seed.field);
        if got != seed.traced {
            wrong.push(format!(
                "`{}` {}={got} — the file says `{} = {}` (which traces as `{}`), and this \
                 build's default is `{}`",
                seed.event,
                seed.field,
                seed.key,
                seed.file,
                seed.traced,
                defaults.get(index).map_or("<unmeasured>", String::as_str)
            ));
            if !wrong_events.contains(&seed.event) {
                wrong_events.push(seed.event);
            }
        }
    }

    if !wrong.is_empty() {
        // ★★ Three escalations rather than two, because this check spans three
        // windows and the SHAPE of the failure names the layer. All of them is
        // the path; all of one window's is that window's constructor; a
        // scattering is individual arms.
        let diagnosis = if wrong.len() == asserted {
            "★ EVERY asserted field, which points at the whole path rather than at any one \
             window: either the preferences file is not being read at start-up, or the three \
             dispatch arms are handing the windows a `Default::default()`. Check \
             `app::dispatch::exchange` — each arm must pass `&self.prefs.export.<kind>`."
        } else if wrong_events.len() == 1 {
            "★ Every failure is on ONE window's line and the other two windows restored \
             correctly, which proves the file was read, parsed and adopted. The fault is in \
             that window's constructor — the struct literal in its `open`, where a field may \
             have been left as a literal instead of taking `remembered.*`."
        } else {
            "★ Some but not all, scattered across windows, which points at individual arms \
             rather than at the path: the fields that DID arrive prove the file was read and \
             adopted. Look at each named key in `Prefs::parse`'s `export_*` arms and at the \
             matching field in the window's `open`; a key written and never parsed is what \
             `the_writer_emits_no_key_the_parser_rejects` exists to prevent, so if that test \
             is green the fault is more likely in the adoption."
        };
        return Ok(Some(format!(
            "★★★ {} OF THE {asserted} REMEMBERED EXPORT SETTINGS DID NOT COME BACK.\n\n{}\n\n\
             {diagnosis}\n\n\
             The file on disk is {} and it was written by this check, in the same token \
             vocabulary `Prefs::write_to_string` uses. Trace: {}.",
            wrong.len(),
            wrong
                .iter()
                .map(|w| format!("  • {w}"))
                .collect::<Vec<_>>()
                .join("\n"),
            prefs_path.display(),
            session.trace_path().display()
        )));
    }

    report.note(format!(
        "★★★ all three Export windows opened on the remembered settings — {asserted} of the 13 \
         asserted, each one a value this build does not ship"
    ));
    Ok(None)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// ★★★ **Every seeded value must be spelled the way the file spells it.**
    ///
    /// The seed is written straight into `preferences.txt`. A value the parser
    /// does not recognise would be dropped with a `PrefNote::BadValue`, the
    /// field would come back as its default, and this check would report a
    /// defect in the application that is really a typo in this file.
    ///
    /// This test cannot call `pdfcer-gui`'s parser — `ui-verify` does not
    /// depend on it — so it pins the shape instead: non-empty, lower case, and
    /// free of the whitespace that would split it into two trace fields.
    #[test]
    fn every_seeded_value_is_a_single_file_token() {
        for seed in &SEED {
            assert!(!seed.file.is_empty(), "{} has an empty value", seed.key);
            assert!(
                !seed.file.contains(char::is_whitespace),
                "{} = {} contains whitespace, so the trace would split it into two fields and \
                 `get(\"{}\")` would return only the first",
                seed.key,
                seed.file,
                seed.field
            );
            assert_eq!(
                seed.file.to_ascii_lowercase(),
                seed.file,
                "{} = {} is not lower case; every file token in `app::prefs::exporting` is",
                seed.key,
                seed.file
            );
        }
    }

    /// **Every seeded key is an `export_` key.**
    ///
    /// The file this check writes replaces the whole preferences file. Seeding
    /// a key outside its subject would be this check quietly changing
    /// something else — and, under `--shared-profile`, changing it for every
    /// check that runs after it.
    #[test]
    fn the_seed_touches_only_exporting() {
        for seed in &SEED {
            assert!(
                seed.key.starts_with("export_"),
                "{} is not an export preference; this check must not write one",
                seed.key
            );
        }
    }

    /// ★★ **A row is identified by `(event, field)`, and `scope` proves why.**
    ///
    /// `scope=` appears on the image window's line and on the text window's,
    /// with different shipped defaults. A uniqueness test over `field` alone
    /// would fail on a correct table; one that then "fixed" it by dropping a
    /// row would silently stop asserting one of the two windows' page scope.
    ///
    /// So this asserts the pairs are distinct **and** that the duplication is
    /// still there — if a future edit renamed one of them, this test says so
    /// rather than going quietly green on eleven rows.
    #[test]
    fn each_seeded_row_reads_a_different_event_and_field() {
        let mut seen: Vec<(&str, &str)> = Vec::new();
        for seed in &SEED {
            let pair = (seed.event, seed.field);
            assert!(
                !seen.contains(&pair),
                "`{}` {}= appears twice",
                seed.event,
                seed.field
            );
            seen.push(pair);
        }
        assert_eq!(seen.len(), 13, "the seed covers all 13 remembered settings");

        let scopes = SEED.iter().filter(|s| s.field == "scope").count();
        assert_eq!(
            scopes, 2,
            "`scope=` is expected on both the image and the text window's line; if that is no \
             longer true the module header's five-column argument needs re-reading"
        );
    }

    /// **No key is seeded twice.**
    #[test]
    fn each_seeded_row_writes_a_different_key() {
        let mut seen: Vec<&str> = Vec::new();
        for seed in &SEED {
            assert!(
                !seen.contains(&seed.key),
                "preference key `{}` appears twice",
                seed.key
            );
            seen.push(seed.key);
        }
    }

    /// ★★★ **The fifth column exists for exactly one reason, and this pins it.**
    ///
    /// The file spells a bool `true`/`false` (`opening::bool_key`); the three
    /// traces spell it `1`/`0` (`u8::from`). Every other row is the same
    /// string on both sides. A row whose two columns differ for any *other*
    /// reason is a typo that would report a defect in the application, and it
    /// would look exactly like a deliberate translation.
    #[test]
    fn only_the_boolean_rows_translate_between_the_file_and_the_trace() {
        for seed in &SEED {
            let expected = match seed.file {
                "true" => "1",
                "false" => "0",
                other => other,
            };
            assert_eq!(
                seed.traced, expected,
                "{} = {} should trace as `{}`; the only permitted translation is a bool, which \
                 the file writes as a word and the trace writes with `u8::from`",
                seed.key, seed.file, expected
            );
        }
    }

    /// ★ **The commands rung and the lines read are the same three.**
    ///
    /// The list is an environment variable and the events are constants, so
    /// nothing but this ties them together. Adding a fourth export window
    /// means both, and a seed row naming an event nobody rings would report
    /// *"the window did not open"* for ever.
    #[test]
    fn the_invoke_list_names_the_command_behind_every_event_the_seed_reads() {
        let rung: Vec<&str> = INVOKE_LIST.split(',').map(str::trim).collect();
        assert_eq!(
            rung.len(),
            EVENTS.len(),
            "`{INVOKE_LIST}` rings {} commands and {} events are read",
            rung.len(),
            EVENTS.len()
        );
        for (index, (event, command)) in EVENTS.iter().enumerate() {
            assert_eq!(
                rung[index], *command,
                "the command rung at rung {index} is not the one that opens `{event}`"
            );
        }
        for seed in &SEED {
            assert!(
                EVENTS.iter().any(|(event, _)| *event == seed.event),
                "{} reads `{}`, which no rung of `{INVOKE_LIST}` produces",
                seed.key,
                seed.event
            );
        }
    }
}
