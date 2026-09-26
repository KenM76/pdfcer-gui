//! # ui-verify — verification that drives the running application
//!
//! ## The standing rule this crate exists to serve
//!
//! `GUI_ROADMAP.md` § "A standing rule this investigation earned":
//!
//! Design and rationale: `docs/modules/ui-verify/lib.md`.

pub mod capture;
pub mod checks;
pub mod coords;
pub mod error;
pub mod fixture;
pub mod geom;
pub mod image;
pub mod input;
pub mod launch;
pub mod pixels;
pub mod png;
pub mod profile;
pub mod report;
pub mod sandbox;
pub mod sys;
pub mod trace;
