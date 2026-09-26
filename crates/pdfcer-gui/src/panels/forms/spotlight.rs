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
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Spotlight {
    /// The field's fully-qualified name (§12.7.3.2).
    pub field: String,
}

/// **Point the spotlight at `field`.**
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

    /// Set, read, clear — the whole contract, over a real `Context`.
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

    /// The key is distinct from the canvas's own focus key.
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
