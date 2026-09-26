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

## Item notes

### `fn duplicate`

# Why it is here and not an extension of `edit.paste_duplicate`

That question was asked first, because the two names are one word apart.
`edit.paste_duplicate` **does** already route by selection kind — this
module's header says what it does over a markup: *"falls through to the
ordinary paste … a markup has no second sense to duplicate into"*. Its
second sense is a **form field's**: `Ctrl+V` plants a copied field as a new
field, `Ctrl+Shift+V` plants it as another widget of the same one.

⇒ Teaching it to duplicate the **selection** over a markup would give one id
two unrelated behaviours — a paste verb that acts when the clipboard is
empty and ignores the clipboard when it is not — behind a chord named for
the behaviour it would stop having. The same argument this module's header
makes for the two pastes being two commands, applied once more.

# What it does NOT do, and it is the feature

It does not put anything on the clipboard and does not read what is there.
An operator laying out a row of revision marks keeps whatever they were
carrying — a part number, a title-block string — which `Ctrl+C`/`Ctrl+V`
destroyed once per mark. `crate::text::commands::edit_duplicate`'s tooltip
leads with that clause for the same reason.

# The mode gate is `author_markup`, and the sentence is its own

A duplicate authors an annotation, so Review — the mode whose whole purpose
is marking up somebody else's drawing — must be able to do it, and Read must
not. That is the same gate `paste` applies to a markup clip.

The **sentence** is not the same: `ModeRefusal::PasteMarkup` says *"switch
to Review to paste this"*, and nothing was pasted. A seventh variant —
`DuplicateMarkup` — carries the wording, and its doc comment argues why a
shared remedy still owes its own sentence.

It records through `decline::record_mode_refusal`, which draws in the
`⊗` slot meaning *this did not happen* — never through
`actions::record_note`, which draws under `⚑ About your last edit:` and
would report a press where nothing happened as an edit. Fourth application
of the split this module's header states.

# The operand refusals go through `record_note`, unchanged

Nothing selected, an annotation the engine will not carry, a selection that
has outlived its annotation — those are
[`crate::canvas::clipboard::Refusal`]s about the *operand* rather than the
stance, and they take the same route the other clipboard verbs' operand
refusals take, worded by the same `text::clipboard::refusal`. That routing
is named as unfinished rather than principled in this module's header, and
this arm inherits the note rather than inventing a second answer.

### `fn copy_as_vector`

`OPERATOR_REQUESTS.md` **O120**, 2026-09-03: *"Also I'd like to be able to
copy and paste anything to other software - like copy and paste vector
graphics into word or inkscape for example if possible."*

# Why this is a fifth id and not a modifier on `edit.copy`

The same argument the two pastes make one screen up, and it holds harder
here: **a command is the unit this shell can register, bind, place on a
ribbon, put in a menu and withhold by mode.** A modifier read inside
`copy_or_cut` would be reachable from the keyboard alone — nothing to draw
in the Clipboard group, nothing to name in a tooltip, and nothing for an
operator to discover. This is a *discoverability* feature as much as a
capability one: the operator did not know pdfcer could do it, which is why
he asked.

⇒ And the two verbs genuinely differ in what they produce. `edit.copy` puts
an internal clip plus a picture on the clipboard, for pasting back into
pdfcer. This puts four public formats on it, for pasting into somebody
else's program, and touches the internal clipboard not at all — so a copy-out
does not destroy what the operator had copied for an in-pdfcer paste.

# It says something on SUCCESS, which no other clipboard verb here does

Because it alone has two possible operands and the button cannot show which
was taken: the selection if there is one, the whole page otherwise. An
operator who selected three parts and silently got the sheet finds out in
Word, minutes later. `text::clipboard::copied_as_vector` carries the wording
and the argument.

# No mode gate, deliberately

*Copying is not authoring* — the operator's own ruling, the same line that
leaves `edit.copy` ungated above and put `file.copy_page_text` in Read. The
Edit tab is not shown outside Edit mode, so the control is absent rather than
refusing there; that is visibility doing the work, which is the rule
`app::modes` states.

### `fn handles_the_six_and_not_the_registered_absence`

`edit.paste_in_place` is the trap this test exists for: it is a
registered ABSENCE, and a prefix rule would claim it the day it became
real, routing it here with no body and no failure.
