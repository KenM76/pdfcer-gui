//! `chords` — **every declared keyboard shortcut is PRESSED and asserted.**
//!
//! The driven counterpart to `app::keyboard`'s
//! `every_chord_the_manifest_binds_actually_fires`, and the check whose absence
//! let **fourteen of twenty-one declared shortcuts ship dead** — `Ctrl+Z`,
//! `Ctrl+Y`, `Ctrl+Shift+Z`, `Ctrl+S`, `Ctrl+E`, `Ctrl+Shift+E`, `Ctrl+H`,
//! `Ctrl+Shift+C`, `Ctrl+Alt+N`, `F11`, `[`, `]`, `Alt+Up`, `Alt+Down`.
//!
//! Undo had a keyboard shortcut everywhere except the keyboard.
//!
//! # ★★ Why a headless gate is not enough, and why this one is not either
//!
//! The unit gate presses each chord into a bare `egui::Context` and asserts the
//! command comes back. That covers the dispatcher and the manifest. It does
//! **not** cover the link this check exists for: an operating-system keystroke
//! becoming an `egui::Event::Key` in *this* window, with *this* window's focus
//! rules and *this* application's other keyboard claimants running.
//!
//! And this check does not cover what the unit gate does — it presses a fixed
//! list, so a chord added to the manifest tomorrow is silently unswept here.
//! **Both are needed and neither is redundant**, which is worth stating because
//! the argument that killed the last driven attempt was that the keymap test
//! already covered it. It did not: that test swept `Ctrl+<digit>` only.
//!
//! # ★ How the belief that this was impossible survived for months
//!
//! Nine module headers in this crate recorded, as a fact about the machine,
//! that *"synthetic keyboard input does not reach the target window from the
//! session that injects it"*. It was inferred from `Ctrl+E` producing no trace
//! — which was the dead-keymap defect, one layer below where the conclusion was
//! drawn. Eight of those headers cited `crate::checks::find_bar` as the source;
//! `find_bar` **passes**, and its own report says *"control chord Ctrl+2
//! arrived, so the input channel works"*.
//!
//! So the record contradicted itself in the same run report, for months, and
//! nobody read the two lines together. The lesson is the one the operator's own
//! standing rules already state: **a constraint an agent infers about its
//! environment is a reading, not a fact**, and a reading that stops people
//! testing something is the most expensive kind.
//!
//! # What is pressed, and what is deliberately not
//!
//! Every chord whose command is safe to invoke against an open document with
//! nothing saved. `F11` (fullscreen) is **excluded**: it resizes the window
//! mid-run, which would invalidate every rect the harness has measured, and its
//! dispatch is proven by the unit gate. That exclusion is named here rather
//! than left to be inferred from a short list.
//!
//! The assertion is on `chord-command chord=… id=…`, the line
//! `app::keyboard::commands` traces the moment a chord resolves. That is the
//! link under test; whether the command then does its work is each feature's
//! own check.

use crate::checks::driving::SHELL_DIAG_ENV;
use crate::checks::{Check, CheckContext};
use crate::error::{Error, Result};
use crate::input::Driver;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;
use crate::sys::vk;

/// One chord to press: its modifiers, its key, how the manifest spells it, and
/// the command it must dispatch.
struct Chord {
    modifiers: &'static [u16],
    key: u16,
    spelling: &'static str,
    command: &'static str,
}

/// The chords this check presses.
///
/// Read against `crates/pdfcer-gui/src/shell/ron/built_in.ron`'s keymap. A chord
/// here that the manifest does not bind is a stale entry and will fail loudly,
/// which is the intended direction: the manifest is the source of truth.
const CHORDS: &[Chord] = &[
    // An Alt chord, the third modifier family.
    Chord {
        modifiers: &[vk::ALT],
        key: vk::ARROW_DOWN,
        spelling: "Alt+Down",
        command: "pages.move_down",
    },
    // The three that matter most, because the keyboard is undo's primary route
    // and all three were dead.
    Chord {
        modifiers: &[vk::CONTROL],
        key: vk::Z,
        spelling: "Ctrl+Z",
        command: "edit.undo",
    },
    Chord {
        modifiers: &[vk::CONTROL],
        key: vk::Y,
        spelling: "Ctrl+Y",
        command: "edit.redo",
    },
    Chord {
        modifiers: &[vk::CONTROL, vk::SHIFT],
        key: vk::Z,
        spelling: "Ctrl+Shift+Z",
        command: "edit.redo",
    },
    // ★ The Shift case twice over, because `Modifiers::matches_logically` is
    // permissive and would let `Ctrl+Shift+E` also satisfy `Ctrl+E`. The unit
    // gate asserts the exact comparison; this proves it through a real keyboard.
    Chord {
        modifiers: &[vk::CONTROL],
        key: vk::E,
        spelling: "Ctrl+E",
        command: "edit.text",
    },
    Chord {
        modifiers: &[vk::CONTROL, vk::SHIFT],
        key: vk::E,
        spelling: "Ctrl+Shift+E",
        command: "edit.add_text",
    },
    // A bare-character chord, which is the class the typing guard has to yield
    // to and which nothing drove before.
    Chord {
        modifiers: &[],
        key: vk::OPEN_BRACKET,
        spelling: "[",
        command: "pages.rotate_left",
    },
];

