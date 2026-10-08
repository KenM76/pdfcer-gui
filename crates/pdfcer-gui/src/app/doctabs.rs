//! # `app::doctabs` — the document tab strip, and the tab that springs open
//! under a drag
//!
//! Design and rationale: `docs/modules/pdfcer-gui/app/doctabs.md`.

use eframe::egui;

use crate::app::PdfcerApp;
use crate::app::actions::Action;
use crate::app::state::{Origin, Status};

/// **How long the pointer must rest on a tab before it springs open.**
pub const SPRING_DWELL: f64 = 0.6;

/// Named region: the strip as a whole.
const REGION_STRIP: &str = "doc-tabs"; // ui-text-exempt: trace region name, never displayed

/// Named region prefix: one per **drawn** tab, with the slot appended.
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

        // Everything that needs `&self` is read BEFORE the strip is drawn,
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
        // A tab's menu acts on that tab, so the window-move conditions are
        // asked again for each tab rather than for the one on screen.
        let per_tab: Vec<egui_shell::commands::ConditionSet> = (0..count)
            .map(|slot| {
                let mut set = conditions.clone();
                self.window_conditions(&mut set, slot);
                set
            })
            .collect();

        let strip = egui_shell::tabstrip::strip(ui, &theme, &tabs, active);

        // The context menu, attached to each tab's own response.
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
        if let Some(shell) = self.shell.as_ref() {
            for (slot, response) in &strip.responses {
                let Some(conditions) = per_tab.get(*slot) else {
                    continue;
                };
                let host = crate::shell::menus::MenuHost::new(shell, &self.commands, conditions);
                for token in host.attach(response, crate::shell::menus::DOCUMENT_TAB) {
                    menu_tokens.push((*slot, token));
                }
            }
        }
        // NOT `drop(host)`. `MenuHost` is `Copy`, so dropping it does
        // nothing at all and clippy says so — the borrow of `self.shell` and
        // `self.commands` ends where the binding's last USE is, which is the
        // loop above. Naming that here rather than trusting it: everything
        // below this line needs `&mut self`, and it compiles because non-lexical
        // lifetimes have already released both.

        crate::diag::ui_rect(REGION_STRIP, ui.max_rect());
        for (slot, rect) in &strip.drawn {
            crate::diag::ui_rect(&format!("{REGION_TAB_PREFIX}{slot}"), *rect);
        }

        // Spring-loading, before the intents are applied.
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

        // The menu's commands, dispatched after the borrow that drew them
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
    fn tab_item(&self, slot: usize) -> egui_shell::tabstrip::TabItem {
        use crate::text::doctabs as t;
        match self.slot(slot) {
            Some(Status::Open(doc)) => {
                // `EditSession::is_modified` rather than an epoch counter kept
                // here: the engine owns the command log, and a shell-side copy
                // would be a second answer to a question with one owner. See
                // `app::conditions`' undo/redo note, which makes the same
                // argument at greater length.
                // O65: `is_modified()` is the engine's "differs from the
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
            Some(Status::NeedsPassword { path, .. }) => egui_shell::tabstrip::TabItem::new(
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

        // A diagnostic at the ENTRY of each gate, naming it.
        //
        // **An instrument that can only return one answer cannot detect the
        // thing it was added to detect.** `doc-tab-spring` is emitted only when
        // the spring FIRES, so on its own its absence has three
        // indistinguishable meanings: no drag, no hover, or a hover that never
        // reached the dwell. This line separates the second from the third,
        // which is the pair a driven run has to tell apart to diagnose a
        // spring that never sprang.
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

    /// **The unsaved marker leads the label.**
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
