# `panels::properties::annotdelete` — whether the selected annotation can
be deleted, and what would go with it


| query | what it answers | what this section does with it |
|---|---|---|
| `EditSession::annotation_deletion_refusal` | *would `delete_annotation` refuse right now?* | withholds every Delete control and puts a sentence in their place |
| `EditSession::annotation_deletion_preview` | *what else would go with it?* | states the collateral **before** the press |

## ★★★ The defect the first query closes, and it is the day-before defect
wearing a different `/Subtype`


**The annotation half was the same defect, one file along, and it was still
open.** `annotation_deletion_refusal` is `&self`, side-effect-free, and its
own doc comment names this call site by rule number — *"safe to call every
frame from a UI (R83: ask before offering the control)"*. Nothing called it.
So on a certified or encrypted drawing:

* the **Format tab's Delete** was drawn and enabled,
* the **canvas right-click's Delete** was drawn and enabled,
* the **Delete key** raised the action,


⇒ ★★ The generalisation, and it is the audit's rather than this file's: **a
query the engine wrote for a shell is not consumed by being read.** Both of
these carry doctests spelling out the call site, and both sat unused for the
whole life of the crate. The instrument that found them was
`tools/verb-coverage.py` — asking what the engine offers — not a re-reading
of this shell.

## ★★★ Where a gate refuses, the control is NOT DRAWN and a sentence takes
its place (R9)

The established shape, set by the forms fix and followed here rather than
re-invented:

- Greying is for a capability that is **temporarily** unavailable, and is
  always explained on hover. A certification signature is not temporary and
  cannot be argued out of; nor is `/Encrypt`; nor is §12.5.3 bit 8.
- A permanently-refused capability renders **nothing**, or a sentence saying
  where the thing actually lives. There is no elsewhere here, so it is the
  sentence.
- **A sentence rather than a silence.** A panel that quietly omits half its
  controls looks half-drawn, and an operator who finds Delete missing with no
  explanation has found a broken program rather than a protected document.

The withholding of the ribbon and menu controls is not done here — it cannot
be, because a manifest item is drawn by `egui-shell` and this crate may not
reach into it. It is done by the condition
`crate::app::conditions`' `selection.delete_permitted`, which `format.delete`
carries as its `visible_when` on the Format tab and on both canvas menus.
**One question, two consumers**: the condition and this sentence are derived
from the same three facts in the same order, and [`gate`] is the one function
that derives them.

## ★★ Why the SENTENCE lives in a panel and not in the status bar's decline

`crate::app::status::decline` is this shell's worded-decline surface and it
would have been the reflexive choice. It is the wrong one here, and the
module's own header says why in the general case:

> A decline must be **repeatable** … and a decline changes no document, so
> the epoch never moves.

A decline is a report that *a gesture just failed*. What this section states
is not a gesture's outcome at all — it is a **standing property of the open
document**, true from the moment it was opened until it is closed, and true
whether or not the operator has pressed anything. A sentence that arrives
only after a press, and retires when the next command runs, would deliver
that fact at the one moment R83 exists to get ahead of.

⇒ So it sits in the panel that describes what is selected, permanently, and
the press it prevents is a press the operator never makes.

## ⚠ The cost of the preview, and what was chosen

`annotation_deletion_preview` is `&self` and mutates nothing, but it is **not
free**: it locates the annotation, walks the page's whole `/Annots` array to
find `/IRT` referrers, and validates the `/Popup`. That is O(annotations) per
call. The old shell computed exactly this and gated it on **hover**, one row
at a time, because its Comments panel would otherwise have paid
O(rows × walk) every frame.

This section does better than hover, and the reason it can is that **it is
not a list**. There is one selected annotation, so the worst case is one call
per frame rather than one per row — and even that is not paid, because the
answer is memoised in [`DeletionPreview`] on `(annotation id, edit epoch)`.
Both halves of that key are load-bearing:

* the **id** changes when the operator selects something else, which is the
  only other thing that can change the answer;
* the **epoch** changes on every accepted edit, which is what makes a reply
  added or removed since the last frame visible here.

⇒ In steady state — an annotation selected, nothing being edited — the cost
is one `Option` comparison per frame and no engine call at all. A hover gate
would have been cheaper only in the frames where the answer is not wanted,
and it would have hidden the fact behind a gesture the operator has no reason
to make.

## What this section does NOT do

**It raises no action and offers no control.** It reads and it draws
sentences; `super::body`'s contract is `&OpenDoc` shared, so that is a
compile-time fact rather than a convention. The Delete controls stay where
they are — the Format tab, the canvas menus, the Delete key — and this
section only decides what is *said* about them.

**It does not run the delegated route's gate.** `annotation_deletion_preview`
reports a `/Redact` mark or a ce dimension with its
`AnnotationDeletionRoute` and **zeroed counts**, and the engine states
plainly that it does not run the destination verb's certification gate for
those. Because this section says nothing at all when every count is zero, no
false claim is made on that path: a delegated target produces silence here,
not a promise that the delete would work. The engine's own note says the
honest fix is for those verbs to grow refusal queries of their own, and
*"inventing a half-answer here would be worse than a stated gap."*
