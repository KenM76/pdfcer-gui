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

### `const DRAG_STEPS`

Enough that the application sees the pointer *travel* rather than teleport —
see that method's docs on why a two-point drag can be delivered as a click.
Not more, because each step costs [`DRAG_STEP_SETTLE`] and a check that holds
the operator's desktop is one that should finish.

Readable outside this module because the *size* of one step decides whether
egui calls the press a drag on the first step or on a later one, and so
decides how much of the travel a check may predict will arrive. A check that
predicts the whole travel is asserting something about this number.

### `fn click_at`

The window is raised first: a click on a window that is not in front is
consumed by the click-to-focus of whatever *is*, and the application
under test sees nothing. That failure looks identical to a hit test
returning nothing.

### `fn right_click_at`

## The first driver for a gesture class this project has shipped
## since Phase 1

pdfcer has had canvas context menus for months and **not one driven check
has ever opened one**. Everything asserted about them is a unit test over
`MenuHost::would_open`, which asks whether the *manifest* would offer
something — a real question, and not the same question as *"does a right
-click on this pixel open a menu"*.

⇒ R1's own words: *"the tests pass" is not a report of working
software*. A gesture with no driver is a gesture R1 cannot reach, and
the gap left no failing test behind to advertise itself.

## What this deliberately does NOT do

It does not click a menu **item**. An `egui` popup is positioned by the
pointer and sized by its content, so a harness that aimed at "the second
row" would be encoding a layout, and would silently start clicking the
wrong verb the day a menu grows an entry. The oracle is the application's
own `canvas-menu context=…` trace line, which says which menu it
resolved and how many items it offered — the fact under test, without a
coordinate to go stale.

Escape is **not** pressed afterwards, deliberately: leaving the popup
open is what lets a following screenshot show it. A check that wants it
closed presses Escape itself, and says why.

### `fn drag`

# Why the harness had none until now

It did not need one. Every check that drives the ribbon uses
[`Self::click_at`], and even `markup_rectangle` — whose *subject* is a
drag gesture — asserts on the ribbon arming rather than on the band,
because a markup band is checkable from the command trace. Canvas **text
selection** is the first feature whose entire behaviour is a drag: there
is no button to press that produces one, and a click alone can only ever
clear a selection.

# The intermediate moves are the whole reason this is not three calls

egui decides that a press has become a *drag* rather than a *click* by
distance travelled, and it samples the pointer once per frame. A press
followed immediately by a release at a distant point is delivered as a
single jump: egui sees one position, then another, and may report a
**click** at the far end rather than a drag at all — which for this
feature is the difference between selecting a paragraph and clearing the
selection.

So the pointer is walked in [`DRAG_STEPS`] increments with a settle
between each, which is what a hand does. The application gets several
frames of `dragged_by(Primary)` with a moving position, which is
precisely the sequence `canvas::gesture::GestureState` is written for.

The button is held across the walk rather than being pressed at each
step: a released-and-pressed pointer is *n* gestures, not one.

### `fn drag_with_modifier`

The band gesture's combining arms are modifier-selected — plain
replaces, Shift adds, Ctrl subtracts — and the application samples the
modifier on the frame it handles the release. A harness that pressed the
key only at the last instant would pass against a build whose caption
and band colour never followed the key, so it is held across the press,
the walk and the release, which is what a hand does.

# Errors

As [`Self::drag`].
The foreground is established BEFORE the key goes down, and that is not
redundant with the raise [`Self::drag_unmodified`] does: `SendInput`
delivers to whatever window is in front at the instant it is called, so
a key-down issued while another window still holds the foreground is a
modifier pressed into the operator's own application — and released
there too, leaving a key stuck down in a window this harness never
touched.

### `fn drag_via`

The gesture a **spring-loaded** target needs: press here, walk to
there, *stay* long enough for the application's dwell timer to fire,
then walk on and release. Windows Explorer's folders, every browser's
tabs and pdfcer's document tab strip all work this way, and none of them
can be driven by [`Self::drag`] — which walks straight through and never
rests anywhere.

`dwell` is how long the pointer sits on `via`. It must exceed the
application's own threshold with room to spare: the check that uses this
passes twice `crate::pdfcer::SPRING_DWELL`, because a dwell measured
against a *frame clock* on a machine that is also rasterizing a CAD
sheet is not a dwell measured against a stopwatch.

