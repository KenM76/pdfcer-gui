//! # `dialogs::offpage` — **which of my drawings have marks outside the sheet?**
//!
//!
//! ## ★★★ The third of the operator's question that was still open
//!
//! **Ken, 2026-09-10:** *"how do I view and edit objects that are off of the
//! page? we added this feature but I didn't see how to enable it."*
//!
//! Design and rationale: `docs/modules/pdfcer-gui/dialogs/offpage.md`.

use egui::Ui;

use pdfcer_core::offpage::{self, PageScan, UnreadablePage};

use crate::app::actions::{Action, RedactAction};
use crate::app::state::{OpenDoc, Status};
use crate::text::offpage as t;

/// The window body's rect, for `ui-verify`.
// ui-text-exempt: trace region name, never displayed
pub const REGION_BODY: &str = "offpage.body";
/// The Mark button — the one control here that changes anything.
// ui-text-exempt: trace region name, never displayed
pub const REGION_MARK: &str = "offpage.mark";

/// How many pages the scan advances per frame.
///
/// ★★ One, and the constant exists to be *named* rather than to be tuned. The
/// worst sheet this project has measured costs 469 ms to decompose; two per
/// frame would be a second of unresponsiveness per frame on that document,
/// which is the freeze this design exists to avoid, merely chopped up.
///
/// A cheap document scans at frame rate, which for a sixty-page office PDF is
/// one second. There is no configuration here because there is no trade to make:
/// the slow case is the one that matters and it wants the smallest step.
const PAGES_PER_FRAME: usize = 1;

/// The census window, and the walk it is part-way through.
pub struct OffPageDialog {
    /// The next page index to scan. When it reaches [`Self::total_pages`] the
    /// walk is done and the window is showing a complete answer.
    next_page: usize,
    /// How many pages the document had **when the window opened**.
    ///
    /// ★ Snapshotted rather than re-read per frame, and it is load-bearing: an
    /// edit that deletes a page while this window is open would otherwise walk
    /// off the end of a shortened `doc.pages`, and — worse — the progress line
    /// would count down to a total that moved. The walk is bounds-checked
    /// against the live vector anyway (see [`Self::advance`]), so a shrunken
    /// document ends the scan early rather than panicking.
    total_pages: usize,
    /// Every page that has something outside its boundary, in page order.
    ///
    /// ★★ **Clean pages are dropped, not stored.** A `PageScan` for a clean
    /// sheet carries a page box, a drawn extent and an empty object list — all
    /// of it true and none of it anything this window draws or the marking
    /// action uses (`offpage_bands` answers with an empty vec for a clean scan).
    /// Keeping them would make the list rendering and the mark loop both carry
    /// an "is it empty?" test that the collection can carry once.
    scans: Vec<PageScan>,
    /// Pages whose content streams would not decode, with the engine's reason.
    ///
    /// ★★★ Kept **separately** from `scans` and never merged into it, on the
    /// engine's own instruction: *"'no findings' and 'I could not look' must not
    /// print the same way."* This is the one list that stops this window from
    /// issuing a clean bill it has not earned.
    unreadable: Vec<UnreadablePage>,
    /// Running total of off-page objects across [`Self::scans`], so the summary
    /// line does not re-sum a growing list every frame.
    objects: usize,
    mark_requested: bool,
    close_requested: bool,
}

