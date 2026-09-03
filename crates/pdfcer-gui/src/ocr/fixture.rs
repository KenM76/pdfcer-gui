//! # `ocr::fixture` — building the image-only PDF this project did not have
//!
//! Generates `fixtures/synthetic-image-only.pdf`, and runs the whole OCR
//! pipeline against it. Both are `#[ignore]`d: the first writes into the
//! repository, and the second costs several seconds and needs the model
//! weights on disk. Run them by name, exactly as the RON regeneration is run:
//!
//! ```text
//! cargo test -p pdfcer-gui --lib write_synthetic_image_only -- --ignored
//! cargo test -p pdfcer-gui --lib recognises_the_synthetic_page -- --ignored --nocapture
//! ```
//!
//! ## ★★ WHAT THIS FIXTURE IS, AND — MORE IMPORTANTLY — WHAT IT IS NOT
//!
//! **It is not a scan.** It is a page rendered from vector text and then thrown
//! away as pixels. Read the name as the whole caveat: *synthetic*, *image-only*.
//!
//! ### Why it exists anyway
//!
//! Because the alternative was verifying nothing. `HANDOFF.md` and
//! `FEATURES.md` both record the engine's own warning — *"its only test
//! documents are vector PDFs that already contain text"* — and that is still
//! exactly true of this repository: `D:\Dev\temp\pdfcer\` holds thirty PDFs and
//! `fixtures/` held one, and **every single one has extractable text on it.**
//! Measured, not assumed: a scan of both directories for text-showing operators
//! (`Tj`, `TJ`, `'`, `"`) inside every inflatable stream found them in all
//! thirty-one.
//!
//! So before this file, **no part of the OCR chain could be exercised at all**
//! — not the detection of an image-only page, not the Find offer, not the
//! recogniser, not the sandwich, not the round trip.
//!
//! ### ✅ What a green result here DOES establish
//!
//! The **plumbing**, which is most of what the shell was asked to build:
//!
//! 1. a document with no text layer is *detected* as having none
//!    (`OpenDoc::page_has_extractable_text` answers `false`);
//! 2. Find therefore offers OCR on it rather than reporting an ordinary empty
//!    result;
//! 3. the models resolve, the recogniser loads and runs;
//! 4. words come back positioned, survive the y-flip into page space, and reach
//!    `pdfcer_core::ocr::layer::add_ocr_layer`;
//! 5. the invisible layer is written at text rendering mode **3**;
//! 6. and the result is a document whose text pdfcer can extract — which is the
//!    property the whole feature exists to produce.
//!
//! ### ❌ What it does NOT establish, and must never be read as establishing
//!
//! **Recognition quality on a real scan.** A render-to-raster has no scanner
//! noise, no skew, no JPEG ringing, no bleed-through from the reverse side, no
//! uneven platen lighting, no dust, no fold shadow and no halftone screen. Those
//! are precisely the conditions that make OCR hard, and a fixture without them
//! **flatters the recogniser**.
//!
//! `HANDOFF.md` §10 carries the lesson in almost these words, from a different
//! feature: the ink-simplification fixture disturbed its samples *along* the arc
//! they lay on, so it "only re-spaced samples along a path whose shape never
//! changed", reported suspiciously good numbers, and was measuring something
//! other than what it claimed. *"A measurement that moves in the right
//! direction is not evidence that it measures the right thing."*
//!
//! This fixture is in that category by construction and says so rather than
//! being caught at it later. **Quality on real scanned material remains
//! unproven**, which is the engine's own position and is unchanged by a shell
//! having shipped a surface for it.
//!
//! ## What the page says, and why those words
//!
//! Fourteen lines of drawing-office prose at 11 pt -- general notes, a drawing
//! number, a revision. [`LINES`] carries the full argument for why it is a
//! **page** and not a caption, and it is worth reading: the first version of
//! this fixture was two words on a blank card, and it uncovered a real and
//! previously unrecorded failure mode in `ocrs` by being unrepresentative.
//!
//! ## How it is built
//!
//! By hand, in five steps, with no dependency beyond what already ships:
//!
//! 1. a tiny one-page PDF with real text is written as literal PDF syntax
//!    ([`source_pdf`]);
//! 2. `pdfcer_render` rasterizes it at [`FIXTURE_DPI`];
//! 3. the RGBA pixmap becomes 8-bit greyscale through [`super::greyscale`] —
//!    the same function the OCR path itself uses, so the fixture and the
//!    feature agree about what a grey pixel is;
//! 4. the greyscale is `/FlateDecode`d into a `/DeviceGray` image XObject;
//! 5. and a second one-page PDF is written whose entire content stream is
//!    `q W 0 0 H 0 0 cm /Im0 Do Q` — **one image operator and not one text
//!    operator anywhere in the file.**
//!
//! Step 5 is what makes the result genuinely image-only rather than merely
//! image-heavy, and [`tests::the_fixture_contains_no_text_operator_at_all`]
//! asserts it against the emitted bytes rather than trusting the construction.

