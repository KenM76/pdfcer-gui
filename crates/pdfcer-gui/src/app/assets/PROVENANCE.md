# PROVENANCE — the blank-document template

`blank-a4.pdf` beside this note is **redistributed** by pdfcer-gui: it is
compiled into `pdfcer-gui.exe` by an `include_bytes!` in
`crates/pdfcer-gui/src/app/blank.rs`, so every operator who is handed a binary
is handed these 443 bytes. `cargo-about` generates `THIRD_PARTY_LICENSES.md`
from `Cargo.lock` and is **structurally incapable** of seeing a file that is
not a Cargo dependency, so nothing about this asset would appear there
automatically. `tools/gates/check-shipped-assets.py` is what requires this
file to exist and to state terms.

This note follows the format of the sibling record at
`crates/pdfcer-gui/src/icons/assets/PROVENANCE.md`, which in turn follows
`D:\Dev\pdfcer`'s own asset provenance records, so that a reader moving between
the three meets one convention rather than three.

- **Source:** none. The file was **authored from scratch for pdfcer-gui** on
  2026-08-14, by emitting the object bodies and cross-reference table
  described below and computing the byte offsets exactly. It is not a copy,
  an export, a trace, or a reduction of anybody's document — there was no
  input file at any point.
- **Creator:** written for Ken Mantle's pdfcer-gui, in his tree, as part of
  `file.new`.
- **Licence: the project licence — MIT**, the same grant as the rest of this
  tree (`LICENSE`, `Copyright (c) 2026 Ken Mantle`). This is **own work**;
  there is no third-party grant to reproduce and no upstream attribution to
  carry.
- **Changes made by pdfcer:** not applicable — there is no upstream to have
  changed.

## Why there is no `about.hbs` entry and no About-dialog entry

The same reason the icon set has neither, and the gate encodes it as the
own-work exemption from its checks 4 and 5: `about.hbs`'s static epilogue is
the route by which a **third-party** grant reaches the generated
`THIRD_PARTY_LICENSES.md`, and the About dialog is the route by which it
reaches an operator who never opens the folder. Own work needs neither. The
`LICENSE` file already ships in the portable folder and already covers this
file, and adding a section that reproduced pdfcer's own MIT grant a second
time under a heading saying "third party" would make the notice say something
false.

The engine's copy of that checker records the day check 4 first ran and
flagged the GUI icon set: *"That was a false positive, and a gate that fires
on a correct state is one people learn to ignore."* This directory is the
second instance of the same correct state.

## The file, and exactly what is in it

`blank-a4.pdf` — 443 bytes, one page, nothing drawn on it. This note **covers
every file in this directory**.

It is a classic cross-reference-table PDF (ISO 32000-1 §7.5.4), not a
cross-reference *stream*, because the whole point of the asset is that a
person can read it: `cat` it and every byte is legible except the four-byte
binary comment on line 2.

| obj | contents |
|---|---|
| — | `%PDF-1.7`, then the §7.5.2 binary-comment line `%` + four bytes above 127 |
| 1 | `<< /Type /Catalog /Pages 2 0 R >>` |
| 2 | `<< /Type /Pages /Kids [3 0 R] /Count 1 >>` |
| 3 | `<< /Type /Page /Parent 2 0 R /MediaBox [0 0 595.276 841.89] /Resources << >> /Contents 4 0 R >>` |
| 4 | a **zero-length** content stream — a page that draws nothing |
| — | `xref` (5 twenty-byte entries), `trailer << /Size 5 /Root 1 0 R >>`, `startxref`, `%%EOF` |

Four deliberate choices, each of which a reader would otherwise have to guess
at:

1. **`/MediaBox [0 0 595.276 841.89]`** is ISO 216 **A4** — 210 × 297 mm at
   72 units to the inch. The decision to make A4 the default, and the
   reference applications it was taken from, is argued in full at
   `crates/pdfcer-gui/src/app/blank.rs`; it is not repeated here, because a
   provenance note's job is what the bytes are and where they came from.
2. **`/Resources << >>`** is present and empty. ISO 32000-1 Table 30 makes
   `/Resources` required but inheritable; a page that draws nothing needs no
   resources, and stating an empty dictionary is more honest than relying on
   inheritance from a `/Pages` node that also has none.
3. **`/Contents` points at a real stream of length zero**, rather than being
   omitted. `/Contents` is optional (§7.7.3.3) and a page without it is legal,
   but every real producer emits one, so this template exercises the same
   renderer path a document from SolidWorks or Word does. A template that took
   a code path nothing else takes would be a template that proves less than it
   appears to.
4. **Every `xref` entry is exactly 20 bytes** (`%010d %05d n \n`), which
   §7.5.4 requires. This repository's `.gitattributes` header records what
   destroys that property — `core.autocrlf` inflating an entry to 21 bytes at
   `git add` time — and the `*.pdf binary` rule there is what protects this
   file. Do not remove it.

## ★ What would change this

Any file added here that is **not** own work — a template exported from
another application, a sample document, a stationery PDF from a vendor —
stops being covered by the paragraph above. It then needs its own provenance
entry naming its actual licence and a section in `about.hbs` reproducing that
licence's required notice, because the `LICENSE` file that covers this
directory today would no longer cover all of it. The gate would catch the
missing `about.hbs` citation only if this note also stopped claiming own work,
so **update this file first and the directory second.**

Editing `blank-a4.pdf` in place is the other way to make this note false, and
it is easier to do by accident: the cross-reference table stores absolute byte
offsets, so changing one character of one object dictionary invalidates every
offset after it. The file is not meant to be hand-edited. If a second template
is ever needed — a size picker is the obvious reason — emit it whole, the way
this one was, and add a row to the table above.
