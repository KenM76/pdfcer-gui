# `app::dispatch::clipboard` — cut, copy, the two pastes, and duplicate

Four ids — `edit.cut`, `edit.copy`, `edit.paste`, `edit.paste_duplicate` —
over **three kinds of operand**, and the whole subject of this module is the
fork that decides which of them a keystroke is about.

Six now. `edit.copy_as_vector` joined on 2026-09-04 (the copy-OUT) and
`edit.duplicate` on 2026-09-06 (`Ctrl+D`) — and the second of those is the
one that stretches the module's name, because **it never touches the
clipboard at all**. It is here because *"make another one of this"* is what
an operator was doing with Copy-then-Paste before it existed, and because
what it needs is this file's fork: which operand does the gesture mean?
Its own function carries the argument for why it is a separate id from
`edit.paste_duplicate`, which the name invites a reader to assume it is not.

## Why this is a module and not four match arms

`super`'s file crossed R2's 1,500-line ceiling for the fourth time when
`edit.paste_duplicate` arrived on 2026-08-29. It joins [`super::pages`],
[`super::images`] and [`super::textcopy`] as the fourth application of the
same seam, and it is the right seam independently of the line count: the
three-way fork below is the *entire* logic here, and a reader trying to
answer *"what does Ctrl+C do?"* should find it in one screen rather than
interleaved with tool arming and zoom.

## The fork, in priority order, and why each rung is where it is

| rung | operand | who answers | why it is above the next |
|---|---|---|---|
| 1 | **swept text or a live text draft** | `canvas::textsel` / the widget | the operator made the narrower statement more recently, and every program in the class resolves it this way |
| 2 | **a selected form field** | [`crate::canvas::fieldclip`] | a `/Widget` is deliberately not an annotation selection here, so nothing below can see one |
| 3 | **an annotation, or page content** | [`crate::canvas::clipboard`] | the general case |

Rung 1 is `text_owns_the_chord`, and its full argument — including why a
focused Find box counts even with no selection in it — lives on that
function beside the claim it enforces.

**Rung 2 is the one that was missing**, and its absence was not a lossy
path but *no path at all*. `canvas::clipboard::copy` reads `doc.selection`;
a selected form field lives on `doc.selected_field`; so `Ctrl+C` over a
field with visible grips around it fell through to the content copy and
refused with *"nothing is selected"*. `DEFECTS.md` D4a's shape exactly: a
sentence describing a different world than the one on screen.

## The two pastes are two commands, not one command with a modifier

**Ken, 2026-08-29:** *"ctrl v for paste as new. ctrl shift v for paste as
duplicate."* — `OPERATOR_REQUESTS.md` **O58**.

They are separate ids because a command is the unit this shell can
*register*, *bind*, *place on a ribbon*, *put in a context menu* and
*withhold by mode* (R8). A single `edit.paste` that read the modifier keys
itself would be reachable only from the keyboard: there would be nothing to
put in the Edit menu beside Paste, nothing to grey out with an explanation
when the clipboard holds a markup rather than a field, and nothing for the
keymap editor to rebind.

`edit.paste_duplicate` over a **non-field** clipboard is not an error and
not a silent no-op: it falls through to the ordinary paste. A markup has no
second sense to duplicate into, so the honest answer to *"paste that as a
duplicate"* is the paste. Refusing would punish an operator for pressing the
more specific chord when the general one was all that applied.

## Mode gating, and why each gate reads a different thing

- **Cut** is gated on *what is selected*, because a cut removes that thing.
- **Paste** is gated on *what is on the clipboard*, because a paste has no
  operand on the page to look at.
- **Copy** is gated on nothing. The operator's own ruling — *copying is not
  authoring* — the same line that put `file.copy_page_text` in Read mode.

A form field is content: cutting or pasting one takes `edit_content`, the
same predicate the Delete key reads, because a form field is part of the
document rather than a comment on it.



```text
chord-command      chord="Ctrl+C" id=edit.copy  via=clipboard-event
chord-command      chord="Ctrl+V" id=edit.paste via=clipboard-event
chord-not-offered  id=edit.paste mode=review
```

**Copy was offered in Review and paste was not**, so the mode whose entire
purpose is marking up somebody else's drawing could copy a comment and had
nowhere to put it. The gates in *this* file were already right; they had
simply never been reached from Review.

⇒ The four ids now escape their tab
(`app::modes::capability::GATED_BY_THEIR_DISPATCHER`) and this module is the
only thing standing between a mode and a verb it may not do. **That makes a
silent `return` here a keypress that does nothing and says nothing** — the
project's founding defect class — where before it was at least a
`chord-not-offered` line. So both mode gates below call
`app::status::decline::record_mode_refusal`, which draws in the `⊗` slot
that means *this did not happen*.

⚠ **`app::actions::record_note` is the wrong slot for these and must not be
used**: it draws under `⚑ About your last edit:`, which claims an edit
happened. `app::status::decline::clipboard`'s header carries the argument.
The clipboard's **other** refusals — `canvas::clipboard::Refusal`, which are
about the operand rather than about the stance — still go through
`record_note`, and that is named there as unfinished rather than principled.
