//! # `opendoc::drawnvalues` — form values drawn for the screen only
//!
//! A widget with no `/AP` asks the viewer to draw its value (§12.7.3.3), and
//! Acrobat does. This canvas does too, through a **display copy**: the
//! session's current bytes reopened, with `EditSession::regenerate_appearances`
//! applied, handed to the page renderer for the pages that hold such a widget.
//!
//! Contract:
//! - Nothing here writes the open document. The copy is never saved, never on
//!   the undo stack, and is rebuilt when [`OpenDoc::edit_epoch`] or the session
//!   changes. *Redraw values* remains the way to write the values into the file.
//! - Pages without an undrawn widget render from the real session.
//! - A copy that cannot be built (signed form, a refused fill guard) is `None`:
//!   the page renders as the file states, and the trace says why.
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/opendoc/drawnvalues.md`.

use std::cell::{Cell, RefCell};
use std::collections::BTreeSet;
use std::sync::Arc;
use std::time::Instant;

use pdfcer_core::document::Document;
use pdfcer_core::edit::EditSession;
use pdfcer_core::forms::{AcroForm, FieldType, FieldValue};
use pdfcer_core::object::ObjId;

use crate::opendoc::OpenDoc;
use crate::settings::SettingsExt;

/// The session the renderer draws undrawn form values from.
pub struct DisplayCopy {
    /// The current revision with every undrawn value drawn.
    pub session: Arc<EditSession>,
    /// 0-based pages holding a widget with no `/AP`.
    pub pages: BTreeSet<usize>,
}

/// The per-document cache of [`DisplayCopy`], keyed on the revision.
#[derive(Default)]
pub struct DrawnValues {
    /// `(edit_epoch, session pointer)` the slot below describes.
    built_for: Cell<Option<(u64, usize)>>,
    copy: RefCell<Option<Arc<DisplayCopy>>>,
}

impl OpenDoc {
    /// The session to rasterize `page_index` from: the display copy when the
    /// page holds a widget with no `/AP`, otherwise the real session.
    #[must_use]
    pub fn display_session(&self, page_index: usize) -> Arc<EditSession> {
        match self.display_copy() {
            Some(copy) if copy.pages.contains(&page_index) => Arc::clone(&copy.session),
            _ => Arc::clone(&self.session),
        }
    }

    /// The display copy for this revision, building it on first use.
    #[must_use]
    pub fn display_copy(&self) -> Option<Arc<DisplayCopy>> {
        let key = (self.edit_epoch, Arc::as_ptr(&self.session) as usize);
        if self.drawn_values.built_for.get() != Some(key) {
            self.drawn_values.built_for.set(Some(key));
            *self.drawn_values.copy.borrow_mut() = self.build_display_copy().map(Arc::new);
        }
        self.drawn_values.copy.borrow().clone()
    }

    fn build_display_copy(&self) -> Option<DisplayCopy> {
        let form = pdfcer_core::forms::parse_acroform(&self.session.view())?;
        let undrawn = undrawn_widgets(&form);
        if undrawn.is_empty() {
            return None;
        }
        let pages: BTreeSet<usize> = (0..self.pages.len())
            .filter(|&p| {
                self.session
                    .widget_rects(p)
                    .iter()
                    .any(|(id, _)| undrawn.contains(id))
            })
            .collect();
        let started = Instant::now();
        let built = self.drawn_copy(&form);
        let ms = started.elapsed().as_millis();
        crate::diag::trace(|| match &built {
            Ok(_) => format!(
                // ui-text-exempt: diagnostic trace, never displayed in the UI
                "drawn-values widgets={} pages={} ms={ms} outcome=ok",
                undrawn.len(),
                pages.len()
            ),
            Err(why) => format!(
                // ui-text-exempt: diagnostic trace, never displayed in the UI
                "drawn-values widgets={} pages={} ms={ms} outcome=failed reason={why:?}",
                undrawn.len(),
                pages.len()
            ),
        });
        built.ok().map(|session| DisplayCopy {
            session: Arc::new(session),
            pages,
        })
    }

    /// The current bytes reopened with every undrawn value drawn.
    fn drawn_copy(&self, form: &AcroForm) -> Result<EditSession, String> {
        let options = self.settings.save_options();
        let (bytes, _) = self
            .session
            .to_incremental_bytes(&options)
            .map_err(|e| e.to_string())?;
        let document = Document::from_bytes(bytes).map_err(|e| e.to_string())?;
        let mut copy = self.settings.open_session(document);
        // An empty text field has no `/V`, so `regenerate_appearances` skips
        // it; an empty fill draws its border and background.
        for field in empty_undrawn_text_fields(form) {
            copy.fill_text_field(&field, "")
                .map_err(|e| e.to_string())?;
        }
        copy.regenerate_appearances().map_err(|e| e.to_string())?;
        Ok(copy)
    }
}

/// Every widget with no usable normal appearance.
fn undrawn_widgets(form: &AcroForm) -> BTreeSet<ObjId> {
    form.fields
        .iter()
        .flat_map(|f| f.widgets.iter())
        .filter(|w| !w.has_normal_appearance)
        .map(|w| w.id)
        .collect()
}

/// Text fields holding no value with a widget that draws nothing.
fn empty_undrawn_text_fields(form: &AcroForm) -> Vec<String> {
    form.fields
        .iter()
        .filter(|f| f.field_type == Some(FieldType::Text) && !f.is_rich_text())
        .filter(|f| matches!(f.value, FieldValue::Absent))
        .filter(|f| f.widgets.iter().any(|w| !w.has_normal_appearance))
        .map(|f| f.fully_qualified_name.clone())
        .collect()
}
