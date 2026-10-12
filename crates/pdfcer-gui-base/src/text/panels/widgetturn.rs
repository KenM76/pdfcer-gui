//! # `text::panels::widgetturn` — a form field's box at a free angle, and
//! its opacity

/// The Properties readout under the quarter-turn line, while turned.
#[must_use]
pub fn turned(degrees: f64) -> String {
    format!("Tilted a further {} anticlockwise.", angle(degrees))
}

/// The button that stands a tilted box back up.
#[must_use]
pub const fn stand_upright() -> &'static str {
    "Stand upright"
}

/// The hover on Turn left and Turn right while the box is tilted.
#[must_use]
pub const fn quarter_needs_upright() -> &'static str {
    "Stand the box upright first. A quarter turn is stored where every viewer keeps it, and it \
     cannot be combined with a tilt."
}

/// The status line after a tilt.
#[must_use]
pub fn tilted(now: f64) -> String {
    if now == 0.0 {
        "The box stands upright again.".to_owned()
    } else {
        format!("Tilted to {} anticlockwise.", angle(now))
    }
}

/// The refusal for a box with no appearance to turn.
#[must_use]
pub const fn needs_appearance() -> &'static str {
    "This box has no drawn appearance to tilt. Fill it in or restyle it first, then tilt it."
}

/// The refusal for a box that shares its appearance in an encrypted file.
#[must_use]
pub const fn shared_appearance() -> &'static str {
    "This box shares its drawing with another box, and the file is encrypted, so it cannot be \
     given its own copy to tilt. Nothing was changed."
}

/// The refusal for an operation that cannot keep a tilt.
#[must_use]
pub fn needs_upright(degrees: f64) -> String {
    format!(
        "This box is tilted {}. Stand it upright first, then resize or quarter-turn it. Nothing \
         was changed.",
        angle(degrees)
    )
}

/// The Opacity row's label.
#[must_use]
pub const fn opacity() -> &'static str {
    "Opacity"
}

/// The noun the receipt names after an opacity change.
#[must_use]
pub const fn touched_opacity() -> &'static str {
    "the opacity"
}

/// The Opacity row's button that removes the setting.
#[must_use]
pub const fn opacity_clear() -> &'static str {
    "Clear"
}

/// The hover on the Opacity row.
#[must_use]
pub const fn opacity_hint() -> &'static str {
    "How solid the box is drawn, 0 to 100 %. Clear removes the setting, which draws it solid."
}

/// The status line after an opacity change below 100 %.
#[must_use]
pub const fn opacity_disclosed() -> &'static str {
    "Some readers ignore a form field's opacity when they draw it, so it can look solid \
     elsewhere."
}

/// An angle in degrees, to one place when it has a fraction.
fn angle(degrees: f64) -> String {
    let rounded = (degrees * 10.0).round() / 10.0;
    if rounded.fract() == 0.0 {
        format!("{rounded:.0}\u{00b0}")
    } else {
        format!("{rounded:.1}\u{00b0}")
    }
}
