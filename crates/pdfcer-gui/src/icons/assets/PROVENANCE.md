# PROVENANCE — the pdfcer icon set

The SVG files beside this note are **redistributed** by pdfcer-gui: each one
is compiled into `pdfcer-gui.exe` by an `include_str!` in
`crates/pdfcer-gui/src/icons/assets.rs`, so every operator who is handed a
binary is handed this art. `cargo-about` generates `THIRD_PARTY_LICENSES.md`
from `Cargo.lock` and is **structurally incapable** of seeing a file that is
not a Cargo dependency, so nothing about these assets would appear there
automatically. `tools/gates/check-shipped-assets.py` is what requires this
file to exist and to state terms.

This note follows the format of `D:\Dev\pdfcer`'s own asset provenance records
(`crates/pdfcer-render/assets/fonts/PROVENANCE.md`,
`crates/pdfcer-core/assets/models/ocrs/PROVENANCE.md`) so that a reader moving
between the two repositories meets one convention rather than two.

- **Source:** `D:\Dev\ScripTree\icons\*.svg`, plus glyphs authored directly
  for pdfcer in the same style.
- **Creator:** Ken Mantle (the operator). He owns both projects.
- **Licence: the project licence — MIT**, the same grant as the rest of this
  tree (`LICENSE`, `Copyright (c) 2026 Ken Mantle`). This is the operator's
  **own art**; there is no third-party grant to reproduce and no upstream
  attribution to carry.
- **Changes made by pdfcer:** files copied from ScripTree are byte-identical
  apart from being **renamed** to pdfcer's *role* rather than ScripTree's shape
  name (`icon-folder.svg` → `folder.svg`, and so on). Glyphs with no ScripTree
  ancestor were drawn for pdfcer.

## Why there is no `about.hbs` entry, and why that is not an oversight

`about.hbs` is the `cargo-about` template whose static epilogue is the only
route by which a **non-Cargo** asset's licence reaches the generated
`THIRD_PARTY_LICENSES.md`, and therefore the only route by which it reaches
someone who was given a binary rather than the source. That route exists for
**third-party** grants that must be reproduced.

Own work needs none of it: the `LICENSE` file already ships in the portable
folder and already covers this art. Adding a section that reproduced pdfcer's
own MIT grant a second time, under a heading saying "third party", would make
the notice file say something false.

`tools/gates/check-shipped-assets.py` therefore exempts a directory whose
`PROVENANCE.md` declares own work — and the exemption is deliberate rather
than incidental. The engine's copy of that checker records the day check 4
first ran and flagged the GUI icon set: *"That was a false positive, and a
gate that fires on a correct state is one people learn to ignore."*

## The operator confirmation that closed the licensing question

The primary record is the module header of
`crates/pdfcer-gui/src/icons/assets.rs` §1–§4 — it is long, it is the thing a
reader of the code meets first, and it is not duplicated here. What it records,
in one paragraph:

`docs/ui_specs/icon-set-and-toolbar.md` §7.2 required the provenance of this
set to be **confirmed, not assumed**, before any art was bundled — the open
question being whether the ScripTree glyphs were drawn from scratch or adapted
from a third-party icon pack (Feather, Lucide, Font Awesome, …) whose own
licence would then travel with them. The operator answered directly on
2026-08-02:

> "Scriptree icons are mine, use from it what makes sense and create new ones
> in its style when necessary, try to make them close to what inkscape and
> Adobe use for similar commands without running into copyright issues."

Two consequences bind every future asset dropped into this directory:

1. **Metaphor-level resemblance is allowed; asset-level copying is not.** A
   magnifier means zoom and a curved arrow means undo, in every application
   that has ever had a toolbar; matching that convention is what makes a
   ribbon legible to someone arriving from Acrobat or Inkscape. Tracing,
   importing or "adapting" any Adobe or Inkscape SVG, icon font or screenshot
   is forbidden outright. Every glyph here is constructed from primitives, and
   every asset carries an XML comment naming the concept it depicts.
2. **The rule is stricter than copyright law requires** for simple geometric
   glyphs, deliberately: it removes the question rather than leaving a
   judgement call in a file nobody will re-examine.

## The files

Every `.svg` file in this directory, including every one added to it in
future. This note **covers the whole directory** — a per-glyph table would be
one row per file saying the same sentence, and the thing that varies (what
the glyph depicts, and which neighbouring glyph it was drawn to stay
distinguishable from) is recorded in each asset's own embedded XML comment,
which is where someone editing the art will actually be looking.

`crates/pdfcer-gui/src/icons/catalog.rs` holds exactly one more `Icon` variant
than there are assets here, because `folder.svg` is deliberately shared by two
roles (Open, and the font-folder control).
`tests::only_the_folder_asset_is_shared` asserts that is the only sharing in
the set — which is the durable statement, and the reason no count appears in
this paragraph.

★ **Deliberately no totals anywhere in this note.** It carried "79 files" and
"80 variants" from 2026-08-14 until 2026-08-21, by which point the real figures
were 85 and 86. Nothing failed, because no gate reads them and
`check-shipped-assets.py` accounts for the directory by *contents* rather than
by count. A number in prose that nothing verifies is a number that rots, and
this project has spent several corrections on exactly that failure. Count with
`ls *.svg | wc -l` if you need a figure; do not write one down here.

## ★ What would change this

Any asset added here that is **not** the operator's own work — a glyph from an
icon pack, a vendor mark, a traced shape — stops being covered by the paragraph
above. It then needs its own provenance entry naming its actual licence, and a
section in `about.hbs` reproducing that licence's required notice, because the
`LICENSE` file that covers this directory today would no longer cover all of
it. The gate would catch the missing `about.hbs` citation only if this note
also stopped claiming own work, so **update this file first and the directory
second.**
