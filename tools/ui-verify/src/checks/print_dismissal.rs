//! `the_print_window_forgets_what_cancel_undid` — **operator request O185,
//! driven.**
//!
//! # The report
//!
//!
//! > *"I set the printer up, close the window to go check something, and it's
//! > all gone."*
//!
//! and, in the same breath, the other half — that a window which keeps what you
//! set up needs a way to say *no, not that*:
//!
//! > *"they revert back to what they were when we opened the print dialogue"*
//!
//! O166 had already made the print window remember its settings **when Print is
//! pressed**. O185 is about the other three ways out. There are now four, and
//! they mean two different things:
//!
//! | route | meaning |
//! |---|---|
//! | **Print** | remember, spool, close *(O166, unchanged)* |
//! | **Keep and close** | remember, print nothing |
//! | **Cancel** | put the settings back to what this window opened with |
//! | the OS close button, and Escape | the same as **Cancel** |
//!
//! That last row is `ui-conventions/dialogs.md` **G4** held to the letter: the
//! chrome, Escape and the cancel button are deliberately indistinguishable, so
//! all three keep one meaning, and the meaning they keep has to be the safe
//! one. *Keep and close* is a fourth, positively-chosen route that G4 never
//! contemplated.
//!
//! # ★★★ What this check is FOR, said before what it does
//!
//! **It is the assertion that the two labelled routes out are not the same
//! button.** Everything else here is scaffolding for that one sentence.
//!
//! A build in which *Keep and close* and *Cancel* are wired identically — the
//! obvious regression, and the state of the program the day before this landed
//! — is a build in which every screenshot is correct, every unit test passes,
//! and the operator's actual complaint is unfixed. Two buttons sit in the
//! footer, both close the window, and which one you press changes nothing.
//! Nothing a photograph can show distinguishes that from a working build.
//!
//! So the load-bearing assertion is **cross-run**: the value the window opens
//! on after a Cancel and the value it opens on after a Keep must *differ*. Each
//! run also makes its own absolute claim — Cancel restores the original, Keep
//! keeps the change — and those are worth having, but either one alone is
//! satisfiable by a wiring that ignores the distinction. A check that asserted
//! only "after Cancel the setting is the original" passes perfectly against a
//! build that never saves anything at all, which is precisely the build O185
//! replaced.
//!
//! ## ★★★ And the order the three are tested in is not cosmetic
//!
//! The cross-run comparison is tested **first**, and it has to be, because in
//! the order this file was originally written — Cancel-restored, Keep-kept,
//! then the comparison — **the comparison could never fire**. The per-run guard
//! establishes that the window did not open on the value about to be clicked;
//! given that, "Cancel reopened on what it opened with" and "Keep reopened on
//! the clicked value" together already imply the two differ. The claim the
//! module header calls load-bearing was three lines of unreachable `if`.
//!
//! ⚠ **Both falsification runs went red anyway, which is how it survived
//! them** — the two absolute claims caught both planted builds and reported
//! them well. A falsification proves the check *as a whole* discriminates; it
//! says nothing about whether every assertion inside it can be reached. Reading
//! the assertions as a system, and asking of each one *what input reaches this
//! line*, is a separate act and this file is the argument for doing it.
//!
//! With the comparison first, all three are live: a degenerate build stops at
//! it, and a build where one route produces some *third* value — neither the
//! opening token nor the clicked one — passes it and is caught by the absolute
//! claim below.
//!
//! # ★★ What this check deliberately CANNOT establish
//!
//! **It never presses Print, and no future edit may make it.** Committing is
//! how a print job reaches a real device, and this suite runs unattended on the
//! machine whose default printer is the operator's plotter. Four other print
//! checks state that rule in their own words rather than by reference, because
//! the day somebody adds a sixth by copying one of these files, the copied file
//! is what they will read. This is the sixth, and it says it too.
//!
//! One consequence is specific and worth naming, because a reader will
//! otherwise take this green for more than it is:
//!
//! ⚠ **`reverted=true` is unreachable from here.** `Cancel` does two things —
//! it declines to write, and it *puts back* anything already written — and only
//! the first is driveable. The second has exactly one reachable cause in the
//! program: **a spool the driver refuses.** `remember()` runs before the spool
//! and whether or not the spool succeeds, and a refused spool leaves the window
//! open, so an operator can press Print, be refused, change more settings and
//! then press Cancel — at which point the preferences on disk hold what the
//! failed press wrote. That is the one path where the word *revert* earns its
//! place over *decline*, and reaching it from here would mean pressing Print.
//!
//! What stands in for it:
//!
//! | The claim | What holds it |
//! |---|---|
//! | Cancel declines to write | ★ **this check** — the Cancel run's reopen |
//! | Keep writes | ★ **this check** — the Keep run's reopen |
//! | the two are different buttons | ★ **this check** — the cross-run assertion |
//! | Cancel puts back a value already written | ⚠ **nothing automated** — see below |
//! | the settings survive to disk at all | `the_print_window_opens_on_the_settings_you_last_used`, which reads a seeded file back in a second process |
//!
//! ⚠⚠ **That fourth row says "nothing", and the word is measured rather than
//! modest.** An earlier draft of this table cited *"`remembered`'s unit
//! tests"*; that file has no `#[cfg(test)]` module, and neither does
//! `export_remembered`, which has the same shape — this layer's convention in
//! this crate is that a driven check covers it. A citation to a holder that
//! does not exist is worse than an admitted gap, because a reader who checks it
//! finds a plausible file and stops.
//!
//! What genuinely constrains the write-back, short of a test:
//! `PrintDialog::store` is the **sole** writer of `prefs.print` and the sole
//! emitter of `print-remembered`, so `restore` cannot write by some other
//! route; it differs from `remember` only in the payload it hands over
//! (`opened_with` against `habits()`) and in the `how=` token; and
//! `print-dismissed reverted=` discloses at runtime whether a put-back
//! happened. **None of that is a measurement.** Reaching the real path needs a
//! spool the driver refuses, which needs a Print press, which this suite may
//! not make — so the gap is structural and is recorded as such in
//! `DESIGNS.md` §O185 rather than left as a silence somebody has to rediscover.
//!
//! Neither half is claimed by the other, and a reader who took this green as
//! covering the failed-spool revert would be taking more than is here.
//!
//! # The gesture, and why it is the paper policy and not something friendlier
//!
//! The check has to *change a setting* between opening the window and leaving
//! it. Of the twenty-odd controls in the print window, **one** publishes a
//! rectangle a driver can aim at: the paper combo, `print.paper`, with its
//! entries under `print.paper.item.N` and `print.paper.auto`. Copies, collate,
//! reverse, duplex and the rest are drawn and unaddressable.
//!
//! ★ Within that combo the target is **`print.paper.auto`**, not a numbered
//! form, and the difference matters more than it looks:
//!
//! - A numbered entry is one of the **driver's** forms, so which one exists and
//!   what it is called is a property of the machine the suite happens to be
//!   running on. `print_paper` has to try up to five of them in a loop for
//!   exactly that reason, and says so at length.
//! - Worse for *this* check, a hand-picked `Form(id)` is **not remembered as
//!   itself**. `paper_key` reduces every form to the token `device` — deliberate
//!   loss, argued on `habits()` — so choosing a form and choosing nothing come
//!   back from the preferences file as the same string. The gesture would be
//!   invisible to the oracle.
//! - `Auto` is pdfcer's own second *policy*, it is at a fixed region name
//!   outside the numbered namespace, and it persists as its own token,
//!   `match-pages`. One click, no loop, no machine dependence.
//!
//! # The oracle: three lines, and the third is the one that matters
//!
//! ```text
//! print-open   … paper=device …                          (the window as opened)
//! print-plan   … pick=auto  auto=matched …               (the gesture took effect)
//! print-dismissed reason=revert saved=true reverted=false (which way out was taken)
//! print-open   … paper=device …                          (the window, reopened)
//! ```
//!
//! `print-open`'s `paper=` is spelled by the preferences file's own
//! `paper_key`, so the token this check compares is the token that would be
//! written to disk — not a second spelling free to drift. `print-plan`'s
//! `pick=` is a stable one-word token from `autopaper::pick_token` for the same
//! reason, and it is read here as a **precondition, not an assertion**: if the
//! click on the combo entry did not change the live choice, this check has
//! learned nothing about dismissal and says so as a SKIP rather than accusing
//! the application of a defect it caused itself.
//!
//! ★ `print-dismissed` did not exist before O185. Nothing traced a close at
//! all — no `print-close`, no event on `frame.closed` — so a build that took
//! the wrong branch on the way out was indistinguishable from one that took the
//! right branch and wrote nothing. This is the fifth time in this project that
//! sitting down to write a driven check found a trace that could not tell apart
//! the two states the check exists for.
//!
//! # ★ `reverted=false` on a green Cancel run is correct, and is not a bug
//!
//! The Cancel run's `print-dismissed` line reads `reverted=false`, and a reader
//! meeting that for the first time will read it as *the revert did not happen*.
//! It means the opposite of a defect: `prefs.print` still equalled
//! `opened_with`, because nothing in this session had written over it, so there
//! was nothing to put back. `restore` reports what it *did*, not what it
//! *would have done*, which is what makes the field usable for telling a Cancel
//! on an untouched window from a Cancel that undid a failed print.
//!
//! `saved=true` beside it is the other half: the preferences **now hold** the
//! settings the window opened with, which they do, trivially, by never having
//! stopped.
//!
//! # Every way this reports SKIP
//!
//! No binary; `--no-input`; no `--pdf` (the Print command is gated on a
//! document being open, so the ribbon control is greyed and there is no dialog
//! to reach); no ui-rect channel; the ribbon control not declared; the dialog
//! not opening; the spooler refusing on this machine; the paper combo's popup
//! not opening within the settle; the Auto entry leaving the live choice
//! unchanged; or the shipped default already being `match-pages`, which would
//! make the gesture a no-op and every assertion below it vacuous. Each says
//! which, and none of them is reported as a pass.

