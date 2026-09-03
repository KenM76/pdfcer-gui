//! # `panels::objects` — everything drawn on the current page
//!
//! Salvaged from three places in the old shell: the panel body from
//! `main.rs`, the decomposition from `object_provider.rs` (→
//! [`provider`]) and the per-object description from `object_summary.rs`
//! (→ [`summary`]). `SALVAGE.md` calls the Objects panel *"the single
//! strongest thing pdfcer has"*.
//!
//! ## What it is FOR — the operator's own words
//!
//! > *"I'd like to have a layer tree there for the document that I can also
//! > click on to select objects. at least that way we can troubleshoot
//! > better what I am clicking on in the GUI area."*
//!
//! So its first job is answering **"what am I looking at"**, and every
//! design choice below is subordinate to that. It is a diagnostic instrument
//! first and a navigation aid second. The clause about *selecting* is the
//! half that arrives at S4 — see "What is not here yet".
//!
//! ## Front-most FIRST — justified, not merely conventional
//!
//! The list is drawn in **reverse paint order**: the last-painted (topmost)
//! object is the first row. Two reasons, in priority order:
//!
//! 1. **It matches what a click does.** Hit-testing resolves overlapping
//!    candidates topmost-first, so the object an operator is most likely
//!    confused about is at the top of the list, not scrolled to the bottom
//!    of a thousand rows. For a panel whose whole purpose is "what did I
//!    just click", any other order buries the answer.
//! 2. It is the prevailing convention for layer/object panels (top of list =
//!    top of z-order), cited strictly as a metaphor-level convention.
//!
//! **The row's visible `#n` is the PAINT-ORDER index, not the display
//! position.** That is deliberate: `#n` is the number `pdfcer
//! object-list` prints as `index=`, and the number
//! `object-move` / `object-delete` / `node-move` take as an operand. A
//! display-position number would look equally authoritative and address a
//! different object.
//!
//! ## The nesting is the LEVEL LADDER, and nothing else
//!
//! **object → part → point**, which is exactly the ladder the canvas walks
//! when it lands. `PathObject` owns `subpaths` and each subpath owns its
//! segments; a text object owns its runs. Every level here is structure
//! `pdfcer-core` already models and addresses with the same indices, so the
//! tree and the canvas will agree **by construction** rather than by care.
//!
//! [`provider::PartKind`] is the one place that dispatch is decided —
//! a path's part is a subpath, a text object's is a run, an image has none —
//! so the row builder never matches on `VectorObject` itself. A text object
//! caps at two rungs by construction: a run has no anchors, so
//! `subpath_node_points` cannot produce a point for it and no guard is
//! needed anywhere.
//!
//! ### What is deliberately NOT nested here
//!
//! **Marked-content / optional-content grouping.**
//! `pdfcer_core::vector::PageObjects` has **no optional-content-group
//! membership for page content at all** — `VectorObject::{Path,Text,Image}`
//! carry no `/OC` — so there is no such grouping to render, and inventing
//! one would be a lie about the document's structure. That level becomes
//! available only once `decompose_page` tracks `BDC`/`EMC` membership.
//! Deferred, not overlooked.
//!
//! (Do not conflate this with the ce-dimension group OCGs, which are an
//! annotation-layer visibility mechanism and have their own surface.)
//!
//! ## ★ The two defects this rebuild fixes
//!
//! ### 1. Row text no longer clips
//!
//! `SALVAGE.md`'s row for `object_summary.rs` states the requirement:
//! *"Row text must not clip; the old panel truncated with no horizontal
//! scroll."*
//!
//! The old panel drew each row with
//! `ui.add_sized(vec2(ui.available_width(), row_height), …)` inside a
//! **vertical-only** `ScrollArea`. A row wider than the pane was cut off at
//! the pane's edge with no bar on either axis and nothing to say so — and a
//! row here can easily be wide: `#1382  Path · filled (even-odd) and
//! stroked #1A73E8, 0.50 pt wide · 6681 node(s) · zero height` is not a
//! contrived example.
//!
//! Two fixes, together, because either alone leaves a hole:
//!
//! - **`ScrollArea::both`, with the container's own width stated.**
//!   `Ui::allocate_ui*` and `add_sized` CLAMP a requested size to the space
//!   left in the parent, so a wide row is silently squeezed, the area
//!   measures content == viewport, and no bar appears. The fix is
//!   [`super::content_width`] fed to `Ui::set_width`, which *grows*
//!   `max_rect`. The width is the **intrinsic** text width from
//!   [`super::text_width`], never a measurement of a laid-out row —
//!   measuring the laid-out row is what produced the squeezed number in the
//!   first place.
//! - **A tooltip carrying the full row text**, on every row that was
//!   measured wider than the pane. Scrolling sideways is a gesture an
//!   operator has to think of; hovering is one they already do. Attached
//!   only when it is needed, because a tooltip that always repeats the
//!   visible label is noise.
//!
//! And a third guard on top, because the first two do not help a 6,681-row
//! part: [`POINT_ROWS_PER_PART`] caps how many point rows one part
//! contributes, and the cap is **disclosed with both numbers** rather than
//! the list being quietly shortened. A silently truncated list is
//! indistinguishable from a short one.
//!
//! ### 2. Scrollbars are visible
//!
//! egui's default `ScrollStyle` is `floating()` — 2 pt, zero allocated
//! width, fully transparent when the pointer is elsewhere — so a working
//! scroll area is indistinguishable in a screenshot from content clipped at
//! the container edge. `super::scroll_style` fixes it for every panel; the
//! full measurement is in [`super`]'s header.
//!
//! ## Virtualized, never silently truncated
//!
//! A complex drawing decomposes to tens of thousands of objects.
//! `ScrollArea::show_rows` lays out only the rows actually on screen, so the
//! list stays cheap at any size and **no cap is applied to objects** — there
//! is nothing to disclose because nothing is hidden. The one cap that does
//! exist, on points within a part, prints both numbers.
//!
//! The visible rows are materialised into a flat `Vec` once per frame rather
//! than walked recursively, because `show_rows` needs a row **count** and
//! the ability to draw an arbitrary slice — and a recursive tree walk cannot
//! answer "what is row 4,000" without walking to it. With everything
//! collapsed the list is exactly the object count, so the nesting costs
//! nothing until it is used.
//!
//! ## What is not here yet
//!
//! **Selection.** Clicking an object row points the [`super::properties`]
//! panel at it, and that is all it does — no canvas highlight, no
//! multi-select, no Shift+click, no scroll-to-reveal, because there is no
//! selection model to reveal *into*. `super::PanelsState::focus`'s own docs
//! spell out the difference between a panel focus and a selection and state
//! that the field is deleted rather than grown when the real one lands.
//!
//! Part and point rows are therefore **not clickable**. A row that responded
//! to a click by focusing its parent object instead would be a control
//! answering a different question from the one it was asked.
//!
//! ## ★ The row's right-click, and the one command it deliberately does not
//! offer
//!
//! An **object** row carries the `objects.row` context menu
//! ([`crate::shell::menus::OBJECTS_ROW`]). Right-clicking a row **focuses
//! it** first — the panel's equivalent of the canvas's select-first rule, so
//! the menu is about the row the pointer is on rather than about whichever
//! row was last clicked — and then offers `file.properties`, which is the
//! command that puts the focused object's description on screen.
//!
//! Part and point rows carry **no** menu, for the same reason they are not
//! clickable: a menu on a part row would have to be about its parent object,
//! which is answering a question nobody asked.
//!
//! **`format.delete` is not on this menu, and that is a safety decision.**
//! The focus below is *not* the selection — `super::ObjectTreeUi::focus`'s
//! own docs and `super::tests::the_panel_focus_has_not_quietly_become_a_selection`
//! defend the distinction at length — so a Delete offered here would be
//! enabled by `selection.any`, which describes the **canvas** selection, and
//! would remove objects the operator never pointed at. That is precisely the
//! failure that test exists to prevent, arriving through a menu instead of
//! through a predicate. A Delete belongs here on the day the row click
//! becomes a selection gesture and the focus field is deleted, which is the
//! commit its own documentation names.

