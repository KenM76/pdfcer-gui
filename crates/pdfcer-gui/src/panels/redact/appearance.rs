//! # `panels::redact::appearance` — what a redaction looks like once applied
//!
//! The operator's **one** choice of fill colour and overlay caption, held for
//! the whole panel and applied to every mark authored from it.
//!
//! ## ★ This shipped as three `None`s, and the reason it did is worth keeping
//!
//! `NO_SURFACE.md` listed fill, overlay text and quadding as three tunables
//! with no control, and the obvious task was to add three controls. Reading
//! what consumed each value said otherwise, and on 2026-08-17 all three were
//! recorded **blocked** rather than unbuilt:
//!
//! * `fill` was honoured only when the shell built the spec itself.
//!   `EditSession::author_text_matches` hard-coded `fill: None`, so every mark
//!   from *Find and mark* ignored it. A swatch would have worked on whole-page
//!   marks and been silently dropped on searched ones.
//! * `overlay_text` was **written into the PDF and never read**. An operator
//!   would type *REDACTED*, apply, and get plain black boxes with nothing said.
//! * `quadding` justifies the overlay text and had nothing to justify.
//!
//! Both were filed. Both came back **fixed** the same day (`a7210a4`,
//! `a705d14`), and both replies end *"build the control"*. This module is that.
//!
//! The lesson survives the unblocking and is the reason this paragraph stays:
//! **an engine field that exists, is documented, and is written into the file
//! is not evidence that anything reads it.** Two of these three reached the PDF
//! the whole time.
//!
//! ## ★★ `fill: None` changed meaning, and the change is silent and dangerous
//!
//! Under the old engine `None` meant *black box*; its doc comment said so.
//! Under `a705d14` it means **transparent** — ISO 32000-1 Table 192 says an
//! absent `/IC` leaves the interior transparent, and the old behaviour was
//! simply wrong.
//!
//! For this shell that is a behaviour change of the worst possible shape. The
//! content is still removed, so it is not a security failure — but the
//! operator sees **no box**, which reads as *the redaction did nothing*, on
//! the one operation they cannot undo and most need to trust.
//!
//! So [`RedactAppearance::fill`] here defaults to an explicit
//! `Some(Color::Gray(0.0))` rather than to `None`. That is the standing rule
//! for a capability becoming choosable — *a build that omits nothing must
//! behave as it did before the choice existed* — applied to a default that
//! moved underneath us rather than to one we changed.
//!
//! `Transparent` is still offered, because it is what the standard describes
//! and because there is a real use for it (removing content without announcing
//! where), but it is a thing the operator chooses rather than a thing they get
//! by not choosing.

use crate::text::redact as t;
use pdfcer_core::annot_author::{Color, RedactAppearance};
use pdfcer_core::vartext::Quadding;

/// The region the fill swatch publishes, so a check can find and drive it.
pub const REGION_FILL: &str = "redact.appearance.fill"; // ui-text-exempt: trace region name, never displayed
/// The region the overlay-text field publishes.
pub const REGION_OVERLAY: &str = "redact.appearance.overlay"; // ui-text-exempt: trace region name, never displayed

/// The longest overlay caption offered.
///
/// **64 characters.** Not a format limit — `/OverlayText` is a PDF string and
/// has none worth naming — but a legibility one: the engine lays the caption
/// out inside the marked box and auto-sizes it into a 4–12 pt clamp, so a
/// sentence over a one-line mark becomes unreadably small or is clipped. 64 is
/// comfortably longer than every caption anybody actually writes (*REDACTED*,
/// *Exemption 5*, a case number) and short enough that the operator meets the
/// bound while typing rather than in the applied document.
pub const MAX_OVERLAY_CHARS: usize = 64;

