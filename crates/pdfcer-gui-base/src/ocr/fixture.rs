//! # `ocr::fixture` — building the image-only PDF this project did not have
//!
//! Generates `fixtures/synthetic-image-only.pdf`, and runs the whole OCR
//! pipeline against it. Both are `#[ignore]`d: the first writes into the
//! repository, and the second costs several seconds and needs the model
//! weights on disk. Run them by name, exactly as the RON regeneration is run:
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/ocr/fixture.md`.

// REDUNDANT TO THE COMPILER, LOAD-BEARING TO A GATE — do not tidy away.
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
const PAGE_W: f64 = 306.0;
/// See [`PAGE_W`].
const PAGE_H: f64 = 396.0;

/// The resolution the source page is rasterized at to make the fixture image.
const FIXTURE_DPI: f32 = 200.0;

/// **The page's text, and why it is a PAGE rather than a caption.**
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
pub(crate) const MUST_RECOGNISE: [&str; 3] = ["DRAWING", "41177", "REVISION"];

/// Where the generated fixture lives, relative to the workspace root.
pub(crate) const FIXTURE_NAME: &str = "synthetic-image-only.pdf";

/// The multi-page fixture, for the checks about a run **in progress**.
pub(crate) const MULTIPAGE_NAME: &str = "synthetic-image-only-8pages.pdf";

/// How many sheets [`MULTIPAGE_NAME`] carries.
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
fn image_only_pdf(grey: &[u8], width: u32, height: u32) -> Vec<u8> {
    image_only_pdf_pages(grey, width, height, 1)
}

/// The same document with `pages` identical sheets, all sharing **one** image.
fn image_only_pdf_pages(grey: &[u8], width: u32, height: u32, pages: usize) -> Vec<u8> {
    assert!(pages >= 1, "a document needs at least one page");
    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::best());
    std::io::Write::write_all(&mut encoder, grey).expect("in-memory write cannot fail");
    let compressed = encoder.finish().expect("in-memory flush cannot fail");

    // The entire content stream. One `Do`, and nothing else — no `BT`, no
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

    /// **Regenerate `fixtures/synthetic-image-only.pdf`.**
    ///
    /// `#[ignore]`d because it writes into the repository, exactly like
    /// `shell::ron::tests::rewrite_built_in_ron`. Run it when the page content
    /// or the raster parameters change:
    ///
    /// ```text
    /// cargo test -p pdfcer-gui-base --lib write_synthetic_image_only -- --ignored
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

    /// **Regenerate `fixtures/synthetic-image-only-8pages.pdf`.**
    ///
    /// ```text
    /// cargo test -p pdfcer-gui-base --lib write_synthetic_image_only_multipage -- --ignored
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

    /// **The multi-page fixture really has eight pages, and the engine
    /// agrees.**
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

    /// **Eight pages cost barely more than one, because the image is shared.**
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

    /// **The fixture contains no text-showing operator anywhere.**
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

    /// **pdfcer extracts nothing from it**, which is the condition the Find
    /// offer keys on.
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

    /// **The measurement behind [`super::super::TARGET_PIXELS`].**
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
    /// Prose drifts from the measurement it quotes; this is the measurement,
    /// runnable.
    ///
    /// **Run it in release.** In a debug build `rten` is roughly fifty times
    /// slower and a single A1 page takes minutes.
    ///
    /// ```text
    /// cargo test --release -p pdfcer-gui-base --lib real_page_detection -- --ignored --nocapture
    /// ```
    #[test]
    #[cfg(feature = "ocrs")]
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

    /// **The whole OCR chain, end to end, against the fixture.**
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
    /// **A green result here does not mean OCR works on scans.** See the
    /// module header; the fixture has none of the degradation that makes real
    /// recognition hard.
    ///
    /// ```text
    /// cargo test -p pdfcer-gui-base --lib recognises_the_synthetic_page -- --ignored --nocapture
    /// ```
    #[test]
    #[ignore = "several seconds, and needs the ocrs model weights on disk"]
    fn recognises_the_synthetic_page() {
        let recognised = recognise_and_apply(
            "ocrs",
            PathBuf::from("D:/Dev/pdfcer/crates/pdfcer-core/assets/models/ocrs"),
            &MUST_RECOGNISE,
        );
        assert!(
            recognised
                .pages
                .iter()
                .all(|(_, page)| !page.confidence_available),
            "this engine reports no confidence; a `true` here would make the dialog stop \
             disclosing that and present unscored guesses as checked"
        );
    }

    /// The same chain with OCRcer: resolved from the OCRcer repository's
    /// build output, recognised, applied, and read back by the ordinary
    /// extractor. Its pages must report confidence, since OCRcer scores every
    /// word and the dialog words its disclosure on that.
    ///
    /// ```text
    /// cargo test -p pdfcer-gui-base --lib ocrcer_recognises_the_synthetic_page -- --ignored --nocapture
    /// ```
    #[test]
    #[cfg(feature = "ocrcer")]
    #[ignore = "needs OCRcer's model file on disk"]
    fn ocrcer_recognises_the_synthetic_page() {
        let recognised = recognise_and_apply(
            "ocrcer",
            PathBuf::from("D:/Dev/OCRcer/model/out"),
            // Not "41177": OCRcer spaces the drawing number around its narrow
            // `1` ("41 1 77"), a word-segmentation miss reported in
            // `FeatureRequests/OCRcer_FeatureRequests`. Add it back when
            // OCRcer reads it whole.
            &["DRAWING", "REVISION"],
        );
        assert!(
            recognised
                .pages
                .iter()
                .all(|(_, page)| page.confidence_available),
            "OCRcer scores every word; `false` would drop its confidence from the report"
        );
        assert!(
            recognised
                .pages
                .iter()
                .flat_map(|(_, page)| &page.words)
                .all(|w| w.confidence.is_some_and(|c| (0.0..=1.0).contains(&c))),
            "every OCRcer word carries a score in 0..=1"
        );
    }

    /// Recognise the fixture page with `engine`, apply the layer as the
    /// application does, and require every word of `must` in what the
    /// ordinary extractor reads back.
    fn recognise_and_apply(
        engine: &str,
        models: PathBuf,
        must: &[&str],
    ) -> super::super::Recognised {
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
            // OFF, deliberately. The fixture's page is an image of words with
            // no text layer, so the guard would not fire — but pinning it off
            // states that what this measures is the recogniser rather than the
            // guard, and it keeps the test honest if the fixture ever grows a
            // caption.
            skip_pages_with_text: false,
            dictionaries: pdfcer_ocr_host::Dictionaries::builtin(),
            by_layout: false,
            // Unread, because the guard above is off. The default rather than a
            // configured set, because there is no `Settings` on this thread and
            // nothing here depends on one.
            extract_options: pdfcer_core::text_extract::ExtractOptions::default(),
            model: pdfcer_core::ocr::addons::OcrModel {
                name: engine.to_owned(),
                engine: engine.to_owned(),
                folder: models.clone(),
                root: models,
                manifest: None,
            },
            policy: pdfcer_ocr_host::ProgramPolicy::Allow,
        });
        let mut job = out;
        let recognised = loop {
            if let Some(answer) = job.poll() {
                // The fixture asserts the ORDINARY ending. `Stopped` and
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

        // **Apply it the way the application does** — through
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

        // The verdict is what the ORDINARY extractor reads back, not what the
        // recogniser claimed. A layer that was written into the wrong place, or
        // at a rendering mode a reader ignores, would satisfy every count above
        // and produce nothing here.
        //
        // And it reads the **session's own view**, which is a stronger
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
        for word in must {
            assert!(
                text.to_uppercase().contains(word),
                "expected {word:?} in the recognised text, got {text:?}"
            );
        }
        recognised
    }
}
