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

### `fn advance`

Called before anything else this module draws — see the header. Does
nothing at all on the overwhelming majority of frames: `tabnav::take`
answers `None` unless the hook claimed a press, which it does only while the
canvas holds egui's keyboard focus.

### `fn hold_or_settle`

[`super::editor`]'s two undrawable branches. A focus whose page is not in
the strip, or whose box is outside the clip rect, is normally a focus the
operator has scrolled away from — and committing it is the old spec's rule
and the right one, because a half-typed value is something they typed on
purpose.

A focus a **Tab** put there is the exception, and [`Focus::waiting`] is how
the two are told apart: it names a box the ring chose, whose reveal is still
in flight. Settling it on the frame the reveal was asked for would make Tab
appear to do nothing whenever the next field was off screen, which on a form
worth tabbing through is most of the time.

Returns `false` in both cases, because neither drew an editor and so neither
claimed the frame's click.

### `fn button_focus`

The button half of [`super::editor`]. There is no caret and no draft to
edit: the whole of the state is *this box has the keyboard*, drawn as a ring
and spent by Space or Enter.

# Why the ring is a cursor and not a mark on the content

pdfcer's rule 4 forbids styling applied content as provisional and admits
the cursor in full. A focus ring says where the next keystroke goes; it
states nothing about the document, disappears the moment focus leaves, and
is drawn over the finished raster so it reaches no print, no export and no
save. It is the same affordance as the I-beam this module already sets over
a fillable field.
