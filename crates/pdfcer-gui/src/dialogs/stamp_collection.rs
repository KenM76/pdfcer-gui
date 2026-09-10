//! # `dialogs::stamp_collection` — turning this document into stamps Acrobat
//! will show in its own menu
//!
//! `OPERATOR_REQUESTS.md` **O169**, the operator's words:
//!
//! > *"if acrobat has a way of adding custom stamps or text, we need the same
//! > feature too with the same import/export to make the stamps as Adobe has
//! > and is compatible with adobe's"*
//!
//! ## ★★ The finding that decided the whole shape of this window
//!
//! **There is no interchange format, because Acrobat has none.** A stamp
//! collection *is an ordinary PDF* — one file per category, one page per stamp,
//! the category in `/Info` `/Title` and the names in the catalog's `/Names` →
//! `/Pages` name tree. So "the same import/export" is not a format to
//! implement: **handing someone the PDF is the export**, and dropping one into
//! Acrobat's stamps folder is the import. What the operator was missing was not
//! a converter — it was a way to *author* one of those files, and that is
//! exactly what this window does.
//!
//! ⇒ The window therefore has no format picker, no options page and no
//! "compatibility mode". It asks two questions — **what is this set called**
//! and **what is each page called** — and writes a file measured against the
//! four Adobe collections on this machine.
//!
//! ## ★★★ The trap this window is arranged around
//!
//! `pdfcer_core::stamp_file::name_stamp_pages` names `stamps[i]` to **page `i`
//! of the document it is given — by counting, not by lookup.** Untick page 1,
//! hand the engine the full name list, and every stamp names the page below the
//! one it describes: a file that opens, holds the right number of stamps,
//! appears in Acrobat's menu, and **stamps the wrong picture every time**.
//! There is no symptom short of looking at the artwork.
//!
//! The guard is structural rather than careful. [`crate::stamps::Plan`]'s
//! `for_engine()` and `pages_to_extract()` are the *same filter over the same
//! rows*, so the name list and the extracted page list cannot drift apart, and
//! `crate::stamps::tests::plan_and_extraction_agree` is the unit test that
//! fails if somebody makes them two filters again.
//!
//! ## Why the operator's document is never touched
//!
//! `Save as stamp collection…` is a **Read-mode-legal act** by the standing
//! rule — *Read may produce a new document; it may not modify this one*. The
//! write path never takes a mutable handle on his session at all: it extracts
//! the ticked pages into a new document and names *those*. See
//! [`crate::stamps::write`]'s header for the two reasons that matters.
//!
//! ## Rule 4, in this window
//!
//! Every adjustment pdfcer makes to a *hidden* identifier is listed **before**
//! the write, off-canvas, in [`Self::adjustments_group`] — and only when there
//! is something to list. Nothing on the page is marked, tinted or flagged,
//! because the operator's own words are that *"the nagging and red flagging in
//! the original GUI made for a lot of extra bugs in the visibility when
//! editing."*
//!
//! The half that is easy to skip is the one that matters most here: an operator
//! **cannot see** the identifier. His name is written exactly as typed; the
//! identifier beside it may have lost a space, a `#` or eight characters, and
//! the only place he could ever learn that is a sentence like these.

use egui::Ui;

use crate::app::actions::Action;
use crate::app::state::{OpenDoc, Status};
use crate::stamps::{Blocker, Plan};
use crate::text::stamps as t;

/// The region this dialog publishes for its body.
pub const REGION_BODY: &str = "dialog:stamp-collection"; // ui-text-exempt: trace region name, never displayed
/// The region the category field publishes.
pub const REGION_CATEGORY: &str = "stamp-collection.category"; // ui-text-exempt: trace region name, never displayed
/// The region the scrolling row list publishes.
pub const REGION_ROWS: &str = "stamp-collection.rows"; // ui-text-exempt: trace region name, never displayed
/// The prefix each row's own region carries, suffixed with the page number.
pub const REGION_ROW_PREFIX: &str = "stamp-collection.row"; // ui-text-exempt: trace region name, never displayed
/// The region the adjustment disclosure publishes, when it is drawn at all.
pub const REGION_ADJUSTMENTS: &str = "stamp-collection.adjustments"; // ui-text-exempt: trace region name, never displayed
/// The region the Save button publishes.
pub const REGION_SAVE: &str = "stamp-collection.save"; // ui-text-exempt: trace region name, never displayed

