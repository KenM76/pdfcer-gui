# `canvas::textedit::disposition` — **which way the rest of the line moves**

One public function, [`choose`], and the whole argument for its answer. It
decides the single field of
[`pdfcer_core::text_edit::EditOptions`](pdfcer_core::text_edit::EditOptions) —
the [`FollowerDisposition`] — which a caller that constructs
`EditOptions::default()` never decides at all.

## The two commits this rule exists to prevent

`DEFECTS.md` **D4b** lists two cases where a text edit is not merely
unhelpful but **wrong on commit**, and both have the same shape: the engine
already carries the mechanism, and a shell that passes
`EditOptions::default()` — i.e. [`FollowerDisposition::Reflow`] —
unconditionally never selects it, for any run on any page of any document.

Both claims below are **checked against the engine's source** rather than
taken from the defect register. A claim about that repository that has not
been read there is a claim this shell has invented.

### 1. A right-aligned, centred or justified tail moves the wrong way

[`FollowerDisposition::Pin`] exists for precisely this, and its own doc
comment in the engine (`pdfcer-core`'s `FollowerDisposition`) says so: it
pins survivors in place with a compensating `TJ` number, **for a justified
or right-aligned tail that must not move.**

Under `Reflow` the engine walks the operators after the anchor and adds `ΔA`
to every following absolute `Tm`'s `e`. On a **left**-aligned line that is
right: the line grows to the right and its tail follows. On a right-aligned,
centred or justified line the tail is *flush against something* — a margin,
a centre, a column edge — and moving it is the one thing that must not
happen. `Pin` leaves every follower `Tm` untouched and consumes `ΔA` with a
compensating number inside the anchor operator instead.

### 2. Rotated or skewed text is shifted along the wrong axis

This is the sharper of the two, and it is the one that bites this operator's
documents specifically, because rotated text is what a CAD title block is
made of. The engine's reflow branch (`pdfcer-core`'s `plan_edit`) rewrites
each follower `Tm` by adding the cumulative advance to element `e` and
leaving `a`, `b`, `c` and `d` alone.

`ΔA` is an advance in **text space**. `e` is the translation component of
`Tm`, and `Tm` maps text space to **user space**, so a text-space advance of
`Δ` displaces a follower by `Δ·(a, b)` in user space — not by `(Δ, 0)`. The
two agree only when `(a, b) = (1, 0)`, i.e. when the text is upright and
unscaled in x. On a 90°-rotated title-block line the baseline runs up the
page and the engine slides the tail *sideways*.

There is **no rotation guard on this path**. The engine's `same_line`
requires a follower to carry the *same* rotation as the anchor, which is not
the same as refusing rotation: a rotated line whose tail is rotated to match
is exactly the case that gets shifted along the wrong axis. The
reflow-**apply** path does guard — `check_uniform_axis_aligned` refuses when
`|b| > MTX_EPS || |c| > MTX_EPS`, with `MTX_EPS = 1e-6` — and this module
ports that predicate ([`is_upright`]) rather than inventing a second
tolerance.

## Why the rotation answer is `Pin` and not a refusal

`reflow_apply` *refuses* rotated text. This module does not, and the
asymmetry is deliberate rather than a relaxation.

Those are different operations. A **re-wrap** invents line breaks and new
line origins, so under rotation it would have to re-derive a whole
two-dimensional layout in a frame it does not understand; refusing is the
only honest answer. An **in-place replace** does not: it rewrites one show
operator's string and then has to answer one question — what happens to
what came after it. And `Pin` answers that question **correctly under
rotation**, not merely less wrongly:

* no follower `Tm` is written at all, so nothing can be displaced along the
  wrong axis — the entire mechanism of the defect is not reached;
* the compensation is a `TJ` number, and `TJ` offsets are applied in **text
  space** (§9.4.3), which is the rotated baseline's own frame. An
  advance-relative follower inside the same show sequence is therefore
  compensated *along the baseline*, which is where it actually needs to
  move.

So under rotation `Pin` is the right answer for the same reason it is the
right answer for a right-aligned tail: the thing that must not happen is a
follower being moved by a number computed in the wrong frame.

