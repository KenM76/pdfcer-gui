//! # `canvas::textsel::gate` — **who owns the primary button**, and the whole
//! argument for the answer
//!
//! Design and rationale: `docs/modules/pdfcer-gui/canvas/textsel/gate.md`.

use crate::app::modes::Capabilities;
use crate::canvas::tool::CanvasTool;

/// **Whether a press on the canvas means *select text*.**
///
/// The mode gate, and the whole of it — this module's header. One expression,
/// read by exactly two callers so they cannot disagree about what a press means:
/// [`crate::canvas::gesture::press_kind`], which decides it, and
/// `canvas::interact`, which routes the resulting click. That is the same
/// two-reader shape `CanvasTool::measure_kind` already has.
///
/// Note what it does **not** consult: any [`Capabilities`] field of its own.
/// `caps.edit_content` appears here as *"is the primary button already spoken
/// for?"*, not as a permission — see the header for why a `select_text`
/// capability would be the operator's *copying is not authoring* ruling restated
/// in a place free to disagree with it.
///
/// # Two disjuncts, and the second one is the original rule unchanged
///
/// > A press means text when the **text tool is armed**, *or* when the select
/// > tool is active and the mode cannot select content.
///
/// The order they are written in is the order they are read in, and it is also
/// the order of *deliberateness*: the first is a tool the operator chose, the
/// second is what an un-armed canvas does in a mode with nothing else for the
/// primary button to mean. Adding the first changed **nothing** about the
/// second — Read and Review answer `true` through the same expression they
/// always did, so an operator who never presses the new control cannot tell it
/// exists.
///
/// `is_text` rather than `matches!(tool, CanvasTool::Text)` spelled here: the
/// same predicate decides the ribbon's pressed state, and
/// [`CanvasTool::is_text`]'s docs record all three of its callers for the reason
/// `markup_kind` and `measure_kind` do.
#[must_use]
pub fn takes_the_press(tool: CanvasTool, caps: Capabilities) -> bool {
    tool.is_text() || (matches!(tool, CanvasTool::Select) && !caps.edit_content)
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::canvas::textedit::TextEditKind;

    /// **The caret tool is not a sweep tool**, in every mode.
    #[test]
    fn the_caret_tool_never_takes_the_press_for_a_text_sweep() {
        for mode in ["read", "review", "edit"] {
            for kind in [TextEditKind::Edit, TextEditKind::Add] {
                assert!(
                    !takes_the_press(CanvasTool::TextEdit(kind), caps_for(mode)),
                    "{mode}: an armed {kind:?} caret must not be read as a text sweep"
                );
            }
        }
    }

    /// The gate's three shipped answers, from the manifest the product actually
    /// ships rather than from hand-built capabilities.
    fn caps_for(mode: &str) -> Capabilities {
        Capabilities::for_mode(Some(&crate::shell::manifest::built_in()), Some(mode))
    }

    // =======================================================================
    // The mode gate — this module's header
    // =======================================================================

    /// **Read and Review select text; Edit selects content.**
    #[test]
    fn a_press_means_text_exactly_where_it_cannot_mean_content() {
        assert!(
            takes_the_press(CanvasTool::Select, caps_for("read")),
            "Read is measured against a reader that selects text"
        );
        assert!(
            takes_the_press(CanvasTool::Select, caps_for("review")),
            "Review cannot select content either, so its select tool is free"
        );
        assert!(
            !takes_the_press(CanvasTool::Select, caps_for("edit")),
            "Edit's primary button is already the content marquee"
        );
    }

    /// **With the SELECT tool, text and content can never both be
    /// available** — the exclusive-or, unchanged by the arrival of the text
    /// tool.
    #[test]
    fn the_two_selections_are_mutually_exclusive_under_the_select_tool() {
        for markup in [false, true] {
            for measure in [false, true] {
                for content in [false, true] {
                    let caps = Capabilities {
                        edit_content: content,
                        author_markup: markup,
                        author_measure: measure,
                    };
                    assert_ne!(
                        takes_the_press(CanvasTool::Select, caps),
                        crate::app::modes::capability::content_gesture(caps),
                        "exactly one of text and content may own the press: {caps:?}"
                    );
                }
            }
        }
    }

    /// **The armed text tool takes the press in EVERY mode — Edit included,
    /// which is the whole reason it exists.**
    #[test]
    fn the_armed_text_tool_takes_the_press_in_every_mode() {
        for markup in [false, true] {
            for measure in [false, true] {
                for content in [false, true] {
                    let caps = Capabilities {
                        edit_content: content,
                        author_markup: markup,
                        author_measure: measure,
                    };
                    assert!(
                        takes_the_press(CanvasTool::Text, caps),
                        "the text tool authors nothing, so no capability may refuse it: {caps:?}"
                    );
                }
            }
        }
        for mode in ["read", "review", "edit"] {
            assert!(
                takes_the_press(CanvasTool::Text, caps_for(mode)),
                "`{mode}` must sweep text with the text tool armed"
            );
        }
        // …and the select tool in Edit is untouched, which is the half a
        // permissive build would break: arming Text must not have widened the
        // *un-armed* rule.
        assert!(
            !takes_the_press(CanvasTool::Select, caps_for("edit")),
            "Edit's primary button is still the content marquee when nothing is armed"
        );
    }

    /// An armed **authoring** tool keeps its own press, in every mode. A markup
    /// or measure tool is armed *deliberately*, and handing its press to a text
    /// selection would be the tool arming and then doing nothing — the
    /// *"visible control, silently inert"* failure, wearing a crosshair.
    #[test]
    fn an_armed_authoring_tool_is_not_a_text_gesture() {
        use crate::canvas::markup::MarkupKind;
        use crate::canvas::measure::MeasureKind;
        let read = caps_for("read");
        assert!(!takes_the_press(
            CanvasTool::Markup(MarkupKind::Rectangle),
            read
        ));
        assert!(!takes_the_press(
            CanvasTool::Measure(MeasureKind::Linear),
            read
        ));
        assert!(
            !takes_the_press(CanvasTool::Hand, read),
            "the hand pans; `canvas::interact` hands the gesture machine a blank frame for it"
        );
    }
}
