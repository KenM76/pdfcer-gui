//! What travels when a document moves to another window, and how the window it
//! arrives in takes it.
//!
//! A document travels as its path, the page on screen, the password it was
//! opened with, and — when it has unsaved edits — a copy of itself written
//! through the save path into `carry/` beside the discovery files. The
//! receiver reads the copy in place of the file, binds it to the original
//! path, deletes it, and counts its edits as unsaved, so Save writes them
//! over the original. The password crosses only the pipe, which admits the
//! current user alone, so the receiver never asks for it.
//!
//! The undo history stays behind: the engine cannot hand a session's steps to
//! another process, and the receiver says so off-canvas.
//!
//! The sender closes its tab on the receiver's `ok`, which is answered once
//! the document is open there, and only when no edit was made while the
//! request was in flight.

use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use super::wire::{Arrival, Carry};
use crate::app::PdfcerApp;
use crate::app::state::{Origin, Status};
use crate::text::siblings::{self as t, WindowMoveRefusal};

/// The carry directory's name inside the discovery directory.
const CARRY_DIR: &str = "carry"; // ui-text-exempt: a directory name, never displayed

/// A carry file older than this belongs to no move still waiting on an answer:
/// it was left by a window that died between writing it and hearing back.
const CARRY_AGE: Duration = Duration::from_secs(600);

/// A document packed to move.
pub(super) struct Parcel {
    /// The tab's own path, as `slot_of_path` compares it.
    pub(super) path: PathBuf,
    /// What is sent.
    pub(super) arrival: Arrival,
    /// The document's edit epoch when it was packed. A later edit keeps the
    /// tab, because the other window has the document without it.
    pub(super) epoch: u64,
}

impl Parcel {
    /// Whether the document needs a window already listening on its pipe: a
    /// path and page fit on a command line, a copy and a password do not.
    pub(super) fn needs_pipe(&self) -> bool {
        self.arrival.carry.is_some() || self.arrival.key.is_some()
    }
}

/// Delete `arrival`'s carry file, if it has one.
pub(super) fn discard(arrival: &Arrival) {
    if let Some(carry) = &arrival.carry {
        let _ = std::fs::remove_file(&carry.file);
    }
}

/// Delete the carry files older than [`CARRY_AGE`].
pub(super) fn prune(dir: &Path) {
    for entry in std::fs::read_dir(dir.join(CARRY_DIR))
        .into_iter()
        .flatten()
        .flatten()
    {
        let stale = entry
            .metadata()
            .and_then(|m| m.modified())
            .ok()
            .and_then(|t| t.elapsed().ok())
            .is_some_and(|age| age > CARRY_AGE);
        if stale {
            let _ = std::fs::remove_file(entry.path());
        }
    }
}

/// A carry-file path no other move uses.
fn carry_path(dir: &Path) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_nanos());
    dir.join(CARRY_DIR)
        .join(format!("{}-{nanos}.pdf", std::process::id()))
}

impl PdfcerApp {
    /// Whether the document in `slot` can move: opened from a file that still
    /// exists, with no redaction staged.
    pub(super) fn movable(&self, slot: usize) -> bool {
        self.refusal(slot).is_none()
    }

    /// Why the document in `slot` cannot move, when it cannot.
    pub(super) fn refusal(&self, slot: usize) -> Option<WindowMoveRefusal> {
        match self.slot(slot) {
            Some(Status::Open(doc)) if doc.origin == Origin::Opened && doc.path.is_file() => doc
                .session
                .has_pending_redaction()
                .then_some(WindowMoveRefusal::RedactionStaged),
            _ => Some(WindowMoveRefusal::NoFile),
        }
    }

    /// Pack the document in `slot` to move, writing its carry file when it
    /// has unsaved edits, or say why it cannot move.
    pub(super) fn parcel(
        &self,
        slot: usize,
        dir: Option<&Path>,
    ) -> Result<Parcel, WindowMoveRefusal> {
        if let Some(why) = self.refusal(slot) {
            return Err(why);
        }
        let Some(Status::Open(doc)) = self.slot(slot) else {
            return Err(WindowMoveRefusal::NoFile);
        };
        let carry = if crate::app::save::has_unsaved_edits(doc) {
            let dir =
                dir.ok_or_else(|| WindowMoveRefusal::NotCarried(t::no_carry_folder().to_owned()))?;
            let file = carry_path(dir);
            if let Some(parent) = file.parent() {
                std::fs::create_dir_all(parent)
                    .map_err(|e| WindowMoveRefusal::NotCarried(e.to_string()))?;
            }
            crate::app::save::carry_copy(doc, &file).map_err(WindowMoveRefusal::NotCarried)?;
            Some(Carry {
                file,
                edits: crate::app::save::unsaved_edit_count(doc).max(1),
            })
        } else {
            None
        };
        Ok(Parcel {
            path: doc.path.clone(),
            arrival: Arrival {
                path: crate::app::remote::absolute(&doc.path),
                page: Some(doc.view.page_index),
                carry,
                key: doc.opened_with.clone(),
            },
            epoch: doc.edit_epoch,
        })
    }

    /// Open a document another window sent, answering whether it is open
    /// here now. The carry file is deleted whatever the outcome.
    pub(super) fn receive(&mut self, arrival: &Arrival) -> Result<(), String> {
        let outcome = self.take(arrival);
        discard(arrival);
        outcome
    }

    fn take(&mut self, arrival: &Arrival) -> Result<(), String> {
        if let Some(slot) = self.slot_of_path(&arrival.path) {
            // A copy carrying edits beside the file already open here would be
            // two documents of one file; the sender keeps its own.
            if arrival.carry.is_some() {
                return Err(t::already_open().to_owned());
            }
            if matches!(self.slot(slot), Some(Status::Open(_))) {
                self.activate_slot(slot);
                return Ok(());
            }
            // A tab that failed to open the file, or is waiting for its
            // password, gives way to the document arriving.
            self.close_slot(slot);
        }
        let source = arrival.carry.as_ref().map(|c| c.file.as_path());
        self.open_arrival(arrival.path.clone(), source, arrival.key.as_ref());
        let Status::Open(doc) = &mut self.status else {
            self.close_slot(self.active_slot);
            return Err(t::not_opened().to_owned());
        };
        if let Some(carry) = &arrival.carry {
            doc.carried_edits = carry.edits;
            crate::app::actions::record_note(doc.edit_epoch, t::arrived_with_edits(carry.edits));
        }
        if let Some(page) = arrival.page {
            self.show_page(page);
        }
        Ok(())
    }
}
