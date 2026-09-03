//! # `redact::proof` — the absence proof, and the only thing entitled to the
//! word *verified*
//!
//! Salvaged from `D:\Dev\pdfce\crates\pdfce-gui\src\redact_apply.rs:338-428`
//! on 2026-08-15, with every paragraph of its reasoning carried across. That
//! file is the subject of `SALVAGE.md`'s starred note — *"★ This file is
//! currently the ONLY place the proof exists"* — and this module is the half of
//! it that does the proving.
//!
//! ## What a proof is for, here
//!
//! [`pdfcer_core::redact::apply_redactions`] returns a **[`RedactionReport`]** —
//! a description of what the surgery believes it did. It does not return a
//! verdict, and at the engine's HEAD on 2026-08-15 there is no
//! `RedactionVerdict` and no `verify_redaction` anywhere in `pdfcer-core`
//! (checked directly rather than quoted; `SALVAGE.md`'s Pass 72.0 note stands).
//! So a caller that writes the report's bytes has been told what happened by
//! the code that did it, which — as `tools/ui-verify`'s `save_copy` check
//! records from the other side — is *"a trace line written by the code under
//! test, about itself."*
//!
//! This module is the independent reader. It takes the finished bytes and the
//! list of strings the surgery says it removed, and it goes and looks.
//!
//! [`RedactionReport`]: pdfcer_core::redact::RedactionReport
//!
//! ## The three verdicts, and why the middle one is not a refusal
//!
//! [`pdfcer_core::redact::RedactionReport::redacted_text`] carries the distinct
//! strings the surgery decoded while removing them — kept, in core's own words,
//! *"for the absence-proof gate to grep"*. Each one is looked for twice:
//!
//! | Where the string still occurs | Verdict | Why |
//! |---|---|---|
//! | in a **decoded stream** of the output | **REFUSE** — write nothing | A decoded stream is content a renderer or a text extractor will read back. Its survival is a real leak, not a coincidence, and no acknowledgement checkbox makes it acceptable. |
//! | in the **raw bytes only** (no decoded stream) | **DISCLOSE** as a residual requiring the operator's explicit acknowledgement | pdfcer cannot tell a genuine un-recognised carrier from an unrelated coincidence (the same byte run inside a font name, an ID string, a compressed blob). Refusing would be a trap the operator cannot act on; claiming removal would be a lie. Naming it is the only honest option. |
//! | nowhere | **verified** | This is what licenses [`crate::text::redact`]'s wording contract to use the word "verified" at all. |
//!
//! Strings shorter than [`MIN_VERIFIABLE_LEN`] are excluded from the raw-byte
//! half and **counted separately** (see
//! [`AbsenceVerification::strings_too_short_for_raw_check`]) rather than
//! silently skipped: a two-character redaction would match somewhere in any
//! real file, so a raw-byte grep for it carries no information, and pretending
//! it does would turn the disclosure into noise operators learn to click
//! through. They are still checked against decoded streams, where a survival
//! *is* meaningful.
//!
//! ## ★ One departure from the source, and it is the only one
//!
//! The old file called `verify_absence` and `leaked_in_decoded_streams` one
//! after the other, and **each decoded every stream in the document
//! independently** — two full inflate passes over the finished file for one
//! question. Here [`prove`] decodes once and hands the same blobs to both
//! halves. The two halves stay separate functions because they answer two
//! separate questions and each has its own test; what is shared is the
//! evidence, not the reasoning.
//!
//! Everything else — the four-character floor, the wide stream sweep, the local
//! `contains`, the classification table above — is carried verbatim, including
//! the arguments for each.

use pdfcer_core::document::Document;
use pdfcer_core::object::Object;

/// The shortest redacted string whose absence from the **raw** output bytes is
/// worth asserting.
///
/// Below this length a byte-run match tells you nothing: `"Dr"` occurs inside
/// `/Widths`-adjacent binary, font names, dates and half the words in any
/// document, so a raw-byte hit would fire on a perfectly good redaction. Four
/// characters is the point at which a coincidental match stops being the
/// expected outcome — chosen deliberately conservatively, and paired with the
/// fact that short strings are still verified against decoded streams (where
/// the same match *is* meaningful because it is content).
///
/// The count of strings this excludes is reported, never hidden — see
/// [`AbsenceVerification::strings_too_short_for_raw_check`], and
/// [`crate::text::redact::verification_limit_line`], which is the sentence that
/// puts the number in front of the operator.
pub const MIN_VERIFIABLE_LEN: usize = 4;

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
    /// How many of those were too short for the raw-byte half to say anything
    /// about ([`MIN_VERIFIABLE_LEN`]).
    ///
    /// Reported so the operator can see the proof's own limit rather than
    /// inferring a completeness it does not have.
    pub strings_too_short_for_raw_check: usize,
    /// Redacted strings that still occur somewhere in the raw output bytes
    /// while occurring in **no** decoded stream.
    ///
    /// Disclosed, acknowledgement-gated, never silently dropped — and never
    /// described as a confirmed leak either, because pdfcer genuinely cannot
    /// tell an un-recognised carrier from a coincidental byte run (module
    /// docs).
    pub raw_byte_residuals: Vec<String>,
}

