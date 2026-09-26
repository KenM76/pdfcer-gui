# `ui-verify/checks/drop_onto_thumbnails`

`a_drawing_dropped_on_the_thumbnails_becomes_pages` — **drag a PDF from
Explorer onto the page grid and its sheets go in where you pointed.**

# The request


> *"I should be able to drag and drop documents into the thumbnails section
> of another pdf to import the pages."*

## The one part that cannot be driven, and everything that can

A harness moves a pointer and presses keys. It cannot **originate an OLE
drag** — that is Explorer's side of a protocol between two processes, and
no amount of `SendInput` produces one. So `app::filedrag` carries the same
kind of seam `app::dropped` already had (`PDFCER_DIAG_DROP_PATH`), which
makes the application behave as though a file had been dropped.

**But the seam alone would test the wrong thing.** This feature is
entirely about *where* the file landed, and a drop simulated at startup
lands nowhere in particular — so a check built on it would pass on a build
that ignored the position completely, which is precisely the build that
existed before O67.

⇒ Hence `PDFCER_DIAG_DROP_AFTER_MS`. The drop is held back; the check puts
the **real cursor** on the tile it means and waits; and the application then
reads the position from the operating system with the same line of code a
genuine drop uses (`native_window::cursor_position`). The position is real,
the geometry is real, the insert is real. Only the payload is synthetic, and
the payload is the one part that cannot be otherwise.

## Why the pointer is jiggled while waiting

`egui` repaints on events. An idle window with a file hovering over it
produces exactly one (`HoveredFile`, on entry) — which is why the
application requests a repaint while a hover is in flight — but the
*simulated* drop has no hover, so an idle application would never reach the
frame that fires it. Moving the cursor a point at a time keeps frames
coming without leaving the half of the tile the check is aiming at.

## The sequence

| # | step | oracle |
|---|---|---|
| A | open the Pages panel, count the pages | `canvas … pages=N` |
| B | park the pointer on the LEFT half of tile 1 | — |
| C | the held-back drop fires | `file-dropped … at=x,y` |
| D | the panel claims it | `pages-import-dropped files=1 pages=M gap=1` |
| E | and the document actually grew | `canvas … pages=N+M` |

Step D asserts the **gap**, not merely that an import happened. The left
half of tile 1 means *before page 2*, so `gap=1` is the answer that
distinguishes "the drop used the pointer" from "the drop appended at the
end", and appending is what every position-blind build would do.

Step E is the one that cannot be satisfied by wiring alone. A build that
raised the action and never reached the engine passes A–D and fails here.

## Item notes

### `const DROP_AFTER_MS`

Long enough for the mode click, the panel, and the pointer to be in place —
and it is a floor rather than a schedule, because the check then *waits for
the trace line* rather than assuming the drop has happened by now.

### `const PARK_ACROSS`

A quarter, so the LEFT half is unambiguous: the panel resolves the nearer
vertical edge, and a point near the middle is where a rounding difference
between the application's `f32` rectangle and this harness's reading could
flip the answer — `pages_drag`'s reasoning, mirrored.

### `const TILE_INDEX`

Not tile 0. Its left edge is gap 0, which is also what a build that
defaulted to `Start` would produce, and this check must not have a passing
answer that a position-blind build can reach.
