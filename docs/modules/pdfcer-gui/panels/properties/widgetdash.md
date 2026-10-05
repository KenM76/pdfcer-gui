# `panels::properties::widgetdash` — a dashed widget border's pattern

One row under the border style in a field's Appearance section, drawn only
while the selected widget's border is `Dashed`: the engine draws `/BS /D` on
that style alone, so a pattern row on any other style would change nothing
the operator can see (R9).

- **Offered:** the three dashes of `linestyle::LineStyle` (Dashed, Long dash,
  Dash-dot). Solid is the style row's choice, not this one's.
- **Shown:** what the file says, read off the widget dictionary by
  `linestyle::read`, because `forms::Widget` carries no dash (G126). A pattern
  pdfcer has no name for reads as the foreign label and selects nothing.
- **A pick:** `FieldAction::EditWidget` with `WidgetEdit::with_border_dash`
  and `ForeignAppearance::Replace`, so another program's artwork is redrawn
  with the new pattern rather than kept.
- **Trace:** `widget-dash-shown field= widget= dash=` with
  `solid|dashed|long-dash|dash-dot|foreign`; the picker's rect is
  `properties.widget_edit.dash`, entry `i` is `.dash.{i}`.

Not offered at creation (`WidgetChrome::with_border_dash`): the new-field
window authors solid borders only.

Driven by `a_widget_borders_dash_can_be_chosen`.