/// Height reserved below the scroll area for the separator and button row.
///
/// ★ A named constant used **both** by the opening-height calculation and by
/// the scroll area's `max_height`, which is `dialogs::formfield`'s recorded
/// finding: those were a literal `40.0` in one place and nothing at all in the
/// other, which is how the two halves of one reservation drift apart.
const FOOTER_PTS: f32 = 46.0;

/// Height one row occupies — a checkbox, a page label and a text field.
const ROW_PTS: f32 = 26.0;

/// Height reserved for everything above the rows: title paragraph, category
/// field and its hint, and the row heading.
const HEADER_PTS: f32 = 150.0;

/// How many rows the window opens tall enough to show.
///
/// ★ A **constant inventory**, never a measurement from inside the scroll area
/// — R128's feedback loop. A document of eighty sheets must not open a window
/// eighty rows tall; it opens at eight and scrolls, which is a decision made
/// here rather than a number egui arrives at by growing.
const ROWS_SHOWN: usize = 8;

/// The scrollbar width, matching `dialogs::print::layout::SCROLLBAR_WIDTH_PTS`
/// so two scrolling dialogs do not disagree about how wide a scrollbar is.
const SCROLLBAR_WIDTH_PTS: f32 = 10.0;

/// The window's live state.
pub struct StampCollectionDialog {
    /// The whole collection as it will be written, edited in place.
    ///
    /// No shadow copy of anything in it. A window that mirrors its plan into
    /// local fields is a window that can show one thing and write another, and
    /// [`Plan`] is already exactly the value the writer takes.
    plan: Plan,
    /// Whether the open document already **is** a stamp collection.
    ///
    /// Kept because it changes what the window is claiming: with it set, the
    /// names in the rows came out of the file rather than out of a default,
    /// and saying so is the difference between "pdfcer guessed these" and
    /// "these are yours".
    reopened: bool,
    /// How many rows the window opened with, frozen for the height sum.
    rows: usize,
    /// Set by Save, consumed after the window's closure returns.
    save_requested: bool,
    /// Set by Cancel, consumed by [`Self::show`].
    close_requested: bool,
}

