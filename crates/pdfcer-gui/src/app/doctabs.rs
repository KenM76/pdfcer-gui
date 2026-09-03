//! # `app::doctabs` — the document tab strip, and the tab that springs open
//! under a drag
//!
//! pdfcer's half of [`egui_shell::tabstrip`]. That module draws a row of tabs
//! and knows nothing about documents; this one turns [`Status`] into tabs,
//! turns the operator's clicks back into [`Action`]s, and owns the one
//! behaviour the shell deliberately refused: **spring-loading**.
//!
//! ---
//!
//! ## 1. Why there is a tab strip at all
//!
//! Because the operator asked to *"open multiple PDFs at once"*, and every
//! application that does answers with tabs — Acrobat, Bluebeam, PDF-XChange,
//! Foxit, Illustrator, VS Code, every browser. `CONTINUE.md` §1b makes that
//! convergence the specification rather than a starting point for a better
//! idea:
//!
//! > **What do Illustrator, Inkscape, Acrobat, Word and the OLD shell do?** If
//! > they agree, that is the answer. The convergence is the specification.
//!
//! They agree, so this is a tab strip: labels left to right, the active one
//! emphasised, a ✕ on each, middle-click to close, Ctrl+Tab to cycle, an
//! overflow menu when there are too many. Nothing here is new and nothing here
//! is meant to be.
//!
//! ## 2. It is drawn whenever anything is open, including one document
//!
//! Chrome, VS Code and Acrobat all show the strip with a single tab. Hiding it
//! below two documents would save 26 points of a CAD sheet and cost the
//! feature its discoverability — an operator who has never seen a tab has no
//! reason to believe a second document is possible, which is the state this
//! project has already paid for once with text editing (`CONTINUE.md` §4.1:
//! *"it works and nobody can find it" is never a documentation problem*).
//!
//! Its height is a **constant** ([`egui_shell::tabstrip::STRIP_HEIGHT`]) in an
//! `exact_size` panel, for R128's reason: a chrome surface whose height varies
//! above a viewport that fits a page to itself is a measured feedback loop.
//!
//! ## 3. ★ Spring-loading — the gesture that makes a cross-document drag
//! possible
//!
//! With one canvas and one Pages panel, only one document's page list is on
//! screen at a time. So how does a page get from document A's list to document
//! B's?
//!
//! The answer every operator already has: **drag onto the other tab, wait, and
//! it opens.** Windows Explorer does it with folders and with taskbar buttons,
//! every browser does it with tabs, macOS Finder does it, Acrobat does it. It
//! is called spring-loading and nobody has ever had to be taught it.
//!
//! So: while a page drag is in flight ([`crate::pagedrag`]), dwelling on a tab
//! for [`SPRING_DWELL`] activates that document. The drag continues — it lives
//! in `egui::Memory` precisely so that switching documents cannot destroy it —
//! and the operator drops into the newly shown page list or page view at a
//! caret.
//!
//! ### The dwell is real time, not frames
//!
//! Measured against `egui`'s own input time so it behaves identically at 30
//! and 144 Hz. A frame count would spring in a third of a second on a fast
//! machine and a second and a half on a slow one.
//!
//! ### It is cancelled by moving to another tab, and by ending the drag
//!
//! The timer records **which** tab is being dwelt on. Moving the pointer to a
//! different tab restarts it; leaving the strip clears it. Without that, a
//! pointer that swept across five tabs on its way somewhere would arrive
//! having activated whichever one it happened to be over when the clock ran
//! out.
//!
//! ## 4. What a tab says
//!
//! [`crate::text::doctabs`] owns every string. The two decisions worth
//! knowing here:
//!
//! - the **unsaved marker leads** the label, because the ellipsis eats the
//!   tail and a crowded strip is exactly when the marker matters;
//! - a tab whose file **failed to open** is still a tab, with the reason in
//!   its tooltip — see [`crate::app::documents`] §2 for why a failed open must
//!   not evict the operator's other documents.

