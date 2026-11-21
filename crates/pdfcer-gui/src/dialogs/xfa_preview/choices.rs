//! The Form preview window's Reading choices: the table of open choices, the
//! operator's answers, and the engine settings they make.
//!
//! An answer is an index into [`t::choice`]'s answers, `0` the default. The
//! window redraws with [`Choices::views`], [`Choices::merge`],
//! [`Choices::layout`] and [`Choices::paint`]; Use these pages and a field's
//! fill carry the same answers, so what is written is what was shown.

use pdfcer_core::xfa::{
    BareRef, BindItemsMissing, BindItemsRepeat, BindItemsUnbound, CaptionFont, CheckCaption,
    CheckEdge, ChoiceInstances, DataAttributes, EdgePlacement, FontWeight, ImageOverflow,
    ImagePlacement, LayoutOptions, Medium, MergeOptions, MissingFill, PaintOptions,
    PictureRounding, ScopeSearch, Stipple, TableCellHeight, ThreeDStroke, TiledSplit, ViewMatch,
    Views,
};
use pdfcer_gui_base::formpreview::{CHOICE_COUNT, ReadingChoices};

use crate::text::xfachoices::{self as t, ChoiceKey};

/// Every open choice, in display order; [`Choices`] is indexed the same way.
pub(super) const CHOICES: [ChoiceKey; CHOICE_COUNT] = [
    ChoiceKey::View,
    ChoiceKey::ViewMatch,
    ChoiceKey::ScopeSearch,
    ChoiceKey::BareRef,
    ChoiceKey::DataAttributes,
    ChoiceKey::BindItemsRepeat,
    ChoiceKey::BindItemsMissing,
    ChoiceKey::BindItemsUnbound,
    ChoiceKey::BindItemsSearch,
    ChoiceKey::ChoiceInstances,
    ChoiceKey::TiledSplit,
    ChoiceKey::TableCellHeight,
    ChoiceKey::ThreeDStroke,
    ChoiceKey::CheckCaption,
    ChoiceKey::CheckEdge,
    ChoiceKey::EdgePlacement,
    ChoiceKey::MissingFill,
    ChoiceKey::Stipple,
    ChoiceKey::FontWeight,
    ChoiceKey::CaptionFont,
    ChoiceKey::ImagePlacement,
    ChoiceKey::ImageOverflow,
    ChoiceKey::PictureRounding,
    ChoiceKey::Medium,
];

/// A choice's name in trace lines and region names.
const fn slug(key: ChoiceKey) -> &'static str {
    use ChoiceKey as K;
    // ui-text-exempt: trace and region names, never displayed
    match key {
        K::View => "view",
        K::ViewMatch => "view-match",
        K::ScopeSearch => "scope-search",
        K::BareRef => "bare-ref",
        K::DataAttributes => "data-attributes",
        K::BindItemsRepeat => "bind-items-repeat",
        K::BindItemsMissing => "bind-items-missing",
        K::BindItemsUnbound => "bind-items-unbound",
        K::BindItemsSearch => "bind-items-search",
        K::ChoiceInstances => "choice-instances",
        K::TiledSplit => "tiled-split",
        K::TableCellHeight => "table-cell-height",
        K::ThreeDStroke => "three-d-stroke",
        K::CheckCaption => "check-caption",
        K::CheckEdge => "check-edge",
        K::EdgePlacement => "edge-placement",
        K::MissingFill => "missing-fill",
        K::Stipple => "stipple",
        K::FontWeight => "font-weight",
        K::CaptionFont => "caption-font",
        K::ImagePlacement => "image-placement",
        K::ImageOverflow => "image-overflow",
        K::PictureRounding => "picture-rounding",
        K::Medium => "medium",
    }
}

/// The operator's answers, indexed as [`CHOICES`].
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) struct Choices(pub(crate) ReadingChoices);

impl Choices {
    fn get(self, key: ChoiceKey) -> u8 {
        CHOICES
            .iter()
            .position(|k| *k == key)
            .map_or(0, |i| self.0[i])
    }

