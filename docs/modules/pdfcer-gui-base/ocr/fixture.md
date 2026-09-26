# `ocr::fixture` — building the image-only PDF this project did not have

Generates `fixtures/synthetic-image-only.pdf`, and runs the whole OCR
pipeline against it. Both are `#[ignore]`d: the first writes into the
repository, and the second costs several seconds and needs the model
weights on disk. Run them by name, exactly as the RON regeneration is run:

```text
cargo test -p pdfcer-gui-base --lib write_synthetic_image_only -- --ignored
cargo test -p pdfcer-gui-base --lib recognises_the_synthetic_page -- --ignored --nocapture
```

## WHAT THIS FIXTURE IS, AND — MORE IMPORTANTLY — WHAT IT IS NOT

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

## Item notes

### `const PAGE_W`

306 x 396 pt. At [`FIXTURE_DPI`] that is an 850 x 1100 raster, which
compresses to tens of kilobytes and is a reasonable thing to keep in a
repository. An A1 sheet at 300 DPI, for comparison, is 70 megapixels.

### `const FIXTURE_DPI`

200, which is not the resolution recognition will run at, and the
difference is deliberate: the
fixture should not be rendered at exactly the resolution it will later be
recognised at, or the recogniser would be reading back a raster it could
have been handed unresampled. A scan is never at the resolution the reader
chooses either.

### `fn source_pdf`

The *source* for the fixture, never the fixture itself — it is thrown away
as pixels in step 2. Written as literal syntax rather than authored through
`EditSession` because every byte of it needs to be inspectable: this is the
thing whose text must survive a round trip through a raster and a
recogniser, and a document assembled by the same engine that will later be
asked to read it would make the test partly self-referential.

Standard-14 Helvetica, so nothing is embedded and the file stays under two
kilobytes. 11 pt on a 396 pt page with 24 pt leading is ordinary document
type. See [`LINES`] for why the fixture is a page of text rather than a
caption, and what the first version of it discovered by not being one.

`TL`/`T*` rather than a `Td` per line: one text object with a set leading is
how a real producer writes a paragraph, and a fixture whose content stream is
shaped like nothing any producer emits is a fixture testing a shape nobody
meets.

### `fn stream_object`

`extra` is spliced into the dictionary before `/Length`, which is how the
image object gets its `/Filter`, `/Width`, `/Height` and colour space
without a second assembler.

### `fn assemble`

A cross-reference **table** rather than a stream, and a `%PDF-1.4` header,
on purpose: both are the oldest and most widely-agreed forms, so a fixture
that failed to open would be a defect in whatever opened it rather than an
argument about which of two encodings was meant. The offsets are counted
from the emitted bytes as they are written — never computed in advance —
because an xref whose offsets are one byte out is a file that opens on some
readers and not others, which is the worst kind of broken fixture.

### `fn image_only_pdf`

`grey` is one byte per pixel, row-major, top-down — the layout
[`super::greyscale`] produces and the layout `/DeviceGray` at
`/BitsPerComponent 8` expects, so no transposition happens here. The image
XObject is defined on the unit square (§8.9.4), so the whole of placement is
the one `cm` matrix that scales it to the page box.

### `fn image_only_pdf_pages`

# Why a multi-page image-only fixture has to exist

The one-page fixture is the right subject for *"did the recogniser read this
page"*. It is the wrong subject for pages-done, words-and-characters
detected, and a Stop control, because **every one of those is a statement
about a run in progress**, and a one-page run has no observable middle. It
is started and
then it is finished; a Stop pressed during it can only ever race the single
page, and a progress line that draws once carries no evidence that it
advances.

So this exists to give the driven checks a run with a **middle**:
[`MULTIPAGE_PAGES`] sheets, recognised one after another, long enough that a
harness can see `attempted` climb and can press Stop with pages still to go.

# Why the pages are identical, which looks like a shortcut and is not

Each page is the same rendered notes sheet, and every page dictionary points
at the **same** image XObject. Three consequences, all wanted:

* the file is ~40 kB rather than ~300 kB, because the pixels are stored once
  — a fixture that has to be committed should not be a third of a megabyte;
* every page recognises to the **same word count**, so a check can assert
  the totals are consistent with the pages attempted rather than having to
  accept any number at all;
* a page that is skipped or dropped is visible as an arithmetic hole rather
  than as a plausible smaller number.

The pages sharing an XObject is *also* representative: it is what a real
scanner-produced PDF does not do, but what every stamp, logo and repeated
figure in a real document does, and a recogniser that assumed one image per
page would break on both.

# What it still does not establish

The same caveat the one-page fixture carries, and it is not weakened by
there being more of them: this is a **rendered** page, not a scan. No
scanner noise, no skew, no JPEG ringing, no uneven lighting. It establishes
the plumbing of a multi-page run. It establishes nothing about recognition
quality, and the driven checks say so in their own reports.

### `fn raster`

Split out when the multi-page fixture arrived, so that the two documents are
**the same pixels** by construction rather than by two call sites happening
to pass the same DPI. A one-page and an eight-page fixture that disagreed
about the raster would make their word counts incomparable, and comparing
them is half of what the multi-page checks do.

### `fn the_multipage_fixture_has_the_page_count_it_claims`

Pinned because the whole value of that fixture is the page COUNT, and
the count is produced by hand-written object numbering with a classic
xref table — the one part of this module where an off-by-one produces a
file that still opens. A `/Count 8` over seven `/Kids` is a document
most readers will show, and every driven check over it would then be
asserting against a denominator that is a lie.

