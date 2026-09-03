//! # `text::pages` — every string the Pages panel shows
//!
//! One area of the catalog described in [`crate::text`]'s header, consumed by
//! [`crate::panels::pages`] — the grid, its captions and its tile states — and
//! by [`crate::app::actions::pages`], which words what a page **delete** broke.
//! Those are the only two readers.
//!
//! The second joined the first rather than getting a module of its own because
//! both are sentences about *pages* in the same vocabulary — sheets, page
//! numbers, this document — and a reader who came here for one half would not
//! find the other. See the disclosure section at the foot of this file for the
//! rule-4 obligation those strings discharge.
//!
//! It is a sibling of [`crate::text::panels`] rather than a module inside it
//! for the same reason [`crate::text::forms`] is: that directory's own header
//! declares it covers *"the three document-structure panels"* and their two
//! inspector siblings, and the Pages panel is neither. It is a **navigator**
//! whose copy is about pictures, page geometry and the cost of drawing —
//! vocabulary that has nothing in common with a font inventory or a signature
//! byte range, and that would be read past by anyone maintaining either.
//!
//! ## ★ The posture: an undrawn thumbnail must SAY it is undrawn
//!
//! This is the whole reason half the strings below exist, and it is the
//! project's no-placeholders rule (`RIBBON_IA.md` P3) applied to a picture
//! rather than to a control.
//!
//! A page thumbnail that has not been rasterized yet is, on screen, a
//! rectangle. A rectangle the colour of paper **is a picture of an empty
//! page** — and an empty page is a thing a real PDF can contain. So a
//! thumbnail grid that draws blank rectangles while it works is not
//! "loading"; it is *asserting something false about the document*, and the
//! operator has no way to tell the two apart. The old shell drew exactly that
//! (`main.rs`'s `thumbnail_rail`: a bordered rect in `extreme_bg_color` with
//! the page number), and it is the one part of that rail this panel did not
//! carry across.
//!
//! Every state a tile can be in therefore has **words**:
//!
//! | State | String | Says |
//! |---|---|---|
//! | queued, previews on | [`thumbnail_not_drawn_yet`] | *this is not a picture of the page yet* |
//! | previews off | [`thumbnail_previews_off`] | *and it will not become one until you say so* |
//! | the render hit the time ceiling | [`thumbnail_abandoned`] | *pdfcer started and stopped* |
//! | the page would not draw | [`thumbnail_failed`] | *this page is the problem, not the panel* |
//!
//! No spinner, and that is deliberate rather than an omission: a dozen
//! spinning icons is motion, not information, and only one page is ever
//! being drawn at a time anyway.
//!
//! ## Conventions, restated from [`crate::text`] because they bind here
//!
//! - **Sentence case, no trailing period on labels; full sentences with
//!   punctuation for prose.**
//! - **Name the thing and what the operator can do about it.**
//!   [`previews_paused_note`] is the worked example: it names the page, the
//!   measured cost, and the control that resumes.
//! - **Never state a capability the build does not have.**

/// The document's page count, as the panel's first line.
///
/// Singular and plural are spelled out rather than assembled with a `(s)`,
/// which reads as a form field rather than as a sentence. One page is a
/// reachable and perfectly ordinary case — most drawings are one sheet.
#[must_use]
pub fn pages_count(total: usize) -> String {
    if total == 1 {
        "1 page".to_owned()
    } else {
        format!("{total} pages")
    }
}

/// How many pages the operator has picked, shown only when that is not zero.
///
/// ★ **This number is the operand list the ribbon's Pages tab already
/// promises.** Every one of those commands' tooltips says *"the selected
/// pages"* — `pages.delete` is *"Remove the selected pages from this
/// document"* — so the count here is not decoration, it is the answer to
/// *"selected where?"* that the tooltips leave open. Wording it as a plain
/// count rather than as an instruction keeps it a statement of fact.
#[must_use]
pub fn pages_selected(selected: usize) -> String {
    if selected == 1 {
        "1 page selected".to_owned()
    } else {
        format!("{selected} pages selected")
    }
}

