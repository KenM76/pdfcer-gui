//! # `app::window` — the two verbs in View ▸ Window that change the *shape of
//! the application* rather than anything about the document
//!
//! `view.read_mode` (`Ctrl+H`) and `view.fullscreen` (`F11`). Both were
//! registered, drawn with a glyph, given a group of their own and bound to a
//! chord on the day the View tab was built, and both had **no dispatch arm**
//! until 2026-08-15 — five surfaces promising a behaviour that did not exist.
//! `RIBBON_IA.md` §3 named them together as the defect being fixed:
//!
//! > Read mode and full screen have **no ribbon control at all** — they are
//! > keyboard-only (Ctrl+H, F11) on a tab literally named View. This is the
//! > single most confusing thing in the current ribbon.
//!
//! They got controls; this file is the behaviour arriving behind them. It
//! exists as a module rather than as two `match` limbs because
//! `app::dispatch`'s header states the rule the limbs have to keep — *"the arms
//! route; they do not compute"* — and every decision below (what read mode
//! hides, what it deliberately keeps, how the two states are stored, which one
//! is allowed a shadow copy and which is not) is a rule rather than a route.
//!
//! # ★ 1. `view.read_mode` is NOT `mode.read`, and the difference decides
//! whether this file should exist at all
//!
//! This was the open question when the arm was written, and it is worth
//! recording in full because the honest answer to it could have been *delete
//! the command*. This shell has a **Read mode in the mode selector**
//! (`mode.read`, `Ctrl+1`), and a command called `view.read_mode` sitting one
//! tab over is exactly the shape of a duplicate that should be removed rather
//! than wired twice.
//!
//! It is not a duplicate. The two answer different questions, and they compose:
//!
//! | | `mode.read` | `view.read_mode` |
//! |---|---|---|
//! | What it is | a **named workspace plus a capability set** — `app::modes` | a **view stance**: chrome on or off |
//! | What it changes | which ribbon tabs exist, which panels are mounted, whether the canvas will author anything (`app::modes::capability`) | whether the ribbon and the docks are *drawn at all* |
//! | What it does not change | it still draws a ribbon (Read is shown `file` and `view`), still mounts panels, still has a status bar | nothing about capability: an operator in Edit ▸ read-mode can still author with a chord, and the canvas gate is untouched |
//! | Persistence | the mode and its arrangement are remembered per mode, on disk | per session, and per *window* rather than per document |
//! | Where it lives | the mode selector | View ▸ Window, beside Full screen |
//!
//! The reference applications agree, which is standing instruction 4's test.
//! **Acrobat** has both: a *Read Mode* (`Ctrl+H`) that hides the toolbars and
//! panes and gives the window to the page, and a separate notion of what the
//! reader is allowed to do to the file. **Inkscape** has the identical pair —
//! `F11` full screen and a "wide screen"/focus toggle that hides dialogs —
//! neither of which is a permission. **SolidWorks**' full screen (`F11`) hides
//! the CommandManager and the FeatureManager and changes nothing about whether
//! the model can be edited. Three of three separate *chrome* from *capability*,
//! and the chord this command carries — `Ctrl+H` — is Acrobat's chord for
//! precisely the chrome half.
//!
//! So the two are kept, they are orthogonal, and pressing one does not touch
//! the other: an operator can be in Review with the chrome hidden, and that is
//! a meaningful state rather than a contradiction.
//!
//! # ★ 2. What read mode hides, and the one thing it deliberately does not
//!
//! [`crate::text::commands::view_read_mode`] is the promise the operator is
//! shown, and it is the specification this keeps:
//!
//! > *Hide the ribbon and the panels and give the whole window to the page
//! > (Ctrl+H).*
//!
//! Two surfaces, named. So [`draws_chrome`] governs exactly two composition
//! steps in `PdfcerApp::ui` — the ribbon band and the docks — and **the status
//! bar stays**. That is a decision rather than an omission, and it has two
//! independent reasons:
//!
//! 1. **The words say ribbon and panels.** A status bar that vanished would be
//!    a third thing hidden by a control that named two, which is the sort of
//!    quiet over-delivery that makes an operator distrust the next label.
//! 2. **It is the only remaining route to page navigation and zoom.**
//!    `RIBBON_IA.md` §6 puts `page ◀ n/N ▶`, the editable page box and
//!    `zoom −/%/+` on that bar *because* they are the controls a reader touches
//!    constantly. Hiding them would leave a reading mode in which the reader
//!    cannot turn the page except by keyboard — the opposite of what the mode
//!    is for. Acrobat's own Read Mode keeps exactly this cluster, as a floating
//!    strip; pdfcer already has it as a fixed bar, so nothing has to float and
//!    `view.app_initiative`'s default of **Never** is not strained.
//!
//! ## Getting back out
//!
//! `Ctrl+H` again, and the tooltip on the control states the chord *before* the
//! operator presses it — which is the one moment they can still see the
//! control. That is Acrobat's contract too.
//!
//! **Escape was considered and declined.** `canvas::keys` already ranks four
//! claimants for Escape (a focused form field, the selection ladder's rungs, an
//! in-flight gesture, the page box's draft), and `SelectionLevel::ascend`
//! returning `EscapeOutcome::Nothing` is a documented fall-through seam that
//! this could have used. It is declined because the fall-through is *quiet*: an
//! operator who presses Escape to abandon a half-made marquee and instead finds
//! the whole application's chrome coming back has been surprised by a key that
//! means "no" everywhere else in this shell. If that changes it should change
//! as a ruling about Escape's ladder, in `canvas::keys`, and not as a fifth
//! claimant added here.
//!
//! # ★ 3. Where each state lives, and why they are stored differently
//!
//! | | stored in | read by |
//! |---|---|---|
//! | read mode | [`egui::Memory`], under [`READ_MODE_ID`] | [`read_mode`] — the frame composition and the `selected:` condition |
//! | full screen | the **viewport itself** (`ViewportInfo::fullscreen`) | [`fullscreen`] — the `selected:` condition |
//!
//! The asymmetry is deliberate and is the same argument `app::conditions`'
//! header makes about the armed canvas tool:
//!
//! > A shadow copy … would put the truth about which tool is armed in two
//! > places, and the failure mode is a ribbon that says Hand while the canvas
//! > selects — a disagreement no test would catch, because each half would be
//! > self-consistent.
//!
//! **Full screen has an owner outside this program.** The window manager can
//! put the window in and out of full screen without pdfcer being asked — a
//! double-click on the title bar, a window-manager chord, a display change. A
//! `bool` on `PdfcerApp` would then say one thing while the window said another,
//! and the ribbon control would render pressed over a windowed application.
//! `ViewportInfo::fullscreen` is the backend's own report, so there is nothing
//! to drift.
//!
//! **Read mode has no owner outside this program**, because nothing but this
//! command creates it. It needs a home the condition set can reach, and
//! `app::conditions` takes an `&egui::Context` and no `&mut self`, so
//! `egui::Memory` is the same route the armed tool and the armed region zoom
//! already take. It is *not* a shadow of anything.
//!
//! ## Consequences of that choice worth knowing before changing it
//!
//! * Read mode is **per window and per session**. It is not written to the
//!   layout store, so relaunching starts with the chrome shown. That is the
//!   right default — a shell that opened with no ribbon and no explanation of
//!   how to get one back is a shell that looks broken — and it is why nothing
//!   here touches [`crate::app::persistence`].
//! * Read mode is **not** per document. Closing and opening a file leaves it
//!   where the operator put it, which is what every reader in the class does.
//!
//! # 4. Why neither verb raises an `Action`
//!
//! `HANDOFF.md` §6's funnel is for work that touches a **document** or that
//! must not happen part-way through laying out a frame. Neither applies:
//! nothing here changes a byte of the file, so there is nothing for the undo
//! log to hold and nothing to order against. That is `file.print`'s and
//! `edit.find`'s reasoning, unchanged — a toggle that only decides what is
//! drawn next frame is exactly the class the funnel is not for.
//!
//! It also could not be done through the funnel without weakening it: the apply
//! phase is deliberately handed no [`egui::Context`], and both of these need
//! one — the memory write for read mode, and `send_viewport_cmd` for full
//! screen.

