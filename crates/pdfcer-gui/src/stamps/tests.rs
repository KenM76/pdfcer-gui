//! # Tests for the stamp-collection model
//!
//! ⚠ **These are the floor, not the ceiling — R1.** Not one of them proves the
//! feature works: they prove the *derivation* is right, which is a different
//! and much smaller claim. The check that proves an operator can make a stamp
//! collection is `tools/ui-verify`'s `stamp_collection_reaches_the_engine`,
//! which drives the release binary and reads the file that lands on disk.
//!
//! This project has been bitten by exactly the gap these tests could create:
//! eight green unit tests while the feature did one step of fourteen, because
//! every test called the verb directly and none of them could see the chain in
//! front of it. So what is asserted here is deliberately narrow and stated as
//! narrow.
//!
//! ## The one test that is worth more than the rest
//!
//! [`plan_and_extraction_agree`]. `name_stamp_pages` names `stamps[i]` to page
//! `i` **by counting**, so the name list and the extracted page list must be
//! the same length in the same order — and a build that gets that wrong writes
//! a file that opens, has the right stamp count, appears in Acrobat's menu and
//! stamps the wrong picture every time. There is no symptom short of looking
//! at the artwork.

//! ## ★ `#![cfg(test)]` as well as the parent's `#[cfg(test)] mod tests;`
//!
//! Redundant to the compiler and load-bearing to two gates.
//!
//! `tools/gates/check-ui-strings.sh` stops scanning a file at a `#[cfg(test)]`
//! line — assertion messages are prose read by whoever is staring at a failing
//! test, never by an operator — and a whole-file test module has no such line
//! to stop at. **This file was written without one and the gate reported 19 of
//! its assertion messages as operator-facing copy**, which is exactly what
//! `shell/commands/tests.rs`'s own header records happening to
//! `canvas/selection/tests.rs` (28 there). Its wording is worth repeating:
//! *"the noise is the actual hazard"*, because a report full of false
//! positives trains people to ignore it.
//!
//! `check-theme-colors.sh` recognises the same inner attribute from the AST,
//! and `app::settings`' `syn` check recognises it too. All three state why it
//! is the marker rather than the filename: the property that earns the
//! exemption is *"not in the shipped binary"*, and a filename is a restatement
//! of that which goes stale the moment another such module is written.

#![cfg(test)]

use super::{Adjustment, Blocker, ExistingName, Plan, derive_internal};

/// A plan over `pages` pages with no pre-existing collection.
fn plan(pages: usize) -> Plan {
    Plan::new(pages, "Signatures", &[])
}

/// The names an already-written collection carries, as the plan sees them.
///
/// ★ Deliberately built from `(page, display)` rather than from a
/// `StampCollection`: `super::existing_names` is the one place the engine's
/// `#[non_exhaustive]` type is converted, and the plan's contract is about
/// this narrower shape. `stamp_collection_reaches_the_engine` in
/// `tools/ui-verify` is what exercises the real reader, on a real file.
fn existing(entries: &[(Option<usize>, &str)]) -> Vec<ExistingName> {
    entries
        .iter()
        .map(|(page_index, display)| ExistingName {
            page_index: *page_index,
            display: (*display).to_owned(),
        })
        .collect()
}

// ─────────────────────────────────────────────────────────────────────────
// derive_internal — one test per clause, in the order the clauses run
// ─────────────────────────────────────────────────────────────────────────

#[test]
fn an_ordinary_name_survives_unchanged_and_discloses_nothing() {
    let (name, adjustments) = derive_internal("Approved", &[]);
    assert_eq!(name, "Approved");
    assert!(
        adjustments.is_empty(),
        "the common case must be silent — a disclosure on every stamp is a \
         disclosure nobody reads"
    );
}

#[test]
fn spaces_are_dropped_rather_than_becoming_underscores() {
    // Adobe's own keys are unspaced camel case (`SBApproved`, `SHInitialHere`).
    // Matching the neighbours is worth more than preserving word boundaries in
    // a string no operator ever reads.
    let (name, adjustments) = derive_internal("Sign Here", &[]);
    assert_eq!(name, "SignHere");
    assert_eq!(adjustments, vec![Adjustment::CharactersRemoved(1)]);
}

