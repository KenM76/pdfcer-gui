# `ocr::fixture` — building the image-only PDF this project did not have

Generates `fixtures/synthetic-image-only.pdf`, and runs the whole OCR
pipeline against it. Both are `#[ignore]`d: the first writes into the
repository, and the second costs several seconds and needs the model
weights on disk. Run them by name, exactly as the RON regeneration is run:

```text
cargo test -p pdfcer-gui-base --lib write_synthetic_image_only -- --ignored
cargo test -p pdfcer-gui-base --lib recognises_the_synthetic_page -- --ignored --nocapture
```

## ★★ WHAT THIS FIXTURE IS, AND — MORE IMPORTANTLY — WHAT IT IS NOT

**It is not a scan.** It is a page rendered from vector text and then thrown
away as pixels. Read the name as the whole caveat: *synthetic*, *image-only*.

### Why it exists anyway

Because the alternative is verifying nothing. Every real document this
project can reach is a vector PDF that already contains text:
`D:\Dev\temp\pdfcer\` holds thirty PDFs and `fixtures/` held one, and **every
single one has extractable text on it.** Measured, not assumed: a scan of
both directories for text-showing operators (`Tj`, `TJ`, `'`, `"`) inside
every inflatable stream finds them in all thirty-one.

So before this file, **no part of the OCR chain could be exercised at all**
— not the detection of an image-only page, not the Find offer, not the
recogniser, not the sandwich, not the round trip.

### ✅ What a green result here DOES establish

The **plumbing**, which is most of what the shell was asked to build:

1. a document with no text layer is *detected* as having none
   (`OpenDoc::page_has_extractable_text` answers `false`);
2. Find therefore offers OCR on it rather than reporting an ordinary empty
   result;
3. the models resolve, the recogniser loads and runs;
4. words come back positioned, survive the y-flip into page space, and reach
   `pdfcer_core::ocr::layer::add_ocr_layer`;
5. the invisible layer is written at text rendering mode **3**;
6. and the result is a document whose text pdfcer can extract — which is the
   property the whole feature exists to produce.

### ❌ What it does NOT establish, and must never be read as establishing

**Recognition quality on a real scan.** A render-to-raster has no scanner
noise, no skew, no JPEG ringing, no bleed-through from the reverse side, no
uneven platen lighting, no dust, no fold shadow and no halftone screen. Those
are precisely the conditions that make OCR hard, and a fixture without them
**flatters the recogniser**.

**A measurement that moves in the right direction is not evidence that it
measures the right thing.** A fixture that disturbs its samples only along
the shape it is supposed to be testing reports suspiciously good numbers and
is measuring something other than what it claims.

This fixture is in that category by construction and says so rather than
being caught at it later. **Quality on real scanned material remains
unproven**, which is the engine's own position and is unchanged by a shell
having shipped a surface for it.

## What the page says, and why those words


## How it is built

By hand, in five steps, with no dependency beyond what already ships:

1. a tiny one-page PDF with real text is written as literal PDF syntax
   ([`source_pdf`]);
2. `pdfcer_render` rasterizes it at [`FIXTURE_DPI`];
3. the RGBA pixmap becomes 8-bit greyscale through [`super::greyscale`] —
   the same function the OCR path itself uses, so the fixture and the
   feature agree about what a grey pixel is;
4. the greyscale is `/FlateDecode`d into a `/DeviceGray` image XObject;
5. and a second one-page PDF is written whose entire content stream is
   `q W 0 0 H 0 0 cm /Im0 Do Q` — **one image operator and not one text
   operator anywhere in the file.**

Step 5 is what makes the result genuinely image-only rather than merely
image-heavy, and [`tests::the_fixture_contains_no_text_operator_at_all`]
asserts it against the emitted bytes rather than trusting the construction.
