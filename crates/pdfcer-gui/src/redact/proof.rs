//! # `redact::proof` — the absence proof, and the only thing entitled to the
//! word *verified*
//!
//! Design and rationale: `docs/modules/pdfcer-gui/redact/proof.md`.

use pdfcer_core::document::Document;
use pdfcer_core::object::{ObjId, Object};

/// The shortest redacted string whose presence **outside every content-bearing
/// stream** is worth asserting anything about.
///
/// Below this length a byte-run match tells you nothing: `"Dr"` occurs inside
/// `/Widths`-adjacent binary, font names, dates and half the words in any
/// document, so such a hit would fire on a perfectly good redaction. Four
/// characters is the point at which a coincidental match stops being the
/// expected outcome — chosen deliberately conservatively, and paired with the
/// fact that short strings are still verified against **content-bearing**
/// streams (where the same match *is* meaningful, because it is being drawn).
///
/// The floor governs the whole disclosure half — raw bytes **and** opaque
/// decoded streams — not the raw bytes alone. A two-character run inside a
/// compressed font program is the same coincidence this constant exists to
/// refuse to draw conclusions from, merely wearing a `/FlateDecode`.
///
/// The count of strings this excludes is reported, never hidden — see
/// [`AbsenceVerification::strings_too_short_for_raw_check`], and
/// [`crate::text::redact::verification_limit_line`], which is the sentence that
/// puts the number in front of the operator.
///
/// The floor governs BOTH halves of the proof — the refusal as well as the
/// disclosure. See [`leaked_in_content_streams`] for the per-glyph producer
/// that makes a refusal-half floor necessary.
pub const MIN_VERIFIABLE_LEN: usize = 4;

/// **Where a disclosed residual was found**, so the sentence about it can name
/// the place rather than say *"somewhere in the saved file"*.
///
/// This exists because the disclosure it feeds has to be **actionable**. A
/// residual an operator cannot place is a warning they can only ignore, and a
/// warning that is always ignored is worse than none, because it also trains
/// them to ignore the real one.
///
/// Naming the site converts *"the text is still in the file somewhere"* into
/// *"the text also spells a word inside an embedded font program"*, which the
/// operator can weigh in a second. It is still a disclosure and never a verdict:
/// pdfcer states where the bytes are, not what they mean.
///
/// The variants are **carriers**, deliberately in the engine's vocabulary
/// (`pdfcer_core::redact::CarrierStatus::carrier`), so the two disclosure
/// vocabularies on one screen do not diverge.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResidualSite {
    /// **In drawn content — a page's content stream, a form XObject, a
    /// pattern or a Type 3 glyph procedure — somewhere in the document.**
    ///
    /// **A hit here is a doubt, not a proven leak.** The proof greps the
    /// WHOLE output for each removed string while the mark covered one region
    /// of one page, so a hit is far more often than not another occurrence of
    /// the same word that was never selected — a title-block label, a note
    /// repeated on every sheet. pdfcer cannot tell that apart from the removed
    /// glyphs still being drawn. Resolving the doubt by refusing to write
    /// anything makes a multi-page drawing set unredactable whenever a marked
    /// word appears anywhere else in it.
    ///
    /// So it is a **disclosure behind the acknowledgement gate**, like every
    /// other residual: the window lists each such string with this site's
    /// sentence — which says *outside the area you marked* in as many words —
    /// and the operator ticks the box to proceed. The hard refusal survives
    /// for exactly one case: a survivor found at write time that was NOT in
    /// the acknowledged list (the document changed between the two), which is
    /// the module's "removal and report disagree" line.
    DrawnContent,
    /// An embedded font program — `/FontFile`, `/FontFile2` or `/FontFile3`.
    ///
    /// By far the most common site, and the one that made this whole
    /// classification necessary: a font's `name` table carries its family name,
    /// its copyright, its licence URL and the English descriptions of every
    /// OpenType feature it implements. Ordinary words live there by
    /// construction.
    FontProgram,
    /// The sample data of an image XObject.
    ///
    /// Arbitrary bytes. A four-character run occurring in a megapixel of
    /// photographic noise is unremarkable; a run occurring in a *screenshot of
    /// the redacted text* is not, and pdfcer cannot tell those apart, so it says
    /// where it looked and stops.
    ImageSamples,
    /// A compressed object container (`/Type /ObjStm`).
    ///
    /// Engine rule R38's case: promoting an object out of a container leaves the
    /// container's own copy of its previous value behind. Page content can never
    /// live in one (ISO 32000-1 §7.5.7), so this cannot be drawn text — but it
    /// can be a string in a dictionary, which a viewer may still show.
    ObjectContainer,
    /// An embedded file attachment (`/Type /EmbeddedFile`).
    ///
    /// The one site on this list where a hit is most likely to be **real**: an
    /// attachment is a whole other document, and redaction does not reach into
    /// it. The engine discloses `attachments` as a carrier for the same reason.
    Attachment,
    /// A metadata stream (`/Type /Metadata`).
    Metadata,
    /// A decoded stream this build does not classify further.
    OtherStream,
    /// Not inside any decoded stream — in the file's raw bytes.
    ///
    /// A string in a dictionary, an unfiltered stream, a name object, a
    /// cross-reference table. The original middle verdict, unchanged.
    RawBytes,
}

