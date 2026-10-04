//! # `text::screenannot` — the words for placing a media clip on a page
//!
//! Also the MIME type a clip's file extension suggests: ISO 32000 §13.2.4
//! names no required types or codecs, so the suggestion is editable in the
//! window and named in the status note.

use pdfcer_core::annot_author::{MediaTempAccess, ScreenTrigger};

/// Extension → MIME type, for the clip types readers are known to play.
const TYPES: &[(&str, &str)] = &[
    ("mp4", "video/mp4"),
    ("m4v", "video/mp4"),
    ("mov", "video/quicktime"),
    ("webm", "video/webm"),
    ("avi", "video/x-msvideo"),
    ("wmv", "video/x-ms-wmv"),
    ("mpg", "video/mpeg"),
    ("mpeg", "video/mpeg"),
    ("flv", "video/x-flv"),
    ("mp3", "audio/mpeg"),
    ("m4a", "audio/mp4"),
    ("aac", "audio/aac"),
    ("wav", "audio/wav"),
    ("ogg", "audio/ogg"),
];

/// The type a clip gets when its extension is not in the table.
pub const FALLBACK_TYPE: &str = "application/octet-stream";

/// The MIME type `file_name`'s extension suggests, case-insensitively, or
/// `None` when the table has no entry for it.
#[must_use]
pub fn media_type(file_name: &str) -> Option<&'static str> {
    let (_, ext) = file_name.rsplit_once('.')?;
    let ext = ext.to_ascii_lowercase();
    TYPES.iter().find(|(e, _)| *e == ext).map(|(_, t)| *t)
}

/// The line naming the chosen clip.
#[must_use]
pub fn file_line(name: &str, bytes: u64) -> String {
    format!(
        "Clip: {name} ({})",
        super::panels::byte_size(usize::try_from(bytes).unwrap_or(usize::MAX))
    )
}

/// The label over the MIME-type field.
#[must_use]
pub const fn type_label() -> &'static str {
    "Media type"
}

/// Under the MIME-type field, when the extension suggested it.
#[must_use]
pub const fn type_suggested() -> &'static str {
    "Suggested from the file's extension. The PDF standard names no required \
     formats, so whether it plays depends on the reader."
}

/// Under the MIME-type field, when the extension suggested nothing.
#[must_use]
pub const fn type_unknown() -> &'static str {
    "pdfcer does not recognise this file's extension. Type its media type \
     (for example video/mp4), or readers will not know how to play it."
}

/// The label over the trigger choice.
#[must_use]
pub const fn trigger_heading() -> &'static str {
    "Starts playing"
}

/// One trigger, as an operator reads it.
#[must_use]
pub const fn trigger_label(trigger: ScreenTrigger) -> &'static str {
    match trigger {
        ScreenTrigger::Click => "When the region is clicked",
        ScreenTrigger::PageOpen => "When the page opens",
    }
}

/// The label of the temporary-file choice.
#[must_use]
pub const fn temp_label() -> &'static str {
    "Readers may copy the clip to a temporary file"
}

/// One temporary-file permission, as an operator reads it.
#[must_use]
pub const fn temp_access_label(access: MediaTempAccess) -> &'static str {
    match access {
        MediaTempAccess::Never => "Never",
        MediaTempAccess::Access => "If the document allows copying",
        MediaTempAccess::Always => "Always",
    }
}

/// The temporary-file choice's hover: why it is a choice at all.
#[must_use]
pub const fn temp_hover() -> &'static str {
    "Some media players can only play a clip from a file on disk. The PDF \
     standard's default is Never, which stops those players; it does not say \
     which players they are, so pdfcer starts at \"If the document allows \
     copying\"."
}

/// The disclosure after a clip is placed: the type and the two choices
/// that decide whether and when it plays.
#[must_use]
pub fn placed(
    name: &str,
    page: usize,
    content_type: &str,
    trigger: ScreenTrigger,
    access: MediaTempAccess,
) -> String {
    format!(
        "{name} is now stored in this PDF, on page {}, as {content_type}. It plays {}; \
         temporary copies: {}. Whether it plays at all depends on the reader.",
        page + 1,
        match trigger {
            ScreenTrigger::Click => "when the region is clicked",
            ScreenTrigger::PageOpen => "when the page opens",
        },
        temp_access_label(access).to_lowercase()
    )
}

/// The refusal when the picked file cannot be read.
#[must_use]
pub fn unreadable(detail: &str) -> String {
    format!("pdfcer could not read that file, so no clip was placed: {detail}")
}

/// The picker's title.
#[must_use]
pub const fn pick_title() -> &'static str {
    "Choose a video or audio clip to place"
}

/// The picker's filter name.
#[must_use]
pub const fn filter_media() -> &'static str {
    "Video and audio clips"
}

/// The extensions the picker's media filter lists.
#[must_use]
pub fn media_extensions() -> Vec<&'static str> {
    TYPES.iter().map(|(e, _)| *e).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_extension_suggests_its_type_whatever_its_case() {
        assert_eq!(media_type("clip.MP4"), Some("video/mp4"));
        assert_eq!(media_type("a.b.mov"), Some("video/quicktime"));
    }

    #[test]
    fn an_unknown_or_missing_extension_suggests_nothing() {
        assert_eq!(media_type("clip.xyz"), None);
        assert_eq!(media_type("clip"), None);
    }
}
