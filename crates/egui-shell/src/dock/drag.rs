//! `dock::drag` — dragging a tab, and where releasing it would put the panel.
//!
//! ## The gesture
//!
//! Press a tab, move, and the tab strip shows an insertion caret at the
//! boundary the release would use. Release, and the panel lands there. A
//! press that does not move is still a click, so activating a tab is
//! unchanged — which is why the tab senses `click_and_drag` and not
//! [`egui::Sense::drag`], and why every predicate here names
//! [`egui::PointerButton::Primary`] rather than using `egui`'s
//! button-agnostic `drag_started()`.
//!
//! ## ★ The drag lives in `egui::Memory`, keyed on the dock, not on the strip
//!
//! It has to outlive a frame, and nothing else in the dock does — the layout
//! is a snapshot, the geometry is rebuilt, [`super::ctx::Ctx`] is dropped. The
//! key is the dock's `id_salt` rather than the tab strip's `Ui` id, so a drag
//! that begins in one compartment is still readable while a different
//! compartment is being drawn. Two docks in one window have two salts and
//! therefore two independent drags.
//!
//! ## ★★ The preview is proposed in the strip and settled centrally
//!
//! [`preview`] runs inside the origin strip's draw, because a boundary has no
//! position until the tabs are laid out. [`settle`] runs once, at the end of
//! [`super::Dock::show`], and reads the release from **raw pointer input**
//! rather than from the tab's own `Response`.
//!
//! That split is the property that matters: a `Response` only reports releases
//! inside the widget that produced it, so a drag that ends over the canvas, or
//! whose strip stopped being drawn mid-gesture — the side was collapsed, the
//! window narrowed until the tab overflowed — would never end, and would
//! survive into the next frame as a caret nobody can get rid of. Reading raw
//! input means a drag always ends, whatever happened to the thing it started
//! on.

use egui::Rect;

use super::ctx::{Ctx, Intent};
use super::geometry::StackAddr;
use super::model::{PanelAddress, PanelId};

/// A tab drag in flight, between frames.
#[derive(Clone, Debug, PartialEq)]
struct TabDrag {
    /// The panel the press landed on.
    panel: PanelId,
    /// Where it was when the press landed.
    from: PanelAddress,
}

/// **A tab drag in flight, and where releasing would put it.**
///
/// Published on [`super::DockFrameReport`] so an application can say it in
/// words and a driven check can assert on it: a hairline between two
/// near-identical tab labels is precise and not checkable.
#[derive(Clone, Debug, PartialEq)]
pub struct TabDragPreview {
    /// The panel being dragged.
    pub panel: PanelId,
    /// Where it started.
    pub from: PanelAddress,
    /// The boundary a release would insert it at, in its own stack.
    ///
    /// A boundary, not a destination index — `0` is before the first tab and
    /// `tabs.len()` is after the last. See [`super::geometry::DockGeometry::gap_in`].
    pub gap: usize,
}

impl TabDragPreview {
    /// **Whether releasing here would actually move the panel.**
    ///
    /// False at the two boundaries against the dragged tab's own edges: they
    /// are legal drops and they permute nothing. Derived rather than published
    /// as a field of its own, because it is a function of two fields already on
    /// this struct, and a stored copy is a second answer that can disagree.
    #[must_use]
    pub const fn lands(&self) -> bool {
        self.gap != self.from.tab && self.gap != self.from.tab + 1
    }
}

/// The drag's key in `egui::Memory`. See the module header.
fn key(ctx: &Ctx<'_>) -> egui::Id {
    ctx.id_salt.with("dock-tab-drag") // ui-text-exempt: an id, never displayed
}

/// Begin a drag, called from the tab that was pressed.
pub(super) fn begin(ui: &egui::Ui, ctx: &Ctx<'_>, panel: &PanelId, from: PanelAddress) {
    let id = key(ctx);
    let drag = TabDrag {
        panel: panel.clone(),
        from,
    };
    ui.ctx().data_mut(|d| d.insert_temp(id, drag));
}

