//! # `app::prefs::chrome` — how big the program's own controls are drawn
//!
//! One preference, and it is the only one in this store that is an
//! **accessibility** control rather than a taste or a speed trade.
//!
//! ## ★ Why this existed nowhere, which is the interesting part
//!
//! `NO_SURFACE.md` §4 recorded it as a single line against the icon size:
//!
//! > `Icon size` · 16.0 pt · `icons/mod.rs:171` · none — **no UI-scale or
//! > base-font-size control anywhere**
//!
//! That is not a hard-coded constant like the others in that table. It is a
//! whole capability that nothing in the shell had: `egui` has offered
//! `Context::set_zoom_factor` throughout, and **no line in this crate ever
//! called it**. Every control, every label, every icon and every panel in this
//! shell has been drawn at exactly one size, on every machine, for the whole
//! life of the project.
//!
//! The reason it went unnoticed is worth recording, because it is a general
//! trap: `egui` has a *built-in* `Ctrl` + `+`/`-`/`0` handler for precisely
//! this, so on a stock `eframe` application the capability appears to exist
//! without anyone building it. This shell **switches that handler off** —
//! deliberately, in [`crate::app::configure_context`], because in a document
//! viewer those chords mean *page* zoom, as they do in every browser, in
//! Acrobat and in every other PDF reader. So the one path that would have
//! surfaced it was closed for a good reason, and closing it removed a feature
//! nobody had noticed was being provided.
//!
//! **A framework default you switch off may have been carrying a capability you
//! never decided to have.** That is the reusable half.
//!
//! ## Where the reference applications put it, and where the chords went
//!
//! Standing instruction 4 — *match Inkscape, Acrobat and SolidWorks, but first
//! ask which of them actually has the surface.*
//!
//! | application | UI scale control | chord |
//! |---|---|---|
//! | **Inkscape** | Preferences ▸ Interface ▸ *Interface scale* | none |
//! | **SolidWorks** | not a scale control; it follows the Windows display setting | none |
//! | **Acrobat** | no UI-scale control at all | — |
//!
//! So: **a settings control and no chord**, unanimously among the two that
//! have the surface. That is also the only answer available here, because
//! `Ctrl` + `+`/`-`/`0` are taken by page zoom and `Ctrl+1`/`2`/`3` are the
//! mode selector — an invented third family would be a chord nobody's muscle
//! memory has and would collide with the two that do.
//!
//! ## ★ It lives in the *Appearance* group, beside the theme
//!
//! Not in *Drawing the page*, which holds this store's other four preferences.
//! The window's groups are a **navigation model** — an operator arrives with a
//! symptom and the heading is how the symptom finds its setting — and the
//! symptom here is *"the program's text is too small to read"*, which is a
//! question about the window, not about the page.
//!
//! Theme and UI scale are the two settings that change **the program's own
//! appearance and nothing about the document**, and they belong together for
//! that reason. It is also the only group whose two members share the live
//! preview below.

/// The smallest scale offered.
///
/// **0.8, not lower.** Below about four fifths the ribbon's two-row groups
/// stop being able to fit their captions and the dock's tab labels start to
/// clip — so a smaller value would not show the operator more, it would show
/// them the same controls with the words cut off. `egui` will accept 0.1 and
/// the result is unusable, which is exactly why the range is stated here
/// rather than left to the widget.
pub const MIN_UI_SCALE: f32 = 0.8;

/// The largest scale offered.
///
/// **2.0.** At double size the shipped 1100 x 800 window holds the ribbon, a
/// status bar and very little else, which is the point at which the control
/// stops helping and starts hiding the document. Someone who needs more than
/// this needs their operating system's display scaling, which multiplies with
/// this one and is the setting built for the job.
pub const MAX_UI_SCALE: f32 = 2.0;

/// The shipped scale.
///
/// **Exactly 1.0**, which means *whatever the operating system says* — see
/// [`crate::app::prefs::Prefs::ui_scale`] on why this multiplies rather than
/// replaces. The standing rule for a capability becoming choosable: a build
/// that omits nothing must behave as the build before the choice existed, and
/// before this preference the shell simply never touched `zoom_factor`.
pub const DEFAULT_UI_SCALE: f32 = 1.0;

