# `ocr` — turning the page on screen into an invisible, searchable text layer

This module is the **shell's half** of OCR: it decides *what image the
recogniser sees*, runs the job off the UI thread, and hands back either a
finished document and a disclosure report, or a named refusal. It authors
no PDF syntax of any kind — `pdfcer_core::ocr::layer` does all of that, and
this module exists partly to make sure nobody re-implements it here.

## The pipeline, and which crate owns each step

| # | Step | Owner |
|---|---|---|
| 1 | refuse early — no engine, no models, unsaved edits, no such page | **this module** |
| 2 | rasterize the page at [`fitted_dpi`] | `pdfcer-render` |
| 3 | RGBA → 8-bit greyscale | **this module** ([`greyscale`]) |
| 4 | detect words, group into lines, recognise | `pdfcer_core::ocr::engine_ocrs` |
| 5 | image pixels (y-down) → PDF user space (y-up), **including `/Rotate`** | `pdfcer_core::ocr::words_to_page_space_on` |
| 6 | write the mode-3 sandwich and save incrementally | `pdfcer_core::ocr::layer::add_ocr_layer` |
| 7 | put the bytes somewhere the operator named | `pdfcer-gui`'s `dialogs::ocr` |

Steps 4, 5 and 6 are deliberately not touched here. In particular **the
y-flip is not done in this module and must never be**: `words_to_page_space_on`
is a free function precisely so that the flip happens once, for every
engine, in one place. `pdfcer-core`'s own note is that a "helpful" flip at a
call site produces a layer that is mirrored *twice* — i.e. correct — for
one engine and mirrored once for the next, "which is the kind of defect
that gets attributed to the wrong module for a long time."

## pdfcer-gui is the first consumer of `add_ocr_layer` anywhere

Worth knowing before trusting anything downstream of step 6. Grepping
`D:\Dev\pdfcer` for `add_ocr_layer` finds the function, its own tests, and
**no caller**: `pdfcer` has no `ocr` command and `EditSession` has no
OCR verb. So the sandwich writer is exercised by unit tests and by this
module, and by nothing else in either repository.

## Why this runs on a thread, when `file.copy_document_text` does not

`app::dispatch`'s document-text arm blocks the UI thread on purpose and
says so: a whole-document extraction is 331–449 ms on this project's
benchmark sheet, which is a stutter. Recognition is not in that class. It
rasterizes a page at [`fitted_dpi`] and then runs two neural networks over it,
and on a full sheet that is **seconds**, not milliseconds. A frozen window
for that long is indistinguishable from a hung program, and an operator who
cannot tell those apart kills the process.

So [`Job`] is a `std::thread` plus a channel, in the same shape as
`render::worker` — with two differences that follow from OCR being a
deliberate act rather than a per-frame consequence:

* **No cancellation token.** The render worker cancels because the operator
  scrolling makes the in-flight raster unwanted. Nothing makes a recognition
  unwanted halfway through: it was asked for once, by name, and its result
  is still the answer to the question when it arrives.
* **No staleness key.** There is exactly one job at a time, held by the one
  dialog that started it, and the dialog cannot start a second while the
  first is running.

## What the recogniser is given, and why the obvious answer was wrong

**The raster size is [`TARGET_PIXELS`] = 8.4 million, not a DPI**, and that
constant carries the measurement that produced it. The short version, because
it is the most surprising thing this module learned:

`ocrs` resizes every image to its detection model's **fixed input size**, so
what decides whether a small character survives is the whole raster's shrink
factor, not its resolution. This module's first implementation used **300
DPI** — the scanning standard, the answer nobody would question — and,
measured against a real drawing's own vector text as ground truth, it scored
**3.3 %**: an order of magnitude worse than 72 DPI, and the worst of the five
resolutions tried. The best was 150 DPI at **44.7 %**, which on that sheet is
8.4 megapixels. Hence the constant, and hence its unit.

