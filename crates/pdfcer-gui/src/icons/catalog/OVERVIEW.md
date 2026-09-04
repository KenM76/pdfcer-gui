# icons::catalog — which glyphs exist, and what each one means

The [`Icon`] enum is the whole vocabulary: one variant per drawn glyph,
named for the **role** the icon plays rather than for the artwork, so a
future re-draw changes one constant in [`super::assets`] and touches no
call site.

Salvaged from `D:\Dev\pdfce\crates\pdfce-gui\src\icons.rs` (Class A,
`SALVAGE.md`). Every variant's doc comment is carried across, because
several of them are not descriptions at all — they are *rulings*. Three
kinds recur and each one is a decision somebody paid for:

* **"This glyph was authored because a text character had no face."**
  [`Icon::Back`], [`Icon::Close`], [`Icon::ChevronUp`],
  [`Icon::ChevronDown`] each replace a Unicode character that was
  VERIFIED to render as a tofu box in the shipped font stack. The
  operator's standing ruling (2026-08-06) is that a missing glyph is
  **authored**, not worked around by rewording the control.
* **"This glyph must not be that other glyph."** [`Icon::Back`] vs
  [`Icon::ChevronLeft`], [`Icon::ShowPoints`] vs [`Icon::EditObjects`],
  [`Icon::Layers`] vs [`Icon::Combine`]. Each pair states the shape cue
  that keeps them apart at 16 px, and losing that note is how the pair
  quietly converges in a later "consistency" pass.
* **"An icon is a claim."** [`Icon::Signatures`] must not be a seal,
  badge, shield or checkmark, because pdfcer performs no cryptographic
  verification and those shapes read as VALIDATED. [`Icon::Fonts`] must
  not be a pencil or an I-beam, because the Fonts panel writes nothing.
  A glyph reaches the operator's eye before the panel's first line does.

## The one key namespace

[`Icon::name`] is the string an `egui_shell::Command` names with
`.with_icon("…")`, and [`Icon::from_key`] is the reverse. There is
exactly one spelling of each key and it lives in `name`; `from_key`
searches [`Icon::ALL`] rather than carrying a second `match`, so the two
cannot drift. `every_name_round_trips_through_from_key` pins it anyway,
because "cannot drift" is a property of today's implementation and the
test is a property of the contract.

---

## ★★★ The real seam, and why this was only half of it

`catalog/mod.rs` reached **1,495 of 1,500** on the day five glyphs were
added, and the next `Icon` variant would have forced a split. Moving the
header buys back about forty lines — one variant's worth — so it is a
reprieve rather than a fix, and the fix is worth stating here so the next
session does not spend the reprieve and then improvise.

**The bulk of this file is per-variant RULINGS** — *"this glyph must not be
that glyph, and here is the shape cue that keeps them apart"*, *"an icon is
a claim, and this one may not make it"*. They are the most valuable prose in
the icon set and they are **in the wrong file**.

`assets/PROVENANCE.md` already says where they belong: each asset carries an
embedded XML comment naming what it depicts and which neighbour it was drawn
to stay distinguishable from, *"which is where someone editing the art will
actually be looking."* The 2026-09-04 batch was authored that way from the
start; the older variants predate the convention.

⇒ **The seam is: the ruling lives with the art, and the variant carries a
one-line role plus a pointer to it.** That empties most of this file into the
forty-odd assets whose rulings they are, puts each one in front of the person
who could break it, and leaves the enum reading as what it is — a list of
roles. It is a migration rather than a split, which is why it was not done
under deadline pressure at 1,495 lines.

★ Until it happens, treat this file as full. A variant added here without
moving one out is a variant added to a file that is already at its limit.
