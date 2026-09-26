//! # `text::pick` — every word the selection filter says
//!
//! The strings for [`crate::canvas::pick`] and for the status-bar popup that
//! drives it. That module's header carries the design and the invariants; this
//! file carries the copy.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/text/pick.md`.

use crate::canvas::pick::PickClass;

// ===========================================================================
// Block A — the status-bar control itself
// ===========================================================================

/// The label on the status-bar button that opens the selection filter.
#[must_use]
pub fn filter_button() -> &'static str {
    "Select"
}

/// Hover text for the selection-filter button.
#[must_use]
pub fn filter_button_tooltip() -> &'static str {
    "Choose what a click on the page can select."
}

/// The popup's heading.
#[must_use]
pub fn filter_heading() -> &'static str {
    "Selectable"
}

/// The row that switches every class on.
#[must_use]
pub fn filter_all() -> &'static str {
    "All"
}

/// The row that switches every class off.
#[must_use]
pub fn filter_none() -> &'static str {
    "None"
}

/// Shown on the status bar whenever **nothing at all** is selectable.
#[must_use]
pub fn nothing_selectable() -> &'static str {
    "Nothing on the page can be selected"
}

/// Hover text for [`nothing_selectable`], naming the way out.
#[must_use]
pub fn nothing_selectable_tooltip() -> &'static str {
    "Every class is switched off in Select. Choosing All turns them back on."
}

// ===========================================================================
// Block B — one label per class
// ===========================================================================

/// The operator-facing name of one selectable class.
#[must_use]
pub fn class_label(class: PickClass) -> &'static str {
    match class {
        PickClass::Text => "Text",
        PickClass::Path => "Lines",
        PickClass::Image => "Pictures",
        PickClass::FormXObject => "Blocks",
        PickClass::Part => "Parts",
        PickClass::Node => "Points",
        PickClass::Markup => "Markup",
        PickClass::CeDimension => "Dimensions",
        PickClass::FormField => "Form fields",
        PickClass::Link => "Links",
        PickClass::Characters => "Characters",
    }
}

/// What switching one class **off** does, for the row's hover text.
#[must_use]
pub fn class_tooltip(class: PickClass) -> &'static str {
    match class {
        PickClass::Text => {
            "Off: clicks pass through text and reach whatever is behind it. Sweeping to \
             copy is the Characters row."
        }
        PickClass::Path => {
            "Off: clicks pass through the drawing's line work. Useful for reaching \
             something buried under dense geometry."
        }
        PickClass::Image => "Off: clicks pass through pictures.",
        PickClass::FormXObject => {
            "Off: clicks pass through blocks — title blocks, borders, and anything else \
             stored as one nested drawing."
        }
        PickClass::Part => {
            "Off: selection stops at whole objects. Double-clicking no longer goes \
             inside one."
        }
        PickClass::Node => {
            "Off: corner points are never selected and never offered as drag handles."
        }
        PickClass::Markup => "Off: clicks pass through notes, shapes and stamps.",
        PickClass::CeDimension => {
            "Off: clicks pass through the dimensions you have placed. Dimensions drawn \
             by the program that made the file are page content, and belong to Lines \
             and Text."
        }
        PickClass::FormField => "Off: clicks pass through form fields, and none can be filled in.",
        PickClass::Link => {
            "Links cannot be selected in this build. Clicking one in Read or Review \r
             follows it instead. The row is here for when they can be selected."
        }
        PickClass::Characters => {
            "Off: dragging across text no longer selects letters to copy. Clicking the \
             text itself is the Text row."
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every class has a label, and no two share one. Two rows reading the
    /// same word is one row the operator cannot tell from another.
    #[test]
    fn every_class_has_a_distinct_label() {
        let mut seen = std::collections::BTreeSet::new();
        for class in PickClass::ALL {
            let label = class_label(class);
            assert!(!label.is_empty(), "{class:?} has an empty label");
            assert!(seen.insert(label), "duplicate label {label:?}");
        }
    }

    /// Every class has a tooltip, and no two share one.
    #[test]
    fn every_class_has_a_distinct_tooltip() {
        let mut seen = std::collections::BTreeSet::new();
        for class in PickClass::ALL {
            let tip = class_tooltip(class);
            assert!(!tip.is_empty(), "{class:?} has an empty tooltip");
            assert!(seen.insert(tip), "duplicate tooltip for {class:?}");
        }
    }

    /// The Links row must not promise what the build cannot do.
    #[test]
    fn the_links_row_says_it_does_nothing_yet() {
        assert!(
            !PickClass::Link.on_by_default(),
            "Link became pickable: rewrite class_tooltip(Link), which still says it cannot be"
        );
        assert!(class_tooltip(PickClass::Link).contains("cannot be selected"));
    }

    /// The Dimensions row's tooltip must keep naming the distinction Rule 15
    /// exists for, because the LABEL deliberately does not. If the sentence
    /// about CAD-drawn dimensions is ever trimmed, the one place the operator
    /// can learn which dimensions this row filters goes with it.
    #[test]
    fn the_dimensions_tooltip_still_separates_ours_from_the_cad_packages() {
        let tip = class_tooltip(PickClass::CeDimension);
        assert!(tip.contains("you have placed"));
        assert!(tip.contains("page content"));
    }
}
