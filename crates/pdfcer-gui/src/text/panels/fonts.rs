//! # `text::panels::fonts` — the Fonts panel's inventory report
//!
//! Every string the Fonts panel shows. Salvaged verbatim from the old
//! shell's `ui_text.rs`, with the doc comments that record why each is
//! worded as it is.
//!
//! ## What is here, and what deliberately is not
//!
//! The old panel had two halves: a **report** (what fonts this document
//! declares, what their embedded programs cost, and what would block
//! removing one) and two **destructive/constructive controls** (unembed a
//! font's program, embed a missing one). Only the report came across.
//!
//! That is not an oversight and it is not a snippet-salvage. Both controls
//! push a mutation through [`pdfcer_core::edit::EditSession`], and at stage
//! S3 [`crate::app::actions::Action`] has seven variants, all of which are
//! zoom or page navigation — there is no mutating action, no command log, no
//! undo, and `crate::app::state::OpenDoc::edit_epoch`'s own doc comment
//! names itself as the documented seam the *first* mutating arm must bump.
//! A control that cannot commit is an affordance for something that cannot
//! work, which `RIBBON_IA.md` P3 forbids by name.
//!
//! The report is where nearly all the panel's value is anyway, and the
//! reason is measured: from a 64-file survey of the PDFBox corpus, of the 30
//! files that embed fonts, **87 % embed subsets, 40 % use `Identity-H`, and
//! only 50 % carry `/ToUnicode`**. So the common case for "just remove the
//! embedded fonts" is a case where removal destroys the document, and the
//! operator has no way to know that from a font list alone. Telling them
//! *why* a font is refused is the panel's main reason to exist — and it does
//! that with no verb attached.
//!
//! ## ★ Why the panel says *why*, when the parity reference does not
//!
//! Acrobat refuses to unembed a font whose character codes are glyph indices
//! into its own embedded program, and it refuses **silently** — the font
//! simply is not in its unembed list, with no reason shown anywhere
//! (`Acrobat_Features/optimize__font_unembedding.md`, sourced to a former
//! Adobe Principal Scientist; independently corroborated by a user whose
//! largest, most size-costly font was absent from the list with no
//! explanation offered).
//!
//! A shorter list is not actionable. "This font's character codes are
//! positions inside this specific embedded program" is. That is project rule
//! 4 applied to a refusal rather than to a suggestion.
//!
//! ## ★ The verdict words are TWO WORDS EACH, and that is a measurement
//!
//! [`font_verdict_removable`] and its four siblings were sentences ("No
//! blocking condition found.", "Locked to this embedded program.") until a
//! screenshot of the running panel showed the row **clipped at the dock's
//! edge**: the byte size, the field an operator opens this panel for, was
//! cut to `59`. egui lays an over-wide row out anyway, off the edge of its
//! parent, and hands back a perfectly ordinary `Response` — so every
//! headless assertion was green while the row was unreadable. That is the
//! failure mode
//! `D:\dev\rag\egui\headless_trace_asserts_reached_not_visible_a_clipped_widget_needs_a_pixel_oracle.md`
//! records, reproduced exactly.
//!
//! The full sentences survive as the `font_reason_*` copy inside the
//! disclosed row, where there is width for them. Nothing was cut, only
//! moved.
//!
//! Field order on the row is **verdict, size, name last** — deliberately, so
//! that if a very long `/BaseFont` overflows anyway, what clips is the one
//! field recoverable from somewhere else ([`font_full_name_tooltip`] carries
//! the full name). See [`font_row_header`].
//!
//! ## Every verdict is drawn at the SAME visual weight
//!
//! As a plain label, never as a button and never in an error colour. Two
//! reasons, and both are requirements rather than taste:
//!
//! - A blocked verdict is a fact about the **file**, not a pdfcer failure, so
//!   it must not carry error styling.
//! - There is no removal control here. A "safe" verdict rendered as a
//!   checkmark or an accent colour reads as an invitation to click something
//!   that does not exist, which is worse than saying nothing.
//!
//! The wording is non-agentive throughout — no "cannot", no "failed" — for
//! the same reason.

