//! # `ribbon::collapsed` — a group drawn as one button
//!
//! **The rendering half of S3.** [`super::plan::collapse`] decides *which*
//! groups give up their rows; this draws one that has.
//!
//! ## What it looks like, and why exactly that
//!
//! A single control the height of the band's row area, carrying the group's
//! **caption** and a **chevron**, with the group's real contents one click
//! away in a popup. That is Word's collapsed group, measured from
//! `evidence/word-ribbon/ribbon-0800.png`: at 800 pt its Font and Paragraph
//! groups are exactly this — a captioned button with a `⌄` beneath — while
//! Clipboard beside them is untouched.
//!
//! ★ The caption is **kept**, not replaced by an icon. Word can use an icon
//! there because Office ships one per group and its operators have seen them
//! for twenty years. This shell has no group icons in its manifest at all, and
//! inventing a glyph per group would be the mistake recorded against the
//! ribbon's item sizes: *Word's icon-only clusters work because `B`/`I`/`U`
//! are findable by shape; the same treatment on "Export form data…" would be
//! two mystery glyphs.* A collapsed group whose caption still reads **Export**
//! is findable. One showing an invented arrow-in-a-box is not.
//!
//! ## ★★ The popup is the SAME renderer as the band
//!
//! [`super::band::captioned_group`] draws it, with the same row split it would
//! have had expanded — exactly as the overflow menu does. This is not code
//! tidiness, it is the fix for a defect the salvage source actually shipped:
//! a second, simpler drawing path for a menu is how two groups ended up with
//! no caption at all. One closure, three surfaces (band, overflow menu,
//! collapsed popup), and a group reads identically in all three.
//!
//! ★ `GroupBox::NATURAL` is passed, whose `rows` is `0.0`. That is deliberate
//! and it is the reason `sizing::render_large` has to be as tall as its own
//! content when handed a zero — a fact that cost a shipped, unclickable
//! **Print** button in the overflow menu and is documented at that function.
//! A collapsed group's popup is the third caller to depend on it.

use egui::{TextStyle, Vec2};

use super::band::{BandOutcome, GroupBox, captioned_group};
use super::ctx::Ctx;
use super::plan::GroupRows;
use crate::manifest::Group;

/// The chevron that says *"there is more inside"*.
///
/// The same glyph the overflow affordance uses, deliberately: an operator who
/// has learned what `⏷` means at the end of the band should not have to learn
/// a second symbol for the same promise three inches to the left.
const CHEVRON: &str = "⏷";

/// Padding either side of a collapsed group's caption.
///
/// Matches `sizing`'s `LARGE_SIDE_PADDING`, because a collapsed group sits in
/// the band beside Large controls and a different inset would read as a
/// misalignment rather than as a different kind of thing.
const SIDE_PADDING: f32 = 10.0;

/// **What a collapsed group costs the band.**
///
/// Measured from the caption, since that is the only thing whose width can
/// vary. The ladder needs this before it can decide anything, which is why it
/// is a free function taking a `&Ui` rather than something the renderer
/// returns — a width that were only known after drawing would be a
/// measurement fed back into a layout, which is the shape this project has
/// paid for twice.
pub(crate) fn width(ui: &egui::Ui, group: &Group) -> f32 {
    let caption = super::band::caption_text(group);
    let text = super::measure::text_width(ui, caption, &TextStyle::Button);
    let chevron = super::measure::text_width(ui, CHEVRON, &TextStyle::Button);
    text.max(chevron) + SIDE_PADDING * 2.0
}

