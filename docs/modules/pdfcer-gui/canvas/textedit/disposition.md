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

## ★ Why the rotation answer is `Pin` and not a refusal

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
