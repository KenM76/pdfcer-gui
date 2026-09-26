# `ui-verify/sys/win32`

The Windows implementation of the platform API.

Everything here is a thin, documented wrapper over one Win32 call. The
interesting decisions are recorded at the function that embodies them; the
cross-cutting ones are here.

## Why `GetDC(NULL)` + `BitBlt` and not `PrintWindow`

[`capture_screen`] photographs the **composited desktop**, not the window's
own device context. The window's DC is the obvious choice and it is wrong
for this application: eframe renders through glow/wgpu, and a
GPU-composited surface frequently comes back **blank** from a
`PrintWindow`/`BitBlt` of the window DC. A blank capture is the worst
possible failure here, because it is indistinguishable from a real one at
the call site — the file exists, the call succeeded, and only a human
looking at the PNG can tell it is not evidence. This project's predecessor
recorded exactly that, twice, and recorded a plausible-but-invented cause
being attached to it before the real one was found.

The consequence of reading the desktop is that whatever is *in front of*
the window is what gets photographed. Hence [`raise_window`], and hence the
near-uniformity guard in [`crate::pixels::region_not_uniform`], which is
the mechanical version of "a human looked at it".

## Why the window search is by process id

Not by title, and not by class. Titles change with the open document; class
names are winit's business and not a contract. The process id is the one
thing the harness knows for certain, because it launched the process. It
also guarantees the harness can never drive a window belonging to an
instance the operator opened for their own work — a hazard the predecessor
scripts hit hard enough to write four paragraphs about.

## Item notes

### `const CHORD_GAP`

`modifiers` are virtual-key codes (`VK_CONTROL` 0x11, `VK_SHIFT` 0x10,
`VK_MENU` 0x12) held down for the duration of the stroke.

# Every modifier is released, on every path, in reverse order

A modifier left down is not a failed keystroke — it is a **stuck key on
the operator's real keyboard**, applied to whatever they do next, until
they happen to press and release that key themselves. Ctrl left down turns
their next `s` into a save and their next `w` into a close-window. This
harness already refuses to type when the target window is not in front for
the same class of reason; leaking a modifier is the same hazard with a
longer tail, because the wrong window is at least visible and a stuck Ctrl
is not.

So the releases are unconditional and there is no early return between the
press and the release. Reverse order because that is what a human hand
does, and because a shell watching for a chord may key on the release
sequence.
# The pauses are load-bearing, and their absence is why chords silently
did nothing

Without them this function posts four or more `keybd_event` calls in the
same microsecond, and **no chord the harness sends reaches the
application** — while a plain [`key_stroke`] of the very same key works.
That asymmetry reads as *"synthetic keyboard input does not reach the
target window"* and is not: the truth is narrower and is about **ordering,
not delivery**.

`keybd_event` posts into the system input queue **asynchronously**. The
target reads that queue on its own schedule — for an `egui`/`winit`
application, once per frame. A modifier-down and a key-down that arrive in
the same batch give the application no frame in which the modifier is held
and the key is not yet pressed, so its notion of "current modifiers" at the
moment it processes the key can still be empty. The key is then delivered
**unmodified**: `Ctrl+2` arrives as a bare `2`, which this shell's keymap
binds to nothing, so nothing is traced and the check reports silence.

That is also exactly why a plain keystroke was unaffected and why the
earlier investigation (which confirmed foreground rights, then tried a
prior click) found nothing: neither had anything to do with it.

12 ms is one frame at 60 Hz plus margin — enough that the application sees
at least one frame with the modifier down and the key not yet pressed, and
small enough that a chord still completes in well under a tenth of a
second.

### `const CF_UNICODETEXT`

Deliberately not `CF_TEXT`: that is code-page-dependent, and a check that
compared a round-tripped ANSI string would fail or pass depending on the
machine's locale rather than on the application's behaviour.

### `const OPEN_ATTEMPTS`

The clipboard is a **global, singly-owned** resource: `OpenClipboard`
fails outright while any other process holds it, and on a live desktop
something always might — a clipboard manager, an editor polling for
changes, the shell itself. A single attempt would make this check flake in a
way indistinguishable from the defect it exists to catch, which is the worst
possible failure mode for a harness.

### `fn with_clipboard`

The release is unconditional — including on the `body` panicking, which is
why `body`'s result is captured rather than returned directly through the
`?`. A harness that left the clipboard open would wedge every other program
on the operator's desktop, and it is his desktop.

### `struct WindowHandle`

Wrapped rather than passed as a raw `HWND` so the rest of the crate never
has a pointer in its types, and so the `unsupported` build can offer the
same shape.

### `fn windows_for_pid`

