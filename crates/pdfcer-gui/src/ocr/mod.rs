//! # `ocr` — turning the page on screen into an invisible, searchable text layer
//!
//! This module is the **shell's half** of OCR: it decides *what image the
//! recogniser sees*, runs the job off the UI thread, and hands back either a
//! finished document and a disclosure report, or a named refusal. It authors
//! no PDF syntax of any kind — `pdfcer_core::ocr::layer` does all of that, and
//! this module exists partly to make sure nobody re-implements it here.
//!
//! ## The pipeline, and which crate owns each step
//!
//! | # | Step | Owner |
//! |---|---|---|
//! | 1 | refuse early — no engine, no models, unsaved edits, no such page | **this module** |
//! | 2 | rasterize the page at [`fitted_dpi`] | `pdfcer-render` |
//! | 3 | RGBA → 8-bit greyscale | **this module** ([`greyscale`]) |
//! | 4 | detect words, group into lines, recognise | `pdfcer_core::ocr::engine_ocrs` |
//! | 5 | image pixels (y-down) → PDF user space (y-up), **including `/Rotate`** | `pdfcer_core::ocr::words_to_page_space_on` |
//! | 6 | write the mode-3 sandwich and save incrementally | `pdfcer_core::ocr::layer::add_ocr_layer` |
//! | 7 | put the bytes somewhere the operator named | `crate::dialogs::ocr` |
//!
//! Steps 4, 5 and 6 are deliberately not touched here. In particular **the
//! y-flip is not done in this module and must never be**: `words_to_page_space_on`
//! is a free function precisely so that the flip happens once, for every
//! engine, in one place. `pdfcer-core`'s own note is that a "helpful" flip at a
//! call site produces a layer that is mirrored *twice* — i.e. correct — for
//! one engine and mirrored once for the next, "which is the kind of defect
//! that gets attributed to the wrong module for a long time."
//!
//! ## ★ pdfcer-gui is the first consumer of `add_ocr_layer` anywhere
//!
//! Worth knowing before trusting anything downstream of step 6. Grepping
//! `D:\Dev\pdfcer` for `add_ocr_layer` finds the function, its own tests, and
//! **no caller**: `pdfcer` has no `ocr` command and `EditSession` has no
//! OCR verb. So the sandwich writer is exercised by unit tests and by this
//! module, and by nothing else in either repository.
//!
//! ## ★ Why this runs on a thread, when `file.copy_document_text` does not
//!
//! `app::dispatch`'s document-text arm blocks the UI thread on purpose and
//! says so: a whole-document extraction is 331–449 ms on this project's
//! benchmark sheet, which is a stutter. Recognition is not in that class. It
//! rasterizes a page at [`fitted_dpi`] and then runs two neural networks over it,
//! and on a full sheet that is **seconds**, not milliseconds. A frozen window
//! for that long is indistinguishable from a hung program, and an operator who
//! cannot tell those apart kills the process.
//!
//! So [`Job`] is a `std::thread` plus a channel, in the same shape as
//! `render::worker` — with two differences that follow from OCR being a
//! deliberate act rather than a per-frame consequence:
//!
//! * **No cancellation token.** The render worker cancels because the operator
//!   scrolling makes the in-flight raster unwanted. Nothing makes a recognition
//!   unwanted halfway through: it was asked for once, by name, and its result
//!   is still the answer to the question when it arrives.
//! * **No staleness key.** There is exactly one job at a time, held by the one
//!   dialog that started it, and the dialog cannot start a second while the
//!   first is running.
//!
//! ## ★★ What the recogniser is given, and why the obvious answer was wrong
//!
//! **The raster size is [`TARGET_PIXELS`] = 8.4 million, not a DPI**, and that
//! constant carries the measurement that produced it. The short version, because
//! it is the most surprising thing this module learned:
//!
//! `ocrs` resizes every image to its detection model's **fixed input size**, so
//! what decides whether a small character survives is the whole raster's shrink
//! factor, not its resolution. This module's first implementation used **300
//! DPI** — the scanning standard, the answer nobody would question — and,
//! measured against a real drawing's own vector text as ground truth, it scored
//! **3.3 %**: an order of magnitude worse than 72 DPI, and the worst of the five
//! resolutions tried. The best was 150 DPI at **44.7 %**, which on that sheet is
//! 8.4 megapixels. Hence the constant, and hence its unit.
//!
//! Greyscale rather than colour because that is the trait's contract:
//! `OcrEngine::recognize` takes "row-major, top-down, one byte per pixel — the
//! layout every candidate engine takes". Converting here rather than inside the
//! engine adapter keeps the adapter a pure binding.
//!
//! ## ★ Why recognition reads the document as it was OPENED
//!
//! `add_ocr_layer` takes a `&Document` — the base revision — and writes an
//! incremental section on top of it. That is what keeps the scan
//! byte-identical (project rule 3: an object pdfcer did not logically modify is
//! re-emitted unchanged or omitted entirely), and it is the whole reason OCR
//! does not cost a JPEG a decode/re-encode cycle.
//!
//! The consequence is that **unsaved edits are not carried**, and this module
//! refuses rather than discloses. A recognised copy taken while markup was
//! pending would be a copy of the original with the operator's work missing
//! and nothing on screen to say so — a file that looks like what they asked
//! for and is not. `Refusal::UnsavedEdits` is the honest answer, and it is
//! reachable only in a build that can make edits, which this one is.
//!
//! ## Where the disclosure goes
//!
//! [`Recognised::report`] is `pdfcer-core`'s own `OcrLayerReport`, carried out
//! whole. `crate::dialogs::ocr` renders `report.disclosures()` verbatim. That
//! is the engine's instruction — the lines are built inside `pdfcer-core` "so
//! the GUI and the CLI cannot disagree about what was disclosed" — and it is
//! why nothing in this module summarises, rounds or re-words a count.

/// Builds `fixtures/synthetic-image-only.pdf` and runs the whole chain against
/// it. **Test-only**, and its header is the argument for what a green result
/// there does and does not prove — the short version being that it establishes
/// the plumbing and establishes **nothing** about recognition quality on a real
/// scan, because a rendered raster has none of the degradation that makes OCR
/// hard.
#[cfg(test)]
mod fixture;

/// **What the recogniser is doing, and the two ways to end it early** —
/// the operator asked for both on 2026-09-01. Its header carries why Cancel
/// and Stop must never collapse into one act.
/// **A recognition running on a thread**, and the two ways to end it early.
/// Split from this module on 2026-09-01: running a recognition is a different
/// subject from performing one.
pub mod job;
pub mod progress;

pub use job::{Job, Tally};

use std::path::{Path, PathBuf};
use std::sync::Arc;

use pdfcer_core::edit::EditSession;
use pdfcer_core::ocr::{OcrPage, models};
use pdfcer_core::page_tree::{self, Rect};

