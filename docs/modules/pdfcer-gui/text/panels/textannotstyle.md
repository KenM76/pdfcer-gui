# `text::panels::textannotstyle` — the words for restyling a mark that
carries WORDS

Copy for `crate::panels::properties::markup::textannot`: the sticky note's
and the stamp's style rows, which reach `EditSession::set_text_annot_style`
rather than `set_markup_style`.

## ★★ Why a file of its own beside `text::panels::properties`

**R2**, and a subject seam that survives it. `properties.rs` was at 1,487
lines — thirteen short of the ceiling — when `Pass 253.2` landed, so the
four strings below had nowhere to go in it. The cut is the same one the
code took one directory over: `properties::markup` draws what one verb
reaches and `properties::markup::textannot` draws what the other does, and
the copy follows the code rather than the file it happened to start in.
`text::panels::annotgeometry` is the precedent for a sibling here.


That function's own doc keeps the superseded sentence verbatim. What
belongs here is why it went wrong and what replaced it, because both are
about the words in this file.

It said *"pdfcer does not redraw this kind of mark, so its colour, line
width and opacity cannot be changed here"*, and it was shown for the three
subtypes `set_markup_style` refuses: `/Text`, `/FreeText`, `/Stamp`. Every
word was true **of the only annotation-style verb that existed when it was
written**. `pdfcer-core` shipped a second one the same afternoon —
`set_text_annot_style`, a different reader, a different style struct — and
it restyles a sticky note's icon and colour and a stamp's colour.

⇒ From that moment the sentence made a **false claim about two of the three
subtypes it was shown for**, and false in the direction that matters: it
told the operator a capability did not exist on the very day it did, over a
row open since 2026-09-05 carrying his own *"check that these are fully
editable while you are at it."* That is the `set_button_action` shape
`check-verb-coverage.sh` exists because of — *"if your surface tells the
operator that pdfcer never authors an action, it is now saying something
untrue in the direction that matters."*

★ **The fix was not a reworded sentence. It was fewer marks reaching one.**

| subtype | before | now |
|---|---|---|
| `/Text` | the refusal sentence | the icon and colour rows |
| `/Stamp` | the refusal sentence | the colour row |
| `/FreeText` | the refusal sentence | [`markup_text_box_not_restylable`] — a narrower claim, and still true |
| anything else | the refusal sentence | the refusal sentence, reworded to stop implying a family |

★★ Note which row is the interesting one. The `/FreeText` sentence is not
the old refusal kept for one subtype: the old one claimed the capability
was **missing**, and the new one says this shell **declines** to use a
capability that exists, for a measured reason. Reusing the string would
have kept a false premise alive under a true-looking conclusion.