// ★ REDUNDANT TO THE COMPILER, LOAD-BEARING TO A GATE — do not tidy away.
//
// `crate::ocr` already declares this module `#[cfg(test)]`, so the attribute
// below changes nothing about what is compiled. What it changes is what
// `tools/gates/check-ui-strings.sh` reads: its exclusion 2 stops scanning a
// file at the **first `#[cfg(test)]` at column 0**, on the reasoning that
// everything after it is test-only and is read by whoever is staring at a
// failing test rather than by an operator.
//
// The gate cannot see the parent's declaration — it scans one file at a time —
// so without this line it reads this file's PDF SYNTAX as operator-visible
// copy: `<< /Type /Catalog /Pages 2 0 R >>`, `stream`, `xref\n0 6`,
// `/BaseFont /Helvetica`. Thirty-two of them, every one a false positive, and
// the alternative was thirty-two `// ui-text-exempt:` comments each saying the
// same thing about a different fragment of a file format.
//
// Stated at this length because a redundant attribute is exactly the kind of
// thing a later reader deletes as noise, and the symptom would be a gate that
// fails on a file containing no operator strings at all.
#[cfg(test)]
mod scanned_as_test_only {}

use std::path::PathBuf;

use flate2::Compression;
use flate2::write::ZlibEncoder;

/// The fixture's page size, in points -- half of US Letter.
///
/// 306 x 396 pt. At [`FIXTURE_DPI`] that is an 850 x 1100 raster, which
/// compresses to tens of kilobytes and is a reasonable thing to keep in a
/// repository. An A1 sheet at 300 DPI, for comparison, is 70 megapixels.
const PAGE_W: f64 = 306.0;
/// See [`PAGE_W`].
const PAGE_H: f64 = 396.0;

/// The resolution the source page is rasterized at to make the fixture image.
///
/// 200, which is not the resolution recognition will run at, and the
/// difference is deliberate: the
/// fixture should not be rendered at exactly the resolution it will later be
/// recognised at, or the recogniser would be reading back a raster it could
/// have been handed unresampled. A scan is never at the resolution the reader
/// chooses either.
const FIXTURE_DPI: f32 = 200.0;

/// ★★ **The page's text, and why it is a PAGE rather than a caption.**
///
/// The first version of this fixture was two words in 28 pt on an otherwise
/// blank card, on the reasoning that a legible fixture is one that fails only
/// for real reasons. **It failed for a real reason, and the reason is worth the
/// paragraphs below**, because it is a fact about the engine pdfcer ships that
/// nothing in either repository knew.
///
/// ### What happened
///
/// `ocrs`'s detection model produced a *perfect* probability map -- four clean
/// blobs, exactly over the four words. Measured, not assumed: the map was
/// dumped and its connected components counted by hand, and there were four, at
/// the right places and the right sizes. And `ocrs::detect_words` returned
/// **three** rectangles, the first of which was the entire page.
///
/// The cause is a threshold, and it is in the open.
/// `TextDetectorParams::default()` sets `text_threshold: 0.2`, under the
/// upstream comment *"Ideally the threshold would be 0.5 as a neutral value."*
/// On this fixture the model's output over blank paper measured **0.148 to
/// 0.208** -- straddling that threshold. So the background itself binarised as
/// text in patches, the patches connected, and one component swallowed the
/// page. The recogniser was then handed the whole sheet squeezed into a
/// 127 x 64 line crop, and returned `"SE"`, `"1"`, `"P"`.
///
/// ### Why the FIXTURE changed and not the threshold
///
/// Raising `text_threshold` would have been one line, and would have been
/// **tuning the tool until the test passed** -- `HANDOFF.md` §10's warning
/// about a fixture that flatters what it measures, run in reverse. The
/// threshold is upstream's, chosen empirically against upstream's training
/// distribution, and this project has no evidence on which to overrule it.
///
/// What was actually wrong was the fixture's *representativeness*. `ocrs` is
/// trained on HierText -- photographs and dense document pages -- and a sheet
/// that is 96 % blank paper is neither. **A page of text is.** So the fixture
/// became one: fourteen lines at a realistic size and spacing, which is both
/// what the feature will meet and what the model was trained against.
///
/// ### ★ The finding stands regardless of this fixture
///
/// **`ocrs` at its default threshold can fail catastrophically on a sparse,
/// clean page** -- not degrade, fail: one whole-page "word" and three
/// characters of output. A scanned drawing with a small title block on a large
/// empty sheet is exactly that shape, and it is the shape this project's own
/// documents come in. It is recorded here rather than as a comment on a passing
/// test, and it is in the report to the operator.
///
/// ### Why these words
///
/// Drawing-office vocabulary, so an assertion can be about *content* -- a check
/// that asserted "some words came back" would pass against a recogniser
/// returning noise. Ordinary sentence case at 11 pt rather than large capitals:
/// the point is to look like a page, and the earlier version's 28 pt capitals
/// were part of what made it unrepresentative.
pub(crate) const LINES: [&str; 14] = [
    "GENERAL NOTES",
    "1. All dimensions are in millimetres unless noted",
    "otherwise. Do not scale from this drawing.",
    "2. Weld preparation to ISO 9692-1. Fillet welds are",
    "6 mm leg length unless a size is called out.",
    "3. Material: flange plate in S355J2+N, 20 mm thick.",
    "Plate flatness to EN 10029 class N.",
    "4. Holes 22 mm diameter drilled, not punched, on a",
    "PCD of 340 mm as shown on section A-A.",
    "5. Remove all burrs and sharp edges before painting.",
    "6. Surface preparation Sa 2.5 to ISO 8501-1.",
    "DRAWING NUMBER 41177",
    "REVISION C",
    "SHEET 1 OF 1",
];

