//! # `text::settings` — settings copy that names an app type
//!
//! Re-exports [`pdfcer_gui_base::text::settings`] and adds the modules that
//! reach into `crate::app`.

pub use pdfcer_gui_base::text::settings::*;

#[cfg(test)]
mod app_tests;
