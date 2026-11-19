# `ui-verify/checks/form_widgets`

`form_preview_fields_open_as_their_widgets` — in Form preview a check box
turns on with one click, and a choice list opens as its choices (a drop-down
below the field, a list always shown over it) and writes the one picked, and
a list that takes several keeps every Ctrl+click pick through the fill, the
redraw and Use these pages.

# What it drives

A copy of `fixtures/xfa-widgets.pdf`, opened off-screen under `--no-input` by
`form_pages::launch_fixture`. Its fields, in page order: Agree (check box,
data `N`), Pets (check box), the Plan radio pair, City (drop-down holding
Ottawa), Colour (list always shown holding `g`, Green), Note (multi-line),
Towns (`multiSelect`, Toronto / New York / Boston, holding nothing).

1. Click `form-preview.field.0.0` (Agree). Require `form-preview-pick` with
   `kind=check state=on`, then `form-fill-applied`, then a `form-preview`
   redraw whose `digest=` differs from the one before.
2. Click `form-preview.field.0.4` (City). Require `form-preview-list` with
   `kind=drop-down items=3`. Click `form-preview.item.0` (Toronto); require
   `form-preview-pick item=0`, the fill and a new digest.
3. The same for `form-preview.field.0.5` (Colour) with `kind=list items=3`,
   picking Red.
4. Click `form-preview.field.0.7` (Towns); require `form-preview-list
   kind=multi items=3 ticked=none`. Ctrl+click `form-preview.item.1` and
   `form-preview.item.2`, each requiring `form-preview-toggle item=k
   state=on`. Press Enter; require `form-preview-pick kind=multi item=1,2`,
   the fill and a new digest.
5. Reopen Towns; require `ticked=1,2`, read from the form's data after the
   redraw. Press Enter (nothing changed, nothing written).
6. Press Use these pages (`form_pages::press_use`), then require a
   `form-box kind=choice` line naming Towns with `selected=2`.

# What would make it fail

- Ignoring `field_widget`, so every field opens as text, gives `clicking
  form-preview.field.0.0 traced no form-preview-pick line`.
- Ignoring `ChoiceOpen` gives `must carry kind=drop-down items=3` or `kind=list
  items=3`.
- A Ctrl+click that replaces the picks instead of adding to them gives `must
  carry kind=multi item=1,2`.
- Writing the picks space-separated gives `ticked=2` at step 5: the engine
  reads a one-line value that is no save value as words, so `New York` is lost.
