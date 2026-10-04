//! # `dialogs::split_pages` — Pages ▸ Split…
//!
//! The operator chooses where each new file starts — every N pages, after the
//! pages he names (the thumbnails' picks fill the box), or at each top-level
//! bookmark — plus the file-name pattern, the folder and whether page labels
//! are kept. There is no default rule: the window opens with none chosen unless
//! pages were picked, and Split stays greyed until the preview lists files.
//!
//! The preview is `app::actions::split::plan`, recomputed only when an input
//! or the document's edit epoch changes.

use std::path::PathBuf;

use egui::Ui;
use pdfcer_core::pageops::{ExtractedPageLabels, SplitCriterion};

use crate::app::actions::Action;
use crate::app::actions::pages::PageAction;
use crate::app::actions::split::{Preview, SplitRequest, plan, stem_of};
use crate::app::files::{self, Picked};
use crate::app::state::{OpenDoc, Status};
use crate::text::export_text as tt;
use crate::text::split_pages as t;

/// The region this dialog publishes for its body.
pub const REGION_BODY: &str = "dialog:split-pages"; // ui-text-exempt: trace region name, never displayed
/// The region the Split button publishes.
pub const REGION_SPLIT: &str = "split-pages.split"; // ui-text-exempt: trace region name, never displayed
/// The every-N radio and its number box.
pub const REGION_EVERY: &str = "split-pages.rule.every"; // ui-text-exempt: trace region name, never displayed
/// The every-N number box.
pub const REGION_EVERY_N: &str = "split-pages.rule.every.n"; // ui-text-exempt: trace region name, never displayed
/// The after-pages radio.
pub const REGION_AFTER: &str = "split-pages.rule.after"; // ui-text-exempt: trace region name, never displayed
/// The after-pages box.
pub const REGION_AFTER_PAGES: &str = "split-pages.rule.after.pages"; // ui-text-exempt: trace region name, never displayed
/// The top-level-bookmarks radio.
pub const REGION_BOOKMARKS: &str = "split-pages.rule.bookmarks"; // ui-text-exempt: trace region name, never displayed
/// The file-name pattern box.
pub const REGION_TEMPLATE: &str = "split-pages.template"; // ui-text-exempt: trace region name, never displayed
/// The folder box.
pub const REGION_FOLDER: &str = "split-pages.folder"; // ui-text-exempt: trace region name, never displayed
/// The Browse… button.
pub const REGION_BROWSE: &str = "split-pages.browse"; // ui-text-exempt: trace region name, never displayed
/// The keep-labels box.
pub const REGION_LABELS: &str = "split-pages.labels"; // ui-text-exempt: trace region name, never displayed

/// Where each new file starts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Rule {
    EveryN,
    AfterPages,
    Bookmarks,
}

/// The Split window's live state.
pub struct SplitPagesDialog {
    /// The document this window splits, by path; another tab on screen
    /// disables Split rather than splitting the wrong file.
    doc_path: PathBuf,
    page_count: usize,
    /// Whether the top-level bookmarks divide this document at all.
    bookmarks_divide: bool,
    rule: Option<Rule>,
    every_text: String,
    after_text: String,
    template: String,
    folder_text: String,
    /// `None` when the document has no page labels: the box is not drawn.
    keep_labels: Option<bool>,
    /// The inputs and edit epoch the cached preview was computed for.
    preview_key: Option<(SplitInputs, u64)>,
    preview: Preview,
    split_requested: bool,
    close_requested: bool,
}

/// What the preview depends on, besides the document.
type SplitInputs = Result<SplitRequest, String>;

