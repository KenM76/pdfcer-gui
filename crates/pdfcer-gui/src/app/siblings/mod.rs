//! Moving a document between pdfcer-gui windows: View ▸ Window ▸ Move to
//! other window, and Move to new window, on the ribbon and on a tab's menu.
//!
//! A move hands the other window the document — its path, the page on
//! screen, its unsaved edits and the password it was opened with (`parcel`) —
//! and this window closes its tab once the other has it open. A document with
//! no file behind it, or with a redaction staged, does not move. A window
//! whose last document moves away closes, as a browser window does when its
//! last tab is dragged out. Dragging a tab off the strip does the same, by
//! where it is dropped (`drag`); a selection dragged onto another window is
//! pasted there (`objdrop`).
//!
//! Frame order: [`PdfcerApp::siblings_poll`] beside `remote_poll`, and
//! [`PdfcerApp::siblings_picker`] after the actions apply.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/app/siblings.md`.

mod drag;
mod objdrop;
mod parcel;
mod wire;

pub(crate) use objdrop::selection_released;

use std::path::{Path, PathBuf};
use std::sync::mpsc::{Receiver, Sender, channel};
use std::time::{Duration, Instant};

use eframe::egui;

use super::PdfcerApp;
use crate::app::actions::Action;
use crate::app::state::Status;
use crate::text::siblings::{self as t, WindowMoveRefusal};
use parcel::Parcel;
use wire::{Incoming, Peer, Request};

/// How long a peer list is reused before the discovery directory is read
/// again.
const PEERS_FRESH: Duration = Duration::from_secs(1);

/// The environment variable that places a window for the test harness.
const VIEWPORT_ENV: &str = "PDFCER_DIAG_VIEWPORT"; // ui-text-exempt: an environment variable name

/// The prefix of every diagnostic variable; see [`spawn_window`].
const DIAG_PREFIX: &str = "PDFCER_DIAG_"; // ui-text-exempt: an environment variable prefix

/// How long a torn-off window has to start listening before the document
/// meant for it is kept here instead.
const STARTUP_WAIT: Duration = Duration::from_secs(30);

/// The answer to one send, carried back to the frame.
struct Reply {
    pid: u32,
    /// The tab's own path, as `slot_of_path` compares it.
    path: PathBuf,
    /// The document's edit epoch when it was sent.
    epoch: u64,
    result: Result<(), String>,
}

/// The window-to-window state.
pub struct Siblings {
    started: bool,
    dir: Option<PathBuf>,
    inbox: Option<Receiver<Incoming>>,
    peers: Vec<Peer>,
    peers_read: Option<Instant>,
    /// What the discovery file last said, as written.
    published: Option<(String, Option<[i32; 4]>)>,
    replies: (Sender<Reply>, Receiver<Reply>),
    drop_replies: (Sender<objdrop::DropReply>, Receiver<objdrop::DropReply>),
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
            drop_replies: channel(),
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
        parcel::prune(&dir);
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

    /// Publish this window's open documents and client area when either
    /// changed.
    fn publish(&mut self, documents: String, rect: Option<[i32; 4]>) {
        let Some(dir) = &self.dir else {
            return;
        };
        let now = (documents, rect);
        if self.published.as_ref() == Some(&now) {
            return;
        }
        if wire::publish(dir, &wire::own_pipe(), &now.0, now.1).is_ok() {
            if let Some([l, t, r, b]) = now.1 {
                crate::diag::trace(|| {
                    // ui-text-exempt: diagnostic trace, never displayed
                    format!(
                        "window-rect pid={} rect={l},{t},{r},{b}",
                        std::process::id()
                    )
                });
            }
            self.published = Some(now);
        }
    }

    /// Send `parcel` to the window `pid` listens on without blocking the
    /// frame, first waiting up to `wait` for that window to start listening;
    /// the answer arrives through [`Self::replies`]. A refused parcel's carry
    /// file is deleted here, since no window will read it.
    fn send(&self, ctx: &egui::Context, pid: u32, parcel: Parcel, wait: Duration) {
        let Parcel {
            path,
            arrival,
            epoch,
        } = parcel;
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed
            format!(
                "window-move-sending to={pid} path={path:?} page={} carried={} key={}",
                arrival.page.map_or(-1, |p| p as i64),
                arrival.carry.as_ref().map_or(0, |c| c.edits),
                arrival.key.is_some()
            )
        });
        let replies = self.replies.0.clone();
        let wake = ctx.clone();
        let pipe = wire::pipe_of(pid);
        std::thread::spawn(move || {
            let started = Instant::now();
            while !native_pipe::exists(&pipe) && started.elapsed() < wait {
                std::thread::sleep(Duration::from_millis(100));
            }
            let result = wire::send_open(&pipe, &arrival);
            if result.is_err() {
                parcel::discard(&arrival);
            }
            let _ = replies.send(Reply {
                pid,
                path,
                epoch,
                result,
            });
            wake.request_repaint();
        });
    }
}

/// Whether this window was placed by the test harness, which also means it
/// must never take the focus.
fn harness_placed() -> bool {
    std::env::var_os(VIEWPORT_ENV).is_some()
}

