//! # `panels::signatures` — what each digital signature covers
//!
//! Salvaged from the old shell's `panels_structure.rs`. The body is
//! unchanged in substance; what changed is that it is a free function over
//! `&OpenDoc` rather than a method on the whole application, and that the
//! panel now says out loud which state of the file it measured.
//!
//! # It is not a validity check, and the panel says so first
//!
//! pdfcer performs no cryptographic verification. A panel headed
//! "Signatures", listing byte counts, is the single most likely place in
//! this application for an operator to take away more than was said — so the
//! caveat is the **first line, above the list**, not a tooltip and not a
//! footnote. A caveat below a list arrives after the reader has already
//! drawn a conclusion.
//!
//! The ribbon command's own tooltip says the same thing
//! (`view.panel_signatures`: *"Show what each digital signature covers.
//! pdfcer does not check whether they are valid."*), because a control
//! labelled "Signatures" invites the assumption before the panel is even
//! open.
//!
//! # The file length is read from DISK each time, deliberately
//!
//! `/ByteRange` is a claim about bytes, so it can only be checked against
//! bytes — which is why
//! [`pdfcer_core::signature::byte_range_coverage`] takes the length as a
//! parameter rather than reading it from the object graph: *"the object
//! model cannot check a claim about bytes against itself."*
//!
//! The length used is the file **on disk right now**, not a length captured
//! when the document was opened, and that choice changes what the numbers
//! mean. It answers *"does the signature cover the file as it currently
//! exists"*, which is the question worth asking. A captured length would
//! answer "did it cover the file when you opened it" and would go stale the
//! moment anything appended to it — including pdfcer's own incremental save.
//!
//! A `std::fs::metadata` call per frame is a stat, not a read, and the
//! alternative is a cached number that silently describes a file that no
//! longer exists in that form.
//!
//! # What changed at salvage
//!
//! **The panel now states which state of the file it measured.** The old
//! body's doc comment carried this and the old body never printed it:
//!
//! > Unsaved edits are not counted, and cannot be: they are not in the file
//! > yet. The panel says which state it is describing rather than leaving an
//! > operator to assume.
//!
//! The second sentence was a promise the code did not keep. It is kept here
//! by [`crate::text::panels::signatures_measured_on_disk`]. At S3 there are
//! no unsaved edits to miscount, which is exactly why it is cheap to add
//! now and expensive to remember later — the day editing lands, "does it
//! cover the file" and "does it cover what I am looking at" become different
//! questions with different answers, and only one of them is being answered
//! here.
//!
//! # This panel raises no actions, and cannot
//!
//! Signature verification is *"part 3"* in `pdfcer-core`'s own consumer map,
//! and nothing about a signature is editable from anywhere in pdfcer. The
//! `actions` parameter is present because the dock calls every panel through
//! one signature; this one never pushes to it.

use crate::app::actions::Action;
use crate::app::state::OpenDoc;
use crate::panels::PanelsState;
use crate::text::panels as t;

/// Draw the Signatures panel.
pub fn body(
    ui: &mut egui::Ui,
    doc: &OpenDoc,
    _state: &mut PanelsState,
    _actions: &mut Vec<Action>,
) {
    // A stat, not a read. Cheap enough per frame, and the alternative is a
    // cached number that silently describes a file that no longer exists in
    // that form.
    let Ok(meta) = std::fs::metadata(&doc.path) else {
        ui.label(t::signatures_file_unreadable());
        return;
    };
    let coverage = pdfcer_core::signature::byte_range_coverage(&doc.session.view(), meta.len());

    if coverage.is_empty() {
        ui.label(t::signatures_none());
        return;
    }

    // The caveat FIRST. Everything below it is a measurement, and a
    // measurement read as a verdict is the failure this prevents.
    ui.label(
        egui::RichText::new(t::signatures_not_a_validity_check())
            .small()
            .weak(),
    );
    ui.label(
        egui::RichText::new(t::signatures_measured_on_disk())
            .small()
            .weak(),
    );
    ui.separator();

    egui::ScrollArea::vertical()
        .id_salt("signatures-rows")
        .show(ui, |ui| {
            for c in &coverage {
                let name = c
                    .field_name
                    .clone()
                    .unwrap_or_else(|| t::signature_unnamed().to_owned());
                ui.label(egui::RichText::new(name));

                // Malformed is reported BEFORE coverage, because it changes
                // what the coverage numbers mean: a reader that rejects the
                // array computes something else, or nothing.
                if !c.ranges_well_formed {
                    ui.label(t::signature_range_malformed());
                }
                if c.pair_count == 1 {
                    ui.label(t::signature_single_range());
                }
                ui.label(if c.covers_to_eof() {
                    t::signature_covers_whole_file(c.covered)
                } else {
                    t::signature_leaves_tail(c.covered, c.uncovered_tail)
                });
                crate::diag::trace(|| {
                    format!(
                        "signature-row field={:?} covered={} tail={} pairs={} well_formed={}",
                        c.field_name,
                        c.covered,
                        c.uncovered_tail,
                        c.pair_count,
                        c.ranges_well_formed
                    )
                });
                ui.separator();
            }
        });
}

#[cfg(test)]
mod tests {
    use crate::text::panels as t;

    /// **The caveat leads, and it denies validity in the first sentence.**
    ///
    /// Not a test of the layout code — the ordering is one line above and a
    /// human can read it — but of the *sentence*, which is the half that
    /// could be softened by a well-meaning copy edit into something that
    /// merely implies the limitation.
    ///
    /// A panel headed "Signatures" listing byte counts is the likeliest
    /// place in this application for an operator to conclude more than was
    /// said, and the words are the only thing standing between them and that
    /// conclusion.
    #[test]
    fn the_caveat_denies_validity_checking_explicitly() {
        let caveat = t::signatures_not_a_validity_check();
        assert!(caveat.contains("does not check"), "{caveat}");
        assert!(
            caveat.contains("valid"),
            "the caveat must use the word an operator is thinking: {caveat}"
        );
        // And it must say what the numbers below it ARE, or it is a denial
        // with nothing in its place.
        assert!(caveat.contains("COVERS"), "{caveat}");
    }

    /// **"No signatures" and "pdfcer could not measure the file" are
    /// different sentences.**
    ///
    /// The first is a statement about the document. The second is a
    /// statement about pdfcer's ability to look, and rendering it as the
    /// first would be a claim about the operator's file made from an
    /// inability to read it.
    #[test]
    fn an_unreadable_file_is_not_reported_as_an_unsigned_one() {
        let none = t::signatures_none();
        let unreadable = t::signatures_file_unreadable();
        assert_ne!(none, unreadable);
        assert!(
            unreadable.contains("could not read"),
            "the failure must name itself: {unreadable}"
        );
        assert!(
            unreadable.contains("Nothing here is a statement about the document"),
            "and must actively deny the reading it would otherwise invite: {unreadable}"
        );
    }
}