use egui::{Id, ViewportCommand};

/// `egui::Memory` key for "the chrome is hidden".
///
/// A named constant rather than a literal at both call sites, for the reason
/// [`crate::shell::commands::FILE_RECENT`] gives about its own id: a key spelled
/// twice that stops agreeing produces **silence** — a toggle that writes one
/// slot and a composition step that reads another — rather than an error.
const READ_MODE_ID: &str = "pdfcer-read-mode"; // ui-text-exempt: widget id, never displayed

/// Whether the ribbon and the docks are drawn this frame.
///
/// The single question `PdfcerApp::ui` asks of this module, phrased as what the
/// **frame** wants rather than as what the operator toggled, so the composition
/// step reads as a statement about the frame and does not have to know that
/// "read mode" is the reason.
///
/// The status bar is deliberately outside this — see §2 of the module header.
#[must_use]
pub fn draws_chrome(ctx: &egui::Context) -> bool {
    !read_mode(ctx)
}

/// Whether read mode is on.
///
/// The published state, read by [`draws_chrome`] and by
/// `PdfcerApp::conditions`, which turns it into the `selected:` condition that
/// renders the View ▸ Window control pressed. Two readers, one derivation.
#[must_use]
pub fn read_mode(ctx: &egui::Context) -> bool {
    ctx.data(|d| d.get_temp::<bool>(Id::new(READ_MODE_ID)))
        .unwrap_or(false)
}