/// ★★ **The raster size recognition is run at, as a pixel count** — measured,
/// not chosen.
///
/// # Why a pixel count and not a DPI, which is what this constant used to be
///
/// `ocrs`'s detector **resizes every image to its model's fixed input size**
/// before running it (`detection.rs`: *"Resize images to the text detection
/// model's input size"*), then resizes the probability mask back. So the thing
/// that decides whether a 3 mm character survives detection is not its
/// resolution in the raster — it is **how much the whole raster is shrunk to
/// reach the model's input**, which is a function of total pixels and nothing
/// else. A DPI is only a proxy for that, and it is a bad one: the same DPI is
/// a 2× reduction on a postcard and an 8× reduction on an A0 sheet.
///
/// # The measurement
///
/// Run against `D:\Dev\temp\pdfcer\SW41177.pdf` — a real 36-sheet SolidWorks
/// drawing whose **vector text is the ground truth**, which is what makes this
/// an accuracy figure rather than an impression. Recognised tokens of three or
/// more characters were compared against the page's own extracted text:
///
/// # ★★★ RE-MEASURED 2026-08-26, against a detector that works
///
/// The first version of this table was produced by a text-detection model that
/// **did not work** — `pdfcer-core`'s bundled build had been broken since the
/// engine landed, returning fragments clustered at a page margin plus one
/// "word" the size of the page. Every number in it was therefore a measurement
/// of how *noise* varies with resolution, and it was retracted rather than
/// adjusted. Fixed engine-side in Pass 129.0; re-run here against
/// `text-detection.rten` **2,510,284 B / `f15cfb56…`**, verified by hash before
/// measuring, because measuring the same broken thing twice is the obvious way
/// to waste the exercise.
///
/// `SW41177.pdf` page 1, 130 ground-truth tokens of 3+ characters:
///
/// | DPI | raster | Mpx | recognised ≥3 chars | exactly in ground truth | was (noise) |
/// |---:|---|---:|---:|---|---:|
/// | 72 | 1584×1224 | 1.9 | 207 | 117 (56.5 %) | 34.8 % |
/// | **100** | **2200×1700** | **3.7** | **210** | **119 (56.7 %)** | 20.0 % |
/// | 150 | 3300×2550 | 8.4 | 191 | 104 (54.5 %) | 44.7 % |
/// | 200 | 4400×3400 | 15.0 | 191 | 103 (53.9 %) | 53.9 % ← was 27.5 |
/// | 300 | 6600×5100 | 33.7 | 191 | **67 (35.1 %)** | 3.3 % |
///
/// ## What survived the retraction, and what did not
///
/// **Survived: more resolution is not better, and the conventional answer is
/// the worst one.** 300 DPI — the scanning standard, and what this module's
/// first implementation used — is still clearly the poorest row, now by 21
/// points rather than by 41. The mechanism the old table was explained by is
/// unchanged and is a property of the crate rather than of the weights: `ocrs`
/// resizes every image to its model's fixed input, so **pixel count governs,
/// not resolution**, and past a point more pixels only means more downscaling
/// before the model ever sees them.
///
/// **Did not survive: the sharp peak at 150.** The real curve is a *plateau*
/// from 72 to 200 — 56.5, 56.7, 54.5, 53.9, a spread of under three points,
/// which is inside the noise of a 130-token sample — and then a cliff. The old
/// curve's jagged shape (34.8 → 20.0 → 44.7 → 27.5) was the detector failing
/// differently at each size, and reading a maximum out of it was reading a
/// maximum out of noise.
///
/// ★★ **That is why [`TARGET_PIXELS`] does not move.** 8.4 Mpx puts the
/// benchmark sheet at 150 DPI, which is inside the plateau and 2.2 points off
/// the nominal best — a difference this sample cannot resolve. The constant was
/// right for a wrong reason and is now right for a measured one, which is worth
/// distinguishing: nothing about the code changed, and everything about what is
/// *known* about it did.
///
/// ## What this is still not
///
/// Two documents, one of them small. `fixtures/a1-titleblock.pdf` has 16
/// ground-truth tokens and produced 11.1 / 0.0 / 10.0 / 33.3 / 20.0 % across the
/// same sweep — too few tokens for any row to mean anything individually,
/// though it agrees that 300 is not the answer. A defensible *general* figure
/// needs a corpus of real scans rather than two CAD sheets, and that is
/// outstanding.
///
/// **The first implementation of this module used 300 DPI**, on the entirely
/// conventional reasoning that 300 is the scanning standard and that more
/// resolution cannot hurt. It is the worst row on the table — 35.1 % against
/// 56.7 % — and it was the worst row on the broken table too. That finding has
/// now been made twice, by two different detectors, which is about as much
/// confirmation as a single-document measurement can offer.
///
/// 8,400,000 is the 150-DPI row, expressed as the quantity that actually
/// governs. A small scanned page therefore gets *more* DPI than 150 and a large
/// sheet gets less, which is exactly what the detector's fixed-size resize
/// wants and what a constant DPI cannot express.
///
/// # What this figure is and is not
///
/// It is two documents, both CAD — dense linework, which is adversarial for a
/// model trained on photographs and document pages. **56.7 % is not a quality
/// claim for pdfcer's OCR on ordinary material**: on a synthetic scan of
/// ordinary text at 200 dpi, blurred and skewed with sensor noise, the engine
/// reads 47 of 47 words. These figures are the hard end, not the typical one.
/// See `ocr::fixture` for what is and is not established about recognition
/// quality, and the report to the operator for the plain-English version.
pub const TARGET_PIXELS: u64 = 8_400_000;

/// The most resolution a page is ever rasterized at, in DPI.
///
/// A ceiling for **small** pages, where [`TARGET_PIXELS`] would otherwise ask
/// for an absurd magnification: a business card at 8.4 megapixels is over 1,000
/// DPI, which costs time and adds nothing — the ink has no more detail in it
/// than the source had. 300 is the scanning standard and is the right ceiling
/// even though it is the wrong *target*.
pub const MAX_DPI: f32 = 300.0;

/// The least resolution a page is ever rasterized at, in DPI.
///
/// A floor for pages so large that [`TARGET_PIXELS`] would ask for less than
/// one device pixel per point. Below this the recognition crops are too small
/// to carry a glyph at all, and the honest failure — a refusal, or a page of
/// nonsense the disclosure warns about — is preferable to spending the time.
pub const MIN_DPI: f32 = 50.0;

/// The engine directory name, re-exported so the shell names it once.
///
/// `pdfcer_core::ocr::engine_ocrs::MODEL_DIR` when the recogniser is compiled
/// in; the same literal otherwise, because a build without the engine still
/// has to be able to say *where* the models it cannot use would have gone.
#[cfg(feature = "ocrs")]
pub const MODEL_DIR: models::EngineDirName = pdfcer_core::ocr::engine_ocrs::MODEL_DIR;
/// See the `ocrs`-enabled twin above.
#[cfg(not(feature = "ocrs"))]
pub const MODEL_DIR: models::EngineDirName = "ocrs";

/// Why recognition did not happen, in the operator's terms.
///
/// Every variant is a **named** cause with a different action behind it. The
/// engine's own error type does the same thing and for the same stated reason:
/// on a portable install "the weights are not beside the binary" is the most
/// likely failure by a wide margin and is entirely fixable — but only if the
/// message says so.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Refusal {
    /// ★ **This page already draws text**, so recognising it would add an
    /// invisible duplicate rather than make anything findable.
    ///
    /// Measured 2026-08-27: a second pass over a recognised page takes it from
    /// 427 extracted codes to 854 — `add_ocr_layer` adds a layer, it does not
    /// replace one. So this is a refusal rather than a warning, and it is the
    /// default, exactly as `--skip-text` is OCRmyPDF's.
    ///
    /// Per **page**, never per run: on a mixed document — a scanned drawing
    /// bound with a typed cover sheet — the cover is skipped and the scan is
    /// recognised, which is the outcome the operator wants and would not think
    /// to ask for.
    AlreadyHasText,
    /// **The operator pressed Cancel.** Nothing was kept.
    ///
    /// ★ A refusal rather than an outcome, because every caller that handles
    /// "nothing came back" already handles this shape — and because it IS a
    /// refusal from the run's point of view: it produced no result, on purpose.
    /// The count is carried so the status line can say what was discarded
    /// rather than only that something was.
    Cancelled {
        /// Pages attempted before the press.
        attempted: usize,
    },
    /// This build was compiled without the `ocrs` feature.
    ///
    /// Distinct from [`Self::ModelsMissing`] on purpose: *cannot look* and
    /// *could not find the files to look with* are different problems with
    /// different fixes, and `pdfcer-core`'s feature block is explicit that
    /// "found no text" and "cannot look for text" must never be the same
    /// answer, least of all on a scan.
    EngineAbsent,
    /// No `models/<engine>` directory was found. Carries every path tried, in
    /// the order they were tried.
    ModelsMissing(Vec<PathBuf>),
    /// There is no such page.
    NoSuchPage(usize),
    /// The page has no area to rasterize.
    EmptyPage,
    /// Recognition ran and placed no word.
    NothingRecognised,
    /// The engine, the rasterizer or the layer writer refused, carrying its
    /// own sentence rather than a paraphrase of it.
    Engine(String),
}

