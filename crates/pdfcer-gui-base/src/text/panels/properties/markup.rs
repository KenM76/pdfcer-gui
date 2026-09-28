//! # `text::panels::properties::markup` — every word the **markup style**
//! section of the Properties panel says
//!
//! ## Why the markup section has a file of its own
//!
//! `properties.rs` carries four subjects — the markup's style, the selection's
//! geometry, the selected text's style and the document's own properties. The
//! markup section is the largest of the four and the most self-contained:
//! nothing outside it reads these strings, and `pdfcer-core`'s
//! `set_markup_style` is the single verb every one of them is about.
//!
//! ## These strings are claims about the engine, and they age
//!
//! Several of them state what `set_markup_style` will or will not do. The
//! engine grows capabilities faster than a limitation sentence can be
//! re-read, and a limitation sentence that has gone stale is worse than none:
//! it withholds a capability the operator has, in words that sound
//! authoritative.
//!
//! ⇒ **Before repeating any claim here about what the engine will or will not
//! do, re-read the verb at the current pin.** Do not quote this file as a
//! source about the engine; it is a record of what this shell chose to say.
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/text/panels/properties/markup.md`.

// ===========================================================================
// The selected markup's style — `set_markup_style`
// ===========================================================================

/// The heading over the markup restyle controls.
#[must_use]
pub const fn markup_heading() -> &'static str {
    "This markup"
}

/// The line under it: what kind of mark is selected.
#[must_use]
pub fn markup_subtype(subtype: &str) -> String {
    let name = match subtype {
        "Square" => "Rectangle",
        "Circle" => "Ellipse",
        "Line" => "Arrow or line",
        "Polygon" => "Polygon or revision cloud",
        "PolyLine" => "Polyline",
        "Ink" => "Freehand",
        "Highlight" => "Highlight",
        "Underline" => "Underline",
        "StrikeOut" => "Strikeout",
        "Squiggly" => "Squiggly",
        "FreeText" => "Text box",
        "Text" => "Sticky note",
        "Stamp" => "Stamp",
        // Not "Unknown". A subtype this catalogue has no word for is still a
        // real mark the operator can see and is about to restyle, and the
        // file's own spelling is the most honest thing left to show them.
        other => other,
    };
    format!("{name} on this page")
}

/// The colour control's label.
#[must_use]
pub const fn markup_colour_label() -> &'static str {
    "Colour"
}

/// The width control's label.
#[must_use]
pub const fn markup_width_label() -> &'static str {
    "Line width"
}

/// The Line style row's label.
#[must_use]
pub const fn markup_line_style_label() -> &'static str {
    "Line style"
}

/// The cloudy-border checkbox's label.
#[must_use]
pub const fn markup_cloud_label() -> &'static str {
    "Cloudy border"
}

/// The hover text on the cloud intensity field.
#[must_use]
pub const fn markup_cloud_intensity_hover() -> &'static str {
    "How far the scallops bulge: 0 is shallow, 2 is the deepest the PDF standard allows"
}

/// The opacity control's label.
#[must_use]
pub const fn markup_opacity_label() -> &'static str {
    "Opacity"
}

/// The suffix on the opacity control.
#[must_use]
pub const fn markup_opacity_suffix() -> &'static str {
    " %"
}

/// The button that removes a property, restoring the file's own default.
#[must_use]
pub const fn markup_clear() -> &'static str {
    "Clear"
}

/// What restyling costs, said once under the whole section.
#[must_use]
pub const fn markup_note() -> &'static str {
    "Changing any of these redraws the mark from the shape pdfcer has recorded for it. A wider \
     line also makes the mark's own box bigger, except on rectangles and ellipses."
}

/// Why the controls are greyed on a locked annotation.
#[must_use]
pub const fn markup_locked() -> &'static str {
    "This mark is locked by the document, so its appearance cannot be changed here. You can \
     still delete it."
}

/// **What is possible on a mark this shell cannot restyle** — the sentence
/// that stands where live controls that could not commit would otherwise be.
#[must_use]
pub const fn markup_not_restylable() -> &'static str {
    "pdfcer cannot read this mark's shape back, so its appearance cannot be changed here. You \
     can still move it, resize it, delete it, and edit the note it carries."
}

/// The fill control's label — `/IC`, the interior colour.
#[must_use]
pub const fn markup_fill_label() -> &'static str {
    "Fill"
}

/// What sits beside the fill swatch when the mark has no `/IC` at all.
#[must_use]
pub const fn markup_fill_none() -> &'static str {
    "None"
}

/// The label over the chooser for the ending drawn at a line's **start** —
/// `/LE`'s first element (§12.5.6.7, Table 176).
#[must_use]
pub const fn markup_line_start_label() -> &'static str {
    "Line start"
}

/// The label over the chooser for the ending drawn at a line's **end** —
/// `/LE`'s second element.
#[must_use]
pub const fn markup_line_end_label() -> &'static str {
    "Line end"
}

/// One line-ending style, in the operator's words.
#[must_use]
pub const fn markup_line_ending_name(
    ending: pdfcer_core::annot_author::LineEnding,
) -> &'static str {
    use pdfcer_core::annot_author::LineEnding as L;
    // Exhaustive on purpose, with no wildcard: `LineEnding` is NOT
    // `#[non_exhaustive]`, so an ending the engine learns to draw breaks this
    // match at compile time rather than silently reaching a fallback word. That
    // is the whole reason the list is not written out in the panel module.
    match ending {
        L::None => "No end",
        L::OpenArrow => "Open arrow",
        L::ClosedArrow => "Closed arrow",
    }
}

