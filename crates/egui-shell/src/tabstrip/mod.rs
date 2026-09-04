//! # `tabstrip` — a row of document tabs, and nothing that knows what a
//! document is
//!
//! One reusable surface: the strip of tabs an application draws across the top
//! of its work area when the operator has **several things open at once**.
//! Chrome's tabs, VS Code's editor tabs, Acrobat's document tabs, Illustrator's
//! — every one of them is this widget, and every operator already knows how it
//! behaves before they see it.
//!
//! It is deliberately *not* [`crate::dock`]'s tab bar, and the two must not be
//! merged. They answer different questions:
//!
//! | | [`crate::dock`]'s tabs | this |
//! |---|---|---|
//! | what a tab names | a **panel** — a tool, always available, part of the workspace | a **document** — an operand, transient, the thing the workspace acts on |
//! | who owns the list | the operator's saved layout | whatever the operator has open right now |
//! | closing one | takes a tool off screen; the ✕ is on a context menu, because a per-tab glyph would make the width arithmetic depend on closability | destroys an operand; the ✕ is **on the tab**, because that is where every tabbed application in the world puts it |
//! | how many | a handful, chosen deliberately | as many as the operator opened, chosen by accident |
//!
//! What they *do* share is the thing that is hard: the **overflow
//! reservation**. `MODES_AND_PANELS.md` Part 2 failure mode #8 — *"past ~6
//! tabs the overflow button itself gets hidden, leaving no route to the hidden
//! tabs"* — applies to a strip of twelve open drawings at least as much as to
//! a stack of panels. So the arithmetic is [`crate::dock::plan`]'s, unchanged
//! and un-copied. A second implementation of a reservation rule is how one of
//! them comes to be subtly wrong.
//!
//! ---
//!
//! ## What this module refuses to know
//!
//! R7: `egui-shell` never learns what a PDF is, and this file is a place the
//! temptation is real, because "is this document modified?" and "which
//! document should spring open under a drag?" are both questions the strip
//! could plausibly answer.
//!
//! It answers neither.
//!
//! - **Modified** is not a field here. The caller puts whatever marker its
//!   domain uses into [`TabItem::label`], and this draws the label. A
//!   `modified: bool` would immediately raise *"drawn how?"*, and the answer
//!   differs per application (an asterisk, a dot, a colour, an italic).
//! - **Spring-loading** — the browser and file-manager convention where
//!   hovering a tab during a drag activates it — is not implemented here
//!   either. This reports [`TabStrip::hovered`] and the caller decides whether
//!   a hover means anything, because *what is being dragged* is exactly the
//!   domain knowledge this crate must not acquire. The dwell timer, and the
//!   question of whether a drag is in flight at all, belong to the
//!   application.
//!
//! Both of those are extension points rather than exceptions, which is what
//! `R7` asks for when a shell surface seems to need to know something.
//!
//! ---
//!
//! ## The gestures, and where each one comes from
//!
//! | gesture | effect | precedent |
//! |---|---|---|
//! | primary click on a tab | [`TabIntent::Activate`] | universal |
//! | primary click on the ✕ | [`TabIntent::Close`] | universal |
//! | **middle** click anywhere on a tab | [`TabIntent::Close`] | every browser, VS Code, and most editors. Costs nothing and is the gesture a heavy user reaches for |
//! | the overflow affordance | a menu of the hidden tabs, each activating | [`crate::dock`]'s, for the reason above |
//!
//! Every one of them is reported as an **intent**, never applied. The strip
//! does not own the list it draws, and an application that has to ask about
//! unsaved work before closing a tab cannot have the close already done by the
//! time it is told. That is the same discipline [`crate::dock`] follows for
//! the same reason.

use egui::{Align, Layout, Rect, RichText, UiBuilder, Vec2};

use crate::dock::plan;
use crate::theme::Theme;