/// One disclosed residual: a removed string that is absent from everything the
/// document draws, and present somewhere else.
///
/// A struct rather than a bare `String`, so the site travels with the text
/// instead of being re-derived (or, more likely, lost) by whichever surface
/// renders it. Rule 15's spirit: a value that means *"the text `X` occurs in a
/// font program"* must not be able to degrade into a value that means *"`X`"*.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Residual {
    /// The removed string that was found again.
    pub text: String,
    /// Where it was found.
    pub site: ResidualSite,
}

/// What a decoded stream **is**, for the one question this module asks of it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum StreamRole {
    /// A renderer draws this and a text extractor reads it: page content, a
    /// form XObject (which is also what an annotation's appearance stream is),
    /// a tiling pattern's cell, a Type 3 glyph procedure.
    ///
    /// A redacted string surviving here is the glyphs still being painted.
    Content,
    /// Everything else. Bytes that are *about* the document rather than bytes
    /// the document shows.
    Opaque(ResidualSite),
}

/// One decoded stream and what it is.
struct DecodedStream {
    /// What the stream is for.
    role: StreamRole,
    /// Its decoded bytes.
    bytes: Vec<u8>,
}

/// What the absence proof found, for the report the operator reads before
/// confirming.
///
/// This is the structure the wording contract reads: *"never say **verified**
/// unless a real verification step ran"*. [`Self::is_clean`] is the predicate
/// that licenses the stronger word, and
/// [`crate::text::redact::verified_line`] is the only sentence in the catalog
/// permitted to use it.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct AbsenceVerification {
    /// Distinct redacted strings the proof checked against decoded streams.
    pub strings_checked: usize,
    /// How many of those were too short for the **disclosure** half to say
    /// anything about ([`MIN_VERIFIABLE_LEN`]).
    ///
    /// Reported so the operator can see the proof's own limit rather than
    /// inferring a completeness it does not have. These strings *were* checked
    /// against every content-bearing stream, where a hit is still a refusal —
    /// which is what [`crate::text::redact::verification_limit_line`] tells the
    /// operator in so many words.
    pub strings_too_short_for_raw_check: usize,
    /// Redacted strings that still occur somewhere in the output while
    /// occurring in **no content-bearing stream** — with the place named.
    ///
    /// Disclosed, acknowledgement-gated, never silently dropped — and never
    /// described as a confirmed leak either, because pdfcer genuinely cannot
    /// tell an un-recognised carrier from a coincidental byte run (module
    /// docs).
    ///
    /// The name is deliberately not `raw_byte_residuals`: an opaque decoded
    /// hit is a disclosure too, so this field carries more than raw-byte hits,
    /// and a name that is a lie about half its contents is, on this screen,
    /// the one thing that must never happen.
    pub residuals: Vec<Residual>,
}

impl AbsenceVerification {
    /// Whether every checked string is absent from the output by every measure
    /// this build applies — the condition under which the post-apply wording may
    /// say "verified".
    #[must_use]
    pub const fn is_clean(&self) -> bool {
        self.residuals.is_empty()
    }

    /// Whether `text` was disclosed to the operator as a drawn-content hit
    /// at preparation — the write-time check's question.
    #[must_use]
    pub fn disclosed_in_drawn_content(&self, text: &str) -> bool {
        self.residuals
            .iter()
            .any(|r| r.site == ResidualSite::DrawnContent && r.text == text)
    }
}