/// The disclosure under the two line-ending choosers.
#[must_use]
pub const fn markup_line_ending_note() -> &'static str {
    "pdfcer draws three of the standard's line ends. A mark that carries any other one shows as \
     No end here, and redrawing it does not put that end back."
}

/// **The fifth state of the arrowhead controls — take the setting OUT of the
/// file rather than write "none" into it.**
#[must_use]
pub const fn markup_endings_clear() -> &'static str {
    "Clear the setting"
}

/// Why an operator would press [`markup_endings_clear`] when the line looks
/// identical either way.
#[must_use]
pub const fn markup_endings_clear_hint() -> &'static str {
    "Takes the arrowhead setting out of the file instead of writing \"no arrowheads\" into it. \
     The line looks the same either way. Use it when a mark arrived without arrowheads and you \
     want it to go back out the way it came in — on a signed or issued drawing, a setting that \
     was not there before is a difference someone will find."
}

/// The narrowing the colour swatches perform, said where the operator can
/// see it — **before** the click rather than after.
#[must_use]
pub const fn markup_colour_narrowed() -> &'static str {
    "This mark's colour is recorded in CMYK and the swatches above are an approximation of it. \
     Picking a new colour here records an RGB one in its place."
}

/// What regenerating an appearance LOST, in the operator's terms.
#[must_use]
pub const fn markup_dropped(dropped: pdfcer_core::edit::DroppedProperty) -> &'static str {
    use pdfcer_core::edit::DroppedProperty as D;
    match dropped {
        D::BorderEffect => {
            "This mark had a cloudy or hand-drawn edge that pdfcer does not redraw. It is now a plain outline."
        }
        D::BorderStyle => {
            "This mark's border was declared in a style that is not in the new outline — a bevel, an inset, an underline, or a dash. It is drawn as a plain line now."
        }
        D::DashPattern => {
            "This mark carried a dash pattern that is not in the new outline. Its border is drawn solid now."
        }
        D::RectDifferences => {
            "This mark's own box was inset from the area it covered, and that inset is gone. The mark is drawn to the box now."
        }
        D::LineEnding => {
            "This mark had arrowheads or line ends pdfcer does not redraw, and they are gone."
        }
        // `DroppedProperty` is `#[non_exhaustive]`, so a wildcard is required
        // rather than optional. It answers with the general form of the same
        // fact, which is true of every member: something the file expressed is
        // not in the picture pdfcer just drew, and saying so imprecisely is far
        // better than saying nothing.
        _ => {
            "This mark carried something pdfcer does not redraw, and it is not in the new appearance."
        }
    }
}
