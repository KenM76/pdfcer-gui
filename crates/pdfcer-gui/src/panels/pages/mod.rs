//! # `panels::pages` — the document's pages, as pictures
//!
//! The thumbnail grid. `FEATURES.md`'s Phase 3 row — *"**Thumbnail grid** —
//! the Pages panel is not registered yet"* — and the last of the surfaces
//! `MODES_AND_PANELS.md` Part 1's table gives **all three** modes.
//!
//! | | |
//! |---|---|
//! | Ribbon command | `view.panel_pages` |
//! | Salvaged from | the old shell's `main.rs::thumbnail_rail` (~250 lines) and `raster::ThumbnailCache` |
//! | Acts on the document | [`Action::GoToPage`], and **nothing else** |
//! | Owns | [`select::PageSelection`] — the operand list the ribbon's Pages tab already promises |
//!
//! ## ★ Why a page panel sits in **Review**, and not only in Edit
//!
//! It is in all three default arrangements (`crate::app::modes::defaults::spec`), and
//! Review is the placement that needed an argument. `README.md` records the
//! operator's, and it is the reason this panel offers page verbs rather than
//! only navigation:
//!
//! > Reviewing a set means rotating a sheet to read it and extracting the
//! > pages you were asked about. The stance that matters is the content is
//! > not yours to alter, and page operations do not alter content.
//!
//! That is what separates rotate/extract/delete from the Edit tab's verbs. A
//! rotation changes `/Rotate`; an extraction writes a *different* file. Neither
//! touches a single content-stream operator, so neither breaches the stance
//! Review takes.
//!
//! ## What this panel draws, and what that costs
//!
//! The rendering and caching policy is [`thumbnails`]', and its header is the
//! one to read before changing anything here — it carries the measurements.
//! The one sentence that decides the shape of this file:
//!
//! > **A two-pixel render of the benchmark CAD drawing costs 691 ms.** ~99 %
//! > of a page's cost is resolution-independent, so a thumbnail is *not*
//! > cheap because it is small.
//!
//! Consequently this body renders **at most one page per frame**, only for
//! tiles that are actually on screen, and stops on its own the first time a
//! page proves expensive. An undrawn tile says so **in words**: a blank
//! rectangle the colour of paper is a picture of an *empty page*, which is a
//! thing a real PDF contains, so drawing one would assert something false
//! about the document rather than merely look unfinished.
//!
//! ## ★ Two surfaces this panel was built for and could not reach — **both
//! closed**
//!
//! Both were `shell/`'s rather than this module's, which is why they were
//! named here rather than left to be rediscovered, in the shape
//! `crate::shell::manifest::PLANNED` and `crate::app::modes::ABSENT_PANELS`
//! use for the same purpose. They are kept rather than deleted because each
//! closed a live hazard, and the next person to consider reopening one should
//! have to read what it cost.
//!
//! ### 1. — closed
//!
//! `view.panel_pages` had no registration. `crate::shell::manifest::PLANNED`
//! carried it, with a reason written for the *old* shell's furniture: *"page
//! thumbnails are the sidebar rail's first pane and have no independent
//! toggle. `view.sidebar` shows the rail."* There is no sidebar rail in this
//! build — there is a dock, and `crate::app::mod`'s panel registry registers a
//! panel **only if its command is registered**, so this panel was filtered out
//! of every default arrangement by `SHELL_FRAMEWORK.md` §5b's capability rule
//! and an operator never saw it. It was invisible rather than broken, which is
//! the honest failure and also the silent one.
//!
//! The entry was **stale rather than early** and is gone; the command is
//! registered and drawn, and `every_panel_is_reachable_from_the_ribbon` is the
//! test that made the staleness visible.
//!
//! ### 2. — closed
//!
//! `pages.row` had no definition when this panel was written: [`PAGES_ROW`] is
//! attached to every tile below, on every frame, through the same
//! [`MenuHost`] the canvas and the Objects panel use, but
//! `crate::shell::menus::built_in` defined four contexts and this was not one
//! of them — and `egui_shell::menu::Menu::attach` treats an unknown context as
//! *"this surface has no menu yet"*, so the right-click opened nothing at all.
//!
//! It is defined now, with the six verbs listed below, and **no edit was
//! needed here**: the attach site was already correct and the menu simply
//! started existing. That is the whole payoff of routing a right-click through
//! a context id rather than through a list of items at the call site.
//!
//! ## What a click does, and what it does not
//!
//! [`select`] owns the rule; the summary is that a plain click navigates and
//! picks one page, Ctrl+click toggles without navigating, and Shift+click
//! extends a range without navigating. **Only a plain click navigates**,
//! because building a five-page set that dragged the canvas through five
//! renders would cost ~4 s on a drawing set to perform a gesture that changes
//! nothing about what the operator is looking at.
//!
//! ## The selection is not a decoration
//!
//! Every one of the ribbon's Pages-tab tooltips already says *"the selected
//! pages"* — `pages.delete` is *"Remove **the selected pages** from this
//! document"* — and `crate::shell::commands`' own comment on that band says
//! those commands *"respect the thumbnail rail's selection when there is
//! one"*. This panel is where that selection comes from, and
//! [`crate::panels::PanelsState::selected_pages`] is how a dispatch arm reads
//! it.
//!
//! **Those arms exist now.** This paragraph used to end *"None of those arms
//! exists yet"* — for the whole of v0.1.0, during which every one of the six
//! verbs this panel's own context menu offers traced `command-unimplemented`
//! and did nothing. **All six now work** — rotate left and right, delete,
//! extract, move up and move down — through five dispatch arms, since the two
//! rotations share one and so do the two moves. The reading path is exactly the
//! one recorded here: through that accessor and through [`ops::operands`],
//! which is the single place the *"with nothing picked, act on the current
//! page"* rule is written down.
//!
//! ## ★ What an edit does to this panel's own state
//!
//! Nothing here has to remember anything, and that is by construction rather
//! than by discipline:
//!
//! | | how it is kept honest |
//! |---|---|
//! | the **thumbnails** | keyed on `(edit_epoch, pixels_per_point)`; [`thumbnails::ThumbnailCache::sync`] empties itself the moment the epoch moves, and the epoch moves on every edit |
//! | the **page count** and the tiles | read from `doc.pages` every frame, which `crate::app::actions::pages::resync` refreshes from the session on every edit |
//! | the **picks**, after a delete | cleared by the apply arm — every picked sheet is gone, so there is nothing to point at |
//! | the **picks**, after a reorder | **remapped**, through [`select::PageSelection::remap`]: the permutation states where each sheet went, so the arrows stay usable twice in a row |
//! | the **picks**, after anything else | [`select::PageSelection::retain_below`] on the next frame, which is belt to all of the above |

/// **A drawing dropped on the thumbnails becomes pages in this one** —
/// `OPERATOR_REQUESTS.md` O67. The panel's half of `app::filedrag`'s claim
/// protocol; every refusal in it is a fall-through to the ordinary meaning of
/// a drop rather than a message.
pub mod import;
pub mod ops;
pub mod select;
pub mod thumbnails;

use egui_shell::HandlerToken;

use crate::app::actions::Action;
use crate::app::actions::pages::PageAction;
use crate::app::state::OpenDoc;
use crate::panels::PanelsState;
use crate::shell::menus::MenuHost;
use crate::text::pages as t;
use thumbnails::TileState;

