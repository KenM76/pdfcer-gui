# `pdfcer-gui/panels/properties/markup/rows`

**One function per markup property row.** Each takes the value the
parent already read and offers exactly one edit.

# Why this is its own file


What the parent keeps is the judgement: `section` decides there is a
selection worth a panel, `markup_rows` reduces the annotation to a
[`Current`] and asks [`MarkupStyleSupport`] which of these rows the
subtype even has. **A row that should not exist is never called** — it is
not called and then disabled, which is R9 (`no placeholders`), and it is
why none of these functions takes a `support` argument.

# The contract every row in this file honours

1. **It pushes an [`Action`], never an edit.** Nothing here touches an
   `EditSession`. The action goes through
   `app::actions::funnel::vector_edit`, which is the single place an
   engine refusal becomes a sentence the operator reads.
2. **A `Set` and a `Clear` are different writes and both are offered**
   when the key is present. [`StyleEdit::Set`] writes the key;
   [`StyleEdit::Clear`] removes it and restores the standard's default.
   A control that could only `Set` makes the key a one-way door, and the
   difference shows in another viewer even when it does not show here.
   `colour_row` carries the full argument; the others cite it.
3. **Clear is absent, not greyed, when there is nothing to clear.** Same
   rule, same reason — greying is reserved for *temporarily*
   unavailable.
4. **It reads `super`'s vocabulary and defines none of its own.**
   [`Current`], [`MIN_WIDTH_PT`], [`MAX_WIDTH_PT`] and [`DASH_WIDTH`] all
   live in the parent. The child reaching up is what keeps one owner for
   each; a constant copied down here would be the start of two answers to
   the same question.

# ⚠ `swatch_of` is NOT here, and it looks like it should be

It converts an annotation's `/C` or `/IC` into something a swatch can
show and reports whether that cost a narrowing. Its callers are the
parent's gathering `match` and [`super::textannot`] — neither is a row.
It is a colour-space conversion, and a file about controls is the wrong
home for one.

## Item notes

### `fn colour_row`

A **swatch plus a Clear**, not a swatch alone. `StyleEdit` has two arms
and they mean different things in the file: `Set` writes `/C`, and `Clear`
removes it, restoring the standard's default. A control that could only set
would make `/C` a one-way door — once an operator gave a mark a colour there
would be no way back to the file's own, and the difference is visible in
another viewer even when it is not visible here.

### `fn fill_row`

`MarkupStyle::interior` shipped in the engine with `set_markup_style` and had
**zero GUI callers** until 2026-09-06. The module header carries the argument
that kept it that way, and carries the correction beside it; the short form
is that *"a filled comment shape hides the drawing it is a comment about"* is
a sound reason to author `interior: None` and not a reason to refuse an
operator the ability to fill a shape they have already placed.

`canvas::markup::spec` is untouched by this. **No fill at author time, fill
available on restyle.**

# Absent for a shape with no interior — and the ENGINE says which

A `/Line`, an `/Ink`, a `/PolyLine` and a text markup have no interior for
`/IC` to mean anything in, so a Fill control there would be drawn, live, and
dropped on the floor. That is the same defect this session came to fix, one
control down, and R9's answer is the same: the row is absent, the way
[`width_row`] is absent for a highlight.

⚠ **Corrected 2026-09-06.** The paragraph above used to justify the list
with *"`apply_markup_style` does not read `style.interior` on those arms"* —
a fact about the engine's source, restated here, where nothing checks it.
The list is now asked for: `MarkupStyleSupport::takes_interior`, through
[`Current::offers_fill`]. The old sentence was not wrong; it was a copy, and
a copy is what this project filed a request to be rid of.

# The swatch shape mirrors [`colour_row`] exactly, including the Clear

Set writes `/IC`; Clear removes it and the shape is unfilled again. The one
addition is the word beside the swatch when there is no fill — a swatch
cannot show *absence*, and a black square next to "Fill" says the opposite of
the truth. See [`t::markup_fill_none`].

### `fn dash_row`

# Why this exists, and what it took to make it SAFE

`RIBBON_IA.md` §5.8's Markup row lists eight controls and this was the
eighth. It read **⛔ no engine verb exists**, and that was true: `MarkupStyle`
carried colour, interior, width, opacity and endings and had no dash field at
all, so there was nothing for a control to reach.


* `/BS` `/S` and `/D` are read back on the way IN, so a restyle that does
  not mention the dash **preserves** it — including a dash pdfcer never
  authored;
* `MarkupOptions::dash` authors one, so a shape can be *drawn* dashed rather
  than drawn and then corrected.