/// The operator's choice of how an applied redaction looks.
///
/// Held on [`super::RedactUi`] — the panel's own state, not the document's —
/// because it is a property of *what the operator is about to author*, exactly
/// like `canvas::markup::Pen`. It is read at the moment a mark is created and
/// never afterwards.
///
/// # ★ Why the fill is an enum and not an `Option<Color>`
///
/// Because the engine's `Option<Color>` has three operator-facing meanings and
/// only two of them are colours: *black*, *some other colour*, and *no box at
/// all*. An `Option` in the UI would put "transparent" and "the operator has
/// not picked yet" into the same value, and this is the feature where that
/// ambiguity is least affordable.
#[derive(Debug, Clone, PartialEq)]
pub struct Appearance {
    /// What fills the redacted region on apply.
    pub fill: Fill,
    /// The caption drawn over the fill, or empty for none.
    ///
    /// Stored as a `String` rather than an `Option<String>` because that is
    /// what a text field edits; [`Self::to_core`] is the single place the
    /// empty-means-none rule is applied, so a caller cannot get it wrong.
    pub overlay_text: String,
    /// How the caption is justified inside the box.
    pub quadding: Quadding,
}

impl Default for Appearance {
    /// **Black, no caption, left-justified** — what every mark this shell has
    /// ever authored applied as, before the choice existed.
    ///
    /// Black is written **explicitly**, and that is the whole point: see the
    /// module header. Leaving it `None` would inherit the engine's new
    /// transparent default and silently change what every existing operator's
    /// redactions look like.
    fn default() -> Self {
        Self {
            fill: Fill::Black,
            overlay_text: String::new(),
            quadding: Quadding::Left,
        }
    }
}

/// What fills a redacted region once the redaction is applied.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Fill {
    /// A solid black box. The convention, and what pdfcer has always drawn.
    Black,
    /// A solid white box, for a redaction that should read as blank paper.
    White,
    /// A solid box in a colour the operator picked.
    ///
    /// Carried as RGB components in `0.0..=1.0`, matching
    /// `pdfcer_core::annot_author::Color::Rgb`, so no conversion happens
    /// anywhere but [`Fill::to_core`].
    Custom(f64, f64, f64),
    /// **No box at all.** The content is removed and nothing marks where it
    /// was.
    ///
    /// Offered because ISO 32000-1 Table 192 describes it and because there is
    /// a real use — removing content without advertising its position — but
    /// never the default, and the panel says out loud what it does.
    Transparent,
}

impl Fill {
    /// The engine's form.
    #[must_use]
    pub const fn to_core(self) -> Option<Color> {
        match self {
            Self::Black => Some(Color::Gray(0.0)),
            Self::White => Some(Color::Gray(1.0)),
            Self::Custom(r, g, b) => Some(Color::Rgb(r, g, b)),
            Self::Transparent => None,
        }
    }

    /// Whether a caption drawn over this fill would be legible.
    ///
    /// # ★ This exists because the engine told us it would not be
    ///
    /// `a7210a4`'s reply carries the warning verbatim:
    ///
    /// > **Black-on-dark is illegible, and it is our gap, not yours.** The
    /// > `/DA` we author hard-codes black text. Wire a fill-colour picker, let
    /// > someone choose a dark red, and the caption will be black on dark red
    /// > — we saw it in our own verification render. **There is no
    /// > overlay-text colour on the API yet.**
    ///
    /// So the panel must not let an operator walk into it silently. The
    /// predicate is deliberately crude — relative luminance against a
    /// mid-point — because the failure it guards is crude: black text on
    /// anything dark. A precise WCAG ratio would imply a precision the engine
    /// cannot honour, since the text colour is not ours to set.
    ///
    /// **Black fill with a caption is the loudest case** and is not special-
    /// cased away: black on black is invisible, and an operator who chooses
    /// the default fill and types a caption deserves to be told before they
    /// apply rather than after.
    #[must_use]
    pub fn caption_would_be_legible(self) -> bool {
        let Some(colour) = self.to_core() else {
            // Transparent: the caption is drawn over whatever the page had,
            // which this shell cannot know. Not flagged — an unknown backdrop
            // is not a known-bad one, and a warning that fires on every
            // transparent redaction would be noise.
            return true;
        };
        let luminance = match colour {
            Color::Gray(g) => g,
            // Rec. 709 luma. Approximate on purpose: the question is "is this
            // dark?", not "what is its exact perceived lightness?".
            Color::Rgb(r, g, b) => 0.2126f64.mul_add(r, 0.7152f64.mul_add(g, 0.0722 * b)),
            // Unreachable from `Fill`, and answered rather than panicking: a
            // future fill that is CMYK should degrade to "assume legible"
            // rather than crash a panel.
            Color::Cmyk(..) => 1.0,
        };
        // 0.5 rather than a tuned threshold. The engine draws black text; the
        // question is whether the backdrop is nearer white than black.
        luminance > 0.5
    }
}

