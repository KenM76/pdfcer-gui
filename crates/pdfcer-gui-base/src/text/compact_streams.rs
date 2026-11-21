//! # `text::compact_streams` — how Save a compacted copy compresses streams
//!
//! Every word of the Brotli switch.

/// The switch that compresses with Brotli in place of Flate.
#[must_use]
pub const fn brotli_toggle() -> &'static str {
    "Compress with Brotli instead of Flate (smaller, but few readers draw it)"
}

/// Hover on it.
#[must_use]
pub const fn brotli_tooltip() -> &'static str {
    "Brotli packs page content and other parts tighter than the usual Flate. The copy then \
     needs PDF 2.0 and a reader that supports Brotli; the window says what that means for the \
     readers you use once the copy is built. Pictures keep their own storage. In a copy made \
     compatible with an older version, the parts Brotli would have packed are left as they \
     are, and the window says so."
}

/// Hover on it while recompressing is off.
#[must_use]
pub const fn brotli_needs_recompress() -> &'static str {
    "Turn on recompressing first: Brotli is a way of recompressing."
}
