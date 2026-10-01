# `egui-shell/dock/tabs`

## Item notes

### `fn text_width`

Uses [`egui::Color32::PLACEHOLDER`] so the galley produced here is the
**same cache entry** the widget will later ask for with its real
colour — `egui` memoizes layout jobs, and a placeholder-coloured
galley is the form it stores. Measuring therefore costs a hash lookup
rather than a second text layout.

Deliberately a local six-line function rather than a call into the
ribbon's identical helper. The two surfaces are independently
refactorable, and a shared private helper between them would make the
dock's width arithmetic break when the ribbon's file layout changed —
a coupling with no upside, since the body of the function is the
documentation.

### `fn a_stack_of_nine_panels_keeps_every_one_reachable`

Capping a stack at two panes is the cheap way to dodge an engine
that hides overflowing tabs behind scroll arrows, and a test on the
cap asserts a proxy rather than the property. This asserts the
property itself: at a width that cannot show nine tabs, the
affordance exists — so nothing is stranded.

### `fn the_overflow_affordance_is_drawn_inside_its_tab_bar`

The unit tests in [`super::super::plan`] prove the arithmetic; this
proves the *drawing* obeys it. Both are needed, because the failure
mode describes a control that is computed correctly and placed where
nobody can click it.

### `struct Harness`

The `egui::Context` is kept across frames deliberately: `egui`
interacts against the **previous** frame's widget rects and keeps a
popup's open state in memory, so a click is a two-frame event and a
menu is a three-frame one. A per-frame context would make every
interaction test silently do nothing.

### `fn click`

**Two frames, and the first one is not optional.** `egui`
resolves a press against the hit test it computed *before* the
frame ran, from the pointer position it had then — so a
pointer that arrives and presses in the same pass presses on
whatever was under its *previous* position, and a widget that
has just appeared (a menu row, say) is never hovered and never
clicked. Measured here: with move-and-press in one frame the
menu row reported `contains_pointer = true` and
`hovered = false`, the click went to the layer underneath, and
the menu closed as "clicked outside" — a test that would have
concluded the built-in Close was broken.

So: one frame to move, one to click. This is a property of
driving `egui` from synthetic input, not of the dock.

### `fn popup_rect`

A context menu is an `Area` at [`egui::Order::Foreground`], so
the top foreground layer *is* the menu. Read from memory rather
than from the dock's rect sink because the popup is `egui`'s
surface, not the dock's — the dock never learns where it went.

### `fn a_supplied_handler_is_offered_every_drawn_tab_with_its_own_panel_id`

The seam's central claim. If the handler were offered the wrong
`PanelId` — the stack's active one, say, or the last one drawn —
every application menu would act on the wrong panel while looking
completely correct, because the popup would still appear under the
pointer. That is the defect this test exists to make impossible.

### `fn a_tab_hidden_behind_the_overflow_affordance_is_not_offered`

Stated as a test rather than left implicit: an application that
counted handler calls to enumerate its panels would be wrong, and
the honest answer — ask the layout, not the seam — is the same one
[`DockFrameReport::panels_drawn`]'s documentation gives.

### `fn a_handler_that_requests_a_close_goes_through_the_intent_queue`

`request_close` is what makes the seam compatible with the intent
model instead of a hole in it: the panel is gone, the frame report
names it, and `layout_changed` is set, so an application that
persists on that flag persists this close exactly as it persists a
splitter drag.

### `fn the_built_in_close_still_closes_a_panel_with_no_handler`

The compatibility guarantee, driven end to end through real pointer
input rather than asserted about the code: secondary-click the tab,
the dock's own menu opens, click its one row, the panel is gone.
Any consumer that has not adopted the seam must see exactly this,
which is why the seam landing must not be able to break it silently.

### `fn a_supplied_handler_takes_the_secondary_click_away_from_the_dock`

A handler that attaches nothing means a right-click that does
nothing — the dock must not "helpfully" fall back to its own menu,
because a fallback is exactly the second writer of
`response.id.with("popup")` that the seam exists to avoid, and it
would arrive on whichever frame the application's own menu declined
to open. The contrast with the previous test is the whole point:
same input, same tab, opposite outcome, decided by one builder call.

### `fn a_tab_announces_its_purpose_even_when_a_handler_owns_its_menu`

Observed through `egui`'s own output events: a clicked widget emits
`OutputEvent::Clicked` carrying the `WidgetInfo` the widget
published, which is the same value that fills an accesskit node.
The name is the panel's **purpose** — its tooltip — per
`crate::ribbon::a11y`'s convention and this module's header.

### `fn tab_bar`

`rect` is the whole bar. The affordance's reservation is taken from
its right edge; see the module header for why that subtraction comes
first.

The bar is `surface` with an outline baseline; each tab is frameless with
`tabshape::body` behind it, so the selected tab joins the `panel` body below.
