//! # `text::panels::docprops` — the Document properties panel's copy
//!
//! Every string the **Document properties** panel says about the file itself:
//! the five `/Info` fields' labels, the seven read-only facts, and the two
//! disclosures a document can owe an operator — a value pdfcer could not decode
//! exactly, and an index pdfcer had to rebuild in order to open the file at all.
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/text/panels/docprops.md`.

/// The heading over the document's own facts and fields.
#[must_use]
pub const fn heading() -> &'static str {
    "This document"
}

/// What is editable here, and what an empty box means.
#[must_use]
pub const fn note() -> &'static str {
    "These are stored in the file and travel with it. Type to change one; \
     empty a box to remove it from the document altogether."
}

/// What the metadata rows cannot do yet, under them.
#[must_use]
pub const fn not_yet() -> &'static str {
    "Entries of other names and the dates cannot be added or changed here yet."
}

/// The label on one document-information field.
#[must_use]
pub fn info_label(field: pdfcer_core::edit::InfoField) -> &'static str {
    match field {
        pdfcer_core::edit::InfoField::Title => "Title",
        pdfcer_core::edit::InfoField::Author => "Author",
        pdfcer_core::edit::InfoField::Subject => "Subject",
        pdfcer_core::edit::InfoField::Keywords => "Keywords",
        pdfcer_core::edit::InfoField::Creator => "Application",
        // A field this build does not know a word for. `key()` is
        // `&'static [u8]` of ASCII, so the decode cannot fail — and it is
        // spelled `unwrap_or` rather than `expect` because a panic in a label
        // takes the window with it, and a blank box is a less bad outcome than
        // no window. // ui-text-exempt: the fallback is the engine's own PDF key, not authored copy
        _ => core::str::from_utf8(field.key()).unwrap_or(""),
    }
}

/// **A field was changed but the file's XMP copy of it was not**, so the two
/// now disagree; readers that prefer XMP keep showing the old value.
#[must_use]
pub fn xmp_skipped(field: &str, reason: pdfcer_core::edit::XmpSkip) -> String {
    use pdfcer_core::edit::XmpSkip;
    let why = match reason {
        XmpSkip::Off => "keeping that copy in step is switched off",
        XmpSkip::Signed => "the document is signed and rewriting it would break the signature",
        XmpSkip::PdfaCustomKey { .. } => {
            "the file claims PDF/A, which forbids an undeclared entry there"
        }
        XmpSkip::Unrepresentable => "the value holds a character that copy cannot store",
        XmpSkip::Unreadable => "that copy could not be read",
        _ => "pdfcer left it alone",
    };
    format!(
        "{field} was changed, but the file's second (XMP) copy of it was not, because \
         {why}. Some programs read that copy and will still show the old value."
    )
}

/// **The value shown is pdfcer's reading of bytes it could not fully
/// decode.**
#[must_use]
pub const fn info_not_exact() -> &'static str {
    "Some characters in this value could not be read with certainty and are \
     shown as substitutes. Leave the box alone and the file keeps its own \
     bytes; type in it and what you type replaces them."
}

/// The label on the file row.
#[must_use]
pub const fn file_label() -> &'static str {
    "File"
}

/// A document that has never been written to disk.
#[must_use]
pub const fn file_unsaved() -> &'static str {
    "not saved to a file yet"
}

/// The label on the size row.
#[must_use]
pub const fn size_label() -> &'static str {
    "Size on disk"
}

/// The size shown is the file as it was OPENED, not as it would be saved.
#[must_use]
pub const fn size_is_base() -> &'static str {
    "This is the file as it was opened. Your unsaved changes are not counted."
}

/// The label on the PDF-version row.
#[must_use]
pub const fn version_label() -> &'static str {
    "PDF version"
}

/// The label on the page-count row.
#[must_use]
pub const fn pages_label() -> &'static str {
    "Pages"
}

/// The label on the sheet-size row.
#[must_use]
pub const fn page_size_label() -> &'static str {
    "Sheet size"
}

/// One sheet size, in millimetres, from a size given in **points**.
#[must_use]
pub fn page_size(width_pts: f64, height_pts: f64) -> String {
    let width_mm = crate::units::whole_mm_from_points(width_pts);
    let height_mm = crate::units::whole_mm_from_points(height_pts);
    format!("{width_mm} × {height_mm} mm")
}

/// A document whose sheets are not all the same size.
#[must_use]
pub fn page_size_mixed(width_pts: f64, height_pts: f64) -> String {
    let width_mm = crate::units::whole_mm_from_points(width_pts);
    let height_mm = crate::units::whole_mm_from_points(height_pts);
    format!("mixed — page 1 is {width_mm} × {height_mm} mm")
}

