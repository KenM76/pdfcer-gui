//! # `panels::output_preview::proof` — Output Preview's simulation profile
//!
//! Soft proofing: the separation preview's composited ink shown as a chosen
//! CMYK output profile prints it ([`SoftProof`]). Offered: the document's own
//! output intent when it carries one, and every installed CMYK output
//! profile. *Simulate paper colour* implies *Simulate black ink*, as in
//! Acrobat. Screen only; spot inks are not proofed.
//!
//! Disclosure: a refused profile is named with the engine's sentence, a
//! profile with no black point compensation says that both switches off looks
//! like black ink, and spot inks on the page are named as unproofed.
//!
//! Traces `output-preview-proof source=<none|intent|installed|other>
//! paper=<0|1> black=<0|1> bpc=<0|1>` on change, and the regions below.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use pdfcer_print::icc_output::ProfileSpace;
use pdfcer_print::icc_profiles::{InstalledProfile, installed_profiles};
use pdfcer_render::{SeparationPreview, SoftProof};

use crate::app::actions::Action;
use crate::app::state::OpenDoc;
use crate::text::panels::output_preview as t;

/// The simulation profile combo box.
pub const REGION_PROFILE: &str = "output-preview.proof"; // ui-text-exempt: trace region name, never displayed
/// The *Simulate paper colour* switch.
pub const REGION_PAPER: &str = "output-preview.paper"; // ui-text-exempt: trace region name, never displayed
/// The *Simulate black ink* switch.
pub const REGION_BLACK: &str = "output-preview.black-ink"; // ui-text-exempt: trace region name, never displayed
/// One entry in the open combo box, suffixed `none`, `intent` or the
/// installed profile's index.
pub const REGION_CHOICE_PREFIX: &str = "output-preview.proof:"; // ui-text-exempt: trace region name, never displayed

/// A proof profile's origin; the cache key.
#[derive(Clone, PartialEq, Eq, Hash)]
enum Source {
    /// The output intent of the document with this serial.
    Intent(u64),
    Installed(PathBuf),
}

/// What the combo box was asked for this frame.
enum Choice {
    Off,
    Intent,
    Installed(usize),
}

/// The operator's proofing state that outlives one document: the installed
/// list and every profile already read, each read once.
#[derive(Default)]
pub struct ProofUi {
    listed: Option<Vec<InstalledProfile>>,
    loaded: HashMap<Source, Result<SoftProof, String>>,
    /// Whether each document (by serial) has an output intent to offer.
    intents: HashMap<u64, bool>,
    /// The engine's sentence for the last profile it refused, until the next
    /// choice.
    refused: Option<String>,
}

impl ProofUi {
    fn listed(&mut self) -> &[InstalledProfile] {
        self.listed.get_or_insert_with(|| {
            installed_profiles()
                .into_iter()
                .filter(|p| p.color_space == ProfileSpace::Cmyk && &p.class == b"prtr")
                .collect()
        })
    }

    /// Whether `doc` carries an output intent, reading (and caching) it once.
    fn has_intent(&mut self, doc: &OpenDoc) -> bool {
        if let Some(has) = self.intents.get(&doc.serial) {
            return *has;
        }
        let read = SoftProof::from_output_intent(&doc.session.view());
        let has = read.is_some();
        if let Some(read) = read {
            self.loaded
                .insert(Source::Intent(doc.serial), read.map_err(|e| e.to_string()));
        }
        self.intents.insert(doc.serial, has);
        has
    }

    /// Which cached source `proof` was read from.
    fn source_of(&self, proof: &SoftProof) -> Option<&Source> {
        self.loaded.iter().find_map(|(source, read)| {
            read.as_ref()
                .ok()
                .filter(|p| Arc::ptr_eq(p.profile(), proof.profile()))
                .map(|_| source)
        })
    }

    fn load(&mut self, source: Source) -> Result<SoftProof, String> {
        self.loaded
            .entry(source.clone())
            .or_insert_with(|| match &source {
                Source::Installed(path) => std::fs::read(path)
                    .map_err(|e| t::proof_unreadable(&e.to_string()))
                    .and_then(|bytes| SoftProof::from_bytes(bytes).map_err(|e| e.to_string())),
                Source::Intent(_) => Err(t::proof_no_intent().to_owned()),
            })
            .clone()
    }
}

