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

# It has a disclosure list, and what is in it

Nothing is quietly *lost* here: unlike `set_markup_style` this verb
re-bakes from a spec the same reader hands the authoring path, so there is
no `dropped` catalogue to render.

What it can do instead is **decide**. Changing a stamp's label size means a
re-bake at a new size, and that can widen the box, shrink the words, or cut
characters off the ends. Every one of those is an inference, and the ones a
screenshot cannot show owe a sentence off-canvas under R8b rule 4. So the
funnel's `Vec<String>` is not empty, and what goes in it is decided below
on `TextAnnotStyleChange::stamp_label_fit`.

**A "the engine cannot do this" sentence in a shell comment is a claim with
a shelf life and no gate.** Nothing in this build fails when the engine
grows the capability, so every such sentence here is re-checked against the
engine rather than trusted.
