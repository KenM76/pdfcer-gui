# `panels::properties::choiceopts` — a choice field's `/Opt` list

`FORMS_PARITY.md` §8.1 row 1, and §7 names it as **one** sibling row rather
than seven features: *add · remove · reorder · rename display ·
export ≠ display · sort · default choice*. Plus the three `/Ff` flags
Acrobat's Options tab draws beside them and this shell could not set — bit
19 `Edit`, bit 23 `DoNotSpellCheck`, bit 27 `CommitOnSelChange`.

## `/Opt` is one property, and that decides the shape of [`section`]

`FieldEdit::with_options` **replaces the whole list** — Table 230 makes
`/Opt`'s order significant, so a per-entry merge has no defined meaning.
Every operation here therefore sends the entire list, and `fieldedit`'s
one-press-one-undo-entry rule is satisfied rather than bent.

The corollary is that this module emits **at most one `FieldEdit` per
frame**, built from the typed draft. Pressing a button steals focus from a
half-typed box, so the box's commit and the button's operation arrive in the
same frame; two pushes would be two undo entries for one press, and the
second — built from the pre-rename list — would silently take the rename
back.

## Sort is one act, and both sides of the boundary now perform it

`NewChoiceField::sort` sorts `/Opt` and sets bit 20. `FieldEdit::with_sort`
sets bit 20 alone, and `edit_field` sorts only when the same edit supplies
a replacement list — a caller that hands over the whole list has no
pre-existing order to destroy. Bit 20 set on its own still reorders nothing
and is disclosed as `sort_claim_unmet`.

This panel always sends the whole list, so it is in the sorting case. It
reorders anyway, because the operator must see the order that will be
written before it is written, and it reorders through
`pdfcer_core::edit::sort_choice_options` rather than through a comparator
of its own: one exported ordering cannot disagree with the gate that checks
it. `FieldEditOutcome::options_sorted` is the tripwire for the case where
it does anyway — see [`crate::app::actions::forms`].

## The duplicate refusal, and which half of it is the shell's

Both `add_choice_field` and `edit_field` refuse a repeated export by name
(`EditError::ChoiceOptionDuplicate`): the fill verb resolves to the first
match, so the second entry would be unselectable for ever.

[`refuse_duplicate`] survives that as the **worded** half, not as a second
authority. An engine refusal reaching `vector_edit` is shown as
`Declined::EditRefused` — *"That change was refused"* — which names neither
the rule nor the value, and the operator's list is long enough that finding
the repeat unaided is the whole difficulty. The shell therefore asks first
and says which value repeated.

It asks with `pdfcer_core::edit::duplicate_choice_export`, the same
predicate `edit_field` refuses on. `G027` was filed because that predicate
was private while the ordering beside it was public, which left this panel
spelling the rule itself and agreeing by construction rather than by
contract; the engine exported it, so the only thing spelled twice now is
the sentence, which is the part that has to be in the operator's
language.

## Rule 4

Nothing here marks the canvas. Removing an option the field is set to leaves
the field showing that answer; `edit_field` returns `value_no_longer_fits`
and the status row says so. Re-pointing the selection would be inventing an
answer the operator did not give.
