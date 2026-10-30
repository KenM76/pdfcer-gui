//! A header or footer on the printed copy only.
//!
//! The engine's `EditSession::stamp_running_text` writes the line, as
//! `pdfcer print --header/--footer` does: never onto the open document, but
//! onto a copy reopened from its current bytes (unsaved edits included). The
//! preview and the printer both render from that copy, so the preview shows
//! what prints.
//!
//! The copy is rebuilt only when the applied lines or the document change. A
//! text box applies its line when it loses focus, not per keystroke: a
//! rebuild serialises and reparses the whole document.

use std::hash::{Hash, Hasher};
use std::sync::Arc;

use egui::Ui;
use pdfcer_core::edit::EditSession;
use pdfcer_core::page_tree::Page;
use pdfcer_core::running_text::{RunningText, RunningTextAlign, RunningTextKind};
use pdfcer_gui_base::settings::SettingsExt as _;

use crate::app::state::OpenDoc;
use crate::text::print::running as t;
use crate::text::running as tr;

/// The published region of the header text box.
pub(super) const REGION_HEADER: &str = "print.running.header";
/// The published region of the footer text box.
pub(super) const REGION_FOOTER: &str = "print.running.footer";

/// One edge's line.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
struct Line {
    text: String,
    align: RunningTextAlign,
}

/// What a copy was built from.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Key {
    serial: u64,
    epoch: u64,
    lines: [Line; 2],
}

/// The stamped copy the preview and the printer render from.
pub(crate) struct Copy {
    session: Arc<EditSession>,
    pages: Vec<Page>,
    /// A digest of what it was built from, never 0; for the preview's keys.
    id: u64,
}

/// The pages and session to render: the document's own, or the copy's.
#[derive(Clone, Copy)]
pub(crate) struct Source<'a> {
    pub(crate) session: &'a Arc<EditSession>,
    pub(crate) pages: &'a [Page],
    /// The copy's id, or 0 for the document: what a trace reports rendered.
    pub(crate) stamp: u64,
}

impl<'a> Source<'a> {
    /// The copy when there is one, else the document.
    pub(crate) fn of(doc: &'a OpenDoc, copy: Option<&'a Copy>) -> Self {
        match copy {
            Some(c) => Self {
                session: &c.session,
                pages: &c.pages,
                stamp: c.id,
            },
            None => Self {
                session: &doc.session,
                pages: &doc.pages,
                stamp: 0,
            },
        }
    }
}

/// The dialog's header and footer; per job, never remembered.
#[derive(Default)]
pub(crate) struct State {
    /// Header then footer, as typed.
    draft: [Line; 2],
    /// Header then footer, as the copy carries them.
    applied: [Line; 2],
    /// What the last build was for, whether or not it succeeded.
    built: Option<Key>,
    copy: Option<Arc<Copy>>,
    /// Why the last build failed.
    error: Option<String>,
}

impl State {
    /// The copy to render from, when a line is applied and it built.
    pub(crate) fn copy(&self) -> Option<Arc<Copy>> {
        self.copy.clone()
    }

    /// The preview cache's stamp: 0 with no copy.
    pub(crate) fn stamp(&self) -> u64 {
        self.copy.as_ref().map_or(0, |c| c.id)
    }

    /// Why the printed copy cannot carry the lines, when it cannot.
    pub(crate) fn error(&self) -> Option<&str> {
        self.error.as_deref()
    }

    /// Take the typed lines as applied. The commit calls this so a line still
    /// being typed prints.
    pub(crate) fn apply_draft(&mut self) {
        self.applied.clone_from(&self.draft);
    }

    /// Bring the copy up to date with the applied lines and `doc`.
    pub(crate) fn ensure(&mut self, doc: &OpenDoc) {
        let stamps = stamps(&self.applied);
        if stamps.is_empty() {
            self.built = None;
            self.copy = None;
            self.error = None;
            return;
        }
        let key = Key {
            serial: doc.serial,
            epoch: doc.edit_epoch,
            lines: self.applied.clone(),
        };
        if self.built.as_ref() == Some(&key) {
            return;
        }
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        key.hash(&mut hasher);
        let id = hasher.finish().max(1);
        self.built = Some(key);
        match build(doc, &stamps, id) {
            Ok(copy) => {
                self.copy = Some(Arc::new(copy));
                self.error = None;
            }
            Err(why) => {
                // ui-text-exempt: diagnostic trace
                crate::diag::trace(|| format!("print-running refused={why:?}"));
                self.copy = None;
                self.error = Some(why);
            }
        }
    }
}

