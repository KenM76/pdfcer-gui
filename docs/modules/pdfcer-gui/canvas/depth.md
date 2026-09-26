# `canvas::depth` — how deep the last click reached, and how deep it could have

Two numbers, remembered from the last selecting click: **which** candidate
was taken, and **how many** there were under the pointer.

## Why they are worth remembering at all

The operator, 2026-08-26: *"when I click on one of the objects all I get is
the page selected."*

Two things were true. The engine did not enter form XObjects, so most of
what he could see was not in the object model — that is filed as an engine
request and is not this module's business. And **this shell threw away the
rest of the list**: `hit_test_all` returns every candidate under a point,
front to back, and the pick called `.find()` on it. Anything underneath
anything was unreachable.

`Alt`+click now walks the stack ([`crate::canvas::clicking`]'s
`CycleCursor`). But a cycling gesture with no readout is a gesture nobody
discovers: the operator clicks, gets the wrong object, and has no way to
learn that there were four more behind it. **The count is what turns a
mystery into a diagnosis** — *"1 of 5 here"* says both that this is not the
only answer and that there is a way to ask for the others.

## Why a memory slot rather than a field on `OpenDoc`

Because it is a fact about the last **gesture**, not about the document. It
does not survive a document change, it must not be persisted, and nothing
that re-derives the selection should re-derive it — a selection restored
after an edit was not clicked for, and claiming *"1 of 5"* about it would be
describing a click that never happened.

`egui::Memory` is where per-frame and per-gesture UI state lives in this
shell, and it drops on its own when the context does.

## ★ It is deliberately NOT part of the selection

`SelectionState` is the answer to *"what is being worked on"*, and it is
read by the overlay, by every transform verb and by two panels. A depth is
the answer to *"how did we get here"*, which is a different question with a
different lifetime — it is stale the instant the selection changes by any
route other than a click.

Putting it on the selection would make every consumer carry a field none of
them can use, and would make a restored or programmatic selection have to
decide what to claim about a click that did not occur. Keeping it apart lets
the honest answer be *nothing*: [`taken`] returns `None` and the status line
says nothing rather than something untrue.