/// The words the end-to-end test asserts came back.
///
/// A subset of [`LINES`], and deliberately the ones a reader most often needs
/// Find to reach on a real drawing -- a drawing number and a revision -- two of
/// which carry digits, exercising a different part of the model's alphabet from
/// the prose.
pub(crate) const MUST_RECOGNISE: [&str; 3] = ["DRAWING", "41177", "REVISION"];

/// Where the generated fixture lives, relative to the workspace root.
pub(crate) const FIXTURE_NAME: &str = "synthetic-image-only.pdf";

/// The multi-page fixture, for the checks about a run **in progress**.
pub(crate) const MULTIPAGE_NAME: &str = "synthetic-image-only-8pages.pdf";

/// How many sheets [`MULTIPAGE_NAME`] carries.
///
/// ★ **Eight, and the number was measured rather than chosen.** One page of
/// this fixture recognises in roughly a second in a release build, and a page
/// of the operator's own scanned parts manual measured **2.6 s** through
/// `pdfcer ocr`. Eight pages is therefore a run of eight to twenty seconds:
///
/// * long enough that a driven check can watch `attempted` climb, press Stop
///   with pages still to go, and have the result be unambiguous — a Stop that
///   lands on the last page is indistinguishable from a run that finished;
/// * short enough that three driven checks over it cost under a minute, which
///   is what keeps them in the ordinary sweep rather than in a "slow" tier
///   nobody runs.
///
/// It is also **not** a round number by accident: it matches the eight pages
/// extracted from the operator's manual for the real-material run, so the two
/// reports are read side by side without arithmetic.
pub(crate) const MULTIPAGE_PAGES: usize = 8;

/// The workspace root, from this crate's manifest directory.
pub(crate) fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
}

/// The generated fixture's path.
pub(crate) fn fixture_path() -> PathBuf {
    workspace_root().join("fixtures").join(FIXTURE_NAME)
}

/// The multi-page fixture's path.
pub(crate) fn multipage_path() -> PathBuf {
    workspace_root().join("fixtures").join(MULTIPAGE_NAME)
}

/// A minimal one-page PDF carrying [`LINES`] as real text.
///
/// The *source* for the fixture, never the fixture itself — it is thrown away
/// as pixels in step 2. Written as literal syntax rather than authored through
/// `EditSession` because every byte of it needs to be inspectable: this is the
/// thing whose text must survive a round trip through a raster and a
/// recogniser, and a document assembled by the same engine that will later be
/// asked to read it would make the test partly self-referential.
///
/// Standard-14 Helvetica, so nothing is embedded and the file stays under two
/// kilobytes. 11 pt on a 396 pt page with 24 pt leading is ordinary document
/// type. See [`LINES`] for why the fixture is a page of text rather than a
/// caption, and what the first version of it discovered by not being one.
///
/// `TL`/`T*` rather than a `Td` per line: one text object with a set leading is
/// how a real producer writes a paragraph, and a fixture whose content stream is
/// shaped like nothing any producer emits is a fixture testing a shape nobody
/// meets.
fn source_pdf() -> Vec<u8> {
    let mut content = String::from(
        "BT /F1 11 Tf 24 TL 36 360 Td
",
    );
    for line in LINES {
        content.push_str(&format!(
            "({line}) Tj T*
"
        ));
    }
    content.push_str(
        "ET
",
    );
    let objects: Vec<(u32, Vec<u8>)> = vec![
        (1, b"<< /Type /Catalog /Pages 2 0 R >>".to_vec()),
        (2, b"<< /Type /Pages /Kids [3 0 R] /Count 1 >>".to_vec()),
        (
            3,
            format!(
                "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 {PAGE_W} {PAGE_H}] \
                 /Resources << /Font << /F1 5 0 R >> >> /Contents 4 0 R >>"
            )
            .into_bytes(),
        ),
        (4, stream_object(b"", content.as_bytes())),
        (
            5,
            b"<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>".to_vec(),
        ),
    ];
    assemble(&objects, 1)
}

/// A `<< dict >> stream … endstream` body with `/Length` filled in.
///
/// `extra` is spliced into the dictionary before `/Length`, which is how the
/// image object gets its `/Filter`, `/Width`, `/Height` and colour space
/// without a second assembler.
fn stream_object(extra: &[u8], data: &[u8]) -> Vec<u8> {
    let mut out = Vec::new();
    out.extend_from_slice(b"<< ");
    out.extend_from_slice(extra);
    out.extend_from_slice(format!(" /Length {} >>\nstream\n", data.len()).as_bytes());
    out.extend_from_slice(data);
    out.extend_from_slice(b"\nendstream");
    out
}