/// Right-click on a page tile in the Pages panel.
///
/// Defined by `crate::shell::menus::built_in` — see this module's header,
/// section 2. The constant lives here rather than being spelled at the attach
/// site for the reason that module gives for its own four: *"a context id is
/// used in exactly two places that must agree… a typo in either produces
/// silence rather than an error."*
///
/// `crate::shell::menus::PAGES_ROW` is the other spelling, and
/// [`tests::the_page_tile_menu_context_is_named_and_defined`] asserts the two
/// agree — because a menu attached to a context nobody defines opens nothing at
/// all, silently.
///
/// **The six verbs it offers all work now**, and none of them did for the whole
/// of v0.1.0: `crate::app::dispatch` had no `pages.*` arm at all, so every row
/// of this menu traced `command-unimplemented`. The three page commands that
/// still have no arm — `pages.split`, `pages.merge_into`,
/// `pages.insert_from_file` — are deliberately **not** on this menu, and the
/// dispatcher records what each of them is waiting for.
pub const PAGES_ROW: &str = "pages.row"; // ui-text-exempt: a menu context id, never displayed

/// The narrowest a tile may be drawn before the grid drops to one column.
///
/// Below roughly this width a drawing sheet's title block is no longer
/// legible and one thumbnail stops being distinguishable from the next, which
/// is the only job a thumbnail has. `crate::app::modes::defaults::NAVIGATOR_WIDTH` is
/// 280 pt *because* it fits two of these, and the two numbers are meant to
/// stay in step.
const MIN_TILE_WIDTH_PTS: f32 = 112.0;

/// The height reserved under each tile for its page number.
const CAPTION_HEIGHT_PTS: f32 = 16.0;

/// How thick the ring around the current page is drawn.
const CURRENT_RING_PTS: f32 = 2.0;

/// How much coloured mat a selected tile gets on each side.
///
/// A *shape* difference as well as a colour one — the tile visibly gains a
/// border where an unselected one has none. The old shell's rail put the same
/// reasoning behind its checkbox: *"a glyph AND a fill, never colour alone: a
/// colour-only state is invisible to a substantial fraction of operators."*
/// A mat is the version of that rule which needs no glyph, and therefore
/// cannot land on a font that has no glyph to draw — the failure that turned
/// the old rail's reorder arrows into empty boxes.
///
/// The header's *"N pages selected"* line is the third, wholly textual,
/// statement of the same fact.
const SELECTION_MAT_PTS: f32 = 3.0;

/// Points per millimetre, for the tooltip's sheet size.
///
/// A PDF user-space unit is 1/72 inch by definition (§8.3.2.3), and an inch
/// is 25.4 mm.
const PTS_PER_MM: f32 = 72.0 / 25.4;

