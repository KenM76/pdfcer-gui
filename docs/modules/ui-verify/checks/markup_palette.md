# `ui-verify/checks/markup_palette`

`a_new_markup_is_drawn_in_acrobats_red` — the colour an operator's first
comment shape actually comes out of the program, measured off the glass.

# The defect

`canvas::markup::palette` was written on 2026-09-06 from Acrobat DC's own
registry — `HKCU\…\DC\Annots\cAnnots\<subtype>\cstrokeColor`, read twice,
minutes apart, agreeing to six decimals. Shapes are **`#DB3425`**. The value
it replaced had been written from memory, and the same commit found the
highlighter's default is **orange `#FF6200`, not yellow** — *"what everybody
knows was wrong"*.

A palette written from memory is a defect that ships silently: nothing
crashes, every test is green, and the only symptom is that a drawing marked
up in pdfcer does not look like a drawing marked up in Acrobat when the two
are put side by side — which is the comparison the operator's own request
("*make sure you've used the same default colours and style look for these
things as Adobe*") is about.

# Why the sixteen unit tests beside it cannot see this

`palette::tests` is thorough and it is thorough about **the table**:
`each_constant_is_the_registry_value_it_claims_to_be` divides the byte by 255
and compares against the float from the registry;
`every_shipped_default_is_one_click_away_in_the_grid` checks the swatch grid
contains every default; `pen::tests` asserts `PenSlot::Shape` maps to
`MARKUP_RED`. **Every one of them passes on a build where a drawn rectangle
comes out black**, because none of them is downstream of the drawing.

Between `MARKUP_RED` and a red line on the glass there are six links, and no
test in the workspace observes more than one of them at a time:

| # | link | why a unit test cannot see it |
|---|---|---|
| 1 | the Rectangle tool arms in Review | `Capabilities::for_mode` reads the real manifest; the mode is entered by a segment click or by `PDFCER_DIAG_INVOKE` |
| 2 | a drag becomes a `/Square` with the **pen's** colour | `canvas::markup::spec` builds the `MarkupSpec` from the live `Pen`; the drag's two corners come from the canvas transform, and nothing in-process performs one |
| 3 | `add_markup` writes `/C` | the engine's, and it is exercised in-process against a session that never paints |
| 4 | the appearance stream is **baked** with that colour | `/AP` generation is the engine's; nothing in the GUI's tests renders one |
| 5 | `pdfcer-render` paints the `/AP` onto the page raster | a separate crate, driven by the canvas |
| 6 | the composited window shows it | the compositor, the theme, and whatever the canvas draws over the top |

## What this check measures is the DEFAULT, and that is a deliberate scope

`canvas::markup::pen`'s header is explicit — the pen *"is deliberately **not
persisted** to the settings file"*, because a pen colour is a preference
rather than one of the ambiguities `pdfcer_core::settings` exists for, and
persisting it *"belongs with the ribbon layout and the keymap ... in their own
file"*, which is not built. So every launch starts from
`PenSlot::Shape`'s constant, and what this check measures is exactly what an
operator's **first** comment shape comes out — the reading that the operator's
own request is about.

⚠ A first draft of this header claimed the opposite, that the pen was loaded
from `userdata/` and that a private profile was therefore load-bearing here.
It was written from the shape of the neighbouring modules rather than from
`pen`'s own words, and it is corrected rather than quietly deleted because
**a premise nobody rechecks is how a check ends up asserting the wrong
thing**. The private profile ([`crate::sandbox`]) is still what this check
runs under and still matters — the stored **mode** reaches the ribbon, and a
check that inherited Edit would arm its tool in a different mode — but it is
not the pen that makes it matter.

⇒ A colour an operator has *changed* is a different subject, reached through
the Format ▸ Markup band's swatch, and its popup publishes no regions for a
harness to aim at. `markup_style`'s header records that limit for the same
control; it is named here rather than implied.

# What it measures, and the baseline that makes it mean anything

Three readings from two captures:

| | before the drag | after it |
|---|---|---|
| **the edge strip** — a thin box lying along where the rectangle's top edge will be | must be **blank paper** | must hold ink, and that ink must be `#DB3425` |
| **the interior box** — well inside the shape | blank | still blank |

**The blank baseline is not politeness, it is what stops the check
passing for the wrong reason.** A "the ink here is red" assertion is
satisfied by any red thing: a red title block, a red revision cloud already
on the sheet, a red selection outline. This project has paid for exactly that
once — `markup_node_edit`'s first draft sampled a corner of `four-pages.pdf`
that carries a **coloured title block**, and its assertion passed on a delta
of 28 pixels against a floor of 423. So the strip is asserted **empty first**,
and a fixture whose own content lies under it makes this check SKIP — a
statement that it could not measure — rather than pass on the fixture's ink.

The interior box is the differential: it says the shape is an **outline**,
not a filled blob. `canvas::markup::spec` authors every shape with
`interior: None` for a stated reason — *"a filled comment shape hides the
drawing it is a comment about, which on a CAD sheet is the whole content
under it"* — and that is a claim about the picture, so it is asserted from
the picture.

# How the stroke's colour is extracted — and why the obvious way fails

**There is no core pixel to sample.** `a1-titleblock.pdf` is a 2384 × 1684 pt
A1 sheet displayed fit-page in an 1100 × 800 window, which is **20 % zoom**:
a 2 pt stroke is **0.36 px wide**. Every pixel it lays down is a blend of the
ink with the paper behind it, and no threshold, no percentile and no amount
of taking-the-darkest-quarter recovers a pure sample from a line thinner than
a pixel.


## The measure that survives it: the direction away from the paper

Compositing `α` of an ink over paper gives, per channel,

```text
c' = α·c + (1 − α)·paper
so   paper − c' = α·(paper − c)
```

⇒ **The vector from the paper to the measured colour is the vector from the
paper to the ink, scaled by `α`.** Its *direction* does not depend on `α` at
all. So [`hue_from_paper`] normalises that difference by its largest
component and compares directions, and the dilution — the thing that defeats
every absolute comparison — divides out exactly.

Worked, on that same first run:

| | `paper − c`, per channel | normalised |
|---|---|---|
| Acrobat's `#DB3425` | 36, 203, 218 | **0.165, 0.931, 1.000** |
| measured `#EB9D96` | 20, 98, 105 | **0.190, 0.933, 1.000** |
| the highlighter's `#FF6200` | 0, 157, 255 | 0.000, 0.616, 1.000 |
| the old yellow `#FFFF00` | 0, 0, 255 | 0.000, 0.000, 1.000 |

The measurement lands **0.025** from the right answer and **0.315** from the
nearest wrong one — a factor of twelve, on a reading taken through a
sub-pixel stroke. [`HUE_TOLERANCE`] sits between them, and
[`tests::the_tolerance_cannot_swallow_a_near_miss`] refuses to let it be
widened past any of them.

⚠ **This measures hue and not strength**, deliberately and with a cost: a
rectangle drawn in Acrobat's red at 10 % opacity passes. That is the right
trade here — `/CA` is a different control with a check of its own to be
written — but it is a limit rather than an oversight, so it is written down.
What it does still require is that the ink be *far enough* from the paper to
have a direction at all; see [`MIN_PAPER_GAP`].

## Why not [`crate::pixels::contrast_at`]

That oracle answers *"is this legible"*: the **dominant** bucket is the
background and the bucket with the largest luminance gap is the foreground,
both quantised and then averaged over every pixel in the bucket. For a
sub-pixel stroke the largest-gap bucket is the least-diluted skirt averaged
with the rest of the skirt. It is the right tool for a caption on a plate and
the wrong one for *what colour is this line*.

# Calibration

```text
--pdf fixtures/a1-titleblock.pdf --doc-point 0,300,500
```

⚠ `--doc-point` is **0-based** and is not what this check aims with — it
places its own shape in page fractions, so it works on any single-page
fixture with blank paper in the middle third. The point is still required by
the harness's convention and is passed through to nothing here.

The fixture must have **blank paper across the middle of page 1**. Both
`fixtures/a1-titleblock.pdf` (a real CAD sheet, blank inside the frame) and
`fixtures/four-pages.pdf` satisfy that; a fixture that does not makes this
check SKIP with the strip's ink count in the reason.

# Every way this reports SKIP, and why none of them is a pass

* no binary, no `--pdf`, `--no-input` — the harness never began;
* the fixture's own content lies under the strip or inside the shape, so
  neither reading could be attributed to the mark this check draws;
* the canvas is not showing page 1, so the page fractions describe something
  other than what is on screen;
* the rectangle tool authored nothing — that is `markup_move`'s subject and
  there is no mark here to have a colour.
