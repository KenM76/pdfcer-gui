# `app::status::anomalies` — one reading of `load_anomalies()`, for two
surfaces

The derivation behind the status bar's *"this file contradicted itself"* line
and the Document-properties list underneath it. It counts, orders and words
nothing itself — [`crate::text::anomalies`] holds the words — but it decides
**which anomalies are reported, in what order, and how they are grouped**,
and it decides that **once**.

## Why the derivation is split out rather than written at each site

This is the same seam, for the same reason, as [`super::notes::findings`],
whose header states it plainly:

> Split out … so that the status bar's one line and the Render-diagnostics
> dialog's list are the *same ten decisions* … Two tables would agree on the
> day they were written and disagree the first time an eleventh counter was
> added to one of them, and the symptom would be two surfaces describing one
> raster differently, which is the worst available outcome for a *diagnostic*.


Every word of that transfers. `LoadAnomaly` is `#[non_exhaustive]`; a fifth
variant is not a hypothetical, it is the thing
`tools/gates/check-engine-api-drift` exists to catch. One match statement is
one place to teach.

It lives under `app::status` rather than beside the document state because
[`super::notes`] set that precedent and `crate::dialogs::diagnostics` already
reaches across for it. A second convention for the same shape would cost a
reader more than the slight misfiling does.

## The lifetime question, answered honestly

Two of this module's neighbours in [`super::disclosure`] — the fill and edit
disclosures — are keyed on [`crate::app::state::OpenDoc::edit_epoch`] and
**retire on the next edit**. That is right for them: they describe *something
the operator just did*, and a later edit moves the document past it.

A load anomaly is not that. It is true of the **file as it was opened**, and
it stays true for as long as that document is open — through every edit,
every undo, and every save. Keying it on `edit_epoch` would delete a
permanent fact the first time the operator nudged a line, which is precisely
the failure decision 145 was written against: the operator would be told once
and then quietly un-told.

So there is **no epoch key here, and no cached state anywhere**. The census
is recomputed from `doc.session.document().load_anomalies()` on the frame it
is drawn, exactly as [`super::disclosure::recovered_disclosure`] recomputes
from `Document::recovery()`. The slice lives on the `Document`, is populated
at load, and is replaced wholesale when a different document is opened into
the tab — so there is nothing to clear, nothing to invalidate, and no way for
one file's anomalies to be shown against another's. State that must be
cleared is state that will one day be shown against the wrong document; this
has none.

⚠ The list is short by construction — one entry per contradiction in the file
— so recomputing per frame is a walk over a handful of items, not a parse.
The one file that motivated the feature has two.

## `recovery()` and `load_anomalies()` are different questions

The shell already discloses `Document::recovery()`, and it would be easy to
assume this is the same fact twice. It is not, and the two are disjoint in
both directions:

| | `recovery()` | `load_anomalies()` |
|---|---|---|
| fires when | the stored cross-reference table could not be parsed and pdfcer rebuilt the index by scanning | any object contradicted itself, **including on a file whose index was perfect** |
| the operator's file | index was fine | doubled `/PageMode`, and a `/Metadata` stream with no `/Length` |

The file behind engine decision 145 has a **sound xref** and would light
neither the status line nor the Properties note that existed before today.
That is the gap this module closes.