/// A finished recognition, before it is anywhere on disk.
///
/// ★ **The bytes and the report travel together and are only ever handed over
/// together.** `pdfcer-core`'s report type says a caller "that builds a layer
/// and drops the report has made pdfcer silent about a page of guesses", and
/// keeping them in one struct is how that is made awkward to do by accident.
#[derive(Debug, Clone)]
pub struct Recognised {
    /// **How many pages had been attempted when the operator pressed Stop**, or
    /// `None` for a run that finished on its own.
    ///
    /// ★★★ Carried on the result rather than inferred from a page count,
    /// because the two are not the same: a complete run over pages that were
    /// all skipped also has fewer written pages than requested. Only this
    /// distinguishes *"the document is done"* from *"the operator ended it at
    /// page 40"* — and reporting the second as the first is how somebody
    /// discovers, months later, that a word on page 150 is not in the layer.
    pub stopped_after: Option<usize>,
    /// ★★★ **The recognised words, per page, ready to be applied to the open
    /// session as one undoable edit.**
    ///
    /// # This used to be `bytes: Vec<u8>` — a whole PDF — and the change is the
    /// point
    ///
    /// `pdfcer_core::ocr::layer::add_ocr_layer` takes an immutable `&Document`
    /// and hands back a complete file, which made recognition the one
    /// capability in pdfcer that was not an *edit*. A shell holding an open
    /// session could only offer *"here is a different file, somewhere else"*,
    /// and the operator said what he thought of that on 2026-08-26: *"Why do I
    /// have to save a copy instead of just go back into my pdf and save over
    /// it?"*
    ///
    /// `EditSession::add_ocr_layer` landed in the engine on 2026-08-27 (Pass
    /// 135.0). The layer goes into the session, the ordinary Save writes it,
    /// undo takes it back out, and the whole Save-as apparatus this dialog grew
    /// around the old signature is gone.
    ///
    /// # ★★ And it deletes the unsaved-edits refusal, which no guard could fix
    ///
    /// The free function read the document's **base** revision, so a recognised
    /// copy taken after any edit silently omitted that edit. This shell
    /// therefore refused to run OCR on a dirty session — correctly, because
    /// silent omission is worse than a refusal. But a session never becomes
    /// clean again, not even after a save, so **OCR died for the rest of the
    /// session the first time the operator edited and saved anything.**
    ///
    /// The verb plans against the session graph, so the divergence is removed
    /// rather than policed, and the guard has nothing left to guard.
    ///
    /// Paired `(page_index, words)` rather than two parallel vectors, matching
    /// `OcrPageLayer`'s own reasoning: two lists can differ in length or in
    /// order, and either mistake puts one page's words on another page with no
    /// diagnostic short of reading the output.
    pub pages: Vec<(usize, pdfcer_core::ocr::OcrPage)>,
    /// The resolution the page was actually rasterized at.
    ///
    /// Derived from the page's area by [`fitted_dpi`], so it varies per page and
    /// is not a constant anyone can look up. Reported rather than assumed: it is
    /// the single number that most affects what comes back, and a recognition
    /// that read badly at a resolution nobody was told about would be blamed on
    /// the engine.
    pub effective_dpi: f32,
    /// How many words the recogniser produced before the layer writer filtered
    /// them.
    ///
    /// Carried beside `report.words_written` so the two can be compared. They
    /// differ exactly when words were dropped as unplaceable, which is a real
    /// diagnosis — a large gap means the engine and the page geometry disagree
    /// — and it is invisible from either number alone.
    pub words_recognised: usize,
    /// ★ **How many pages produced words**, across a multi-page run.
    ///
    /// `1` for the single-page case this used to be the only shape of. Reported
    /// so the dialog can say *"12 of 36 pages"* rather than a word count alone,
    /// which on a long scan tells the operator nothing about coverage.
    pub pages_written: usize,
    /// How many pages were visited and produced nothing — blank sheets,
    /// photographs with no text, and pages skipped because they already had
    /// text (see [`Request::skip_pages_with_text`]).
    ///
    /// Reported rather than hidden: a run over forty pages that wrote two is a
    /// result the operator needs to see, and a bare *"success"* would let them
    /// believe the other thirty-eight had been done.
    pub pages_skipped: usize,
}

/// Device pixels per PDF user-space unit for a given DPI.
///
/// `dpi / 72.0`, because a PDF user-space unit is 1/72 inch by definition
/// (ISO 32000-1 §8.3.2.3). One line, in one place, so no call site does the
/// division by hand and gets 96 into it.
#[must_use]
pub fn raster_scale(dpi: f32) -> f32 {
    dpi / 72.0
}

/// The DPI to rasterize a page of `width_pt` × `height_pt` at.
///
/// Solves [`TARGET_PIXELS`] for this page's area, then clamps to
/// [`MIN_DPI`]..=[`MAX_DPI`]. Returns a DPI rather than a scale so that the
/// number reported to the operator and the number handed to the rasterizer are
/// derived from one another instead of computed twice.
///
/// A page with no area yields [`MAX_DPI`] rather than infinity: the caller has
/// already refused an empty page by then, and a non-finite scale out of a clamp
/// would be a worse failure than the one it is guarding.
#[must_use]
pub fn fitted_dpi(width_pt: f64, height_pt: f64) -> f32 {
    let area_in_sq_inches = (width_pt / 72.0) * (height_pt / 72.0);
    // `is_sign_positive` beside `is_finite` rather than `> 0.0`, and the pair is
    // exact rather than defensive: a NaN compares `false` against every ordering
    // operator, so `!(x > 0.0)` catches it but reads as though it were about
    // sign, and `x > 0.0` alone would let `inf` through. `is_finite` rejects
    // both NaN and infinity; `is_sign_positive` then rejects zero's and a
    // negative's sign. `pdfcer-core`'s own `add_image` records the same trap.
    if !area_in_sq_inches.is_finite() || area_in_sq_inches <= 0.0 {
        return MAX_DPI;
    }
    #[allow(clippy::cast_precision_loss)]
    let ideal = ((TARGET_PIXELS as f64) / area_in_sq_inches).sqrt();
    #[allow(clippy::cast_possible_truncation)]
    let ideal = ideal as f32;
    ideal.clamp(MIN_DPI, MAX_DPI)
}

