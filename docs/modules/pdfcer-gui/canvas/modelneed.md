# `canvas::modelneed` — **does this frame need the page's object model?**

One question, asked once, for the whole canvas frame. It decides whether
[`crate::app::state::OpenDoc::page_objects`] is called at all — and
therefore whether every consumer below it sees a decomposition or a `None`.

## Why this is a module and not four lines inside `canvas::interact`

It **was** four lines inside `canvas::interact`, in the form of a
hand-maintained `matches!` over [`GestureOutcome`], and that list has been
the defect **four separate times**. Its own comments recorded three of them
before the fourth arrived:

| date | what shipped needing the model and not asking for it | how it presented |
|---|---|---|
| 2026-08-19 | `GestureOutcome::Resize` | *"the gesture does nothing"* — a whole driving session |
| 2026-08-19 | `GestureOutcome::Handle` | the same, a second driving session |
| 2026-08-20 | `GestureOutcome::DimensionVertex` | quieter: the drag worked and **never snapped**, which is indistinguishable from a snap that found nothing |
| 2026-09-05 | **the Delete key** at the Part and Node rungs | `canvas-delete-declined level=Part sel=1 reason=NoObjectModel` — three shipped verbs reachable by nothing |

**The fourth is the one that proves the list was the wrong shape, not
merely out of date.** `Resize`, `Handle` and `DimensionVertex` were each
fixed by adding a variant to the list, so each fix left the mechanism
intact and the next recurrence inevitable. **Delete is a keystroke, not a
gesture outcome**, so there is no variant to add: a list keyed on
`GestureOutcome` is structurally unable to express *"the operator pressed a
key that will need the model"*. Widening it was never going to be enough.

So the shape changed, in two ways that between them make the fifth
recurrence loud rather than silent:

1. **The gesture half is an exhaustive `match` with no wildcard arm.**
   [`gesture_needs_model`] names every variant of [`GestureOutcome`]
   explicitly. Adding a variant to that enum is now a **compile error here**
   until somebody answers the question for it. A `matches!` silently
   answers `false` for a variant it has never heard of, which is precisely
   how `Resize` shipped.
2. **The keyboard half exists at all.** [`Need::delete_at_a_deeper_rung`]
   is the term a gesture list cannot hold, and every future *"this keystroke
   needs the model"* belongs beside it rather than in a fifth place.

And a third guard lives at the far end, where the refusal is raised:
`canvas::keys`' Delete arm carries a `debug_assert` that fires when
`Refusal::NoObjectModel` is declined on a frame that **never asked** for the
decomposition — the difference between *"the page would not decompose"*
(honest) and *"nobody requested it"* (this bug, four times). The release
build says the same thing on the diagnostic channel: `asked=false`.

## The cost, measured rather than reasoned about

`pdfcer_core::decompose_page` walks every content stream on the page and
**has no cache anywhere in `pdfcer-core`**, which is why this gate exists in
the first place. Measured on the operator's benchmark drawing —
`D:\Dev\pdfTests\ncored-benchmark-cad-drawing.pdf`, 5.6 MB — by launching
the real binary under `PDFCER_DIAG=1` and reading its own line:

```text
pdfcer-diag page-objects-built page=0 objects=129758 leaves=10256 ms=531
pdfcer-diag objects n=129758 page=0 paths=129515 text=242 images=0 forms=1 leaves=10256
```

**531 ms, 129,758 objects, once.** One `page-objects-built` line in the
whole session: the shell *does* cache what the engine does not.
[`crate::app::cache`] keys the decomposition on `(page,
page_content_generation)` — the engine's digest of the page's content
dependencies, which moves when content moves and holds still for an
annotation — so a second `page_objects()` on the same page at the same
generation is a `Cell` comparison and a `Ref`, not a walk.

⇒ **That measurement is what decides the keyboard term's shape.** Two
candidates were on the table:

* *"ask whenever a deeper rung is selected"* — correct, and on a frame
  after a content edit it pays 531 ms **while the operator is still
  holding the selection**, for a model nothing on that frame reads;
* *"ask on the frame a delete key arrives"* — what is built. The rung
  cannot have been *entered* without a decomposition (the hit test that
  descends is itself a consumer), so on the ordinary press this is a cache
  **hit** and costs nothing measurable. When the generation has moved since,
  the rebuild is not overhead: a stale model would address the wrong index,
  which the engine's own words call *"silent corruption of the operator's
  drawing, reported as success"*.

The narrower term is therefore both cheaper and no less correct, and it is
the one that ships. Neither candidate was chosen from architecture; the
number above is why. (`BENCHMARK.md` exists because an earlier analysis
asserted a performance weakness from architecture and was wrong.)

## What this module is not

It is not a policy about *what* the model is used for, and it holds no
`egui` state of its own. It is a pure predicate over the frame's facts, so
every rule in it is a unit test rather than something to be hoped for in a
running window.

## Item notes

### `struct Need`

A struct rather than six positional arguments, for the reason
`canvas::keys::Keys` gives: the list has grown four times and every growth
was a defect being fixed, so the next one should read as a new named fact
rather than as a seventh `bool` nobody can order correctly at the call site.

### `fn wanted`

The four terms are OR-ed and each is documented on its own field or
function. Nothing here short-circuits for cost reasons — every term is a
field read or a `matches!`, and the expensive thing is what this decides
to call, not the deciding.

### `fn delete_at_a_deeper_rung`

Delete at the Part or Node rung reaches
[`crate::canvas::deleting::subject`], which needs the decomposition to
answer *what kind of part is this* — a subpath and a show operator wear
the same `subpath: Some(n)` field on a
[`crate::canvas::selection::Selection`] and reach **different engine
verbs** (`delete_subpath` and `delete_text_run`). Without a model it
declines `NoObjectModel`, which is the correct refusal for *"the page
would not decompose"* and was, for one commit, reporting *"nobody asked
for it"*.

# Why the rung is part of the condition

The **Object** rung answers from the selection alone: an entry already
holds a resolved `TargetId` and `object_indices_on` is a filter over
four integers. Asking for a decomposition on every Delete would make
the commonest destructive keystroke in the program pay 531 ms after
each content edit for a value that arm never reads. `subject`'s own
signature encodes the same asymmetry and says why.

# Why it is deliberately over-broad in the other direction

This does not replicate `canvas::keys`' guards — a focused text widget,
a draft in flight, a mode without `edit_content`, a form field or an
annotation claiming the key first. Every one of those would be a second
statement of a rule that already exists three hundred lines away, free
to drift from it; and being wrong in this direction costs a **cache
hit**, while being wrong in the other direction is the defect this
module exists to end. Over-asking is the safe error and it is chosen on
purpose.

### `fn gesture_needs_model`

An exhaustive `match` with **no wildcard arm**, and that is the whole
point of the function. A new [`GestureOutcome`] variant is a compile error
here until somebody answers this question for it — where the `matches!`
this replaces would have answered `false` in silence, which is exactly how
`Resize`, `Handle` and `DimensionVertex` each shipped needing the model and
not asking for it.

⚠ **Do not add a `_ =>` arm.** It compiles, it looks tidy, and it restores
the defect this function was written to remove. If a variant genuinely does
not need the model, say so by name — the `false` arms below are a list of
deliberate answers, not a default.
