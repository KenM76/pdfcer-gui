# `canvas::constrain` — what Shift does to a drag, written down once

## Why this module exists at all

`ui-conventions/drag-moves.md` D5: modifiers constrain, and the constraint
is announced. Shift-preserves-aspect is not an advanced feature, it is *the*
resize convention — PowerPoint, Illustrator, Inkscape, Figma, Visio and
AutoCAD all answer Shift the same way. An operator who holds Shift and gets
a free-form resize does not conclude that pdfcer chose differently. They
conclude it is broken, and they are close enough to right.

## ★★ The reason it is ONE module and not five call sites

There are five drags on this canvas that a modifier ought to constrain —
move, resize, Bézier handle, ce-dimension label, ce-dimension vertex — and
the arithmetic for four of them is *the same arithmetic*. If each spelled it
for itself, they would agree on the day they were written and separate under
maintenance.

> **A predicate with two claimants must exist exactly once.**

That rule is enforced mechanically for the typing guard —
`tools/gates/check-typing-guard.sh` fails the build on a second copy of
`text_edit_focused()` — and held by construction here: one axis rule, one
aspect rule, one announcement, and the five call sites are wiring.

## The two rules, and why each is the one every program uses

### 1. Axis lock — [`axis`] and [`toward`]

A constrained *translation* keeps the axis the pointer has travelled
furthest along and zeroes the other. Not the axis it started on: an operator
who begins a drag slightly off-horizontal and then commits to vertical
expects the object to follow, and a lock sampled once at the press would
trap them on the axis of their first three pixels. Re-deciding every frame
is what Illustrator, Inkscape and every CAD package do, and it is
self-correcting — let go of Shift and the object returns to the free path,
because the constraint is a *filter on the live delta* and holds no state.

### 2. Aspect lock — [`aspect`]

A constrained *resize* applies one factor to both axes, and the factor it
keeps is the one the pointer travelled furthest to produce, measured as a
**fraction of the box's own extent on that axis**. That is exactly
`|s − 1|`, because [`crate::canvas::resizing::factors`] computes
`s = 1 + d/extent` — so comparing the two factors' distance from unity *is*
comparing relative travel, with no second derivation to drift.

★ **And the mid-edge grips fall out for free.** `East` and `West` leave
`sy` at exactly `1.0`; `North` and `South` leave `sx` at `1.0`. A factor of
`1.0` is distance zero from unity, so it can never win, so the live axis's
factor is applied to both — which is proportional resize driven from one
edge, which is what Figma and Google Slides do with Shift on a side handle.
One rule, both cases, no branch on the grip. This module therefore does
**not** take a [`crate::canvas::handles::Grip`], and that absence is the
design rather than an omission.

## What Shift does NOT do here, stated so it is a decision

- **It does not scale about the centre.** That is Alt in most programs and
  is a separate, unbuilt thing. Nothing in this module pretends otherwise.
- **It does not disable snapping.** That is Ctrl in most programs, and this
  shell's snapping is `canvas::snap`'s to own.
- **It does not constrain to 45°.** A diagonal lock is the third common
  axis-lock flavour (Illustrator offers it on a plain translate). It is not
  built because a 45° move on a CAD sheet is a coincidence rather than an
  intent, and offering it would make the *horizontal* and *vertical* cases
  harder to hit — the operator would have to be within 22.5° of an axis
  instead of within 45°. Recorded as a decision; say the word and it is four
  lines.

## The announcement, and why it is a memory slot

D5's second clause — *"the affordance shows the constraint while it is
active"* — is answered twice, deliberately:

1. **The ghost itself.** A locked drag visibly stops moving off-axis and a
   locked resize visibly keeps its proportion. That is the primary feedback
   and it needs no words.
2. **A sentence on the status row**, [`caption`], because the ghost answers
   *"the object is behaving like this"* and not *"because you are holding
   Shift"* — and an operator who cannot tell whether the modifier did
   anything is exactly D5's stated failure mode.

It travels through `egui::Memory` rather than a field on `PdfcerApp`, for the
reason `crate::pagedrag::caption` already established for the identical
problem: the status bar is composed **before** the central panel (a
full-width bar must be added before any side panel or it does not span the
window — `crate::app`'s header), so the canvas cannot hand it a value on the
same frame. A memory slot stamped with the frame number costs one lookup per
frame, lags by one frame at 60 Hz — which no eye resolves — and, crucially,
**retires itself**: [`caption`] answers `None` as soon as the stamp is more
than a frame old, so there is no state anybody has to remember to clear.
State that must be cleared is state that will one day be shown against the
wrong document.
