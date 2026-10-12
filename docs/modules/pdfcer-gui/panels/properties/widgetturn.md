# `panels::properties::widgetturn` — the Properties rows for a widget's turn and opacity

# Contract

- `rotation_row`: the quarter-turn readout, *Turn left* (+90) and *Turn right*
  (−90) through `RotateWidget`, and while the box is tilted the tilt readout
  and *Stand upright* (`TurnWidget{0}`). While tilted, the quarter-turn buttons
  are greyed with the reason on hover, because `rotate_widget` refuses a
  turned box.
- `opacity_row`: `/CA` as 0–100 %, committed on drag stop or focus loss
  through `EditWidget` `with_opacity`; *Clear* sends `clearing_opacity`.
- Regions: `properties.widget_edit.rotation`, `.rotate_left`, `.rotate_right`
  (declared in `widgetedit`), `.upright` and `.opacity`.

# Why the quarter turns stay

A tilt is stored only in the appearance matrix, which a regenerating viewer
discards; `/MK /R` survives. The panel keeps both, and the canvas drag writes
`/MK /R` when it lands on a quarter turn.
