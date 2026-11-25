//! # `dialogs::prepress` — File ▸ Print's Fix hairlines, Flatten
//! transparency, Convert colours and Ink Manager, and the Preflight panel's
//! Convert to PDF/X
//!
//! Contract: one window per [`Kind`], document-scoped. Each shows its engine
//! verb's options and, except for PDF/X and the Ink Manager (which are about
//! the whole file), the
//! same [`PageScope`] Recognise text uses. Apply pushes
//! [`Action::Prepress`] with a fresh run number and the window stays open;
//! when the handler (`app::actions::prepress`) has run the verb as one undo
//! step it parks a [`PrepressNote`] carrying that number on the document, and
//! this window shows it under its options: what changed, what could not, and
//! every disclosure. Nothing is drawn on the canvas (R8b).
//!
//! Traces `prepress-asked verb= run= pages=` on Apply and
//! `prepress-shown run= refused=` when the note arrives.

mod colour;
mod inks;

use pdfcer_core::transparency::FlattenOptions;
use pdfcer_gui_base::prepress::{PrepressNote, PrepressRun, PrepressVerb, next_run};

use crate::app::actions::Action;
use crate::app::state::{OpenDoc, Status};
use crate::text::prepress as t;

use super::page_scope::PageScope;

/// Every region below is published suffixed `.{kind}` ([`Kind::token`]),
/// because two windows of different kinds can be open at once.
///
/// The window body.
pub const REGION_BODY: &str = "prepress.body"; // ui-text-exempt: trace region name, never displayed
/// The page-scope group.
pub const REGION_SCOPE: &str = "prepress.scope"; // ui-text-exempt: trace region name, never displayed
/// Apply.
pub const REGION_APPLY: &str = "prepress.apply"; // ui-text-exempt: trace region name, never displayed
/// Close.
pub const REGION_CLOSE: &str = "prepress.close"; // ui-text-exempt: trace region name, never displayed
/// The hairline minimum.
pub const REGION_MIN: &str = "prepress.min-width"; // ui-text-exempt: trace region name, never displayed
/// The flatten resolution.
pub const REGION_PPI: &str = "prepress.ppi"; // ui-text-exempt: trace region name, never displayed

/// Which verb the window runs.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Hairlines,
    Flatten,
    Colours,
    Pdfx,
    Inks,
}

impl Kind {
    /// The kind a command id opens.
    pub fn of_command(id: &str) -> Option<Self> {
        match id {
            "file.fix_hairlines" => Some(Self::Hairlines),
            "file.flatten_transparency" => Some(Self::Flatten),
            "file.convert_colours" => Some(Self::Colours),
            "file.convert_to_pdfx" => Some(Self::Pdfx),
            "file.ink_manager" => Some(Self::Inks),
            _ => None,
        }
    }

    /// `hairlines`, `flatten`, `colours` or `pdfx`.
    pub const fn token(self) -> &'static str {
        match self {
            Self::Hairlines => "hairlines", // ui-text-exempt: a region suffix
            Self::Flatten => "flatten",     // ui-text-exempt: a region suffix
            Self::Colours => "colours",     // ui-text-exempt: a region suffix
            Self::Pdfx => "pdfx",           // ui-text-exempt: a region suffix
            Self::Inks => "inks",           // ui-text-exempt: a region suffix
        }
    }

    /// `base` suffixed with this kind.
    pub(super) fn region(self, base: &str) -> String {
        format!("{base}.{}", self.token())
    }

    fn title(self) -> &'static str {
        match self {
            Self::Hairlines => t::hairlines_title(),
            Self::Flatten => t::flatten_title(),
            Self::Colours => t::colours_title(),
            Self::Pdfx => t::pdfx_title(),
            Self::Inks => crate::text::inks::title(),
        }
    }

    fn intro(self) -> &'static str {
        match self {
            Self::Hairlines => t::hairlines_intro(),
            Self::Flatten => t::flatten_intro(),
            Self::Colours => t::colours_intro(),
            Self::Pdfx => t::pdfx_intro(),
            Self::Inks => crate::text::inks::intro(),
        }
    }

    /// A viewport key per kind, so two can be open at once.
    const fn key(self) -> &'static str {
        match self {
            Self::Hairlines => "prepress-hairlines", // ui-text-exempt: a viewport key
            Self::Flatten => "prepress-flatten",     // ui-text-exempt: a viewport key
            Self::Colours => "prepress-colours",     // ui-text-exempt: a viewport key
            Self::Pdfx => "prepress-pdfx",           // ui-text-exempt: a viewport key
            Self::Inks => "prepress-inks",           // ui-text-exempt: a viewport key
        }
    }

    /// Whether the window offers a page scope: not for whole-file verbs.
    const fn scoped(self) -> bool {
        !matches!(self, Self::Pdfx | Self::Inks)
    }
}

