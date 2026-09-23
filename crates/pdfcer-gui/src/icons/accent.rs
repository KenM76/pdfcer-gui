//! # `icons::accent` — the optional two-colour icon set
//!
//! With **Settings › Appearance › Coloured icons** on, an icon draws its
//! outline in the theme foreground as always and one meaningful part in a
//! muted hue, the way Office and SOLIDWORKS toolbars do: the plus on *New* in
//! green, the cross on *Close* in red, the arrows on *Undo* in blue, the
//! highlighter's band in amber. Off — the default — every icon is the plain
//! single-colour mask it has always been, drawn by the same code path.
//!
//! ## Contract
//!
//! - [`accent`] names, per icon, the hue role and the shapes (indices in the
//!   asset's paint order) that take it. An icon with no entry never changes.
//! - The hues are the theme's [`egui_shell::theme::IconAccents`]; this module
//!   chooses roles, never colours.
//! - The accent is part of the glyph, not a state: hover, press and the
//!   selected toggle's ink recolour the outline only. A disabled control
//!   fades both parts together, because the accent inherits the tint's alpha.
//! - The switch reaches painters through the `egui::Context` ([`sync`],
//!   [`enabled`]), because the ribbon's painter is handed a `Painter` and
//!   nothing else.

use egui_shell::theme::IconAccents;

use super::Icon;
use super::cache::Baked;

/// Which of the theme's four accent roles a glyph's accent takes.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Hue {
    /// Arrows, handles, the working part of a tool.
    Primary,
    /// Adds, creates or confirms.
    Affirm,
    /// Removes, closes or strikes out.
    Remove,
    /// Marks or keeps.
    Mark,
}

impl Hue {
    /// This role's colour in `accents`.
    #[must_use]
    pub const fn colour(self, accents: IconAccents) -> egui::Color32 {
        match self {
            Self::Primary => accents.primary,
            Self::Affirm => accents.affirm,
            Self::Remove => accents.remove,
            Self::Mark => accents.mark,
        }
    }
}

/// The accent of `icon`: its hue role and the indices, in the asset's paint
/// order, of the shapes drawn in it. `None` for an icon that stays one colour.
///
/// Indices are into [`super::IconArt`]'s shapes; the test below holds every
/// one in range, so an edited asset that loses a shape fails `cargo test`
/// rather than silently dropping its accent.
#[must_use]
pub const fn accent(icon: Icon) -> Option<(Hue, &'static [usize])> {
    Some(match icon {
        Icon::ApplyRedactions | Icon::Accept => (Hue::Affirm, &[0]),
        Icon::CheckBox | Icon::FinishShape | Icon::AddText => (Hue::Affirm, &[1]),
        Icon::RenderDiagnostics | Icon::FormField | Icon::InsertImage => (Hue::Affirm, &[2]),
        Icon::New | Icon::InsertPages => (Hue::Affirm, &[2, 3]),
        Icon::Permissions => (Hue::Affirm, &[2, 4]),
        Icon::Open | Icon::FontFolders | Icon::EditText => (Hue::Mark, &[0]),
        Icon::Locked | Icon::TextSticky => (Hue::Mark, &[0, 1]),
        Icon::Markup | Icon::ShapeHighlight | Icon::Paste => (Hue::Mark, &[1]),
        Icon::Encrypt => (Hue::Mark, &[2, 3]),
        Icon::SaveAs => (Hue::Mark, &[3]),
        Icon::Bookmarks | Icon::Layers | Icon::Signatures | Icon::Combine | Icon::ZoomSelection => {
            (Hue::Primary, &[0])
        }
        Icon::Undo | Icon::Redo | Icon::Split | Icon::Cut | Icon::FormFlatten => {
            (Hue::Primary, &[0, 1])
        }
        Icon::Convert | Icon::ManageList => (Hue::Primary, &[0, 1, 2]),
        Icon::DimensionGroups => (Hue::Primary, &[0, 3, 6]),
        Icon::RadioButton
        | Icon::Recent
        | Icon::Save
        | Icon::SelectAll
        | Icon::TextUnderline
        | Icon::ZoomRegion
        | Icon::Guides
        | Icon::PickPath => (Hue::Primary, &[1]),
        Icon::EmbedFonts
        | Icon::MeasureLength
        | Icon::MeasureRadius
        | Icon::OffPage
        | Icon::FitWidth
        | Icon::FitHeight
        | Icon::RotateCcw
        | Icon::RotateCw
        | Icon::EditObjects
        | Icon::Info
        | Icon::Pointer
        | Icon::ImportFormData
        | Icon::Export
        | Icon::PageExtract
        | Icon::PickText
        | Icon::PickLink => (Hue::Primary, &[1, 2]),
        Icon::ShowPoints | Icon::Settings | Icon::CursorNode => (Hue::Primary, &[1, 2, 3]),
        Icon::DropDown
        | Icon::MeasureAngle
        | Icon::MergeInto
        | Icon::NewFromTemplate
        | Icon::Comment
        | Icon::ZoomOut
        | Icon::ZoomIn => (Hue::Primary, &[2]),
        Icon::NextDocument
        | Icon::PreviousDocument
        | Icon::RecogniseText
        | Icon::SaveCompacted
        | Icon::ExportImage
        | Icon::ResetLayout => (Hue::Primary, &[2, 3]),
        Icon::CopyPageText | Icon::CopyAsVector => (Hue::Primary, &[2, 3, 4]),
        Icon::Sign | Icon::PickFormXObject => (Hue::Primary, &[3]),
        Icon::CopyDocumentText | Icon::Reflow => (Hue::Primary, &[3, 4]),
        Icon::WheelFlip => (Hue::Primary, &[4]),
        Icon::Properties | Icon::SetScale => (Hue::Primary, &[6, 7, 8]),
        Icon::Close | Icon::Stamp => (Hue::Remove, &[0]),
        Icon::Delete => (Hue::Remove, &[0, 1]),
        Icon::TextStrikeout | Icon::TextSquiggly => (Hue::Remove, &[1]),
        Icon::UnembedFonts => (Hue::Remove, &[1, 2]),
        Icon::CloseOthers | Icon::OpenInAcrobat => (Hue::Remove, &[2, 3]),
        _ => return None,
    })
}