The application has one window per open dialog, not one window overall. A
harness that knows only the first cannot raise the one it is aiming at — it
raises the main window instead, which puts the dialog BEHIND it, and the
click lands on the application hundreds of pixels from the control it named.

Order is `EnumWindows`' own, which is **z-order, front to back**. Callers
that want a specific window must identify it by geometry rather than by
position in this list: z-order is what the raise is about to change.

### `fn window_frame`

The **client** area, not the window rect. The two differ by the title bar
and the border, and every logical coordinate the application traces is
relative to the client origin. Using the window rect would put a constant
offset — around 30 px vertically, and DPI-dependent — into every single
conversion, which is small enough to still hit *something* and therefore
exactly the kind of error that gets diagnosed as a hit-test bug.

### `fn raise_window`

Best-effort by Windows' own rules — a process without foreground rights may
be refused, so the boolean result is deliberately not turned into an error.
The consequence of a refusal is a screenshot of whatever is in front, which
is why the uniformity guard exists downstream rather than here.

pdfcer's predecessor script added this after a capture returned a
pixel-perfect screenshot of a completely different application: the target
had started, run its whole script and traced correctly, but its window was
created behind an already-maximised window, so the capture photographed
whoever owned those pixels.

### `fn maximize_window`

# Why a harness needs this, and what it stops being a false failure

A ribbon **overflows** when it is wider than its window: groups past the fold
move into an overflow menu, and their controls stop publishing a rect. That
is correct application behaviour and it is indistinguishable, from a check's
point of view, from *"the control does not exist"*.

It was found the way these things are always found: `settings_theme` clicked
the File tab, asked for `ribbon.item.file.settings`, and was told the tab
published ten controls of which that was not one — because at the default
window size the File tab's last two groups were in the overflow. The check
would have reported a shipped feature as missing.

Maximising is the right fix rather than *"open the overflow menu"* for two
reasons. A menu's contents are not published as regions, so a check could
not aim at them anyway; and a maximised window is the state an operator
running a drawing tool on a desktop is overwhelmingly in, so it is also the
state most worth verifying.

What it costs is real and belongs in the record: a check that maximises is
**not** testing the narrow-window layout. Whether every control survives a
small window is a separate question and needs its own check.

### `fn cursor_position`

Read before a run so it can be put back afterwards. Driving the real cursor
is the cost of testing the real input path (see [`crate::input`]); moving
the operator's pointer and *leaving* it moved is not part of that bargain.

### `fn set_cursor_position`

# Why it tries twice

`SetCursorPos` can fail at a coordinate that is demonstrably on screen and
inside the target window, and succeed at that same coordinate a moment
later. The cause is a **transient**: for a few milliseconds after a process
holding a pointer capture dies — the previous session’s window, killed by
the harness itself — the platform declines to move the cursor at all.

The cost of not retrying is a check reporting SKIP — *"unable to
begin"* — for a reason that has nothing to do with the application. This
harness exists to turn "told you nothing" into something, and a suite whose
members randomly do not run is that same failure wearing another colour.

**One retry, not a loop.** A genuinely bad coordinate — off every
monitor, which is an arithmetic error in the calling check — must still
fail, and fail quickly, with the message that names it. A loop would turn a
check’s own mistake into a slow timeout.

### `fn mouse_button`

`mouse_event` rather than `SendInput`: for a plain button at the current
position they are equivalent, and `mouse_event`'s signature has no
variable-length array to get wrong. The one thing `SendInput` would buy —
atomic multi-event batches — is not wanted here, because a real user's
click is not atomic either and the point of this harness is to exercise the
real path.

### `fn mouse_button_secondary`

Without it no check can open a context menu at all, and the canvas menus
(`canvas.object`, `canvas.empty`) are then covered only by unit tests over
`MenuHost::would_open` — a question about the manifest rather than about
the running program.

**A missing capability in a harness leaves no failing test behind.** A
whole gesture class can therefore sit outside R1's reach with nothing
saying so, which is why these primitives are worth auditing against the
gestures the application actually offers.

### `fn wheel`

`notches` is in wheel detents — positive scrolls **up** (away from the
operator), negative down, which is the sign convention `WM_MOUSEWHEEL`
itself uses.

# Why the harness needs this at all

Because a dock panel is a few hundred points tall and a real document's
content is not. A check that can only click what is on screen at launch can
only ever verify the top of every list — and it reports everything below the
fold as *"the control is drawn and inert"*, which is a **confident, wrong
defect report about a control that works**.

`mouse_event` rather than `SendInput` for the same reason the button press
uses it: no variable-length array to get the size of, and at the current
pointer position the two are equivalent. `WHEEL_DELTA` is 120, the constant
the API defines one detent as.

### `fn key_stroke`