/// Serialize numbered objects into a complete PDF with a classic xref table.
///
/// A cross-reference **table** rather than a stream, and a `%PDF-1.4` header,
/// on purpose: both are the oldest and most widely-agreed forms, so a fixture
/// that failed to open would be a defect in whatever opened it rather than an
/// argument about which of two encodings was meant. The offsets are counted
/// from the emitted bytes as they are written — never computed in advance —
/// because an xref whose offsets are one byte out is a file that opens on some
/// readers and not others, which is the worst kind of broken fixture.
fn assemble(objects: &[(u32, Vec<u8>)], root: u32) -> Vec<u8> {
    let mut out: Vec<u8> = b"%PDF-1.4\n%\xE2\xE3\xCF\xD3\n".to_vec();
    let mut offsets: Vec<(u32, usize)> = Vec::new();
    for (num, body) in objects {
        offsets.push((*num, out.len()));
        out.extend_from_slice(format!("{num} 0 obj\n").as_bytes());
        out.extend_from_slice(body);
        out.extend_from_slice(b"\nendobj\n");
    }
    let xref_at = out.len();
    let count = objects.len() as u32 + 1;
    out.extend_from_slice(format!("xref\n0 {count}\n0000000000 65535 f \n").as_bytes());
    for (_, at) in &offsets {
        out.extend_from_slice(format!("{at:010} 00000 n \n").as_bytes());
    }
    out.extend_from_slice(
        format!("trailer\n<< /Size {count} /Root {root} 0 R >>\nstartxref\n{xref_at}\n%%EOF\n")
            .as_bytes(),
    );
    out
}

/// Build the image-only PDF: no text operator anywhere in it.
///
/// `grey` is one byte per pixel, row-major, top-down — the layout
/// [`super::greyscale`] produces and the layout `/DeviceGray` at
/// `/BitsPerComponent 8` expects, so no transposition happens here. The image
/// XObject is defined on the unit square (§8.9.4), so the whole of placement is
/// the one `cm` matrix that scales it to the page box.
fn image_only_pdf(grey: &[u8], width: u32, height: u32) -> Vec<u8> {
    image_only_pdf_pages(grey, width, height, 1)
}

/// The same document with `pages` identical sheets, all sharing **one** image.
///
/// # ★★★ Why a multi-page image-only fixture has to exist
///
/// The one-page fixture is the right subject for *"did the recogniser read this
/// page"*. It is the wrong subject for everything the operator asked for on
/// 2026-09-01 — *"pages done, words/characters detected … a cancel and stop
/// button"* — because **every one of those is a statement about a run in
/// progress**, and a one-page run has no observable middle. It is started and
/// then it is finished; a Stop pressed during it can only ever race the single
/// page, and a progress line that draws once carries no evidence that it
/// advances.
///
/// So this exists to give the driven checks a run with a **middle**:
/// [`MULTIPAGE_PAGES`] sheets, recognised one after another, long enough that a
/// harness can see `attempted` climb and can press Stop with pages still to go.
///
/// # Why the pages are identical, which looks like a shortcut and is not
///
/// Each page is the same rendered notes sheet, and every page dictionary points
/// at the **same** image XObject. Three consequences, all wanted:
///
/// * the file is ~40 kB rather than ~300 kB, because the pixels are stored once
///   — a fixture that has to be committed should not be a third of a megabyte;
/// * every page recognises to the **same word count**, so a check can assert
///   the totals are consistent with the pages attempted rather than having to
///   accept any number at all;
/// * a page that is skipped or dropped is visible as an arithmetic hole rather
///   than as a plausible smaller number.
///
/// ★ The pages sharing an XObject is *also* representative: it is what a real
/// scanner-produced PDF does not do, but what every stamp, logo and repeated
/// figure in a real document does, and a recogniser that assumed one image per
/// page would break on both.
///
/// # What it still does not establish
///
/// The same caveat the one-page fixture carries, and it is not weakened by
/// there being more of them: this is a **rendered** page, not a scan. No
/// scanner noise, no skew, no JPEG ringing, no uneven lighting. It establishes
/// the plumbing of a multi-page run. It establishes nothing about recognition
/// quality, and the driven checks say so in their own reports.
fn image_only_pdf_pages(grey: &[u8], width: u32, height: u32, pages: usize) -> Vec<u8> {
    assert!(pages >= 1, "a document needs at least one page");
    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::best());
    std::io::Write::write_all(&mut encoder, grey).expect("in-memory write cannot fail");
    let compressed = encoder.finish().expect("in-memory flush cannot fail");

    // ★ The entire content stream. One `Do`, and nothing else — no `BT`, no
    // `Tf`, no `Tj`. That is what makes this fixture image-ONLY rather than
    // image-heavy, and `tests::the_fixture_contains_no_text_operator_at_all`
    // asserts it against these bytes after the fact rather than trusting this
    // comment.
    let content = format!("q {PAGE_W} 0 0 {PAGE_H} 0 0 cm /Im0 Do Q\n");

    // Object numbering, and it must stay contiguous and ascending because
    // `assemble` writes a classic xref table that assumes exactly that.
    //
    //   1                  catalog
    //   2                  page tree
    //   3 ..= 2 + pages    the page dictionaries
    //   3 + pages          the shared content stream
    //   4 + pages          the shared image
    let first_page = 3u32;
    let content_obj = first_page + pages as u32;
    let image_obj = content_obj + 1;

    let kids: String = (0..pages as u32)
        .map(|i| format!("{} 0 R ", first_page + i))
        .collect();

    let mut objects: Vec<(u32, Vec<u8>)> = vec![
        (1, b"<< /Type /Catalog /Pages 2 0 R >>".to_vec()),
        (
            2,
            format!(
                "<< /Type /Pages /Kids [{}] /Count {pages} >>",
                kids.trim_end()
            )
            .into_bytes(),
        ),
    ];
    for i in 0..pages as u32 {
        objects.push((
            first_page + i,
            format!(
                "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 {PAGE_W} {PAGE_H}] \
                 /Resources << /XObject << /Im0 {image_obj} 0 R >> >> \
                 /Contents {content_obj} 0 R >>"
            )
            .into_bytes(),
        ));
    }
    objects.push((content_obj, stream_object(b"", content.as_bytes())));
    objects.push((
        image_obj,
        stream_object(
            format!(
                "/Type /XObject /Subtype /Image /Width {width} /Height {height} \
                 /ColorSpace /DeviceGray /BitsPerComponent 8 /Filter /FlateDecode"
            )
            .as_bytes(),
            &compressed,
        ),
    ));
    assemble(&objects, 1)
}

