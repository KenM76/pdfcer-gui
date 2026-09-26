# `app::actions::export` — writing part of the document out as something
else

## Why this is its own file

The sixth sibling of [`super::apply`], drawn along the same seam the other
five are: *what class of thing does this verb act on?* — pages there,
annotations in `annots`, the dimensioning model in `dimensions`, page
content in `apply`, redaction marks in `redact`. This is **what leaves the
document**.

It is a real subject rather than a size-driven cut, and the evidence is the
property every verb here shares and no verb elsewhere does: **none of them
changes the document at all.** No `vector_edit`, no undo entry, no epoch
bump, no cache invalidation. They read the open file and write a different
one, which makes every rule the mutation funnel enforces irrelevant to them
and every rule about *file* handling — a save picker, an overwrite, a
partial write — apply instead.

`super::pages::extract` is the same shape and stayed in `pages` because its
subject is a page set. If a third export lands, that is the moment to move
it here.

## The third export landed, and the trigger above FIRED

[`image`] — `OPERATOR_REQUESTS.md` **O120**, PNG / JPEG / SVG — is the third,
and it is written here rather than in a module of its own, which is the
easier half of what the sentence above asks for. The harder half is
**`pages::extract` has not moved**, and that is recorded rather than quietly
not done:

* The condition is met. Three exports now exist and the family is real.
* Moving `extract` is a change to `pages`, to `apply`'s dispatch and to
  whatever names it, made in the same pass as a new feature — and this
  project's own record of what that produces is `RIBBON_IA.md`'s repeated
  lesson that a taxonomy move and a capability arriving together make a diff
  nobody can review as either.

⇒ So the trigger is left **armed and stated** rather than silently reset. The
next reader of this header is looking at a condition that has fired, with
the reason it was not acted on written beside it, which is the shape this
project uses everywhere else for a decision deferred on purpose. What must
not happen is the sentence above being read as still-waiting: it is not.

## Why an export is an `Action` at all

`super::apply`'s header answers it for `SaveCopy` and the answer is the same
here: **a native file dialog must not open inside a layout pass.** It is a
modal OS window that blocks the thread, and opening one from a widget's
`clicked()` branch means egui is part-way through building a frame that will
not finish until the operator has answered.

Nothing about the document is being ordered — there is nothing to order —
so the funnel's *invariant* does not apply. Its **reason** does.