Goes to the **foreground window**, whichever that is — which is why callers
raise the target first and why the input driver refuses to type when the
foreground window is not the one under test. A keystroke sent to the wrong
window is not a failed keystroke; it is a keystroke into the operator's
editor.

Two doc blocks separated by a blank line concatenate onto whatever item
follows, so a block written for one function silently documents the next.
`tools/gates/check-orphan-docs.py` is what refuses that.

### `fn window_at`

`SetForegroundWindow` succeeding says the target has focus. It says
**nothing about what is drawn over it**, and an always-on-top window — the
Windows on-screen keyboard is the one that bit us — sits above a focused
window and swallows every click aimed at the region it covers.

The failure that produces is the worst shape available: checks reporting
the ribbon as unresponsive, intermittently, over a build in which it works.
The oracle that settles it is a screenshot (`D:/dev/rag/egui/` — *a layout
or reachability defect has exactly one oracle*), which shows the covering
window — `osk.exe` is the usual one — lying across the region the click was
aimed at.

So a click now asks who owns the point first, and refuses rather than
missing. `WindowFromPoint` returns the deepest child; the ancestor walk is
what makes the answer comparable to a top-level handle.

### `fn desktop_bounds`

The *virtual* screen — every monitor as one rectangle — because that is the
space `SetCursorPos` accepts and the space a multi-monitor operator's
windows live in. On a single 1920 × 1080 display it is `(0, 0, 1920, 1080)`;
with a second monitor to the left it starts at a negative `x`, which is why
the origin is returned rather than assumed to be zero.

# Why a harness needs this at all

Because **`SetCursorPos` clamps.** Asked for a coordinate beyond the
desktop it moves the pointer to the nearest edge and reports success, so a
click aimed off the screen is still delivered — somewhere else, to whatever
control is at the clamped position. Nothing in the API says the ask was not
honoured, and nothing in the trace can, because from the application's side
a click arrived exactly where the pointer was.

Measured: a check asking for a 1400 px window, [`crate::launch`]'s
`SAFE_ORIGIN_X` placing it at desktop x = 780 on a 1920 px screen, and the
click aimed at a control in the right-hand panel landing six points above
it — reported for a week as the application failing to record an edit. See
`Driver::confirm_uncovered`, which is the guard this feeds.

# `WindowFromPoint` cannot answer this question, and that was the first
attempt

It hit-tests **window rectangles, not monitors**, so it returns the target
window quite happily for a point 150 px past the edge of the screen. A guard
written on *"no window owns this pixel"* therefore never fires for the case
it was written for — measured, on the falsification run for this very fix.
The desktop rectangle is the only thing that knows where the screen stops.

### `fn move_window`

A harness that lets Windows choose gets a *cascade*: every launch steps down
and right from the last, so a long session marches its windows towards the
edge of the desktop and eventually off it. A known position also makes a
failure reproducible, which a cascading one is not.

### `fn resize_window`

Required by `OPERATOR_REQUESTS.md` **O55**, whose whole subject is *"if the
canvas window is resized the pdf should resize to match"*. Without it a
harness can move a window and not resize one, and a fit's behaviour across
a resize is outside R1's reach entirely.

The pattern is worth naming: a harness grows a primitive when a feature
needs it, so the primitives it has are a map of the features somebody
already had to prove — and the ones it lacks are where nothing has been
proved at all.

`SWP_NOMOVE | SWP_NOZORDER` keeps the position and the stacking exactly as
they were; the size is in **physical pixels**, which is what
`SetWindowPos` takes and what `describe_window`'s client rect reports.

### `fn is_foreground`

# Why this is asked rather than assumed

[`raise_window`] is **best-effort by Windows' own rules**: a process
without foreground rights is refused, and the refusal is silent — it
returns a boolean nobody was obliged to read. So "we called
`SetForegroundWindow`" is not "the window is in front", and the gap between
those two is where a keystroke lands in the operator's editor.

This is the function that makes [`key_stroke`]'s promise true — that the
input driver refuses to type when the foreground window is not the one
under test. A driver that checks only that a target *exists* has stated an
intent, not enforced it.

### `fn foreground_window`

Distinct from [`is_foreground`] in the way that matters: that answers
*"is THIS window in front"*, and the question a harness needs once the
application has several windows is *"which of them is"*. A dialog that just
opened has the foreground and was never clicked, so no record of a click can
answer it.

### `fn describe_foreground`

# Why a refused raise must name the window that refused it

`SetForegroundWindow` fails for exactly one reported reason — *"this
process does not have foreground rights"* — and that sentence is true of
two completely different situations which need opposite responses:

| what is really happening | what to do |
|---|---|
| the harness is a background process and Windows' foreground lock is doing its job | nothing; retry, or run the check when the desktop is free |
| **another window is holding the foreground and will not yield it** | dismiss that window — no amount of retrying will help |