use std::path::{Path, PathBuf};

use crate::checks::driving::{
    ITEM_PREFIX, SHELL_DIAG_ENV, TAB_EVENT, declared, declared_in, declared_names,
    declared_or_in_overflow, frame_for, list, shell_trace,
};
use crate::checks::{Check, CheckContext};
use crate::error::{Error, Result};
use crate::input::Driver;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;
use crate::trace::{Trace, TraceLine};

/// The ribbon control that opens the dialog, and the tab it lives on.
const SUBJECT: &str = "ribbon.item.file.print";
const TAB_ID: &str = "file";
const TAB: &str = "ribbon.tab.file";

/// Emitted once as the dialog is built. Read twice per run: as opened, and
/// again after the window has been dismissed and reopened.
const OPEN_EVENT: &str = "print-open";

/// The per-frame line carrying what the job was planned with. Read only for
/// `pick=`, and only as a precondition.
const PLAN_EVENT: &str = "print-plan";

/// The line O185 added, at the window's single return. The only place that
/// knows which of the four routes out was taken.
const DISMISS_EVENT: &str = "print-dismissed";

/// The paper combo, and the entry inside it this check clicks.
///
/// ★ `print.paper.auto` is deliberately outside the `print.paper.item.N`
/// namespace — see that constant's own doc in `dialogs/print/mod.rs`, which
/// says numbering it would have silently re-aimed an existing driven check.
/// This check depends on that separation: it wants *pdfcer's policy*, at a
/// fixed name, not the driver's first enumerated form.
const PAPER: &str = "print.paper";
const PAPER_AUTO: &str = "print.paper.auto";

