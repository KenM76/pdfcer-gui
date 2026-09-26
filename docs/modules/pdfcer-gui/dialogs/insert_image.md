# `dialogs::insert_image` — putting a picture on the page

## The gap this closes

`edit.insert_image` was registered, drawn on Edit ▸ Insert, carried a `P3`
mark in `shell::commands::reach`'s `SCAFFOLDED` list — and its recorded
reason was, verbatim, **"No recorded reason for the missing arm."** One of
only three entries in that list of which that was true.

`EditSession::add_image` shipped long before this window: an image XObject,
an optional `/SMask`, a `q…cm…Do…Q` overlay stream and the page patches, as
**one undo entry**, additive, with the original bytes left verbatim.

## Why placement is numeric here, and not a drag on the canvas

Every other editor lets you drag a box, and the standing tie-breaker in this
project is *"make it work the way other programs do"*. This one asks for a
rectangle in millimetres instead, and that is a decision with three reasons
rather than a shortcut:

1. **It is the better gesture for this document.** The operator's sheets are
   CAD drawings. A picture going into one is a logo in a title block or a
   site photograph in a detail box — both of which have a *position on the
   sheet* that somebody decided, and neither of which is served by a
   freehand drag to "about there".
2. **A placed image cannot be moved afterwards.** It is page **content**,
   not an annotation, so it carries no `/Rect` and the move-and-resize
   surface `FEATURES.md` plans reaches annotations only. A one-shot
   freehand placement with no correction but undo is a worse offer than a
   box you can type into.
3. **It can be verified.** A drag needs a new `CanvasTool` variant, a
   `DragKind`, and an arm in `canvas::gesture::press_kind` — machinery this
   project's own RAG warns is the least safe thing here to change without
   driving the binary, and the harness needs the operator's machine. R1 says
   a phase is not done until it is driven; shipping a gesture that cannot be
   is shipping the thing R1 exists to stop.

**A drag-to-place gesture is a second ROUTE to the same action**, not a
replacement for this window, and it is the natural next slice: `Action::
InsertImage` already carries everything it would produce.

## What this window previews, and the one thing it does not

It previews **where the picture lands**, from `NewImage::placed_rect()` —
the engine's own function, public for exactly this, whose doc says
*"re-deriving the arithmetic in the GUI is how a preview and a result drift
apart."* Nothing here computes a rectangle.

It previews **the resolution** too, as of 2026-08-19 — filed that morning
and shipped the same day. `NewImage::effective_dpi()` and
`below_screen_resolution()` are pure, and the half that mattered is that
`add_image` now **calls** them rather than repeating the formula, so the
preview and the outcome cannot disagree.

The four-line version this window nearly computed would have been wrong.
Under `ImageFit::Contain` the placed rectangle is the *letterboxed*
sub-rectangle, not the box the operator typed, so measuring `rect` reports a
resolution low by exactly the letterbox ratio. The pure sibling was not
saving four lines; it was saving the letterbox.

## Why the import happens BEFORE the window opens

So a file that cannot be placed is refused at the moment it is chosen,
naming the file's own problem — *"pdfcer does not place GIF images"*, *"this
image uses {feature}, which pdfcer cannot place"* — rather than opening a
window full of controls over a picture that was never going to go in.

It also means the window can state the picture's real facts: its format, its
pixel dimensions **as displayed** (an EXIF-rotated photograph is transposed
by the importer), and whether the resolution it reports is one the file
declared or one pdfcer assumed.

## Item notes

### `const MIN_MM`

One millimetre. Below that the picture is not a picture on any sheet this
application is for, and a zero-area box is refused separately with its own
sentence — `no_area` — because *"give it a size"* and *"that is too small"*
are different instructions.

### `fn spec`

The builder rather than a struct literal — `NewImage` is
`#[non_exhaustive]`, so a downstream crate cannot construct it
field-by-field, and the constructor is what keeps a field added upstream
from silently defaulting here.

**One function, three readers**: the landing preview, the resolution
preview, and the trace the harness cross-checks against the outcome. The
apply arm builds it the same way, which is what makes `placed_rect()`
here and the rectangle written there the same answer rather than two —
and it is the same argument [`rect_pt`] makes about the millimetre
conversion, one layer up.

### `fn refusal`

**Refused rather than clamped**, and the refusal names the problem.
A box silently moved back onto the sheet is a placement the operator did
not make, and they would discover it by looking at the drawing rather
than at this window — which is `Tolerance::validate`'s rule applied one
feature along: *"a corrected value the operator never saw is exactly the
sneaky case."*

### `fn rect_pt`

