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
//! reservation**. `MODES_AND_PANELS.md` Part 2 failure mode #8 — *"past a
//! handful of tabs the overflow button itself gets hidden, leaving no route to
//! the hidden tabs"* — applies to a strip of twelve open drawings as much as to
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
//!
//! Design and rationale: `docs/modules/egui-shell/tabstrip/mod.md`.

use egui::{Align, Layout, Rect, RichText, UiBuilder, Vec2};

use crate::dock::plan;
use crate::theme::Theme;

/// **The height of the strip**, in logical points.
pub const STRIP_HEIGHT: f32 = 26.0;

/// The width reserved inside each tab for the close control.
const CLOSE_WIDTH: f32 = 16.0;

/// The close glyph.
const CLOSE_GLYPH: &str = "\u{00d7}"; // ui-text-exempt: a glyph, not a sentence

/// One tab.
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
#[derive(Clone, Copy, Debug, PartialEq)]
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
    /// # `gap` is a BOUNDARY, not a destination index
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
    /// **The tab at `from` was dragged off the strip and released at `at`.**
    ///
    /// Raised instead of [`Self::Reorder`] when the release is farther from
    /// the strip than one strip-height above or below it, or beyond its ends.
    /// `at` is in this window's points and may lie outside the window: the
    /// platform keeps reporting the pointer to the window that holds the
    /// press. What a tab dropped elsewhere becomes — a window of its own,
    /// another window's tab — is the caller's to decide.
    DragOut {
        /// The tab being dragged.
        from: usize,
        /// Where the pointer was released, in this window's points.
        at: egui::Pos2,
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
    /// Published rather than left to be derived. A harness that computes a
    /// tab's position from an index and a width can be wrong in the same
    /// direction as the code under test — `D:\dev\rag\egui\a_ui_rect_change_log_produces_confident_wrong_failures_in_BOTH_directions.md`
    /// and the *"do not compute a coordinate the application could publish"*
    /// rule. Hidden tabs are absent from this list, which is itself the fact a
    /// check about overflow wants.
    pub drawn: Vec<(usize, Rect)>,
    /// How many tabs did not fit and are reachable only through the overflow
    /// menu.
    pub hidden: usize,
    /// **Each drawn tab's own `Response`**, handed out so the caller can attach
    /// a context menu to it.
    ///
    /// Handed out rather than used here, and that is a hard constraint rather
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
    /// The tab being dragged while the pointer is off the strip, where a
    /// release raises [`TabIntent::DragOut`]. `None` otherwise.
    pub dragging_out: Option<usize>,
}

/// **Draw the strip.**
#[must_use]
pub fn strip(ui: &mut egui::Ui, theme: &Theme, tabs: &[TabItem], active: usize) -> TabStrip {
    let mut out = TabStrip::default();
    let rect = ui.max_rect();
    if tabs.is_empty() || rect.width() <= 0.0 {
        return out;
    }

    ui.painter().rect_filled(rect, 0.0, theme.palette.surface);
    ui.painter().hline(
        rect.x_range(),
        rect.bottom() - 0.5,
        egui::Stroke::new(1.0, theme.palette.outline),
    );

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

    // **Which tab the pointer is over, resolved GEOMETRICALLY.**
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
    // A rectangle and a pointer position are facts that do not care who owns the
    // interaction, which is what makes them the right instrument here — the same
    // reason a drop target is resolved from `pointer_latest_pos()` against a
    // tile rect rather than from the tile's own response.
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
    settle_reorder(ui, theme, rect, tabs, &mut out);

    out
}

/// **A tab drag in flight**, between frames.
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
fn settle_reorder(
    ui: &mut egui::Ui,
    theme: &Theme,
    strip_rect: Rect,
    tabs: &[TabItem],
    out: &mut TabStrip,
) {
    let id = drag_id(ui);
    let Some(drag) = ui.ctx().data(|d| d.get_temp::<TabDrag>(id)) else {
        return;
    };
    let released = ui
        .ctx()
        .input(|i| i.pointer.button_released(egui::PointerButton::Primary));
    let Some(pointer) = ui.ctx().pointer_latest_pos() else {
        // No position to land at: a release ends the drag with no intent,
        // so it cannot outlive the button.
        if released {
            ui.ctx().data_mut(|d| d.remove_temp::<TabDrag>(id));
        }
        return;
    };
    if is_off_strip(strip_rect, pointer) {
        out.dragging_out = Some(drag.from);
        if let Some(tab) = tabs.get(drag.from) {
            paint_lifted(ui, theme, &tab.label, pointer);
        }
        ui.ctx().set_cursor_icon(egui::CursorIcon::Grabbing);
        if released {
            ui.ctx().data_mut(|d| d.remove_temp::<TabDrag>(id));
            out.dragging_out = None;
            out.intents.push(TabIntent::DragOut {
                from: drag.from,
                at: pointer,
            });
        }
        return;
    }

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

    // The release is read from RAW POINTER INPUT, not from the tab's own
    // `Response`.
    //
    // A drag begun on a tab may end anywhere — past the last tab, over the
    // canvas, off the window — and a `Response` only reports releases inside
    // the widget that produced it. Reading the input means a drag always ends,
    // which is the property that stops a half-finished drag surviving into the
    // next frame as a caret nobody can get rid of.
    if released {
        ui.ctx().data_mut(|d| d.remove_temp::<TabDrag>(id));
        out.reordering = None;
        out.intents.push(TabIntent::Reorder {
            from: drag.from,
            gap,
        });
    }
}

