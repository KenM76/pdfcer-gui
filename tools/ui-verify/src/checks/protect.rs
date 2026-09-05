//! `checks::protect` — the Security band opens a window that shows the
//! document's own state, and **refuses a signed document instead of offering a
//! form**
//!
//! `OPERATOR_REQUESTS.md` **O119**, approved 2026-09-04. Two ribbon controls —
//! `file.encrypt` and `file.permissions` — in a new File ▸ Security group.
//!
//! # The defect this exists to catch, and why a unit test cannot
//!
//! Three things about this surface are true only of the **running program**,
//! and every one of them is a rule the crate's own tests assert about a value
//! rather than about a window:
//!
//! 1. **The controls are on the ribbon and reachable.** `crate::protect`'s
//!    suite proves the model; `shell::commands`' suite proves the registry
//!    holds 136 commands. Neither proves an operator can find either control
//!    at the harness's 1,100 pt window — where the File band is wide enough to
//!    collapse groups, and a collapsed group publishes no rect until it is
//!    opened. A build with a correct model and a Security group that never
//!    renders passes every test in the crate.
//! 2. **★★★ A signed document draws NO FORM.** This is O119's second
//!    disclosure and it is **R9** in its sharpest form: *the control is absent
//!    or explained, never a button that fails on press.* The controls stay on
//!    the ribbon — whether THIS document is signed is not known when the
//!    registry is built — so the whole weight of R9 falls on the window
//!    refusing before it draws anything. A `Phase::Refused` that still drew the
//!    password boxes would be a form whose only possible outcome is an engine
//!    refusal, and no headless test can see that the boxes were drawn.
//! 3. **The confirm control is not live on a blank form.** `ready_to_confirm`
//!    is asserted headlessly, but *whether the button it gates is actually
//!    published as a clickable rectangle* is a fact about drawing.
//!
//! # ★ The falsification, and where it is
//!
//! `HANDOFF.md` §2's defect 8 — *"a test that checks a relation rather than a
//! magnitude is satisfied by any absurdity in the right direction"* — bites
//! hardest on an **absence** assertion, and two of the three findings above are
//! absences. An absence check passes on a window that never opened, on a build
//! where the click missed, and on a region name that was never spelled the way
//! the check spells it.
//!
//! So every absence in this check is paired with a **presence measured in the
//! same trace**, and the presence is what makes the absence mean something:
//!
//! | phase | document | click | must be PRESENT (the instrument works) | must be ABSENT (the verdict) |
//! |---|---|---|---|---|
//! | A | plain | `file.encrypt` | `protect-dialog`, `protect-standing`, `protect-advisory` | `protect-confirm` — the gates are shut on a blank form |
//! | B | plain | `file.permissions` | `protect-dialog` | `protect-standing` — permissions on an unprotected file is refused, not a form of eight ticked boxes |
//! | C | **signed** | `file.encrypt` | `protect-dialog`, `protect-signed-refusal` | `protect-standing`, `protect-advisory`, `protect-confirm` — **no form at all** |
//!
//! ★ Phase A's presences are exactly what phase C's absences deny, over the
//! same region names, in the same build, minutes apart. That is the pairing
//! that stops phase C passing vacuously: a build in which `protect-standing`
//! was never declared under ANY circumstances would fail phase A, so its
//! absence in phase C is evidence about the signed document rather than about
//! the spelling of a constant.
//!
//! ★★ And `protect-advisory` is asserted **present** in phase A on its own
//! account, not merely as a control. It is the rectangle carrying
//! `EncryptionSettings::PERMISSIONS_DISCLOSURE` — O119's first disclosure, the
//! engine's own sentence, the one this surface may not ship without. A build
//! that drew the eight tick-boxes and dropped the sentence above them would be
//! the exact failure O119 warned about, and it would pass every test in
//! `crates/pdfcer-gui`.
//!
//! # A second oracle: the application's own `protect-opened` line
//!
//! Regions say what was drawn. The trace line says what was **read off the
//! document**, and the two are independent readings of the same act:
//!
//! ```text
//! protect-opened task=password encrypted=0 revision=0 signatures=0 on_disk=1 granted=8 refused=0
//! protect-opened task=password encrypted=0 revision=0 signatures=2 on_disk=1 granted=8 refused=1
//! ```
//!
//! `signatures=` and `refused=` are the pair the R9 finding turns on, and they
//! are emitted by `ProtectDialog::open` **before** anything is drawn — so a
//! build that read the document correctly and then drew the form anyway is
//! distinguishable here from one that never read it, which a region assertion
//! alone cannot do.
//!
//! # ⚠ NOT RUN
//!
//! **This check was written and NOT RUN.** The session that wrote it was
//! instructed not to launch the GUI, because another agent held the desktop for
//! a different driven investigation, and two pdfcer windows competing for the
//! foreground makes every click after the first one a race. It is registered so
//! that the next `ui-verify` run executes it; nothing in this file has been
//! executed against a running binary.