Greyscale rather than colour because that is the trait's contract:
`OcrEngine::recognize` takes "row-major, top-down, one byte per pixel — the
layout every candidate engine takes". Converting here rather than inside the
engine adapter keeps the adapter a pure binding.

## Why recognition reads the document as it was OPENED

`add_ocr_layer` takes a `&Document` — the base revision — and writes an
incremental section on top of it. That is what keeps the scan
byte-identical (project rule 3: an object pdfcer did not logically modify is
re-emitted unchanged or omitted entirely), and it is the whole reason OCR
does not cost a JPEG a decode/re-encode cycle.

The consequence is that **unsaved edits are not carried**, and this module
refuses rather than discloses. A recognised copy taken while markup was
pending would be a copy of the original with the operator's work missing
and nothing on screen to say so — a file that looks like what they asked
for and is not. `Refusal::UnsavedEdits` is the honest answer, and it is
reachable only in a build that can make edits, which this one is.

## Where the disclosure goes

[`Recognised::report`] is `pdfcer-core`'s own `OcrLayerReport`, carried out
whole. `pdfcer-gui`'s `dialogs::ocr` renders `report.disclosures()` verbatim. That
is the engine's instruction — the lines are built inside `pdfcer-core` "so
the GUI and the CLI cannot disagree about what was disclosed" — and it is
why nothing in this module summarises, rounds or re-words a count.

## Item notes

### `fn the_benchmark_sheet_lands_on_the_dpi_the_target_implies`

`SW41177.pdf`'s first page is 1584 × 1224 pt, and [`TARGET_PIXELS`] is
the megapixel count this function is built around — so a page of that
size must come out at the DPI the constant implies. That is arithmetic,
and it holds whatever the constant's *value* turns out to be.



The engine's note on the retraction made the general point, and it is
why this test is shaped differently now:

> *"A test that asserts a number fails on every legitimate change, and
> gets edited without the evidence."*

Exactly so. Had the sweep been re-run and `TARGET_PIXELS` moved, this
test would have gone red for a **correct** change — and the cheapest way
to make it green is to edit the band, which quietly destroys the link it
existed to protect. It now asserts what cannot become false by
re-measuring: that this page's DPI is the one `TARGET_PIXELS` implies.

When the sweep is re-run and the constant moves, this test should
**pass unchanged**. If it does not, the fitting arithmetic has come
apart — which is precisely what the old version could not tell you.

### `fn an_impossibly_large_page_still_gets_a_usable_resolution`

`1.0e6` points square is 13,888 inches on a side — not a page anyone has,
and that is the point: the assertion is about the clamp holding at the
far end of the range rather than about a realistic sheet. Without the
floor, `fitted_dpi` would answer about 0.2 DPI there, and a raster of a
few hundred pixels would be handed to the recogniser as though it were a
page.

The ordering of the two constants is asserted through `fitted_dpi`'s
behaviour rather than by comparing them directly: clippy rejects the
direct comparison as constant-valued, and it is right to — a literal
`MIN_DPI < MAX_DPI` is checked by the compiler's own constant folding
and tells a reader nothing the two declarations do not.

### `fn a_small_page_is_capped_at_the_scanning_standard`

A business card at 8.4 megapixels is over 1,000 DPI — resolution with no
ink behind it, paid for in seconds.

**US Letter is the interesting row and is asserted separately.** It
lands at 299.7 DPI, a quarter of a DPI under the ceiling: the measured
8.4-megapixel target and the conventional 300-DPI scanning standard
coincide almost exactly on the commonest page size in the world. That is
a coincidence rather than a design, and it is pinned because it explains
something a reader would otherwise find contradictory — the module header
says 300 DPI measured *worst*, and on a Letter page 300 DPI is what this
function will very nearly choose. Both are true: the figure that ruined
recognition was 300 DPI on a **36-inch drawing sheet**, which is 33
megapixels, not 8.4.

### `fn an_enormous_sheet_is_reduced_towards_the_target`