use eframe::egui;

use crate::app::PdfcerApp;
use crate::app::actions::Action;
use crate::app::state::{Origin, Status};

/// **How long the pointer must rest on a tab before it springs open.**
///
/// 600 ms. Windows' own spring-loaded folder delay is roughly this; browsers
/// sit between 400 and 800 ms. Short enough not to feel stuck, long enough
/// that sweeping the pointer across the strip on the way to the far tab does
/// not open three documents in passing.
pub const SPRING_DWELL: f64 = 0.6;

/// Named region: the strip as a whole.
const REGION_STRIP: &str = "doc-tabs"; // ui-text-exempt: trace region name, never displayed

/// Named region prefix: one per **drawn** tab, with the slot appended.
///
/// Indexed by slot rather than by position among the drawn, so a check that
/// scrolls the strip keeps naming the same document. Absent for a tab behind
/// the overflow affordance, which is itself the fact an overflow check wants.
const REGION_TAB_PREFIX: &str = "doc-tab."; // ui-text-exempt: trace region name, never displayed

/// Trace slot for the once-per-change summary of what the strip drew.
const STRIP_SLOT: &str = "doc-tabs"; // ui-text-exempt: trace slot name, never displayed

/// Trace slot for *"the pointer is resting on a tab with a drag in flight"* —
/// the gate between "no hover" and "hovered but never dwelt long enough".
const HOVER_SLOT: &str = "doc-tab-hover"; // ui-text-exempt: trace slot name, never displayed

/// What the spring timer is watching, between frames.
/// `Default` is derived only because `egui::IdTypeMap::remove_temp` requires
/// it. The defaulted value — slot 0 at time 0 — is never constructed by this
/// module and means nothing; every real one is built beside a live hover.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
struct Spring {
    /// The tab the pointer has been resting on.
    slot: usize,
    /// When it arrived, on `egui`'s input clock.
    since: f64,
}

