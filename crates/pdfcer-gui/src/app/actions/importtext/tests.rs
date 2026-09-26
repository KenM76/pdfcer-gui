//! # `app::actions::importtext` tests — the receipt, and what it stays quiet
//! about
//!
//! ## What these can and cannot prove
//!
//! They cannot prove an import works: `import` needs an `OpenDoc` and does file
//! I/O, and the whole chain from a ribbon press to a page on screen is
//! `tools/ui-verify`'s to assert. R1 stands.
//!
//! What they prove is [`super::disclosures`] — a pure function over a report,
//! and the part of this module with the decisions in it. Its job is to be
//! **quiet**, and quietness is exactly what a test can pin and a person cannot
//! notice.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/app/actions/importtext/tests.md`.

#![cfg(test)]

use super::*;

/// A report with nothing to disclose: an ordinary import that went perfectly.
fn clean(pages: usize) -> PlaceTextReport {
    let mut report = PlaceTextReport::default();
    report.pages_created = pages;
    report.coalesced = true;
    report
}

/// **A perfect import says exactly ONE thing.**
#[test]
fn a_perfect_import_discloses_only_how_many_pages_arrived() {
    let notes = disclosures(&clean(11), 4);
    assert_eq!(
        notes.len(),
        1,
        "a clean import must say one thing and not seven: {notes:#?}"
    );
    assert!(
        notes[0].contains("11") && notes[0].contains('4') && notes[0].contains("15"),
        "the one sentence must carry how many arrived, how many there were, and how many there \
         are now — an operator who chose 'at the start' has no other way to find them: {:?}",
        notes[0]
    );
}

/// **The undo warning appears exactly when the engine says the fold
/// failed.**
#[test]
fn the_undo_warning_tracks_coalesced_in_both_directions() {
    let mut folded = clean(3);
    folded.undo_entries = 1;
    assert!(
        !disclosures(&folded, 0).iter().any(|n| n.contains("Undo")),
        "a coalesced import must not warn about undo"
    );

    let mut split = clean(300);
    split.coalesced = false;
    split.undo_entries = 300;
    let notes = disclosures(&split, 0);
    assert!(
        notes
            .iter()
            .any(|n| n.contains("300") && n.contains("Undo")),
        "an import the engine could not fold must say how many presses it will take: {notes:#?}"
    );
}

/// **Each of the six judgements appears only when its count is non-zero**,
/// and every one of them can appear.
#[test]
fn every_judgement_is_reported_when_it_happened_and_silent_when_it_did_not() {
    /// A word the sentence must contain, and the field that produces it.
    type Judgement = (&'static str, fn(&mut PlaceTextReport));

    let cases: &[Judgement] = &[
        ("paragraph", |r| r.paragraphs_split_across_pages = 2),
        ("tab", |r| r.tabs_collapsed = 12),
        ("page break", |r| r.explicit_page_breaks = 3),
        ("wider than the column", |r| r.overlong_words = 1),
        ("non-printing", |r| r.chars_dropped_control = 5),
        ("could not be written", |r| r.chars_dropped_unmappable = 7),
    ];
    for (needle, set) in cases {
        let mut report = clean(2);
        set(&mut report);
        let notes = disclosures(&report, 0);
        assert!(
            notes.iter().any(|n| n.contains(needle)),
            "setting the field for {needle:?} produced no sentence containing it: {notes:#?}"
        );
        assert_eq!(
            notes.len(),
            2,
            "one judgement must produce exactly one sentence beside the page count, and \
             {needle:?} produced {}: {notes:#?}",
            notes.len()
        );
    }
}

/// **The engine's self-check is reported LAST and worded as a fault.**
#[test]
fn the_engines_self_check_is_last_and_says_it_is_a_fault() {
    let mut report = clean(2);
    report.tabs_collapsed = 1;
    report.box_overflow_lines = 4;
    let notes = disclosures(&report, 0);

    let last = notes.last().expect("there is at least one sentence");
    assert!(
        last.contains("fault in"),
        "the self-check must be worded as a fault in pdfcer rather than as a disclosure about \
         the file, and it must be last: {notes:#?}"
    );
}

/// **Two refusals name the two CONTROLS that fix them**, because both are
/// answerable in the window that is still open behind the message.
#[test]
fn the_chooser_refusals_name_the_controls_rather_than_the_geometry() {
    for message in [t::no_column(), t::page_too_short()] {
        assert!(
            message.contains("sheet size") && message.contains("margin"),
            "a refusal an operator can fix in the window must name the controls: {message:?}"
        );
        assert!(
            message.contains("nothing was imported"),
            "and it must say the document is untouched — `place_text` plans before it writes, \
             so this is a guarantee rather than a hope: {message:?}"
        );
    }
}

/// **The unmappable refusal carries the engine's LISTING verbatim.**
#[test]
fn the_unmappable_refusal_carries_the_listing_and_offers_no_button_that_does_not_exist() {
    let message = t::unmappable_refused("Helvetica", 12, "U+2014 '\u{2014}' x12");
    assert!(
        message.contains("U+2014"),
        "the listing must survive verbatim — it is the part the operator acts on: {message:?}"
    );
    assert!(
        message.contains("different font"),
        "and the remedy must name a control this window has: {message:?}"
    );
    assert!(
        !message.contains("dropped"),
        "it must NOT offer 'ask for them to be dropped' — the engine's third remedy is a control \
         this window does not draw: {message:?}"
    );
}