/// Draw the Pages panel.
///
/// Returns the handler tokens a right-click produced — **intent**, never an
/// executed command. See [`crate::panels::Panel::show`] on why a panel must
/// not translate a context-menu command into an [`Action`] for itself.
#[must_use]
pub fn body(
    ui: &mut egui::Ui,
    doc: &OpenDoc,
    state: &mut PanelsState,
    host: Option<&MenuHost<'_>>,
    actions: &mut Vec<Action>,
) -> Vec<HandlerToken> {
    let pixels_per_point = ui.ctx().pixels_per_point();
    let page_count = doc.pages.len();
    let current = doc.view.page_index;
    // **Which document this panel is drawing**, as a tab position and a label.
    //
    // A panel is handed a `&OpenDoc` and no idea which tab it belongs to, so
    // the answer comes from the context — see `crate::pagedrag::ActiveDocument`
    // for why that is published rather than threaded, and for the
    // `Theme::of` precedent it follows.
    //
    // `unwrap_or_default()` is slot 0 with no label, which is what a unit test
    // that calls this body directly gets. It is never what a running
    // application gets: the frame publishes it before any surface draws.
    let here = crate::pagedrag::active(ui.ctx()).unwrap_or_default();
    let pages = state.pages_mut();

    // Everything that must be true before a tile is drawn, in one place:
    // the cache describes this revision at this density, and no picked page
    // names a sheet that has stopped existing.
    pages.cache.sync(&doc.page_epochs, pixels_per_point);
    pages.selection.retain_below(page_count);

    ui.label(t::pages_count(page_count));
    if page_count == 0 {
        // Reachable: a damaged `/Pages` node can flatten to nothing while the
        // file still opens. An empty grid would read as a panel that failed.
        ui.label(t::pages_none());
        return Vec::new();
    }
    if !pages.selection.is_empty() {
        ui.label(t::pages_selected(pages.selection.len()));
    }
    // ★★★ **THE DRAG CAPTION IS NOT DRAWN HERE, AND THAT IS A MEASURED
    // DEFECT RATHER THAN A PREFERENCE.**
    //
    // It was, for one day. A label above the grid saying where the drop would
    // land is the obvious place to put it — this panel's own `drag_landing`
    // sentence lived here from the day the reorder drag shipped, and the words
    // belong beside the caret they describe.
    //
    // It **moves the grid under the pointer while the drag is in flight.**
    // Driven, 2026-08-20, `pages_drag_shows_where_it_lands`, from the trace:
    //
    // ```text
    // before the drag   panel-pages-tile.1 rect=[[132 251.1] - [260 333.9]]
    // mid-drag          panel-pages-tile.1 rect=[[132 300.1] - [260 382.9]]   +49
    // mid-drag          panel-pages-tile.1 rect=[[132 285.1] - [260 367.9]]   +34
    // after the release panel-pages-tile.1 rect=[[132 251.1] - [260 333.9]]
    // ```
    //
    // Forty-nine points, then thirty-four, then back — because the caption is a
    // wrapping label in a 260 pt panel and **its own height depends on its own
    // wording**, which changes as the pointer moves between gaps and as Shift
    // goes down. So the sheet the operator is aiming at slides out from under
    // them, by a distance that is a function of the sentence describing what
    // they are aiming at.
    //
    // That is R128's feedback loop in a third place (`bottom_panel_height_...`
    // in the egui RAG is the first; the dimension-groups window was the
    // second), and the rule it yields is more general than R128's own wording:
    // **a surface may not change size in response to a gesture that is aimed
    // at it.**
    //
    // The status bar carries the sentence instead. It has a fixed height by
    // construction (`app::status`'s `exact_size` plus its own `set_min_height`,
    // both for R128), it is on screen in every mode including Read, and it is
    // where rule 4 puts off-canvas disclosure anyway. Nothing was lost by the
    // move except the defect.

    // The previews control. Read from the cache and written straight back, so
    // "is the box ticked" and "will anything be drawn" are one expression
    // rather than two that can disagree — see `ThumbnailCache::previews_on`.
    let mut previews_on = pages.cache.previews_on();
    if ui
        .checkbox(&mut previews_on, t::previews_label())
        .on_hover_text(t::previews_tooltip())
        .changed()
    {
        pages.cache.force_on(previews_on);
    }
    // The disclosure sits ABOVE the grid, not below it — the same rule the
    // Bookmarks, Signatures and Fonts panels state: an operator who looks at
    // a grid of undrawn tiles and stops has already drawn a conclusion by the
    // time a footnote would reach them.
    if let Some(slow) = pages.cache.slow() {
        ui.label(
            egui::RichText::new(t::previews_paused_note(slow.page_index, slow.millis))
                .small()
                .weak(),
        );
    }
    ui.separator();

    let mut go: Option<usize> = None;
    let mut tokens: Vec<HandlerToken> = Vec::new();
    let mut visible: Vec<usize> = Vec::new();
    // Where a drag in flight would land, resolved during the layout pass
    // because that is the only place a tile's rectangle exists. `None` when no
    // drag is in flight, or when the pointer is over no tile at all.
    let mut drop: Option<DropTarget> = None;

    let grid = egui::ScrollArea::vertical()
        .id_salt("pages-grid")
        .show(ui, |ui| {
            grid_rows(
                ui,
                doc,
                pages,
                current,
                host,
                &here,
                &mut visible,
                &mut go,
                &mut tokens,
                &mut drop,
            );
            // Painted INSIDE the scroll area and AFTER the rows, which is what
            // puts it on top of the tiles rather than under them — egui paints
            // in call order. Outside the closure the caret would be in the
            // parent's coordinate space and would not scroll with the grid it
            // is pointing into.
            paint_caret(ui, drop.as_ref());
        });

    // ★ The release is read from RAW POINTER INPUT, not from the tile's own
    // `Response`.
    //
    // The same discipline `canvas::guides::release` uses, and for the same
    // reason: a drag that began on a tile may end anywhere — over the header,
    // outside the panel, past the end of the last row — and a `Response` only
    // knows about releases inside the widget that produced it. Reading the
    // input means a drag always ends, which is the property that stops a
    // half-finished drag surviving into the next frame as a caret nobody can
    // get rid of.
    //
    // It runs AFTER the grid because `drop` is what the grid resolved.
    // Recorded BEFORE the settle, which consumes the drag: the header reads it
    // on the next frame, and a drag that has just ended must leave nothing for
    // it to read.
    // Publish where the drop would go, for the caption on the NEXT frame —
    // `DropLanding`'s own docs carry the one-frame argument, which is
    // `PagesUi::drag_landing`'s verbatim and unchanged by the move into
    // memory: a gap has no position until the rows have been placed, and the
    // rows are placed below the header.
    //
    // Written only when this panel actually resolved one. Not clearing it
    // otherwise is deliberate and is the whole reason the slot is rotated once
    // per frame instead: a panel can say *"the pointer is not over one of my
    // tiles"*, and cannot say whether it is over the page view instead. See
    // `crate::pagedrag::landing_shown_key`.
    if let Some(d) = drop {
        crate::pagedrag::set_landing(
            ui.ctx(),
            crate::pagedrag::DropLanding {
                target_slot: here.slot,
                gap: d.gap,
                page_count,
                lands: d.lands,
            },
        );
    }
    settle_drag(ui, doc, pages, drop.as_ref(), &here, actions);

    // ★★ **A file dropped on this panel imports its pages** —
    // `OPERATOR_REQUESTS.md` O67. Immediately after the page drag settles,
    // because the two are the same gesture with different operands and a
    // reader comparing them should find them together.
    //
    // The gap comes from whatever the grid resolved this frame, which for a
    // file drag was resolved from `app::filedrag`'s pointer rather than the
    // toolkit's — see `tile`. `None` means the pointer was on the panel but
    // over no tile, and `import::claim` reads that as the end of the document.
    import::claim(
        ui.ctx(),
        ui.min_rect(),
        drop.as_ref().map(|d| d.gap),
        page_count,
        actions,
    );

    // The two named regions a pixel check aims at. The panel's own rect comes
    // from the `Ui` rather than from a response, because the body is a column
    // of widgets and not a single one.
    crate::diag::ui_rect("panel-pages", ui.min_rect());
    crate::diag::ui_rect("panel-pages-grid", grid.inner_rect);

    // ★ One page per frame, chosen from what is on screen. See `thumbnails`'
    // header for why this is one and not two, and why it is here rather than
    // on the render worker.
    //
    // AFTER the grid rather than during it: the scheduling rule wants the
    // whole visible set, and rendering mid-layout would hold the frame in the
    // middle of a scroll area with half its rows placed.
    // ★★★ **AND NOT WHILE THE OPERATOR IS DOING SOMETHING** —
    // `OPERATOR_REQUESTS.md` O74, in his words:
    //
    // > *"The last thing that should matter is updating the preview."*
    //
    // That is a priority rule, not a bug report, and it is worth more than the
    // per-page invalidation it arrived with. A thumbnail render runs **inline
    // on the UI thread** (see `thumbnails`' header for why it is not on the
    // worker, and why moving it there would break `Arc::get_mut` in the edit
    // funnel), so a single expensive page can put 282 ms between a click and
    // what it does — measured, on his own 36-sheet set.
    //
    // Per-page invalidation shrinks how OFTEN that happens; it cannot stop a
    // page that genuinely needs redrawing from landing on the frame after the
    // click that dirtied it. This does: **the rail waits for a quiet moment.**
    //
    // Two conditions, and each answers a different way of being busy:
    //
    // 1. **No pointer or keyboard event this frame.** A drag, a chord, a scroll
    //    — anything the operator is in the middle of. Asked of `egui::Context`
    //    rather than of any one widget, because the question really is "is the
    //    operator doing something *anywhere*", which is the one case where the
    //    global read is the correct one.
    // 2. **`SETTLE_AFTER_EDIT` has passed since the last edit landed.**
    //    `OpenDoc::last_edit_at` is stamped in the edit funnel, in the same
    //    statement group as the epoch bump, precisely so a consumer can ask
    //    this. An edit is usually followed by another — a form is filled field
    //    after field — and re-rendering between two keystrokes is work thrown
    //    away before it is looked at.
    //
    // ★ It cannot stall the rail. `request_repaint_after` below wakes the
    // window when the quiet period expires, so a document left alone fills
    // itself; and an operator who keeps working keeps the deferral, which is
    // exactly the trade he asked for.
    let busy = ui.ctx().input(|i| {
        i.pointer.any_down()
            || i.pointer.is_moving()
            || !i.events.is_empty()
            || i.smooth_scroll_delta != egui::Vec2::ZERO
    });
    let settling = doc
        .last_edit_at
        .is_some_and(|at| at.elapsed() < SETTLE_AFTER_EDIT);
    if busy || settling {
        // Say why nothing was rendered, so a driven check can tell "deferred"
        // from "nothing to do" — two states with the same screenshot, which is
        // this project's recorded reason for tracing a decision rather than
        // only its outcome.
        crate::diag::trace_changed(DEFER_SLOT, || {
            format!(
                // ui-text-exempt: diagnostic trace, never displayed in the UI
                "pages-thumbnail-deferred busy={} settling={}",
                u8::from(busy),
                u8::from(settling)
            )
        });
        // Come back when the quiet period is over. Without this the window can
        // go idle mid-deferral and the rail stops filling until the operator
        // moves the mouse.
        ui.ctx().request_repaint_after(SETTLE_AFTER_EDIT);
    } else if let Some(page_index) = pages.cache.next_to_render(&visible, current)
        && let Some(page) = doc.pages.get(page_index)
    {
        let centre = viewport_centre(&visible, current);
        let elapsed = pages
            .cache
            .render(ui.ctx(), doc, page_index, page, pixels_per_point, centre);
        crate::diag::trace(|| {
            format!(
                "pages-thumbnail page={} ms={} state={:?} cached={}",
                page_index + 1,
                elapsed.as_millis(),
                pages.cache.state(page_index),
                pages.cache.ready_count(),
            )
        });
        // A page still to draw means another frame is wanted even if nothing
        // moved — otherwise the grid would fill only while the operator
        // happened to be generating input.
        ui.ctx().request_repaint();
    }

    crate::diag::trace_changed(PANEL_SLOT, || {
        format!(
            "pages-panel pages={page_count} current={} selected={} visible={} \
             drawn={} previews={}",
            current + 1,
            pages.selection.len(),
            visible.len(),
            pages.cache.ready_count(),
            u8::from(pages.cache.previews_on()),
        )
    });

    if let Some(page) = go {
        actions.push(Action::GoToPage(page));
    }
    tokens
}