3370 × 2384 pt at 300 DPI would be 138 megapixels and 550 MB of RGBA
before anything is recognised. More to the point, the measurement says a
raster that large is where this engine reads *worst* — so the reduction
is about accuracy first and memory second, which is the opposite of how
the first version of this code justified it.

### `fn greyscale_produces_exactly_one_byte_per_pixel`

The engine validates `len == w * h` and refuses a mismatch outright, so
a length bug here would surface as an unexplained engine error rather
than as a bad picture.

### `fn a_coloured_pixel_is_weighted_rather_than_averaged`

The reason the luma weights are there rather than `(r+g+b)/3`. Pure
blue averages to 85 — a mid-tone the binarizer may keep — and weights
to 29, which is ink. Pure green averages to the same 85 and weights to
150, which is background. A page marked up in blue and highlighted in
yellow is exactly the case where the two disagree, and it is a common
one on a scanned drawing.

### `fn an_empty_model_directory_is_rejected_but_a_filled_one_resolves`

The hazard `pdfcer-core` built `resolve_model_dir_with` for, and it is
nastier than a plain "not found". Resolution asking only `is_dir()`
means an empty directory beside the executable **wins**: this shell
tells the operator the models were found, their own good copy is never
reached, and the failure surfaces one layer down in the engine's
vocabulary — a missing model file, after we said there was not one.

Realistic rather than contrived. A part-finished extraction, an
antivirus quarantine that took the weights and left the folder, or an
operator creating the directory by hand before copying into it all
produce exactly this state.

The positive half is asserted too, and it is what makes this test
discriminate. Its first draft checked only that an empty directory
fails — which the OLD resolver also does when the path is wrong, so the
test passed against the very code it was written to condemn. Putting a
file in and requiring success is what proves the failure above was about
EMPTINESS rather than about the path.

### `const TARGET_PIXELS`

