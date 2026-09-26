# `panels::forms::rows` — one field, one row

The per-field half of the Forms panel: what a text field, a check box, a
radio group, a choice list and a rich-text field each look like, and which
[`FormEdit`] each of them can raise.

The form-wide half — the disclosures, the recompute and reset sections, the
two whole-form buttons — is in [`super`], and the split is by *scope*
rather than by size: a control that acts on the whole form and a control
that acts on one field answer to different rules about placement, about
disclosure and about when they may be offered at all.

## Every unfillable row is DISABLED AND EXPLAINED, never hidden

`RIBBON_IA.md` R83. An operator scrolling past a signature field should see
that pdfcer knows it is there; a row that vanishes teaches nothing, while a
disabled one with a sentence beside it teaches what would enable it.

The reason is asked in a fixed order — see [`block_reason`] — because a
field can be blocked several ways at once and the most specific answer is
the useful one.

## A check box with no ON state is disabled, never offered and refused

`pdfcer_core::forms::Widget::on_states` lists the button on-state names a
widget's `/AP` `/N` subdictionary defines, **excluding `Off`** (§12.7.4.2.3).
Two rules follow from that exclusion:

- **Never ask `on_states` whether an `Off` appearance exists.** A predicate
  of the shape `on || widgets.any(|w| w.on_states.contains("Off"))` reduces
  to `on`, because the right-hand disjunct is false by construction — it
  compiles, it reads like a real check, and it answers the wrong half of the
  clicks. The fact itself lives on `Widget::has_off_appearance`, which core
  keeps deliberately separate for exactly this reason.
- An **empty** `on_states` means there is no ON state at all.
  `EditSession::set_button_state` refuses any name but `Off` that no widget
  defines — `EditError::FieldStateUnknown` — so the box is drawn disabled
  with [`crate::text::forms::form_field_no_on_state_note`] beside it. R83
  rather than a disclosure after the fact: the control that would always
  error is not offered at all.

## Deliberately absent: field creation, deletion, renaming, widget moving

A Rename editor, a per-widget Delete, a whole-field Delete and a
grouping-node roster are `Edit ▸ Forms` **authoring** commands
(`edit.form_create_field`, `edit.form_manage_fields`). They answer to core's
*structural* certification gate rather than the fill gate, and a reader that
fills a form does not create fields in it. They land with the commands that
name them.