/// Trace slot for the panel's once-per-change summary.
const PANEL_SLOT: &str = "pages-panel"; // ui-text-exempt: trace slot name, never displayed

/// Trace slot for the deferral line — `OPERATOR_REQUESTS.md` O74.
///
/// Its own slot rather than `PANEL_SLOT`'s, because the two answer different
/// questions and would overwrite each other every frame: one says what the
/// panel is showing, this says why it is not rendering.
const DEFER_SLOT: &str = "pages-thumbnail-deferred"; // ui-text-exempt: trace slot name, never displayed

/// ★★★ **How long after an edit the page rail stays out of the way** — O74.
///
/// The operator: *"The last thing that should matter is updating the
/// preview."*
///
/// # Why 250 ms, and what the number is answerable to
///
/// It has to be longer than the gap between two deliberate acts in one
/// sequence — ticking two check boxes, tabbing between two fields — because
/// rendering between them is work thrown away before anybody looks at it. And
/// it has to be short enough that a single edit followed by a pause feels like
/// the rail simply kept up.
///
/// 250 ms sits above the ~100-150 ms of a comfortable double act and well
/// below the ~500 ms at which a delay stops reading as "just happened". It is
/// deliberately **not** derived from a render cost: a slow document should
/// defer for the same period as a fast one, because the quantity being waited
/// for is the OPERATOR settling, not the renderer finishing.
///
/// ★ It is a constant rather than a literal so the next person to tune it does
/// so once, with a paper trail — the same argument `render::settle`'s
/// `ZOOM_SETTLE` makes, and this is its sibling on the other end of the frame.
const SETTLE_AFTER_EDIT: std::time::Duration = std::time::Duration::from_millis(250);

/// The prefix of the per-tile region names; the 0-based page index is appended.
///
/// Indexed by **page index**, not by position in the visible set, so a check
/// that scrolls or reorders keeps naming the same sheet.
const TILE_REGION_PREFIX: &str = "panel-pages-tile."; // ui-text-exempt: trace region name, never displayed

/// **Where a drag in flight would land**, resolved during the layout pass.
///
/// ## Why this exists at all, rather than the drag storing a gap
///
/// Because a gap has no position until the grid has been laid out. The panel
/// is a hand-rolled row layout over sheets of mixed sizes at a column count
/// derived from the dock's current width, so *"the boundary between page 6 and
/// page 7"* is a rectangle that only exists inside [`grid_rows`]. Resolving it
/// there and carrying it out is the same shape `visible`, `go` and `tokens`
/// already have: an answer only the layout pass is in a position to give.
///
/// ## Why the caret is a `Rect` and not a stroke
///
/// It is a **line**, and the two endpoints are all the layout pass knows. A
/// `Rect` carries both in one value the grid can build and [`paint_caret`] can
/// consume without either naming a colour or a width — which keeps the
/// *geometry* decision beside the tile rectangles and the *appearance*
/// decision beside the theme.
#[derive(Debug, Clone, Copy, PartialEq)]
struct DropTarget {
    /// The gap index the pointer is nearest, in [`ops`]' sense: `0` is before
    /// the first sheet, `page_count` is after the last.
    gap: usize,
    /// The line to draw, in the scroll area's own coordinate space.
    caret: egui::Rect,
    /// Whether releasing here would actually change the order.
    ///
    /// Computed by [`ops::drag_is_a_no_op`] against the operand set — the one
    /// place that rule lives, and the reason it is carried rather than
    /// recomputed at paint time is that the paint pass has no operand set: it
    /// runs after the borrow of `PagesUi` the grid needed.
    lands: bool,
}

/// How thick the insertion caret is drawn.
///
/// The same weight as [`CURRENT_RING_PTS`], deliberately: both are "the panel
/// pointing at something", and a caret thinner than the ring would read as a
/// hairline artefact on a dense grid rather than as a deliberate mark.
const CARET_PTS: f32 = 2.0;

/// How much of the caret's colour survives when the drop would change nothing.
///
/// ★ **Dimmed, not hidden.** Drawing no caret over a boundary that would not
/// land cannot be told apart from the panel having stopped tracking the
/// pointer — and the no-op boundary is where *every* drag begins, because a
/// block starts out hovering over itself. This is the same full-strength /
/// dimmed pair `canvas::guides::preview` uses to say *"release here and this
/// does not happen"*, at a comparable ratio (it uses 170 and 60 out of 255).
const CARET_DIMMED: f32 = 0.35;

/// Draw the insertion caret for a drag in flight.
///
/// # Rule 4: this is the cursor, not a mark on content
///
/// A drop caret is in exactly the class the rule permits by name — *"snap
/// indicators, hover highlights, rubber-bands and selection handles are the
/// cursor and are welcome"*. It draws nothing into a page, tints no thumbnail,
/// and disappears the instant the pointer is released. A screenshot of this
/// grid with a drag in flight differs from one of the same document saved and
/// reopened only by where the pointer is, which is the one-line test.
///
/// # The colour is the theme's, never a literal
///
/// `visuals().selection.stroke.color` — the same source the current-page ring
/// and the guide preview take, so a preset that changes the accent changes all
/// three together. `gamma_multiply` rather than a second, paler constant, for
/// the same reason: one colour with a stated relationship beats two colours
/// that have to be kept in step.
fn paint_caret(ui: &egui::Ui, drop: Option<&DropTarget>) {
    let Some(drop) = drop else {
        return;
    };
    let base = ui.visuals().selection.stroke.color;
    let colour = if drop.lands {
        base
    } else {
        base.gamma_multiply(CARET_DIMMED)
    };
    ui.painter().line_segment(
        [drop.caret.left_top(), drop.caret.left_bottom()],
        egui::Stroke::new(CARET_PTS, colour),
    );
    crate::diag::ui_rect_visible(
        // ui-text-exempt: trace region name, never displayed
        "panel-pages-drop-caret",
        drop.caret.expand(CARET_PTS),
        ui.clip_rect(),
    );
}

