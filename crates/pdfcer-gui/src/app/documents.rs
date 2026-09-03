//! # `app::documents` — more than one document open at once
//!
//! The operator's request, 2026-08-19, verbatim:
//!
//! > *"make it so we can open multiple PDFs at once and drag and drop pages
//! > from one thumbnail image sidebar to another or onto the canvas to add
//! > pages and insert them in between the pages we've dragged to on the canvas
//! > or the thumbnail preview area."*
//!
//! This file is the first half of that: **what it means for several documents
//! to be open**. The tab strip that shows them is [`crate::app::doctabs`], the
//! page drag between them is [`crate::panels::pages`], and the drop onto the
//! page view is [`crate::canvas::pagedrop`].
//!
//! ---
//!
//! ## 1. The shape: one active document, and the rest parked
//!
//! [`PdfcerApp::status`] is unchanged and still means *the document the
//! operator is looking at*. Everything else — every panel, the canvas, the
//! status bar, the ribbon's condition set, the find bar — reads that one field
//! and did not have to change. Beside it now sits `PdfcerApp::parked`: the
//! other open documents, in tab order with the active one removed, and
//! `PdfcerApp::active_slot`, the position the active document occupies in that
//! order.
//!
//! So the operator's tab strip, left to right, is
//!
//! ```text
//! parked[0] … parked[active_slot-1]   status        parked[active_slot] … parked[n-1]
//!  slot 0        slot active_slot-1   slot           slot active_slot+1     slot n
//!                                     active_slot
//! ```
//!
//! ### ★ Why this and not `Vec<Status>` with an index
//!
//! Because the alternative costs a hundred edits to buy nothing. `self.status`
//! is named in 105 places, and a large number of them are **split borrows** —
//! `let Status::Open(doc) = &mut self.status` taken in the same expression as
//! `&self.find`, `&mut self.dialogs`, `&self.commands`. Rust splits borrows of
//! struct *fields*; it does not split the borrow a `fn active_mut(&mut self)`
//! accessor takes. Replacing the field with a method would have turned every
//! one of those sites into a borrow error to be worked around individually,
//! and a borrow-checker workaround written a hundred times is a hundred
//! chances to change behaviour by accident.
//!
//! It is also the shape [`crate::app::PdfcerApp`]'s own header predicted, in
//! those words, before any of this was built: *"It will grow — settings, the
//! command log, the dock layout, **the parked documents**"*.
//!
//! The one thing it costs is that the tab order is expressed in two fields
//! rather than one, so every reordering operation goes through
//! [`PdfcerApp::take_slots`] / [`PdfcerApp::put_slots`], which flatten to a
//! single `Vec` and rebuild. Those two functions are the only code in the
//! application that knows the encoding, and they are a handful of lines each.
//!
//! ---
//!
//! ## 2. What counts as a tab
//!
//! **Every [`Status`] except [`Status::Empty`].** A file that failed to open,
//! one pdfcer does not support, and one waiting for a password each get a tab
//! that says so — the same way a browser tab survives a failed page load. The
//! alternative (only `Status::Open` gets a tab) would mean an operator who
//! opened four files and had one fail would find themselves looking at an
//! error with no way back to the other three, because the error would have
//! replaced whatever was active.
//!
//! [`Status::Empty`] is therefore not a document but the **absence of all
//! documents**, and the invariant that makes the encoding total is:
//!
//! > If `parked` is non-empty, `status` is not [`Status::Empty`].
//!
//! [`PdfcerApp::document_count`] is the one predicate that reads it, and it is
//! what every other function here asks rather than testing the fields.
//!
//! ---
//!
//! ## 3. Opening the same file twice activates the tab it is already in
//!
//! Acrobat, Word, VS Code and every browser do this, so pdfcer does. The
//! alternative is two tabs over one path, two independent `EditSession`s, two
//! undo stacks, and a save from either silently discarding the other's work —
//! which is a correctness problem wearing a usability problem's clothes.
//!
//! Matched on the path as stored, which is already absolutised for anything
//! that came through the recent list. A created document
//! ([`crate::app::state::Origin::Created`]) never matches, because its path is
//! a *name* rather than a location and two `Untitled 2.pdf` documents cannot
//! exist anyway — the counter never repeats within a session.
//!
//! ---
//!
//! ## 4. What switching documents forgets, and what it must not
//!
//! Exactly what [`PdfcerApp::close_document`] forgets, minus the document:
//!
//! | forgotten | why |
//! |---|---|
//! | the panels' view state | expansion sets and the Properties focus are **paint-order indices** into one page of one revision. Carried to another document they name different objects, confidently. |
//! | the find hits | a hit is a page index and a page-space rectangle. The epoch test that catches an *edit* cannot catch a *different document* — the other one's `edit_epoch` may match by coincidence. |
//! | the de-duplicated trace slots | so the document switched *to* re-declares its canvas line and its regions instead of inheriting them because the numbers happened to agree. |
//!
//! And what it must **not** forget, which is the part that makes tabs worth
//! having at all:
//!
//! - **The parked document's view.** Its page, zoom, scroll, fit and overlay
//!   state live on its own `OpenDoc` and are simply moved aside. Coming back
//!   to a tab puts you where you left it. This is why switching does **not**
//!   call `Prefs::seed_view` the way [`PdfcerApp::open_path`] does — that seeds
//!   a *new* document from the opening preferences, and applying it here would
//!   throw away the operator's place every time they glanced at another sheet.
//! - **The parked document's rasters.** A parked `OpenDoc` keeps its page
//!   texture and its strip cache. That is memory spent deliberately:
//!   `BENCHMARK.md` measures the benchmark CAD drawing at **877 ms** for one
//!   full-page render, so dropping the texture on park would make every tab
//!   switch a visible stall — which is the one thing a tab strip promises not
//!   to be. If this ever needs bounding it should be bounded by a *count of
//!   parked documents that keep rasters*, not by dropping them all.
//! - **The recent list, the dock arrangement and the mode**, for
//!   [`PdfcerApp::close_document`]'s reasons, unchanged.
//!
//! ---
//!
//! ## 5. Closing
//!
//! [`PdfcerApp::close_slot`] removes one tab. Which tab becomes active
//! afterwards is the browser rule, because every operator already has it:
//!
//! - closed a tab **left** of the active one → the same document stays active
//!   (its index shifts down by one)
//! - closed a tab **right** of it → unchanged entirely
//! - closed **the active** one → the tab that was to its right takes its
//!   place, or the new last tab if it was the rightmost
//!
//! Closing the last document leaves [`Status::Empty`], which is exactly where
//! the application starts, so nothing downstream needs a second empty state.
//!
//! **The unsaved-edits question is asked by the caller, not here.** See
//! [`crate::app::actions::document`], whose header carries the guard table and
//! whose test enumerates the arms that must ask. This module moves documents
//! around; it does not decide whether the operator meant it.