impl Appearance {
    /// The engine's form, ready for `add_redaction` or either `_styled` verb.
    ///
    /// The single place `""` becomes `None`, so no call site has to remember
    /// that an empty field means no caption.
    #[must_use]
    pub fn to_core(&self) -> RedactAppearance {
        let trimmed = self.overlay_text.trim();
        RedactAppearance {
            fill: self.fill.to_core(),
            overlay_text: if trimmed.is_empty() {
                None
            } else {
                Some(trimmed.to_owned())
            },
            quadding: self.quadding,
        }
    }

    /// Whether the operator has asked for a caption at all.
    ///
    /// Used to decide whether the justification control and the legibility
    /// warning are drawn — neither has anything to say about a redaction with
    /// no caption, and drawing them anyway is how a panel accumulates controls
    /// that do nothing.
    #[must_use]
    pub fn has_overlay(&self) -> bool {
        !self.overlay_text.trim().is_empty()
    }

    /// Whether the operator should be warned before applying.
    ///
    /// True only when there **is** a caption and the fill would make it
    /// illegible. See [`Fill::caption_would_be_legible`].
    #[must_use]
    pub fn caption_is_illegible(&self) -> bool {
        self.has_overlay() && !self.fill.caption_would_be_legible()
    }
}

/// **Draw the appearance controls**, editing the panel's own state in place.
///
/// # It edits in place and raises nothing
///
/// No `Action` and no return value, exactly as `canvas::markup::swatch` does
/// for the pen. The funnel's invariant is that no path runs from a widget to a
/// **document**, and this touches none: it sets what the *next* mark will be
/// authored with. There is nothing to undo and nothing to order against.
///
/// # ★ Collapsed by default
///
/// The shipped appearance — a plain black box, no caption — is what almost
/// every redaction wants, so these controls should cost nothing until an
/// operator asks for them. This panel has already shipped its primary verb
/// below the bottom of its own pane once, and everything added to it now is
/// measured against that.
pub fn show(ui: &mut egui::Ui, state: &mut crate::panels::PanelsState) {
    egui::CollapsingHeader::new(egui::RichText::new(t::appearance_heading()))
        .default_open(false)
        .show(ui, |ui| {
            ui.label(egui::RichText::new(t::appearance_intro()).small().weak());
            ui.add_space(6.0);
            controls(ui, &mut state.redact_mut().appearance);
        });
}