/// RGBA (or BGRA) pixels to 8-bit greyscale, row-major and top-down.
///
/// # Why the luma weights and not a plain average
///
/// ITU-R BT.601's `0.299 R + 0.587 G + 0.114 B` — the same coefficients
/// `pdfcer-core`'s own JPEG paths use. A flat average treats a saturated blue
/// stamp as mid-grey and a yellow highlighter as near-white, which is exactly
/// backwards for a page that has been marked up: the blue ink a human reads
/// easily would fade and the yellow wash the human ignores would swallow the
/// text under it.
///
/// # Why the channel order does not matter here
///
/// `tiny_skia::Pixmap` is premultiplied RGBA. The weights below are applied in
/// that order. If a future backend hands over BGRA the red and blue weights
/// swap, which shifts a *coloured* pixel's grey by at most 0.185 of full scale
/// and leaves every neutral pixel — which is nearly all of a scan — exactly
/// where it was. Stated rather than guarded, because a guard against a
/// hypothetical byte order would be untestable here.
///
/// Alpha is ignored: the rasterizer is asked for a white-backed page, so every
/// pixel is already composited and an alpha channel that is uniformly opaque
/// carries no information.
#[must_use]
pub fn greyscale(rgba: &[u8], width: u32, height: u32) -> Vec<u8> {
    let expected = (width as usize).saturating_mul(height as usize);
    let mut out = Vec::with_capacity(expected);
    for px in rgba.chunks_exact(4).take(expected) {
        let luma = 0.299_f32.mul_add(
            f32::from(px[0]),
            0.587_f32.mul_add(f32::from(px[1]), 0.114 * f32::from(px[2])),
        );
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        out.push(luma.clamp(0.0, 255.0) as u8);
    }
    // A short buffer is padded to white rather than truncated. The engine
    // validates `len == w*h` and rejects a mismatch outright — which would
    // turn a rasterizer quirk into an unexplained refusal — and white is the
    // colour of paper the recogniser will correctly find nothing on.
    out.resize(expected, 0xFF);
    out
}

/// Where this shell looks for model files.
///
/// Two locations, in `pdfcer-core`'s order: beside the running executable
/// (the portable-folder case, which is how `tools/package-portable.py` ships
/// them), then the platform user-data directory (so a developer running out of
/// `target/` can put them somewhere durable without copying 12 MB into a
/// build output that `cargo clean` deletes).
///
/// No operator-supplied path is passed today because no setting offers one.
/// `resolve_model_dir`'s first parameter is left `None` rather than
/// synthesised, which keeps its documented rule — *a named path that is
/// missing is an error, never a silent fallback* — reachable the day a setting
/// exists.
///
/// # Errors
///
/// [`models::ModelsNotFound`], carrying every path that was tried, which is
/// the actionable half of the message.
pub fn resolve_models(
    exe_dir: Option<&Path>,
    user_data: Option<&Path>,
) -> Result<models::ModelSource, models::ModelsNotFound> {
    // ★★★ `_with`, NAMING THE FILES — adopted 2026-08-26, and it closes a
    // shadowing hazard rather than tidying a call.
    //
    // The plain `resolve_model_dir` asks only `is_dir()`. So an **empty**
    // `models/ocrs` beside the executable RESOLVES — and, worse, it wins the
    // search order, so an operator's own good copy further down is never
    // reached. The failure then surfaces later and in the wrong vocabulary: the
    // engine reports a missing model file after this shell has already told
    // them the models were found.
    //
    // ★ The filenames are the engine's own published constants, not string
    // literals invented here. A shell that spelled them itself would keep
    // resolving successfully on the day the engine renamed one, and would fail
    // one layer down with a message about a file nobody asked for.
    models::resolve_model_dir_with(
        MODEL_DIR,
        None,
        exe_dir,
        user_data,
        &[
            pdfcer_core::ocr::engine_ocrs::DETECTION_MODEL,
            pdfcer_core::ocr::engine_ocrs::RECOGNITION_MODEL,
        ],
    )
}

/// The directory the running executable is in, if it can be determined.
///
/// `None` rather than a guess when `current_exe` fails: a wrong directory here
/// produces "models not found" naming a path nobody has, which is worse than
/// naming one fewer place that was genuinely searched.
#[must_use]
pub fn exe_dir() -> Option<PathBuf> {
    std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(Path::to_path_buf))
}

/// Everything a recognition needs, assembled on the UI thread and moved whole
/// onto the worker.
///
/// A struct rather than six arguments because it is what crosses the thread
/// boundary, and because the compiler then checks that every field is `Send`
/// in one place instead of at a `spawn` call.
#[derive(Clone)]
pub struct Request {
    /// The session to read. Only its **base document** is used — see the
    /// module header on why, and [`Refusal::UnsavedEdits`] for what guarantees
    /// that base is what the operator is looking at.
    ///
    /// ★ Kept as the FALLBACK since 2026-08-26. When [`Self::source`] is
    /// `Some`, the file on disk is read instead — see that field.
    pub session: Arc<EditSession>,
    /// ★★★ **The pages to recognise, zero-based, in order.**
    ///
    /// # Why this is a list
    ///
    /// The operator, 2026-08-26: *"how do I OCR more than one page? Why does
    /// the tool stop at one? […] Where is the option to select more than one
    /// page? How did we end up with the most useless and un-userfriendly of
    /// options for the OCR?"*
    ///
    /// It was a `usize`. Nothing in `pdfcer-core` required that — the engine's
    /// own `add_ocr_layer` takes one page at a time, but its output is a
    /// complete PDF that can be fed straight back in, so a caller can chain
    /// them. **Measured before this was built**, because the audit flagged it
    /// UNVERIFIED and a wrong answer would have corrupted a file: two
    /// successive in-place recognitions of one page produce a document that
    /// round-trips byte-identical and extracts both layers. Revisions chain.
    ///
    /// ★★ **And the same measurement found a hazard**: a second pass over a
    /// page that already has a layer **adds a second one** — 427 codes became
    /// 854 — rather than replacing it. It does not affect a run that visits
    /// each page once, which is every run this shell issues, but it is why
    /// [`Self::skip_pages_with_text`] exists and defaults to on.
    ///
    /// Empty is not a valid request and the dialog will not build one; if one
    /// arrives, the job reports [`Refusal::NothingRecognised`] rather than
    /// succeeding at nothing.
    pub pages: Vec<usize>,
    /// ★★ **Leave alone any page that already has real text on it.**
    ///
    /// The default, and the safe one. It is OCRmyPDF's `--skip-text`, which is
    /// that tool's default for the same reason: recognising a page that is
    /// already text adds an invisible duplicate of text the file already has,
    /// which doubles every search hit and every copy.
    ///
    /// ★ "Real text" means text the page draws **visibly**. A page that already
    /// carries an invisible OCR layer counts as having text, so re-running over
    /// a recognised document is the no-op an operator would expect rather than
    /// a doubling.
    pub skip_pages_with_text: bool,
    /// ★ **The operator's extraction settings**, carried rather than defaulted.
    ///
    /// Read only by the [`Self::skip_pages_with_text`] guard, and it would have
    /// been tempting to call `ExtractOptions::default()` at the point of use —
    /// the guard only asks whether a page has *any* text, and no setting turns
    /// text into no text.
    ///
    /// `app::settings::tests::no_call_site_builds_its_own_options` refused it,
    /// and it was right to. The rule it enforces is that **no call site in this
    /// application decides extraction options for itself**, because the one
    /// that does is invisible until the day a setting starts mattering to it
    /// and nobody remembers this was the exception. `unmappable_code` alone is
    /// enough to make the reasoning above wrong: a page whose every glyph is
    /// unmappable extracts to a run of sentinels under one value and to nothing
    /// under another.
    ///
    /// Built on the UI thread by the dialog, where `Settings` lives, and moved
    /// to the worker with the rest of the request.
    pub extract_options: pdfcer_core::text_extract::ExtractOptions,
    /// The directory holding the two `.rten` files.
    pub model_dir: PathBuf,
}

