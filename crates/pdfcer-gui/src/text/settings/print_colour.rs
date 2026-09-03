//! # `text::settings::print_colour` — the copy for the two print-ready colour
//! controls, and the field wash
//!
//! Split out of [`super::look`] on 2026-09-02 under R2, when that file crossed
//! the 1,500-line ceiling and neither of the day's two additions could land
//! without a seam.
//!
//! ## What is here, and why these three together
//!
//! Two of them are one subject from two sides — *what overprints a spot colour*
//! and *what a spot colour is* — and both are reached by the same symptom: white
//! behaving unexpectedly on a print-ready drawing. Keeping their copy adjacent is
//! the same argument that puts their controls adjacent in the window.
//!
//! ★ The third, the **field wash**, is here because it arrived in the same
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

/// **Shading the fillable fields** — `OPERATOR_REQUESTS.md` O96.
///
/// ★ The copy leads with what the operator gains, not with what is drawn: they
/// are not looking for *"a wash over rectangles"*, they are looking for *"which
/// of these boxes can I type in"*. The radius line carries the fact that matters
/// most and is the one nobody would assume — **it never reaches the file.**
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
///
/// # ★★★ Why this is a real choice and not a bug with a switch
///
/// `SpotColorantDeviceModel`, new in `pdfcer-core 0.20`. The two values are
/// **both conformant** and they disagree about what a spot colour is:
///
/// * *simulate separations* renders for a device that **has** the ink — the
///   spot keeps its own plate and overprint preserves it. ISO 32000-2 §10.8.3.
///   The engine's default, and the one the print-conformance corpus expects.
/// * *alternate space substitution* renders for the **actual composite device**,
///   which has no such ink: the separation is converted through its tint
///   transform the moment its space is set and is ordinary process ink from then
///   on. ISO 32000-1 §8.6.6.4's `shall`, and what Acrobat's default view shows.
///
/// ★★ The visible difference is narrow and sharp: a **white object over a spot
/// colour knocks the ink out** under one model and **preserves it** under the
/// other. That is the whole of it, it only happens under overprint, and it is
/// the difference between a proof that matches the press and one that matches
/// the screen.
///
/// ★ The copy below leads with the SYMPTOM rather than the standard, because
/// the operator arriving here got here by seeing white behave unexpectedly on a
/// print-ready drawing — not by reading §10.8.3.
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
        // ★ `#[non_exhaustive]`, so a newer engine may add a model. Named as
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
