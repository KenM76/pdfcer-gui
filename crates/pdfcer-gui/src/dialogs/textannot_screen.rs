//! # `dialogs::textannot::screen` — the media-clip half of the note dialog
//!
//! The clip is picked before the window opens (in `BeginTextAnnot`'s apply
//! arm); this names it, offers its MIME type (suggested from the extension,
//! editable), what starts playback and the temporary-file permission, and
//! builds the action Add raises. The bytes are read at apply time.

use std::path::PathBuf;

use egui::Ui;
use pdfcer_core::annot_author::{MediaTempAccess, ScreenTrigger};
use pdfcer_core::page_tree::Rect;

use crate::app::actions::Action;
use crate::canvas::textannot::painted_text;
use crate::text::screenannot as t;
use pdfcer_gui_base::wordmarkup::{SCREEN_TRIGGERS, TEMP_ACCESS};

/// The MIME-type field's region.
pub const REGION_SCREEN_TYPE: &str = "text-annot.screen-type"; // ui-text-exempt: trace region name, never displayed
/// The trigger chooser's region; each trigger publishes `<this>.<token>`,
/// e.g. `text-annot.screen-trigger.PageOpen`.
pub const REGION_SCREEN_TRIGGER: &str = "text-annot.screen-trigger"; // ui-text-exempt: trace region name, never displayed
/// The temporary-file chooser's region; each permission publishes
/// `<this>.<its /TF string>`, e.g. `text-annot.screen-temp.TEMPALWAYS`.
pub const REGION_SCREEN_TEMP: &str = "text-annot.screen-temp"; // ui-text-exempt: trace region name, never displayed

/// A trigger's stable token, for region names and the trace.
pub(crate) const fn trigger_token(trigger: ScreenTrigger) -> &'static str {
    match trigger {
        ScreenTrigger::Click => "Click",
        ScreenTrigger::PageOpen => "PageOpen",
    }
}

/// The clip chosen for a media region, and how it will play.
pub(super) struct Chosen {
    /// The file, as the picker named it.
    file: PathBuf,
    /// Its base name, as shown and as embedded.
    name: String,
    /// Its size when the window opened, or `None` if it could not be read.
    bytes: Option<u64>,
    /// The MIME type, as the field holds it.
    content_type: String,
    /// Whether the extension suggested a type; decides the sentence under
    /// the field.
    suggested: bool,
    /// What starts playback.
    trigger: ScreenTrigger,
    /// The `/TF` permission.
    temp_access: MediaTempAccess,
}

impl Chosen {
    /// Name `file` for the dialog, with the extension's suggested type and
    /// the engine's defaults for the two playback choices.
    pub(super) fn new(file: PathBuf) -> Self {
        let name = file
            .file_name()
            .map_or_else(String::new, |n| n.to_string_lossy().into_owned());
        let bytes = std::fs::metadata(&file).ok().map(|m| m.len());
        let suggestion = t::media_type(&name);
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed
            let size = bytes.map_or_else(|| "unreadable".to_owned(), |b| b.to_string());
            format!(
                "screen-annot-open name={name:?} bytes={size} suggested={}",
                suggestion.unwrap_or("none")
            )
        });
        Self {
            file,
            name,
            bytes,
            content_type: suggestion.unwrap_or_default().to_owned(),
            suggested: suggestion.is_some(),
            trigger: ScreenTrigger::default(),
            temp_access: MediaTempAccess::default(),
        }
    }

    /// The file line, the type field and the two playback choices.
    pub(super) fn show(&mut self, ui: &mut Ui) {
        ui.add_space(8.0);
        ui.label(t::file_line(&self.name, self.bytes.unwrap_or(0)));
        ui.add_space(8.0);
        ui.label(t::type_label());
        let r = ui.text_edit_singleline(&mut self.content_type);
        crate::diag::ui_rect(REGION_SCREEN_TYPE, r.rect);
        ui.label(
            egui::RichText::new(if self.suggested {
                t::type_suggested()
            } else {
                t::type_unknown()
            })
            .small()
            .weak(),
        );
        ui.add_space(8.0);
        ui.label(t::trigger_heading());
        for trigger in SCREEN_TRIGGERS {
            let r = ui.radio_value(&mut self.trigger, *trigger, t::trigger_label(*trigger));
            let token = trigger_token(*trigger);
            crate::diag::ui_rect(&format!("{REGION_SCREEN_TRIGGER}.{token}"), r.rect);
        }
        ui.add_space(8.0);
        ui.label(t::temp_label()).on_hover_text(t::temp_hover());
        for access in TEMP_ACCESS {
            let r = ui
                .radio_value(
                    &mut self.temp_access,
                    *access,
                    t::temp_access_label(*access),
                )
                .on_hover_text(t::temp_hover());
            let token = String::from_utf8_lossy(access.as_bytes());
            crate::diag::ui_rect(&format!("{REGION_SCREEN_TEMP}.{token}"), r.rect);
        }
    }

    /// The action Add raises. An empty description is `None`; an empty type
    /// is the fallback, which the status note names.
    pub(super) fn action(&self, page: usize, rect: Rect, typed: &str) -> Action {
        let text = painted_text(typed);
        let content_type = match self.content_type.trim() {
            "" => t::FALLBACK_TYPE,
            typed => typed,
        };
        Action::NewComment(pdfcer_gui_base::newcomment::NewComment::Screen {
            page,
            rect,
            file: self.file.clone(),
            content_type: content_type.to_owned(),
            trigger: self.trigger,
            temp_access: self.temp_access,
            description: (!text.is_empty()).then(|| text.to_owned()),
        })
    }
}
