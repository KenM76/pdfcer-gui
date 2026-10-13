# `ui-verify/checks/filter_rows`

Two checks of the Select filter's **Form fields** (row 8) and **Characters**
(row 10). The filter persists across launches, so each sets it to All before
switching its row off and leaves it at All.

## `form_fields_row_lets_clicks_through_fields`

`fixtures/three-text-fields.pdf` (300×260), launched in Edit.

1. Popup ▸ All, then popup ▸ row 8. Requires `pick-filter-toggled
   class=form-field on=false`.
2. Click (165, 237), inside FieldOne. FAIL on a `form-field-selected` naming
   a field.
3. Switch to Read, click again. FAIL on any `form-focus`.
4. Popup ▸ All, click again. FAIL when no `form-focus` follows: the click
   never reached the field and steps 2–3 proved nothing.

## `characters_row_stops_text_sweeps`

`fixtures/word-fragmented-lines.pdf` (612×792), launched in Read.

1. Popup ▸ All, then popup ▸ row 10. Requires `class=characters on=false`.
2. The last `tool-status` line must contain `selects no text`.
3. Drag (74, 704) → (160, 704) along the first line. FAIL on a
   `canvas-text-selection` with `chars>0`.
4. Popup ▸ All. FAIL if the last `tool-status` line still says `selects no
   text`.
5. The same drag. FAIL when it selects nothing: the drag never reached the
   text and step 3 proved nothing.

The off sweep runs first on purpose: `canvas-text-selection` is traced only
when it changes, so a second identical sweep would trace nothing whether or
not it selected.

## Falsified

Each mutation run alone:

- The pre-change build (control exe) → both FAIL at their off step: the Edit
  click selected FieldOne; the drag selected 15 characters.
- `Press::sweeps_text` forced true in `canvas::pressing::look` → the
  Characters check FAILs at step 3 (15 characters).
- The `!fields` early return in `canvas::forms::overlay` disabled → the Form
  fields check FAILs at step 2.
- The tool status line's `sweeps_text` branch forced true → the Characters
  check FAILs at step 2.

## What it does not prove

Right-click and text-edit routing on a field with Form fields off, and a
picture under text taking a Read click with Characters off, are wired and not
driven.
