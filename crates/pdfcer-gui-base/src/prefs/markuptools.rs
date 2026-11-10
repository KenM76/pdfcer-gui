//! # `prefs::markuptools` — the markup tools' remembered choices
//!
//! Two keys: `highlight_shape`, what the Highlight tool draws, and
//! `region_delete`, whether Delete in region takes objects wholly inside the
//! box or every object touching it. Both are chosen in Tool Options and kept
//! across launches; an unreadable value keeps the default for that key.
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/prefs/markuptools.md`.

use super::printing::KeyOutcome;
use crate::markupkind::MarkupKind;

// ui-text-exempt: a file KEY, written into preferences.txt and parsed back.
const SHAPE_KEY: &str = "highlight_shape";
// ui-text-exempt: a file KEY, written into preferences.txt and parsed back.
const REGION_KEY: &str = "region_delete";
// ui-text-exempt: a file value, written into preferences.txt and parsed back.
const INSIDE: &str = "inside";
// ui-text-exempt: a file value, written into preferences.txt and parsed back.
const TOUCHING: &str = "touching";

/// What one highlighter drag draws.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum HighlightShape {
    /// Follow the text under the drag; a box where there is none.
    #[default]
    Text,
    /// A box, whatever is under it.
    Area,
    /// A washed ellipse.
    Ellipse,
    /// A washed freehand stroke.
    Freehand,
    /// A washed polygon, a click per vertex.
    Polygon,
}

impl HighlightShape {
    /// Every shape, in the order Tool Options offers them.
    pub const ALL: [Self; 5] = [
        Self::Text,
        Self::Area,
        Self::Ellipse,
        Self::Freehand,
        Self::Polygon,
    ];

    /// The markup kind whose gesture draws this shape.
    #[must_use]
    pub const fn kind(self) -> MarkupKind {
        match self {
            Self::Text | Self::Area => MarkupKind::Highlight,
            Self::Ellipse => MarkupKind::Ellipse,
            Self::Freehand => MarkupKind::Ink,
            Self::Polygon => MarkupKind::Polygon,
        }
    }

    /// The value written to the file, and the trace token.
    #[must_use]
    pub const fn token(self) -> &'static str {
        match self {
            Self::Text => "text",         // ui-text-exempt: a file value and trace token
            Self::Area => "area",         // ui-text-exempt: a file value and trace token
            Self::Ellipse => "ellipse",   // ui-text-exempt: a file value and trace token
            Self::Freehand => "freehand", // ui-text-exempt: a file value and trace token
            Self::Polygon => "polygon",   // ui-text-exempt: a file value and trace token
        }
    }

    /// The shape a file value names.
    #[must_use]
    pub fn from_token(value: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|s| s.token() == value)
    }
}

/// The markup tools' remembered choices.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct MarkupToolPrefs {
    /// What the Highlight tool draws.
    pub highlight_shape: HighlightShape,
    /// Delete in region takes every object touching the box, not only those
    /// wholly inside it.
    pub region_touching: bool,
}

/// Read one `key = value` line into [`MarkupToolPrefs`], if it belongs here.
pub(super) fn parse_key(prefs: &mut MarkupToolPrefs, key: &str, value: &str) -> KeyOutcome {
    let value = value.trim();
    match key {
        SHAPE_KEY => HighlightShape::from_token(value).map_or(KeyOutcome::BadValue, |shape| {
            prefs.highlight_shape = shape;
            KeyOutcome::Accepted
        }),
        REGION_KEY => match value {
            INSIDE | TOUCHING => {
                prefs.region_touching = value == TOUCHING;
                KeyOutcome::Accepted
            }
            _ => KeyOutcome::BadValue,
        },
        _ => KeyOutcome::NotMine,
    }
}

/// Write this group's commented block into the file.
pub(super) fn write_block(prefs: &MarkupToolPrefs, out: &mut String) {
    out.push_str(
        "\n\
         # The markup tools' choices, as last made in Tool Options:\n\
         #   highlight_shape: text (default), area, ellipse, freehand or polygon\n\
         #   region_delete: inside (default) or touching\n\
         ",
    );
    // ui-text-exempt: a file KEY and its separator, never displayed.
    out.push_str("highlight_shape = ");
    out.push_str(prefs.highlight_shape.token());
    // ui-text-exempt: a file KEY and its separator, never displayed.
    out.push_str("\nregion_delete = ");
    out.push_str(if prefs.region_touching {
        TOUCHING
    } else {
        INSIDE
    });
    out.push('\n');
}

#[cfg(test)]
mod tests {
    use super::*;

    fn read_back(original: &MarkupToolPrefs) -> MarkupToolPrefs {
        let mut out = String::new();
        write_block(original, &mut out);
        let mut prefs = MarkupToolPrefs::default();
        for line in out.lines().filter(|l| !l.starts_with('#')) {
            if let Some((key, value)) = line.split_once('=') {
                assert_eq!(
                    parse_key(&mut prefs, key.trim(), value),
                    KeyOutcome::Accepted
                );
            }
        }
        prefs
    }

    #[test]
    fn every_choice_round_trips_through_the_block() {
        for highlight_shape in HighlightShape::ALL {
            for region_touching in [false, true] {
                let original = MarkupToolPrefs {
                    highlight_shape,
                    region_touching,
                };
                assert_eq!(read_back(&original), original);
            }
        }
    }

    #[test]
    fn an_unreadable_value_is_ours_and_keeps_the_default() {
        let mut prefs = MarkupToolPrefs::default();
        assert_eq!(
            parse_key(&mut prefs, SHAPE_KEY, "star"),
            KeyOutcome::BadValue
        );
        assert_eq!(
            parse_key(&mut prefs, REGION_KEY, "near"),
            KeyOutcome::BadValue
        );
        assert_eq!(
            parse_key(&mut prefs, "snapshot_dpi", "150"),
            KeyOutcome::NotMine
        );
        assert_eq!(prefs, MarkupToolPrefs::default());
    }
}