/// ★ **Where a drag will put the pages it is carrying** — the sentence beside
/// the insertion caret.
///
/// The caret says *where* graphically; this says it in words, and both are
/// needed. A caret between two tiles in a scrolling grid of near-identical
/// drawing sheets is precise and not **checkable**: an operator cannot read a
/// page number off a hairline. The Insert-from-file dialog reached the same
/// conclusion for the same reason and its comment is the precedent — *"the
/// dialog is centred over a document the operator may have scrolled: the
/// number is what makes the choice checkable."*
///
/// # ★ The vocabulary is the Insert dialog's, deliberately
///
/// *Before page N*, *the start*, *the end* — the same three phrasings the
/// insert-position radios use, because a drag and an insert answer the same
/// question about the same document and two different vocabularies for one
/// destination is how an operator ends up unsure whether they mean the same
/// thing. `gap` is a **gap index** in `panels::pages::ops`' sense: `0` is
/// before the first sheet, `page_count` is after the last.
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
///
/// ★ **Said rather than shown by absence.** The alternative is to hide the
/// caret when the drop would not land, and that is worse: an operator whose
/// caret has vanished cannot tell *"this drop does nothing"* from *"the panel
/// has stopped tracking my pointer"*. The caret is dimmed and this sentence
/// explains the dimming.
///
/// It is the ordinary state at the beginning of every drag — a block hovering
/// over itself — so the wording has to read as information rather than as a
/// refusal.
#[must_use]
pub const fn drag_lands_nowhere() -> &'static str {
    "Release here and the order does not change — drag to a boundary outside \
     the pages you picked up."
}

/// A document with a page tree that resolved to nothing.
///
/// Rare and not impossible: a damaged `/Pages` node can flatten to an empty
/// vector while the file still opens. Saying so beats an empty grid, which
/// reads as a panel that failed rather than as a document that is empty.
#[must_use]
pub fn pages_none() -> &'static str {
    "This document has no pages."
}

/// A tile's caption — the page number an operator would say out loud.
///
/// **1-based.** Everything inside pdfcer indexes from 0; a human counts from
/// 1, and the conversion happens here and at no other point in the panel, so
/// there is exactly one place the off-by-one could be.
#[must_use]
pub fn page_number(page_index: usize) -> String {
    format!("{}", page_index + 1)
}

/// A tile's tooltip: which page, how big it is, and what a click does.
///
/// The page size is in **millimetres**, from the page's own extent in points
/// — because the operator this panel is for is looking at a drawing sheet
/// set, and "A1" or "841 × 594" is how they identify a sheet. It is also the
/// one useful fact a tile can carry before its picture exists, which is why
/// the tooltip is worth having on an undrawn tile at all.
///
/// The gestures are named because none of them is discoverable: nothing on
/// screen says that Ctrl adds to the selection.
#[must_use]
pub fn page_tile_tooltip(page_index: usize, width_mm: f32, height_mm: f32) -> String {
    format!(
        "Page {} — {width_mm:.0} × {height_mm:.0} mm. Click to go there, \
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
///
/// Distinct from [`thumbnail_not_drawn_yet`] on purpose. "Not drawn yet"
/// promises a picture is coming; with previews off, none is, and an operator
/// waiting for one that will never arrive has been misled by a word.
#[must_use]
pub fn thumbnail_previews_off() -> &'static str {
    "Preview off"
}

/// A tile whose render pdfcer started and abandoned.
///
/// Reachable when the document is edited, or the panel closed, while a page
/// is being drawn. Not a failure — nothing is wrong with the page — so it
/// must not read like one.
#[must_use]
pub fn thumbnail_abandoned() -> &'static str {
    "Not finished"
}

/// A tile whose page the renderer refused.
///
/// Names the *page* as the subject, because that is what is true: the panel
/// works, and this one page did not draw. The canvas will say the same thing
/// at more length if the operator navigates to it, which is the right place
/// for the detail.
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
///
/// ★ **The number in this sentence is measured, not estimated.**
/// `BENCHMARK.md` records a real CAD drawing whose content stream costs
/// ~0.74 s to interpret *at any scale* — a one-by-one-**point** region of it
/// costs 691 ms — so a thumbnail of such a page is not cheap merely because
/// it is small. That is the single most surprising fact about this panel and
/// it belongs where the operator meets it.
#[must_use]
pub fn previews_tooltip() -> &'static str {
    "Draw a picture of each page. A dense drawing can take most of a second \
     per page whatever size it is drawn at, because the cost is in reading \
     the page rather than in filling the pixels — so pdfcer stops on its own \
     when it meets one."
}