/// One prepress window.
pub struct PrepressDialog {
    kind: Kind,
    /// The document it was opened on.
    serial: u64,
    scope: PageScope,
    page_count: usize,
    min_width_pt: f64,
    flatten: FlattenOptions,
    colours: colour::ColourForm,
    inks: inks::InkForm,
    /// The run whose note this window waits for or shows.
    asked: Option<u64>,
    /// The note last traced as shown.
    shown: Option<u64>,
    apply_requested: bool,
    close_requested: bool,
}

/// The hairline minimum a window opens with, in points.
const DEFAULT_MIN_PT: f64 = 0.25;

impl PrepressDialog {
    /// A window of `kind` on `doc`, with the rail's `picked` pages captured.
    pub fn new(kind: Kind, doc: &OpenDoc, picked: Vec<usize>) -> Self {
        Self {
            kind,
            serial: doc.serial,
            scope: PageScope::new(doc.view.page_index, picked),
            page_count: doc.pages.len(),
            min_width_pt: DEFAULT_MIN_PT,
            flatten: FlattenOptions::new(),
            colours: colour::ColourForm::new(doc),
            inks: inks::InkForm::default(),
            asked: None,
            shown: None,
            apply_requested: false,
            close_requested: false,
        }
    }

    /// Which verb it runs.
    pub const fn kind(&self) -> Kind {
        self.kind
    }

    /// Draw it. Returns whether it stays open.
    pub fn show(
        &mut self,
        ctx: &egui::Context,
        status: &Status,
        actions: &mut Vec<Action>,
    ) -> bool {
        let Status::Open(doc) = status else {
            return false;
        };
        if doc.serial != self.serial {
            return false;
        }
        if self.kind == Kind::Inks {
            self.inks.refresh(doc);
        }
        let note = doc
            .prepress_note
            .as_ref()
            .filter(|n| Some(n.run) == self.asked);
        let (frame, ()) = crate::dialogs::host::Host::new(
            self.kind.key(),
            self.kind.title(),
            egui::vec2(520.0, 560.0),
            egui::vec2(380.0, 320.0),
        )
        .show(ctx, |ui| {
            crate::diag::ui_rect(&self.kind.region(REGION_BODY), ui.max_rect());
            egui::ScrollArea::vertical().show(ui, |ui| self.body(ui, note));
        });
        if std::mem::take(&mut self.apply_requested) {
            self.apply(actions);
        }
        !frame.closed && !std::mem::take(&mut self.close_requested)
    }

    fn body(&mut self, ui: &mut egui::Ui, note: Option<&PrepressNote>) {
        ui.label(self.kind.intro());
        ui.add_space(8.0);
        if self.kind.scoped() {
            let region = self.kind.region(REGION_SCOPE);
            self.scope
                .show(ui, self.page_count, &region, "prepress-scope");
            ui.add_space(8.0);
        }
        match self.kind {
            Kind::Hairlines => self.hairline_options(ui),
            Kind::Flatten => self.flatten_options(ui),
            Kind::Colours => self.colours.colour_options(ui, self.kind),
            Kind::Pdfx => self.colours.pdfx_options(ui, self.kind),
            Kind::Inks => self.inks.options(ui),
        }
        ui.add_space(10.0);
        ui.separator();
        self.buttons(ui);
        if let Some(note) = note {
            self.note(ui, note);
        }
    }

