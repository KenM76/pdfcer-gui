//! # `dialogs::diagnostics` — the render report, off the status bar at last
//!
//! The dispatch target for `tools.render_diagnostics`, on **Tools ▸
//! Diagnostics**.
//!
//! ## ★ Why this exists, and why it is a dialog rather than a second status
//! line
//!
//! The command was registered, drawn with a glyph, given a group of its own
//! and gated on `doc.open` — and had **no dispatch arm** until 2026-08-15,
//! which `shell::commands::reach` called the least defensible entry on its
//! whole list *because the work behind it was already done*: the renderer has
//! been producing this report since S0 and the status bar has been showing a
//! one-line summary of it since S2.
//!
//! `shell::manifest::tools`' header is the only argument on record for the
//! command, and it is an argument about **placement** rather than about
//! capability:
//!
//! > It is currently a run of text in the status bar. That surface is for the
//! > controls a user touches constantly, and a diagnostic readout is neither a
//! > control nor constant — it is a thing you go and look at when something is
//! > wrong. Moving it here also gives it room to be more than one line.
//!
//! Three requirements fall straight out of that sentence, and this file is
//! them:
//!
//! 1. **A thing you go and look at.** Opened deliberately, holding one
//!    question's worth of answers, forgotten when closed — which is
//!    [`super::DialogsState`]'s own definition of a dialog as against a panel.
//!    A panel would keep state across documents and sit in the dock competing
//!    for width with the Objects list; nobody wants a render census permanently
//!    mounted.
//! 2. **When something is wrong.** So it opens on demand and never on its own.
//!    `view.app_initiative`'s specified default is **Never** — pdfcer may not
//!    float a surface over the canvas unasked — and a diagnostic window that
//!    appeared because a page happened to substitute a glyph would be the
//!    clearest possible violation of it.
//! 3. **Room to be more than one line.** The bar gets one elided line under
//!    **R128**; this gets the findings one per row, plus the three
//!    measurements of the render itself, plus the two counters the bar
//!    deliberately excludes.
//!
//! ## ★ The status bar keeps its line, and that is not a duplicate
//!
//! Both surfaces read the **same** derivation —
//! [`crate::app::status::notes::findings`] — so they cannot disagree about what
//! a raster compromised on. What differs is only the room: the bar answers *is
//! anything worth looking at?* at a glance, this answers *what, exactly, and
//! how expensive was it?*. `DEFECTS.md`'s "Not defects" table settled the
//! prominence question already (*"Excellent information, wrong prominence"*) —
//! the fix was to demote it, not to delete it, and demoting it is what makes a
//! deliberate route to the full report worth having.
//!
//! ## What it shows that the bar cannot
//!
//! | | bar | here |
//! |---|---|---|
//! | the findings | one elided line, joined by `·` | one row each, in the same order |
//! | how long the render took | — | `RenderedPixels::elapsed`, measured around the rasterization alone |
//! | the raster scale and pixel size | — | from `RenderKey` and the uploaded texture |
//! | `tolerated` / `compat_skipped` | excluded on editorial grounds | shown, with a sentence saying they are not faults |
//!
//! The duration and the scale are drawn **together**, deliberately.
//! `HANDOFF.md` §10: *"~99 % of render cost is resolution-independent on dense
//! CAD. A small thumbnail is not a cheap thumbnail. A 1×1 point region costs
//! 691 ms."* A duration on its own invites the operator to zoom out and expect
//! relief that will not come.
//!
//! ## Document-scoped, and it closes with the document
//!
//! It describes *this page of this file*. A window left up over a closed
//! document would be reporting measurements of a raster that no longer exists,
//! which is the same reason print is document-scoped and About is not.
//!
//! ## Why it pushes no `Action`
//!
//! [`super::DialogsState`]'s rule: the funnel exists for changes to **document**
//! state. This one reads a texture that has already been uploaded and renders
//! nothing new. It has nothing to undo, nothing to order against and nothing
//! that could alias.

use egui_shell::theme::Theme;

use crate::app::state::OpenDoc;
use crate::text::diagnostics as t;

/// Named region: the whole dialog body.
///
/// Matched literally by `tools/ui-verify`, so renaming it silently un-aims
/// whatever check was measuring it.
const REGION_BODY: &str = "dialog:render-diagnostics"; // ui-text-exempt: trace region name, never displayed

/// The Render-diagnostics dialog. Its existence is its "open" state — see
/// [`super::DialogsState`]'s header for why there is no `open: bool`.
///
/// It holds **no configuration**, exactly as [`super::about::AboutDialog`]
/// does, and for the same reason: everything it shows is read from the open
/// document's current texture on the frame it is drawn, so there is nothing for
/// the operator to change and nothing for closing it to forget.
///
/// ★ Reading live rather than snapshotting on open is a decision. A snapshot
/// would freeze the report of whichever raster happened to be current when the
/// command was pressed, and the operator's very next act while diagnosing is to
/// change the zoom or the page — at which point a frozen window would be
/// describing a picture that is no longer on the canvas while looking exactly
/// like one that is. The title says *the picture currently on the canvas*, and
/// this is what makes that true.
#[derive(Debug, Default)]
pub struct DiagnosticsDialog {
    /// Set by the Close button, consumed by [`Self::show`].
    ///
    /// The same two-step every dialog here uses: a widget inside the window's
    /// closure cannot drop the state it is being drawn from, so it records the
    /// request and the caller acts on it after the closure returns.
    close_requested: bool,
}