pub mod provider;
pub mod summary;

use crate::app::actions::Action;
use crate::app::state::OpenDoc;
use crate::panels::{PanelsState, content_width, text_width};
use crate::shell::menus::{self, MenuHost};
use crate::text::panels::objects as t;
use egui_shell::HandlerToken;
use provider::ObjectModelProvider;

/// How many point rows one part may contribute to the tree.
///
/// ★ **This number is a measurement, not a guess.** One path object on a
/// real CAD export holds **6,681 anchors**, and that number is why the point
/// rung is scoped to a part at all
/// ([`provider::ObjectModelProvider::subpath_node_points`] carries the same
/// figure and the same reasoning for the *canvas* pick set).
///
/// Virtualization makes a wall of rows cheap to *draw* and no more useful to
/// *read*, and materialising 6,681 `ObjectTreeRow`s to find one costs a
/// frame on exactly the sheet where the panel matters most. 200 is enough to
/// see the shape of a part's point list and to read off an index for
/// `node-move`.
///
/// The cap is **disclosed with both numbers** by
/// [`crate::text::panels::objects::object_tree_points_capped`]. A quietly
/// shortened list is indistinguishable from a short one — the same defect
/// `bookmarks_truncated` exists to prevent one panel over.
pub const POINT_ROWS_PER_PART: usize = 200;

/// One visible line of the object tree.
///
/// A row's `(object, part, point)` triple **is** an address into the level
/// ladder, so a canvas descent will have an exact row to land on when the
/// two are connected at S4.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ObjectTreeRow {
    /// A page object, at paint-order `index`.
    Object {
        /// Paint-order index — the number every command-line verb takes.
        index: usize,
    },
    /// A part within an expanded object: a path's subpath, or a text
    /// object's run.
    Part {
        /// The owning object's paint-order index.
        object: usize,
        /// The part's index within that object.
        part: usize,
    },
    /// An anchor within an expanded part.
    Point {
        /// The owning object's paint-order index.
        object: usize,
        /// The owning part's index.
        part: usize,
        /// The anchor's **object-scoped** index — it keeps counting across a
        /// part boundary, because that is the number `node-move --node N`
        /// takes.
        node: usize,
    },
    /// The disclosure that a part's point list was capped.
    ///
    /// A row rather than a label under the list, because the list is
    /// virtualized: a label outside `show_rows` would sit at a fixed place
    /// on screen while the part it describes scrolled away.
    PointsCapped {
        /// How many point rows were emitted.
        shown: usize,
        /// How many the part actually has.
        total: usize,
    },
}