impl AbsenceVerification {
    /// Whether every checked string is absent from the output by both measures
    /// — the condition under which the post-apply wording may say "verified".
    #[must_use]
    pub const fn is_clean(&self) -> bool {
        self.raw_byte_residuals.is_empty()
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
    /// The disclosure half: what was checked, what the raw-byte grep could not
    /// speak to, and which strings survive in the raw bytes alone.
    pub(super) verification: AbsenceVerification,
    /// The refusal half: redacted strings still present in a **decoded stream**
    /// of the output. `None` is clean by this measure; `Some` is a leak and the
    /// caller must write nothing.
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
        survivors: leaked_in_decoded_streams(redacted, &decoded),
    }
}

/// **The decoded-stream half, run on its own against `bytes`.**
///
/// The write path's last gate: [`super::PreparedRedaction::write_to`] re-asks
/// this question about the exact buffer it is a statement away from handing to
/// the file system. See that method's docs for why a second run of a check that
/// has already passed is not redundancy but the thing that makes the proof
/// **structural** rather than procedural.
pub(super) fn survivors_in_decoded_streams(
    bytes: &[u8],
    redacted: &[String],
) -> Option<Vec<String>> {
    leaked_in_decoded_streams(redacted, &decoded_streams_of(bytes))
}

/// The decoded-stream half of the absence proof, isolated so the refusal branch
/// in [`super::prepare_redaction_apply`] reads as one question.
///
/// Returns `Some(survivors)` when any redacted string is still present in a
/// decoded stream — content a renderer or extractor would read back — and
/// `None` when the output is clean by that measure. Strings of **any** length
/// are checked here (unlike the raw-byte half): inside a decoded content stream
/// even a two-character survival is the redacted glyphs still being drawn.
fn leaked_in_decoded_streams(redacted: &[String], decoded: &[Vec<u8>]) -> Option<Vec<String>> {
    if redacted.is_empty() {
        return None;
    }
    let survivors: Vec<String> = redacted
        .iter()
        .filter(|needle| {
            !needle.is_empty() && decoded.iter().any(|blob| contains(blob, needle.as_bytes()))
        })
        .cloned()
        .collect();
    if survivors.is_empty() {
        None
    } else {
        Some(survivors)
    }
}

/// Build the [`AbsenceVerification`] the report renders: how much was checked,
/// how much the raw-byte half could not speak to, and which strings survive in
/// the raw bytes without surviving in any decoded stream.
///
/// `decoded` is passed in rather than computed because a residual is *"in the
/// raw bytes AND in no decoded stream"*, so both halves are needed to classify
/// a single hit — and because [`prove`] already has them.
fn verify_absence(bytes: &[u8], redacted: &[String], decoded: &[Vec<u8>]) -> AbsenceVerification {
    let mut out = AbsenceVerification {
        strings_checked: redacted.iter().filter(|s| !s.is_empty()).count(),
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
        let raw_hit = contains(bytes, needle.as_bytes());
        let decoded_hit = decoded.iter().any(|blob| contains(blob, needle.as_bytes()));
        if raw_hit && !decoded_hit {
            out.raw_byte_residuals.push(needle.clone());
        }
    }
    out
}

/// Parse `bytes` and decode every stream in it.
///
/// A document that cannot be re-parsed yields an **empty** list rather than a
/// panic or an error. That looks like a false clean bill and is not, for a
/// reason worth stating plainly: these are bytes pdfcer itself just wrote, so an
/// unparsable output means a **writer** bug, and the raw-byte half of the proof
/// still covers the whole buffer either way. A skip narrows the evidence rather
/// than fabricating it — and [`super::prepare_redaction_apply`] is separately
/// unable to produce such a buffer, because it re-parses the output itself.
fn decoded_streams_of(bytes: &[u8]) -> Vec<Vec<u8>> {
    Document::from_bytes(bytes.to_vec())
        .map(|doc| decode_every_stream(&doc))
        .unwrap_or_default()
}

