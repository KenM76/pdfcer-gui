//! # `pagedrag` — a page drag in flight, wherever it started and wherever it
//! ends
//!
//! Design and rationale: `docs/modules/pdfcer-gui/pagedrag.md`.

/// **A page drag in flight.**
///
/// Cheap to clone — a slot number, a short vector of page indices and a label
/// — because every reader clones it out of memory rather than holding a
/// borrow across a closure that draws.
/// `Default` is derived for one reason: `egui::IdTypeMap::remove_temp` demands
/// it of anything it can take back out. A defaulted `PageDrag` — slot 0,
/// carrying nothing — is never constructed here and is not a state the
/// application can reach; [`current`] answers `Option`, so "no drag" is
/// `None` and never an empty drag.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PageDrag {
    /// **Which document the pages came from**, as a tab position.
    ///
    /// A tab position rather than a path or a pointer, because that is what
    /// every consumer needs to compare against `PdfcerApp::active_slot` and
    /// what `PdfcerApp::slot` takes. Slot positions are stable while a drag is
    /// in flight: activating a tab does not renumber the strip
    /// (`crate::app::documents`' `the_strip_order_is_independent_of_which_tab_is_active`
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
///
/// Written every frame by exactly one surface — the pages panel or the canvas,
/// whichever the pointer is inside — and cleared by both when the pointer is
/// over neither. Read one frame later by the caption, for
/// `PagesUi::drag_landing`'s reason: *a gap has no position until the rows have
/// been placed, and the rows are placed below the header*.
/// `Default` is derived for [`PageDrag`]'s reason and means as little.
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
    /// refused — see [`crate::text::doctabs::drag_refused_self_copy`].
    pub lands: bool,
}

/// **Which document every surface is drawing this frame**, published once by
/// the application.
///
/// ## Why this is in memory rather than a parameter
///
/// Because three surfaces need it and none of them is given it: the Pages
/// panel is handed a `&OpenDoc` and no idea which tab it belongs to, the
/// canvas the same, and the status bar reads a `&Status`. Threading a slot
/// number and a label through `panels::Panel::show`, `canvas::show` and
/// `status::show` would put a document-tab concept into three signatures that
/// have nothing else to do with tabs, and every panel that does not care would
/// carry it anyway.
///
/// The precedent is `egui_shell::theme::Theme::of`, which does exactly this
/// for exactly this reason — a fact the whole application needs, published
/// once per frame into the context, read wherever it is wanted. The property
/// that makes it safe in both cases is that there is **one writer**, at a
/// known point in the frame, before anything reads.
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
///
/// `gap` counts boundaries: `0` is before the first sheet, `page_count` is
/// after the last. `pdfcer_core::pageops::InsertPosition` counts pages, so the
/// two ends have their own names.
///
/// `Start` and `End` rather than `Before(0)` and `Before(count)`, even
/// though `InsertPosition::slot` clamps both to the same answer. The named
/// variants say *"at the beginning"* and *"at the end"* — which is what the
/// operator meant and what survives the document changing length between the
/// gesture and the edit. `Before(12)` on an eleven-page document is a request
/// that has to be repaired; `End` is one that cannot go wrong.
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
///
/// Returns what was in flight so the caller can act on it. Also clears the
/// landing, because a landing that outlived its drag would be a caret nobody
/// can get rid of — the exact failure `panels::pages::settle_drag` documents
/// for the same reason.
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
///
/// Shift, read live — so the answer changes as the key goes down and up, the
/// caption follows it, and the state **at the moment of release** is what the
/// drop uses. That is what Windows does: the modifier is not latched at the
/// press, it is sampled at the drop, which is why Explorer's cursor badge
/// changes under your hand mid-drag.
///
/// ## Why Shift, and not Ctrl
///
/// Because on this desktop Ctrl means *copy* and Shift means *move*, and has
/// since the mid-nineties. `crate::text::doctabs::drag_landing_move` carries
/// the table. Copy is already the unmodified behaviour here — two documents
/// are two files with two undo stacks, which is the "different volumes" case —
/// so Ctrl is a no-op that asks for what it already gets, and Shift is the one
/// that changes the verb.
///
/// ## It means nothing within one document
///
/// A drag that begins and ends in the same document is a reorder, which is
/// already a move; there is nothing for a modifier to select between. Callers
/// consult this only on the cross-document branch, and the caption only offers
/// the hint there.
#[must_use]
pub fn wants_move(ctx: &egui::Context) -> bool {
    ctx.input(|i| i.modifiers.shift)
}

/// **The sentence describing what this drag is about to do**, or `None` when
/// no drag is in flight.
///
/// Here rather than in the status bar because it is the one place that has
/// both halves — the drag and the landing — and because putting it in the
/// caller would mean writing it twice, once for the Pages panel's header and
/// once for the status row.
///
/// R8b rule 4: this is **off-canvas disclosure**. The caret drawn into the
/// page list and the page view is a *pre-commit affordance* — a cursor — which
/// that rule explicitly welcomes. What it forbids is styling content that has
/// already been applied, and nothing here does that: the moment the drop is
/// made, the arrived pages render exactly as pages that were always there.
#[must_use]
pub fn caption(ctx: &egui::Context) -> Option<String> {
    let drag = current(ctx)?;
    let Some(landing) = landing(ctx) else {
        return Some(crate::text::doctabs::drag_over_nothing().to_owned());
    };
    if !landing.lands {
        return Some(crate::text::pages::drag_lands_nowhere().to_owned());
    }
    if landing.target_slot == drag.source_slot {
        // Within one document the drag is a reorder, which is already a move.
        // No modifier applies and none is offered.
        return Some(crate::text::doctabs::drag_landing_here(
            drag.pages.len(),
            landing.gap,
            landing.page_count,
        ));
    }
    if wants_move(ctx) {
        return Some(crate::text::doctabs::drag_landing_move(
            drag.pages.len(),
            landing.gap,
            &drag.source_label,
            landing.page_count,
        ));
    }
    // The copy sentence AND the hint, from one catalogue function rather
    // than joined here. How two operator-visible sentences meet is itself an
    // operator-visible decision, so it belongs in the catalogue with them;
    // joining them with a `format!` here would put a sentence of UI text in a
    // module `check-ui-strings` does not read as one.
    Some(crate::text::doctabs::drag_landing_copy_with_hint(
        drag.pages.len(),
        landing.gap,
        &drag.source_label,
        landing.page_count,
    ))
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

    /// **The caption distinguishes a reorder from a copy**, because those are
    /// the two different things the same gesture does and the operator has no
    /// other way to tell which one they are about to get.
    #[test]
    fn the_caption_says_copy_only_when_the_documents_differ() {
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
        let same = caption(&ctx).expect("a drag is in flight");
        assert!(
            !same.to_lowercase().contains("copy"),
            "a reorder within one document was described as a copy: {same}"
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
        let other = caption(&ctx).expect("a drag is in flight");
        assert!(
            other.to_lowercase().contains("copy"),
            "a cross-document drag did not say it copies: {other}"
        );
        assert!(
            other.contains("source.pdf"),
            "the caption did not name the document the pages came from: {other}"
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

    /// **No drag, no caption.** The status row asks unconditionally.
    #[test]
    fn there_is_nothing_to_say_when_nothing_is_being_dragged() {
        let ctx = egui::Context::default();
        assert!(caption(&ctx).is_none());
    }
}
