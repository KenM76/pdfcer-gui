//! # `text::panels::docprops` — the Document properties panel's copy
//!
//! Every string the **Document properties** panel says about the file itself:
//! the four `/Info` fields' labels, the seven read-only facts, and the two
//! disclosures a document can owe an operator — a value pdfcer could not decode
//! exactly, and an index pdfcer had to rebuild in order to open the file at all.
//!
//! ## Why this is its own module, and it is two reasons rather than one
//!
//! **1. The panel is its own panel** — the operator, 2026-09-05: *"the document
//! properties are still always visible in the properties tab. it needs to get
//! out of there and be in its own document properties tab."* These strings moved
//! out of [`super::properties`] with the section they belong to, in the same
//! commit, because copy that lives in the catalog of a surface it is no longer
//! drawn on is copy nobody finds when they come to change it. Every one of them
//! was reached from exactly one file before the move and from exactly one file
//! after it.
//!
//! **2. R2, measured rather than anticipated.** `text/panels/properties.rs`
//! stood at **1,469 lines against the 1,500-line ceiling** on the day of the
//! move — its own `textobject` module records having been split off at 1,446 for
//! the same reason. So the move is also the split that gate was going to force
//! within a couple of sentences, and it is a split along a subject boundary
//! rather than an arithmetic one: what remains in `properties` describes **what
//! is selected**; what is here describes **the file**.
//!
//! ## ★ The names lost their `properties_` prefix, deliberately
//!
//! Every function here was `properties_something` and is now `something`. A
//! prefix naming the surface a string used to be drawn on is exactly the stale
//! reasoning this project keeps finding in its own prose, in the one place a
//! reader cannot argue with it — the identifier. Called as
//! `t::heading()`, `t::size_is_base()`, `t::encryption_note()` from
//! [`crate::panels::docprops`], which is the only caller of any of them.
//!
//! ★ The three `recovered_*` functions kept their names: they were never
//! prefixed, they name the *event* rather than the surface, and renaming them
//! would have been churn with no reader served.
//!
//! ## What is NOT here
//!
//! The empty state. A panel with no document open never draws its own body —
//! [`crate::panels::Panel::show`] answers that case once for all twelve panels
//! with [`super::panel_no_document`] — so there is no "open a document first"
//! sentence in this file to drift from the other eleven.

/// The heading over the document's own facts and fields.
///
/// ★ **"This document" rather than "Document properties"**, and the reason
/// survived the move to a panel that is *called* Document properties. The tab
/// says what the surface is; the heading says what the words under it are
/// about. Repeating the tab's own label immediately below it is a line of
/// screen that tells the operator nothing they did not learn by clicking.
///
/// It also reads as the answer to a question — *which document?* — which is the
/// live one with several documents open.
#[must_use]
pub const fn heading() -> &'static str {
    "This document"
}

/// What is editable here, and what an empty box means.
///
/// ★ The second sentence is the one that could not be guessed. Clearing a box
/// **removes the key from the file** rather than storing an empty string —
/// `set_info_field(field, None)` against `Some("")` — and they are different
/// documents. It is also the only action in this panel that removes anything,
/// which is why it is stated at the top rather than left to be discovered.
#[must_use]
pub const fn note() -> &'static str {
    "These are stored in the file and travel with it. Type to change one; \
     empty a box to remove it from the document altogether."
}

