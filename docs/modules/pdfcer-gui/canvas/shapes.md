# `canvas::shapes` — **the shape itself, following your hand**

The live geometry preview. `OPERATOR_REQUESTS.md` **O63**.

## What this replaces, and the convention it overrules

**Ken, 2026-08-30:** *"if I moved the end of a line, it didn't show me the
shape change of the line, it just had a perimeter box around it. this goes
for anything I change right now. there isn't a real preview like there is in
inkscape."*

He is right, and it was **deliberate**. `canvas/handledrag.rs` states the
rule this module exists to reverse:

> *"a preview shows the cursor, the render shows the document."*

That was a defensible position while the alternative looked like a second
rendering path. **It is overruled by operator ruling, by name, against a
named comparison** — Inkscape shows the line bend while you drag its end, and
so must this. Recorded as *reversed* rather than quietly contradicted,
because the sentence is repeated across several modules and the next session
would otherwise re-derive it and delete this file.

## Why this is cheap, when the rest of O63 is not

The measurement that framed O63 says a **rasterised** preview is impossible:
on the operator's CAD drawing a *two-pixel* render costs 691 ms, because ~99 %
of render cost is content-stream interpretation rather than fill. Anything
that goes through `pdfcer-render` is a second away.

**This does not go through `pdfcer-render`.** `vector::decompose_page` has
already produced the real geometry — `PathObject::page_subpaths()` gives
page-space `Line` and `Cubic` segments with control points resolved, plus the
paint style, the line width and both colours — and the shell already caches
it (`app::cache::page_objects`, keyed on `(page, edit_epoch)`).

⇒ Transform that in memory and hand it to egui's painter. No engine call, no
raster, no decomposition. **Pointer speed, and exact for geometry** — this is
not the "fuzzy" half of O63 at all.

## Rule 4, which this is on the right side of

A pre-commit affordance is *the cursor*, and the cursor is explicitly
permitted: snap indicators, rubber bands and selection handles are all
welcome. What is forbidden is styling **applied** content as though it were
provisional.

This draws a shape that has not been applied yet, in the selection stroke, and
it disappears the moment the real one is rendered. Nothing already in the
document is marked, tinted, badged or outlined because of it.

And it is **derived from the commit**, which is this canvas's standing
convention D2: the transform painted here is the *same* transform the release
hands to `EditSession`, so the operator cannot be shown one shape and given
another.

## What it deliberately does not draw

**Text, images and form XObjects.** A `PathObject` carries its own geometry;
a text run carries glyph provenance and an image carries a bounding box, and
neither can be drawn by this shell without becoming a second renderer. Those
keep the bounding outline they have today — which is honest, because a
rectangle *is* all the shell knows about where an image is going.

⇒ So the preview is **exact where it exists and absent where it does not**,
rather than approximate everywhere. A half-right glyph is worse than no glyph.

## Item notes

### `const MAX_OBJECTS`

# Why there is a cap at all, and why it is disclosed rather than silent

A marquee across a CAD sheet can select thousands of paths, each with
thousands of segments. Painting all of them every frame would turn the
gesture this feature exists to make smooth into the slowest thing in the
program — the exact inversion of the point.

Past the cap the preview is **absent**, and the bounding outline that was
there before this module existed is what the operator sees. That is a
graceful floor rather than a failure: it is what the shell did yesterday.

`canvas-shape-preview` traces `capped=1` when it fires, because an absence
with no account of itself is indistinguishable from a defect — the lesson
`painting.rs`'s anchor census already carries.

### `const MAX_SEGMENTS`

The second half of the same guard, and the one that actually fires on this
operator's drawings: `SW41177.pdf` carries a single object with **4,972
anchors**. One object is under [`MAX_OBJECTS`] and would still cost five
thousand line segments a frame.

### `fn average_scale`

`hypot` per axis rather than reading `a` and `d`, because a rotation puts
scale into `b` and `c` and reading the diagonal alone would report a rotated
object as having shrunk to zero width at 90°.

### `fn trace`

Written on **every** build, including the empty one. An absent preview and
a preview nobody asked for are different states and a trace that only spoke
when there was something to say could not tell them apart — the lesson
`painting.rs`'s anchor census carries, applied before it can bite here.

### `fn stroke_shape`

Shared by the erase pass and the preview pass so the two cannot disagree
about what the shape's outline *is*: an erase that traced a different path
from the preview would leave part of the original showing, and the part left
showing would look like the program had drawn it on purpose.

### `fn screen`