**What it costs, and it is disclosed rather than hidden**: a pinned tail
does not make room. If the replacement is longer than what it replaced, the
edited text grows into the pinned tail. The engine discloses overflow for
`Reflow` and says nothing for `Pin`, so [`Reason`] carries the fact and
`crate::text::textedit` turns it into the sentence the status bar shows.
Rule 4: disclosure lives off-canvas.

## The order of the two rules, which is a decision

Rotation is tested **first** and wins. The two can disagree — a rotated
left-aligned title-block line asks for `Reflow` on the alignment rule and
`Pin` on the rotation rule — and the rotation rule wins because the two
claims are not the same kind of claim. Alignment says *"this would look
wrong"*; rotation says *"the arithmetic the other branch would perform is
not the arithmetic this frame requires"*. A preference loses to a
correctness bound.

## What is deliberately NOT here

The **third** case, which is the honest limit of this fix and is stated so
it is not mistaken for coverage: alignment is detected by the engine from a
block's *lines*, and `ReflowEngine::infer_alignment` returns
[`AlignmentSource::SingleLineDefault`] — alignment `Left` — for any block
with one line. A one-line right-aligned run is therefore **indistinguishable
from a one-line left-aligned run** by anything in the engine, and this
module does not pretend otherwise: it answers `Reflow` and records
[`Reason::AlignmentUndetectable`], which is a *disclosed* fall-back rather
than a claim. Widening it would need a block-relative or margin-relative
signal that `pdfcer-core` does not publish, and inventing one here would be
the shell deciding a question the engine owns.

## Item notes

### `fn a_rotated_ctm_is_not_upright_even_with_an_upright_text_matrix`

The case a `Tm`-only guard would miss, and the one that matters most
here: a landscape CAD plot rotates the whole content stream with one
`cm`, leaving every `Tm` on the page reading as identity. A guard that
looked only at the text matrix would answer "upright" for every glyph on
such a sheet and the defect would survive the fix intact.

### `fn a_rotated_run_pins_regardless_of_alignment`

The rung-order assertion. It is written against `None` and against a
real left-aligned detection in `the_engines_own_findings_drive_the_choice`,
so a future edit that moved the alignment test above the rotation test
fails here by name rather than by producing a subtly displaced tail on a
document nobody re-opens.

### `fn an_unresolvable_block_is_an_undetectable_alignment_and_not_a_left_one`

Both answer `Reflow`, so the disposition alone cannot tell them apart —
which is why [`Reason`] exists and why this asserts the *reason* rather
than only the disposition.

### `fn a_line_made_of_several_pieces_pins_over_a_left_alignment`

The failure it forbids is concrete: a SolidWorks parts table writes one
show operator per cell, and every cell is left-flush, so the alignment
detector answers `Left` and `LeftAligned` reflows. Under `Reflow` the
engine adds `ΔA` to the `e` of every following absolute `Tm` in the text
object — so widening `PART` to `PARTS` slides `DESCRIPTION` and `QTY`
sideways. **Content the operator did not touch, moved by an edit that did
not mention it.**

Written against a real left-aligned *detection* rather than against
`None`, because `None` would pass on a build where the rung sat below
alignment: `AlignmentUndetectable` also reflows, so only a positive
`Left` finding can prove the rung order.

### `fn rotation_outranks_the_multi_run_rung`

Both pin, so the *disposition* cannot tell them apart — which is exactly
why `Reason` exists and why this asserts the reason. A reader who sees
`Rotated` is being told the sharper fact: a follower shift computed in
user-space x on a rotated baseline is wrong in a way that has nothing to
do with how many pieces the line has.

### `fn the_fallback_is_byte_identical_to_what_the_old_shell_passed`

This is the assertion that says the rule adds a decision rather than
changing one: an upright, left-aligned or unclassifiable run commits with
exactly the options `EditOptions::default()` carries.

### `fn pins_the_tail_agrees_with_the_disposition_for_every_reason`

An arithmetic-identity test: two derived facts about one value, asserted
to agree, rather than a comment asking the next reader to keep them in
step.

### `type Finding`