#[test]
fn a_leading_hash_is_removed_even_when_a_space_follows_it() {
    // ★ THE ORDERING TEST. `#` is stripped AFTER the character filter, because
    // only the filter makes `# Approved` and `#Approved` the same string.
    // Reverse the two clauses and this input keeps its marker.
    for input in ["#Approved", "# Approved", "#  Approved"] {
        let (name, adjustments) = derive_internal(input, &[]);
        assert_eq!(name, "Approved", "input {input:?}");
        assert!(
            adjustments.contains(&Adjustment::DynamicMarkerRemoved),
            "input {input:?} must disclose the marker removal — it is the one \
             adjustment with a correctness consequence"
        );
    }
}

#[test]
fn a_non_leading_hash_is_removed_too() {
    // Legal in a name tree, but a `#` that a later edit could move to the
    // front is a trap, and no Adobe key contains one.
    let (name, _) = derive_internal("Rev#2", &[]);
    assert_eq!(name, "Rev2");
}

#[test]
fn a_name_that_sanitises_to_nothing_falls_back() {
    let (name, _) = derive_internal("★★★", &[]);
    assert_eq!(
        name, "Stamp",
        "a name-tree key of \"\" is a file Acrobat shows an empty menu row for"
    );
}

#[test]
fn a_long_name_is_truncated_and_says_so() {
    let long = "A".repeat(super::MAX_INTERNAL_LEN + 10);
    let (name, adjustments) = derive_internal(&long, &[]);
    assert_eq!(name.chars().count(), super::MAX_INTERNAL_LEN);
    assert!(adjustments.contains(&Adjustment::Truncated));
}

#[test]
fn truncation_happens_before_deduplication() {
    // ★ Reverse the order and the appended number is cut back off, silently
    // reintroducing the collision the number existed to resolve.
    let long = "A".repeat(super::MAX_INTERNAL_LEN + 10);
    let taken = vec!["A".repeat(super::MAX_INTERNAL_LEN)];
    let (name, _) = derive_internal(&long, &taken);
    assert_ne!(
        name, taken[0],
        "the second stamp must not collide with the first"
    );
    assert!(name.ends_with('2'), "got {name:?}");
}

#[test]
fn a_duplicate_is_numbered_and_the_new_name_is_disclosed() {
    let (name, adjustments) = derive_internal("Approved", &["Approved".to_owned()]);
    assert_eq!(name, "Approved2");
    assert_eq!(
        adjustments,
        vec![Adjustment::MadeUnique("Approved2".to_owned())],
        "the disclosure carries the name that was ACTUALLY written — \"we \
         renamed it\" without saying to what costs a round trip to check"
    );
}

#[test]
fn numbering_keeps_climbing_past_an_existing_number() {
    let taken = vec!["Approved".to_owned(), "Approved2".to_owned()];
    let (name, _) = derive_internal("Approved", &taken);
    assert_eq!(name, "Approved3");
}

// ─────────────────────────────────────────────────────────────────────────
// Plan
// ─────────────────────────────────────────────────────────────────────────

#[test]
fn a_fresh_plan_names_every_page_one_based() {
    let p = plan(3);
    let names: Vec<&str> = p.stamps.iter().map(|s| s.display.as_str()).collect();
    assert_eq!(names, ["Stamp 1", "Stamp 2", "Stamp 3"]);
    assert_eq!(p.included(), 3);
}

#[test]
fn plan_and_extraction_agree() {
    // ★★ THE LOAD-BEARING TEST. See the module header.
    let mut p = plan(5);
    p.stamps[0].include = false;
    p.stamps[3].include = false;
    p.rederive();

    let names = p.for_engine();
    let pages = p.pages_to_extract();

    assert_eq!(
        names.len(),
        pages.len(),
        "the name list and the page list are the same filter over the same \
         rows; a length disagreement means the two have drifted apart"
    );
    assert_eq!(pages, vec![1, 2, 4]);
    assert_eq!(
        names
            .iter()
            .map(|(_, display)| display.as_str())
            .collect::<Vec<_>>(),
        ["Stamp 2", "Stamp 3", "Stamp 5"],
        "position i of the name list must describe the page at position i of \
         the extraction, or every stamp names the wrong artwork"
    );
}