/// **The height of the strip**, in logical points.
///
/// A constant, and it must stay one. A surface whose height varies with its
/// content and which sits above a viewport that fits a page to itself forms a
/// measured feedback loop — `D:\dev\rag\egui\bottom_panel_height_change_retriggers_fit_to_viewport_zoom.md`
/// records a 230 % → 224 % → 215 % zoom drift from exactly that shape. The
/// caller is expected to give this an `exact_size` panel, not a
/// `default_height` one.
///
/// Two points taller than [`plan::TAB_BAR_HEIGHT`], deliberately: a document
/// tab carries a close glyph beside its label and a 24 pt row makes the two
/// touch.
pub const STRIP_HEIGHT: f32 = 26.0;

/// The width reserved inside each tab for the close control.
///
/// Fixed rather than measured, because it is a fixed glyph in a fixed size and
/// measuring it would make the tab-width arithmetic depend on the font — which
/// is exactly the dependency that makes an overflow reservation drift.
const CLOSE_WIDTH: f32 = 16.0;

/// The close glyph.
///
/// **U+00D7 MULTIPLICATION SIGN**, not U+2715 MULTIPLICATION X and not U+2716.
/// It is in Latin-1, so it is present in every font this application could
/// possibly fall back to, and `epaint`'s `has_glyph` is not a coverage oracle
/// (`D:\dev\rag\egui\epaint_has_glyph_is_resolved_face_vs_replacement_face_not_a_coverage_oracle.md`)
/// — so "will this render?" is a question best answered by not asking it.
const CLOSE_GLYPH: &str = "\u{00d7}"; // ui-text-exempt: a glyph, not a sentence

/// One tab.
///
/// Built fresh each frame by the caller from whatever it has open. There is no
/// retained model here on purpose: a strip that cached its own list would be a
/// second copy of *what is open*, and the two would disagree the first time a
/// close failed.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TabItem {
    /// **What the operator reads.** Already carrying any domain marker — see
    /// this module's header on why `modified` is not a field here.
    ///
    /// Truncated with an ellipsis when the tab is narrower than the text, so a
    /// caller that prefixes a marker keeps it visible and one that suffixes it
    /// does not.
    pub label: String,
    /// The hover text, and the **accessible name**. Expected to be the
    /// unabbreviated thing — a full path where the label is a file name — so
    /// that a truncated tab is still identifiable.
    pub tooltip: String,
    /// Whether this tab may be closed from the strip.
    ///
    /// `false` draws no ✕ and ignores a middle click. Present because a
    /// caller may have a tab that is not the operator's to close, and because
    /// "the button is there and does nothing" is the failure this project
    /// names `R9`.
    pub closable: bool,
}

impl TabItem {
    /// A closable tab with `label` reading and `tooltip` announcing.
    #[must_use]
    pub fn new(label: impl Into<String>, tooltip: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            tooltip: tooltip.into(),
            closable: true,
        }
    }
}

/// What the operator asked for. **Never applied here.**
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TabIntent {
    /// Show the tab at this index.
    Activate(usize),
    /// Close the tab at this index.
    ///
    /// The caller may refuse, ask first, or defer. Nothing about this strip
    /// assumes the tab is gone on the next frame.
    Close(usize),
    /// **Move the tab at `from` to the boundary `gap`.**
    ///
    /// Raised when a tab is dragged along the strip and released somewhere
    /// else. Every tabbed application allows this and nobody has to be taught
    /// it; a strip whose order is fixed is the one thing about a tab strip an
    /// operator notices as missing.
    ///
    /// # ★ `gap` is a BOUNDARY, not a destination index
    ///
    /// `0` is before the first tab and `len` is after the last, which is the
    /// same vocabulary an insertion caret is drawn in and the same one a page
    /// drop uses. It is deliberately *not* "the index it ends up at", because
    /// those two differ by one whenever the tab moves rightward — the tab is
    /// removed before it is re-inserted — and a caller that got the convention
    /// wrong would be off by one in one direction only, which is the hardest
    /// kind of off-by-one to notice.
    ///
    /// A gap of `from` or `from + 1` is where the tab already is. The strip
    /// raises the intent anyway rather than filtering it; the caller is the one
    /// that knows whether a no-op is worth tracing.
    Reorder {
        /// The tab being moved.
        from: usize,
        /// The boundary it is moving to.
        gap: usize,
    },
}

