# `text::export_dxf` — the words the Export-DXF window shows

## Rule 15, and it decides a sentence here

The scale this window offers is inferred from the **ce dimensions** the
operator has drawn — the ones pdfcer authors. It is *not* read from **pdf
dimensions**, the CAD-exported page content pdfcer reads and must not alter,
and it could not be: those are anonymous vector geometry with no recorded
measurement. So the copy says *"the dimensions you have drawn on it"*, which
is both unambiguous and the honest boundary — an operator who has drawn none
is told pdfcer has no evidence rather than being given a number.

## The one sentence this whole window exists for

`pdfcer-core`'s own doc on `DxfOptions::scale`:

> **This is the field the whole feature turns on.** Every generic PDF→DXF
> converter exports at paper scale and says nothing, so a **1:2 detail
> arrives at half size and looks plausible.**

*Looks plausible* is the whole problem. A DXF at the wrong scale opens
cleanly, measures consistently, and is wrong — and the person who finds out
is whoever cuts from it. Everything below is arranged so that the scale is
either stated with its evidence, or stated as unknown.

## Three answers, never two

`DxfScaleSuggestion` is deliberately not an `Option<f64>`, and this catalog
keeps its three cases apart because they ask different things of an
operator:

| | what it means | what the window does |
|---|---|---|
| `Calibrated` | every calibrated group agrees | states the number **and the group it came from** |
| `Uncalibrated` | nothing on this page carries a scale | says pdfcer has no evidence, and that 1:1 is a *choice* rather than a finding |
| `Conflicting` | calibrated groups disagree | lists every candidate and makes the operator pick |

Collapsing `Uncalibrated` into "1.0" is exactly the silent paper-scale
export the feature exists to beat.

## Item notes

### `fn an_uncalibrated_page_is_not_told_a_number`

The assertion this catalog exists for. `pdfcer-core` names the failure it
prevents — every generic converter exports at paper scale and says
nothing, so a 1:2 detail arrives at half size *looking plausible* — and
the only defence is a sentence that refuses to present a default as a
finding.

### `fn skipped_text_and_unreadable_text_are_different_sentences`

`skipped` is *you asked*; `unreadable` is *pdfcer could not read it*. The
second is a fact about the source PDF and the reason labels the operator
can see on screen are absent from the file — and rolling them together
would let it hide inside a sentence about their own choice.

### `fn a_skipped_picture_names_the_formats_limit_not_pdfcers`

*"The format has no way to carry a raster"* rather than *"images are not
supported"*: the first is a fact about DXF that no future version of
pdfcer will change, and the second reads as a gap somebody might fix.

### `fn intro`

Says what a DXF **is not**, first. A drafter opening one in SOLIDWORKS
expects the drawing; what arrives is its vector geometry, and the sentence
that saves a support question is the one about what was left behind.

### `fn scale_from_group`

The group's name is in the sentence because the number alone is a claim the
operator cannot check. *"1 paper unit is 50 real units"* is unverifiable;
*"from the group Site plan"* points at something they set up themselves and
can go and look at.

`agreeing` above one is stated as corroboration rather than as a second
finding: two groups agreeing is the same answer, arrived at twice, and
saying so is worth a clause because it is the case an operator is most
entitled to trust.

### `fn scale_uncalibrated`

**The most important string in this catalog.** The alternative — defaulting
to 1.0 and saying nothing — is precisely what `pdfcer-core` describes every
generic converter as doing, and its consequence is a detail arriving at the
wrong size while looking perfectly ordinary.

So it says three things: that pdfcer does not know, that 1:1 is therefore a
**choice** rather than a finding, and what the operator can do to make it a
finding instead.

### `fn scale_conflicting`

Not refused, and not resolved by pdfcer picking one. A sheet holding a 1:50
plan and a 1:5 detail is a **correct drawing**, and the disagreement is a
true statement about it: one DXF scale cannot serve both. The operator is
the only one who knows which half they are exporting for.

### `fn units_name`

pdfcer writes only inches and millimetres, and `DxfUnits::for_unit` maps
feet onto inches and metres onto millimetres — *"the NUMBERS stay exact
either way … this choice affects only what the header declares"*. The
wording therefore names what the file will **say it is**, not what the
operator measured in, because those legitimately differ and only the first
is what this control sets.

### `fn fit_arcs_hint`

The engine measured it: *"not recognising them is what produced a measured
**767 KB for forty washers**."* PDF has no arc primitive, so every hole and
fillet arrives as cubic Béziers, and a converter that emits them as splines
produces a file that is enormous and that no CAD package will let you snap
to a centre in.

### `fn no_geometry`

Reachable while the page is still being read, and on a page whose content
streams could not be resolved. Says which, because *"not yet"* and *"not at
all"* are different situations and only the first is worth waiting for.

### `fn exported`

The disclosure half, and the `skipped` clauses are the reason it exists.
`pdfcer-core` states the case in its own field doc:

> an operator whose drawing was half annotation gets a DXF that looks like
> the geometry went missing, and *"the labels are not in this file"* is a
> sentence they need **before** they open it in SOLIDWORKS, not after.

`skipped_text` and `unreadable_text` are kept apart, exactly as the engine
keeps them apart, because they ask different things:

- **skipped** — the operator turned text off. Nothing is wrong.
- **unreadable** — pdfcer *could not read it*: no font resolver in scope, or
  an `Identity-H` encoding with no `/ToUnicode` whose codes map to nothing.
  That is a fact about the source PDF and the reason a DXF is missing labels
  the operator can plainly see on screen.

Rolling them together would let the second hide inside the first, which is
the failure mode the engine wrote a paragraph to prevent.
# Why it takes the ENGINE's outcome rather than eight counts

It was written as eight `usize` parameters, on this catalog's usual rule
that a wording function takes primitives so it can be tested without the
engine. Clippy's argument budget refused it, and the refusal was right for a
reason the rule does not cover: **these eight are not independent facts.**
They are one value — what the writer did — and a caller assembling them by
hand has eight chances to pass `skipped_text` where `unreadable_text`
belongs, which is exactly the pair whose confusion this function exists to
prevent.

`DxfOutcome` is `Default`, so the tests below still construct one in a line
without touching a document.
