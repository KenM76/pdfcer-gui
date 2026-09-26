# `canvas::notepopup` — reading a comment where the comment is

The window that opens when an operator clicks a note on the page, and the
tooltip that appears when they hover one. **The canvas half of the review
surface**, and the half that was missing.

## ★★★ The report this closes

The operator:

> *"check how the review functions work. unless something has changed I
> could add a yellow sticky note but even in read mode I don't think I
> could figure out how to read it. the review features should look and act
> the same as they do in Acrobat Reader."*

Two facts hold the answer up:

1. **`pdfcer-core` writes a `/Popup` for every sticky note.**
   `annot_author`'s `sticky_note` authors the companion dictionary, its
   `/Open`, and a rectangle 150 pt wide beside the note — so the state is
   already in an operator's files whether or not a shell draws it.
2. **Acrobat *Reader* is a read-only product and reading comments is its
   whole purpose.** A mode named Read that cannot read the comments has the
   posture exactly backwards.

⇒ Which decides where this lives. A pop-up on the canvas is **canvas
behaviour, not a ribbon item**, so it is mode-independent *by
construction*: no future edit to a tab list, a manifest or a panel default
can take it away again. That property is the whole reason this was built
before the panel work, and it is why it is not a fourth panel.

## The interaction, and every part of it is the convention rather than an
invention

This project's standing rule — *"use the conventional interaction, never
invent one — the convergence of the product class IS the spec"*:

- **Hover** a comment ⇒ a tooltip with the author and the words. The
  cheap half of the same question, in every reader in the class.
- **Click** a comment ⇒ its pop-up opens; clicking it again closes it. A
  *drag* moves the annotation and is a different gesture entirely — egui
  reports a click only when press and release land together — so the two
  cannot collide.
- **×** on the pop-up ⇒ closes it, as every window in the class does.
- A note the file marks `/Open` ⇒ **opens with the document**, no click.
  §12.5.6.4 Table 172 and §12.5.6.14 Table 183 both say so, and the
  state is in the file.

★ **A single click rather than a double.** In a reader, one click opens the
note; in an editor with the comment tool armed, one click selects and two
open. pdfcer has to serve both stances from one canvas, and a single click
serves both because opening a pop-up **does not consume the click**: in
Review and Edit the same press still selects the annotation, so the
selection outline, the grips and the Format tab all behave exactly as they
did. Nothing was taken away to add this.

## ★★★ Rule 4: the pop-up is CHROME, and the page is untouched

*"Fuzzy never sneaky"*, and the one-line test this project uses for it:
**would a screenshot of the editing canvas differ from a screenshot of the
same document saved and reopened?**

A pop-up is the same class of thing as a selection handle or a snap marker
— the cursor's own furniture. So:

- It is drawn in an `egui::Area`, a **separate layer** above the page
  raster. Not one pixel of it is composited into anything that is saved.
- It is drawn at a **fixed size in screen points and does not scale with
  zoom**, which is what makes it unmistakably interface rather than
  content. Acrobat's pop-up behaves the same way and for the same reason.
  ★ Its *position* does follow the page, because it is about a particular
  note; its *size* does not, because it is about the operator's eyes.
- Nothing it shows is inferred. The words, the byline and the date are
  verbatim from the file; the open state is read from `/Open`, never
  defaulted (see [`model::read_open`], and the request filed beside it).
- **Closing a pop-up changes the screen, not the file** — and
  [`crate::text::annotpopup::popup_close_tooltip`] says so, because an
  operator has every reason to assume otherwise.

## What it can do, by mode

| | Read | Review | Edit |
|---|---|---|---|
| hover tooltip | ✅ | ✅ | ✅ |
| open the pop-up and read the note | ✅ | ✅ | ✅ |
| read the thread of replies | ✅ | ✅ | ✅ |
| edit the note, remove it, delete the comment | — | ✅ | ✅ |

### ★★★ Read shows and does not edit, and the reason is on screen

`MODES_AND_PANELS.md`'s stance for Read is *"the page content is not yours
to alter"*, and **reading is not editing** — which is the whole argument
for this module existing. So the editor is not drawn in Read.

It is not drawn *greyed*, either. R9: *"an unavailable capability renders
nothing; a temporarily unavailable one may grey and must explain on
hover."* Read mode is the purest example of temporary this program has —
the operator chose a stance and a labelled three-position control changes
it — so the pop-up carries **one sentence naming the mode that can**
([`crate::text::annotpopup::popup_read_only`]) and no dead text box. A
disabled `TextEdit` would be the half-built surface the no-placeholders
rule exists to forbid.

## ★★★ What it CANNOT do, and what kind of absence each is

