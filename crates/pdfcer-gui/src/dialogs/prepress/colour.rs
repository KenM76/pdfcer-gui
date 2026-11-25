//! # `dialogs::prepress::colour` — the destination and options of Convert
//! colours and Convert to PDF/X
//!
//! Destinations offered: the document's own output intent when it carries
//! one (Convert colours only — PDF/X embeds a profile's bytes, which the
//! intent's reader does not hand out), and every installed output-class
//! profile. A profile file is read when Apply is pressed; one that cannot be
//! read is named here and nothing is pushed.

use pdfcer_core::color::RenderingIntent;
use pdfcer_core::preflight::PdfxLevel;
use pdfcer_core::prepress::{
    ColourConversion, ColourDestination, IntentChoice, PdfxConversion, PdfxOutputIntent,
};
use pdfcer_print::icc_profiles::{InstalledProfile, installed_profiles};

use super::Kind;
use crate::app::state::OpenDoc;
use crate::text::prepress as t;

/// Each region is suffixed `.{kind}` as in the parent module; an entry's
/// suffix follows its kind's.
///
/// The destination combo box.
pub const REGION_DESTINATION: &str = "prepress.destination"; // ui-text-exempt: trace region name, never displayed
/// One destination entry, suffixed `intent` or the installed profile's index.
pub const REGION_DESTINATION_PREFIX: &str = "prepress.destination:"; // ui-text-exempt: trace region name, never displayed
/// The PDF/X level combo box.
pub const REGION_LEVEL: &str = "prepress.level"; // ui-text-exempt: trace region name, never displayed
/// One level entry, suffixed with the level's engine name.
pub const REGION_LEVEL_PREFIX: &str = "prepress.level:"; // ui-text-exempt: trace region name, never displayed

/// The rendering intents offered, `None` meaning each object's own.
const INTENTS: [Option<RenderingIntent>; 5] = [
    None,
    Some(RenderingIntent::Perceptual),
    Some(RenderingIntent::RelativeColorimetric),
    Some(RenderingIntent::Saturation),
    Some(RenderingIntent::AbsoluteColorimetric),
];

fn intent_label(intent: Option<RenderingIntent>) -> &'static str {
    match intent {
        None => t::intent_objects_own(),
        Some(RenderingIntent::Perceptual) => t::intent_perceptual(),
        Some(RenderingIntent::Saturation) => t::intent_saturation(),
        Some(RenderingIntent::AbsoluteColorimetric) => t::intent_absolute(),
        Some(_) => t::intent_relative(),
    }
}

/// Where colours go.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Dest {
    Intent,
    Installed(usize),
}

/// The colour half of a prepress window.
pub(super) struct ColourForm {
    listed: Vec<InstalledProfile>,
    has_intent: bool,
    dest: Option<Dest>,
    device_rgb: bool,
    icc_rgb: bool,
    lab: bool,
    cal_rgb: bool,
    grey: bool,
    keep_black: bool,
    bpc: bool,
    /// Index into [`INTENTS`].
    intent: usize,
    embed: bool,
    /// Index into [`PdfxLevel::ALL`].
    level: usize,
    condition: String,
    /// The last profile that could not be read, until the next Apply.
    refused: Option<String>,
}

impl ColourForm {
    /// The engine's defaults, with the document's output intent chosen when
    /// it has one.
    pub(super) fn new(doc: &OpenDoc) -> Self {
        let has_intent =
            pdfcer_render::SoftProof::from_output_intent(&doc.session.view()).is_some();
        let defaults = ColourConversion::new(ColourDestination::OutputIntent);
        Self {
            listed: installed_profiles()
                .into_iter()
                .filter(|p| &p.class == b"prtr")
                .collect(),
            has_intent,
            dest: has_intent.then_some(Dest::Intent),
            device_rgb: defaults.device_rgb,
            icc_rgb: defaults.icc_rgb,
            lab: defaults.lab,
            cal_rgb: defaults.cal_rgb,
            grey: defaults.grey,
            keep_black: defaults.keep_pure_black,
            bpc: defaults.black_point_compensation,
            intent: 2,
            embed: false,
            level: PdfxLevel::ALL.len() - 1,
            condition: String::new(),
            refused: None,
        }
    }