/// **Resolve the drag against one strip and draw its caret**, if that strip is
/// the one the drag began in.
///
/// Called at the end of a tab bar's layout, after every tab it drew has been
/// recorded in [`Ctx::geometry`] — the caret's position is read from those
/// rects rather than computed from a width, for the rule
/// [`super::geometry::DockGeometry::push_tab`] carries.
///
/// Cross-compartment drops are not this function: a drag over a *different*
/// strip proposes nothing and draws nothing, which is honest — there is no
/// grammar behind it yet, and a caret that promised an outcome the release
/// would not deliver is the disclosure failure this project names failure
/// mode #2.
///
/// ## ★ The band, and why it is not the strip's own rectangle
///
/// A reorder is resolved by the pointer's **x alone**, so the pointer's y
/// decides only whether it is a reorder at all. Bounding it by the strip
/// exactly would make the caret flicker out under the few points of vertical
/// wander any horizontal drag has; not bounding it at all would make a drag
/// pulled down into the document silently reorder the strip it left, which is
/// what every application of this class treats as a tear-out instead.
/// [`REORDER_SLACK_PTS`] is the tolerance between those, and the branch this
/// function declines is where tearing a panel out will attach.
pub(super) fn preview(ui: &mut egui::Ui, ctx: &mut Ctx<'_>, addr: StackAddr, strip: Rect) {
    let Some(drag) = ui.ctx().data(|d| d.get_temp::<TabDrag>(key(ctx))) else {
        return;
    };
    if StackAddr::from(drag.from) != addr {
        return;
    }
    let Some(pointer) = ui.ctx().pointer_latest_pos() else {
        return;
    };
    if pointer.y < strip.top() - REORDER_SLACK_PTS || pointer.y > strip.bottom() + REORDER_SLACK_PTS
    {
        return;
    }
    let Some(gap) = ctx.geometry.gap_in(addr, pointer) else {
        return;
    };

    // The caret's x: the left edge of the tab that would be pushed rightwards,
    // or the right edge of the last one when the boundary is past the end.
    let x = ctx.geometry.tab_rect(addr.tab(gap)).map_or_else(
        || {
            ctx.geometry
                .tabs_of(addr)
                .last()
                .map_or(strip.left(), |(_, r)| r.right())
        },
        |r| r.left(),
    );
    // Held inside the strip by its own half-width. The outermost boundaries sit
    // on the strip's edges, and a caret centred on one of those is half clipped
    // away — the two boundaries in every strip that would be drawn at half the
    // weight of the rest, exactly where the operator is least able to tell a
    // thin marker from none. `hi` is floored at `lo` because a strip narrower
    // than the caret would otherwise invert the range and panic.
    let half = CARET_PTS / 2.0;
    let lo = strip.left() + half;
    let hi = (strip.right() - half).max(lo);
    let x = x.clamp(lo, hi);
    let caret = Rect::from_min_max(
        egui::pos2(x - half, strip.top()),
        egui::pos2(x + half, strip.bottom()),
    );
    // Dimmed at the two boundaries against the dragged tab's own edges. They
    // are legal drops, so refusing them would be a lie; they permute nothing,
    // so a full-strength caret promises a move that will not happen. The page
    // rail, the page grid, the bookmark tree and the field list all say it this
    // way, and this is the fifth.
    let landing = TabDragPreview {
        panel: drag.panel.clone(),
        from: drag.from,
        gap,
    };
    let ink = if landing.lands() {
        ctx.theme.palette.accent
    } else {
        ctx.theme.palette.accent.gamma_multiply(CARET_DIMMED)
    };
    ui.painter().line_segment(
        [caret.center_top(), caret.center_bottom()],
        egui::Stroke::new(CARET_PTS, ink),
    );
    ui.ctx().set_cursor_icon(egui::CursorIcon::Grabbing);
    // Published as a region, not left to the pixels: the caret exists only
    // while the pointer is down, so nothing captured after the gesture can see
    // it, and a build that reorders correctly and marks nothing is the half of
    // this feature that fails silently.
    ctx.reporter.report(ui, caret, || {
        super::report::tab_caret(addr.side, addr.column, addr.stack)
    });

    ctx.tab_drag = Some(landing);
}

/// **End a drag that the operator has released**, wherever they released it.
///
/// Called once, after both sides have drawn. Clearing the memory is
/// unconditional on release; raising the intent is not, because a drag with no
/// boundary this frame — its strip was not drawn, or the pointer left the band
/// [`preview`] describes — has nothing to name and must land nowhere rather
/// than land at a guess.
pub(super) fn settle(ui: &egui::Ui, ctx: &mut Ctx<'_>) {
    let id = key(ctx);
    if ui.ctx().data(|d| d.get_temp::<TabDrag>(id)).is_none() {
        return;
    }
    if !ui
        .ctx()
        .input(|i| i.pointer.button_released(egui::PointerButton::Primary))
    {
        return;
    }
    // `remove`, not `remove_temp`: the latter wants `Default`, and a default
    // `TabDrag` would be a drag of the empty panel from column zero — a value
    // that means something and is never true.
    ui.ctx().data_mut(|d| d.remove::<TabDrag>(id));
    let Some(preview) = ctx.tab_drag.take() else {
        return;
    };
    ctx.intents.push(Intent::ReorderTab {
        stack: StackAddr::from(preview.from),
        from: preview.from.tab,
        gap: preview.gap,
    });
}

/// How thick the insertion caret is drawn — the weight the document strip and
/// the page rail already use, so the three read as one affordance.
const CARET_PTS: f32 = 2.0;

/// How far above or below its tab strip a drag may wander and still be a
/// reorder of that strip — one strip height. See [`preview`].
const REORDER_SLACK_PTS: f32 = super::plan::TAB_BAR_HEIGHT;

/// How far the caret's ink is knocked back at a boundary that changes nothing —
/// the value the page rail, the page grid, the bookmark tree and the field list
/// already use, so the five read as one affordance.
const CARET_DIMMED: f32 = 0.35;