/// **Draw one collapsed group**: the button, and the popup behind it.
///
/// `rows` is the split the group *would* have had expanded, passed through
/// untouched so the popup is identical to the band's rendering. `box_` is the
/// band's box, used for the button's height only — the popup gets
/// [`GroupBox::NATURAL`] like every other menu surface.
#[allow(clippy::too_many_arguments)]
pub(crate) fn render(
    ui: &mut egui::Ui,
    ctx: &mut Ctx<'_>,
    tab_id: &str,
    group: &Group,
    gutter: f32,
    rows: &GroupRows,
    box_: GroupBox,
    outcome: &mut BandOutcome,
) {
    let caption = super::band::caption_text(group);
    let w = width(ui, group);
    // `pad_top + rows` rather than `rows` alone since 2026-09-04: the band's
    // row area now starts BELOW a stated top padding (`GroupBox::pad_top`), so
    // "as tall as the rows" is the sum of the two. `total` still wins in the
    // band, where it is the full height; the `max` is for the overflow menu,
    // whose `GroupBox::NATURAL` makes every term zero.
    let h = box_.total.max(box_.pad_top + box_.rows);

    let (rect, response) = ui.allocate_exact_size(Vec2::new(w, h), egui::Sense::click());

    // ★ Announced as a button whose label is the group's caption, so a screen
    // reader hears "Font" rather than "collapsed group 2". The collapse is a
    // layout fact and not something the operator asked for; naming it would
    // report our arithmetic instead of their ribbon.
    response.widget_info(|| {
        egui::WidgetInfo::labeled(
            egui::WidgetType::Button,
            ui.is_enabled(),
            caption.to_owned(),
        )
    });

    if ui.is_rect_visible(rect) {
        let visuals = ui.style().interact(&response);
        ui.painter()
            .rect_filled(rect, visuals.corner_radius, visuals.weak_bg_fill);

        let line = ui.text_style_height(&TextStyle::Button);
        let stack = line * 2.0;
        let top = rect.center().y - stack / 2.0;
        ui.painter().text(
            egui::pos2(rect.center().x, top),
            egui::Align2::CENTER_TOP,
            caption,
            TextStyle::Button.resolve(ui.style()),
            visuals.text_color(),
        );
        ui.painter().text(
            egui::pos2(rect.center().x, top + line),
            egui::Align2::CENTER_TOP,
            CHEVRON,
            TextStyle::Button.resolve(ui.style()),
            visuals.text_color(),
        );
    }

    // Published for `ui-verify` under the group's own region name plus a
    // suffix, so a driven check can tell *"the group is on the band,
    // collapsed"* from *"the group is on the band"* and from *"the group is in
    // the overflow menu"* — three states a bare rect could not distinguish.
    ctx.reporter
        .report(rect, || super::report::group_collapsed(tab_id, &group.id));

    egui::Popup::menu(&response).show(|ui| {
        captioned_group(
            ui,
            ctx,
            tab_id,
            group,
            gutter,
            rows,
            GroupBox::NATURAL,
            outcome,
        );
    });
}

// ★ There is deliberately no `count` helper here, and the absence is worth a
// line because the first draft had one.
//
// `band`'s `debug_assert_eq!(groups_rendered, captions_emitted)` is the
// tripwire for a group drawn without a caption. A collapsed group contributes
// to NEITHER counter while its popup is closed, and to BOTH — via
// `captioned_group`, the one function that draws a caption — the moment it
// opens. So the invariant holds on both sides without help, and a helper that
// incremented the pair here would have made the tripwire pass by construction
// instead of by observation, which is the one thing it must not do.

#[cfg(test)]
mod tests {
    use super::*;

    /// **A collapsed group is narrower than its caption is long, plus padding
    /// — and never zero.**
    ///
    /// The width feeds the ladder, and a zero would make the ladder believe
    /// collapsing is free, which is how every group ends up collapsed at a
    /// width where two would have fitted.
    #[test]
    fn a_collapsed_group_is_at_least_its_padding_wide() {
        let ctx = egui::Context::default();
        super::super::testfont::install(&ctx);
        let mut measured = 0.0;
        let _ = ctx.run_ui(egui::RawInput::default(), |ui| {
            measured = width(ui, &Group::new("font", "Font"));
        });
        assert!(
            measured >= SIDE_PADDING * 2.0,
            "a collapsed group must cost at least its own padding (got {measured})"
        );
    }

    /// **A longer caption costs more**, which is what makes the ladder's
    /// re-measure meaningful rather than decorative.
    #[test]
    fn a_longer_caption_is_wider() {
        let ctx = egui::Context::default();
        // ★ Without a real font every width is zero and BOTH assertions below
        // pass vacuously on the first and fail confusingly on the second —
        // which is exactly what happened when this test was first written
        // ("20 vs 20", both of them the padding). A width test with no font
        // measures nothing; `width_tests` carries the same note.
        super::super::testfont::install(&ctx);
        let (mut short, mut long) = (0.0, 0.0);
        let _ = ctx.run_ui(egui::RawInput::default(), |ui| {
            short = width(ui, &Group::new("a", "Font"));
            long = width(ui, &Group::new("b", "Dimension groups"));
        });
        assert!(
            long > short,
            "collapsed width must track the caption ({long} vs {short})"
        );
    }
}