/// The footer's three buttons. `Host::footer` publishes all three.
const BUTTON_CANCEL: &str = "dialog.buttons.cancel";
const BUTTON_KEEP: &str = "dialog.buttons.keep";

/// The preferences file, beside the executable under test.
///
/// ★ **Reset to the bare sandbox seed** before each of the two runs — never
/// deleted. The two are not the same act: deletion takes `ask_default_app =
/// false` with it, and the symptom is the O173 offer opening a real OS window
/// in front of the very click this check is about to make. That cost
/// `print_remembered` two sweeps and five documents' worth of wrong diagnosis;
/// `sandbox::reset_prefs` exists to close the class.
///
/// Safe only because the suite is **never** pointed at a published build — that
/// is the standing rule, and the two print checks are among the reasons for it.
/// Pointed at the operator's own install, this one would leave his print
/// settings holding whatever the Keep run chose.
const PREFS_FILE: &str = "preferences.txt";

/// The token `paper_key` gives the choice this check makes.
///
/// Written here rather than measured because it is not a claim about the
/// application's *defaults* — it is the name of the thing being clicked. The
/// Auto entry sets `PaperChoice::AutoFromPages`, and
/// `app::prefs::printing::paper_key` spells that `match-pages`. If that
/// spelling ever moves, this check goes red naming both strings rather than
/// silently agreeing with the new one.
const CHOSEN_TOKEN: &str = "match-pages";

/// The `pick=` token `print-plan` reports once the Auto entry has been chosen.
const CHOSEN_PICK: &str = "auto";

/// **Which way out of the window a run takes.**
///
/// Two runs, two routes, and the pair is the assertion. Kept as a type rather
/// than a `bool` so the failure messages can name the button the operator
/// pressed rather than a flag.
#[derive(Clone, Copy)]
enum Route {
    /// The *Cancel* button. Also what the OS close button and Escape mean —
    /// `dialogs.md` G4 — though this check presses the labelled one, because a
    /// synthetic Escape would additionally be asserting the keymap.
    Cancel,
    /// The *Keep and close* button. The fourth route, and the only one with no
    /// unlabelled twin.
    Keep,
}

impl Route {
    /// The published region this route's button lives at.
    const fn region(self) -> &'static str {
        match self {
            Self::Cancel => BUTTON_CANCEL,
            Self::Keep => BUTTON_KEEP,
        }
    }

    /// The word `print-dismissed reason=` must carry for this route.
    const fn reason(self) -> &'static str {
        match self {
            Self::Cancel => "revert",
            Self::Keep => "keep",
        }
    }

    /// How the button reads in the footer, for a failure message aimed at
    /// somebody who is looking at the window.
    const fn label(self) -> &'static str {
        match self {
            Self::Cancel => "Cancel",
            Self::Keep => "Keep and close",
        }
    }
}

