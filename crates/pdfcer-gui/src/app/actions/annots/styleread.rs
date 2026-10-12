//! **What a markup restyle left in the file** — the `set-markup-style-applied`
//! trace line, read back from the session after `set_markup_style` succeeds.
//!
//! Read back rather than echoed from the request, so a verb that accepted the
//! style and wrote something else shows the difference.

use pdfcer_core::annot_author::{Color, LineEnding, MarkupSpec};
use pdfcer_core::edit::EditSession;
use pdfcer_core::object::{ObjId, Object};

/// The trace line's first token. Distinct from the funnel's `set-markup-style`.
const TRACE: &str = "set-markup-style-applied"; // ui-text-exempt: diagnostic trace name

/// `set-markup-style-applied id=… subtype=… interior=… endings=…`.
///
/// `interior=` is `none` or the colour's channels; `endings=` is
/// `start/end` for a `/Line` and `-` for everything else.
pub(in crate::app::actions) fn trace_applied(session: &EditSession, id: ObjId) {
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed in the UI.
        let graph = session.graph();
        let spec = match session.value(id) {
            Some(Object::Dict(dict)) => {
                pdfcer_core::annot_author::spec_from_dict(&graph, dict).ok()
            }
            _ => None,
        };
        let (subtype, interior, endings) = match spec {
            Some(MarkupSpec::Line {
                endings, interior, ..
            }) => (
                "Line",
                colour(interior),
                format!("{}/{}", ending(endings.0), ending(endings.1)),
            ),
            Some(MarkupSpec::Square { interior, .. }) => ("Square", colour(interior), "-".into()),
            Some(MarkupSpec::Circle { interior, .. }) => ("Circle", colour(interior), "-".into()),
            Some(MarkupSpec::Polygon { interior, .. }) => ("Polygon", colour(interior), "-".into()),
            Some(_) => ("other", "none".to_owned(), "-".into()),
            None => ("unread", "none".to_owned(), "-".into()),
        };
        format!(
            "{TRACE} id={} subtype={subtype} interior={interior} endings={endings}",
            id.num
        )
    });
}

/// `none`, or the channels joined by commas to three places.
fn colour(c: Option<Color>) -> String {
    match c {
        None => "none".to_owned(),
        Some(Color::Gray(g)) => format!("gray:{g:.3}"),
        Some(Color::Rgb(r, g, b)) => format!("rgb:{r:.3},{g:.3},{b:.3}"),
        Some(Color::Cmyk(c, m, y, k)) => format!("cmyk:{c:.3},{m:.3},{y:.3},{k:.3}"),
    }
}

/// The `/LE` name, spelled as the file spells it.
const fn ending(e: LineEnding) -> &'static str {
    match e {
        LineEnding::None => "None",
        LineEnding::OpenArrow => "OpenArrow",
        LineEnding::ClosedArrow => "ClosedArrow",
    }
}
