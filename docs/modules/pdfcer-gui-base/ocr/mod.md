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