/// Draw the Objects panel, and report any command an operator invoked from
/// an object row's context menu.
///
/// The returned `egui_shell::HandlerToken`s are **intent**: this function
/// executes nothing, exactly as it mutates nothing. See
/// [`crate::panels::Panel::show`] for why a panel hands tokens on rather
/// than translating them.
#[must_use]
pub fn body(
    ui: &mut egui::Ui,
    doc: &OpenDoc,
    state: &mut PanelsState,
    host: Option<&MenuHost<'_>>,
    actions: &mut Vec<Action>,
) -> Vec<HandlerToken> {
    let page_index = doc.view.page_index;
    // The document's own decomposition, not a second one built here — see
    // `crate::app::state::OpenDoc::page_objects`. The `Ref` is held for the
    // whole body, which is exactly as long as the rows are being drawn from
    // it; nothing in this frame can take `&mut OpenDoc` meanwhile, because a
    // panel is only ever handed a shared reference.
    let Some(provider) = doc.page_objects() else {
        // `None` means the page's content could not be decoded. Stated in
        // words rather than shown as an empty list, because a failure state
        // must never be visually indistinguishable from a success state that
        // happens to have no content.
        ui.label(t::objects_dock_decompose_failed_hint());
        return Vec::new();
    };
    let provider = &*provider;
    // The operator's own state, from the panels' side. These used to come
    // back from one call together, because the provider and the tree had to
    // be two disjoint borrows of one struct; they are now two different
    // objects and the pairing is gone.
    let tree = state.tree_mut();
    let objects = &provider.page_objects().objects;
    if objects.is_empty() {
        ui.label(t::objects_dock_empty_page_hint());
        return Vec::new();
    }

    // `object_kind` rather than `describe_object(..).kind`: the full
    // description counts every anchor of every subpath, and on the measured
    // CAD sheet one object alone holds 6,681 of them. A header line must not
    // cost more than the list beneath it.
    let census = summary::census(objects.iter().map(summary::object_kind));
    ui.label(t::objects_dock_intro());
    ui.label(t::objects_dock_summary(census));
    ui.separator();

    let rows = build_rows(
        provider,
        &tree.objects_expanded,
        &tree.parts_expanded,
        POINT_ROWS_PER_PART,
    );
    let total_rows = rows.len();
    let object_count = objects.len();
    crate::diag::trace(|| {
        format!("objects-panel page={page_index} objects={object_count} rows={total_rows}")
    });

    // Three intents, captured and applied after the closure so a click
    // cannot mutate state the rows drawn after it are still reading. The
    // same defer-then-apply the rest of this shell uses, at its smallest
    // scale — and here it is also what keeps `tree` immutably borrowed for
    // the whole of the draw.
    let mut toggle_object: Option<usize> = None;
    let mut toggle_part: Option<(usize, usize)> = None;
    let mut focus: Option<usize> = None;
    // The commands invoked from a row's context menu, collected across the
    // visible rows and returned. Not applied here — a panel raises intent.
    let mut tokens: Vec<HandlerToken> = Vec::new();
    // ★★★ **The row highlight is READ FROM THE SELECTION**, since 2026-08-26 —
    // it used to be `tree.focus()`, a panel-local variable the canvas neither
    // wrote nor read.
    //
    // That is the whole of the change. This panel no longer *holds* an opinion
    // about which object is being worked on; it renders the one opinion there
    // is. A click on the canvas therefore highlights the matching row, which it
    // never did before, and a click on a row selects on the canvas, which it
    // never did either — the two are the same fact seen from two ends.
    //
    // ★ Object-scoped, and deliberately: `object_indices_on` answers about page
    // CONTENT, which is what this tree lists. An annotation or a ce dimension
    // selected on the canvas leaves every row here unhighlighted, correctly —
    // none of these rows is that thing.
    let focused = doc.selection.object_indices_on(page_index).first().copied();
    let expanded_objects = &tree.objects_expanded;
    let expanded_parts = &tree.parts_expanded;

    // A selectable label's height floor is `interact_size.y`; declaring the
    // same value as the row height is what keeps `show_rows`' virtual-scroll
    // arithmetic in step with what is actually painted.
    let row_height = ui.spacing().interact_size.y;
    let viewport = ui.available_width();

    egui::ScrollArea::both()
        .id_salt("objects-tree-rows")
        .show_rows(ui, row_height, total_rows, |ui, range| {
            // Label every row in the visible range FIRST, so the container's
            // width can be stated before anything is laid out. Doing it
            // after would measure clamped rows, which is the defect this
            // exists to fix.
            let labelled: Vec<(ObjectTreeRow, String, f32)> = range
                .filter_map(|i| rows.get(i).copied())
                .map(|row| {
                    let label = row_label(provider, row);
                    let width = text_width(ui, &label) + row_indent(row) + EXPANDER_SPACE;
                    (row, label, width)
                })
                .collect();
            // The fix: state the content's own width, so a wide row makes
            // the area scroll rather than being squeezed into the pane and
            // clipped. `set_width` GROWS `max_rect`; `auto_shrink` and
            // `max_width` do not help, because they bound the viewport,
            // which was already right.
            let width = content_width(labelled.iter().map(|(_, _, w)| *w), viewport);
            ui.set_width(width);

            for (row, label, measured) in labelled {
                // The tooltip is the recovery path for an operator who does
                // not think to scroll sideways. Attached only when the row
                // really is wider than the pane, because a tooltip that
                // repeats a fully visible label is noise.
                let overflows = measured > viewport;
                match row {
                    ObjectTreeRow::Object { index } => {
                        ui.horizontal(|ui| {
                            if provider.part_count(index) > 0 {
                                let open = expanded_objects.contains(&index);
                                if expander(ui, open).clicked() {
                                    toggle_object = Some(index);
                                }
                            } else {
                                // R83: the gap is held so labels stay
                                // aligned, but no dead control is drawn — a
                                // leaf has nothing to expand and should not
                                // offer to.
                                ui.add_space(t::OBJECT_TREE_EXPANDER_WIDTH);
                            }
                            let mut resp = ui
                                .selectable_label(focused == Some(index), label.as_str())
                                .on_hover_text(t::objects_dock_row_tooltip());
                            if overflows {
                                resp = resp.on_hover_text(label.as_str());
                            }
                            if resp.clicked() {
                                focus = Some(index);
                            }
                            // ★ Right-click focuses the row FIRST, so the
                            // menu is about the row the pointer is on rather
                            // than about whichever row was last clicked.
                            // Guarded on the row not already being focused
                            // because `set_focus` is a TOGGLE — a bare
                            // `focus = Some(index)` would make a right-click
                            // on the focused row *un*focus it, and the
                            // Properties panel would empty at the moment the
                            // operator asked to look at it.
                            if resp.secondary_clicked() && focused != Some(index) {
                                focus = Some(index);
                            }
                            // Attached on every frame, not only on the frame
                            // of the click: `egui` draws an open popup until
                            // it is dismissed. With no host, or with nothing
                            // on offer, this does nothing and opens nothing.
                            if let Some(host) = host {
                                tokens.extend(host.attach(&resp, menus::OBJECTS_ROW));
                            }
                        });
                    }
                    ObjectTreeRow::Part { object, part } => {
                        ui.horizontal(|ui| {
                            ui.add_space(t::OBJECT_TREE_INDENT);
                            // A part has points exactly when it is a
                            // SUBPATH; a text run has no anchors. Asking the
                            // provider's one dispatcher rather than matching
                            // on the object's kind here is what stops a
                            // second predicate drifting out of step with it.
                            if matches!(
                                provider.part_kind(object),
                                Some(provider::PartKind::Subpath)
                            ) {
                                let open = expanded_parts.contains(&(object, part));
                                if expander(ui, open).clicked() {
                                    toggle_part = Some((object, part));
                                }
                            } else {
                                ui.add_space(t::OBJECT_TREE_EXPANDER_WIDTH);
                            }
                            let resp = ui
                                .label(label.as_str())
                                .on_hover_text(t::object_tree_part_tooltip());
                            if overflows {
                                resp.on_hover_text(label.as_str());
                            }
                        });
                    }
                    ObjectTreeRow::Point { .. } => {
                        ui.horizontal(|ui| {
                            ui.add_space(POINT_ROW_INDENT);
                            ui.label(label.as_str())
                                .on_hover_text(t::object_tree_node_tooltip());
                        });
                    }
                    ObjectTreeRow::PointsCapped { .. } => {
                        ui.horizontal(|ui| {
                            ui.add_space(POINT_ROW_INDENT);
                            ui.label(egui::RichText::new(label.as_str()).small().weak());
                        });
                    }
                }
            }
        });

    if let Some(index) = toggle_object {
        tree.toggle_object(index);
    }
    if let Some((object, part)) = toggle_part {
        tree.toggle_part(object, part);
    }
    // ★★★ A row click SELECTS, since 2026-08-26.
    //
    // It used to call `tree.set_focus(index)`, which wrote a panel-local
    // `focus` that the canvas neither wrote nor read — the second of three
    // parallel notions of *"the thing I am working on"* that the interaction
    // audit named as the cause of the operator's *"when I have an object
    // selected like text the Tool tab doesn't switch to giving me the editable
    // stuff for that object."*
    //
    // Now the two ends write the same thing: a row click sets the canvas
    // selection, and the canvas selection is what the Properties panel
    // describes. Neither can disagree with the other.
    //
    // ★ The toggle is preserved and it matters. `set_focus` cleared the focus
    // when the already-focused row was clicked — *"a row click is its own
    // undo"* — and with a real selection model that reading is even better than
    // it was: clicking the selected row deselects, which is what clicking a
    // selected item does in every list in every application.
    if let Some(index) = focus {
        let already = doc
            .selection
            .object_indices_on(page_index)
            .first()
            .copied()
            .is_some_and(|selected| selected == index);
        actions.push(Action::SelectObject {
            page: page_index,
            object: (!already).then_some(crate::canvas::target::TargetId::Object(index as u64)),
        });
    }
    tokens
}

