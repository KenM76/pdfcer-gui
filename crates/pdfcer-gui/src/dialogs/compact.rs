//! # `dialogs::compact` — the window before a full rewrite
//!
//! `file.save_compacted`, wired 2026-08-28. `OPERATOR_REQUESTS.md` **O48**,
//! answered *"yes to all three"*.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/dialogs/compact.md`.

use egui::Ui;
use pdfcer_core::space_audit::SpaceCategory;

use crate::app::actions::Action;
use crate::app::state::{OpenDoc, Status};
use crate::text::compact as t;

/// The window body's rect, for `ui-verify`.
// ui-text-exempt: trace region name, never displayed
pub const REGION_BODY: &str = "compact.body";
/// The button that goes on to the picker.
// ui-text-exempt: trace region name, never displayed
pub const REGION_SAVE: &str = "compact.commit";

/// The window, and the bytes it measured.
pub struct CompactDialog {
    /// The compacted file, already serialised. See the header.
    bytes: Vec<u8>,
    /// What the document occupies on disk now.
    ///
    /// Read from the **file**, not from the session's own byte count, because
    /// the number the operator compares against is the one Explorer shows them.
    ///
    /// `0` for a document that has never been saved — a blank one — and that
    /// falls out correctly rather than by accident: `size_change` compares
    /// `after >= before`, so a zero `before` takes the *"no smaller"* branch and
    /// says the honest thing. A file that does not exist yet cannot be shrunk,
    /// and the alternative — an `Option` and a fourth sentence — would be a
    /// branch for a case the arithmetic already answers.
    /// Read from the **file** each time the window opens, never cached: the
    /// operator may have saved since, and a stale `before` would quote a saving
    /// against a file that is no longer there.
    before: u64,
    /// How many digital signatures the copy will not keep.
    signatures: usize,
    /// The file on disk, audited for the breakdown's Now column.
    path: std::path::PathBuf,
    /// The breakdown, measured the first time its fold is opened; the inner
    /// `None` is a copy that would not parse back.
    space: Option<Option<Breakdown>>,
    save_requested: bool,
    close_requested: bool,
}

impl CompactDialog {
    /// Serialise the compacted copy and open on the result.
    ///
    /// `None` when the engine refuses the rewrite — see [`open_for`], which
    /// turns that into a sentence rather than into silence.
    pub fn open(doc: &OpenDoc) -> Result<Self, String> {
        use crate::app::settings::SettingsExt;
        // The SAME `SaveOptions` the ordinary save uses. Two settings ride on
        // it — the cross-reference entry line ending and the trailing newline —
        // and both change the bytes of the file the operator receives. A
        // `::default()` here would produce a compacted copy that differed from
        // an ordinary one in ways nobody chose, on a command whose whole subject
        // is the bytes.
        let (bytes, _report) = doc
            .session
            .to_full_bytes(&doc.settings.save_options())
            .map_err(|error| t::refused(&error.to_string()))?;
        let before = std::fs::metadata(&doc.path).map(|m| m.len()).unwrap_or(0);
        Ok(Self {
            bytes,
            before,
            signatures: doc.session.signature_census().signatures,
            path: doc.path.clone(),
            space: None,
            save_requested: false,
            close_requested: false,
        })
    }

