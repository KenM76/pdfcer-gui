//! # `text::new_document` — the copy of the sized-New dialog
//!
//! Every operator-facing string `crate::dialogs::new_document` draws. One
//! function per string, per `crate::text`'s contract: the gate
//! `tools/gates/check-ui-strings.sh` fails the build for a literal that
//! reaches a widget from anywhere else.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/text/new_document.md`.

/// The dialog's window title.
#[must_use]
pub const fn window_title() -> &'static str {
    "New document"
}

/// The one-line introduction, above the size list.
#[must_use]
pub const fn intro() -> &'static str {
    "One blank page, at the size you choose."
}

/// Heading over the size list.
#[must_use]
pub const fn size_heading() -> &'static str {
    "Page size"
}

/// The entry that opens the two width/height fields.
#[must_use]
pub const fn size_custom() -> &'static str {
    "Custom…"
}

/// The presentable name of a standard size — `"A1"`, `"Letter"`, `"ANSI D"`.
#[must_use]
pub fn size_name(size: pdfcer_core::paper::PaperSize) -> String {
    use pdfcer_core::paper::PaperSize as P;
    match size {
        P::A0 => "A0".to_owned(),
        P::A1 => "A1".to_owned(),
        P::A2 => "A2".to_owned(),
        P::A3 => "A3".to_owned(),
        P::A4 => "A4".to_owned(),
        P::A5 => "A5".to_owned(),
        P::A6 => "A6".to_owned(),
        P::Letter => "Letter".to_owned(),
        P::Legal => "Legal".to_owned(),
        P::Tabloid => "Tabloid".to_owned(),
        P::Executive => "Executive".to_owned(),
        P::AnsiA => "ANSI A".to_owned(),
        P::AnsiB => "ANSI B".to_owned(),
        P::AnsiC => "ANSI C".to_owned(),
        P::AnsiD => "ANSI D".to_owned(),
        P::AnsiE => "ANSI E".to_owned(),
        // ui-text-exempt: not a written string — the engine's own identifier,
        // uppercased, for a size added to `PaperSize` after this build.
        other => other.id().to_uppercase(),
    }
}

/// One entry in the size list: the standard's own name, then its millimetres.
///
/// `name` comes from [`size_name`]; the millimetres are computed here so the
/// list and [`sheet_summary`] round identically.
#[must_use]
pub fn size_entry(name: &str, size_pt: (f64, f64)) -> String {
    if imperial(name) {
        return format!("{name} — {} × {} in", inches(size_pt.0), inches(size_pt.1));
    }
    use crate::units::whole_mm_from_points as mm;
    format!("{name} — {} × {} mm", mm(size_pt.0), mm(size_pt.1))
}

/// **Is this sheet DEFINED in inches?**
fn imperial(name: &str) -> bool {
    matches!(name, "Letter" | "Legal" | "Tabloid" | "Executive")
        || name.starts_with("ANSI ")
        || name.starts_with("ARCH ")
}

/// A dimension in inches, to the nearest sixteenth, with the fraction spelled
/// the way a drawing office writes it.
fn inches(pt: f64) -> String {
    // Round ONCE, in sixteenths, then split — rather than truncating to a
    // whole inch and rounding the remainder separately.
    //
    //
    //     pt = 719.5  ->  9.993055... in
    //                     whole      = 9
    //                     remainder  = 0.993055 × 16 = 15.888 -> rounds to 16
    //                     printed    "9 16/16", reduced to "9 1/1"
    //
    // 719.5 pt is 9.993 in, which is an ordinary custom size for anyone laying
    // out to a 10 in trim. The four named imperial sheets are all exact
    // multiples of a sixteenth, which is why this never showed.
    //
    // Rounding the sixteenth COUNT puts the carry inside the rounding: 159.888
    // rounds to 160, and 160 / 16 is 10 with no remainder, so the same input
    // now prints "10".
    let sixteenths_total = crate::units::whole(crate::units::inches_from_points(pt) * 16.0);
    debug_assert!(
        sixteenths_total >= 0,
        "sheet sizes are positive; a negative length would make the / and % below \
         truncate toward zero and print a sign in the wrong place"
    );

    let whole = sixteenths_total / 16;
    let sixteenths = sixteenths_total % 16;
    if sixteenths == 0 {
        return format!("{whole}");
    }
    // Reduce the fraction: 8/16 is a half, not eight sixteenths.
    let mut num = sixteenths;
    let mut den = 16_i64;
    while num % 2 == 0 && den % 2 == 0 {
        num /= 2;
        den /= 2;
    }
    if whole == 0 {
        format!("{num}/{den}")
    } else {
        format!("{whole} {num}/{den}")
    }
}

