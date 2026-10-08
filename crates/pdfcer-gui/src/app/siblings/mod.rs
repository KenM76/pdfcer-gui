//! Moving a document between pdfcer-gui windows: View ▸ Window ▸ Move to
//! other window, and Move to new window, on the ribbon and on a tab's menu.
//!
//! A move hands the other window the file's path; it opens the file from disk
//! and this window closes its tab once the other has taken it. A document can
//! therefore move only when what is on screen is what is on disk: opened from
//! a file, with no unsaved edits. A window whose last document moves away
//! closes, as a browser window does when its last tab is dragged out.
//!
//! Frame order: [`PdfcerApp::siblings_poll`] beside `remote_poll`, and
//! [`PdfcerApp::siblings_picker`] after the actions apply.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/app/siblings.md`.

mod wire;

use std::path::{Path, PathBuf};
use std::sync::mpsc::{Receiver, Sender, channel};
use std::time::{Duration, Instant};

use eframe::egui;

use super::PdfcerApp;
use crate::app::state::{Origin, Status};
use crate::text::siblings::{self as t, WindowMoveRefusal};
use wire::Peer;

/// How long a peer list is reused before the discovery directory is read
/// again.
const PEERS_FRESH: Duration = Duration::from_secs(1);

/// The environment variable that places a window for the test harness.
const VIEWPORT_ENV: &str = "PDFCER_DIAG_VIEWPORT"; // ui-text-exempt: an environment variable name

/// The prefix of every diagnostic variable; see [`spawn_window`].
const DIAG_PREFIX: &str = "PDFCER_DIAG_"; // ui-text-exempt: an environment variable prefix

/// The answer to one send, carried back to the frame.
struct Reply {
    pid: u32,
    /// The tab's own path, as `slot_of_path` compares it.
    path: PathBuf,
    result: Result<(), String>,
}

/// The window-to-window state.
pub struct Siblings {
    started: bool,
    dir: Option<PathBuf>,
    inbox: Option<Receiver<PathBuf>>,
    peers: Vec<Peer>,
    peers_read: Option<Instant>,
    published: Option<String>,
    replies: (Sender<Reply>, Receiver<Reply>),
    /// The document whose destination the picker is asking for.
    picking: Option<PathBuf>,
}

impl Default for Siblings {
    fn default() -> Self {
        Self {
            started: false,
            dir: None,
            inbox: None,
            peers: Vec::new(),
            peers_read: None,
            published: None,
            replies: channel(),
            picking: None,
        }
    }
}

impl std::fmt::Debug for Siblings {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Siblings")
            .field("dir", &self.dir)
            .field("peers", &self.peers)
            .finish_non_exhaustive()
    }
}

impl Drop for Siblings {
    fn drop(&mut self) {
        if let Some(dir) = &self.dir {
            wire::withdraw(dir);
        }
    }
}

impl Siblings {
    /// Start listening on first use. Unit tests never listen: many harness
    /// apps share one process id, and so one pipe name.
    fn ensure_started(&mut self, ctx: &egui::Context) {
        if self.started || cfg!(test) {
            return;
        }
        self.started = true;
        let Some(dir) = wire::discovery_dir() else {
            return;
        };
        wire::prune(&dir);
        let (inbox, rx) = channel();
        let pipe = wire::own_pipe();
        let wake = ctx.clone();
        match wire::serve(pipe.clone(), inbox, move || wake.request_repaint()) {
            Ok(()) => {
                self.inbox = Some(rx);
                self.dir = Some(dir);
                // ui-text-exempt: diagnostic trace, never displayed
                crate::diag::trace(|| {
                    format!("window-pipe pid={} pipe={pipe}", std::process::id())
                });
            }
            // ui-text-exempt: diagnostic trace, never displayed
            Err(e) => crate::diag::trace(|| format!("window-pipe-failed {e}")),
        }
    }

    /// Re-read the other windows when the list is older than [`PEERS_FRESH`].
    fn refresh_peers(&mut self) {
        let Some(dir) = &self.dir else {
            return;
        };
        if self.peers_read.is_some_and(|at| at.elapsed() < PEERS_FRESH) {
            return;
        }
        self.peers = wire::peers(dir);
        self.peers_read = Some(Instant::now());
        let pids: Vec<String> = self.peers.iter().map(|p| p.pid.to_string()).collect();
        crate::diag::trace_changed("window-peers", || {
            // ui-text-exempt: diagnostic trace, never displayed
            format!(
                "window-peers pid={} n={} peers={}",
                std::process::id(),
                pids.len(),
                pids.join(",")
            )
        });
    }

