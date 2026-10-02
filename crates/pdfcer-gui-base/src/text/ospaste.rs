//! The words for pasting what another program copied.

use crate::clippaste::dib::DibError;
use std::borrow::Cow;

/// Why a paste of another program's copy placed nothing.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum OsPasteRefusal {
    /// A picture that could not be decoded; the field is the reason.
    Unreadable(String),
    /// Nothing a page can take.
    Nothing,
}

impl OsPasteRefusal {
    /// The sentence for this refusal.
    #[must_use]
    pub fn line(&self) -> Cow<'static, str> {
        match self {
            Self::Unreadable(why) => Cow::Owned(format!(
                "The clipboard holds a picture pdfcer could not read: {why}. Nothing was pasted."
            )),
            Self::Nothing => Cow::Borrowed(
                "The clipboard holds nothing pdfcer can paste onto a page. Copy a picture, some text, or something in pdfcer, and paste again.",
            ),
        }
    }
}

/// Why a clipboard bitmap could not be decoded.
#[must_use]
pub fn dib_error(e: DibError) -> &'static str {
    match e {
        DibError::Truncated => "the bitmap is shorter than its header says",
        DibError::Unsupported => "it is a kind of bitmap pdfcer does not read",
        DibError::Size => "the bitmap has no area or is impossibly large",
    }
}

/// A bitmap with no pixels.
#[must_use]
pub fn empty_bitmap() -> &'static str {
    "the bitmap has no area"
}

/// A bitmap too large to hold in memory.
#[must_use]
pub fn bitmap_too_large() -> &'static str {
    "the bitmap is too large to hold"
}
