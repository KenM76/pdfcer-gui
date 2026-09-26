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

## Item notes

### `const FULLSCREEN_EVENT`

`asked`, not `on`: a viewport command is queued and answered by the
windowing backend, so the application cannot honestly claim the window *is*
full screen on the frame it requested it. Phase 0 therefore reads this line
only as evidence that the **arm ran**, and asks the window manager for the
result.

### `const TOGGLE_EVENT`

Quoted verbatim into the restore failure, because it is the one line that
separates the two defects that produce an unrestored window: a shell that
asked for full screen twice (`asked=true` on both) and a window manager that
declined the restore (`asked=false` and the window still filled).

### `const PRESS_TRIES`

# Why a retry and not a longer settle


A click that is not delivered is not delivered no matter how long the
harness then waits, so the answer is to press again and read the
application's own invocation log, not to sleep longer. Three, because the
cost of a wasted press here is a fraction of a second and the cost of
giving up too early is the operator's whole display staying filled.

It is safe to retry precisely BECAUSE the count is read between attempts:
each landed press toggles once, so pressing again after a press that landed
would undo it. [`press_until_invoked`] returns the moment the count moves.

### `const CANVAS`

`app::mod`'s `REGION_CENTRAL_PANEL`. It is the outermost region the
application owns, and its **top edge** is the measurement this check turns
on: with the ribbon drawn, the central panel starts below it; with the
ribbon gone, it starts at the top of the client area.

### `const MIN_RISE_PTS`

A floor rather than an equality. The exact height of a two-row ribbon band
is a layout fact this harness must not re-derive — `check-file-size.sh`'s
sibling argument, and the reason `profile::PDFCER_GUI` ships no region
fractions at all: a number written here is correct until the first time the
band's padding changes, and then it is a check that fails for a reason that
is not a defect.

40 pt is comfortably under one row of controls (the theme's `control_height`
is 24 pt before the band's own padding and its caption row) and comfortably
over any rounding, a scrollbar, or a one-pixel splitter. What it is really
asserting is *"a whole bar's worth"*, not a specific bar.

### `const MIN_REPAINT_DELTA`

[`driving::MIN_PRESSED_DELTA`]'s derivation applies unchanged and is not
restated: two identically filled regions in a lossless BGRA capture differ
by **0**, not by a small number, so anything above the noise floor is a real
change. Ribbon band against canvas backdrop is a far larger difference than
the pressed/unpressed pair that constant was measured on, so borrowing it is
conservative in the direction that matters.

### `const VIEWPORT`

Wide enough for **all seven** of the View tab's groups, because the control
this check clicks is in the last one — see the note at the launch site for
why that is a finding and not a workaround. `0,0` rather than a negative
off-desktop origin: the harness has to photograph this window, and a window
placed off the visible desktop is captured as whatever the compositor last
had for it.

### `fn fullscreen_round_trip`

Returns `Ok(None)` for a clean round trip, `Ok(Some(_))` for a failure
sentence, and `Err` only for the states that are the harness's business —
no control declared, no click delivered, no window frame readable.

# Why the client area and not the trace

`app::window::toggle_fullscreen` sends `ViewportCommand::Fullscreen`, which
is a **request**. The application cannot know whether it was granted, which
is why its trace line says `asked=` — so a check that read the trace alone
would report the request and call it the result. `WindowFrame::client_size`
comes from `GetClientRect` on the real window: it is the windowing system's
answer, and it is the only one worth having here.

# Why it restores before returning, even on the failing paths that can

A check that filled the operator's display and then reported a failure would
leave them to find the window and fix it. Every path below that has already
pressed the control presses it again; the one that cannot — a second click
that does not land — says so in its own sentence rather than being silent
about a display it has taken.

### `fn press_until_invoked`

Returns `true` as soon as a new `ribbon-command-invoked id=view.fullscreen`
line appears in the shell trace, and `false` after [`PRESS_TRIES`] attempts
with no new line. It never presses again after a press that landed, so the
toggle is moved exactly once whatever happens.

# Why phase 0 cannot use a bare `click_at`, and the day that cost

`Driver::click_at` answers `Ok(())` when the **pointer input was sent**. It
raises the owning window, refuses if the foreground could not be taken, and
confirms the point is not covered — all real guards, and none of them is the
statement *"the application processed a click on that control"*. Between the
two lies a window-manager transition, an egui frame boundary, and a ribbon
that may have re-laid itself out.


> *"a second press that does not restore means the state it reads is not the
> state the OS is in"*

naming `app::window::next_fullscreen`. The trace carried **one**
`ribbon-command-invoked id=view.fullscreen` line and **one**
`fullscreen-toggle reported=Some(false) pending=None asked=true` for the
whole run: there was no second press to read a stale state with.
`next_fullscreen` had itself been written to fix a real instance of exactly
that defect a fortnight earlier, so the report was a plausible accusation
against the code that already closed it.

⇒ The rule this encodes, which is `checks/mod.rs` rule 3 applied to input
rather than to selection:

> **Nothing measured after a press is evidence about the program until the
> press is shown to have arrived.** The application's own invocation log
> says so; `Ok(())` from the input layer does not.

Re-reading the control's rectangle each attempt is not defensive padding.
The window changes size between the two presses, and a ribbon is laid out
from the window's width — `ui-rect` is a change log, so an unmoved control
simply publishes nothing and [`declared`] answers with the rect that still
stands. Reading it fresh costs one trace parse and follows the control if it
ever does move.

### `fn click_tab`

`markup_shapes::click_tab`'s shape, with its own tab and its own reason.
Not folded into [`driving`] in this change: that module's header records
that it is *"a widening, not a refactor"*, and a third copy is the point at
which folding becomes worth doing on its own rather than in the change that
happens to need it.

### `fn the_region_names_match_the_ids_they_describe`

Spelled as literals in this file and as `format!`s in the shell, so this
is the seam where a rename in `egui_shell::ribbon::report` would
otherwise turn every assertion above into a silent SKIP — *"the
application declared no region"* — which reads as a missing control
rather than as a renamed one.

### `fn the_rise_threshold_is_a_floor_rather_than_a_band_height`

Pinned so that a future edit which "tightens" it to an exact band height
has to argue with this comment first: an equality here would fail the
day the band's padding changes, for a reason that is not a defect.

Written against a runtime copy rather than the constant itself, because
`clippy::assertions_on_constants` refuses an assertion the compiler can
fold — correctly, in general, and this is the case where the assertion's
value is the **sentence attached to it** rather than the arithmetic.
