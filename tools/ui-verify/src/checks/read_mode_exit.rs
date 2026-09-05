//! `read_mode_says_how_to_get_back_out` — while read mode is on, **the way out
//! is written on two surfaces, and it names the key the keymap actually
//! holds.**
//!
//! # ⬜ NOT RUN — written 2026-09-05 and never driven
//!
//! Stated first, in its own section, because a check nobody has run is not
//! evidence and this project has shipped twenty modules in that state. The
//! session that wrote it did not launch it: three other tracks were live in the
//! tree, one of them owning the pointer, and `RESUME.md`'s standing note is that
//! **a green test count is not a substitute for R1.** Whoever runs it first
//! should expect to correct it; see *What could still be wrong* at the foot.
//!
//! # The operator, 2026-09-05
//!
//! > *"I didn't see a way to get back out of read mode. if there is a shortcut
//! > for this it should have a note what the key combo is in the top bar that
//! > holds the window controls."*
//!
//! `view.read_mode` hides the ribbon and the docks. The only control that turns
//! it off — View ▸ Window ▸ Read mode — is **on the ribbon**. So from the moment
//! the mode is on, the control that undoes it is hidden by the thing it
//! toggles, and the only remaining route is a chord nothing on screen names.
//!
//! `app::window`'s header had answered this with *"the tooltip on the control
//! states the chord before the operator presses it"*. That is true about
//! Acrobat and it does not follow: `Ctrl+H` is a **bound chord**, pressable from
//! memory or by accident having pointed at nothing, and a click is not a hover.
//! **A tooltip is not a disclosure; it is a disclosure available to somebody who
//! already knows where to point.**
//!
//! # ★★★ The vacuous shapes this is written to avoid
//!
//! Three, and each of them is a check that passes on a broken build:
//!
//! | vacuous assertion | passes on |
//! |---|---|
//! | *the hint exists* | a hint naming a chord nothing is bound to |
//! | *the hint exists* | a hint shown permanently, including when read mode is **off** — furniture, and a false statement for every minute the mode is not on |
//! | *the hint says `Ctrl+H`* | ★ nothing — but it FAILS on a legitimate rebind, which makes it a second copy of the binding rather than a test of it |
//!
//! So what is asserted is an **identity between two derivations that a wrong
//! build breaks and a rebind does not**:
//!
//! 1. The application publishes `read-mode-exit chord="…"`, resolved by
//!    `app::window::publish_exit_chord` **from the keymap that dispatches**.
//!    That field is the keymap's own answer, whatever the manifest says today.
//! 2. Both operator-visible strings — the status line (`line=`) and the window
//!    title — must **quote that value verbatim**.
//!
//! A catalog that re-introduced a hard-coded `"Ctrl+H"` passes today and goes
//! red the first time anybody rebinds the command, which is exactly the moment
//! it becomes a lie. A rebind with the mechanism intact moves `chord=`, `line=`
//! and the title together, and this check stays green.
//!
//! # ★★ And the absence half, taken from a run that REACHED the state
//!
//! An absence assertion is vacuous when the run never reaches the state it is
//! asserting absence in. This one reads a **single launch that does both**:
//! `PDFCER_DIAG_INVOKE=view.read_mode` turns the mode on some frames after
//! start-up, and `window-title` is traced whenever it changes, so one trace
//! carries the ordinary title *and* the read-mode title.
//!
//! The absence is then derived rather than spelled:
//!
//! ```text
//! first  window-title  "pdfcer — 2026-09-05 06:24 UTC"
//! last   window-title  "Read mode — Ctrl+H to exit — pdfcer — 2026-09-05 06:24 UTC"
//! ```
//!
//! `last.ends_with(first)` and `last != first`, so the statement is a **prefix**
//! that was not there before. Nothing in this file spells the sentence, so a
//! rewording of the operator copy cannot make it fail — which is the property a
//! `contains("Read mode")` assertion would not have.
//!
//! ★ The prefix being a prefix is itself load-bearing, not incidental. A taskbar
//! button truncates from the right; a hint appended after the build stamp would
//! be the first thing the ellipsis eats, on the window of the one operator who
//! most needs it. It is also what keeps
//! [`super::title_build_stamp`]'s right-hand parse aimed at the build stamp.
//!
//! # ★ Full screen is checked by its absence
//!
//! `fullscreen=` must be **empty** here. Full screen hides no chrome of pdfcer's
//! own — the ribbon stays, its control stays, `app::conditions` renders it
//! pressed — so naming `F11` when the window is not full screen would be the
//! furniture this feature is written not to add. The one state where it *is* a
//! trap is read mode **and** full screen together, where the ribbon is gone and
//! the control with it; that combined state is covered by
//! `app::status::readmode`'s unit tests and is deliberately not driven here,
//! because filling the operator's display is a cost `read_mode_chrome` already
//! pays once and should not pay twice.
//!
//! # It needs no input at all
//!
//! No pointer, no keyboard. `PDFCER_DIAG_VIEWPORT` lays out a real window
//! without taking focus and `PDFCER_DIAG_INVOKE` rings the command through the
//! same `dispatch_command` a chord reaches. So this can run beside somebody
//! working — which for a check about a state an operator gets *stuck in* is
//! worth having, because it can then run often.
//!
//! ★ And no `--pdf`. Read mode is per **window**, not per document
//! (`app::window` §3), so the statement must appear with nothing open — and
//! `title_build_stamp`'s note applies: a check whose subject does not need a
//! document should not acquire a dependency on one, or a moved fixture turns it
//! into a SKIP and a SKIP is not red.
//!
//! # What a passing run does NOT prove
//!
//! That pressing the advertised key works. That is `chords`' subject — it
//! presses `Ctrl+H` through the OS and asserts `chord-command … id=view.read_mode`
//! — and it needs `--allow-input`. The two together are the whole claim: this
//! one says *the surface advertises what the keymap holds*, that one says *what
//! the keymap holds arrives*. Neither alone is enough and neither is redundant.
//!
//! # What could still be wrong, for whoever runs it first
//!
//! * **The invoke may not have landed by the settle.** `scripted_invoke` rings
//!   one id per frame and the first frames are start-up; if `read-mode on=true`
//!   is missing the check SKIPs with that said, rather than failing, because a
//!   command that never ran is not a defect in what it would have done.
//! * **The title trace is de-duplicated on change.** If a future build sent the
//!   title unconditionally, `first` and `last` would still be right, but a build
//!   that stopped sending it at all leaves one line and the check SKIPs.
//! * **`line=` and the title quote the chord with different spellings** would be
//!   a real failure and is the one this check is most likely to catch first.

