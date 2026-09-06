# `subset-font-floor.pdf` — a page whose font carries only the letters it prints

<!-- old-name-exempt-file: this whole document is about a fixture whose one embedded
font is NAMED `SUBSET+pdfceSubsetDemo`, and every occurrence of the old spelling below
is that identifier or the engine's own command-line output quoting it. It is data inside
a PDF, not prose about this project: the engine deliberately stopped its rename at the
file-format boundary, because changing a resource name would alter the bytes of every
document pdfcer has ever produced and break round-tripping with them. Marking eight
individual lines would bury the account in machinery, which is the case this gate's
file-level exemption exists for. If this file ever stops being about that fixture, the
marker goes with the subject. -->

**Copied on 2026-09-05 from `D:\Dev\pdfcer\fixtures\synthetic\text\subset-simple-embedded.pdf`**,
byte for byte, and renamed for what this repository uses it for. The engine
generates it with `tools/gen-subset-font-fixtures.py` (in `D:\Dev\pdfcer`); that
script's own header is the authoritative account of how it is built and why.

1,721 bytes. One page, 612 × 792 pt, one text run — `ABC` at
`[72.0, 588.0, 158.4, 636.0]`, so its centre is **(115.2, 612.0)** in PDF user
space. One font: `SUBSET+pdfceSubsetDemo`, a simple TrueType with a real
`/FontFile2` and a six-uppercase-letter subset tag.

## Why this repository needs a fixture of its own for it

`OPERATOR_REQUESTS.md` **O141**, and the check that drives it,
`tools/ui-verify/src/checks/refused_character_face.rs`.

> *"if the character isn't available in a pdf are we able to change to a
> different font?"* — the operator, 2026-09-05

That question is about a wall an operator meets on documents somebody else
produced: a producer embeds only the letters the page prints, so a character the
page never used has no code in the font and pdfcer refuses to write one.
**Nothing else in `fixtures/` can reach it**, and it is not an accident of what
happens to be here — every other fixture in this directory is either a
non-embedded standard-14 face (whose `WinAnsiEncoding` covers a `€` at code 128
and therefore accepts the edit), a fully embedded non-subset face (the
`class.embedded && class.subset` floor never fires), or a symbolic built-in-cmap
face that refuses **every** edit for an unrelated reason and offers no remedy.

A check driven against any of those would be measuring something else and would
be unable to fail, which is exactly the shape of report this project has filed
by accident three times in one day.

## What the engine actually does with it, measured 2026-09-05

Not reasoned about — run, with `pdfcer.exe` 0.40.0:

```text
edit-text --page 1 --find "ABC" --replace "€"
  → refused: R-INV-1 (embedded-subset floor): character U+20AC '€' maps to
    code 128 which font 'SUBSET+pdfceSubsetDemo' (an embedded SUBSET) does not
    already carry on this page                                        (exit 9)

edit-text --page 1 --find "ABC" --replace "CBA"
  → OK, base_font=SUBSET+pdfceSubsetDemo, glyph_source=Embedded

format-text --page 1 --find "ABC" --set-font Helvetica
  → set_font=SUBSET+pdfceSubsetDemo->Helvetica

edit-text --page 1 --find "ABC" --replace "€"           (on the swapped file)
  → OK, base_font=Helvetica
extract-text --pages 1
  → €
```

Four lines, and they are the four states the driven check walks: a refusal that
names the character, an edit the same font accepts, a face swap, and the
character landing. **The `Refusal` the first line raises carries
`character: Some('€')` and `trigger: RInvTrigger::TargetAbsent`** — the two
fields `panels::properties::refusedchar` reads.

## Its relation to the operator's own file

`C:\Users\Ken\OneDrive\pdfTests\apartment work - signed.pdf` shows the identical
refusal — `this font has no glyph for '€'` on `AAAAAA+Arimo-Bold`, an 8,640-byte
subset holding about thirty letters — and **cannot be used by a driven check**,
for a reason that has nothing to do with fonts: its page 2 is written one show
operator per glyph, so the caret refuses before the font question is ever asked
(`OPERATOR_REQUESTS.md` O140). The engine's command line reaches it because it
searches the whole page; a click cannot.

⇒ So the fixture is the portable floor and his file is the subject. Any claim
this check makes should be re-measured against his document with `pdfcer.exe`
before it is repeated to him.

## Do not "improve" it

The subset tag, the embedded `/FontFile2` and the three-letter run are each
load-bearing:

- **drop the six-uppercase-letter tag** and `is_subset_tag` says no, the floor
  never fires, and the check goes green against a build with the feature ripped
  out;
- **drop the `/FontFile2`** and `class.embedded` is false, same result;
- **make the run longer or split it across operators** and the caret's own
  `one_operator` measurement changes, which routes the refusal to a different
  `EditRefusal` and a different sentence.
