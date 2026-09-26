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

## ★★★ Why a SECOND verb, and why the guard between them is a `match`

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

## ★★★ The three things this module does NOT offer, each for its own reason

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
