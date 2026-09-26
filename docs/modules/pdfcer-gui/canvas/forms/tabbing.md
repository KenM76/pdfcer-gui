# `canvas::forms::tabbing` — Tab walks the form, and a focused button waits

`OPERATOR_REQUESTS.md` O204, the field half:

> *"when I press tab while in a form I end up tabbing through the menus
> instead of the form items. The tab should tab through whatever space I
> have clicked on … if I've clicked on a form item it should tab forward and
> shift-tab backwards to the next one."*

## Contract

Two entry points, both called from [`super::overlay`]:

* [`advance`] spends a press [`crate::canvas::tabnav`] took off egui, moves
  [`super::Focus`] to the next stop of [`super::ring`]'s table, commits what
  was being typed, and asks for the least scroll that brings the new stop
  into view.
* [`button_focus`] is the other half of [`super::editor`]: a check box or a
  radio button cannot hold a caret, so it holds a **focus ring** instead and
  reads Space, Enter and the arrow keys.

## Why the ring must advance before the editor draws

[`advance`] runs first in the frame, so the editor [`super::editor`] draws
is the one Tab has just arrived at. Running it after would draw the *old*
field, request focus for it, and move the caret a frame late — which on a
held Tab is a ring that lags one stop behind the key and never catches up.

## Why a cross-page ring

O204 decision 3. A form is a document, not a page: tabbing off the last
field of sheet one lands on the first field of sheet two, because that is
what every program that fills forms does and because the alternative — a
ring that traps the operator on a page — makes Tab useless on the multi-page
forms it is most needed for. The object ring wraps within its page instead,
for the reason its own caller carries.

## Item notes

### `fn locate`

Falls back to *the ring stop belonging to the same field* when the box
itself is not a stop, which is the radio group: [`ring::assemble`] collapses
a group to its first widget, and an arrow key can leave the focus on one of
the others. Without the fallback the next Tab would find no position and
stop dead in the middle of a form.

### `fn move_focus`

`leaving` is `None` for a move that had no previous focus. The commit is
guarded exactly as [`super::settle`] guards its own: a draft describing a
document or a revision that is no longer on screen is dropped rather than
written, because writing it would write a value against a document the
operator has not seen since they typed it.

### `fn activate`

The same rule [`super::click`] applies to a pointer press, called rather
than restated so a keyboard activation and a click cannot come to mean
different things.

### `fn sibling`

`None` for anything that is not a radio group, which is what keeps the arrow
keys out of a check box's way: a lone check box has no siblings and the
arrows should go on meaning whatever the canvas means by them.