/// Summary line above the list.
#[must_use]
pub fn fonts_count(total: usize) -> String {
    if total == 1 {
        "1 font.".to_owned()
    } else {
        format!("{total} fonts.")
    }
}

/// Shown when the document declares no fonts at all.
#[must_use]
pub fn fonts_none() -> &'static str {
    "This document declares no fonts."
}

/// The document-wide embedded-font total.
///
/// The equivalent of the parity reference's Audit Space Usage "Fonts"
/// bucket, which is a paid-tier feature there and gives no per-font
/// breakdown at all. Summed over DISTINCT font objects, so a font used on
/// four hundred pages is counted once.
#[must_use]
pub fn fonts_total_size(total: &str) -> String {
    format!("Embedded font data in this document: {total}.")
}

/// ★ The coverage disclosure, shown unconditionally above the list.
///
/// Not a caveat and not a footnote. An operator reading a font inventory to
/// decide what to delete needs the shape of the evidence, and "there is one
/// place pdfcer did not look" is part of the answer rather than a hedge on
/// it. A list that quietly missed a surface and looked complete is this
/// project's most-repeated defect shape.
///
/// Acrobat's own coverage here is recorded as an unconfirmed GAP, so pdfcer
/// states its own scope rather than assuming parity with a behaviour nobody
/// has measured.
#[must_use]
pub fn fonts_coverage_note() -> &'static str {
    "Covers fonts on each page (including inherited page resources, nested form objects, patterns and soft-mask groups), inside Type 3 fonts, in the interactive form's shared resources, and in every annotation's own appearance stream. It does NOT cover font objects that nothing in the document refers to — those still take up space in the file but do not appear here."
}

/// Shown when pdfcer could not walk the page tree at all.
///
/// Without this, "this document has no fonts" and "pdfcer could not look"
/// render identically, and an operator would read the second as the first.
/// It goes FIRST, above everything, because it changes what an empty list
/// beneath it means.
#[must_use]
pub fn fonts_page_scan_failed() -> &'static str {
    "pdfcer could not read this document's page tree, so no page's fonts are listed below. A short list here is not a statement about the document."
}

/// Shown when the resource sweep hit its ceiling.
#[must_use]
pub fn fonts_scan_truncated() -> &'static str {
    "This document nests resources more deeply than pdfcer follows, so some fonts may be missing from the list."
}

/// The end state: nothing is missing an embedded program.
///
/// ★ Deliberately NOT "ready to submit", "passes embedding checks", or
/// anything naming PDF/A or a print service. Those are claims about a third
/// party's acceptance that pdfcer has not verified. This states only what
/// pdfcer measured.
#[must_use]
pub fn fonts_all_embedded() -> &'static str {
    "Every font this document declares now has an embedded program."
}

/// How many of this document's fonts have no embedded program.
///
/// **New at salvage.** The old panel answered this only through a control —
/// the embed block's "n exact, n substitute" summary, which is a statement
/// about a *plan*, not about the document. With no embed control here, the
/// document-level fact would otherwise be recoverable only by opening every
/// row, and it is the fact that sends an operator to a print service's
/// rejection notice.
///
/// It states the count and nothing more. Naming a remedy pdfcer cannot
/// perform in this build would be the placeholder rule broken in prose
/// instead of in a widget.
#[must_use]
pub fn fonts_missing_programs(n: usize) -> String {
    if n == 1 {
        "1 of them has no embedded program; the document relies on the reader having a copy."
            .to_owned()
    } else {
        format!(
            "{n} of them have no embedded program; the document relies on the reader having a copy."
        )
    }
}

// ---------------------------------------------------------------------------
// Verdict summary words (row-level, always visible)
// ---------------------------------------------------------------------------

/// Verdict: nothing blocks removing this font's embedded program.
#[must_use]
pub fn font_verdict_removable() -> &'static str {
    "No blocker"
}

/// Verdict: the text is keyed to this exact embedded program.
#[must_use]
pub fn font_verdict_blocked_identity() -> &'static str {
    "Locked to program"
}

/// Verdict: a Type 3 font has no external equivalent.
#[must_use]
pub fn font_verdict_blocked_type3() -> &'static str {
    "No substitute"
}

