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

## Item notes

### `fn habits`

The membership rule and the argument for every inclusion and every
omission live on [`crate::app::prefs::ExportDxfPrefs`], which is the type
this returns; this function is only the projection. It is one struct
literal with **no `..Default::default()`**, so a field added to
`ExportDxfPrefs` is a compile error here rather than a preference written
to disk as its own default and never actually remembered.

⚠ **`scale` is not here, and it is not an oversight.** It is the one
field of `DxfOptions` this window edits that must be derived per page
rather than carried between them.

### `fn scale_disclosure`

Drawn **above** the field rather than below it, because it is the reason
the number in the field is what it is — and a caveat under a control is
a caveat read after the control has been used.

### `fn habits`

The anti-vacuity fixture, and it is asserted rather than trusted. A
`seeded_options` that ignored `remembered` entirely and returned
`DxfOptions::default()` would satisfy every assertion below if the
fixture happened to equal that default. So each field is checked
against the engine's own value here, once, and the whole module fails
loudly if the engine ever moves a default onto this fixture.

That is this project's standing lesson about a check whose input was
chosen for convenience: what is fed in is part of the assertion.

### `fn a_calibrated_page_overrules_the_remembered_units`

The operator habitually exports millimetres. This page was dimensioned
in inches and pdfcer can prove it. If the habit won, the DXF would be
out by 25.4× — and it would open cleanly, measure consistently, and be
wrong, which is the failure this module exists to prevent. The person
who discovers it is whoever cuts from the file.

**Before O196 this could not happen**, because there was no habit: the
window always started from `DxfOptions::default()` and a calibration
only ever overwrote a default. Remembering is what created the ordering
question, so remembering is what owes it a test.

### `fn a_conflicting_page_leaves_the_habit_standing`

Nothing has overruled it: the window is about to say it cannot choose,
and a window that cannot choose a scale has not thereby learned
anything about units either.

### `fn every_suggestion_kind_has_its_own_stable_token`

The `export-dxf-open` trace is machine-read by `ui-verify`, and
[`suggestion_key`] records at length why a `{suggestion:?}` was not good
enough: the `Debug` payload carries a group name that varies with the
document, so a substring check degrades to *"the window opened"* — a
claim satisfied by every build ever shipped, including the one O196
exists to replace.
