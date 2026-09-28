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
