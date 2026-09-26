# `text::panels::annotgeometry` — the words the **annotation** half of the
Properties panel's geometry section owns

## Why a module of its own, when the four field labels are next door

Because the four field labels are **not** this module's, and that is the
finding worth writing down rather than the reason for the split.

[`crate::panels::properties::geometry`] draws one section over two kinds of
subject — a page-content object and a markup annotation — and it draws them
with the **same heading, the same units note, the same four labels and the
same Apply button**. Those all live in [`super::properties`] and are called
by that section unchanged whichever subject it has. Duplicating them under
an annotation-shaped name would have produced a second copy of *"Points,
measured to the bottom-left corner. Y increases upward."*, which is the one
sentence in this whole feature that must not exist twice: it states the
coordinate convention, and two copies of a convention is how a panel ends up
presenting Y from the top in one half and from the bottom in the other.

⇒ **What is here is only what the annotation subject adds**, which is the
one reason a control can be unavailable over an annotation and cannot be
over a path: the *file* has refused, before the operator has typed anything.

## The register: a hover on a greyed control answers "why can't I", not
"what went wrong"

The sentence below is read while the pointer rests on a control that is
visibly present and visibly dead. The operator has not acted yet — nothing
has failed, nothing is unchanged, and a sentence written in the past tense
(*"that change was refused"*) would describe an event that has not happened.
So it is present tense and it names the **agent**: the file marks it locked.
[`crate::text::markup::NodeEditRefusal::line`]'s `Locked` arm is the same fact
in the other tense, for the surface where the operator has already pulled a
grip and watched nothing move, and the two are deliberately worded
differently rather than shared.

## What is deliberately NOT here, and it is the interesting half

**A pre-press warning that a non-uniform resize may be refused.**

`EditSession::resize_annotation` refuses to scale an annotation whose
appearance stream pdfcer did not draw, unless the scale is uniform or
`ResizeOptions::allow_appearance_distortion` takes the distortion
knowingly. It is a real refusal an operator can reach from these fields by
typing a Width and leaving Height alone.

It gets **no string here**, because the condition is not one this panel can
evaluate. The engine decides it by rebuilding the annotation's appearance
from its own spec and **comparing bytes** — a question about the file that
cannot be answered without doing the work. A hover reading *"this may be
refused"* would therefore be a guess, and it would be the wrong guess for
every mark pdfcer drew itself, which is most of the marks on this operator's
sheets.

⇒ So the refusal is surfaced **after** the press and **by name**, through
`app::actions::annots::resize`'s existing `inspect_err` arm and
`app::status::decline::record_resize_not_rebuildable` — the identical
sentence the eight resize grips already produce for the identical engine
error. One refusal, one wording, one place. A typed Width that the engine
declines says exactly what a dragged one says, which is the property that
makes the typed route a second *input* rather than a second *feature*.
