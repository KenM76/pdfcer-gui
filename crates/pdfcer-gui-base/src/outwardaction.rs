//! # `outwardaction` — the acts of a link that leaves the document
//!
//! Raised by the canvas and the Security Warning; applied by
//! `app::actions::outward`. Design: `DESIGNS.md` § O320.

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
}
