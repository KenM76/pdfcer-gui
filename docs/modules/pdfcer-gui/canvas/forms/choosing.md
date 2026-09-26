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

## Item notes

### `const LIST_MAX_H`

`/Opt` is unbounded — a country list is two hundred entries — so the list
scrolls rather than growing to fit, and this is the height it scrolls
within. Above it the popup stops looking like a dropdown and starts looking
like a page of its own.

### `const LIST_MIN_W`

A choice widget may be 30 pt wide on the sheet and its options may be
words. The list is at least this wide whatever the box measures, which is
the same trade `boxes::MIN_EDITOR` makes for the text editor: a
control too small to read is a worse lie than one a little wider than the
field it belongs to.

### `const LIST_PAD`

It is not a gap. A combo's list is drawn **flush** against the widget, with
no rounding, no shadow and no frame margin, because that is what Acrobat
draws and because a floating card hovering a few points below a field reads
as a tooltip rather than as the field's own options.

### `const LIST_MIN_H`

The floor is deliberately this low rather than a comfortable minimum: a
larger one would exceed the room on the chosen side for a widget near the
edge of the screen, and `constrain_to` would then slide the popup back over
the widget, which is the failure the module header's is about.

### `const ROW_VPAD`

Small on purpose. `egui`'s own `selectable_label` sets a button's padding,
which gives rows roughly half again the height Acrobat draws — so a list of
eight options needed a scroll bar where Acrobat needed none, and the two
surfaces could not be compared row for row.

### `fn forget`

Called on every path that clears the focus, so that clicking the same
field again a minute later opens a list positioned and highlighted from
scratch rather than from wherever the last visit left it.

### `fn list`

# Two presentations, and they are not styling variants

`/Ff` `Combo` decides **where the options are drawn**, and the two answers
are structurally different surfaces:

* a **combo box** drops its list *outside* the widget, below it when there
  is room and above it otherwise — the module header's governs the side,
  the constraint and the scroll height;
* a **list box** draws its options **inside its own rectangle**, opaquely
  covering the appearance stream, with a scroll bar the moment they do not
  fit. It is anchored to the widget and deliberately overlaps it, so none
  of the half-plane arithmetic applies.

### The opacity is load-bearing, and Acrobat is the counter-example

An in-place list that lets the page show through is unreadable over the one
document type this program exists for. O209, on a SolidWorks-exported
drawing: *"I tested some on the SW drawing so I guess the background on the
drawing interferes with the list box in Acrobat."* Acrobat's own list is
legible over a blank form and not over dense vector line work, so the fill
here is the theme's opaque text-entry background rather than anything
derived from the widget or blended with what is underneath.

That is measured against Acrobat rather than assumed, and it is what O209
reports: *"the list option is somehow hidden from view in Acrobat until I
click on it, then I can select options and if the box is too small for all
of the options it gives a scroll bar."*

# Why the frame is built here instead of using [`egui::Frame::popup`]

`Frame::popup` is a rounded, shadowed, generously padded card — correct for
a menu floating over an application's own chrome, wrong for a control
belonging to a rectangle on a page. Acrobat's list is a square 1 px box
flush against the field, no shadow and no margin, and the difference is
most of what *"look different than they do in Acrobat when they are clicked
on"* names. The border takes `canvas_selection_ink` so it matches the focus
ring [`choose`] has already drawn around the same widget.

### `fn row`

# Why the row is painted rather than assembled from `selectable_label`

Three things have to be true at once and no stock widget delivers them
together: the row is the **full width** of the list (a click anywhere along
it picks, as in every list the operator has used), it is **compact** enough
to compare against Acrobat row for row, and a selected row is a **solid
plate with legible ink** rather than a tinted button.

The plate is `Theme::accent_pair` — the sanctioned spelling of *"paint this
as the emphasised thing"*, contrast-gated at the theme, and the only legal
route to a solid emphasis colour outside the theme module.
`tools/gates/check-selection-channel.sh` forbids reading `visuals.selection`
here and `check-theme-colors.sh` forbids naming a colour outright; both are
satisfied, and the result is Acrobat's solid-row treatment expressed in this
shell's own palette instead of copied out of a screenshot.

A **multi-select** row gets the same plate and no check box. That is
Acrobat's answer too, measured: a multi-select list marks its chosen rows by
filling them, and adding a check box would invent an affordance the product
class does not have — which is why this takes no `multi` argument.

### `fn pointer_in_list`

Read from the `Area`'s rectangle rather than from a row's response, because
a point on the frame's padding is still inside the popup — and because the
`Area`'s rect is in `egui` **memory**, carried over from the pass that drew
it, so this answers before this frame's popup has been laid out. That is
what makes it usable by [`choose`]'s focus guard, which runs above the call
to [`list`].

### `fn pressed_in_list`

[`pointer_in_list`] plus *"and a button went down this frame"* — the extra
half that makes it a claim on **this** press, which is what
[`super::overlay`] needs to know before reading the same press as a request
to focus some other field.

### `fn wanted`

Single-select is the one option picked. Multi-select toggles it against
what is already selected.

# Rebuilt from `/Opt`, which is not what the panel does

The current selection is recovered by asking each *option* whether it is
selected, rather than by copying `/V` and editing it. The difference shows
on a field whose `/V` holds a value matching no option — a real state, set
by another program or left behind when the option list changed. Carrying it
forward hands `set_choice_value` a value it must refuse
(`ChoiceValueNotInOptions`), so the operator's tick would fail with a
refusal naming a value they never touched. Rebuilding drops it instead,
which is the same thing picking a new value on any other surface does.

### `fn arrow`

Vertical only, which is where this differs from
[`super::tabbing`]'s `arrow`: a radio group may be laid out in a row, so
the horizontal arrows mean something there. A list is a list, and Left and
Right are left to whatever the canvas means by them.

### `fn focus_choice`

[`super::focus_button`]'s twin, and `draft` is seeded the same way and for
the same reason — equal to what the document holds, so that
[`super::commit`] on the way out finds nothing changed and writes nothing.
A choice field's value is never a draft: every pick is a complete command
the instant it is made.

### `fn choose`

Returns whether this frame's primary press belonged to the popup, so
[`super::overlay`] does not also read it as a request to focus something
else.

The `rect` is [`super::boxes::editor_rect`]'s — the widget's own rectangle,
grown to a legible minimum — so the ring and the popup's anchor are the
same rectangle the text editor would have used on the same widget.