Parsed by `pdfcer_core` rather than grepped, deliberately: the question
is *what will the application see*, and the application sees whatever
the page-tree walker sees.

### `fn the_multipage_fixture_shares_one_image_rather_than_copying_it`

The property that makes committing this fixture reasonable. Asserted as
a ratio rather than an absolute size so it survives a change to the
raster DPI: if somebody later gives each page its own copy of the
pixels, the eight-page file becomes ~8× the one-page file and this
fails, which is the moment to notice — not at the next `git push`.

### `fn the_multipage_fixture_contains_no_text_operator_either`

Separate from [`the_fixture_contains_no_text_operator_at_all`] rather
than folded into it. The two documents are built by two functions, and
the assertion that matters — *any text on this page came from the
recogniser* — has to hold of the one the checks actually drive. A shared
test over only the one-page build would leave the eight-page build
unasserted while looking like it covered both.

### `fn the_fixture_contains_no_text_operator_at_all`

The property that makes it a valid test of OCR rather than a test of
nothing, asserted against the **emitted bytes** rather than against the
construction that produced them. Both streams are checked: the content
stream is uncompressed and inspectable directly, and the image stream is
binary and could in principle contain the byte pairs by accident — which
is why the assertion is on the content stream's region specifically.

Without this, a future change that "helpfully" kept a caption on the page
would leave every OCR check passing for the wrong reason: the text would
already be extractable, the offer would never appear, and the round trip
would succeed without the recogniser contributing anything.

### `fn the_engine_finds_no_text_on_the_fixture`

The previous test asserts the *bytes*; this asserts what the **engine
makes of them**, which is the thing `OpenDoc::page_has_extractable_text`
actually asks. They are not the same claim: a content stream with no text
operator could still carry text through an annotation appearance or a
form XObject, and the extractor is what would know.

### `fn the_source_page_does_have_the_text_the_fixture_throws_away`

The control, and it is the load-bearing half of the pair: rule 4 of
`tools/ui-verify`'s own checks — *never treat an absence as evidence
unless you have shown the thing that would have produced it was
working* — applies just as much to a unit test. Without this, an
extractor that returned nothing for **every** document would satisfy the
assertion above perfectly.

### `const LINES`

The first version of this fixture was two words in 28 pt on an otherwise
blank card, on the reasoning that a legible fixture is one that fails only
for real reasons. **It failed for a real reason, and the reason is worth the
paragraphs below**, because it is a fact about the engine pdfcer ships that
nothing in either repository knew.

### What happened

`ocrs`'s detection model produced a *perfect* probability map -- four clean
blobs, exactly over the four words. Measured, not assumed: the map was
dumped and its connected components counted by hand, and there were four, at
the right places and the right sizes. And `ocrs::detect_words` returned
**three** rectangles, the first of which was the entire page.

The cause is a threshold, and it is in the open.
`TextDetectorParams::default()` sets `text_threshold: 0.2`, under the
upstream comment *"Ideally the threshold would be 0.5 as a neutral value."*
On this fixture the model's output over blank paper measured **0.148 to
0.208** -- straddling that threshold. So the background itself binarised as
text in patches, the patches connected, and one component swallowed the
page. The recogniser was then handed the whole sheet squeezed into a
127 x 64 line crop, and returned `"SE"`, `"1"`, `"P"`.

### Why the FIXTURE changed and not the threshold

Raising `text_threshold` would have been one line, and would have been
**tuning the tool until the test passed** -- the flattering-fixture failure
run in reverse. The
threshold is upstream's, chosen empirically against upstream's training
distribution, and this project has no evidence on which to overrule it.

What was actually wrong was the fixture's *representativeness*. `ocrs` is
trained on HierText -- photographs and dense document pages -- and a sheet
that is 96 % blank paper is neither. **A page of text is.** So the fixture
became one: fourteen lines at a realistic size and spacing, which is both
what the feature will meet and what the model was trained against.

### The finding stands regardless of this fixture

**`ocrs` at its default threshold can fail catastrophically on a sparse,
clean page** -- not degrade, fail: one whole-page "word" and three
characters of output. A scanned drawing with a small title block on a large
empty sheet is exactly that shape, and it is the shape this project's own
documents come in. It is recorded here rather than as a comment on a passing
test, and it is in the report to the operator.

### Why these words

### `const MUST_RECOGNISE`

A subset of [`LINES`], and deliberately the ones a reader most often needs
Find to reach on a real drawing -- a drawing number and a revision -- two of
which carry digits, exercising a different part of the model's alphabet from
the prose.

### `const MULTIPAGE_PAGES`

**Eight, and the number was measured rather than chosen.** One page of
this fixture recognises in roughly a second in a release build, and a page
of the operator's own scanned parts manual measured **2.6 s** through
`pdfcer ocr`. Eight pages is therefore a run of eight to twenty seconds:

* long enough that a driven check can watch `attempted` climb, press Stop
  with pages still to go, and have the result be unambiguous — a Stop that
  lands on the last page is indistinguishable from a run that finished;
* short enough that three driven checks over it cost under a minute, which
  is what keeps them in the ordinary sweep rather than in a "slow" tier
  nobody runs.

It is also **not** a round number by accident: it matches the eight pages
extracted from the operator's manual for the real-material run, so the two
reports are read side by side without arithmetic.
