//! The copy for the Remote control settings page: whether another program on
//! this computer may drive the window.

use crate::remotecontrol::RemoteControl;

/// The page's heading.
#[must_use]
pub const fn group_remote() -> &'static str {
    "Remote control"
}

/// The setting's name.
#[must_use]
pub const fn remote_control_title() -> &'static str {
    "Programs that ask to control this window"
}

/// What happens if you never touch it.
#[must_use]
pub const fn remote_control_silence() -> &'static str {
    "pdfcer asks you, in a bar under the ribbon, each time a program such as an AI assistant \
     wants to run commands in the window. Nothing runs until you answer."
}

/// What it covers, and what it does not.
#[must_use]
pub const fn remote_control_radius() -> &'static str {
    "Only programs running as you, on this computer, can reach the window at all. A program \
     that is allowed runs the same commands you can, with the same undo; the status line shows \
     which program is connected and has a Disconnect button."
}

/// One option's name.
#[must_use]
pub const fn remote_control_label(value: RemoteControl) -> &'static str {
    match value {
        RemoteControl::Ask => "Ask each time (pdfcer's default)",
        RemoteControl::Always => "Always allow",
        RemoteControl::Never => "Never allow",
    }
}

/// One option's description.
#[must_use]
pub const fn remote_control_note(value: RemoteControl) -> &'static str {
    match value {
        RemoteControl::Ask => {
            "You choose per request: once, for this session, or always. Unanswered for two \
             minutes counts as a refusal."
        }
        RemoteControl::Always => {
            "Any program running as you connects without a question. Every command it runs is \
             still listed in the log, reached from the status line."
        }
        RemoteControl::Never => "Every request is refused without a question.",
    }
}