`measure::page_to_screen`, not arithmetic here. `coords`' standing rule is
that a coordinate is produced by exactly one conversion in exactly one place,
and the ce-dimension placement preview already goes through this door.

### `struct PreviewShape`

A copy rather than a borrow, deliberately: the provider is behind a `Ref`
that must be dropped before anything is painted (`painting.rs` says so at its
anchor draw), and a preview that borrowed it would hold that `Ref` across the
paint.

### `fn transformed`

The builder for move, resize and rotate — the three gestures whose whole
effect *is* a matrix, and which reach `EditSession::transform_objects` or
`move_objects` with exactly this transform.

# The matrix is the COMMIT's matrix, not a second one

Convention D2 on this canvas: the preview is derived from the value the
release will hand to the engine. `moving::action`, `resizing::action` and
`rotating`'s `TransformObjects` all build a page-space `Matrix`; this takes
that same value. Building a *parallel* transform here — "translate by the
canvas delta, converted again" — is how a preview and a commit come to
disagree by a rounding step, and the disagreement is invisible until an
operator lines something up against a guide.

# Returns

`None` when there is no decomposition this frame — a document still loading,
or a page that would not decompose. **Not an error**: the caller falls back
to the bounding outline, which is what it drew before this module existed.

### `fn with_nodes_moved`

The builder for the gesture the operator actually named: *"if I moved the end
of a line, it didn't show me the shape change of the line"*.

# The anchor indices are OBJECT-SCOPED, and the walk order is the
provider's, not a second one

`move_node` / `move_nodes` address an anchor by an index that counts across
the whole object, flattening its subpaths — the space
`ObjectModelProvider::object_node_points` reports and `pdfcer node-move`
speaks.

This walks the same order: `page_subpaths()`, then within a subpath the
`start` anchor followed by each segment's end, which is exactly
`Subpath::anchors()`. **That agreement is asserted by a test rather than
assumed** — see `the_walk_agrees_with_the_provider`. R74's rule is that a
matching rule must not be re-derived in the shell, and index arithmetic that
*must* match another module's is the same hazard wearing a smaller hat.

# Handles move with their anchor, and that is a choice

Displacing an on-curve anchor leaves its two control points where they were,
which is Inkscape's *"corner"* behaviour and produces a visibly different
curve from Inkscape's default *"smooth"* drag. **This preview does whatever
`EditSession::move_node` does**, and what it does is move the anchor alone —
so the preview is right about pdfcer even where it would be wrong about
Inkscape. Getting this backwards would produce the one failure this whole
module must not have: a preview that is prettier than the commit.

### `fn for_move_subject`

One function, so that the mapping from *the verb the release will call* to
*the shape the operator sees* lives in one readable table and a sixth rung
cannot be added without appearing in it.

It takes the [`MoveSubject`] the commit will use, not the selection.
That is convention D2 — *derived from commit* — enforced by the type rather
than by discipline: there is no way to reach this function without having
already computed what the release is going to do, so a preview cannot be
drawn for a gesture that would then refuse.

`dx`/`dy` are **PDF user-space** and Y is up, exactly as
[`crate::canvas::moving::PageDelta`] carries them.

### `fn erased`

The builder for a delete: an erase list and **no** shapes, so the preview is
pure subtraction.

# Why a delete needs a preview at all, when nothing is being drawn

Because the raster underneath does not know. The operator presses Delete,
the object is gone from the document — and it stays on screen for one to two
seconds on a dense drawing, because that is how long the page takes to
redraw. There is no gesture in flight to explain the wait, so what they see
is *a delete that did nothing*, and the natural response is to press Delete
again, which deletes something else.

⇒ This makes the object disappear at the moment it is deleted, which is what
every operator on earth expects and what the program was already doing to
the document. The picture simply catches up with it.

# It must be built BEFORE the commit

`app::cache::page_objects` is keyed on `(page, edit_epoch)` and the commit
bumps the epoch, so the geometry this needs is thrown away by the very edit
it describes. Called from the apply arm with the pre-edit model still in
hand; a caller that reached for it afterwards would find the objects gone and
silently hold nothing, which is the old behaviour wearing a new name.

### `struct StrokeRule`

# The report

**Ken, 2026-09-12:** *"The live preview blue outlines that appear when we
drag an object scale with zooming in and out of the page instead of being
independent of zoom - at high zoom levels they end up being the width of the
canvas. I think they keep the same size as the line widths they are moving
and that is ok - but if we set the line width view to the one pixel width
option the preview lines should also be affected by this setting."*

Two rulings in one paragraph, and they are not the same ruling:

1. the preview may take its width from **the line width of the object it is
   moving** - he says so explicitly, and it carries real information: a
   highlighter stroke and a hairline are different things and previewing
   both as the same thread would throw that away;
2. but that width is **in points, drawn as that many device pixels**, and
   zoom must not touch it.

# Why zoom-invariance is the correct answer and not merely the asked-for one

This module already argues the case, one paragraph up, for why a preview is
stroked and never filled:

> *"A filled shape following the pointer would hide what is under it, and
> what is under it is the page the operator is aligning against."*

**A stroke two hundred pixels wide IS a fill.** At 3,000 % a 6 pt
highlighter outline is 180 device pixels across, which hides precisely the
geometry the operator zoomed in to line up with - so the zoom-scaled preview
was defeating the reason the preview exists, by the module's own argument,
at exactly the zoom where alignment is the whole task.

⇒ The preview is **the cursor**. A cursor does not grow when the document
is magnified, any more than the pointer arrow does.

# Why the ERASE pass is deliberately NOT zoom-invariant

The erase pass and the preview pass look like the same drawing and are
statements about two different things:

| pass | what it is a statement about | space |
|---|---|---|
| preview | the cursor - *what you will get* | screen, zoom-invariant |
| erase | the **raster underneath**, which still holds the object where it started | screen, and the raster scaled with zoom, so this must too |

The erase has to cover ink that a renderer actually put on the texture. That
ink is `line_width x zoom` wide because that is what a zoom does to a
stroke. Shrinking the erase to a zoom-invariant width would leave the
original object showing down both sides of its own footprint, which reads as
a rendering artefact rather than as a preview - the one outcome
[`ShapePreview::erase`] exists to prevent.

So a single constant cannot serve both, and the two methods below are kept
apart on purpose rather than folded into one with a flag.

# `real_widths` - the second half of the ruling

`view.line_weights` (O137) is the operator's *"draw every stroke at one
device pixel"* view. When it is off, the renderer put one device pixel on
the texture for **every** stroke regardless of what the file said, so:

- the preview must be one pixel, because that is what the object looks like;
- and the erase must be sized for one pixel too, because that is what is
  actually on the raster it is covering. Sizing the erase from the file's
  line width under a hairline view would paint a white band far wider than
  the ink it is hiding.

Both follow from the same sentence: **draw the preview the way the thing
itself is being drawn.**

# Why this is a pair and not two parameters

A zoom and a display rule handed to a function as two loose arguments are
two things a caller can supply inconsistently - and this canvas has already
shipped one defect of exactly that shape, where the same measurement was
passed twice under two names. Bundling them means there is one place that
knows how a preview width is computed, and every caller gets that place.

### `fn preview_px`

Zoom does not appear in this function, and that absence is the fix.

The `max(1.0)` floor is older than O184 and survives it: a hairline
(`0 w`, PDF 32000-1 section 8.4.3.2) is one *device* pixel, and a zero-
width egui stroke vanishes under antialiasing. A preview nobody can see
is the same as no preview.

### `fn erase_px`

Zoom DOES appear here, because the ink being covered scaled with it.

The extra 1.5 px is not slack: an erase exactly as wide as the line
leaves a hairline of the original visible down both sides, because the
raster's antialiasing spread the ink half a pixel further than the
geometry says.

### `fn draw`

# Stroke only, never fill — and this is the one place the preview is
deliberately *less* than the truth

A filled shape following the pointer would hide what is under it, and what is
under it is the page the operator is aligning against. Every drawing program
that previews a transform previews it as an outline for exactly that reason.

⇒ So a filled path previews as its **boundary**. That is fuzzy in the
permitted sense — less than the truth, never different in meaning — and it is
the same compromise `backdrop.rs` documents for the low-resolution page.

# Why the selection colour and not the object's own

Because it is the *selection* moving. Painting a shape in its own stroke
colour would make the preview indistinguishable from committed content, and
there would then be two of it on screen — the stale raster still holds the
object where it was. One of the two has to be legible as "this is the one
following your hand".

### `fn paper`

Two callers, one value: an erased footprint is painted in it, and
[`crate::canvas::ocrlayer::draw_veil`] fades a raster towards it. Both are
covering ink with the ground it was composited onto, so a second constant
would be two answers to one question.

# Why this is a constant and not read from the document

PDF has no page-background colour. A page is whatever its content paints,
and the overwhelming majority of pages paint nothing at all outside their
ink — which a viewer composites over **white**, because that is what
`render_page`'s own backdrop is (§11.4.7's page group is composited onto an
opaque white backdrop when a document does not say otherwise).