/// **End a drag** — read the release, raise the reorder, clear the state.
///
/// # Why the release is read from raw input
///
/// `canvas::guides::release`'s discipline, and its reason applies here
/// unchanged: a drag that began on a tile may end anywhere — over the header,
/// outside the panel, past the end of the last row, or after the pointer left
/// the window entirely. A `Response` only reports releases inside the widget
/// that produced it, so a release elsewhere would leave the drag in flight
/// with a caret nobody could get rid of.
///
/// # Why it runs unconditionally
///
/// Because a drag that has started has to be able to end. Gating this on a
/// drop target being resolved would strand every drag released over empty
/// space below the last row — which is exactly where an operator lets go when
/// they mean *"put it at the end"* and miss.
///
/// # A drag that lands nowhere raises NO action
///
/// Not an identity permutation. `reorder_pages` would accept one, record an
/// undo entry, and bump the edit epoch — so a document would be marked dirty
/// and a `Ctrl+Z` would appear to do nothing. `ops::drop_order` refuses it by
/// name and this function drops the refusal, which is the same choice
/// `app::dispatch::pages`' move arm makes for `MoveRefusal::AtTheEdge`.
fn settle_drag(
    ui: &egui::Ui,
    doc: &OpenDoc,
    pages: &mut PagesUi,
    drop: Option<&DropTarget>,
    here: &crate::pagedrag::ActiveDocument,
    actions: &mut Vec<Action>,
) {
    let Some(drag) = crate::pagedrag::current(ui.ctx()) else {
        return;
    };
    // The cursor says the panel is carrying something. Set every frame of the
    // drag rather than once at the start: egui resolves the cursor per frame
    // from whatever asked most recently, so a request made at `drag_started`
    // would be overwritten by the next widget the pointer passed over.
    ui.ctx().set_cursor_icon(egui::CursorIcon::Grabbing);
    if !ui
        .ctx()
        .input(|i| i.pointer.button_released(egui::PointerButton::Primary))
    {
        return;
    }
    // ★ Ends the drag AND clears the landing, in one call, which is what stops
    // a landing sentence outliving the gesture that produced it by a frame.
    // See `pagedrag::end`.
    crate::pagedrag::end(ui.ctx());
    let _ = &pages;

    let page_count = doc.pages.len();
    let Some(target) = drop else {
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed
            "pages-drag-release gap=none reordered=0".to_owned()
        });
        return;
    };

    // ★★ **The branch that makes this two features.**
    //
    // Released in the document the pages came from, this is the reorder it has
    // always been — one `reorder_pages`, one undo entry, nothing copied.
    //
    // Released anywhere else, it is a **copy into this document**, and the
    // source is left exactly as it was. `crate::app::actions::crossdoc` §2
    // carries the reason it is a copy rather than a move, which is that a move
    // would be two commands on two undo stacks and no single Ctrl+Z could
    // reverse it.
    if drag.source_slot != here.slot {
        let position = crate::pagedrag::insert_position(target.gap, page_count);
        // ★ Sampled HERE, at the release, not at the press. Windows reads the
        // drag modifiers at the drop — which is why Explorer's cursor badge
        // changes under your hand as you press and release the key mid-drag —
        // and it is what lets an operator start a drag, read the caption, and
        // change their mind without letting go.
        let take = crate::pagedrag::wants_move(ui.ctx());
        crate::diag::trace(|| {
            format!(
                // ui-text-exempt: diagnostic trace, never displayed
                "pages-drag-release from-slot={} gap={} moving={} copied=1 take={}",
                drag.source_slot,
                target.gap,
                drag.pages.len(),
                u8::from(take),
            )
        });
        actions.push(Action::InsertPagesFromOpenDocument {
            source_slot: drag.source_slot,
            pages: drag.pages,
            position,
            take,
        });
        return;
    }

    match ops::drop_order(&drag.pages, page_count, target.gap) {
        Ok(order) => {
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed
                format!(
                    "pages-drag-release gap={} moving={} reordered=1",
                    target.gap,
                    drag.pages.len()
                )
            });
            actions.push(Action::Page(PageAction::ReorderPages { order }));
        }
        Err(refusal) => {
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed
                format!(
                    "pages-drag-release gap={} moving={} reordered=0 refusal={refusal:?}",
                    target.gap,
                    drag.pages.len()
                )
            });
        }
    }
}

/// Lay the grid out row by row, drawing only the rows that are on screen.
///
/// # Why rows are laid out by hand rather than with `horizontal_wrapped`
///
/// Two reasons, and the second is the load-bearing one.
///
/// A wrapped layout decides where the break falls from the widths it is
/// handed, so a grid of pages with **different sheet sizes** — which is what
/// a real drawing set is — wraps at a different column count on different
/// rows. The eye reads that as a fault in the panel.
///
/// And a wrapped layout gives no seam at which to ask *"is this row on
/// screen?"*. Culling is the whole reason a 900-page document is affordable
/// here: every row is *allocated* (so the scroll bar is honest and nothing
/// jumps as pictures arrive) but only a visible row is painted, interacted
/// with, or considered for rendering.
#[allow(
    clippy::too_many_arguments,
    reason = "each argument is a distinct output the body collects during one \
              layout pass — navigation, menu tokens and the visible set are \
              three different answers, and bundling them into a struct would \
              name a type whose only purpose is to be destructured immediately" // ui-text-exempt: clippy lint justification, never displayed
)]
fn grid_rows(
    ui: &mut egui::Ui,
    doc: &OpenDoc,
    pages: &mut PagesUi,
    current: usize,
    host: Option<&MenuHost<'_>>,
    here: &crate::pagedrag::ActiveDocument,
    visible: &mut Vec<usize>,
    go: &mut Option<usize>,
    tokens: &mut Vec<HandlerToken>,
    drop: &mut Option<DropTarget>,
) {
    let spacing = ui.spacing().item_spacing.x;
    let full_width = ui.available_width();
    let columns = columns_for(full_width, spacing);
    let tile_width = tile_width_for(full_width, spacing, columns);

    let mut first = 0usize;
    while first < doc.pages.len() {
        let last = (first + columns).min(doc.pages.len());
        let row = first..last;

        // The row's height is the tallest sheet in it, so a landscape A1
        // beside a portrait A4 sit on one baseline instead of stepping.
        let thumb_height = row
            .clone()
            .filter_map(|i| doc.pages.get(i))
            .map(|p| tile_height_for(p, tile_width))
            .fold(1.0f32, f32::max);
        let row_height = thumb_height + CAPTION_HEIGHT_PTS;

        let (row_rect, _) =
            ui.allocate_exact_size(egui::vec2(full_width, row_height), egui::Sense::hover());
        if ui.is_rect_visible(row_rect) {
            for (column, page_index) in row.clone().enumerate() {
                let Some(page) = doc.pages.get(page_index) else {
                    continue;
                };
                visible.push(page_index);
                let height = tile_height_for(page, tile_width);
                let origin = egui::pos2(
                    row_rect.left() + column as f32 * (tile_width + spacing),
                    // Bottom-aligned within the row, so the captions line up
                    // and the sheets stand on a common baseline.
                    row_rect.top() + (thumb_height - height),
                );
                let rect = egui::Rect::from_min_size(origin, egui::vec2(tile_width, height));
                tile(
                    ui, doc, pages, page_index, current, rect, host, here, go, tokens, drop,
                );
            }
        }
        first = last;
    }
}