/// Render the source page and wrap it as an image-only document.
///
/// Returns the finished PDF bytes. Split from the test that writes them so that
/// the *generation* is reachable from an assertion without touching the
/// repository — `tests::the_fixture_contains_no_text_operator_at_all` runs on
/// every `cargo test` and needs the bytes, not the file.
pub(crate) fn build() -> Vec<u8> {
    let (grey, w, h) = raster();
    image_only_pdf(&grey, w, h)
}

/// The same, with [`MULTIPAGE_PAGES`] sheets — see [`image_only_pdf_pages`].
pub(crate) fn build_multipage() -> Vec<u8> {
    let (grey, w, h) = raster();
    image_only_pdf_pages(&grey, w, h, MULTIPAGE_PAGES)
}

/// Rasterize the source page once. Shared by both builders.
///
/// Split out when the multi-page fixture arrived, so that the two documents are
/// **the same pixels** by construction rather than by two call sites happening
/// to pass the same DPI. A one-page and an eight-page fixture that disagreed
/// about the raster would make their word counts incomparable, and comparing
/// them is half of what the multi-page checks do.
fn raster() -> (Vec<u8>, u32, u32) {
    let doc = pdfcer_core::document::Document::from_bytes(source_pdf())
        .expect("the hand-written source PDF must parse");
    let pages = pdfcer_core::page_tree::pages(&doc).expect("one page");
    let rendered = pdfcer_render::render_page(&doc, &pages[0], FIXTURE_DPI / 72.0)
        .expect("a page of Helvetica must rasterize");
    let (w, h) = (rendered.pixmap.width(), rendered.pixmap.height());
    let grey = super::greyscale(rendered.pixmap.data(), w, h);
    (grey, w, h)
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;

    /// ★ **Regenerate `fixtures/synthetic-image-only.pdf`.**
    ///
    /// `#[ignore]`d because it writes into the repository, exactly like
    /// `shell::ron::tests::rewrite_built_in_ron`. Run it when the page content
    /// or the raster parameters change:
    ///
    /// ```text
    /// cargo test -p pdfcer-gui --lib write_synthetic_image_only -- --ignored
    /// ```
    #[test]
    #[ignore = "writes into fixtures/; run deliberately"]
    fn write_synthetic_image_only() {
        let bytes = build();
        let path = fixture_path();
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, &bytes).unwrap();
        println!("wrote {} ({} bytes)", path.display(), bytes.len());
    }

    /// ★ **Regenerate `fixtures/synthetic-image-only-8pages.pdf`.**
    ///
    /// ```text
    /// cargo test -p pdfcer-gui --lib write_synthetic_image_only_multipage -- --ignored
    /// ```
    #[test]
    #[ignore = "writes into fixtures/; run deliberately"]
    fn write_synthetic_image_only_multipage() {
        let bytes = build_multipage();
        let path = multipage_path();
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, &bytes).unwrap();
        println!("wrote {} ({} bytes)", path.display(), bytes.len());
    }

    /// ★★★ **The multi-page fixture really has eight pages, and the engine
    /// agrees.**
    ///
    /// Pinned because the whole value of that fixture is the page COUNT, and
    /// the count is produced by hand-written object numbering with a classic
    /// xref table — the one part of this module where an off-by-one produces a
    /// file that still opens. A `/Count 8` over seven `/Kids` is a document
    /// most readers will show, and every driven check over it would then be
    /// asserting against a denominator that is a lie.
    ///
    /// Parsed by `pdfcer_core` rather than grepped, deliberately: the question
    /// is *what will the application see*, and the application sees whatever
    /// the page-tree walker sees.
    #[test]
    fn the_multipage_fixture_has_the_page_count_it_claims() {
        let bytes = build_multipage();
        let doc = pdfcer_core::document::Document::from_bytes(bytes)
            .expect("the generated multi-page fixture must parse");
        let pages = pdfcer_core::page_tree::pages(&doc).expect("its page tree must walk");
        assert_eq!(
            pages.len(),
            MULTIPAGE_PAGES,
            "the page tree must hand back exactly the pages the /Count claims"
        );
    }

    /// ★★ **Eight pages cost barely more than one, because the image is shared.**
    ///
    /// The property that makes committing this fixture reasonable. Asserted as
    /// a ratio rather than an absolute size so it survives a change to the
    /// raster DPI: if somebody later gives each page its own copy of the
    /// pixels, the eight-page file becomes ~8× the one-page file and this
    /// fails, which is the moment to notice — not at the next `git push`.
    #[test]
    fn the_multipage_fixture_shares_one_image_rather_than_copying_it() {
        let one = build().len();
        let eight = build_multipage().len();
        assert!(
            eight < one * 2,
            "eight pages sharing one image XObject must not approach eight times the size of \
             one: one page is {one} bytes and eight are {eight}, a ratio of {:.2}. A ratio near \
             8 means every page got its own copy of the pixels.",
            eight as f64 / one as f64
        );
        assert!(
            eight > one,
            "it must still be bigger — {eight} vs {one} — or the extra page dictionaries were \
             never written and the /Count is describing objects that do not exist"
        );
    }

    /// The multi-page fixture is image-only too.
    ///
    /// ★ Separate from [`the_fixture_contains_no_text_operator_at_all`] rather
    /// than folded into it. The two documents are built by two functions, and
    /// the assertion that matters — *any text on this page came from the
    /// recogniser* — has to hold of the one the checks actually drive. A shared
    /// test over only the one-page build would leave the eight-page build
    /// unasserted while looking like it covered both.
    #[test]
    fn the_multipage_fixture_contains_no_text_operator_either() {
        let bytes = build_multipage();
        let text = String::from_utf8_lossy(&bytes);
        // The image stream is binary and may contain anything, so the search is
        // for the text-OBJECT delimiters, which the content stream would carry
        // and which compressed pixel data has no reason to spell.
        let content = format!("q {PAGE_W} 0 0 {PAGE_H} 0 0 cm /Im0 Do Q");
        assert!(
            text.contains(&content),
            "the shared content stream must be exactly the one `Do`"
        );
        assert!(
            !text.contains(" BT\n") && !text.contains("\nBT "),
            "no text object may appear in the multi-page fixture"
        );
    }

    /// ★★ **The fixture contains no text-showing operator anywhere.**
    ///
    /// The property that makes it a valid test of OCR rather than a test of
    /// nothing, asserted against the **emitted bytes** rather than against the
    /// construction that produced them. Both streams are checked: the content
    /// stream is uncompressed and inspectable directly, and the image stream is
    /// binary and could in principle contain the byte pairs by accident — which
    /// is why the assertion is on the content stream's region specifically.
    ///
    /// Without this, a future change that "helpfully" kept a caption on the page
    /// would leave every OCR check passing for the wrong reason: the text would
    /// already be extractable, the offer would never appear, and the round trip
    /// would succeed without the recogniser contributing anything.
    #[test]
    fn the_fixture_contains_no_text_operator_at_all() {
        let bytes = build();
        let content = format!("q {PAGE_W} 0 0 {PAGE_H} 0 0 cm /Im0 Do Q");
        assert!(
            bytes
                .windows(content.len())
                .any(|w| w == content.as_bytes()),
            "the content stream is not what this module claims to emit"
        );
        // The whole content stream, isolated: from `stream\n` after object 4's
        // dictionary to the `endstream` that closes it.
        let marker = b"/Length ";
        let first_stream = bytes
            .windows(marker.len())
            .position(|w| w == marker)
            .expect("object 4 is a stream");
        let start = bytes[first_stream..]
            .windows(7)
            .position(|w| w == b"stream\n")
            .expect("stream keyword")
            + first_stream
            + 7;
        let end = bytes[start..]
            .windows(9)
            .position(|w| w == b"endstream")
            .expect("endstream")
            + start;
        let stream = &bytes[start..end];
        for op in [b"BT".as_slice(), b"Tj", b"TJ", b"Tf", b"ET"] {
            assert!(
                !stream.windows(op.len()).any(|w| w == op),
                "the page's content stream contains `{}` — this fixture would then have \
                 extractable text and would test nothing about OCR",
                String::from_utf8_lossy(op)
            );
        }
    }

    /// ★ **pdfcer extracts nothing from it**, which is the condition the Find
    /// offer keys on.
    ///
    /// The previous test asserts the *bytes*; this asserts what the **engine
    /// makes of them**, which is the thing `OpenDoc::page_has_extractable_text`
    /// actually asks. They are not the same claim: a content stream with no text
    /// operator could still carry text through an annotation appearance or a
    /// form XObject, and the extractor is what would know.
    #[test]
    fn the_engine_finds_no_text_on_the_fixture() {
        let doc = pdfcer_core::document::Document::from_bytes(build()).unwrap();
        let text = pdfcer_core::text_extract::extract_document_view(
            &doc.view(),
            &pdfcer_core::text_extract::ExtractOptions::default(),
        )
        .expect("the fixture must at least parse");
        assert!(
            text.plain_text().trim().is_empty(),
            "the fixture has extractable text on it: {:?}",
            text.plain_text()
        );
    }

    /// ★★ **The measurement behind [`super::super::TARGET_PIXELS`].**
    ///
    /// Recognition accuracy against DPI, on the two real documents this project
    /// has, using **each page's own vector text as ground truth**. That is what
    /// makes it an accuracy figure rather than an impression: a recognised token
    /// either appears in the text the page actually contains or it does not, and
    /// no judgement is involved.
    ///
    /// It is `#[ignore]`d because it is minutes of work and reads two files
    /// outside the repository, and it is kept because the constant it produced
    /// is otherwise a number with a table beside it that nobody can re-derive.
    /// `HANDOFF.md` §10's standing complaint is prose that drifts from the
    /// measurement it quotes; this is the measurement, runnable.
    ///
    /// **Run it in release.** In a debug build `rten` is roughly fifty times
    /// slower and a single A1 page takes minutes.
    ///
    /// ```text
    /// cargo test --release -p pdfcer-gui --lib real_page_detection -- --ignored --nocapture
    /// ```
    #[test]
    #[ignore = "minutes of work; reads documents outside the repository"]
    fn real_page_detection() {
        use pdfcer_core::ocr::OcrEngine as _;
        use std::collections::HashSet;
        let models = std::path::Path::new("D:/Dev/pdfcer/crates/pdfcer-core/assets/models/ocrs");
        let engine = pdfcer_core::ocr::engine_ocrs::OcrsEngine::from_model_dir(models).unwrap();
        for (name, path) in [
            ("SW41177", "D:/Dev/temp/pdfcer/SW41177.pdf"),
            (
                "a1-titleblock",
                "D:/Dev/pdfcer-gui/fixtures/a1-titleblock.pdf",
            ),
        ] {
            let doc = pdfcer_core::document::Document::load(std::path::Path::new(path)).unwrap();
            let pages = pdfcer_core::page_tree::pages(&doc).unwrap();
            // Ground truth: the page's own vector text.
            let truth = pdfcer_core::text_extract::extract_page_view(
                &doc.view(),
                &pages[0],
                0,
                &pdfcer_core::text_extract::ExtractOptions::default(),
            )
            .unwrap()
            .plain_text()
            .to_uppercase();
            let truth_tokens: HashSet<String> = truth
                .split(|c: char| !c.is_alphanumeric())
                .filter(|t| t.len() >= 3)
                .map(str::to_owned)
                .collect();
            println!(
                "== {name}: {} ground-truth tokens of 3+ chars",
                truth_tokens.len()
            );
            for dpi in [72.0f32, 100.0, 150.0, 200.0, 300.0] {
                let r = pdfcer_render::render_page(&doc, &pages[0], dpi / 72.0).unwrap();
                let (w, h) = (r.pixmap.width(), r.pixmap.height());
                let g = crate::ocr::greyscale(r.pixmap.data(), w, h);
                let t0 = std::time::Instant::now();
                let words = engine.recognize(w, h, &g).unwrap();
                let ms = t0.elapsed().as_millis();
                let got: Vec<String> = words.iter().map(|x| x.text.to_uppercase()).collect();
                let long: Vec<&String> = got.iter().filter(|t| t.len() >= 3).collect();
                let hits = long.iter().filter(|t| truth_tokens.contains(**t)).count();
                println!(
                    "   dpi={dpi:>5} {w}x{h} words={:>4} long={:>4} exact-in-truth={:>4} ({:>5.1}%) ms={ms}",
                    words.len(),
                    long.len(),
                    hits,
                    if long.is_empty() {
                        0.0
                    } else {
                        100.0 * hits as f64 / long.len() as f64
                    }
                );
            }
        }
    }

    /// The source page, by contrast, DOES have the two lines on it.
    ///
    /// ★ The control, and it is the load-bearing half of the pair: rule 4 of
    /// `tools/ui-verify`'s own checks — *never treat an absence as evidence
    /// unless you have shown the thing that would have produced it was
    /// working* — applies just as much to a unit test. Without this, an
    /// extractor that returned nothing for **every** document would satisfy the
    /// assertion above perfectly.
    #[test]
    fn the_source_page_does_have_the_text_the_fixture_throws_away() {
        let doc = pdfcer_core::document::Document::from_bytes(source_pdf()).unwrap();
        let text = pdfcer_core::text_extract::extract_document_view(
            &doc.view(),
            &pdfcer_core::text_extract::ExtractOptions::default(),
        )
        .unwrap()
        .plain_text();
        for line in LINES {
            assert!(text.contains(line), "source is missing {line:?}: {text:?}");
        }
    }

    /// ★★ **The whole OCR chain, end to end, against the fixture.**
    ///
    /// Recognise the image-only page, write the invisible layer, and read the
    /// words back out of the resulting document with the ordinary text
    /// extractor. That last step is what makes this a test of the *feature*
    /// rather than of the recogniser: it asserts the property an operator
    /// actually gets, which is that Find and copy start working.
    ///
    /// `#[ignore]`d for two reasons, both real: it takes seconds, and it needs
    /// the model weights on disk. It resolves them from the engine tree
    /// directly rather than from beside the test binary, because `cargo test`
    /// runs out of `target/debug/deps` where no packaging has put them.
    ///
    /// ★ **A green result here does not mean OCR works on scans.** See the
    /// module header; the fixture has none of the degradation that makes real
    /// recognition hard.
    ///
    /// ```text
    /// cargo test -p pdfcer-gui --lib recognises_the_synthetic_page -- --ignored --nocapture
    /// ```
    #[test]
    #[ignore = "several seconds, and needs the ocrs model weights on disk"]
    fn recognises_the_synthetic_page() {
        let models = PathBuf::from("D:/Dev/pdfcer/crates/pdfcer-core/assets/models/ocrs");
        assert!(
            models.is_dir(),
            "the model weights are not at {}; this test cannot run without them, and \
             reporting a pass would be reporting a run that never happened",
            models.display()
        );
        let bytes = build();
        let session = std::sync::Arc::new(pdfcer_core::edit::EditSession::new(
            pdfcer_core::document::Document::from_bytes(bytes.clone()).unwrap(),
        ));
        let started = std::time::Instant::now();
        let out = super::super::Job::spawn(super::super::Request {
            session,
            pages: vec![0],
            // ★ OFF, deliberately. The fixture's page is an image of words with
            // no text layer, so the guard would not fire — but pinning it off
            // states that what this measures is the recogniser rather than the
            // guard, and it keeps the test honest if the fixture ever grows a
            // caption.
            skip_pages_with_text: false,
            // Unread, because the guard above is off. The default rather than a
            // configured set, because there is no `Settings` on this thread and
            // nothing here depends on one.
            extract_options: pdfcer_core::text_extract::ExtractOptions::default(),
            model_dir: models,
        });
        let mut job = out;
        let recognised = loop {
            if let Some(answer) = job.poll() {
                // ★ The fixture asserts the ORDINARY ending. `Stopped` and
                // `Cancelled` are reachable only by pressing a button, and
                // nothing presses one here — so meeting either would mean the
                // control flag was set by something other than an operator,
                // which is worth failing loudly rather than unwrapping past.
                break match *answer {
                    crate::ocr::progress::Outcome::Complete(result) => {
                        *result.expect("recognition must not refuse on this fixture")
                    }
                    other => panic!("this fixture presses no button; got {other:?}"),
                };
            }
            std::thread::sleep(std::time::Duration::from_millis(50));
        };
        let elapsed = started.elapsed();

        println!(
            "recognised={} pages={} skipped={} dpi={:.0} ms={}",
            recognised.words_recognised,
            recognised.pages_written,
            recognised.pages_skipped,
            recognised.effective_dpi,
            elapsed.as_millis()
        );

        // ★★★ **Apply it the way the application does** — through
        // `EditSession::add_ocr_layer`, as an edit, not by writing a file.
        //
        // A second session over the same bytes rather than the one the worker
        // held: the worker's is behind an `Arc` the `Job` may still own, and
        // fighting that here would be testing `Arc` rather than recognition.
        // The bytes are identical, which is the only property this needs.
        let mut applying = pdfcer_core::edit::EditSession::new(
            pdfcer_core::document::Document::from_bytes(bytes).unwrap(),
        );
        let layers: Vec<pdfcer_core::edit::OcrPageLayer<'_>> = recognised
            .pages
            .iter()
            .map(|(index, page)| pdfcer_core::edit::OcrPageLayer {
                page_index: *index,
                recognised: page,
            })
            .collect();
        let reports = applying
            .add_ocr_layer(&layers, &pdfcer_core::ocr::layer::OcrLayerOptions::new())
            .expect("the layer must apply to the session");
        for report in &reports {
            println!(
                "written={} skipped={} substituted={} clamped={} confidence_available={}",
                report.words_written,
                report.words_skipped,
                report.words_substituted,
                report.words_scale_clamped,
                report.confidence_available,
            );
            for line in report.disclosures() {
                println!("  disclosure: {line}");
            }
        }

        // ★ The verdict is what the ORDINARY extractor reads back, not what the
        // recogniser claimed. A layer that was written into the wrong place, or
        // at a rendering mode a reader ignores, would satisfy every count above
        // and produce nothing here.
        //
        // ★★ And it reads the **session's own view**, which is a stronger
        // assertion than the old one made: the old test serialised to bytes and
        // re-parsed them, so it could not have caught a layer that reached the
        // file and not the live session. That is precisely the direction this
        // whole change moved in.
        let after_bytes = applying
            .to_incremental_bytes(&pdfcer_core::writer::SaveOptions::default())
            .expect("the session serialises")
            .0;
        let after = pdfcer_core::document::Document::from_bytes(after_bytes).unwrap();
        let text = pdfcer_core::text_extract::extract_document_view(
            &after.view(),
            &pdfcer_core::text_extract::ExtractOptions::default(),
        )
        .unwrap()
        .plain_text();
        println!("extracted after OCR: {text:?}");

        assert!(
            !text.trim().is_empty(),
            "the recognised document has no extractable text, so the layer did not land"
        );
        // Content, not a count. A recogniser returning noise would pass a
        // word-count assertion and fail this one.
        for word in MUST_RECOGNISE {
            assert!(
                text.to_uppercase().contains(word),
                "expected {word:?} in the recognised text, got {text:?}"
            );
        }
        assert!(
            recognised
                .pages
                .iter()
                .all(|(_, page)| !page.confidence_available),
            "this engine reports no confidence; a `true` here would make the dialog stop \
             disclosing that and present unscored guesses as checked"
        );
    }
}
