# `panels::properties::markup::textannot` — restyling the marks that carry
WORDS

A sticky note's **icon and colour**, and a stamp's **colour** and **label
size**, on a mark that is already on the page.
`EditSession::set_text_annot_style` is the verb, and this module is its only
caller in the shell.

Without it, an operator who wants a different sticky icon — or a reviewer
who wants their comments in a different colour after the fact — has to
delete the mark and place another, losing its `/M` stamp, its object id and
any reply thread hung off it.

## Why a SECOND verb, and why the guard between them is a `match`

`pdfcer-core` has two annotation-style verbs, and the split is not a
tidiness decision anybody took. `set_markup_style` reaches its annotation
through `annot_author::spec_from_dict`, whose arms are the geometric family
and the four text markups — **there is no `/Text` arm**, and none for
`/Stamp` or `/FreeText`. The two verbs read through different functions
because the two families are modelled by different spec types.

⇒ Two readers, two spec types, two style structs, two verbs. The parent
module's [`super::section`] therefore routes on [`Reach`], an enum whose
arms the compiler makes exhaustive, and **not** on a `/Subtype` string
compared in an `if`. A `/Stamp` sent to `set_markup_style` gives the
operator live controls whose every press is refused, and the fix for a
mis-route is not a better string comparison — it is making the wrong turn
fail to compile.

## The three things this module does NOT offer, each for its own reason

Every one of these is **absent**, not greyed. R9 reserves greying for a
capability that is *temporarily* unavailable and can explain itself on
hover, and none of these three can ever become available by anything the
operator does.

1. **No Clear beside the colour swatch.** `TextAnnotStyle::color` cannot
   clear, and the engine explains that rather than merely lacking it: each
   `TextAnnotSpec` variant carries a **required** `Color` — a sticky's icon,
   a stamp's face and a free text's frame are each drawn in one — so *"no
   colour"* is not a state the authoring type can express, and a Clear here
   would have to invent a fallback, silently picking yellow for a note whose
   colour an operator asked to remove. [`super::colour_row`]'s Clear belongs
   to `MarkupStyle::stroke` and does not generalise.

2. **No opacity row.** `/CA` is `set_markup_style`'s, and that verb cannot
   reach these subtypes at all. So it is not that this verb declines the
   property — there is no route to it for a `/Text` or a `/Stamp` from any
   verb the engine publishes. Recorded here because the parent's
   [`super::opacity_row`] is right above and its absence would otherwise
   read as an oversight.

3. **No width, fill, dash or endings.** Same reason, one level up: they are
   `MarkupStyleSupport`'s properties, asked of a verb that has no arm for
   these subtypes.

## ⚠ AND A FOURTH: a `/FreeText` is not routed here, though the verb takes one

`set_text_annot_style` accepts a `/FreeText` and will restyle its frame
colour. **This shell does not send it one.** [`Reach::TextBoxWithheld`]
carries that refusal, so a selected text box gets
[`crate::text::panels::textannotstyle::markup_text_box_not_restylable`]
rather than a live control.

The hazard the refusal was written against: `annot_author::
text_spec_from_dict` returns `multiline: false` for **every** `/FreeText`,
always — §12.5.6.6 gives the subtype no multiline key, `/Ff` is a form-field
entry and a `/FreeText` is not a field — so a verb that re-baked straight
from that spec would publish every wrapped callout as a single unwrapped
line. `crate::canvas::textannot::spec` authors `multiline: true` on every
text box this shell places, so that is not an edge case: it is every pdfcer
callout, and the operator's second sentence would be pushed off the page
with nothing on screen to say so.

⚠ **The engine now measures it.** `set_text_annot_style` calls
`EditSession::measure_free_text_multiline` against the original spec before
amending it, exactly as `set_markup_note` does, so the unwrapping described
above no longer happens. The withholding here has not been re-examined
against that: treat it as an open question, not as a settled limit.

⚠ **Do not quote this module as a source about the engine.** Everything
above is a claim about verbs in another crate at a moving pin. Re-read the
verb before repeating any of it.

## Item notes

### `const MIN_LABEL_PT`

# ⚠ These are THIS SHELL's bounds. The engine has none.

`set_text_annot_style` does not clamp `font_size`, and `StampStyle` does
not either — a caller may ask for 0.1 pt or 900 pt and get it. So these two
numbers are a usability judgement made here, not a limit reported from
anywhere, and this comment exists so that nobody later quotes them as an
engine fact. Below about four points Helvetica Bold is a smudge on paper;
above about a gross the box `GrowToText` produces is wider than a letter
page, so the stamp leaves the sheet.

