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
//! *"for the absence-proof gate to grep"*. Each one is looked for in three
//! places, and **where** it turns up decides everything:
//!
//! | Where the string still occurs | Verdict | Why |
//! |---|---|---|
//! | in a decoded **content-bearing** stream of the output — page content, a form XObject, a tiling pattern, a Type 3 glyph procedure | **REFUSE** — write nothing | These are the streams a renderer draws and a text extractor reads. A survival here means core's removal and core's report disagree; there is no reading of it under which the file is safe to hand over, and no acknowledgement checkbox makes it acceptable. |
//! | anywhere else in the output — in a decoded **opaque** stream (a font program, image samples, an ICC profile, an object-stream container, an attachment) or in the **raw bytes** | **DISCLOSE** as a residual requiring the operator's explicit acknowledgement, naming *where* it was found | pdfcer cannot tell a genuine un-recognised carrier from an unrelated coincidence. Refusing would be a trap the operator cannot act on; claiming removal would be a lie. Naming it, and naming the place, is the only honest option. |
//! | nowhere | **verified** | This is what licenses [`crate::text::redact`]'s wording contract to use the word "verified" at all. |
//!
//! Strings shorter than [`MIN_VERIFIABLE_LEN`] are excluded from the
//! **disclosure** half and **counted separately** (see
//! [`AbsenceVerification::strings_too_short_for_raw_check`]) rather than
//! silently skipped: a two-character redaction would match somewhere in any
//! real file, so a byte grep for it over raw bytes or over a compressed blob
//! carries no information, and pretending it does would turn the disclosure
//! into noise operators learn to click through. They are still checked against
//! **content-bearing** streams, where a survival *is* meaningful — which is
//! exactly what [`crate::text::redact::verification_limit_line`] has always
//! said on screen (*"those were checked against the decoded page content
//! only"*).
//!
//! ## ★★★ 2026-09-04 — the correction that made this proof usable at all
//!
//! Until this date the first row of that table read *"in a **decoded stream**
//! of the output"*, with no qualification, and [`decode_every_stream`] handed
//! it **every** stream in the file. The prose justifying that said *"A decoded
//! stream is content a renderer or a text extractor will read back"* — which is
//! true of a content stream and false of most of the streams in a real
//! document.
//!
//! The consequence was measured, not theorised. On this repository's own
//! `fixtures/a1-titleblock.pdf` — a drawing sheet of exactly the kind this
//! program is for — marking the word *construction* and applying produced:
//!
//! ```text
//! REFUSED: VerificationFailed { survivors: [" construction"] }
//! ```
//!
//! Nothing was written. The removal had in fact **succeeded**: 13 characters
//! deleted from 1 content stream, 1 mark applied, 0 retained. The byte run the
//! proof found was inside object 9 — a stream with `/Length1 19092` and no
//! `/Type`, i.e. an **embedded TrueType font program**, whose `name` table
//! carries the OpenType stylistic-set descriptions *"Classic construction"* and
//! *"Closed construction"*. A font's description of its own letterforms had
//! vetoed the operator's redaction.
//!
//! That is not a rare shape. **Every** PDF with an embedded font carries an
//! English-language `name` table, so every redaction of an ordinary English
//! word on an ordinary document was liable to be refused outright — which is
//! precisely what the operator reported on 2026-09-04: *"it always finds text
//! that wasn't redacted, and it always … counts everything I selected as
//! unredactable … What is the purpose of a redaction tool that refuses every
//! time to do any work?"*
//!
//! ★ **The classification was inverted.** The raw-byte half already had the
//! right instinct — a byte run in a place nothing draws is a coincidence pdfcer
//! cannot rule out, so *disclose* — and [`MIN_VERIFIABLE_LEN`] exists entirely
//! because of it. The decoded half applied the **opposite** rule to the **same**
//! kind of evidence: the identical coincidence, merely because it happened to
//! sit inside a Flate stream rather than beside one, earned the harshest verdict
//! in the module instead of the mildest.
//!
//! So the sweep still decodes every stream — narrowing *that* would hide
//! evidence — but each decoded blob is now classified by [`role_of`], and only
//! a **content-bearing** blob can produce a refusal. Everything else it finds is
//! promoted into the disclosure list with the place named, so nothing that used
//! to refuse now passes silently: it is reported, in the operator's face, and
//! gated behind the residual acknowledgement.
//!
//! ★★ What this deliberately does **not** relax: a survival in page content, a
//! form XObject (which is what an annotation appearance stream is), a tiling
//! pattern or a Type 3 glyph procedure is still a hard refusal that writes
//! nothing. Those are the streams that get drawn. The test
//! `the_sweep_reaches_a_stream_that_is_not_page_content` — which predates this
//! change and passes unaltered — is the one that holds that line.
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
//! `contains` — is carried verbatim, including the arguments for each.

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
/// ★ 2026-09-04: the floor now governs the whole disclosure half — raw bytes
/// **and** opaque decoded streams — rather than the raw bytes alone. Before
/// that date a two-character run inside a compressed font program was a hard
/// refusal, which is the same coincidence this constant exists to refuse to
/// draw conclusions from, merely wearing a `/FlateDecode`.
///
/// The count of strings this excludes is reported, never hidden — see
/// [`AbsenceVerification::strings_too_short_for_raw_check`], and
/// [`crate::text::redact::verification_limit_line`], which is the sentence that
/// puts the number in front of the operator.
pub const MIN_VERIFIABLE_LEN: usize = 4;