/// The worker body. Runs on the spawned thread; touches no GUI type.
///
/// Written as a free function taking `&Request` for the same reason
/// `render::worker::render_on_worker` is: a body that cannot reach `self` is a
/// body that provably shares nothing with the UI thread.
/// ★★★ **Recognise every requested page, chaining the revisions.**
///
/// # The shape, and why it is a fold rather than a map
///
/// `add_ocr_layer` takes a whole `Document` and returns a whole PDF. So page
/// two must be recognised **against the output of page one**, not against the
/// original — otherwise the second write would be an incremental revision over
/// a base that does not have the first layer, and the first page's words would
/// be silently dropped.
///
/// That chaining was the audit's one UNVERIFIED risk and it was measured before
/// this was written: two successive in-place recognitions produce a file that
/// round-trips byte-identical and extracts both layers.
///
/// # What a failure on one page does to the rest
///
/// **Nothing.** A page with no recognisable text — a blank sheet, a photograph
/// of a wall — reports `NothingRecognised`, and on a forty-page scan that must
/// not abandon the other thirty-nine. So a per-page refusal is *counted*, not
/// propagated, and the run reports how many pages produced words.
///
/// The exception is an **engine** failure, which is not about the page: if the
/// recogniser itself is broken, every remaining page will fail the same way and
/// grinding through thirty-nine more is only a slower way to say so.
///
/// # ★ A run that recognised nothing anywhere is a refusal
///
/// If no page produced a single word, there is nothing to write and nothing to
/// save, and reporting success would leave the operator with a dialog saying it
/// worked and a document with nothing in it.
pub(in crate::ocr) fn recognise(
    request: &Request,
    report: &job::Reporter,
) -> Result<Recognised, Refusal> {
    if request.pages.is_empty() {
        return Err(Refusal::NothingRecognised);
    }
    // `Some(n)` once Stop has been honoured, carrying how many pages were
    // attempted. Read by the caller to choose `Outcome::Stopped` over
    // `Outcome::Complete` — the two must not be confused, or a run ended at
    // page 40 of 200 reports as a whole document recognised.
    let mut stopped_after: Option<usize> = None;

    // ONCE, before the loop. See `Recogniser` for what this used to cost.
    let recogniser = Recogniser::load(&request.model_dir)?;

    let mut pages = Vec::new();
    let mut total_words = 0usize;
    let mut pages_skipped = 0usize;
    let mut dpi = 0.0f32;
    // ★★ Counted separately from `pages_skipped`, and only so that a run which
    // produced nothing can say WHY. See the refusal below.
    let mut already_had_text = 0usize;

    let of = request.pages.len();
    for (attempted, &page_index) in request.pages.iter().enumerate() {
        // ★★★ **CHECKED BETWEEN PAGES, NEVER INSIDE ONE**, and that is what
        // makes "Stop keeps the finished pages" true by construction: there is
        // no moment in this loop at which a half-recognised page exists.
        //
        // Cancel is checked here too rather than only at the top, so an
        // abandonment costs at most the page in hand — the same bound Stop
        // has, for the same reason. What differs is what happens to that page,
        // not how long it takes.
        match report.wish() {
            progress::Wish::Cancel => {
                return Err(Refusal::Cancelled { attempted });
            }
            progress::Wish::StopAfterThisPage => {
                stopped_after = Some(attempted);
                break;
            }
            progress::Wish::Continue => {}
        }
        match recognise_one(request, &recogniser, page_index) {
            Ok(one) => {
                total_words += one.words;
                dpi = one.dpi;
                // ★ Sent BEFORE the page is pushed, so the count the operator
                // reads is the count of pages ATTEMPTED rather than kept. A
                // progress line that stalled on a run of skipped pages would
                // look exactly like the freeze this feature exists to disprove.
                report.page(progress::PageDone {
                    index: page_index,
                    attempted: attempted + 1,
                    of,
                    words: one.words,
                    chars: one.chars,
                });
                pages.push((page_index, one.recognised));
            }
            // A page with nothing on it is not a failure of the run.
            Err(Refusal::NothingRecognised) => {
                pages_skipped += 1;
                report.page(progress::PageDone {
                    index: page_index,
                    attempted: attempted + 1,
                    of,
                    words: 0,
                    chars: 0,
                });
            }
            Err(Refusal::AlreadyHasText) => {
                pages_skipped += 1;
                already_had_text += 1;
                report.page(progress::PageDone {
                    index: page_index,
                    attempted: attempted + 1,
                    of,
                    words: 0,
                    chars: 0,
                });
            }
            Err(other) => return Err(other),
        }
    }

    if pages.is_empty() {
        // ★★★ **WHICH nothing, and the distinction was found by driving.**
        //
        // A full driven run on 2026-08-27 pointed this at the operator's own
        // CAD sheet — every page of which already has text — and got
        // `NothingRecognised`, which reads as *"the recogniser could not read
        // your document"*. It had not looked at it. The remedy for the two is
        // different: one is "there is nothing readable here", the other is
        // "turn off the skip if you meant it", and only the second is
        // actionable.
        //
        // `> 0` rather than `== request.pages.len()`: a run where some pages
        // were blank and some already had text still has the skip as its
        // actionable cause, and a mixed run reported as "nothing readable"
        // sends the operator looking at their scanner.
        return Err(if already_had_text > 0 {
            Refusal::AlreadyHasText
        } else {
            Refusal::NothingRecognised
        });
    }
    Ok(Recognised {
        pages_written: pages.len(),
        pages,
        effective_dpi: dpi,
        words_recognised: total_words,
        pages_skipped,
        stopped_after,
    })
}

/// What recognising one page produced, before anything is applied.
struct OnePage {
    /// The words, in PDF default user space, y-up.
    recognised: pdfcer_core::ocr::OcrPage,
    /// How many the recogniser produced.
    words: usize,
    /// How many characters those words hold.
    ///
    /// ★ Asked for by name on 2026-09-01, and it is the better of the two for
    /// showing that a long run is alive: a dense drawing can yield hundreds of
    /// characters inside a handful of "words", so this number moves when the
    /// word count barely does.
    chars: usize,
    /// What it was rasterized at.
    dpi: f32,
}

