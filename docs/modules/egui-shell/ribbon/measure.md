# `egui-shell/ribbon/measure`

## Item notes

### `fn button_padding`

`pub(crate)` because the tab strip budgets buttons too — a tab, a QAT
control and a band control are all `egui::Button`s and must be measured
with the same constants, or one row's estimate disagrees with another's
for no reason a reader could find.

### `const SEPARATOR_LINE`

`egui::Separator`'s default `spacing` is 6 pt in the cross direction,
with the 1 pt rule painted down the middle of it. It is not exposed as
a constant, so it is named here rather than left as a bare literal at a
call site.

### `fn separator_width`

This is the band's inter-group figure — `[group][gap][rule][gap][group]`
— and is what [`plan::plan_band`] is handed as `separator`. It is *not*
the right number for a separator that is an item inside a group; see
[`measure_item`].

`pub(crate)` because [`super::qat`] ends with the same `ui.separator()`
and must charge itself the same figure for it.

### `fn text_width`

Uses [`egui::Color32::PLACEHOLDER`] so the galley this produces is the
**same cache entry** the widget will later ask for with its real
colour — `egui` memoizes layout jobs, and a placeholder-coloured
galley is the form it stores. Measuring therefore costs a hash lookup
rather than a second text layout.

`pub(crate)` for the reason [`button_padding`] gives: every row of the
ribbon that plans its own width must measure text the same way.
