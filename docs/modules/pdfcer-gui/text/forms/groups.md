# `text::forms::groups` — every word the Field-groups section says

The copy for [`crate::panels::forms::groups`], which is the shell's route to
`EditSession::delete_field_group` and its companion
`EditSession::field_group_deletion_preview`.

## Why this is its own file rather than more lines in [`super`]

**R2.** `text/forms/mod.rs` stood at 1,294 of the 1,500-line budget, and the
seam here is the same one [`super::authoring`] and [`super::tab_order`] were
cut along: a *surface*, not a size. Every sentence in this file is about one
question — *what is a field group, and what goes when you delete one* — and
a reviewer of that wording should be reading a file that contains nothing
else. It is re-exported from [`super`], so no call site knows the split
exists.

## ★★★ The one sentence-writing problem this surface has, which no other
panel in the shell has

**A form field is invisible on a printed page, and a grouping node is
invisible even in the Forms panel's own field list.**

`AcroForm::groups` is the field-name tree's *interior*: `Personal` in
`Personal.Address.Zip`. It has no type, no value, no widget and no
rectangle — §12.7.3 gives it existence only as a link in a `/Parent` chain.
It is drawn nowhere, on any page, in any viewer. So an operator who deletes
one sees:

- the same page, pixel for pixel;
- a field list four rows shorter, if they happen to be looking at it;
- nothing else.

⇒ Every consequence of this verb is off-canvas, which makes rule 4's
*"disclosure lives off-canvas"* not a constraint here but the **entire
delivery mechanism**. If the sentence does not say what went, nothing does.

That is why the numbers in this file are three and not one:

| number | why it is not derivable from the others |
|---|---|
| **fields** | what the operator thinks of as "the things in the form" |
| **boxes** | one field may draw on three pages (§12.7.3.1's split field/widget shape), so this is not `fields` |
| **groups** | deleting `Personal` may also empty `Personal.Address`, a node the operator never named |

and why the terminal fields are named rather than merely counted: *"4
fields"* is a quantity, *"`Personal.Name`, `Personal.Address.Zip`, …"* is a
decision. `FieldGroupDeletion::terminals`' own doc comment makes the same
ruling on core's side — *"by name rather than by count"*.

## Where each sentence is read

| function | where | when |
|---|---|---|
| [`field_groups_heading`], [`field_groups_explainer`] | the section, above everything | always, while the form has a group |
| [`field_groups_refusal`] | in place of every control | the document refuses structural change (R83) |
| [`field_group_row`], [`field_group_delete_button`] | one row per grouping node | always |
| [`field_group_preview_summary`], [`field_group_preview_names`] | under the armed row | **before** the destructive press |
| [`field_group_deleted`] | the status bar's disclosure row | **after** it, from the engine's report |

## Conventions, which bind here as everywhere in [`crate::text`]

- Sentence case; full sentences with punctuation for prose, no trailing
  period on a button label.
- **Never state a capability the build does not have.** Nothing here offers
  to *rename* a group or to *move* a field out of one; the shell has neither
  route today, and a sentence implying otherwise is a promise the program
  breaks.
- **A refusal is a sentence, never a silence.** [`field_groups_refusal`]
  exists because the alternative — drawing no controls and saying nothing —
  is indistinguishable from a feature nobody built.
