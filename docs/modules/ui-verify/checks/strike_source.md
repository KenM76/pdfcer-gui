# `ui-verify/checks/strike_source`

`a_strikethrough_says_where_its_line_came_from`: striking a word traces
`FormatReport::strike_source`, and the edit's disclosure carries the engine's
clause for that source and no other.

The engine writes the clause into `FormatReport::disclosures`, which
`app::actions::textstyle::format_op` passes through verbatim. The shell
composes no strikethrough sentence of its own, so what it owes is the trace:
`text-strike-source page=… source=font_table|x_height|quarter_em`, and the
gesture's notes on `text-style-disclosed … notes=…` so a check can read what
the status bar was given.

# What it drives

`fixtures/paragraph.pdf` (Helvetica 12 pt, unembedded). Caret into `drawing`
at (112, 703), Format tab (expanding the Font group if it is collapsed),
Strikethrough. Owed: `source=x_height` (no strikeout table and no `/XHeight`,
so the AFM x-height), and a `text-style-disclosed` line holding
`the strikethrough is inferred at half the font's x-height` and neither
`the strikethrough is placed by` nor `the strikethrough is guessed at`. The
engine's generic decoration sentence names all three sources in passing, so
the needles are the per-source clauses.

# Falsified

- With `format_op` dropping `report.disclosures`, the clause finding fires.
- With the source token for `XHeight` spelled as `quarter_em`, the source
  finding fires.

# Driven off-screen

Scripted pointer, window at `-4200,-4200`; it runs under `--no-input`.