/// Leading space on a point row: two indents plus the expander column a
/// point row never fills (a point has nothing beneath it).
const POINT_ROW_INDENT: f32 = t::OBJECT_TREE_INDENT * 2.0 + t::OBJECT_TREE_EXPANDER_WIDTH;

/// Horizontal space an expander occupies, whether or not one is drawn.
///
/// Added to every measured row width so the container is wide enough for the
/// control as well as the text. Held even for a leaf, so labels stay aligned
/// down the column (R83: hold the space, draw no dead control).
const EXPANDER_SPACE: f32 = t::OBJECT_TREE_EXPANDER_WIDTH;

/// One expander button.
///
/// A **separate control** from the row, not a click on the label: expanding
/// to look inside and pointing the Properties panel at the object are
/// different intents, and one gesture cannot mean both without one of them
/// being a surprise.
///
/// Drawn as ASCII rather than as chevron art. The old shell used
/// `Icon::ChevronDown`/`ChevronRight` from its own icon set, and that set is
/// a separate `SALVAGE.md` Class A row (`icons.rs`, 1,747 lines) that has
/// not landed — so this is the honest interim, not a preference. It is a
/// real, clickable, tooltipped control either way; only the glyph changes
/// when the icon set arrives.
fn expander(ui: &mut egui::Ui, open: bool) -> egui::Response {
    let glyph = if open { "v" } else { ">" };
    ui.add_sized(
        egui::vec2(t::OBJECT_TREE_EXPANDER_WIDTH, ui.spacing().interact_size.y),
        egui::Button::new(glyph).small().frame(false),
    )
    .on_hover_text(t::object_tree_expander_tooltip())
}