    fn hairline_options(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label(t::hairlines_min());
            let r = ui
                .add(
                    egui::DragValue::new(&mut self.min_width_pt)
                        .range(0.01..=10.0)
                        .speed(0.01)
                        .suffix(" pt"), // ui-text-exempt: a unit symbol
                )
                .on_hover_text(t::hairlines_min_hover());
            crate::diag::ui_rect(&self.kind.region(REGION_MIN), r.rect);
        });
    }

    fn flatten_options(&mut self, ui: &mut egui::Ui) {
        ui.checkbox(&mut self.flatten.keep_text, t::flatten_keep_text());
        ui.checkbox(&mut self.flatten.keep_strokes, t::flatten_keep_strokes())
            .on_hover_text(t::flatten_keep_strokes_hover());
        ui.horizontal(|ui| {
            ui.label(t::flatten_ppi());
            let (min, max) = (FlattenOptions::MIN_PPI, FlattenOptions::MAX_PPI);
            let r = ui
                .add(
                    egui::DragValue::new(&mut self.flatten.raster_ppi)
                        .range(min..=max)
                        .speed(5.0)
                        .suffix(" ppi"), // ui-text-exempt: a unit symbol
                )
                .on_hover_text(t::flatten_ppi_hover(min, max));
            crate::diag::ui_rect(&self.kind.region(REGION_PPI), r.rect);
        });
    }

    /// Why Apply is greyed, or `None` when it is not.
    fn blocked(&self) -> Option<String> {
        if self.kind.scoped() && self.scope.pages(self.page_count).is_none() {
            return Some(t::names_no_page().to_owned());
        }
        match self.kind {
            Kind::Colours | Kind::Pdfx => self
                .colours
                .blocked(self.kind == Kind::Pdfx)
                .map(str::to_owned),
            Kind::Inks => self.inks.blocked(),
            _ => None,
        }
    }

    fn buttons(&mut self, ui: &mut egui::Ui) {
        let blocked = self.blocked();
        let (apply_at, close_at) = (
            self.kind.region(REGION_APPLY),
            self.kind.region(REGION_CLOSE),
        );
        ui.horizontal(|ui| {
            let apply = ui
                .add_enabled(blocked.is_none(), egui::Button::new(t::apply()))
                .on_disabled_hover_text(blocked.as_deref().unwrap_or_default());
            crate::diag::ui_rect_visible(&apply_at, apply.rect, ui.clip_rect());
            if apply.clicked() {
                self.apply_requested = true;
            }
            let close = ui.button(t::close());
            crate::diag::ui_rect_visible(&close_at, close.rect, ui.clip_rect());
            if close.clicked() {
                self.close_requested = true;
            }
        });
    }

    fn note(&mut self, ui: &mut egui::Ui, note: &PrepressNote) {
        if self.shown != Some(note.run) {
            self.shown = Some(note.run);
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed
                format!(
                    "prepress-shown run={} refused={} lines={}",
                    note.run,
                    u8::from(note.refused),
                    note.text.lines().count()
                )
            });
        }
        ui.add_space(8.0);
        let heading = if note.refused {
            t::refused_heading()
        } else {
            t::report_heading()
        };
        ui.label(heading);
        ui.add(egui::Label::new(note.text.as_str()).wrap());
    }

    /// Build the verb from the options and push it.
    fn apply(&mut self, actions: &mut Vec<Action>) {
        let verb = match self.kind {
            Kind::Hairlines => PrepressVerb::Hairlines {
                min_width_pt: self.min_width_pt,
            },
            Kind::Flatten => PrepressVerb::Flatten(self.flatten.clone()),
            Kind::Colours => match self.colours.conversion() {
                Ok(c) => PrepressVerb::ConvertColours(c),
                Err(why) => return self.colours.refuse(why),
            },
            Kind::Pdfx => match self.colours.pdfx() {
                Ok((level, conversion)) => PrepressVerb::Pdfx {
                    level,
                    conversion: Box::new(conversion),
                },
                Err(why) => return self.colours.refuse(why),
            },
            Kind::Inks => self.inks.take_verb(),
        };
        let pages = self.scope.pages(self.page_count).unwrap_or_default();
        let run = next_run();
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed
            format!(
                "prepress-asked verb={} run={run} pages={}",
                verb.token(),
                pages.len()
            )
        });
        self.asked = Some(run);
        actions.push(Action::Prepress(Box::new(PrepressRun { run, pages, verb })));
    }
}
