# `canvas::textedit::place` — where a press puts the caret

## The seam

Split out of [`super`] on 2026-08-21 under R2, when the text box took that
file past the 1,500-line ceiling. It is the seam the file already drew with
its own banner — *"Starting a draft"* — and it is a real subject rather than
a size-driven cut: everything here answers **where does a press put the
caret**, and nothing here knows what typing does afterwards.

## The two gestures, and why they are two

| gesture | anchor | what commits |
|---|---|---|
| **click** on existing text | [`Anchor::Run`] | `edit_text` — one show operator's text replaced |
| **click** on bare page | [`Anchor::Origin`] | `add_text` — one single-line run at a point |
| **drag** a rectangle | [`Anchor::Box`] | `add_text` boxed — a wrapped paragraph |

The third arrived on 2026-08-21, on the operator's *"I should be able to
make it multi line."* It has to be a drag, and the reason is the file format
rather than a preference: **a PDF has no paragraph.** Each visual line is its
own show operator at its own absolute position, so something must decide
where the second line starts — a width to wrap against — and a width is a
rectangle somebody draws.

## What this module refuses, and why each refusal is a sentence

[`Refusal`]'s variants are shown on the status row, never dropped. That is
`DEFECTS.md` D4a's whole lesson: the old shell's answer to a caret it could
not place was a boolean and a keyboard that stopped responding, and the
operator reported the feature as broken for weeks.

## Item notes

### `const MIN_PT`

Twelve is one default line's height: below that the box could not show a
single line of text at the pen's default size, so it is not a box the
operator can have meant.

### `fn has_no_anchor`

A thin forward to [`crate::app::state::OpenDoc::run_has_no_anchor`], which
owns the extraction and the cache. It is worth a named function here anyway:
this is the one place in the shell that asks *"can pdfcer-core edit this
run"*, so there is one line to change when the answer changes — which it
did, on 2026-08-20, when form editing landed and this stopped being about
forms at all.

# Why the answer is cached one level down and not here

Because the only way to ask is a **second extraction of the whole page with
provenance on** - `PageTextCache` deliberately leaves provenance off, since
the canvas, the find bar and the text sweep do not need it and it costs.
Measured on the benchmark CAD sheet: **336 ms**. Doing it inline froze the
UI for a third of a second on every click that landed on text, and made a
driven check flake because the trace had not been written by the time the
settle window closed - a performance defect that presented as harness
flakiness, which this project has been caught by before.

`None` means **not measured**, never "yes". See `FormRunCache::flags`.

### `fn caret_index_at`

Returns a character index in `0..=glyphs.len()`, or `None` when the run or
the page cannot be read.

# How it decides

Every glyph `pdfcer-core` publishes carries its origin `x` and its `advance`,
so a run's character boundaries are `x[0]`, `x[0]+adv[0]`, `x[1]+adv[1]`, …
The click's x is compared against each glyph's MIDPOINT: past the midpoint
means the caret belongs after that glyph. That is the rule every text field
uses and it is what makes clicking "on" a character feel like clicking
*near* the boundary the operator was aiming at, rather than requiring them
to hit a one-pixel gap.

# Why the x axis alone

Because a run is one show operator, which is one baseline. The vertical
question - *which line?* - was already answered by `resolve_run`'s hit test
before this is called, and asking it again here with different arithmetic is
how a caret comes to land on a different line from the one that was clicked.

Rotated runs. The comparison is done in **PDF user space** against the
glyph origins as published, which is the same space `resolve_run` works in.
For a run rotated off the horizontal this compares the wrong axis and the
caret will land at a boundary the operator did not aim at - it is still
inside the run, and still better than always landing at the end, but it is
not right. Fixing it properly means projecting the click onto the run's own
baseline direction, which needs the text matrix rather than the glyph
boxes. Recorded here rather than silently approximated.

### `fn resolve_run`

Two hops, and the first is the one `canvas::mapping`'s header calls *the
classic silent defect*: the canvas is Y-down from the page's top-left with
`/Rotate` applied, and every glyph position `pdfcer-core` publishes is in PDF
user space — Y-up from the un-rotated CropBox. `viewer::canvas_to_pdf_space`
is the single bridge, and it works by inverting the **renderer's own**
transform, so the geometry and the picture agree by construction. This is
deliberately the identical route `canvas::textsel::hit` takes, because a
second conversion here is how a caret comes to land on a different line from
the highlight.
