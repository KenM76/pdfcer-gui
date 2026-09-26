//! # `panels::forms::spotlight` — **the panel→canvas channel: which field is
//! being filled**
//!
//! Design and rationale: `docs/modules/pdfcer-gui/panels/forms/spotlight.md`.

use egui::Id;

/// The temp-store key. Distinct from `canvas::forms`' focus key: that one is
/// *"the field the canvas is typing into"*, this is *"the field the panel is
/// pointing at"*, and a build that conflated them would move the caret when the
/// operator clicked a row.
const KEY: &str = "pdfcer-forms-panel-spotlight"; // ui-text-exempt: internal memory id, never displayed

/// **The field the Forms panel is pointing at.**
///
/// Named by its fully-qualified name rather than by an index, for the reason
/// [`crate::panels::forms::rows`] names everything that way: an index into a
/// walk of the form is only valid for the revision it was taken from, and this
/// value crosses a frame boundary.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Spotlight {
    /// The field's fully-qualified name (§12.7.3.2).
    pub field: String,
}

/// **Point the spotlight at `field`.**
///
/// Called by a row that was clicked or whose value box holds focus. Writing the
/// same value twice is free and is the common case — a focused box writes every
/// frame it is focused, which is what keeps the spotlight alive without a timer.
pub fn set(ctx: &egui::Context, field: &str) {
    ctx.data_mut(|d| {
        d.insert_temp(
            Id::new(KEY),
            Spotlight {
                field: field.to_owned(),
            },
        );
    });
}

/// **Put it out.**
///
/// ★ Called by the panel when nothing in it is focused — *not* by the canvas.
/// The writer owns the lifetime, because a reader that cleared what it read
/// would race any other reader and would put the spotlight out on the first
/// frame it was drawn.
pub fn clear(ctx: &egui::Context) {
    ctx.data_mut(|d| d.remove::<Spotlight>(Id::new(KEY)));
}

/// What the spotlight is on, if anything.
#[must_use]
pub fn get(ctx: &egui::Context) -> Option<Spotlight> {
    ctx.data(|d| d.get_temp::<Spotlight>(Id::new(KEY)))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// ★ Set, read, clear — the whole contract, over a real `Context`.
    ///
    /// Worth a test despite being three lines of `data_mut`, because the key is
    /// a string constant and a typo between the writer and the reader would
    /// produce a feature that silently never lights up — with no error, no
    /// panic and nothing in the trace.
    #[test]
    fn the_spotlight_round_trips_through_the_temp_store() {
        let ctx = egui::Context::default();
        assert_eq!(get(&ctx), None, "nothing is spotlit to begin with");
        set(&ctx, "Drawn By");
        assert_eq!(
            get(&ctx).map(|s| s.field),
            Some("Drawn By".to_owned()),
            "the writer and the reader must agree about the key"
        );
        set(&ctx, "Checked By");
        assert_eq!(
            get(&ctx).map(|s| s.field),
            Some("Checked By".to_owned()),
            "a second set replaces rather than stacking"
        );
        clear(&ctx);
        assert_eq!(get(&ctx), None, "clear puts it out");
    }

    /// ★★ The key is distinct from the canvas's own focus key.
    ///
    /// Pinned because the two are adjacent in purpose and a shared key would be
    /// the worst kind of bug here: clicking a panel row would move the canvas's
    /// text caret into that field, which is a different act from pointing at it
    /// and one the operator did not ask for.
    #[test]
    fn the_key_is_not_the_canvas_focus_key() {
        assert!(
            KEY.contains("spotlight"),
            "the key must name what it is, or the next reader reuses the wrong one"
        );
        // The canvas's own, spelled out so a rename there is caught here.
        assert_ne!(KEY, "pdfcer-canvas-form-focus");
    }
}