/// One page: rasterize it, read it, and put the words in page space.
///
/// ★ **Applies nothing.** It used to call `add_ocr_layer` and return a whole
/// PDF; the writing now happens once, on the UI thread, for the whole run. That
/// is what makes a forty-page recognition **one** undo entry rather than forty.
fn recognise_one(
    request: &Request,
    recogniser: &Recogniser,
    page_index: usize,
) -> Result<OnePage, Refusal> {
    // ★★★ **THE SESSION'S VIEW, NOT ITS BASE AND NOT THE FILE.**
    //
    // This is the whole reason the engine grew a session verb. The old code
    // read the operator's file off disk — correct at the time, because
    // `add_ocr_layer` wrote an incremental revision over the base and anything
    // else would have produced a copy with his saved work missing.
    //
    // `EditSession::add_ocr_layer` plans against the session graph, so the
    // words must be recognised from what the session currently *draws*. A page
    // the operator has edited this session renders differently from both its
    // base and its file, and recognising either would put words where the ink
    // no longer is.
    let view = request.session.view();
    let pages = pages_of(request)?;
    let page = pages
        .get(page_index)
        .ok_or(Refusal::NoSuchPage(page_index))?;

    // ★★ **The doubling guard, and the reason it is measured rather than
    // assumed.**
    //
    // Measured 2026-08-26 on a one-page fixture: recognising an
    // already-recognised page took it from **427 character codes to 854**. The
    // OCR layer is Table 106 mode 3 — rendered but invisible — so nothing on
    // screen changes and nothing warns. What changes is that every Find match
    // is doubled and every copy comes out twice.
    //
    // An extraction failure is NOT treated as "has text". A page whose content
    // stream will not parse is exactly the kind of page a recogniser is for,
    // and refusing to look at it because the parser gave up would be the guard
    // producing the harm it exists to prevent.
    if request.skip_pages_with_text {
        let has_text = pdfcer_core::text_extract::extract_page_view(
            &view,
            page,
            page_index,
            &request.extract_options,
        )
        .map(|text| !text.plain_text().trim().is_empty())
        .unwrap_or(false);
        if has_text {
            return Err(Refusal::AlreadyHasText);
        }
    }

    let box_ = page.crop_box;
    let width_pt = box_.urx - box_.llx;
    let height_pt = box_.ury - box_.lly;
    // `is_finite` first, then a plain comparison — see [`fitted_dpi`] for why
    // the negated form is avoided. A degenerate crop box is not hypothetical:
    // a malformed `/CropBox` normalises to a zero-area rect rather than
    // failing, and rasterizing one would produce an image with no pixels for
    // the recogniser to reject in a less comprehensible way.
    if !width_pt.is_finite() || !height_pt.is_finite() || width_pt <= 0.0 || height_pt <= 0.0 {
        return Err(Refusal::EmptyPage);
    }

    let dpi = fitted_dpi(width_pt, height_pt);
    let rendered = pdfcer_render::render_page_view(&view, page, raster_scale(dpi))
        .map_err(|e| Refusal::Engine(e.to_string()))?;
    let (w, h) = (rendered.pixmap.width(), rendered.pixmap.height());
    let grey = greyscale(rendered.pixmap.data(), w, h);

    let words = recogniser.recognise(w, h, &grey)?;
    let words_recognised = words.len();
    // ★ The flip, and the ONLY place it happens. See the module header.
    //
    // ★★★ `..._on` WITH THE PAGE'S `/Rotate`, NEVER the bare
    // `words_to_page_space` — corrected 2026-08-25 on the engine's report
    // (Pass 129.0).
    //
    // `pdfcer-render` honours `/Rotate`: `page_device_geometry` swaps the
    // raster's axes at 90° and 270°. The mapping BACK to page space did not,
    // so on an odd quarter turn every recognised word landed on the wrong axis
    // at the wrong scale.
    //
    // ★★ And the failure is invisible by construction, which is why it needed
    // reporting rather than noticing. The OCR layer is Table 106 mode 3 —
    // rendered but not shown — so a page whose every word is misplaced **looks
    // exactly like a page whose every word is right**. The only symptom is that
    // selecting or searching picks the wrong thing, and an operator meeting
    // that would reasonably blame the recogniser rather than the geometry.
    //
    // ★ Not an edge case in the one population OCR exists for: scanner drivers
    // and "rotate pages" commands in other tools write `/Rotate` rather than
    // re-imaging the pixels, so a rotated scan is the norm rather than the
    // exception.
    let placed = pdfcer_core::ocr::words_to_page_space_on(
        &words,
        w,
        h,
        pdfcer_core::ocr::PagePlacement::new(
            // `page_rect` is the CROP box rather than the media box: the
            // rasterizer draws the crop box (Table 30 — content is clipped to
            // it at display time), so the image the recogniser saw covers
            // exactly that region and nothing else. Handing the media box here
            // would offset and scale every word by the difference on any page
            // whose two boxes differ, which is most scanned material and all
            // trimmed drawings.
            Rect::from_corners(box_.llx, box_.lly, box_.urx, box_.ury),
            i32::from(page.rotate),
        ),
    );
    if placed.is_empty() {
        return Err(Refusal::NothingRecognised);
    }

    // ★ Counted from the words that were actually PLACED, not from what the
    // recogniser emitted — the two differ whenever a word is dropped on the way
    // into page space, and the number an operator watches must describe what
    // ended up in their document.
    let chars_recognised = placed.iter().map(|w| w.text.chars().count()).sum();

    Ok(OnePage {
        recognised: OcrPage {
            words: placed,
            // Asked of the engine rather than assumed.
            // `OcrEngine::reports_confidence` is a required method with no
            // default precisely so this cannot be guessed at either
            // optimistically or pessimistically.
            confidence_available: reports_confidence(),
        },
        words: words_recognised,
        chars: chars_recognised,
        dpi,
    })
}

/// The page list, with the page-tree error carried out by name.
fn pages_of(request: &Request) -> Result<Vec<page_tree::Page>, Refusal> {
    request
        .session
        .pages()
        .map_err(|e| Refusal::Engine(e.to_string()))
}

/// Whether the compiled-in recogniser scores its output.
///
/// **`false` today, and that is a fact about `ocrs` rather than a placeholder**
/// — its output type is a character and a rectangle, with no score on a
/// character, a word, a line or the page. Read through a function rather than
/// written as a literal at the call site so that the day a second engine lands
/// there is one place that has to learn to ask it.
#[must_use]
fn reports_confidence() -> bool {
    #[cfg(feature = "ocrs")]
    {
        use pdfcer_core::ocr::OcrEngine as _;
        // Answered by the type rather than by a constant, so a future upstream
        // change is picked up rather than contradicted. Constructing an engine
        // just to ask would need the models, so the answer is taken from a
        // value that does not exist — which is why this is written as a match
        // on the trait's own implementation through a zero-sized shim below.
        struct Never;
        impl pdfcer_core::ocr::OcrEngine for Never {
            type Error = std::io::Error;
            fn recognize(
                &self,
                _w: u32,
                _h: u32,
                _p: &[u8],
            ) -> Result<Vec<pdfcer_core::ocr::RecognizedWord>, Self::Error> {
                unreachable!("the shim is never recognised with")
            }
            fn reports_confidence(&self) -> bool {
                // Mirrors `OcrsEngine::reports_confidence`, which returns
                // `false` because there is no score to report.
                false
            }
        }
        Never.reports_confidence()
    }
    #[cfg(not(feature = "ocrs"))]
    {
        false
    }
}

/// ★★ **The loaded recognition models, held for the whole run.**
///
/// # Why this is a type rather than a function call per page
///
/// It used to be `recognise_image(model_dir, …)`, which read every model file
/// off disk and built the engine **once per page**. That was invisible while
/// the dialog could only do one page. It stops being invisible the moment a
/// fifty-page run exists: the detection and recognition models are tens of
/// megabytes, and paying for them fifty times is the difference between a run
/// an operator waits through and one they abandon.
///
/// The gap document called this out as the one thing that had to change
/// *underneath* the new page-scope control rather than beside it — a scope
/// selector over a per-page model load would have shipped a feature whose cost
/// grew with the number the operator typed.
///
/// # Why it still carries the whole feature gate
///
/// The `ocrs`-absent twin below has the same two methods and refuses by name.
/// Everything above this line compiles and runs identically in a stripped
/// build, which is what makes the gated-out path a **named refusal** rather
/// than a silently different program — R8's rule, applied to a Cargo feature.
///
/// ★ The stripped build refuses at [`Self::load`], which is **before** any page
/// is rasterized. A build with no recogniser therefore spends no time rendering
/// images it has nothing to read.
#[cfg(feature = "ocrs")]
struct Recogniser(pdfcer_core::ocr::engine_ocrs::OcrsEngine);