/// The label on one document-information field.
///
/// Takes the engine's `InfoField` rather than a string, so the panel
/// enumerates `InfoField::all()` and this maps the result — which is the
/// discipline that enum's own doc comment asks for: *"so a front end
/// enumerates the real list instead of hard-coding one that drifts when a
/// field is added."*
///
/// The words are the PDF spec's own field names in ordinary English. "Subject"
/// and "Keywords" are not obvious, and neither is improved by inventing
/// something friendlier: an operator who has met these fields in any other PDF
/// tool has met them under these names, and a novel word would be a novel
/// thing to learn for no gain.
///
/// # ★ `InfoField` is `#[non_exhaustive]`, so the compiler CANNOT catch a new
/// field here
///
/// This function was first written as a `const fn` with four arms and no
/// wildcard, on the assumption that a fifth variant would break the build. It
/// would not: `#[non_exhaustive]` forces a downstream crate to write `_`, and
/// with a `_` arm the match compiles for ever no matter what the engine adds.
/// The protection people expect from an exhaustive match is **not available
/// across a crate boundary** when the enum is marked that way, and assuming it
/// is available is how a new field ends up silently unlabelled.
///
/// So the safety is built another way instead:
///
/// **The fallback is the field's own PDF key**, taken from `InfoField::key()` —
/// the engine's answer, not a guess. A field added upstream appears in the
/// panel labelled `Producer` or `Creator` rather than disappearing or reading
/// "Unknown". Imperfect English, correct, and reachable, which beats all three
/// alternatives.
///
/// # ⚠ CORRECTED 2026-09-05 — the second safeguard was claimed and does not
/// exist, and it **cannot**
///
/// This doc used to end with a second numbered point:
///
/// > **A test asserts none of the four known fields reaches the fallback.**
/// > That is the alarm the compiler cannot raise: if the mapping is ever broken
/// > the four named fields start rendering as their raw keys, and the test says
/// > which.
///
/// No such test was ever written, and one was attempted while moving this
/// function into its own module. It fails on the first field it looks at, for a
/// reason that is not a defect: **`InfoField::Title`'s PDF key IS `Title`.** So
/// is `Author`'s, `Subject`'s and `Keywords`'s. The mapping and the fallback
/// return byte-identical strings for all four, which means a deleted arm
/// changes nothing an operator or a test could see.
///
/// ⇒ The arms are not a safety net for these four; they are a **place to put a
/// better word** when the engine adds a field whose key is not already English
/// (`Producer`, `CreationDate`, `Trapped`). That is a real job and the
/// wildcard is a real floor under it. What is gone is the false comfort of a
/// test nobody could have written — and the shape is the one this project keeps
/// finding: *a safeguard described in prose is not a safeguard*, and the only
/// way to tell is to try to write it.
///
/// What IS asserted, in [`crate::panels::docprops`]'s own tests: every field
/// `InfoField::all()` returns has a non-empty label and no two share one. That
/// catches the failure that can actually reach the operator — two boxes they
/// cannot tell apart, or an unlabelled one.
#[must_use]
pub fn info_label(field: pdfcer_core::edit::InfoField) -> &'static str {
    match field {
        pdfcer_core::edit::InfoField::Title => "Title",
        pdfcer_core::edit::InfoField::Author => "Author",
        pdfcer_core::edit::InfoField::Subject => "Subject",
        pdfcer_core::edit::InfoField::Keywords => "Keywords",
        // A field this build does not know a word for. `key()` is
        // `&'static [u8]` of ASCII, so the decode cannot fail — and it is
        // spelled `unwrap_or` rather than `expect` because a panic in a label
        // takes the window with it, and a blank box is a less bad outcome than
        // no window. // ui-text-exempt: the fallback is the engine's own PDF key, not authored copy
        _ => core::str::from_utf8(field.key()).unwrap_or(""),
    }
}

/// ★ **The value shown is pdfcer's reading of bytes it could not fully
/// decode.**
///
/// Drawn under a field whose `InfoText::exact` is `false`. That flag means
/// re-encoding the displayed text would **not** reproduce the file's own
/// bytes, so what is on screen is a rendering with substitutions in it, not a
/// copy.
///
/// This is rule 4's surviving half in its purest form: an inference the
/// operator **cannot see**. A replacement character in a metadata field looks
/// like a character rather than like a gap, and without this sentence the
/// operator's only clue that pdfcer is guessing would be a glyph they might
/// read as the document's own.
///
/// Worded as a fact about the **document**, not as a pdfcer failure — the file
/// really does carry bytes in an encoding it does not declare well enough to
/// resolve — and it says what to do about it, which is the part that makes it
/// actionable rather than alarming: leaving it alone is safe.
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
///
/// `OpenDoc::has_file` is `false` for a document `file.new` created, and its
/// `path` in that state is a *name*, not a location. Showing the name as
/// though it were a file would tell the operator their work is somewhere it is
/// not.
#[must_use]
pub const fn file_unsaved() -> &'static str {
    "not saved to a file yet"
}

