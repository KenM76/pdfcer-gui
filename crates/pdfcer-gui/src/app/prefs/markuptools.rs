//! # `app::prefs::markuptools` — keeping the markup tools' choices across launches
//!
//! The highlighter's shape and Delete in region's Inside only / Touching live
//! in `egui` memory, where Tool Options reads and writes them. On the first
//! frame [`keep`] seeds memory from the preferences file; on every later frame
//! it compares memory with the file's copy and saves on a difference, so no
//! writer of either choice has to remember to. Saving waits for every pointer
//! button to be up, as `formstyle` does.
//!
//! Design: `docs/modules/pdfcer-gui/app/prefs/markuptools.md`.

use crate::app::prefs::Prefs;
use crate::canvas::markup::highlight;
use crate::canvas::region;

/// Set once memory holds the file's choices.
const SEEDED_KEY: &str = "pdfcer-markup-tools-seeded"; // ui-text-exempt: internal memory id, never displayed

/// Seed memory from `prefs` once, then save `prefs` when memory moves.
pub fn keep(ctx: &egui::Context, prefs: &mut Prefs) {
    let id = egui::Id::new(SEEDED_KEY);
    if !ctx.data(|d| d.get_temp::<bool>(id).unwrap_or(false)) {
        let chosen = prefs.markup_tools;
        highlight::seed(ctx, chosen.highlight_shape);
        region::set_touching(ctx, chosen.region_touching);
        ctx.data_mut(|d| d.insert_temp(id, true));
        crate::diag::trace(|| {
            format!(
                // ui-text-exempt: diagnostic trace, never displayed in the UI
                "markup-tools-seeded shape={} touching={}",
                chosen.highlight_shape.token(),
                chosen.region_touching
            )
        });
        return;
    }
    let live = pdfcer_gui_base::prefs::MarkupToolPrefs {
        highlight_shape: highlight::shape(ctx),
        region_touching: region::touching(ctx),
    };
    if live == prefs.markup_tools || ctx.input(|i| i.pointer.any_down()) {
        return;
    }
    prefs.markup_tools = live;
    let saved = prefs.save().is_ok();
    crate::diag::trace(|| {
        format!(
            // ui-text-exempt: diagnostic trace, never displayed in the UI
            "markup-tools-saved ok={saved} shape={} touching={}",
            live.highlight_shape.token(),
            live.region_touching
        )
    });
}
