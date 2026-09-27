# `app::actions::vector` — everything that changes page GEOMETRY

## Why this is its own file

**R2**, and the seam [`super::Action`] already draws twice:
[`super::dimensions`] is *what happens to the dimensioning model*,
[`super::pages`] is *what happens to a page*, [`super::annots`] is *what
happens to an annotation that already exists*. This is **what happens to the
marks on the page**.

It is a real subject rather than a size-driven cut, and the evidence is a
property every variant here shares and nothing elsewhere does: **each one
addresses paint-order indices into one page's content stream.** That makes
all of them subject to one rule nothing else in `actions` has to think
about — `docs/core-api/02-editing-and-saving.md` §1.10.1, *which verbs
RENUMBER* — and it is why the page travels on every variant rather than
being re-derived at apply time.

## The two verbs that both "move things", and why there are two

[`VectorAction::MoveSelection`] reaches `move_objects`, which rewrites
numeric **operands** in place. [`VectorAction::TransformObjects`] reaches
`transform_objects`, which wraps each object's operator run in `q <cm> … Q`
and never looks at an operand at all.

The second can express everything the first can and more — rotation, scale,
and *any object kind*, because text runs and images carry no coordinate
operands, which is exactly why `move_objects` is path-only by name. The
obvious tidy is therefore to route everything through the transform, and it
would be worse.

`move_objects` rewrites the numbers the producer wrote, so a moved path's
operands still say where its marks are. The transform leaves those operands
naming the old place and adds a `q <cm> … Q` the producer never wrote to
say how far. Both render identically — R8b's screenshot test cannot tell
them apart — so the whole of the difference is in the bytes a CAD re-import
reads, and pdf content is the one thing this program is not allowed to
alter casually.

⇒ The transform earns the gestures operand rewriting cannot express at
all: rotation and scale, which have no `re` spelling and no user-space
scalar for `line_width`, and every object kind, because text runs and
images have no coordinate operands and `move_objects` answers `NotAPath`
for both. It does not earn the gestures the operand rewrite already
expresses exactly.

## Item notes

### `fn from`

Thirty-five places raise one of these, and almost all of them are a
single `actions.push(…)` at the end of a gesture. Making each write
`Action::Vector(VectorAction::MoveNodes { … })` would put the enum's
filing system into every one of them — and the filing system is an R2
artefact rather than something a drag needs to know about.

`.into()` at the push, `From` here, one line.

### `fn census`

# Why this is measured at all, and why BOTH numbers

`RESUME.md`'s standing rule: *a trace line must carry the number a wrong
build would get wrong.* For a part-delete that number is not "did something
get deleted" — it is **whether the enclosing object survived**. That is the
entire subject of `Pass 32.0`: on the operator's drawing one text object
holds all 237 pdf-dimension labels, and a build that deleted the object
instead of the label removes every label on the sheet while looking, from a
trace that reported only success, exactly like a build that worked.

So the pair is:

* `objects` — the page's own paint-order count. **Unchanged** is the whole
  claim: the enclosing object is still there.
* `parts` — the entered object's lines, runs or points. **One fewer** is the
  other half: something really was removed.

Either number alone is satisfiable by a wrong build. `parts` alone is
ambiguous after a whole-object delete, because deletion **renumbers** — the
index the caller held then names a different object, and asking it for a run
count answers about whatever moved into the slot. `objects` alone cannot
tell a delete that removed a line from one that removed nothing.

# Reading it costs a decomposition, and that is already paid

`page_objects` is keyed on `(page, edit_epoch)`. The "before" read is the
decomposition the canvas already built to draw the selection outline the
operator is looking at; the "after" read is the one the very next frame will
build anyway to re-resolve the selection. Neither is a second walk.

The `Ref` is dropped at the end of the statement, before `vector_edit`
takes `&mut doc` — the same ordering `DeleteSelection`'s erase preview has
to observe, and for the same reason.

### `fn fold_undo`

A visual line is however many show operators its producer wrote it as, so
one drag or one Delete is `pieces` engine commands. `coalesce_last` answers
`false` **only** when the undo stack was shorter than `pieces`; every change
is applied and only the grouping failed, so the contract's instruction is to
disclose and neither retry nor ignore.

The de-duplication is not cosmetic. `plan_move_text_run` emits one identical
inserted-`Td` sentence per run it had to place an operator for, and a line
of nine pieces would otherwise report the same fact nine times — which
reads as nine separate things having happened. First-seen order is kept, so
the sentences still arrive in the order the engine produced them.

### `fn apply`

Routed here from `super::apply` rather than living there, which is the shape
[`super::dimensions::apply`] already sets: the family module owns both the
vocabulary and what the vocabulary does. `super::apply` stays a routing
table.

Every arm goes through `super::apply::vector_edit` — the four-step protocol
(cancel the render worker, mutate through `Arc::get_mut`, bump the epoch,
drop the texture) whose whole reason for existing is that seven hand-written
copies would be seven chances to omit a step.