/// What one frame of the strip produced.
#[derive(Clone, Debug, Default)]
pub struct TabStrip {
    /// What the operator asked for, in the order it happened.
    pub intents: Vec<TabIntent>,
    /// **The tab the pointer is over**, if any — the raw fact a caller needs
    /// to build a spring-loaded hover on top of, without this module learning
    /// what is being dragged.
    ///
    /// `None` when the pointer is over the strip's background, over the
    /// overflow affordance, or off the strip entirely.
    pub hovered: Option<usize>,
    /// Every tab that was actually drawn, with the rectangle it was drawn in.
    ///
    /// ★ Published rather than left to be derived. A harness that computes a
    /// tab's position from an index and a width can be wrong in the same
    /// direction as the code under test — `D:\dev\rag\egui\a_ui_rect_change_log_produces_confident_wrong_failures_in_BOTH_directions.md`
    /// and the *"do not compute a coordinate the application could publish"*
    /// rule that came out of 2026-08-19. Hidden tabs are absent from this
    /// list, which is itself the fact a check about overflow wants.
    pub drawn: Vec<(usize, Rect)>,
    /// How many tabs did not fit and are reachable only through the overflow
    /// menu.
    pub hidden: usize,
    /// **Each drawn tab's own `Response`**, handed out so the caller can attach
    /// a context menu to it.
    ///
    /// ★ Handed out rather than used here, and that is a hard constraint rather
    /// than a preference. A `Response` carries exactly **one** popup id
    /// (`response.id.with("popup")`), so a widget can host exactly one context
    /// menu: if this module attached its own, an application could never add
    /// one, and two menus on one response are two writers of one flag in
    /// `egui`'s memory. [`crate::dock::tabs`] hit the same wall and resolved it
    /// the same way.
    ///
    /// *What* a right-click on a document tab should offer is the application's
    /// business — close, close others, detach — and none of it is expressible
    /// without knowing what a document is, which R7 forbids this crate from
    /// knowing.
    ///
    /// In drawn order, absent for a tab behind the overflow affordance.
    pub responses: Vec<(usize, egui::Response)>,
    /// **A tab drag in flight, and where it would land**, as a `(from, gap)`
    /// pair — for a caller that wants to say so in words.
    ///
    /// `None` when no tab is being dragged. The caret itself is drawn here; this
    /// is the same fact in numbers, for the same reason the page grid publishes
    /// its landing: *a hairline between two near-identical labels is precise and
    /// not checkable*.
    pub reordering: Option<(usize, usize)>,
}

