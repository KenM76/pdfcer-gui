//! A selection dragged off this window's canvas and released over another
//! pdfcer-gui window is copied there: this window copies it to the clipboard
//! and asks the other to paste at the release point. Shift held at the
//! release moves it instead, by cutting it here once the other window has
//! taken it. The convention is the file manager's across containers: a drag
//! copies, Shift moves.
//!
//! The canvas decides on the release frame ([`selection_released`]) and the
//! frame acts on it afterwards ([`PdfcerApp::selection_drop_poll`]), because
//! the canvas holds the document borrowed and the copy and the send need the
//! window. The peers' rectangles reach the canvas through the context, written
//! each frame by `siblings_poll`.

use eframe::egui;

use super::PdfcerApp;
use super::wire::Peer;
use crate::app::actions::Action;
use crate::app::state::Status;
use crate::text::siblings::WindowMoveRefusal;

/// The other windows, as `siblings_poll` last read them.
#[derive(Clone, Default)]
struct PeerRects(Vec<Peer>);

/// A selection released over another window, waiting for the frame.
#[derive(Clone, Copy)]
struct Dropped {
    pid: u32,
    desktop: egui::Pos2,
    shift: bool,
}

/// The answer to one paste request, carried back to the frame.
pub(super) struct DropReply {
    pid: u32,
    result: Result<(), String>,
    /// The document and edit epoch to cut from when the drop was a move.
    then_cut: Option<(std::path::PathBuf, u64)>,
}

fn peers_id() -> egui::Id {
    egui::Id::new("pdfcer.siblings.peer-rects") // ui-text-exempt: a memory key
}

fn dropped_id() -> egui::Id {
    egui::Id::new("pdfcer.siblings.selection-drop") // ui-text-exempt: a memory key
}

/// Hand the other windows to the canvas for this frame.
pub(super) fn share_peers(ctx: &egui::Context, peers: &[Peer]) {
    ctx.data_mut(|d| d.insert_temp(peers_id(), PeerRects(peers.to_vec())));
}

/// A move drag of the selection has been released. When the pointer is
/// outside this window and over another, the drop is recorded for the frame
/// and `true` says the move must not be committed here.
pub(crate) fn selection_released(ctx: &egui::Context, shift: bool) -> bool {
    let Some(at) = ctx.input(|i| i.pointer.interact_pos()) else {
        return false;
    };
    if ctx.content_rect().contains(at) {
        return false;
    }
    let Some(desktop) = super::drag::desktop_px(ctx, at) else {
        return false;
    };
    let peers = ctx.data(|d| d.get_temp::<PeerRects>(peers_id()).unwrap_or_default());
    let Some(peer) = peers.0.iter().find(|w| w.contains(desktop.x, desktop.y)) else {
        return false;
    };
    let pid = peer.pid;
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed
        format!(
            "window-selection-dropped onto={pid} desktop={:.0},{:.0} shift={shift}",
            desktop.x, desktop.y
        )
    });
    ctx.data_mut(|d| {
        d.insert_temp(
            dropped_id(),
            Dropped {
                pid,
                desktop,
                shift,
            },
        );
    });
    ctx.request_repaint();
    true
}

impl PdfcerApp {
    /// Copy a selection dropped on another window and ask that window to
    /// paste it, then settle the answers to earlier requests.
    pub(super) fn selection_drop_poll(&mut self, ctx: &egui::Context, actions: &mut Vec<Action>) {
        if let Some(drop) = ctx.data_mut(|d| {
            let drop = d.get_temp::<Dropped>(dropped_id());
            d.remove::<Dropped>(dropped_id());
            drop
        }) {
            self.send_selection(ctx, drop);
        }
        let replies: Vec<DropReply> = self.siblings.drop_replies.1.try_iter().collect();
        for reply in replies {
            self.settle_drop(ctx, reply, actions);
        }
    }

    fn send_selection(&mut self, ctx: &egui::Context, drop: Dropped) {
        let Some(peer) = self
            .siblings
            .peers
            .iter()
            .find(|w| w.pid == drop.pid)
            .cloned()
        else {
            return;
        };
        let Status::Open(doc) = &self.status else {
            return;
        };
        let copied = crate::canvas::clipboard::copy(ctx, doc);
        if !crate::app::dispatch::clipboard::report_copy(doc, copied) {
            return;
        }
        let then_cut = drop.shift.then(|| (doc.path.clone(), doc.edit_epoch));
        let replies = self.siblings.drop_replies.0.clone();
        let wake = ctx.clone();
        let (pid, pipe) = (peer.pid, peer.pipe);
        let egui::Pos2 { x, y } = drop.desktop;
        std::thread::spawn(move || {
            let result = super::wire::send_paste(&pipe, x, y);
            let _ = replies.send(DropReply {
                pid,
                result,
                then_cut,
            });
            wake.request_repaint();
        });
    }

    /// Say why the other window did not paste, or finish a move by cutting
    /// here what it pasted.
    fn settle_drop(&mut self, ctx: &egui::Context, reply: DropReply, actions: &mut Vec<Action>) {
        if let Err(why) = reply.result {
            // ui-text-exempt: diagnostic trace, never displayed
            crate::diag::trace(|| format!("window-paste-refused to={} why={why:?}", reply.pid));
            crate::app::status::decline::record_window_move(WindowMoveRefusal::SelectionNotSent(
                why,
            ));
            return;
        }
        // ui-text-exempt: diagnostic trace, never displayed
        crate::diag::trace(|| format!("window-paste-sent to={}", reply.pid));
        // A move cuts only what was copied: the same document, unedited since.
        let Some((path, epoch)) = reply.then_cut else {
            return;
        };
        let unchanged = matches!(&self.status, Status::Open(doc)
            if doc.path == path && doc.edit_epoch == epoch);
        if unchanged {
            self.dispatch_command(ctx, "edit.cut", actions);
        }
    }

    /// Paste the clipboard at a desktop point another window named.
    pub(super) fn paste_requested(
        &mut self,
        ctx: &egui::Context,
        x: f32,
        y: f32,
        actions: &mut Vec<Action>,
    ) {
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed
            format!(
                "window-paste-received pid={} desktop={x:.0},{y:.0}",
                std::process::id()
            )
        });
        let at = ctx.input(|i| i.viewport().inner_rect).map(|inner| {
            (egui::pos2(x, y).to_vec2() / ctx.pixels_per_point() - inner.min.to_vec2()).to_pos2()
        });
        crate::app::dispatch::clipboard::paste_at(self, ctx, at, actions);
    }
}