/// Start another copy of this program on `path`, showing the zero-based
/// `page`, or with no document when `path` is `None`.
///
/// Every `PDFCER_DIAG_*` variable but the placement is removed first, so a
/// window the test harness placed off the desktop starts its child there too
/// and nothing that drove the parent drives the child. The child's output goes
/// nowhere: inherited, its trace would interleave with the parent's, and a
/// harness reading the parent's would act on the child's regions.
fn spawn_window(path: Option<&Path>, page: Option<usize>) -> std::io::Result<u32> {
    use std::process::Stdio;
    let exe = std::env::current_exe()?;
    let mut command = std::process::Command::new(exe);
    if let Some(path) = path {
        command.arg(super::remote::absolute(path));
        if let Some(page) = page {
            command.arg(crate::PAGE_ARG).arg((page + 1).to_string());
        }
    }
    command
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
    /// Show the zero-based `page` of the active document, clamped to its
    /// length; used right after an open.
    pub(crate) fn show_page(&mut self, page: usize) {
        if let Status::Open(doc) = &mut self.status {
            let count = doc.pages.len();
            doc.view.go_to_page(page, count);
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
        if !self.movable(slot) {
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
    pub(super) fn siblings_poll(&mut self, ctx: &egui::Context, actions: &mut Vec<Action>) {
        self.siblings.ensure_started(ctx);
        let received: Vec<Incoming> = self
            .siblings
            .inbox
            .as_ref()
            .map(|rx| rx.try_iter().collect())
            .unwrap_or_default();
        for Incoming { request, answer } in received {
            let arrival = match request {
                Request::Open(arrival) => arrival,
                Request::Paste { x, y } => {
                    self.paste_requested(ctx, x, y, actions);
                    continue;
                }
            };
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed
                format!(
                    "window-move-received pid={} path={:?} page={} carried={} key={}",
                    std::process::id(),
                    arrival.path,
                    arrival.page.map_or(-1, |p| p as i64),
                    arrival.carry.as_ref().map_or(0, |c| c.edits),
                    arrival.key.is_some()
                )
            });
            let result = self.receive(&arrival);
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed
                format!(
                    "window-move-opened pid={} ok={} unsaved={} why={:?}",
                    std::process::id(),
                    result.is_ok(),
                    matches!(&self.status, Status::Open(d) if crate::app::save::has_unsaved_edits(d)),
                    result.as_ref().err().map_or("", String::as_str)
                )
            });
            if let Some(answer) = answer {
                let _ = answer.send(result);
            }
            if !harness_placed() {
                ctx.send_viewport_cmd(egui::ViewportCommand::Focus);
            }
        }
        let replies: Vec<Reply> = self.siblings.replies.1.try_iter().collect();
        for reply in replies {
            self.settle_send(ctx, reply);
        }
        self.siblings.refresh_peers();
        objdrop::share_peers(ctx, &self.siblings.peers);
        self.selection_drop_poll(ctx, actions);
        let documents = self.document_names();
        self.siblings.publish(documents, drag::client_rect_px(ctx));
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
        // window has the document without it, and closing here would discard
        // the edit.
        if let Some(slot) = self.slot_of_path(&reply.path)
            && matches!(self.slot(slot), Some(Status::Open(d)) if d.edit_epoch == reply.epoch)
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

    /// Pack the document in `slot`, or record why it cannot move.
    fn pack(&self, slot: usize) -> Option<Parcel> {
        self.parcel(slot, self.siblings.dir.as_deref())
            .map_err(crate::app::status::decline::record_window_move)
            .ok()
    }

    /// Send the document in `slot` to `peer`.
    pub(super) fn send_to(&self, ctx: &egui::Context, peer: &Peer, slot: usize) {
        if let Some(parcel) = self.pack(slot) {
            self.siblings.send(ctx, peer.pid, parcel, Duration::ZERO);
        }
    }

    /// `view.move_to_new_window`: open the document in `slot` in a window of
    /// its own and close it here.
    ///
    /// A document that is only a path and a page starts the new window on its
    /// file and closes here at once. One carrying edits or a password starts
    /// an empty window and is sent to it once it listens, closing here when
    /// that window has it open.
    pub(super) fn move_to_new_window(&mut self, ctx: &egui::Context, slot: usize) {
        let Some(parcel) = self.pack(slot) else {
            return;
        };
        let carried = parcel.needs_pipe();
        let spawned = if carried {
            spawn_window(None, None)
        } else {
            spawn_window(Some(&parcel.path), parcel.arrival.page)
        };
        match spawned {
            Ok(pid) => {
                crate::diag::trace(|| {
                    // ui-text-exempt: diagnostic trace, never displayed
                    format!(
                        "window-torn-off pid={pid} path={:?} page={} carried={carried}",
                        parcel.path,
                        parcel.arrival.page.map_or(-1, |p| p as i64)
                    )
                });
                if carried {
                    self.siblings.send(ctx, pid, parcel, STARTUP_WAIT);
                } else {
                    self.close_slot(slot);
                }
            }
            Err(e) => {
                parcel::discard(&parcel.arrival);
                crate::app::status::decline::record_window_move(WindowMoveRefusal::NotStarted(
                    e.to_string(),
                ));
            }
        }
    }

    /// `view.move_to_window`: send the document in `slot` to the one other
    /// window, or ask which when there are several.
    pub(super) fn move_to_window(&mut self, ctx: &egui::Context, slot: usize) {
        if let Some(why) = self.refusal(slot) {
            crate::app::status::decline::record_window_move(why);
            return;
        }
        match self.siblings.peers.as_slice() {
            [] => crate::app::status::decline::record_window_move(WindowMoveRefusal::NotSent(
                t::no_other_window().to_owned(),
            )),
            [only] => self.send_to(ctx, &only.clone(), slot),
            _ => {
                if let Some(Status::Open(doc)) = self.slot(slot) {
                    self.siblings.picking = Some(doc.path.clone());
                }
            }
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
        if let (Some(peer), Some(slot)) = (choice, self.slot_of_path(&path)) {
            self.send_to(ctx, &peer, slot);
        }
    }
}