use crate::app::PdfcerApp;
use crate::app::state::{Origin, Status};

impl PdfcerApp {
    /// **How many documents are open**, which is how many tabs are drawn.
    ///
    /// `0` and only `0` means [`Status::Empty`] with nothing parked — see this
    /// module's §2 for why that is the one state without a tab.
    #[must_use]
    pub fn document_count(&self) -> usize {
        if self.parked.is_empty() && matches!(self.status, Status::Empty) {
            0
        } else {
            self.parked.len() + 1
        }
    }

    /// The document in tab position `slot`, or `None` past the end.
    ///
    /// The read half of the encoding described in §1. Written out rather than
    /// routed through [`Self::take_slots`] because it must not move anything:
    /// the tab strip calls it once per tab per frame.
    #[must_use]
    pub fn slot(&self, slot: usize) -> Option<&Status> {
        if self.document_count() == 0 {
            return None;
        }
        match slot.cmp(&self.active_slot) {
            std::cmp::Ordering::Equal => Some(&self.status),
            std::cmp::Ordering::Less => self.parked.get(slot),
            std::cmp::Ordering::Greater => self.parked.get(slot - 1),
        }
    }

    /// **Flatten the two fields into one tab-ordered vector**, leaving the
    /// application document-less.
    ///
    /// Half of the only code that knows the encoding. Always paired with
    /// [`Self::put_slots`] inside the same function — leaving the application
    /// in the state this returns would show an empty shell with the documents
    /// still alive on the stack.
    ///
    /// Returns an **empty** vector when nothing is open, rather than
    /// `[Status::Empty]`: the caller wants a list of documents, and a list
    /// containing "no documents" is the bug this early return removes.
    fn take_slots(&mut self) -> Vec<Status> {
        if self.document_count() == 0 {
            return Vec::new();
        }
        let mut all = std::mem::take(&mut self.parked);
        let active = std::mem::replace(&mut self.status, Status::Empty);
        let at = self.active_slot.min(all.len());
        all.insert(at, active);
        all
    }