    fn on(self, key: ChoiceKey) -> bool {
        self.get(key) == 1
    }

    /// The views the form is read for: its own, and print when chosen.
    pub(crate) fn views(self, own: Views) -> Views {
        let views = if self.on(ChoiceKey::View) {
            own.with_view("print") // ui-text-exempt: the XFA system view's name
        } else {
            own
        };
        views.with_matching(if self.on(ChoiceKey::ViewMatch) {
            ViewMatch::Any
        } else {
            ViewMatch::All
        })
    }

    pub(crate) fn merge(self) -> MergeOptions {
        let pick = |key, a, b| if self.on(key) { b } else { a };
        MergeOptions::new()
            .with_scope_search(pick(
                ChoiceKey::ScopeSearch,
                ScopeSearch::Upward,
                ScopeSearch::Children,
            ))
            .with_bare_ref(if self.on(ChoiceKey::BareRef) {
                BareRef::DataRef
            } else {
                BareRef::Ignore
            })
            .with_data_attributes(if self.on(ChoiceKey::DataAttributes) {
                DataAttributes::Ignore
            } else {
                DataAttributes::Load
            })
            .with_bind_items_repeat(if self.on(ChoiceKey::BindItemsRepeat) {
                BindItemsRepeat::First
            } else {
                BindItemsRepeat::Last
            })
            .with_bind_items_missing(if self.on(ChoiceKey::BindItemsMissing) {
                BindItemsMissing::Skip
            } else {
                BindItemsMissing::Empty
            })
            .with_bind_items_unbound(if self.on(ChoiceKey::BindItemsUnbound) {
                BindItemsUnbound::Ignore
            } else {
                BindItemsUnbound::Scope
            })
            .with_bind_items_search(pick(
                ChoiceKey::BindItemsSearch,
                ScopeSearch::Upward,
                ScopeSearch::Children,
            ))
            .with_choice_instances(if self.on(ChoiceKey::ChoiceInstances) {
                ChoiceInstances::One
            } else {
                ChoiceInstances::Occur
            })
    }

    pub(crate) fn layout(self) -> LayoutOptions {
        LayoutOptions::new()
            .with_tiled_split(if self.on(ChoiceKey::TiledSplit) {
                TiledSplit::TbOnly
            } else {
                TiledSplit::Split
            })
            .with_table_cell_height(if self.on(ChoiceKey::TableCellHeight) {
                TableCellHeight::Declared
            } else {
                TableCellHeight::Content
            })
    }

    pub(crate) fn paint(self) -> PaintOptions {
        self.paint_boxes()
            .with_missing_fill(if self.on(ChoiceKey::MissingFill) {
                MissingFill::White
            } else {
                MissingFill::None
            })
            .with_stipple(if self.on(ChoiceKey::Stipple) {
                Stipple::Spec
            } else {
                Stipple::Viewer
            })
            .with_font_weight(if self.on(ChoiceKey::FontWeight) {
                FontWeight::Bold
            } else {
                FontWeight::Normal
            })
            .with_caption_font(if self.on(ChoiceKey::CaptionFont) {
                CaptionFont::Field
            } else {
                CaptionFont::Own
            })
            .with_image_placement(if self.on(ChoiceKey::ImagePlacement) {
                ImagePlacement::Para
            } else {
                ImagePlacement::TopLeft
            })
            .with_image_overflow(if self.on(ChoiceKey::ImageOverflow) {
                ImageOverflow::Show
            } else {
                ImageOverflow::Crop
            })
            .with_picture_rounding(match self.get(ChoiceKey::PictureRounding) {
                1 => PictureRounding::HalfEven,
                2 => PictureRounding::Truncate,
                _ => PictureRounding::HalfUp,
            })
            .with_medium(if self.on(ChoiceKey::Medium) {
                Medium::Paper
            } else {
                Medium::Screen
            })
    }