    /// Publish this window's open documents when they changed.
    fn publish(&mut self, documents: String) {
        let Some(dir) = &self.dir else {
            return;
        };
        if self.published.as_ref() == Some(&documents) {
            return;
        }
        if wire::publish(dir, &wire::own_pipe(), &documents).is_ok() {
            self.published = Some(documents);
        }
    }

    /// Ask `peer` to open `path` without blocking the frame; the answer
    /// arrives through [`Self::replies`].
    fn send(&self, ctx: &egui::Context, peer: &Peer, path: &Path) {
        // ui-text-exempt: diagnostic trace, never displayed
        crate::diag::trace(|| format!("window-move-sending to={} path={path:?}", peer.pid));
        let replies = self.replies.0.clone();
        let wake = ctx.clone();
        let (pid, pipe, path) = (peer.pid, peer.pipe.clone(), path.to_path_buf());
        std::thread::spawn(move || {
            let result = wire::send_open(&pipe, &super::remote::absolute(&path));
            let _ = replies.send(Reply { pid, path, result });
            wake.request_repaint();
        });
    }
}

/// Whether this window was placed by the test harness, which also means it
/// must never take the focus.
fn harness_placed() -> bool {
    std::env::var_os(VIEWPORT_ENV).is_some()
}

/// Start another copy of this program on `path`.
///
/// Every `PDFCER_DIAG_*` variable but the placement is removed first, so a
/// window the test harness placed off the desktop starts its child there too
/// and nothing that drove the parent drives the child. The child's output goes
/// nowhere: inherited, its trace would interleave with the parent's, and a
/// harness reading the parent's would act on the child's regions.
fn spawn_window(path: &Path) -> std::io::Result<u32> {
    use std::process::Stdio;
    let exe = std::env::current_exe()?;
    let mut command = std::process::Command::new(exe);
    command
        .arg(super::remote::absolute(path))
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    for (name, _) in std::env::vars_os() {
        let name = name.to_string_lossy();
        if name.starts_with(DIAG_PREFIX) && name != VIEWPORT_ENV {
            command.env_remove(name.as_ref());
        }
    }
    command.spawn().map(|child| child.id())
}

impl PdfcerApp {
    /// The path of the document in `slot` when it may move to another
    /// window: opened from a file that still exists, with no unsaved edits.
    fn movable(&self, slot: usize) -> Option<PathBuf> {
        match self.slot(slot) {
            Some(Status::Open(doc))
                if doc.origin == Origin::Opened
                    && !crate::app::save::has_unsaved_edits(doc)
                    && doc.path.is_file() =>
            {
                Some(doc.path.clone())
            }
            _ => None,
        }
    }

    /// Set `docs.tear_off` and `docs.move_to_window` for the document in
    /// `slot`.
    pub(super) fn window_conditions(
        &self,
        set: &mut egui_shell::commands::ConditionSet,
        slot: usize,
    ) {
        set.clear("docs.tear_off");
        set.clear("docs.move_to_window");
        if self.movable(slot).is_none() {
            return;
        }
        if self.document_count() > 1 {
            set.set("docs.tear_off");
        }
        if !self.siblings.peers.is_empty() {
            set.set("docs.move_to_window");
        }
    }

