//! # `text::xfachoices` — the Form preview window's Reading choices
//!
//! Every place the XFA standard leaves an answer open that Form preview
//! offers, as a label and its answers, pdfcer's default first. The window
//! (`pdfcer_gui::dialogs::xfa_preview::choices`) maps each answer to the
//! engine's merge, layout, view or paint setting.

/// One open choice.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChoiceKey {
    /// `Views::with_view("print")`.
    View,
    /// `Views::with_matching`.
    ViewMatch,
    /// `MergeOptions::scope_search`.
    ScopeSearch,
    /// `MergeOptions::bare_ref`.
    BareRef,
    /// `MergeOptions::data_attributes`.
    DataAttributes,
    /// `MergeOptions::bind_items_repeat`.
    BindItemsRepeat,
    /// `MergeOptions::bind_items_missing`.
    BindItemsMissing,
    /// `MergeOptions::bind_items_unbound`.
    BindItemsUnbound,
    /// `MergeOptions::bind_items_search`.
    BindItemsSearch,
    /// `MergeOptions::choice_instances`.
    ChoiceInstances,
    /// `LayoutOptions::tiled_split`.
    TiledSplit,
    /// `LayoutOptions::table_cell_height`.
    TableCellHeight,
    /// `PaintOptions::three_d_stroke`.
    ThreeDStroke,
    /// `PaintOptions::check_caption`.
    CheckCaption,
    /// `PaintOptions::check_edge`.
    CheckEdge,
    /// `PaintOptions::edge_placement`.
    EdgePlacement,
    /// `PaintOptions::missing_fill`.
    MissingFill,
    /// `PaintOptions::stipple`.
    Stipple,
    /// `PaintOptions::font_weight`.
    FontWeight,
    /// `PaintOptions::caption_font`.
    CaptionFont,
    /// `PaintOptions::image_placement`.
    ImagePlacement,
    /// `PaintOptions::image_overflow`.
    ImageOverflow,
    /// `PaintOptions::picture_rounding`.
    PictureRounding,
    /// `PaintOptions::medium`.
    Medium,
}

/// The four groups the choices are listed under.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChoiceSection {
    /// Which views the form is read for.
    Views,
    /// How the data fills the form.
    Data,
    /// How the pages are laid out.
    Layout,
    /// How the pages are drawn.
    Drawing,
}

/// The heading over one group of choices.
#[must_use]
pub fn section_heading(section: ChoiceSection) -> &'static str {
    match section {
        ChoiceSection::Views => "Views",
        ChoiceSection::Data => "Filling in the data",
        ChoiceSection::Layout => "Laying out the pages",
        ChoiceSection::Drawing => "Drawing",
    }
}

/// The group `key` is listed under.
#[must_use]
pub const fn section(key: ChoiceKey) -> ChoiceSection {
    use ChoiceKey as K;
    match key {
        K::View | K::ViewMatch => ChoiceSection::Views,
        K::ScopeSearch
        | K::BareRef
        | K::DataAttributes
        | K::BindItemsRepeat
        | K::BindItemsMissing
        | K::BindItemsUnbound
        | K::BindItemsSearch
        | K::ChoiceInstances => ChoiceSection::Data,
        K::TiledSplit | K::TableCellHeight => ChoiceSection::Layout,
        _ => ChoiceSection::Drawing,
    }
}

