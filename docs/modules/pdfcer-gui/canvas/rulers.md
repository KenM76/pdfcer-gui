# `canvas::rulers` — the ruler gutters, the tick ladder, and the drawing grid

`RIBBON_IA.md` §5.2's View ▸ Display row *"Rulers · Grid · Guides"*, and
`FEATURES.md`'s last unbuilt Phase 3 line. This module owns two of the
three; [`super::guides`] owns the third and is built on the arithmetic
here.

---

## ★ 1. What unit the ruler reads in — the question, and the answer

`RIBBON_IA.md` says *"rulers along the canvas edges, **in the document's
units**"*, and that phrase hides a real decision rather than a formatting
detail. pdfcer is a CAD-drawing tool: `pdfcer-core` carries a whole
dimensioning subsystem in which a **group** owns a scale, a display unit
and a number format, and in which the displayed value of a length is
*derived* — `value = measured_points × scale`.

The rule this module implements, in one sentence:

> **The ruler reads in exactly the same units, at exactly the same scale,
> and through exactly the same formatter as a dimension placed on the same
> sheet would.**

Concretely, three states:

- `ScaleState::NeverSet`, no scale ever set → **PDF points**: `100.00 pt`,
  `200.00 pt` …
- `ScaleState::OneToOne`, explicitly 1:1 → the group's unit at true size,
  so a 72 pt span reads `25.40 mm`.
- `ScaleState::Calibrated` → the group's unit at the operator's scale, so
  on a sheet drawn 1:50 in metres a 72 pt span reads `1.270 m`.

### Why points is the *default*, and why that is not an arbitrary pick

Because it is the only unit the document itself states. Every other number
this application shows about geometry — the `canvas-pointer` trace's
`page=` reading, the Properties panel's extents, an annotation `/Rect` — is
in PDF user-space units, and a ruler that invented millimetres while every
neighbouring readout said points would be a second measuring system with no
way to tell which one a number came from. Points is what the file says;
anything else is an interpretation, and an interpretation needs the
operator to have supplied it.

### Why a set scale changes the ruler, and why the numbers come from *core*

Because an operator who has calibrated the sheet has said what the drawing
*means*, and a ruler that ignored that would be measuring the paper while
the drawing is about a building. The two readings must also **agree to the
digit**: someone who runs a linear dimension along a wall and then checks
it against the ruler is entitled to the same number, in the same unit, with
the same decimal marker and the same number of places.

The only way to guarantee that is to call the same function, so [`Scale`]
calls [`pdfcer_core::dimension::format_measurement`] — the function that
renders every dimension label and every live measurement readout — and this
module never formats a length itself. That has a second, quieter payoff:
**the ruler contributes no operator-visible string to `crate::text` at
all.** Its labels are core's spelling of a measurement, unit abbreviation
and ISO comma included. The dimensioning subsystem's own rule is
*"disclosures rendered by core, never invented at the GUI layer"*; a ruler
tick is a measurement rendered by core, for the same reason.

### Where the scale is read from, and the honest limit of that today

[`Scale::of`] reads [`pdfcer_core::edit::EditSession::dimension_model`] and
takes the **default group** (`DEFAULT_GROUP_ID`), which `pdfcer-core`
guarantees always exists. That is the document's own
`/PieceInfo /pdfcer /Private` sidecar, so a sheet calibrated in a previous
session — by this application once the scale surface lands, or by
`pdfcer` today — is honoured on open with nothing else wired.

**The limit, stated rather than left to be discovered:** this build has no
way to *set* a scale. `measure.set_scale` is registered and has no dispatch
arm, and `EditSession`'s scale verbs have no caller in the GUI. So today
the ruler reads points on every document that has not been through the CLI,
which is every test document in `D:\Dev\temp\pdfcer`. That is why this is
written as *reading a document property* rather than as *reading a pdfcer
setting*: when the scale surface lands it needs no change here, and until
it does the ruler is not pretending to a calibration nobody supplied.

**There is no cache, and the number is measured rather than assumed.**
`dimension_model()` walks catalog → `/PieceInfo` → `/pdfcer` → `/Private`
and deserialises; on a document with no sidecar — every ordinary PDF — it
stops at the first `None` after four dictionary lookups and one catalog
clone. **Measured on `ncored-benchmark-cad-drawing.pdf` (A3, 129,758
objects): 3.65 µs per call, called twice a frame** — once by the rulers and
once by the grid. The same sheet's *raster* is 1,244 ms
(`render-async-done ms=1244`), and a 60 Hz frame is 16,667 µs.

Caching it would need a key on `edit_epoch`, a `RefCell` on `OpenDoc` and a
staleness question, bought for 7 µs against a frame that has just uploaded
a page texture; the same argument [`crate::viewer::strip`]'s header makes
about not caching the strip.

## Measured cost of the whole overlay

On the same sheet, at its fit zoom of 1.3634 in a 2,200 × 1,300 window:

| | |
|---|---|
| ruler ticks | 223, of which 22 are labelled |
| grid lines | 205 |
| tick walk + label formatting | **8.9 µs per frame** |
| `Scale::of`, twice | **7.3 µs per frame** |
| **total** | **≈16 µs, or 0.1 % of a 60 Hz frame** |