/// Verdict: there is no embedded program at all.
///
/// A statement about the file rather than a verdict about removability —
/// there is nothing here to remove, so "safe" would be a misleading yes.
#[must_use]
pub fn font_verdict_not_embedded() -> &'static str {
    "Not embedded"
}

/// Verdict: pdfcer did not establish enough to classify this font.
///
/// "Unclassified" rather than the bare word "Unknown", because the `fsType`
/// line inside the same row independently reads as unknown for an unrelated
/// reason. Two bare "Unknown"s in one row look like one fact stated twice.
#[must_use]
pub fn font_verdict_unknown() -> &'static str {
    "Unclassified"
}

// ---------------------------------------------------------------------------
// Verdict reason sentences (disclosed row)
// ---------------------------------------------------------------------------

/// Reason for [`font_verdict_removable`].
#[must_use]
pub fn font_reason_removable() -> &'static str {
    "This font's character codes are standard, so another font could draw the same text."
}

/// ★ Reason for [`font_verdict_blocked_identity`] — the sentence this whole
/// panel exists to say.
///
/// Two tiers, because two independently-bad outcomes stack here and a
/// 64-file survey found them stacking on most real files: without the
/// embedded program the text cannot be DRAWN, and without a `/ToUnicode` map
/// it cannot be RECOVERED either. The parity reference refuses these fonts
/// too and shows no reason at all — it simply leaves them off its list.
#[must_use]
pub fn font_reason_blocked_identity(has_to_unicode: bool) -> String {
    let base = "This font uses Identity encoding: its character codes are positions inside this specific embedded program, not standard character codes. Removing the program would leave this text undrawable by any other font.";
    if has_to_unicode {
        format!(
            "{base} The font does carry a /ToUnicode map, so the underlying characters could still be extracted."
        )
    } else {
        format!(
            "{base} There is also no /ToUnicode map, so the characters themselves could not be recovered."
        )
    }
}

/// Reason for [`font_verdict_blocked_type3`].
#[must_use]
pub fn font_reason_blocked_type3() -> &'static str {
    "Type 3 glyphs are small drawing programs this document defines itself. There is no embedded font program to remove, and no installed font could stand in for them."
}

/// Reason for [`font_verdict_not_embedded`].
#[must_use]
pub fn font_reason_not_embedded() -> &'static str {
    "This font has no embedded program; the document already relies on the reader's own copy of it."
}

/// Reason: a symbolic font whose encoding lives inside its own program.
#[must_use]
pub fn font_reason_unknown_symbolic() -> &'static str {
    "This font declares no standard encoding and is marked symbolic, so what its character codes mean is defined inside the embedded program itself."
}

/// Reason: a composite font on a predefined, non-Identity CMap.
#[must_use]
pub fn font_reason_unknown_predefined_cmap() -> &'static str {
    "This font uses a predefined CMap for a named character collection. A font built for the same collection would work in its place; whether one is installed is not something this document can say."
}

/// Reason: a composite font whose CMap is an embedded stream.
#[must_use]
pub fn font_reason_unknown_embedded_cmap() -> &'static str {
    "This font's encoding is an embedded CMap that pdfcer does not interpret here, so what its character codes mean has not been established."
}

/// Reason: the declared program's bytes could not be read.
#[must_use]
pub fn font_reason_unknown_program_unreadable() -> &'static str {
    "This font declares an embedded program, but its bytes could not be read. The document is damaged in this respect."
}

/// Reason: a composite font with no usable descendant.
#[must_use]
pub fn font_reason_unknown_no_descendant() -> &'static str {
    "This is a composite font whose descendant font is missing or malformed, so it has no glyph source to classify."
}

/// Reason: no `/Subtype`, or one pdfcer does not model.
#[must_use]
pub fn font_reason_unknown_subtype() -> &'static str {
    "This font dictionary declares no type, or one pdfcer does not model, so how its character codes reach glyphs has not been established."
}

