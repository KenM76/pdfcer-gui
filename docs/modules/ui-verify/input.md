# `ui-verify/input`

Drive the pointer and the keyboard **through the operating system**.

# Why OS-level injection, and not the two easier alternatives

This is the module's central design decision, and it is required by the
defect the harness exists to catch. Three ways to get input into an egui
application were available; two of them cannot see D1.

## Rejected: `PostMessage(WM_MOUSEMOVE / WM_LBUTTONDOWN)`

Tried first in this project's predecessor, and it **does not work for an
off-screen window** — with a silent failure, which is the worst kind. winit
calls `TrackMouseEvent` on the move; Windows answers `WM_MOUSELEAVE`
because the physical cursor is elsewhere; `egui-winit` then drops the
button entirely, because it emits `PointerButton` only when it knows the
pointer position. The observed event list was `[PointerMoved, PointerGone]`
in **every** message ordering tried, including move and button posted back
to back. That finding is recorded in `D:\dev\rag\egui\`, and it is recorded
here too so nobody rebuilds it.

## Rejected as the *primary* driver: in-process injection

The application already has one — `PDFCER_DIAG_SCRIPT` feeds steps through
eframe's `raw_input_hook`. It is excellent, it needs no screen, and it is
the right tool for a behavioural question on a machine the operator is
using. It is **the wrong oracle for D1**, and precisely because of what it
skips.

D1's causal chain is:

1. the canvas calls `request_focus()` when `Response::clicked()` fires;
2. `ctx.egui_wants_keyboard_input()` — which means *any widget has focus*,
   not *a text field has focus* — therefore returns `true` forever after;
3. so the unmodified-key bindings, `Delete` among them, are never
   installed.

Every link in that chain is about **what egui's focus machinery does with a
real click**. A harness that hands egui a synthetic `PointerButton` event
is asserting on the same layer that is broken. It might well reproduce the
bug — but a green result from it would not be evidence, because the thing
it skipped is the thing in question. The only way to be sure the click that
selects the object is the same click that focuses the canvas is to make the
window manager deliver it.

Put plainly: **the harness that must catch D1 has to go through the OS,
because D1 is a defect in how the application responds to the OS.**

## Chosen: `SetCursorPos` + `mouse_event` + `keybd_event`

System-level injection. The cursor really moves, the click lands on
whatever window is in front, and the keystroke goes to the foreground
window. Everything downstream — hit testing, focus, hover, capture — runs
exactly as it does for a person.

`mouse_event`/`keybd_event` rather than `SendInput`: for a button at the
current position they are equivalent, and their signatures have no
variable-length array to get wrong. The one thing `SendInput` would buy is
atomic multi-event batches, which is not wanted — a real click is not
atomic either.

## What that costs, and how it is paid

It commandeers the real desktop. Three mitigations, all mechanical:

* [`Driver::new`] records the pointer position and [`Driver`]'s [`Drop`]
  puts it back, on every path including a panic.
* [`Driver::click_at`] and [`Driver::press`] raise the target window first,
  and `press` refuses if there is no window — a keystroke sent to the wrong
  window is not a failed keystroke, it is a keystroke into the operator's
  editor.
* Checks are short, and each one holds the desktop for a couple of seconds
  rather than a couple of minutes.

The harness is honest about this rather than clever: it is a foreground
activity, it says so when it starts, and `--no-input` turns it off (whereupon
the checks that need it report SKIPPED, never PASS).

# The PowerShell fallback

[`PowerShellDriver`] does the same three operations by shelling out to
`Add-Type`'d `user32` P/Invokes. It exists for two reasons: it is what the
predecessor scripts used, so a finding reproduced there can be reproduced
here; and it keeps the harness usable if the `windows-sys` binding ever has
to be dropped. It is **not** the default — it costs a process per event,
which turns a three-event click into three process spawns and makes the
timing unlike a real click.

## Item notes

### `const CLICK_HOLD`

Long enough that the application sees a press and a release on different
frames, which is what a real click looks like. Zero-length clicks have been
observed to be coalesced by frameworks that sample input once per frame,
and a coalesced click is a click the application never saw.

### `const MOVE_SETTLE`

The application needs at least one frame to process the move and update its
hover state; several widgets only respond to a click when they were hovered
on the preceding frame.

### `const DOUBLE_CLICK_GAP`

`egui`'s own threshold is **300 ms between PRESSES**, and it is a
compiled-in constant rather than the operator's Windows double-click speed —
which is the thing a reader assumes and which would make this harness behave
differently on a machine where that setting had been changed.

With `CLICK_HOLD` at 60 ms, 40 ms here puts the presses 100 ms apart:
comfortably inside the threshold with room for two slow frames, and slow
enough that they land in **different frames**, which they must — `egui`
counts clicks, and two presses inside one frame are one press.

### `const DRAG_STEP_SETTLE`

Shorter than [`MOVE_SETTLE`]: the application does not need to settle at each
waypoint, it only needs to *observe* each one, which is one frame at 60 Hz.
25 ms is comfortably more than one frame on any machine that can render a
PDF page at all.

### `const CARRY_SETTLE`

Much longer than [`DRAG_STEP_SETTLE`] because the pointer of a carry crosses
two viewports before it becomes a drop offer: it is sensed in the float
window's pass, reported to the dock, and resolved into a zone by the *next*
frame in the main window. Three frames is the floor; this is an order of
magnitude over it, because the frame that has to run is one that rasterizes
a CAD sheet behind a compass.

### `const OBSERVE_DWELL`

Long enough that a frame is certain to have run with the pointer at rest
where the observer is about to photograph it, on a machine that is also
rasterizing a CAD sheet. It is not a threshold the application has to
beat - nothing in pdfcer waits on a timer to draw a pre-commit
affordance - so it buys frames, not a dwell.

### `const DWELL_NUDGE_TICKS`

A stationary pointer generates no input, and an application that repaints
only on input would never run the frame being photographed. Same argument
as [`Driver::drag_via`]'s dwell, and the same count.

### `fn walk`

Extracted from [`Self::drag`] when [`Self::drag_via`] needed the same
walk twice. The arithmetic is unchanged and the reason for it is
unchanged: integers, because the endpoints are whole pixels and an
intermediate point should be one a real mouse could produce.

### `fn window_owning`

# Why z-order cannot be the answer


1. the check aims at a control inside a dialog, correctly;
2. `click_at` raises **the main window**, because that is the target;
3. the dialog is not *owned* by the main window — `eframe 0.35` has no
   owner option at all — so it goes **behind** it;
4. the point is now on the application's own canvas, `window_at`
   agrees it belongs to the target, the cover guard is satisfied, and
   the click lands on a page.

Every step is individually correct and the result is a check reporting
a working feature as broken. So the window is chosen from the process's
windows by **whose client rectangle contains the point**, which no
raise can change.

Ties go to the SMALLEST window. A dialog is inside the application's
bounds on screen, so both contain the point; the dialog is the one in
front of the other in every arrangement an operator would produce, and
it is always the smaller. Stated rather than implied because the
alternative — first match — depends on enumeration order, which is
z-order, which is the thing this function exists to not use.

### `fn confirm_on_the_desktop`

`SetCursorPos` does not fail for a point beyond the desktop. It moves
the pointer to the nearest edge, returns success, and the click is
delivered **somewhere else** — to whatever control happens to sit at the
clamped position. From the application's side nothing is wrong: a click
arrived where the pointer was. From the check's side nothing is wrong
either: the primitive returned `Ok`. The only trace of the lie is a
verdict about a control the pointer never touched.

The recorded case is `checks::form_field`. It asked for a 1400 px-wide
window; [`crate::launch`]'s `SAFE_ORIGIN_X` places **every** launched
window at desktop x = 780 — a constant whose own doc does the arithmetic
for *"a 1100 px client"* ending at 1880 on a 1920-wide desktop — so
780 + 1400 = 2180 put 260 px of window, including most of the Properties
panel, past the right edge of the screen. The wheel aimed at the pane's
centre was clamped back onto the panel and scrolled it, which made the
step look healthy; the click aimed at a checkbox was clamped 6 points
above it, and the check reported for a week that ticking Required
*"reached nothing"* — an accusation against the application for a pixel
the harness could never deliver.

⇒ **A harness primitive that silently does something other than what it
was asked will eventually be believed.** The same sentence
`driving::arm_select_from_ribbon` earned for the `V` chord, and the same
remedy: refuse, and say the arithmetic out loud.

# Why this is not folded into the cover guard's own test

The first attempt was, and **it never fired.** `WindowFromPoint`
hit-tests window rectangles rather than monitors, so it returns the
target window quite happily for a point 150 px off the side of the
screen — the guard's `None` arm is for a point over the *desktop*, not
for a point over nothing. Measured on the falsification run for this
fix. Only [`sys::desktop_bounds`] knows where the screen stops.

# It does not care whether the point is on the window

A check aiming deliberately off-window — `off_page_marquee` is the
standing example — is entitled to a point no window owns, and gets it,
as long as it is on the desktop. What no check is entitled to is a
coordinate the operating system will quietly rewrite.

A zero-size desktop (the non-Windows stub) disables the guard, which is
correct: nothing drives a pointer there.

### `fn target_client_rect`

Used by [`Self::confirm_uncovered`] to tell *"covered"* from *"off the
window"*, which are different diagnoses with different remedies — see
the argument there.

### `fn raise_and_confirm_at`

Raises whichever of the application's windows actually contains the
point — see [`Self::window_owning`] — and falls back to the target when
the point is on no window of this process, which is the case a check
aiming off-window is entitled to and which the cover guard reports on
its own terms.

### `fn raise_and_confirm`

**`raise()` is a request, not a result.** `SetForegroundWindow` is
refused outright for a process without foreground rights — silently,
via a boolean return nobody is obliged to read — so a window that was
created behind an already-active one can stay behind it through any
number of raise calls. Windows' foreground lock exists precisely to
stop background processes stealing focus, and this harness IS a
background process.

Without this check the failure is not "the keystroke did not arrive".
It is:

* the keystroke arriving **in the operator's own window** — and for a
  chord that means running one of their commands, not typing a
  character; and
* the check reporting that the FEATURE is broken, when the truth is
  that nothing was ever typed at it. A false failure naming the wrong
  subsystem is worse than no check, because somebody then goes and
  looks at working code.

Both were observed: a `find_opens_and_finds` run reported "Ctrl+F did
not dispatch `edit.find`" against a build in which Ctrl+F works.

**Bring the target to the front and PROVE it got there**, or refuse.


> **A click sent to a window that is not in front goes to whatever
> window IS, and the check then reports the feature as broken.**

Windows refuses `SetForegroundWindow` to a process without foreground
rights, and this harness is a background process — so the raise is a
*request*, not a fact, and whether it is honoured depends on which
process last had focus and on how recently the operator typed. That
makes it **intermittent**, which is the worst available property: it
works while a suite is running and fails on a single check run alone,
or the reverse, and every failure it produces is a confident, specific
accusation against code that is fine.

Measured on 2026-08-20: `markup_rectangle_arms_from_the_ribbon` and
`insert_image_places_a_picture` both reported the ribbon as unresponsive
— *"the click on `ribbon.tab.markup` produced no `ribbon-tab-activated`"*
— over a build in which the ribbon works, and both had passed in a full
suite an hour earlier. The old build reproduced it too, which is what
ruled the application out.

The message this returns is deliberately long. Whoever meets it is one
step from diagnosing a feature that was never clicked.

### `fn application_has_the_foreground`

# Why a harness must not raise when the answer is yes

Because the application now opens windows *of its own accord*, and a
window it opened has the foreground without anybody having clicked it.
A text-annotation dialog appears in answer to a drag on the canvas and
takes the keyboard immediately — which is the behaviour under test, and
which the check verifies by typing WITHOUT clicking the field, *"the way
an operator does"*.

A `press` that raises the main window first destroys exactly that: the
dialog loses focus, the characters land on the page, and the check
reports that the dialog ignored the keyboard. The feature is fine; the
harness broke it and then measured it.

So the rule is **do not steal focus from the application** — only
reclaim it from something else. It is the same idea as the cover guard's
correction on the same day: the question is whose PROCESS owns what is
in front, not which handle.

### `fn confirm_uncovered`

`SetForegroundWindow` succeeding means the target has **focus**. It says
nothing about what is **drawn over it** — an always-on-top window sits
above a focused one and swallows every click aimed at the region it
covers. The click is delivered, to something else, and the check reports
the feature as broken.

The culprit here was `osk.exe`, the Windows on-screen keyboard, lying
across the ribbon's tab row. It is summoned by synthetic keystrokes, so
**this harness brings it on itself**, and it cannot be closed from a
process of ordinary integrity — `taskkill`, `CloseMainWindow` and
`ShowWindow(SW_HIDE)` were all refused by UIPI.

The symptoms were the worst available: intermittent, and confidently
wrong. `markup_rectangle_arms_from_the_ribbon` and
`insert_image_places_a_picture` reported the ribbon as unresponsive over
a build in which it works; both had passed in a full suite an hour
before; the pre-multi-document build reproduced it, which is what ruled
the application out. The oracle that settled it was a **screenshot** —
`D:/dev/rag/egui/` is unambiguous that a reachability defect has exactly
one — and the on-screen keyboard was plainly visible in it.

So: ask who owns the point, and if it is not the target, say so and
stop. An error becomes a SKIP, which is *"this did not run"*, rather
than a FAIL, which is an accusation.

It asks [`Self::confirm_on_the_desktop`] first, because a coordinate
that is not on the screen at all cannot meaningfully be *covered* — and
because the two failures have completely different remedies.
