# `canvas::forms::choosing` — picking an option **where the field is drawn**

The choice half of [`super`]: a `/Ch` combo box or list box, filled by
clicking it on the page rather than by finding its row in the side panel.
`FORMS_PARITY.md` §8.1 row 2.

## Contract

Two entry points, both called from [`super`]:

* [`focus_choice`] takes a click on a choice widget — it stores the
  [`super::Focus`] and opens the list, and is [`super::focus_button`]'s
  twin.
* [`choose`] is the third arm of [`super::editor`]: a choice field holds no
  caret and toggles no state, so it holds a **focus ring plus an anchored
  option list**, and reads Space, Enter, the arrow keys and Escape.

Every outcome leaves as [`FormEdit::SetChoice`] on `actions` — the panel's
own command, carrying **export** values, reaching `set_choice_value`. There
is no fill path here.

## Why the list is a separate open/closed state from the focus

Choosing a value must **not** end the ring. O204's complaint is that Tab
inside a form escapes to the ribbon, and dropping focus on a pick would
reproduce it from the commonest gesture on the surface: pick a country,
press Tab, land in the ribbon. So a pick closes the *list* and keeps the
*focus*, which is also what every program that fills forms does.

That needs a second bit of state, and it lives in `egui` memory keyed on
the editor id rather than on [`super::Focus`], because nothing outside this
module has any use for it — [`super::commit`], [`super::tabbing`] and the
panel's `live_draft` mirror all ask about a *draft*, and a choice field
keeps none.

A Tab arrival deliberately leaves the list **closed**: tabbing through a
form would otherwise spray open dropdowns over the sheet, and no program
behaves that way. Space, Enter or either vertical arrow opens it.

## The popup is constrained to the half-plane it chose, not to the screen

A page-anchored popup constrained to the viewport slides **back over its own
anchor** when it does not fit — and then it takes that anchor's clicks,
because it is the topmost layer. `D:/dev/rag/egui/` carries the general
finding; here it would mean an option list covering the very box the
operator is trying to fill.

So the side is chosen first (below when there is room, above otherwise) and
the constraint rectangle is the half-plane on that side of the widget. An
overlap is then arithmetically impossible rather than merely unlikely, and
the scroll height is taken from the room actually available on the chosen
side.

## What it does not draw

No facsimile of the widget, no in-place list, and nothing at all over an
**unfocused** choice field — rule 4's one-line test, exactly as [`super`]'s
§3 applies it to the text editor. A closed list leaves the page's own
appearance stream showing, which is the value the document holds.