/// Draw one tile and read whatever the operator did to it.
#[allow(
    clippy::too_many_arguments,
    reason = "same as `grid_rows` — three independent outputs plus the four \
              inputs a tile is a function of" // ui-text-exempt: clippy lint justification, never displayed
)]
fn tile(
    ui: &mut egui::Ui,
    doc: &OpenDoc,
    pages: &mut PagesUi,
    page_index: usize,
    current: usize,
    rect: egui::Rect,
    host: Option<&MenuHost<'_>>,
    here: &crate::pagedrag::ActiveDocument,
    go: &mut Option<usize>,
    tokens: &mut Vec<HandlerToken>,
    drop: &mut Option<DropTarget>,
) {
    let id = ui.id().with(("pages-tile", page_index));
    // ★ `click_and_drag`, not `click`. The tile was click-only for the whole
    // life of this panel, which is why reordering was two ribbon buttons that
    // move one place at a time — the gesture every operator tries first was
    // not sensed at all.
    let response = ui.interact(rect, id, egui::Sense::click_and_drag());
    let visuals = ui.visuals().clone();
    let painter = ui.painter();

    // The selection mat, painted first so everything else sits on top of it.
    if pages.selection.contains(page_index) {
        painter.rect_filled(
            rect.expand(SELECTION_MAT_PTS),
            2.0,
            visuals.selection.bg_fill,
        );
    }
    // The sheet itself: paper, then a hairline, then either the picture or
    // the words that say why there is not one.
    painter.rect_filled(rect, 2.0, visuals.extreme_bg_color);
    painter.rect_stroke(
        rect,
        2.0,
        visuals.widgets.noninteractive.bg_stroke,
        egui::StrokeKind::Inside,
    );

    match pages.cache.state(page_index) {
        TileState::Ready => {
            if let Some(texture) = pages.cache.texture(page_index) {
                egui::Image::from_texture(texture)
                    .fit_to_exact_size(rect.size())
                    .paint_at(ui, rect);
            }
        }
        state => {
            // ★ Words, never a blank rectangle. See this module's header and
            // `crate::text::pages`': paper-coloured emptiness is a picture of
            // an empty page, and an empty page is a thing a PDF can contain.
            let words = match state {
                // ui-text-exempt: a panic message for an arm the match above
                // already took; never rendered.
                TileState::Ready => unreachable!("handled above"),
                TileState::NotDrawnYet => t::thumbnail_not_drawn_yet(),
                TileState::PreviewsOff => t::thumbnail_previews_off(),
                TileState::Abandoned => t::thumbnail_abandoned(),
                TileState::Failed => t::thumbnail_failed(),
            };
            ui.painter().text(
                rect.center(),
                egui::Align2::CENTER_CENTER,
                words,
                egui::TextStyle::Small.resolve(ui.style()),
                visuals.weak_text_color(),
            );
        }
    }

    // ★ Every visible tile publishes its rectangle, so a driven check can aim
    // at a page rather than at a guess.
    //
    // Added 2026-08-18 with the drag-to-reorder gesture, and it closes a gap
    // this panel had for its whole life: `panel-pages` and `panel-pages-grid`
    // named the container and `panel-pages-current-tile` named exactly one
    // tile, so **no check anywhere read this panel**. A drag needs two tiles —
    // one to lift and one to land beside — and neither could be addressed.
    //
    // `ui_rect_visible` rather than `ui_rect`: this is inside a `ScrollArea`,
    // and a tile scrolled out of view must not keep publishing a rectangle a
    // check would then click on. `diag.rs`'s own header records the
    // false-failure that rule exists for.
    crate::diag::ui_rect_visible(
        // ui-text-exempt: trace region name, never displayed
        &format!("{TILE_REGION_PREFIX}{page_index}"),
        rect,
        ui.clip_rect(),
    );

    // The current page's ring, outside the sheet so it cannot hide a hairline
    // of the picture. "Which page am I on" must be answerable at a glance,
    // and it must be answerable on an undrawn tile too — which is why it is
    // drawn after the words rather than only over a picture.
    if page_index == current {
        ui.painter().rect_stroke(
            rect,
            2.0,
            egui::Stroke::new(CURRENT_RING_PTS, visuals.selection.bg_fill),
            egui::StrokeKind::Outside,
        );
        crate::diag::ui_rect("panel-pages-current-tile", rect);
    }

    // The caption. Always drawn, for every state — the number and the sheet's
    // shape both come from the page tree rather than from rendering, so a
    // tile is never a row that says nothing.
    ui.painter().text(
        egui::pos2(rect.center().x, rect.bottom() + 2.0),
        egui::Align2::CENTER_TOP,
        t::page_number(page_index),
        egui::TextStyle::Small.resolve(ui.style()),
        if page_index == current {
            visuals.strong_text_color()
        } else {
            visuals.text_color()
        },
    );

    let (width_pts, height_pts) = doc
        .pages
        .get(page_index)
        .map_or((0.0, 0.0), crate::viewer::page_extent_pts);
    let response = response.on_hover_text(t::page_tile_tooltip(
        page_index,
        width_pts / PTS_PER_MM,
        height_pts / PTS_PER_MM,
    ));

    // ★ A drag begins here, and it begins by settling the OPERAND SET.
    //
    // The same rule the context menu already follows
    // (`PageSelection::right_click`): a gesture's verbs must apply to the tile
    // the operator pointed at. Dragging a page that is not in the selection
    // means *"move this one"*, not *"move those other ones I picked earlier"* —
    // and a drag that silently carried a selection made three minutes ago on a
    // different part of the document would be the most surprising thing this
    // panel could do.
    // `drag_started_by(Primary)`, not `drag_started()`. egui's plain predicate
    // is true for the middle button as well, and a right-press that wandered a
    // few pixels before releasing would start a reorder the operator meant as
    // a context menu — the gesture they use to reach the very commands this
    // one replaces.
    if response.drag_started_by(egui::PointerButton::Primary) {
        pages.selection.right_click(page_index);
        // ★★ The operand set is CAPTURED HERE, not resolved at release —
        // reversing what this panel's own reorder drag used to do, and
        // `crate::pagedrag`'s header carries the argument.
        //
        // The one-line form: a drag that crosses into another document
        // springs a tab open on the way, and activating a tab clears this
        // panel's selection. Resolving at release would resolve against the
        // *target's* selection, or against nothing at all.
        let carrying = ops::operands(pages.selection.pages(), current, doc.pages.len());
        crate::pagedrag::begin(
            ui.ctx(),
            crate::pagedrag::PageDrag {
                source_slot: here.slot,
                origin: page_index,
                source_label: here.label.clone(),
                pages: carrying,
            },
        );
    }

    // While a drag is in flight, every tile the pointer is over offers itself
    // as a landing boundary. The nearer vertical edge wins, which is what makes
    // a caret feel like it snaps to a gap rather than to a tile.
    // ★★ **TWO drags reach this block, and they resolve the same geometry.**
    //
    // A page drag from a thumbnail (possibly in another document), and — added
    // 2026-08-31 for `OPERATOR_REQUESTS.md` O67 — **a FILE dragged in from
    // Explorer**. They differ in where the pointer comes from and in whether a
    // gap can be a no-op; they do not differ in what a gap *is*, so they share
    // this code rather than growing a second copy of the nearer-edge rule.
    //
    // ★ The file drag's pointer comes from `crate::app::filedrag` because the
    // toolkit does not have one: `winit` discards the OLE drop point and no
    // mouse-move message arrives during a drag, so `pointer_latest_pos` is
    // stale from before the drag began. Its header carries the citation.
    let dragging: Option<(egui::Pos2, Option<crate::pagedrag::PageDrag>)> =
        match crate::pagedrag::current(ui.ctx()) {
            Some(drag) => ui.ctx().pointer_latest_pos().map(|p| (p, Some(drag))),
            // A file drag: still hovering, or landed this frame and not yet
            // claimed. Both use the same point, which is what stops the caret
            // an operator watched from being a promise the drop does not keep.
            None => (crate::app::filedrag::hovering(ui.ctx())
                || crate::app::filedrag::landed(ui.ctx()).is_some())
            .then(|| crate::app::filedrag::aim(ui.ctx()))
            .flatten()
            .map(|p| (p, None)),
        };

    if let Some((pointer, drag)) = dragging
        && rect.expand(SELECTION_MAT_PTS).contains(pointer)
    {
        let after = pointer.x > rect.center().x;
        let gap = if after { page_index + 1 } else { page_index };
        // ★ ONE spelling of the nearer-edge rule above, and one of the
        // no-op rule here. The file drag has no operands of its own in this
        // document, so every gap accepts it.
        let lands = drag.is_none_or(|d| {
            d.source_slot != here.slot
                || !ops::drag_is_a_no_op(&d.pages.iter().copied().collect(), gap)
        });
        let caret_x = if after { rect.right() } else { rect.left() };
        *drop = Some(DropTarget {
            gap,
            // Half the inter-tile spacing beyond the sheet on each side, so the
            // line sits IN the gap rather than on the sheet's own edge, where
            // it would read as a border the page had grown.
            caret: egui::Rect::from_min_max(
                egui::pos2(caret_x, rect.top() - SELECTION_MAT_PTS),
                egui::pos2(caret_x, rect.bottom() + SELECTION_MAT_PTS),
            ),
            // ★ Two different questions, because a drop from ELSEWHERE is a
            // different verb from a drop from here — resolved above, where the
            // pointer is, because the answer depends on which drag this is.
            //
            // Within one document the drag is a reorder, and a reorder onto a
            // boundary inside its own operand run moves nothing —
            // `ops::drag_is_a_no_op` is the one place that rule lives.
            //
            // From another document, or from Explorer, it is a copy, and a
            // copy always lands: every gap is a legal place to put a sheet that
            // is not there yet, including the boundaries either side of a page
            // that happens to share an index with one of the operands. Asking
            // the no-op question of a cross-document drag would dim the caret
            // over exactly the pages the operator was aiming between.
            //
            // ★ The set is built from the drag's own captured operands rather
            // than from `pages.selection`. They agree today — a drag starts by
            // selecting the tile it began on — and they would stop agreeing the
            // moment a drag could cross a document, because activating another
            // tab clears the selection and the caret would go dim over every
            // gap.
            lands,
        });
    }

    if response.clicked() {
        let modifiers = ui.input(|i| i.modifiers);
        // `command` rather than `ctrl`: it is Ctrl everywhere and Cmd on
        // macOS, which is what an operator's hand expects on the machine
        // they are using.
        let outcome = pages
            .selection
            .click(page_index, modifiers.command, modifiers.shift);
        if outcome.navigate {
            *go = Some(page_index);
        }
        crate::diag::trace(|| {
            format!(
                "pages-tile-click page={} ctrl={} shift={} navigate={} selected={}",
                page_index + 1,
                u8::from(modifiers.command),
                u8::from(modifiers.shift),
                u8::from(outcome.navigate),
                pages.selection.len(),
            )
        });
    }
    // The operand rule, before the attach: a menu's verbs must apply to the
    // tile the operator pointed at. See `select::PageSelection::right_click`.
    if response.secondary_clicked() && pages.selection.right_click(page_index) {
        crate::diag::trace(|| {
            format!(
                "pages-tile-right-click page={} selected={}",
                page_index + 1,
                pages.selection.len(),
            )
        });
    }
    if let Some(host) = host {
        tokens.extend(host.attach(&response, PAGES_ROW));
    }
}

