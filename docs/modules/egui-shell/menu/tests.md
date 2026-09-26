# `egui-shell/menu/tests`

## Item notes

### `fn right_clicking_something_with_nothing_to_offer_opens_no_menu`

Not "opens and closes", not "flashes", not "shows three greyed rows" —
nothing. The menu here is the real one; the only difference from the
test below is that no condition is set, so every command's predicate is
false.

This is the assertion that cannot be made in [`super::plan`], because
it is about `egui`'s popup memory rather than about a list of slots.

### `fn right_clicking_something_with_one_enabled_command_opens_a_menu`

Together with the test above this pins that the difference is the
*offer* and nothing else — same document, same registry, same
right-click, one condition flipped.

### `fn an_open_menu_closes_when_its_offer_evaporates`

The half of decision 1 that is easy to leave out. `egui` remembers a
popup as open in memory, not by whether anyone drew it — so a renderer
that merely *stopped drawing* would leave the menu ready to reappear at
the old pointer position the moment the offer came back, with no
right-click behind it.

The scenario is ordinary: the menu is open, and the selection is
deleted by a keystroke.

### `fn a_disabled_command_is_drawn_and_an_unregistered_one_is_not`

The rendered half of [`super::plan`]'s rule 1. `edit.paste` is
registered and its predicate is false, so it is a row; `edit.rasterize`
is not registered at all, so there is no row — and no gap where one
would have been.

### `fn only_a_painted_icon_slot_publishes_a_rectangle`

The only signal a *driven* check has on this surface, and the reason it
exists is in [`super::report`]'s header: a menu row is justified to the
body width, so it measures the same whether its slot holds a glyph,
holds a blank, or does not exist. The QAT's trick — an icon-only
control is square and a text button is a word wide — has no menu
equivalent, so without this name a harness cannot tell an application
that wired an icon painter from one that did not.

The fixture is the mixed case on purpose: `edit.cut` has a key and the
other three do not, so the menu reserves a column and three rows draw a
blank into it. Publishing four names here would make the signal mean
"a slot was reserved" — true of a blank — and it would be worthless.

The containment assertion is the second half: the published rectangle
has to be *inside the row it belongs to*, or a check that clicked or
sampled it would be aiming at somewhere else on the menu.

### `fn choosing_a_row_reports_its_token`

The seam, end to end: the shell hands back
[`crate::commands::HandlerToken`] and the application dispatches. There
is no handler anywhere in this crate for a test to have to stub.

### `fn a_menu_whose_only_offer_is_a_custom_row_opens`

The shell cannot evaluate an application's own control. Refusing to
open would silently delete a control the application asked for, which
is a worse failure than opening a menu that turns out to be useless.

### `fn a_chosen_row_announces_its_label_and_its_chord`

[`super::a11y`] argues from `egui` 0.35's source that the chord has
nowhere else to go. This is the end-to-end proof: click Copy and read
the `OutputEvent` `egui` actually emitted.

If this ever fails while `super::a11y`'s unit tests pass, the wiring
broke rather than the rule — which is exactly the distinction worth
being able to make.

### `fn a_customization_layer_changes_what_the_right_click_offers`

The claim `SHELL_FRAMEWORK.md` §1 makes for the whole design — *"the
shell is data … not code that has to be recompiled to change"* —
exercised through the surface rather than through the document: a RON
layer an operator could have typed, applied with
[`Menus::overlay`], and then right-clicked.

### `fn show_draws_into_a_caller_owned_ui_and_declines_to_draw_an_empty_menu`

Its documented weakness is asserted here too: it cannot decline to
open, because by the time it runs the popup exists. What it *can* do is
draw nothing, which is what an application that skips
[`Menu::would_open`] gets.

### `fn announced`

# Why the *last* one

One click produces **two** `OutputEvent::Clicked`. `egui::Button`
publishes its own default info from inside `atom_ui` — the atoms
flattened into text, which for a menu row reads `"Copy Ctrl+C"` —
and the shell then publishes the real one
([`super::a11y::describe_item`]) immediately afterwards.

The shell's is the one that counts, and not only by being second:
`Response::widget_info` also calls `register_widget_info`, whose
later value **replaces** the earlier one, so the accesskit node an
assistive technology actually reads carries the shell's name. The
duplicated output *event* is `egui`'s behaviour for any widget that
refines its own info; the ribbon's band has it too.

### `fn open_menu`

# Why the settle frame is not optional

An `egui::Area` that has never been shown runs a **sizing pass**: it is
laid out invisibly, at a provisional size, purely to measure its
content (`egui-0.35.0/src/containers/area.rs`, `Area::begin`). Two
consequences bite a test that skips it:

- the row rectangles reported on the opening frame are the sizing
  pass's, so a click aimed at one of them lands somewhere else and
  nothing is invoked;
- `cross_justify` is switched off during a sizing pass
  (`egui-0.35.0/src/ui.rs`), so the rows are at their intrinsic widths
  and the chord column is not yet where it will end up.

This is not a defect in the menu; it is how `egui` sizes any
auto-sizing area, and the operator never sees the pass because it is
painted invisibly. But a test that asserted against it would be
asserting against a frame that is never shown to anybody.