// ---------------------------------------------------------------------------
// fsType
//
// ★ CLAIM-BEARING COPY. Every sentence below says what the FONT VENDOR'S
// BITS ASSERT — never what "the licence permits". The OpenType specification
// is explicit that `fsType` is the vendor's machine-readable assertion of
// intent and not the licence itself, and that a face may permit more or less
// than its bits say. Saying "the licence permits" would be pdfcer making a
// legal claim about a document it has only read four bits of.
//
// ★ Four states, and **none of them may look like `0`.** `fsType == 0`
// genuinely *means* Installable — the most permissive value the field can
// express — so a blank, a dash, or an empty line for "we could not read it"
// would assert the broadest embedding right there is on the strength of
// bytes nobody read. The OpenType specification defines no default for the
// absent case, so pdfcer defines none either.
// ---------------------------------------------------------------------------

/// `fsType` usage value 0.
#[must_use]
pub fn font_fstype_installable(raw: u16) -> String {
    format!("Installable (0x{raw:04X}) — the font vendor's bits assert no embedding restriction.")
}

/// `fsType` usage value 2.
#[must_use]
pub fn font_fstype_restricted(raw: u16) -> String {
    format!(
        "Restricted License (0x{raw:04X}) — the font vendor's bits assert that embedding needs the legal owner's explicit permission."
    )
}

/// `fsType` usage value 4.
#[must_use]
pub fn font_fstype_preview_print(raw: u16) -> String {
    format!(
        "Preview & Print (0x{raw:04X}) — the font vendor's bits assert this program may be loaded for viewing or printing, not for editing."
    )
}

/// `fsType` usage value 8.
#[must_use]
pub fn font_fstype_editable(raw: u16) -> String {
    format!(
        "Editable (0x{raw:04X}) — the font vendor's bits assert this program may be loaded and its text edited."
    )
}

/// More than one usage bit set — a non-conforming combination from `OS/2`
/// version 3 onward, and pdfcer reports the ambiguity rather than resolving
/// it.
#[must_use]
pub fn font_fstype_ambiguous(raw: u16) -> String {
    format!(
        "Ambiguous (0x{raw:04X}) — more than one permission bit is set, which the font format does not allow. pdfcer does not pick one."
    )
}

/// Only the deprecated reserved bit 0 is set.
#[must_use]
pub fn font_fstype_unspecified(raw: u16) -> String {
    format!(
        "Unspecified (0x{raw:04X}) — only a bit the font format reserves is set. The format states no meaning for this, so pdfcer reads none."
    )
}

/// Bit 8, appended to the value sentence.
#[must_use]
pub fn font_fstype_no_subsetting() -> &'static str {
    "The bits also assert this font must not be subsetted before embedding."
}

/// Bit 9, appended to the value sentence.
#[must_use]
pub fn font_fstype_bitmap_only() -> &'static str {
    "The bits also assert only bitmap glyphs, not outlines, may be embedded."
}

/// Bits 8 and 9 were suppressed because the `OS/2` table predates them.
#[must_use]
pub fn font_fstype_version_gated() -> &'static str {
    "This font's OS/2 table is too old for the subsetting and bitmap bits to have had a meaning, so pdfcer ignores them as the format requires."
}

/// ★ `fsType` could not be read.
///
/// Must never be mistaken for value 0 — which genuinely means Installable,
/// the most permissive value the field can express. The word "unknown" is in
/// the sentence itself, not carried by styling, and the sentence says pdfcer
/// read nothing rather than implying it read a permissive value.
#[must_use]
pub fn font_fstype_unknown() -> &'static str {
    "Unknown — pdfcer could not read this font's embedding-permission bits. That is not the same as no restriction."
}

/// The program format has no `fsType` field at all.
///
/// Structurally different from "unknown": nothing failed, and there is
/// nothing to read. Type 1 and bare-CFF programs have no `OS/2` table.
#[must_use]
pub fn font_fstype_no_field() -> &'static str {
    "This font program's format has no embedding-permission field."
}

/// There is no embedded program, so there are no bits.
#[must_use]
pub fn font_fstype_not_embedded() -> &'static str {
    "No embedded program, so there are no embedding-permission bits."
}

// ---------------------------------------------------------------------------
// Row fields
// ---------------------------------------------------------------------------

/// A composite font's type, parent and descendant together.
///
/// `Type0 / CIDFontType2`. Both halves are shown because the parent alone
/// says nothing about the glyph source — the descendant is where the
/// outlines and the font descriptor actually live (§9.8.1).
#[must_use]
pub fn font_composite_type(parent: &str, descendant: &str) -> String {
    format!("{parent} / {descendant}")
}

