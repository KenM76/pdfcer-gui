//! # `dialogs::settings::display` — how pdfcer draws, as distinct from what it draws
//!
//! The eighth group, and the only one whose settings are **not** about the PDF
//! standard. Every other group in this window exists because a clause declines
//! to have an opinion; these two exist because a machine has a speed.
//!
//! ## ★ Two settings, out of seven commissioned
//!
//! `RIBBON_IA.md` §5.2 specified a View ▸ Render group of five, plus two
//! behaviour settings on the same tab, and `shell::manifest::DIRECTED` carried
//! all seven as *"named individually, with their value sets and their defaults,
//! when this shell was commissioned"* — which is a stronger statement of intent
//! than a status mark, and is why they were emitted despite carrying no `G`.
//!
//! They were registered, drawn, and inert. Checked against the engine on
//! 2026-08-17, **five of the seven have nothing behind them**:
//!
//! - **Render strategy** — there is no tiled-progressive path in this shell.
//!   `pdfcer_render::render_page_region` exists, so it is buildable, but that is
//!   a rendering architecture and not a setting.
//! - **Thin lines** and **Antialiasing** — `RenderOptions` had neither field.
//!   `interpret.rs` sets `anti_alias: true` as a literal at two call sites.
//!   ⚠ **The thin-lines half expired on 2026-09-05** and is corrected here
//!   rather than left standing: the operator asked for that control back by
//!   name (`OPERATOR_REQUESTS.md` **O137**), the engine shipped
//!   `RenderOptions::stroke_display` the same day (`Pass 254.0`), and it is now
//!   `view.line_weights` on **View ▸ Display** — deliberately not a setting in
//!   this window. It is a reading aid he flips several times while reading one
//!   sheet, which is an activity, and it is per **document** so two open
//!   drawings can disagree. `crate::text::commands::view_line_weights` carries
//!   the whole argument and says where a persisted default would go if he asks
//!   for one. Antialiasing is still real and still unasked-for.
//! - **Floating panels** — `egui-shell`'s dock has no floating mode.
//! - **App initiative** — *nothing in this build opens a surface unasked*. The
//!   specified default is **Never**, and it is already true by construction, so
//!   the control would exist to switch off a behaviour pdfcer does not have.
//!
//! `DIRECTED`'s own doc comment anticipated this outcome and named the remedy:
//! *"if it turns out to be wrong, the fix is deleting eight rows from one list
//! rather than re-deriving which entries were deliberate."* Six rows went; the
//! two that survived became these controls and left the ribbon, because a
//! setting belongs in the settings window and `RIBBON_IA.md` §6's own list of
//! what does not go on the ribbon now has a real destination to point at.
//!
//! `crate::app::prefs`' header carries the full table with the evidence for
//! each verdict.
//!
//! ## Why these two are not in the engine's settings file
//!
//! They are **preferences**, not answers to a silent standard, and this
//! window's own opening paragraph promises the latter. They live in
//! `userdata/preferences.txt` beside `settings.txt` — same roof, same
//! fail-soft parser, different file — for the reason `crate::app::prefs`
//! states. The group sits in this window because a *window* is where an
//! operator looks for a choice, and which file a choice is stored in is not
//! their concern.

use egui::Ui;

use super::widgets;
use crate::app::prefs::{
    MAX_SETTLE_MS, MIN_SETTLE_MS, OpeningFit, PageCache, PasteChords, Prefs, RenderQuality,
    WheelPaging,
};
use crate::text::settings as t;

/// How sharply a page is rasterised.
///
/// # Why this is a real setting on the drawings this shell is for
///
/// The benchmark sheet is 5.6 MB of dense vector site plan, and rasterising it
/// is the expensive thing this program does. The multiplier is the only control
/// an operator has over that cost, and both directions are wanted by real
/// people: someone panning a big sheet looking for a detail wants `Faster`, and
/// someone reading small text over a hairline grid wants `Sharper`.
///
/// The radius line says it affects speed as well as appearance, because that is
/// the trade being made and a control that mentioned only sharpness would be
/// describing half of itself.
pub fn render_quality(ui: &mut Ui, prefs: &mut Prefs) {
    widgets::header(
        ui,
        t::quality_title(),
        t::quality_silence(),
        t::quality_radius(),
    );
    for option in RenderQuality::ALL {
        widgets::option(
            ui,
            &mut prefs.render_quality,
            *option,
            t::quality_label(*option),
            Some(t::quality_note(*option)),
        );
    }
}