/// One open choice: its label and its answers, the default first.
#[must_use]
pub fn choice(key: ChoiceKey) -> (&'static str, &'static [&'static str]) {
    use ChoiceKey as K;
    match key {
        K::View => (
            "The form is read for",
            &["Its own views", "Printing as well"],
        ),
        K::ViewMatch => (
            "A part that names several views is kept when",
            &["All of them are chosen", "Any one is chosen"],
        ),
        K::ScopeSearch => (
            "A short data name is looked for",
            &[
                "In its group, then each enclosing one",
                "In its own group only",
            ],
        ),
        K::BareRef => (
            "A binding with a reference but no match",
            &["Is ignored", "Is followed as a data reference"],
        ),
        K::DataAttributes => (
            "Attributes in the form's data",
            &["Are values", "Are not data"],
        ),
        K::BindItemsRepeat => (
            "A list with several sources of items takes",
            &["The last", "The first"],
        ),
        K::BindItemsMissing => (
            "A list item whose value or label is not in the data",
            &["Is kept, empty", "Is left out"],
        ),
        K::BindItemsUnbound => (
            "A list bound to no data takes its items from",
            &["The data around it", "Its own items in the form"],
        ),
        K::BindItemsSearch => (
            "A list item's value or label is looked for",
            &["In its item, then each enclosing group", "In its item only"],
        ),
        K::ChoiceInstances => (
            "The section a choice set picks is copied",
            &["As often as its data allows", "Once"],
        ),
        _ => layout_or_drawing(key),
    }
}

/// [`choice`] for the layout and drawing groups.
fn layout_or_drawing(key: ChoiceKey) -> (&'static str, &'static [&'static str]) {
    use ChoiceKey as K;
    match key {
        K::TiledSplit => (
            "A section laid out in rows, at the end of a page",
            &["Splits between its rows", "Moves whole to the next page"],
        ),
        K::TableCellHeight => (
            "A table cell with a set height is",
            &["As tall as its content", "The set height"],
        ),
        K::ThreeDStroke => ("3-D borders", &["Bevelled", "Plain lines"]),
        K::CheckCaption => (
            "A check box caption with no set width",
            &["Fits its text", "Takes everything but the box"],
        ),
        K::CheckEdge => (
            "A square check box with no outline set",
            &["Has a thin black outline", "Has no outline"],
        ),
        K::EdgePlacement => (
            "A border line",
            &["Is centred on the edge", "Sits inside the edge"],
        ),
        K::MissingFill => (
            "A border with no fill",
            &["Is not filled", "Is filled white"],
        ),
        K::Stipple => (
            "A stippled fill blends",
            &["Over white, as viewers do", "Over the fill colour"],
        ),
        K::FontWeight => ("Text with no weight set is", &["Normal", "Bold"]),
        K::CaptionFont => (
            "A caption with no font of its own is set in",
            &["Courier 10 pt", "Its field's font"],
        ),
        K::ImagePlacement => (
            "An image smaller than its box sits",
            &["At the top left", "Where the box's alignment puts it"],
        ),
        K::ImageOverflow => (
            "An image larger than its box is",
            &["Cropped to the box", "Shown whole"],
        ),
        K::PictureRounding => (
            "A number with more decimals than its pattern shows is",
            &["Rounded half up", "Rounded half to even", "Cut off"],
        ),
        _ => (
            "Drop-down buttons and signature boxes",
            &["Are drawn, as on screen", "Are left off, as printed"],
        ),
    }
}

/// The heading over the choices.
#[must_use]
pub fn choices_heading() -> &'static str {
    "Reading choices"
}

/// What the choices are, under their heading.
#[must_use]
pub fn choices_caveat() -> &'static str {
    "Where the XFA standard leaves a choice open. The first answer of each is \
     pdfcer's default; changing one redraws the form."
}

/// One part of the form the chosen views leave out, named by its path of
/// names from the form's root.
#[must_use]
pub fn left_out_by_views(path: &str) -> String {
    format!("“{path}” is left out: the views chosen exclude it")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_choice_has_two_or_three_answers() {
        for key in [
            ChoiceKey::View,
            ChoiceKey::BindItemsSearch,
            ChoiceKey::TableCellHeight,
            ChoiceKey::PictureRounding,
            ChoiceKey::Medium,
        ] {
            let (label, answers) = choice(key);
            assert!(!label.is_empty());
            assert!((2..=3).contains(&answers.len()), "{key:?}");
        }
        assert_eq!(choice(ChoiceKey::PictureRounding).1.len(), 3);
    }

    #[test]
    fn a_view_names_the_part_it_leaves_out() {
        assert!(left_out_by_views("form1.body.screen_only").contains("form1.body.screen_only"));
        assert_eq!(section(ChoiceKey::Medium), ChoiceSection::Drawing);
        assert_eq!(section(ChoiceKey::TiledSplit), ChoiceSection::Layout);
    }
}
