# `dialogs::print::verdicts` — what the operator has actually LOOKED at

## The contradiction this module exists to remove

Operator request O113 made the preview's clip hatch **ink-aware**: on a 1:1
CAD sheet whose overhang is empty paper, nothing is hatched and the caption
says *"This sheet hangs over the printable area, but nothing is printed
there — the overhang is blank."*

[`Job::clipped`] stayed **geometric** — it counts sheets whose page box
exceeds the printable rectangle, a plan-time test taken with no raster in
hand — so the commit button went on reading *"Print — 1 sheet will be
clipped"* over a picture showing nothing lost.

Both sentences were true. They read as contradicting each other, and the
button is the louder surface.

## The wording was NOT weakened, because that is how the next defect
## gets built

The obvious fix — soften the button to *"1 sheet may be clipped"* — takes a
statement that is exactly true and makes it vaguer so that it stops
disagreeing with a better one. The disagreement is then hidden rather than
resolved, and the surface that was *right* is the one that got worse.


> *"Remember the blank/not-blank verdict for each sheet as the operator
> steps through the preview, and label the button with the geometric count
> minus the sheets known blank, with the ones not yet looked at still
> counted. Every claim stays true and nothing has to render the whole
> job."*

**Make the COUNT better, not the sentence vaguer.** That is what this
module does, and the wording then follows from what the count can support
rather than from what reads comfortably.

## The arithmetic, and what it is a bound ON

Every clipped sheet is in exactly one of three states:

| state | what is known | in the count? |
|---|---|---|
| **known blank** | the preview rendered it and the ink test found nothing in the overhang | **no** — subtracted |
| **known inked** | the preview rendered it and there is ink out in the band | yes |
| **unexamined** | nothing has looked at it, or it would not render | yes |

```text
displayed = geometric − known_blank = known_inked + unexamined
```

### The displayed number is a CEILING, not a floor

The request that authorised this work called it *"a floor rather than a
total."* That is the wrong way round, and the direction decides the
wording, so it is worth doing the inequality rather than adopting the
phrase.

Let `T` be the number of sheets that really will lose ink. Every known-blank
sheet is definitely not in `T`; every known-inked sheet definitely is; each
unexamined sheet may or may not be. So

```text
known_inked  ≤  T  ≤  known_inked + unexamined  =  displayed
```

The number on the button is the **largest** `T` could be. `known_inked` is
the floor and it is not the number displayed — displaying it would be the
genuinely dangerous mistake, because it would under-report loss on exactly
the sheets nobody has looked at.

A ceiling can only be said with a hedge, and *that hedge is a correction
rather than a weakening*: it is added at the same moment the number stops
being a count of anything measured. See [`ClipClaim`], where each of the
four states carries the strongest sentence that state can support.

## The key: a cached verdict is a CLAIM ABOUT PIXELS, and it must not
## outlive them

`preview::PreviewKey`'s doc comment states the discipline this module
inherits: a cache key is a claim that *"if these are equal, re-rendering
would produce the same image."* A verdict cached under a weaker key is a
verdict about a page that has changed — and it would be **confidently**
wrong, which is worse than no cache at all, because the whole point of the
entry is to remove a warning.

So the key here is not merely as strong as the texture's; it is strictly
stronger, and it is built so that it *cannot* be weaker:

- [`Context`] holds the job-wide half — the annotation scope, the whole
  `Settings`, and the printable rectangle. Its [`Context::preview_key`] is
  the **only** construction site of a `PreviewKey` in the crate, so the
  texture the preview shows is keyed on a value *derived from* this
  context. The texture cache therefore cannot be keyed on anything this one
  is not.
- Each entry additionally pins the sheet's own geometry — the `Placement`
  and the page's size in pdf dimensions — because the verdict depends on
  *where the band falls*, and the same pixels under a different placement
  give a different answer. `PreviewKey` deliberately omits the placement
  (it does not change one pixel of the raster); a verdict that omitted it
  would survive a switch from Fit to 100 % and answer for the wrong band.

### Why an over-strong key is safe and an under-strong one is not

The failure modes are asymmetric, and the key is chosen toward the safe
one deliberately:

- **Too strong** ⇒ a live verdict is dropped ⇒ the sheet counts as
  unexamined ⇒ the number goes *up* ⇒ a warning the operator did not need
  is shown once more, and one more look at the preview removes it.
- **Too weak** ⇒ a stale verdict is trusted ⇒ the number goes *down* ⇒ a
  warning is **removed** on evidence about a different page. That is a
  false claim on the one surface in this application with no undo behind
  it.

Everything ambiguous therefore resolves to "unexamined". That is also why
[`Overhang::Unknown`] — *"the page would not render, so we could not
look"* — is counted as unexamined rather than as knowledge: not being able
to look is not a finding.

## What is NOT keyed, and the hole that leaves

The document's own edit state. `PreviewKey` does not carry an edit
generation, so an unsaved edit does not invalidate the preview texture
either — the preview renders `session.view()` but is keyed on the settings.
That hole is inherited rather than introduced: this cache is exactly as
stale as the pixels it describes and never staler, which is the strongest
property available without changing the texture key. Closing it belongs in
`PreviewKey`, in the commit that gives this crate an edit generation to key
on, and both halves must move together.

## Cost

One `BTreeMap` entry per previewed page — a `Placement`, two `f64` and a
one-byte enum, bounded by the document's page count — and one `Settings`
clone per frame for the context. `claim` is a linear pass over the plan
list of `bool` tests and map lookups, which is the same order as
[`Job::clipped`] itself. **Nothing here renders anything**, which is the
constraint the whole design is built to satisfy: an ink-aware count that
rasterised the job would make opening the dialog cost more than the print.

## Item notes

### `struct Sheet`

Not the plan's position in the send order. A job may print the same
document page twice (uncollated copies) or in a reversed or filtered
sequence, and the verdict is a fact about *this page under this placement*,
not about a position in a list. Two plans naming the same page carry
identical placements by construction — `super::spooler::plan` computes one
placement per page — so the second one inherits the first's verdict
legitimately: it is derived, not invented.

### `fn of`

One function, called by both the write path and the read path, so
the two cannot build the identity differently. A remembered verdict
that could never be found again would look exactly like a preview that
was never opened — a silent, permanent over-count with nothing to say
why.

`page_sizes` is indexed by `plan.index`, the **document** page, and
never by a position in the plan list. That is the same defect
`super::preview::paint` carries a comment about, and it would be
re-introduced here by using the loop counter.

### `fn verdict`

Three ways to get `None`, and they are deliberately indistinguishable
to the caller because they mean the same thing — *no claim can be made
about this sheet*:

1. the context has moved on (different settings, scope or device);
2. the plan names a page that is no longer there;
3. the sheet was never previewed, or was previewed under a different
   placement or page size.
