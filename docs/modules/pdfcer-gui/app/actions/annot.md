# `app::actions::annot` — the verbs whose subject is a whole annotation

Move it, resize it, remove it, write the note on it, **answer it, and
record whether its window opens**. Held here rather than in
[`super::action`] so that the action enum stays inside R2's 1,500-line
ceiling.

## Which family comes out next

[`super::action`]'s header names **markup** as its largest sub-enum and the
obvious candidate. The rule it states is *"the next family of variants to
**grow**"*, and growth is what decides: size alone does not move a family
out of that file.

## What these all share, and it is not "they are annotations"

**None of them uses a page index to FIND its operand.** Every other
authoring verb in this crate does. The reason is a property of the engine's
annotation verbs: `move_annotation`, `resize_annotation`,
`delete_annotation`, `set_markup_note` and `clear_markup_note` all find
their operand by **stable object id**, so a page number would be a second
way of naming a thing that is already named — and one that goes wrong the
moment a page is reordered between the gesture and the queue draining.

Two variants carry a page anyway and neither is an address. `Delete`'s is
for the **trace and the disclosure**. `Arrange`'s is the engine's own
operand — the `/Annots` array being permuted belongs to a page. Both
asymmetries are documented on the variant rather than smoothed away,
because smoothing them would mean handing a page to the verbs that must not
use one.

## `CommitMarkup` and `PasteMarkup` are deliberately NOT here

They **author** an annotation, which needs a page, a spec and a pen. The
verbs here act on one that exists. Authoring and editing are different
subjects
however much they share a noun, and a sub-enum drawn around the noun rather
than around the subject would be the larger of the two families and the less
useful one.
