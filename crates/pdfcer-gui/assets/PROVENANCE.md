# PROVENANCE — the application icon

Three files sit beside this note and all three are **redistributed** by
pdfcer-gui:

| file | how it ships |
|---|---|
| `pdfcer-gui.ico` | compiled into `pdfcer-gui.exe`'s `.rsrc` section by `pdfcer-gui.rc`, via `build.rs` |
| `pdfcer-gui.rc` | the resource script that does it — source, not shipped, but kept here with what it names |
| `window-icon-64.rgba` | `include_bytes!` in `src/lib.rs`, handed to `eframe` as the window icon |

`cargo-about` generates `THIRD_PARTY_LICENSES.md` from `Cargo.lock` and is
**structurally incapable** of seeing a file that is not a Cargo dependency, so
nothing about these assets would appear there automatically.
`tools/gates/check-shipped-assets.py` is what requires this file to exist and
to state terms — and it did: this note was written because that gate failed the
build on the commit that added the icon, which is the gate working exactly as
designed rather than a formality.

This follows the format of the sibling records in this tree
(`crates/pdfcer-gui/src/icons/assets/PROVENANCE.md`,
`crates/pdfcer-gui/src/app/assets/PROVENANCE.md`) and of `D:\Dev\pdfcer`'s own,
so a reader moving between them meets one convention rather than three.

---

## ★ Read this first: the author is NOT the operator

The sibling record for the ribbon glyph set opens by naming **Ken Mantle** as
creator, because that art came from his own ScripTree icon set and from glyphs
he drew. `shell/commands/catalog.rs` states the rule that follows from it, and
states it as a refusal:

> the icon directory is declared the operator's **own art**, so a new glyph is
> not a build session's to add.

**This icon breaks that rule, and it breaks it on instruction.** The operator,
2026-08-18:

> *"make and add a pdf icon to the exe so it shows as the icon when I associate
> it with pdfs."*

So the authorship is different from every other asset in this repository and
saying so is the entire reason this section is at the top:

- **Source:** authored for pdfcer-gui on 2026-08-18 by a Claude Code build
  session, at the operator's direct request.
- **Creator:** not the operator. Generated art, produced under his instruction
  and owned by him as work made for this project.
- **Licence: the project licence — MIT**, the same grant as the rest of this
  tree (`LICENSE`, `Copyright (c) 2026 Ken Mantle`). No third-party grant is
  reproduced because none is involved.
- **Third-party material: none.** No traced logo, no downloaded glyph, no font
  file, no clip art. Every pixel is computed from geometry in
  `tools/make-icon.py`; there is no image the drawing was made *from*.

### On the red PDF badge, and why it carries no third-party claim

The icon is a white page with a folded corner and a red badge reading **PDF**
over **CE** — the product's own name, split the way it is spelled. The second
line was added on the operator's instruction, 2026-08-18: *"just add CE below
PDF in the same red box."*

The page-and-badge form is deliberately the generic file-type idiom — the same one Chrome,
Firefox, macOS Preview, Okular, SumatraPDF and every file manager use — because
its whole job is to be recognised in Explorer as *the thing that opens PDFs*,
which is what the request asked for.

It is **not** Adobe's mark and reproduces nothing of theirs: no Acrobat "A", no
Adobe logotype, no brand colour lifted from their guidelines, no typeface of
theirs. The red is `#C62828`, chosen to read at 16 px against both light and
dark Explorer backgrounds. "PDF" is the name of an **ISO standard**
(ISO 32000), not a brand, and every letter is drawn here from primitives —
P, D, F and E from rectangles and half-capsules, C from an ellipse ring with
its right flank opened. There is no font in this repository and none was used.

---

## ★ The art is a SCRIPT, and the binaries are outputs

`tools/make-icon.py` is the source. It draws the page, the fold, the badge and
both lines of lettering from geometry, supersamples 4×, and writes every file listed
above plus a review strip at `evidence/app-icon.png`.

Regenerate with:

```bash
python tools/make-icon.py
```

Two consequences worth stating, because they are why it was done this way:

1. **A change is a diff.** A checked-in `.ico` is art nobody can review or
   adjust; a reader who wants the badge two pixels lower has to open a paint
   program and hope. Every number in this icon is named, commented and
   editable.
2. **The two artefacts cannot drift.** `pdfcer-gui.ico` and
   `window-icon-64.rgba` are written from **one** render pass, so the icon
   Explorer shows and the icon in the title bar are the same picture by
   construction rather than by somebody remembering to export twice.

## ★ It is meant to be replaced

This is competent generated art, not drawn art, and it should be treated as a
floor rather than a finish. If the operator wants a real icon:

- **the simple route** — replace `pdfcer-gui.ico` with any multi-size `.ico`,
  delete `tools/make-icon.py`, and amend this file to name the new author and
  terms. Nothing in `build.rs` or `pdfcer-gui.rc` needs to change;
  `window-icon-64.rgba` needs regenerating by whatever produced the new art.
- **the smaller route** — edit the geometry constants at the top of
  `tools/make-icon.py`. Margins, fold size, badge position, stroke weight and
  every colour are named constants in one block.

**Look at `evidence/app-icon.png` after any change.** It shows all seven sizes
at 1:1 on mid grey. An icon that is fine at 256 px and mud at 16 px is the
normal failure and it is invisible from the source — and 16 px is the size an
operator sees most, because it is what a file list shows.

`evidence/embedded-icon-check.png` is the other half of that: the icon
**extracted back out of the built executable** by Windows' own
`ExtractAssociatedIcon`, which is the only evidence that the resource actually
landed rather than that the `.ico` on disk is correct.

## What the `.rc` also carries, and why

`pdfcer-gui.rc` embeds a `VERSIONINFO` block as well as the icon, so the
executable's Properties ▸ Details tab is filled in rather than blank. That
matters for a program an operator associates with a file type: it is what
Windows shows in the *Open with* list, in SmartScreen prompts and in Task
Manager's description column, and a blank one reads as an unidentified program.
Its strings are ASCII on purpose — see the note in that file for the mojibake
that taught it.
