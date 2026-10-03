//! The words for pasting what another program copied.

use crate::clippaste::dib::DibError;
use std::borrow::Cow;

/// Why a paste of another program's copy placed nothing.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum OsPasteRefusal {
    /// A picture that could not be decoded; the field is the reason.
    Unreadable(String),
    /// A picture that could not be made into a stamp; the field is the reason.
    Unplaceable(String),
    /// No picture, where only a picture will do.
    NoPicture,
    /// Neither a picture nor text, where either would become pages.
    NothingForPages,
    /// A picture that could not be made into a page; the field is the reason.
    NotAPage(String),
    /// Text the engine would not set as pages; the field is its sentence.
    PagesRefused(String),
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
            Self::Unplaceable(why) => Cow::Owned(format!(
                "pdfcer could not make a stamp of the clipboard picture: {why}. Nothing was pasted."
            )),
            Self::NoPicture => Cow::Borrowed(
                "The clipboard holds no picture. Copy one in another program, then paste it as a stamp.",
            ),
            Self::NothingForPages => Cow::Borrowed(
                "The clipboard holds no picture or text to make pages from. Copy one in another program, then try again.",
            ),
            Self::NotAPage(why) => Cow::Owned(format!(
                "pdfcer could not make a page of the clipboard picture: {why}. Nothing was added."
            )),
            Self::PagesRefused(why) => Cow::Owned(why.clone()),
            Self::Nothing => Cow::Borrowed(
                "The clipboard holds nothing pdfcer can paste onto a page. Copy a picture, some text, or something in pdfcer, and paste again.",
            ),
        }
    }
}

/// The name a pasted picture's stamp carries.
#[must_use]
pub const fn pasted_picture() -> &'static str {
    "Pasted picture"
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