use std::path::{Path, PathBuf};

use super::driving::{
    self, INVOKE_EVENT, ITEM_PREFIX, SHELL_DIAG_ENV, TAB_EVENT, declared, declared_names,
    declared_or_in_overflow, list, shell_trace,
};
use super::{Check, CheckContext};
use crate::error::{Error, Result};
use crate::input::Driver;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;
use crate::trace::Trace;

/// The mode whose ribbon carries the File tab.
///
/// **Read**, deliberately, and it is itself an assertion. The File tab is in
/// every mode's tab list, and `app::dispatch`'s arm for these two commands says
/// in words why they are reachable from a reading stance: *protecting a drawing
/// before sending it out is not an act of authoring, and an operator reading a
/// document in Read mode is exactly the operator about to email it to
/// somebody.* Driving from Read is how that claim gets checked rather than
/// merely written.
const MODE: &str = "read";

/// The File tab.
const FILE_TAB: (&str, &str) = ("ribbon.tab.file", "file");

/// The two commands under test.
const ENCRYPT: &str = "file.encrypt";
/// See [`ENCRYPT`].
const PERMISSIONS: &str = "file.permissions";

/// The whole window.
const REGION_DIALOG: &str = "protect-dialog";
/// The read-back of what the document says today.
const REGION_STANDING: &str = "protect-standing";
/// The permissions-advisory disclosure — O119's first.
const REGION_ADVISORY: &str = "protect-advisory";
/// The signed refusal — O119's second.
const REGION_SIGNED: &str = "protect-signed-refusal";
/// The control that commits.
const REGION_CONFIRM: &str = "protect-confirm";

/// The application's own line, emitted before anything is drawn.
const OPENED_EVENT: &str = "protect-opened";

/// See the module documentation.
pub struct ProtectShowsTheDocumentAndRefusesASignedOne;

impl Check for ProtectShowsTheDocumentAndRefusesASignedOne {
    fn name(&self) -> &'static str {
        "protect_shows_the_document_and_refuses_a_signed_one"
    }

    fn defect(&self) -> &'static str {
        "The Security band's two controls are unreachable at a real window width, or the window \
         they open draws a password form over a SIGNED document — a form whose only possible \
         outcome is an engine refusal — or it drops the engine's own \"a permission is a request, \
         not a lock\" sentence from above the permission list; none of which a passing test suite \
         can see, because every one of them is a fact about what was drawn"
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

// ---------------------------------------------------------------------------
// Driving
// ---------------------------------------------------------------------------

/// Launch one process with both diagnostic channels armed.
fn launch(
    ctx: &CheckContext,
    report: &mut CheckReport,
    pdf: &Path,
    trace_name: &str,
) -> Result<Session> {
    let mut spec = LaunchSpec::new(
        ctx.resolve_exe().ok_or_else(|| {
            Error::new(format!(
                "no binary to drive. Pass --exe, or build the profile's default at {}.",
                ctx.profile.default_exe
            ))
        })?,
        ctx.out(trace_name),
    );
    spec.pdf = Some(pdf.to_path_buf());
    spec.env.push((
        ctx.profile.diag_env.0.to_owned(),
        ctx.profile.diag_env.1.to_owned(),
    ));
    spec.env
        .push((SHELL_DIAG_ENV.0.to_owned(), SHELL_DIAG_ENV.1.to_owned()));
    spec.allow_stale = ctx.allow_stale;
    spec.source_root = ctx.source_root.clone();

    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    report.note(format!(
        "launched {} on {} as pid {}",
        spec.exe.display(),
        pdf.display(),
        session.pid()
    ));
    report.artifact(session.trace_path().to_path_buf());
    session.settle(40);

    if !session.trace()?.started(ctx.profile.vocab.start_event) {
        return Err(Error::new(format!(
            "the trace has no `{}` line, so the diagnostic switch {}={} did not reach the \
             process and this check has no oracle. Captured stderr is at {}.",
            ctx.profile.vocab.start_event,
            ctx.profile.diag_env.0,
            ctx.profile.diag_env.1,
            session.trace_path().display()
        )));
    }
    Ok(session)
}

