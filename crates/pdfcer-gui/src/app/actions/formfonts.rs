//! Edit ▸ Forms ▸ Repair fonts: move every inline font in `/AcroForm /DR /Font`
//! into an object of its own, so Acrobat draws filled text fields.
//!
//! Acrobat shows a filled text field blank when its `/DA` font resolves to an
//! inline `/DR` entry; `EditSession::promote_inline_dr_fonts` repairs a form
//! saved that way. The count goes to the status row; zero is a worded decline,
//! so a press on a sound form says so and changes nothing.

use crate::app::state::OpenDoc;
use crate::app::status::decline;
use crate::app::unlock::Refusal;

/// Why the repair wrote nothing.
enum RepairError {
    Edit(pdfcer_core::edit::EditError),
    NothingInline,
}

impl std::fmt::Display for RepairError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Edit(e) => write!(f, "{e}"),
            Self::NothingInline => f.write_str("no inline /DR font"),
        }
    }
}

impl Refusal for RepairError {
    fn encrypted(&self) -> bool {
        matches!(self, Self::Edit(e) if e.encrypted())
    }
}

/// Run the repair through the edit funnel.
pub(super) fn repair(doc: &mut OpenDoc) {
    super::apply::vector_edit(doc, "form-repair-fonts", 0, 0, |session| {
        match session.promote_inline_dr_fonts() {
            Ok(0) => {
                decline::record_form_fonts_nothing_to_repair();
                Err(RepairError::NothingInline)
            }
            Ok(n) => {
                crate::diag::trace(|| {
                    // ui-text-exempt: diagnostic trace, never displayed in the UI
                    format!("form-repair-fonts-applied fonts={n}")
                });
                Ok(vec![crate::text::formfonts::repaired(n)])
            }
            Err(e) => Err(RepairError::Edit(e)),
        }
    });
}