/// Heading over the orientation pair.
#[must_use]
pub const fn orientation_heading() -> &'static str {
    "Orientation"
}

/// Taller than wide.
#[must_use]
pub const fn orientation_portrait() -> &'static str {
    "Portrait"
}

/// Wider than tall.
#[must_use]
pub const fn orientation_landscape() -> &'static str {
    "Landscape"
}

/// Label for the custom width field. States its unit.
#[must_use]
pub const fn custom_width() -> &'static str {
    "Width (mm)"
}

/// Label for the custom height field.
#[must_use]
pub const fn custom_height() -> &'static str {
    "Height (mm)"
}

/// The resulting sheet, echoed under the controls in both units.
///
/// # Why it is echoed at all when the list entry already says it
///
/// Because the list entry says the size **portrait**, and the orientation
/// toggle beside it can transpose it. A reader who picks A1 and then Landscape
/// has been shown "841 × 1189 mm" and is about to get 1189 × 841. One line
/// that reports the actual outcome removes the arithmetic.
///
/// It also reports **points**, which the list deliberately does not. Points
/// are what the `/MediaBox` will say and what every other measurement in this
/// application is in, so an operator comparing this sheet against a drawing
/// that arrived from CAD needs them. Millimetres first because that is the
/// unit the decision was made in.
#[must_use]
pub fn sheet_summary(width_pt: f64, height_pt: f64) -> String {
    use crate::units::whole_mm_from_points as mm;
    format!(
        "Sheet: {} × {} mm  ·  {} × {} in  ·  {} × {} pt",
        mm(width_pt),
        mm(height_pt),
        inches(width_pt),
        inches(height_pt),
        crate::units::whole(width_pt),
        crate::units::whole(height_pt),
    )
}

/// Why a custom size is being refused, shown in place of [`sheet_summary`].
#[must_use]
pub fn custom_refused(min_mm: i64, max_mm: i64) -> String {
    format!("Each side must be between {min_mm} and {max_mm} mm. Nothing is made until both are.")
}

/// The Create button.
#[must_use]
pub const fn create() -> &'static str {
    "Create"
}

/// Hover text for [`create`], stating the consequence.
#[must_use]
pub const fn create_tooltip() -> &'static str {
    "Makes the document and replaces what is open."
}

/// The Cancel button.
#[must_use]
pub const fn cancel() -> &'static str {
    "Cancel"
}

#[cfg(test)]
mod imperial_tests {
    use super::{size_entry, size_name};
    use pdfcer_core::paper::PaperSize as P;

    /// **Every US and ANSI sheet reads in inches, and every ISO one in
    /// millimetres.**
    #[test]
    fn a_us_sheet_reads_in_inches_and_an_iso_sheet_in_millimetres() {
        let entry = |p: P| size_entry(&size_name(p), p.size_pt());

        assert_eq!(entry(P::AnsiB), "ANSI B — 11 × 17 in");
        assert_eq!(entry(P::Tabloid), "Tabloid — 11 × 17 in");
        assert_eq!(entry(P::AnsiA), "ANSI A — 8 1/2 × 11 in");
        assert_eq!(entry(P::Letter), "Letter — 8 1/2 × 11 in");
        assert_eq!(entry(P::Legal), "Legal — 8 1/2 × 14 in");
        assert_eq!(entry(P::AnsiD), "ANSI D — 22 × 34 in");

        assert_eq!(entry(P::A4), "A4 — 210 × 297 mm");
        assert_eq!(entry(P::A1), "A1 — 594 × 841 mm");
    }