/// The simulation profile section, under the ink list.
pub(super) fn section(
    ui: &mut egui::Ui,
    doc: &OpenDoc,
    preview: &SeparationPreview,
    spots: bool,
    state: &mut ProofUi,
    actions: &mut Vec<Action>,
) {
    ui.separator();
    let has_intent = state.has_intent(doc);
    let current = preview.soft_proof.as_ref();
    let source = current.and_then(|p| state.source_of(p)).cloned();
    let label = current_label(state, current, source.as_ref());
    let listed = state.listed().to_vec();
    let mut chosen = None;
    let combo = ui
        .horizontal(|ui| {
            ui.label(t::proof_label());
            egui::ComboBox::from_id_salt("output-preview-proof")
                .selected_text(label)
                .show_ui(ui, |ui| {
                    let menu = Menu {
                        serial: doc.serial,
                        off: current.is_none(),
                        has_intent,
                        listed: &listed,
                        source: source.as_ref(),
                    };
                    menu.show(ui, &mut chosen);
                })
                .response
        })
        .inner
        .on_hover_text(t::proof_hover());
    crate::diag::ui_rect(REGION_PROFILE, combo.rect);
    if let Some(choice) = chosen {
        apply(choice, doc.serial, &listed, preview, state, actions);
    }
    if let Some(why) = &state.refused {
        ui.label(egui::RichText::new(t::proof_refused(why)).small().weak());
    }
    let Some(proof) = current else {
        trace_proof("source=none"); // ui-text-exempt: diag trace value
        return;
    };
    switches(ui, preview, proof, actions);
    trace_proof(&format!(
        "source={} paper={} black={} bpc={}", // ui-text-exempt: diag trace value
        match source {
            Some(Source::Intent(_)) => "intent",
            Some(Source::Installed(_)) => "installed",
            None => "other",
        },
        u8::from(proof.simulates_paper_colour()),
        u8::from(proof.simulates_black_ink()),
        u8::from(proof.compensates_black())
    ));
    if !proof.compensates_black() {
        ui.label(egui::RichText::new(t::proof_no_bpc()).small().weak());
    }
    if spots {
        ui.label(egui::RichText::new(t::proof_spots()).small().weak());
    }
}

/// The closed combo box's text.
fn current_label(state: &ProofUi, current: Option<&SoftProof>, source: Option<&Source>) -> String {
    match (current, source) {
        (None, _) => t::proof_none().to_owned(),
        (Some(_), Some(Source::Intent(_))) => t::proof_intent().to_owned(),
        (Some(_), Some(Source::Installed(path))) => state
            .listed
            .as_ref()
            .and_then(|l| l.iter().find(|p| &p.path == path))
            .map_or_else(|| path.display().to_string(), |p| p.name.clone()),
        (Some(_), None) => t::proof_other().to_owned(),
    }
}

/// The open combo box's entries.
struct Menu<'a> {
    serial: u64,
    off: bool,
    has_intent: bool,
    listed: &'a [InstalledProfile],
    source: Option<&'a Source>,
}

impl Menu<'_> {
    fn show(&self, ui: &mut egui::Ui, chosen: &mut Option<Choice>) {
        let mut entry = |ui: &mut egui::Ui, key: &str, on: bool, text: &str, choice: Choice| {
            let r = ui.selectable_label(on, text);
            crate::diag::ui_rect(&format!("{REGION_CHOICE_PREFIX}{key}"), r.rect);
            if r.clicked() {
                *chosen = Some(choice);
            }
        };
        entry(ui, "none", self.off, t::proof_none(), Choice::Off); // ui-text-exempt: region key, never displayed
        if self.has_intent {
            let on = self.source == Some(&Source::Intent(self.serial));
            entry(ui, "intent", on, t::proof_intent(), Choice::Intent); // ui-text-exempt: region key, never displayed
        }
        for (i, p) in self.listed.iter().enumerate() {
            let on = matches!(self.source, Some(Source::Installed(path)) if path == &p.path);
            entry(ui, &i.to_string(), on, &p.name, Choice::Installed(i));
        }
    }
}

/// Attach the chosen profile, keeping the current switches.
fn apply(
    choice: Choice,
    serial: u64,
    listed: &[InstalledProfile],
    preview: &SeparationPreview,
    state: &mut ProofUi,
    actions: &mut Vec<Action>,
) {
    let source = match choice {
        Choice::Off => {
            state.refused = None;
            actions.push(Action::SetSeparations(Some(
                preview.clone().with_soft_proof(None),
            )));
            return;
        }
        Choice::Intent => Source::Intent(serial),
        Choice::Installed(i) => match listed.get(i) {
            Some(p) => Source::Installed(p.path.clone()),
            None => return,
        },
    };
    let (paper, black) = preview.soft_proof.as_ref().map_or((false, false), |p| {
        (p.simulates_paper_colour(), p.simulates_black_ink())
    });
    match state.load(source) {
        Ok(proof) => {
            state.refused = None;
            let proof = proof.with_paper_colour(paper).with_black_ink(black);
            actions.push(Action::SetSeparations(Some(
                preview.clone().with_soft_proof(Some(proof)),
            )));
        }
        Err(why) => state.refused = Some(why),
    }
}

/// *Simulate paper colour* and *Simulate black ink*; paper implies black ink.
fn switches(
    ui: &mut egui::Ui,
    preview: &SeparationPreview,
    proof: &SoftProof,
    actions: &mut Vec<Action>,
) {
    let mut paper = proof.simulates_paper_colour();
    let r = ui
        .checkbox(&mut paper, t::proof_paper())
        .on_hover_text(t::proof_paper_hover());
    crate::diag::ui_rect(REGION_PAPER, r.rect);
    let mut black = proof.simulates_black_ink();
    let b = ui
        .add_enabled(
            !proof.simulates_paper_colour(),
            egui::Checkbox::new(&mut black, t::proof_black()),
        )
        .on_hover_text(t::proof_black_hover())
        .on_disabled_hover_text(t::proof_black_implied());
    crate::diag::ui_rect(REGION_BLACK, b.rect);
    if r.changed() || b.changed() {
        let next = proof.clone().with_paper_colour(paper).with_black_ink(black);
        actions.push(Action::SetSeparations(Some(
            preview.clone().with_soft_proof(Some(next)),
        )));
    }
}

/// The section's state line, on change only.
fn trace_proof(state: &str) {
    crate::diag::trace_on_change("output-preview-proof", || state.to_owned());
}
