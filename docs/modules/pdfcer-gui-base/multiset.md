# `multiset` — the merge rules every multi-selection shares

A multi-selection here is an ordered list whose last item is the primary:
the one every single-item surface (grips, Properties, Delete) reads.
Form fields keep it in `FieldSet` (anchor and others); annotations keep it in
`SelectionState::annot` and `annot_others`. Both route their two gestures
through this module:

- `toggle(all, item, same)` — Shift- or Ctrl-click. Removes a present item,
  appends an absent one. Removing the primary promotes the most recently
  added item, because the last item is always the primary.
- `band(all, hits, merge, same)` — a marquee. `Replace` drops `all`; `Add`
  appends hits not already present; `Subtract` removes them. Existing items
  keep their order, so the primary changes only when the merge says so.

`same` is the identity: field name, widget and page for fields; the
annotation id for annotations. Rules particular to one kind stay with its
caller — annotations restart on another page, because Align acts on one.

`BandMerge` lives here; `fieldset::BandMerge` re-exports it.

Driven by `shift_and_ctrl_marquees_add_and_remove_fields`
(`ui-verify/checks/fieldbandmerge.md`); the click toggle by
`several_form_fields_align_as_one_undo_step` and
`several_annotations_align_as_one_undo_step`.