/// The label on the size row.
#[must_use]
pub const fn size_label() -> &'static str {
    "Size on disk"
}

/// ★ The size shown is the file as it was OPENED, not as it would be saved.
///
/// `Document::bytes()` is documented as *"the base revision, not the edited
/// state"*, so with unsaved edits this number is the file on disk and not what
/// a save would produce. Shown only while edits are pending, because on an
/// unedited document the two are the same and the sentence would be noise that
/// trains the operator to skip it.
///
/// "Size on disk" rather than "File size" for the same reason: the label
/// itself carries most of the distinction, and the sentence carries the rest
/// when it matters.
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

/// One sheet size, in millimetres.
///
/// Millimetres rather than points, because a drafter knows an A3 by
/// `420 × 297` and nobody's intuition is in 72nds of an inch. The page tile's
/// tooltip made the same choice for the same reason.
#[must_use]
pub fn page_size(width_mm: f32, height_mm: f32) -> String {
    format!("{width_mm:.0} × {height_mm:.0} mm")
}

/// ★ A document whose sheets are not all the same size.
///
/// **The common case for this operator**, not an edge case: a drawing set is
/// an A1 general arrangement with A3 details behind it. Reporting page one's
/// size alone would be a true number that reads as a claim about the document,
/// so the mixed case says so and gives the first sheet's size as an example
/// rather than as the answer.
#[must_use]
pub fn page_size_mixed(width_mm: f32, height_mm: f32) -> String {
    format!("mixed — page 1 is {width_mm:.0} × {height_mm:.0} mm")
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
///
/// ★ Stated rather than left blank. This panel's posture is that its silences
/// must be as legible as its numbers, and an absent encryption row is
/// indistinguishable from a panel that does not check — which on this
/// particular question is exactly the wrong impression to leave.
#[must_use]
pub const fn not_encrypted() -> &'static str {
    "Not encrypted"
}

/// ★ What pdfcer does NOT tell you about an encrypted document.
///
/// The Signatures panel's discipline applied here: *say what you cannot tell
/// you, first*. A row reading "Encrypted" invites the operator to conclude
/// something about what the document permits, and pdfcer reports nothing about
/// permissions in this panel.
///
/// The reason is in `pdfcer-core`'s own `DocumentEncryption::perms` doc: `/Perms`
/// is *"the only integrity check in PDF encryption"*, it is a `should` rather
/// than a `shall`, and it is `NotApplicable` for every `/R` ≤ 4 document —
/// which is *"the ordinary answer, not a failed check, and a front end must
/// not render it as one."* Reporting permissions properly means reporting that
/// distinction properly, and this build does not.
#[must_use]
pub const fn encryption_note() -> &'static str {
    "pdfcer opened it, so it could read it. This panel does not report what the \
     encryption permits — printing, copying, changing — and an encrypted \
     document may restrict any of them."
}

/// Heading for the recovered-file disclosure.
///
/// ★ Plain, and about the FILE rather than about pdfcer. "pdfcer had to repair
/// this file" would read as pdfcer struggling; the file is the thing that is
/// damaged, and the operator's next question is about the file.
#[must_use]
pub const fn recovered_heading() -> &'static str {
    "This file's index was damaged, and pdfcer rebuilt it to open it"
}

/// The detail line: what the rebuild involved.
///
/// ★★ Three numbers, and only the middle one is a warning. Objects recovered
/// says how big the job was; **objects defined more than once** is the one that
/// can put a line in the wrong place, because pdfcer had to choose; and repaired
/// says how much else needed inference. Naming them separately lets an operator
/// tell "large but clean" from "small and ambiguous", which a single "recovered
/// N objects" cannot.
#[must_use]
pub fn recovered_detail(objects: usize, collisions: usize, repaired: usize) -> String {
    format!(
        "{objects} objects were recovered by scanning the file. {collisions} were defined more than once, so pdfcer chose one of each. {repaired} needed repairing."
    )
}