/// How thick the reorder caret is drawn.
const CARET_PTS: f32 = 2.0;

/// Whether a tab drag at `pointer` has left the strip: beyond its ends, or
/// more than one strip-height above or below it. The vertical slack keeps a
/// sloppy sideways drag a reorder.
fn is_off_strip(strip_rect: Rect, pointer: egui::Pos2) -> bool {
    !strip_rect
        .expand2(Vec2::new(0.0, strip_rect.height()))
        .contains(pointer)
}

/// The dragged tab's label, drawn at the pointer above everything else while
/// it is off the strip, so the drag visibly carries the tab.
fn paint_lifted(ui: &egui::Ui, theme: &Theme, label: &str, pointer: egui::Pos2) {
    let painter = ui.ctx().layer_painter(egui::LayerId::new(
        egui::Order::Tooltip,
        drag_id(ui).with("lifted"), // ui-text-exempt: an id, never displayed
    ));
    let font = egui::TextStyle::Button.resolve(ui.style());
    let galley = painter.layout_no_wrap(label.to_owned(), font, theme.palette.text);
    let pad = Vec2::new(plan::TAB_PADDING, 4.0);
    let rect = Rect::from_min_size(pointer + Vec2::new(12.0, 8.0), galley.size() + pad * 2.0);
    painter.rect(
        rect,
        4.0,
        theme.palette.surface,
        egui::Stroke::new(1.0, theme.palette.accent),
        egui::StrokeKind::Inside,
    );
    painter.galley(rect.min + pad, galley, theme.palette.text);
}

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
    // The close control is laid out FIRST and the label takes what is left.
    //
    // The other order is the obvious one and it is wrong: a label allowed to
    // claim the whole tab pushes the ✕ out of the rect on exactly the tabs
    // that are too narrow — which is every tab, once enough documents are
    // open. Reserving the control and truncating the label is the same
    // discipline the overflow affordance gets one level up, applied inside the
    // tab.
    // The tab's body covers label and close control alike; it is painted
    // into this slot once the label's hover is known (`crate::tabshape`).
    let backdrop = ui.painter().add(egui::Shape::Noop);

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

    // R84: the joined outline and the accent rule carry selection; `.strong()`
    // only changes colour in `egui`, so it is a third cue, not a second.
    let text = if selected {
        RichText::new(&tab.label).strong().color(theme.palette.text)
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
                        .frame(false)
                        .selected(selected)
                        // `click_and_drag`, so the tab can be **reordered**.
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
    ui.painter().set(
        backdrop,
        crate::tabshape::body(
            &theme.palette,
            theme.metrics.corner_radius,
            theme.palette.panel,
            rect,
            selected,
            response.hovered(),
        ),
    );

    if response.clicked() {
        out.intents.push(TabIntent::Activate(index));
    }
    // `drag_started_by(Primary)`, not `drag_started()`. `egui`'s plain
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
    // Handed out for the caller's context menu, AFTER the accessible name is
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
                        theme.palette.text
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

    /// A drag leaves the strip only past one strip-height of vertical slack,
    /// or beyond its ends.
    #[test]
    fn a_drag_leaves_the_strip_past_its_slack() {
        let strip = Rect::from_min_size(egui::pos2(0.0, 100.0), Vec2::new(800.0, 30.0));
        assert!(!is_off_strip(strip, egui::pos2(400.0, 115.0)));
        assert!(!is_off_strip(strip, egui::pos2(400.0, 155.0)));
        assert!(!is_off_strip(strip, egui::pos2(400.0, 75.0)));
        assert!(is_off_strip(strip, egui::pos2(400.0, 170.0)));
        assert!(is_off_strip(strip, egui::pos2(400.0, 60.0)));
        assert!(is_off_strip(strip, egui::pos2(-5.0, 115.0)));
        assert!(is_off_strip(strip, egui::pos2(805.0, 115.0)));
    }

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

    /// **The route to the hidden tabs survives the tabs.**
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