[`DetectedAlignment`] is `#[non_exhaustive]`, so nothing outside
`pdfcer-core` can build one, so a [`choose`] that took it could only ever be
tested through a real page. Its *fields* are two plain `Copy` enums whose
variants are constructible anywhere, and they are the entire input to the
rule — the three raggedness measurements and the tolerance beside them are
evidence for the finding, not part of it.

So the seam is here: [`from_detection`] does the one-line reduction at the
single place a real detection arrives, and every case in the table on
[`choose`] is a unit test with no fixture. That is the same shape
`canvas::textsel::gate` uses — a pure predicate over two small values,
separated from the page that produces them.

### `const MTX_EPS`

**Ported, not chosen.** It is `pdfcer-core`'s own `MTX_EPS` from
`text_edit/reflow_apply.rs`, the constant its `check_uniform_axis_aligned`
compares `b` and `c` against before refusing a rotated block. A second
tolerance picked here would be a second answer to "is this upright", free to
disagree with the engine's on exactly the matrices where it matters.

### `enum Reason`

[`choose`] returns this beside the disposition rather than only the
disposition, for two reasons that are both about honesty rather than
tidiness:

1. `Pin` has a cost (a tail that does not make room) and `Reflow` has a cost
   (a line that may overrun its margin). Which one the operator is about to
   pay is a fact they are entitled to before they press Accept, and it is a
   different fact in each case — so one generic "the line may move" sentence
   would be a sentence that is never quite true.
2. [`Self::AlignmentUndetectable`] is a **fall-back, not a finding**, and the
   difference is invisible from the disposition alone: it produces the same
   `Reflow` a confidently left-aligned block does. Collapsing them would make
   the shell state as detected something it defaulted to, which is the exact
   shape rule 4 forbids.

### `fn disposition`

Written as a method on the reason rather than as a second `match` in
[`choose`] so the two can never disagree: a reason is the *whole* of the
input to the choice, and a future fifth reason is a compile error here
rather than a silent `Reflow`.

### `fn pins_the_tail`

The predicate the status-bar disclosure gates on, kept here beside the
reason it derives from rather than re-spelled as a `matches!` at the one
call site, for the reason `CanvasTool::markup_kind`'s docs give: a
predicate with two readers is a predicate that drifts.

### `fn is_upright`

`true` when neither the text matrix nor the CTM carries a non-zero
off-diagonal term. Both are checked because either can rotate the glyphs:
§9.4.4's text rendering matrix is `Tm × CTM` (with the font scale between
them), so a page whose whole content stream sits inside a rotating `cm` puts
the rotation in the CTM while every `Tm` on it reads as upright. A guard
that looked only at `Tm` would pass every glyph on a rotated sheet — which
is exactly the SolidWorks landscape-plot case this fix is for.

`f32` in, because that is what
[`GlyphProvenance`](pdfcer_core::text_extract::GlyphProvenance) publishes;
widened to `f64` for the comparison so the tolerance is compared in the same
type the engine compares it in.

### `fn choose`

Pure: a matrix pair and the engine's own alignment finding in, an answer
out. No document, no session, no page — which is what lets every case in
the table below be a unit test rather than a fixture.

`alignment` is `None` when the caller could not resolve a block for the
caret at all (an empty page, a caret on a run the block recogniser did not
place). That is treated as [`Reason::AlignmentUndetectable`] and **not** as
left alignment, because "no block" and "a left-aligned block" are different
findings and only one of them is a finding.

| `Tm`/CTM | alignment | → | why |
|---|---|---|---|
| rotated/skewed | *anything* | `Pin` | the follower shift would be in the wrong frame |
| upright | `Right` / `Center` / `Justified` | `Pin` | the tail is flush against something |
| upright | `Left`, detected | `Reflow` | the line is meant to grow right |
| upright | single-line / ambiguous / `None` | `Reflow` | the engine's default, **disclosed as a fall-back** |

### `fn options`

A one-line adapter, and it exists so that **no call site constructs
`EditOptions` itself**. That is the whole defect this module prevents stated
as a rule: a default is what a call site gets whenever the type is
constructible at the point of use, and the default is wrong for two whole
classes of run. Here the only way to obtain one is to have already answered
the question.
