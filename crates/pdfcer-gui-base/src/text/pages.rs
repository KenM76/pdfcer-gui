//! # `text::pages` — every string the Pages panel shows
//!
//! One area of the catalog described in [`crate::text`]'s header, consumed by
//! `pdfcer_gui::panels::pages` — the grid, its captions and its tile states — and
//! by `pdfcer_gui::app::actions::pages`, which words what a page **delete** broke.
//! Those are the only two readers.
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/text/pages.md`.

/// The document's page count, as the panel's first line.
#[must_use]
pub fn pages_count(total: usize) -> String {
    if total == 1 {
        "1 page".to_owned()
    } else {
        format!("{total} pages")
    }
}

/// How many pages the operator has picked, shown only when that is not zero.
#[must_use]
pub fn pages_selected(selected: usize) -> String {
    if selected == 1 {
        "1 page selected".to_owned()
    } else {
        format!("{selected} pages selected")
    }
}

/// **Where a drag will put the pages it is carrying** — the sentence beside
/// the insertion caret.
#[must_use]
pub fn drag_landing(moving: usize, gap: usize, page_count: usize) -> String {
    let what = if moving == 1 {
        "Moving 1 page".to_owned()
    } else {
        format!("Moving {moving} pages")
    };
    if gap == 0 {
        format!("{what} to the start")
    } else if gap >= page_count {
        format!("{what} to the end")
    } else {
        // `gap` is the boundary before page `gap`, and page numbers an
        // operator reads are 1-based — so the sheet this lands in front of is
        // `gap + 1`. The two conversions are done in one place, here, because
        // doing them at the call site is how a caret and its caption come to
        // name different sheets.
        format!("{what} to before page {}", gap + 1)
    }
}

/// A drag hovering a boundary that would change nothing.
#[must_use]
pub const fn drag_lands_nowhere() -> &'static str {
    "Release here and the order does not change — drag to a boundary outside \
     the pages you picked up."
}

/// A document with a page tree that resolved to nothing.
#[must_use]
pub fn pages_none() -> &'static str {
    "This document has no pages."
}

/// A tile's caption — the page number an operator would say out loud.
#[must_use]
pub fn page_number(page_index: usize) -> String {
    format!("{}", page_index + 1)
}

/// A tile's caption on a labelled document: the label, then the page number
/// when the label is something else.
#[must_use]
pub fn page_caption(page_index: usize, label: Option<&str>) -> String {
    match label {
        Some(label) => format!("{label} ({})", page_index + 1),
        None => page_number(page_index),
    }
}

/// A tile's tooltip: which page, how big it is, and what a click does.
#[must_use]
pub fn page_tile_tooltip(page_index: usize, width_pts: f64, height_pts: f64) -> String {
    let width_mm = crate::units::whole_mm_from_points(width_pts);
    let height_mm = crate::units::whole_mm_from_points(height_pts);
    format!(
        "Page {} — {width_mm} × {height_mm} mm. Click to go there, \
         Ctrl+click to add it to the selection, Shift+click to extend.",
        page_index + 1
    )
}

/// A tile whose page has not been rasterized yet.
///
/// See this module's header: the alternative is a blank rectangle, which is
/// a picture of an empty page and therefore a lie about the document.
#[must_use]
pub fn thumbnail_not_drawn_yet() -> &'static str {
    "Not drawn yet"
}

/// A tile whose page will not be rasterized, because previews are off.
#[must_use]
pub fn thumbnail_previews_off() -> &'static str {
    "Preview off"
}

/// A tile whose render pdfcer started and abandoned.
#[must_use]
pub fn thumbnail_abandoned() -> &'static str {
    "Not finished"
}

/// A tile whose page the renderer refused.
#[must_use]
pub fn thumbnail_failed() -> &'static str {
    "Would not draw"
}

/// The label of the control that turns page previews on and off.
#[must_use]
pub fn previews_label() -> &'static str {
    "Draw page previews"
}

/// …and its tooltip, which states the cost rather than hiding it.
#[must_use]
pub fn previews_tooltip() -> &'static str {
    "Draw a picture of each page. A dense drawing can take most of a second \
     per page whatever size it is drawn at, because the cost is in reading \
     the page rather than in filling the pixels — so pdfcer stops on its own \
     when it meets one."
}

/// **The suffix on the time-limit box** — the unit, and nothing else.
#[must_use]
pub fn previews_budget_suffix() -> &'static str {
    " s"
}

/// The prefix on the time-limit box, which is what makes an unlabelled number
/// box readable.
#[must_use]
pub fn previews_budget_prefix() -> &'static str {
    "≤ "
}

/// **What the time-limit box does, with the measurement that makes the
/// number choosable.**
#[must_use]
pub fn previews_budget_tooltip() -> &'static str {
    "How long pdfcer may spend drawing one page. Set it to 0 and there is no \
     limit at all: every page finishes however long it takes, and the window \
     does not respond while one is drawing. An ordinary drawing sheet \
     takes well under a tenth of a second; the densest CAD page measured takes \
     nearly one second. A page that runs over is skipped on its own — the rest \
     of the document still draws — and raising this draws it again."
}

/// **What the time-limit box shows when it is set to nothing** —
/// `OPERATOR_REQUESTS.md` **O187**, 2026-09-12: *“setting it to 0 should set
/// it to infinity (never time out)”*.
#[must_use]
pub fn previews_budget_never() -> &'static str {
    "no limit"
}

/// Which page was skipped, what it was given, and how to give it more.
#[must_use]
pub fn previews_skipped_note(page_index: usize, millis: u128) -> String {
    let seconds = millis as f32 / 1000.0;
    format!(
        "Page {} needed more than {seconds:.1} s to draw and was skipped. \
         Raise the time beside “{}” to draw it.",
        page_index + 1,
        previews_label()
    )
}

// ---------------------------------------------------------------------------
// THE PAGE VERBS' DISCLOSURES — what a delete broke, in words
//
// A second audience for this module, and the header's *"consumed by
// `pdfcer_gui::panels::pages` and by nothing else"* is now *"and by
// `pdfcer_gui::app::actions::pages`, which words what a page delete broke"*. The
// two belong together rather than in `crate::text::status`: they are sentences
// about **pages**, they use the same vocabulary as the panel above them
// (sheets, page numbers, this document), and splitting them would put half the
// page copy where a reader looking for the other half would not find it.
//
// # Why these exist at all — rule 4, and the engine asking for them
//
// `EditSession::delete_pages` returns a `DanglingReport` and its own
// documentation says what it is for:
//
//   > pdfcer **exceeds** Acrobat here on purpose. … surface (don't silently
//   > leave) dangling bookmarks/links/destinations as a reviewable post-delete
//   > report … rather than silently leaving them broken the way Acrobat does.
//
// The engine reports and deliberately does **not** repair, because repointing
// a bookmark at "whatever page now occupies that index" would be pdfcer
// deciding what the author meant. That leaves exactly one obligation on this
// side: say so. A delete that quietly broke 300 bookmarks and drew nothing is
// the shape of failure rule 4 exists to forbid — the drawing is unchanged, the
// file is not, and the operator would find out from a diff.
//
// # Why they are counted and not listed
//
// The engine's own choice, and this follows it: *"a delete that orphans 300
// bookmarks should say '300', not list them."* The status bar has **one row**
// that may not grow (R128), so a list could not be drawn there even if the
// report carried one.
//
// # The wording rule these follow
//
// Each names **what is now wrong** rather than what pdfcer did, because that is
// the sentence an operator can act on. "3 bookmarks now point at pages that
// are no longer here" is actionable; "the dangling reference census reported
// 3" is a status line about pdfcer.
// ---------------------------------------------------------------------------

/// Bookmarks (outline items, §12.3.3) whose destination page was removed.
#[must_use]
pub fn deleted_dangling_bookmarks(count: usize) -> String {
    if count == 1 {
        "1 bookmark now points at a page that is no longer in this document.".to_owned()
    } else {
        format!("{count} bookmarks now point at pages that are no longer in this document.")
    }
}

/// Links on **surviving** pages whose destination page was removed (§12.5.6.5).
#[must_use]
pub fn deleted_dangling_links(count: usize) -> String {
    if count == 1 {
        "1 link on the pages that remain points at a page that was removed.".to_owned()
    } else {
        format!("{count} links on the pages that remain point at pages that were removed.")
    }
}

/// Named destinations (§12.3.2.3) that resolved to a removed page.
#[must_use]
pub fn deleted_dangling_destinations(count: usize) -> String {
    if count == 1 {
        "1 named destination now points at a page that is no longer in this document.".to_owned()
    } else {
        format!(
            "{count} named destinations now point at pages that are no longer in this document."
        )
    }
}

/// The document carries a `/PageLabels` tree (§12.4.2) the deletion left
/// numerically stale.
#[must_use]
pub fn deleted_page_labels_stale() -> &'static str {
    "This document numbers its own pages, and those numbers were left as they were — the \
     sheets that remain still carry the labels they had before the deletion."
}

/// Preseparated page sets (§14.11.4) that lost at least one plate.
#[must_use]
pub fn deleted_separations_repaired(sets: usize) -> String {
    if sets == 1 {
        "1 set of printing plates lost a member, and the plates that remain were updated to \
         list only each other."
            .to_owned()
    } else {
        format!(
            "{sets} sets of printing plates lost members, and the plates that remain were \
             updated to list only each other."
        )
    }
}

// ---------------------------------------------------------------------------
// Insert from file
// ---------------------------------------------------------------------------

/// The title on the picker `pages.insert_from_file` opens.
#[must_use]
pub const fn insert_dialog_title() -> &'static str {
    "Insert pages from a PDF"
}

/// **What arrived, and the two different ways the rest did not.**
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Structures {
    /// The source document had bookmarks, and they did not come across.
    pub outline_dropped: bool,
    /// The source document had page labels, and they did not come across.
    pub labels_dropped: bool,
    /// This document's own page-label ranges now describe different sheets.
    pub labels_stale: bool,
}

/// # TWO numbers, because they are two different pieces of news
#[must_use]
pub fn inserted(
    count: usize,
    orphans: usize,
    unrecoverable: usize,
    structures: Structures,
    after_page_index: usize,
) -> String {
    let after = after_page_index.saturating_add(1);
    let pages = if count == 1 { "page" } else { "pages" };
    let mut line = format!("Inserted {count} {pages} after page {after}.");
    if structures.outline_dropped {
        line.push_str(" That file's bookmarks did not come across.");
    }
    // Two forms, because "Nor did..." needs something to follow.
    //
    // Found by the test beside this, not by reading: with `outline_dropped`
    // false the sentence came out *"Inserted 1 page after page 1. Nor did its
    // page numbering..."*, which is broken English and reads as a missing
    // sentence rather than a deliberate one. A conditional clause written as a
    // continuation is only correct in the branch its author had in mind.
    if structures.labels_dropped {
        if structures.outline_dropped {
            line.push_str(" Nor did its page numbering");
        } else {
            line.push_str(" That file's page numbering did not come across either");
        }
        line.push_str(
            " — the inserted sheets take whatever numbers this document already gives that \
             position.",
        );
    }
    // LAST of the three, and about THIS document rather than the source.
    //
    // Ordered deliberately: the first two are facts about a file the operator
    // has finished with, and this one is about the sheets in front of them. A
    // sentence whose most actionable clause is first would be read and then
    // abandoned at the part about a document nobody is looking at any more.
    if structures.labels_stale {
        line.push_str(
            " This document numbers its own pages, and those numbers now describe different \
             sheets than they did — the ranges were left exactly as they were.",
        );
    }
    // Saturating rather than a plain subtraction: the two numbers come from one
    // struct and the engine's contract is that the second counts a subset of the
    // first, but a disclosure is the last place to trust an invariant it can
    // cheaply not need. An underflow here would be a panic in the middle of a
    // successful edit.
    match orphans.saturating_sub(unrecoverable) {
        0 => {}
        1 => line.push_str(
            " 1 form control needs re-registering before it can be filled — \
             Forms, Tab order lists it.",
        ),
        n => line.push_str(&format!(
            " {n} form controls need re-registering before they can be filled — \
             Forms, Tab order lists them."
        )),
    }
    // "N MORE" only works when something came before it.
    //
    // Found by a driven run, not by reading: on a source whose orphans are ALL
    // unrecoverable the re-registering clause is skipped, and the sentence came
    // out *"Inserted 2 pages after page 2. 3 more lost their field definitions
    // entirely..."* — more than what? It reads as a sentence with one deleted
    // in front of it.
    //
    // This is the SECOND continuation-clause defect in this one function, and
    // the first was fixed an hour earlier three clauses up ("Nor did its page
    // numbering"). That is the lesson worth more than either fix: a conditional
    // clause written as a continuation is only correct in the branch its author
    // had in mind, and finding one is a reason to sweep the whole sentence
    // rather than to patch the instance.
    let led = orphans.saturating_sub(unrecoverable) > 0;
    match unrecoverable {
        0 => {}
        1 if led => line.push_str(
            " 1 more lost its field definition entirely; to get that one back, insert the \
             pages again from the document they came from.",
        ),
        1 => line.push_str(
            " 1 form control lost its field definition entirely and cannot be registered here; \
             to get it back, insert the pages again from the document it came from.",
        ),
        n if led => line.push_str(&format!(
            " {n} more lost their field definitions entirely; to get those back, insert the \
             pages again from the document they came from."
        )),
        n => line.push_str(&format!(
            " {n} form controls lost their field definitions entirely and cannot be registered \
             here; to get them back, insert the pages again from the document they came from."
        )),
    }
    line
}

/// The chosen file could not be opened, and why.
#[must_use]
pub fn insert_failed(detail: &str) -> String {
    format!("Nothing was inserted. {detail}")
}

/// The chosen file has no pages to insert.
#[must_use]
pub const fn insert_empty() -> &'static str {
    "That PDF has no pages, so nothing was inserted."
}

/// The insert dialog's window title.
#[must_use]
pub const fn insert_window_title() -> &'static str {
    "Insert pages"
}

/// Which file, and how big it is — the first thing the dialog says.
#[must_use]
pub fn insert_source(name: &str, pages: usize) -> String {
    if pages == 1 {
        format!("{name} — 1 page")
    } else {
        format!("{name} — {pages} pages")
    }
}

/// Heading over the which-pages radios.
#[must_use]
pub const fn insert_which_heading() -> &'static str {
    "Pages to insert"
}

/// The take-everything option, with the count in it.
#[must_use]
pub fn insert_all(pages: usize) -> String {
    if pages == 1 {
        "All (1 page)".to_owned()
    } else {
        format!("All ({pages} pages)")
    }
}

/// The typed-range option.
#[must_use]
pub const fn insert_range() -> &'static str {
    "Pages"
}

/// What the range field accepts, shown only while it is selected.
#[must_use]
pub const fn insert_range_hint() -> &'static str {
    "e.g. 1-4 or 3,1-2. They are inserted in the order you type, and a page may be listed twice."
}

/// The range does not name any page of the source.
#[must_use]
pub const fn insert_range_unparsable() -> &'static str {
    "That does not name any page of this file, so there is nothing to insert."
}

/// Heading over the where-it-goes radios.
#[must_use]
pub const fn insert_where_heading() -> &'static str {
    "Where"
}

/// After the page the operator was looking at. **The default.**
#[must_use]
pub fn insert_after_page(page_number: usize) -> String {
    format!("After page {page_number}")
}

/// Before it.
#[must_use]
pub fn insert_before_page(page_number: usize) -> String {
    format!("Before page {page_number}")
}

/// Before every existing page.
#[must_use]
pub const fn insert_at_start() -> &'static str {
    "At the start of the document"
}

/// After every existing page.
#[must_use]
pub const fn insert_at_end() -> &'static str {
    "At the end of the document"
}

/// How many pages the current choice would insert.
#[must_use]
pub fn insert_summary(count: usize) -> String {
    if count == 1 {
        "1 page will be inserted.".to_owned()
    } else {
        format!("{count} pages will be inserted.")
    }
}

/// The commit button, with the count in its own label.
#[must_use]
pub fn insert_commit(count: usize) -> String {
    if count == 1 {
        "Insert 1 page".to_owned()
    } else {
        format!("Insert {count} pages")
    }
}

/// The dialog's Cancel.
#[must_use]
pub const fn insert_cancel() -> &'static str {
    "Cancel"
}

//
// Why this is a different verb from an insert, and not a convenience over
// it. `insert_pages` takes SOME pages and **orphans** the widgets on them; a
// form field that arrives that way is drawn and unfillable. `merge_document`
// re-parents the widgets to their fields, so — the engine's own words —
// *"a merged field arrives fillable … that is the whole point of the verb"*.
//
// So the two commands are not "some pages" versus "all pages". They are
// *"pages"* versus *"a document, with the things that make its pages work"*:
// its form, its bookmarks, its named destinations. The copy below has to carry
// that, because an operator choosing between two entries on one tab has no
// other way to find out.
// ===========================================================================

/// What a merge brought across, and what it had to rename to do it.
#[must_use]
pub fn merged(outcome: &pdfcer_core::edit::MergeOutcome) -> Vec<String> {
    let mut notes = vec![format!(
        "Merged {} page(s) into this document.",
        outcome.pages_merged
    )];
    if outcome.fields_merged > 0 {
        notes.push(format!(
            "{} form field(s) came across and are fillable here.",
            outcome.fields_merged
        ));
    }
    if outcome.fields_renamed > 0 {
        notes.push(format!(
            "{} field name(s) were already in use here, so the arriving ones were renamed. \
             Anything that fills this form by name — a script, an FDF, a calculation — will \
             not match them.",
            outcome.fields_renamed
        ));
    }
    if outcome.named_destinations_renamed > 0 {
        notes.push(format!(
            "{} link target name(s) clashed and were renamed. Bookmarks that came with the \
             file were updated; a link from a THIRD document to the old name now points at \
             this document's own target instead.",
            outcome.named_destinations_renamed
        ));
    }
    if outcome.outline_items_carried > 0 {
        notes.push(format!(
            "{} bookmark(s) came across, added after this document's own.",
            outcome.outline_items_carried
        ));
    }
    if outcome.page_label_ranges > 0 {
        notes.push("Every page keeps the page number it showed in its own file.".to_owned());
    }
    if outcome.layers_merged > 0 {
        notes.push(format!(
            "{} layer(s) came across. Each stays separate, even where a layer here has the same name, so hiding one never hides the other document's content.",
            outcome.layers_merged
        ));
    }
    if outcome.layer_configs_dropped > 0 {
        notes.push(format!(
            "{} alternate layer view(s) in the merged file were left behind: they name only that file's layers. Its default view came across.",
            outcome.layer_configs_dropped
        ));
    }
    notes
}

/// The merge could not read the file it was given.
#[must_use]
pub fn merge_failed(detail: &str) -> String {
    format!("That document could not be merged: {detail}")
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **A document with no form controls gets no sentence about form
    /// controls.**
    #[test]
    fn no_orphans_means_no_clause_about_them() {
        let quiet = inserted(4, 0, 0, Structures::default(), 6);
        assert!(
            !quiet.contains("form"),
            "a zero count must say nothing: {quiet}"
        );
        assert!(
            quiet.contains("after page 7"),
            "1-based, as everywhere: {quiet}"
        );
    }

    /// A source that had nothing to lose is told nothing about losing it.
    #[test]
    fn a_source_with_no_structures_produces_no_clause_about_them() {
        let bare = inserted(2, 0, 0, Structures::default(), 0);
        assert_eq!(
            bare, "Inserted 2 pages after page 1.",
            "nothing may be claimed about structures the source did not have"
        );
    }

    /// Each of the three structure facts says its own thing.
    #[test]
    fn each_structure_fact_has_its_own_sentence() {
        let outline = inserted(
            1,
            0,
            0,
            Structures {
                outline_dropped: true,
                ..Structures::default()
            },
            0,
        );
        assert!(
            outline.contains("bookmarks did not come across"),
            "{outline}"
        );
        assert!(!outline.contains("numbering"), "{outline}");

        let labels = inserted(
            1,
            0,
            0,
            Structures {
                labels_dropped: true,
                ..Structures::default()
            },
            0,
        );
        assert!(
            labels.contains("page numbering did not come across either"),
            "with no bookmark clause before it the labels clause must stand alone, not \
             continue a sentence that was never written: {labels}"
        );
        assert!(!labels.contains("Nor did"), "{labels}");
    }

    /// The stale-label clause is about THIS document, and it is last.
    #[test]
    fn the_stale_clause_is_about_this_document_and_comes_last() {
        let all = inserted(
            1,
            0,
            0,
            Structures {
                outline_dropped: true,
                labels_dropped: true,
                labels_stale: true,
            },
            0,
        );
        let stale = all.find("This document numbers its own pages").expect(&all);
        let dropped = all.find("Nor did its page numbering").expect(&all);
        assert!(
            stale > dropped,
            "the clause about the open document must come after the ones about the source: {all}"
        );
        assert!(
            all.contains("left exactly as they were"),
            "it must say pdfcer did NOT renumber, which is the choice: {all}"
        );
    }

    /// A real count is stated, unhedged, agrees in number, and names the route.
    #[test]
    fn a_real_count_is_stated_without_hedging() {
        let one = inserted(1, 1, 0, Structures::default(), 0);
        assert!(one.contains("1 form control needs re-registering"), "{one}");
        assert!(!one.contains("Any"), "the hedge is gone: {one}");
        assert!(one.contains("Tab order"), "the route is named: {one}");

        let many = inserted(2, 3, 0, Structures::default(), 0);
        assert!(
            many.contains("3 form controls need re-registering"),
            "{many}"
        );
        assert!(many.contains("lists them"), "{many}");
    }

    /// The two counts are two sentences, and the recoverable one is the
    /// **difference**, not the total.
    #[test]
    fn the_recoverable_count_excludes_the_ones_that_cannot_be_recovered() {
        let measured = inserted(1, 13, 2, Structures::default(), 0);
        assert!(
            measured.contains("11 form controls need re-registering"),
            "13 minus the 2 that cannot be: {measured}"
        );
        assert!(
            measured.contains("2 more lost their field definitions"),
            "{measured}"
        );
        assert!(
            measured.contains("insert the pages again"),
            "the only remedy that works must be named: {measured}"
        );
    }

    /// Every orphan being unrecoverable produces one sentence, not a zero.
    #[test]
    fn all_unrecoverable_means_no_re_registering_clause() {
        let all_lost = inserted(1, 2, 2, Structures::default(), 0);
        assert!(
            !all_lost.contains("re-registering"),
            "there is nothing to re-register: {all_lost}"
        );
        assert!(
            all_lost.contains("2 form controls lost their field definitions"),
            "{all_lost}"
        );
        assert!(
            !all_lost.contains("more"),
            "\"N more\" needs a clause before it, and there is none: {all_lost}"
        );
    }

    /// The unrecoverable clause reads correctly with NOTHING before it.
    #[test]
    fn every_conditional_clause_reads_alone_as_well_as_in_sequence() {
        // Each clause as the ONLY one, which is the case a continuation breaks.
        let only_unrecoverable = inserted(1, 3, 3, Structures::default(), 0);
        let only_labels = inserted(
            1,
            0,
            0,
            Structures {
                labels_dropped: true,
                ..Structures::default()
            },
            0,
        );
        for line in [&only_unrecoverable, &only_labels] {
            for continuation in [" more ", "Nor did", " either lost"] {
                assert!(
                    !line.contains(continuation),
                    "{continuation:?} continues a clause that was not written: {line}"
                );
            }
        }
        // And in sequence, where the continuations ARE correct and shorter.
        let both = inserted(
            1,
            5,
            3,
            Structures {
                outline_dropped: true,
                labels_dropped: true,
                ..Structures::default()
            },
            0,
        );
        assert!(both.contains("3 more lost"), "{both}");
        assert!(both.contains("Nor did"), "{both}");
    }

    /// A count larger than the total cannot panic.
    #[test]
    fn an_impossible_pair_does_not_panic() {
        let odd = inserted(1, 1, 4, Structures::default(), 0);
        assert!(!odd.contains("re-registering"), "{odd}");
        assert!(odd.contains("4 form controls lost their"), "{odd}");
    }

    /// The page count agrees in number too.
    #[test]
    fn one_page_is_a_page_and_two_are_pages() {
        assert!(inserted(1, 0, 0, Structures::default(), 0).contains("Inserted 1 page after"));
        assert!(inserted(2, 0, 0, Structures::default(), 0).contains("Inserted 2 pages after"));
    }
}