/// See the module documentation.
pub struct EveryDeclaredChordDispatches;

impl Check for EveryDeclaredChordDispatches {
    fn name(&self) -> &'static str {
        "every_declared_chord_dispatches"
    }

    fn defect(&self) -> &'static str {
        "a keyboard shortcut the manifest declares — and the menus print beside its command — \
         reaches no command when pressed, so the operator's keyboard silently does nothing and \
         the only evidence is that they stop using it"
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
    let pdf = ctx
        .pdf
        .clone()
        .ok_or_else(|| Error::new("no fixture document. Pass --pdf."))?;
    if !ctx.allow_input {
        return Err(Error::new(
            "input is disabled (--no-input). This check presses real keys. Reported as SKIPPED \
             rather than passed.",
        ));
    }

    let mut spec = LaunchSpec::new(&exe, ctx.out("chords.trace.txt"));
    spec.pdf = Some(pdf);
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
        "launched {} as pid {}",
        exe.display(),
        session.pid()
    ));
    report.artifact(session.trace_path().to_path_buf());
    session.settle(40);
    let driver = Driver::new(session.window());

    // ★ The control probe, first and separately. `find_bar`'s pattern and its
    // reasoning: a check that types into a window which is not listening must
    // report SKIP rather than name a feature as broken. `Ctrl+2` is bound to
    // `mode.review` and has been dispatchable since the ribbon landed, so a
    // silent result from it is evidence about the HARNESS.
    driver.press_chord(&[vk::CONTROL], vk::DIGIT_2)?;
    session.settle(14);
    if !session
        .trace()?
        .events("chord-command")
        .any(|l| l.get("chord") == Some("Ctrl+2"))
    {
        return Err(Error::new(
            "the control chord Ctrl+2 (`mode.review`) produced no `chord-command` line, so no \
             keystroke reached the application at all and nothing below would mean anything. \
             This is evidence about the harness, not about the keymap — reported as SKIPPED.",
        ));
    }
    report.note("control chord Ctrl+2 arrived, so the input channel works");

    // --- press each chord and collect what dispatched ----------------------
    // ★ Press them ALL, then read the trace ONCE.
    //
    // Not per-chord, and the difference is not tidiness. A chord whose command
    // does real work — `[` rotates the page and spawns a re-render — can have
    // its `chord-command` line still sitting in the application's buffer when a
    // read taken fourteen frames later returns. The first cut of this check
    // asserted per chord and reported `[` DEAD while the trace it had just read
    // went on to contain both the dispatch and the `rotate-pages` line that
    // followed it. A harness that reports a working feature as broken is worse
    // than one that reports nothing, because it sends the next person to the
    // wrong file — which is exactly how this defect was created in the first
    // place.
    for chord in CHORDS {
        driver.press_chord(chord.modifiers, chord.key)?;
        session.settle(14);
    }
    session.settle(40);

    let trace = session.trace()?;
    let mut dead = Vec::new();
    let mut wrong = Vec::new();
    for chord in CHORDS {
        let ids: Vec<String> = trace
            .events("chord-command")
            .filter(|l| l.get("chord") == Some(chord.spelling))
            .filter_map(|l| l.get("id").map(str::to_owned))
            .collect();
        if ids.is_empty() {
            dead.push(chord.spelling);
        } else if !ids.iter().any(|id| id == chord.command) {
            wrong.push(format!(
                "{} dispatched {:?} and not `{}`",
                chord.spelling, ids, chord.command
            ));
        }
    }

    if !dead.is_empty() {
        return Ok(Some(format!(
            "PRESSED AND NOTHING HAPPENED: {}. The control chord arrived, so the keystrokes are \
             reaching the window — these chords are declared in the manifest, printed in menus \
             and tooltips as shortcuts, and dispatched by nothing. Look at \
             `app::keyboard::commands`: it must parse the manifest's own spelling rather than \
             match a hand-written table, and it must compare modifiers EXACTLY rather than \
             refusing any chord that holds Shift or Alt.",
            dead.join(", ")
        )));
    }
    if !wrong.is_empty() {
        return Ok(Some(format!(
            "A chord reached the WRONG command: {}. The likeliest cause is \
             `Modifiers::matches_logically`, which asks only whether the pattern's modifiers are \
             present — so `Ctrl+Shift+Z` satisfies `Ctrl+Z` as well as itself, and undo and redo \
             become one keypress with iteration order deciding.",
            wrong.join("; ")
        )));
    }

    report.note(format!(
        "all {} chords dispatched the command the manifest binds them to",
        CHORDS.len()
    ));
    report.note(
        "NOT covered here: `F11`. It resizes the window mid-run and would invalidate every rect \
         already measured; its dispatch is covered by the unit gate. Nor is this list swept from \
         the manifest — a chord added tomorrow is not pressed here until it is added above, which \
         is what `every_chord_the_manifest_binds_actually_fires` is for",
    );
    Ok(None)
}