/// The controls themselves, over a borrowed [`Appearance`].
///
/// Split from [`show`] so the layout can be exercised without a
/// `PanelsState` — and, more usefully, so the *order* of the controls is
/// readable in one screen. It is: what covers it, what is written on it, how
/// that is lined up. Each depends on the one before, and two of the three
/// disappear when the one before makes them meaningless.
fn controls(ui: &mut egui::Ui, appearance: &mut Appearance) {
    // ---- the fill --------------------------------------------------------
    ui.horizontal(|ui| {
        ui.label(t::fill_label());
        for option in [Fill::Black, Fill::White, Fill::Transparent] {
            // A `selectable_value` rather than a radio: three named
            // alternatives on one row, which is what the settings window's
            // `option` helper would draw vertically and there is no room for
            // that here.
            ui.selectable_value(&mut appearance.fill, option, t::fill_option_label(option));
        }
        // ★ The custom colour is a SWATCH, not a fourth segment, because the
        // only useful preview of a colour is the colour. Seeded from whatever
        // is currently chosen so that switching from Black to a custom colour
        // starts somewhere sensible rather than at an arbitrary hue.
        let mut rgb = match appearance.fill {
            Fill::Custom(r, g, b) => [r as f32, g as f32, b as f32],
            Fill::White => [1.0, 1.0, 1.0],
            _ => [0.0, 0.0, 0.0],
        };
        let swatch = ui
            .color_edit_button_rgb(&mut rgb)
            .on_hover_text(t::fill_option_label(Fill::Custom(0.0, 0.0, 0.0)));
        crate::diag::ui_rect(REGION_FILL, swatch.rect);
        if swatch.changed() {
            appearance.fill = Fill::Custom(f64::from(rgb[0]), f64::from(rgb[1]), f64::from(rgb[2]));
        }
    });
    if appearance.fill == Fill::Transparent {
        // Said at the control and not in a tooltip: a tooltip is not read
        // before a choice is made, and this is the choice an operator can
        // misread as "do not redact".
        ui.label(egui::RichText::new(t::fill_transparent_note()).small());
    }

    ui.add_space(8.0);

    // ---- the caption -----------------------------------------------------
    ui.label(t::overlay_label());
    let field = ui.add(
        egui::TextEdit::singleline(&mut appearance.overlay_text)
            .hint_text(t::overlay_hint())
            .char_limit(MAX_OVERLAY_CHARS),
    );
    crate::diag::ui_rect(REGION_OVERLAY, field.rect);

    // Everything below is about a caption, so none of it is drawn when there
    // is not one. A justification control for text that does not exist is the
    // "control governing a control governing nothing" this module's header
    // names.
    if !appearance.has_overlay() {
        return;
    }

    // ★ The legibility warning, and it is a DISCLOSURE rather than advice —
    // `.small()` without `.weak()`, the same weight the settings window
    // reserves for something pdfcer owes the operator rather than something it
    // is explaining. The engine cannot colour this text and told us so; an
    // operator who applies a black caption onto a black box has lost nothing
    // recoverable, but they have lost the caption.
    if appearance.caption_is_illegible() {
        ui.add_space(4.0);
        ui.label(
            egui::RichText::new(t::overlay_illegible_warning())
                .small()
                .color(egui_shell::theme::Theme::of(ui.ctx()).palette.danger),
        );
    }

    ui.add_space(6.0);
    ui.horizontal(|ui| {
        ui.label(t::quadding_label());
        for option in [Quadding::Left, Quadding::Center, Quadding::Right] {
            ui.selectable_value(
                &mut appearance.quadding,
                option,
                t::quadding_option_label(option),
            );
        }
    });
    ui.add_space(4.0);
    ui.label(egui::RichText::new(t::overlay_bound()).small().weak());
}

#[cfg(test)]
mod tests {
    use super::*;

    /// ★ **The shipped appearance is an explicit black box.**
    ///
    /// The regression test for the engine's default changing underneath this
    /// shell. `a705d14` made `RedactSpec::fill = None` mean **transparent**
    /// where it had meant black — correctly, per Table 192 — so a shell that
    /// passed `None` would silently stop drawing the box over every redaction
    /// its operators applied. The content would still be removed, which is why
    /// nothing would fail; the operator would simply see no evidence that
    /// anything had happened, on the one operation they cannot undo.
    ///
    /// Asserted against the ENGINE's type rather than against `Fill::Black`,
    /// so it fails if the mapping is what breaks rather than the default.
    #[test]
    fn the_shipped_fill_is_an_explicit_black_box_not_a_none() {
        let core = Appearance::default().to_core();
        assert_eq!(
            core.fill,
            Some(Color::Gray(0.0)),
            "the default redaction fill must be EXPLICIT black — `None` now means \
             transparent, so a redaction would remove the content and draw nothing over it"
        );
        assert!(
            core.overlay_text.is_none(),
            "no caption unless the operator writes one — inventing a default would put \
             words on their page that they did not write"
        );
    }