/// Flip read mode, and report the state it landed in.
///
/// **The body of `view.read_mode`.** Returns the new value so the dispatch arm
/// can trace it without asking a second time — a second read is a second frame's
/// worth of opportunity for the two to disagree, and the trace is the only
/// evidence a harness has that the command did anything.
pub fn toggle_read_mode(ctx: &egui::Context) -> bool {
    let next = !read_mode(ctx);
    ctx.data_mut(|d| d.insert_temp(Id::new(READ_MODE_ID), next));
    // The chrome appearing or disappearing changes the space left for the
    // canvas, which an active `FitMode` recomputes its zoom from. Requesting
    // the repaint here rather than relying on the click's own is what makes
    // the **chord** behave identically: egui wakes on input, and a chord
    // pressed while nothing else is happening would otherwise leave the new
    // composition undrawn until the next unrelated event.
    ctx.request_repaint();
    next
}

/// Whether the window is in full screen, as the **windowing system** reports
/// it.
///
/// `None` from `ViewportInfo` — a backend that does not report the flag, and
/// the state of every headless `egui::Context` in the test suite — is read as
/// *not full screen*, which is the honest default: it is what a window that has
/// never been asked to fill the display is.
#[must_use]
pub fn fullscreen(ctx: &egui::Context) -> bool {
    ctx.input(|i| i.viewport().fullscreen).unwrap_or(false)
}

/// The `egui::Memory` key the last full-screen **request** is parked under,
/// with the frame it was made on.
const PENDING_FULLSCREEN: &str = "pdfcer.window.fullscreen-asked"; // ui-text-exempt: memory key.

/// How many frames a full-screen request is believed over the viewport's own
/// report before the report wins again.
///
/// ★ Bounded rather than latched, and the bound is the whole safety property. A
/// request the platform silently refuses — a window manager that does not do
/// full screen, a compositor that declines — would otherwise leave this shell
/// permanently convinced of a state the window is not in, and every subsequent
/// press would toggle a fiction.
///
/// Four is generous: `eframe` answers a viewport command on the next frame it
/// pumps, and this only has to outlast the round trip.
const PENDING_FRAMES: u64 = 4;

