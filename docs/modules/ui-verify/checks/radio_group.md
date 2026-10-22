# `ui-verify/checks/radio_group`

Both checks drive `fixtures/all-field-kinds.pdf` (one 420 × 620 page). Its
radio field `RadioGroup` is object 18 with `/Kids [16 0 R 17 0 R]`: member
16 at `[170 452 186 468]` turns on as `/A`, member 17 at
`[260 452 276 468]` as `/B`. Every field sits at x ≥ 170, so x 20–150 is
empty page. Neither check needs the real pointer or keyboard; both run under
`--no-input` and neither has a SKIP arm.

The saved file is judged by `checks::savedannots::last_definitions`: the
field whose last body carries `/T (RadioGroup)`, its `/Kids`, and, per kid,
its `/Parent`, `/AS`, `/Rect` and the non-`Off` name in `/AP /N`.

# `a_copied_radio_pastes_into_its_group_and_stays_exclusive`

Launched with `mode.edit,file.properties`, `PDFCER_DIAG_SAVE_PATH` and
`PDFCER_DIAG_CLIPBOARD_DIR` (Ken's OS clipboard is never written).

1. Click (178, 460); require `form-field-selected field=RadioGroup`.
2. `Ctrl+C`; require `fieldclip-copy`.
3. Hover (60, 560), press `Ctrl+Shift+V`; require `radio-paste-applied`.
4. Save a copy. Require three kids, each naming field 18 as `/Parent`, with
   three distinct on-states.
5. Re-open that copy in a second session with `mode.read`. Click member A,
   then the pasted member; require a `form-button` line for each. Save.
6. Require exactly one kid with `/AS` other than `Off`, that it is the pasted
   member (the one left of x = 100), and that the field's `/V` is its state.

Step 6 is the claim that matters: two members on at once is what a shared
on-state or a second field produces, and only a re-opened file shows it.

# `a_radio_button_joins_a_group_from_properties`

Launched with `mode.edit,file.properties,edit.form_radio_button`.

1. Drag (40, 480)→(56, 464); require `form-field-open`; accept. The new
   button is its own field.
2. Type `RadioGroup` into `properties.form_field.radio_group.name` and press
   `properties.form_field.radio_group.move`; require `radio-move-applied`.
3. Re-arm Forms ▸ Fields ▸ Radio button (placing puts the tool down) and drag
   (40, 420)→(56, 404); accept.
4. Type `RadioGroup` into `properties.form_field.rename_name` and press
   `properties.form_field.rename`; require `radio-move-applied` — a rename
   onto another radio group's name joins it, as in Acrobat.
5. Save a copy. Require four kids of field 18 with distinct on-states, and
   no field in `/Fields` the fixture did not have.

## Falsified

- Paste route disabled in `forms::paste`: the first check fails at step 3.
- Rename join disabled (`rename_joins` returns `None`): the second fails at
  step 4.
- The Move button pushing nothing: the second fails at step 2.
- `free_export` returning the copied export unchanged: the first fails at
  step 3 (the engine refuses `RadioExportValueTaken`).
- `judge_members` and `member` are unit-tested on a shared on-state and on
  a saved body.
