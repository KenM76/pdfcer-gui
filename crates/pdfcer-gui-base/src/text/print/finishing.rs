//! # `text::print::finishing` — the words for staple, punch and output bin

use crate::printspooler::finishing::{Edge, Staple};

/// Above the staple choice.
#[must_use]
pub const fn staple_label() -> &'static str {
    "Staple"
}

/// Above the hole-punch choice.
#[must_use]
pub const fn punch_label() -> &'static str {
    "Hole punch"
}

/// Above the output-bin choice.
#[must_use]
pub const fn bin_label() -> &'static str {
    "Output tray"
}

/// The choice that asks the printer for nothing.
#[must_use]
pub const fn none() -> &'static str {
    "None"
}

/// The bin choice that leaves the printer's own.
#[must_use]
pub const fn bin_default() -> &'static str {
    "Printer's choice"
}

/// One staple position.
#[must_use]
pub const fn staple(s: Staple) -> &'static str {
    match s {
        Staple::TopLeft => "Top left",
        Staple::TopRight => "Top right",
        Staple::BottomLeft => "Bottom left",
        Staple::BottomRight => "Bottom right",
        Staple::DualLeft => "Two on the left",
        Staple::DualRight => "Two on the right",
        Staple::DualTop => "Two along the top",
        Staple::DualBottom => "Two along the bottom",
        Staple::Saddle => "Saddle stitch",
        _ => "Other",
    }
}

/// One punch edge.
#[must_use]
pub const fn edge(e: Edge) -> &'static str {
    match e {
        Edge::Left => "Left edge",
        Edge::Top => "Top edge",
        Edge::Right => "Right edge",
        Edge::Bottom => "Bottom edge",
        _ => "Other edge",
    }
}

/// Under the choices: the driver is the judge, and refusal stops the job.
#[must_use]
pub const fn finishing_note() -> &'static str {
    "Offered by the printer's driver. If it refuses any of these at print time, nothing is sent and the reason is shown."
}
