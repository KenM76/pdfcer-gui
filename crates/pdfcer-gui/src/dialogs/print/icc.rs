//! # `dialogs::print::icc` — print into the printer's ICC profile, or proof a press
//!
//! Every sheet is converted after rendering, before Colour / Grey / Black only,
//! by `pdfcer_print::icc_output::PrinterColour` (printer profile alone) or
//! `pdfcer_print::icc_proof::ProofColour` (a simulated press, landing in the
//! printer profile or in sRGB). The document is never touched, and a converted
//! sheet always goes as an image: the vector route hands the driver unmanaged
//! colour.
//!
//! The preview is a soft proof: with a press simulated it shows the press leg
//! landing in sRGB; a printer profile alone is not applied to it, because that
//! conversion produces the printer's device values, which a screen shows wrongly.
//!
//! Both transforms are compiled when a choice changes and reused for every
//! sheet and preview until the next change. A profile that cannot be used
//! refuses the job, with the engine's reason, rather than printing unmanaged.
//!
//! The choices are remembered in `PrintColourPrefs`.

use std::path::Path;

use pdfcer_gui_base::prefs::print_colour::{INTENTS, PrintColourPrefs, intent_key};
use pdfcer_print::icc_output::{PrinterColour, PrinterProfile, PrinterTransform};
use pdfcer_print::icc_profiles::{InstalledProfile, installed_profiles};
use pdfcer_print::icc_proof::{ProofColour, ProofProfile};

use crate::text::print::icc as t;

/// Region of the printer profile list; its entries add `.item.<i>`.
pub(super) const REGION_PRINTER: &str = "print.icc.printer"; // ui-text-exempt: region name
/// Region of the simulated press list; its entries add `.item.<i>`.
pub(super) const REGION_PROOF: &str = "print.icc.proof"; // ui-text-exempt: region name
/// Region of the intent list; its entries add `.item.<i>`.
const REGION_INTENT: &str = "print.icc.intent"; // ui-text-exempt: region name
const REGION_BPC: &str = "print.icc.bpc"; // ui-text-exempt: region name
const REGION_PAPER: &str = "print.icc.paper"; // ui-text-exempt: region name

/// The dialog's colour-management state.
#[derive(Default)]
pub(super) struct Icc {
    /// What is chosen, as remembered.
    pub(super) prefs: PrintColourPrefs,
    /// The installed profiles, read on the first draw of the controls.
    listed: Option<Vec<InstalledProfile>>,
    /// The sheets' transform for `prefs`, or why it cannot be built.
    sheet: Compiled,
    /// The preview's transform for `prefs`.
    preview: Compiled,
}

/// A transform built for the current choices.
#[derive(Default)]
enum Compiled {
    /// Nothing chosen, so nothing to convert.
    #[default]
    None,
    /// Ready, and whether black point compensation had to be dropped.
    Ready(PrinterTransform),
    /// The engine refused a profile; its reason.
    Refused(String),
}

impl Compiled {
    fn of(built: Option<Result<PrinterTransform, String>>) -> Self {
        match built {
            None => Self::None,
            Some(Ok(transform)) => Self::Ready(transform),
            Some(Err(why)) => Self::Refused(why),
        }
    }
}

/// A profile file, read for its role.
fn read_printer(path: &str) -> Result<PrinterProfile, String> {
    PrinterProfile::read(Path::new(path)).map_err(|e| e.to_string())
}

fn read_proof(path: &str) -> Result<ProofProfile, String> {
    ProofProfile::read(Path::new(path)).map_err(|e| e.to_string())
}

impl Icc {
    /// The state for remembered `prefs`, with its transforms built.
    pub(super) fn new(prefs: PrintColourPrefs) -> Self {
        let mut icc = Self {
            prefs,
            ..Self::default()
        };
        icc.rebuild();
        icc
    }

    /// Whether sheets are converted, so they must go as images.
    pub(super) fn managed(&self) -> bool {
        self.prefs.managed()
    }

    /// Why the job cannot print, when a chosen profile was refused.
    pub(super) fn refusal(&self) -> Option<String> {
        match &self.sheet {
            Compiled::Refused(why) => Some(t::refused(why)),
            _ => None,
        }
    }

    /// Build both transforms for the current choices.
    fn rebuild(&mut self) {
        let p = &self.prefs;
        let sheet = match (&p.proof_profile, &p.printer_profile) {
            (None, None) => None,
            (None, Some(printer)) => Some(read_printer(printer).and_then(|profile| {
                PrinterColour {
                    profile,
                    intent: p.intent,
                    black_point_compensation: p.black_point_compensation,
                }
                .compile()
                .map_err(|e| e.to_string())
            })),
            (Some(proof), printer) => Some(self.proof_transform(proof, printer.as_deref())),
        };
        let preview = p
            .proof_profile
            .as_deref()
            .map(|proof| self.proof_transform(proof, None));
        self.sheet = Compiled::of(sheet);
        self.preview = Compiled::of(preview);
        crate::diag::trace(|| {
            format!(
                // ui-text-exempt: diagnostic trace, never displayed in the UI
                "print-icc-chosen printer={} proof={} intent={} bpc={} paper={} sheet={} \
                 preview={}",
                p.printer_profile.as_deref().unwrap_or("none"),
                p.proof_profile.as_deref().unwrap_or("none"),
                intent_key(p.intent),
                u8::from(p.black_point_compensation),
                u8::from(p.simulate_paper),
                state_word(&self.sheet),
                state_word(&self.preview),
            )
        });
    }

