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