impl PdfcerApp {
    /// **Draw the document tab strip**, and act on what the operator did to
    /// it.
    ///
    /// Draws nothing at all when no document is open — the strip is not a
    /// place to put an "open a file" invitation, and an empty strip is 26
    /// points of furniture asserting that there is something to switch
    /// between.
    ///
    /// Activation is applied **here**, immediately, rather than raised as an
    /// [`Action`]: switching documents destroys nothing and asks nobody, so
    /// routing it through the action funnel would buy the funnel's guarantee
    /// (one choke point for things that change the document) at the cost of a
    /// frame of latency on a control the operator is watching. Closing is the
    /// opposite and *is* an action, because it discards work and has to go
    /// through the unsaved-edits guard.
    pub(super) fn document_tabs(&mut self, ui: &mut egui::Ui, actions: &mut Vec<Action>) {
        let count = self.document_count();
        if count == 0 {
            // Nothing open. Clear any spring left over from a drag that ended
            // with the last document closing, so the next one does not inherit
            // a clock that started before it existed.
            ui.ctx().data_mut(|d| d.remove_temp::<Spring>(spring_id()));
            return;
        }

        // ★ Everything that needs `&self` is read BEFORE the strip is drawn,
        // and everything that needs `&mut self` is applied after.
        //
        // The reason is the menu host: it borrows `self.shell` and
        // `self.commands` for as long as it lives, and the intents the strip
        // produces need `&mut self` to apply. Building the tabs and the host
        // first, drawing into locals, and mutating afterwards is the same
        // "draw first, dispatch second" shape `crate::app::surfaces::central`
        // uses for the canvas, and for the same borrow.
        let tabs: Vec<egui_shell::tabstrip::TabItem> =
            (0..count).map(|slot| self.tab_item(slot)).collect();
        let theme = egui_shell::theme::Theme::of(ui.ctx());
        let active = self.active_slot;
        let conditions = self.conditions(ui.ctx());
        let host = self
            .shell
            .as_ref()
            .map(|s| crate::shell::menus::MenuHost::new(s, &self.commands, &conditions));

        let strip = egui_shell::tabstrip::strip(ui, &theme, &tabs, active);

        // ★★ The context menu, attached to each tab's own response.
        //
        // `egui_shell::tabstrip` deliberately attaches none of its own — a
        // `Response` carries exactly one popup id, so whoever attaches first
        // owns it, and *what* a right-click on a document should offer is
        // domain knowledge R7 forbids that crate. See `TabStrip::responses`.
        //
        // The tab under the pointer is remembered as the menu's **operand**,
        // because `window.close_document` and `window.close_other_documents`
        // act on the tab that was right-clicked and not on the one on screen.
        // Parked for one frame in the same shape `recent_choice` uses, and for
        // the same reason: the shell's menu reports a `HandlerToken` and has no
        // channel for an operand.
        let mut menu_tokens: Vec<(usize, egui_shell::HandlerToken)> = Vec::new();
        if let Some(host) = &host {
            for (slot, response) in &strip.responses {
                for token in host.attach(response, crate::shell::menus::DOCUMENT_TAB) {
                    menu_tokens.push((*slot, token));
                }
            }
        }
        // ★ NOT `drop(host)`. `MenuHost` is `Copy`, so dropping it does
        // nothing at all and clippy says so — the borrow of `self.shell` and
        // `self.commands` ends where the binding's last USE is, which is the
        // loop above. Naming that here rather than trusting it: everything
        // below this line needs `&mut self`, and it compiles because non-lexical
        // lifetimes have already released both.

        crate::diag::ui_rect(REGION_STRIP, ui.max_rect());
        for (slot, rect) in &strip.drawn {
            crate::diag::ui_rect(&format!("{REGION_TAB_PREFIX}{slot}"), *rect);
        }

        // ★ Spring-loading, before the intents are applied.
        //
        // Before, because a spring that fires this frame changes
        // `active_slot`, and an `Activate` intent produced by a click in the
        // same frame must win over it — the operator's click is a statement
        // and the dwell is an inference.
        self.spring_loaded_hover(ui.ctx(), strip.hovered);

        for intent in strip.intents {
            match intent {
                egui_shell::tabstrip::TabIntent::Activate(slot) => self.activate_slot(slot),
                // Through the funnel, and therefore through both guards. See
                // this function's own docs for why activation is not.
                egui_shell::tabstrip::TabIntent::Close(slot) => {
                    actions.push(Action::CloseDocument(slot));
                }
                // Applied here with activation, and for the same reason:
                // rearranging the strip discards nothing and asks nobody. It is
                // also the one act in this file that must be visible on the
                // frame it happens — a tab that lags a frame behind the pointer
                // that dropped it reads as a strip that did not take the drop.
                egui_shell::tabstrip::TabIntent::Reorder { from, gap } => {
                    self.move_slot(from, gap);
                }
            }
        }

        // ★ The menu's commands, dispatched after the borrow that drew them
        // has ended — and through the ordinary dispatcher, so a row in this
        // menu and the same command anywhere else cannot diverge.
        //
        // The operand is parked immediately before each dispatch rather than
        // once for the frame: a menu can only produce one token, but parking it
        // beside its own dispatch is what keeps *"which tab did this come
        // from"* impossible to get wrong if that ever stops being true.
        for (slot, token) in menu_tokens {
            self.tab_menu_target = Some(slot);
            self.dispatch_token(ui.ctx(), token, actions);
            self.tab_menu_target = None;
        }

        crate::diag::trace_changed(STRIP_SLOT, || {
            format!(
                // ui-text-exempt: diagnostic trace, never displayed in the UI
                "doc-tabs open={count} active={} drawn={} hidden={}",
                self.active_slot,
                strip.drawn.len(),
                strip.hidden,
            )
        });
    }

