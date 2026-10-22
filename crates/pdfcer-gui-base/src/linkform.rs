//! # `linkform` — the "goes to" and border controls a link is made with
//!
//! One form, drawn by both the Add-link window and the Properties link
//! section, so a link is made and changed with the same controls and the
//! same validation.
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/linkform.md`.

use egui::Ui;
use pdfcer_core::annot_author::{BorderDash, Color};
use pdfcer_core::edit::{LinkBorder, LinkTarget};
use pdfcer_core::outline::DestView;

use crate::colourpick;
use crate::linkread::{Border, Goes, LinkReading};
use crate::text::linkedit as t;

/// The narrowest border offered, in points.
pub const MIN_WIDTH_PT: f64 = 0.25;
/// The widest.
pub const MAX_WIDTH_PT: f64 = 12.0;
/// The dash a dashed border gets: 3 on, 3 off, in points.
const DASH_PT: f64 = 3.0;

/// Which kind of destination the form is building.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Choice {
    /// A page of this document.
    Page,
    /// A web address.
    Web,
    /// A named destination.
    Named,
}

/// The form's state: every field the operator can type into, kept for all
/// three choices so switching between them loses nothing.
#[derive(Debug, Clone, PartialEq)]
pub struct LinkForm {
    /// Which destination is chosen.
    pub choice: Choice,
    /// 1-based page number.
    pub page_number: usize,
    /// The web address.
    pub uri: String,
    /// The destination name.
    pub name: String,
    /// Whether a visible rectangle is drawn.
    pub border_on: bool,
    /// Its width in points.
    pub width: f64,
    /// Its colour, in the space it was read or chosen in.
    pub colour: Color,
    /// Whether it is dashed.
    pub dashed: bool,
}

impl Default for LinkForm {
    /// A link to page 1 with no visible border — how most producers write
    /// one, and the choice that changes nothing on the printed sheet.
    fn default() -> Self {
        Self {
            choice: Choice::Page,
            page_number: 1,
            uri: String::new(),
            name: String::new(),
            border_on: false,
            width: 1.0,
            colour: Color::Rgb(0.0, 0.0, 0.0),
            dashed: false,
        }
    }
}

/// Region names a caller publishes for one drawing of the form.
#[derive(Debug, Clone, Copy)]
pub struct Regions {
    /// The page choice.
    pub page: &'static str,
    /// The web address choice.
    pub web: &'static str,
    /// The named destination choice.
    pub named: &'static str,
    /// The text field of the current choice.
    pub field: &'static str,
    /// The visible-border switch.
    pub border: &'static str,
    /// The colour swatch.
    pub colour: &'static str,
}

impl LinkForm {
    /// **The form seeded from a link already in the file.** A link running an
    /// action this shell does not author seeds as a page link, which applies
    /// only when the operator presses Apply.
    #[must_use]
    pub fn from_reading(reading: &LinkReading) -> Self {
        let mut form = Self::default();
        match &reading.goes {
            Goes::Page(index) => form.page_number = index + 1,
            Goes::Uri(uri) => {
                form.choice = Choice::Web;
                form.uri.clone_from(uri);
            }
            Goes::Named(key) => {
                form.choice = Choice::Named;
                form.name = String::from_utf8_lossy(key).into_owned();
            }
            Goes::Other(_) | Goes::Nowhere => {}
        }
        if let Some(Border {
            width,
            colour,
            dashed,
        }) = &reading.border
        {
            form.border_on = true;
            form.width = width.clamp(MIN_WIDTH_PT, MAX_WIDTH_PT);
            form.colour = colour.unwrap_or(Color::Rgb(0.0, 0.0, 0.0));
            form.dashed = *dashed;
        }
        form
    }

    /// **The target the form describes**, or the sentence saying what to
    /// fix. `page_count` bounds the page number.
    ///
    /// # Errors
    ///
    /// An empty or non-ASCII web address, or an empty destination name.
    pub fn target(&self, page_count: usize) -> Result<LinkTarget, &'static str> {
        match self.choice {
            Choice::Page => Ok(LinkTarget::Page {
                page_index: self.page_number.clamp(1, page_count.max(1)) - 1,
                view: DestView::Fit,
            }),
            Choice::Web => {
                let uri = self.uri.trim();
                if uri.is_empty() {
                    Err(t::needs_web())
                } else if !uri.is_ascii() || uri.bytes().any(|b| b.is_ascii_control()) {
                    Err(t::web_not_ascii())
                } else {
                    Ok(LinkTarget::Uri(uri.to_owned()))
                }
            }
            Choice::Named => {
                if self.name.is_empty() {
                    Err(t::needs_name())
                } else {
                    Ok(LinkTarget::Named(self.name.as_bytes().to_vec()))
                }
            }
        }
    }

    /// **The border the form describes**, or `None` for none.
    #[must_use]
    pub fn border(&self) -> Option<LinkBorder> {
        if !self.border_on {
            return None;
        }
        let border = LinkBorder::new(self.width.clamp(MIN_WIDTH_PT, MAX_WIDTH_PT), self.colour);
        Some(
            match self
                .dashed
                .then(|| BorderDash::new(vec![DASH_PT]))
                .flatten()
            {
                Some(dash) => border.with_dash(dash),
                None => border,
            },
        )
    }

    /// **Draw the "Goes to" controls.** Returns whether anything changed.
    pub fn goes_ui(&mut self, ui: &mut Ui, page_count: usize, regions: Regions) -> bool {
        let before = self.clone();
        ui.label(t::goes_to());
        for (choice, label, region) in [
            (Choice::Page, t::choice_page(), regions.page),
            (Choice::Web, t::choice_web(), regions.web),
            (Choice::Named, t::choice_named(), regions.named),
        ] {
            let r = ui.radio_value(&mut self.choice, choice, label);
            crate::diag::ui_rect(region, r.rect);
        }
        ui.add_space(4.0);
        match self.choice {
            Choice::Page => {
                ui.horizontal(|ui| {
                    ui.label(t::page_label());
                    let r = ui.add(
                        egui::DragValue::new(&mut self.page_number)
                            .range(1..=page_count.max(1))
                            .speed(0.1),
                    );
                    crate::diag::ui_rect(regions.field, r.rect);
                });
                ui.label(egui::RichText::new(t::page_note(page_count)).small().weak());
            }
            Choice::Web => {
                // escape-disposition: keeps-draft — the form lives in the
                // dialog's or the panel's state across frames.
                let r = ui.add(
                    egui::TextEdit::singleline(&mut self.uri)
                        .hint_text(t::web_hint())
                        .desired_width(f32::INFINITY),
                );
                crate::diag::ui_rect(regions.field, r.rect);
            }
            Choice::Named => {
                // escape-disposition: keeps-draft — as the address above.
                let r = ui.add(
                    egui::TextEdit::singleline(&mut self.name)
                        .hint_text(t::named_hint())
                        .desired_width(f32::INFINITY),
                );
                crate::diag::ui_rect(regions.field, r.rect);
                ui.label(egui::RichText::new(t::named_note()).small().weak());
            }
        }
        *self != before
    }

    /// **Draw the border controls.** Returns whether anything changed.
    pub fn border_ui(&mut self, ui: &mut Ui, salt: &str, regions: Regions) -> bool {
        let before = self.clone();
        ui.label(t::border_heading());
        let r = ui
            .checkbox(&mut self.border_on, t::border_visible())
            .on_hover_text(t::border_visible_tip());
        crate::diag::ui_rect(regions.border, r.rect);
        if self.border_on {
            ui.horizontal(|ui| {
                ui.label(t::border_width());
                ui.add(
                    egui::DragValue::new(&mut self.width)
                        .range(MIN_WIDTH_PT..=MAX_WIDTH_PT)
                        .speed(0.05)
                        .suffix(t::width_suffix()),
                );
                ui.label(t::border_colour());
                // No opacity: `/C` carries none.
                let shown = colourpick::Shown::Agreed(colourpick::Pick {
                    ink: colourpick::Ink::from(self.colour),
                    opacity: None,
                });
                if let Some(pick) =
                    colourpick::show(ui, salt, shown, colourpick::Spaces::ALL, regions.colour, "")
                {
                    self.colour = pick.ink.into();
                }
                ui.checkbox(&mut self.dashed, t::border_dashed());
            });
        }
        *self != before
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_page_number_becomes_a_fitted_zero_based_index() {
        let form = LinkForm {
            page_number: 3,
            ..LinkForm::default()
        };
        assert_eq!(
            form.target(5),
            Ok(LinkTarget::Page {
                page_index: 2,
                view: DestView::Fit
            })
        );
        assert_eq!(
            LinkForm {
                page_number: 9,
                ..LinkForm::default()
            }
            .target(5),
            Ok(LinkTarget::Page {
                page_index: 4,
                view: DestView::Fit
            }),
            "a page past the end is the last page"
        );
    }

    #[test]
    fn an_empty_or_non_ascii_address_is_refused_with_a_sentence() {
        let mut form = LinkForm {
            choice: Choice::Web,
            ..LinkForm::default()
        };
        assert_eq!(form.target(1), Err(t::needs_web()));
        form.uri = "https://exämple.com".to_owned();
        assert_eq!(form.target(1), Err(t::web_not_ascii()));
        form.uri = "  https://example.com ".to_owned();
        assert_eq!(
            form.target(1),
            Ok(LinkTarget::Uri("https://example.com".to_owned()))
        );
    }

    #[test]
    fn no_border_is_none_and_a_dashed_one_carries_its_dash() {
        assert_eq!(LinkForm::default().border(), None);
        let form = LinkForm {
            border_on: true,
            dashed: true,
            colour: Color::Rgb(1.0, 0.0, 0.0),
            width: 2.0,
            ..LinkForm::default()
        };
        let border = form.border().expect("a visible border");
        assert!((border.width - 2.0).abs() < 1e-9);
        assert_eq!(border.color, Color::Rgb(1.0, 0.0, 0.0));
        assert!(border.dash.is_some());
    }

    #[test]
    fn a_reading_seeds_the_choice_it_names() {
        let reading = LinkReading {
            goes: Goes::Uri("https://example.com".to_owned()),
            border: Some(Border {
                width: 2.0,
                colour: Some(Color::Cmyk(1.0, 0.5, 0.0, 0.1)),
                dashed: true,
            }),
        };
        let form = LinkForm::from_reading(&reading);
        assert_eq!(form.choice, Choice::Web);
        assert_eq!(form.uri, "https://example.com");
        assert!(form.border_on && form.dashed);
        assert_eq!(
            form.colour,
            Color::Cmyk(1.0, 0.5, 0.0, 0.1),
            "a CMYK border opens as CMYK"
        );
    }
}