Neither absence here is an engine gap. `EditSession::add_reply` writes
`/IRT` and `/RT /R` with its own `/Popup`, and `EditSession::add_review_state`
writes `/State` and `/StateModel` (§12.5.6.4 Table 171); both are reached
from the **Comments panel** (`crate::panels::comments`), which is where a
reviewer's work list already lives.

- **No Reply control on this surface.** The window already *shows* the
  thread ([`thread`]); composing in it is wiring nobody has asked for.
- **No Accepted / Rejected / Completed control on this surface.** The same
  decision and the same place: a review status is a property of the work
  list rather than of one open window.

★★ Both are **scope** decisions rather than capability ones, and the
distinction is worth keeping sharp, because the two expire on different
events: a capability absence is a statement about the **program** and ends
when the engine moves, while a scope absence is a statement about **one
surface** and ends when somebody decides that surface should compose.
Conflating them is how a scope decision comes to be defended with a
capability argument that is no longer true.

⚠ Where an absence *is* a capability one, R9 governs and **nothing is
drawn**: no empty status row. A control that no state of the program could
enable is not an affordance, it is a promise.

## ★★★ Recording `/Open`

`EditSession::set_annotation_open` writes the window state **into the
document**, and `controls`' `open_default` is the control for it. The read
half is `model`'s `read_open`, over
`pdfcer_core::annot::Annotation::open`.

⚠ **Opening and closing a bubble on screen still writes nothing**, and that
is a decision rather than a leftover. [`open_default`]'s doc comment carries
the undo argument in full; the short form is that reading a marked-up
drawing *is* opening and closing bubbles, and an undo log in which nineteen
entries in twenty say *"looked at a comment"* cannot do the job an undo log
is for.

## Where the pieces are

- [`model`] — the pure read: what notes are here, where their windows
  go, what replies hang off them.
- [`open`] — which pop-ups are showing, and the override rule that lets
  the file speak first.
- This file — the drawing, the two hooks, and the trace.

## The two hooks, and how small they are

[`show`] is called from `crate::app::surfaces` on the line after
`canvas::show` returns — **one statement** — because a floating layer does
not belong in the canvas paint order at all (`crate::canvas::painting`'s
header states that order and every position in it is an argument; a pop-up
has no position in it). [`clicked_on`] is called from
`crate::canvas::clicking` beside the annotation hit test — **one
statement**, consuming nothing.

★ [`show`] takes this frame's mapping from `crate::canvas::zoom::last_frame`
rather than being handed one. `canvas::present` publishes it through
`remember_frame` **before** it calls `interact`, so by the time this runs
it is this frame's map and not the previous one — which is what keeps the
window from lagging a pan by a frame.

## ★★★ A pop-up NEVER covers the annotation it belongs to

An `egui::Area` at `Order::Middle` takes every press inside it, so a pop-up
laid over its own note swallows every gesture on that note — and the
symptom reads as a canvas defect rather than as a placement one: the move
drag and the grip drag never arrive, while the rotate handle, drawn clear
of the box, keeps working.

[`popup_origin`] flips rather than slides, and [`clear_of_anchor`] carries
the candidate order, the measurement and the one case that has no
answer. ⇒ **A window that describes a thing must not be laid over the
thing**, and on an immediate-mode canvas that is not a cosmetic rule: the
window is an input surface, and the thing underneath becomes unreachable.

## ★★ WHEN a pop-up opens, which is a separate question from where

A comment with no words must not open an empty pop-up. [`model::under`]
answers for every annotation that *can* carry a note rather than for those
that do, so [`model::has_something_to_read`] is asked at the click site;
without it, a click meant only to select a shape produces a blank window.

The rule is **not** simply "has words" — a sticky note is a note whether or
not anybody has typed in it, and an operator who has just placed one needs
the window in order to write. Subtype decides for the two whose purpose is
the note; content decides for every mark that merely *may* carry one.

Under a continuous or facing display mode, pop-ups are drawn for the
**acting page's** annotations only. That is not a decision of this module:
it inherits `crate::canvas::selection::annot::under_pointer`'s frame of
reference exactly — one `page_index`, one `PageMapping` — so a pop-up
appears wherever an annotation is *selectable*, and nowhere else. Fixing it
is the same piece of work as making annotation selection reach a second
visible page, and doing it here alone would put a window over a note the
canvas will not let you click.

## `PDFCER_DIAG` proves what this computed

One `note-popup` line per frame with something to say: how many notes the
page carries, how many carry words, how many pop-ups are open, how many of
those the **file** asked for rather than the operator, and whether a
tooltip was shown. A screenshot cannot tell you that a pop-up opened
because `/Open` was true rather than because a click landed, and that
distinction is the whole of [`open`]'s contract.
