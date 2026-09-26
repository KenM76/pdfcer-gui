# `pdfcer-gui/app/floats`

## Item notes

### `mod float_window_tests`

Floating is the one dock capability that needs **two** calls per frame:
[`egui_shell::dock::Dock::show`] for the docked panels and
[`egui_shell::dock::Dock::show_floating`] for the windows. The second
cannot live inside the first — a child viewport must be opened from the
top of the frame rather than from inside a half-composed side panel — so
an application can forget it, and the symptom is a panel that is in the
layout, reports as on screen, and is drawn nowhere.

**That is the exact class of defect this project has shipped before**:
three panels laid out, publishing correct rectangles, unreachable, with
every gate green. `crate::diag::ui_rect_visible` is the answer for a
surface that has a rect; this is the answer for one whose window was
never opened, and it is why
[`egui_shell::dock::DockFrameReport::floats_undrawn`] exists at all.

### `fn drawing_both_halves_leaves_no_float_undrawn`

Two frames, and the second is the assertion. `Dock::show` measures
`floats_undrawn` against the count `show_floating` recorded on the
PREVIOUS frame — because `show` runs first — so a genuine first frame
reports the float it is about to draw and the number settles on the
next one. A harness drives two frames anyway; the test says so
explicitly rather than leaving the one-frame lag to be discovered.

### `fn forgetting_the_float_windows_is_reported_rather_than_silent`

The falsification, written as a test rather than performed by hand:
the same fixture, the same frames, and the second call simply not
made. If this ever reports zero, the guard has stopped guarding and
the next tear-out consumer ships unreachable panels with nothing red.

### `fn a_float_window_whose_panel_says_one_sentence_is_not_reported_empty`

Two claims in one test, and the second is the one worth the words.

1. `FloatFrameReport::empty_bodies` distinguishes an open window with
   a panel in it from an open window with nothing in it. That is the
   number a check asking *"does a floated panel open an OS window and
   draw nothing inside it?"* needs and cannot otherwise get.
2. **`ui.label` measures a real rectangle here**, which is *not* true
   one crate down. `egui-shell` pins `egui` with
   `default-features = false` — its `Cargo.toml` says so, "so this
   crate does not silently acquire fonts" — and in its tests every
   galley is empty and every label is a **zero-sized** rect. The
   twin of this test over there had to be written with
   `allocate_space` for exactly that reason, and it says so at the
   call. This crate links `eframe`, which brings the default fonts,
   so the sentence an R9-correct empty panel draws is real content
   and is measured as such.

⇒ Keeping both tests, one per crate, is deliberate: the pair is what
says the measurement means the same thing in the shell's unit tests
and in the running program, which is the only place it matters.

### `fn a_float_window_with_an_empty_body_is_named_rather_than_counted_a_success`

The falsification of the test above, in the crate the operator
actually runs. Every other number the frame produces still reports
success — the window is in `drawn`, `floats_undrawn` is zero — which
is precisely why the empty case needed a number of its own.

### `fn a_layout_with_no_floats_draws_no_windows`

The common case, and the one that must stay free: an application that
has never floated a panel calls `show_floating` on every frame
forever, and it must return immediately.