    /// A proof of `proof` landing in `printer`, or in sRGB for `None`.
    fn proof_transform(
        &self,
        proof: &str,
        printer: Option<&str>,
    ) -> Result<PrinterTransform, String> {
        let p = &self.prefs;
        let printer = printer.map(read_printer).transpose()?;
        ProofColour {
            proof: read_proof(proof)?,
            intent: p.intent,
            black_point_compensation: p.black_point_compensation,
            simulate_paper: p.simulate_paper,
            printer,
        }
        .compile()
        .map_err(|e| e.to_string())
    }

    /// Convert one sheet raster (premultiplied RGBA8).
    pub(super) fn sheet(&self, rgba: &mut [u8]) {
        apply(&self.sheet, rgba, "sheet"); // ui-text-exempt: trace token
    }

    /// Convert one preview raster (premultiplied RGBA8).
    pub(super) fn preview(&self, rgba: &mut [u8]) {
        apply(&self.preview, rgba, "preview"); // ui-text-exempt: trace token
    }
}

fn state_word(compiled: &Compiled) -> &'static str {
    match compiled {
        Compiled::None => "none",          // ui-text-exempt: trace token
        Compiled::Ready(_) => "ready",     // ui-text-exempt: trace token
        Compiled::Refused(_) => "refused", // ui-text-exempt: trace token
    }
}

fn apply(compiled: &Compiled, rgba: &mut [u8], what: &str) {
    let Compiled::Ready(transform) = compiled else {
        return;
    };
    let changed = transform.apply(rgba);
    crate::diag::trace(|| {
        format!(
            // ui-text-exempt: diagnostic trace, never displayed in the UI
            "print-icc of={what} changed={changed} pixels={}",
            rgba.len() / 4
        )
    });
}

/// Which list a combo box fills.
#[derive(Clone, Copy)]
enum Role {
    Printer,
    Proof,
}

impl Role {
    fn region(self) -> &'static str {
        match self {
            Self::Printer => REGION_PRINTER,
            Self::Proof => REGION_PROOF,
        }
    }

    fn offers(self, profile: &InstalledProfile) -> bool {
        match self {
            Self::Printer => profile.is_print_destination(),
            Self::Proof => profile.is_proof_source(),
        }
    }
}

/// The colour-management controls. Returns whether a choice changed, so the
/// caller drops its preview rasters.
pub(super) fn controls(ui: &mut egui::Ui, icc: &mut Icc) -> bool {
    let listed = icc.listed.get_or_insert_with(|| {
        let list = installed_profiles();
        trace_listed(&list);
        list
    });
    let before = icc.prefs.clone();
    ui.label(t::heading());
    egui::Grid::new("print-icc-grid")
        .num_columns(2)
        .show(ui, |ui| {
            ui.label(t::printer_label())
                .on_hover_text(t::printer_hover());
            profile_combo(ui, Role::Printer, listed, &mut icc.prefs.printer_profile);
            ui.end_row();
            ui.label(t::proof_label()).on_hover_text(t::proof_hover());
            profile_combo(ui, Role::Proof, listed, &mut icc.prefs.proof_profile);
            ui.end_row();
            if icc.prefs.managed() {
                ui.label(t::intent_label());
                intent_combo(ui, &mut icc.prefs);
                ui.end_row();
            }
        });
    if icc.prefs.managed() {
        switches(ui, &mut icc.prefs);
    }
    let changed = icc.prefs != before;
    if changed {
        icc.rebuild();
    }
    notes(ui, icc);
    changed
}

/// Black point compensation, and paper simulation when a press is simulated.
fn switches(ui: &mut egui::Ui, prefs: &mut PrintColourPrefs) {
    let bpc = ui
        .checkbox(&mut prefs.black_point_compensation, t::bpc())
        .on_hover_text(t::bpc_hover());
    crate::diag::ui_rect_visible(REGION_BPC, bpc.rect, ui.clip_rect());
    if prefs.proof_profile.is_some() {
        let paper = ui
            .checkbox(&mut prefs.simulate_paper, t::paper())
            .on_hover_text(t::paper_hover());
        crate::diag::ui_rect_visible(REGION_PAPER, paper.rect, ui.clip_rect());
    }
}