    /// **Put a tab-ordered vector back**, with `active` as the one on screen.
    ///
    /// The other half. An empty vector is the legitimate way to say *"nothing
    /// is open now"* and restores [`Status::Empty`] — which is what makes
    /// closing the last tab need no special case anywhere else.
    ///
    /// `active` is clamped rather than asserted. Every caller computes it from
    /// a length that has just changed, and an off-by-one there should land the
    /// operator on the last tab rather than panic in the middle of a close.
    fn put_slots(&mut self, mut all: Vec<Status>, active: usize) {
        if all.is_empty() {
            self.status = Status::Empty;
            self.parked = Vec::new();
            self.active_slot = 0;
            return;
        }
        let active = active.min(all.len() - 1);
        self.status = all.remove(active);
        self.parked = all;
        self.active_slot = active;
    }

    /// **The tab this path is already open in**, if it is.
    ///
    /// §3's rule. Only [`Origin::Opened`] documents can match: a created
    /// document's path is a name, not a location.
    #[must_use]
    pub fn slot_of_path(&self, path: &std::path::Path) -> Option<usize> {
        (0..self.document_count()).find(|slot| match self.slot(*slot) {
            Some(Status::Open(doc)) => doc.origin == Origin::Opened && doc.path == path,
            Some(Status::Failed { path: p, .. })
            | Some(Status::Unsupported { path: p, .. })
            | Some(Status::NeedsPassword { path: p }) => p == path,
            Some(Status::Empty) | None => false,
        })
    }

    /// **Park the active document and make `incoming` the active one**, as a
    /// new tab at the end of the strip.
    ///
    /// The one entry point for "a document has just been produced" — an open,
    /// a create, a failed open. It does **not** run `PdfcerApp::adopt`; the
    /// caller does, because `adopt` is also what seeds a new document's view
    /// from the opening preferences and only the caller knows whether this is
    /// a new document or a returning one.
    ///
    /// A new tab goes at the **end**, which is where every tabbed application
    /// puts one. Inserting beside the active tab was considered and rejected:
    /// browsers that do that do it for tabs *spawned by* the current page, and
    /// an Open is not that.
    pub fn park_and_adopt(&mut self, incoming: Status) {
        if self.document_count() == 0 {
            self.status = incoming;
            self.parked = Vec::new();
            self.active_slot = 0;
            return;
        }
        let mut all = self.take_slots();
        all.push(incoming);
        let last = all.len() - 1;
        self.put_slots(all, last);
        self.forget_previous_documents_view();
    }

    /// **Show the document in tab position `slot`.**
    ///
    /// A no-op if it is already active or the slot does not exist, which is
    /// what lets the tab strip call it unconditionally on a click.
    ///
    /// Forgets what §4 says it must and nothing more. In particular it does
    /// not touch the incoming document's view, its rasters or its selection —
    /// those are the state that makes coming back to a tab worth doing.
    pub fn activate_slot(&mut self, slot: usize) {
        if slot >= self.document_count() || slot == self.active_slot {
            return;
        }
        let all = self.take_slots();
        self.put_slots(all, slot);
        self.forget_previous_documents_view();
        crate::diag::trace(|| {
            format!(
                // ui-text-exempt: diagnostic trace, never displayed in the UI
                "document-activate slot={slot} of={} path={:?}",
                self.document_count(),
                self.active_path(),
            )
        });
    }