    /// Take what other windows sent, settle this window's own sends, and keep
    /// the discovery file current.
    pub(super) fn siblings_poll(&mut self, ctx: &egui::Context) {
        self.siblings.ensure_started(ctx);
        let received: Vec<PathBuf> = self
            .siblings
            .inbox
            .as_ref()
            .map(|rx| rx.try_iter().collect())
            .unwrap_or_default();
        for path in received {
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed
                format!(
                    "window-move-received pid={} path={path:?}",
                    std::process::id()
                )
            });
            self.open_path(path);
            if !harness_placed() {
                ctx.send_viewport_cmd(egui::ViewportCommand::Focus);
            }
        }
        let replies: Vec<Reply> = self.siblings.replies.1.try_iter().collect();
        for reply in replies {
            self.settle_send(ctx, reply);
        }
        self.siblings.refresh_peers();
        let documents = self.document_names();
        self.siblings.publish(documents);
    }

    /// Close the tab another window has taken, or say why it did not.
    fn settle_send(&mut self, ctx: &egui::Context, reply: Reply) {
        if let Err(why) = reply.result {
            // ui-text-exempt: diagnostic trace, never displayed
            crate::diag::trace(|| format!("window-move-refused to={} why={why:?}", reply.pid));
            crate::app::status::decline::record_window_move(WindowMoveRefusal::NotSent(why));
            return;
        }
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed
            format!("window-move-sent to={} path={:?}", reply.pid, reply.path)
        });
        // An edit made while the send was in flight keeps the tab: the other
        // window has the file as saved, and closing here would discard the
        // edit.
        if let Some(slot) = self.slot_of_path(&reply.path)
            && self.movable(slot).is_some()
        {
            self.close_slot(slot);
        }
        self.close_if_emptied(ctx);
    }

    /// A window whose last document moved away has nothing left to show.
    fn close_if_emptied(&self, ctx: &egui::Context) {
        if self.document_count() == 0 {
            // ui-text-exempt: diagnostic trace, never displayed
            crate::diag::trace(|| format!("window-emptied pid={}", std::process::id()));
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }
    }

    /// The file names of the open documents, as the discovery file carries
    /// them for another window's picker.
    fn document_names(&self) -> String {
        (0..self.document_count())
            .filter_map(|slot| match self.slot(slot)? {
                Status::Open(doc) => Some(doc.path.clone()),
                Status::Failed { path, .. }
                | Status::Unsupported { path, .. }
                | Status::NeedsPassword { path, .. } => Some(path.clone()),
                Status::Empty => None,
            })
            .filter_map(|p| p.file_name().map(|n| n.to_string_lossy().into_owned()))
            .collect::<Vec<_>>()
            .join(t::document_separator())
    }

    /// `view.move_to_new_window`: open the document in `slot` in a window of
    /// its own and close it here.
    pub(super) fn move_to_new_window(&mut self, slot: usize) {
        let Some(path) = self.movable(slot) else {
            crate::app::status::decline::record_window_move(WindowMoveRefusal::Unsaved);
            return;
        };
        match spawn_window(&path) {
            Ok(pid) => {
                // ui-text-exempt: diagnostic trace, never displayed
                crate::diag::trace(|| format!("window-torn-off pid={pid} path={path:?}"));
                self.close_slot(slot);
            }
            Err(e) => crate::app::status::decline::record_window_move(
                WindowMoveRefusal::NotStarted(e.to_string()),
            ),
        }
    }

    /// `view.move_to_window`: send the document in `slot` to the one other
    /// window, or ask which when there are several.
    pub(super) fn move_to_window(&mut self, ctx: &egui::Context, slot: usize) {
        let Some(path) = self.movable(slot) else {
            crate::app::status::decline::record_window_move(WindowMoveRefusal::Unsaved);
            return;
        };
        match self.siblings.peers.as_slice() {
            [] => crate::app::status::decline::record_window_move(WindowMoveRefusal::NotSent(
                t::no_other_window().to_owned(),
            )),
            [only] => self.siblings.send(ctx, only, &path),
            _ => self.siblings.picking = Some(path),
        }
    }

    /// The window that asks which other window a document goes to, while one
    /// is being asked.
    pub(super) fn siblings_picker(&mut self, ctx: &egui::Context) {
        let Some(path) = self.siblings.picking.clone() else {
            return;
        };
        let mut chosen: Option<Option<Peer>> = None;
        egui::Window::new(t::picker_title())
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
            .show(ctx, |ui| {
                ui.label(t::picker_prompt());
                for peer in &self.siblings.peers {
                    let button = ui.button(t::picker_window(&peer.documents));
                    crate::diag::ui_rect(&format!("window-pick.{}", peer.pid), button.rect);
                    if button.clicked() {
                        chosen = Some(Some(peer.clone()));
                    }
                }
                let cancel = ui.button(t::picker_cancel());
                crate::diag::ui_rect("window-pick.cancel", cancel.rect);
                if cancel.clicked() {
                    chosen = Some(None);
                }
            });
        let Some(choice) = chosen else {
            return;
        };
        self.siblings.picking = None;
        if let Some(peer) = choice {
            self.siblings.send(ctx, &peer, &path);
        }
    }
}