/// The small lines under the controls: a refusal, dropped BPC, or what the
/// preview does not show.
fn notes(ui: &mut egui::Ui, icc: &Icc) {
    let muted = egui_shell::theme::Theme::of(ui.ctx()).palette.text_muted;
    let mut said = |text: String| {
        ui.label(egui::RichText::new(text).small().color(muted));
    };
    match &icc.sheet {
        Compiled::Refused(why) => said(t::refused(why)),
        Compiled::Ready(transform) if transform.black_point_compensation_refused() => {
            said(t::bpc_refused().to_owned());
        }
        _ => {}
    }
    if icc.prefs.printer_profile.is_some() && icc.prefs.proof_profile.is_none() {
        said(t::preview_unconverted().to_owned());
    }
}

/// The label a list shows for `chosen`: an installed profile's name, or the
/// file name of any other.
fn shown(listed: &[InstalledProfile], chosen: Option<&String>, none: &str) -> String {
    let Some(path) = chosen else {
        return none.to_owned();
    };
    listed
        .iter()
        .find(|p| p.path.as_os_str() == path.as_str())
        .map(|p| p.name.clone())
        .or_else(|| {
            Path::new(path)
                .file_name()
                .map(|f| f.to_string_lossy().into_owned())
        })
        .unwrap_or_else(|| path.clone())
}

/// One profile list: none, every installed profile the role accepts, then
/// any file.
fn profile_combo(
    ui: &mut egui::Ui,
    role: Role,
    listed: &[InstalledProfile],
    chosen: &mut Option<String>,
) {
    let none = match role {
        Role::Printer => t::printer_none(),
        Role::Proof => t::proof_none(),
    };
    let region = role.region();
    let combo = egui::ComboBox::from_id_salt(region)
        .width(260.0)
        .selected_text(shown(listed, chosen.as_ref(), none))
        .show_ui(ui, |ui| {
            let item = ui.selectable_label(chosen.is_none(), none);
            crate::diag::ui_rect_visible(&format!("{region}.item.0"), item.rect, ui.clip_rect());
            if item.clicked() {
                *chosen = None;
            }
            for (i, profile) in listed.iter().filter(|p| role.offers(p)).enumerate() {
                let path = profile.path.to_string_lossy().into_owned();
                let item = ui.selectable_label(chosen.as_ref() == Some(&path), &profile.name);
                crate::diag::ui_rect_visible(
                    &format!("{region}.item.{}", i + 1),
                    item.rect,
                    ui.clip_rect(),
                );
                if item.clicked() {
                    *chosen = Some(path);
                }
            }
            if ui.selectable_label(false, t::other_file()).clicked()
                && let Some(file) = crate::filedialog::FileDialog::new()
                    .add_filter(t::file_filter(), &["icc", "icm"])
                    .pick_file()
            {
                *chosen = Some(file.to_string_lossy().into_owned());
            }
        });
    crate::diag::ui_rect_visible(region, combo.response.rect, ui.clip_rect());
}

/// The rendering intent list.
fn intent_combo(ui: &mut egui::Ui, prefs: &mut PrintColourPrefs) {
    let names = t::intent_names();
    let at = INTENTS
        .iter()
        .position(|(i, _)| *i == prefs.intent)
        .unwrap_or(0);
    let combo = egui::ComboBox::from_id_salt(REGION_INTENT)
        .width(260.0)
        .selected_text(names[at])
        .show_ui(ui, |ui| {
            for (i, ((intent, _), name)) in INTENTS.iter().zip(names).enumerate() {
                let item = ui.selectable_label(prefs.intent == *intent, name);
                crate::diag::ui_rect_visible(
                    &format!("{REGION_INTENT}.item.{i}"),
                    item.rect,
                    ui.clip_rect(),
                );
                if item.clicked() {
                    prefs.intent = *intent;
                }
            }
        });
    crate::diag::ui_rect_visible(REGION_INTENT, combo.response.rect, ui.clip_rect());
}

/// One line per offered entry of each list, numbered as its region is, after
/// one line counting both lists, so an empty machine is told from undrawn
/// controls.
fn trace_listed(list: &[InstalledProfile]) {
    let count = |role: Role| list.iter().filter(|p| role.offers(p)).count();
    crate::diag::trace(|| {
        format!(
            // ui-text-exempt: diagnostic trace, never displayed in the UI
            "print-icc-lists printers={} proofs={}",
            count(Role::Printer),
            count(Role::Proof)
        )
    });
    for role in [Role::Printer, Role::Proof] {
        for (i, profile) in list.iter().filter(|p| role.offers(p)).enumerate() {
            crate::diag::trace(|| {
                format!(
                    // ui-text-exempt: diagnostic trace, never displayed in the UI
                    "print-icc-listed list={} item={} space={} name=\"{}\"",
                    role.region(),
                    i + 1,
                    space_word(profile),
                    profile.name.replace('"', "'")
                )
            });
        }
    }
}

fn space_word(profile: &InstalledProfile) -> &'static str {
    use pdfcer_print::icc_output::ProfileSpace;
    match profile.color_space {
        ProfileSpace::Gray => "grey", // ui-text-exempt: trace token
        ProfileSpace::Rgb => "rgb",   // ui-text-exempt: trace token
        ProfileSpace::Cmyk => "cmyk", // ui-text-exempt: trace token
        _ => "other",                 // ui-text-exempt: trace token
    }
}