/// ★★ How much memory the page cache may hold.
///
/// # Why this is in *Drawing the page* and not in a group of its own
///
/// This window files by the **symptom that brings an operator looking**
/// (`super`'s header), and the symptom here is *"scrolling back to a sheet
/// makes me wait"* — which is a fact about how a frame is drawn, exactly like
/// the two controls above it. A "Memory" group would file it by what it spends
/// rather than by what it does, and nobody arrives with the symptom *"pdfcer is
/// using the wrong amount of RAM"*.
///
/// # ★ Third rather than first in the group, and it is the newest
///
/// Quality and settle are read on every frame by an operator who is *looking at
/// the page*; this one is read when they are annoyed by a wait they have
/// already had. That is the same ordering argument the group makes about the
/// two opening-view controls below: settings that affect what you are looking
/// at now, then settings that affect what happens next.
pub fn page_cache(ui: &mut Ui, prefs: &mut Prefs) {
    widgets::header(
        ui,
        t::page_cache_title(),
        t::page_cache_silence(),
        t::page_cache_radius(),
    );
    for option in PageCache::ALL {
        widgets::option(
            ui,
            &mut prefs.page_cache,
            *option,
            // ★ Owned, unlike every other label in this window, because the
            // megabyte figure is COMPUTED from the budget rather than written
            // beside it. `widgets::option` takes `&str`, and a `String` that
            // lives to the end of the call is the smallest thing that works —
            // the alternative is four `const` labels carrying four hand-copied
            // numbers, which is the drift this derivation exists to prevent.
            &t::page_cache_label(*option),
            Some(t::page_cache_note(*option)),
        );
    }
}

/// How long a zoom must stop changing before the page is redrawn sharply.
///
/// # A slider, and linear
///
/// Linear because the useful resolution is even: 50 ms against 150 ms matters
/// about as much as 500 against 600, since both answer *how long am I willing
/// to look at a soft page*. That is the same argument the parallel-tolerance
/// slider makes and the opposite of the word-gap one, whose useful range is all
/// at the low end.
///
/// The range is the store's own `MIN_SETTLE_MS..=MAX_SETTLE_MS`, not a local
/// pair of literals — the third instance of that rule in this window, and it
/// exists for the same reason each time: a control narrower than what the file
/// accepts silently rewrites a hand-edited value on open, and the operator
/// never touched the control.
pub fn zoom_settle(ui: &mut Ui, prefs: &mut Prefs) {
    widgets::header(
        ui,
        t::settle_title(),
        t::settle_silence(),
        t::settle_radius(),
    );
    ui.add(
        egui::Slider::new(&mut prefs.zoom_settle_ms, MIN_SETTLE_MS..=MAX_SETTLE_MS)
            .suffix(t::settle_suffix())
            .text(t::settle_slider_label()),
    );
    ui.label(egui::RichText::new(t::settle_note()).small().weak());
}

/// How the first page of a newly opened document is sized to the window.
///
/// # Why this is offered at all, when a fit command already exists
///
/// View ▸ Zoom has *Fit page* and *Fit width*, so an operator can already get
/// any of these three in one click. What they cannot do is get it **without**
/// the click, on every document, forever — and `viewer::remembered` persists
/// the page-display arrangement and nothing else, deliberately. This is the
/// difference between a command and a preference, and it is the whole content
/// of the operator's 2026-08-17 report: the capability was there and the
/// *default* was not settable.
///
/// # A radio group, not a dropdown
///
/// Three values, each needing a sentence about what it costs on a large sheet.
/// A dropdown shows one at a time and hides the comparison, which is the only
/// thing that makes the choice decidable.
pub fn opening_fit(ui: &mut Ui, prefs: &mut Prefs) {
    widgets::header(
        ui,
        t::opening_fit_title(),
        t::opening_fit_silence(),
        t::opening_fit_radius(),
    );
    for option in OpeningFit::ALL {
        widgets::option(
            ui,
            &mut prefs.opening_fit,
            *option,
            t::opening_fit_label(*option),
            Some(t::opening_fit_note(*option)),
        );
    }
}

