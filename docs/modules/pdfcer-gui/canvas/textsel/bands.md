# `canvas::textsel::bands` — from glyph cells to the boxes a selection shows

The accumulation half of `textsel`'s §5 promise: **what is highlighted is
what is copied**. [`super::resolve`] walks a range once and, for each glyph
it covers, decides which band the glyph belongs to and grows that band by the
glyph's cell. This module is the two types that make "grow" mean the right
thing in each of the two frames the shell now has to work in.

## Why two frames exist at all

`pdfcer-core` publishes a glyph's advance as a **length** and never publishes
its **direction** — see [`super::writing`], which recovers the direction from
the glyphs themselves, and the operator report that made it necessary. So a
page can carry lines running along x, which the engine groups correctly, and
lines running at 90° to it, which the engine splits at every letter.

A selection over the first kind is a union of axis-aligned rectangles. A
selection over the second kind is a union taken **in the line's own axes**,
because a band across rotated text is a rotated band and its bounding
rectangle in page axes is not the same shape.

[`Band`] names which of the two a glyph is in; [`Accum`] does the arithmetic
for that one. They are separate types because the question *"which band"* is
answered once per glyph from three maps, and the question *"how does this
band grow"* is answered from the band's own frame — and merging them would
mean the second question could be asked of a glyph whose band was never
settled.

## The invariant this file is built around

**Only equal bands are merged, and a band's identity fixes its variant.** A
[`Band::Engine`] glyph always produces an [`Accum::Page`] and a
[`Band::Rotated`] glyph always produces an [`Accum::Frame`], so
[`Accum::absorb`] is never called on a mismatched pair. That is why its
mismatch arm does nothing rather than panicking: the state is unreachable,
and a drag that killed the application would be a far worse outcome than a
selection one box short.

## What this module does NOT do

It does not project into canvas space, does not read the page's `/Rotate`,
and does not know what a selection is for. [`Accum::quad`] hands back PDF
user space corners and [`super::resolve`] takes them through
`find::reveal::quad_to_canvas` — the same function Find projects its hits
with, which is what makes a selected word and a found word land in the same
place on a rotated page.

## Item notes

### `enum Band`

One enum rather than two parallel maps because the whole point is that a
glyph belongs to exactly one of these: a rotated line, an engine line, or
neither. Two maps would admit the state where a glyph is in both, and the
box drawn from it would be whichever [`super::resolve`] happened to read first.

### `fn merges`

False only for [`Band::Loose`], which is per-glyph by construction: a
shared "unclaimed" key would merge every orphan on the page into one
box spanning the sheet, which is the failure the previous `usize::MAX`
sentinel guarded against by hand.

### `enum Accum`

Two variants rather than one general parallelogram because the page-axis
case is the overwhelming majority and reducing it to a degenerate rotated
frame would put every ordinary selection through trigonometry to produce the
number it already had.

### `fn absorb`

The two are always the same variant, because [`Band`] decides the
variant and only equal bands are merged. A mismatch is therefore
impossible rather than merely unexpected, and is left as a no-op rather
than a panic: a selection that silently drew one box short is a far
smaller failure than a drag that killed the application.

### `fn quad`

The corner naming is `/QuadPoints`' (§12.5.6.10) and is relative to
**the text's own baseline**, not to the page: `ul`/`ur` are the ascender
side and `ll`/`lr` the descender side, `ll`/`ul` the start of the text
and `lr`/`ur` its end. For `dir = (1, 0)` that is exactly
[`Quad::from_rect`]'s assignment, which is the check that the rotated
construction below generalises rather than replaces it.
