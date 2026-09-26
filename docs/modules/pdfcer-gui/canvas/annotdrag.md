# `canvas::annotdrag` — dragging a markup annotation to where it belongs

The other half of the annotation-drag fork. [`crate::canvas::dimdrag`]
answers for a **ce dimension**; this answers for everything else pdfcer puts
on a page — a stamp, an ink stroke, a callout box, a highlight, a note.

## What this closes, and how long it was open

`FEATURES.md` recorded it under the Format contextual tab:

> *"In `pdfcer-gui` a placed markup can be selected and deleted but not moved
> or resized yet — that is the Format-tab slice, still building."*

Selecting worked. Restyling worked. Deleting worked. **Dragging did
nothing**, and it did nothing in the most confusing way available: the
gesture was *consumed*. `canvas::interact` forks on `selection.annot()
.is_some()`, so an annotation selection took the dimension branch, and that
branch answers `None` for anything that is not a ce dimension. The content
branch — which does move things — was unreachable behind an annotation
selection by construction.

So the operator pressed inside a stamp, dragged it across the sheet, let go,
and the stamp was where it started with no message anywhere. That reads as a
broken program rather than as a missing feature.

## The half a canvas cannot see, and it is why this needed an engine Pass


> A move has two halves and **only one of them shows up in a render.**
>
> 1. **`/Rect`** moves the painted result for free — §12.5.5 recomputes the
>    placement matrix from the appearance `BBox` and the new `/Rect`.
> 2. **The geometry keys** — `/L`, `/Vertices`, `/InkList`, `/QuadPoints`,
>    `/CL` — hold *absolute page coordinates*, and they are what **any other
>    tool** regenerates an appearance from.
>
> Move only (1) and the annotation looks right in your canvas, right in a
> screenshot, right in pdfcer — and is reconstructed **in the old place** by
> the next viewer that rebuilds it.

⇒ **That is a defect this shell could have shipped and never seen.** Every
instrument this project owns — the rendered canvas, a screenshot, a driven
pixel check — reads the appearance stream, and all four would have agreed
the stamp moved. The operator would have found out a week later, in Acrobat,
and reported it as *"it moved back"*. Recorded here because the class
generalises: **when a document format stores one fact twice, a renderer is
not an oracle for whether both copies were written.**

## Why there is no shell-side geometry arithmetic here at all

This module computes a `(dx, dy)` in page points and sends it. It does not
touch `/Rect`, does not enumerate geometry keys, and does not know which
subtypes have them. That is not laziness about coverage — it is the same
rule `dimdrag` states for placement: a second implementation of the engine's
own arithmetic is a second thing to keep in step, and the one that drifts is
the one whose tests are thinner.

`AnnotationMove::geometry_keys_moved` reports which keys were found, **and
an empty list is a correct answer** — a Text note, a Stamp or a Link has no
geometry key because its `/Rect` *is* its geometry. The engine says so
explicitly, and reading empty as failure is the mistake it warned about.

## Two refusals, and the engine names the verb for each

`EditError::AnnotationMoveWrongVerb` fires for a **widget** (use
`move_widget`) and for a **ce dimension** (use `move_dimension`, which
re-measures). Neither can reach here:

| | why it cannot arrive |
|---|---|
| widget | `selection::annot::selectable` excludes `/Widget` outright — the form surface owns those presses |
| ce dimension | [`crate::canvas::dimdrag`] claims it first, and `AnnotKind` makes the fork a `match` the compiler checks |

⇒ The engine's refusals are the **backstop**, not the mechanism. That is the
arrangement `set_markup_style` established — the shell routes by `AnnotKind`
and the engine refuses by name — and it has now paid three times.

## Rule 4

The ghost drawn while a drag is in flight is **the cursor**, which the rule
permits by name: the same class as a snap indicator, a rubber band or a
resize grip. Nothing about the annotation itself is tinted, badged or
flagged, and the one-line test passes — a screenshot of the canvas mid-drag
differs from the saved file by a marching outline, which is where the
pointer is and not what the document says.