    /// The border and check-box settings of [`Self::paint`].
    fn paint_boxes(self) -> PaintOptions {
        PaintOptions::new()
            .with_three_d_stroke(if self.on(ChoiceKey::ThreeDStroke) {
                ThreeDStroke::Plain
            } else {
                ThreeDStroke::Bevel
            })
            .with_check_caption(if self.on(ChoiceKey::CheckCaption) {
                CheckCaption::Remainder
            } else {
                CheckCaption::Fit
            })
            .with_check_edge(if self.on(ChoiceKey::CheckEdge) {
                CheckEdge::Spec
            } else {
                CheckEdge::Plain
            })
            .with_edge_placement(if self.on(ChoiceKey::EdgePlacement) {
                EdgePlacement::Inward
            } else {
                EdgePlacement::Grammar
            })
    }

    /// The trace's spelling: each changed choice as `slug:answer`, comma
    /// separated, or `default`.
    pub(super) fn code(self) -> String {
        let set: Vec<String> = CHOICES
            .iter()
            .zip(self.0)
            .filter(|(_, a)| *a != 0)
            .map(|(k, a)| format!("{}:{a}", slug(*k)))
            .collect();
        if set.is_empty() {
            "default".to_owned() // ui-text-exempt: trace value, never displayed
        } else {
            set.join(",")
        }
    }
}

/// The choices as radio rows under their group headings. Returns `true`
/// when one changed. Each answer declares `form-preview.choice.<slug>.<n>`.
pub(super) fn show(ui: &mut egui::Ui, choices: &mut Choices) -> bool {
    let mut changed = false;
    egui::CollapsingHeader::new(t::choices_heading())
        .id_salt("form-preview-choices") // ui-text-exempt: widget id, never displayed
        .default_open(true)
        .show(ui, |ui| {
            ui.label(t::choices_caveat());
            let mut section = None;
            for (i, key) in CHOICES.into_iter().enumerate() {
                if section != Some(t::section(key)) {
                    section = Some(t::section(key));
                    ui.add_space(4.0);
                    ui.label(egui::RichText::new(t::section_heading(t::section(key))).strong());
                }
                let (label, answers) = t::choice(key);
                ui.horizontal_wrapped(|ui| {
                    ui.label(label);
                    for (n, answer) in answers.iter().enumerate() {
                        let on = usize::from(choices.0[i]) == n;
                        let r = ui.radio(on, *answer);
                        // ui-text-exempt: trace region name, never displayed
                        let region = format!("form-preview.choice.{}.{n}", slug(key));
                        crate::diag::ui_rect(&region, r.rect);
                        if r.clicked() && !on {
                            choices.0[i] = u8::try_from(n).unwrap_or(0);
                            changed = true;
                        }
                    }
                });
            }
        });
    changed
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_defaults_are_the_engines() {
        let c = Choices::default();
        assert_eq!(c.merge(), MergeOptions::new());
        assert_eq!(c.layout(), LayoutOptions::new());
        assert_eq!(c.paint(), PaintOptions::new());
        assert_eq!(c.code(), "default");
    }

    #[test]
    fn each_answer_reaches_its_setting() {
        let mut c = Choices::default();
        let at = |k| CHOICES.iter().position(|x| *x == k).unwrap();
        c.0[at(ChoiceKey::MissingFill)] = 1;
        c.0[at(ChoiceKey::PictureRounding)] = 2;
        c.0[at(ChoiceKey::TiledSplit)] = 1;
        c.0[at(ChoiceKey::ChoiceInstances)] = 1;
        assert_eq!(c.paint().missing_fill, MissingFill::White);
        assert_eq!(c.paint().picture_rounding, PictureRounding::Truncate);
        assert_eq!(c.layout().tiled_split, TiledSplit::TbOnly);
        assert_eq!(c.merge().choice_instances, ChoiceInstances::One);
        assert_eq!(
            c.code(),
            "choice-instances:1,tiled-split:1,missing-fill:1,picture-rounding:2"
        );
    }

    #[test]
    fn every_choice_has_an_answer_for_each_index_it_can_hold() {
        for key in CHOICES {
            assert!(t::choice(key).1.len() >= 2, "{key:?}");
        }
    }
}