    /// One tab, built from one slot's [`Status`].
    ///
    /// Four arms because there are four things a tab can be, and collapsing
    /// the three unopened ones would lose the distinction
    /// [`crate::app::lifecycle`] exists to preserve: *the file is wrong*, *the
    /// file is fine and pdfcer is not finished*, *the file is encrypted and
    /// pdfcer has not been told the password*.
    fn tab_item(&self, slot: usize) -> egui_shell::tabstrip::TabItem {
        use crate::text::doctabs as t;
        match self.slot(slot) {
            Some(Status::Open(doc)) => {
                // `EditSession::is_modified` rather than an epoch counter kept
                // here: the engine owns the command log, and a shell-side copy
                // would be a second answer to a question with one owner. See
                // `app::conditions`' undo/redo note, which makes the same
                // argument at greater length.
                // ★★★ O65: `is_modified()` is the engine's "differs from the
                // BASE revision", and an incremental save takes `&self`, so
                // the base never moves and the marker never cleared. A tab
                // that keeps its dot after a successful save is the visible
                // half of the same defect that made Close ask about a saved
                // document.
                let unsaved = crate::app::save::has_unsaved_edits(doc);
                egui_shell::tabstrip::TabItem::new(
                    t::tab_label(&doc.path, unsaved),
                    if doc.origin == Origin::Created {
                        t::tab_tooltip_created(&doc.path)
                    } else {
                        t::tab_tooltip_open(&doc.path, unsaved)
                    },
                )
            }
            Some(Status::Failed { path, message } | Status::Unsupported { path, message }) => {
                egui_shell::tabstrip::TabItem::new(
                    t::tab_label(path, false),
                    t::tab_tooltip_unopened(path, message),
                )
            }
            Some(Status::NeedsPassword { path }) => egui_shell::tabstrip::TabItem::new(
                t::tab_label(path, false),
                t::tab_tooltip_unopened(path, t::tab_reason_needs_password()),
            ),
            // Unreachable while the invariant in `documents` §2 holds, and
            // rendered rather than panicked for the reason that module's
            // `put_slots` clamps instead of asserting: a wrong tab is a
            // cosmetic fault and a panic mid-close costs every other document.
            Some(Status::Empty) | None => egui_shell::tabstrip::TabItem::new(
                t::tab_label(std::path::Path::new(""), false),
                t::tab_label(std::path::Path::new(""), false),
            ),
        }
    }