/// **Draw the strip.**
///
/// `active` is the index of the tab currently on screen; out-of-range is
/// treated as "none of them is active", which is a state a caller can reach
/// legitimately for one frame while a close is being confirmed.
///
/// Returns intents. Applies nothing.
///
/// # Layout, in the order that makes the reservation hold
///
/// The same six steps [`crate::dock::tabs`] documents, because it is the same
/// arithmetic:
///
/// 1. measure every label (egui memoizes the galley, so this is a hash lookup)
/// 2. add [`CLOSE_WIDTH`] to each, then clamp through [`plan::tab_width`]
/// 3. subtract the overflow affordance's width from the strip's — **first**
/// 4. choose the visible window inside what is left
/// 5. lay the tabs into a rect that *is* that budget
/// 6. lay the affordance into the space nothing else was allowed to touch
///
/// Step 3 before step 4 is what makes it impossible for the number of tabs to
/// eat the route to the tabs.
#[must_use]
pub fn strip(ui: &mut egui::Ui, theme: &Theme, tabs: &[TabItem], active: usize) -> TabStrip {
    let mut out = TabStrip::default();
    let rect = ui.max_rect();
    if tabs.is_empty() || rect.width() <= 0.0 {
        return out;
    }

    ui.painter().rect_filled(rect, 0.0, theme.palette.panel);

    // 1 & 2 — measure, and pay for the close control up front.
    let widths: Vec<f32> = tabs
        .iter()
        .map(|t| {
            let close = if t.closable { CLOSE_WIDTH } else { 0.0 };
            plan::tab_width(text_width(ui, &t.label) + close)
        })
        .collect();

    // 3 — the reservation, subtracted before anything is placed.
    let overflow_w = plan::overflow_width(tabs.len(), plan::TAB_PADDING, |s| text_width(ui, s));
    let bar = plan::plan_tabs(
        &widths,
        active.min(tabs.len().saturating_sub(1)),
        rect.width(),
        plan::TAB_GAP,
        overflow_w,
    );
    out.hidden = bar.hidden;

    // 4 & 5 — the visible window, inside a rect that is the budget.
    let mut x = rect.left();
    for i in bar.start..bar.start + bar.shown {
        let width = widths[i].min((rect.left() + bar.tab_budget - x).max(0.0));
        if width <= 0.0 {
            break;
        }
        let tab_rect =
            Rect::from_min_size(egui::pos2(x, rect.top()), Vec2::new(width, rect.height()));
        draw_tab(ui, theme, &tabs[i], i, i == active, tab_rect, &mut out);
        out.drawn.push((i, tab_rect));
        x += width + plan::TAB_GAP;
    }

    // ★★ **Which tab the pointer is over, resolved GEOMETRICALLY.**
    //
    // NOT from `Response::hovered()`, and the difference is the whole
    // spring-loading feature.
    //
    // While a drag is in flight `egui` locks interaction to the widget that was
    // pressed, so **every other widget reports `hovered() == false`** — including
    // the tab the operator is deliberately holding the pointer over. A hover
    // built from a `Response` is therefore false in exactly the one situation a
    // spring-loaded target exists for, and it fails silently: the tab is
    // visibly under the pointer and nothing happens.
    //
    // Measured, 2026-08-20: `a_page_dragged_between_documents_is_copied` drove
    // this, the trace carried `page-drag-start` and no spring, and the drop
    // landed back in the source document.
    //
    // A rectangle and a pointer position are facts that do not care who owns the
    // interaction, which is what makes them the right instrument here. It is the
    // same reason `pdfcer`'s own page grid resolves its drop target from
    // `pointer_latest_pos()` against a tile rect rather than from the tile's
    // response.
    //
    // Resolved over `out.drawn` — the tabs actually laid out this frame — so a
    // tab behind the overflow affordance cannot be hovered, which is correct:
    // it is not on screen.
    if let Some(pointer) = ui.ctx().pointer_latest_pos() {
        out.hovered = out
            .drawn
            .iter()
            .find(|(_, r)| r.contains(pointer))
            .map(|(i, _)| *i);
    }

    // 6 — the affordance, in reserved space.
    if bar.has_overflow() {
        let affordance = Rect::from_min_max(
            egui::pos2(rect.left() + bar.tab_budget + plan::TAB_GAP, rect.top()),
            rect.max,
        );
        draw_overflow(ui, tabs, bar.hidden, affordance, &mut out);
    }

    // 7 — the reorder drag, resolved and painted after everything else.
    settle_reorder(ui, theme, rect, &mut out);

    out
}

/// **A tab drag in flight**, between frames.
///
/// In `egui::Memory` rather than in a field on [`TabStrip`], because
/// [`TabStrip`] is built fresh every frame — the strip is deliberately
/// stateless (see [`TabItem`]) and a drag has to outlive a frame. This is the
/// only thing about the strip that does.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
struct TabDrag {
    /// The tab the press landed on.
    from: usize,
}

/// The reorder drag's memory key. Salted with the `Ui`'s own id so two strips
/// in one application cannot share a drag.
fn drag_id(ui: &egui::Ui) -> egui::Id {
    ui.id().with("tabstrip-drag") // ui-text-exempt: an id, never displayed
}

