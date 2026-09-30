# `pointerscripted` — clicks and drags without the OS mouse

`PDFCER_DIAG_POINTER=<file>` lets a harness drive the pointer of a window
placed off the desktop, so driven checks can run while the operator is using
the machine (O260). The seam is honoured only when the diagnostic trace is
on, and it is registered at runtime, so no `cfg` is involved and R8 does not
come into it.

## Why the input goes in through a plugin

The events are injected into `RawInput` through `egui::Plugin::input_hook`.
They then pass through egui's own hit-testing, click counting and drag
threshold, the same as a real press. Two other routes were considered and
rejected:

- **`eframe::App::raw_input_hook`** is never called for immediate
  viewports, and every dialog here is one of those.
- **`ctx.input_mut`**, the route `keyscripted` uses, runs after
  `begin_pass`. By then `PointerState` and hit-testing have already been
  built from `RawInput`, so a click pushed there is invisible to
  `clicked()`.

## Frames

egui resolves a press against the previous frame's widget rects, so each
verb spreads its events across frames:

- **Click:** a move, then the press, then the release.
- **Double-click:** the move and five frames in all.
- **Drag:** a move, the press, `steps` moves, the release.
- **Wheel:** a move, then the wheel.
- **Key:** `key NAME [mods=ctrl+shift] [vp=V]` — the press, then the release,
  carrying the modifiers. `NAME` is egui's key name (`A`, `Enter`, `Tab`).
- **Type:** `type [vp=V] TEXT` — one frame with the rest of the line as text,
  spaces included. It goes to whichever widget has focus, so click the field
  first.

- **Shot:** one empty frame, then egui's own screenshot (below).

Within one step the input clock advances at most 1/60 s a frame. egui counts
a double-click by the time between releases and a click by how long the
button was held, so a slow frame would otherwise turn a scripted double into
two singles. Each capped time still exceeds the last, and the next step's
frames take the real clock again, so time stays monotonic.

A discarded pass calls `begin_pass` again within the same frame. Steps are
therefore keyed on `(viewport, cumulative_frame_nr_for)`, so a step is never
handed out twice.

## Sync

The application follows the file. The root viewport polls it every 50 ms
and queues only complete lines. It answers each step with
`diag-pointer seq= verb= vp= frames=` once the step's last frame has gone
out, or with `diag-pointer-refused seq= line=` for a line it cannot read.
All waiting is done on the harness side, so the application stays a
doorbell.

An idle application draws no frames, so the seam requests its own:
`request_repaint_after(50 ms)` for the root while the queue is empty, and
`request_repaint` while a step is pending. These are made in
`Plugin::on_end_pass`, inside the pass. eframe ignores the output's
`repaint_delay`. A request made from `input_hook` is also lost: it comes
before the pass, and the pass's own begin clears it. The application then
stops drawing, and every step times out unacknowledged.

`diag-pointer-armed path=` is written once, when the plugin is added.

## Screenshots

`shot` is the only oracle for pixels of a window placed off the desktop: an
OS capture there sees whatever is on screen at those coordinates. The seam
sends `ViewportCommand::Screenshot` from `on_end_pass` (a command issued
before `begin_pass` is discarded by it), finds the `Event::Screenshot` in a
later frame's input, and writes it as a binary PPM (`P6`, maxval 255, no
comments) at `<step file>.shot-<seq>.ppm`. The step is acknowledged only
then, with `path= w= h=` added, or `error=` if the write failed.
`ScriptedPointer::screenshot` reads it back and writes a PNG.

Only the root viewport answers. `shot vp=<dialog>` is never acknowledged:
the dialog windows are immediate viewports, and no `Event::Screenshot` for
them reaches the seam. A check about a dialog asserts on its trace events,
not on a picture of it.

## Coordinates

Points are in egui logical points of the target viewport, the same space
`ui-rect` lines use. `vp=` is either `root` or the `viewport=` token of a
`ui-rect` line, with or without its quotes. A dialog is its own viewport,
so its controls are clicked with its token and its own coordinates.

## What it cannot test

Anything decided below egui is out of reach:

- OS activation and focus (`viewport().focused` stays false off-screen);
- pointer capture;
- OS drag-and-drop;
- the focus-chain regressions.

Those checks keep OS input.