/// How far a row is indented, in points.
///
/// Part of the measured row width, because indentation is width: a point row
/// indented twice needs 28 pt more container than its text alone.
fn row_indent(row: ObjectTreeRow) -> f32 {
    match row {
        ObjectTreeRow::Object { .. } => 0.0,
        ObjectTreeRow::Part { .. } => t::OBJECT_TREE_INDENT,
        ObjectTreeRow::Point { .. } | ObjectTreeRow::PointsCapped { .. } => {
            t::OBJECT_TREE_INDENT * 2.0
        }
    }
}

/// The text of one row.
///
/// Object rows go through
/// [`summary::describe_object`] and
/// [`crate::text::panels::objects::object_row`], which is the **single
/// description path** the Properties panel also reads — so a fill colour
/// cannot be described one way here and another way there.
fn row_label(provider: &ObjectModelProvider, row: ObjectTreeRow) -> String {
    match row {
        ObjectTreeRow::Object { index } => provider
            .page_objects()
            .objects
            .get(index)
            .map_or_else(String::new, |o| {
                t::object_row(index, &summary::describe_object(o))
            }),
        ObjectTreeRow::Part { object, part } => match provider.part_kind(object) {
            Some(provider::PartKind::Run) => t::object_tree_run_row(part),
            // A path's part, and the fallback for an object whose kind
            // cannot be established — "Part" is the general word and the row
            // exists because the part does.
            _ => t::object_tree_subpath_row(part),
        },
        ObjectTreeRow::Point { node, .. } => t::object_tree_node_row(node),
        ObjectTreeRow::PointsCapped { shown, total } => t::object_tree_points_capped(shown, total),
    }
}

/// Materialise the visible rows for one frame, front-most first.
///
/// Only expanded parents contribute children, so a fully-collapsed tree
/// costs exactly the object count — the same row budget a flat list would
/// have.
///
/// `point_cap` is a parameter rather than a constant read from
/// [`POINT_ROWS_PER_PART`] so the tests can drive the capping path with two
/// points instead of two hundred. The panel always passes the constant.
#[must_use]
pub fn build_rows(
    provider: &ObjectModelProvider,
    objects_expanded: &std::collections::BTreeSet<usize>,
    parts_expanded: &std::collections::BTreeSet<(usize, usize)>,
    point_cap: usize,
) -> Vec<ObjectTreeRow> {
    let total = provider.page_objects().objects.len();
    let mut rows = Vec::with_capacity(total);
    for display in 0..total {
        // Front-most first: display row 0 is the LAST-painted object.
        let index = total - 1 - display;
        rows.push(ObjectTreeRow::Object { index });
        if !objects_expanded.contains(&index) {
            continue;
        }
        for part in 0..provider.part_count(index) {
            rows.push(ObjectTreeRow::Part {
                object: index,
                part,
            });
            if !parts_expanded.contains(&(index, part)) {
                continue;
            }
            // A text run has no anchors, so this is empty for one and no
            // guard on the object's kind is needed — the ladder caps itself
            // at two rungs for text by construction.
            let points = provider.subpath_node_points(index, part);
            let total_points = points.len();
            for (node, _) in points.into_iter().take(point_cap) {
                rows.push(ObjectTreeRow::Point {
                    object: index,
                    part,
                    node,
                });
            }
            if total_points > point_cap {
                rows.push(ObjectTreeRow::PointsCapped {
                    shown: point_cap,
                    total: total_points,
                });
            }
        }
    }
    rows
}

