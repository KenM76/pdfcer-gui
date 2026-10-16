# `ui-verify/checks/fieldbandmerge`

`shift_and_ctrl_marquees_add_and_remove_fields`, on
`fixtures/all-field-kinds.pdf` in Edit, reusing `field_multiselect`'s fixture
points and its window band around CheckOne and both radio members.

PASS needs, in order:

1. A click on TextOne selects it.
2. A Shift-marquee over the band traces `caught=3 merge=add`; a Shift-click
   on ComboOne then reports `before=4`. A band that replaced reads 3.
3. A Ctrl-marquee over the same band traces `caught=3 merge=subtract`; a
   Shift-click on ComboOne (which removes it) reports `before=2`. A band that
   kept its catch reads 5.

The selection count is read from the toggle's `before=`, the only trace that
states it.

## Falsified

- `Add` clearing like `Replace`: FAIL.
- `Subtract` doing nothing: FAIL.

## What it does not prove

- Annotation marquees with modifiers; they go through the same
  `multiset::band`, after a filter to the band's page.
- Crossing bands with modifiers.