/// The step the control moves in.
///
/// A twentieth. Fine enough that an operator can land on a size that feels
/// right rather than choosing between two that do not, and coarse enough that
/// the value reads as a percentage they could describe to somebody else.
pub const UI_SCALE_STEP: f32 = 0.05;

/// Round a scale to the nearest [`UI_SCALE_STEP`] and clamp it to the offered
/// range.
///
/// # Why the file's value is rounded and not merely clamped
///
/// Because the file is hand-editable and a slider is not. An operator who
/// writes `ui_scale = 1.234` gets a value the control cannot represent, so the
/// next time they touch that slider it would silently jump — a change they did
/// not make, to a setting they did. Rounding at load makes the file and the
/// control agree from the first frame, and the loader reports it as a
/// [`crate::app::prefs::PrefNote::Clamped`] so the substitution is not silent.
///
/// Returns the rounded value; the caller compares it against the original to
/// decide whether to report.
#[must_use]
pub fn normalise_ui_scale(raw: f32) -> f32 {
    let clamped = raw.clamp(MIN_UI_SCALE, MAX_UI_SCALE);
    (clamped / UI_SCALE_STEP).round() * UI_SCALE_STEP
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The shipped scale is the identity, and it is reachable on its control.
    ///
    /// Both halves matter. Identity is the "a build that omits nothing behaves
    /// as it did before" rule; reachability is the third instance in this
    /// project of a default that must sit inside its own widget's range, or
    /// the first operator to open the window has their value rewritten without
    /// touching anything.
    #[test]
    fn the_shipped_scale_is_the_identity_and_is_reachable() {
        assert!((DEFAULT_UI_SCALE - 1.0).abs() < f32::EPSILON);
        assert!((MIN_UI_SCALE..=MAX_UI_SCALE).contains(&DEFAULT_UI_SCALE));
        assert!(
            (normalise_ui_scale(DEFAULT_UI_SCALE) - DEFAULT_UI_SCALE).abs() < 1e-6,
            "the shipped scale is not on the control's own step"
        );
    }

    /// ★ Normalising is idempotent.
    ///
    /// The property that makes the load path safe to run on its own output —
    /// which it is, every time pdfcer saves and reloads. A rounding that moved
    /// a value it had already produced would make the preference drift a step
    /// per restart, which is the kind of defect that takes a fortnight to be
    /// noticed and is then very hard to attribute.
    #[test]
    fn normalising_twice_changes_nothing() {
        let mut value = MIN_UI_SCALE;
        while value <= MAX_UI_SCALE {
            let once = normalise_ui_scale(value);
            let twice = normalise_ui_scale(once);
            assert!(
                (once - twice).abs() < 1e-6,
                "{value} normalised to {once} and then to {twice}"
            );
            value += UI_SCALE_STEP / 3.0;
        }
    }

    /// Out-of-range values are pulled to the ends, in both directions.
    #[test]
    fn an_out_of_range_scale_clamps_to_the_offered_range() {
        assert!((normalise_ui_scale(0.1) - MIN_UI_SCALE).abs() < 1e-6);
        assert!((normalise_ui_scale(99.0) - MAX_UI_SCALE).abs() < 1e-6);
    }

    /// ★ Every value the control can produce survives normalising unchanged.
    ///
    /// The weld between the widget's step and the file's grammar. If the
    /// slider could land on a value the loader would round away, the operator
    /// would set a scale, restart, and find a different one — with the file
    /// on disk holding what they chose and the program showing something else.
    #[test]
    fn every_value_the_control_offers_round_trips() {
        let steps = ((MAX_UI_SCALE - MIN_UI_SCALE) / UI_SCALE_STEP).round() as i32;
        for i in 0..=steps {
            #[allow(clippy::cast_precision_loss)]
            let value = UI_SCALE_STEP.mul_add(i as f32, MIN_UI_SCALE);
            let back = normalise_ui_scale(value);
            assert!(
                (value - back).abs() < 1e-5,
                "the control can produce {value}, which the loader rewrites to {back}"
            );
        }
    }
}