/// Why previews stopped, and what resumes them.
///
/// Named parts: the page that was slow, what it cost, and the control. A
/// message that said only "previews paused" would leave the operator hunting
/// for a cause pdfcer already knows.
///
/// The cost is printed in **seconds to one decimal** rather than in
/// milliseconds, because the number's job here is to justify a decision, and
/// "0.8 s" justifies it in a way "812 ms" does not.
#[must_use]
pub fn previews_paused_note(page_index: usize, millis: u128) -> String {
    let seconds = millis as f32 / 1000.0;
    format!(
        "Page previews stopped: page {} took {seconds:.1} s to draw. \
         Turn “{}” back on to carry on drawing them.",
        page_index + 1,
        previews_label()
    )
}

// ---------------------------------------------------------------------------
// ★ THE PAGE VERBS' DISCLOSURES — what a delete broke, in words
//
// A second audience for this module, and the header's *"consumed by
// `crate::panels::pages` and by nothing else"* is now *"and by
// `crate::app::actions::pages`, which words what a page delete broke"*. The
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
///
/// Singular and plural spelled out rather than assembled with `(s)`, exactly
/// as [`pages_count`] does and for the same reason: one broken bookmark is a
/// perfectly ordinary outcome of deleting one page, and `1 bookmark(s)` reads
/// as a form field rather than as a sentence.
#[must_use]
pub fn deleted_dangling_bookmarks(count: usize) -> String {
    if count == 1 {
        "1 bookmark now points at a page that is no longer in this document.".to_owned()
    } else {
        format!("{count} bookmarks now point at pages that are no longer in this document.")
    }
}

/// Links on **surviving** pages whose destination page was removed (§12.5.6.5).
///
/// "on the pages that remain" is load-bearing: links that left with their own
/// page are deliberately not counted by the engine, because reporting them
/// would inflate the number with references that no longer exist to be broken.
/// The sentence says which set it is talking about so the number can be
/// trusted.
#[must_use]
pub fn deleted_dangling_links(count: usize) -> String {
    if count == 1 {
        "1 link on the pages that remain points at a page that was removed.".to_owned()
    } else {
        format!("{count} links on the pages that remain point at pages that were removed.")
    }
}

/// Named destinations (§12.3.2.3) that resolved to a removed page.
///
/// Named destinations are reached from *outside* this document as well as from
/// within it — another PDF's link, a URL fragment, a script — which is why they
/// are disclosed separately from bookmarks rather than added to that count.
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
///
/// A sentence rather than a count, because the underlying fact is a boolean:
/// the tree is one object and the operator's question is *"are my page numbers
/// wrong now?"*.
///
/// It says pdfcer left them **deliberately**, because the alternative reading —
/// that pdfcer failed to update them — invites the operator to report a bug
/// against behaviour that matches Acrobat's and was chosen. Acrobat leaves them
/// stale and silent; this is the "and says so" half.
#[must_use]
pub fn deleted_page_labels_stale() -> &'static str {
    "This document numbers its own pages, and those numbers were left as they were — the \
     sheets that remain still carry the labels they had before the deletion."
}

/// Preseparated page sets (§14.11.4) that lost at least one plate.
///
/// The one class of broken reference the engine **repairs** rather than
/// reporting, and it is still disclosed for exactly that reason: something in
/// the file changed that the operator did not ask for. `DeleteOutcome`'s own
/// docs draw the line — a bookmark's target is a question about *authorial
/// intent* that pdfcer must not guess at, while a separation dictionary's
/// `/Pages` array is a *structural* fact pdfcer knows the answer to.
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
///
/// Not the Open dialog's title. The two pick a PDF and mean opposite things —
/// one replaces what is on screen, the other adds to it — and a picker headed
/// *"Open a PDF"* over a document the operator is editing is a sentence that
/// says the wrong thing at the moment they are most likely to read it.
#[must_use]
pub const fn insert_dialog_title() -> &'static str {
    "Insert pages from a PDF"
}