The second one is expensive to diagnose without this. Measured once at
forty minutes: nine driven checks reporting SKIP with the foreground-rights
sentence, three raise strategies probed against a running build, and the
harness itself under suspicion — where the cause was a stray
**`OpenWith.exe` "Open With" dialog** holding the foreground the way a
system modal does and yielding it to nothing. One `taskkill` fixed all
nine.

The diagnosis is a `GetForegroundWindow` followed by `GetClassNameW` — two
calls the harness can make itself, at the moment of failure, when it
already knows something is wrong. **A check that reports a refusal
without naming the refuser has withheld the only fact that distinguishes
"wait" from "act".** That is the same shape as the `osk.exe` finding
recorded against [`window_at`]: an unrelated always-on-top window silently
eating the harness's input, diagnosed only by looking at what was actually
on the screen. This function makes the harness look, so a human does not
have to.

Returns something printable in every case, including no foreground window
at all — which is itself a distinct and diagnosable state (a locked
workstation, or a switch to the secure desktop).

### `fn describe_window`

# The rule, stated once and applied to both guards

[`describe_foreground`] carries it: *a check that reports a refusal without
naming the refuser has withheld the only fact that distinguishes "wait"
from "act".* The **cover** guard (`Driver::confirm_uncovered`) refuses for
the same class of reason, so it names its subject with this function rather
than guessing at one. A refusal reading *"the point (1627, 895) belongs to
another window"* followed by a guess at which window is unactionable
whenever the guess is wrong.

**When a guard learns to name its subject, check every other guard that
refuses for the same kind of reason.** A lesson applied at one call site and
not at its sibling is a lesson half-learned, and the sibling is where it will
be paid for again.

### `fn capture_screen`

Returns `region.w * region.h * 4` bytes. See the module docs for why this
reads the desktop rather than the window.

The GDI object dance is the standard one and every handle is released on
every path, including the error paths: a harness that leaks a DC per run
will exhaust the desktop heap during a long CI session, and the failure
looks like an unrelated rendering bug in whatever runs next.

### `fn with_modifiers`

# Why this takes a closure instead of exposing down/up

Because a leaked modifier is a stuck key on the operator's real keyboard,
and this harness runs on the operator's real desktop. `key_stroke_with`'s own
comment makes the point about early returns; a pair of public `modifier_down`
/ `modifier_up` functions would move the obligation to every caller, and the
first caller to add a `?` between them would leave Shift held down system
wide until the operator noticed their typing had gone into capitals.

A closure makes the release structural. `body` may panic and the modifiers
still come up, because the loop below is after the call in a function that
does not unwind past it — and every caller in this harness returns `Result`
rather than panicking anyway.

### `fn clipboard_text`

`None` covers three genuinely different situations and the caller must treat
them as one, because Windows does not distinguish them either: the clipboard
is empty, it holds something that is not text (a bitmap, a file list), or
another process would not let go of it. All three mean *"the operator would
not get text if they pasted"*, which is the question a check is asking.

### `fn clear_clipboard`

**A check that asserts on the clipboard MUST call this first**, and the
reason is the whole shape of defect O18. The failing build left a marker
sentence on the clipboard; a check that copied, read, and found that marker
would fail correctly — but a check that copied *nothing at all* and found a
marker left by an earlier run would fail identically, and one that found
yesterday's correct text would **pass while the application did nothing**.

Clearing first is what makes the read afterwards a statement about this run.

### `fn clipboard_formats`

The oracle `checks::copy_as_vector` needs and the one
[`clipboard_text`] cannot supply. `OPERATOR_REQUESTS.md` O120's whole design
is an *order*: a pasting application "typically retrieves … the first format
it recognizes", so what makes a Word paste an editable graphic rather than a
flat picture is not *whether* the SVG is there — it is whether the SVG is
there **first**. A check that asked "is `image/svg+xml` available?" would
pass on a build that placed it last, which is the build that fails in Word.

`EnumClipboardFormats` answers exactly that question: it walks the formats
*in the order they were placed*, which is also the priority order a reader
sees. Anything Windows synthesised (it makes `CF_DIB` and `CF_BITMAP` out of
a `CF_DIBV5`) comes after the ones that were really placed, so a caller can
assert on a prefix and ignore the tail.

Each entry is `(id, name)`. The name comes from
`GetClipboardFormatNameW` for a registered format and is empty for a
predefined `CF_*` one — Windows has no name for those — so a caller matches
predefined formats by id and registered ones by name, which is exactly how
they are placed.

`None` means the clipboard could not be opened at all, which is a different
fact from "the clipboard is empty" and must not be collapsed into it: the
first is a flake, the second is a defect.