/// The hover explanation.
///
/// ★★★ States the consequence in the operator's terms and stops. It does not
/// tell them to do anything, because there is nothing reliable to tell them:
/// the file may be perfectly fine, and the only real remedy is a good copy from
/// whoever produced it. Inventing an action would be worse than naming the
/// uncertainty.
#[must_use]
pub const fn recovered_tooltip() -> &'static str {
    "Every PDF carries an index saying where its contents are. This one's was wrong or missing — usually an interrupted download, a crashed writer, or a tool that appended to it badly — so pdfcer scanned the whole file and rebuilt the index from what it found. The document opens and prints normally. Where something was defined more than once pdfcer had to pick one, so if anything looks out of place, check it against the original before relying on it."
}

#[cfg(test)]
mod tests {
    use super::*;

    /// ⚠ **The test this module's own doc claimed, written, run, and DELETED —
    /// 2026-09-05.**
    ///
    /// [`info_label`]'s doc promised *"a test asserts none of the four known
    /// fields reaches the fallback"*. It was written while this module was
    /// being split out, in the form the promise implies:
    ///
    /// ```text
    /// assert_ne!(info_label(field), from_utf8(field.key()));
    /// ```
    ///
    /// It failed on the first field, and the message is the whole finding:
    ///
    /// ```text
    /// Title fell through to the engine's own key `Title`
    ///   left: "Title"   right: "Title"
    /// ```
    ///
    /// **`InfoField::Title`'s PDF key is the word `Title`.** The mapping and
    /// the fallback are byte-identical for all four known fields, so *"did this
    /// field reach the fallback?"* has no observable answer and the promised
    /// test cannot exist in any form. The doc is corrected in place rather than
    /// quietly satisfied by a weaker assertion wearing the same name — which is
    /// what a test asserting the four literals would have been.
    ///
    /// This stub is left as the record, because a deleted test leaves nothing
    /// behind and the next reader of that doc comment would try the same thing.
    ///
    /// What replaces it is in [`crate::panels::docprops`]:
    /// `every_info_field_is_labelled_and_no_label_repeats`, which asserts the
    /// property that can actually reach the operator.
    #[test]
    fn the_fallback_and_the_mapping_are_indistinguishable_for_every_known_field() {
        for field in pdfcer_core::edit::InfoField::all() {
            let key = core::str::from_utf8(field.key()).unwrap_or("");
            assert_eq!(
                info_label(field),
                key,
                "{field:?} now has a label that differs from its PDF key. That is \
                 not a failure — it is the case the `match` arms exist FOR, and \
                 it means a fallback test is finally possible for this field. \
                 Read this test's doc comment before changing it."
            );
        }
    }

    /// **The document heading does not repeat the tab's own label.**
    ///
    /// The panel's tab is called *Document properties* — it takes its name from
    /// `file.document_properties`' label, which is how every dock tab in this
    /// build is named. A heading reading the same words immediately under it is
    /// a line of an inspector that tells the operator nothing they did not
    /// learn by clicking, and it is the obvious thing for a later edit to
    /// "tidy" the heading into.
    #[test]
    fn the_heading_is_not_the_tabs_name_again() {
        let tab = crate::text::commands::file_document_properties().label;
        assert_ne!(
            heading(),
            tab,
            "the heading repeats the tab's own label; see this function's doc"
        );
    }

    /// The two disclosures are sentences about the **document**, not about
    /// pdfcer failing.
    ///
    /// Both are drawn at ordinary weight beside facts, and the wording is what
    /// carries the distinction — a sentence that opened "pdfcer could not…"
    /// would read as a defect report about the program in a panel whose whole
    /// subject is the operator's file.
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
