//! # `text::pick` — every word the selection filter says
//!
//! The strings for [`crate::canvas::pick`] and for the status-bar popup that
//! drives it. That module's header carries the design and the invariants; this
//! file carries the copy.
//!
//! ## ★ The vocabulary is the OPERATOR's, not the PDF specification's
//!
//! This is the rule the whole file follows, and it is worth stating because
//! every row here has a perfectly good technical name that would be the wrong
//! label.
//!
//! | this file says | the specification calls it | why the operator's word wins |
//! |---|---|---|
//! | **Lines** | path object | It is the word he used when he asked for the feature — *"text, points, lines, etc"*. On a CAD sheet, path objects **are** the line work |
//! | **Points** | anchor, on a subpath | Likewise his word. "Anchor" is Illustrator's, "node" is Inkscape's, "vertex" is CAD's, and "point" is what he says |
//! | **Blocks** | form XObject | ★ See below — this is the most load-bearing choice in the file |
//! | **Pictures** | image XObject / inline image | Already the shell's word elsewhere: *"Select a picture and drag it"* |
//! | **Characters** | the text-selection sweep | Names the unit, which is what distinguishes it from the Text row |
//!
//! ### ★★ Why a form XObject is called a "Block"
//!
//! A form XObject is an entire nested drawing that the page treats as one
//! opaque object. There is no everyday English word for that — and there is a
//! perfect **CAD** word for it, which is the vocabulary this operator actually
//! has: a *block*. An AutoCAD block is precisely this. One insert, one
//! selectable thing, a hundred visible marks inside it.
//!
//! The two alternatives were both worse:
//!
//! - *"Groups"* collides with **layers** (optional content groups), which this
//!   shell has a whole panel for. Two different things called groups, one
//!   panel apart.
//! - *"Form XObjects"* is correct, is what the file contains, and is
//!   meaningless to anybody who has not read the specification. A filter row
//!   nobody can decode is a row nobody switches.
//!
//! This is the *"use the conventional interaction, never invent one"* rule
//! applied to a noun rather than to a gesture: the product class the operator
//! comes from already named this thing, and borrowing that name costs nothing
//! and teaches nothing new.
//!
//! ## Why "Dimensions" is not a Rule 15 violation
//!
//! Rule 15 forbids a bare *"dimension"* in code, comments, commits and specs,
//! because **ce dimensions** (the ones pdfcer authors) and **pdf dimensions**
//! (CAD-exported page content pdfcer must not alter) have opposite properties.
//!
//! The label below is deliberately bare anyway, and the reason is that the
//! ambiguity does not exist on this surface. The row filters `AnnotKind::
//! CeDimension` — annotations pdfcer itself wrote. A pdf dimension exported by
//! a CAD package is page content and is picked by the **Lines** and **Text**
//! rows like any other ink; it can never arrive at this row. The operator has
//! exactly one kind of thing to think about here, and the shell already says
//! "Dimension" and "Dimension groups" to him elsewhere. A label reading "ce
//! dimensions" would introduce a distinction on the one surface where it
//! cannot apply.
//!
//! The doc comments below stay precise; the labels stay short.
//!
//! ## Two rules inherited from `text::tool`, and they hold here too
//!
//! **Every sentence states a fact, never a tip.** *"Clicks pass through text"*
//! is a statement. *"Try switching text off to reach the drawing underneath!"*
//! is a tip, and there are none here.
//!
//! **A tooltip says what switching the row OFF does**, not what the class is.
//! The label already names the class; the operator hovering it is asking what
//! the control does, and for a subtractive filter the interesting direction is
//! always off.

use crate::canvas::pick::PickClass;

// ===========================================================================
// Block A — the status-bar control itself
// ===========================================================================

/// The label on the status-bar button that opens the selection filter.
///
/// One word, because the status bar's whole right-hand cluster is one word per
/// control ("Find", "Fit width", "Fit page") and a longer label here would be
/// the widest thing on the bar.
///
/// "Select" rather than "Filter" because it names what the control *governs*
/// rather than the mechanism it uses to govern it. An operator scanning the bar
/// for "how do I stop grabbing the border" is looking for the word for the
/// thing they are doing.
#[must_use]
pub fn filter_button() -> &'static str {
    "Select"
}

/// Hover text for the selection-filter button.
///
/// Names the count, because the button's own label cannot: "Select" looks
/// identical whether every class is on or one is, and the difference is the
/// entire state of the control.
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

/// ★ Shown on the status bar whenever **nothing at all** is selectable.
///
/// This exists because the state is legitimate and its symptom is
/// indistinguishable from a fault. An operator who switched everything off
/// half an hour ago, and has since been panning and reading, will click an
/// object, get nothing, click again, get nothing, and reasonably conclude the
/// program is broken. They would be right to: from where they are sitting, a
/// canvas that ignores every click *is* broken.
///
/// So the shell says so, on the bar, in the operator's own terms, next to the
/// control that caused it. Not a dialog, not a toast, not a mark on the page —
/// a standing statement that goes away when the cause does.
#[must_use]
pub fn nothing_selectable() -> &'static str {
    "Nothing on the page can be selected"
}

/// Hover text for [`nothing_selectable`], naming the way out.
///
/// The one place in this file that comes close to an instruction, and it earns
/// it: the operator reading this has already concluded something is wrong, and
/// the fact they need is *which control did this*.
#[must_use]
pub fn nothing_selectable_tooltip() -> &'static str {
    "Every class is switched off in Select. Choosing All turns them back on."
}

// ===========================================================================
// Block B — one label per class
// ===========================================================================

/// The operator-facing name of one selectable class.
///
/// A single function over the enum rather than eleven free functions, because
/// the popup renders `PickClass::ALL` in a loop and a per-variant function set
/// would need a `match` at the call site anyway — one that could silently omit
/// a variant. This way, adding a class is a compile error here.
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
///
/// Every sentence describes the same shape of consequence — *clicks stop
/// landing on X and reach whatever is behind it* — because that is genuinely
/// what a subtractive filter does, and eleven differently-phrased sentences
/// would imply eleven different mechanisms.
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

    /// ★ The Links row must not promise what the build cannot do.
    ///
    /// `PickClass::Link` is off by default because nothing can pick a link, and
    /// its tooltip is the only place the operator is told that. If link picking
    /// lands and this tooltip is not rewritten, the shell starts lying in the
    /// helpful direction — which is the direction nobody reports.
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