/// How many columns fit in `available` points.
///
/// At least one, always: a dock dragged narrower than a single tile must show
/// a column of squeezed thumbnails rather than none at all, because zero
/// columns is a panel that has silently emptied itself.
#[must_use]
pub fn columns_for(available: f32, spacing: f32) -> usize {
    if !available.is_finite() || available <= 0.0 {
        return 1;
    }
    let per_column = MIN_TILE_WIDTH_PTS + spacing;
    #[allow(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "the value is clamped to at least 1.0 and bounded above by a \
                  window width in points, so it cannot exceed a few hundred" // ui-text-exempt: clippy lint justification, never displayed
    )]
    let columns = (((available + spacing) / per_column).floor() as usize).max(1);
    columns
}

/// How wide each tile is, once `columns` of them and their gaps share
/// `available`.
#[must_use]
pub fn tile_width_for(available: f32, spacing: f32, columns: usize) -> f32 {
    #[allow(
        clippy::cast_precision_loss,
        reason = "a column count is at most a few hundred and is exact in f32" // ui-text-exempt: clippy lint justification, never displayed
    )]
    let n = columns as f32;
    let gaps = spacing * (n - 1.0);
    // A floor of one point rather than zero: a zero-width rect makes egui lay
    // nothing out and the panel goes blank, which reads as a crash.
    ((available - gaps) / n).max(1.0)
}

/// How tall a tile is, from its page's own aspect ratio.
///
/// The shape is free — it comes from the page tree, not from rendering — so
/// every tile is the right shape from the first frame, before any picture
/// exists. That is what makes the scroll bar honest while the grid fills:
/// each row occupies its final height whether or not its pictures have
/// arrived, so nothing jumps.
#[must_use]
pub fn tile_height_for(page: &pdfcer_core::page_tree::Page, tile_width: f32) -> f32 {
    let (width, height) = crate::viewer::page_extent_pts(page);
    if width > 0.0 && height > 0.0 {
        (tile_width * height / width).max(1.0)
    } else {
        // A degenerate `/CropBox`. Square is a visibly odd shape rather than
        // a plausible-looking wrong one, which is the right failure for a
        // page whose geometry the file did not state.
        tile_width
    }
}

/// The page at the middle of what is on screen — the centre eviction
/// measures distance from.
///
/// Falls back to the current page when nothing is visible, which happens on
/// the frame a panel is first mounted and on any frame the dock gives it no
/// height.
#[must_use]
pub fn viewport_centre(visible: &[usize], current: usize) -> usize {
    if visible.is_empty() {
        return current;
    }
    visible[visible.len() / 2]
}

/// The Pages panel's own state, between frames.
///
/// Held by [`crate::panels::PanelsState`], which owns every panel's
/// inter-frame state — see its header for why that is there rather than on
/// `PdfcerApp`.
#[derive(Default)]
pub struct PagesUi {
    /// Which pages the operator has picked.
    pub selection: select::PageSelection,
    /// The pictures, and the policy that fills them.
    pub cache: thumbnails::ThumbnailCache,
}

impl std::fmt::Debug for PagesUi {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PagesUi")
            .field("selection", &self.selection.len())
            .field("cache", &self.cache)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The dock's default navigator width fits **two** columns, which is the
    /// number `crate::app::modes::defaults::NAVIGATOR_WIDTH`'s own doc comment claims.
    ///
    /// Asserted here rather than trusted, because the two constants live in
    /// different modules and the claim is only true for a particular tile
    /// width: *"a thumbnail rail one column wide wastes the dock, and three
    /// columns makes each too small to recognise a drawing by."*
    #[test]
    fn the_default_navigator_width_fits_two_columns() {
        // The dock's 280 pt, less the panel's own margins and the scroll bar
        // this panel reserves 10 pt for.
        assert_eq!(columns_for(250.0, 8.0), 2);
        assert_eq!(columns_for(120.0, 8.0), 1, "a narrow dock is one column");
        assert!(
            columns_for(500.0, 8.0) >= 4,
            "a wide dock must use the width it was given"
        );
    }

    /// A dock dragged to nothing still asks for one column.
    ///
    /// Zero columns divides by zero in [`tile_width_for`] and lays out
    /// nothing, which reads as a panel that crashed rather than one that ran
    /// out of room.
    #[test]
    fn a_degenerate_width_still_produces_a_usable_grid() {
        for available in [0.0, -5.0, f32::NAN, 1.0] {
            let columns = columns_for(available, 8.0);
            assert!(columns >= 1, "{available} gave {columns} columns");
            let width = tile_width_for(available, 8.0, columns);
            assert!(width.is_finite() && width > 0.0, "{available} gave {width}");
        }
    }