The pointer is **moved slightly** during the dwell rather than being
held perfectly still, in the same place, for a second. A stationary
pointer generates no input, and an application that only repaints on
input would never run the frame its own timer fires on. pdfcer asks for a
repaint while a spring is armed precisely so this is not required — but
a harness that depended on that would be testing the repaint request
rather than the spring, and would report a false failure the day the
request moved.

# Errors

As [`Self::drag`].
`modifier` is held down for the **whole** gesture, press to release.

Which is more than the application strictly needs — pdfcer samples the
drag modifier at the *release*, as Windows does — and it is deliberately
more. Holding it throughout is what an operator's hand actually does,
and it also exercises the frames in between, where the caption has to
follow the key. A harness that pressed the key only at the last instant
would pass against a build whose caption never updated.

### `fn carry`

# It is [`Self::drag`] with two deliberate differences

**1. It raises the window the gesture *begins* in, not the main one.**
[`Self::drag`] opens with [`Self::raise_and_confirm`], which raises the
application's main window. A carry starts on a float window's header
strip, and raising the main window over it would put the press
underneath whatever it just covered. [`Self::raise_and_confirm_at`]
raises whichever of the process's windows owns the point.

Both endpoints still go through [`Self::confirm_uncovered`] unchanged:
it accepts *any* same-process window, so a `from` inside the float and a
`to` inside the main window both pass, while an unrelated application
lying across either one still fails the check rather than the feature.

**2. It rests on `to` before releasing.** The dock does not read this
pointer where it lands. A float is drawn in a child viewport whose pass
runs after the main window's, so the pointer is sensed in the child,
reported through `DockState::set_float_drag`, and consumed by the *next*
frame's `Dock::show` — which is the frame that resolves a zone and
publishes the offer. Releasing on the frame the pointer arrives releases
before any offer exists, and lands nothing at all.

The rest is [`Self::drag_via`]'s dwell trick and for the same reason: a
stationary pointer generates no input, so the settle is a run of
one-pixel nudges rather than one sleep. A harness that held perfectly
still would be relying on the application asking for repaints it is not
obliged to ask for.

One pixel is safe against a zone boundary because every caller aims at
a zone *centre*; a verb that aimed at an edge would have to say so.

# Errors

As [`Self::drag`], plus the raise failing for the window owning `from`.

### `fn double_click_at`

# Why the gap is a named constant and not a guess

`egui` decides a double click from the interval between two presses, and
its threshold is a fixed 300 ms — it does **not** read the operator's
Windows double-click speed, which is the thing a reader assumes. A
harness that clicked twice as fast as it could would be relying on
scheduler luck; one that used the OS setting would break on a machine
where the operator has slowed it down. So the gap is chosen against the
framework's own number, with room for a slow frame.

# Errors

As [`Self::click_at`].

### `fn click_with_modifier`

# Why this is not `press_chord` plus `click_at`

Because the modifier has to be held **across** the mouse press, and
`press_chord` releases it as part of sending a keystroke. The
application reads `modifiers.shift` on the frame it processes the
pointer event, so a Shift that went down and up before the click is a
plain click — which is the failure mode that would make this check
report "the second anchor was not picked" over a perfectly working
build.

# Errors

As [`Self::click_at`], and additionally refuses with no target window:
a modifier held over the operator's own desktop is a stuck key.

### `fn press_held`

# Why this exists beside [`Self::press_chord`], which looks identical

Because they are not identical and the difference is a whole class of
silent failure. `press_chord` posts the modifier down, sleeps, posts the
key, and releases the modifier — **per press**. This holds the modifier
across all of them, so the application sees one modifier transition and
N key presses inside it.

The reason it was written: `shift_arrows_select_text` sent
`press_chord(&[SHIFT], ARROW_RIGHT)` three times and the application
traced `Modifiers::NONE` on all three. Every arrow arrived; not one of
them carried Shift. The pointer path had never had that problem, and it
uses `with_modifiers` — the modifier held across the whole gesture —
which is what this is.

The finding is about the toolkit, not about this harness: modifier
state reaches `egui` through winit's `ModifiersChanged`, and a modifier
that goes down and up again inside one frame's event batch can be
applied and undone before the key that was supposed to carry it is
dispatched. Holding it removes the race instead of tuning it — the same
answer this project reached about the fit-zoom loop and about
`ViewportCommand` lag, and for the same reason: **an intermittent is a
defect with a timing dependency, and a sleep is not a fix.**