/// **Resolve a reorder drag, draw its caret, and settle its release.**
///
/// Runs after every tab is laid out, because the boundary the caret marks does
/// not exist until they are — the same reason a page grid resolves its drop
/// target inside its layout pass.
///
/// # The gap is resolved by CENTRES, not by edges
///
/// A tab whose centre is left of the pointer is a tab the dragged one has
/// passed. That makes the boundary flip when the pointer crosses the middle of
/// a neighbour, which is what every tab strip does and what stops the caret
/// jittering between two gaps while the pointer sits over the seam between two
/// tabs.
fn settle_reorder(ui: &mut egui::Ui, theme: &Theme, strip_rect: Rect, out: &mut TabStrip) {
    let id = drag_id(ui);
    let Some(drag) = ui.ctx().data(|d| d.get_temp::<TabDrag>(id)) else {
        return;
    };
    let Some(pointer) = ui.ctx().pointer_latest_pos() else {
        return;
    };

    // The boundary: how many drawn tabs the pointer has passed the middle of.
    // Seeded from the leftmost drawn tab so a scrolled strip cannot report a
    // gap left of what is on screen.
    let mut gap = out.drawn.first().map_or(0, |(i, _)| *i);
    for (i, r) in &out.drawn {
        if pointer.x > r.center().x {
            gap = i + 1;
        }
    }
    out.reordering = Some((drag.from, gap));

    // The caret, at the boundary. Painted after the tabs, so it is over them
    // rather than under — in an immediate-mode painter that is call order and
    // nothing else.
    //
    // Its x is read from a drawn rectangle rather than computed from a width,
    // for the rule this crate's own `drawn` field carries: do not derive a
    // coordinate the layout already knows.
    let x = out.drawn.iter().find(|(i, _)| *i == gap).map_or_else(
        || {
            out.drawn
                .last()
                .map_or(strip_rect.left(), |(_, r)| r.right())
        },
        |(_, r)| r.left(),
    );
    ui.painter().line_segment(
        [
            egui::pos2(x, strip_rect.top()),
            egui::pos2(x, strip_rect.bottom()),
        ],
        egui::Stroke::new(CARET_PTS, theme.palette.accent),
    );
    ui.ctx().set_cursor_icon(egui::CursorIcon::Grabbing);

    // ★ The release is read from RAW POINTER INPUT, not from the tab's own
    // `Response`.
    //
    // A drag begun on a tab may end anywhere — past the last tab, over the
    // canvas, off the window — and a `Response` only reports releases inside
    // the widget that produced it. Reading the input means a drag always ends,
    // which is the property that stops a half-finished drag surviving into the
    // next frame as a caret nobody can get rid of.
    if ui
        .ctx()
        .input(|i| i.pointer.button_released(egui::PointerButton::Primary))
    {
        ui.ctx().data_mut(|d| d.remove_temp::<TabDrag>(id));
        out.reordering = None;
        out.intents.push(TabIntent::Reorder {
            from: drag.from,
            gap,
        });
    }
}

/// How thick the reorder caret is drawn.
///
/// The same weight the dock's and the page grid's carets use: thin enough to
/// read as a boundary rather than as a tab, thick enough not to look like a
/// rendering artefact on a dense strip.
const CARET_PTS: f32 = 2.0;

