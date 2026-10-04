//! # `dialogs::textannot::attach` — the attach-file half of the note dialog
//!
//! The file is picked before the window opens (in `BeginTextAnnot`'s apply
//! arm, never inside a layout pass); this names it, offers the marker icon,
//! and builds the action Add raises.

use std::path::PathBuf;

use egui::Ui;
use pdfcer_core::annot_author::AttachmentIcon;
use pdfcer_core::page_tree::Rect;

use crate::app::actions::Action;
use crate::canvas::textannot::{ATTACHMENT_ICONS, DEFAULT_ATTACHMENT_ICON, painted_text};
use crate::text::attachannot as t;

/// The region the marker-icon chooser publishes; each icon publishes
/// `<this>.<PDF icon name>`, e.g. `text-annot.attach-icon.Paperclip`.
pub const REGION_ATTACH_ICON: &str = "text-annot.attach-icon"; // ui-text-exempt: trace region name, never displayed

/// The file chosen for a page attachment, and the marker it will carry.
pub(super) struct Chosen {
    /// The file, as the picker named it.
    file: PathBuf,
    /// Its base name, as shown and as embedded.
    name: String,
    /// Its size when the window opened, or `None` if it could not be read;
    /// the bytes themselves are read at apply time.
    bytes: Option<u64>,
    /// The marker's icon.
    icon: AttachmentIcon,
}

impl Chosen {
    /// Name `file` for the dialog.
    pub(super) fn new(file: PathBuf) -> Self {
        let name = file
            .file_name()
            .map_or_else(String::new, |n| n.to_string_lossy().into_owned());
        let bytes = std::fs::metadata(&file).ok().map(|m| m.len());
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed
            let size = bytes.map_or_else(|| "unreadable".to_owned(), |b| b.to_string());
            format!("attach-annot-open name={name:?} bytes={size}")
        });
        Self {
            file,
            name,
            bytes,
            icon: DEFAULT_ATTACHMENT_ICON,
        }
    }

    /// The file line and the icon chooser.
    pub(super) fn show(&mut self, ui: &mut Ui) {
        ui.add_space(8.0);
        ui.label(t::file_line(&self.name, self.bytes.unwrap_or(0)));
        ui.add_space(8.0);
        ui.label(t::icon_heading());
        let top = ui.cursor().min;
        for icon in ATTACHMENT_ICONS {
            let r = ui.radio_value(&mut self.icon, icon.clone(), t::icon_label(icon));
            crate::diag::ui_rect(
                &format!(
                    "{REGION_ATTACH_ICON}.{}",
                    String::from_utf8_lossy(icon.name())
                ),
                r.rect,
            );
        }
        crate::diag::ui_rect(
            REGION_ATTACH_ICON,
            egui::Rect::from_min_max(top, ui.cursor().min),
        );
    }

    /// The action Add raises. An empty description is `None`, so no empty
    /// `/Contents` or `/Desc` is written.
    pub(super) fn action(&self, page: usize, rect: Rect, typed: &str) -> Action {
        let text = painted_text(typed);
        Action::NewComment(pdfcer_gui_base::newcomment::NewComment::Attachment {
            page,
            rect,
            file: self.file.clone(),
            icon: self.icon.clone(),
            description: (!text.is_empty()).then(|| text.to_owned()),
        })
    }
}
