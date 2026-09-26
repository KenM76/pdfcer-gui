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