/// Draw one document tab: the label, and the ✕ beside it.
fn draw_tab(
    ui: &mut egui::Ui,
    theme: &Theme,
    tab: &TabItem,
    index: usize,
    selected: bool,
    rect: Rect,
    out: &mut TabStrip,
) {
    // ★ The close control is laid out FIRST and the label takes what is left.
    //
    // The other order is the obvious one and it is wrong: a label allowed to
    // claim the whole tab pushes the ✕ out of the rect on exactly the tabs
    // that are too narrow — which is every tab, once enough documents are
    // open. Reserving the control and truncating the label is the same
    // discipline the overflow affordance gets one level up, applied inside the
    // tab.
    // ★★★ THE SELECTED TAB'S PLATE, PAINTED ACROSS THE WHOLE RECT AND BEFORE
    // IT IS SPLIT — 2026-09-03. Two contrast defects converge here and one
    // paint closes both.
    //
    // (1) The label button below is `.selected(selected)` with no `.fill()`,
    //     and `egui::Style::button_style` overwrites the fill from
    //     `visuals.selection.bg_fill` — which this theme points at the 27 %
    //     CANVAS tint. `on_accent` over that, composited on `palette.panel`,
    //     is a luminance gap of Quiet 44.8 / Airy 28.2 / Dark 52.6 against a
    //     floor of 90.
    //
    // (2) **The ✕ is worse, and it is `DEFECTS.md` D2's exact shape.** Its
    //     `close_rect` is carved OUT of the tab rect immediately below, and it
    //     is drawn `.frame(false)`, so *nothing paints behind it at all*. The
    //     `on_accent` glyph therefore lands on the bare `palette.panel` from
    //     this strip's own background fill: gap **Quiet 17.9 / Airy 5.0 /
    //     Dark 29.1**. Airy is white-on-white to within five levels of
    //     luminance. That is a plate colour used against a background nobody
    //     ever paired it with — which is the definition of D2.
    //
    // Painting the plate first means every child of this rect — label and ✕
    // alike — sits on `accent`, which is the background `on_accent` is NAMED
    // for. The ✕'s own colour choice below then becomes correct rather than
    // being worked around, which is why it is left untouched.
    //
    // ★ Before the split, deliberately: after it, there are two rects and the
    // gap between them is the one the ✕ sits in.
    if selected {
        ui.painter()
            .rect_filled(rect, theme.metrics.corner_radius, theme.palette.accent);
    }

    let close_rect = if tab.closable {
        Rect::from_min_max(
            egui::pos2(rect.right() - CLOSE_WIDTH, rect.top()),
            rect.right_bottom(),
        )
    } else {
        Rect::from_min_max(rect.right_top(), rect.right_bottom())
    };
    let label_rect = Rect::from_min_max(
        rect.left_top(),
        egui::pos2(close_rect.left(), rect.bottom()),
    );

    // R84 — a selected tab is never distinguished by colour alone. Weight is
    // the cue that survives greyscale and colour-vision deficiency, and this
    // project has found colour-fill-only selection to be a recurring blind
    // spot.
    let text = if selected {
        RichText::new(&tab.label)
            .strong()
            .color(theme.palette.on_accent)
    } else {
        RichText::new(&tab.label).color(theme.palette.text_muted)
    };

    let response = ui
        .scope_builder(
            UiBuilder::new()
                .id_salt(("tabstrip-tab", index))
                .max_rect(label_rect)
                .layout(Layout::left_to_right(Align::Center)),
            |ui| {
                ui.set_max_width(label_rect.width());
                ui.add(
                    egui::Button::new(text)
                        .min_size(label_rect.size())
                        .truncate()
                        .selected(selected)
                        // ★ States the fill, so `egui` cannot substitute the
                        // canvas tint OVER the plate painted above. Without
                        // this the wash composites on `accent` and the label's
                        // background becomes a third value again — see the
                        // block above `close_rect` for the arithmetic.
                        .fill(if selected {
                            theme.palette.accent
                        } else {
                            ui.visuals().widgets.inactive.weak_bg_fill
                        })
                        // ★ `click_and_drag`, so the tab can be **reordered**.
                        //
                        // A `Button` senses clicks only, and adding the drag
                        // does not cost the click: `egui` still reports
                        // `clicked()` when the press and release are close
                        // enough together in space and time, which is exactly
                        // the distinction between "I meant this tab" and "I
                        // meant to move this tab". That is how every tab strip
                        // on this desktop behaves and it needs no threshold of
                        // our own.
                        .sense(egui::Sense::click_and_drag()),
                )
            },
        )
        .inner;

    if response.clicked() {
        out.intents.push(TabIntent::Activate(index));
    }
    // ★ `drag_started_by(Primary)`, not `drag_started()`. `egui`'s plain
    // predicate is button-agnostic, so a middle-press that wandered a few
    // pixels before releasing would start a reorder the operator meant as a
    // close — and a right-press one they meant as a context menu.
    if response.drag_started_by(egui::PointerButton::Primary) {
        let id = drag_id(ui);
        ui.ctx()
            .data_mut(|d| d.insert_temp(id, TabDrag { from: index }));
    }
    // Middle click closes. Read from the response rather than from raw input
    // so it is scoped to this tab, and gated on `closable` so a tab that shows
    // no ✕ also does not answer the gesture that means the same thing.
    if tab.closable && response.middle_clicked() {
        out.intents.push(TabIntent::Close(index));
    }

    // The accessible name is published before anything else touches the
    // response, for `dock::tabs`' reason: whatever a caller layers on top, the
    // tab announces itself.
    let response = response.on_hover_text(tab.tooltip.clone());
    let tooltip = tab.tooltip.clone();
    response.widget_info(|| {
        egui::WidgetInfo::selected(egui::WidgetType::SelectableLabel, true, selected, &tooltip)
    });
    // ★ Handed out for the caller's context menu, AFTER the accessible name is
    // published and before anything else can claim the response's one popup id.
    // See [`TabStrip::responses`].
    out.responses.push((index, response));
    if !tab.closable {
        return;
    }
    let close = ui
        .scope_builder(
            UiBuilder::new()
                .id_salt(("tabstrip-close", index))
                .max_rect(close_rect)
                .layout(Layout::left_to_right(Align::Center)),
            |ui| {
                ui.set_max_width(close_rect.width());
                ui.add(
                    egui::Button::new(RichText::new(CLOSE_GLYPH).color(if selected {
                        theme.palette.on_accent
                    } else {
                        theme.palette.text_muted
                    }))
                    .min_size(close_rect.size())
                    .frame(false),
                )
            },
        )
        .inner;
    if close.clicked() {
        out.intents.push(TabIntent::Close(index));
    }
}