    /// Draw it. Returns whether it stays open.
    pub fn show(&mut self, ctx: &egui::Context, actions: &mut Vec<Action>) -> bool {
        let (frame, ()) = crate::dialogs::host::Host::new(
            "compact-copy", // ui-text-exempt: a viewport key, never displayed.
            t::window_title(),
            egui::vec2(520.0, 340.0),
            egui::vec2(380.0, 240.0),
        )
        .show(ctx, |ui| {
            crate::diag::ui_rect(REGION_BODY, ui.max_rect());
            self.body(ui);
        });
        let open = !frame.closed;

        if std::mem::take(&mut self.save_requested) {
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed
                format!(
                    "compact-requested before={} after={} signatures={}",
                    self.before,
                    self.bytes.len(),
                    self.signatures
                )
            });
            // The measured bytes travel with the action. They are the operand
            // — see the header — and rebuilding them in the apply arm would put
            // a second serialisation between what the window promised and what
            // the operator receives.
            actions.push(Action::Write(
                crate::app::actions::write::WriteAction::Compacted {
                    bytes: std::mem::take(&mut self.bytes),
                    before: self.before,
                },
            ));
            return false;
        }
        open && !std::mem::take(&mut self.close_requested)
    }

    fn body(&mut self, ui: &mut Ui) {
        ui.label(t::intro());
        ui.add_space(8.0);
        ui.label(t::size_change(self.before, self.bytes.len() as u64));
        ui.add_space(8.0);
        ui.small(t::revisions_line());
        // The signature warning LAST of the three and drawn full-size, not
        // `small`, when it applies. It is the only irreversible loss in the
        // window and the only one most documents do not have — so it is
        // conditional, and where it appears it is the thing the eye lands on.
        if self.signatures > 0 {
            ui.add_space(8.0);
            ui.label(t::signature_line(self.signatures));
        }

        ui.add_space(8.0);
        self.space_fold(ui);

        ui.add_space(12.0);
        ui.separator();
        ui.horizontal(|ui| {
            // Never greyed. Every state this window can be in is one an
            // operator may legitimately proceed from — including "no smaller",
            // which is an accurate answer about a tidy file and not a reason to
            // refuse them a copy they asked for.
            let save = ui.button(t::save_button());
            crate::diag::ui_rect_visible(REGION_SAVE, save.rect, ui.clip_rect());
            if save.clicked() {
                self.save_requested = true;
            }
            if ui.button(t::cancel_button()).clicked() {
                self.close_requested = true;
            }
        });
    }
}

impl CompactDialog {
    /// The fold listing where the bytes go. Closed by default: auditing a
    /// dense file twice is not free, so it is paid only when asked for.
    fn space_fold(&mut self, ui: &mut Ui) {
        let fold = egui::CollapsingHeader::new(t::space_heading())
            .id_salt("compact-space") // ui-text-exempt: an id, never displayed
            .show(ui, |ui| {
                let (path, bytes) = (&self.path, &self.bytes);
                let space = self.space.get_or_insert_with(|| {
                    let started = std::time::Instant::now();
                    let now = std::fs::read(path).ok();
                    let measured = breakdown(now.as_deref(), bytes);
                    crate::diag::trace(|| {
                        format!(
                            "compact-space rows={} ms={}", // ui-text-exempt: diagnostic trace
                            measured.as_ref().map_or(0, |b| b.rows.len()),
                            started.elapsed().as_millis()
                        )
                    });
                    measured
                });
                if let Some(space) = space {
                    space_table(ui, space);
                }
            });
        fold.header_response.on_hover_text(t::space_tooltip());
    }
}

fn space_table(ui: &mut Ui, space: &Breakdown) {
    egui::ScrollArea::vertical()
        .max_height(220.0)
        .show(ui, |ui| {
            egui::Grid::new("compact-space-grid") // ui-text-exempt: an id, never displayed
                .striped(true)
                .show(ui, |ui| {
                    ui.label(t::space_column_what());
                    ui.label(t::space_column_now());
                    ui.label(t::space_column_copy());
                    ui.end_row();
                    for &(category, now, copy) in &space.rows {
                        ui.label(t::space_category(category));
                        ui.label(now.map_or_else(|| t::space_no_file().to_owned(), t::space_bytes));
                        ui.label(t::space_bytes(copy));
                        ui.end_row();
                    }
                });
            for note in &space.notes {
                ui.small(note.as_str());
            }
        });
}
/// Where the bytes of the file and of the copy go, by category.
pub struct Breakdown {
    /// `(category, now, copy)`, largest first by the larger of the two;
    /// categories empty in both are left out. `now` is `None` for a document
    /// with no file.
    rows: Vec<(SpaceCategory, Option<u64>, u64)>,
    /// What either audit could not measure, in the engine's sentences.
    notes: Vec<String>,
}