    /// **Close the document in tab position `slot`.**
    ///
    /// §5's rule for what becomes active afterwards. Closing the last one
    /// leaves [`Status::Empty`].
    ///
    /// ★ The unsaved-edits question belongs to the caller. This is reached
    /// from [`PdfcerApp::close_document`] (which is behind both guards) and
    /// from the tab strip's ✕ (which raises an action that goes through the
    /// same guards). Nothing may call it directly from a click.
    pub fn close_slot(&mut self, slot: usize) {
        if slot >= self.document_count() {
            return;
        }
        crate::diag::trace(|| match self.slot(slot) {
            Some(Status::Open(doc)) => format!(
                // ui-text-exempt: diagnostic trace, never displayed in the UI
                "close slot={slot} path={:?} pages={}",
                doc.path,
                doc.pages.len()
            ),
            Some(Status::Failed { path, .. })
            | Some(Status::Unsupported { path, .. })
            | Some(Status::NeedsPassword { path }) => format!(
                // ui-text-exempt: diagnostic trace, never displayed in the UI
                "close slot={slot} unopened path={path:?}"
            ),
            // ui-text-exempt: diagnostic trace, never displayed in the UI
            Some(Status::Empty) | None => format!("close slot={slot} nothing-open"),
        });

        let was_active = self.active_slot;
        let mut all = self.take_slots();
        all.remove(slot);
        // The browser rule, stated once. Closing a tab left of the active one
        // shifts the active document down; closing the active one hands the
        // position to its right-hand neighbour, which is the same index once
        // the removal has happened.
        let next = if slot < was_active {
            was_active.saturating_sub(1)
        } else {
            was_active
        };
        self.put_slots(all, next);
        self.forget_previous_documents_view();
    }

    /// **Move the tab at `from` to the boundary `gap`**, keeping the same
    /// document on screen.
    ///
    /// `gap` is a **boundary**, not a destination index: `0` is before the
    /// first tab and `document_count()` is after the last, which is the same
    /// vocabulary the insertion caret is drawn in and the same one a page drop
    /// uses. `egui_shell::tabstrip::TabIntent::Reorder` carries the argument
    /// for why it is not "the index it ends up at" — the two differ by one
    /// whenever a tab moves rightward, because it is removed before it is
    /// re-inserted, and a caller with the wrong convention is off by one in one
    /// direction only.
    ///
    /// # ★ The document on screen does not change, and that is arithmetic
    ///
    /// Reordering tabs is not navigation. An operator dragging tab 5 to the
    /// front has not asked to *look* at it, so the active document has to
    /// follow its own tab through the permutation rather than staying at an
    /// index. Getting that wrong would switch document as a side effect of
    /// tidying the strip, which no application does.
    ///
    /// Three cases, and the third is the one that needs the `+1`:
    ///
    /// | the active tab | where it goes |
    /// |---|---|
    /// | **is** the one being moved | wherever it lands |
    /// | was to the **right** of `from` | one place left, because a tab was removed in front of it |
    /// | ends up at or after the insertion point | one place right, because a tab was inserted in front of it |
    ///
    /// The two adjustments compose — a tab can be both — which is why they are
    /// applied in sequence rather than as a `match`.
    pub fn move_slot(&mut self, from: usize, gap: usize) {
        let count = self.document_count();
        if from >= count {
            return;
        }
        // Where it actually lands. Moving rightward, the removal has already
        // shifted every later tab down by one, so the boundary `gap` is one
        // place further along than the index to insert at.
        let insert_at = if gap > from { gap - 1 } else { gap }.min(count - 1);
        if insert_at == from {
            // Dropped where it already is. Not traced and not applied: a
            // reorder that changes nothing is a gesture the operator abandoned,
            // and treating it as an event would put a line in the log for every
            // tab they thought better of moving.
            return;
        }

        let was_active = self.active_slot;
        let mut all = self.take_slots();
        let moved = all.remove(from);
        all.insert(insert_at, moved);

        let active = if was_active == from {
            insert_at
        } else {
            let shifted = if was_active > from {
                was_active - 1
            } else {
                was_active
            };
            if shifted >= insert_at {
                shifted + 1
            } else {
                shifted
            }
        };
        self.put_slots(all, active);
        crate::diag::trace(|| {
            format!(
                // ui-text-exempt: diagnostic trace, never displayed in the UI
                "document-reorder from={from} gap={gap} to={insert_at} active={}",
                self.active_slot
            )
        });
    }

    /// **Move to the next or previous tab**, wrapping.
    ///
    /// Wrapping because Ctrl+Tab wraps in every application that has it, and
    /// an operator with two documents open would otherwise find the chord dead
    /// half the time.
    pub fn cycle_document(&mut self, forward: bool) {
        let count = self.document_count();
        if count < 2 {
            return;
        }
        let next = if forward {
            (self.active_slot + 1) % count
        } else {
            (self.active_slot + count - 1) % count
        };
        self.activate_slot(next);
    }