/// What one run measured.
struct Run {
    /// `print-open paper=` on the window as first opened. Measured, never
    /// asserted: the shipped default belongs to the application.
    opened_on: String,
    /// `print-open paper=` on the window reopened after the dismissal. The
    /// answer this check exists to compare.
    reopened_on: String,
    /// The whole `print-dismissed` line, for the report.
    dismissed: String,
}

/// A run either measured something or found a defect; both are ordinary
/// outcomes and neither is an error.
enum Verdict {
    Measured(Box<Run>),
    Defect(String),
}

/// See the module documentation.
pub struct ThePrintWindowForgetsWhatCancelUndid;

impl Check for ThePrintWindowForgetsWhatCancelUndid {
    fn name(&self) -> &'static str {
        "the_print_window_forgets_what_cancel_undid"
    }

    fn defect(&self) -> &'static str {
        "Cancel and Keep-and-close do the same thing to the print window's remembered settings, \
         so there is no way to leave the window without keeping what you set up in it"
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

/// Click the ribbon's Print control and wait for the window.
///
/// Factored out because each run does it **twice** — once to set the window up
/// and once to read what the next window opens on — and the two must reach the
/// dialog by identical means. A reopen that arrived through a different route
/// would not be measuring the same thing.
///
/// ⚠ It re-resolves the control every time rather than caching a rect. The
/// ribbon can fold Print into its overflow menu at the harness's window width,
/// and `declared_or_in_overflow` opens that menu — which changes the layout, so
/// a rect captured before the first open is not guaranteed to be where the
/// button is before the second.
fn open_the_window(session: &Session, driver: &Driver, ui_rect: &str, which: &str) -> Result<()> {
    let Some(control) = declared_or_in_overflow(session, driver, ui_rect, SUBJECT)? else {
        let trace = session.trace()?;
        return Err(Error::new(format!(
            "the File tab is active and neither it nor its overflow declares `{SUBJECT}` \
             ({which} open). Controls declared: {}. That is `print_dialog`'s defect, not this \
             one — it is reported there.",
            list(&declared_names(&trace, ui_rect, ITEM_PREFIX))
        )));
    };
    driver.click_at(session.frame()?.declared_center(control))?;
    // Enumerating printers touches the spooler, which BLOCKS on a network
    // printer, and the dialog also enumerates the selected device's forms.
    // Same settle as `print_paper`'s, for the same reason.
    session.settle(40);
    Ok(())
}

/// The `n`-th `print-open` line, or a message saying which of the causes
/// applies.
///
/// `nth` is 0 for the window as opened and 1 for the window reopened after the
/// dismissal. Reading by index rather than by `last()` is deliberate: a dock or
/// a dialog that failed to close would leave the first line as the newest one
/// this check could see, and `last()` would then quietly re-report the *before*
/// value as the *after* value — an assertion that can only pass.
fn open_line(trace: &Trace, nth: usize, which: &str) -> Result<TraceLine> {
    let line = trace.events(OPEN_EVENT).nth(nth).cloned().ok_or_else(|| {
        Error::new(format!(
            "the trace carries {} `{OPEN_EVENT}` line(s), so the {which} print window never \
             opened. That is `print_dialog`'s subject and it reports the causes apart; nothing \
             about dismissal can be learned here.",
            trace.events(OPEN_EVENT).count()
        ))
    })?;
    if line.get("unavailable").unwrap_or("<absent>") != "None" {
        return Err(Error::new(format!(
            "the spooler refused on this machine, so the {which} dialog has no device and \
             several of its controls are not drawn. Reported as SKIPPED: a refused enumeration \
             proves nothing either way about what a close does to the settings."
        )));
    }
    Ok(line)
}

/// `print-plan pick=` as of the newest plan line.
fn pick(session: &Session) -> Result<String> {
    let trace = session.trace()?;
    Ok(trace
        .events(PLAN_EVENT)
        .last()
        .and_then(|l| l.get("pick").map(str::to_owned))
        .unwrap_or_else(|| "<absent>".to_owned()))
}

