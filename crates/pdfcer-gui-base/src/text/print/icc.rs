//! # `text::print::icc` — the words for the Print dialog's colour management

/// Above the profile choices.
#[must_use]
pub const fn heading() -> &'static str {
    "Colour management"
}

/// Beside the printer profile list.
#[must_use]
pub const fn printer_label() -> &'static str {
    "Printer profile"
}

/// The printer list's first entry.
#[must_use]
pub const fn printer_none() -> &'static str {
    "None: send colour as it is"
}

/// Hover on the printer profile list.
#[must_use]
pub const fn printer_hover() -> &'static str {
    "Converts every sheet into this printer's own colours, so the paper matches the screen. \
     Sheets then go to the printer as pictures."
}

/// Beside the simulated press list.
#[must_use]
pub const fn proof_label() -> &'static str {
    "Simulate press"
}

/// The press list's first entry.
#[must_use]
pub const fn proof_none() -> &'static str {
    "Nothing"
}

/// Hover on the simulated press list.
#[must_use]
pub const fn proof_hover() -> &'static str {
    "Prints how the pages would look on this press or paper (a proof), on the printer you have."
}

/// The last entry of either list: pick a profile file.
#[must_use]
pub const fn other_file() -> &'static str {
    "Other file…"
}

/// The file picker's filter name.
#[must_use]
pub const fn file_filter() -> &'static str {
    "ICC colour profiles"
}

/// Beside the rendering intent list.
#[must_use]
pub const fn intent_label() -> &'static str {
    "Intent"
}

/// The four rendering intents, in `INTENTS` order.
#[must_use]
pub const fn intent_names() -> [&'static str; 4] {
    [
        "Relative colorimetric",
        "Perceptual",
        "Saturation",
        "Absolute colorimetric",
    ]
}

/// The black point compensation switch.
#[must_use]
pub const fn bpc() -> &'static str {
    "Black point compensation"
}

/// Hover on the black point compensation switch.
#[must_use]
pub const fn bpc_hover() -> &'static str {
    "Keeps dark shades apart instead of letting them print as one black. \
     Does nothing with Absolute colorimetric."
}

/// The paper simulation switch.
#[must_use]
pub const fn paper() -> &'static str {
    "Simulate paper colour"
}

/// Hover on the paper simulation switch.
#[must_use]
pub const fn paper_hover() -> &'static str {
    "Prints the simulated paper's tint and black, rather than your paper's white and your \
     printer's black."
}

/// Under the choices when the profile has no black point to compensate for.
#[must_use]
pub const fn bpc_refused() -> &'static str {
    "This profile's black point cannot be found, so black point compensation is off for it."
}

/// Under the choices when a printer profile is chosen and no press is
/// simulated.
#[must_use]
pub const fn preview_unconverted() -> &'static str {
    "The preview shows the page as the screen does. The printer profile is what makes the \
     paper match it."
}

/// Under the choices when a chosen profile cannot be used; printing is
/// refused until it is changed.
#[must_use]
pub fn refused(why: &str) -> String {
    format!("This profile cannot be used, so printing is stopped until it is changed: {why}")
}

/// Under Print as image while colour management is on.
#[must_use]
pub const fn forced_images() -> &'static str {
    "With colour management on, every sheet goes as a picture."
}
