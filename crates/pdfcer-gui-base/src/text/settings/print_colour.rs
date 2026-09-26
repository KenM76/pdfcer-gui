//! # `text::settings::print_colour` — the copy for the two print-ready colour
//! controls, and the field wash
//!
//!
//! ## What is here, and why these three together
//!
//! Two of them are one subject from two sides — *what overprints a spot colour*
//! and *what a spot colour is* — and both are reached by the same symptom: white
//! behaving unexpectedly on a print-ready drawing. Keeping their copy adjacent is
//! the same argument that puts their controls adjacent in the window.
//!
//! The third, the **field wash**, is here because it arrived in the same
//! commit and pushed the same file over the line. That is an honest reason and a
//! weak one, and it is stated rather than dressed up: if this module grows, the
//! wash is the entry to move out, because it is a *display* preference and has
//! nothing to do with ink.
//!
//! ## The rule this copy follows
//!
//! Every setting answers three obligations — a **title**, what happens if you
//! **never touch it**, and what it **costs or does not affect**. `super`'s
//! header carries the argument; the catalog test enforces it mechanically.
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/text/settings/print_colour.md`.

/// **Shading the fillable fields** — `OPERATOR_REQUESTS.md` O96.
#[must_use]
pub const fn field_shade_title() -> &'static str {
    "Show which boxes can be filled in"
}

/// What happens if you never touch it.
#[must_use]
pub const fn field_shade_silence() -> &'static str {
    "Fillable form fields are washed with a pale tint so you can see them at a glance, the way Acrobat does."
}

/// What it costs, and what it does not affect.
#[must_use]
pub const fn field_shade_radius() -> &'static str {
    "On screen only. It is never printed, never exported, never saved into the document, and it changes nothing about how the page itself is drawn."
}

/// The toggle's own label.
#[must_use]
pub const fn field_shade_label() -> &'static str {
    "Shade fillable fields"
}

/// The note under the toggle.
#[must_use]
pub const fn field_shade_note() -> &'static str {
    "Turn this off for a clean view of the page. The fields still work — the pointer still changes over one, and clicking still fills it."
}

/// **Whether a spot ink keeps its own plate, or is mixed down first.**
#[must_use]
pub const fn spot_model_title() -> &'static str {
    "Spot inks in print-ready files"
}

/// What happens if you never touch it.
#[must_use]
pub const fn spot_model_silence() -> &'static str {
    "A spot colour keeps its own printing plate, so anything overprinting it leaves it showing through — which is what a press does."
}

/// What it costs, and what it does not affect.
#[must_use]
pub const fn spot_model_radius() -> &'static str {
    "Changes how spot colours are drawn and printed. It never changes the file, and it does nothing at all unless a page uses a named ink AND asks for overprint — which outside print-ready artwork is almost never."
}

/// One model's name.
#[must_use]
pub const fn spot_model_label(
    model: pdfcer_core::settings::SpotColorantDeviceModel,
) -> &'static str {
    use pdfcer_core::settings::SpotColorantDeviceModel as M;
    match model {
        M::SimulateSeparations => "Keep the ink on its own plate (pdfcer's default)",
        M::AlternateSpaceSubstitution => "Mix it down, the way a screen viewer does",
        // `#[non_exhaustive]`, so a newer engine may add a model. Named as
        // unknown rather than folded onto a neighbour — `blend_space_label`
        // makes that argument once and it is the same one.
        _ => "A newer pdfcer added this option; this build cannot describe it",
    }
}

/// One model's description.
#[must_use]
pub const fn spot_model_note(
    model: pdfcer_core::settings::SpotColorantDeviceModel,
) -> &'static str {
    use pdfcer_core::settings::SpotColorantDeviceModel as M;
    match model {
        M::SimulateSeparations => {
            "What a printing press does: the named ink has its own plate, so white printed over it knocks nothing out. Choose this to proof what will come off the press."
        }
        M::AlternateSpaceSubstitution => {
            "What Acrobat shows on screen: the named ink is converted to ordinary process colour before anything is drawn, so white printed over it knocks it out. Choose this to match what a colleague sees on their monitor."
        }
        _ => "This build cannot describe it, so it is left alone rather than guessed at.",
    }
}
