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
