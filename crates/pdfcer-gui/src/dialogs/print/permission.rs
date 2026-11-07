//! What the password that opened the document allows a print: everything, low
//! quality only, or nothing (ISO 32000 Table 22). Printing renders pages here, so
//! low quality is honoured by capping the resolution.

use pdfcer_core::document::Document;
use pdfcer_core::edit::{PrintReadyError, check_print_permission};

use crate::text::print as t;

/// The most a low-quality print renders at, as `pdfcer print` does.
const LOW_QUALITY_DPI: u32 = 150;

/// The document's answer to printing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(super) enum Permission {
    /// Print at any resolution.
    #[default]
    Full,
    /// Print at no more than [`LOW_QUALITY_DPI`].
    LowQuality,
    /// Do not print.
    Forbidden,
}

impl Permission {
    /// The answer for `doc`.
    pub(super) fn of(doc: &Document) -> Self {
        match check_print_permission(doc) {
            Ok(()) => Self::Full,
            Err(PrintReadyError::LowQualityPrintingOnly) => Self::LowQuality,
            Err(_) => Self::Forbidden,
        }
    }

    /// The resolution ceiling when the operator asked for `wanted`.
    pub(super) fn ceiling(self, wanted: u32) -> u32 {
        match self {
            Self::LowQuality => wanted.min(LOW_QUALITY_DPI),
            Self::Full | Self::Forbidden => wanted,
        }
    }

    /// What the window says about it, when it limits anything.
    pub(super) fn sentence(self) -> Option<String> {
        match self {
            Self::Full => None,
            Self::LowQuality => Some(t::low_quality_only(LOW_QUALITY_DPI)),
            Self::Forbidden => Some(t::printing_forbidden().to_owned()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_a_low_quality_print_lowers_the_ceiling() {
        assert_eq!(Permission::Full.ceiling(600), 600);
        assert_eq!(Permission::LowQuality.ceiling(600), LOW_QUALITY_DPI);
        assert_eq!(Permission::LowQuality.ceiling(100), 100);
    }

    #[test]
    fn a_limit_is_said_and_no_limit_is_not() {
        assert_eq!(Permission::Full.sentence(), None);
        assert!(Permission::LowQuality.sentence().is_some());
        assert!(Permission::Forbidden.sentence().is_some());
    }
}
