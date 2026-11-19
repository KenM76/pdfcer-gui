//! Strings for the 3D viewer's *Colour by light level*: the switch, its
//! blend, and the legend said under the picture.
//! Consumed by `pdfcer_gui::dialogs::model3d::lightmap`.

/// The switch.
#[must_use]
pub fn map_switch() -> &'static str {
    "Colour by light level"
}

/// Its tip.
#[must_use]
pub fn map_switch_tooltip() -> &'static str {
    "Colour every surface by the light arriving on it, measured as the light meter measures it. The colours step at the recommended lighting levels, so a desk lit too little or too much reads as a different colour from one that is right. The side measured is the side you see; turning the model round measures again once it stops."
}

/// The blend switch.
#[must_use]
pub fn map_blend() -> &'static str {
    "Blend between levels"
}

/// Its tip.
#[must_use]
pub fn map_blend_tooltip() -> &'static str {
    "Off: each surface takes the colour of the level it reaches. On: the colour shades gradually from one level to the next."
}

/// Measuring, `done` of `total` points.
#[must_use]
pub fn map_measuring(done: usize, total: usize) -> String {
    if total == 0 {
        "Measuring the light…".to_owned()
    } else {
        format!(
            "Measuring the light… {:.0}%",
            done as f64 * 100.0 / total as f64
        )
    }
}

/// The map could not be measured; the model is drawn as before.
#[must_use]
pub fn map_failed(why: &str) -> String {
    format!("Light level not shown: {why}.")
}

/// The legend's heading, in lux.
#[must_use]
pub fn map_legend_lux() -> &'static str {
    "Light level, lux:"
}

/// The legend's heading for a relative map.
#[must_use]
pub fn map_legend_relative() -> &'static str {
    "Light level, relative (not lux):"
}

/// One band's lower bound beside its swatch.
#[must_use]
pub fn map_band_from(from: f64) -> String {
    format!("{from:.0}+")
}

/// A band's tip: the areas whose recommended level is `lux`.
#[must_use]
pub fn map_band_tooltip(lux: f64, areas: &[String]) -> String {
    if areas.is_empty() {
        format!("{lux:.0} lux or more")
    } else {
        format!(
            "{lux:.0} lux or more. Recommended at this level:\n{}",
            areas.join("\n")
        )
    }
}

/// One area in a band's tip, marked when the level is a maximum.
#[must_use]
pub fn map_area(area: &str, maximum: bool) -> String {
    if maximum {
        format!("• {area} (at most)")
    } else {
        format!("• {area}")
    }
}

/// The lowest and highest figure on the model.
#[must_use]
pub fn map_range(lo: f64, hi: f64, lux: bool) -> String {
    if lux {
        format!("Lowest {lo:.0} lux, highest {hi:.0} lux.")
    } else {
        format!("Lowest {lo:.3}, highest {hi:.3}.")
    }
}

/// Why the figures are not lux, so the levels do not apply.
#[must_use]
pub fn map_not_lux(reasons: &str) -> String {
    format!("Not in lux, so the recommended levels do not apply, because {reasons}.")
}

/// What the meter assumed.
#[must_use]
pub fn map_notes(notes: &str) -> String {
    format!("Assumed: {notes}.")
}

/// Where the levels come from.
#[must_use]
pub fn map_source(title: &str, citation: &str, current_to: &str) -> String {
    format!("Levels: {title}, {citation}, current to {current_to}.")
}

/// The source's tip: its terms, its scope and where it is published.
#[must_use]
pub fn map_source_tooltip(reproduced: &str, scope: &str, url: &str) -> String {
    format!("{reproduced}.\n{scope}\n{url}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn progress_is_a_percentage_and_a_range_names_its_unit() {
        assert_eq!(map_measuring(1, 4), "Measuring the light… 25%");
        assert_eq!(map_measuring(0, 0), "Measuring the light…");
        assert_eq!(
            map_range(12.4, 840.0, true),
            "Lowest 12 lux, highest 840 lux."
        );
        assert_eq!(map_range(0.1, 2.0, false), "Lowest 0.100, highest 2.000.");
        assert!(map_band_tooltip(500.0, &[]).starts_with("500 lux"));
    }
}