/// Audit `now` (the file on disk, when there is one) and `copy`.
///
/// `None` when the copy does not parse back, which would be an engine defect:
/// the window then lists nothing rather than a breakdown of half the trade.
pub fn breakdown(now: Option<&[u8]>, copy: &[u8]) -> Option<Breakdown> {
    use pdfcer_core::document::Document;
    use pdfcer_core::space_audit::audit;
    let copy = audit(&Document::from_bytes(copy.to_vec()).ok()?);
    let now = now
        .and_then(|b| Document::from_bytes(b.to_vec()).ok())
        .map(|d| audit(&d));
    let mut rows: Vec<_> = copy
        .categories
        .iter()
        .map(|c| {
            (
                c.category,
                now.as_ref().map(|n| n.bytes_in(c.category)),
                c.bytes,
            )
        })
        .filter(|&(_, now, copy)| copy > 0 || now.is_some_and(|n| n > 0))
        .collect();
    rows.sort_by_key(|&(_, now, copy)| std::cmp::Reverse(now.unwrap_or(0).max(copy)));
    let mut notes = now.map(|n| n.notes).unwrap_or_default();
    notes.extend(copy.notes);
    notes.dedup();
    Some(Breakdown { rows, notes })
}

/// Build it for the current document, or answer why not.
pub fn open_for(status: &Status) -> Option<Result<CompactDialog, String>> {
    match status {
        Status::Open(doc) => Some(CompactDialog::open(doc)),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `hello.pdf` with an incremental update that replaces its content
    /// stream, so the replaced body is still in the file.
    fn updated() -> (Vec<u8>, Vec<u8>) {
        let path = crate::panels::objects::test_support::engine_fixture("hello.pdf");
        let clean = std::fs::read(path).expect("fixture");
        let mut file = clean.clone();
        let body = file.len() + 1;
        file.extend_from_slice(b"\n5 0 obj\n<< /Length 0 >>\nstream\n\nendstream\nendobj\n");
        let xref = file.len();
        file.extend_from_slice(
            format!(
                "xref\n0 1\n0000000000 65535 f\r\n5 1\n{body:010} 00000 n\r\n\
                 trailer\n<< /Size 6 /Root 1 0 R /Prev 526 >>\nstartxref\n{xref}\n%%EOF\n"
            )
            .as_bytes(),
        );
        (file, clean)
    }

    /// Each column adds up to its file, the replaced body shows as space the
    /// copy drops, and the rows run largest first with no empty ones.
    #[test]
    fn the_breakdown_accounts_for_every_byte_of_both_files() {
        let (now, copy) = updated();
        let space = breakdown(Some(&now), &copy).expect("both parse");
        let now_sum: u64 = space.rows.iter().filter_map(|r| r.1).sum();
        let copy_sum: u64 = space.rows.iter().map(|r| r.2).sum();
        assert_eq!(now_sum, now.len() as u64);
        assert_eq!(copy_sum, copy.len() as u64);
        let superseded = space.rows.iter().find(|r| r.0 == SpaceCategory::Superseded);
        assert!(
            matches!(superseded, Some(&(_, Some(n), 0)) if n > 0),
            "{:?}",
            space.rows
        );
        let sizes: Vec<u64> = space
            .rows
            .iter()
            .map(|r| r.1.unwrap_or(0).max(r.2))
            .collect();
        assert!(sizes.windows(2).all(|w| w[0] >= w[1]), "{sizes:?}");
        assert!(sizes.iter().all(|&n| n > 0));
    }

    /// A document with no file yet is broken down for the copy alone.
    #[test]
    fn an_unsaved_document_has_no_now_column() {
        let (_, copy) = updated();
        let space = breakdown(None, &copy).expect("parses");
        assert!(space.rows.iter().all(|r| r.1.is_none()));
        assert_eq!(
            space.rows.iter().map(|r| r.2).sum::<u64>(),
            copy.len() as u64
        );
    }
}
