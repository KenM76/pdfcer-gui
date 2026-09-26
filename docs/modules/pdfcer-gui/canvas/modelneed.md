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