/// A `/FontFile3` program's descriptor key together with its own `/Subtype`.
///
/// `FontFile3 (Type1C)`. The subtype is the part that says which format the
/// bytes are in, and for `/FontFile3` the key alone does not.
#[must_use]
pub fn font_program_key_with_subtype(key: &str, subtype: &str) -> String {
    format!("{key} ({subtype})")
}

/// The `/Subtype` line.
#[must_use]
pub fn font_type_line(kind: &str) -> String {
    format!("Type: {kind}")
}

/// The `/Encoding` line.
#[must_use]
pub fn font_encoding_line(encoding: &str) -> String {
    format!("Encoding: {encoding}")
}

/// How the program is embedded, and under which descriptor key.
#[must_use]
pub fn font_embedded_line(key: &str) -> String {
    format!("Embedded via {key}")
}

/// The program's byte size, rounded — and exact whenever rounding lost
/// anything.
///
/// The rounded figure is for ranking two hundred rows at a glance; the exact
/// figure is the measurement, and this project shows the measurement. Below
/// 1024 the two are the same number, and printing `474 B (474 bytes)` is
/// noise that teaches an operator to stop reading the parenthesis on the
/// rows where it carries information.
#[must_use]
pub fn font_size_line(rounded: &str, exact: usize) -> String {
    if exact < 1024 {
        format!("Size in file: {rounded}")
    } else {
        format!("Size in file: {rounded} ({exact} bytes)")
    }
}

/// The decoded program size, when it differs from the stored size.
///
/// Shown only when the two differ, which is when the program is compressed.
/// A line repeating the number above would be noise, and noise is how the
/// lines that matter get skimmed past.
#[must_use]
pub fn font_decoded_size_line(rounded: &str) -> String {
    format!("Uncompressed program: {rounded}")
}

/// `/ToUnicode` present.
#[must_use]
pub fn font_to_unicode_present() -> &'static str {
    "Has a /ToUnicode map (text can be extracted)."
}

/// `/ToUnicode` absent.
#[must_use]
pub fn font_to_unicode_absent() -> &'static str {
    "No /ToUnicode map."
}

/// Where the font is used, with the page list already range-collapsed.
#[must_use]
pub fn font_pages_line(ranges: &str, total: usize) -> String {
    if total == 1 {
        format!("Page {ranges}.")
    } else {
        format!("Pages {ranges} — {total} pages.")
    }
}

/// The font is reached from the interactive form's shared resources.
#[must_use]
pub fn font_found_in_form_resources() -> &'static str {
    "Also used by the interactive form's shared resources."
}

/// The font is reached from an annotation's appearance stream.
#[must_use]
pub fn font_found_in_annotation() -> &'static str {
    "Also used inside an annotation's appearance."
}

/// The font is reached from inside a Type 3 font's own resources.
#[must_use]
pub fn font_found_in_type3() -> &'static str {
    "Used inside a Type 3 font's glyph procedures."
}

/// A font that no page references — reached only from a shared surface.
///
/// **New at salvage.** `FontRecord::pages` being empty is NOT "unused" (the
/// core API map's trap T-9.4 says so explicitly: a font reached only through
/// the AcroForm `/DR` has no page list but is a live form-default font). The
/// old panel simply omitted the pages line in that case, which left an
/// operator to infer *"this font is on no page"* from an absence — and the
/// three "also used in…" lines below it are easy to miss.
///
/// Stated rather than inferred, because the inference is wrong.
#[must_use]
pub fn font_no_pages_line() -> &'static str {
    "No page references this font directly."
}

/// Tooltip on the display name, carrying the full `/BaseFont`.
///
/// The row shows the family name with the six-letter subset tag stripped,
/// because the tag reads as noise when scanning. But two independent subsets
/// of one face de-prefix to the SAME name, so back-to-back identical-looking
/// rows would read as a rendering fault rather than as the real and useful
/// fact that the document subsetted the face twice. The tag has to resurface
/// somewhere, and this is where.
#[must_use]
pub fn font_full_name_tooltip(full_base_font: &str) -> String {
    format!("Full name in the file: {full_base_font}")
}