/// The label on the encryption row.
#[must_use]
pub const fn encryption_label() -> &'static str {
    "Encryption"
}

/// An encrypted document.
#[must_use]
pub const fn encrypted() -> &'static str {
    "Encrypted"
}

/// An unencrypted document.
#[must_use]
pub const fn not_encrypted() -> &'static str {
    "Not encrypted"
}

/// What pdfcer does NOT tell you about an encrypted document.
#[must_use]
pub const fn encryption_note() -> &'static str {
    "pdfcer opened it, so it could read it. This panel does not report what the \
     encryption permits — printing, copying, changing — and an encrypted \
     document may restrict any of them."
}

/// Heading for the recovered-file disclosure.
#[must_use]
pub const fn recovered_heading() -> &'static str {
    "This file's index was damaged, and pdfcer rebuilt it to open it"
}

/// The detail line: what the rebuild involved.
#[must_use]
pub fn recovered_detail(objects: usize, collisions: usize, repaired: usize) -> String {
    format!(
        "{objects} objects were recovered by scanning the file. {collisions} were defined more than once, so pdfcer chose one of each. {repaired} needed repairing."
    )
}

/// The hover explanation.
#[must_use]
pub const fn recovered_tooltip() -> &'static str {
    "Every PDF carries an index saying where its contents are. This one's was wrong or missing — usually an interrupted download, a crashed writer, or a tool that appended to it badly — so pdfcer scanned the whole file and rebuilt the index from what it found. The document opens and prints normally. Where something was defined more than once pdfcer had to pick one, so if anything looks out of place, check it against the original before relying on it."
}

/// **How many places the scan could not read, split by which kind of
/// unreadable they were.**
#[must_use]
pub fn dropped_summary(unparseable: usize, id_mismatch: usize) -> String {
    let mut out = String::new();
    if unparseable > 0 {
        out.push_str(&format!(
            "{unparseable} more looked like the start of an object but could not be read. That is usually harmless — compressed picture and drawing data can contain bytes that look like an object heading — but if something is missing from this document, these are where it went."
        ));
    }
    if id_mismatch > 0 {
        if !out.is_empty() {
            out.push(' ');
        }
        out.push_str(&format!(
            "{id_mismatch} were found in a place that disagreed with their own numbering, so pdfcer left them out rather than trust them."
        ));
    }
    out
}

/// **The object numbers themselves, elided at a fixed count.**
#[must_use]
pub fn dropped_numbers(numbers: &[u32], limit: usize) -> String {
    let shown: Vec<String> = numbers.iter().take(limit).map(u32::to_string).collect();
    let rest = numbers.len().saturating_sub(shown.len());
    if rest == 0 {
        format!("Objects: {}.", shown.join(", "))
    } else {
        format!("Objects: {}, and {rest} more.", shown.join(", "))
    }
}

/// The hover explanation for the dropped-object lines.
#[must_use]
pub const fn dropped_tooltip() -> &'static str {
    "When pdfcer rebuilds a damaged index it scans the whole file for anything that looks like the start of an object. Some of what it finds cannot be read back. Most of those are not really objects — they are ordinary compressed data that happens to look like one — and nothing is lost. Occasionally one is real, and then a piece of the document is genuinely gone: a missing drawing, a blank page, an annotation that is not there any more. Compare the pages against the original if you can."
}

/// The four page boxes, in the order `panels::docprops::boxes` walks them.
const PAGE_BOXES: [&str; 4] = ["crop box", "bleed box", "trim box", "art box"];

/// What Table 30 uses in place of each of [`PAGE_BOXES`] when it is unusable.
const PAGE_BOX_DEFAULTS: [&str; 4] = [
    "the whole sheet",
    "the crop box",
    "the crop box",
    "the crop box",
];

/// Box `which` (0 crop, 1 bleed, 2 trim, 3 art) runs past the sheet's edge
/// on `pages` and is read as the part on the sheet.
#[must_use]
pub fn page_box_clipped(which: usize, pages: &str) -> String {
    let name = PAGE_BOXES[which];
    format!(
        "The {name} on page(s) {pages} runs past the edge of the sheet; pdfcer uses the part that is on the sheet. The file is not changed."
    )
}

/// Box `which` on `pages` misses the sheet or is malformed and its default
/// stands in.
#[must_use]
pub fn page_box_unusable(which: usize, pages: &str) -> String {
    let (name, instead) = (PAGE_BOXES[which], PAGE_BOX_DEFAULTS[which]);
    format!(
        "The {name} on page(s) {pages} misses the sheet or is not a rectangle, so pdfcer uses {instead} in its place. The file is not changed."
    )
}

/// The label on the PDF/X claim row.
#[must_use]
pub const fn pdfx_label() -> &'static str {
    "PDF/X"
}

