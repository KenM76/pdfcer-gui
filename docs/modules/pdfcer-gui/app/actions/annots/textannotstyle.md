# `pdfcer-gui/app/actions/annots/textannotstyle`

## Item notes

### `fn set_text_annot_style`

# Why this is a second function beside the markup restyle, not a branch
# inside it

Because it is a second **verb over a second spec family**, and the engine's
own doc on `edit::TextAnnotStyle` is the argument:

> `MarkupStyle` reaches its annotation through
> `annot_author::spec_from_dict`, whose arms are the geometric family and
> the four text markups. **There is no `/Text` arm** … So the two verbs are
> not a split anyone chose for tidiness — they read through different
> functions because the two families are modelled by different spec types.

A single body with an `if subtype == "Text"` inside it would put the
routing decision where nothing checks it. It is a `match` instead, on
`crate::panels::properties::markup::textannot::Reach`, decided in the panel
by asking each of the two engine readers in turn — and this function is only
ever reached down one arm of it.

# The page is `0`, and that is not a defect

`set_text_annot_style` takes an `ObjId` and nothing else — the property that
puts this variant in `AnnotAction` at all — so there is no page to pass.
[`clear_note`] and the node verbs pass `0` for the identical reason, and the
funnel uses the argument for its undo label and its raster invalidation
rather than to find anything. `EditScope::Document` (the plain
[`crate::app::actions::apply::vector_edit`], not the `_on_page` twin) is right for the
same reason it is right for a note: the annotation may not be on the page
the view is showing, and a `/Popup` companion may not be on its own.

# What the trace carries, and why it is not the values

`icon_written` and `color_written` — the engine's own two booleans off
[`pdfcer_core::edit::TextAnnotStyleChange`] — plus how the appearance was
written. **Not the icon name**, and that is deliberate: the icon is
invisible in pdfcer's own picture by construction
(`annot_author::sticky_note` draws one marker for all seven), so what a
driven check needs is *did `/Name` change at all*, which no screenshot can
answer. A name in the trace would only restate what the panel already
displays.

# The undo entry it pushes

`pdfcer_core::edit::CommandKind::SetTextAnnotStyle` — the engine's own
label for this command, pushed by the verb itself rather than by the funnel,
so an undo of a restyle is one entry and names the act rather than the
appearance rewrite underneath it.

# Its disclosures

`disclosures` turns a `TextAnnotStyleChange` into the funnel's sentences: a
stamp-label fit the engine chose (`StampLabelFit::is_inference`), a look
another program drew that the restyle replaced (`appearance_was_foreign`),
and `/RC` keys a plain redraw dropped (`rich_text_dropped`). Each is an
inference a screenshot cannot show, so it is reported off-canvas.

### `fn set_annot_opacity`

`EditSession::set_annot_opacity`: any subtype, dictionary only, so it never
re-bakes and never refuses for a foreign look. The trace carries the
engine's `previous`, `current` and `clamped`.

### `fn set_marker_colour`

`EditSession::set_marker_style` for a `/Caret`, `/FileAttachment`, `/Sound`
or `/Screen`. `redraw_as_plain` is false on the first attempt; the retry
from the redraw offer sets it.

### `fn settle_offer`

A refusal `RedrawOffer::of` recognises (foreign text box, rich text box,
foreign marker) becomes `OpenDoc::redraw_offer`, carrying the refused style
so the retry repeats the same change with the redraw allowed. A later
success on the same annotation clears it. One offer per document: a refusal
on another mark replaces it, and the panel shows it only while its
annotation is selected.