    /// The active document's path, for a trace line and the window title.
    /// `None` when nothing is open.
    #[must_use]
    pub fn active_path(&self) -> Option<&std::path::Path> {
        match &self.status {
            Status::Open(doc) => Some(doc.path.as_path()),
            Status::Failed { path, .. }
            | Status::Unsupported { path, .. }
            | Status::NeedsPassword { path } => Some(path.as_path()),
            Status::Empty => None,
        }
    }

    /// Everything that a **different** document being on screen makes stale.
    ///
    /// §4's table, as three statements. Deliberately the same three
    /// [`PdfcerApp::close_document`] makes, and deliberately *not* `adopt` —
    /// see §4 on why re-seeding the view would be wrong here.
    fn forget_previous_documents_view(&mut self) {
        self.panels.forget_document();
        self.find.forget_document();
        crate::diag::reset_change_gates();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    /// A `Status` that is a tab but is not a whole document, so the encoding
    /// can be exercised without building four `EditSession`s.
    ///
    /// ★ Using `Failed` rather than `Open` is not a shortcut around the real
    /// type — §2 makes a failed open a first-class tab, so this *is* one of
    /// the states the encoding has to carry, and the tests below are testing
    /// the tab arithmetic rather than anything about documents.
    fn tab(name: &str) -> Status {
        Status::Failed {
            path: PathBuf::from(name),
            // ui-text-exempt: test fixture, never displayed
            message: String::from("fixture"),
        }
    }

    /// The paths of every tab, left to right — the operator's strip as a
    /// string, which is what makes the assertions below readable.
    fn strip(app: &PdfcerApp) -> Vec<String> {
        (0..app.document_count())
            .map(|slot| match app.slot(slot) {
                Some(Status::Failed { path, .. }) => path.display().to_string(),
                other => format!("{}", other.is_some()),
            })
            .collect()
    }

    fn app_with(names: &[&str]) -> PdfcerApp {
        let mut app = PdfcerApp::new();
        for name in names {
            app.park_and_adopt(tab(name));
        }
        app
    }

    /// **Nothing open is zero tabs, not one empty one.**
    ///
    /// The invariant §2 states. Every other function here asks
    /// `document_count`, so this is the assertion the rest rest upon.
    #[test]
    fn an_empty_application_has_no_tabs() {
        let app = PdfcerApp::new();
        assert_eq!(app.document_count(), 0);
        assert!(app.slot(0).is_none());
    }

    /// **A new document goes at the end of the strip and becomes active.**
    #[test]
    fn opening_appends_a_tab_and_shows_it() {
        let app = app_with(&["a", "b", "c"]);
        assert_eq!(app.document_count(), 3);
        assert_eq!(strip(&app), ["a", "b", "c"]);
        assert_eq!(app.active_slot, 2, "the newest document is the one shown");
    }

    /// **The encoding survives a round trip through every active position.**
    ///
    /// The property that makes `parked` + `active_slot` safe: whichever tab is
    /// active, the strip reads the same left to right. A naive encoding that
    /// pushed the outgoing document onto the end of `parked` would pass with
    /// `active_slot == 2` and reorder the operator's tabs on any other.
    #[test]
    fn the_strip_order_is_independent_of_which_tab_is_active() {
        for active in 0..4 {
            let mut app = app_with(&["a", "b", "c", "d"]);
            app.activate_slot(active);
            assert_eq!(
                strip(&app),
                ["a", "b", "c", "d"],
                "activating slot {active} reordered the strip"
            );
            assert_eq!(app.active_slot, active);
        }
    }

    /// **Closing a tab left of the active one keeps the same document on
    /// screen.**
    #[test]
    fn closing_a_tab_to_the_left_keeps_the_active_document() {
        let mut app = app_with(&["a", "b", "c"]);
        app.activate_slot(2);
        app.close_slot(0);
        assert_eq!(strip(&app), ["b", "c"]);
        assert_eq!(app.active_slot, 1, "still looking at c");
    }

    /// **Closing the active tab shows its right-hand neighbour**, and the
    /// rightmost falls back to the new last tab. The browser rule, §5.
    #[test]
    fn closing_the_active_tab_moves_right_then_clamps() {
        let mut app = app_with(&["a", "b", "c"]);
        app.activate_slot(1);
        app.close_slot(1);
        assert_eq!(strip(&app), ["a", "c"]);
        assert_eq!(app.active_slot, 1, "c took b's position");

        app.close_slot(1);
        assert_eq!(strip(&app), ["a"]);
        assert_eq!(app.active_slot, 0, "the rightmost close clamps");
    }

    /// **Closing the last tab is the empty state the application starts in**,
    /// which is what keeps every downstream surface free of a second one.
    #[test]
    fn closing_the_last_tab_is_the_start_up_state() {
        let mut app = app_with(&["only"]);
        app.close_slot(0);
        assert_eq!(app.document_count(), 0);
        assert!(matches!(app.status, Status::Empty));
        assert!(app.parked.is_empty());
        assert_eq!(app.active_slot, 0);
    }

    /// **Ctrl+Tab wraps in both directions**, and does nothing at all with one
    /// document — the state a non-wrapping implementation gets right by
    /// accident and a broken one gets wrong by panicking.
    #[test]
    fn cycling_wraps_both_ways_and_is_inert_below_two_documents() {
        let mut app = app_with(&["a"]);
        app.cycle_document(true);
        assert_eq!(app.active_slot, 0);

        let mut app = app_with(&["a", "b", "c"]);
        app.activate_slot(2);
        app.cycle_document(true);
        assert_eq!(
            app.active_slot, 0,
            "forward from the last wraps to the first"
        );
        app.cycle_document(false);
        assert_eq!(app.active_slot, 2, "back from the first wraps to the last");
    }

    /// ★★ **Reordering tabs keeps the same document on screen.**
    ///
    /// The property that makes `move_slot` correct and the one a naive
    /// implementation gets wrong: dragging a tab is tidying, not navigation, so
    /// the active document has to follow its own tab through the permutation
    /// rather than staying at an index.
    ///
    /// Swept across **every** `(from, gap)` pair on a four-tab strip with each
    /// of the four active in turn — 4 x 5 x 4 = 80 cases — rather than spot
    /// checked, because the arithmetic has two adjustments that compose and the
    /// composition is where an off-by-one hides. Three hand-picked cases would
    /// very likely all miss it.
    #[test]
    fn reordering_tabs_never_changes_which_document_is_on_screen() {
        for active in 0..4 {
            for from in 0..4 {
                for gap in 0..=4 {
                    let mut app = app_with(&["a", "b", "c", "d"]);
                    app.activate_slot(active);
                    let looking_at = strip(&app)[active].clone();

                    app.move_slot(from, gap);

                    assert_eq!(
                        app.document_count(),
                        4,
                        "a reorder lost or gained a tab: from={from} gap={gap}"
                    );
                    assert_eq!(
                        strip(&app)[app.active_slot],
                        looking_at,
                        "moving tab {from} to gap {gap} with {looking_at} on screen \
                         switched document"
                    );
                    let mut sorted = strip(&app);
                    sorted.sort();
                    assert_eq!(
                        sorted,
                        ["a", "b", "c", "d"],
                        "a reorder duplicated or dropped a document: from={from} gap={gap}"
                    );
                }
            }
        }
    }

    /// **A tab dropped where it already is changes nothing**, and the two gaps
    /// that mean that are both of them.
    ///
    /// `gap == from` is *before itself* and `gap == from + 1` is *after
    /// itself*; a strip that treated the second as a real move would shuffle
    /// the document one place every time an operator picked a tab up and put it
    /// back.
    #[test]
    fn dropping_a_tab_where_it_already_is_does_nothing() {
        for from in 0..4 {
            for gap in [from, from + 1] {
                let mut app = app_with(&["a", "b", "c", "d"]);
                app.move_slot(from, gap);
                assert_eq!(
                    strip(&app),
                    ["a", "b", "c", "d"],
                    "from={from} gap={gap} moved a tab that was already there"
                );
            }
        }
    }

    /// **A path that is already open is found**, so the caller can activate it
    /// rather than opening a second session over the same file (§3).
    #[test]
    fn a_path_that_is_already_open_is_found() {
        let app = app_with(&["a", "b", "c"]);
        assert_eq!(app.slot_of_path(std::path::Path::new("b")), Some(1));
        assert_eq!(app.slot_of_path(std::path::Path::new("z")), None);
    }
}
