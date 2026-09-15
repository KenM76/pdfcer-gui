//! The per-menu render context, and the one seam a menu needs that the
//! ribbon's does not.
//!
//! # What is borrowed rather than redefined
//!
//! Almost everything. A menu paints icons through
//! [`crate::ribbon::IconPainter`] and [`crate::ribbon::IconRequest`], and
//! publishes rectangles through [`crate::ribbon::report::Reporter`] and
//! [`crate::ribbon::report::RectSink`] — **the same types the ribbon
//! uses**, not look-alikes.
//!
//! That is not thrift. An application supplies exactly one icon set and
//! wires exactly one harness sink; if the menu asked for its own callback
//! types, every application would carry two adapters whose only job was to
//! forward, and the day the two signatures drifted apart would be the day
//! a menu started drawing different icons from the ribbon.
//!
//! Those types are also honestly general: an [`crate::ribbon::IconRequest`]
//! names a key, a rectangle, a tint and an enabled flag, and there is
//! nothing about a ribbon in any of them.
//!
//! # The one thing that could *not* be borrowed
//!
//! [`crate::ribbon::CustomItem`] carries `tab` and `group` — the two
//! coordinates that say where in a ribbon a custom control sits. A menu
//! has neither, and passing `tab: "canvas.object", group: "menu"` to reuse
//! the type would be a lie encoded in a field name: an application
//! matching on `tab` would be matching on something that is not a tab.
//!
//! So [`MenuCustomItem`] exists, with the one coordinate a menu actually
//! has. It is a *different question*, not a second answer to the same one
//! — which is the test this crate applies before allowing a parallel type
//! at all.

use crate::commands::HandlerToken;
use crate::ribbon::IconPainter;
use crate::ribbon::report::Reporter;
use crate::theme::Theme;

/// One [`crate::manifest::Item::Custom`] in a menu, handed back to the
/// application to draw.
///
/// The menu reserves a row in its vertical flow, hands over `kind` and
/// `payload`, and gets out of the way — the same contract the ribbon's
/// band offers, and for the same reason: the alternative is an item
/// vocabulary that grows a variant per widget an application happens to
/// want, which is the road by which a reusable shell stops being reusable.
#[derive(Debug, Clone, Copy)]
pub struct MenuCustomItem<'a> {
    /// The application-defined kind, e.g. `"colour_swatch"`.
    pub kind: &'a str,
    /// The application-defined payload, if the document carried one.
    pub payload: Option<&'a str>,
    /// The context id of the menu this row is in, so one renderer can
    /// serve a kind that appears in more than one menu.
    pub context: &'a str,
}

/// Draws one custom menu row.
///
/// Returns a token if the operator invoked something, so an
/// application-drawn control reports through the same channel as a
/// command — see [`super::render`]'s header on the seam.
pub type MenuCustomRenderer<'a> =
    dyn FnMut(&mut egui::Ui, &MenuCustomItem<'_>) -> Option<HandlerToken> + 'a;

/// Everything one menu body needs while it is being drawn.
///
/// Not `Clone`, not stored: it borrows the application's callbacks for the
/// duration of one popup body and is dropped at the end of it.
pub(crate) struct Ctx<'a> {
    /// The context id, for report names, disclosures and widget ids.
    ///
    /// Owned rather than borrowed, and that is a lifetime decision rather
    /// than a careless one: `icons` and `custom` fix `'a` to the
    /// application's callbacks, and a `&'a str` here would force every
    /// caller's context id to live that long too — ruling out
    /// `&format!("pages.thumbnail.{n}")`, which is exactly how a
    /// per-item context id gets built. One `String` per menu *body*, i.e.
    /// per frame the menu is actually open, is not a cost worth that.
    pub context: String,
    /// The look in force, read from the `egui` context once.
    pub theme: Theme,
    /// Where drawn rectangles are published, if anywhere.
    pub reporter: Reporter<'a>,
    /// How to paint an icon, if the application supplied a painter.
    pub icons: Option<&'a mut IconPainter<'a>>,
    /// How to draw a custom row, if the application supplied a renderer.
    pub custom: Option<&'a mut MenuCustomRenderer<'a>>,
    /// The base `egui::Id` every menu widget id is derived from.
    pub base_id: egui::Id,
    /// The tokens the operator invoked, in the order the rows were drawn.
    pub invoked: Vec<HandlerToken>,
}

impl Ctx<'_> {
    /// An `egui::Id` for a menu widget, derived from the base id.
    ///
    /// Derived rather than auto-generated for the reason
    /// [`crate::ribbon::ctx`] gives: `egui` keeps hover and focus state per
    /// id, and an id that shifts when a row above it is filtered out by
    /// the no-placeholders rule produces a control that loses focus when
    /// the *selection* changes — which reads as a focus bug rather than as
    /// an id bug and is very hard to attribute.
    pub(crate) fn id(&self, kind: &str, key: &str) -> egui::Id {
        self.base_id.with(kind).with(key)
    }

    /// Record that the operator invoked something.
    pub(crate) fn invoke(&mut self, token: HandlerToken) {
        self.invoked.push(token);
    }
}

impl std::fmt::Debug for Ctx<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Ctx")
            .field("context", &self.context)
            .field("preset", &self.theme.preset)
            .field("reporter", &self.reporter)
            .field("icons", &self.icons.is_some())
            .field("custom", &self.custom.is_some())
            .field("invoked", &self.invoked)
            .finish()
    }
}
