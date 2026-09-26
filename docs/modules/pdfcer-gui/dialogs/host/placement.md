# `pdfcer-gui/dialogs/host/placement`

`dialogs::host::placement` — WHERE a dialog's window opens.

# Why this is its own file


It is a `mod placement;` declared inside `host.rs`, so it lives at
`dialogs/host/placement.rs` and needs no entry in `dialogs/mod.rs` — which
is at 1,496 lines and has no room for one. R2 is satisfied by splitting on a
real seam rather than by shaving comments off the file that grew.


An outside review found the sticky-note dialog *"opens at the window
origin"*. It did, and the cause was two lines in two files that each looked
finished:

- `dialogs/textannot.rs` computed a considered opening position — centred
  across the application window, a third of the way down — and then wrote
  `let _ = pos;`, with a comment saying the computation *"is retired with the
  `egui::Window` it fed"*;
- `dialogs/host.rs` placed every dialog with no remembered position at a flat
  [`OPEN_INSET_PT`] from the application window's top-left corner, because
  that was the only placement any caller had ever asked for.

Neither half is wrong on its own. Together they are a dialog that discards a
position it computed and opens in the corner instead — **every single
time**, because a note dialog is opened and dismissed dozens of times in a
markup session and therefore almost never has a remembered position to
restore. The half-second of hunting for it is small; it is paid on every
note.

★ The review's wording was *"a click-relative position"*, and that is not
what the discarded computation was. It is centred horizontally and a third
of the way down the application window — the same placement the Set-scale
dialog uses — and it is deliberately **not** over the annotation: the
dialog's own header records that *"an operator writing a callout is usually
looking at the thing they are calling out, and a window pinned over it would
make them close the window to read what they were annotating."* A genuinely
click-relative dialog would be a regression against that stance, and would
additionally need the canvas's page-to-screen transform, which this half of
the program does not own. What was restored is the position that was
computed, not a different one.

# ★★ The rule this file holds

> **A chosen opening position is clamped onto the application window, and
> nothing about the dialog's own content participates in the arithmetic.**

Both halves matter.

The clamp is what makes a caller-supplied position safe to honour at all. A
caller computes in the application window's own coordinates, which say
nothing about how big the desktop is or where on it the application sits; a
position that is sensible in that space can still put half a dialog past the
right-hand edge of the monitor. Clamping onto the **parent window** is the
monitor-agnostic way to say "on screen": the application window is on a
monitor, so anything inside it is too, and neither `ViewportInfo::monitor_size`
(a size with no origin, useless on a second monitor) nor a work-area query
(which `eframe 0.35` does not expose) is needed.

The second half is R128's rule, the one this project has been bitten by
three times — see `print/layout.rs`'s header and `Host::fit`'s. Every number
below comes from the **parent window's rectangle**, the dialog's **declared**
size and **constants**. Nothing is measured from a laid-out body, so no
quantity here can be its own cause.
