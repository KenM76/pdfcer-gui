# `pdfcer-gui/dialogs/print/tabs`

## Item notes

### `fn publish_scale_region`

`ui_rect_visible` and not `ui_rect`, because the options column scrolls: a
radio scrolled out of view must stop being published rather than hand a
driver a rectangle it would click through to whatever is on top.

The percentage inside `ScaleMode::Custom` is ignored — all four modes are
one radio each, and `Custom(0.35)` and `Custom(1.0)` are the same control.

### `fn resolution`

The limit field is drawn in EVERY state. Drawn only while the cap bound, it
vanished the moment the operator raised the cap to the printer's own
resolution — the next job had nothing to cap, so the control that would
lower it again was gone.

### `fn the_typed_order_is_preserved_and_duplicates_are_kept`

Both are *behaviours*, not accidents, and both are shared with the
CLI — which is the whole reason there is one parser. A future "tidy"
that sorted or de-duplicated here would make the same text mean two
different jobs depending on which surface the operator typed it into,
and neither surface would say which.

### `fn malformed_input_refuses_rather_than_recovering`

The property the whole "one parser" argument rests on: a range that
cannot be read must not become a job. Each of these would be a
plausible thing to "recover" from, and recovering would print pages
nobody asked for.

### `fn a_stale_current_page_selects_nothing`

Reachable rather than theoretical: the dialog holds the page index it
opened on, and a document can be closed and a shorter one opened while
it is up. Selecting *something* here would print a page that is not
the one the radio names.

### `fn the_three_tabs_are_distinguishable`

The tabs earn their keep only if their names distinguish them; three
tabs sharing a tooltip would be the drawer this design replaced,
wearing a strip of buttons.