/// The value a press of `view.fullscreen` should ask the viewport for.
///
/// `reported` is what `ViewportInfo` says; `pending` is `(frame, state)` for a
/// request this shell has made and not yet seen confirmed, and `now` is the
/// current frame.
///
/// # ★★★ THIS IS A DEFECT FIX, AND IT LEFT THE OPERATOR'S DISPLAY FILLED
///
/// It was one line — `!current.unwrap_or(false)` — reading `ViewportInfo`
/// directly. The docs immediately below it already stated the reason it could
/// not work, and stated it as a *labelling* concern rather than as a bug:
///
/// > *"the command is queued and answered by the backend, so
/// > `ViewportInfo::fullscreen` still reports the old value on this frame."*
///
/// If the report lags the request, then a **second press before the backend has
/// caught up reads the pre-first-press state and asks for the same thing
/// again**. Full screen turns on and will not turn off.
///
/// Found by driving, and it is not rare: `read_mode_hides_the_chrome` reported
/// it on **two of three runs**, and the run it passed on was the one with more
/// frames between the presses. It was written off as harness flakiness after
/// the first — which is exactly the reading `D:/dev/rag/egui/`'s chord-matcher
/// finding warns about, where the same conclusion cost that project its whole
/// keyboard surface for months. **Three failures in three runs is an
/// intermittent, and an intermittent is a defect with a timing dependency.**
///
/// The failure branch of that check says *"the display has been left filled;
/// close the window to recover it"*, which is what an operator gets: a program
/// covering their screen that will not give it back except by being closed.
///
/// # The rule
///
/// **Trust the report, unless we have an outstanding request it has not yet
/// reflected.** Once the report agrees with what was asked, the request is
/// spent and the report wins again — so a full screen the operator triggers
/// *outside* this shell (a window manager's own key, a double-clicked title
/// bar) is honoured on the very next press rather than fought.
#[must_use]
pub fn next_fullscreen(reported: Option<bool>, pending: Option<(u64, bool)>, now: u64) -> bool {
    let current = match pending {
        // A request this shell made, recently, that the report has not caught
        // up with. Ours is the truth for now.
        Some((then, asked))
            if now.saturating_sub(then) <= PENDING_FRAMES && reported != Some(asked) =>
        {
            asked
        }
        // Either there is no outstanding request, or the report has confirmed
        // it, or it has been outstanding too long to believe. In all three the
        // windowing system's answer is the one to use — and an unreported state
        // counts as windowed, which is the honest default: it is what a window
        // that has never been asked to fill the display is.
        _ => reported.unwrap_or(false),
    };
    !current
}

