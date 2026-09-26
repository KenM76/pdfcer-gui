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
rendering path. ★★★ **It is overruled by operator ruling, by name, against a
named comparison** — Inkscape shows the line bend while you drag its end, and
so must this. Recorded as *reversed* rather than quietly contradicted,
because the sentence is repeated across several modules and the next session
would otherwise re-derive it and delete this file.

## ★★ Why this is cheap, when the rest of O63 is not

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

★ And it is **derived from the commit**, which is this canvas's standing
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