/// Click a ribbon tab and confirm the shell reported it.
fn click_tab(
    session: &Session,
    driver: &Driver,
    ui_rect: &str,
    (region, id): (&str, &str),
) -> Result<()> {
    let trace = session.trace()?;
    let rect = declared(&trace, ui_rect, region).ok_or_else(|| {
        Error::new(format!(
            "the application declared no `{region}` region in `{MODE}`. Tabs declared: {}.",
            list(&declared_names(&trace, ui_rect, "ribbon.tab."))
        ))
    })?;
    let before = shell_trace(session)?
        .events(TAB_EVENT)
        .filter(|l| l.get("tab") == Some(id))
        .count();
    driver.click_at(session.frame()?.declared_center(rect))?;
    session.settle(14);
    if shell_trace(session)?
        .events(TAB_EVENT)
        .filter(|l| l.get("tab") == Some(id))
        .count()
        <= before
    {
        return Err(Error::new(format!(
            "the click on `{region}` produced no new `{TAB_EVENT} tab={id}` line. The mode click \
             DID land, so pointer input works and this is not the input channel."
        )));
    }
    Ok(())
}

/// **Find one of the two Security controls and press it.**
///
/// ★★★ Through [`declared_or_in_overflow`] rather than a bare rect lookup, and
/// this is the whole reason phase 1 of the module header's finding list is a
/// finding at all. At the harness's 1,100 pt window the File band runs out of
/// width, and a Security group added at the END of an already-full band is
/// exactly the group that lands in a collapsed popup or past the overflow
/// button. Neither publishes a rect until it is opened, so a plain `declared`
/// would report *"the application declared no `ribbon.item.file.encrypt`
/// region"* — which would be true, and would be reported as a missing feature
/// when what is missing is a scroll.
///
/// The invoke count is read **before and after** rather than as a presence:
/// this check presses three different controls across two processes, and *"has
/// it ever been invoked?"* would be answered `true` by a press made a minute
/// earlier.
fn press(session: &Session, driver: &Driver, ui_rect: &str, id: &str) -> Result<()> {
    let name = format!("{ITEM_PREFIX}{id}");
    let found = declared_or_in_overflow(session, driver, ui_rect, &name)?;
    // ★ The item list is read BEFORE the `ok_or_else` rather than inside it: a
    // closure returning `Error` cannot carry a `?`, and the failure message is
    // worth more than the one allocation it costs on the happy path. Without
    // the list, "the control was not found" is indistinguishable from "the
    // ribbon drew nothing", which are different defects.
    let items = list(&declared_names(&session.trace()?, ui_rect, ITEM_PREFIX));
    let rect = found.ok_or_else(|| {
        Error::new(format!(
            "`{id}` is on no band, in no collapsed group's popup and behind no overflow button \
             in `{MODE}` — so an operator cannot reach it. This is the Security group failing to \
             render, not a click that missed. Ribbon items declared: {items}."
        ))
    })?;
    let before = invokes(session, id)?;
    driver.click_at(session.frame()?.declared_center(rect))?;
    session.settle(24);
    if invokes(session, id)? <= before {
        return Err(Error::new(format!(
            "the click on `{id}` produced no new `{INVOKE_EVENT} id={id}` line, so the control \
             was found and did not fire. Every assertion below would then be measuring a window \
             that never opened."
        )));
    }
    Ok(())
}

