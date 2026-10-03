//! # `ocr` — turning the page on screen into an invisible, searchable text layer
//!
//! This module is the **shell's half** of OCR: it decides *what image the
//! recogniser sees*, runs the job off the UI thread, and hands back either a
//! finished document and a disclosure report, or a named refusal. It authors
//! no PDF syntax of any kind — `pdfcer_core::ocr::layer` does all of that, and
//! this module exists partly to make sure nobody re-implements it here.
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/ocr/mod.md`.

/// Builds `fixtures/synthetic-image-only.pdf` and runs the whole chain against
/// it. **Test-only**, and its header is the argument for what a green result
/// there does and does not prove — the short version being that it establishes
/// the plumbing and establishes **nothing** about recognition quality on a real
/// scan, because a rendered raster has none of the degradation that makes OCR
/// hard.
#[cfg(test)]
mod fixture;

/// Pins what the engine makes of `fixtures/ocr-layer.pdf` — a page that already
/// carries an OCR layer, readable without the recogniser and without the model
/// weights. **Test-only.** Its header carries the three controls and the wrong
/// build each one catches; the generator carries the document's shape.
#[cfg(test)]
mod layer_fixture;

/// **A recognition running on a thread**, and the two ways to end it early.
/// Running a recognition is a different subject from performing one. The
/// module header carries why Cancel and Stop must never collapse into one act.
pub mod job;
pub mod progress;

pub use job::{Job, Tally};

/// The models discovery finds under the bundled and extra folders, and which
/// of them this build can run.
pub mod catalog;

/// Which recognisers this build carries, their model directories, and the
/// loaded model a run holds.
mod engines;
pub use engines::{Dictionary, EngineId, OCRCER_MODEL_DIR, OCRCER_MODEL_FILE, available};

use engines::Recogniser;

use std::path::{Path, PathBuf};
use std::sync::Arc;

use pdfcer_core::edit::EditSession;
use pdfcer_core::ocr::{OcrPage, models};
use pdfcer_core::page_tree::{self, Rect};

/// **The raster size recognition is run at, as a pixel count** — measured,
/// not chosen.
pub const TARGET_PIXELS: u64 = 8_400_000;

/// The most resolution a page is ever rasterized at, in DPI.
pub const MAX_DPI: f32 = 300.0;

/// The least resolution a page is ever rasterized at, in DPI.
pub const MIN_DPI: f32 = 50.0;

/// Why recognition did not happen, in the operator's terms.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Refusal {
    /// **This page already draws text**, so recognising it would add an
    /// invisible duplicate rather than make anything findable.
    ///
    ///
    /// Per **page**, never per run: on a mixed document — a scanned drawing
    /// bound with a typed cover sheet — the cover is skipped and the scan is
    /// recognised, which is the outcome the operator wants and would not think
    /// to ask for.
    AlreadyHasText,
    /// **The operator pressed Cancel.** Nothing was kept.
    ///
    /// A refusal rather than an outcome, because every caller that handles
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
#[derive(Debug, Clone)]
pub struct Recognised {
    /// **How many pages had been attempted when the operator pressed Stop**, or
    /// `None` for a run that finished on its own.
    ///
    /// Carried on the result rather than inferred from a page count,
    /// because the two are not the same: a complete run over pages that were
    /// all skipped also has fewer written pages than requested. Only this
    /// distinguishes *"the document is done"* from *"the operator ended it at
    /// page 40"* — and reporting the second as the first is how somebody
    /// discovers, months later, that a word on page 150 is not in the layer.
    pub stopped_after: Option<usize>,
    /// **The recognised words, per page, ready to be applied to the open
    /// session as one undoable edit.**
    ///
    ///
    /// `pdfcer_core::ocr::layer::add_ocr_layer` takes an immutable `&Document`
    /// and hands back a complete file, which made recognition the one
    /// capability in pdfcer that was not an *edit*. A shell holding an open
    /// session could only offer *"here is a different file, somewhere else"*.
    /// The operator's requirement is that it not: recognition lands in the
    /// open document and saves over it, like every other edit.
    ///
    ///
    /// # And it deletes the unsaved-edits refusal, which no guard could fix
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
    /// The recogniser that read these pages; its key is written into the
    /// layer's marker, so the file says which engine produced the text.
    pub engine: EngineId,
    /// The character dictionary the recogniser read through, for an engine
    /// that has one; the report names it.
    pub dictionary: Option<Dictionary>,
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
    /// **How many pages produced words**, across a multi-page run.
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
#[must_use]
pub fn raster_scale(dpi: f32) -> f32 {
    crate::units::scale_from_dpi(f64::from(dpi)) as f32
}

/// The DPI to rasterize a page of `width_pt` × `height_pt` at.
#[must_use]
pub fn fitted_dpi(width_pt: f64, height_pt: f64) -> f32 {
    let area_in_sq_inches =
        crate::units::inches_from_points(width_pt) * crate::units::inches_from_points(height_pt);
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
pub fn resolve_models(
    engine: EngineId,
    exe_dir: Option<&Path>,
    user_data: Option<&Path>,
) -> Result<models::ModelSource, models::ModelsNotFound> {
    //
    // The plain `resolve_model_dir` asks only `is_dir()`. So an **empty**
    // `models/ocrs` beside the executable RESOLVES — and, worse, it wins the
    // search order, so an operator's own good copy further down is never
    // reached. The failure then surfaces later and in the wrong vocabulary: the
    // engine reports a missing model file after this shell has already told
    // them the models were found.
    models::resolve_model_dir_with(
        engine.model_dir(),
        None,
        exe_dir,
        user_data,
        engine.model_files(),
    )
}

/// The directory the running executable is in, if it can be determined.
#[must_use]
pub fn exe_dir() -> Option<PathBuf> {
    std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(Path::to_path_buf))
}

/// Everything a recognition needs, assembled on the UI thread and moved whole
/// onto the worker.
#[derive(Clone)]
pub struct Request {
    /// The session to read. Only its **base document** is used — see the
    /// module header on why, and [`Refusal::UnsavedEdits`] for what guarantees
    /// that base is what the operator is looking at.
    ///
    pub session: Arc<EditSession>,
    /// **The pages to recognise, zero-based, in order.**
    ///
    /// # Why this is a list
    ///
    /// One recognition covers a set of pages the operator chooses, not the
    /// current page. A one-page-at-a-time recogniser is refused: the documents
    /// this is used on are multi-page sheets, and driving it page by page is
    /// the whole cost of the capability.
    ///
    /// It was a `usize`. Nothing in `pdfcer-core` required that — the engine's
    /// own `add_ocr_layer` takes one page at a time, but its output is a
    /// complete PDF that can be fed straight back in, so a caller can chain
    /// them. **Measured before this was built**, because the audit flagged it
    /// UNVERIFIED and a wrong answer would have corrupted a file: two
    /// successive in-place recognitions of one page produce a document that
    /// round-trips byte-identical and extracts both layers. Revisions chain.
    ///
    /// **And the same measurement found a hazard**: a second pass over a
    /// page that already has a layer **adds a second one** — 427 codes became
    /// 854 — rather than replacing it. It does not affect a run that visits
    /// each page once, which is every run this shell issues, but it is why
    /// [`Self::skip_pages_with_text`] exists and defaults to on.
    ///
    /// Empty is not a valid request and the dialog will not build one; if one
    /// arrives, the job reports [`Refusal::NothingRecognised`] rather than
    /// succeeding at nothing.
    pub pages: Vec<usize>,
    /// **Leave alone any page that already has real text on it.**
    ///
    /// The default, and the safe one. It is OCRmyPDF's `--skip-text`, which is
    /// that tool's default for the same reason: recognising a page that is
    /// already text adds an invisible duplicate of text the file already has,
    /// which doubles every search hit and every copy.
    ///
    /// "Real text" means text the page draws **visibly**. A page that already
    /// carries an invisible OCR layer counts as having text, so re-running over
    /// a recognised document is the no-op an operator would expect rather than
    /// a doubling.
    pub skip_pages_with_text: bool,
    /// **The operator's extraction settings**, carried rather than defaulted.
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
    /// The recogniser to run; one [`available`] reports.
    pub engine: EngineId,
    /// The directory holding `engine`'s model files, from [`resolve_models`]
    /// for the same engine.
    pub model_dir: PathBuf,
}

/// The worker body. Runs on the spawned thread; touches no GUI type.
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

    let recogniser = Recogniser::load(request.engine, &request.model_dir)?;
    debug_assert_eq!(
        recogniser.reports_confidence(),
        request.engine.reports_confidence(),
        "the dialog words its disclosure from EngineId; the page is stamped from the engine"
    );

    let mut pages = Vec::new();
    let mut total_words = 0usize;
    let mut pages_skipped = 0usize;
    let mut dpi = 0.0f32;
    // Counted separately from `pages_skipped`, and only so that a run which
    // produced nothing can say WHY. See the refusal below.
    let mut already_had_text = 0usize;

    let of = request.pages.len();
    for (attempted, &page_index) in request.pages.iter().enumerate() {
        // **CHECKED BETWEEN PAGES, NEVER INSIDE ONE**, and that is what
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
                // Sent BEFORE the page is pushed, so the count the operator
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
        // **WHICH nothing — the two empty results are not the same.**
        //
        // A CAD sheet whose every page already has text selects nothing, and
        // reporting that as `NothingRecognised` reads as *"the recogniser
        // could not read your document"* when it never looked. The two are
        // different states and the remedy for each is
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
        engine: request.engine,
        dictionary: recogniser.dictionary(),
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
    chars: usize,
    /// What it was rasterized at.
    dpi: f32,
}

/// One page: rasterize it, read it, and put the words in page space.
///
fn recognise_one(
    request: &Request,
    recogniser: &Recogniser,
    page_index: usize,
) -> Result<OnePage, Refusal> {
    // **THE SESSION'S VIEW, NOT ITS BASE AND NOT THE FILE.**
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

    // **The doubling guard, and the reason it is measured rather than
    // assumed.**
    //
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
    // The flip, and the ONLY place it happens. See the module header.
    //
    //
    // `pdfcer-render` honours `/Rotate`: `page_device_geometry` swaps the
    // raster's axes at 90° and 270°. The mapping BACK to page space did not,
    // so on an odd quarter turn every recognised word landed on the wrong axis
    // at the wrong scale.
    //
    // And the failure is invisible by construction, which is why it needed
    // reporting rather than noticing. The OCR layer is Table 106 mode 3 —
    // rendered but not shown — so a page whose every word is misplaced **looks
    // exactly like a page whose every word is right**. The only symptom is that
    // selecting or searching picks the wrong thing, and an operator meeting
    // that would reasonably blame the recogniser rather than the geometry.
    //
    // Not an edge case in the one population OCR exists for: scanner drivers
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

    // Counted from the words that were actually PLACED, not from what the
    // recogniser emitted — the two differ whenever a word is dropped on the way
    // into page space, and the number an operator watches must describe what
    // ended up in their document.
    let chars_recognised = placed.iter().map(|w| w.text.chars().count()).sum();

    Ok(OnePage {
        recognised: OcrPage {
            words: placed,
            // Asked of the loaded engine rather than assumed: `ocrs` scores
            // nothing, OCRcer scores every word.
            confidence_available: recogniser.reports_confidence(),
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

/// Whether this build carries a recogniser at all.
#[must_use]
pub const fn engine_compiled_in() -> bool {
    cfg!(any(feature = "ocrs", feature = "ocrcer"))
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

    /// **The measured optimum is reproduced for the sheet it was measured
    /// on.**
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

    /// **An A0 sheet is reduced, and the reduction lands near the target.**
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

    /// **A saturated colour is not mid-grey, which a flat average would
    /// make it.**
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

    /// **`ocrs` reports no confidence, and the shell says so.** If this
    /// became `true` the dialog would drop its "nothing here has been scored"
    /// statement and a page of unscored guesses would present as checked.
    #[test]
    #[cfg(feature = "ocrs")]
    fn ocrs_scores_nothing() {
        assert!(!EngineId::Ocrs.reports_confidence());
    }

    /// The two absences are two different refusals.
    #[test]
    fn a_missing_engine_and_missing_models_are_distinct_refusals() {
        assert_ne!(Refusal::EngineAbsent, Refusal::ModelsMissing(Vec::new()));
    }

    /// The model directory name is the engine's own, not a second spelling.
    #[test]
    fn the_model_directory_is_the_engines_own_name() {
        assert_eq!(EngineId::Ocrs.model_dir(), "ocrs");
        assert_eq!(EngineId::Ocrcer.model_dir(), OCRCER_MODEL_DIR);
    }

    /// Nothing is resolved from a directory that does not exist, and every
    /// place that was looked in comes back.
    #[test]
    fn a_failed_resolution_reports_everywhere_it_looked() {
        // temp-path-exempt: never created -- the assertion is that resolving
        // against a directory that is not there fails.
        let nowhere = std::env::temp_dir().join("pdfcer-no-models-here-4c1a");
        let err =
            resolve_models(EngineId::Ocrs, Some(&nowhere), None).expect_err("nothing is there");
        assert_eq!(err.engine, EngineId::Ocrs.model_dir());
        assert_eq!(err.searched.len(), 1);
        assert!(err.to_string().contains("ocrs"));
    }

    /// **An EMPTY `models/ocrs` does not resolve, and so cannot shadow a
    /// good copy further down the search order.**
    #[test]
    #[cfg(feature = "ocrs")]
    fn an_empty_model_directory_is_rejected_but_a_filled_one_resolves() {
        let root =
            std::env::temp_dir().join(format!("pdfcer-empty-models-9f3b-{}", std::process::id()));
        let dir = root.join("models").join(EngineId::Ocrs.model_dir());
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&dir).expect("temp dir");

        // Empty: must be refused, or it shadows.
        let err = resolve_models(EngineId::Ocrs, Some(&root), None)
            .expect_err("an empty models directory must NOT resolve, or it shadows a good one");
        assert_eq!(err.engine, EngineId::Ocrs.model_dir());
        assert!(
            !err.searched.is_empty(),
            "the directory must be REPORTED as searched, so the message names a place the operator can go and look"
        );

        // Filled: must be accepted — otherwise the assertion above proves
        // nothing about emptiness.
        for f in EngineId::Ocrs.model_files() {
            std::fs::write(dir.join(f), b"not a real model, but a real file").expect("write");
        }
        assert!(
            resolve_models(EngineId::Ocrs, Some(&root), None).is_ok(),
            "a directory containing both model files must resolve"
        );

        let _ = std::fs::remove_dir_all(&root);
    }
}
