//! # `redact::proof` — the absence proof, read through the engine's verb
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/redact/proof.md`.

use pdfcer_core::redact::verify_absence;
pub use pdfcer_core::redact::{AbsenceVerdict, AbsenceVerification, Residual, ResidualSite};

/// Everything one pass of the proof establishes.
#[derive(Debug, Clone, Default)]
pub(super) struct Proof {
    /// The disclosure half: what was checked, what the length floor could not
    /// speak to, what could not be searched, and every residual with its site.
    pub(super) verification: AbsenceVerification,
    /// The refusal half: redacted strings still present in **drawn** content.
    /// `None` is clean by this measure; `Some` is a leak the operator must be
    /// shown before anything is written.
    pub(super) survivors: Option<Vec<String>>,
}

/// **Run the engine's absence proof over `bytes`.**
pub(super) fn prove(bytes: &[u8], redacted: &[String]) -> Proof {
    let verification = verify_absence(bytes, redacted);
    let survivors: Vec<String> = verification.drawn_survivors().map(str::to_owned).collect();
    Proof {
        verification,
        survivors: (!survivors.is_empty()).then_some(survivors),
    }
}

/// **The refusal half alone: redacted strings still in drawn content.**
pub(super) fn survivors_in_content_streams(
    bytes: &[u8],
    redacted: &[String],
) -> Option<Vec<String>> {
    if redacted.is_empty() {
        return None;
    }
    prove(bytes, redacted).survivors
}

/// Whether `text` was disclosed to the operator as a drawn-content hit at
/// preparation — the write-time check's question.
pub(super) fn disclosed_in_drawn_content(verification: &AbsenceVerification, text: &str) -> bool {
    verification.drawn_survivors().any(|s| s == text)
}

/// Whether `hay` contains `needle` as a byte subsequence — the tests' own
/// witness, independent of the proof under test.
#[cfg(test)]
pub(super) fn contains(hay: &[u8], needle: &[u8]) -> bool {
    if needle.is_empty() || needle.len() > hay.len() {
        return false;
    }
    hay.windows(needle.len()).any(|w| w == needle)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A one-page PDF whose content stream draws `text`, uncompressed.
    fn pdf_drawing(text: &str) -> Vec<u8> {
        let content = format!("BT /F1 12 Tf 20 100 Td ({text}) Tj ET");
        let stream = format!(
            "<< /Length {} >>\nstream\n{content}\nendstream",
            content.len()
        );
        super::super::tests::assemble(&[
            "<< /Type /Catalog /Pages 2 0 R >>",
            "<< /Type /Pages /Kids [3 0 R] /Count 1 >>",
            "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 400 200] \
             /Resources << /Font << /F1 5 0 R >> >> /Contents 4 0 R >>",
            &stream,
            "<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>",
        ])
    }

    /// **The instrument registers a survival.**
    #[test]
    fn a_string_still_in_a_decoded_stream_is_reported_as_a_survivor() {
        let bytes = pdf_drawing("KEEPTHISSECRET");
        let redacted = vec!["KEEPTHISSECRET".to_owned()];
        let proof = prove(&bytes, &redacted);
        assert_eq!(
            proof.survivors,
            Some(vec!["KEEPTHISSECRET".to_owned()]),
            "the proof did not see a string sitting in plain sight in a page \
             content stream; every absence it reports elsewhere is worthless"
        );
    }

    /// …and a document that never contained the string is clean by both
    /// measures.
    #[test]
    fn a_string_that_was_never_there_is_clean() {
        let bytes = pdf_drawing("SOMETHINGELSE");
        let redacted = vec!["KEEPTHISSECRET".to_owned()];
        let proof = prove(&bytes, &redacted);
        assert_eq!(proof.survivors, None);
        assert!(proof.verification.is_clean());
        assert_eq!(proof.verification.strings_checked, 1);
        assert_eq!(proof.verification.strings_too_short, 0);
    }

    /// **A string in the raw bytes but in no decoded stream is a disclosed
    /// residual, not a refusal.**
    #[test]
    fn a_raw_byte_run_outside_every_stream_is_disclosed_rather_than_refused() {
        let content = "BT /F1 12 Tf 20 100 Td (ordinary) Tj ET";
        let stream = format!(
            "<< /Length {} >>\nstream\n{content}\nendstream",
            content.len()
        );
        let bytes = super::super::tests::assemble(&[
            "<< /Type /Catalog /Pages 2 0 R >>",
            "<< /Type /Pages /Kids [3 0 R] /Count 1 >>",
            "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 400 200] \
             /Resources << /Font << /F1 5 0 R >> >> /Contents 4 0 R >>",
            &stream,
            "<< /Type /Font /Subtype /Type1 /BaseFont /MARGARETHALE >>",
        ]);
        let proof = prove(&bytes, &["MARGARETHALE".to_owned()]);
        assert_eq!(
            proof.survivors, None,
            "a run outside every decoded stream is not a leak of drawn content, \
             and refusing on it would be a trap the operator cannot act on"
        );
        assert_eq!(
            proof.verification.residuals,
            vec![Residual {
                text: "MARGARETHALE".to_owned(),
                site: ResidualSite::RawBytes,
            }],
            "…and it must be DISCLOSED rather than passed over, because pdfcer \
             cannot tell a coincidence from an unrecognised carrier — and it \
             must say WHERE it looked, or the operator has nothing to weigh"
        );
        assert!(!proof.verification.is_clean());
    }

    /// **A short string is counted, not silently skipped — and it is not
    /// refused on, even inside drawn content.**
    #[test]
    fn a_short_string_is_counted_as_unverifiable_and_the_long_one_still_refuses() {
        // Too short, and absent — counted, no residual, no refusal.
        let clean = pdf_drawing("nothing here");
        let proof = prove(&clean, &["ab".to_owned()]);
        assert_eq!(proof.verification.strings_too_short, 1);
        assert_eq!(
            proof.verification.strings_checked, 0,
            "a needle the proof cannot check is not counted as checked"
        );
        assert!(proof.verification.residuals.is_empty());
        assert_eq!(proof.survivors, None);
        // Too short, and PRESENT in a content stream — counted, NOT refused.
        let leaking = pdf_drawing("ab is drawn");
        let proof = prove(&leaking, &["ab".to_owned()]);
        assert_eq!(
            proof.survivors, None,
            "a needle under the floor is the alphabet, not a leak — refusing on \
             it refused every drawing a per-glyph producer ever wrote"
        );
        assert_eq!(proof.verification.strings_too_short, 1);
        // The control that keeps the line where it belongs: a needle AT the
        // floor, drawn, is still the hard refusal.
        let proof = prove(&pdf_drawing("abcd is drawn"), &["abcd".to_owned()]);
        assert_eq!(proof.survivors, Some(vec!["abcd".to_owned()]));
        assert_eq!(proof.verification.strings_checked, 1);
    }

    /// **The operator's file, in miniature.** A per-glyph producer's
    /// `redacted_text` — every character of `3.5 TYP` as its own string — over
    /// a page that still draws those characters elsewhere (every drawing does)
    /// is clean by the refusal half, and the disclosure half says exactly how
    /// many pieces it could not speak to. Without the floor on this half the
    /// proof refuses with five survivors on a removal that succeeded.
    #[test]
    fn a_per_glyph_producers_single_character_needles_do_not_refuse() {
        let page = pdf_drawing("DRAWING 3 OF 5, TYP. NOTES");
        let needles: Vec<String> = "3.5 TYP".chars().map(|c| c.to_string()).collect();
        let proof = prove(&page, &needles);
        assert_eq!(proof.survivors, None, "single characters are not survivors");
        assert_eq!(proof.verification.strings_too_short, 7);
        assert_eq!(proof.verification.strings_checked, 0);
        assert!(proof.verification.is_clean());
    }

    /// An empty needle and an empty list are both no-ops rather than matches.
    #[test]
    fn an_empty_needle_matches_nothing() {
        assert!(!contains(b"anything", b""));
        assert!(!contains(b"", b"x"));
        let bytes = pdf_drawing("ordinary");
        assert_eq!(prove(&bytes, &[]).survivors, None);
        assert_eq!(
            prove(&bytes, &[String::new()]).verification.strings_checked,
            0
        );
    }

    /// Unparsable bytes narrow the evidence rather than fabricating it.
    #[test]
    fn bytes_that_do_not_parse_still_get_the_raw_byte_half() {
        let junk = b"this is not a pdf at all, MARGARETHALE".to_vec();
        let proof = prove(&junk, &["MARGARETHALE".to_owned()]);
        assert_eq!(proof.survivors, None, "nothing could be decoded");
        assert_eq!(
            proof.verification.residuals,
            vec![Residual {
                text: "MARGARETHALE".to_owned(),
                site: ResidualSite::RawBytes,
            }],
            "the raw half covers the whole buffer whatever the parser thinks"
        );
    }

    /// A one-page PDF that draws `drawn` and carries one extra stream whose
    /// dictionary is `extra_dict` and whose body is `extra_body`.
    fn pdf_with_extra_stream(drawn: &str, extra_dict: &str, extra_body: &str) -> Vec<u8> {
        let content = format!("BT /F1 12 Tf 20 100 Td ({drawn}) Tj ET");
        let stream = format!(
            "<< /Length {} >>\nstream\n{content}\nendstream",
            content.len()
        );
        let extra = format!(
            "<< {extra_dict} /Length {} >>\nstream\n{extra_body}\nendstream",
            extra_body.len()
        );
        super::super::tests::assemble(&[
            "<< /Type /Catalog /Pages 2 0 R >>",
            "<< /Type /Pages /Kids [3 0 R] /Count 1 >>",
            "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 400 200] \
             /Resources << /Font << /F1 5 0 R >> >> /Contents 4 0 R >>",
            &stream,
            &extra,
        ])
    }

    /// **THE REGRESSION TEST. A removed word that also occurs inside an
    /// embedded font program is DISCLOSED, and the redaction goes ahead.**
    ///
    /// The defect this closes, in its smallest reproducing shape: a tool that
    /// refuses to redact anything because it always finds text that was not
    /// redacted. Were this font program classified as content, the fixture
    /// would produce `survivors: Some([…])`, which
    /// [`super::prepare_redaction_apply`] turns into
    /// [`super::RedactApplyRefusal::VerificationFailed`]: no file, no
    /// confirmation, no way forward.
    ///
    /// The needle is `construction` because that is the exact word that failed
    /// on `fixtures/a1-titleblock.pdf`, and the body it is hidden in is a
    /// three-word excerpt of the JetBrains Mono `name` table that caused it.
    ///
    /// Three assertions, and each one closes a different way of "fixing" this
    /// badly:
    ///
    /// 1. `survivors` is `None` — it does not refuse. A build that still
    ///    refuses fails here.
    /// 2. the residual is **present and names the site** — it does not go
    ///    silent. A build that "fixed" this by narrowing the sweep, or by
    ///    dropping non-content hits on the floor, fails here, and that is the
    ///    dangerous fix this test exists to forbid.
    /// 3. `is_clean()` is **false** — so the report cannot use the word
    ///    *verified*, and [`crate::dialogs::redact`] still demands the residual
    ///    acknowledgement before writing anything.
    #[test]
    fn a_word_inside_an_embedded_font_program_is_disclosed_rather_than_refused() {
        let bytes = pdf_with_extra_stream(
            "ordinary",
            "/Length1 3400",
            "Classic constructionClosed constructionBroken equals ligatures",
        );
        let proof = prove(&bytes, &[" construction".to_owned()]);
        assert_eq!(
            proof.survivors, None,
            "★ a font program's description of its own letterforms vetoed a \
             completed redaction: nothing was written, and the operator was \
             told the text they removed was still there"
        );
        assert_eq!(
            proof.verification.residuals,
            vec![Residual {
                text: " construction".to_owned(),
                site: ResidualSite::FontProgram,
            }],
            "★★ …and the other failure is worse: not refusing must not mean \
             not telling. The byte run IS in the file and the operator is owed \
             it, with the place named so it can be weighed"
        );
        assert!(
            !proof.verification.is_clean(),
            "a disclosed residual must forfeit the word 'verified' and raise \
             the acknowledgement gate"
        );
    }

    /// **Each opaque site is recognised and named**, so the sentence the
    /// operator reads is about the place the bytes actually are.
    #[test]
    fn every_opaque_site_is_recognised_by_its_own_dictionary() {
        let cases: &[(&str, ResidualSite)] = &[
            ("/Length1 3400", ResidualSite::FontProgram),
            ("/Subtype /Type1C", ResidualSite::FontProgram),
            ("/Subtype /OpenType", ResidualSite::FontProgram),
            (
                "/Type /XObject /Subtype /Image /Width 4 /Height 4",
                ResidualSite::ImageSamples,
            ),
            ("/Type /ObjStm /N 0 /First 0", ResidualSite::ObjectContainer),
            ("/Type /EmbeddedFile", ResidualSite::Attachment),
            ("/Type /Metadata /Subtype /XML", ResidualSite::Metadata),
            ("/Some /Thing", ResidualSite::OtherStream),
        ];
        for (dict, expected) in cases {
            let bytes = pdf_with_extra_stream("ordinary", dict, "MARGARETHALE lives here");
            let proof = prove(&bytes, &["MARGARETHALE".to_owned()]);
            assert_eq!(proof.survivors, None, "{dict}: must not refuse");
            assert_eq!(
                proof.verification.residuals,
                vec![Residual {
                    text: "MARGARETHALE".to_owned(),
                    site: *expected,
                }],
                "{dict}: the disclosed site is what the operator reads"
            );
        }
    }

    /// **A survivor in drawn content IS listed as a residual — at its own
    /// site — so the window can show it and the operator can decide.**
    #[test]
    fn a_survivor_in_drawn_content_is_listed_as_a_residual_at_its_own_site() {
        let bytes = pdf_drawing("the SECRET is drawn here");
        let proof = prove(&bytes, &["SECRET".to_owned()]);
        assert_eq!(proof.survivors, Some(vec!["SECRET".to_owned()]));
        assert_eq!(
            proof.verification.residuals,
            vec![Residual {
                text: "SECRET".to_owned(),
                site: ResidualSite::DrawnContent,
            }],
            "the operator is owed the string and the place, behind the gate"
        );
        assert!(!proof.verification.is_clean());
    }

    /// **A tiling pattern and a Type 3 glyph procedure are drawn content.**
    #[test]
    fn a_tiling_pattern_and_a_type3_glyph_procedure_are_drawn_content() {
        let pattern = pdf_with_extra_stream(
            "ordinary",
            "/PatternType 1 /PaintType 1 /TilingType 1 /BBox [0 0 8 8] /XStep 8 /YStep 8 \
             /Resources << >>",
            "BT (MARGARETHALE) Tj ET",
        );
        assert_eq!(
            prove(&pattern, &["MARGARETHALE".to_owned()]).survivors,
            Some(vec!["MARGARETHALE".to_owned()]),
            "a tiling pattern's cell is painted, repeatedly, all over the page"
        );

        // Object 6 is the glyph procedure; object 5 is the Type 3 font that
        // names it. The procedure's own dictionary is `/Length` and nothing
        // else, exactly like a page content stream's.
        let content = "BT /F1 12 Tf 20 100 Td (ordinary) Tj ET";
        let stream = format!(
            "<< /Length {} >>\nstream\n{content}\nendstream",
            content.len()
        );
        let glyph = "0 0 d0 BT (MARGARETHALE) Tj ET";
        let glyph_stream = format!("<< /Length {} >>\nstream\n{glyph}\nendstream", glyph.len());
        let type3 = super::super::tests::assemble(&[
            "<< /Type /Catalog /Pages 2 0 R >>",
            "<< /Type /Pages /Kids [3 0 R] /Count 1 >>",
            "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 400 200] \
             /Resources << /Font << /F1 5 0 R >> >> /Contents 4 0 R >>",
            &stream,
            "<< /Type /Font /Subtype /Type3 /FontBBox [0 0 8 8] \
             /FontMatrix [0.001 0 0 0.001 0 0] /CharProcs << /a 6 0 R >> \
             /Encoding << >> /FirstChar 97 /LastChar 97 /Widths [8] >>",
            &glyph_stream,
        ]);
        assert_eq!(
            prove(&type3, &["MARGARETHALE".to_owned()]).survivors,
            Some(vec!["MARGARETHALE".to_owned()]),
            "★ a Type 3 glyph procedure carries no /Type and no /Subtype — it \
             is recognised only by walking the font that names it, and missing \
             it would downgrade drawn glyphs to a tick-box"
        );
    }

    /// The wide sweep reaches a stream that is **not** page content.
    #[test]
    fn the_sweep_reaches_a_stream_that_is_not_page_content() {
        let page_content = "q /Fx0 Do Q";
        let page_stream = format!(
            "<< /Length {} >>\nstream\n{page_content}\nendstream",
            page_content.len()
        );
        let xobject_content = "BT /F1 12 Tf 10 10 Td (MARGARETHALE) Tj ET";
        let xobject = format!(
            "<< /Type /XObject /Subtype /Form /BBox [0 0 100 100] /Length {} >>\n\
             stream\n{xobject_content}\nendstream",
            xobject_content.len()
        );
        let bytes = super::super::tests::assemble(&[
            "<< /Type /Catalog /Pages 2 0 R >>",
            "<< /Type /Pages /Kids [3 0 R] /Count 1 >>",
            "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 400 200] \
             /Resources << /XObject << /Fx0 5 0 R >> >> /Contents 4 0 R >>",
            &page_stream,
            &xobject,
        ]);
        assert_eq!(
            prove(&bytes, &["MARGARETHALE".to_owned()]).survivors,
            Some(vec!["MARGARETHALE".to_owned()]),
            "the sweep stopped at page content; a form XObject is drawn by \
             every renderer and would have shipped the text"
        );
    }
}