    /// Why Apply is greyed, or `None`.
    pub(super) fn blocked(&self, pdfx: bool) -> Option<&'static str> {
        match self.dest {
            None => Some(t::choose_profile()),
            Some(Dest::Intent) if pdfx => Some(t::choose_profile()),
            _ if !pdfx
                && !(self.device_rgb || self.icc_rgb || self.lab || self.cal_rgb || self.grey) =>
            {
                Some(t::names_no_colour())
            }
            _ => None,
        }
    }

    /// Name a profile that could not be read.
    pub(super) fn refuse(&mut self, why: String) {
        self.refused = Some(why);
    }

    /// Convert colours' options.
    pub(super) fn colour_options(&mut self, ui: &mut egui::Ui, kind: Kind) {
        self.destination(ui, kind, true);
        ui.add_space(6.0);
        ui.label(t::colours_sources());
        ui.horizontal_wrapped(|ui| {
            ui.checkbox(&mut self.device_rgb, t::source_device_rgb());
            ui.checkbox(&mut self.icc_rgb, t::source_icc_rgb());
            ui.checkbox(&mut self.cal_rgb, t::source_cal_rgb());
            ui.checkbox(&mut self.lab, t::source_lab());
            ui.checkbox(&mut self.grey, t::source_grey())
                .on_hover_text(t::source_grey_hover());
        });
        ui.checkbox(&mut self.keep_black, t::keep_black())
            .on_hover_text(t::keep_black_hover());
        ui.checkbox(&mut self.bpc, t::bpc());
        self.intent_combo(ui);
        if matches!(self.dest, Some(Dest::Installed(_))) {
            ui.checkbox(&mut self.embed, t::embed_intent());
        }
    }

    /// Convert to PDF/X's options.
    pub(super) fn pdfx_options(&mut self, ui: &mut egui::Ui, kind: Kind) {
        ui.horizontal(|ui| {
            ui.label(t::pdfx_level());
            let current = PdfxLevel::ALL[self.level].claim();
            let r = egui::ComboBox::from_id_salt("prepress-level")
                .selected_text(current)
                .show_ui(ui, |ui| {
                    for (i, level) in PdfxLevel::ALL.iter().enumerate() {
                        let e = ui.selectable_value(&mut self.level, i, level.claim());
                        let key = format!("{}{}", kind.region(REGION_LEVEL_PREFIX), level.as_str());
                        crate::diag::ui_rect(&key, e.rect);
                    }
                })
                .response;
            crate::diag::ui_rect(&kind.region(REGION_LEVEL), r.rect);
        });
        self.destination(ui, kind, false);
        ui.horizontal(|ui| {
            ui.label(t::pdfx_condition());
            ui.text_edit_singleline(&mut self.condition);
        });
        self.intent_combo(ui);
    }

    fn destination(&mut self, ui: &mut egui::Ui, kind: Kind, offer_intent: bool) {
        if self.listed.is_empty() && !(offer_intent && self.has_intent) {
            ui.label(t::no_profiles());
            return;
        }
        let label = match self.dest {
            None => t::choose_profile().to_owned(),
            Some(Dest::Intent) => t::colours_output_intent().to_owned(),
            Some(Dest::Installed(i)) => self.listed[i].name.clone(),
        };
        let caption = if offer_intent {
            t::colours_destination()
        } else {
            t::pdfx_profile()
        };
        ui.horizontal(|ui| {
            ui.label(caption);
            let r = egui::ComboBox::from_id_salt("prepress-destination")
                .selected_text(label)
                .width(300.0)
                .show_ui(ui, |ui| self.destinations(ui, kind, offer_intent))
                .response;
            crate::diag::ui_rect(&kind.region(REGION_DESTINATION), r.rect);
        });
        if let Some(why) = &self.refused {
            ui.label(egui::RichText::new(why.as_str()).small().weak());
        }
    }

    fn destinations(&mut self, ui: &mut egui::Ui, kind: Kind, offer_intent: bool) {
        let prefix = kind.region(REGION_DESTINATION_PREFIX);
        if offer_intent && self.has_intent {
            let e = ui.selectable_value(
                &mut self.dest,
                Some(Dest::Intent),
                t::colours_output_intent(),
            );
            crate::diag::ui_rect(&format!("{prefix}intent"), e.rect); // ui-text-exempt: region key
        }
        for (i, p) in self.listed.iter().enumerate() {
            let e = ui.selectable_value(&mut self.dest, Some(Dest::Installed(i)), &p.name);
            crate::diag::ui_rect(&format!("{prefix}{i}"), e.rect);
        }
    }

    fn intent_combo(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label(t::intent());
            egui::ComboBox::from_id_salt("prepress-intent")
                .selected_text(intent_label(INTENTS[self.intent]))
                .show_ui(ui, |ui| {
                    for (i, intent) in INTENTS.iter().enumerate() {
                        ui.selectable_value(&mut self.intent, i, intent_label(*intent));
                    }
                });
        });
    }

    fn intent_choice(&self) -> IntentChoice {
        INTENTS[self.intent].map_or(IntentChoice::ObjectsOwn, IntentChoice::Fixed)
    }

    /// The chosen installed profile's bytes and name.
    fn profile(&self) -> Result<(Vec<u8>, String), String> {
        let Some(Dest::Installed(i)) = self.dest else {
            return Err(t::choose_profile().to_owned());
        };
        let p = &self.listed[i];
        std::fs::read(&p.path)
            .map(|bytes| (bytes, p.name.clone()))
            .map_err(|e| t::profile_unreadable(&p.path.display().to_string(), &e.to_string()))
    }

    /// Convert colours' engine options.
    pub(super) fn conversion(&mut self) -> Result<ColourConversion, String> {
        self.refused = None;
        let destination = match self.dest {
            Some(Dest::Intent) => ColourDestination::OutputIntent,
            _ => ColourDestination::Profile(self.profile()?.0),
        };
        let embed = self.embed && matches!(self.dest, Some(Dest::Installed(_)));
        let mut c = ColourConversion::new(destination);
        c.intent = self.intent_choice();
        c.device_rgb = self.device_rgb;
        c.icc_rgb = self.icc_rgb;
        c.lab = self.lab;
        c.cal_rgb = self.cal_rgb;
        c.grey = self.grey;
        c.keep_pure_black = self.keep_black;
        c.black_point_compensation = self.bpc;
        c.embed_output_intent = embed.then(|| "GTS_PDFX".to_owned()); // ui-text-exempt: a PDF name
        Ok(c)
    }

    /// Convert to PDF/X's level and engine options.
    pub(super) fn pdfx(&mut self) -> Result<(PdfxLevel, PdfxConversion), String> {
        self.refused = None;
        let (bytes, name) = self.profile()?;
        let condition = if self.condition.trim().is_empty() {
            name
        } else {
            self.condition.trim().to_owned()
        };
        let intent = PdfxOutputIntent::new(bytes).with_condition(condition);
        let conversion = PdfxConversion::new(intent).with_rendering_intent(self.intent_choice());
        Ok((PdfxLevel::ALL[self.level], conversion))
    }
}
