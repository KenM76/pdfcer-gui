//! **What the Print window remembers between jobs** — `OPERATOR_REQUESTS.md`
//! **O166**, the operator's words of 2026-09-10: *"the printer dialogue box
//! needs to remember our last settings."*
//!
//! Design and rationale: `docs/modules/pdfcer-gui/dialogs/print/remembered.md`.

use super::PrintDialog;

impl PrintDialog {
    /// **This dialog's state, reduced to what a different document would still
    /// want** — the producing half of `OPERATOR_REQUESTS.md` **O166**.
    pub(super) fn habits(&self) -> crate::app::prefs::PrintPrefs {
        crate::app::prefs::PrintPrefs {
            printer: self.printers.get(self.selected).map(|p| p.name.clone()),
            orientation: self.device.orientation,
            duplex: self.device.duplex,
            pick_tray_by_page_size: self.device.pick_tray_by_page_size,
            // A hand-picked `Form(id)` is stored as "from the printer's own
            // settings" — `paper_key` does that reduction and argues it. It is
            // deliberate loss, not a gap.
            paper: self.device.paper,
            scale: self.scale,
            custom_percent: self.custom_percent,
            scope: self.scope,
            max_dpi: self.max_dpi,
            copies: self.copies,
            uncollated: self.uncollated,
            subset: self.subset,
            reverse: self.reverse,
            poster: self.poster,
            lines: self.lines,
        }
    }

    /// Write [`Self::habits`] into the preferences file.
    pub(super) fn remember(&self, prefs: &mut crate::app::prefs::Prefs) -> bool {
        self.store(prefs, self.habits(), "habits").stored
    }

    /// Put the preferences back to [`super::PrintDialog::opened_with`] — the
    /// undoing half of `OPERATOR_REQUESTS.md` **O185**.
    pub(super) fn restore(&self, prefs: &mut crate::app::prefs::Prefs) -> Written {
        self.store(prefs, self.opened_with.clone(), "restored")
    }

    /// **The only writer of `prefs.print`, and the only emitter of the
    /// `print-remembered` trace.**
    fn store(
        &self,
        prefs: &mut crate::app::prefs::Prefs,
        wanted: crate::app::prefs::PrintPrefs,
        how: &'static str,
    ) -> Written {
        if prefs.print == wanted {
            return Written {
                changed: false,
                stored: true,
            };
        }
        prefs.print = wanted;
        let saved = prefs.save();
        crate::diag::trace(|| {
            format!(
                // ui-text-exempt: diagnostic trace, never displayed in the UI
                "print-remembered how={} saved={} orientation={:?} duplex={:?} paper={} \
                 scale={} copies={} subset={:?}",
                how,
                saved.is_ok(),
                prefs.print.orientation,
                prefs.print.duplex,
                // Stable lowercase tokens, never `{:?}`, for the two fields a
                // driven check reads back. This project's standing lesson:
                // never `Debug`-format a field a machine reads — a `{:?}` on a
                // payload-carrying variant prints the payload too, and a check
                // grepping `scale=custom` would miss `scale=Custom(2.5)` while
                // quoting the truth in its own failure message.
                crate::app::prefs::printing::paper_key(prefs.print.paper),
                crate::app::prefs::printing::scale_key(prefs.print.scale),
                prefs.print.copies,
                prefs.print.subset,
            )
        });
        Written {
            changed: true,
            stored: saved.is_ok(),
        }
    }
}

/// What a call to [`PrintDialog::store`] actually did.
pub(super) struct Written {
    /// The preferences did not already hold the wanted value, so it was
    /// written over and the `print-remembered` trace was emitted.
    pub(super) changed: bool,
    /// The preferences now hold the wanted value. False only when a write was
    /// attempted and [`crate::app::prefs::Prefs::save`] refused — a read-only
    /// or missing `userdata` folder.
    pub(super) stored: bool,
}