/// **Where a disclosed residual was found**, so the sentence about it can name
/// the place rather than say *"somewhere in the saved file"*.
///
/// ★ This exists because the disclosure it feeds has to be **actionable**. The
/// operator's complaint on 2026-09-04 was not only that the tool refused; it
/// was that what it reported was unusable — *"it always counts everything I
/// selected as unredactable"*. A residual an operator cannot place is a warning
/// they can only ignore, and a warning that is always ignored is worse than
/// none, because it also trains them to ignore the real one.
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
    /// An embedded font program — `/FontFile`, `/FontFile2` or `/FontFile3`.
    ///
    /// ★ By far the most common site, and the one that made this whole
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
///
/// See the module docs' 2026-09-04 section. The distinction is not cosmetic: it
/// is the difference between a refusal that writes nothing and a disclosure the
/// operator can act on, applied to byte-for-byte identical evidence.
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
    /// ★ 2026-09-04: renamed from `raw_byte_residuals` and widened. It used to
    /// hold *only* raw-byte hits, because every decoded hit had already been
    /// turned into a refusal further up. Now that an opaque decoded hit is a
    /// disclosure, the field has to be able to carry it — and the old name
    /// would have been a lie about half its contents, which on this screen is
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
///
/// Returns `Some(survivors)` when any redacted string is still present in a
/// **content-bearing** decoded stream — bytes a renderer draws and an extractor
/// reads — and `None` when the output is clean by that measure. Strings of
/// **any** length are checked here (unlike the disclosure half): inside a
/// content stream even a two-character survival is the redacted glyphs still
/// being drawn.
///
/// ★ 2026-09-04: the filter on [`StreamRole::Content`] is the whole of this
/// work's change to the refusal. Before it, this function saw every stream in
/// the file, and a font program's own description of its ligatures could and did
/// veto a completed redaction. See the module docs.
fn leaked_in_content_streams(
    redacted: &[String],
    decoded: &[DecodedStream],
) -> Option<Vec<String>> {
    if redacted.is_empty() {
        return None;
    }
    let survivors: Vec<String> = redacted
        .iter()
        .filter(|needle| !needle.is_empty() && in_content(decoded, needle))
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
///
/// `decoded` is passed in rather than computed because a residual is *"in no
/// content-bearing stream AND somewhere else"*, so both halves of the sweep are
/// needed to classify a single hit — and because [`prove`] already has them.
///
/// # The order of the four questions, which is the whole of the logic
///
/// 1. **Is it in a content-bearing stream?** Then it is not a residual at all —
///    it is a survivor, [`leaked_in_content_streams`] will refuse the write, and
///    listing it here as well would put one finding on screen twice under two
///    different verdicts.
/// 2. **Is it shorter than [`MIN_VERIFIABLE_LEN`]?** Then count it as
///    unverifiable and stop. It has already had the check that can say something
///    about it, at step 1.
/// 3. **Is it in an opaque decoded stream?** Disclose it, naming that stream's
///    kind. Checked before the raw bytes because the answer is more specific:
///    an uncompressed font program would satisfy both, and *"inside an embedded
///    font program"* tells the operator more than *"somewhere in the file"*.
/// 4. **Is it in the raw bytes?** Disclose it as [`ResidualSite::RawBytes`].
fn verify_absence(
    bytes: &[u8],
    redacted: &[String],
    decoded: &[DecodedStream],
) -> AbsenceVerification {
    let mut out = AbsenceVerification {
        strings_checked: redacted.iter().filter(|s| !s.is_empty()).count(),
        ..AbsenceVerification::default()
    };
    for needle in redacted {
        if needle.is_empty() || in_content(decoded, needle) {
            continue;
        }
        if needle.chars().count() < MIN_VERIFIABLE_LEN {
            out.strings_too_short_for_raw_check += 1;
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
///
/// A document that cannot be re-parsed yields an **empty** list rather than a
/// panic or an error. That looks like a false clean bill and is not, for a
/// reason worth stating plainly: these are bytes pdfcer itself just wrote, so an
/// unparsable output means a **writer** bug, and the raw-byte arm of the
/// disclosure still covers the whole buffer either way. A skip narrows the
/// evidence rather than fabricating it — and [`super::prepare_redaction_apply`]
/// is separately unable to produce such a buffer, because it re-parses the
/// output itself.
fn decoded_streams_of(bytes: &[u8]) -> Vec<DecodedStream> {
    Document::from_bytes(bytes.to_vec())
        .map(|doc| decode_every_stream(&doc))
        .unwrap_or_default()
}

/// Decode **every** stream in the document, not merely page content, and label
/// each with what it is.
///
/// The wide sweep is still the point, and it is unchanged: a redaction that only
/// *looked at* page content streams would say nothing about a form XObject, a
/// metadata stream, an embedded file, or an **object-stream container**, whose
/// compressed payload can carry a stale copy of a dictionary that was promoted
/// out of it (engine rule R38). Decoding the container like any other stream is
/// what lets a grep see that copy at all.
///
/// ★ What changed on 2026-09-04 is not the sweep but the **verdict** each blob
/// can produce. Every stream is still decoded and still searched; only a blob
/// [`role_of`] calls [`StreamRole::Content`] can refuse a write. The rest can
/// only disclose. See the module docs for the measurement that forced this.
///
/// Streams whose filters this build cannot decode are skipped rather than
/// failed: their *raw* bytes are still covered by the raw-byte arm of the
/// disclosure, so a skip narrows the evidence rather than fabricating it.
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
///
/// These two cannot be recognised from the stream's own dictionary — a page
/// content stream carries no `/Type` and no `/Subtype` at all (it is the
/// *emptiest* dictionary in the file, typically just `/Length`), and a Type 3
/// glyph procedure is the same shape. They have to be found from the other end,
/// by walking what refers to them.
///
/// ★ That asymmetry is why the classification is a whitelist reached two ways
/// rather than a blacklist of known-opaque kinds. A blacklist gets the default
/// wrong in the safe-looking direction and then has to be complete forever; this
/// gets the default wrong in the *disclosing* direction, where being wrong costs
/// the operator a sentence to read rather than a refused document.
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
///
/// The single decision the 2026-09-04 correction turns on. Getting it wrong in
/// one direction (calling a drawn stream opaque) would let a real leak be
/// disclosed instead of refused; getting it wrong in the other (calling an
/// opaque stream content) is the defect being fixed — a coincidence refusing a
/// completed redaction.
///
/// The three ways a stream is recognised as content:
///
/// 1. **by reference** — it is in some page's `/Contents`, or it is a Type 3
///    glyph procedure. See [`content_stream_ids`] for why these cannot be
///    recognised any other way.
/// 2. **`/Subtype /Form`** — a form XObject. This is also the shape of an
///    **annotation appearance stream**, which is why appearances need no case of
///    their own: a `/Widget`'s or a `/FreeText`'s `/AP` `/N` is a form XObject
///    and is caught here.
/// 3. **`/PatternType 1`** — a tiling pattern, whose stream is the content of
///    one cell, painted repeatedly.
///
/// Everything else is opaque, and the `site` it is given is what the operator
/// will be shown. The font-program test comes first among those because it is
/// the common case and because its markers are unambiguous: `/Length1` is
/// defined by ISO 32000-1 Table 127 as the length of an uncompressed **font
/// program**, and `/Subtype /Type1C`, `/CIDFontType0C` and `/OpenType` are the
/// three `/FontFile3` subtypes.
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
        assert!(proof.verification.residuals.is_empty());
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
    ///
    /// The extra stream is deliberately **not** referenced from the page: the
    /// question every test below asks is what [`role_of`] makes of a stream's
    /// own dictionary, and a reference would answer a different question.
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

    /// ★★★ **THE REGRESSION TEST. A removed word that also occurs inside an
    /// embedded font program is DISCLOSED, and the redaction goes ahead.**
    ///
    /// This is the defect the operator reported on 2026-09-04 —
    /// *"it refuses to redact anything because it always finds text that wasn't
    /// redacted"* — reduced to its smallest reproducing shape. Before that date
    /// this fixture produced `survivors: Some([…])`, which
    /// [`super::prepare_redaction_apply`] turns into
    /// [`super::RedactApplyRefusal::VerificationFailed`]: no file, no
    /// confirmation, no way forward.
    ///
    /// The needle is `construction` because that is the exact word that failed
    /// on `fixtures/a1-titleblock.pdf`, and the body it is hidden in is a
    /// three-word excerpt of the JetBrains Mono `name` table that caused it.
    ///
    /// ★ Three assertions, and each one closes a different way of "fixing" this
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

    /// ★ **Each opaque site is recognised and named**, so the sentence the
    /// operator reads is about the place the bytes actually are.
    ///
    /// One fixture per site rather than one assertion over a table: a table
    /// that got the *same* wrong answer for every row would still be internally
    /// consistent and would pass.
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

    /// ★★ **A survivor in drawn content is a refusal AND is not also a
    /// residual.**
    ///
    /// The two verdicts are mutually exclusive by construction, and the
    /// exclusion is load-bearing in both directions. Listed twice, one finding
    /// would appear on screen under two different verdicts — *"pdfcer refuses
    /// this"* and *"tick to proceed anyway"* — which is the one thing a report
    /// about a redaction must never say at once.
    #[test]
    fn a_survivor_in_drawn_content_is_not_also_listed_as_a_residual() {
        let bytes = pdf_drawing("KEEPTHISSECRET is drawn");
        let proof = prove(&bytes, &["KEEPTHISSECRET".to_owned()]);
        assert_eq!(
            proof.survivors,
            Some(vec!["KEEPTHISSECRET".to_owned()]),
            "the instrument must still see a string in plain sight"
        );
        assert!(
            proof.verification.residuals.is_empty(),
            "★ a refusal and a disclosure are different verdicts about \
             different evidence; one finding may not wear both"
        );
    }

    /// ★ **A tiling pattern and a Type 3 glyph procedure are drawn content.**
    ///
    /// Neither can be recognised the way a form XObject can. A tiling pattern's
    /// stream carries `/PatternType 1` and no `/Subtype`; a Type 3 glyph
    /// procedure carries **nothing at all** and is reachable only through its
    /// font's `/CharProcs`. Both paint glyphs, so a survival in either is the
    /// redacted content still on the page — and a whitelist that missed them
    /// would silently downgrade a real leak to a tick-box.
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
    ///
    /// The case that motivated `decode_every_stream`: a proof that only read
    /// `/Contents` would report this file clean while the string sat in a form
    /// XObject that every renderer draws.
    ///
    /// ★ This test predates the 2026-09-04 classification and passes unaltered.
    /// It is the one that holds the line the correction deliberately did not
    /// move: a form XObject — which is also the shape of every annotation
    /// appearance stream — is drawn, so a survival in one is still a refusal.
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
