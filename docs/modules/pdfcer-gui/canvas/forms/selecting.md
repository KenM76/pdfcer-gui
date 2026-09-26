# `pdfcer-gui/canvas/forms/selecting`

**Selecting a form field, rather than filling it** — the Edit-mode half of
[`super`].

# The two surfaces, and why they are genuinely different subjects

The parent module fills a field: a click opens a live `egui` text box over
the raster, keystrokes go into a draft, and a commit turns the draft into
one `Action`. **Nothing here does any of that.** This half answers a
different question — *which field is the properties panel talking about* —
and its output is a selection, an outline, eight grips and a cursor.

They are separated by **mode**, not by taste: filling is the Read/Review
reading of a click on a widget, selecting is the Edit reading, and
[`super::surface`] chooses between them once per frame. A reader debugging
"my click did the wrong thing" needs to know which of the two ran; a reader
debugging "the outline is in the wrong place" needs only this file.

# What every function here has in common

**None of them mutate.** Each reads `boxes::FieldTarget`s — the memoised
placement the parent's [`super::placed`] owns — and either paints, sets a
cursor, or pushes an [`Action`]. The selection itself lives on the
document and is applied by the action queue at the end of the frame, which
is why a hit test rather than a state read is the right question to ask
during one (see `canvas::rightclick`'s table).

# Visibility contract

Everything here is `pub(super)` except [`right_click_hits_a_field`], which
is `pub` and re-exported by the parent so that
`canvas::forms::right_click_hits_a_field` resolves for `canvas::rightclick`.
Narrowing it breaks that caller.

## Item notes

### `fn seeded_select`

Called before [`select_click`] on the frames that one is called on, so a
real click in the same frame wins: both raise `FieldAction::Select` and the
queue applies them in order.

### `fn select_click`

A click on empty paper CLEARS, and that is deliberate rather than
incidental. Every selection model the operator uses works that way, and
without it the properties panel would go on describing a field long after
they had moved on — a panel that will not let go is worse than one that is
empty, because its contents look current.

Nothing is mutated here. The outcome leaves as an [`Action`], like every
other thing this canvas decides.

### `fn right_click_hits_a_field`

## Why this exists instead of reading `doc.selected_field`

Because on the frame of the click that field is **not selected yet**.
[`select_click`] does not mutate — it raises `FieldAction::Select`, which
the queue applies at the end of the frame — so `doc.selected_field` still
holds whatever was selected before, and a menu keyed on it would show the
*previous* field's menu, or the view menu, on the first right-click.

⇒ That is precisely the stale-snapshot hazard `shell::menus::MenuHost::with_conditions`
exists for, met one layer further out: `egui`'s popup is opened **by** the
secondary click, so there is no later frame on which the right answer could
arrive. The first right-click on a field would silently show the wrong menu
for ever.

It is the twin of [`crate::canvas::menus::right_clicked_object`], and it
answers the same question the same way — by hit-testing the click's own
position rather than by consulting state one frame behind it.

## It reproduces the surface's own gates, and it must

`edit_content` and `annotations_visible`: a form field is only *selectable*
in Edit mode with annotations shown, and a menu offered where selection is
not is a menu whose Delete acts on nothing. Read from the same two places
[`surface`] reads them, one frame later.

### `fn select_cursor`

`PointingHand`, the same cursor the fill surface uses, and deliberately
**not** a bespoke one. It says *"there is something here"*, which is the
only claim either surface needs to make; what differs is what a click does,
and a cursor is a poor place to say that. `ui-conventions` has no row for
this because it is not a convention question — both readings of the click
are "act on the thing under the pointer".

### `fn selection_overlay`

`OPERATOR_REQUESTS.md` **O53**: a selected field must be visibly distinct
from an unselected one.

It is drawn **here** rather than in `canvas::overlay::draw_selection`,
and the reason is that a form field is not in `SelectionState` at all:
`canvas::selection::annot` excludes `/Widget` outright so the form surface
owns those presses, and the selection lives on the document. The overlay
draws what the selection state holds; this draws what this surface owns.

The rectangle is the **same one** `hit_target` matched and
`widgetdrag::grab_box` projects — one rectangle for what the operator can
see, what they can grab and what moves. That is rule H7, and the third use
is the one that was missing.

Nothing is drawn when the selection names a widget the form no longer has
— a field deleted while selected, or a page that has changed underneath.
An outline around nothing is a claim about a field that is gone.