#[cfg(feature = "ocrs")]
impl Recogniser {
    /// Read the models off disk. Once per run — see the type's header.
    fn load(model_dir: &Path) -> Result<Self, Refusal> {
        use pdfcer_core::ocr::engine_ocrs::OcrsEngine;
        OcrsEngine::from_model_dir(model_dir)
            .map(Self)
            .map_err(|e| Refusal::Engine(e.to_string()))
    }

    /// Recognise one greyscale image.
    fn recognise(
        &self,
        width: u32,
        height: u32,
        grey: &[u8],
    ) -> Result<Vec<pdfcer_core::ocr::RecognizedWord>, Refusal> {
        use pdfcer_core::ocr::OcrEngine as _;
        self.0
            .recognize(width, height, grey)
            .map_err(|e| Refusal::Engine(e.to_string()))
    }
}

/// See the `ocrs`-enabled twin above.
#[cfg(not(feature = "ocrs"))]
struct Recogniser;

#[cfg(not(feature = "ocrs"))]
impl Recogniser {
    /// A build with no recogniser refuses at the point of loading, which is
    /// before any page is rasterized — so a stripped build spends no time
    /// rendering images it has nothing to read.
    fn load(_model_dir: &Path) -> Result<Self, Refusal> {
        Err(Refusal::EngineAbsent)
    }

    fn recognise(
        &self,
        _width: u32,
        _height: u32,
        _grey: &[u8],
    ) -> Result<Vec<pdfcer_core::ocr::RecognizedWord>, Refusal> {
        Err(Refusal::EngineAbsent)
    }
}

/// Whether this build carries a recogniser at all.
///
/// Read by the dialog before it looks for models: *cannot look* and *could not
/// find the files to look with* are different refusals, and asking in the
/// wrong order would report the second when the first is true.
#[must_use]
pub const fn engine_compiled_in() -> bool {
    cfg!(feature = "ocrs")
}

#[cfg(test)]
#[allow(clippy::float_cmp)]
mod tests {
    use super::*;

    /// The scale is DPI over 72, which is the definition of a user-space unit.
    #[test]
    fn the_raster_scale_is_dpi_over_seventy_two() {
        assert_eq!(raster_scale(72.0), 1.0);
        assert_eq!(raster_scale(MAX_DPI), 300.0 / 72.0);
    }

    /// ★★ **The measured optimum is reproduced for the sheet it was measured
    /// on.**
    ///
    /// `SW41177.pdf`'s first page is 1584 × 1224 pt, and [`TARGET_PIXELS`] is
    /// the megapixel count this function is built around — so a page of that
    /// size must come out at the DPI the constant implies. That is arithmetic,
    /// and it holds whatever the constant's *value* turns out to be.
    ///
    /// # ★★★ This asserts the RELATIONSHIP, not the number — rewritten 2026-08-25
    ///
    /// It used to assert `145..=155 DPI`, on the grounds that 150 was the best
    /// row of a measured accuracy table. **That table was retracted**: it was
    /// produced by a text-detection model that did not work (see
    /// [`TARGET_PIXELS`]), so the 150 it pinned was a property of noise.
    ///
    /// The engine's note on the retraction made the general point, and it is
    /// why this test is shaped differently now:
    ///
    /// > *"A test that asserts a number fails on every legitimate change, and
    /// > gets edited without the evidence."*
    ///
    /// Exactly so. Had the sweep been re-run and `TARGET_PIXELS` moved, this
    /// test would have gone red for a **correct** change — and the cheapest way
    /// to make it green is to edit the band, which quietly destroys the link it
    /// existed to protect. It now asserts what cannot become false by
    /// re-measuring: that this page's DPI is the one `TARGET_PIXELS` implies.
    ///
    /// ★ When the sweep is re-run and the constant moves, this test should
    /// **pass unchanged**. If it does not, the fitting arithmetic has come
    /// apart — which is precisely what the old version could not tell you.
    #[test]
    fn the_benchmark_sheet_lands_on_the_dpi_the_target_implies() {
        let (w_pt, h_pt) = (1584.0_f64, 1224.0_f64);
        let dpi = f64::from(fitted_dpi(w_pt, h_pt));

        // The DPI at which this page is exactly TARGET_PIXELS: solve
        // (w/72 · d) · (h/72 · d) = TARGET_PIXELS for d.
        #[allow(clippy::cast_precision_loss)] // 8.4e6 is exact in f64
        let implied = (TARGET_PIXELS as f64 / (w_pt / 72.0 * (h_pt / 72.0))).sqrt();

        assert!(
            (dpi - implied).abs() <= 1.0,
            "a {w_pt}×{h_pt} pt page should rasterise at {implied:.1} DPI to reach TARGET_PIXELS, and this function chose {dpi:.1} — the fitting arithmetic and the constant have come apart"
        );
        #[allow(clippy::cast_possible_truncation)] // bounds check only
        let as_f32 = dpi as f32;
        assert!(
            (MIN_DPI..=MAX_DPI).contains(&as_f32),
            "and the answer must be inside the clamp: {dpi:.1}"
        );
    }

    /// The clamp is a real band, and a page too large for the target still
    /// gets a usable resolution rather than a fraction of one.
    ///
    /// `1.0e6` points square is 13,888 inches on a side — not a page anyone has,
    /// and that is the point: the assertion is about the clamp holding at the
    /// far end of the range rather than about a realistic sheet. Without the
    /// floor, `fitted_dpi` would answer about 0.2 DPI there, and a raster of a
    /// few hundred pixels would be handed to the recogniser as though it were a
    /// page.
    ///
    /// The ordering of the two constants is asserted through `fitted_dpi`'s
    /// behaviour rather than by comparing them directly: clippy rejects the
    /// direct comparison as constant-valued, and it is right to — a literal
    /// `MIN_DPI < MAX_DPI` is checked by the compiler's own constant folding
    /// and tells a reader nothing the two declarations do not.
    #[test]
    fn an_impossibly_large_page_still_gets_a_usable_resolution() {
        let dpi = fitted_dpi(1.0e6, 1.0e6);
        assert_eq!(dpi, MIN_DPI, "the floor must bind, not the target");
        assert!(
            dpi < MAX_DPI,
            "…and the floor must be below the ceiling, or the clamp has no band at all"
        );
    }

    /// A small page is capped rather than magnified absurdly.
    ///
    /// A business card at 8.4 megapixels is over 1,000 DPI — resolution with no
    /// ink behind it, paid for in seconds.
    ///
    /// ★ **US Letter is the interesting row and is asserted separately.** It
    /// lands at 299.7 DPI, a quarter of a DPI under the ceiling: the measured
    /// 8.4-megapixel target and the conventional 300-DPI scanning standard
    /// coincide almost exactly on the commonest page size in the world. That is
    /// a coincidence rather than a design, and it is pinned because it explains
    /// something a reader would otherwise find contradictory — the module header
    /// says 300 DPI measured *worst*, and on a Letter page 300 DPI is what this
    /// function will very nearly choose. Both are true: the figure that ruined
    /// recognition was 300 DPI on a **36-inch drawing sheet**, which is 33
    /// megapixels, not 8.4.
    #[test]
    fn a_small_page_is_capped_at_the_scanning_standard() {
        assert_eq!(fitted_dpi(180.0, 90.0), MAX_DPI, "a business card");
        assert_eq!(fitted_dpi(72.0, 72.0), MAX_DPI, "one square inch");

        let letter = fitted_dpi(612.0, 792.0);
        assert!(
            (299.0..=300.0).contains(&letter),
            "US Letter should land just under the ceiling, got {letter}"
        );
    }

