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
//! ### 1. Row text no longer clips — and since O123, it ELLIPSISES
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
//! ★★★ **This shell answered that with a two-axis scroll area for a month, and
//! on 2026-09-04 the operator replaced the answer** — `OPERATOR_REQUESTS.md`
//! O123: *"rows that ellipsise with a tooltip instead of hard-clipping
//! mid-character."*
//!
//! What ships now, and why each half is needed:
//!
//! - **[`crate::panels::elide_to_width`] per row, against that row's own
//!   room.** The pane's width less this row's indent and its expander column,
//!   because a point row indented twice has 28 pt less text room than the
//!   object row above it. The row is shortened to fit and ends in a single
//!   ellipsis, so the eye is told it was cut.
//! - **The full text on hover, on every shortened row**, including the point
//!   rows and the capped-rows disclosure — which never carried one before,
//!   because they could not overflow while the pane scrolled sideways and they
//!   can now.
//! - **`ScrollArea::vertical`.** The horizontal axis existed to reach the part
//!   of a row past the pane's edge. There is no such part any more, and a
//!   scroll bar with nothing beyond its viewport is a control that cannot do
//!   anything — R9.
//!
//! ★ `SALVAGE.md`'s requirement is still met, by a different route: what it
//! forbade was **silent** loss, and nothing here is silent. An operator seeing
//! `#27 Text · "A1" · AAAAAA+SpaceGrotesk-Bold 1` with a `2` invisibly cut off
//! was the defect; `…` plus a hover is not it.
//!
//! ⚠ What was given up, stated rather than glossed: an operator could
//! previously read a very long row **in the panel**, by dragging a bar. Now
//! they read it in a tooltip. That is the operator's own trade and it is why
//! the same request also widened Edit's dock to 360 pt — the width at which the
//! *common* row stops needing either affordance.
//!
//! And a third guard on top, because neither helps a 6,681-row part:
//! [`POINT_ROWS_PER_PART`] caps how many point rows one part contributes, and
//! the cap is **disclosed with both numbers** rather than the list being
//! quietly shortened. A silently truncated list is indistinguishable from a
//! short one.
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
use crate::panels::{PanelsState, elide_to_width, text_width};
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

    // ★★★ **`ScrollArea::vertical`, not `both`, since 2026-09-04** —
    // `OPERATOR_REQUESTS.md` O123.
    //
    // A horizontal bar is what this panel used to offer instead of shortening a
    // row: the container stated its own intrinsic width via
    // `crate::panels::content_width`, the area measured content wider than
    // viewport, and a bar appeared. That worked, and the operator did not want
    // it — *"rows that ellipsise with a tooltip instead of hard-clipping
    // mid-character."*
    //
    // ⇒ With every row shortened to the pane, there is nothing left of a row to
    // scroll sideways to, and a horizontal bar with nothing beyond its viewport
    // is a control that cannot do anything. R9. So the axis goes with the
    // mechanism it existed for.
    egui::ScrollArea::vertical()
        .id_salt("objects-tree-rows")
        .show_rows(ui, row_height, total_rows, |ui, range| {
            // Label every row in the visible range FIRST, and measure each
            // against its own indent — a point row indented twice has 28 pt
            // less text room than the object row above it, and shortening both
            // to the same character count would leave the deep row short and
            // the shallow row cut.
            let labelled: Vec<(ObjectTreeRow, String, Option<String>)> = range
                .filter_map(|i| rows.get(i).copied())
                .map(|row| {
                    let label = row_label(provider, row);
                    // The text's own room: the pane, less the indentation and
                    // the expander column this row will spend before its first
                    // character.
                    let room = viewport - row_indent(row) - EXPANDER_SPACE;
                    let shortened =
                        elide_to_width(&label, room, |candidate| text_width(ui, candidate));
                    (row, label, shortened)
                })
                .collect();

            // ★★★ **The elision report, and it is the ONLY channel a driven
            // check can read this decision through** — `OPERATOR_REQUESTS.md`
            // O123.
            //
            // `objects-panel` above counts LAID-OUT rows, produced before the
            // draw, and the recorded lesson
            // `a_per_item_diagnostic_line_is_not_a_list_of_what_you_can_click`
            // is about exactly that: it answers *what was computed*, never
            // *what is on screen*. This line answers the second question, for
            // the rows actually in the viewport.
            //
            // ⚠ It is the application marking its own homework, and it is
            // published anyway because there is no substitute — the harness
            // cannot read the text a panel renders. What makes the check built
            // on it non-circular is the PIXEL half:
            // `the_objects_rows_fit_the_inspector` samples the pane's right
            // edge, and a column of background there cannot be produced by an
            // arithmetic error in the same function that writes this line.
            //
            // `trace_changed`, not `trace`: a still panel would otherwise write
            // this sixty times a second.
            crate::diag::trace_changed(ROWS_SLOT, || {
                let elided = labelled.iter().filter(|(_, _, e)| e.is_some()).count();
                // ui-text-exempt: diagnostic trace, never displayed.
                format!(
                    "objects-rows visible={} elided={elided} pane={viewport:.1}",
                    labelled.len()
                )
            });

            for (row, full, shortened) in labelled {
                // ★ Two strings per row from here down, and the pair is the
                // whole of O123's row work: `label` is what is DRAWN and `full`
                // is what the hover SHOWS. They are the same string on a row
                // that fits, and the hover is not attached then — a tooltip
                // repeating a fully visible label is noise.
                let overflows = shortened.is_some();
                let label = shortened.unwrap_or_else(|| full.clone());
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
                                resp = resp.on_hover_text(full.as_str());
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
                                resp.on_hover_text(full.as_str());
                            }
                        });
                    }
                    ObjectTreeRow::Point { .. } => {
                        ui.horizontal(|ui| {
                            ui.add_space(POINT_ROW_INDENT);
                            // ★ A point row gains the recovery hover it never
                            // had. It could not overflow before — it was
                            // horizontally scrollable like every other row —
                            // and it can now, because it is shortened like
                            // every other row. A coordinate pair cut
                            // mid-number is exactly the failure O123 names.
                            let resp = ui
                                .label(label.as_str())
                                .on_hover_text(t::object_tree_node_tooltip());
                            if overflows {
                                resp.on_hover_text(full.as_str());
                            }
                        });
                    }
                    ObjectTreeRow::PointsCapped { .. } => {
                        ui.horizontal(|ui| {
                            ui.add_space(POINT_ROW_INDENT);
                            let resp = ui.label(egui::RichText::new(label.as_str()).small().weak());
                            // The capped-rows notice carries BOTH numbers by
                            // design — a silently truncated list is
                            // indistinguishable from a short one — so it is the
                            // one row here that must never lose its tail
                            // without a way back to it.
                            if overflows {
                                resp.on_hover_text(full.as_str());
                            }
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

/// The change-log slot [`body`]'s per-frame elision report is keyed on.
///
/// ★ Its own slot rather than sharing `objects-panel`'s, because the two lines
/// change on different events: the panel line moves when the page does, and
/// this one moves when the dock is dragged. Sharing a slot would make each
/// suppress the other's changes.
const ROWS_SLOT: &str = "objects-rows"; // ui-text-exempt: trace slot name, never displayed

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

    /// ★★★ **A long row is SHORTENED to the pane, and the whole of it stays
    /// reachable** — `OPERATOR_REQUESTS.md` O123.
    ///
    /// This test used to be `a_long_row_widens_the_container_past_the_viewport`
    /// and asserted the opposite mechanism: that
    /// `crate::panels::content_width` grew the container past the viewport so
    /// `ScrollArea::both` would draw a horizontal bar. That was the shipped
    /// answer to `SALVAGE.md:44` for a month, and the operator has replaced it
    /// — *"rows that ellipsise with a tooltip instead of hard-clipping
    /// mid-character."*
    ///
    /// ★ The requirement it enforced has not changed and is asserted here in
    /// its new form: **nothing is lost silently.** The drawn row fits the pane,
    /// it ends in the ellipsis so the eye knows it was cut, and it is a prefix
    /// of the full row the hover carries.
    #[test]
    fn a_long_row_is_shortened_to_the_pane_and_stays_recoverable() {
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
        let room = PANE - row_indent(rows[0]) - EXPANDER_SPACE;
        assert!(
            measure(&label) > room,
            "the fixture row is not wide enough to exercise the elision path: \
             {} <= {room}",
            measure(&label)
        );

        let shown = crate::panels::elide_to_width(&label, room, measure)
            .expect("a row wider than its room must be shortened");
        assert!(
            measure(&shown) <= room,
            "the shortened row is {} pt in {room} pt of room",
            measure(&shown)
        );
        assert!(
            shown.ends_with(crate::panels::ELLIPSIS),
            "{shown:?} does not say that it was cut"
        );
        let kept = &shown[..shown.len() - crate::panels::ELLIPSIS.len_utf8()];
        assert!(
            label.starts_with(kept),
            "{shown:?} is not a prefix of the row it shortened"
        );
        // And the half that makes it not a loss: the hover carries the whole
        // thing, so the tail is one gesture away rather than gone.
        assert!(label.len() > kept.len());
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