/// Everything one pass of the proof establishes.
///
/// Two fields because the two answers have **different consequences** and must
/// not be collapsed: [`Self::survivors`] is a refusal and
/// [`Self::verification`] is a disclosure. A single "how did it go" value would
/// invite a caller to treat the worse one as the milder one, which is precisely
/// the reading this whole module exists to prevent.
#[derive(Debug, Clone, Default)]
pub(super) struct Proof {
    /// The disclosure half: what was checked, what the length floor could not
    /// speak to, and which strings survive outside everything the document
    /// draws — each with the place it was found.
    pub(super) verification: AbsenceVerification,
    /// The refusal half: redacted strings still present in a **content-bearing**
    /// decoded stream of the output. `None` is clean by this measure; `Some` is
    /// a leak and the caller must write nothing.
    pub(super) survivors: Option<Vec<String>>,
}

/// **Run both halves of the proof over `bytes`, decoding the document once.**
///
/// See the module docs for the one departure from the salvage source this
/// represents, and for why the two halves are still separate functions.
pub(super) fn prove(bytes: &[u8], redacted: &[String]) -> Proof {
    let decoded = decoded_streams_of(bytes);
    Proof {
        verification: verify_absence(bytes, redacted, &decoded),
        survivors: leaked_in_content_streams(redacted, &decoded),
    }
}

/// **The refusal half, run on its own against `bytes`.**
///
/// The write path's last gate: [`super::PreparedRedaction::write_to`] re-asks
/// this question about the exact buffer it is a statement away from handing to
/// the file system. See that method's docs for why a second run of a check that
/// has already passed is not redundancy but the thing that makes the proof
/// **structural** rather than procedural.
pub(super) fn survivors_in_content_streams(
    bytes: &[u8],
    redacted: &[String],
) -> Option<Vec<String>> {
    leaked_in_content_streams(redacted, &decoded_streams_of(bytes))
}

/// The refusal half of the absence proof, isolated so the refusal branch in
/// [`super::prepare_redaction_apply`] reads as one question.
fn leaked_in_content_streams(
    redacted: &[String],
    decoded: &[DecodedStream],
) -> Option<Vec<String>> {
    if redacted.is_empty() {
        return None;
    }
    let survivors: Vec<String> = redacted
        .iter()
        .filter(|needle| {
            needle.chars().count() >= MIN_VERIFIABLE_LEN && in_content(decoded, needle)
        })
        .cloned()
        .collect();
    if survivors.is_empty() {
        None
    } else {
        Some(survivors)
    }
}

/// Whether `needle` occurs in any stream the document actually draws.
fn in_content(decoded: &[DecodedStream], needle: &str) -> bool {
    decoded
        .iter()
        .filter(|s| s.role == StreamRole::Content)
        .any(|s| contains(&s.bytes, needle.as_bytes()))
}

/// Build the [`AbsenceVerification`] the report renders: how much was checked,
/// how much the length floor could not speak to, and which strings survive
/// somewhere the document does not draw.
fn verify_absence(
    bytes: &[u8],
    redacted: &[String],
    decoded: &[DecodedStream],
) -> AbsenceVerification {
    let mut out = AbsenceVerification {
        strings_checked: redacted
            .iter()
            .filter(|s| s.chars().count() >= MIN_VERIFIABLE_LEN)
            .count(),
        ..AbsenceVerification::default()
    };
    for needle in redacted {
        if needle.is_empty() {
            continue;
        }
        if needle.chars().count() < MIN_VERIFIABLE_LEN {
            out.strings_too_short_for_raw_check += 1;
            continue;
        }
        // A content hit is DISCLOSED here, and refused only if it is still
        // undisclosed at write time. See `ResidualSite::DrawnContent`.
        if in_content(decoded, needle) {
            out.residuals.push(Residual {
                text: needle.clone(),
                site: ResidualSite::DrawnContent,
            });
            continue;
        }
        let opaque_site = decoded
            .iter()
            .find(|s| s.role != StreamRole::Content && contains(&s.bytes, needle.as_bytes()))
            .and_then(|s| match s.role {
                StreamRole::Opaque(site) => Some(site),
                StreamRole::Content => None,
            });
        let site = match opaque_site {
            Some(site) => Some(site),
            None if contains(bytes, needle.as_bytes()) => Some(ResidualSite::RawBytes),
            None => None,
        };
        if let Some(site) = site {
            out.residuals.push(Residual {
                text: needle.clone(),
                site,
            });
        }
    }
    out
}

