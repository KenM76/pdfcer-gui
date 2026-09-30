//! The live link's application half. Transport, consent and the operator's
//! banner are the `command-remote` crate; this module decides which verbs a
//! connected program may send and feeds them into the frame at the same choke
//! points a keystroke or ribbon click reaches, so a remote edit is an ordinary
//! edit with ordinary undo.
//!
//! Frame order: [`PdfcerApp::remote_poll`] after the chord and scripted-invoke
//! dispatch; [`PdfcerApp::remote_banner`] under the ribbon;
//! [`PdfcerApp::remote_settle`] after `apply_actions`, which answers the
//! requests whose reply describes the state their actions produced.

mod verbs;

use std::path::PathBuf;
use std::time::Instant;

use command_remote::{Config, Policy, Remote, Request};
use eframe::egui;

use super::PdfcerApp;
use super::actions::Action;
use crate::app::prefs::RemoteControl;

/// The name in the pipe and in the discovery directory.
pub const APP: &str = "pdfcer";

/// The link's state inside the application.
#[derive(Default)]
pub struct Link {
    remote: Option<Remote>,
    started: bool,
    log_open: bool,
    document: Option<Option<PathBuf>>,
    /// Requests answered after this frame's actions apply.
    deferred: Vec<(Request, String)>,
}

impl std::fmt::Debug for Link {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Link")
            .field("remote", &self.remote)
            .field("deferred", &self.deferred.len())
            .finish_non_exhaustive()
    }
}

const fn policy_of(value: RemoteControl) -> Policy {
    match value {
        RemoteControl::Ask => Policy::Ask,
        RemoteControl::Always => Policy::Always,
        RemoteControl::Never => Policy::Never,
    }
}

impl Link {
    /// Start listening on first use. Unit tests never listen: many harness
    /// apps share one process id, and so one pipe name.
    fn ensure_started(&mut self, ctx: &egui::Context, policy: Policy) {
        if self.started || cfg!(test) {
            return;
        }
        self.started = true;
        let ctx = ctx.clone();
        let config = Config {
            app: APP.to_owned(),
            discovery_dir: command_remote::default_discovery_dir(APP),
            policy,
        };
        match Remote::start(config, move || ctx.request_repaint()) {
            Ok(remote) => self.remote = Some(remote),
            Err(e) => crate::diag::trace(|| format!("remote-start-failed {e}")),
        }
    }

    /// The status-line item: the connected program and a Disconnect button.
    pub fn status_item(&mut self, ui: &mut egui::Ui) {
        if let Some(remote) = self.remote.as_mut()
            && command_remote::ui::status_item(ui, remote)
                == command_remote::ui::StatusClick::OpenLog
        {
            self.log_open = true;
        }
    }
}

impl PdfcerApp {
    /// Take the requests that passed consent and act on them.
    pub(super) fn remote_poll(&mut self, ctx: &egui::Context, actions: &mut Vec<Action>) {
        let policy = policy_of(self.prefs.remote_control);
        self.remote.ensure_started(ctx, policy);
        let document = self.active_path().map(absolute);
        let Some(remote) = self.remote.remote.as_mut() else {
            return;
        };
        if remote.policy() != policy {
            remote.set_policy(policy);
        }
        if self.remote.document.as_ref() != Some(&document) {
            remote.set_document(document.as_deref());
            self.remote.document = Some(document);
        }
        let requests = remote.poll(Instant::now());
        for request in requests {
            verbs::handle(self, ctx, request, actions);
        }
    }

    /// The operator's question, in a fixed-height bar, while one is waiting.
    pub(super) fn remote_banner(&mut self, ui: &mut egui::Ui) {
        let Some(remote) = self.remote.remote.as_mut() else {
            return;
        };
        if remote.knock().is_none() {
            return;
        }
        egui::Panel::top("remote-banner")
            .exact_size(command_remote::ui::BANNER_HEIGHT)
            .show(ui, |ui| {
                command_remote::ui::banner(ui, remote, Instant::now())
            });
        // "Always allow" is a preference, so it is saved like one.
        if remote.policy() == Policy::Always && self.prefs.remote_control != RemoteControl::Always {
            self.prefs.remote_control = RemoteControl::Always;
            let _ = self.prefs.save();
        }
    }

    /// Answer the deferred requests, and draw the log window if open.
    pub(super) fn remote_settle(&mut self, ctx: &egui::Context) {
        for (request, message) in std::mem::take(&mut self.remote.deferred) {
            let reply = verbs::state_reply(self, &message);
            if let Some(remote) = self.remote.remote.as_mut() {
                remote.respond(request, reply);
            }
        }
        if self.remote.log_open
            && let Some(remote) = self.remote.remote.as_ref()
        {
            command_remote::ui::log_window(ctx, remote, &mut self.remote.log_open);
        }
    }
}

/// `path` made absolute, so a client in another directory can use it.
pub(super) fn absolute(path: &std::path::Path) -> PathBuf {
    std::path::absolute(path).unwrap_or_else(|_| path.to_path_buf())
}