Most of the 8.9 µs is the 22 `format_measurement` calls, each of which
allocates a `String`. That is the thing to attack if this ever needs to be
faster, and it does not: the page it is drawn over costs four orders of
magnitude more.

---

## ★ 2. Which space the grid is drawn in — page space, per page

Under a continuous mode several pages are on screen at once, so "where is
the grid" has two candidate answers and only one survives contact with a
drafter.

**A viewport-space grid** is drawn once, over the whole scrolling area, and
is anchored to the window. Scroll, and it slides across the paper: a line
that sat on an intersection comes off it, and the same feature on two
sheets falls at two different places in the grid. It is wallpaper, not a
reference. It is also cheaper and easier to write, which is why it is the
one to be careful about.

**A page-space grid** is drawn per page, anchored to that page's own
top-left corner, and clipped to that page's rectangle. It scrolls *with*
the sheet, so an intersection is a fixed place on the drawing; every sheet
in a set gets the same grid in the same place relative to its own border;
and the gaps between pages carry no grid, which is truthful — there is no
paper there to be ruled.

pdfcer draws the second, and the reason is what a grid is *for*. A drafter
uses one to judge alignment and spacing **on the drawing**, and every
answer read off it is a statement about the sheet. A grid not attached to
the sheet cannot make such a statement. The same argument settles the
guides, which is why [`super::guides`] stores a guide against a **page**.

**Every numbered ruler tick has a grid line under it**, because both come
from the same 1-2-5 [`Ladder`] — which is the whole reason to ship a ruler
and a grid rather than two independent ornaments: a feature sitting on a
grid line can be read off the ruler without counting. ⚠ Not the stronger
"every *heavy* grid line is numbered", which is **false**: both steps are
1-2-5 numbers and a 1-2-5 number is not always divisible by a smaller one
(500 over 200 is 2.5). Asserted by `canvas::grid`'s
`every_ruler_label_has_a_grid_line_under_it`.

The two ladders are resolved by **different** constructors, and which one
is which is load-bearing: [`Ladder::for_labels`] bounds the *labelled*
step, because that is what must not overlap; [`Ladder::for_lines`] bounds
the *drawn* step, because every grid line is drawn. ⚠ Using the first for
the grid puts a line every 1.4 screen pixels on the benchmark sheet — a
tint rather than a grid — and neither a screenshot nor the suite catches
it.

---

## ★ 3. Rulers reserve layout space, and the reservation is a CONSTANT

The gutters sit along the canvas edges, so switching them on shrinks the
viewport the strip is laid out into — and the viewport is what
[`crate::viewer::ViewState::apply_fit`] divides by, **on every frame a fit
mode is active**. That is rule **R128** (`app::status`'s header carries the
measured case): *a panel whose size feeds a fit-to-viewport computation has
a fixed size.* A content-driven gutter — one that grew to fit its widest
label — would be a measured feedback loop: a wider label on frame N is a
smaller fit scale on frame N+1 is a different label on frame N+2, which
presents as a page visibly shrinking frame after frame.

So [`THICKNESS_PTS`] is a constant, it is the *only* thing [`reserve`]
subtracts, and nothing here measures a string before deciding how much room
to take. Labels are laid out **inside** a gutter whose size was already
settled, and one that does not fit is clipped — the same posture the dock
takes with a panel body, for the same reason.

Switching rulers on therefore costs exactly one re-fit, once, which is what
the operator asked for.
[`tests::the_gutters_are_a_constant_bite_out_of_the_viewport`] asserts the
constancy rather than trusting this paragraph.

---

## ★ 4. Rule 4 — why chrome the operator switched on is allowed

`panels`' one-line test: *would a screenshot of the editing canvas differ
from a screenshot of the same document saved and reopened?* For a ruler and
a grid the answer is **yes, and the operator is the reason**. Rule 4
forbids pdfcer drawing *its own inferences* onto the page — a badge saying
"this text was OCRed", a dashed outline meaning "this bound is
approximate". Nothing here is keyed on any property of the content: the
grid's spacing comes from the zoom and the document's stated scale, never
from what is on the sheet; the ruler's numbers come from the page geometry,
never from an analysis of it; and both vanish the instant the operator
switches them off, which is the second half of what makes them chrome
rather than marking.

The version that would fail the test is a grid that **snapped to something
pdfcer found** — a detected drawing frame, an inferred module size. There is
no such code here and there must not be: that is an inference, and an
inference owes an off-canvas report.

---

## What is in this file

- [`THICKNESS_PTS`], [`reserve`], [`Gutters`] — the constant bite out of
  the viewport, and the child `Ui` the canvas is drawn into.
- [`CanvasGeometry`] — what the frame learned about where its pages are,
  handed back so the gutters can be drawn against it.
- [`Scale`] — what unit the ruler reads in, read from the document.
- [`Ladder`], [`nice_step`] — the 1-2-5 tick ladder, chosen in display
  units and returned in points.
- [`draw`] — the gutters: ticks, labels, the page's own edges, the pointer.

The grid is [`super::grid`], drawn one per visible page in that page's own
space and off the [`Ladder`] here.
