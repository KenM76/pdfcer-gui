# `canvas::textannot` — the three markup kinds that carry WORDS


## ★ Why they were left out, and why that was right at the time

`shell::commands::reach`'s register carries the reason verbatim, quoting
`canvas::markup`'s own table of kinds it deliberately does not handle:

> *Note · text box · sticky · stamp — Text-bearing, not geometric. A
> different gesture (place, then type) and a different spec type
> (`TextAnnotSpec`).*

Both halves are true and neither is small. **Nothing about the
drag-and-release machinery the seven geometric kinds share applies here**:
those author on release, from geometry alone, with a pen. These cannot —
releasing the mouse produces an *empty box*, and an empty box is not an
annotation, it is a rectangle nobody asked for.

## The gesture: place, then type, then commit

| kind | placing gesture | why |
|---|---|---|
| [`TextAnnotKind::TextBox`] | **drag a rectangle** | a `/FreeText` is painted *into* its rect and wraps to it, so the operator is choosing how wide the text is. A click would have to invent a width |
| [`TextAnnotKind::Sticky`] | **one click** | a `/Text` marker is fixed-size and `NoZoom` — its rect's width and height do not affect what is drawn, so asking the operator to drag one would be asking for a number that is discarded |
| [`TextAnnotKind::Stamp`] | **drag a rectangle** | pdfcer's stamp appearance is a framed label scaled into its rect, so the drag is choosing how big the stamp is |

Then the dialog opens, and **nothing is authored until Accept**. That is
rule 4 applied to a gesture whose output is words: a half-typed note
committed on a stray click would be content the operator did not write.

## ★ Escape has two meanings here and they are ordered

A placing drag in flight is abandoned by Escape, exactly as a markup band
is — that rung already exists and this kind rides it. Escape with the
**dialog** open is the dialog's, and closes it without authoring.

The two cannot both be live: the dialog only opens once the drag is over.
Stating it because the ordering is the kind of thing that looks obvious
until a third claimant is added to `canvas::keys`' ladder.