/// ★ **What arrived, and the two different ways the rest did not.**
///
/// # ★★ This sentence was WRONG for two hours, and the correction is the point
///
/// It read: *"Bookmarks, form fields and page labels from that file did not
/// come across — its pages did."* Three nouns, one verb, and the verb is true
/// of two of them and **false of the third**.
///
/// Measured rather than assumed, after the operator asked why the structures
/// could not simply be re-added: a source with **12 form fields** inserted into
/// a blank document produces **13 widget annotations and no `/AcroForm` at
/// all**. The widgets came across — `insert_pages` copies everything reachable
/// from the page, and a page's `/Annots` reaches its widgets. What did not come
/// across is the **field tree that names them**.
///
/// So they are not absent. They are **orphaned**: boxes that draw exactly like
/// form fields, that an operator will click on, and that nothing can fill
/// because no field claims them. That is this project's own *"visible control,
/// silently inert"* failure arriving through a document instead of a ribbon —
/// and the old sentence would have sent an operator looking for the missing
/// fields rather than at the ones in front of them.
///
/// **A disclosure that names the wrong failure is worse than none**, because it
/// is believed. It was written from the engine's summary — *"does not merge the
/// source's document-level structures"* — which is accurate about `/AcroForm`
/// and says nothing about the widgets, and I did not check.
///
/// # The three fates, which is what the sentence now distinguishes
///
/// | structure | what happens |
/// |---|---|
/// | pages, content, resources, fonts | copied at fresh object numbers |
/// | **form fields** | widgets **arrive**, `/AcroForm` does not — inert boxes, and the engine now **counts** them exactly |
/// | bookmarks, page labels, named destinations | genuinely absent |
///
/// # Why the page number is 1-based
///
/// *"after page 7"* is the sheet the operator was looking at, in the numbering
/// the page box and the thumbnails use. A 0-based index here would be the only
/// place in the application that counted differently.
/// # ★ The orphan clause is now a NUMBER, and it is exact
///
/// This sentence used to hedge — *"**Any** form fields on those pages arrived
/// as boxes…"* — because the shell had no way to know whether there were any,
/// so a document with no form controls got a paragraph about form controls.
///
/// `EditSession::insert_pages` returns `InsertOutcome { pages_inserted,
/// orphaned_widgets }` as of 2026-08-19, and the engine's reply is explicit
/// that the count is **exact rather than an upper bound**: `/AcroForm` is
/// document-level and is not merged, and the copy remaps every object number,
/// so no field in the target can be claiming a widget that has just arrived.
/// *"There is no case where a counted widget turns out to have an owner, and
/// you can put the number in front of an operator without hedging it."*
///
/// So `orphans == 0` drops the clause entirely. That is not a cosmetic saving:
/// a sentence about form controls on a drawing with none trains the operator
/// to stop reading the sentence, and the drawings this application is for
/// almost never have any.
///
/// # ★ And the clause is PERMANENT, which the wording has to survive
///
/// The engine over-ruled the framing this shell filed it under. It was
/// proposed as *"the count now, carrying the definitions later"*, and the
/// answer was that a field's widgets can be **split** across inserted and
/// non-inserted pages — so a residue survives *any* merge and the count exists
/// for ever. `Pass 102.1` will reduce the number and can never make it always
/// zero.
///
/// The sentence is therefore worded as a fact about what arrived, not as an
/// interim apology for a feature that is coming.
/// What an insert did to the two document-level structures that do not travel
/// with a page.
///
/// # ★★ Three booleans, and there are three because the REMEDIES differ
///
/// The engine's ruling on the two label fields, adopted verbatim and extended
/// to the third: *"a stale tree wants renumbering; a dropped one wants
/// creating. A single 'page labels are wrong' message names neither, which is
/// why I did not merge them."*
///
/// | field | what is true | what the operator would do about it |
/// |---|---|---|
/// | `outline_dropped` | the source file had bookmarks and they did not come | re-create them in the Bookmarks panel |
/// | `labels_dropped` | the source file had its own page numbering and it did not come | accept this document's numbering, or author one |
/// | `labels_stale` | **this** document's numbering now points at different sheets | renumber the ranges |
///
/// The third is the one worth having and the one this shell would never have
/// asked for. The first two are about a file the operator has finished with;
/// `labels_stale` is about the document **in front of them**, and it says the
/// page numbers they are looking at have quietly stopped describing the pages
/// they are on.
///
/// # ★ Why the outline case is a boolean and not "always"
///
/// Because it used to be "always", and that made it a **disclaimer rather than
/// a disclosure**. The sentence said *"Bookmarks and page labels from that file
/// did not come across"* on every insert, including a CAD drawing whose source
/// had neither — a paragraph about two things that never existed, which is how
/// an operator learns to stop reading the sentence that also carries the clause
/// about form controls.
///
/// It was unconditional only because nothing reported the fact, not because it
/// was always true. `source_outline_dropped` shipped on request the same day.
///
/// The engine's note on *why* bookmarks never came is kept here because it is
/// what makes the remedy obvious: `/Outlines` is a **catalog** entry,
/// unreachable from any page, so a copy that walks outward from the pages never
/// sees it. They are not lost in transit — they were never in the set of
/// objects being copied. Which is why carrying them means replaying the source
/// outline through `add_outline_item`, i.e. exactly what the Bookmarks panel
/// now does by hand.
///
/// # ★ Why pdfcer deliberately does not match Acrobat on page labels
///
/// Carried in this type's own documentation, because the first review question
/// about a *"pdfcer wrote nothing"* disclosure is always *"what does Acrobat
/// do?"* — and the answer being **something worse** is what nobody would guess.
///
/// Acrobat does neither of the two things anyone assumes. It does not carry the
/// source's labels and it does not leave the inserted pages unlabelled: it
/// **overwrites every inserted page with a static copy of the label on the page
/// preceding the insertion point**. Not incrementing — the same string on all
/// of them. The engine sourced three independent Adobe Community threads
/// (2024-2025) in which a twelve-page chapter labelled `10-1`...`10-12`,
/// inserted after a page labelled `9-45`, came out with **all twelve showing
/// `9-45`**. Those threads are complaints about it.
///
/// So matching it would be matching a defect. This type is what makes *"pdfcer
/// wrote nothing"* a stated choice rather than a silence.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Structures {
    /// The source document had bookmarks, and they did not come across.
    pub outline_dropped: bool,
    /// The source document had page labels, and they did not come across.
    pub labels_dropped: bool,
    /// This document's own page-label ranges now describe different sheets.
    pub labels_stale: bool,
}