impl SplitPagesDialog {
    /// Open for the document on screen. `picked` is the thumbnails'
    /// selection: picks fill the after-pages box and choose that rule.
    #[must_use]
    pub fn open(doc: &OpenDoc, picked: &[usize]) -> Self {
        let page_count = doc.pages.len();
        let breaks: Vec<usize> = picked
            .iter()
            .copied()
            .filter(|p| p + 1 < page_count)
            .collect();
        let bookmarks_divide = pdfcer_core::pageops::plan_split(
            &doc.session.view(),
            &SplitCriterion::TopLevelBookmarks,
            pdfcer_core::pageops::split::DEFAULT_NAME_TEMPLATE,
            &stem_of(doc),
        )
        .is_ok();
        let folder_text = doc
            .stored_under()
            .and_then(std::path::Path::parent)
            .map(|p| p.display().to_string())
            .unwrap_or_default();
        let dialog = Self {
            doc_path: doc.path.clone(),
            page_count,
            bookmarks_divide,
            rule: (!breaks.is_empty()).then_some(Rule::AfterPages),
            every_text: String::new(),
            after_text: pdfcer_gui_base::pageselection::format_page_range(&breaks),
            template: pdfcer_core::pageops::split::DEFAULT_NAME_TEMPLATE.to_owned(),
            folder_text,
            keep_labels: doc.page_labels().is_some().then_some(true),
            preview_key: None,
            preview: Preview::Refused(t::choose_rule().to_owned()),
            split_requested: false,
            close_requested: false,
        };
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed
            format!(
                "split-pages-open pages={page_count} picked={} bookmarks={} labelled={}",
                breaks.len(),
                u8::from(bookmarks_divide),
                u8::from(dialog.keep_labels.is_some()),
            )
        });
        dialog
    }

    /// Draw it. Returns `false` when it should close.
    pub fn show(
        &mut self,
        ctx: &egui::Context,
        actions: &mut Vec<Action>,
        status: &Status,
    ) -> bool {
        let doc = match status {
            Status::Open(doc) if doc.path == self.doc_path => Some(&**doc),
            _ => None,
        };
        self.refresh(doc);
        let (frame, ()) = crate::dialogs::host::Host::new(
            "split-pages", // ui-text-exempt: a viewport key, never displayed.
            t::window_title(),
            egui::vec2(480.0, 560.0),
            egui::vec2(380.0, 360.0),
        )
        .show(ctx, |ui| {
            crate::diag::ui_rect(REGION_BODY, ui.max_rect());
            self.body(ui);
        });
        if std::mem::take(&mut self.split_requested)
            && let (Some(_), Ok(request), Preview::Parts { .. }) =
                (doc, self.inputs(), &self.preview)
        {
            actions.push(Action::Page(PageAction::SplitDocument(request)));
            return false;
        }
        !frame.closed && !std::mem::take(&mut self.close_requested)
    }

    /// The request the inputs describe, or the sentence for why they describe
    /// none.
    fn inputs(&self) -> SplitInputs {
        let criterion = match self.rule {
            None => return Err(t::choose_rule().to_owned()),
            Some(Rule::EveryN) => match self.every_text.trim().parse::<usize>() {
                Ok(n) if n > 0 => SplitCriterion::EveryN(n),
                _ => return Err(t::every_invalid().to_owned()),
            },
            Some(Rule::AfterPages) => {
                let pages = pdfcer_gui_base::pageselection::parse_page_range(
                    &self.after_text,
                    self.page_count,
                )
                .ok_or_else(|| t::after_invalid(self.page_count))?;
                SplitCriterion::AfterPages(pages)
            }
            Some(Rule::Bookmarks) => SplitCriterion::TopLevelBookmarks,
        };
        let labels = if self.keep_labels == Some(false) {
            ExtractedPageLabels::Drop
        } else {
            ExtractedPageLabels::Keep
        };
        Ok(SplitRequest {
            criterion,
            template: self.template.trim().to_owned(),
            folder: PathBuf::from(self.folder_text.trim()),
            labels,
        })
    }

    /// Recompute the preview when an input or the document changed.
    fn refresh(&mut self, doc: Option<&OpenDoc>) {
        let Some(doc) = doc else {
            let name = self.doc_path.file_name().map_or_else(
                || self.doc_path.display().to_string(),
                |n| n.to_string_lossy().into_owned(),
            );
            self.preview = Preview::Refused(t::other_document(&name));
            self.preview_key = None;
            return;
        };
        let key = (self.inputs(), doc.edit_epoch);
        if self.preview_key.as_ref() == Some(&key) {
            return;
        }
        self.preview = match &key.0 {
            Err(why) => Preview::Refused(why.clone()),
            Ok(request) => plan(
                &doc.session.view(),
                request,
                &stem_of(doc),
                doc.stored_under(),
            ),
        };
        crate::diag::trace(|| match &self.preview {
            // ui-text-exempt: diagnostic trace, never displayed
            Preview::Parts { parts, existing } => format!(
                "split-pages-preview files={} existing={existing} names={:?} ranges={} folder={:?}",
                parts.len(),
                parts.iter().map(|p| p.name.as_str()).collect::<Vec<_>>(),
                parts
                    .iter()
                    .map(|p| format!("{}-{}", p.first_page + 1, p.last_page + 1))
                    .collect::<Vec<_>>()
                    .join(","),
                self.folder_text.trim(),
            ),
            Preview::Refused(why) => format!("split-pages-preview files=0 refused={why:?}"),
        });
        self.preview_key = Some(key);
    }

    fn body(&mut self, ui: &mut Ui) {
        ui.label(t::intro());
        ui.add_space(8.0);
        self.rule_section(ui);
        ui.add_space(8.0);
        self.output_section(ui);
        ui.add_space(8.0);
        ui.separator();
        self.preview_section(ui);
        ui.separator();
        let ready = matches!(self.preview, Preview::Parts { .. });
        ui.horizontal(|ui| {
            let response = ui.add_enabled(ready, egui::Button::new(t::split_button()));
            crate::diag::ui_rect(REGION_SPLIT, response.rect);
            if response.clicked() {
                self.split_requested = true;
            }
            if ui.button(tt::cancel_button()).clicked() {
                self.close_requested = true;
            }
        });
    }

    fn rule_section(&mut self, ui: &mut Ui) {
        ui.label(t::rule_heading());
        ui.horizontal(|ui| {
            let r = ui.radio(self.rule == Some(Rule::EveryN), t::rule_every());
            crate::diag::ui_rect(REGION_EVERY, r.rect);
            if r.clicked() {
                self.rule = Some(Rule::EveryN);
            }
            // escape-disposition: dialog-cancels
            let field =
                ui.add(egui::TextEdit::singleline(&mut self.every_text).desired_width(48.0));
            crate::diag::ui_rect(REGION_EVERY_N, field.rect);
            if field.changed() {
                self.rule = Some(Rule::EveryN);
            }
            ui.label(t::rule_every_suffix());
        });
        ui.horizontal(|ui| {
            let r = ui.radio(self.rule == Some(Rule::AfterPages), t::rule_after());
            crate::diag::ui_rect(REGION_AFTER, r.rect);
            if r.clicked() {
                self.rule = Some(Rule::AfterPages);
            }
            let field = ui.text_edit_singleline(&mut self.after_text);
            crate::diag::ui_rect(REGION_AFTER_PAGES, field.rect);
            if field.changed() {
                self.rule = Some(Rule::AfterPages);
            }
        });
        ui.weak(t::rule_after_hint());
        let r = ui
            .add_enabled(
                self.bookmarks_divide,
                egui::RadioButton::new(self.rule == Some(Rule::Bookmarks), t::rule_bookmarks()),
            )
            .on_disabled_hover_text(t::rule_bookmarks_none());
        crate::diag::ui_rect(REGION_BOOKMARKS, r.rect);
        if r.clicked() {
            self.rule = Some(Rule::Bookmarks);
        }
    }

    fn output_section(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label(t::template_label());
            let field = ui.text_edit_singleline(&mut self.template);
            crate::diag::ui_rect(REGION_TEMPLATE, field.rect);
        });
        ui.weak(t::template_hint());
        ui.horizontal(|ui| {
            ui.label(t::folder_label());
            let field = ui.text_edit_singleline(&mut self.folder_text);
            crate::diag::ui_rect(REGION_FOLDER, field.rect);
            let browse = ui.button(t::browse_button());
            crate::diag::ui_rect(REGION_BROWSE, browse.rect);
            if browse.clicked()
                && let Picked::Path(folder) = files::pick_split_folder()
            {
                self.folder_text = folder.display().to_string();
            }
        });
        if let Some(keep) = self.keep_labels.as_mut() {
            let response = ui
                .checkbox(keep, t::keep_labels_label())
                .on_hover_text(t::keep_labels_tooltip());
            crate::diag::ui_rect(REGION_LABELS, response.rect);
        }
    }

    fn preview_section(&self, ui: &mut Ui) {
        match &self.preview {
            Preview::Refused(why) => {
                ui.label(why);
            }
            Preview::Parts { parts, existing } => {
                ui.label(t::preview_heading(parts.len()));
                egui::ScrollArea::vertical()
                    .max_height(160.0)
                    .auto_shrink([false, true])
                    .show(ui, |ui| {
                        for part in parts {
                            ui.monospace(t::preview_row(
                                &part.name,
                                part.first_page + 1,
                                part.last_page + 1,
                            ));
                        }
                    });
                if *existing > 0 {
                    ui.label(t::replaces_existing(*existing));
                }
            }
        }
    }
}