/// Parse `bytes`, decode every stream in it, and say what each one is.
fn decoded_streams_of(bytes: &[u8]) -> Vec<DecodedStream> {
    Document::from_bytes(bytes.to_vec())
        .map(|doc| decode_every_stream(&doc))
        .unwrap_or_default()
}

/// Decode **every** stream in the document, not merely page content, and label
/// each with what it is.
fn decode_every_stream(doc: &Document) -> Vec<DecodedStream> {
    let view = doc.view();
    let content_ids = content_stream_ids(doc);
    let mut out = Vec::new();
    for object in doc.objects() {
        let Object::Stream(stream) = &object.value else {
            continue;
        };
        let Some(raw) = view.slice(stream.data_span) else {
            continue;
        };
        if let Ok(decoded) = pdfcer_core::filters::decode_stream(&stream.dict, raw) {
            out.push(DecodedStream {
                role: role_of(&stream.dict, object.id, &content_ids),
                bytes: decoded,
            });
        }
    }
    out
}

/// Every object id this document reaches as **drawn content by reference**:
/// each page's `/Contents`, and every Type 3 font's `/CharProcs` entries.
fn content_stream_ids(doc: &Document) -> Vec<ObjId> {
    let mut ids: Vec<ObjId> = pdfcer_core::page_tree::pages(doc)
        .map(|pages| pages.iter().flat_map(|p| p.contents.clone()).collect())
        .unwrap_or_default();
    // Type 3 glyph procedures: `/Subtype /Type3` fonts hold a `/CharProcs`
    // dictionary whose every value is a content stream drawn for one character
    // code. A redacted string surviving in one is the redacted glyph itself.
    for object in doc.objects() {
        let Some(dict) = object.value.as_dict() else {
            continue;
        };
        if dict
            .get(b"Subtype")
            .map(|o| doc.resolve(o))
            .and_then(Object::as_name)
            .is_none_or(|n| n.as_bytes() != b"Type3")
        {
            continue;
        }
        let Some(procs) = dict
            .get(b"CharProcs")
            .map(|o| doc.resolve(o))
            .and_then(Object::as_dict)
        else {
            continue;
        };
        ids.extend(procs.iter().filter_map(|(_, v)| v.as_reference()));
    }
    ids
}

/// **Classify one stream: does the document DRAW this, or is it about the
/// document?**
fn role_of(dict: &pdfcer_core::object::Dict, id: ObjId, content_ids: &[ObjId]) -> StreamRole {
    if content_ids.contains(&id) {
        return StreamRole::Content;
    }
    let subtype = dict
        .get(b"Subtype")
        .and_then(Object::as_name)
        .map(|n| n.as_bytes().to_vec());
    let type_ = dict
        .get(b"Type")
        .and_then(Object::as_name)
        .map(|n| n.as_bytes().to_vec());
    let subtype = subtype.as_deref();
    match subtype {
        Some(b"Form") => return StreamRole::Content,
        Some(b"Image") => return StreamRole::Opaque(ResidualSite::ImageSamples),
        Some(b"Type1C" | b"CIDFontType0C" | b"OpenType") => {
            return StreamRole::Opaque(ResidualSite::FontProgram);
        }
        _ => {}
    }
    if dict.get(b"PatternType").and_then(Object::as_int) == Some(1) {
        return StreamRole::Content;
    }
    if dict.contains_key(b"Length1") {
        return StreamRole::Opaque(ResidualSite::FontProgram);
    }
    match type_.as_deref() {
        Some(b"ObjStm") => StreamRole::Opaque(ResidualSite::ObjectContainer),
        Some(b"EmbeddedFile") => StreamRole::Opaque(ResidualSite::Attachment),
        Some(b"Metadata") => StreamRole::Opaque(ResidualSite::Metadata),
        _ => StreamRole::Opaque(ResidualSite::OtherStream),
    }
}

/// Whether `hay` contains `needle` as a byte subsequence.
///
/// The same naive scan `pdfcer-core`'s own absence tests use, kept local rather
/// than exported from core: it is three lines, and **an absence proof that
/// shared its search routine with the code it is auditing would be a weaker
/// proof.**
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
        assert_eq!(proof.verification.strings_too_short_for_raw_check, 0);
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
        assert_eq!(proof.verification.strings_too_short_for_raw_check, 1);
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
        assert_eq!(proof.verification.strings_too_short_for_raw_check, 1);
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
        assert_eq!(proof.verification.strings_too_short_for_raw_check, 7);
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
