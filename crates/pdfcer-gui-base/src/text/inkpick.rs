//! Tools ▸ Diagnostics ▸ Ink picker: the tool strip's sentences.

/// What the armed tool does.
#[must_use]
pub const fn instruction() -> &'static str {
    "Click a point on the page to read the ink a press would put there."
}

/// What the reading means.
#[must_use]
pub const fn explained() -> &'static str {
    "Ink is read from the colours the page was mixed in before they were turned into screen \
     colour, so a 100% cyan reads as 100% even where the screen cannot show it. Printing \
     settings and hidden layers are honoured; the View toggles are not."
}

/// The hover: the instruction, then what the reading means.
#[must_use]
pub fn help() -> String {
    format!("{} {}", instruction(), explained())
}

/// While the render runs.
#[must_use]
pub fn reading(x: f64, y: f64) -> String {
    format!("Reading the ink at {x:.0}, {y:.0} pt…")
}

/// `C 100%  M 100%  Y 0%  K 0%`, then any spot inks.
#[must_use]
pub fn inks(x: f64, y: f64, cmyk: [f32; 4], spots: &[(String, f32)]) -> String {
    let pc = |v: f32| (v * 100.0).round();
    let [c, m, ye, k] = cmyk;
    let mut line = format!(
        "At {x:.0}, {y:.0} pt: C {}%  M {}%  Y {}%  K {}%",
        pc(c),
        pc(m),
        pc(ye),
        pc(k)
    );
    for (name, tint) in spots {
        line.push_str(&format!("  {name} {}%", pc(*tint)));
    }
    line
}

/// Nothing painted there.
#[must_use]
pub fn bare_paper(x: f64, y: f64) -> String {
    format!("At {x:.0}, {y:.0} pt: no ink, bare paper")
}

/// The page was mixed in screen colour, so there are no ink numbers.
#[must_use]
pub fn screen_only(x: f64, y: f64, srgb: Option<[u8; 3]>) -> String {
    let shown = srgb.map_or_else(String::new, |[r, g, b]| {
        format!("; it shows #{r:02X}{g:02X}{b:02X}")
    });
    format!(
        "At {x:.0}, {y:.0} pt: this page is mixed in screen colour, so it has no ink to \
         read{shown}"
    )
}

/// The point is off the page.
#[must_use]
pub fn off_page(x: f64, y: f64) -> String {
    format!("At {x:.0}, {y:.0} pt: that point is off the page")
}

/// The render failed.
#[must_use]
pub fn failed(why: &str) -> String {
    format!("Could not read the ink there: {why}")
}

/// The render returned no reading.
#[must_use]
pub const fn no_probe() -> &'static str {
    "the renderer returned no reading"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inks_read_as_percentages_with_spots_after() {
        let line = inks(
            400.0,
            152.0,
            [1.0, 1.0, 0.0, 0.0],
            &[("Gold".to_owned(), 0.4)],
        );
        assert_eq!(line, "At 400, 152 pt: C 100%  M 100%  Y 0%  K 0%  Gold 40%");
    }
}