    /// The columns and their gaps fill the width they were given, and never
    /// exceed it.
    ///
    /// Exceeding it is the defect `crate::panels::content_width`'s docs
    /// describe from the other side: a row wider than the viewport is
    /// silently squeezed and the overflow is clipped with nothing to say so.
    #[test]
    fn the_columns_and_their_gaps_exactly_fill_the_width() {
        for available in [250.0f32, 300.0, 512.0, 1000.0] {
            let spacing = 8.0;
            let columns = columns_for(available, spacing);
            let width = tile_width_for(available, spacing, columns);
            #[allow(
                clippy::cast_precision_loss,
                reason = "a small column count is exact in f32" // ui-text-exempt: clippy lint justification, never displayed
            )]
            let used = width * columns as f32 + spacing * (columns as f32 - 1.0);
            assert!(
                (used - available).abs() < 0.01,
                "{columns} columns of {width} used {used} of {available}"
            );
        }
    }

    /// **★ A tile is the shape of its page before any picture exists.**
    ///
    /// The property that keeps the scroll bar honest while the grid fills:
    /// the aspect ratio comes from the page tree, which is free, so each row
    /// occupies its final height from the first frame and nothing jumps as
    /// pictures arrive.
    #[test]
    fn a_tile_takes_its_shape_from_the_page_tree() {
        use crate::panels::objects::test_support::engine_fixture;
        let path = engine_fixture("pageops/four-pages.pdf");
        let doc = pdfcer_core::document::Document::load(&path).expect("the fixture loads");
        let pages = pdfcer_core::page_tree::pages(&doc).expect("a page tree");
        assert!(!pages.is_empty());
        for page in &pages {
            let (w, h) = crate::viewer::page_extent_pts(page);
            let height = tile_height_for(page, 120.0);
            assert!(height.is_finite() && height > 0.0);
            assert!(
                (height - 120.0 * h / w).abs() < 0.01,
                "a tile must be the page's own shape, not a fixed box"
            );
        }
    }

    /// The eviction centre is the middle of what is on screen, and falls back
    /// to the current page when nothing is.
    #[test]
    fn the_viewport_centre_is_the_middle_of_what_is_on_screen() {
        assert_eq!(viewport_centre(&[10, 11, 12, 13, 14], 0), 12);
        assert_eq!(
            viewport_centre(&[], 7),
            7,
            "a panel with no height must still name a centre"
        );
    }

    /// **★ The menu context this panel attaches is spelled once, and is
    /// now defined.**
    ///
    /// `crate::shell::menus`' own rule: *"a context id is used in exactly two
    /// places that must agree… a typo in either produces silence rather than
    /// an error."* This pins the spelling on both sides.
    ///
    /// It used to assert the opposite — that the context was **not** yet
    /// defined — so that the day it was, this test would fail and be updated
    /// in the same commit that made the right-click work. That day is this
    /// commit, and the assertion is inverted rather than deleted: the pairing
    /// it guards is the same pairing either way, and a menu attached to a
    /// context nobody defines opens nothing at all, silently.
    #[test]
    fn the_page_tile_menu_context_is_named_and_defined() {
        assert_eq!(PAGES_ROW, "pages.row");
        assert_eq!(
            PAGES_ROW,
            crate::shell::menus::PAGES_ROW,
            "the two spellings of this context id have drifted apart, which detaches every tile's menu with no error anywhere"
        );
        let menus = crate::shell::menus::built_in();
        assert!(
            menus.get(PAGES_ROW).is_some(),
            "`{PAGES_ROW}` is attached by every tile and defined by nothing, so the right-click opens nothing"
        );
        assert!(
            crate::shell::menus::CONTEXTS.contains(&PAGES_ROW),
            "the context list and the menu document disagree"
        );
    }

    /// **★ Every page verb this panel means to offer is a registered
    /// command.**
    ///
    /// The rule `crate::shell::menus`' header states — *only real commands* —
    /// checked from the panel's side before the menu exists, so the menu can
    /// be written from this list rather than from memory. A verb that failed
    /// here would be one to leave out, not one to add and grey.
    #[test]
    fn every_page_verb_the_menu_would_offer_is_registered() {
        use egui_shell::CommandRegistry;
        let mut registry = CommandRegistry::new();
        crate::shell::commands::register(&mut registry);
        for id in [
            "pages.rotate_left",
            "pages.rotate_right",
            "pages.delete",
            "pages.extract",
            "pages.move_up",
            "pages.move_down",
        ] {
            assert!(
                registry.get(id).is_some(),
                "`{id}` is not registered, so a menu row for it would render \
                 nothing at all"
            );
        }
        // …and the one the menu deliberately leaves out is absent from that
        // list rather than forgotten: `pages.merge_into` is a document-level
        // verb that acts on the whole file rather than on the sheets pointed
        // at. It stays on the ribbon's Pages tab.
        //
        // ★★★ `pages.split` was the second id here until 2026-08-31 and is
        // now UNREGISTERED — `OPERATOR_REQUESTS.md` O68. It was drawn, enabled
        // and had no dispatch arm; R9 says a capability that is not built
        // renders nothing, and its blocker (no boundary chooser, no
        // destination directory, no name template) is real and unchanged. It
        // comes back with `tools.split_files` when the chooser exists.
        //
        // ★ A single assertion rather than a one-element loop — clippy
        // refuses the loop and is right to. It becomes a loop again with two
        // members and a reason when `pages.split` comes back.
        let id = "pages.merge_into";
        assert!(
            registry.get(id).is_some(),
            "`{id}` is expected to exist on the ribbon even though the \
             tile menu does not offer it"
        );
    }

    /// **★ The measurement behind this panel's policy, on the real
    /// documents.**
    ///
    /// Ignored by default: it rasterizes whole pages and takes seconds, which
    /// is not a unit test's job. It is kept because the numbers in
    /// [`thumbnails`]' header are the entire argument for the design, and a
    /// claim about performance with no way to re-run it is a claim that
    /// quietly stops being true.
    ///
    /// ```text
    /// cargo test -p pdfcer-gui -- --ignored --nocapture thumbnail_cost
    /// ```
    #[test]
    #[ignore = "measurement: rasterizes real pages, takes seconds"]
    fn thumbnail_cost_on_the_benchmark_documents() {
        use std::path::PathBuf;
        use std::time::Instant;

        let candidates = [
            PathBuf::from(r"D:\Dev\temp\pdfcer\ncored-benchmark-cad-drawing.pdf"),
            PathBuf::from(r"D:\Dev\temp\pdfcer\SW41177.pdf"),
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/a1-titleblock.pdf"),
            crate::panels::objects::test_support::engine_fixture("pageops/four-pages.pdf"),
        ];

        for path in candidates {
            if !path.exists() {
                println!("skipped (absent): {}", path.display());
                continue;
            }
            let document = pdfcer_core::document::Document::load(&path).expect("loads");
            let pages = pdfcer_core::page_tree::pages(&document).expect("a page tree");
            let session = pdfcer_core::edit::EditSession::new(document);
            let view = session.view();
            let mut options = pdfcer_render::RenderOptions::default();
            options.annotations = true;

            for (index, page) in pages.iter().enumerate().take(4) {
                let scale = thumbnails::raster_scale_for(page, 2.0);
                let started = Instant::now();
                let outcome = pdfcer_render::render_page_with_view(&view, page, scale, &options);
                let ms = started.elapsed().as_millis();
                let size = outcome
                    .as_ref()
                    .map(|r| format!("{}x{}", r.pixmap.width(), r.pixmap.height()))
                    .unwrap_or_else(|e| e.to_string());
                println!(
                    "{}  page {} scale {scale:.3}  {ms} ms  {size}",
                    path.file_name().unwrap_or_default().to_string_lossy(),
                    index + 1,
                );
            }
        }
    }
}
