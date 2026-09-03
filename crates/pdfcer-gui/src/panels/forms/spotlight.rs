//! # `panels::forms::spotlight` — **the panel→canvas channel: which field is
//! being filled**
//!
//! Operator, 2026-09-02, `OPERATOR_REQUESTS.md` O98:
//!
//! > *"when we have the fill form panel visible and I click on fields in it
//! > instead it should highlight the field on the canvas that is being filled."*
//!
//! ## ★★★ This was already named as missing, and named as PERMITTED
//!
//! [`super`]'s header has carried the gap since the panel was written, and it is
//! worth quoting because it settles the rule-4 question in advance:
//!
//! > *"the old shell's Forms panel did draw on the canvas: hovering a row
//! > highlighted the field's rectangle on the page … It was answering a real
//! > question — 'which of these is the one I am about to type into?' — and the
//! > answer is welcome under rule 4's fourth clause, which permits 'a snap
//! > indicator, a hover highlight, a rubber-band, a selection handle — these are
//! > the cursor'. It is still not carried, and the reason has changed: the
//! > mechanism now exists … so what is missing is only the panel→canvas channel
//! > for which row is hovered."*
//!
//! **This is that channel.** Nothing else had to be built: `crate::canvas::forms`
//! already places every fillable widget in canvas space.
//!
//! ## ★★ Why it is a cursor and not a mark on the content
//!
//! Rule 4 forbids *applied content* being styled differently from saved content.
//! A spotlight is neither applied nor content: it is transient, it follows the
//! operator's attention, and it disappears the moment they look elsewhere. It is
//! the same class as the pointing hand over a widget and the marquee band.
//!
//! ⇒ The one-line test — *would a screenshot of the canvas differ from a
//! screenshot of the same document saved and reopened?* — answers **yes, while
//! a row is focused**, and that is correct for the same reason a text caret is.
//!
//! ## Why a temp-memory channel rather than a field on the document
//!
//! Because it is **frame state, not document state**. It must not survive a
//! reload, must not be persisted, and must not travel with the document to
//! another tab. `egui`'s temp store is exactly that lifetime, and
//! `crate::canvas::forms` already uses it for the focused field it types into —
//! so this is the same mechanism at the same scope, not a second one.
//!
//! ★ The panel writes it and the canvas reads it, both once per frame. A panel
//! that is not drawn writes nothing, so hiding the panel puts the spotlight out
//! by construction rather than by anybody remembering to clear it.

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