use crate::checks::driving::SHELL_DIAG_ENV;
use crate::checks::{Check, CheckContext};
use crate::error::{Error, Result};
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;

/// The command rung through `PDFCER_DIAG_INVOKE`.
const SUBJECT_ID: &str = "view.read_mode";

/// The environment variable that rings a command without touching the pointer.
const INVOKE_ENV: &str = "PDFCER_DIAG_INVOKE";

/// The trace event the dispatcher writes when read mode flips.
const TOGGLED: &str = "read-mode";

/// The trace event the status bar's exit line writes.
const EXIT_LINE: &str = "read-mode-exit";

/// The trace event the frame writes when the window title changes.
const TITLE: &str = "window-title";

/// The region the status bar publishes for the exit line.
const REGION: &str = "status-group:read-mode-exit";

/// The status bar's own content strip, for the containment note.
const REGION_BAR: &str = "status-bar";

/// The rect-publishing event every named region uses.
const UI_RECT: &str = "ui-rect";

/// Where and how large the window is placed. `with_active(false)`, so it takes
/// neither focus nor pointer.
///
/// Wide, because a narrow bar sheds groups and this check would rather read a
/// bar that is not under width pressure — the shedding is `fitting`'s subject,
/// not this one's.
const VIEWPORT: &str = "0,0,1400,900";

/// See the module documentation.
pub struct ReadModeSaysHowToGetBackOut;