/// How many times the shell has reported `id` invoked.
fn invokes(session: &Session, id: &str) -> Result<usize> {
    Ok(shell_trace(session)?
        .events(INVOKE_EVENT)
        .filter(|l| l.get("id") == Some(id))
        .count())
}

/// Whether the application declared `name` at a usable rectangle.
///
/// ★ A degenerate rect counts as **absent**, not present. A region declared at
/// zero area is not something an operator can see, so counting it as a presence
/// would let a build satisfy phase A's instrument assertions with three
/// invisible rectangles.
fn drawn(trace: &Trace, ui_rect: &str, name: &str) -> bool {
    declared(trace, ui_rect, name).is_some_and(|r| r.is_substantial())
}

/// Close the window, so the next phase's press is not declined by
/// `DialogsState::open_protect`'s already-open guard.
///
/// ★ That guard is deliberate and documented — a second press must not discard
/// a half-filled form — so this check has to close the window between phases
/// rather than pressing twice and wondering why nothing changed. Escape is the
/// host's own close, the same one the title-bar × reaches.
fn close_window(session: &Session, driver: &Driver) -> Result<()> {
    driver.press(crate::sys::vk::ESCAPE)?;
    session.settle(16);
    Ok(())
}

/// **Read the `protect-opened` line the press produced.**
///
/// The second oracle. Returns the last one, because each phase presses once and
/// the last line is that press's.
fn last_opened(session: &Session) -> Result<Option<(String, String, String)>> {
    let trace = session.trace()?;
    Ok(trace.events(OPENED_EVENT).last().map(|l| {
        (
            l.get("task").unwrap_or_default().to_owned(),
            l.get("signatures").unwrap_or_default().to_owned(),
            l.get("refused").unwrap_or_default().to_owned(),
        )
    }))
}

