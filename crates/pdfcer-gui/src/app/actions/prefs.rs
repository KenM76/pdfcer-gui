//! **The verbs whose subject is a PREFERENCE, not a document.**
//!
//! Design and rationale: `docs/modules/pdfcer-gui/app/actions/prefs.md`.

use crate::app::prefs::Prefs;

pub use pdfcer_gui_base::subactions::PrefAction;

/// Write this preference through to `prefs` and save the file.
pub fn apply(action: PrefAction, prefs: &mut Prefs) {
    match action {
        PrefAction::FindZoom(on) => {
            prefs.find_zoom_on_jump = on;
            let _ = prefs.save();
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed in the UI
                format!("find-zoom-persisted on={on}")
            });
        }
        PrefAction::PagePreviews { on, budget_ms } => {
            prefs.page_previews = on;
            prefs.page_preview_budget_ms = budget_ms;
            let _ = prefs.save();
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed in the UI
                //
                // `budget_ms` PLAIN, and `0` reaching the trace as `0`
                // rather than as a word: a harness reads this field, and a
                // Debug-formatted or prettified value in a field a machine
                // parses has already produced one driven check in this repo
                // that reported the opposite of the truth.
                format!("page-previews-persisted on={on} budget_ms={budget_ms}")
            });
        }
        PrefAction::WalkEyeHeight(mm) => {
            prefs.walk_eye_mm = mm;
            let _ = prefs.save();
            let mm = mm.map_or_else(|| "auto".to_owned(), |mm| mm.to_string()); // ui-text-exempt: diagnostic trace, never displayed
            crate::diag::trace(|| format!("walk-eye-persisted mm={mm}")); // ui-text-exempt: diagnostic trace, never displayed
        }
        PrefAction::ModelView(settings) => {
            let keys = settings.len();
            prefs.model_view = settings;
            let _ = prefs.save();
            crate::diag::trace(|| format!("model-view-settings-persisted keys={keys}")); // ui-text-exempt: diagnostic trace, never displayed
        }
        PrefAction::ModelViewMerge(settings) => {
            let keys = settings.len();
            prefs.model_view.extend(settings);
            let _ = prefs.save();
            crate::diag::trace(|| format!("model-view-settings-merged keys={keys}")); // ui-text-exempt: diagnostic trace, never displayed
        }
    }
}
