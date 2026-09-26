# `dialogs::export_dxf` — the page's geometry, at a scale somebody can
defend

## The gap this closes

`file.export_dxf` was registered, drawn on File ▸ Export, marked `P3` in
`shell::commands::reach`'s `SCAFFOLDED` list, and its recorded reason was
**"No recorded reason anywhere. Scaffolded by omission, not by decision."**
It was the *first* entry in that list and one of only three with no reason
at all — the second of which, `edit.insert_image`, turned out the same way
yesterday: no blocker, only an entry nobody had looked at.

`pdfcer-core`'s `export::dxf` has shipped the whole time, and the old shell
has the feature (`FEATURES.md`'s `gui` column, which is this project's
acceptance criteria).

## The sentence the whole window is arranged around

`DxfOptions::scale`'s own doc:

> **This is the field the whole feature turns on.** Every generic PDF→DXF
> converter exports at paper scale and says nothing, so a **1:2 detail
> arrives at half size and looks plausible.**

*Looks plausible* is the problem. A DXF at the wrong scale opens cleanly,
measures consistently, and is wrong, and the person who discovers it is
whoever cuts from it.

pdfcer can do better than guess because it already has the operator's own
calibration — the ce dimensions they drew and the group scale they set — and
`suggest_scale_for_groups` is the query that turns that into an answer.

## Three answers, and the window says which one it has

`DxfScaleSuggestion` is deliberately not an `Option<f64>`:

| | the window |
|---|---|
| `Calibrated` | seeds the field, and names **the group the number came from** — a bare figure is a claim the operator cannot check |
| `Uncalibrated` | seeds 1.0 and says in words that this is a **choice rather than a measurement**, and how to make it a measurement |
| `Conflicting` | lists every candidate and makes the operator pick. A sheet with a 1:50 plan and a 1:5 detail is a *correct drawing*; one DXF scale cannot serve both, and only the operator knows which half they are exporting for |

## The PAGE-scoped query, not the document one

`suggest_scale_for_groups` with `dimension_groups_on_page`, never
`suggest_scale`. The engine spells out what the document-wide one costs a
per-page export: *"a sheet set whose page 3 is a 1:5 detail will either
refuse a perfectly unambiguous page-1 export or — worse, when page 1 has no
calibration of its own — **silently export it at page 3's scale**."*

That is the same defect the feature exists to prevent, arriving through the
front door.