# Errors

If the target window cannot be brought to the front, for the reason
[`Self::press_chord`] refuses.

### `fn move_to`

Guarded by [`Self::confirm_on_the_desktop`], which is the ONE place
the silent clamp can be caught for every gesture at once: this is the
call every wheel, hover and drag makes, and `SetCursorPos` rewrites an
off-screen coordinate rather than refusing it. `click_at` has the same
guard on its own path because it does not come through here.

### `fn scroll_at`

Moves the pointer there first, because a wheel event goes to whatever is
under the cursor — scrolling "the panel" means putting the pointer in it.

# Why a check needs this, and what its absence looked like

A dock panel is a few hundred points tall and a real document's content
is not, so a check that can only reach what is on screen at launch can
only verify the top of any list. Worse, it reports everything below the
fold as *"the control is drawn and inert"* — which is a **confident,
specific, wrong defect report about a control that works**, and this
harness produced three of those in one day before this existed.

# Errors

If the pointer cannot be moved.

### `fn scroll_at_held`

# Why this is not `scroll_at` with a flag

It shares `with_modifiers`' whole-frame lead-in, and that lead-in is
load-bearing for the same reason [`Self::press_held`]'s is: a modifier
posted less than a frame before the event it is meant to carry arrives
in the same batch but *after* it, so egui builds the event with
`Modifiers::NONE`. A real hand holds Ctrl for tens of frames first. A
Ctrl+wheel that loses its Ctrl is an ordinary scroll — the view pans
instead of zooming, and the check reports the zoom as broken.

# Why a check wants this rather than the status bar's `+`

Zoom-to-cursor keeps the point under the pointer fixed, so a check can
put the pointer on the content it cares about **once** and keep
rolling — the content stays under it all the way down. The `+` button
zooms about the viewport centre, which on a page whose interesting
detail is off-centre magnifies blank paper. The operator's own words,
2026-08-22: *"Right now you are just zooming into a blank area on the
canvas."*

### `fn scroll_burst`

Unlike [`Self::scroll_at_held`] nothing waits after the last notch: the
caller photographs what the program shows while it is still catching
up, which is the point.

# Errors
As [`Self::scroll_at_held`].

### `fn press`

# Errors

If there is no target window. Refusing is the whole point: keystrokes
go to the foreground window, and if the harness does not know which
window that should be, the keystroke lands in whatever the operator was
typing in. There is no safe default here, so there is no default.

### `fn type_ascii`

# Why this refuses rather than skipping what it cannot type

It handles lowercase letters, uppercase letters (with Shift) and digits,
and returns an error for anything else. The alternative — silently
dropping a character it has no virtual-key code for — would type
`userp` where the caller asked for `userpw`, and the check would then
report the *application* as rejecting a correct password. A harness that
mistypes and blames the program is the worst failure available to it,
and this project has recorded several.

It is **real keystrokes through the OS**, not a seeded buffer. That is
the whole point: a password field is a focused `TextEdit` behind a real
viewport, and a check that wrote the string into memory would be
asserting about a program nobody can operate.

# Errors

If a character has no mapping, or a keystroke cannot be delivered.

### `fn press_chord`

The reason this exists: `Ctrl+F` and every other letter chord in the
manifest keymap were unreachable from this harness, so the checks that
would have driven them could not be written. `press` sends a bare
virtual key with no modifiers, and a shell that binds a command to
`Ctrl+F` cannot be reached by sending `F`.

# Errors

If there is no target window, for exactly the reason [`Self::press`]
refuses — and more sharply. A bare keystroke into the operator's editor
types a character. **A chord into the operator's editor runs a
command**, and `Ctrl+W`, `Ctrl+Q` and `Ctrl+S` are all one letter away
from a chord a UI test might plausibly send.

Modifiers are released by [`sys::key_stroke_with`] on every path; see
its docs for why that is not merely tidy.

### `enum Key`

A modifier key this harness can hold across a mouse gesture.

An enum rather than a bare `u16` virtual-key code, because the whole point
of this type is that a caller cannot accidentally hold something that is not
a modifier — a mouse gesture with `A` held is not a gesture any application
defines, and it would arrive as a stray keystroke.