/// What a plain mouse wheel does under a one-page-at-a-time display mode.
///
/// # ★ Here as well as on the status bar, and that is not duplication
///
/// The status-bar control is the one the operator will use: it sits beside the
/// page buttons, which is where they are already looking when they are
/// thinking about pages. This one is how they *find* the choice — and how they
/// read the two sentences that make it decidable, which a one-word toggle in a
/// 24-point bar has no room for. Both write the same field, so neither can
/// **Which chord pastes a form field as a new one, and which as a duplicate.**
///
/// `OPERATOR_REQUESTS.md` **O58**. Ken, 2026-08-29: *"let's make it an option to
/// have it swap to match Acrobat or work the way we have it now."*
///
/// # ★ Here rather than on a keyboard-shortcuts page, and there is no keyboard page
///
/// The shortcuts dialog *lists* bindings; it does not edit them. And this is not
/// really a question about keys — it is a question about **which of two pastes
/// is the ordinary one**, which is why the labels name the behaviour and the
/// chords are the parenthetical rather than the other way round.
///
/// # It is in Display because that is where input-gesture preferences already live
///
/// Beside `wheel_paging`, which is the same shape of question: an operator
/// deciding what a familiar input should mean in this program. Neither is about
/// what the document *is*, which is what keeps them out of Saving and Pages.
pub fn paste_chords(ui: &mut Ui, prefs: &mut Prefs) {
    widgets::header(
        ui,
        t::paste_chords_title(),
        t::paste_chords_silence(),
        t::paste_chords_radius(),
    );
    for option in PasteChords::ALL {
        widgets::option(
            ui,
            &mut prefs.paste_chords,
            *option,
            t::paste_chords_label(*option),
            Some(t::paste_chords_note(*option)),
        );
    }
}

/// drift from the other.
pub fn wheel_paging(ui: &mut Ui, prefs: &mut Prefs) {
    widgets::header(
        ui,
        t::wheel_paging_title(),
        t::wheel_paging_silence(),
        t::wheel_paging_radius(),
    );
    for option in WheelPaging::ALL {
        widgets::option(
            ui,
            &mut prefs.wheel_paging,
            *option,
            t::wheel_paging_label(*option),
            Some(t::wheel_paging_note(*option)),
        );
    }
}

/// Which of the three View ▸ Display overlays are already on when a document
/// opens.
///
/// # ★ One setting with three switches, not three settings
///
/// [`widgets::toggle`]'s own documentation carries the control-shape argument.
/// The reason they are **one setting** is different and is about the operator
/// rather than the widget: the three interlock. A guide is dragged out of a
/// ruler gutter, so `guides` without `rulers` is a switch that appears to do
/// nothing. That relationship needs saying once, in a place all three readers
/// will be looking — which is what a single header and a shared disclosure buy.
///
/// # The disclosure is not a note under the guides switch
///
/// It is true whichever way that switch is set — a document with remembered
/// guides opens with them showing either way — so it belongs to the setting
/// rather than to one of its parts. Putting it under the switch would make it
/// read as an argument for turning guides on, which it is not; it is a fact
/// about what pdfcer will do regardless. Same distinction the replacement-text
/// bound makes, and `widgets::disclosure` exists for exactly this.
/// **Shade the fillable fields** — `OPERATOR_REQUESTS.md` O96.
///
/// ★ In *Display* rather than in a Forms group, which is where he asked for it
/// (*"in our display section"*) and is also right: it changes nothing about the
/// form and everything about what the page looks like. An operator turning it
/// off is tidying their view, not altering how fields behave.
///
/// [`crate::app::prefs::Prefs::shade_form_fields`] carries why a wash over part
/// of a page is an affordance rather than the content marking rule 4 forbids.
pub fn field_shade(ui: &mut Ui, prefs: &mut Prefs) {
    widgets::header(
        ui,
        t::field_shade_title(),
        t::field_shade_silence(),
        t::field_shade_radius(),
    );
    widgets::toggle(
        ui,
        &mut prefs.shade_form_fields,
        t::field_shade_label(),
        Some(t::field_shade_note()),
    );
}