/// The "⏷ N more" affordance and the menu behind it.
///
/// A plain menu of every hidden tab. Unlike the dock's, it offers only
/// activation: closing a document you cannot see is not a gesture any
/// application offers, and offering it here would be inventing one.
fn draw_overflow(
    ui: &mut egui::Ui,
    tabs: &[TabItem],
    hidden: usize,
    rect: Rect,
    out: &mut TabStrip,
) {
    let label = plan::overflow_label(hidden);
    ui.scope_builder(
        UiBuilder::new()
            .id_salt("tabstrip-overflow")
            .max_rect(rect)
            .layout(Layout::left_to_right(Align::Center)),
        |ui| {
            ui.menu_button(label, |ui| {
                for (i, tab) in tabs.iter().enumerate() {
                    // Every tab, not only the hidden ones: a menu that changes
                    // its own contents as the strip scrolls is a menu whose
                    // rows move under the pointer.
                    if ui.button(&tab.label).clicked() {
                        out.intents.push(TabIntent::Activate(i));
                        ui.close();
                    }
                }
            });
        },
    );
}

/// The rendered width of `text` in the body style, memoized by `egui`.
fn text_width(ui: &egui::Ui, text: &str) -> f32 {
    if text.is_empty() {
        return 0.0;
    }
    // `TextStyle::Button`, matching `dock::tabs` — and matching what an
    // `egui::Button` actually lays the label out in. A measurement taken at a
    // different text style than the renderer uses aims its boundary assertion
    // at the wrong width, which this project has already paid for once; the
    // finding is filed under egui in the cross-project RAG as
    // a_layout_test_that_measures_at_a_different_textstyle_than_the_renderer_aims_its_boundary_assertion_at_the_wrong_width.
    let font_id = egui::TextStyle::Button.resolve(ui.style());
    ui.ctx().fonts_mut(|fonts| {
        fonts
            .layout_no_wrap(text.to_owned(), font_id, egui::Color32::PLACEHOLDER)
            .size()
            .x
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn items(n: usize) -> Vec<TabItem> {
        (0..n)
            .map(|i| {
                TabItem::new(
                    format!("document-{i}.pdf"),
                    format!("D:/jobs/document-{i}.pdf"),
                )
            })
            .collect()
    }

    /// Render one frame at `width` and report what was drawn.
    fn render(n: usize, width: f32, active: usize) -> TabStrip {
        let ctx = egui::Context::default();
        let tabs = items(n);
        let theme = Theme::default();
        let mut out = TabStrip::default();
        let input = egui::RawInput {
            screen_rect: Some(Rect::from_min_size(
                egui::Pos2::ZERO,
                Vec2::new(width.max(1.0), 200.0),
            )),
            ..Default::default()
        };
        let _ = ctx.run_ui(input, |ui| {
            let rect = Rect::from_min_size(ui.max_rect().min, Vec2::new(width, STRIP_HEIGHT));
            ui.scope_builder(UiBuilder::new().max_rect(rect), |ui| {
                out = strip(ui, &theme, &tabs, active);
            });
        });
        out
    }

    /// **Every tab fits when there is room**, which is the baseline the
    /// overflow assertions below are only meaningful against.
    #[test]
    fn a_strip_that_fits_draws_every_tab_and_no_affordance() {
        let out = render(3, 900.0, 0);
        assert_eq!(out.drawn.len(), 3);
        assert_eq!(out.hidden, 0);
    }

    /// ★ **The route to the hidden tabs survives the tabs.**
    ///
    /// `MODES_AND_PANELS.md` failure mode #8, asserted from this side of the
    /// reuse: the arithmetic is `dock::plan`'s and is tested there, and this
    /// is the check that this module actually *uses* it rather than laying
    /// tabs out itself and reporting a plausible `hidden`.
    ///
    /// The assertion is that the drawn tabs stop short of the strip's right
    /// edge by at least the affordance's width — measured from the published
    /// rectangles, not recomputed.
    #[test]
    fn a_crowded_strip_reserves_room_for_the_overflow_affordance() {
        let width = 300.0;
        let out = render(12, width, 0);
        assert!(out.hidden > 0, "twelve tabs must not fit in 300 pt");
        let rightmost = out
            .drawn
            .iter()
            .map(|(_, r)| r.right())
            .fold(f32::NEG_INFINITY, f32::max);
        assert!(
            rightmost < width,
            "a drawn tab reached {rightmost} of a {width} pt strip, leaving nothing for the \
             route to the {} hidden ones",
            out.hidden
        );
    }

    /// **The active tab is always drawn**, however far down the list it is —
    /// otherwise an operator on document twelve of twelve would be looking at
    /// a strip that does not contain the document they are looking at.
    #[test]
    fn the_active_tab_is_always_among_the_drawn() {
        let out = render(12, 300.0, 11);
        assert!(
            out.drawn.iter().any(|(i, _)| *i == 11),
            "the active tab was scrolled out of its own strip"
        );
    }

    /// **An out-of-range active index does not panic.** Reachable for one
    /// frame while a close is being confirmed, and a panic there would cost
    /// the operator every other document.
    #[test]
    fn an_impossible_active_index_is_survivable() {
        let out = render(3, 900.0, 99);
        assert_eq!(out.drawn.len(), 3);
    }

    /// **Nothing is drawn for an empty list**, which is what lets a caller
    /// hand the strip whatever it has without a guard of its own.
    #[test]
    fn an_empty_strip_draws_nothing() {
        let ctx = egui::Context::default();
        let theme = Theme::default();
        let mut out = TabStrip::default();
        let _ = ctx.run_ui(egui::RawInput::default(), |ui| {
            out = strip(ui, &theme, &[], 0);
        });
        assert!(out.drawn.is_empty());
        assert!(out.intents.is_empty());
    }
}
