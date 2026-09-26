# `canvas::textedit::cost` — **what a per-keystroke re-measure actually costs**

`DEFECTS.md` **D4b**'s first sentence is *"there is no re-layout per
keystroke"*, and the old shell's own comment agrees in terms: *"Typing →
build/extend the `PendingEdit` (§6.1). **No core call per keystroke.**"* So
"as you type", nothing moves at all, and D4b says that alone accounts for
much of the complaint.

The brief for this work says: **measure what a per-keystroke re-measure
actually costs and report the number**; if it is too slow, say so and
debounce deliberately rather than silently. This module is that measurement,
and it is `#[ignore]`d for the reason every measurement in this repository is
— a timing assertion in CI is a flake, and the value of a number is that
somebody read it.

Run it:

```text
cargo test -p pdfcer-gui --lib canvas::textedit::cost -- --ignored --nocapture
```

## What is being measured, and why it is not a micro-benchmark

There is **no public dry-run** in `pdfcer-core`. `plan_edit` — the function
that locates the anchor, re-encodes, sums the §9.4.4 advances and produces
`EditReport::advance_delta`, which is *exactly* the number a live re-layout
wants — is `pub(crate)`. The two public routes to it are:

| route | what it does beyond planning |
|---|---|
| `text_edit::edit_text` (free fn) | plans, then **performs an incremental save**, returning the whole appended PDF |
| `EditSession::edit_text` | plans, then **commits a command to the undo log** |

Neither is a query. The first allocates a document-sized byte vector per
call; the second mutates. So the honest thing to measure is the *cheapest
public route to a real advance delta*, which is the free function, plus the
two derivations the shell would also have to redo — and to report the
components separately, so the number can be read rather than merely quoted.

## The comparison that decides it

`DEFECTS.md` records Find at **331–449 ms per whole-document call**, which is
why Find never searches on a keystroke. The threshold that matters for typing
is far lower than Find's: a keystroke that is not on screen within roughly
**16 ms** has missed a frame, and one that takes longer than about **50 ms**
is felt as lag. So the question is not "is it faster than Find" but "does it
fit in a frame".

---

# ★★ THE MEASUREMENT, and the decision it forced

`--release`, median of 5, this machine:

| document | extract (prov.) | recognize + align | plan + save | total |
|---|---:|---:|---:|---:|
| `tail-alignment` (3 lines) | 0.12 ms | 0.01 ms | 0.36 ms | **0.49 ms** |
| `SW41177` p1 (a SolidWorks sheet) | 32.07 ms | 0.16 ms | 70.54 ms | **102.77 ms** |
| `ncored-benchmark` A3 (129,758 objects) | 356.53 ms | 2.79 ms | — | **356+ ms** |
| `a1-titleblock` (repo fixture) | 0.46 ms | 0.01 ms | — | — |

(A dash is a refusal rather than a cost: those pages' page-1 runs are not
editable through this API — an embedded subset without the new code — and
timing a refusal and reporting it as a re-measure would be a measurement of
something other than what it claims.)

**So the answer is: it does not fit, on the documents this operator actually
opens.** 102.77 ms per keystroke on a SolidWorks sheet is **six frames**, and
the benchmark A3 spends 356 ms in the *extraction alone* — which is Find's
own 331–449 ms, arriving on every key.

## The decision, stated rather than debounced quietly

**Priority 2 does not land, and it is not debounced either.** A debounce
would have been the easy answer and it is the wrong one here: a re-layout
that appears 150 ms after you stop typing is not *"text moves as you type"*,
it is a second, later, surprise — and D4a already records that this feature's
besetting sin is showing the operator something that is not what the document
will say. Shipping a laggy approximation of the thing they asked for would be
a third.

## What would actually fix it, and where it has to happen

Not here. Two of the three numbers above are avoidable, and neither is
avoidable from this repository:

1. **`plan+save` is the wrong operation.** The number a live re-layout wants
   is `EditReport::advance_delta`, and `plan_edit` computes it *before* any
   write — that is the seam's stated purpose. It is `pub(crate)`. A public
   `measure_edit(&Document, &EditRequest) -> Result<f64, EditError>`, or
   simply making `plan_edit`/`EditPlan` public, removes the incremental save
   from the loop entirely. **This is a feature request for `pdfcer-core`**, and
   it is the whole of what Priority 2 is blocked on.
2. **`extract` is already cached** in the running shell — `app::cache` keys it
   on `(page, edit_epoch)` and typing bumps no epoch — so the 32 ms and the
   356 ms are paid once per page, not per key. The commit path pays a second,
   provenance-capturing extraction (`textedit::plan` explains why); a
   per-keystroke loop would want that one cached too, which is a change here
   and a small one.

With (1) in hand the marginal per-keystroke cost on `SW41177` would be the
plan without the save. That is not measurable from outside the crate, which is
the honest end of this measurement: **the number that decides whether
Priority 2 is affordable cannot be obtained through today's public API**, and
saying so is better than reporting the one that can be.

## The cheap approximation that was considered and NOT taken

Every `ExtractedGlyph` already carries a real `advance` — §9.4.4, with `Tc`,
`Tw`, `Tz` and the `TJ` attribution folded in. A draft could therefore be
measured by looking each character up among the glyphs *already on the page in
the same font*, in microseconds, with no core call at all.

It is real metrics, and it has a hole: a character the page does not already
show has no width, so the measure would be exact for most typing and silently
wrong for the rest — and "silently wrong about where your text ends" is the
defect being fixed, wearing a faster suit. It could be made honest (measure
what is measurable, disclose the rest), and that is a reasonable thing to
build **after** (1), when it would be a fallback rather than the mechanism.