/// # ★★ TWO numbers, because they are two different pieces of news
///
/// `orphaned_widgets_unrecoverable` arrived hours after `orphaned_widgets`,
/// with the engine's own correction of the sentence it had suggested the day
/// before. It measured its own output — 13 orphans from a real AcroForm — and
/// found two shapes:
///
/// | shape | of 13 | registering it |
/// |---|---|---|
/// | **merged field-widget** (§12.7.3.1) | 11 | recovers the field exactly |
/// | **bare kid** (a radio group member) | 2 | **impossible** — its identity is not in this file |
///
/// The engine's words: the undifferentiated sentence *"is true of both rows and
/// useful for only one."* For the 11 it describes **a chore this shell can now
/// offer to complete**. For the 2 it describes **a permanent loss** whose only
/// remedy is going back to the source file — and the combined total says the
/// milder of the two, which is the wrong way for a disclosure to be wrong.
///
/// So the clause splits, and each half is dropped when its number is zero. A
/// document with 11 recoverable and 0 unrecoverable gets one sentence with a
/// route in it. A document with 0 and 2 gets one sentence with no route,
/// because there is none.
///
/// # ★ The recoverable clause names WHERE, and that is the whole point of it
///
/// *"Forms ▸ Tab order lists them"* is the difference between a disclosure and
/// a complaint. Before `EditSession::adopt_widget` shipped there was nothing to
/// name, so the sentence could only report the damage; now the panel that
/// already computes exactly this set can register them one press at a time, and
/// a disclosure that stops short of saying so leaves the operator with a
/// correct description of a problem and no idea it is solvable.
///
/// # Why the unrecoverable clause says "re-insert from the original"
///
/// Because it is the only thing that works, and because the alternative an
/// operator would otherwise try — typing a name into the box — produces
/// something that *looks* like success. A named bare kid is a new, empty,
/// typeless field. It is not the radio button that was lost, and an operator
/// who believes it is will go looking for its group. See
/// [`crate::text::status::adopt_declined_no_name`], which refuses the word
/// *restore* for the same reason.
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
    // ★ Two forms, because "Nor did..." needs something to follow.
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
    // ★ LAST of the three, and about THIS document rather than the source.
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
    // ★★ "N MORE" only works when something came before it.
    //
    // Found by a driven run, not by reading: on a source whose orphans are ALL
    // unrecoverable the re-registering clause is skipped, and the sentence came
    // out *"Inserted 2 pages after page 2. 3 more lost their field definitions
    // entirely..."* — more than what? It reads as a sentence with one deleted
    // in front of it.
    //
    // ★ This is the SECOND continuation-clause defect in this one function, and
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
///
/// `detail` is `pdfcer-core`'s own error `Display`, passed through for the same
/// reason [`crate::text::canvas_render_failed`] passes one through: those
/// errors are specific, and replacing one with *"could not open the file"*
/// discards the half that says whether it was encrypted, truncated or not a
/// PDF at all.
///
/// **Says nothing was inserted.** A failure part-way through a multi-page
/// insert would otherwise leave the operator wondering whether some of it
/// landed; the verb is one command and either records it or does not.
#[must_use]
pub fn insert_failed(detail: &str) -> String {
    format!("Nothing was inserted. {detail}")
}

