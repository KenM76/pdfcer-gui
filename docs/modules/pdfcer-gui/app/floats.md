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

### `fn floating_panels`

The second of the dock's two per-frame calls. It takes an
`&egui::Context` rather than a `&mut Ui` because it opens child
viewports, and a child viewport must be opened from the top of the
frame rather than from inside a half-composed side panel — see
`egui_shell::dock::floatwin`'s header, and `crate::dialogs::host`,
which is called from the same place for the same reason.

# The body closure is the SAME ONE `docks` uses

Not a similar one — the same expression, resolving the same
`PanelId` through the same `Panel::from_command_id` and calling the
same `Panel::show`. That is the property `MODES_AND_PANELS.md`
identified as the thing that makes tear-out cheap here:
`show_viewport_immediate` takes `FnMut` with **no** `Send + Sync +
'static` bound, so a torn-out panel keeps the docked signature and
there is no second rendering path to keep in step.

A previous float-or-dock dual mode is on record as costing *"two
code paths for the same content, each duplicating open-state,
position/size and focus handling"*. This has one.

# Every rect is tagged with its own viewport

A child viewport's coordinates start at **its** origin, so an
untagged `ui-rect` from a float window reads to a harness as a
position in the application window — plausible numbers naming a
different place on the desktop, which
`D:/dev/rag/egui/a_child_viewports_ui_rects_are_relative_to_ITS_origin…`
records as a harness aiming hundreds of points away. The shell has
no diagnostic channel of its own, so the scope is entered here,
inside the body, from the very id the shell used — recovered
through `floatwin::viewport_id`, which is public for this.

# The draw-order invariant, and why it HOLDS

`D:/dev/rag/egui/moving_a_surface_into_a_child_viewport_breaks_the_draw_order_invariant_containment_gave_it_free.md`
records the hazard this change is the exact shape of: *"containment
silently provides a draw order, and moving a surface into a sibling
viewport turns that invariant into the order of your two call
sites."* A panel that writes into shared state while it draws, and
something drawn **after** it that reads that state, is ordered for
free while both are inside one window and by nothing at all once one
of them is a child viewport.

It is checked here rather than assumed, and the answer is that
**tear-out cannot change any reader's side of the fence**, because
every parent reader is ordered before *both* halves of the dock:

| `PdfcerApp::ui` line | what it does |
|---:|---|
| 132 | the ribbon's Font group takes `panels.text_style_mut()` |
| 668 | the ribbon band draws |
| 691 | `dimension_groups.take_scale_request()` |
| **831** | **`docks` — the DOCKED panel bodies** |
| 938 | the dialogs (which read no panel state) |
| **970** | **`floating_panels` — the FLOATED panel bodies** |

A panel that moves from 831 to 970 is still after 132, 668 and 691,
so a reader that saw last frame's write when the panel was docked
sees last frame's write when it is floating. The one-frame lag is
pre-existing, identical in both states, and not this capability's.

**That is a fact about the current call order, not a guarantee.**
Move any reader of panel-written state to between `docks` and here
and the two states diverge — the docked panel would be read this
frame and the floated one next frame, which presents as a control
that is correct until you tear its panel out. The status bar's layer
clause is deliberately NOT such a reader:
`app::status::selected::with_layer` recomputes through
`panels::layers::highlight::resolve(doc)`, a pure function of the
document, rather than reading anything the Layers panel cached while
drawing.