/// The level the file claims, verbatim.
#[must_use]
pub fn pdfx_level(level: &str) -> String {
    format!("claims {level}")
}

/// What the claim is and is not.
#[must_use]
pub const fn pdfx_claim_only() -> &'static str {
    "The file says this about itself; pdfcer has not checked that it conforms."
}

/// The claimed level is not one the PDF/X registry lists.
#[must_use]
pub const fn pdfx_unregistered() -> &'static str {
    "That is not a registered PDF/X level; it is shown as the file wrote it."
}

/// The XMP and Info dictionary claim different levels.
#[must_use]
pub fn pdfx_disagree(xmp: &str, info: &str) -> String {
    format!(
        "The file's two claims differ: its XMP metadata says {xmp} and its document information says {info}. The XMP one is shown."
    )
}

/// The press conditions the file names in its PDF/X output intents.
#[must_use]
pub fn pdfx_intents(names: &[&str]) -> String {
    format!("Printing condition: {}.", names.join(", "))
}

/// No `/GTS_PDFX` output intent, which every PDF/X part requires.
#[must_use]
pub const fn pdfx_no_intent() -> &'static str {
    "It names no printing condition (output intent), which every PDF/X level requires."
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **The two drop reasons never merge into one sentence.**
    #[test]
    fn the_two_drop_reasons_are_never_collapsed() {
        let only_unreadable = dropped_summary(3, 0);
        let only_mismatch = dropped_summary(0, 2);
        let both = dropped_summary(3, 2);

        assert!(
            only_unreadable.contains('3') && !only_unreadable.contains("numbering"),
            "three false positives were described as a numbering disagreement: {only_unreadable}"
        );
        assert!(
            only_mismatch.contains("numbering") && !only_mismatch.contains("harmless"),
            "two untrustworthy definitions were called harmless: {only_mismatch}"
        );
        assert!(
            both.contains("harmless") && both.contains("numbering"),
            "a file with both kinds must say both: {both}"
        );
        assert!(
            dropped_summary(0, 0).is_empty(),
            "nothing was dropped, so there is no sentence — R9, not a reassuring zero"
        );
    }

    /// **An elided list says how many it did not print.**
    #[test]
    fn an_elided_list_of_dropped_objects_counts_what_it_left_out() {
        let few: Vec<u32> = (1..=3).collect();
        let line = dropped_numbers(&few, 12);
        assert!(line.contains('1') && line.contains('3'), "{line}");
        assert!(
            !line.contains("more"),
            "three of three were printed and it still claimed a remainder: {line}"
        );

        let exactly: Vec<u32> = (1..=12).collect();
        assert!(
            !dropped_numbers(&exactly, 12).contains("more"),
            "twelve of twelve printed is not an elision"
        );

        let many: Vec<u32> = (1..=40).collect();
        let elided = dropped_numbers(&many, 12);
        assert!(
            elided.contains("28 more"),
            "40 objects with 12 shown leaves 28; the count must be stated or the line reads as the whole list: {elided}"
        );
    }

    /// `Creator` is labelled as Acrobat labels it, not by its PDF key.
    #[test]
    fn the_creator_is_labelled_application() {
        assert_eq!(
            info_label(pdfcer_core::edit::InfoField::Creator),
            "Application"
        );
    }

    /// **The document heading does not repeat the tab's own label.**
    #[test]
    fn the_heading_is_not_the_tabs_name_again() {
        let tab = crate::text::commands::file_document_properties().label;
        assert_ne!(
            heading(),
            tab,
            "the heading repeats the tab's own label; see this function's doc"
        );
    }

    /// Each reason the XMP copy was left alone gets its own sentence, naming
    /// the field that now disagrees.
    #[test]
    fn every_xmp_skip_reason_reads_differently() {
        use pdfcer_core::edit::XmpSkip;
        let reasons = [
            XmpSkip::Off,
            XmpSkip::Signed,
            XmpSkip::PdfaCustomKey { part: 2 },
            XmpSkip::Unrepresentable,
            XmpSkip::Unreadable,
        ];
        let sentences: std::collections::BTreeSet<String> =
            reasons.iter().map(|r| xmp_skipped("Title", *r)).collect();
        assert_eq!(sentences.len(), reasons.len());
        assert!(sentences.iter().all(|s| s.starts_with("Title ")));
    }

    /// The two disclosures are sentences about the **document**, not about
    /// pdfcer failing.
    #[test]
    fn the_disclosures_describe_the_file() {
        for sentence in [info_not_exact(), recovered_tooltip(), encryption_note()] {
            assert!(!sentence.trim().is_empty());
            assert!(
                !sentence.starts_with("pdfcer could not"),
                "a disclosure that opens by blaming the program: {sentence}"
            );
        }
    }
}
