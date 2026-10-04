# `panels::properties::buttonicon` — a push button's picture and caption position

Drawn by `widgetedit` under the caption row, for a push button only
(`is_push_button`: `FieldType::Button` and `ButtonKind::Push`). Any other
field kind renders nothing: the engine refuses an icon there
(`EditError::NotAPushButton`), and a disabled row would be a placeholder.

| Row | Shown | Raises |
|---|---|---|
| Picture: has a picture / none | always | — |
| *Choose picture…* / *Replace picture…* | always | `FieldAction::PickButtonIcon`, whose picker runs in the apply phase (`actions/buttonicon.md`) |
| *Remove picture* | with a picture | `EditWidget` with `without_button_icon()` |
| Caption combo, `/TP` 0–6 in Table 189's order | with a picture | `EditWidget` with `with_caption_position(choice)` |

The caption combo is hidden without a picture because with none the button
draws its caption alone whatever `/TP` says. Every edit goes through
`push_edit`, which sets `replace_foreign_appearance` (see
`actions/buttonicon.md`).

Trace regions: `properties.widget_edit.button_icon` (the rows), `.choose`,
`.remove`, `.position`, and each open combo entry `.position.<tp>`.
`button-icon-shown field= widget= icon=present|absent position=<tp>` is
written once per change of what the panel shows.

Driven: `a_push_button_takes_a_picture`.