    /// ★ **An A0 sheet is reduced, and the reduction lands near the target.**
    ///
    /// 3370 × 2384 pt at 300 DPI would be 138 megapixels and 550 MB of RGBA
    /// before anything is recognised. More to the point, the measurement says a
    /// raster that large is where this engine reads *worst* — so the reduction
    /// is about accuracy first and memory second, which is the opposite of how
    /// the first version of this code justified it.
    #[test]
    fn an_enormous_sheet_is_reduced_towards_the_target() {
        let dpi = fitted_dpi(3370.0, 2384.0);
        assert!(dpi < MAX_DPI, "the target must bind on A0, got {dpi}");
        assert!(dpi >= MIN_DPI, "…but never below the floor, got {dpi}");
        let px = f64::from(dpi * 3370.0 / 72.0) * f64::from(dpi * 2384.0 / 72.0);
        #[allow(clippy::cast_precision_loss)]
        let target = TARGET_PIXELS as f64;
        assert!(
            px <= target * 1.05,
            "the reduced raster is {px:.0} pixels, well over the {target:.0} target"
        );
    }

    /// A degenerate page does not produce a non-finite scale.
    #[test]
    fn a_zero_sized_page_does_not_produce_an_infinite_scale() {
        assert!(fitted_dpi(0.0, 792.0).is_finite());
        assert!(fitted_dpi(612.0, 0.0).is_finite());
        assert!(fitted_dpi(f64::NAN, 792.0).is_finite());
    }

    /// Greyscale is one byte per pixel, in the layout the trait requires.
    ///
    /// The engine validates `len == w * h` and refuses a mismatch outright, so
    /// a length bug here would surface as an unexplained engine error rather
    /// than as a bad picture.
    #[test]
    fn greyscale_produces_exactly_one_byte_per_pixel() {
        let rgba = vec![0u8; 4 * 6];
        assert_eq!(greyscale(&rgba, 3, 2).len(), 6);
    }

    /// White stays white and black stays black.
    #[test]
    fn the_extremes_survive_the_conversion() {
        let white = greyscale(&[0xFF, 0xFF, 0xFF, 0xFF], 1, 1);
        let black = greyscale(&[0x00, 0x00, 0x00, 0xFF], 1, 1);
        assert_eq!(white[0], 0xFF);
        assert_eq!(black[0], 0x00);
    }

    /// ★ **A saturated colour is not mid-grey, which a flat average would
    /// make it.**
    ///
    /// The reason the luma weights are there rather than `(r+g+b)/3`. Pure
    /// blue averages to 85 — a mid-tone the binarizer may keep — and weights
    /// to 29, which is ink. Pure green averages to the same 85 and weights to
    /// 150, which is background. A page marked up in blue and highlighted in
    /// yellow is exactly the case where the two disagree, and it is a common
    /// one on a scanned drawing.
    #[test]
    fn a_coloured_pixel_is_weighted_rather_than_averaged() {
        let blue = greyscale(&[0x00, 0x00, 0xFF, 0xFF], 1, 1)[0];
        let green = greyscale(&[0x00, 0xFF, 0x00, 0xFF], 1, 1)[0];
        assert_eq!(blue, 29, "0.114 * 255");
        assert_eq!(green, 149, "0.587 * 255");
        assert!(
            blue < 85 && green > 85,
            "a flat average would call both of these 85 and lose the distinction \
             between blue ink and a green wash"
        );
    }

    /// A short buffer is padded to white rather than silently truncated.
    #[test]
    fn a_short_pixel_buffer_is_padded_to_paper_rather_than_shortened() {
        let out = greyscale(&[0x00, 0x00, 0x00, 0xFF], 4, 4);
        assert_eq!(out.len(), 16, "the engine rejects any other length");
        assert_eq!(out[0], 0x00);
        assert_eq!(out[15], 0xFF, "the padding is paper, not ink");
    }

    /// ★ **This engine reports no confidence, and the shell says so.**
    ///
    /// Pinned because the whole disclosure surface turns on it: if this ever
    /// becomes `true` while `ocrs` is still the engine, the dialog would stop
    /// making the "nothing here has been scored" statement and a page of
    /// unscored guesses would present exactly as a page of checked ones.
    #[test]
    fn the_shipped_recogniser_scores_nothing() {
        assert!(
            !reports_confidence(),
            "`ocrs` emits a char and a rectangle and no score; a `true` here would \
             make every word look checked"
        );
    }

    /// The two absences are two different refusals.
    #[test]
    fn a_missing_engine_and_missing_models_are_distinct_refusals() {
        assert_ne!(Refusal::EngineAbsent, Refusal::ModelsMissing(Vec::new()));
    }

    /// The model directory name is the engine's own, not a second spelling.
    #[test]
    fn the_model_directory_is_the_engines_own_name() {
        assert_eq!(MODEL_DIR, "ocrs");
    }

    /// Nothing is resolved from a directory that does not exist, and every
    /// place that was looked in comes back.
    #[test]
    fn a_failed_resolution_reports_everywhere_it_looked() {
        let nowhere = std::env::temp_dir().join("pdfcer-no-models-here-4c1a");
        let err = resolve_models(Some(&nowhere), None).expect_err("nothing is there");
        assert_eq!(err.engine, MODEL_DIR);
        assert_eq!(err.searched.len(), 1);
        assert!(err.to_string().contains("ocrs"));
    }

    /// ★★★ **An EMPTY `models/ocrs` does not resolve, and so cannot shadow a
    /// good copy further down the search order.**
    ///
    /// The hazard `pdfcer-core` built `resolve_model_dir_with` for, and it is
    /// nastier than a plain "not found". Resolution asking only `is_dir()`
    /// means an empty directory beside the executable **wins**: this shell
    /// tells the operator the models were found, their own good copy is never
    /// reached, and the failure surfaces one layer down in the engine's
    /// vocabulary — a missing model file, after we said there was not one.
    ///
    /// ★ Realistic rather than contrived. A part-finished extraction, an
    /// antivirus quarantine that took the weights and left the folder, or an
    /// operator creating the directory by hand before copying into it all
    /// produce exactly this state.
    ///
    /// ★★ The positive half is asserted too, and it is what makes this test
    /// discriminate. Its first draft checked only that an empty directory
    /// fails — which the OLD resolver also does when the path is wrong, so the
    /// test passed against the very code it was written to condemn. Putting a
    /// file in and requiring success is what proves the failure above was about
    /// EMPTINESS rather than about the path.
    #[test]
    fn an_empty_model_directory_is_rejected_but_a_filled_one_resolves() {
        let root = std::env::temp_dir().join("pdfcer-empty-models-9f3b");
        let dir = root.join("models").join(MODEL_DIR);
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&dir).expect("temp dir");

        // Empty: must be refused, or it shadows.
        let err = resolve_models(Some(&root), None)
            .expect_err("an empty models directory must NOT resolve, or it shadows a good one");
        assert_eq!(err.engine, MODEL_DIR);
        assert!(
            !err.searched.is_empty(),
            "the directory must be REPORTED as searched, so the message names a place the operator can go and look"
        );

        // Filled: must be accepted — otherwise the assertion above proves
        // nothing about emptiness.
        for f in [
            pdfcer_core::ocr::engine_ocrs::DETECTION_MODEL,
            pdfcer_core::ocr::engine_ocrs::RECOGNITION_MODEL,
        ] {
            std::fs::write(dir.join(f), b"not a real model, but a real file").expect("write");
        }
        assert!(
            resolve_models(Some(&root), None).is_ok(),
            "a directory containing both model files must resolve"
        );

        let _ = std::fs::remove_dir_all(&root);
    }
}