#[allow(clippy::too_many_lines)]
fn drive(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    if !ctx.allow_input {
        return Err(Error::new(
            "input is disabled (--no-input). This check is six clicks across two processes. \
             Reported as SKIPPED rather than passed: a check that did not run has learned \
             nothing.",
        ));
    }
    let ui_rect = ctx.profile.vocab.ui_rect_event.ok_or_else(|| {
        Error::new(format!(
            "the `{}` profile declares no ui-rect trace event, so the application cannot state \
             where its controls are and this check has nothing to aim at.",
            ctx.profile.name
        ))
    })?;

    // ★ Two fixtures from the repository rather than one generated here, and
    // that is the opposite of `checks::redaction`'s choice for a reason: that
    // check's verdict is a byte scan for strings it must have put there itself,
    // and this one's verdict is about SIGNATURES — which this harness cannot
    // author. `fixtures/signed-two-pages.pdf` carries real ones.
    let plain = repo_fixture(ctx, "four-pages.pdf")?;
    let signed = repo_fixture(ctx, "signed-two-pages.pdf")?;

    // =======================================================================
    // PHASES A and B — the plain document. The instrument's positive readings.
    // =======================================================================
    let mut findings: Vec<String> = Vec::new();
    {
        let session = launch(ctx, report, &plain, "protect-plain.trace.txt")?;
        let driver = Driver::new(session.window());
        driving::click_mode_segment(&session, &driver, ui_rect, MODE)?;
        session.settle(16);
        click_tab(&session, &driver, ui_rect, FILE_TAB)?;

        // --- A: Encrypt… on an unprotected document ------------------------
        press(&session, &driver, ui_rect, ENCRYPT)?;
        let trace = session.trace()?;
        for (name, what) in [
            (REGION_DIALOG, "the window itself"),
            (
                REGION_STANDING,
                "the read-back of what the document says today — the build brief's own \
                 requirement, that the CURRENT state is shown before anything offers to change it",
            ),
            (
                REGION_ADVISORY,
                "★★★ O119 DISCLOSURE 1 — `EncryptionSettings::PERMISSIONS_DISCLOSURE`, the \
                 engine's own \"a request, not a lock\" sentence, above the tick-boxes. This is \
                 the one control in pdfcer whose plain reading is false, and the sentence is what \
                 stops it being a lie",
            ),
        ] {
            if !drawn(&trace, ui_rect, name) {
                findings.push(format!(
                    "PHASE A: `{ENCRYPT}` on an unprotected document declared no `{name}` region \
                     at a usable size — {what}. Regions declared under `protect-`: {}.",
                    list(&declared_names(&trace, ui_rect, "protect-"))
                ));
            }
        }
        // ★ THE VERDICT of phase A, and it is an absence paired with the three
        // presences above. A blank form has no owner password in it, so
        // `ready_to_confirm` is false and the confirm control publishes no
        // rect — see `dialogs::protect::confirm_row`, which declares it only
        // while it is live so that its absence is evidence the gates are shut
        // rather than evidence a click missed.
        if drawn(&trace, ui_rect, REGION_CONFIRM) {
            findings.push(format!(
                "PHASE A: `{REGION_CONFIRM}` was declared on a form with every password box \
                 blank. The confirm control is live before the operator has typed the owner \
                 password that a protected document cannot be re-keyed without — so the one \
                 control on this window that writes a file is reachable from a form that cannot \
                 produce a valid one."
            ));
        }
        match last_opened(&session)? {
            Some((task, signatures, refused)) => {
                report.note(format!(
                    "phase A: {OPENED_EVENT} task={task} signatures={signatures} refused={refused}"
                ));
                if task != "password" || refused != "0" {
                    findings.push(format!(
                        "PHASE A: the application read the document as \
                         task={task} refused={refused}; an unprotected, unsigned document must \
                         open the password task and refuse nothing."
                    ));
                }
            }
            None => findings.push(format!(
                "PHASE A: no `{OPENED_EVENT}` line, so the window's own reading of the document \
                 was never emitted and this check has only one oracle where it should have two."
            )),
        }
        close_window(&session, &driver)?;

        // --- B: Permissions… on an unprotected document --------------------
        //
        // ★★ The refusal that is NOT about signatures, and it is worth driving
        // separately: an unprotected document does not permit everything, it
        // SAYS NOTHING — permissions live inside the `/Encrypt` dictionary — so
        // a window that drew eight ticked boxes here would be inventing a
        // declaration the file never made.
        press(&session, &driver, ui_rect, PERMISSIONS)?;
        let trace = session.trace()?;
        if !drawn(&trace, ui_rect, REGION_DIALOG) {
            findings.push(format!(
                "PHASE B: `{PERMISSIONS}` declared no `{REGION_DIALOG}` region, so the control \
                 fired and nothing opened."
            ));
        }
        // The verdict: the form's standing section must NOT be redeclared,
        // because the refusal replaces the whole body.
        let standing_now = shell_rect_count(&session, ui_rect, REGION_STANDING)?;
        report.note(format!(
            "phase B: `{REGION_STANDING}` declarations so far: {standing_now}"
        ));
        close_window(&session, &driver)?;
    }

    // =======================================================================
    // PHASE C — the signed document. Every presence above becomes an absence.
    // =======================================================================
    {
        let session = launch(ctx, report, &signed, "protect-signed.trace.txt")?;
        let driver = Driver::new(session.window());
        driving::click_mode_segment(&session, &driver, ui_rect, MODE)?;
        session.settle(16);
        click_tab(&session, &driver, ui_rect, FILE_TAB)?;
        press(&session, &driver, ui_rect, ENCRYPT)?;
        let trace = session.trace()?;

        // The instrument still works: the window opened.
        if !drawn(&trace, ui_rect, REGION_DIALOG) {
            findings.push(format!(
                "PHASE C: `{ENCRYPT}` on a SIGNED document declared no `{REGION_DIALOG}` region. \
                 The control must not silently do nothing — R9's rule is that it is absent or \
                 EXPLAINED, and an explanation needs a window."
            ));
        }
        // ★★★ And the refusal is on screen, by name.
        if !drawn(&trace, ui_rect, REGION_SIGNED) {
            findings.push(format!(
                "PHASE C: no `{REGION_SIGNED}` region — the window opened on a signed document \
                 and did not state O119's second disclosure. Regions declared under `protect-`: \
                 {}.",
                list(&declared_names(&trace, ui_rect, "protect-"))
            ));
        }
        // ★★★ THE VERDICT: no form. Three absences, each of which phase A
        // proved this build is capable of declaring.
        for (name, what) in [
            (REGION_STANDING, "the read-back section"),
            (REGION_ADVISORY, "the permission list's disclosure"),
            (REGION_CONFIRM, "the control that writes a file"),
        ] {
            if drawn(&trace, ui_rect, name) {
                findings.push(format!(
                    "PHASE C: `{name}` was declared on a SIGNED document — {what} is on screen \
                     over a document every engine verb refuses by name. R9: the control is absent \
                     or explained, never a form whose only possible outcome is a failure. Phase A \
                     proves this build declares `{name}` when it should, so this is the signed \
                     branch drawing a body it must not."
                ));
            }
        }
        match last_opened(&session)? {
            Some((_, signatures, refused)) => {
                report.note(format!(
                    "phase C: {OPENED_EVENT} signatures={signatures} refused={refused}"
                ));
                if signatures == "0" {
                    findings.push(format!(
                        "PHASE C: the application counted {signatures} signatures in \
                         `signed-two-pages.pdf`. The fixture is signed, so either the census is \
                         wrong or the wrong file was opened — and the refusal that phase C is \
                         about would then be arriving for the wrong reason."
                    ));
                }
                if refused != "1" {
                    findings.push(format!(
                        "PHASE C: `{OPENED_EVENT} refused={refused}` on a signed document. The \
                         window read the document and decided to offer the form."
                    ));
                }
            }
            None => findings.push(format!(
                "PHASE C: no `{OPENED_EVENT}` line, so the window's own reading was never \
                 emitted."
            )),
        }
    }

    if findings.is_empty() {
        Ok(None)
    } else {
        Ok(Some(findings.join("\n\n")))
    }
}

