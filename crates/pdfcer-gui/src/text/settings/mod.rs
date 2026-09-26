//! # `text::settings` — settings copy that names an app type
//!
//! Re-exports [`pdfcer_gui_base::text::settings`] and adds the modules that
//! reach into `crate::app`.

pub use pdfcer_gui_base::text::settings::*;

use egui_shell::theme::Preset;

pub mod redaction;
pub mod shell;
pub use redaction::*;
pub use shell::*;

#[cfg(test)]
mod app_tests;