# Free rather than a method, and `#[non_exhaustive]` is what forced it

`ImportedImage` is `#[non_exhaustive]`, so **this crate cannot construct
one** — which means it cannot construct an [`InsertImageDialog`] either, and
a method on it could not be tested without a real decoded picture on disk.

The constraint pushed toward the better shape, which is the part worth
recording. These two functions are the whole of this window's arithmetic,
they are pure, and they are the same shape
`crate::text::measure::two_line_reading` was pushed into for the same reason
one feature along. A rule stated as a function is a rule that can be
asserted; a rule stated as a method on an unconstructible type is a rule
nobody checks.

It is also the ONE conversion in this window. The validity check, the
landing preview and the action all read it, because three separate
conversions is how a window comes to promise one rectangle and produce
another.

### `fn refusal`

**Refused rather than clamped**, and the refusal names the problem. A box
silently moved back onto the sheet is a placement the operator did not make,
and they would discover it by looking at the drawing rather than at this
window — `Tolerance::validate`'s rule applied one feature along: *"a
corrected value the operator never saw is exactly the sneaky case."*

**An overhang is NOT refused.** Bleeding a picture past the crop box is a
real thing to do deliberately, and refusing it would make this window
stricter than the format — the class of helpfulness that makes an operator
fight their tool. Only a box **wholly** off the sheet is declined, because
that one cannot be anything but a mistake.

### `fn spinner`

One tenth of a millimetre per drag step: a logo in a title block is
positioned to the millimetre and a photograph is not positioned at all, so
finer would be motion nobody uses and coarser would make the common case
need typing.

### `fn a_millimetre_is_the_definition`

A hand-rounded `2.8346` would be wrong in the sixth decimal, and a
picture placed at 210 mm would land 0.0004 mm off A4's edge — invisible,
permanent, and different from every other number in this application.
`dialogs::new_document` makes the same point about `594.0 * 72/25.4`.

The constant this once asserted was a private `PTS_PER_MM` in this
file — the third of six copies. The argument above is why the
replacement is [`crate::units`] and not a fourteenth spelling: of every
surface in this program, this dialogue is the one whose numbers go
STRAIGHT INTO `pdfcer-core` as a rectangle, so it is the one that most
needs the engine's own value rather than its own.

### `fn off_the_sheet_is_refused_and_an_overhang_is_not`

The second half is the decision worth pinning. Bleeding a picture past
the crop box is a real thing to do deliberately, and refusing it would
make this window stricter than the format — the class of helpfulness
that makes an operator fight their tool.

### `fn a_sizeless_box_gets_its_own_refusal`

Different from off-the-page because the instruction is different — *give
it a size* rather than *move it back* — and one message covering both
would tell half the operators the wrong thing to do.

### `const REGION_PLACE`

Published on every frame the dialog draws, which is every frame it is NOT
hiding for a placement — so its absence from a trace means the window has
stepped aside, which is exactly the state a driven check needs to observe.

### `fn open`

# The box is seeded at the picture's NATURAL size, centred

Natural size is what the file asks for — its pixels at the resolution it
declares, or one pixel per point when it declares none — so an operator
who presses Insert immediately gets the placement the picture was made
for. Seeding at some fraction of the page would be pdfcer choosing a
scale nobody asked for, and the operator would have no way to tell that
from the picture's own size.

It is **clamped to the sheet** on both axes, because a 300-pixel-wide
logo at 72 dpi is bigger than an A4 page and a window that opened
refusing its own default would be a window that looks broken. The clamp
preserves the aspect ratio, so a clamped default is still the picture's
shape.

### `fn take_place_request`

One function, read by the validity check, the landing preview and the
action. Three separate conversions is how a window comes to promise one
rectangle and produce another — the same argument
`dialogs::new_document::sheet_pt` makes for its own single derivation.
Drain the operator's request to point at the page — O66.

### `fn place`

Through the INVERSE of [`rect_pt`], not through a second conversion.
That function's own doc comment exists to keep one arithmetic for
millimetres and points; a placement that converted separately would be
the second, and the two would drift by a rounding rule nobody chose.

A **degenerate** rect — what a click produces — writes the corner and
leaves the size alone. The dialog already has a width and a height, typed
or defaulted from the picture's own aspect, and a click is a statement
about *where*, not about *how big*. Overwriting the size with zero would
throw away the one thing the operator did not ask to change.

### `fn open_for`

Applies the two guards every dialog in [`super`] applies at the one place it
is built. The no-document guard is real here rather than ceremonial: the
window's box is seeded from a page's extent, and a window over an empty
canvas would open on a zero-sized sheet and refuse its own default.