/// **The two auto-hide settings** — 2026-09-05.
///
/// ★★ One header over both, because they are one decision the operator makes
/// twice: *"how much of the window do I want the drawing to have?"* Two
/// separate sections would ask it twice and would put the sentence that makes
/// the feature safe — the drawing does not move — under only one of them.
///
/// ★ In *Display* beside the page chrome, and not in *Appearance*: Appearance
/// is about how pdfcer LOOKS (its preset, its colours), and this is about how
/// much of the window the drawing gets. `field_shade` was filed here by the
/// same test and the operator put it here himself.
///
/// The Settings window is also where the **state** of these two lives, which
/// is why they are checkboxes here and plain commands on View ▸ Window: see
/// `shell::commands::catalog::view`'s note on why neither ribbon control
/// renders pressed, and on what Office does.
pub fn auto_hide(ui: &mut Ui, prefs: &mut Prefs) {
    widgets::header(
        ui,
        t::auto_hide_title(),
        t::auto_hide_silence(),
        t::auto_hide_radius(),
    );
    widgets::toggle(
        ui,
        &mut prefs.ribbon_auto_hide,
        t::auto_hide_ribbon_label(),
        Some(t::auto_hide_ribbon_note()),
    );
    widgets::toggle(
        ui,
        &mut prefs.rail_auto_hide,
        t::auto_hide_rail_label(),
        Some(t::auto_hide_rail_note()),
    );
}

pub fn page_chrome(ui: &mut Ui, prefs: &mut Prefs) {
    widgets::header(
        ui,
        t::chrome_title(),
        t::chrome_silence(),
        t::chrome_radius(),
    );
    widgets::toggle(
        ui,
        &mut prefs.chrome.rulers,
        t::chrome_rulers_label(),
        Some(t::chrome_rulers_note()),
    );
    widgets::toggle(
        ui,
        &mut prefs.chrome.grid,
        t::chrome_grid_label(),
        Some(t::chrome_grid_note()),
    );
    widgets::toggle(
        ui,
        &mut prefs.chrome.guides,
        t::chrome_guides_label(),
        Some(t::chrome_guides_note()),
    );
    widgets::disclosure(ui, t::chrome_guides_bound());
}

#[cfg(test)]
mod tests {
    use super::*;

    /// ★ The shipped quality is the identity multiplier.
    ///
    /// The "a build that omits nothing behaves as it did before" rule, at the
    /// one place it can be checked cheaply. `viewer::raster_scale` was
    /// `zoom × pixels_per_point` exactly before this setting existed, so
    /// `Normal` must multiply by one or every raster in the application
    /// silently changed size the day the control landed.
    #[test]
    fn the_shipped_quality_changes_no_raster() {
        assert!((RenderQuality::default().multiplier() - 1.0).abs() < f32::EPSILON);
    }

    /// The three qualities are ordered less-to-more and are distinct.
    ///
    /// The control reads left to right as a scale, so a list whose middle
    /// entry was not between its neighbours would be a scale that does not
    /// scale.
    #[test]
    fn the_qualities_ascend() {
        let m: Vec<f32> = RenderQuality::ALL.iter().map(|q| q.multiplier()).collect();
        assert_eq!(m.len(), 3);
        assert!(m[0] < m[1] && m[1] < m[2], "{m:?}");
    }

    /// The shipped settle is reachable on its own slider.
    ///
    /// A default outside its control's bounds would be silently rewritten the
    /// first time anybody opened this window, on every machine, without a
    /// click. Third instance of this check in the window; third setting with a
    /// range that must be the store's.
    #[test]
    fn the_shipped_settle_is_reachable_on_the_slider() {
        let ms = Prefs::default().zoom_settle_ms;
        assert!(
            (MIN_SETTLE_MS..=MAX_SETTLE_MS).contains(&ms),
            "the shipped settle {ms} is outside the slider's range"
        );
    }
}
