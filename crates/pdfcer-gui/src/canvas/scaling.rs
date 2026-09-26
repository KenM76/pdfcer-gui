//! # `canvas::scaling` — what rides along when a resize scales something
//!
//! Three switches, on the Tool row, for the operator to set before a drag.
//! `OPERATOR_REQUESTS.md` **O51**:
//!
//! Design and rationale: `docs/modules/pdfcer-gui/canvas/scaling.md`.

/// What rides along with the geometry when an annotation is resized.
///
/// Every field maps one-to-one onto a `pdfcer_core::edit::ResizeOptions` field,
/// deliberately. A shell-side name that aggregated two engine options, or
/// inverted one for readability, would be a second vocabulary to keep in step —
/// and the inversion is exactly where such a thing goes wrong silently.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Modifiers {
    /// Scale `/BS /W` by the same factor as the geometry.
    ///
    /// Inkscape's *Scale stroke width*; Illustrator's *Scale Strokes &
    /// Effects*. Both ship it off, and so does this.
    pub scale_stroke_width: bool,
    /// Leave `/RD` — the inset distances — **unscaled**.
    ///
    /// ★ An opt-**out**, matching the engine's own spelling, so that
    /// `Default::default()` is the correct behaviour for every field. Naming it
    /// `scale_rect_differences` here would read better and would put an
    /// inversion between two structs that otherwise correspond exactly, which
    /// is the class of bug nothing catches.
    pub keep_rect_differences: bool,
    /// Proceed when the appearance cannot be rebuilt, accepting a distorted
    /// stroke, rather than taking the engine's named refusal.
    ///
    /// See the module header: this is not a parity item, it is the answer to a
    /// mathematical limit both PDF and SVG have.
    pub allow_distortion: bool,
}

/// The `egui::Memory` key. One key, one struct — see the module header.
const KEY: &str = "pdfcer-resize-modifiers"; // ui-text-exempt: internal memory id, never displayed

/// Read the modifiers, defaulting to the shipped answers.
#[must_use]
pub fn read(ctx: &egui::Context) -> Modifiers {
    ctx.data(|d| d.get_temp::<Modifiers>(egui::Id::new(KEY)))
        .unwrap_or_default()
}

/// Store the modifiers.
pub fn store(ctx: &egui::Context, modifiers: Modifiers) {
    ctx.data_mut(|d| d.insert_temp(egui::Id::new(KEY), modifiers));
}

impl Modifiers {
    /// The engine request these describe.
    ///
    ///
    ///
    /// ⇒ It also made the operator's answer unreachable. Once the switch
    /// exists, deriving the same flag from geometry **overrides them silently**
    /// on exactly the resizes where they were most likely to have an opinion.
    ///
    /// ★ What replaced it is a **worded decline**: the engine's refusal is
    /// caught and turned into a sentence naming both remedies, so the operator
    /// meets a choice rather than a nothing. `app::status::decline` carries it.
    #[must_use]
    pub fn to_options(self) -> pdfcer_core::edit::ResizeOptions {
        // ★ Builders, not a struct literal: `ResizeOptions` is
        // `#[non_exhaustive]`, so the struct form — including
        // `..Default::default()` — is a compile error outside `pdfcer-core`, and
        // the fields being `pub` makes that look like a mistake at this end.
        pdfcer_core::edit::ResizeOptions::new()
            .with_scale_stroke_width(self.scale_stroke_width)
            .with_keep_rect_differences(self.keep_rect_differences)
            .with_allow_appearance_distortion(self.allow_distortion)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// ★★★ **The shipped defaults are the ones O51 argued for.**
    ///
    /// Not a tautology over `Default::default()`: it asserts the three engine
    /// fields, through `to_options`, which is where an inverted mapping would
    /// show up. `keep_rect_differences` is the one that reads backwards —
    /// `false` means `/RD` **does** scale — and a shell that "fixed" that
    /// reading would leave an inset fixed while the rectangle doubled.
    #[test]
    fn the_defaults_are_the_arguments_that_were_accepted() {
        let opts = Modifiers::default().to_options();
        assert!(
            !opts.scale_stroke_width,
            "a line weight is a drafting convention, not a length being scaled"
        );
        assert!(
            !opts.keep_rect_differences,
            "an inset IS a length being scaled, so it scales — this field is an opt-OUT"
        );
        assert!(
            !opts.allow_appearance_distortion,
            "a named refusal beats artwork silently going oval"
        );
    }

    /// ★★ **Every switch reaches the engine**, one for one.
    ///
    /// The failure this guards is a mapping that drops a field: three
    /// checkboxes on the Tool row, two of which do something, and no error
    /// anywhere. It is asserted by turning them on **one at a time**, because
    /// all-three-on would pass on a build that ORed them together.
    #[test]
    fn each_switch_reaches_its_own_engine_field() {
        let stroke = Modifiers {
            scale_stroke_width: true,
            ..Modifiers::default()
        }
        .to_options();
        assert!(stroke.scale_stroke_width);
        assert!(!stroke.keep_rect_differences);
        assert!(!stroke.allow_appearance_distortion);

        let insets = Modifiers {
            keep_rect_differences: true,
            ..Modifiers::default()
        }
        .to_options();
        assert!(!insets.scale_stroke_width);
        assert!(insets.keep_rect_differences);
        assert!(!insets.allow_appearance_distortion);

        let distort = Modifiers {
            allow_distortion: true,
            ..Modifiers::default()
        }
        .to_options();
        assert!(!distort.scale_stroke_width);
        assert!(!distort.keep_rect_differences);
        assert!(distort.allow_appearance_distortion);
    }

    /// ★ A round trip through `egui::Memory`, and the default on an empty one.
    #[test]
    fn the_store_round_trips_and_defaults() {
        let ctx = egui::Context::default();
        assert_eq!(read(&ctx), Modifiers::default());
        let set = Modifiers {
            scale_stroke_width: true,
            keep_rect_differences: true,
            allow_distortion: false,
        };
        store(&ctx, set);
        assert_eq!(read(&ctx), set);
    }
}
