//! The operator's side, drawn by the application where it chooses: the
//! question banner, the status-line item and the log window.
//!
//! None of these draws itself into a document view. The banner is meant for a
//! fixed-height panel under the toolbar ([`BANNER_HEIGHT`]; a content-sized
//! panel would resize the view under it as the text changes).

use std::time::Instant;

use crate::Remote;
use crate::consent::{ASK_TIMEOUT, Scope};

/// The banner's fixed height, in points.
pub const BANNER_HEIGHT: f32 = 30.0;

/// Draw the question, if one is waiting. Returns whether one was drawn, so
/// the application can skip the panel when it returns `false` next frame.
/// The application calls [`Remote::decide`] for nothing: the buttons do.
pub fn banner(ui: &mut egui::Ui, remote: &mut Remote, now: Instant) -> bool {
    let Some(knock) = remote.knock() else {
        return false;
    };
    let left = ASK_TIMEOUT
        .saturating_sub(now.saturating_duration_since(knock.raised))
        .as_secs();
    let purpose = if knock.purpose.is_empty() {
        String::new()
    } else {
        format!(": \u{201c}{}\u{201d}", knock.purpose)
    };
    let text = format!(
        "\u{201c}{}\u{201d} asks to control this window{purpose}",
        knock.client
    );
    let mut choice: Option<Option<Scope>> = None;
    ui.horizontal_centered(|ui| {
        ui.strong(text).on_hover_text(
            "A program on this computer, running as you, wants to run commands in this window. \
             The name is the one it gave; it cannot be verified.",
        );
        if ui
            .button("Allow once")
            .on_hover_text("This connection only")
            .clicked()
        {
            choice = Some(Some(Scope::Once));
        }
        if ui
            .button("Allow for this session")
            .on_hover_text("Any program, until this window closes")
            .clicked()
        {
            choice = Some(Some(Scope::Session));
        }
        if ui
            .button("Always allow")
            .on_hover_text("Any program, from now on, without asking. Change it in Settings.")
            .clicked()
        {
            choice = Some(Some(Scope::Always));
        }
        if ui.button("Refuse").clicked() {
            choice = Some(None);
        }
        ui.weak(format!("refused in {left} s"));
    });
    if let Some(choice) = choice {
        remote.decide(choice, now);
    }
    // The countdown must tick while idle.
    ui.ctx()
        .request_repaint_after(std::time::Duration::from_secs(1));
    true
}

/// What the status-line item asks of the application.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatusClick {
    /// Nothing.
    None,
    /// Show the log window.
    OpenLog,
}

/// Draw the status-line item while a client is allowed: its name, the last
/// command, and a Disconnect button. Draws nothing otherwise.
pub fn status_item(ui: &mut egui::Ui, remote: &mut Remote) -> StatusClick {
    let Some(client) = remote.connected_client().map(str::to_owned) else {
        return StatusClick::None;
    };
    let last = remote
        .log()
        .iter()
        .rev()
        .find(|e| e.client == client && e.asked != "hello")
        .map(|e| e.asked.clone())
        .unwrap_or_else(|| "no commands yet".to_owned());
    let mut click = StatusClick::None;
    if ui
        .link(format!("Remote: {client}"))
        .on_hover_text(format!("Last: {last}\nClick to see every command it ran."))
        .clicked()
    {
        click = StatusClick::OpenLog;
    }
    if ui
        .small_button("Disconnect")
        .on_hover_text("End this connection now")
        .clicked()
    {
        remote.disconnect();
    }
    click
}

/// The log window: every hello and command, newest last.
pub fn log_window(ctx: &egui::Context, remote: &Remote, open: &mut bool) {
    egui::Window::new("Remote control log")
        .open(open)
        .default_width(520.0)
        .show(ctx, |ui| {
            ui.label(format!("Listening on {}", remote.pipe()));
            if let Some(why) = remote.failure() {
                ui.colored_label(ui.visuals().error_fg_color, format!("Not listening: {why}"));
            }
            ui.separator();
            egui::ScrollArea::vertical()
                .stick_to_bottom(true)
                .show(ui, |ui| {
                    if remote.log().is_empty() {
                        ui.weak("Nothing yet.");
                    }
                    for e in remote.log() {
                        let outcome = e.outcome.as_deref().unwrap_or("(running)");
                        ui.horizontal_wrapped(|ui| {
                            ui.monospace(&e.client);
                            ui.monospace(&e.asked);
                            ui.weak(outcome);
                        });
                    }
                });
        });
}