`ocrs`'s detector **resizes every image to its model's fixed input size**
before running it (`detection.rs`: *"Resize images to the text detection
model's input size"*), then resizes the probability mask back. So the thing
that decides whether a 3 mm character survives detection is not its
resolution in the raster — it is **how much the whole raster is shrunk to
reach the model's input**, which is a function of total pixels and nothing
else. A DPI is only a proxy for that, and it is a bad one: the same DPI is
a 2× reduction on a postcard and an 8× reduction on an A0 sheet.

# The measurement

Run against `D:\Dev\temp\pdfcer\SW41177.pdf` — a real 36-sheet SolidWorks
drawing whose **vector text is the ground truth**, which is what makes this
an accuracy figure rather than an impression. Recognised tokens of three or
more characters were compared against the page's own extracted text:


The first version of this table was produced by a text-detection model that
**did not work** — `pdfcer-core`'s bundled build had been broken since the
engine landed, returning fragments clustered at a page margin plus one
"word" the size of the page. Every number in it was therefore a measurement
of how *noise* varies with resolution, and it was retracted rather than
adjusted. Fixed engine-side in Pass 129.0; re-run here against
`text-detection.rten` **2,510,284 B / `f15cfb56…`**, verified by hash before
measuring, because measuring the same broken thing twice is the obvious way
to waste the exercise.

`SW41177.pdf` page 1, 130 ground-truth tokens of 3+ characters:

| DPI | raster | Mpx | recognised ≥3 chars | exactly in ground truth | was (noise) |
|---:|---|---:|---:|---|---:|
| 72 | 1584×1224 | 1.9 | 207 | 117 (56.5 %) | 34.8 % |
| **100** | **2200×1700** | **3.7** | **210** | **119 (56.7 %)** | 20.0 % |
| 150 | 3300×2550 | 8.4 | 191 | 104 (54.5 %) | 44.7 % |
| 200 | 4400×3400 | 15.0 | 191 | 103 (53.9 %) | 53.9 % ← was 27.5 |
| 300 | 6600×5100 | 33.7 | 191 | **67 (35.1 %)** | 3.3 % |

## What survived the retraction, and what did not

**Survived: more resolution is not better, and the conventional answer is
the worst one.** 300 DPI — the scanning standard, and what this module's
first implementation used — is still clearly the poorest row, now by 21
points rather than by 41. The mechanism the old table was explained by is
unchanged and is a property of the crate rather than of the weights: `ocrs`
resizes every image to its model's fixed input, so **pixel count governs,
not resolution**, and past a point more pixels only means more downscaling
before the model ever sees them.

**Did not survive: the sharp peak at 150.** The real curve is a *plateau*
from 72 to 200 — 56.5, 56.7, 54.5, 53.9, a spread of under three points,
which is inside the noise of a 130-token sample — and then a cliff. The old
curve's jagged shape (34.8 → 20.0 → 44.7 → 27.5) was the detector failing
differently at each size, and reading a maximum out of it was reading a
maximum out of noise.

**That is why [`TARGET_PIXELS`] does not move.** 8.4 Mpx puts the
benchmark sheet at 150 DPI, which is inside the plateau and 2.2 points off
the nominal best — a difference this sample cannot resolve. The constant was
right for a wrong reason and is now right for a measured one, which is worth
distinguishing: nothing about the code changed, and everything about what is
*known* about it did.

## What this is still not

Two documents, one of them small. `fixtures/a1-titleblock.pdf` has 16
ground-truth tokens and produced 11.1 / 0.0 / 10.0 / 33.3 / 20.0 % across the
same sweep — too few tokens for any row to mean anything individually,
though it agrees that 300 is not the answer. A defensible *general* figure
needs a corpus of real scans rather than two CAD sheets, and that is
outstanding.

**The first implementation of this module used 300 DPI**, on the entirely
conventional reasoning that 300 is the scanning standard and that more
resolution cannot hurt. It is the worst row on the table — 35.1 % against
56.7 % — and it was the worst row on the broken table too. That finding has
now been made twice, by two different detectors, which is about as much
confirmation as a single-document measurement can offer.

8,400,000 is the 150-DPI row, expressed as the quantity that actually
governs. A small scanned page therefore gets *more* DPI than 150 and a large
sheet gets less, which is exactly what the detector's fixed-size resize
wants and what a constant DPI cannot express.

# What this figure is and is not

It is two documents, both CAD — dense linework, which is adversarial for a
model trained on photographs and document pages. **56.7 % is not a quality
claim for pdfcer's OCR on ordinary material**: on a synthetic scan of
ordinary text at 200 dpi, blurred and skewed with sensor noise, the engine
reads 47 of 47 words. These figures are the hard end, not the typical one.
See `ocr::fixture` for what is and is not established about recognition
quality, and the report to the operator for the plain-English version.

### `const MAX_DPI`

A ceiling for **small** pages, where [`TARGET_PIXELS`] would otherwise ask
for an absurd magnification: a business card at 8.4 megapixels is over 1,000
DPI, which costs time and adds nothing — the ink has no more detail in it
than the source had. 300 is the scanning standard and is the right ceiling
even though it is the wrong *target*.

### `const MIN_DPI`

A floor for pages so large that [`TARGET_PIXELS`] would ask for less than
one device pixel per point. Below this the recognition crops are too small
to carry a glyph at all, and the honest failure — a refusal, or a page of
nonsense the disclosure warns about — is preferable to spending the time.

### `enum Refusal`

Every variant is a **named** cause with a different action behind it. The
engine's own error type does the same thing and for the same stated reason:
on a portable install "the weights are not beside the binary" is the most
likely failure by a wide margin and is entirely fixable — but only if the
message says so.

### `struct Recognised`

**The bytes and the report travel together and are only ever handed over
together.** `pdfcer-core`'s report type says a caller "that builds a layer
and drops the report has made pdfcer silent about a page of guesses", and
keeping them in one struct is how that is made awkward to do by accident.

### `fn raster_scale`

`dpi / 72.0`, because a PDF user-space unit is 1/72 inch by definition
(ISO 32000-1 §8.3.2.3). One line, in one place, so no call site does the
division by hand and gets 96 into it.

That "one place" is now [`crate::units::scale_from_dpi`], one level
further out again: this function was one of THREE that each held the same
one line, which is the same defect at a larger scale. The `f32` signature
stays because every caller hands the result to `pdfcer-render` as a scale;
the widening and narrowing around the call is cheaper to read than an `f32`
twin of the table would be to maintain.

### `fn fitted_dpi`

Solves [`TARGET_PIXELS`] for this page's area, then clamps to
[`MIN_DPI`]..=[`MAX_DPI`]. Returns a DPI rather than a scale so that the
number reported to the operator and the number handed to the rasterizer are
derived from one another instead of computed twice.

A page with no area yields [`MAX_DPI`] rather than infinity: the caller has
already refused an empty page by then, and a non-finite scale out of a clamp
would be a worse failure than the one it is guarding.

### `fn greyscale`

# Why the luma weights and not a plain average

ITU-R BT.601's `0.299 R + 0.587 G + 0.114 B` — the same coefficients
`pdfcer-core`'s own JPEG paths use. A flat average treats a saturated blue
stamp as mid-grey and a yellow highlighter as near-white, which is exactly
backwards for a page that has been marked up: the blue ink a human reads
easily would fade and the yellow wash the human ignores would swallow the
text under it.

# Why the channel order does not matter here

`tiny_skia::Pixmap` is premultiplied RGBA. The weights below are applied in
that order. If a future backend hands over BGRA the red and blue weights
swap, which shifts a *coloured* pixel's grey by at most 0.185 of full scale
and leaves every neutral pixel — which is nearly all of a scan — exactly
where it was. Stated rather than guarded, because a guard against a
hypothetical byte order would be untestable here.

Alpha is ignored: the rasterizer is asked for a white-backed page, so every
pixel is already composited and an alpha channel that is uniformly opaque
carries no information.

### `fn exe_dir`

`None` rather than a guess when `current_exe` fails: a wrong directory here
produces "models not found" naming a path nobody has, which is worse than
naming one fewer place that was genuinely searched.

### `struct Request`

A struct rather than six arguments because it is what crosses the thread
boundary, and because the compiler then checks that every field is `Send`
in one place instead of at a `spawn` call.

### `fn recognise`

Written as a free function taking `&Request` for the same reason
`render::worker::render_on_worker` is: a body that cannot reach `self` is a
body that provably shares nothing with the UI thread.

**Recognise every requested page, chaining the revisions.**

# The shape, and why it is a fold rather than a map

`add_ocr_layer` takes a whole `Document` and returns a whole PDF. So page
two must be recognised **against the output of page one**, not against the
original — otherwise the second write would be an incremental revision over
a base that does not have the first layer, and the first page's words would
be silently dropped.

That chaining was the audit's one UNVERIFIED risk and it was measured before
this was written: two successive in-place recognitions produce a file that
round-trips byte-identical and extracts both layers.

# What a failure on one page does to the rest

**Nothing.** A page with no recognisable text — a blank sheet, a photograph
of a wall — reports `NothingRecognised`, and on a forty-page scan that must
not abandon the other thirty-nine. So a per-page refusal is *counted*, not
propagated, and the run reports how many pages produced words.

The exception is an **engine** failure, which is not about the page: if the
recogniser itself is broken, every remaining page will fail the same way and
grinding through thirty-nine more is only a slower way to say so.

# A run that recognised nothing anywhere is a refusal

If no page produced a single word, there is nothing to write and nothing to
save, and reporting success would leave the operator with a dialog saying it
worked and a document with nothing in it.

### `fn engine_compiled_in`

Read by the dialog before it looks for models: *cannot look* and *could not
find the files to look with* are different refusals, and asking in the
wrong order would report the second when the first is true.