/// The engine's requests for the non-empty lines.
fn stamps(lines: &[Line; 2]) -> Vec<RunningText> {
    [RunningTextKind::Header, RunningTextKind::Footer]
        .into_iter()
        .zip(lines)
        .filter(|(_, line)| !line.text.trim().is_empty())
        .map(|(kind, line)| {
            let mut stamp = RunningText::new(kind, line.text.clone());
            stamp.align = line.align;
            stamp
        })
        .collect()
}

/// Reopen `doc` from its current bytes and stamp `stamps` on every page.
fn build(doc: &OpenDoc, stamps: &[RunningText], id: u64) -> Result<Copy, String> {
    for stamp in stamps {
        stamp.check().map_err(|e| tr::problem(&e))?;
    }
    let options = doc.settings.save_options();
    let (bytes, _) = doc
        .session
        .to_incremental_bytes(&options)
        .map_err(|e| e.to_string())?;
    let base = pdfcer_core::document::Document::from_bytes(bytes).map_err(|e| e.to_string())?;
    let mut session = doc.settings.open_session(base);
    let mut firsts = Vec::with_capacity(stamps.len());
    for stamp in stamps {
        let out = session
            .stamp_running_text(stamp)
            .map_err(|e| e.to_string())?;
        firsts.push(out.first_label);
    }
    let pages = session.pages().map_err(|e| e.to_string())?;
    // ui-text-exempt: diagnostic trace
    crate::diag::trace(|| {
        format!(
            "print-running built pages={} lines={} first={:?}",
            pages.len(),
            stamps.len(),
            firsts.join("|")
        )
    });
    Ok(Copy {
        session: Arc::new(session),
        pages,
        id,
    })
}

/// The section on the Comments tab.
pub(super) fn controls(ui: &mut Ui, state: &mut State) {
    ui.add_space(12.0);
    ui.label(egui::RichText::new(t::heading()).strong());
    ui.label(t::intro());
    let mut focused = false;
    let rows = [(t::header(), REGION_HEADER), (t::footer(), REGION_FOOTER)];
    for (line, (label, region)) in state.draft.iter_mut().zip(rows) {
        ui.horizontal(|ui| {
            ui.label(label);
            // escape-disposition: keeps-draft — the line lives in `draft`, so
            // Escape leaves the box with the words in it, and applies them.
            let edit = ui
                .add(egui::TextEdit::singleline(&mut line.text).desired_width(220.0))
                .on_hover_text(t::line_tooltip());
            crate::diag::ui_rect_visible(region, edit.rect, ui.clip_rect());
            focused |= edit.has_focus();
            egui::ComboBox::from_id_salt(region)
                .selected_text(tr::align(line.align))
                .show_ui(ui, |ui| {
                    for align in [
                        RunningTextAlign::Left,
                        RunningTextAlign::Center,
                        RunningTextAlign::Right,
                    ] {
                        ui.selectable_value(&mut line.align, align, tr::align(align));
                    }
                });
        });
    }
    if !focused {
        state.apply_draft();
    }
    if let Some(why) = state.error() {
        let colour = ui.visuals().warn_fg_color;
        ui.colored_label(colour, t::refused(why));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_non_empty_lines_are_stamped_and_each_keeps_its_edge() {
        let lines = [
            Line {
                text: "  ".into(),
                align: RunningTextAlign::Left,
            },
            Line {
                text: "Page {page}".into(),
                align: RunningTextAlign::Right,
            },
        ];
        let s = stamps(&lines);
        assert_eq!(s.len(), 1);
        assert_eq!(s[0].kind, RunningTextKind::Footer);
        assert_eq!(s[0].align, RunningTextAlign::Right);
    }

    #[test]
    fn nothing_typed_is_no_copy() {
        let state = State::default();
        assert!(state.copy().is_none());
        assert_eq!(state.stamp(), 0);
    }
}