    /// **The fraction is reduced and written the way a title block writes it.**
    #[test]
    fn a_half_inch_is_written_as_a_half() {
        let entry = size_entry(&size_name(P::Letter), P::Letter.size_pt());
        assert!(entry.contains("8 1/2"), "{entry}");
        assert!(
            !entry.contains("8/16"),
            "the fraction was not reduced: {entry}"
        );
        assert!(!entry.contains("8.5"), "a decimal, not a fraction: {entry}");
    }

    /// **The summary answers both offices**, whichever sheet was picked.
    #[test]
    fn the_summary_carries_both_units() {
        let summary = super::sheet_summary(792.0, 1224.0);
        assert!(summary.contains("279 × 432 mm"), "{summary}");
        assert!(summary.contains("11 × 17 in"), "{summary}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The millimetre conversion agrees with the engine's own table.
    #[test]
    fn a_named_size_reads_back_as_its_own_millimetres() {
        let a1 = pdfcer_core::paper::PaperSize::A1.size_pt();
        let label = size_entry("A1", a1);
        assert!(
            label.contains("594 × 841"),
            "A1 must read as its defining millimetres: {label}"
        );

        let a4 = pdfcer_core::paper::PaperSize::A4.size_pt();
        let summary = sheet_summary(a4.0, a4.1);
        assert!(
            summary.contains("210 × 297"),
            "A4 must read as its defining millimetres: {summary}"
        );
        // And the points are there too, because a CAD comparison needs them.
        assert!(
            summary.contains("595"),
            "the summary must state points: {summary}"
        );
    }

    /// The refusal names the ceiling.
    #[test]
    fn the_custom_refusal_states_the_limits() {
        let message = custom_refused(1, 5080);
        assert!(message.contains("5080"), "no upper limit in {message}");
        assert!(message.contains('1'), "no lower limit in {message}");
    }

    /// No size in the list reads like an identifier.
    #[test]
    fn no_size_in_the_list_reads_like_an_identifier() {
        for size in pdfcer_core::paper::PaperSize::ALL {
            let name = size_name(*size);
            assert!(
                !name.contains('-'),
                "{size:?} rendered as {name:?} — a hyphen means it fell through to the \
                 identifier fallback and needs a written name"
            );
            assert!(!name.is_empty(), "{size:?} has an empty name");
        }
        // And the ANSI pair specifically, because they are the ones the
        // fallback would visibly mangle and the ones most likely to be added
        // to in future (ARCH A-E are named as plausible).
        assert_eq!(size_name(pdfcer_core::paper::PaperSize::AnsiD), "ANSI D");
    }

    /// A sheet just under a whole inch used to read `9 1/1`.
    ///
    ///
    /// ```text
    ///     719.5 pt = 9.993055... in
    ///       whole     = total.trunc()          = 9
    ///       remainder = 0.993055 × 16         = 15.888 -> rounds to 16
    ///       printed   "9 16/16" -> reduced -> "9 1/1"
    /// ```
    ///
    ///
    /// The test asserts three things the old shape could not satisfy together:
    /// the carry case, the exact case, and a genuine fraction still reducing.
    #[test]
    fn an_inch_fraction_that_rounds_up_carries_into_the_whole_number() {
        // The carry. 719.5 pt is 9.993 in; to the nearest sixteenth that is 10.
        assert_eq!(
            inches(719.5),
            "10",
            "a sixteenth short of 10 in must read 10"
        );

        // The exact case still reads as a bare whole number, with no " 0/16".
        assert_eq!(inches(720.0), "10");

        // And a real fraction still reduces rather than printing sixteenths.
        assert_eq!(
            inches(612.0),
            "8 1/2",
            "US Letter's width is eight and a half"
        );

        // Below one inch there is no whole part to print.
        assert_eq!(inches(18.0), "1/4");

        // The carry at the top of the sub-inch range: 71.9 pt is 0.9986 in,
        // which rounds to 16 sixteenths — one inch, not "0 1/1".
        assert_eq!(inches(71.9), "1");
    }
}