/// The chosen file has no pages to insert.
///
/// A separate sentence from [`insert_failed`] because it is not a failure: the
/// file opened, it is a valid PDF, and it is empty. Collapsing the two would
/// send an operator looking for corruption in a file that has none.
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
///
/// The **count is the point**. An operator about to insert a document they
/// picked from a folder has no other way to know whether it is the four-page
/// revision or the forty-page one, and finding out afterwards means an undo.
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
///
/// It states the two behaviours that surprise people, because both are useful
/// here rather than merely tolerated: a range is a **sequence**, so `3,1-2`
/// inserts page 3 first, and it is not de-duplicated, so `1,1` inserts a page
/// twice. An operator who wants either has no other way to ask for it in one
/// gesture.
#[must_use]
pub const fn insert_range_hint() -> &'static str {
    "e.g. 1-4 or 3,1-2. They are inserted in the order you type, and a page may be listed twice."
}

/// The range does not name any page of the source.
///
/// One sentence for every way it can fail — a number past the end, a backwards
/// range, a typo — because the remedy is the same for all of them and the
/// source's own page count is already on screen two lines above.
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
///
/// The count is on the control rather than beside it, for the reason the print
/// dialog's commit button carries its clip count: it is on the thing the
/// operator's hand is already on, where it cannot be looked past.
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

