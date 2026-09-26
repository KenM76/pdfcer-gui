# `app::status::filter` — the Select popup: what a click may land on

`OPERATOR_REQUESTS.md` O17's first half. The status bar's **Select**
button, the eleven-row popup behind it, and the standing line that appears
when the operator has left nothing selectable at all.

[`crate::canvas::pick`] holds the model — the eleven classes, the
subtractive invariant, and why the filter composes with the mode as an
`AND` rather than as an override. This file holds only the surface, and the
split is the usual one: that module can be asserted about in a unit test,
while everything here has to be **driven** before it counts (R1).

## Why this is a file and not a section of [`super`]

R2's 1,500-line ceiling forced the split, and — as with
[`super::page_box`], [`super::notes`] and [`super::decline`] before it —
the forced seam turned out to be a real one. Everything else on the bar
answers *what is true about the view*: which page, what zoom, what the last
raster contained, why a command declined. Those are all **reports**.

This is the one thing on the status bar that is not a report. It changes
what the pointer does. That is a different kind of control living on a
surface full of readouts, and it earns its own file for the same reason it
earns its own position in the layout — see [`show`] on why it sits at the
left edge of the fixed cluster rather than inside the zoom group.

## What this module does NOT do

It does not persist anything. The caller compares the filter before and
after and writes it if it moved — see [`crate::app::frame`]'s status-bar
block for why that comparison lives there, and [`crate::app::pickstore`]
for why the write is immediate where the dock layout's is debounced.

## Item notes

### `fn popup`

# `CloseOnClickOutside`, not `CloseOnClick`

egui's default for a menu is to close as soon as anything in it is clicked,
which is right for a list of commands and wrong for a list of checkboxes.
Switching four classes off is one operator decision expressed as four
clicks; a popup that vanished after the first would charge a reopen for each
of the remaining three, and the reopen is the expensive part —
`FEATURES.md` records the measured cost of exactly that ritual as the
complaint this feature answers.

This is the convention every filter list in the class follows: AutoCAD's
object-snap list, Illustrator's layer locks, a browser's cookie panel. A
list you can only make one change to is a menu, not a filter.

### `fn class_icon`

Six of the eleven reuse icons the set already had and five were authored for
this popup; [`crate::icons::Icon`] carries the argument for each. A `match`
rather than a lookup table so that adding a class is a compile error here —
a row that silently drew no glyph would be the one row that looked broken.

### `fn frame`

Only this control is built, not the whole bar: the question under test is
*"does clicking this button open its popup"*, and nothing else on the bar
can change that answer.

### `fn clicking_select_opens_the_popup`

> *"I see a Select button, but this should be a menu that pops up."*

Everything that DID exist — 1,628 unit tests, 17 gates, and an offscreen
smoke launch confirming the button's published rect sat exactly where the
layout intended — observed the **button**, and the button was never the
broken part.

The defect was a second `Popup::toggle_id` beside `Popup::menu`, which
already toggles on click (`egui-0.35.0/src/containers/popup.rs:228`). Two
toggles of one flag in one frame open and close the popup before it is
drawn, which from outside is indistinguishable from a control that was
never wired up at all.

It asserts on `Popup::is_id_open` — the exact flag the two toggles were
fighting over — so a regression fails here rather than somewhere
downstream that merely reads the flag.

### `fn clicking_select_again_closes_the_popup`

The other half of a toggle, and the half a careless fix breaks: deleting
the duplicate could as easily have been deleting *the* toggle, leaving a
popup that opens and cannot be dismissed from the control that opened it.

### `fn an_idle_frame_leaves_the_popup_shut`

Without this, the test above would pass on a build where the popup was
simply always open — which is a different defect wearing the same green
tick.