impl StampCollectionDialog {
    /// Open the window for the document on screen.
    ///
    /// The plan is built **once, here**. Re-deriving it per frame would be a
    /// name-tree read and a full uniqueness pass sixty times a second for an
    /// answer that only changes when the operator types — and, worse, would
    /// throw away every name he had typed on the frame after he typed it.
    ///
    /// # ★ Where the seeded category comes from, in order
    ///
    /// 1. The collection's own category, when this document already is one.
    ///    Re-opening a collection to fix one typo must not retype the heading.
    /// 2. The document's `/Info` `/Title`.
    /// 3. The filename's stem.
    ///
    /// All three are *suggestions in an editable field the operator is looking
    /// at*, which is why none of them owes a disclosure sentence: what will be
    /// written is on screen, in the box, before anything is written.
    #[must_use]
    pub fn open(doc: &OpenDoc) -> Self {
        // ★ `session.document()`, which is the document as loaded. There is no
        // edit verb in this shell that writes a stamp name tree, so the base
        // document and the session agree about the only thing being read here.
        // The *page count* comes from the live `doc.pages`, because inserting
        // a page is a verb, and a plan short of a row would silently drop the
        // last sheet.
        let collection = pdfcer_core::stamp_file::read(doc.session.document());
        let reopened = crate::stamps::is_collection(&collection);
        let existing = crate::stamps::existing_names(&collection);

        let category = collection
            .category
            .clone()
            .filter(|c| !c.trim().is_empty())
            .or_else(|| {
                doc.session
                    .info_text(pdfcer_core::edit::InfoField::Title)
                    .map(|info| info.text)
                    .filter(|title| !title.trim().is_empty())
            })
            .or_else(|| {
                doc.path
                    .file_stem()
                    .map(|stem| stem.to_string_lossy().into_owned())
            })
            .unwrap_or_default();

        let rows = doc.pages.len();
        let plan = Plan::new(rows, &category, &existing);

        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed
            format!(
                "stamp-collection-open pages={rows} reopened={} existing={} dynamic={}",
                u8::from(reopened),
                existing.len(),
                crate::stamps::dynamic_count(&collection),
            )
        });

        Self {
            plan,
            reopened,
            rows,
            save_requested: false,
            close_requested: false,
        }
    }

    /// Draw it. Returns `false` when it should close.
    pub fn show(&mut self, ctx: &egui::Context, actions: &mut Vec<Action>) -> bool {
        let height = HEADER_PTS + ROW_PTS * ROWS_SHOWN.min(self.rows.max(1)) as f32 + FOOTER_PTS;
        let (frame, ()) = crate::dialogs::host::Host::new(
            "stamp-collection", // ui-text-exempt: a viewport key, never displayed.
            t::window_title(),
            egui::vec2(480.0, height),
            egui::vec2(380.0, 260.0),
        )
        .show(ctx, |ui| {
            crate::diag::ui_rect(REGION_BODY, ui.max_rect());
            self.body(ui);
        });
        let open = !frame.closed;

        if std::mem::take(&mut self.save_requested) {
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed
                format!(
                    "stamp-collection-requested stamps={} pages={} adjustments={} category={:?}",
                    self.plan.included(),
                    self.rows,
                    self.plan.adjustments().len(),
                    self.plan.category,
                )
            });
            actions.push(Action::Write(
                crate::app::actions::write::WriteAction::StampCollection {
                    plan: self.plan.clone(),
                },
            ));
            return false;
        }
        open && !std::mem::take(&mut self.close_requested)
    }

    /// The whole window body.
    fn body(&mut self, ui: &mut Ui) {
        // No `.strong()` anywhere in this window — R84 / DEFECTS.md D11.
        ui.label(t::intro());
        if self.reopened {
            ui.add_space(4.0);
            ui.weak(t::reopened_note());
        }
        ui.add_space(8.0);

        self.category_group(ui);
        ui.add_space(8.0);

        ui.label(t::stamps_heading(self.plan.included()));

        // ★ The solid, foreground-coloured scrollbar `dialogs::formfield`
        // records: a default handle is `widgets.inactive.bg_fill`, which in a
        // light preset is a near-white on near-white — measured elsewhere in
        // this shell as *present, opaque, correctly sized and invisible*.
        let mut scroll = egui::style::ScrollStyle::solid();
        scroll.foreground_color = true;
        scroll.bar_width = SCROLLBAR_WIDTH_PTS;
        ui.style_mut().spacing.scroll = scroll;

        let scroll_height = (ui.available_height() - FOOTER_PTS).max(ROW_PTS);
        let area = egui::ScrollArea::vertical()
            .max_height(scroll_height)
            .show(ui, |ui| self.rows_group(ui));
        crate::diag::ui_rect(REGION_ROWS, area.inner_rect);

        self.adjustments_group(ui);

        ui.add_space(6.0);
        ui.separator();
        self.commit_row(ui);
    }

    /// The category field — the heading Acrobat shows this set under.
    fn category_group(&mut self, ui: &mut Ui) {
        ui.label(t::category_label());
        let response = ui
            .add(egui::TextEdit::singleline(&mut self.plan.category).desired_width(f32::INFINITY));
        crate::diag::ui_rect(REGION_CATEGORY, response.rect);
        ui.weak(t::category_hint());
    }

    /// One row per page: tick, page number, name.
    ///
    /// # ★★ Why every keystroke re-derives the WHOLE list
    ///
    /// Uniqueness is a property of the **set**, not of a row. Renaming row 1
    /// can free the name row 4 was renumbered away from, and a per-row update
    /// would leave row 4 wearing a *"we renamed it"* sentence explaining a
    /// collision that no longer exists — a disclosure whose subject has gone.
    /// `crate::stamps::tests::rederiving_after_a_rename_drops_the_stale_disclosure`
    /// is the test that holds this.
    ///
    /// The cost is a quadratic pass over a list whose length is a page count,
    /// run on a keystroke. On the operator's largest sheet set that is a few
    /// hundred string comparisons in a frame that already rasterized a page.
    fn rows_group(&mut self, ui: &mut Ui) {
        let mut changed = false;
        for index in 0..self.plan.stamps.len() {
            let page_number = index.saturating_add(1);
            let row = ui.horizontal(|ui| {
                // ui-text-exempt: an empty checkbox label — the page number and
                // the name field beside it are what the tick refers to, and a
                // second word here would repeat one of them.
                let tick = ui.checkbox(&mut self.plan.stamps[index].include, "");
                if tick.changed() {
                    changed = true;
                }
                tick.on_hover_text(t::include_hint());
                ui.label(t::page_label(page_number));
                ui.add(
                    egui::TextEdit::singleline(&mut self.plan.stamps[index].display)
                        .desired_width(f32::INFINITY),
                )
                .changed()
            });
            if row.inner {
                changed = true;
            }
            crate::diag::ui_rect(
                &format!("{REGION_ROW_PREFIX}{page_number}"),
                row.response.rect,
            );
        }
        if changed {
            self.plan.rederive();
        }
    }

    /// Everything pdfcer had to change about a hidden identifier — and nothing
    /// when it changed nothing.
    ///
    /// ★ Drawn only when the list is non-empty. A permanently-present box
    /// reading *"no adjustments"* trains an operator to stop reading the place
    /// adjustments appear, which costs exactly the one time it matters.
    fn adjustments_group(&self, ui: &mut Ui) {
        let sentences: Vec<String> = self
            .plan
            .adjustments()
            .iter()
            .map(|(index, adjustment)| t::adjustment(index.saturating_add(1), adjustment))
            .collect();
        if sentences.is_empty() {
            return;
        }

        ui.add_space(6.0);
        ui.separator();
        ui.add_space(4.0);
        let start = ui.cursor();
        ui.label(t::adjustments_heading());
        ui.weak(t::adjustments_intro());
        for sentence in sentences {
            ui.weak(sentence);
        }
        crate::diag::ui_rect(REGION_ADJUSTMENTS, start.union(ui.cursor()));
    }

    /// Save and Cancel.
    ///
    /// ★ Greyed with the reason on hover, which is the one situation **R9**
    /// reserves greying for: *temporarily* unavailable, and one keystroke or
    /// one tick makes it live. The blocker's sentence is also drawn beside the
    /// button rather than only on hover, because a hover tooltip is a thing you
    /// find after you have already wondered why nothing happened.
    fn commit_row(&mut self, ui: &mut Ui) {
        let blocker = self.plan.blocker();
        let hint = blocker.map(|blocker| match blocker {
            Blocker::NoStamps => t::blocked_no_stamps(),
            Blocker::NoCategory => t::blocked_no_category(),
        });
        ui.horizontal(|ui| {
            let save = ui.add_enabled(hint.is_none(), egui::Button::new(t::save_button()));
            crate::diag::ui_rect(REGION_SAVE, save.rect);
            if save.clicked() {
                self.save_requested = true;
            }
            if let Some(hint) = hint {
                save.on_disabled_hover_text(hint);
            }
            if ui.button(t::cancel_button()).clicked() {
                self.close_requested = true;
            }
        });
        if let Some(hint) = hint {
            ui.weak(hint);
        }
    }
}

/// Open the window for `status`, or decline.
///
/// The no-document guard is real rather than ceremonial: the window's rows are
/// one per page and its category is seeded from the file, so there is nothing
/// to build without one. An empty document declines for the same reason —
/// a collection with no stamps in it is not a thing Acrobat will show.
#[must_use]
pub fn open_for(status: &Status) -> Option<StampCollectionDialog> {
    match status {
        Status::Open(doc) if !doc.pages.is_empty() => Some(StampCollectionDialog::open(doc)),
        _ => None,
    }
}