/// Decode **every** stream in the document, not merely page content.
///
/// The wide sweep is the point. A redaction that only proved absence from page
/// content streams would say nothing about a form XObject, a metadata stream,
/// an embedded file, or — the case that actually motivated this — an
/// **object-stream container**, whose compressed payload can carry a stale copy
/// of a dictionary that was promoted out of it (engine rule R38). Decoding the
/// container like any other stream is what lets a grep see that copy at all.
///
/// Streams whose filters this build cannot decode are skipped rather than
/// failed: their *raw* bytes are still covered by the raw-byte half of the
/// proof, so a skip narrows the evidence rather than fabricating it.
fn decode_every_stream(doc: &Document) -> Vec<Vec<u8>> {
    let view = doc.view();
    let mut out = Vec::new();
    for object in doc.objects() {
        let Object::Stream(stream) = &object.value else {
            continue;
        };
        let Some(raw) = view.slice(stream.data_span) else {
            continue;
        };
        if let Ok(decoded) = pdfcer_core::filters::decode_stream(&stream.dict, raw) {
            out.push(decoded);
        }
    }
    out
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
    ///
    /// Synthetic rather than a fixture file: the point of every test here is a
    /// *known* byte layout, and a real producer's output would make "the string
    /// is in a decoded stream" an accident of that producer's filter choices.
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

    /// ★ **The instrument registers a survival.**
    ///
    /// The first thing to establish about any absence proof, and the one
    /// `HANDOFF.md` §2's grid lesson is about: a check that only ever reports
    /// "clean" is satisfied by any build at all. So this asserts the *positive*
    /// — a string that genuinely is in a decoded stream is found — before
    /// anything below asserts an absence.
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

    /// ★ **A string in the raw bytes but in no decoded stream is a disclosed
    /// residual, not a refusal.**
    ///
    /// The middle row of the module's table, which is the row a simpler design
    /// would collapse. The fixture puts the run in a place no content stream
    /// reaches — a `/BaseFont` name — which is exactly the "unrelated
    /// coincidence" case the wording is careful not to call a leak.
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
            proof.verification.raw_byte_residuals,
            vec!["MARGARETHALE".to_owned()],
            "…and it must be DISCLOSED rather than passed over, because pdfcer \
             cannot tell a coincidence from an unrecognised carrier"
        );
        assert!(!proof.verification.is_clean());
    }

    /// ★ **A short string is counted, not silently skipped — and it is still
    /// checked against decoded streams.**
    ///
    /// Both halves of [`MIN_VERIFIABLE_LEN`]'s argument, because dropping
    /// either produces a proof that lies in a different direction: skip the
    /// count and the operator is told the file was fully searched when it was
    /// not; skip the decoded check and a two-character redaction that is still
    /// being *drawn* passes as clean.
    #[test]
    fn a_short_string_is_counted_as_unverifiable_and_still_checked_where_it_matters() {
        // Too short for the raw half, and absent — counted, no residual.
        let clean = pdf_drawing("nothing here");
        let proof = prove(&clean, &["ab".to_owned()]);
        assert_eq!(proof.verification.strings_too_short_for_raw_check, 1);
        assert!(proof.verification.raw_byte_residuals.is_empty());
        assert_eq!(proof.survivors, None);

        // Too short for the raw half, and PRESENT in a content stream — still
        // a refusal, because inside a stream even two characters are glyphs
        // that are still being drawn.
        let leaking = pdf_drawing("ab is drawn");
        let proof = prove(&leaking, &["ab".to_owned()]);
        assert_eq!(
            proof.survivors,
            Some(vec!["ab".to_owned()]),
            "the length floor governs the RAW-byte grep only; a short string \
             surviving in a decoded stream is the redacted glyphs still on the \
             page"
        );
    }

    /// An empty needle and an empty list are both no-ops rather than matches.
    ///
    /// `contains` returns `false` for an empty needle deliberately: the
    /// mathematically-correct answer (`true`, every haystack contains the empty
    /// string) would make every proof report a leak.
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
    ///
    /// No stream can be decoded, so the decoded half reports nothing — and the
    /// **raw** half still finds the run, which is what stops this from reading
    /// as a clean bill.
    #[test]
    fn bytes_that_do_not_parse_still_get_the_raw_byte_half() {
        let junk = b"this is not a pdf at all, MARGARETHALE".to_vec();
        let proof = prove(&junk, &["MARGARETHALE".to_owned()]);
        assert_eq!(proof.survivors, None, "nothing could be decoded");
        assert_eq!(
            proof.verification.raw_byte_residuals,
            vec!["MARGARETHALE".to_owned()],
            "the raw half covers the whole buffer whatever the parser thinks"
        );
    }

    /// The wide sweep reaches a stream that is **not** page content.
    ///
    /// The case that motivated `decode_every_stream`: a proof that only read
    /// `/Contents` would report this file clean while the string sat in a form
    /// XObject that every renderer draws.
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