/// Launch, open Print, choose Auto, leave by `route`, reopen Print.
#[allow(clippy::too_many_lines)]
fn one_run(
    ctx: &CheckContext,
    exe: &Path,
    pdf: &Path,
    ui_rect: &str,
    route: Route,
    report: &mut CheckReport,
) -> Result<Verdict> {
    let trace_name = format!("print-dismissal-{}.trace.txt", route.reason());
    let session = Session::launch(
        &spec_for(ctx, exe, pdf, &trace_name),
        ctx.profile.trace_prefix,
    )?;
    report.artifact(session.trace_path().to_path_buf());
    report.note(format!("{} run: pid {}", route.label(), session.pid()));
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

    // --- A. the ribbon -------------------------------------------------------
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

    // --- B. the window, as it opens -----------------------------------------
    open_the_window(&session, &driver, ui_rect, "first")?;
    let opened = open_line(&session.trace()?, 0, "first")?;
    let opened_on = opened.get("paper").unwrap_or("<absent>").to_owned();
    report.note(format!("{}: opened on `{}`", route.label(), opened.raw));

    // ★ The gesture has to CHANGE something, and this is where that is
    // established rather than assumed. If the window already opens on Auto —
    // because a future build ships it as the default, or because a previous
    // check left it in the file — then clicking Auto changes nothing, the
    // reopen reports the same token whatever the dismissal did, and every
    // assertion below would pass on a build with no dismissal logic at all.
    //
    // SKIP, not FAIL: a default this check cannot work with is a fact about
    // the check, and saying "defect" about it would be a false accusation of
    // the kind `check-region-names`' header is written about.
    if opened_on == CHOSEN_TOKEN {
        return Err(Error::new(format!(
            "the print window already opens on `{CHOSEN_TOKEN}`, which is the value this check \
             clicks to. The gesture would change nothing and every assertion after it would \
             hold on a build that ignored the dismissal entirely. Reported as SKIPPED. Point \
             the gesture at a different control, or at a different entry in the paper combo."
        )));
    }

    // --- C. change one setting, in the dialog's own OS window -----------------
    //
    // ★★ FROM HERE THE REGIONS ARE IN A SECOND OS WINDOW. The dialog is a real
    // viewport (`dialogs::host`, G1), so its `ui-rect` rectangles are relative
    // to ITS client area. `session.frame()` is the wrong origin for every one
    // of them and produces coordinates that look entirely reasonable and land
    // several hundred points away. `declared_in` carries the viewport tag and
    // `frame_for` turns it into the right origin, re-resolved per click because
    // the operator — and this harness's own clicks — can move the window.
    let trace = session.trace()?;
    let (paper, paper_vp) = declared_in(&trace, ui_rect, PAPER).ok_or_else(|| {
        Error::new(format!(
            "the dialog published no `{PAPER}` region. The Pages & Layout tab is the dialog's \
             default tab, so this is not a tab problem — either the combo is not being drawn or \
             the device enumerated no forms. Reported as SKIPPED: with no way to change a \
             setting there is nothing for a dismissal to keep or undo."
        ))
    })?;
    driver.click_at(frame_for(&session, &trace, paper_vp.as_deref())?.declared_center(paper))?;
    session.settle(12);

    let trace = session.trace()?;
    let Some((auto, auto_vp)) = declared_in(&trace, ui_rect, PAPER_AUTO) else {
        return Err(Error::new(format!(
            "the click on `{PAPER}` published no `{PAPER_AUTO}` region, so the combo's popup \
             did not open — or opened and closed inside the settle. Entries seen: {}. Reported \
             as SKIPPED: this is a harness timing question, not an application claim.",
            list(&declared_names(&trace, ui_rect, "print.paper."))
        )));
    };
    driver.click_at(frame_for(&session, &trace, auto_vp.as_deref())?.declared_center(auto))?;
    // Longer than a widget settle: the choice re-reads the device geometry
    // through `printer_caps_for`, which opens an information device context.
    session.settle(25);

    let chosen = pick(&session)?;
    if chosen != CHOSEN_PICK {
        return Err(Error::new(format!(
            "the click on `{PAPER_AUTO}` left `{PLAN_EVENT} pick={chosen}` where \
             `{CHOSEN_PICK}` was wanted, so the live paper policy did not change and this run \
             has nothing for a dismissal to act on. Reported as SKIPPED rather than as a defect \
             in dismissal: a gesture that did not land is this harness's problem, and \
             `print_paper` is the check that owns whether the combo works."
        )));
    }
    report.note(format!(
        "{}: the paper policy is now `{PLAN_EVENT} pick={chosen}`",
        route.label()
    ));

    // --- D. leave, by the route this run is about ----------------------------
    let trace = session.trace()?;
    let Some((button, button_vp)) = declared_in(&trace, ui_rect, route.region()) else {
        return Err(Error::new(format!(
            "the dialog published no `{}` region, so the {} button is not being drawn. Footer \
             regions seen: {}. `Host::footer` publishes all three when a job is present; the \
             no-job arm draws only two, and this run reached a window that reported a device.",
            route.region(),
            route.label(),
            list(&declared_names(&trace, ui_rect, "dialog.buttons."))
        )));
    };
    driver.click_at(frame_for(&session, &trace, button_vp.as_deref())?.declared_center(button))?;
    session.settle(20);

    let trace = session.trace()?;
    let Some(dismissed) = trace.events(DISMISS_EVENT).last().cloned() else {
        return Ok(Verdict::Defect(format!(
            "★★ THE {} BUTTON WAS PRESSED AND NOTHING TRACED A DISMISSAL.\n\n\
             `{DISMISS_EVENT}` is emitted from the print window's single return, which is the \
             only place that knows why the window is closing. No line means either the click \
             missed the button — check `{}` in the trace — or the window returned without \
             going through `PrintDialog::dismiss`.\n\nTrace: {}.",
            route.label().to_uppercase(),
            route.region(),
            session.trace_path().display()
        )));
    };
    let got = dismissed.get("reason").unwrap_or("<absent>");
    if got != route.reason() {
        return Ok(Verdict::Defect(format!(
            "★★★ THE {} BUTTON REPORTED `reason={got}`, WHERE `{}` IS THE ONLY CORRECT \
             ANSWER.\n\n`{}`\n\n\
             The four routes out of the print window map to three reasons, and the mapping is \
             the whole of O185. A button that reports the wrong one is doing the wrong thing to \
             the operator's settings — `PrintDialog::dismiss`'s `match` is total, so the wrong \
             word here means the wrong arm ran, not merely a mislabelled trace.\n\nTrace: {}.",
            route.label().to_uppercase(),
            route.reason(),
            dismissed.raw,
            session.trace_path().display()
        )));
    }
    report.note(format!("{}: `{}`", route.label(), dismissed.raw));

    // --- E. reopen, and read what the next window opens on -------------------
    open_the_window(&session, &driver, ui_rect, "second")?;
    let reopened = open_line(&session.trace()?, 1, "second")?;
    let reopened_on = reopened.get("paper").unwrap_or("<absent>").to_owned();
    report.note(format!("{}: reopened on `{}`", route.label(), reopened.raw));

    Ok(Verdict::Measured(Box::new(Run {
        opened_on,
        reopened_on,
        dismissed: dismissed.raw.clone(),
    })))
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
            "input is disabled (--no-input), and this check is two launches and twelve clicks. \
             Reported as SKIPPED rather than passed — a check that did not run has learned \
             nothing.",
        ));
    }
    let ui_rect = ctx.profile.vocab.ui_rect_event.ok_or_else(|| {
        Error::new(format!(
            "the `{}` profile declares no ui-rect trace event, so neither the ribbon's Print \
             control nor the dialog's footer can be found.",
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

    // ★★★ Reset on the way out, on every path, and RESET rather than remove.
    //
    // The Keep run deliberately leaves a print preference on disk — that is the
    // thing it proves — so unlike most checks this one really does dirty the
    // file it found. Under a default `run-all` the whole sandbox directory is
    // removed afterwards and this is redundant; the two runs where it is not
    // are `--shared-profile` and a hand run against a real `--exe`, and those
    // are exactly the two where leaving `print_paper = match-pages` behind
    // would be somebody else's settings changed without being asked.
    //
    // Removing the file instead would take `ask_default_app = false` with it
    // and hand the O173 offer to the next check's window. `sandbox::reset_prefs`
    // writes the header and nothing else, which is the neutral state wanted.
    //
    // A failure to reset is REPORTED and does not change the verdict: this
    // check's assertions are about the application, and a harness that
    // downgraded a real pass because it could not rewrite a file on its way out
    // would be reporting its own housekeeping as a defect in the program.
    struct Neutral<'a>(&'a Path);
    impl Drop for Neutral<'_> {
        fn drop(&mut self) {
            if self.0.exists()
                && let Some(userdata) = self.0.parent()
                && let Err(why) = crate::sandbox::reset_prefs(userdata)
            {
                eprintln!(
                    "ui-verify: WARNING — could not reset {} ({why}). It holds the print paper \
                     policy the Keep run chose, so a later --shared-profile run will not be \
                     starting from the shipped defaults. Reset it by hand to a file holding \
                     `ask_default_app = false` and nothing else.",
                    self.0.display()
                );
            }
        }
    }
    let _neutral = Neutral(&prefs_path);

    // --- the two runs, each from the same starting state ---------------------
    //
    // ★ The reset happens before EACH run, not once before both. The Keep run
    // writes a print preference, and a Cancel run that started from it would be
    // opening on `match-pages` — the value it is about to click — which is the
    // vacuous state guarded against inside `one_run`. It would SKIP rather than
    // mislead, but a check that skips half of itself depending on the order its
    // own two halves ran in is a check nobody can read.
    let mut runs: Vec<(Route, Run)> = Vec::new();
    for route in [Route::Cancel, Route::Keep] {
        if let Err(why) = crate::sandbox::reset_prefs(&dir) {
            return Err(Error::new(format!(
                "could not reset {} to the bare seed before the {} run ({why}), so that run \
                 would start from whatever that file happens to hold.",
                prefs_path.display(),
                route.label()
            )));
        }
        match one_run(ctx, &exe, &pdf, ui_rect, route, report)? {
            Verdict::Defect(failure) => return Ok(Some(failure)),
            Verdict::Measured(run) => runs.push((route, *run)),
        }
    }

    let Some((cancel, keep)) = runs.first().zip(runs.get(1)) else {
        return Err(Error::new(
            "fewer than two runs completed, which cannot happen — every other path out of the \
             loop above returns.",
        ));
    };
    let cancel = &cancel.1;
    let keep = &keep.1;

    // ★★ The cross-run comparison below assumes the two runs STARTED level,
    // and this is where that is established rather than hoped for. Each run
    // resets the preferences to the bare seed, so both windows should open on
    // the same token; if they did not, one of the resets did not take, and a
    // difference between the two reopens could be a difference between the two
    // openings rather than anything a button did.
    //
    // SKIP rather than FAIL, for the reason the in-run guard gives: a starting
    // state this check could not establish is a fact about the check.
    if cancel.opened_on != keep.opened_on {
        return Err(Error::new(format!(
            "the two runs did not start level — the Cancel run's window opened on \
             `paper={}` and the Keep run's on `paper={}`. Both are launched from a \
             freshly reset preferences file, so one of the resets did not take, and a \
             difference between what the two windows REOPEN on could then be a \
             difference between what they opened on. Reported as SKIPPED: nothing here \
             is a claim about the application.",
            cancel.opened_on, keep.opened_on
        )));
    }

    // --- 1. ★★★ they are not the same button ---------------------------------
    //
    // **First, because it is the coarsest true thing and the one the request is
    // about.** A build in which the two routes are wired identically is the
    // regression O185 exists to prevent, and it is invisible to a photograph:
    // two buttons, both close the window, and which one you press changes
    // nothing.
    //
    // ★ It is first for a second reason, and it is the one worth reading. When
    // this check was written the order was 2, 3, then this — and in that order
    // **this assertion could never fire**. The per-run guard establishes
    // `opened_on != CHOSEN_TOKEN`; assertion 2 passing means the Cancel run
    // reopened on `opened_on`; assertion 3 passing means the Keep run reopened
    // on `CHOSEN_TOKEN`; so by the time control reached here the two values
    // were already known to differ. It was three lines of `if` that read as the
    // load-bearing claim of the file and could not, by construction, be
    // reached — the exact shape of `a check that cannot fail is not evidence`,
    // committed by the person who wrote that lesson down, inside the check
    // written to demonstrate the principle.
    //
    // Both falsification runs still went red, which is precisely why it
    // survived the falsification: 2 and 3 caught the two planted builds and
    // reported them well. **A falsification proves the check as a whole
    // discriminates. It says nothing about whether every assertion in it can
    // fire.**
    //
    // Moving it first fixes it. Now a degenerate build lands here, and the
    // three below stay live for the asymmetric cases — a route that produces
    // some THIRD value reaches them with the two reopens differing.
    if cancel.reopened_on == keep.reopened_on {
        // Which degenerate wiring it is, named rather than left to the reader.
        // The agreed value says it: the changed token means both routes wrote,
        // the opening token means neither did.
        let which = if cancel.reopened_on == CHOSEN_TOKEN {
            "Both routes KEPT the change. Cancel is not undoing anything — whichever arm it \
             takes, it ends in the same writer Keep uses."
        } else if cancel.reopened_on == cancel.opened_on {
            "Neither route kept the change. Keep and close is not writing — whichever arm it \
             takes, it ends where Cancel ends, and the operator's original complaint is \
             unfixed with a button on it."
        } else {
            "Both routes produced a THIRD value, which is neither what the window opened with \
             nor what was chosen in it. Something other than these two arms is writing the \
             paper policy."
        };
        return Ok(Some(format!(
            "★★★ CANCEL AND KEEP AND CLOSE ARE THE SAME BUTTON.\n\n\
             Both runs opened the window on `paper={}`, changed the paper policy to \
             `{CHOSEN_TOKEN}`, left by different buttons, and reopened on the same value — \
             `paper={}`.\n\n\
             {which}\n\n\
             cancel: `{}`\n   keep: `{}`\n\n\
             Two buttons in the footer that do the same thing is worse than one button, \
             because the operator now believes there is a way to leave without keeping, and \
             there is not.\n\n\
             The `match` in `PrintDialog::dismiss` is where the two arms diverge. If the \
             `reason=` words above differ while the outcome does not, the routing is right \
             and the two arms are calling the same writer.",
            cancel.opened_on, cancel.reopened_on, cancel.dismissed, keep.dismissed
        )));
    }

    // --- 2. Cancel put it back ----------------------------------------------
    //
    // ★ Reachable past assertion 1, and here is the input that reaches it: the
    // **swapped** wiring, where Cancel remembers and Keep restores. The two
    // reopens then differ — so assertion 1 passes — and this one fires, which
    // is the right outcome, because "the buttons are transposed" deserves a
    // different sentence from "the buttons are the same".
    if cancel.reopened_on != cancel.opened_on {
        return Ok(Some(format!(
            "★★★ CANCEL KEPT THE CHANGE.\n\n\
             The window opened on `paper={}`, the paper policy was changed to `{CHOSEN_TOKEN}`, \
             Cancel was pressed, and the window reopened on `paper={}`.\n\n\
             `{}`\n\n\
             Cancel means *put the settings back to what this window opened with* — the \
             operator's own words, `OPERATOR_REQUESTS.md` O185. A Cancel that keeps is the \
             defect that request exists to fix, one route along: the operator abandons a \
             configuration and inherits it anyway on the next print.\n\n\
             `PrintDialog::dismiss`'s `Dismissal::Revert` arm is what runs here, and \
             `remembered.rs`'s `restore` is the only writer it may call.",
            cancel.opened_on, cancel.reopened_on, cancel.dismissed
        )));
    }

    // --- 3. Keep kept it ----------------------------------------------------
    //
    // ⚠⚠ **This one is UNREACHABLE today, and saying so is the point of the
    // comment.** The gesture's whole domain is two tokens — `paper_key` spells
    // every hand-picked form and the device default alike as `device`, and
    // pdfcer's own policy as `match-pages`, and there is no third answer. Both
    // runs open on `device`. So once assertion 1 has established that the two
    // reopens differ and assertion 2 that the Cancel run's is `device`, the
    // Keep run's can only be `match-pages`, which is what this tests.
    //
    // It is kept, and it is **not evidence** — the claim it names is carried by
    // 1 and 2 together. It is a guard against the domain widening, which is a
    // live possibility rather than a hypothetical: the design for this check
    // records that remembering a form *as itself* was considered and rejected
    // as a loss, and the day `paper_key` grows a third token this is the only
    // line that would catch a Keep which wrote the wrong one.
    //
    // ★ The distinction worth carrying away: an assertion that cannot fire
    // because of how the check is ORDERED is a defect — that was assertion 1
    // before it was moved. An assertion that cannot fire because the system's
    // domain is currently too small is a guard, and the honest thing is to
    // label it rather than to delete it or to let it read as a measurement.
    if keep.reopened_on != CHOSEN_TOKEN {
        return Ok(Some(format!(
            "★★★ KEEP AND CLOSE DID NOT KEEP.\n\n\
             The window opened on `paper={}`, the paper policy was changed to `{CHOSEN_TOKEN}`, \
             Keep and close was pressed, and the window reopened on `paper={}`.\n\n\
             `{}`\n\n\
             That is the operator's original report — *\"I set the printer up, close the window \
             to go check something, and it's all gone\"* — unfixed. The button exists, it \
             closes the window, and the setup is gone anyway.\n\n\
             `PrintDialog::dismiss`'s `Dismissal::Keep` arm calls `remember`, which is the same \
             writer the Print press uses; if this fails while \
             `the_print_window_opens_on_the_settings_you_last_used` passes, the fault is in the \
             arm rather than in the preferences file.",
            keep.opened_on, keep.reopened_on, keep.dismissed
        )));
    }

    report.note(format!(
        "★★★ Cancel reopened on `{}` — what the window opened with — and Keep and close \
         reopened on `{}`. Two buttons, two outcomes.",
        cancel.reopened_on, keep.reopened_on
    ));
    Ok(None)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **The two routes name different regions and different reasons.**
    ///
    /// A copy-paste in [`Route`] that gave both arms the same region would make
    /// the cross-run assertion compare a button against itself — and it would
    /// go green, because pressing Cancel twice does produce two identical
    /// outcomes. The check would then be reporting agreement as a pass while
    /// measuring nothing at all.
    #[test]
    fn the_two_routes_are_actually_two() {
        assert_ne!(Route::Cancel.region(), Route::Keep.region());
        assert_ne!(Route::Cancel.reason(), Route::Keep.reason());
        assert_ne!(Route::Cancel.label(), Route::Keep.label());
    }

    /// **Both regions are under the host's footer namespace.**
    ///
    /// `Host::footer` publishes all three footer buttons under
    /// `dialog.buttons.`, and the failure message for a missing button lists
    /// that prefix's live names to help whoever reads it. A region name that
    /// drifted out of the prefix would produce a failure message that listed
    /// the button it was looking for as absent while showing it present.
    #[test]
    fn both_buttons_are_in_the_footer_namespace() {
        for route in [Route::Cancel, Route::Keep] {
            assert!(
                route.region().starts_with("dialog.buttons."),
                "{} publishes at `{}`, which is outside the namespace this check's failure \
                 messages enumerate",
                route.label(),
                route.region()
            );
        }
    }

    /// **The token the gesture produces is a single file token.**
    ///
    /// [`CHOSEN_TOKEN`] is compared literally against `print-open paper=`,
    /// which is whitespace-split. A value carrying a space would be truncated
    /// at the split and the comparison would fail against a perfectly correct
    /// build.
    #[test]
    fn the_chosen_token_survives_a_whitespace_split() {
        assert!(!CHOSEN_TOKEN.is_empty());
        assert!(!CHOSEN_TOKEN.contains(char::is_whitespace));
        assert_eq!(CHOSEN_TOKEN.to_ascii_lowercase(), CHOSEN_TOKEN);
        assert!(!CHOSEN_PICK.contains(char::is_whitespace));
    }
}
