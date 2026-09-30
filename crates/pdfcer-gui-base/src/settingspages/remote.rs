//! The Remote control page: whether another program may drive the window.
//! The link itself is the `command-remote` crate.

use egui::Ui;

use super::widgets;
use crate::remotecontrol::RemoteControl;
use crate::text::settings as t;

/// Ask, always allow, or never allow.
pub fn remote_control(ui: &mut Ui, prefs: &mut crate::prefs::Prefs) {
    widgets::header(
        ui,
        t::remote_control_title(),
        t::remote_control_silence(),
        t::remote_control_radius(),
    );
    for option in RemoteControl::ALL {
        widgets::option(
            ui,
            &mut prefs.remote_control,
            *option,
            t::remote_control_label(*option),
            Some(t::remote_control_note(*option)),
        );
    }
}