/// Fixtures, for the tests in this module tree.
#[cfg(test)]
pub(crate) mod test_support {
    use std::path::PathBuf;

    /// Resolve a fixture under the **engine's** synthetic fixture tree.
    ///
    /// `D:\Dev\pdfcer\fixtures\synthetic\…`, reached by the same relative
    /// walk this crate's `Cargo.toml` uses to reach `pdfcer-core` — so if the
    /// crate compiles at all, this path resolves. That is why the failure
    /// below is an `assert!` and not a skip: a skip would silently turn
    /// every fixture-backed test in this tree into a no-op, which is the
    /// "gate that guards nothing" failure the ui-strings gate's own header
    /// spends four paragraphs on.
    ///
    /// `D:\Dev\pdfcer` is **read-only** for this project. These tests read
    /// from it and write nothing.
    pub fn engine_fixture(rel: &str) -> PathBuf {
        // ★★ BOTH spellings of the engine's directory are tried, newest first,
        // and that is the temporary rename shim reaching one more place.
        //
        // This project renamed to `pdfcer` before the engine did, so the engine
        // tree is still the pre-rename one — `Cargo.toml`'s `package = ...`
        // keys carry the same bridge. A single hard-coded sibling name pointed
        // at a directory that does not exist yet, and **every fixture-backed
        // test in this crate failed at once**.
        //
        // ⇒ When the engine's rename lands the first candidate resolves and the
        // second becomes dead. `tools/gates/check-engine-rename-shim.sh` fails
        // the build at that moment and names the places to clean up; this is
        // one of them.
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let candidates = [
            "../../../pdfcer/fixtures/synthetic",
            // old-name-exempt: the engine's pre-rename tree, which is the one
            // that exists today. Removed with the shim.
            "../../../pdfce/fixtures/synthetic", // old-name-exempt: the engine tree that exists today
        ];
        let path = candidates
            .iter()
            .map(|engine| root.join(engine).join(rel))
            .find(|p| p.exists())
            .unwrap_or_else(|| root.join(candidates[0]).join(rel));
        assert!(
            path.exists(),
            "the engine fixture {rel} is missing at {}. This crate builds against \
             the engine tree by path, so if it compiled, the tree is there — check \
             the fixture's name rather than the path.",
            path.display()
        );
        path
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pdfcer_core::content::ContentStream;
    use pdfcer_core::vector::{Matrix, NoXObjects, decompose};
    use pdfcer_render::tiny_skia::Transform;
    use std::collections::BTreeSet;

    fn provider(src: &[u8]) -> ObjectModelProvider {
        let cs = ContentStream::parse(src.to_vec()).expect("parse");
        let objects = decompose(&cs, Matrix::IDENTITY, &NoXObjects);
        ObjectModelProvider::from_parts(0, objects, Transform::identity())
    }

    fn collapsed() -> (BTreeSet<usize>, BTreeSet<(usize, usize)>) {
        (BTreeSet::new(), BTreeSet::new())
    }

    /// A provider over a fixture's first page, through the real
    /// `decompose_page`.
    ///
    /// **Necessary for anything about text runs.** The resolver-free
    /// `decompose` used by [`provider`] above reports `runs.len() == 0` for
    /// every text object by construction — the run layout needs the font
    /// resolver — so a content-stream literal cannot exercise the text side
    /// of the part rung at all. That is a property of the seam, not of the
    /// panel, and it is why the geometry cases use one helper and the text
    /// cases use the other.
    fn fixture_provider(rel: &str) -> ObjectModelProvider {
        let path = test_support::engine_fixture(rel);
        let doc = pdfcer_core::document::Document::load(&path).expect("the fixture loads");
        let pages = pdfcer_core::page_tree::pages(&doc).expect("a page tree");
        ObjectModelProvider::build(&doc.view(), &pages[0], 0).expect("the page decomposes")
    }

    /// **A collapsed tree is exactly the object count, front-most first.**
    ///
    /// Two invariants in one, and the second is the panel's whole diagnostic
    /// premise: row 0 is the object painted LAST, because that is the one a
    /// click resolves to first. Getting it backwards buries the answer at
    /// the bottom of a thousand rows.
    #[test]
    fn a_collapsed_tree_lists_every_object_once_topmost_first() {
        // Three rectangles, painted 0, 1, 2.
        let p = provider(b"0 0 10 10 re f 20 20 10 10 re f 40 40 10 10 re f");
        let (o, s) = collapsed();
        let rows = build_rows(&p, &o, &s, POINT_ROWS_PER_PART);
        assert_eq!(
            rows,
            vec![
                ObjectTreeRow::Object { index: 2 },
                ObjectTreeRow::Object { index: 1 },
                ObjectTreeRow::Object { index: 0 },
            ]
        );
    }

    /// **Expanding an object adds its parts, and expanding a part adds its
    /// points — object-scoped numbering intact.**
    ///
    /// The point indices are the ones `node-move --node N` takes, and they
    /// keep counting across a part boundary. A tree that restarted them at 0
    /// per part would print numbers that address a different point.
    #[test]
    fn expanding_walks_object_then_part_then_point() {
        // One object, two parts of two anchors each: points 0,1 then 2,3.
        let p = provider(b"0 0 m 10 0 l 100 5 m 110 5 l S");
        let mut objects = BTreeSet::new();
        objects.insert(0);
        let mut parts = BTreeSet::new();
        parts.insert((0, 1));

        let rows = build_rows(&p, &objects, &parts, POINT_ROWS_PER_PART);
        assert_eq!(
            rows,
            vec![
                ObjectTreeRow::Object { index: 0 },
                ObjectTreeRow::Part { object: 0, part: 0 },
                ObjectTreeRow::Part { object: 0, part: 1 },
                ObjectTreeRow::Point {
                    object: 0,
                    part: 1,
                    node: 2
                },
                ObjectTreeRow::Point {
                    object: 0,
                    part: 1,
                    node: 3
                },
            ],
            "part 1's points must be numbered 2 and 3, not 0 and 1"
        );
    }

    /// **A text object nests one level, not two.**
    ///
    /// Its parts are runs, and a run has no anchors — so the point rung is
    /// unreachable for text *by construction* rather than by a guard. If
    /// this ever produced a point row, something would have started matching
    /// on the object's kind in a second place.
    #[test]
    fn a_text_objects_parts_are_runs_and_have_no_points() {
        // Two explicitly-positioned runs in one text object — the shape a
        // CAD export's labels take, and the reason the rung is shared.
        let p = fixture_provider("text/runs-two-explicit.pdf");
        let text = (0..p.page_objects().objects.len())
            .find(|i| p.part_kind(*i) == Some(provider::PartKind::Run))
            .expect("the fixture must hold a text object");
        assert!(
            p.part_count(text) > 1,
            "the fixture must hold a MULTI-run text object, or the part rung is \
             untested for text"
        );

        let mut objects = BTreeSet::new();
        objects.insert(text);
        let mut parts = BTreeSet::new();
        parts.insert((text, 0));
        let rows = build_rows(&p, &objects, &parts, POINT_ROWS_PER_PART);

        assert!(
            !rows
                .iter()
                .any(|r| matches!(r, ObjectTreeRow::Point { .. })),
            "a run has no anchors, so no point row may exist: {rows:?}"
        );
        // …and the rows are worded as runs, not as parts.
        let part = rows
            .iter()
            .find(|r| matches!(r, ObjectTreeRow::Part { .. }))
            .copied()
            .expect("a part row");
        let label = row_label(&p, part);
        assert!(label.starts_with("Run #"), "{label}");
    }

    /// **An image is a leaf.**
    #[test]
    fn an_image_contributes_no_children_even_when_expanded() {
        let p = provider(b"q 100 0 0 50 10 10 cm BI /W 1 /H 1 /CS /G /BPC 8 ID \x00 EI Q");
        let mut objects = BTreeSet::new();
        objects.insert(0);
        let rows = build_rows(&p, &objects, &BTreeSet::new(), POINT_ROWS_PER_PART);
        assert_eq!(rows, vec![ObjectTreeRow::Object { index: 0 }]);
    }

    /// **★ A capped point list discloses BOTH numbers.**
    ///
    /// The measured case is 6,681 anchors in one path object. A list quietly
    /// shortened to its first N is indistinguishable from a list that is N
    /// long, which is the same defect `bookmarks_truncated` exists to
    /// prevent one panel over.
    #[test]
    fn a_capped_point_list_says_how_many_it_hid() {
        // Four anchors in one part; cap at two.
        let p = provider(b"0 0 m 10 0 l 20 0 l 30 0 l S");
        let mut objects = BTreeSet::new();
        objects.insert(0);
        let mut parts = BTreeSet::new();
        parts.insert((0, 0));

        let rows = build_rows(&p, &objects, &parts, 2);
        let points = rows
            .iter()
            .filter(|r| matches!(r, ObjectTreeRow::Point { .. }))
            .count();
        assert_eq!(points, 2, "the cap must actually cap");
        let capped = rows
            .iter()
            .find_map(|r| match r {
                ObjectTreeRow::PointsCapped { shown, total } => Some((*shown, *total)),
                _ => None,
            })
            .expect("the cap must be disclosed as a row");
        assert_eq!(capped, (2, 4));
        let label = row_label(
            &p,
            ObjectTreeRow::PointsCapped {
                shown: capped.0,
                total: capped.1,
            },
        );
        assert!(label.contains('2') && label.contains('4'), "{label}");

        // …and an UNcapped part gets no such row, because there is nothing
        // to disclose. A disclosure that is always present is one operators
        // stop reading.
        let uncapped = build_rows(&p, &objects, &parts, 99);
        assert!(
            !uncapped
                .iter()
                .any(|r| matches!(r, ObjectTreeRow::PointsCapped { .. })),
            "{uncapped:?}"
        );
    }

    /// **★ THE NO-CLIP TEST: a row wider than the pane widens the container
    /// rather than being squeezed into it.**
    ///
    /// `SALVAGE.md`'s requirement for this panel, made mechanical: *"Row text
    /// must not clip; the old panel truncated with no horizontal scroll."*
    ///
    /// The measurement is the *intrinsic* width of the row's own text plus
    /// its indent and expander space, and the container is the max of that
    /// and the viewport. If the container came out equal to the viewport for
    /// a long row, `ScrollArea` would compare content against viewport, find
    /// them equal, draw no bar, and the row would be cut off at the pane's
    /// edge with nothing to say so — which is exactly the old defect.
    ///
    /// A stand-in measurer is used rather than a live `egui::Fonts`, for the
    /// reason `crate::lib`'s header gives: a windowed UI cannot run on a CI
    /// runner, so the *arithmetic* is pushed into a pure function and tested
    /// there. What is proven here is that the panel feeds that function the
    /// row's own width and not the pane's.
    #[test]
    fn a_long_row_widens_the_container_past_the_viewport() {
        // A row whose text is genuinely long: a filled-and-stroked path with
        // a colour, a width, a node count and a disclosure.
        let p = provider(b"0 0 1 rg 1 0 0 RG 0.5 w 10 20 m 300 20 l B*");
        let (o, s) = collapsed();
        let rows = build_rows(&p, &o, &s, POINT_ROWS_PER_PART);
        let label = row_label(&p, rows[0]);
        assert!(
            label.chars().count() > 40,
            "this test needs a row long enough to overflow a dock pane; got {} \
             chars: {label}",
            label.chars().count()
        );

        // ~6 pt per character is a plausible body-text advance; the exact
        // figure does not matter, only that the row measures wider than the
        // pane.
        const PANE: f32 = 370.0;
        let measure = |s: &str| s.chars().count() as f32 * 6.0;
        let widths: Vec<f32> = rows
            .iter()
            .map(|r| measure(&row_label(&p, *r)) + row_indent(*r) + EXPANDER_SPACE)
            .collect();
        let widest = widths.iter().copied().fold(0.0_f32, f32::max);
        assert!(
            widest > PANE,
            "the fixture row is not wide enough to exercise the overflow path: \
             {widest} <= {PANE}"
        );

        let container = content_width(widths.iter().copied(), PANE);
        assert!(
            container > PANE,
            "the container must be wider than the viewport, or the area draws no \
             horizontal bar and the row is clipped: {container} <= {PANE}"
        );
        assert!((container - widest).abs() < f32::EPSILON);
    }

    /// …and the indent is part of that width.
    ///
    /// A point row indented twice needs 28 pt more container than its text
    /// alone. Measuring the text and forgetting the indent reintroduces
    /// clipping for exactly the deepest rows, which are the ones an operator
    /// had to work hardest to reach.
    #[test]
    fn indentation_counts_toward_the_row_width() {
        assert!((row_indent(ObjectTreeRow::Object { index: 0 })).abs() < f32::EPSILON);
        assert!(
            row_indent(ObjectTreeRow::Part { object: 0, part: 0 })
                < row_indent(ObjectTreeRow::Point {
                    object: 0,
                    part: 0,
                    node: 0
                })
        );
        assert!(
            row_indent(ObjectTreeRow::PointsCapped { shown: 1, total: 2 })
                > row_indent(ObjectTreeRow::Part { object: 0, part: 0 }),
            "the disclosure row sits with the points it describes"
        );
    }

    /// **An object row's label is the shared description, index first.**
    ///
    /// One description path: the row and the Properties panel read the same
    /// `ObjectSummary`, so a fill colour cannot be described one way here
    /// and another way there.
    #[test]
    fn an_object_row_is_labelled_by_the_shared_description() {
        let p = provider(b"0 0 1 rg 10 10 80 80 re f");
        let label = row_label(&p, ObjectTreeRow::Object { index: 0 });
        let direct = t::object_row(0, &summary::describe_object(&p.page_objects().objects[0]));
        assert_eq!(label, direct);
        assert!(label.starts_with("#0"));
    }

    /// A row for an object that is no longer there labels as empty rather
    /// than panicking.
    ///
    /// The rows are materialised from one snapshot of the provider, so this
    /// cannot happen today. It is asserted because the row builder and the
    /// row renderer are separate functions, and the day something rebuilds
    /// one without the other, an index panic in a draw closure is a crash
    /// with no useful stack.
    #[test]
    fn a_row_naming_a_missing_object_does_not_panic() {
        let p = provider(b"10 10 80 80 re f");
        assert_eq!(row_label(&p, ObjectTreeRow::Object { index: 99 }), "");
    }
}
