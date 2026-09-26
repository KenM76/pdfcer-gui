# `canvas::markup::swatch` — the Markup ▸ Style group's one control

The `colour_swatch` custom item the manifest has declared since S2 and
nothing ever drew, so the Style group rendered a caption over an empty band.

## Why it is a Custom item and not three commands

`egui-shell`'s `Item::Custom` is the extension point for a control that is
*not a button* — its own documentation names *"a split button with a
gallery"* — and this is three of those. A `Command` item can only render as
a button, and a button cannot ask *which colour* any more than the Recent
item's button could ask *which document*. The manifest declares
`Item::custom("colour_swatch")` and the application supplies the renderer,
exactly as it already does for Recent.

That is also why this is not three registered commands. A command is a verb
the operator invokes and the shell dispatches; setting a pen colour is not
a verb, it has no undo, it raises no `Action`, and giving it a handler token
would put a no-op through the dispatch `match` for every click of a colour
picker.

## ★ What it sets, and the one thing it deliberately does not

`RIBBON_IA.md` §5.5's Style group is *"Colour · Line width · Fill ·
Opacity"*. Two of the four ship here and two do not, and the two absences
are different in kind:

| control | state | why |
|---|---|---|
| **Colour** | ✅ | two swatches, each opening [`super::palette`]'s grid of Acrobat's own colours — see [`super::pen`] on why there are eight slots and two controls |
| **Line width** | ✅ | a drag value in points, over the pen's own range |
| **Fill** | ⬜ | **a design decision about the PEN, and only about the pen** — see the row's own correction below |
| **Opacity** | ✅ | **shipped 2026-08-28** — a percentage drag value writing `/CA`. This row said *"blocked on the engine"* for four months after it stopped being true; see below |
| **Line style** | ✅ | **shipped 2026-09-06** — a four-entry chooser writing `/BS` `/S` and `/D`. §5.5 does not list it; it is here because it is a property of the next mark exactly as the four above are, and because the engine shipped the author-time half (`MarkupOptions::dash`) beside the restyle half on the day the Format tab got its own copy. See [`super::linestyle`] |

⚠ The table is now **five rows over a four-item specification**, which is a
table that has outgrown its source rather than one that is wrong. §5.5 was
written before a dash was expressible at all; the *Line style* row is the
one entry here that this shell added rather than answered. It is called out
so that the next reader does not spend a minute looking for a fifth bullet in
`RIBBON_IA.md` §5.5 that is not there — the entry they will find is §5.8's,
about the **other** surface.

### ★★★ The Fill row, corrected: it was describing ONE of two surfaces and
### did not say which

It used to read *"a design decision, not a gap"*, followed by [`super::spec`]'s
argument that a filled comment shape hides the drawing it is a comment about.
That argument is **still correct and still in force** — for this control. What
the row failed to say is that there are *two* fills in this program and it was
only ever talking about one of them:

| which fill | whose surface | state |
|---|---|---|
| the fill the **pen** authors a NEW mark with | this module, the Markup ▸ Style group | ⬜ **deliberately absent**, and not a gap. `spec` passes `interior: None` for every shape, and on a CAD sheet an unfilled comment shape is the only kind that does not hide the content it is about |
| the fill of a mark **already on the page** | `panels::properties::markup`, the contextual Format tab | ✅ being built — an operator who wants a filled shape places one and fills it, which is a decision they made about a specific mark rather than a default applied to every future one |

⇒ The distinction is the whole answer to *"why can I fill that rectangle and
not this pen?"*, and a row that named neither surface could not give it.

### ★★★ The Opacity row, corrected: it was FALSE, and false in the direction
### this project has been wrong in before

It used to read *"blocked on the engine. Annotation transparency is `/CA`,
which `pdfcer-core` does not write yet — filed, accepted, not started."* That
was true when written and stopped being true on **2026-08-27**, when
`Pass 81.1` landed `MarkupOptions::opacity` in answer to a request this shell
filed itself. `set_markup_style` writes `/CA`; `add_markup_with` authors a
translucent mark in one verb and one undo entry; [`Pen::opacity`] and
`MIN_OPACITY` have existed since the day after.

⇒ The control was **already drawn** in [`show`] below, with its own trace
region and its own tooltip, while this table three screens above it said it
could not exist. Both were read by everyone who opened the file and only the
table was believed, because a table is what a reader trusts.

★★ That is the **eighth** stale blocker this project has found and it is the
second one *in this file*. The standing rule it produced — **a backlog row is
a record, not evidence** — has a corollary that this instance adds: *a
capability table in a module header is a claim about the module, and the
module is right there.* The correction is written rather than deleted because
the shape of the mistake is the useful part.

## ★★★ THE PALETTE POPUP — what the operator asked for by name

> *"Also make sure you've used the same default colours and style look for
> these things as Adobe."*

The **colours** half is [`super::palette`] and [`super::pen::Pen::default`].
The **style look** half is this module, and it is a specific, nameable change:

| before | after |
|---|---|
| `egui`'s `color_edit_button_srgba` — a generic HSV wheel with hue, saturation and value sliders | a **grid of named preset swatches**, with the full picker one click below it |

Acrobat does not open a colour wheel when you press its comment colour chip.
It shows a small grid of colour cells, each with a name, and *More colours…*
underneath for anything else. That shape is not decoration — it is what makes
the control usable at ribbon speed: an operator marking up a drawing wants
*red*, and a wheel makes them navigate to it and produces a slightly different
red every time. A grid gives them the same red as last time, and — because
every cell is a value measured out of Acrobat — the same red Acrobat would
have given them.

⇒ The full picker is kept, underneath, because *"the ten Adobe uses"* is not
*"the ten that exist"* and an operator with a company standard colour must not
be told no. It expands **in place** rather than opening a second popup: a
popup inside a popup is two dismissal rules the operator has to learn.

## Why the swatch shows the colour rather than naming it

Because the operator is choosing a colour and the only useful preview of a
colour is the colour. The chip is a filled rectangle carrying the pen's
current value; its accessible name comes from the hover text, which is why
both swatches carry one.

**The alpha channel is not offered**, and [`super::pen::Pen::set_ink`]
carries the argument: a PDF annotation's `/C` is three components, and
feeding a picker's alpha into it would be a value with nowhere to go. The
`Opaque` variant of the picker is what says so, and it is why the grid's
cells are opaque too.
