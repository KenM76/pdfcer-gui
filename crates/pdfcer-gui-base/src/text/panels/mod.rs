//! # `text::panels` — every string the dock's panels show
//!
//! One area of the catalog described in [`crate::text`]'s header. It covers
//! the panel bodies in `pdfcer_gui::panels`. The three document-structure
//! panels are in this file; Comments, Fonts, Objects and Properties each have
//! their own module, and Forms and Pages have their own areas one level up.
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/text/panels/mod.md`.

/// **The annotation half of the Properties panel's geometry section** —
/// X/Y/W/H, typeable over a selected markup.
pub mod annotgeometry;
/// The Attachments panel — the whole files a document carries inside itself
/// (§7.11.4.1), and the four verbs opposite them.
pub mod attachments;
/// The words for **moving** a bookmark and for **expanding or collapsing**
/// one — `pdfcer-core` `Pass 161.0`'s two verbs.
pub mod bookmarks;
pub mod comments;
/// The ce-dimension properties section — the bottom tier of the style cascade
/// made reachable, with the tier each value came from named beside it.
pub mod dimension;
/// **The Document properties panel's copy** — the file's own title, author,
/// subject and keywords, the seven facts pdfcer read about it, and the two
/// disclosures it can owe.
pub mod docprops;
/// The **face chooser**, which is one control drawn on two surfaces — the
/// Properties panel's *This text* section and the ribbon's Format ▸ Font group.
pub mod face;
/// **The Layers panel's search field**, and the two lines that describe
/// what it did to the list. Its own module because these four are the only
/// strings here that describe a FILTER'S EFFECT on a list, and the wording
/// rules for that job are written down with them.
pub mod layersearch;

/// The Objects panel, and the wording of every object fact.
pub mod objects;

/// **Every sentence pdfcer says about which layer a selection is on** — the
/// panel's long form and the status bar's short clause, generated from one
/// `crate::layermembership::Membership` so the two surfaces cannot
/// drift. Its own module under R2 and under `DEFECTS.md` D5; the module header
/// argues the seam.
pub mod layers;

/// What the Layers panel says when it creates, edits or deletes a layer.
pub mod layeredit;
pub use layers::layer_selection_unlayered;

/// **A choice field's `/Opt` list**, and the three `/Ff` flags Acrobat groups
/// with it. Its own module under R2 and on the seam the code takes; the header
/// argues the Shown/Sent vocabulary.
pub mod choiceopts;
/// The Fonts panel's inventory report.
pub mod fonts;
/// **The Forms panel and the form-field half of Properties** — a placed
/// field's flags, its tooltip, its maximum length, its default value and its
/// alignment.
pub mod formfield;
/// The Properties panel.
pub mod properties;
/// The Properties panel's **text-annotation style** section — a sticky
/// note's icon and colour and a stamp's colour, which reach
/// `EditSession::set_text_annot_style` rather than `set_markup_style`.
pub mod textannotstyle;
/// The Properties panel's **clicked-text** colour section — O89's object
/// route.
pub mod textobject;

// ---------------------------------------------------------------------------
// Shared
// ---------------------------------------------------------------------------

/// Format a byte count for a listing.
#[must_use]
pub fn byte_size(bytes: usize) -> String {
    #[allow(
        clippy::cast_precision_loss,
        reason = "a display rounding to one or two decimals; the exact count is printed alongside" // ui-text-exempt: clippy lint justification, never displayed
    )]
    let n = bytes as f64;
    if bytes < 1024 {
        format!("{bytes} B")
    } else if bytes < 1024 * 1024 {
        format!("{:.1} KB", n / 1024.0)
    } else {
        format!("{:.2} MB", n / (1024.0 * 1024.0))
    }
}

/// Shown in any panel when no document is open.
#[must_use]
pub fn panel_no_document() -> &'static str {
    "Open a document to see this panel."
}

/// Shown when the dock holds a panel this build does not have.
#[must_use]
pub fn panel_unknown() -> &'static str {
    "This panel is not part of this build. Your saved layout asked for it; \
     everything else in the layout loaded normally."
}

// ---------------------------------------------------------------------------
// Signatures
// ---------------------------------------------------------------------------

// THERE IS NO "pdfcer cannot check whether these signatures are valid"
// SENTENCE HERE, AND ONE MUST NOT BE ADDED BACK.
//
// `signature::verify_all_with_trust` is wired into `pdfcer_gui::panels::signatures`
// (`crate::trust::examine`), so any such sentence is false. ⚠ **A denial of an
// engine capability is the most expensive shape of prose this project has: it
// is true when written, it has a shelf life measured in hours, and it sits
// inside surrounding prose that stays true — so nothing about the paragraph
// looks stale.**
//
// The panel's opening line describes the SHAPE of the three facts it reports
// rather than denying one of them, and it lives in
// `crate::text::trust::panel_intro` beside the rest of the trust copy, because
// the four rules governing it are that module's. The opener test below aims at
// that function, so "each structure panel leads with its own limitation" is
// measured against the sentence actually on screen.

/// No signature carries a byte range.
#[must_use]
pub fn signatures_none() -> &'static str {
    "This document has no signatures. (An empty signature field, waiting to be signed, is not one.)"
}

/// The file could not be measured.
#[must_use]
pub fn signatures_file_unreadable() -> &'static str {
    "pdfcer could not read this file's size from disk, so it cannot say what the signatures cover. Nothing here is a statement about the document."
}

/// A signature field with no name of its own.
#[must_use]
pub fn signature_unnamed() -> &'static str {
    "(unnamed signature)"
}

/// The good case.
#[must_use]
pub fn signature_covers_whole_file(covered: u64) -> String {
    format!("Covers the whole file — {covered} bytes, up to the last one.")
}

/// The case that matters: content exists beyond the signed range.
#[must_use]
pub fn signature_leaves_tail(covered: u64, tail: u64) -> String {
    format!(
        "Covers {covered} bytes, but {tail} bytes come after the signed range — content this signature does not protect. That is allowed by the standard, and it means the signature guarantees less than its presence suggests."
    )
}

/// Overlapping or backwards ranges.
#[must_use]
pub fn signature_range_malformed() -> &'static str {
    "This signature's byte range is malformed — its parts overlap or run backwards, which the standard does not permit. The numbers below are what the file claims; another reader may compute something different, or refuse it."
}

/// A single range, which cannot verify.
#[must_use]
pub fn signature_single_range() -> &'static str {
    "This signature declares one continuous range, so it includes its own signature value in what it signs. A signature in that shape cannot verify anywhere."
}

/// The line naming which state of the file the coverage numbers describe.
#[must_use]
pub fn signatures_measured_on_disk() -> &'static str {
    "Measured against the file as it is on disk right now. Any edits you have not saved are not part of these numbers."
}

// ---------------------------------------------------------------------------
// Layers
// ---------------------------------------------------------------------------

/// Shown when the document declares no optional content at all.
///
/// Distinct from "no layers I could read": most PDFs simply have none, and
/// saying so plainly stops an operator hunting for a panel fault.
#[must_use]
pub fn layers_none() -> &'static str {
    "This document has no layers."
}

/// Count above the list.
#[must_use]
pub fn layers_count(total: usize) -> String {
    if total == 1 {
        "1 layer.".to_owned()
    } else {
        format!("{total} layers.")
    }
}

/// The disclosure above the list, always shown.
#[must_use]
pub fn layers_session_only_note() -> &'static str {
    "Switching a layer changes what you see, not the document. Nothing here is saved — a layer's on or off state lives outside the file, so this document opens with its own settings again next time."
}

/// How many layers currently differ from the document's own configuration.
#[must_use]
pub fn layers_overridden(n: usize) -> String {
    if n == 1 {
        "1 layer differs from the document.".to_owned()
    } else {
        format!("{n} layers differ from the document.")
    }
}

/// Label on the control that drops the operator's layer changes.
#[must_use]
pub fn layers_reset_label() -> &'static str {
    "Reset"
}

/// Tooltip on that control.
#[must_use]
pub fn layers_reset_tooltip() -> &'static str {
    "Go back to the layer states the document itself specifies. This shows hidden layers as hidden again."
}

/// Tooltip on a layer's visibility control.
#[must_use]
pub fn layer_toggle_tooltip() -> &'static str {
    "Show or hide this layer on screen. The document is not changed."
}

/// Tooltip on a layer whose state the operator has changed.
#[must_use]
pub fn layer_overridden_tooltip(document_wanted_visible: bool) -> &'static str {
    if document_wanted_visible {
        "You have hidden this layer. The document shows it."
    } else {
        "You have shown this layer. The document hides it."
    }
}

/// Some layers' states are managed automatically (§8.11.4.4).
#[must_use]
pub fn layers_auto_managed(n: usize) -> String {
    if n == 1 {
        "1 layer switches itself on or off as you zoom. The state shown here is the one the document opens in.".to_owned()
    } else {
        format!(
            "{n} layers switch themselves on or off as you zoom. The states shown here are the ones the document opens in."
        )
    }
}

/// Tooltip on a layer whose `/Intent` excludes viewing (§8.11.2.3).
#[must_use]
pub fn layer_design_intent_tooltip() -> &'static str {
    "This layer is marked for design use, not viewing, so the document's own on or off setting for it does not affect what is drawn. Switching it here does: your choice replaces the document's whole layer configuration for as long as this document is open."
}

/// Placeholder for a layer whose `/Name` is absent.
#[must_use]
pub fn layer_unnamed() -> &'static str {
    "(no name in the file)"
}

/// Text marker for a layer drawn by default. TEXT, never colour alone.
#[must_use]
pub fn layer_visible_marker() -> &'static str {
    "shown"
}

/// Text marker for a layer hidden by default.
#[must_use]
pub fn layer_hidden_marker() -> &'static str {
    "hidden"
}

/// Tooltip on a locked layer.
#[must_use]
pub fn layer_locked_tooltip() -> &'static str {
    "The document marks this layer locked, so a viewer should not offer to switch it. It is an interface lock, not a guarantee — the document's own scripts can still change it."
}

/// Tooltip on a layer that content references but the default configuration
/// never registered.
#[must_use]
pub fn layer_unregistered_tooltip() -> &'static str {
    "Page content uses this layer, but the document never listed it in its layer configuration. Some readers will not show it in their own layer panel at all."
}

/// Tooltip on a layer in a radio-button group.
#[must_use]
pub fn layer_radio_tooltip() -> &'static str {
    "One of a group where switching this layer on switches the others off."
}

/// Tooltip on a radio-group member whose group also contains a locked layer.
#[must_use]
pub fn layer_radio_locked_sibling_tooltip() -> &'static str {
    "Another layer in this group is locked, so pdfcer will not switch it off for you. Switching this layer on can leave two of the group showing at once, which the document says should not happen."
}

// ---------------------------------------------------------------------------
// Bookmarks
// ---------------------------------------------------------------------------

/// Summary line above the tree.
#[must_use]
pub fn bookmarks_count(total: usize) -> String {
    if total == 1 {
        "1 bookmark.".to_owned()
    } else {
        format!("{total} bookmarks.")
    }
}

/// Shown when the document has an outline but no items pdfcer could read.
#[must_use]
pub fn bookmarks_empty() -> &'static str {
    "This document has no bookmarks."
}
// ---------------------------------------------------------------------------
// Bookmarks — writing one
// ---------------------------------------------------------------------------

/// The heading over the add-a-bookmark row.
#[must_use]
pub const fn bookmark_add_heading() -> &'static str {
    "Add a bookmark"
}

/// Where a new bookmark will be filed, when a row has been clicked.
#[must_use]
pub fn bookmark_add_under(parent: &str) -> String {
    format!("Under {parent}")
}

/// Where it will be filed when no row has been clicked.
#[must_use]
pub const fn bookmark_add_at_top() -> &'static str {
    "At the top level"
}

/// The control that clears the chosen parent.
#[must_use]
pub const fn bookmark_add_to_top_button() -> &'static str {
    "Move to top level"
}

/// How a parent is chosen.
///
#[must_use]
pub const fn bookmark_add_parent_hint() -> &'static str {
    "Click a bookmark above to file the new one under it. Clicking also jumps \
     there, as it always does."
}

/// The destination the new bookmark will point at.
#[must_use]
pub fn bookmark_add_destination(page_number: usize) -> String {
    format!("It will point at page {page_number}, the one on screen.")
}

/// **The `/Count` trap, turned into a sentence.**
#[must_use]
pub const fn bookmark_add_under_collapsed() -> &'static str {
    "That bookmark is collapsed, so the new one will not appear until you \
     expand it. It will still be in the file."
}

/// The title field's placeholder.
#[must_use]
pub const fn bookmark_add_title_hint() -> &'static str {
    "What to call it"
}

/// The button that writes the bookmark.
#[must_use]
pub const fn bookmark_add_button() -> &'static str {
    "Add"
}

/// Why the button is unavailable with an empty title.
#[must_use]
pub const fn bookmark_add_needs_a_title() -> &'static str {
    "Type a name first. A bookmark with no title still appears in the list, as \
     a blank row nothing distinguishes."
}

// ---------------------------------------------------------------------------
// Bookmarks — renaming one, and removing one with everything under it
//
// The surface for `EditSession::set_outline_title` and
// `EditSession::delete_outline_item`, both `pdfcer-core` `Pass 156.0`. See
// `pdfcer_gui::panels::bookmarks::edit` for the interaction; this block is only the
// words, and two of them are load-bearing in a way the rest of this file's
// entries are not:
//
//   * `bookmark_delete_takes_subtree` is said BEFORE the press, because the
//     verb's blast radius is larger than the row the operator clicked, and
//     they cannot see how much larger when the row is collapsed.
//   * `bookmark_deleted` is said AFTER it, from the number the ENGINE
//     returned, because the shell's own count is a count of what pdfcer could
//     read and the engine's is a count of what it removed.
// ---------------------------------------------------------------------------

/// The heading over the rename-and-remove block.
#[must_use]
pub const fn bookmark_edit_heading() -> &'static str {
    "Selected bookmark"
}

/// Which bookmark the rename and remove controls act on.
#[must_use]
pub fn bookmark_edit_selected(title: &str) -> String {
    format!("Selected: {title}")
}

/// The label beside the rename field.
#[must_use]
pub const fn bookmark_rename_label() -> &'static str {
    "Name"
}

/// The button that writes the new title.
#[must_use]
pub const fn bookmark_rename_button() -> &'static str {
    "Rename"
}

/// The button that removes the bookmark.
#[must_use]
pub fn bookmark_copy_takes_subtree(descendants: usize) -> String {
    if descendants == 1 {
        "Copying this takes the one bookmark filed under it as well.".to_owned()
    } else {
        format!("Copying this takes the {descendants} bookmarks filed under it as well.")
    }
}

/// The Copy button.
#[must_use]
pub const fn bookmark_copy_button() -> &'static str {
    "Copy"
}

/// The Cut button.
#[must_use]
pub const fn bookmark_cut_button() -> &'static str {
    "Cut"
}

/// The engine declined to copy the bookmark, in its own words.
#[must_use]
pub fn bookmark_copy_refused(engine: &str) -> String {
    format!("That bookmark could not be copied. {engine}")
}

/// What is on the clipboard, above the Paste button.
#[must_use]
pub fn bookmark_paste_heading(items: usize) -> String {
    if items == 1 {
        "1 bookmark copied.".to_owned()
    } else {
        format!("{items} bookmarks copied.")
    }
}

/// **The warning that must be read BEFORE the press.**
#[must_use]
pub fn bookmark_paste_destinations_dropped(needs: usize, has: usize) -> String {
    format!(
        "Some of these point at page {needs}, and this document has {has}. Those bookmarks will \
         arrive with no destination \u{2014} they will show in the list and do nothing when \
         clicked. Add the pages first if you want them to work."
    )
}

/// Where the paste will land, when a bookmark is selected.
#[must_use]
pub fn bookmark_paste_under(title: &str) -> String {
    format!("They will go under \u{201c}{title}\u{201d}.")
}

/// Where the paste will land, when nothing is selected.
#[must_use]
pub const fn bookmark_paste_at_top_level() -> &'static str {
    "They will go at the top level. Select a bookmark first to file them under it."
}

/// The Paste button.
#[must_use]
pub const fn bookmark_paste_button() -> &'static str {
    "Paste bookmarks"
}

/// **How many arrived without their destination** — reported after the paste.
#[must_use]
pub fn bookmark_paste_dropped(n: usize) -> String {
    if n == 1 {
        "One pasted bookmark points at a page this document does not have, so it arrived with no \
         destination and does nothing when clicked."
            .to_owned()
    } else {
        format!(
            "{n} pasted bookmarks point at pages this document does not have, so they arrived \
             with no destination and do nothing when clicked."
        )
    }
}

/// invites the reading that the pages go too.
#[must_use]
pub const fn bookmark_delete_button() -> &'static str {
    "Remove"
}

/// **The subtree warning, said before the press.**
#[must_use]
pub fn bookmark_delete_takes_subtree(descendants: usize) -> String {
    if descendants == 1 {
        "Removing this also removes the 1 bookmark filed under it.".to_owned()
    } else {
        format!("Removing this also removes the {descendants} bookmarks filed under it.")
    }
}

/// The reassurance that goes with it, and it is a fact rather than comfort.
#[must_use]
pub const fn bookmark_delete_keeps_pages() -> &'static str {
    "The pages themselves are not touched — only the way of jumping to them."
}

/// **What was actually removed**, reported after the fact from the count the
/// engine returned.
#[must_use]
pub fn bookmark_deleted(removed: usize) -> String {
    if removed <= 1 {
        "Bookmark removed.".to_owned()
    } else {
        format!(
            "Bookmark removed, along with the {} filed under it. Undo puts them all back.",
            removed - 1
        )
    }
}

/// Why the Rename button is not offered for a blank name.
#[must_use]
pub const fn bookmark_rename_needs_a_title() -> &'static str {
    "A bookmark needs a name. One with a blank title still appears in the \
     list, as a blank row nothing distinguishes."
}

/// Disclosure when pdfcer's own reader had to give up part-way.
#[must_use]
pub fn bookmarks_truncated() -> &'static str {
    "pdfcer stopped reading this outline early — it loops back on itself or is deeper than pdfcer follows. Some bookmarks are missing from this list."
}

/// An outline item with no title of its own.
#[must_use]
pub fn bookmark_untitled() -> &'static str {
    "(untitled)"
}

/// Tooltip on a bookmark row, naming where it goes.
#[must_use]
pub fn bookmark_row_tooltip(page_number: usize) -> String {
    format!("Go to page {page_number}.")
}

/// A heading bookmark: no destination, by design.
#[must_use]
pub fn bookmark_row_heading_tooltip() -> &'static str {
    "A heading. It groups the bookmarks beneath it and does not point at a page of its own."
}

/// Tooltip on a bookmark that points nowhere pdfcer can resolve.
#[must_use]
pub fn bookmark_row_unresolved_tooltip() -> &'static str {
    "This bookmark points somewhere pdfcer could not resolve — it may use a destination form pdfcer does not read yet, or name a page that is not in this document."
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **The three "cannot tell you" openers are genuinely different
    /// sentences.**
    #[test]
    fn each_structure_panel_leads_with_its_own_limitation() {
        let openers = [
            crate::text::trust::panel_intro(),
            layers_session_only_note(),
            bookmarks_truncated(),
        ];
        for (i, a) in openers.iter().enumerate() {
            for b in openers.iter().skip(i + 1) {
                assert_ne!(a, b);
            }
            assert!(a.len() > 40, "an opener too short to be specific: {a}");
        }
    }

    /// **The Layers note says a toggle changes the VIEW, and not the
    /// document.**
    #[test]
    fn the_layers_note_says_a_toggle_changes_the_view_and_not_the_document() {
        let note = layers_session_only_note();
        assert!(
            note.contains("not the document"),
            "the panel now HAS a visibility control, so the note's whole job is \
             to say that using it does not edit the file: {note}"
        );
        assert!(
            note.contains("Nothing here is saved"),
            "an operator who ticks a layer and closes the document must have \
             been told the tick does not travel with the file: {note}"
        );
        assert!(
            !note.contains("not available"),
            "the S3 clause is back and the control is here — the panel is now \
             denying a capability it ships: {note}"
        );
    }

    /// **Reset says what it returns TO, not merely that it resets.**
    #[test]
    fn the_reset_control_names_what_it_returns_to() {
        let label = layers_reset_label();
        assert!(!label.trim().is_empty());
        assert!(
            !label.ends_with('.'),
            "a label is a name and takes no trailing period: {label}"
        );

        let tip = layers_reset_tooltip();
        assert!(
            tip.contains("the document"),
            "reset must name the state it returns to, or it reads as 'turn \
             everything on': {tip}"
        );
        assert!(
            tip.contains("hidden"),
            "the tooltip has to say that hidden layers go back to hidden — that \
             is the half an operator would otherwise get wrong: {tip}"
        );
    }

    /// **The overridden count is a count of layers, and it is singular at
    /// one.**
    #[test]
    fn the_overridden_count_agrees_with_itself_about_number() {
        assert!(
            layers_overridden(1).starts_with("1 layer "),
            "{}",
            layers_overridden(1)
        );
        assert!(
            layers_overridden(3).starts_with("3 layers "),
            "{}",
            layers_overridden(3)
        );
        for n in [1_usize, 3] {
            assert!(
                layers_overridden(n).contains("the document"),
                "the count is only meaningful against what it differs FROM: {}",
                layers_overridden(n)
            );
        }
    }

    /// **The two override tooltips name the document's own state, and say
    /// opposite things.**
    #[test]
    fn an_overridden_layer_says_which_way_the_document_asked() {
        let doc_shows = layer_overridden_tooltip(true);
        let doc_hides = layer_overridden_tooltip(false);
        assert_ne!(doc_shows, doc_hides);
        assert!(doc_shows.contains("hidden"), "{doc_shows}");
        assert!(doc_hides.contains("shown"), "{doc_hides}");
    }

    /// **The delete disclosure speaks about the SAME quantity, before and
    /// after the press — and the two sentences count differently to do it.**
    #[test]
    fn the_delete_promise_and_the_delete_report_name_the_same_quantity() {
        let under = 11;
        let promised = bookmark_delete_takes_subtree(under);
        assert!(promised.contains("11"), "{promised}");

        // What the engine returns for that same removal: the subtree plus the
        // clicked item.
        let reported = bookmark_deleted(under + 1);
        assert!(
            reported.contains("11"),
            "the report must name the same 11, not the 12 the engine counted: {reported}"
        );
        assert!(
            !reported.contains("12"),
            "the inclusive count must not leak into the sentence: {reported}"
        );
    }

    /// **A leaf reports a removal without a subtree clause.**
    #[test]
    fn a_leaf_removal_says_nothing_about_a_subtree() {
        let leaf = bookmark_deleted(1);
        assert!(!leaf.contains('0'), "{leaf}");
        assert!(!leaf.contains("along with"), "{leaf}");
        assert_ne!(leaf, bookmark_deleted(2));
        // Defensive: a refusal cannot reach this function, but a future verb
        // that reports zero removals must not produce "the -1 filed under it".
        assert_eq!(bookmark_deleted(0), leaf);
    }

    /// **The removal disclosure names the undo**, because that is what
    /// stands in for the confirmation dialog this surface deliberately does
    /// not show.
    #[test]
    fn the_removal_disclosure_names_the_way_back() {
        let said = bookmark_deleted(12);
        assert!(said.to_lowercase().contains("undo"), "{said}");
    }

    /// **The subtree warning is singular for one and plural for the rest.**
    #[test]
    fn the_subtree_warning_agrees_in_number() {
        let one = bookmark_delete_takes_subtree(1);
        assert!(one.contains("1 bookmark "), "{one}");
        assert!(!one.contains("bookmarks"), "{one}");
        assert!(bookmark_delete_takes_subtree(2).contains("2 bookmarks"));
    }

    /// **The pages line says what is NOT being removed.**
    #[test]
    fn the_pages_line_names_the_pages() {
        let said = bookmark_delete_keeps_pages();
        assert!(said.contains("pages"), "{said}");
    }

    /// A bookmark's three destination states read as three different things.
    #[test]
    fn the_three_bookmark_states_are_distinguishable() {
        let go = bookmark_row_tooltip(7);
        let heading = bookmark_row_heading_tooltip();
        let unresolved = bookmark_row_unresolved_tooltip();
        assert!(go.contains('7'), "the destination page must be named: {go}");
        assert_ne!(go, heading);
        assert_ne!(heading, unresolved);
        assert_ne!(go, unresolved);
    }

    /// Counted lines say "1 layer", not "1 layers".
    ///
    /// Cheap to get wrong, immediately visible, and the reason both
    /// functions branch rather than appending an `s`.
    #[test]
    fn counted_lines_are_singular_at_one() {
        assert!(layers_count(1).starts_with("1 layer."));
        assert!(layers_count(2).starts_with("2 layers"));
        assert!(bookmarks_count(1).starts_with("1 bookmark."));
        assert!(bookmarks_count(0).starts_with("0 bookmarks"));
        assert!(layers_auto_managed(1).starts_with("1 layer switches"));
        assert!(layers_auto_managed(3).starts_with("3 layers switch"));
    }

    /// Byte sizes cross their unit boundaries where they should.
    ///
    /// Base 1024, and the boundary cases are where an off-by-one shows up as
    /// `1024 B` sitting above `1.0 KB` in a sorted list.
    #[test]
    fn byte_sizes_use_base_1024_and_switch_units_at_the_boundary() {
        assert_eq!(byte_size(0), "0 B");
        assert_eq!(byte_size(1023), "1023 B");
        assert_eq!(byte_size(1024), "1.0 KB");
        assert_eq!(byte_size(1024 * 1024 - 1), "1024.0 KB");
        assert_eq!(byte_size(1024 * 1024), "1.00 MB");
    }

    /// The two signature-coverage sentences state opposite facts and must
    /// not read alike.
    #[test]
    fn full_coverage_and_a_tail_read_as_different_answers() {
        let full = signature_covers_whole_file(4096);
        let tail = signature_leaves_tail(4096, 512);
        assert_ne!(full, tail);
        assert!(full.contains("4096"));
        assert!(tail.contains("4096") && tail.contains("512"));
        // The warning has to name the consequence, not just the numbers.
        assert!(tail.contains("does not protect"));
    }
}