impl Check for ReadModeSaysHowToGetBackOut {
    fn name(&self) -> &'static str {
        "read_mode_says_how_to_get_back_out"
    }

    fn defect(&self) -> &'static str {
        "read mode hides the ribbon, and the only control that turns it off is on the ribbon — so \
         nothing on screen says how to get the application back, and the operator reported \
         exactly that: \"I didn't see a way to get back out of read mode\""
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
    let exe = ctx.resolve_exe().ok_or_else(|| {
        Error::new(format!(
            "no binary to drive. Pass --exe, or build the profile's default at {}.",
            ctx.profile.default_exe
        ))
    })?;

    let mut spec = LaunchSpec::new(&exe, ctx.out("read_mode_exit.trace.txt"));
    spec.env.push((
        ctx.profile.diag_env.0.to_owned(),
        ctx.profile.diag_env.1.to_owned(),
    ));
    spec.env
        .push((SHELL_DIAG_ENV.0.to_owned(), SHELL_DIAG_ENV.1.to_owned()));
    // ★ The whole of the input for this check. One command id, rung through the
    // same choke point a chord reaches — see `app::frame::scripted_invoke`.
    spec.env
        .push((INVOKE_ENV.to_owned(), SUBJECT_ID.to_owned()));
    spec.allow_stale = ctx.allow_stale;
    spec.source_root = ctx.source_root.clone();
    if let Some(name) = ctx.profile.viewport_env {
        spec.env.push((name.to_owned(), VIEWPORT.to_owned()));
    }

    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    report.note(format!(
        "launched {} as pid {} with {INVOKE_ENV}={SUBJECT_ID} — no pointer and no keystroke is \
         sent, so this can run beside somebody working",
        exe.display(),
        session.pid()
    ));
    report.artifact(session.trace_path().to_path_buf());
    session.settle(60);

    let trace = session.trace()?;

    // --- the command has to have run at all --------------------------------
    let Some(toggled) = trace.last(TOGGLED) else {
        return Err(Error::new(format!(
            "`{SUBJECT_ID}` was never dispatched — no `{TOGGLED}` line in the trace. That is a \
             harness condition, not a defect in what the command would have done: \
             `scripted_invoke` rings one id per frame and the settle may have been too short, or \
             `{INVOKE_ENV}` may not be read by this build. Re-run before concluding anything."
        )));
    };
    if toggled.get("on") != Some("true") {
        return Err(Error::new(format!(
            "`{SUBJECT_ID}` ran and left read mode OFF: `{}`. The invoke may have been rung \
             twice, or the mode was already on. Nothing about the exit statement can be judged \
             from a build that is not in the state.",
            toggled.raw.trim()
        )));
    }

    // --- the two titles, before and after ----------------------------------
    let titles: Vec<String> = trace
        .events(TITLE)
        .map(|l| title_of(&l.raw))
        .filter(|t| !t.is_empty())
        .collect();
    let (Some(first), Some(last)) = (titles.first(), titles.last()) else {
        return Err(Error::new(format!(
            "the application traced no `{TITLE}` line, so it never set a window title. Without \
             one there is nothing to compare and the absence half of this check would be vacuous."
        )));
    };
    if first == last {
        return Err(Error::new(format!(
            "the window title never changed: {first:?}. Read mode turned on (the `{TOGGLED}` \
             line is present) and the title is the same before and after — either the title is \
             sent unconditionally and de-duplication hid the change, or read mode reached the \
             title composition after the last send. Re-run with a longer settle."
        )));
    }
    report.note(format!("title before read mode: {first:?}"));
    report.note(format!("title in read mode:     {last:?}"));

    // --- the status bar's line ---------------------------------------------
    let Some(exit) = trace.last(EXIT_LINE) else {
        return Ok(Some(format!(
            "read mode turned on and the status bar said NOTHING about how to leave it — no \
             `{EXIT_LINE}` line at all. This is the reported defect exactly: the ribbon is \
             hidden, the control that undoes read mode is on the ribbon, and the status bar is \
             the one piece of chrome the mode deliberately keeps. If the bar drew the line but \
             did not trace it, the trace is what a check can see and the line is unguarded."
        )));
    };
    let chord = exit.get("chord").unwrap_or_default().to_owned();
    let line = exit.get("line").unwrap_or_default().to_owned();
    let fullscreen = exit.get("fullscreen").unwrap_or_default().to_owned();
    report.note(format!("status line: {line:?}"));

    if chord.is_empty() {
        return Ok(Some(format!(
            "the status bar drew its read-mode line and the keymap resolved NO CHORD for \
             `{SUBJECT_ID}`: {line:?}. In the shipped manifest `Ctrl+H` is bound, so an empty \
             `chord=` means either the keymap did not reach \
             `app::window::publish_exit_chord`, or the binding was removed. Note which: a build \
             with no chord is required to draw a BUTTON instead, and a line that neither names a \
             key nor offers a control is a room with no door."
        )));
    }
    report.note(format!("the keymap resolved the chord {chord:?}"));

    // --- ★★★ the identity this check exists for ----------------------------
    if !line.contains(&chord) {
        return Ok(Some(format!(
            "the keymap holds {chord:?} for `{SUBJECT_ID}` and the sentence the operator is \
             shown does not contain it: {line:?}. That is the drift this mechanism exists to \
             make impossible — a status bar naming one key while the keyboard answers to \
             another, on the one surface somebody turns to when they are already stuck. Look for \
             a hard-coded chord in `crate::text::window`, whose own test forbids one."
        )));
    }

    let Some(prefix) = last.strip_suffix(first.as_str()) else {
        return Ok(Some(format!(
            "the read-mode title is not the ordinary title with a statement in FRONT of it.\n  \
             ordinary: {first:?}\n  read mode: {last:?}\nThe statement must be a prefix. A \
             taskbar button truncates from the right, so a hint appended after the build stamp \
             is the first thing the ellipsis eats — on the window of the operator who most needs \
             it. It is also what keeps `the_title_bar_carries_the_build_time` aimed at the build \
             stamp rather than at this sentence."
        )));
    };
    let prefix = prefix.trim();
    if prefix.is_empty() {
        return Ok(Some(format!(
            "the read-mode title added nothing to the ordinary one: {last:?}"
        )));
    }
    report.note(format!("the title gained the prefix {prefix:?}"));

    if !prefix.contains(&chord) {
        return Ok(Some(format!(
            "the window title's read-mode statement does not name the chord the keymap holds \
             ({chord:?}): {prefix:?}. The operator asked for the key combo in \"the top bar that \
             holds the window controls\" — a statement there that does not name the key answers \
             a different question from the one he asked."
        )));
    }

    // --- ★★ the absence half, on a run that reached the state --------------
    if first.contains(prefix) {
        return Ok(Some(format!(
            "the exit statement is in the ORDINARY title too: {first:?}. A permanent hint is \
             furniture nobody reads, and it is a false statement for every minute read mode is \
             off — on the one surface that reaches an operator who is not looking at the \
             application at all."
        )));
    }
    if first.contains(&chord) {
        return Ok(Some(format!(
            "the ordinary title already names {chord:?}: {first:?}. It must appear only while \
             the mode it explains is on."
        )));
    }

    // --- ★ full screen is named only in the combined state -----------------
    if !fullscreen.is_empty() {
        return Ok(Some(format!(
            "the status line names a full-screen chord ({fullscreen:?}) on a window that is not \
             full screen: {line:?}. Full screen hides no chrome of pdfcer's own — the ribbon \
             stays and its control stays — so an always-on `F11` hint is the furniture this \
             feature was written not to add. It belongs in the combined state and nowhere else."
        )));
    }

    // --- the line occupies pixels, not merely a trace ----------------------
    //
    // ★ A sentence that was constructed and never laid out is indistinguishable
    // in a trace from one the operator can read. This is the cheap half of the
    // "on screen and legible" claim the other named regions on this bar make.
    let rect = trace
        .events(UI_RECT)
        .filter(|l| l.get("name") == Some(REGION))
        .last()
        .and_then(|l| l.get_rect("rect"));
    let Some(rect) = rect else {
        return Ok(Some(format!(
            "the status bar traced its read-mode line but published no `{REGION}` rect, so \
             nothing can say the sentence was laid out rather than merely built. Every other \
             named line on this bar publishes one."
        )));
    };
    if rect.width() <= 0.0 || rect.height() <= 0.0 {
        return Ok(Some(format!(
            "the read-mode line was laid out with no area: {rect:?}. A zero-width label is a \
             sentence the operator cannot read, which for this one is the whole failure."
        )));
    }
    report.note(format!("the line occupies {rect:?}"));

    if let Some(bar) = trace
        .events(UI_RECT)
        .filter(|l| l.get("name") == Some(REGION_BAR))
        .last()
        .and_then(|l| l.get_rect("rect"))
    {
        if bar.contains_rect(rect) {
            report.note(format!("…inside the status bar's strip {bar:?}"));
        } else {
            return Ok(Some(format!(
                "the read-mode line at {rect:?} is not wholly inside the status bar's strip \
                 {bar:?}. A line that runs past the bar's edge is a line whose visible half may \
                 be the half without the key in it — the shape defect O44 found twice on this \
                 surface at ui_scale 1.80."
            )));
        }
    }

    Ok(None)
}

/// The title out of a `window-title "…"` line.
///
/// The application traces it with `{:?}`, so the whole payload is one quoted
/// value rather than `key=value` fields. Parsed the way
/// [`super::title_build_stamp`] parses it, and for the same reason: the
/// left-hand part is a file name and may contain anything a path can.
fn title_of(raw: &str) -> String {
    raw.trim()
        .rsplit_once(TITLE)
        .map_or(String::new(), |(_, rest)| {
            rest.trim().trim_matches('"').to_owned()
        })
}