**And a range is a hazard, which [`size_row`] handles rather than
ignores.** A control narrower than the values its subject accepts *silently
rewrites a value the operator never touched*: open a stamp declaring 200 pt
under a spinner capped at 144, and the spinner shows 144 — then one
keystroke anywhere commits it. So the rule, here and in every other spinner
over a value this shell did not author: the range is **widened to admit
whatever the file said**, and the action is pushed only when the number
actually differs from what was read.

### `fn size_row`

# What this closes, in the operator's own words — asked TWICE

***"I STILL can't adjust the size of a stamp on the canvas, or by entering a
different size in the properties box."***

The first half of that sentence is answered elsewhere — `annots::resize`
sets `scale_stroke_width` and `allow_appearance_distortion` for a `/Stamp`
target, so the canvas grips and the Properties width/height fields scale the
picture. This row is the second half: the engine exposes a stamp's label
size for reading and for writing, and this is the consuming side of it.

# Why it is a size ROW and not another entry in the placing dialog's list

Because the two controls answer different questions. `canvas::textannot::
StampSize` offers a *list* — `Fit the box I drew`, then a ladder of stated
sizes — because at placing time the operator has no stamp to look at and a
ladder is how every tool they own presents a font size. Here the stamp
exists, its size is a **number that came out of the file**, and the act is
*change this number*. A combo box would have to invent an entry for a stamp
whose file says 17 pt.

⚠ **`Fit the box I drew` has no counterpart here, deliberately.** That
choice means *derive the size from the box*, i.e. `font_size: None`, and
`None` on this verb means *leave the size alone* — the field's contract, and
the same contract that keeps a colour change from touching the icon. The two
meanings collide, and the engine's field cannot express the first. Offering
it would be a control whose press does nothing, which is the defect this
panel already shipped once (*"live controls, every press refused"*).

# Absent, not greyed, when the stamp has no label pdfcer can describe

`stamp_label_parameters` answers `None` for a stamp whose appearance shows
no text — **Acrobat's own custom stamps are artwork**, not a laid-out
label — and the engine is explicit that this is *"the honest answer … not a
failure"*. R9: nothing is drawn. A greyed spinner would imply that a size
could appear if something were different, and for a picture of a signature
nothing can be different.

### `fn push_size`

Split out from [`size_row`] so the struct literal that names every field
of `TextAnnotStyle` sits in one place per act rather than inside a closure
three levels deep. The `None`s are the contract, not a formality: *a field
left `None` is left alone*, so a size change does not touch the colour and
cannot touch the icon.

### `fn colour_row`

# A swatch and NO Clear, unlike [`super::colour_row`]

The one structural difference between this row and the parent's, and it is
the engine's decision rather than a control left out: `TextAnnotStyle::color`
has no `Clear` arm to raise. Its doc gives the reason in full — every
`TextAnnotSpec` variant carries a **required** `Color`, so the authoring type
cannot express *no colour*, and a Clear would have to invent a fallback.

⇒ **Absent, not greyed.** R9: a greyed Clear here would imply that clearing
could become possible if something were different, and nothing can be
different — the limit is in the shape of the engine's spec type. The
sentence under the rows says so once rather than a tooltip saying it per
press.

### `fn icon_row`

# The row the request was filed for

> *the operator places a sticky and wants a different icon* → **delete it
> and place another**

That is what this row replaces, and the cost of the old answer was never the
icon: it was the object identity, the `/M` stamp and any reply thread hung
off the note, all lost to a delete-and-replace.

# A combo, where the placing dialog uses radios

Deliberately different, and the difference is the surface rather than the
choice. The dialog is a **transaction** with room to spare and one question
to ask, so seven radios read at a glance. The properties panel is a narrow
column shared with every other section, and seven rows here would push the
colour swatch and the delete control off the visible part of it. §5.8's
division of labour says the panel *carries everything*, which is an argument
for the control existing, not for it being the tallest thing on screen.

# `/Text` only, and absent otherwise

[`Face::takes_icon`]. A `/Stamp`'s face comes from Table 181's own
vocabulary and the engine refuses a `StickyIcon` on one **by name** rather
than ignoring it — `EditError::StylePropertyNotApplicable`, raised before
anything is written. Silently dropping an inapplicable property is the
swallowed-argument failure mode: the caller is told the act succeeded and
the document does not carry what was asked for.
