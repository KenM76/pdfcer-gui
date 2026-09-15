# `icons::catalog` — which glyphs exist, and what each one means

The [`Icon`] enum is the whole vocabulary: one variant per drawn glyph, 142 of
them, named for the **role** the icon plays rather than for the artwork. A
re-draw therefore changes one constant in [`super::assets`] and touches no call
site.

## The one key namespace

[`Icon::name`] is the string an `egui_shell::Command` names with
`.with_icon("…")`, and [`Icon::from_key`] is the reverse. There is exactly one
spelling of each key and it lives in `name`; `from_key` searches [`Icon::ALL`]
rather than carrying a second `match`, so the two cannot drift.
`every_name_round_trips_through_from_key` pins it anyway, because "cannot
drift" is a property of today's implementation and the test is a property of
the contract.

## Where a glyph's ruling lives

**In the `.svg`, as an embedded XML comment — not here.** Each asset states
what it depicts and which neighbour it was drawn to stay distinguishable from,
which is where someone editing the art will actually be looking. A variant's
doc comment carries its *role* in one line plus whatever is true of the
**registration** rather than of the drawing: that it retires a recorded
refusal, that it replaces a borrowed glyph, that it was authored out of a
five-way share of one asset.

The asset records what the glyph IS; the variant records why this shell came to
have one. The split falls cleanly between those two subjects, and the reason to
hold it is that a duplicated ruling reads as authoritative in both places and
the two then drift silently — the same failure `Icon::name`'s single-spelling
rule prevents one file over.

Three kinds of ruling recur in the assets, and each is a decision somebody paid
for:

* **"This glyph was authored because a text character had no face."**
  [`Icon::Back`], [`Icon::Close`], [`Icon::ChevronUp`], [`Icon::ChevronDown`]
  each replace a Unicode character verified to render as a tofu box in the
  shipped font stack. A missing glyph is **authored**, never worked around by
  rewording the control.
* **"This glyph must not be that other glyph."** [`Icon::Back`] vs
  [`Icon::ChevronLeft`], [`Icon::ShowPoints`] vs [`Icon::EditObjects`],
  [`Icon::Layers`] vs [`Icon::Combine`]. Each pair states the shape cue that
  keeps them apart at 16 px, and losing that note is how the pair quietly
  converges in a later "consistency" pass.
* **"An icon is a claim."** [`Icon::Signatures`] must not be a seal, badge,
  shield or checkmark, because pdfcer performs no cryptographic verification
  and those shapes read as VALIDATED. [`Icon::Fonts`] must not be a pencil or
  an I-beam, because the Fonts panel writes nothing. A glyph reaches the
  operator's eye before the panel's first line does.

## This file is near the size gate

`catalog/mod.rs` is **1,313 lines** against `check-file-size.sh`'s 1,500-line
limit (R2). About 117 variants have never been through the de-duplication
described below, so there is room — but a variant added without moving one out
spends it.

### How to reclaim lines, measured rather than by eye

Per **paragraph** of a variant's doc comment, measure the fraction of its
five-letter-and-longer words that appear anywhere in that icon's `.svg`
comment. At **≥ 0.75** the paragraph is a restatement and is deleted; below it,
keep it. Then check every backticked identifier in a deleted paragraph still
exists in either the asset or what remains of the variant — near-misses are
usually re-spellings of the same path rather than losses.

One caution the method earned: **an asset that scores thin is not a target to
paste into.** A thin asset means its ruling was never written, which is a
drawing job, not a text move.