/// Flip full screen, and report the state that was asked for.
///
/// **The body of `view.fullscreen`.** The returned value is what the viewport
/// was *asked* for, not what it is: the command is queued and answered by the
/// backend, so `ViewportInfo::fullscreen` still reports the old value on this
/// frame. That distinction is why the trace line the dispatcher writes says
/// `asked=` rather than `on=` — a reader of a trace from a machine they cannot
/// see should not be told a window is full screen on the strength of a request.
///
/// ★ And it is why the request is **remembered**: see [`next_fullscreen`] for
/// the defect that reading the lagging report alone produced.
pub fn toggle_fullscreen(ctx: &egui::Context) -> bool {
    let id = egui::Id::new(PENDING_FULLSCREEN);
    let now = ctx.cumulative_pass_nr();
    let pending: Option<(u64, bool)> = ctx.data(|d| d.get_temp(id));
    let reported = ctx.input(|i| i.viewport().fullscreen);
    let next = next_fullscreen(reported, pending, now);
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed.
        //
        // ★ It carries BOTH the report and the outstanding request, because the
        // whole defect was the two disagreeing. A line saying only `asked=true`
        // is identical for the build that worked and the build that asked for
        // the same thing twice.
        format!("fullscreen-toggle reported={reported:?} pending={pending:?} asked={next}")
    });
    ctx.data_mut(|d| d.insert_temp(id, (now, next)));
    ctx.send_viewport_cmd(ViewportCommand::Fullscreen(next));
    next
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **Read mode starts off, flips, and flips back.**
    ///
    /// Driven through the real memory slot rather than a local `bool`, because
    /// the slot is the thing two surfaces share: a toggle that wrote one key
    /// while `draws_chrome` read another would leave the ribbon drawn and the
    /// control pressed, and nothing else in the suite would notice.
    #[test]
    fn read_mode_starts_off_and_toggles_both_ways() {
        let ctx = egui::Context::default();
        assert!(
            !read_mode(&ctx),
            "a shell that opened with no ribbon and no way to say how to get \
             one back would look broken"
        );
        assert!(draws_chrome(&ctx));

        assert!(toggle_read_mode(&ctx), "the first press turns it on");
        assert!(read_mode(&ctx));
        assert!(
            !draws_chrome(&ctx),
            "the whole behaviour: the ribbon and the docks are not drawn"
        );

        assert!(!toggle_read_mode(&ctx), "the second press turns it off");
        assert!(draws_chrome(&ctx));
    }

    /// **`draws_chrome` is exactly the negation of `read_mode`**, asserted so
    /// that a future third thing (a presentation mode, a kiosk switch) has to
    /// change this test rather than silently widening one and not the other.
    #[test]
    fn the_frame_draws_chrome_exactly_when_read_mode_is_off() {
        let ctx = egui::Context::default();
        for _ in 0..3 {
            assert_eq!(draws_chrome(&ctx), !read_mode(&ctx));
            toggle_read_mode(&ctx);
        }
    }

    /// **An unreported viewport state counts as windowed.**
    ///
    /// The one rule in the full-screen half that a test can reach, and the one
    /// that decides whether the *first* press does anything. `None` is what a
    /// headless context reports and what a backend that does not track the flag
    /// reports; reading it as `true` would make the first press ask for
    /// `Fullscreen(false)` on a windowed application — a control that visibly
    /// does nothing on the only press most operators ever make.
    #[test]
    fn an_unreported_fullscreen_state_is_read_as_windowed() {
        assert!(
            next_fullscreen(None, None, 0),
            "the first press must fill the screen"
        );
        assert!(next_fullscreen(Some(false), None, 0));
        assert!(!next_fullscreen(Some(true), None, 0));
    }

    /// ★★★ **A second press while the report still lags turns full screen
    /// OFF**, which is the defect this function was rewritten for.
    ///
    /// The sequence, exactly as the driven check performs it:
    ///
    /// 1. frame 10, windowed, nothing outstanding → ask for `true`;
    /// 2. frame 11, **the report still says `false`** because the backend has
    ///    not answered yet — and the old implementation read that and asked for
    ///    `true` again, so full screen turned on and would not turn off.
    ///
    /// `read_mode_hides_the_chrome` reported this on **two of three runs**, and
    /// its failure branch says *"the display has been left filled; close the
    /// window to recover it"* — which is what an operator gets. It was written
    /// off as harness flakiness after the first, which is the reading
    /// `D:/dev/rag/egui/`'s chord-matcher finding warns about by name.
    #[test]
    fn a_second_press_before_the_backend_answers_still_toggles_off() {
        // Press one, on frame 10.
        assert!(next_fullscreen(Some(false), None, 10), "press one fills it");
        // Press two, on frame 11. The report has not caught up.
        assert!(
            !next_fullscreen(Some(false), Some((10, true)), 11),
            "★ the second press must ask for WINDOWED. Reading the lagging report \
             instead asks for full screen a second time, and the display never comes back"
        );
    }

    /// …and once the report agrees, the request is spent and the report wins.
    ///
    /// The other half of the rule, and what makes an **externally** triggered
    /// full screen — a window manager's own key, a double-clicked title bar —
    /// honoured on the very next press rather than fought.
    #[test]
    fn a_confirmed_request_hands_authority_back_to_the_report() {
        // We asked for `true` on frame 10 and the report now agrees.
        assert!(
            !next_fullscreen(Some(true), Some((10, true)), 12),
            "a confirmed request must not be believed over the report"
        );
        // The window manager took us out of full screen behind our back; the
        // next press must fill it again rather than "toggling off" a state we
        // are no longer in.
        assert!(
            next_fullscreen(Some(false), Some((10, true)), 20),
            "a stale request must not outlive its window"
        );
    }

    /// ★ **A request the platform never answers expires**, so a shell cannot be
    /// left permanently convinced of a state its window is not in.
    ///
    /// Bounded rather than latched, and the bound is the safety property: a
    /// window manager that declines full screen outright would otherwise make
    /// every subsequent press toggle a fiction.
    #[test]
    fn an_unanswered_request_expires_rather_than_latching() {
        // Asked on frame 10; it is now well past the window and the report has
        // never agreed. The report wins.
        assert!(
            next_fullscreen(Some(false), Some((10, true)), 10 + PENDING_FRAMES + 1),
            "an unanswered request must stop being believed"
        );
    }

    /// The headless context reports no viewport full-screen flag, which is the
    /// precondition the test above is about.
    ///
    /// Asserted rather than assumed: if a future egui reported `Some(false)`
    /// here, the rule would still be right but the reason written down for it
    /// would have stopped being true, and this is where that shows up.
    #[test]
    fn a_headless_context_reports_no_fullscreen_state() {
        let ctx = egui::Context::default();
        assert!(!fullscreen(&ctx));
    }
}
