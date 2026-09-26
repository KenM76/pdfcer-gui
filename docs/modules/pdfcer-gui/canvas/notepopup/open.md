# `canvas::notepopup::open` — which pop-ups are showing, and who decided

One subject: **the open/closed state of every note pop-up**, and the rule
that the file gets the first word and the operator gets the last.

## The model: overrides, not a set of open windows

The obvious implementation holds a `BTreeSet<ObjId>` of open pop-ups and
seeds it from the document. **Do not**, and the reason is the requirement
this whole feature was commissioned under:

> *"`/Popup`'s `/Open` state is in the file. A note authored open should
> open. Read it; do not default it."*

A seeded set has to be seeded *somewhere*, on some frame, from some
document — and every candidate for that moment is wrong in a way that is
invisible until it bites. Seed on first frame and a document opened into a
second tab never gets seeded. Seed per edit epoch and every keystroke in a
note re-opens every pop-up the file authored open, throwing away what the
operator closed. Seed on document change and you need a document-change
event this canvas does not have.

So this store holds **only what the operator has explicitly done**, as
`ObjId → bool`, and the effective state is:

```text
open(note) = override(note.id)  ?? note.authored_open
```

Three properties fall out, and all three are what was wanted:

1. **The file's `/Open` is honoured with no seeding at all** — an untouched
   note reads its state straight out of the document, on every frame,
   including the first.
2. **An edit cannot reset a pop-up**, because the override does not depend
   on the epoch. Saving a note leaves its window exactly where it was,
   which is what Acrobat does and what an operator mid-review expects.
3. **Closing a pop-up the file authored open is expressible.** A bare set
   seeded from the document cannot represent that without a second set of
   "explicitly closed", which is this map wearing a worse shape.


An override is **interface state and only interface state**. It is never
written back, never saved, and never included in any comparison of what the
document says.

This paragraph used to justify that with *"`pdfcer-core` v0.38.0 has no verb
that could write `/Open` on an existing annotation anyway"*, filed as
`request_a_notes_open_state_cannot_be_changed.md`. **`Pass 253.3` shipped
`EditSession::set_annotation_open`** and that reason expired. The behaviour
here did not change, and must not — see below — so what changed is the
argument, which now has to stand on its own.

### The argument, standing on its own

Reading a marked-up drawing **is** opening and closing bubbles, dozens of
times in a review, and none of it is an edit an operator would recognise as
one. Wire this store to the engine verb and a reviewer who glanced at six
comments ends the session with six `CommandKind::SetAnnotationOpen` entries
between `Ctrl+Z` and the last thing they actually changed, and a document
that reports itself modified after a sitting in which they altered nothing.

⇒ So the honest sentence is still: **closing a pop-up is a thing you do to
your screen, not to the file** — and there is now a second sentence beside
it, which is that *recording it in the file is a separate, explicit act*
with its own control. `crate::canvas::notepopup::open_default` is that
control and carries the full undo argument, including the two alternatives
(coalescing, and accepting the entries as honest) and why each was rejected.

The two interact in exactly one place and it is deliberate: writing the
document also **pins the override to what is currently on screen**, so
recording *"closed by default"* does not make the window the operator is
reading vanish under their hand.

## Where it lives, and why not on `OpenDoc`

`egui::Memory`'s temporary data, keyed by the document's path — the same
store `crate::canvas::interact`'s gesture machine and
`crate::canvas::textedit`'s draft already use, and for the same reason: it
is per-frame interface state with no place in the document model, and
`crate::app::state::OpenDoc` belongs to no track this session.

**Keyed by path, so two open documents do not share a state.** Without
that, opening a second drawing in a new tab would show its notes with the
first document's pop-ups open — object ids collide across files freely, so
the collision is not a rare case, it is the normal one.

⚠ Temporary memory is dropped on restart, which is correct: an override is
a statement about this sitting, and a pop-up the operator closed last
Tuesday should not stay closed against a file that says it is open.