// ===========================================================================
// MERGING A WHOLE DOCUMENT IN — `EditSession::merge_document`, wired 2026-08-28
//
// ★★★ Why this is a different verb from an insert, and not a convenience over
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
///
/// # ★★★ Two renames, and both are disclosures rather than warnings
///
/// **`fields_renamed`** — a field whose name was already taken here arrives
/// under a different one. The engine's note on why that is still the right
/// behaviour is worth carrying: *"two fields sharing a fully qualified name are
/// ONE field, and filling either fills both."* But the operator now has a field
/// whose name is not the one the source document showed them, and **any script,
/// FDF or calculation keyed on the old name no longer matches it.** Nothing
/// else would tell them.
///
/// **`named_destinations_renamed`** — the same for §12.3.2.3 destinations, with
/// a second consequence the engine spells out and this sentence must not lose:
/// pdfcer rewrites the *carried* bookmarks to the new keys, and **cannot rewrite
/// a link in a document it did not copy.** An outside `/GoToR` reference to the
/// old key now resolves to *this* document's destination rather than the
/// source's — a link that still works and goes somewhere else.
///
/// ★ Each clause appears only when its count is non-zero. A clean merge of a
/// form-free drawing set says one thing: how many pages arrived.
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

    /// ★ **A document with no form controls gets no sentence about form
    /// controls.**
    ///
    /// The clause used to be unconditional — *"**Any** form fields on those
    /// pages arrived as boxes…"* — because the shell had no count and had to
    /// hedge. `InsertOutcome::orphaned_widgets` arrived on 2026-08-19 and the
    /// engine's reply says the number is **exact rather than an upper bound**,
    /// so a zero can be believed.
    ///
    /// Worth a test rather than a glance, because the failure is silent in the
    /// direction that matters: a paragraph about form controls on a drawing
    /// that has none trains the operator to stop reading the sentence, and the
    /// sheets this application is for almost never have any. The clause that
    /// gets skipped is then the *bookmarks* one, which is always true.
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

    /// ★★ A source that had nothing to lose is told nothing about losing it.
    ///
    /// The sentence used to end *"Bookmarks and page labels from that file did
    /// not come across"* on **every** insert. On a CAD drawing whose source had
    /// neither — which is most of them, in this application — that is a
    /// paragraph about two things that never existed.
    ///
    /// It is worth a test rather than a glance because the cost is not the
    /// wasted words. It is that the same sentence carries the clause about
    /// orphaned form controls, which is *actionable*, and an operator who has
    /// learned that this sentence is boilerplate stops reading the part that is
    /// not.
    ///
    /// The clause was unconditional only because nothing reported the fact —
    /// which is the difference between a disclosure and a disclaimer.
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

    /// ★★ The stale-label clause is about THIS document, and it is last.
    ///
    /// The one fact in this sentence that describes the sheets in front of the
    /// operator rather than a file they have finished with: their own page
    /// numbers have quietly stopped describing the pages they are on.
    ///
    /// Ordered last on purpose — a sentence whose most actionable clause comes
    /// first is read and then abandoned at the part about a document nobody is
    /// looking at any more. Asserted, because ordering is exactly the kind of
    /// decision a later edit undoes without noticing.
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
    ///
    /// The singular is its own arm rather than an `(s)`: one orphaned control
    /// is an ordinary case — a single signature field on a title sheet — and
    /// *"1 form controls"* is the shape that makes an operator distrust the
    /// number beside it.
    ///
    /// ★ The route is asserted, not just the count. A disclosure that reports a
    /// solvable problem without saying it is solvable leaves the operator with
    /// a correct description and nothing to do with it.
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

    /// ★★ The two counts are two sentences, and the recoverable one is the
    /// **difference**, not the total.
    ///
    /// The engine's correction, asserted. `orphaned_widgets` counts every
    /// orphan; `orphaned_widgets_unrecoverable` counts the subset whose field
    /// identity is not in this file at all. Reporting the total as
    /// re-registerable would send the operator to a panel where two of the
    /// boxes refuse, with a sentence that had promised otherwise.
    ///
    /// The measured case is the fixture: 13 orphans, 2 of them bare kids.
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
    ///
    /// R9's rule applied to prose: *"0 form controls need re-registering"* is a
    /// placeholder wearing a number, and it would be sitting immediately beside
    /// a sentence saying two of them are gone for good.
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

    /// ★★ The unrecoverable clause reads correctly with NOTHING before it.
    ///
    /// Found by a driven run rather than by reading. On a source whose orphans
    /// are all bare kids the re-registering clause is skipped, and the sentence
    /// came out *"Inserted 2 pages after page 2. **3 more** lost their field
    /// definitions entirely…"* — more than what? It reads as a sentence with
    /// one deleted in front of it, which is exactly how an operator concludes
    /// the program is losing text.
    ///
    /// ★ It is the **second** continuation-clause defect in this one function.
    /// The first — *"Nor did its page numbering"* with no bookmarks clause
    /// before it — was fixed an hour earlier, three clauses up, and the sweep
    /// that should have followed it did not happen. Both arms are now asserted
    /// together so the next one cannot be fixed alone.
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
    ///
    /// The invariant says it cannot happen — the second field counts a subset
    /// of the first — and the subtraction saturates anyway. A disclosure runs
    /// at the end of a *successful* edit, which is the worst possible moment to
    /// panic on an arithmetic assumption about another crate's struct.
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