    /// An empty or whitespace caption is no caption.
    ///
    /// The rule lives in one place so no call site can send `Some("   ")` to
    /// the engine, which would author an `/OverlayText`, burn in a box of
    /// spaces, and count as a caption in the report.
    #[test]
    fn a_blank_caption_is_no_caption() {
        for blank in ["", "   ", "\t\n "] {
            let a = Appearance {
                overlay_text: blank.to_owned(),
                ..Appearance::default()
            };
            assert!(a.to_core().overlay_text.is_none(), "{blank:?}");
            assert!(!a.has_overlay(), "{blank:?}");
        }
        let a = Appearance {
            overlay_text: "  REDACTED  ".to_owned(),
            ..Appearance::default()
        };
        assert_eq!(
            a.to_core().overlay_text.as_deref(),
            Some("REDACTED"),
            "a caption must be trimmed, or the justification applies to the spaces"
        );
    }

    /// Every fill maps to the engine colour it names.
    #[test]
    fn every_fill_maps_to_its_engine_colour() {
        assert_eq!(Fill::Black.to_core(), Some(Color::Gray(0.0)));
        assert_eq!(Fill::White.to_core(), Some(Color::Gray(1.0)));
        assert_eq!(
            Fill::Custom(0.8, 0.1, 0.1).to_core(),
            Some(Color::Rgb(0.8, 0.1, 0.1))
        );
        assert_eq!(
            Fill::Transparent.to_core(),
            None,
            "transparent is the ONE fill that is `None`, and it is chosen rather than \
             defaulted into"
        );
    }

    /// ★ **A caption on a dark fill is flagged, including on the default.**
    ///
    /// The engine hard-codes black text in the `/DA` it authors and told us so
    /// when it shipped the feature. Black-on-black is the case an operator
    /// reaches by accident — keeping the default fill and typing a caption —
    /// so it must be flagged rather than special-cased as "well, that is the
    /// default".
    #[test]
    fn a_caption_on_a_dark_fill_is_flagged() {
        let dark = Appearance {
            overlay_text: "REDACTED".to_owned(),
            ..Appearance::default()
        };
        assert!(
            dark.caption_is_illegible(),
            "black text on the default black box is invisible and must be warned about"
        );

        let dark_red = Appearance {
            fill: Fill::Custom(0.4, 0.0, 0.0),
            overlay_text: "REDACTED".to_owned(),
            quadding: Quadding::Left,
        };
        assert!(
            dark_red.caption_is_illegible(),
            "the engine's own reply names dark red as the case it saw go wrong"
        );

        let light = Appearance {
            fill: Fill::White,
            overlay_text: "REDACTED".to_owned(),
            quadding: Quadding::Left,
        };
        assert!(!light.caption_is_illegible(), "black on white is legible");
    }

    /// …and no caption is never flagged, whatever the fill.
    ///
    /// The other half, and the one that stops the warning becoming permanent
    /// furniture: the default appearance is a black box with no caption, which
    /// is the commonest redaction there is and has nothing wrong with it.
    #[test]
    fn a_fill_with_no_caption_is_never_flagged() {
        for fill in [
            Fill::Black,
            Fill::White,
            Fill::Custom(0.0, 0.0, 0.0),
            Fill::Transparent,
        ] {
            let a = Appearance {
                fill,
                ..Appearance::default()
            };
            assert!(
                !a.caption_is_illegible(),
                "{fill:?} with no caption has nothing to be illegible"
            );
        }
    }
}