⇒ Before that, a dashed mark in the operator's file was silently converted to
a solid one the first time anything re-baked its appearance. The engine's
reply records that this was **wider than this shell reported**: the recolour
path was named, and `resize_annotation`, `reshape_annotation` and authoring
solidified a dash too — so it was reachable by dragging a resize handle or a
vertex, not only by pressing the colour swatch. All four carry it now. A
Line style control over the old engine would have been a control its
neighbours undid.

# The "way back to the default" is an ENTRY, not a Clear button

[`colour_row`] and [`fill_row`] each put `StyleEdit::Clear` behind its own
button, because in both cases the cleared state is *the absence of a key* and
has no name in a list: a swatch cannot show *no colour*. A border's cleared
state does have a name. `Clear` makes it **solid**, solid is Table 166's own
`/S` default, and *Solid* is the chooser's first entry — so a separate button
would be a second spelling of one act, and the two would eventually be
pressed expecting different things.

It is also why the button's absence rule does not apply. A Clear beside a
mark with nothing to clear is *"a control whose only possible effect is an
undo entry the operator did not earn"*; a **Solid** entry beside a mark that
is already solid is simply the entry that is currently selected, and
[`crate::canvas::markup::linestyle::chooser`] reports nothing when the
current entry is picked again.

# Absent for a subtype with no border

[`Current::offers_dash`], which is `MarkupStyleSupport::takes_border` and
nothing else — a highlight is a colour wash and has no `/BS` to dash. R9, and
the same answer [`width_row`] gives.

### `fn width_row`

⚠ **This moves `/Rect` for every subtype except `Square` and `Circle`**, and
the engine says so in its own doc: the rectangle is derived from the
geometry plus a margin that contains the stroke and any arrowheads, so a
wider pen needs a bigger box. That is disclosed in [`t::markup_note`]
rather than here, because it is true of the section and not of this control
alone.

### `fn endings_row`

# Two controls, ONE dictionary property — and why that is not a breach of
this module's "one field per action" rule

The rule at the top of this file forbids assembling a whole `MarkupStyle`
from what the widgets happen to show, because two controls read a frame apart
will disagree and the later action will silently revert the earlier one.
`/LE` is a **single two-element array** (Table 176) and
`MarkupStyle::endings` is a single `Option<(LineEnding, LineEnding)>`, so
changing one end necessarily sends both — there is no field that carries half
of it.

That is still one field of one property, and it is still safe, for the
reason the rule actually rests on: the unchanged half comes from
[`Current::endings`], which was read **from the session this frame** through
`spec_from_dict`. It is not a widget's remembered value and cannot be stale.
The failure the rule prevents needs a second control holding a copy of a
value it set earlier, and neither of these two holds anything.


`MarkupStyleSupport::takes_endings` is `true` for `/Line` and for nothing
else, and [`Current::offers_endings`] is what asks it. A chooser on a
polygon would be live and would be **refused** by `set_markup_style` with
`EditError::StylePropertyNotApplicable` — so R9 says absent and the engine
agrees in writing.

The `MarkupSpec` arm supplies the **pair**, because that is a value; it does
not decide whether the control exists. Deriving existence from the arm would
be a restatement of the engine's list inside a shell.

### `fn ending_chooser`

The list is [`ALL_ENDINGS`] rather than a literal written at each call
site, so the two choosers cannot come to offer different sets — and so that
an ending the engine learns to draw appears in both by editing one constant
whose exhaustiveness the compiler checks.

### `const ALL_ENDINGS`

`annot_author::LineEnding` has no `ALL` of its own — unlike `ArrowForm`,
which the ce-dimension panel iterates — so this list is written here, and
`the_ending_list_covers_every_variant_the_engine_has` is what stops it
drifting: that test `match`es an exhaustive set of variants with **no
wildcard**, so an ending the engine gains fails to compile here rather than
quietly going missing from the chooser.

`None` first because it is the state an operator reaches for when they want
a plain line, and because Table 176 lists it first.

### `fn opacity_row`

**This is the control `NO_SURFACE.md` recorded as "blocked on the engine"
for weeks, and the blocker was false.** `set_markup_style` has taken an
opacity since it shipped and writes `/CA` clamped to `0.0..=1.0`; the row
that said otherwise was a claim about a repository this project does not
build, and it could not fail a test. See `NO_SURFACE.md` §1b.

Shown as a **percentage**, because that is the unit every other application
an operator has used states opacity in, and `/CA`'s own `0.0..=1.0` is a
file-format detail they should never meet.
