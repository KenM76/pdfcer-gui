//! # `outwardaction` — the acts of a link that leaves the document
//!
//! Raised by the canvas and the two prompts; applied by
//! `app::actions::outward`. Design: `DESIGNS.md` § O320.

use std::path::PathBuf;

/// A link reaching outside the document, or the operator's answer about one.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OutwardAction {
    /// A `/URI` link was clicked; `uri` is the address exactly as written.
    WebLink {
        /// The address.
        uri: String,
    },
    /// The Security Warning was answered.
    WebAnswer {
        /// The address asked about.
        uri: String,
        /// Allow (open it) or Block.
        allow: bool,
        /// Keep the answer for the site.
        remember: bool,
    },
    /// A `/GoToR` link was clicked: open `file` and arrive at `target`.
    FileLink {
        /// The file, already resolved against the document's folder.
        file: PathBuf,
        /// Where in it.
        target: RemoteGoTo,
    },
    /// A `/Launch` link was clicked.
    Launch {
        /// The file, already resolved against the document's folder.
        file: PathBuf,
    },
    /// The Launch File prompt was answered.
    LaunchAnswer {
        /// The file asked about.
        file: PathBuf,
        /// Open or Do Not Open.
        open: bool,
        /// *Do not show this message again*; honoured with Open only.
        dont_ask: bool,
    },
}

/// Where a `/GoToR` arrives in the file it opens.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RemoteGoTo {
    /// A 0-based page index in that file.
    Page(usize),
    /// A named destination, resolved in that file's own names.
    Named(Vec<u8>),
    /// No usable `/D`: the file opens at its first page.
    Start,
}