#[test]
fn an_excluded_row_claims_no_name_and_frees_it() {
    // ★ An excluded row must not consume a name. If it did, the row that
    // actually claimed the name would show a `MadeUnique` disclosure the
    // operator cannot explain, because the row that took it is not in the file.
    let mut p = Plan::new(2, "Signatures", &[]);
    p.stamps[0].display = "Approved".to_owned();
    p.stamps[1].display = "Approved".to_owned();
    p.stamps[0].include = false;
    p.rederive();

    assert!(p.stamps[0].internal.is_empty());
    assert!(p.stamps[0].adjustments.is_empty());
    assert_eq!(
        p.stamps[1].internal, "Approved",
        "the surviving row gets the plain name, not Approved2"
    );
    assert!(p.stamps[1].adjustments.is_empty());
}

#[test]
fn rederiving_after_a_rename_drops_the_stale_disclosure() {
    // ★ A disclosure has a subject. Renaming row 0 frees `Approved`, so row 1's
    // "we renamed it" sentence must go with the collision that caused it —
    // otherwise the operator reads a permanent explanation of something that is
    // no longer true.
    let mut p = Plan::new(2, "Signatures", &[]);
    p.stamps[0].display = "Approved".to_owned();
    p.stamps[1].display = "Approved".to_owned();
    p.rederive();
    assert_eq!(p.stamps[1].internal, "Approved2");
    assert_eq!(p.adjustments().len(), 1);

    p.stamps[0].display = "Issued".to_owned();
    p.rederive();
    assert_eq!(p.stamps[1].internal, "Approved");
    assert!(
        p.adjustments().is_empty(),
        "the collision is gone, so the sentence explaining it must be gone too"
    );
}

#[test]
fn adjustments_ignore_excluded_rows() {
    let mut p = Plan::new(1, "Signatures", &[]);
    p.stamps[0].display = "#Dynamic".to_owned();
    p.rederive();
    assert_eq!(p.adjustments().len(), 1);

    p.stamps[0].include = false;
    p.rederive();
    assert!(
        p.adjustments().is_empty(),
        "a page that is not in the file owes no disclosure about it"
    );
}

#[test]
fn the_two_blockers_are_reported_separately() {
    let mut p = plan(1);
    assert_eq!(p.blocker(), None);

    p.category = "   ".to_owned();
    assert_eq!(
        p.blocker(),
        Some(Blocker::NoCategory),
        "an untitled collection shows up in Acrobat's menu with no heading"
    );

    p.category = "Signatures".to_owned();
    p.stamps[0].include = false;
    assert_eq!(p.blocker(), Some(Blocker::NoStamps));
}

#[test]
fn the_no_stamps_blocker_wins_when_both_apply() {
    // Not arbitrary: an empty category is fixed by typing into a field that is
    // right there, and an empty stamp list is fixed by scrolling to a row. The
    // more surprising refusal is the more useful one to state.
    let mut p = plan(1);
    p.category = String::new();
    p.stamps[0].include = false;
    assert_eq!(p.blocker(), Some(Blocker::NoStamps));
}

// ─────────────────────────────────────────────────────────────────────────
// Reopening a collection
// ─────────────────────────────────────────────────────────────────────────

#[test]
fn an_existing_collection_is_matched_by_page_not_by_tree_position() {
    // ★★ The `StandardBusiness.pdf` case, in miniature. The tree is sorted
    // lexicographically (§7.9.6) and the pages are not, so zipping the two
    // lists attaches the wrong name to every page.
    let names = existing(&[(Some(0), "Approved"), (Some(4), "Completed")]);
    let p = Plan::new(5, "Standard Business", &names);

    assert_eq!(p.stamps[0].display, "Approved");
    assert_eq!(
        p.stamps[4].display, "Completed",
        "page 5 carries the name the tree gave page 5, not the tree's second \
         entry"
    );
    assert_eq!(
        p.stamps[1].display, "Stamp 2",
        "a page the tree does not name falls back to a default"
    );
}

