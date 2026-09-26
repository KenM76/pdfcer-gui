# `ui-verify/checks/read_mode_chrome`

`read_mode_hides_the_chrome` — the regression test for **`view.read_mode`,
a command that had a control, a glyph, a group, a chord and a line in the
shortcuts reference, and no dispatch arm at all.**

# The defect class this exists for

`RIBBON_IA.md` §3, on the shell being replaced:

> Read mode and full screen have **no ribbon control at all** — they are
> keyboard-only (Ctrl+H, F11) on a tab literally named View. This is the
> single most confusing thing in the current ribbon.

This shell gave both a control, and for a day that was all it gave them:
`shell::commands::reach` found `view.read_mode` among the eleven registered
commands whose honest status was *"the control should not be drawn yet"*,
with the note that a chord that does nothing **cannot even be greyed**. The
arm landed on 2026-08-15 (`app::window`), and this is the check that says so
from outside the process.

# Why a unit test cannot cover it, which is the bar for being here

[`crate::checks`]' rule: *"it must fail against a build where the wiring is
absent, and the wiring must be something no unit test in the workspace can
observe."*

`app::window` is fully unit-tested — the memory slot flips, `draws_chrome`
is its negation, both directions, three times over. **Every one of those
tests passes against a build where `PdfcerApp::ui` never calls
`draws_chrome`.** The whole behaviour is one `if` in the frame composition,
and a composition step's effect is observable only in a window: it is the
identical shape to the defect `measure_linear` exists for, where four
passing unit tests sat behind a `conditions` call site nobody had written
and the button never lit up.

# The phases, and why phase B alone would not do

| Phase | Move | Evidence | If it does not hold |
|---|---|---|---|
| 0 | click `view.fullscreen`, twice | the **client area** grows to the display and comes back | FAIL — see the phase-0 section below |
| A | click View, read `central-panel` | the canvas's rect **before** | SKIP — nothing was measured to compare against |
| B | click `view.read_mode` | `ribbon-command-invoked id=view.read_mode`, then `read-mode on=true` | SKIP if no invoke (no click landed); **FAIL** if invoked and no `read-mode` line — the arm is missing |
| C | read `central-panel` again, and capture | the rect's **top moved up**, and the pixels where the ribbon was **changed** | FAIL — the command ran and the frame did not change |

Phase B alone is what a trace-only check would assert, and it is not enough:
`read-mode on=true` proves the *toggle* flipped, which is precisely what the
unit tests already prove. Phase C is the part that is new — that the frame
composition read the flag — and it is asserted **twice, in two channels**,
because each covers the other's blind spot:

* **the rect** is exact and cheap and would be satisfied by a build that
  moved the canvas without repainting anything;
* **the pixels** cannot be faked by an arithmetic error, and would be
  satisfied by any global repaint — a hover, a theme change, a resize —
  which is why the rect is checked as well.

# It goes one way, and cannot come back

**The exit from read mode is `Ctrl+H`, and this machine cannot inject
keystrokes reliably** — `find_bar`'s first run reported Find broken on a
build where Find worked, which is why every other driving check in this
suite uses the mouse only. Once the chrome is hidden there is no ribbon
control left to click, by construction: that is the feature.

So this check ends in read mode and lets the session be killed, which costs
nothing (the process is torn down after every check and read mode is
per-session by design — see `app::window` §3, which is also why the *next*
check's launch is unaffected). What it means for coverage is stated plainly
rather than papered over: **the return trip is not driven here.** It is
covered by `app::window::tests::read_mode_starts_off_and_toggles_both_ways`
as a state machine, and by nothing at all as a frame. If a way to inject
`Ctrl+H` arrives, phase D is one more `settle` and one more rect read.

# Phase 0 drives `view.fullscreen`, and does it FIRST

The two arms landed in the same change, in the same module, in the same
ribbon group, and they share the one expensive precondition this check has
(a window wide enough for View's seventh group — see the launch site). A
second check module would have to restate that placement argument, which is
the "two hand-written tables" smell one level up.

It runs **before** read mode because it is the only half that can be
reversed: full screen keeps the ribbon, so the same control is still on
screen and a second click restores the window. Read mode removes the ribbon
and is therefore terminal. Doing them the other way round would leave the
display filled with a window that has no visible way back.

Its oracle is the **client area**, read from the window manager rather than
from the application: `WindowFrame::client_size` before and after. That is
the strongest available statement, because it is the windowing system
agreeing — the application can only *ask* for full screen
(`ViewportCommand::Fullscreen`), which is exactly why the dispatch arm
traces `fullscreen asked=` rather than `on=`. A check that believed the
trace would be reporting the request, not the result.

**It takes the display for about four seconds**, and then gives it back.
That is a real cost to whoever is at the machine and it is bounded
deliberately: the restoring click is made before anything else happens, and
a failure to restore is reported in words that say the display has been left
filled rather than being silent about it.

Measured on this machine: 2560 × 1000 → **3440 × 1440** → 2560 × 1000.
