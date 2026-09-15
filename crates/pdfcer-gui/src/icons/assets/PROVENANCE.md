# PROVENANCE — the pdfcer icon set

The SVG files beside this note are **redistributed** by pdfcer-gui: each is
compiled into `pdfcer-gui.exe` by an `include_str!` in
`crates/pdfcer-gui/src/icons/assets.rs`, so every operator handed a binary is
handed this art. `cargo-about` generates `THIRD_PARTY_LICENSES.md` from
`Cargo.lock` and is structurally incapable of seeing a file that is not a Cargo
dependency, so nothing about these assets reaches it automatically.
`tools/gates/check-shipped-assets.py` is what requires this file to exist and
to state terms.

The format follows `D:\Dev\pdfcer`'s own asset provenance records
(`crates/pdfcer-render/assets/fonts/PROVENANCE.md`,
`crates/pdfcer-core/assets/models/ocrs/PROVENANCE.md`) so a reader moving
between the two repositories meets one convention rather than two.

- **Source:** `D:\Dev\ScripTree\icons\*.svg`, plus glyphs authored directly for
  pdfcer in the same style.
- **Creator:** Ken Mantle (the operator). He owns both projects.
- **Licence: the project licence — MIT**, the same grant as the rest of this
  tree (`LICENSE`, `Copyright (c) 2026 Ken Mantle`). This is the operator's
  **own art**; there is no third-party grant to reproduce and no upstream
  attribution to carry.
- **Changes made by pdfcer:** files copied from ScripTree are byte-identical
  apart from being **renamed** to pdfcer's *role* rather than ScripTree's shape
  name (`icon-folder.svg` → `folder.svg`). Glyphs with no ScripTree ancestor
  were drawn for pdfcer.

This note **covers every file in this directory**, including every file added
to it in future.

## Why there is no `about.hbs` entry

`about.hbs` is the `cargo-about` template whose static epilogue is the only
route by which a **non-Cargo** asset's licence reaches the generated
`THIRD_PARTY_LICENSES.md`, and therefore the only route by which it reaches
someone given a binary rather than the source. That route exists for
**third-party** grants that must be reproduced.

Own work needs none of it: the `LICENSE` file already ships in the portable
folder and already covers this art. A section reproducing pdfcer's own MIT
grant a second time, under a heading reading "third party", would make the
notice file say something false.

`check-shipped-assets.py` therefore exempts a directory whose `PROVENANCE.md`
declares own work, and that exemption is deliberate: a gate that fires on a
correct state is one people learn to ignore.

## The licence grant this record rests on

`D:\Dev\pdfcer\docs\ui_specs\icon-set-and-toolbar.md` §7.2 required the
provenance of this set to be **confirmed, not assumed**, before any art was
bundled — the open question being whether the ScripTree glyphs were drawn from
scratch or adapted from a third-party icon pack (Feather, Lucide, Font
Awesome, …) whose own licence would then travel with them. The operator
answered on 2026-08-02:

> "Scriptree icons are mine, use from it what makes sense and create new ones
> in its style when necessary, try to make them close to what inkscape and
> Adobe use for similar commands without running into copyright issues."

Two consequences bind every asset dropped into this directory:

1. **Metaphor-level resemblance is allowed; asset-level copying is not.** A
   magnifier means zoom and a curved arrow means undo in every application that
   has ever had a toolbar, and matching that convention is what makes the
   ribbon legible to someone arriving from Acrobat or Inkscape. Tracing,
   importing or "adapting" any Adobe or Inkscape SVG, icon font or screenshot
   is forbidden outright. Every glyph here is constructed from primitives, and
   every asset carries an XML comment naming the concept it depicts.
2. **The rule is stricter than copyright law requires** for simple geometric
   glyphs, deliberately: it removes the question rather than leaving a
   judgement call in a file nobody will re-examine.

The full style contract is the module header of
`crates/pdfcer-gui/src/icons/assets.rs` §1–§4, which is what a reader of the
code meets first and is not duplicated here.

## What varies per file, and where it is recorded

Not here. What the glyph depicts, and which neighbouring glyph it was drawn to
stay distinguishable from, lives in each asset's own embedded XML comment —
where someone editing the art will actually be looking. A per-glyph table in
this file would be one row per file repeating the same licensing sentence.

`icons::catalog` holds one more `Icon` variant than there are assets here,
because `folder.svg` is deliberately shared by two roles (Open, and the
font-folder control). `icons::catalog::tests::only_the_documented_assets_are_shared`
asserts that is the set's only sharing, and
`every_declared_share_is_still_a_share` asserts the blessing has not outlived
its subject.

**Write no totals in this note.** Nothing verifies a count in prose, and
`check-shipped-assets.py` accounts for the directory by *contents* rather than
by count, so a number written here rots silently. Use `ls *.svg | wc -l` if you
need a figure.

## What would change this

Any asset added here that is **not** the operator's own work — a glyph from an
icon pack, a vendor mark, a traced shape — stops being covered by the grant
above. It then needs its own provenance entry naming its actual licence, and a
section in `about.hbs` reproducing that licence's required notice, because the
`LICENSE` file that covers this directory today would no longer cover all of
it.

The gate catches the missing `about.hbs` citation only once this note stops
claiming own work, so **update this file first and the directory second.**
