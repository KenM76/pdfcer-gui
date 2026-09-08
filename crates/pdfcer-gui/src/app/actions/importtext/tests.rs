//! # `app::actions::importtext` tests — the receipt, and what it stays quiet
//! about
//!
//! ## ★★★ What these can and cannot prove
//!
//! They cannot prove an import works: `import` needs an `OpenDoc` and does file
//! I/O, and the whole chain from a ribbon press to a page on screen is
//! `tools/ui-verify`'s to assert. R1 stands.
//!
//! What they prove is [`super::disclosures`] — a pure function over a report,
//! and the part of this module with the decisions in it. Its job is to be
//! **quiet**, and quietness is exactly what a test can pin and a person cannot
//! notice.

#![cfg(test)]

use super::*;

/// A report with nothing to disclose: an ordinary import that went perfectly.
///
/// ★ Built by mutating `PlaceTextReport::default()` rather than by naming every
/// field, deliberately. `PlaceTextReport` has **23** of them and is
/// `#[non_exhaustive]`; a literal here would not compile, and a helper that
/// listed twenty-three zeroes would have to be edited every time the engine
/// learns to count something new — which is the moment this test is most
/// valuable and least likely to be touched.
fn clean(pages: usize) -> PlaceTextReport {
    let mut report = PlaceTextReport::default();
    report.pages_created = pages;
    report.coalesced = true;
    report
}

/// ★★★ **A perfect import says exactly ONE thing.**
///
/// The whole design of this receipt in one assertion. Six of the seven
/// sentences are conditional, and a build that emitted them unconditionally —
/// *"0 tabs collapsed"*, *"0 paragraphs split"* — would produce a form rather
/// than a receipt, and by the third import nobody would read the line that
/// mattered.
///
/// ★★ It also pins that the **undo** sentence is conditional on `coalesced`.
/// That one is the most tempting to make unconditional, because it sounds
/// reassuring — *"this can be undone in one press"* — and it is the reassurance
/// that would train an operator past the one case where it is false.
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

/// ★★★ **The undo warning appears exactly when the engine says the fold
/// failed.**
///
/// `PlaceTextReport::coalesced` is documented as *checked, not assumed*: past
/// `MAX_UNDO_DEPTH` every page is still placed and they simply are not grouped.
/// A surface that promised one `Ctrl+Z` without reading this would be promising
/// something the engine has already said may not be true.
///
/// ★ Both directions in one test. A test that only asserted the warning appears
/// would pass on a build that showed it always — which is the failure the test
/// above owns, and asserting the pair here is what stops the two tests from
/// being satisfiable by opposite bugs.
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

/// ★★ **Each of the six judgements appears only when its count is non-zero**,
/// and every one of them can appear.
///
/// A table-driven test rather than six, because the property is *the same
/// property* six times and writing it once is what stops a seventh judgement
/// being added with no test. What it cannot check — and no test here can — is
/// that the sentence says something true; that is the doc comment's job on each
/// one in `text::importtext`.
#[test]
fn every_judgement_is_reported_when_it_happened_and_silent_when_it_did_not() {
    /// A word the sentence must contain, and the field that produces it.
    ///
    /// ★ Named rather than written inline: clippy calls the inline form a
    /// *"very complex type"*, and it is right that a reader meeting
    /// `&[(&str, fn(&mut PlaceTextReport))]` has to decode it before the test
    /// says anything.
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

/// ★★★ **The engine's self-check is reported LAST and worded as a fault.**
///
/// `box_overflow_lines` is documented as *"a self-check that must be 0"*, so a
/// non-zero value is a defect in the placer rather than a judgement about the
/// operator's file. If it were phrased and ordered like the other six, an
/// operator would file it under *"things imports do"* and never mention it —
/// and a fault nobody reports is a fault nobody fixes.
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

/// ★★ **Two refusals name the two CONTROLS that fix them**, because both are
/// answerable in the window that is still open behind the message.
///
/// `NoColumn` and `PageTooShort` are the only refusals here an operator can act
/// on before pressing again, and a sentence describing the geometry — *"the
/// margins exceed the media box width"* — would be true and useless. The test
/// pins that both name a control.
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

/// ★★★ **The unmappable refusal carries the engine's LISTING verbatim.**
///
/// The refusal a real text file is most likely to meet — an em dash, a curly
/// quote, an accented name — and the one part of it no rewording improves:
/// `U+2014 '—' ×12` is what the operator needs in order to find those
/// characters in his own file.
///
/// ⚠ It must **not** offer the engine's third remedy. `PlaceTextError::Unmappable`'s
/// own message ends *"or ask for them to be dropped"*, and this window has no
/// such control — a sentence naming a button that does not exist is worse than
/// one remedy fewer.
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
