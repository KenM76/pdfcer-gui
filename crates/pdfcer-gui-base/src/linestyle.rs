//! # `linestyle` — solid or dashed, on all three surfaces that
//! ask
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/linestyle.md`.

use egui::Ui;

use crate::text::markup as t;

/// **The border line style — `/BS` `/S` and `/D` (§12.5.4, Table 166)** — as
/// the four choices this shell offers, on every surface that offers them.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LineStyle {
    /// No dash — Table 166's `/S /S`, and the border every mark this shell
    /// authored before 2026-09-06 carried.
    ///
    /// A **member of this enum** rather than the enum wrapped in an `Option`,
    /// because *solid* is a thing an operator picks from the same list with the
    /// same click as the three dashes. The engine models it the same way:
    /// `StyleEdit::Clear` is an arm of the edit, not the absence of one.
    Solid,
    /// Table 166's own default dash, `[3]` — three points on, three points off.
    Dashed,
    /// A long even dash, `[8 4]`.
    LongDash,
    /// A long dash with a dot between, `[8 3 1 3]`.
    DashDot,
}

impl LineStyle {
    /// Every style, in the order all three choosers offer them.
    pub const ALL: &'static [LineStyle] = &[
        LineStyle::Solid,
        LineStyle::Dashed,
        LineStyle::LongDash,
        LineStyle::DashDot,
    ];

    /// The `/D` run lengths this style writes, in points, or `None` for solid.
    #[must_use]
    pub const fn pattern(self) -> Option<&'static [f64]> {
        match self {
            Self::Solid => None,
            // Table 166's own default — see this type's header on why it is the
            // one sourced entry in the list.
            Self::Dashed => Some(&[3.0]),
            Self::LongDash => Some(&[8.0, 4.0]),
            Self::DashDot => Some(&[8.0, 3.0, 1.0, 3.0]),
        }
    }

    /// **The engine value this style authors with**, or `None` for solid.
    #[must_use]
    pub fn dash(self) -> Option<pdfcer_core::annot_author::BorderDash> {
        self.pattern()
            .and_then(|p| pdfcer_core::annot_author::BorderDash::new(p.to_vec()))
    }

    /// **The restyle edit this style raises**, or `None` when the pattern could
    /// not be built.
    #[must_use]
    pub fn style_edit(
        self,
    ) -> Option<pdfcer_core::edit::StyleEdit<pdfcer_core::annot_author::BorderDash>> {
        match self {
            Self::Solid => Some(pdfcer_core::edit::StyleEdit::Clear),
            _ => self.dash().map(pdfcer_core::edit::StyleEdit::Set),
        }
    }

    /// Which offered style a `/D` array **is**, if it is one of them.
    #[must_use]
    pub fn of_pattern(pattern: &[f64]) -> Option<Self> {
        Self::ALL
            .iter()
            .copied()
            .find(|s| s.pattern() == Some(pattern))
    }

    /// What the operator reads for this entry.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Solid => t::line_style_solid(),
            Self::Dashed => t::line_style_dashed(),
            Self::LongDash => t::line_style_long_dash(),
            Self::DashDot => t::line_style_dash_dot(),
        }
    }
}

/// **What a mark's `/BS` currently says**, in the terms the choosers can show.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DashReading {
    /// No dash. `/S /S`, a `/BS` carrying neither key, or no `/BS` at all.
    Solid,
    /// A dash whose pattern is one of [`LineStyle::ALL`].
    Offered(LineStyle),
    /// A dash the file states in a pattern this shell does not offer.
    Foreign,
    /// A multi-selection whose members disagree. Nothing in the list is
    /// selected, and picking an entry sets every member to it.
    Mixed,
}

impl DashReading {
    /// Which entry of the chooser is selected, if any is.
    ///
    /// `None` for [`Self::Foreign`] — nothing in the list is what the file says,
    /// which is the whole reason that variant exists.
    #[must_use]
    pub const fn selected(self) -> Option<LineStyle> {
        match self {
            Self::Solid => Some(LineStyle::Solid),
            Self::Offered(style) => Some(style),
            Self::Foreign | Self::Mixed => None,
        }
    }

    /// The text the closed chooser shows.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match (self, self.selected()) {
            (_, Some(style)) => style.label(),
            (Self::Mixed, None) => t::line_style_mixed(),
            (_, None) => t::line_style_foreign(),
        }
    }
}

/// **A widget's border dash as a [`DashReading`]**, from the engine's
/// `forms::Widget::border_dash` (`/BS /D`, else `/Border`'s fourth element).
///
/// The engine gives `None` for a `Dashed` border that states no pattern; the
/// border is still drawn with Table 166's `[3]`, so the reading is
/// [`LineStyle::Dashed`]. `dashed` is whether the widget's border style is
/// `Dashed`.
#[must_use]
pub fn of_widget_dash(
    dash: Option<&pdfcer_core::annot_author::BorderDash>,
    dashed: bool,
) -> DashReading {
    match dash {
        Some(d) => {
            LineStyle::of_pattern(d.pattern()).map_or(DashReading::Foreign, DashReading::Offered)
        }
        None if dashed => DashReading::Offered(LineStyle::Dashed),
        None => DashReading::Solid,
    }
}

/// **Read `/BS` back as a [`DashReading`]** — Table 166, §12.5.4.
#[must_use]
pub fn read<G: pdfcer_core::graph::ObjectGraph + ?Sized>(
    graph: &G,
    annot: &pdfcer_core::object::Dict,
) -> DashReading {
    use pdfcer_core::object::Object;

    let Some(Object::Dict(bs)) = annot.get(b"BS").map(|o| graph.resolve(o)) else {
        return DashReading::Solid;
    };
    // Table 166: `/S` defaults to `/S` (solid). `/B`, `/I` and `/U` are borders
    // pdfcer does not author and are not dashes either — the engine returns
    // early on all three, and so does this.
    let declared_dashed = match bs.get(b"S").map(|o| graph.resolve(o)) {
        Some(Object::Name(n)) => match n.as_bytes() {
            b"D" => true,
            _ => return DashReading::Solid,
        },
        // `/S` absent: not dashed on its own, but a `/D` alongside is honoured.
        // The doc comment's third row.
        _ => false,
    };
    let pattern: Option<Vec<f64>> = match bs.get(b"D").map(|o| graph.resolve(o)) {
        Some(Object::Array(items)) => {
            let read: Vec<f64> = items
                .iter()
                .filter_map(|o| graph.resolve(o).as_number())
                .collect();
            // §8.4.3.6, mirrored from `BorderDash::new`: empty IS the solid
            // line, and an array that is negative, non-finite or all-zero
            // describes no line at all. Either way there is no pattern to show.
            let usable = !read.is_empty()
                && read.iter().all(|v| v.is_finite() && *v >= 0.0)
                && read.iter().any(|v| *v != 0.0);
            usable.then_some(read)
        }
        _ => None,
    };
    match pattern {
        Some(p) => LineStyle::of_pattern(&p).map_or(DashReading::Foreign, DashReading::Offered),
        // A `/D` that is present but unusable is a dropped pattern, not a reason
        // to call a declared-dashed border solid: the engine falls back to the
        // table default there, so `/S /D` still reads as dashed, and so does
        // this.
        None if declared_dashed => DashReading::Offered(LineStyle::Dashed),
        None => DashReading::Solid,
    }
}

/// **The chooser**, drawn once and used by all three surfaces.
pub fn chooser(ui: &mut Ui, id_salt: &str, current: DashReading, width: f32) -> Option<LineStyle> {
    let mut picked = None;
    egui::ComboBox::from_id_salt(id_salt)
        .width(width)
        .selected_text(current.label())
        .show_ui(ui, |ui| {
            let selected = current.selected();
            for &style in LineStyle::ALL {
                let is_current = selected == Some(style);
                if ui.selectable_label(is_current, style.label()).clicked() && !is_current {
                    picked = Some(style);
                }
            }
        });
    picked
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A graph in which every value is already direct.
    ///
    /// See [`table_166_is_read_back_the_way_the_engine_reads_it`] for why that
    /// is the right instrument for this reader rather than a weakened one.
    struct DirectGraph;

    impl pdfcer_core::graph::ObjectGraph for DirectGraph {
        fn value(&self, _id: pdfcer_core::object::ObjId) -> Option<&pdfcer_core::object::Object> {
            None
        }

        fn trailer_entry(&self, _key: &[u8]) -> Option<&pdfcer_core::object::Object> {
            None
        }
    }

    /// **Every pattern this shell offers is one `BorderDash::new` accepts.**
    #[test]
    fn every_offered_pattern_is_one_the_engine_accepts() {
        for &style in LineStyle::ALL {
            match style.pattern() {
                Some(pattern) => {
                    assert!(
                        !pattern.is_empty(),
                        "{style:?}: an empty array IS the solid line, not a dash"
                    );
                    assert!(
                        pattern.iter().all(|v| v.is_finite() && *v >= 0.0),
                        "{style:?}: §8.4.3.6 admits no negative or non-finite element"
                    );
                    assert!(
                        pattern.iter().any(|v| *v != 0.0),
                        "{style:?}: an all-zero pattern is never on and never off"
                    );
                    assert!(
                        style.dash().is_some(),
                        "{style:?}: BorderDash::new refused it, so this entry would draw SOLID"
                    );
                }
                // The positive control's other half: solid must have no pattern
                // and must build no dash, or `dash()`'s `Some`/`None` split
                // carries no information and the assertions above prove nothing.
                None => assert_eq!(
                    style,
                    LineStyle::Solid,
                    "only Solid may have no pattern; {style:?} would author a solid border"
                ),
            }
        }
        assert!(
            LineStyle::Solid.dash().is_none(),
            "solid must not build a dash, or the two states are one"
        );
    }

    /// **Solid clears, a dash sets** — the engine's two arms, not one.
    #[test]
    fn solid_clears_the_dash_and_a_dash_sets_one() {
        use pdfcer_core::edit::StyleEdit;
        assert!(
            matches!(LineStyle::Solid.style_edit(), Some(StyleEdit::Clear)),
            "solid must CLEAR; `None` would leave the mark's existing dash alone"
        );
        for &style in LineStyle::ALL {
            if style == LineStyle::Solid {
                continue;
            }
            let Some(StyleEdit::Set(dash)) = style.style_edit() else {
                panic!("{style:?} must Set a dash");
            };
            assert_eq!(
                dash.pattern(),
                style.pattern().expect("a dash has a pattern"),
                "{style:?} set a pattern that is not its own"
            );
        }
    }

    /// The four entries are distinct in **both** the things that distinguish
    /// them — their patterns and their names.
    #[test]
    fn the_four_styles_are_distinct_and_ordered() {
        assert_eq!(
            LineStyle::ALL,
            [
                LineStyle::Solid,
                LineStyle::Dashed,
                LineStyle::LongDash,
                LineStyle::DashDot
            ]
            .as_slice()
        );
        let mut labels: Vec<&str> = LineStyle::ALL.iter().map(|s| s.label()).collect();
        for label in &labels {
            assert!(!label.trim().is_empty());
        }
        let total = labels.len();
        labels.sort_unstable();
        labels.dedup();
        assert_eq!(labels.len(), total, "two styles share a label");

        let mut patterns: Vec<Option<&[f64]>> =
            LineStyle::ALL.iter().map(|s| s.pattern()).collect();
        let total = patterns.len();
        patterns.sort_by(|a, b| a.partial_cmp(b).unwrap());
        patterns.dedup();
        assert_eq!(patterns.len(), total, "two styles share a pattern");
    }

    /// **A pattern round-trips through [`LineStyle::of_pattern`], and a
    /// foreign one does not become one of ours.**
    #[test]
    fn a_foreign_pattern_is_not_mistaken_for_one_this_shell_offers() {
        for &style in LineStyle::ALL {
            if let Some(pattern) = style.pattern() {
                assert_eq!(LineStyle::of_pattern(pattern), Some(style));
            }
        }
        assert_eq!(LineStyle::of_pattern(&[6.0, 2.0]), None);
        assert_eq!(LineStyle::of_pattern(&[8.0, 9.0]), None);
        assert_eq!(
            LineStyle::of_pattern(&[]),
            None,
            "an empty array is the solid line and is not a dash entry"
        );
        assert_eq!(
            LineStyle::of_pattern(&[3.0, 3.0]),
            None,
            "Table 166's [3] is not [3 3]; a near-miss is still the file's own"
        );
    }

    /// A reading names an entry, or names the file — never nothing, and never
    /// one of ours for a pattern that is not.
    #[test]
    fn a_reading_shows_the_entry_it_is_or_says_the_file_owns_it() {
        assert_eq!(DashReading::Solid.selected(), Some(LineStyle::Solid));
        assert_eq!(
            DashReading::Offered(LineStyle::DashDot).selected(),
            Some(LineStyle::DashDot)
        );
        assert_eq!(
            DashReading::Foreign.selected(),
            None,
            "no entry may be shown as selected for a pattern this shell does not offer"
        );
        assert_eq!(DashReading::Foreign.label(), t::line_style_foreign());
        for &style in LineStyle::ALL {
            assert_eq!(DashReading::Offered(style).label(), style.label());
        }
    }

    /// **Table 166 read back, all four rows, including the two a literal
    /// reading gets wrong.**
    #[test]
    fn table_166_is_read_back_the_way_the_engine_reads_it() {
        use pdfcer_core::object::{Dict, Name, Object};

        let graph = DirectGraph;

        let mut plain = Dict::new();
        plain.insert(Name::from(b"Subtype"), Object::Name(Name::from(b"Square")));
        assert_eq!(read(&graph, &plain), DashReading::Solid, "no /BS at all");

        let bs = |entries: Vec<(&[u8], Object)>| {
            let mut inner = Dict::new();
            for (k, v) in entries {
                inner.insert(Name::from(k), v);
            }
            let mut annot = Dict::new();
            annot.insert(Name::from(b"BS"), Object::Dict(inner));
            annot
        };
        let array =
            |values: &[f64]| Object::Array(values.iter().copied().map(Object::Real).collect());

        assert_eq!(
            read(&graph, &bs(vec![(b"S", Object::Name(Name::from(b"S")))])),
            DashReading::Solid,
            "/S /S is the solid border"
        );
        assert_eq!(
            read(&graph, &bs(vec![(b"S", Object::Name(Name::from(b"D")))])),
            DashReading::Offered(LineStyle::Dashed),
            "/S /D with no array is Table 166's [3], not solid"
        );
        assert_eq!(
            read(&graph, &bs(vec![(b"D", array(&[4.0, 2.0]))])),
            DashReading::Foreign,
            "a /D array with no /S is honoured, and [4 2] is not one this shell offers"
        );
        assert_eq!(
            read(&graph, &bs(vec![(b"D", array(&[8.0, 4.0]))])),
            DashReading::Offered(LineStyle::LongDash),
            "…and one that IS ours is named as ours"
        );
        assert_eq!(
            read(&graph, &bs(vec![(b"S", Object::Name(Name::from(b"B")))])),
            DashReading::Solid,
            "a bevelled border is not a dash"
        );
        assert_eq!(
            read(
                &graph,
                &bs(vec![
                    (b"S", Object::Name(Name::from(b"D"))),
                    (b"D", array(&[0.0, 0.0])),
                ])
            ),
            DashReading::Offered(LineStyle::Dashed),
            "an unusable array falls back to the table default rather than to solid"
        );
        assert_eq!(
            read(
                &graph,
                &bs(vec![
                    (b"S", Object::Name(Name::from(b"S"))),
                    (b"D", array(&[0.0, 0.0])),
                ])
            ),
            DashReading::Solid,
            "…but an unusable array on a SOLID border stays solid"
        );
    }
}
