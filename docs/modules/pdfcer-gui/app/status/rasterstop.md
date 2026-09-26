# `app::status::rasterstop` — saying why zooming in stopped

`OPERATOR_REQUESTS.md` **O186**:

> *"If this error is caused by some other limitation that will always happen,
> zoom should stop at the limit and not end up showing an error - the canvas
> will just stop zooming in and can still function. the error can still be
> shown on the bottom bar so the user has some idea as to why zooming stopped
> short of 1 trillion percent."*

The clauses of that ask are split across modules:

| clause | where |
|---|---|
| a refusal becomes a **remembered** limit | [`crate::render::ceiling::RasterCeiling`] |
| the zoom is **pulled back** to it, once | `crate::render::settle::absorb`'s `learn_raster_ceiling` |
| the limit **binds every later gesture** | [`crate::viewer::zoom_ceiling`]'s fourth parameter |
| *"the error can still be shown on the bottom bar"* | **here** |

Without this module the others produce a `+` button and a Ctrl+wheel that
stop responding with nothing anywhere saying why — a silently-inert control,
which is the defect class this shell exists to refuse. A clamp the operator
cannot account for is worse than the error sentence it replaced, because an
error is at least a report.

## A pure function of state — no store, no retirement rule

The other sentences on the bar's left half each need a store and a rule for
when to forget, and each such rule is a small machine that can be wrong. This
one needs neither, because of what the sentence *means*: **"the zoom is at
this page's measured ceiling right now"** is a question about the current
frame and nothing else. So it retires itself on a zoom out (the predicate
simply stops holding on the next frame), on a page turn (the ceiling is keyed
on the page index, and a page that has refused nothing has none), and on an
edit (the ceiling is keyed on
[`crate::app::state::pageepoch::PageEpochs`]). It cannot be stale against the
wrong document, because there is no state here to be stale.

It therefore **reappears** if he zooms back in to the ceiling, and that is
correct: it is a readout of a condition, the same species as the zoom
percentage beside it, not a notification of an event.

## Rule 4 — the off-canvas half, and there is no on-canvas half

pdfcer declined a zoom the operator asked for and nothing on the page looks
different, so it owes a report. The report is one small line in the status bar
and that is the whole of it: no badge, no tint, no marker, nothing drawn into
the page view, nothing positioned relative to the document.

## Why it is not folded into [`super::decline`]

That module rules that a region zoom clamped by the *derived*
`max_zoom_for_page` ceiling is a **partial grant** rather than a decline,
because the region is still framed and centred and the framing verb raises
`Action::ZoomTo` carrying the clamped number, so the readout states the truth
unaided. Two differences put a *learned* ceiling outside that ruling, either
one decisive:

1. **The readout cannot explain a learned ceiling.** He asked to go further
   *in* and the number did not move at all, and a readout identical before and
   after a gesture explains nothing about the gesture.
2. **A decline is about one command; this is about a direction.** Every
   zoom-in gesture from now on does nothing on this page — the same species of
   fact as [`super::filter`]'s empty-filter note, *why every gesture will do
   nothing* rather than *why that one did*, which is why it is drawn
   immediately after that note and before the decline.

## Item notes

### `const NEAR`

A fraction rather than an absolute, because the ceiling varies by better than
an order of magnitude across page geometries — measured at scale 284,964 on
an E-size sheet against 8,053,069 on a business card — so any fixed epsilon
would be meaningless at one end and a gate at the other.

Generous at one part in a thousand rather than tight at `f32::EPSILON`, and
the asymmetry is deliberate. The clamp lands on the ceiling by way of
[`crate::viewer::clamp_zoom`] and a division by `pixels_per_point`, so the
stored value and the live value are the same number arrived at by two routes
and need not be bit-identical. **The two errors do not cost the same**: too
tight and the sentence silently fails to appear in exactly the state it exists
for — a silence nobody reports, because the operator has no way to know a
sentence was owed — while too loose shows it a thousandth of a percent early,
which is true enough to be unnoticeable.

### `fn a_document_that_has_refused_nothing_says_nothing`

The test that makes the module safe rather than the one that makes it
work. `RasterCeiling` is empty for every document except the handful
zoomed past a rasterizer wall, so a predicate that answered `true` by
accident would put a permanent, false sentence on every file's bar.

### `fn the_sentence_appears_at_the_ceiling_and_not_below_it`

Both halves are asserted together because either alone is satisfied by a
constant: a predicate hard-wired to `true` passes the first and one
hard-wired to `false` passes the second.

### `fn zooming_out_retires_the_sentence_with_nothing_remembering_to`

Asserted rather than argued: "it cannot go stale" is a claim about a
mechanism that does not exist, and the only way to keep such a claim true
is to measure the condition it rests on.

### `fn an_edit_to_the_page_retires_the_sentence`

The invalidation rule [`crate::render::ceiling::RasterCeiling`] owns,
asserted from this side of the boundary: a reader asking with a stale
epoch — or with none — would keep the sentence on a page whose content has
been replaced, where the old measurement says nothing.

### `fn the_density_conversion_is_applied_here_too`

The disclosure-side twin of
`viewer::ceiling::tests::the_learned_ceiling_is_a_raster_scale_and_not_a_zoom`.
Both are needed: if only one reader divided, the shell would clamp at one
zoom and explain itself at another, and on a high-DPI screen the sentence
would be a factor of two away from the number beside it.

### `fn a_nonsense_display_density_does_not_silence_the_disclosure`

A bad density makes the permitted zoom infinite, so the predicate answers
`false` forever: the sentence vanishes in exactly the state it exists for,
and the operator is back at a control that stops responding in silence.
That failure mode is **invisible** — nobody reports a sentence they were
never told was owed.

Every row is asserted unconditionally, with no `||` anywhere: an `||`
between a measurement and an excuse (`at_the_ceiling(&doc, bad) ||
bad.is_nan()`) measures neither, and passes whatever the `NaN` case does.

**Both directions, and the second half is not redundant.** A bad density
must not silence the sentence *and* must not conjure it; the loops falsify
different clauses of the guard:

* the first fails under `.max(f32::MIN_POSITIVE)`, because the permitted
  zoom becomes infinite and nothing ever reaches it;
* the second fails under a guard that checks only `> 0.0` and forgets
  `is_finite`, because an infinite density makes the permitted zoom **zero**
  and the sentence then appears at every zoom on the page — worse than its
  absence, since a line that is always on stops being read.