#[test]
fn a_stamp_naming_no_page_does_not_claim_one() {
    let names = existing(&[(None, "Ken")]);
    let p = Plan::new(2, "Signatures", &names);
    assert_eq!(p.stamps[0].display, "Stamp 1");
    assert_eq!(p.stamps[1].display, "Stamp 2");
}

// ---------------------------------------------------------------------------
// The driven check's INPUT, asserted here so a cargo test can see it
// ---------------------------------------------------------------------------

/// **`fixtures/stamp-collection.pdf` really is a stamp collection, and it has
/// the three properties the driven check reads.**
///
/// ⚠ A tripwire for the harness's INPUT, not for this module's logic — the
/// same shape, and for the same reason, as
/// `app::status::anomalies`'s `the_control_fixtures_a_driven_run_uses_are_genuinely_clean`.
/// `ui-verify`'s `a_stamp_collection_discloses_itself` launches the release
/// binary on this file and asserts the Document-properties section is there,
/// then launches again on `four-pages.pdf` and asserts it is not. If this
/// fixture were quietly regenerated without its name tree, the first launch
/// would go red and the report would blame the panel for a defect that is
/// entirely in the file it was pointed at — the harness-input failure this
/// project has already paid for twice, once at the cost of four filed defects
/// against code that was correct.
///
/// ★ And the reverse tripwire in the same test: `four-pages.pdf` is asserted
/// NOT to be a collection. An absence assertion whose control had grown a name
/// tree would be a check that cannot fail.
#[test]
fn the_driven_checks_fixtures_are_what_the_check_believes_they_are() {
    let doc = crate::app::state::open_local_fixture("stamp-collection.pdf");
    let collection = pdfcer_core::stamp_file::read(doc.session.document());

    assert!(
        crate::stamps::is_collection(&collection),
        "stamp-collection.pdf has no /Names -> /Pages tree, so the driven \
         presence check would be asserting a disclosure about an ordinary PDF"
    );
    assert_eq!(
        collection.category.as_deref(),
        Some("Site Review"),
        "the category is the /Info /Title and the panel row prints it verbatim"
    );
    assert_eq!(collection.stamps.len(), 3);
    assert_eq!(
        crate::stamps::dynamic_count(&collection),
        1,
        "one of three, deliberately: it is the count sentence's MIDDLE branch, \
         and the two ends are the easy ones"
    );
    assert_eq!(
        crate::stamps::unresolved_count(&collection),
        0,
        "every name in this fixture resolves. A dangling one would pin an \
         ambiguity pdfcer cannot currently resolve rather than a behaviour — \
         see `unresolved_count`'s own warning"
    );

    // ★★ The property the row regions exist for: TREE ORDER IS NOT PAGE
    // ORDER. `#` sorts before `S`, so the dynamic stamp is tree entry 0 and
    // page 3. A build that enumerated pages instead of tree entries would
    // produce a plausible list in the wrong order, and only this assertion
    // notices.
    assert_eq!(collection.stamps[0].display, "Issued");
    assert_eq!(collection.stamps[0].page_index, Some(2));
    assert!(collection.stamps[0].dynamic);
    assert_eq!(collection.stamps[1].display, "Approved");
    assert_eq!(collection.stamps[1].page_index, Some(0));

    // The control for the absence half.
    let plain = crate::app::state::open_local_fixture("four-pages.pdf");
    assert!(
        !crate::stamps::is_collection(&pdfcer_core::stamp_file::read(plain.session.document())),
        "four-pages.pdf is the CONTROL for the driven absence check and it now \
         reads as a stamp collection. Pick a different control; do not widen \
         the check"
    );
}