impl DiagnosticsDialog {
    /// Build the dialog.
    ///
    /// Takes nothing, because it snapshots nothing — see the type's docs.
    #[must_use]
    pub(super) fn open() -> Self {
        Self::default()
    }

    /// Draw one frame of the dialog. Returns `false` when it should close.
    pub(super) fn show(&mut self, ctx: &egui::Context, doc: &OpenDoc) -> bool {
        // ★ ITS OWN OS WINDOW as of 2026-08-21, and this dialog is the one
        // that most wanted it: it is read *while* zooming and panning the
        // document it describes, so a window locked inside the application's
        // frame necessarily covered the thing being diagnosed. Off on a second
        // monitor is where it belongs.
        let (frame, ()) = crate::dialogs::host::Host::new(
            "render-diagnostics", // ui-text-exempt: a viewport key, never displayed.
            t::title(),
            egui::vec2(520.0, 380.0),
            egui::vec2(380.0, 240.0),
        )
        .show(ctx, |ui| self.body(ui, doc));
        !frame.closed && !std::mem::take(&mut self.close_requested)
    }

    /// Everything inside the window.
    fn body(&mut self, ui: &mut egui::Ui, doc: &OpenDoc) {
        let theme = Theme::of(ui.ctx());
        crate::diag::ui_rect(REGION_BODY, ui.max_rect());

        // ★ The one state that is not a report: a document is open and nothing
        // has been rasterized. Reachable before the first render and after a
        // render failure, which is exactly when an operator is most likely to
        // reach for this command — so it says which of the two nothings this is
        // rather than drawing an empty window that reads as a broken control.
        let Some(texture) = doc.page_texture.as_ref() else {
            ui.label(t::nothing_drawn());
            ui.add_space(12.0);
            self.footer(ui);
            return;
        };

        // The page the TEXTURE is of, not `doc.view.page_index`. They differ
        // for exactly as long as a render is in flight after a page change,
        // and during that window the canvas is still showing the old raster —
        // so the key's page is the one that describes what is on screen. The
        // number is one-based because it is shown to an operator; every index
        // in this crate is not.
        ui.label(
            egui::RichText::new(t::subject(texture.key.page() + 1)).color(theme.palette.text_muted),
        );
        ui.add_space(8.0);

        let size = texture.texture.size();
        ui.label(t::took(texture.elapsed.as_millis()));
        ui.label(t::raster(texture.key.raster_scale(), size[0], size[1]))
            .on_hover_text(t::raster_tooltip());

        ui.add_space(12.0);
        ui.separator();
        ui.add_space(6.0);

        ui.label(egui::RichText::new(t::findings_heading()).heading());
        ui.add_space(6.0);

        // Scrolled, and the scroll starts here rather than around the whole
        // body: the measurements above are the two lines that must stay legible
        // without scrolling, and a scroll area over everything would let them be
        // dragged out of sight. The `max` floor guards the same negative-height
        // trap `about::AboutDialog::body` records — a negative `max_height` is
        // not an error, it is a scroll area that silently draws nothing.
        const FOOTER_RESERVE: f32 = 64.0;
        const LIST_FLOOR: f32 = 48.0;
        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .max_height((ui.available_height() - FOOTER_RESERVE).max(LIST_FLOOR))
            .show(ui, |ui| {
                let findings = crate::app::status::notes::findings(&texture.diagnostics);
                if findings.is_empty() {
                    ui.label(t::clean());
                } else {
                    for finding in &findings {
                        ui.label(finding);
                    }
                }
                ui.add_space(10.0);
                // The two counters the status bar deliberately excludes. Muted
                // rather than plain, because they are context for the list
                // above rather than members of it — the same role About uses
                // for a supporting line, and not `.strong()`, which
                // `DEFECTS.md` D11 records as unusable in this theme.
                ui.label(
                    egui::RichText::new(t::absorbed(
                        texture.diagnostics.tolerated,
                        texture.diagnostics.compat_skipped,
                    ))
                    .color(theme.palette.text_muted),
                );
            });

        self.footer(ui);
    }

    /// The separator and the Close button.
    ///
    /// Its own function because both bodies above need it and the early return
    /// for "nothing drawn" must not be a window with no way out but the title
    /// bar's cross.
    fn footer(&mut self, ui: &mut egui::Ui) {
        ui.separator();
        ui.horizontal(|ui| {
            if ui.button(t::close()).clicked() {
                self.close_requested = true;
            }
        });
    }
}
