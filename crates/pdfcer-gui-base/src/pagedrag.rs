//! # `pagedrag` — a page drag in flight, wherever it started and wherever it
//! ends
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/pagedrag.md`.

/// **A page drag in flight.**
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PageDrag {
    /// **Which document the pages came from**, as a tab position.
    ///
    /// A tab position rather than a path or a pointer, because that is what
    /// every consumer needs to compare against `PdfcerApp::active_slot` and
    /// what `PdfcerApp::slot` takes. Slot positions are stable while a drag is
    /// in flight: activating a tab does not renumber the strip
    /// (`pdfcer_gui::app::documents`' `the_strip_order_is_independent_of_which_tab_is_active`
    /// pins that), and nothing closes a tab mid-drag.
    pub source_slot: usize,
    /// **What is being dragged** — 0-based page indices into the source
    /// document, ascending and distinct.
    ///
    /// Captured at press. See this module's header for why, and for why the
    /// Pages panel's own reorder drag deliberately does the opposite.
    pub pages: Vec<usize>,
    /// The tile the press landed on, for the source document's own no-op test
    /// (*"dropping a page back where it already is changes nothing"*).
    pub origin: usize,
    /// The source document's tab label, for the caption.
    ///
    /// Carried rather than looked up because the caption is drawn by whichever
    /// surface the pointer is over, and that surface has an `OpenDoc` — the
    /// *target's* — rather than the application.
    pub source_label: String,
}

/// **Where the drag would land**, as resolved by whichever surface the pointer
/// is over this frame.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DropLanding {
    /// The document the drop would land in, as a tab position.
    pub target_slot: usize,
    /// The **gap**, 0-based: `0` is before the first page, `page_count` is
    /// after the last.
    pub gap: usize,
    /// The target document's page count, so the caption can say *"at the
    /// end"* rather than *"before page 13"* when there is no page 13.
    pub page_count: usize,
    /// Whether the drop would actually do anything.
    ///
    /// `false` for a same-document drag whose gap is inside its own operand
    /// run (nothing moves) and for a whole-document self-copy, which is
    /// refused — see `pdfcer_gui::text::doctabs::drag_refused_self_copy`.
    pub lands: bool,
}

/// **Which document every surface is drawing this frame**, published once by
/// the application.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ActiveDocument {
    /// Its tab position.
    pub slot: usize,
    /// Its tab label, for the drag caption.
    pub label: String,
}

/// The active document's key.
fn active_key() -> egui::Id {
    egui::Id::new("pdfcer-active-document") // ui-text-exempt: an id, never displayed
}

/// **Publish which document is on screen.** Called once per frame, by the
/// application, before any surface draws.
pub fn publish_active(ctx: &egui::Context, slot: usize, label: String) {
    ctx.data_mut(|d| d.insert_temp(active_key(), ActiveDocument { slot, label }));
}

/// Forget which document is on screen — nothing is open.
pub fn clear_active(ctx: &egui::Context) {
    ctx.data_mut(|d| d.remove_temp::<ActiveDocument>(active_key()));
}

/// Which document is on screen, or `None` when nothing is.
#[must_use]
pub fn active(ctx: &egui::Context) -> Option<ActiveDocument> {
    ctx.data(|d| d.get_temp::<ActiveDocument>(active_key()))
}

/// **Turn a gap between pages into the engine's own insertion vocabulary.**
#[must_use]
pub fn insert_position(gap: usize, page_count: usize) -> pdfcer_core::pageops::InsertPosition {
    use pdfcer_core::pageops::InsertPosition;
    if gap == 0 {
        InsertPosition::Start
    } else if gap >= page_count {
        InsertPosition::End
    } else {
        InsertPosition::Before(gap)
    }
}

/// The memory key. **The only one in the application**, and the reason this
/// module is the only place that names it.
fn key() -> egui::Id {
    egui::Id::new("pdfcer-page-drag") // ui-text-exempt: an id, never displayed
}

/// **This frame's answer**, written by whichever surface the pointer is inside.
///
/// Kept separate from the drag itself so a surface can update where the drop
/// would go without rewriting the drag sixty times a second.
fn landing_key() -> egui::Id {
    egui::Id::new("pdfcer-page-drag-landing") // ui-text-exempt: an id, never displayed
}

/// **The PREVIOUS frame's answer**, which is the one the caption reads.
fn landing_shown_key() -> egui::Id {
    egui::Id::new("pdfcer-page-drag-landing-shown") // ui-text-exempt: an id, never displayed
}

/// **Begin a drag.** Replaces any drag already in flight, which cannot happen
/// — a second press cannot arrive while a button is held — but is the right
/// behaviour if it ever does.
pub fn begin(ctx: &egui::Context, drag: PageDrag) {
    crate::diag::trace(|| {
        format!(
            // ui-text-exempt: diagnostic trace, never displayed in the UI
            "page-drag-start slot={} origin={} carrying={}",
            drag.source_slot,
            drag.origin,
            drag.pages.len()
        )
    });
    ctx.data_mut(|d| d.insert_temp(key(), drag));
}

/// **The drag in flight**, if there is one.
#[must_use]
pub fn current(ctx: &egui::Context) -> Option<PageDrag> {
    ctx.data(|d| d.get_temp::<PageDrag>(key()))
}

/// Is a drag in flight? The cheap question, for surfaces that only need to
/// know whether to offer themselves as a target.
#[must_use]
pub fn in_flight(ctx: &egui::Context) -> bool {
    ctx.data(|d| d.get_temp::<PageDrag>(key()).is_some())
}

/// **End the drag and take it**, leaving nothing behind.
pub fn end(ctx: &egui::Context) -> Option<PageDrag> {
    let taken = ctx.data_mut(|d| d.remove_temp::<PageDrag>(key()));
    ctx.data_mut(|d| {
        d.remove_temp::<DropLanding>(landing_key());
        d.remove_temp::<DropLanding>(landing_shown_key());
    });
    taken
}

/// **Publish where the drop would land**, from the surface the pointer is
/// over.
pub fn set_landing(ctx: &egui::Context, landing: DropLanding) {
    ctx.data_mut(|d| d.insert_temp(landing_key(), landing));
}

/// **Rotate the two landing slots.** Called once per frame by the application,
/// before any surface draws.
///
/// See [`landing_shown_key`] for why the clear belongs here and to nobody
/// else.
pub fn begin_frame(ctx: &egui::Context) {
    let pending = ctx.data_mut(|d| d.remove_temp::<DropLanding>(landing_key()));
    ctx.data_mut(|d| match pending {
        Some(landing) => {
            d.insert_temp(landing_shown_key(), landing);
        }
        None => {
            d.remove_temp::<DropLanding>(landing_shown_key());
        }
    });
}

/// Where the drop would land, as the **previous** frame resolved it.
#[must_use]
pub fn landing(ctx: &egui::Context) -> Option<DropLanding> {
    ctx.data(|d| d.get_temp::<DropLanding>(landing_shown_key()))
}

/// **Is the operator asking for a MOVE rather than a copy?**
#[must_use]
pub fn wants_move(ctx: &egui::Context) -> bool {
    ctx.input(|i| i.modifiers.shift)
}

/// What the drag in flight would do if released now: the decision the
/// status-row caption words, in `text::doctabs::drag_caption`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DragPhase {
    /// Over nothing that accepts pages.
    OverNothing,
    /// Over a page list or view that refuses this drop.
    LandsNowhere,
    /// Within the source document: a reorder, already a move.
    Reorder {
        moving: usize,
        gap: usize,
        page_count: usize,
    },
    /// Into another document with the move modifier held.
    Move {
        moving: usize,
        gap: usize,
        source: String,
        page_count: usize,
    },
    /// Into another document: a copy, the default.
    Copy {
        moving: usize,
        gap: usize,
        source: String,
        page_count: usize,
    },
}

/// The drag in flight's [`DragPhase`], or `None` when no drag is in flight.
#[must_use]
pub fn phase(ctx: &egui::Context) -> Option<DragPhase> {
    let drag = current(ctx)?;
    let Some(landing) = landing(ctx) else {
        return Some(DragPhase::OverNothing);
    };
    if !landing.lands {
        return Some(DragPhase::LandsNowhere);
    }
    let moving = drag.pages.len();
    let (gap, page_count) = (landing.gap, landing.page_count);
    if landing.target_slot == drag.source_slot {
        return Some(DragPhase::Reorder {
            moving,
            gap,
            page_count,
        });
    }
    let source = drag.source_label;
    Some(if wants_move(ctx) {
        DragPhase::Move {
            moving,
            gap,
            source,
            page_count,
        }
    } else {
        DragPhase::Copy {
            moving,
            gap,
            source,
            page_count,
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn drag() -> PageDrag {
        PageDrag {
            source_slot: 1,
            pages: vec![2, 3],
            origin: 2,
            // ui-text-exempt: test fixture, never displayed
            source_label: String::from("source.pdf"),
        }
    }

    /// **A drag survives being read**, which is the property every consumer
    /// depends on: four surfaces look at it in one frame and none of them may
    /// consume it.
    #[test]
    fn reading_a_drag_does_not_end_it() {
        let ctx = egui::Context::default();
        begin(&ctx, drag());
        assert!(in_flight(&ctx));
        assert_eq!(current(&ctx), Some(drag()));
        assert!(in_flight(&ctx), "reading it took it");
        assert_eq!(end(&ctx), Some(drag()));
        assert!(!in_flight(&ctx), "ending it left it");
    }

    /// **Ending a drag clears the landing too.**
    #[test]
    fn ending_a_drag_clears_where_it_would_have_landed() {
        let ctx = egui::Context::default();
        begin(&ctx, drag());
        set_landing(
            &ctx,
            DropLanding {
                target_slot: 0,
                gap: 3,
                page_count: 9,
                lands: true,
            },
        );
        begin_frame(&ctx);
        assert!(landing(&ctx).is_some());
        end(&ctx);
        assert!(landing(&ctx).is_none(), "the caret outlived its drag");
    }

    /// **A reorder and a copy are different phases**, because those are the
    /// two different things the same gesture does.
    #[test]
    fn the_phase_is_copy_only_when_the_documents_differ() {
        let ctx = egui::Context::default();
        begin(&ctx, drag());

        set_landing(
            &ctx,
            DropLanding {
                target_slot: 1, // the source
                gap: 0,
                page_count: 9,
                lands: true,
            },
        );
        begin_frame(&ctx);
        assert_eq!(
            phase(&ctx),
            Some(DragPhase::Reorder {
                moving: 2,
                gap: 0,
                page_count: 9
            })
        );

        set_landing(
            &ctx,
            DropLanding {
                target_slot: 0, // somewhere else
                gap: 0,
                page_count: 9,
                lands: true,
            },
        );
        begin_frame(&ctx);
        assert_eq!(
            phase(&ctx),
            Some(DragPhase::Copy {
                moving: 2,
                gap: 0,
                source: drag().source_label,
                page_count: 9
            })
        );
    }

    /// **A gap becomes the engine's vocabulary, with both ends named.**
    #[test]
    fn a_gap_maps_onto_an_insert_position() {
        use pdfcer_core::pageops::InsertPosition;
        assert_eq!(insert_position(0, 5), InsertPosition::Start);
        assert_eq!(insert_position(1, 5), InsertPosition::Before(1));
        assert_eq!(insert_position(4, 5), InsertPosition::Before(4));
        assert_eq!(insert_position(5, 5), InsertPosition::End);
        assert_eq!(
            insert_position(9, 5),
            InsertPosition::End,
            "a gap past the end is the end, not a request to repair"
        );
        assert_eq!(
            insert_position(0, 0),
            InsertPosition::Start,
            "an empty document has exactly one gap and it is the start"
        );
    }

    /// **No drag, no phase.** The status row asks unconditionally.
    #[test]
    fn there_is_nothing_to_say_when_nothing_is_being_dragged() {
        let ctx = egui::Context::default();
        assert!(phase(&ctx).is_none());
    }
}
