# `pdfcer-gui/dialogs/settings/forms`

## Item notes

### `fn the_slider_spans_everything_the_store_accepts`

The regression test for the silent-edit hazard every slider in this
window carries: a control whose range is narrower than what the engine
accepts rewrites a value the operator chose deliberately, the moment
they open the window to look at something else. `egui::Slider` clamps
on draw, so the narrowing would be silent and the value would be gone.
