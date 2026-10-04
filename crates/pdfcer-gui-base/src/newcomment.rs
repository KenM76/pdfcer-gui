//! # `newcomment` — the comment kinds placed through their own engine verb
//!
//! Each variant authors one annotation the `TextAnnotSpec` route cannot
//! express, because the engine gives it a dedicated `EditSession` verb.

/// A comment to author, raised by the note dialog when it accepts.
#[derive(Debug, Clone, PartialEq)]
pub enum NewComment {
    /// **Attach a file to a page as a marker** (`/FileAttachment`,
    /// §12.5.6.15). The file was picked in `BeginTextAnnot`'s apply arm, so
    /// this carries its path; the bytes are read at apply time.
    Attachment {
        /// The 0-based page.
        page: usize,
        /// The marker's rectangle, in PDF user space.
        rect: pdfcer_core::page_tree::Rect,
        /// The file to embed.
        file: std::path::PathBuf,
        /// The marker's icon (`/Name`).
        icon: pdfcer_core::annot_author::AttachmentIcon,
        /// The description (`/Contents` and the filespec's `/Desc`), trimmed;
        /// `None` when the operator typed none.
        description: Option<String>,
    },
    /// **Attach a recording to a page** (`/Sound`, §12.5.6.16). The file
    /// was picked in `BeginTextAnnot`'s apply arm; it is read and converted
    /// at apply time.
    Sound {
        /// The 0-based page.
        page: usize,
        /// The icon's rectangle, in PDF user space.
        rect: pdfcer_core::page_tree::Rect,
        /// The WAV file.
        file: std::path::PathBuf,
        /// The icon (`/Name`).
        icon: pdfcer_core::annot_author::SoundIcon,
        /// The description (`/Contents`), trimmed; `None` when none was typed.
        description: Option<String>,
        /// How the WAV is converted to a PDF sound.
        import: pdfcer_core::sound::WavImportOptions,
    },
    /// **Play a media clip from a page region** (`/Screen`, §12.5.6.18).
    /// The file was picked in `BeginTextAnnot`'s apply arm; its bytes are
    /// read at apply time.
    Screen {
        /// The 0-based page.
        page: usize,
        /// The play region, in PDF user space.
        rect: pdfcer_core::page_tree::Rect,
        /// The clip.
        file: std::path::PathBuf,
        /// The clip's MIME type (`/CT`), as the window last showed it.
        content_type: String,
        /// What starts playback.
        trigger: pdfcer_core::annot_author::ScreenTrigger,
        /// The clip's temporary-file permission (`/TF`).
        temp_access: pdfcer_core::annot_author::MediaTempAccess,
        /// The description (`/Contents`), trimmed; `None` when none was typed.
        description: Option<String>,
    },
    /// **Mark where words are to be inserted** (`/Caret`, §12.5.6.11).
    Caret {
        /// The 0-based page.
        page: usize,
        /// Where the operator clicked, in PDF user space; the caret's apex
        /// sits there.
        at: (f64, f64),
        /// The words to insert, trimmed; `None` when only a paragraph break
        /// is asked for.
        text: Option<String>,
        /// Whether a paragraph mark accompanies the caret (`/Sy /P`).
        paragraph: bool,
    },
    /// **Propose replacement words for selected text**: a `/StrikeOut` over
    /// the selection grouped under a `/Caret` carrying the words.
    ReplaceText {
        /// The 0-based page the selection is on.
        page: usize,
        /// The caret's apex, in PDF user space: the end of the selection's
        /// last line, a caret's height above that line's bottom.
        at: (f64, f64),
        /// The replacement words, trimmed and non-empty.
        text: String,
        /// The selected lines' boxes, in content order.
        struck: Vec<pdfcer_core::annot_author::Quad>,
    },
}
