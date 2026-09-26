# `canvas::markup::pen` — the colour and width the next markup is authored with

## What this closes

`RIBBON_IA.md` §5.5 specifies a **Style** group on the Markup tab —
*"Colour · Line width · Fill · Opacity"* — and marks it `partial G`,
*"colour only"*, describing the **old** shell. This shell had none of it:
`MarkupKind::rgb()` returned a hard-coded red, `PEN_WIDTH_PTS` was a
hard-coded `2.0`, and the manifest's `colour_swatch` item was declared and
never built, so the Style group rendered an empty caption.

§5.5's own note on it is the operator's complaint restated in advance:

> The `Style` group sets defaults for the next markup. … Both must exist;
> today only the first does, **which is why a placed markup feels final**.

## The seam was already named, and this took it

`MarkupKind::rgb`'s doc comment predicted this module almost exactly:

> The old shell carried `markup_color`/`markup_width` on the application
> and a swatch in its ribbon. This shell has neither … So the pen is a
> **default**, stated once, in the one place that builds a spec, and the
> seam for a real pen control is exactly this function: **give it a colour
> and a width from the document's markup state and nothing else in the
> module changes.**

That is what happened: `spec` and `action` gained a `Pen` parameter and
nothing else in `canvas::markup` moved. The prediction was right down to
the shape of the change, which is worth recording — a doc comment that
names its own seam is the cheapest refactoring aid this project has.

## Why the pen lives on the application and not on the document

A pen is a **tool setting**, not document content. An operator who picks
green expects the next rectangle to be green in *whatever* file they draw
it in, exactly as a pencil does not change colour when you turn the page.
Putting it on `OpenDoc` would reset it on every open, which is the
behaviour of a program that has forgotten what you told it.

It is deliberately **not persisted** to the settings file, and that is a
narrower statement than it sounds. `pdfcer_core::settings` is for choices the
*standard* leaves open — its window says so in its own first paragraph —
and a pen colour is not an ambiguity, it is a preference. Persisting it
belongs with the ribbon layout and the keymap, under the same `userdata/`
roof and in their own file, which is `SHELL_FRAMEWORK.md`'s subject rather
than this one's.

## EIGHT PENS, NOT TWO — and the argument that said otherwise is kept
## here, superseded rather than deleted


> **Two pens, not one, and it is not a per-kind palette.** … What this is
> **not** is a colour per markup kind. `MarkupKind::rgb`'s own note argued
> that down when three kinds were added: they are comment linework, they have
> to be seen against a drawing that is already black on white, and *"a
> per-kind palette would be a style decision made in code where the Style
> group is the surface that owns it."* One pen for every geometric kind is
> the same answer, now made choosable.

### Why that was a good argument

It is worth saying plainly, because the correction is only useful if the
thing being corrected was reasonable. The argument had two halves and both
were sound at the time:

1. **A colour chosen in code is a style decision made in the wrong place.**
   True, and still true. A shell that hard-codes a green underline because
   somebody liked green has put a preference in a source file where no
   operator can reach it.
2. **Comment linework has one job — to be seen over black-on-white CAD.**
   Also true. Nothing about a per-kind palette makes an underline more
   legible over a drawing than the pen colour would.

### Why it is superseded anyway

Because it answered the wrong question. It asked *"can this shell justify
inventing eight colours?"* — and the answer to that is still no. The
operator's ask of 2026-09-06 asks something else:

> *"Also make sure you've used the same default colours and style look for
> these things as Adobe."*

⇒ The values are no longer this shell's to choose. [`super::palette`] reads
them out of **Acrobat's own tool-defaults store**, and Acrobat does not use
one colour for everything: its highlighter is orange, its underline is blue,
its strikeout is a light red that is not the shape red, and its sticky note
is violet. A single pen cannot express that table, so the table is the shape
the pen has to have. The style decision is not being made in code — it is
being **transcribed from the program the operator compares against**, which
is this project's standing tie-breaker for anything of this kind.

Half of the old argument survives intact and is worth keeping: the values
must still be the operator's to override, and every slot below is. What
changed is only where the *shipped* value comes from.

### The slots, and why they are named rather than an array

[`PenSlot`] has one variant per **key Acrobat keeps a separate default
under** — not per [`MarkupKind`] variant, and the difference matters in both
directions:

* The seven geometric kinds share `cSquare`/`cCircle`/`cLine`/… , all holding
  the identical red, so they share [`Pen::ink`]. Giving each its own field
  would be seven copies of one number and seven places for it to drift.
* Squiggly and Stamp hold that same red **under their own registry keys**, so
  they get their own slots even though the shipped values agree today. An
  operator who recolours the shape pen has not thereby asked for a recoloured
  squiggly — Acrobat's do not move together, and collapsing two slots that
  happen to agree is how a per-kind palette quietly becomes a single pen
  again.

[`Pen::colour_for`] is the total function from a kind to its slot's colour;
[`Pen::colour_of`] and [`Pen::set_colour`] are the slot-addressed pair the
swatch uses. Both matches are exhaustive, so a ninth slot fails to compile
rather than silently landing on the shape pen.
