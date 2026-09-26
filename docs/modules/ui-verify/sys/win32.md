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