/// A font dictionary with no `/BaseFont` at all.
#[must_use]
pub fn font_unnamed() -> &'static str {
    "(no name)"
}

/// The collapsed row: verdict, size, name.
///
/// Field order is the scanning order, and it is deliberate. The verdict
/// leads because it is the field an operator sweeping two hundred rows is
/// looking for and the one no other tool shows them; the size ranks it; the
/// name identifies it. Putting the name first would read better in isolation
/// and scan worse in bulk, which is the case that matters here.
///
/// The name is the DE-PREFIXED family name — the six-letter subset tag reads
/// as noise at a glance. It resurfaces in [`font_full_name_tooltip`], which
/// is what keeps two subsets of one face from looking like a duplicated row.
///
/// **The name is LAST, and that is the overflow decision.** A dock pane is
/// ~370 pt and a `/BaseFont` can be arbitrarily long; something has to be
/// allowed to clip. Putting the name last means what clips is the one field
/// an operator can recover from elsewhere (the tooltip), rather than the
/// byte size — which is what actually clipped when this row was first laid
/// out verdict-name-size.
#[must_use]
pub fn font_row_header(name: &str, size: &str, verdict: &str) -> String {
    format!("{verdict} · {size} · {name}")
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **The five verdict words are short enough to survive a dock.**
    ///
    /// This is the clipped-row incident turned into an assertion. The row is
    /// `verdict · size · name`, a dock pane is ~370 pt, and the byte size —
    /// the field the panel exists for — is the one that got cut when the
    /// verdicts were sentences. Twenty characters is a generous ceiling that
    /// still catches a sentence.
    ///
    /// A width in characters is a proxy for a width in points, and a poor
    /// one. It is used anyway because the honest measurement needs a live
    /// frame and a pixel oracle, and a proxy that fires on the regression
    /// that actually happened beats no check at all — the incident was
    /// sentences of 30 and 34 characters replacing words of 10 and 17.
    #[test]
    fn every_verdict_word_stays_short_enough_for_a_narrow_dock() {
        for v in [
            font_verdict_removable(),
            font_verdict_blocked_identity(),
            font_verdict_blocked_type3(),
            font_verdict_not_embedded(),
            font_verdict_unknown(),
        ] {
            assert!(
                v.chars().count() <= 20,
                "`{v}` is {} chars — long enough to push the byte size off the \
                 edge of a dock pane, which is the defect the short verdicts fixed",
                v.chars().count()
            );
            assert!(
                !v.ends_with('.'),
                "`{v}` is a label, not a sentence; the sentence is the font_reason_* copy"
            );
        }
    }

    /// **The five verdicts are five different words.**
    ///
    /// Two verdicts reading alike would collapse two different facts about a
    /// file into one, and the whole panel is an argument that the difference
    /// between them is what the operator needs.
    #[test]
    fn no_two_verdicts_say_the_same_thing() {
        let all = [
            font_verdict_removable(),
            font_verdict_blocked_identity(),
            font_verdict_blocked_type3(),
            font_verdict_not_embedded(),
            font_verdict_unknown(),
        ];
        for (i, a) in all.iter().enumerate() {
            for b in all.iter().skip(i + 1) {
                assert_ne!(a, b);
            }
        }
    }

    /// **`fsType` unknown must never read like `fsType` 0.**
    ///
    /// The single most dangerous confusion in this file: `0` means
    /// Installable — the *broadest* embedding right the field can express —
    /// so a blank or a dash for "we could not read the bits" would assert
    /// that right on the strength of bytes nobody read. Both failure states
    /// say the word "Unknown" in their own sentence, and neither is empty.
    #[test]
    fn an_unreadable_fstype_says_unknown_and_is_never_blank() {
        let unknown = font_fstype_unknown();
        assert!(unknown.contains("Unknown"));
        assert!(!unknown.trim().is_empty());
        assert!(
            unknown.contains("not the same as no restriction"),
            "the sentence must actively deny the permissive reading: {unknown}"
        );
        // "This format has no such field" is a THIRD state and must not be
        // confused with either of the other two.
        assert_ne!(font_fstype_no_field(), unknown);
        assert_ne!(font_fstype_not_embedded(), unknown);
        assert_ne!(font_fstype_not_embedded(), font_fstype_no_field());
    }

    /// **No `fsType` sentence claims to know a licence.**
    ///
    /// Claim-bearing copy. The OpenType specification is explicit that
    /// `fsType` is the vendor's machine-readable assertion of intent and not
    /// the licence, and that a face may permit more or less than its bits
    /// say. Every permission sentence must therefore attribute the claim to
    /// the bits, and none may use the word "licence"/"license" as a verb
    /// about what is permitted.
    ///
    /// `Restricted License` is the `fsType` value's own proper name, so the
    /// check is on the attribution clause rather than on the word.
    #[test]
    fn every_fstype_permission_attributes_the_claim_to_the_vendors_bits() {
        for s in [
            font_fstype_installable(0x0000),
            font_fstype_restricted(0x0002),
            font_fstype_preview_print(0x0004),
            font_fstype_editable(0x0008),
        ] {
            assert!(
                s.contains("the font vendor's bits assert"),
                "an fsType sentence that states a permission without attributing \
                 it to the vendor's bits is pdfcer making a legal claim: {s}"
            );
        }
        // The raw value is always printed, so an operator can check pdfcer's
        // reading against the font's own bytes.
        assert!(font_fstype_installable(0x000C).contains("0x000C"));
    }

    /// The Identity-encoding reason states the second, worse tier only when
    /// it is true.
    ///
    /// Two independently-bad outcomes stack: the text cannot be drawn, and
    /// without `/ToUnicode` it cannot be recovered either. Saying the second
    /// about a font that does carry the map would be a false alarm on the
    /// panel whose entire credibility rests on its refusals being accurate.
    #[test]
    fn the_identity_reason_distinguishes_having_a_tounicode_map_from_not() {
        let with = font_reason_blocked_identity(true);
        let without = font_reason_blocked_identity(false);
        assert_ne!(with, without);
        assert!(with.contains("could still be extracted"));
        assert!(without.contains("could not be recovered"));
        // Both carry the shared explanation of WHY it is blocked.
        for s in [&with, &without] {
            assert!(s.contains("Identity encoding"));
        }
    }

    /// The size line shows the exact count only when rounding lost
    /// something.
    #[test]
    fn the_size_line_repeats_the_exact_count_only_above_a_kilobyte() {
        assert_eq!(font_size_line("474 B", 474), "Size in file: 474 B");
        assert_eq!(
            font_size_line("1.5 KB", 1536),
            "Size in file: 1.5 KB (1536 bytes)"
        );
    }

    /// The row header puts the name last, and the tooltip is where the full
    /// name lives.
    ///
    /// This pins the *overflow decision*: whatever clips must be the field
    /// recoverable from elsewhere. A future reordering that reads better in
    /// isolation would silently reintroduce the clipped byte size.
    #[test]
    fn the_row_header_ends_with_the_name_so_the_size_cannot_clip() {
        let row = font_row_header("HelveticaNeue-CondensedBlack", "59.4 KB", "No blocker");
        assert!(row.starts_with("No blocker"));
        assert!(row.ends_with("HelveticaNeue-CondensedBlack"));
        assert!(
            row.find("59.4 KB") < row.find("HelveticaNeue"),
            "the byte size must precede the name, or a long /BaseFont pushes \
             it off the edge of the dock: {row}"
        );
        // And the un-de-prefixed name is recoverable.
        assert!(font_full_name_tooltip("ABCDEF+Helvetica").contains("ABCDEF+Helvetica"));
    }

    /// A font on no page says so rather than showing nothing.
    ///
    /// Core API trap T-9.4: an empty page list is not "unused". The absence
    /// of a pages line is exactly the inference that trap warns against.
    #[test]
    fn a_font_on_no_page_has_a_sentence_of_its_own() {
        assert!(!font_no_pages_line().is_empty());
        assert_ne!(font_no_pages_line(), font_pages_line("1", 1));
        assert!(font_pages_line("1", 1).starts_with("Page 1"));
        assert!(font_pages_line("1-3, 7", 4).starts_with("Pages 1-3, 7"));
    }
}