    /// §3 — **activate the tab the pointer has been dwelling on**, but only
    /// while a page drag is in flight.
    ///
    /// Gated on the drag, deliberately and not defensively. Spring-loading a
    /// tab under an ordinary pointer would make the strip change documents
    /// because the operator paused on their way to the ribbon, which is the
    /// application taking initiative the operator did not ask for — the
    /// behaviour `view.app_initiative` was retired for *specifying* rather
    /// than for doing.
    fn spring_loaded_hover(&mut self, ctx: &egui::Context, hovered: Option<usize>) {
        if !crate::pagedrag::in_flight(ctx) {
            ctx.data_mut(|d| d.remove_temp::<Spring>(spring_id()));
            return;
        }
        let now = ctx.input(|i| i.time);
        let Some(slot) = hovered.filter(|s| *s != self.active_slot) else {
            ctx.data_mut(|d| d.remove_temp::<Spring>(spring_id()));
            return;
        };

        // ★ A diagnostic at the ENTRY of each gate, naming it.
        //
        // `CONTINUE.md` §7: *an instrument that can only return one answer
        // cannot detect the thing it was added to detect*. `doc-tab-spring` is
        // emitted only when the spring FIRES, so its absence used to have three
        // indistinguishable meanings — no drag, no hover, or a hover that never
        // reached the dwell. This line separates the second from the third,
        // which is the pair that actually cost a driven run.
        //
        // De-duplicated on the slot, so resting on a tab costs one line rather
        // than one per frame.
        crate::diag::trace_changed(HOVER_SLOT, || {
            format!(
                // ui-text-exempt: diagnostic trace, never displayed in the UI
                "doc-tab-hover slot={slot} armed=1"
            )
        });

        let spring = ctx.data(|d| d.get_temp::<Spring>(spring_id()));
        match spring {
            // A different tab, or the first frame over this one: restart the
            // clock. §3's cancellation rule, and the reason the slot is stored
            // beside the timestamp rather than a bare instant.
            Some(Spring { slot: was, .. }) if was != slot => {
                ctx.data_mut(|d| d.insert_temp(spring_id(), Spring { slot, since: now }));
            }
            None => {
                ctx.data_mut(|d| d.insert_temp(spring_id(), Spring { slot, since: now }));
            }
            Some(Spring { since, .. }) => {
                if now - since >= SPRING_DWELL {
                    crate::diag::trace(|| {
                        format!(
                            // ui-text-exempt: diagnostic trace, never displayed in the UI
                            "doc-tab-spring slot={slot} dwell={:.2}",
                            now - since
                        )
                    });
                    ctx.data_mut(|d| d.remove_temp::<Spring>(spring_id()));
                    self.activate_slot(slot);
                }
            }
        }
        // A dwell in progress is a thing that changes with no input, so the
        // frame after it must happen whether or not the pointer moves.
        // Without this the spring fires only when something else asks for a
        // repaint, which on a stationary pointer is never.
        ctx.request_repaint_after(std::time::Duration::from_millis(50));
    }
}

/// The spring timer's memory key.
fn spring_id() -> egui::Id {
    egui::Id::new("pdfcer-doc-tab-spring") // ui-text-exempt: an id, never displayed
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn failed(name: &str) -> Status {
        Status::Failed {
            path: PathBuf::from(name),
            // ui-text-exempt: test fixture, never displayed
            message: String::from("not a PDF"),
        }
    }

    /// **A file that would not open still gets a tab, with the reason on it.**
    ///
    /// `documents` §2's rule, asserted at the surface that would otherwise
    /// quietly drop it — because the failure mode is an operator who opens a
    /// damaged file and loses the three documents they had open.
    #[test]
    fn a_failed_open_is_a_tab_that_says_why() {
        let mut app = PdfcerApp::new();
        app.park_and_adopt(failed("D:/jobs/broken.pdf"));
        let item = app.tab_item(0);
        assert_eq!(item.label, "broken.pdf");
        assert!(
            item.tooltip.contains("not a PDF"),
            "the tab did not carry the reason: {}",
            item.tooltip
        );
    }

    /// ★ **The unsaved marker leads the label.**
    ///
    /// Asserted rather than trusted because the whole argument for the prefix
    /// is about truncation, and a trailing marker would pass any test that
    /// merely looked for the character somewhere in the string.
    #[test]
    fn the_unsaved_marker_is_where_truncation_cannot_reach_it() {
        let label = crate::text::doctabs::tab_label(std::path::Path::new("D:/j/SW41177.pdf"), true);
        assert!(
            label.starts_with('*'),
            "the marker must lead, or a crowded strip eats it: {label}"
        );
        assert!(label.ends_with("SW41177.pdf"));
        let clean =
            crate::text::doctabs::tab_label(std::path::Path::new("D:/j/SW41177.pdf"), false);
        assert_eq!(
            clean, "SW41177.pdf",
            "an unmodified document carries no marker"
        );
    }

    /// **A path with no file name still produces a readable tab.** An empty
    /// tab is indistinguishable from a rendering failure.
    #[test]
    fn a_nameless_path_does_not_produce_an_empty_tab() {
        let label = crate::text::doctabs::tab_label(std::path::Path::new("D:/"), false);
        assert!(!label.is_empty(), "a root path produced a blank tab");
    }
}
