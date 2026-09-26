# `canvas::textedit::plan` — turning a caret and two strings into an
`EditRequest` the engine can answer

**One function, called from one place**, and everything a text commit
decides is decided here: which show operator to address, whether to name it
by provenance pin or by its text, how the rest of the line is allowed to
move, and — since `OPERATOR_REQUESTS.md` **O142** — whether this shell is
entitled to let the engine choose an occurrence at all.

## The seam against [`super`]

[`super`] is about a caret and a draft — where the operator clicked, what he
typed, when a draft opens and closes. This file is about **one request to
`pdfcer-core`**. [`plan`] and [`Plan`] are re-exported from [`super`], so
every caller looks for them in one place regardless.

## What a reader should take away before changing anything here

The three things [`plan`] derives — all from `(page_text, run)` — are the
provenance pin, the matrices in force at the run's first glyph, and the
block's alignment. Each is re-derived from the page **as it now stands**
rather than carried on the `Anchor`, and that is not incidental: a value
sampled when the operator clicked goes stale the moment anything rebuilds
the page, which is `DEFECTS.md` D4b.

⚠ **The single most dangerous line in this file is the one that clears
`pinned_span`.** The pin is the only disambiguator `EditRequest` carries;
dropping it hands the choice of *which* occurrence to edit to the engine's
scan order. [`Plan::occurrences`] carries the whole argument for why that is
never permitted, and `super::glyphwall` holds it as tests over two fixtures
— one where the edit must land, one where it must be refused.