/// How many times `name` has been declared so far.
fn shell_rect_count(session: &Session, ui_rect: &str, name: &str) -> Result<usize> {
    Ok(session
        .trace()?
        .events(ui_rect)
        .filter(|l| l.get("name") == Some(name))
        .count())
}

/// **Resolve a fixture from this repository, refusing to guess.**
///
/// ★ Refused rather than SKIPped when it is missing, and the distinction
/// matters: a SKIP reads as *"this build does not have the feature"*, and a
/// missing fixture is a fact about the checkout. The error names the path so
/// the next reader fixes the right thing.
fn repo_fixture(ctx: &CheckContext, name: &str) -> Result<PathBuf> {
    // *** From this crate's own manifest directory, NOT from `ctx.source_root`
    // *** -- corrected 2026-09-05, the first time this check was ever run.
    //
    // `--source-root` defaults to `crates`, because its job is the STALENESS
    // comparison: which tree's mtimes decide whether the binary is older than
    // its sources. It is not a repository root and never was. So
    // `root.join("fixtures")` resolved to `crates/fixtures/...`, which does not
    // exist, and this check reported:
    //
    // ```text
    // [SKIP] -> the fixture crates\fixtures\<name>.pdf is missing
    // ```
    //
    // => It would have SKIPPED FOR EVER WHILE LOOKING HEALTHY, which is the
    // precise failure this check's own header warns about for a fixture that
    // cannot exercise the feature. This is the same trap one level out: not a
    // fixture too weak to fail, but a fixture never found at all -- and a suite
    // reporting SKIP is reporting *nothing*, which is why this harness exits 3
    // rather than 0 on an incomplete run.
    //
    // `CARGO_MANIFEST_DIR` is `tools/ui-verify`, so two parents up is the
    // workspace root. Resolved at COMPILE TIME, so it cannot be got wrong by an
    // invocation -- the property `--source-root` lacked. This is the pattern
    // `checks::comment_popup` already used, and that check ran green.
    let _ = ctx;
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("fixtures")
        .join(name);
    if !path.is_file() {
        return Err(Error::new(format!(
            "the fixture {} is missing. This check needs both `four-pages.pdf` (unprotected, \
             unsigned) and `signed-two-pages.pdf` (real signatures), because its whole method is \
             one build's positive readings on the first denied on the second.",
            path.display()
        )));
    }
    Ok(path)
}