impl OffPageDialog {
    /// Open, showing nothing yet.
    ///
    /// ★ Infallible and unconditional for a document that is open, unlike every
    /// other window in this folder that computes a plan first. See the header:
    /// the empty answer is an answer.
    #[must_use]
    pub fn open(doc: &OpenDoc) -> Self {
        let total_pages = doc.pages.len();
        // ★ Traced on open, for `crate::dialogs::unembed`'s stated reason: a
        // driven check that cannot tell "the scan has not started" from "the
        // scan found nothing" reports the fixture as a defect in the program.
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed
            format!("offpage-opened pages={total_pages}")
        });
        Self {
            next_page: 0,
            total_pages,
            scans: Vec::new(),
            unreadable: Vec::new(),
            objects: 0,
            mark_requested: false,
            close_requested: false,
        }
    }

    /// Whether the walk still has pages to look at.
    #[must_use]
    const fn scanning(&self) -> bool {
        self.next_page < self.total_pages
    }

    /// Scan up to [`PAGES_PER_FRAME`] more pages.
    ///
    /// ★★ The bounds check against the **live** `doc.pages` rather than against
    /// the snapshotted total is the guard described on [`Self::total_pages`]: a
    /// page deleted under an open window ends the walk where the document now
    /// ends, and the window then shows a complete answer about a shorter
    /// document rather than panicking on an index that no longer exists.
    fn advance(&mut self, doc: &OpenDoc) {
        let view = doc.session.view();
        for _ in 0..PAGES_PER_FRAME {
            let Some(page) = doc.pages.get(self.next_page) else {
                // The document shrank. Stop the walk where it now ends.
                self.total_pages = self.next_page;
                return;
            };
            let index = self.next_page;
            self.next_page += 1;
            match offpage::scan_page(&view, page, index, offpage::DEFAULT_TOLERANCE_PT) {
                Ok(scan) => {
                    if !scan.is_clean() {
                        self.objects += scan.objects.len();
                        self.scans.push(scan);
                    }
                }
                Err(error) => self.unreadable.push((index, error.to_string())),
            }
            if !self.scanning() {
                // ★★★ The line a driven check reads, and the only place this
                // window states a FINISHED result. It is emitted once, on the
                // frame the walk completes, rather than every frame — a trace
                // that repeats is one a `Trace::last()` assertion cannot use to
                // tell one run from the next.
                crate::diag::trace(|| {
                    // ui-text-exempt: diagnostic trace, never displayed
                    format!(
                        "offpage-scanned pages={} dirty={} objects={} unreadable={}",
                        self.total_pages,
                        self.scans.len(),
                        self.objects,
                        self.unreadable.len()
                    )
                });
                return;
            }
        }
    }

    /// Draw it. Returns whether it stays open.
    pub fn show(
        &mut self,
        ctx: &egui::Context,
        doc: &OpenDoc,
        actions: &mut Vec<Action>,
        appearance: pdfcer_core::annot_author::RedactAppearance,
    ) -> bool {
        // ★★ BEFORE the frame is drawn, so the page just scanned is in the list
        // the operator sees this frame rather than next frame. Drawing first
        // would make the window one page stale throughout the walk — invisible
        // on a fast document and a whole page's lag on a slow one.
        if self.scanning() {
            self.advance(doc);
            // ★★★ The line that makes this a scan rather than a stall. egui is
            // an immediate-mode library that draws when something asks it to;
            // without this the window would paint once, sit there having scanned
            // exactly one page, and resume only when the operator jiggled the
            // mouse over it.
            ctx.request_repaint();
        }

        let (frame, ()) = crate::dialogs::host::Host::new(
            "offpage", // ui-text-exempt: a viewport key, never displayed.
            t::window_title(),
            egui::vec2(620.0, 560.0),
            egui::vec2(400.0, 280.0),
        )
        .show(ctx, |ui| {
            crate::diag::ui_rect(REGION_BODY, ui.max_rect());
            self.body(ui);
        });
        let open = !frame.closed;

        if std::mem::take(&mut self.mark_requested) {
            let bands: Vec<(usize, Vec<pdfcer_core::page_tree::Rect>)> = self
                .scans
                .iter()
                //
                // Passing `0.0` here would compile, pass every test, and decline
                // both halves of that fix in silence. The value must track line 217.
                .map(|scan| {
                    (
                        scan.page_index,
                        offpage::offpage_bands(scan, offpage::DEFAULT_TOLERANCE_PT),
                    )
                })
                .filter(|(_, rects)| !rects.is_empty())
                .collect();
            let total: usize = bands.iter().map(|(_, rects)| rects.len()).sum();
            // ★ `-requested`, and the suffix is not decoration: `Trace::last()`
            // matches on a line's first token, and the funnel writes
            // `redact-mark-offpage page=…` when the edit lands. Two lines with
            // the same first token would make a driven check unable to tell the
            // press from the result. `actions::redactsel` carries the same
            // suffix for the same reason.
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed
                format!(
                    "offpage-mark-requested pages={} bands={total} unreadable={}",
                    bands.len(),
                    self.unreadable.len()
                )
            });
            actions.push(Action::Redact(RedactAction::OffPage {
                bands,
                unreadable: self.unreadable.len(),
                appearance,
            }));
            // ★★ Closes on the press, unlike `crate::dialogs::unembed` which
            // also does — and here there is an extra reason worth naming. The
            // marks it just authored change the document, so every scan in this
            // window becomes a statement about a revision that no longer exists.
            // A window left open showing a stale census beside a canvas that has
            // visibly changed is the "two accounts of one run" failure.
            return false;
        }
        open && !std::mem::take(&mut self.close_requested)
    }

    fn body(&mut self, ui: &mut Ui) {
        ui.label(t::intro());
        ui.add_space(8.0);

        // ★★★ The progress line FIRST and above everything, because during the
        // walk it is the only thing on screen that distinguishes an incomplete
        // answer from a complete one. See the header.
        if self.scanning() {
            ui.label(t::scanning(self.next_page, self.total_pages));
            ui.add_space(4.0);
        } else if self.scans.is_empty() && self.unreadable.is_empty() {
            ui.label(t::nothing_found());
            ui.add_space(4.0);
        } else if !self.scans.is_empty() {
            ui.label(t::summary(self.scans.len(), self.objects));
            ui.add_space(4.0);
            // ★★★ The residual disclosure, and it is owed rather than
            // decorative: a picture that only CROSSES the edge is still on
            // this list after a clean, because clearing erases ink and
            // cannot move a placement. See `text::offpage::partial_image_note`
            // for the engine's own statement of it and for why the sentence
            // says pictures rather than objects.
            //
            // Counted here rather than carried on the scan because it is a
            // question about this window's wording, not about the census —
            // `PageScan::partial()` counts all three kinds and is right to.
            let pictures = self
                .scans
                .iter()
                .flat_map(|s| &s.objects)
                // ui-text-exempt: the engine's stable object token, never displayed.
                .filter(|o| o.kind == "image" && o.how == offpage::OffPage::Partial)
                .count();
            if pictures > 0 {
                ui.small(t::partial_image_note(pictures));
                ui.add_space(4.0);
            }
        }

        ui.separator();
        egui::ScrollArea::vertical()
            .max_height((ui.available_height() - 64.0).max(120.0))
            .show(ui, |ui| {
                for scan in &self.scans {
                    ui.label(t::page_heading(
                        scan.page_index,
                        scan.fully_off(),
                        scan.partial(),
                    ));
                    for object in &scan.objects {
                        ui.small(t::object_row(
                            object.kind,
                            object.how,
                            object.text.as_deref(),
                        ));
                    }
                    ui.add_space(6.0);
                }
                // ★★★ LAST in the list and never filtered out, however long the
                // findings above it are. A page pdfcer could not read is the one
                // row in this window that is about the limits of the answer
                // rather than about the document.
                for (index, why) in &self.unreadable {
                    ui.small(t::unreadable_row(*index, why));
                }
            });

        ui.add_space(8.0);
        ui.separator();
        ui.horizontal(|ui| {
            // ★★ Greyed while the walk is still running, as well as when there
            // is nothing to mark — R9's *temporarily* unavailable, which is
            // exactly what this is. Marking from a half-finished census would
            // author marks on the sheets that happened to be checked first and
            // silently leave the rest, and the operator would have no way to
            // know which.
            let can = !self.scanning() && !self.scans.is_empty();
            let mark = ui.add_enabled(can, egui::Button::new(t::mark_button()));
            crate::diag::ui_rect_visible(REGION_MARK, mark.rect, ui.clip_rect());
            let mark = if can {
                mark.on_hover_text(t::mark_tooltip())
            } else if self.scanning() {
                mark.on_disabled_hover_text(t::scanning(self.next_page, self.total_pages))
            } else {
                mark.on_disabled_hover_text(t::nothing_to_mark())
            };
            if mark.clicked() {
                self.mark_requested = true;
            }
            if ui.button(t::close_button()).clicked() {
                self.close_requested = true;
            }
        });
    }
}

/// Open it for the current document, or answer `None` when nothing is open.
#[must_use]
pub fn open_for(status: &Status) -> Option<OffPageDialog> {
    match status {
        Status::Open(doc) => Some(OffPageDialog::open(doc)),
        _ => None,
    }
}
