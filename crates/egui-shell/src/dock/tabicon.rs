//! # `dock::tabicon` — the glyph before a panel tab's label
//!
//! A panel registered with [`super::model::PanelInfo::with_icon`] draws that
//! icon before its label, through the painter the application hands to
//! [`super::Dock::with_icon_painter`] — the same painter, and the same
//! [`IconRequest`], the ribbon takes, so a panel's tab and the command that
//! opens it can show one glyph. With no key or no painter the tab is text
//! alone and measures exactly as it did.
//!
//! The icon's width is part of the tab's planned width ([`tab_width`]), so
//! the overflow reservation stays true; the glyph's rect is published as
//! `tabicon.dock.<panel>` for driven checks.

use egui::{Atoms, Id, RichText, Vec2};

use crate::ribbon::{IconPainter, IconRequest};

use super::Dock;
use super::ctx::Ctx;
use super::model::PanelId;

impl<'a> Dock<'a> {
    /// Supply the painter for [`super::model::PanelInfo::icon`] keys.
    #[must_use]
    pub fn with_icon_painter(
        mut self,
        painter: &'a mut (impl FnMut(&egui::Painter, &IconRequest<'_>) + 'a),
    ) -> Self {
        self.icons = Some(painter as &'a mut IconPainter<'a>);
        self
    }

    /// Whether panel tabs show their glyph, name, or both.
    #[must_use]
    pub fn with_tab_label(mut self, label: crate::tabshape::TabLabel) -> Self {
        self.tab_label = label;
        self
    }
}

/// The panel's icon key and slot id, when it has one, a painter is installed
/// and the dock draws glyphs.
pub(crate) fn slot(ctx: &Ctx<'_>, panel: &PanelId, tab_id: Id) -> Option<(String, Id)> {
    if !ctx.tab_label.draws_glyph() {
        return None;
    }
    ctx.icons.as_ref()?;
    let info = ctx.registry?.get(panel.as_str())?;
    let key = info.icon.as_ref().filter(|k| !k.trim().is_empty())?;
    Some((key.clone(), tab_id.with("icon")))
}

/// Whether the tab draws its name: always, unless it draws a glyph alone.
pub(crate) fn named(ctx: &Ctx<'_>, panel: &PanelId) -> bool {
    ctx.tab_label
        .draws_text(slot(ctx, panel, Id::NULL).is_some())
}

/// A tab's planned width for a label `label_w` points wide: the glyph, the
/// gap and the label, or the glyph alone.
pub(crate) fn tab_width(ui: &egui::Ui, ctx: &Ctx<'_>, panel: &PanelId, label_w: f32) -> f32 {
    let has_icon = slot(ctx, panel, Id::NULL).is_some();
    if !named(ctx, panel) {
        return ctx.theme.metrics.icon_pts + super::plan::TAB_PADDING;
    }
    let icon = if has_icon {
        ctx.theme.metrics.icon_pts + ui.spacing().icon_spacing
    } else {
        0.0
    };
    super::plan::tab_width(label_w) + icon
}

/// The tab's content: the icon slot, then the label when it has one.
pub(crate) fn atoms<'t>(
    ctx: &Ctx<'_>,
    slot: Option<&(String, Id)>,
    text: Option<RichText>,
) -> Atoms<'t> {
    let mut atoms = Atoms::default();
    if let Some((_, slot_id)) = slot {
        let size = ctx.theme.metrics.icon_pts;
        atoms.push_right(egui::Atom::custom(*slot_id, Vec2::splat(size)));
    }
    if let Some(text) = text {
        atoms.push_right(text);
    }
    atoms
}

/// Paint the glyph into the slot the button laid out and publish its rect.
pub(crate) fn paint(
    ui: &egui::Ui,
    ctx: &mut Ctx<'_>,
    panel: &PanelId,
    slot: Option<(String, Id)>,
    laid_out: &egui::AtomLayoutResponse,
    selected: bool,
) {
    let Some((key, slot_id)) = slot else { return };
    let Some(rect) = laid_out.rect(slot_id) else {
        return;
    };
    if let Some(painter) = ctx.icons.take() {
        let visuals = ui.style().interact(&laid_out.response);
        painter(
            ui.painter(),
            &IconRequest {
                key: &key,
                rect,
                tint: visuals.fg_stroke.color,
                enabled: true,
                selected,
            },
        );
        ctx.icons = Some(painter);
    }
    ctx.reporter
        .report(ui, rect, || super::report::tab_icon(panel));
}