/// The `egui::Context` key the switch is mirrored under.
const KEY: &str = "pdfcer-icon-accents";

/// Mirror the preference onto `ctx`, once a frame.
pub fn sync(ctx: &egui::Context, on: bool) {
    if enabled(ctx) != on {
        ctx.data_mut(|d| d.insert_temp(egui::Id::new(KEY), on));
    }
}

/// Whether coloured icons are on. `false` until [`sync`] says otherwise.
#[must_use]
pub fn enabled(ctx: &egui::Context) -> bool {
    ctx.data(|d| d.get_temp(egui::Id::new(KEY)))
        .unwrap_or(false)
}

/// The colours `icon` is to be baked in on a control tinted `tint`, or
/// `None` to draw the plain mask: the option is off, the icon has no accent,
/// the control is disabled (a disabled control is grey whole, as in Office)
/// or the tint is fully transparent.
///
/// The body takes `tint` made opaque; the caller draws the texture tinted
/// `Color32::from_white_alpha(tint.a())`, so a faded control fades both parts.
#[must_use]
pub fn baked(
    ctx: &egui::Context,
    icon: Icon,
    tint: egui::Color32,
    control_enabled: bool,
) -> Option<Baked> {
    if !control_enabled || tint.a() == 0 || !enabled(ctx) {
        return None;
    }
    let (hue, _) = accent(icon)?;
    Some(Baked {
        body: tint.to_opaque(),
        accent: hue.colour(IconAccents::of(ctx)),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::icons::{IconArt, IconWeight};

    #[test]
    fn every_accent_names_shapes_the_asset_has() {
        for &icon in Icon::ALL {
            let Some((_, shapes)) = accent(icon) else {
                continue;
            };
            let n = IconArt::parse(icon.source()).expect("parses").shape_count();
            assert!(!shapes.is_empty(), "{}: empty accent", icon.name());
            for &i in shapes {
                assert!(i < n, "{}: accent shape {i} of {n}", icon.name());
            }
        }
    }

    /// A coloured glyph lights exactly the pixels the plain mask does, so
    /// turning the option on recolours a glyph and never redraws it; and its
    /// accent really draws.
    #[test]
    fn a_coloured_glyph_covers_what_the_plain_mask_covers() {
        // NOT A THEME COLOUR: two test colours told apart by channel.
        let (body, hue) = (egui::Color32::WHITE, egui::Color32::from_rgb(255, 0, 0));
        for &icon in Icon::ALL {
            let Some((_, shapes)) = accent(icon) else {
                continue;
            };
            let art = IconArt::parse(icon.source()).expect("parses");
            let whole = art.rasterize(32, IconWeight::Regular);
            let coloured = art.rasterize_with(32, IconWeight::Regular, |i| {
                Some(if shapes.contains(&i) { hue } else { body })
            });
            let mut accent_px = 0;
            for (k, (w, c)) in whole.pixels.iter().zip(&coloured.pixels).enumerate() {
                assert_eq!(
                    w.a() > 0,
                    c.a() > 0,
                    "{}: pixel {k} differs between the plain and the coloured glyph",
                    icon.name()
                );
                if c.a() > 128 && c.r() > c.g().saturating_add(100) {
                    accent_px += 1;
                }
            }
            assert!(accent_px > 0, "{}: the accent draws nothing", icon.name());
        }
    }

    #[test]
    fn the_switch_is_off_until_synced() {
        let ctx = egui::Context::default();
        assert!(!enabled(&ctx));
        sync(&ctx, true);
        assert!(enabled(&ctx));
        sync(&ctx, false);
        assert!(!enabled(&ctx));
    }
}
