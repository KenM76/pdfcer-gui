//! # `text::settings::shell` — the settings that change pdfcer's own window
//!
//! The fourth row of the blast-radius taxonomy in [`super::look`]'s header.
//! Every setting here stops at what the operator sees or does: none of them
//! writes a byte, none of them changes a page, and each one's `_radius` line
//! says so, because an operator changing how Tab behaves or how big the
//! buttons are drawn has every reason to wonder whether they are also changing
//! the document.
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/text/settings/mod.md`.

use egui_shell::theme::Preset;

// ===========================================================================
// Appearance — theme
// ===========================================================================

/// Theme: what it is.
#[must_use]
pub const fn theme_title() -> &'static str {
    "Theme"
}

/// Theme: what the standard leaves open.
#[must_use]
pub const fn theme_silence() -> &'static str {
    "Changes the window only. It never alters a document, and nothing here is \
     written into a PDF you save."
}

/// Theme: what changing it costs.
#[must_use]
pub const fn theme_radius() -> &'static str {
    "Applies as soon as you pick it, so you can see it. Cancel puts it back."
}

/// One preset's name.
#[must_use]
pub fn theme_preset_label(preset: Preset) -> &'static str {
    match preset {
        Preset::Quiet => "Quiet",
        Preset::Airy => "Airy",
        Preset::Dark => "Dark",
        Preset::System => "Match the system",
        // ui-text-exempt: not a literal — the shell's own key for a preset this
        // catalog predates. See the doc comment.
        other => other.key(),
    }
}

/// One preset's description.
#[must_use]
pub const fn theme_preset_note(preset: Preset) -> &'static str {
    match preset {
        Preset::Quiet => {
            "Muted greys, one accent, tight spacing. The page dominates and the \
             window recedes."
        }
        Preset::Airy => {
            "Lighter and roomier, with softer edges and clearer grouping. Easier \
             to scan; uses more screen."
        }
        Preset::Dark => {
            "A dark window against a light page, as CAD tools do it. Strong page \
             edge, easier on a long session."
        }
        Preset::System => {
            "Light or dark as the system is set, in its accent colour. A \
             colour that would be hard to read is darkened or lightened."
        }
        _ => "",
    }
}

/// The settings file names a theme this build does not have.
#[must_use]
pub fn theme_unknown(token: &str) -> String {
    format!(
        "This settings file asks for a theme named \"{token}\", which this version \
         does not have. Using Quiet for now. The name is kept, not overwritten."
    )
}

// ===========================================================================
// Appearance — how big pdfcer's own controls are drawn
// ===========================================================================
//
// The theme's twin: the second of the two settings in this window that change
// the PROGRAM's appearance and nothing about the document, and the second of
// the two that take effect before Save.

/// UI scale: what it is.
#[must_use]
pub const fn ui_scale_title() -> &'static str {
    "Size of pdfcer's own menus, buttons and text"
}

/// UI scale: what is open.
#[must_use]
pub const fn ui_scale_silence() -> &'static str {
    "Not a standards question. Windows already tells pdfcer how big to draw \
     things; this adjusts that up or down for pdfcer alone, without changing \
     any other program."
}

/// UI scale: what changing it costs.
#[must_use]
pub const fn ui_scale_radius() -> &'static str {
    "Applies as soon as you drag it, so you can see it. Cancel puts it back. \
     It never changes the page or the file — only the window around them."
}

/// The slider's own label.
#[must_use]
pub const fn ui_scale_slider_label() -> &'static str {
    "Size"
}

/// The slider's value, as a percentage of the system setting.
#[must_use]
pub fn ui_scale_percent(multiplier: f64) -> String {
    format!("{:.0} %", multiplier * 100.0)
}

/// How to choose one.
#[must_use]
pub const fn ui_scale_note() -> &'static str {
    "Larger is easier to read and leaves less room for the drawing, because \
     the ribbon and the side panels take more of the window. Smaller gives the \
     drawing more room until the labels start to crowd. pdfcer ships 100 %, \
     which means exactly what Windows asked for."
}

// ===========================================================================
// Display — how pdfcer draws (the SHELL's own preferences)
// ===========================================================================
//
// These two are not spec ambiguities and their `_silence` lines say so
// rather than inventing a clause. That distinction is the window's whole
// framing — its opening paragraph promises that everything below exists
// because the standard declines to have an opinion — so a group that does not
// fit it has to say why it is here, not pretend it fits.

/// Render quality: what it is.
#[must_use]
pub const fn quality_title() -> &'static str {
    "How sharply pages are drawn"
}

/// Render quality: what is open.
#[must_use]
pub const fn quality_silence() -> &'static str {
    "Not a question about the PDF standard — a trade between how sharp a page \
     looks and how long it takes to draw. Big engineering drawings are where it \
     shows."
}

/// Render quality: what changing it costs.
#[must_use]
pub const fn quality_radius() -> &'static str {
    "Affects what you see and how fast it appears. Does not change the file, \
     and does not change what prints."
}

/// One quality's name.
#[must_use]
pub const fn quality_label(quality: crate::prefs::RenderQuality) -> &'static str {
    use crate::prefs::RenderQuality as Q;
    match quality {
        Q::Faster => "Faster",
        Q::Normal => "Normal (pdfcer's default)",
        Q::Sharper => "Sharper",
    }
}

/// One quality's description.
#[must_use]
pub const fn quality_note(quality: crate::prefs::RenderQuality) -> &'static str {
    use crate::prefs::RenderQuality as Q;
    match quality {
        Q::Faster => {
            "Three quarters of the detail, and quicker on a large sheet. Thin \
             lines go soft, which on a drawing made of thin lines is what you \
             notice."
        }
        Q::Normal => {
            "One dot drawn for every dot your screen has. As sharp as the \
             display can show, and no work wasted going finer."
        }
        Q::Sharper => {
            "Half again as much detail as the screen can show. Worth it for \
             small text over fine linework, where a single screen dot has to \
             carry two strokes; wasted effort on anything else."
        }
    }
}

/// Zoom settle: what it is.
#[must_use]
pub const fn settle_title() -> &'static str {
    "How long zooming waits before redrawing"
}

/// Zoom settle: what is open. As the quality setting — nothing.
#[must_use]
pub const fn settle_silence() -> &'static str {
    "Also not a standards question. While you are zooming, pdfcer stretches the \
     picture it already has rather than redrawing the page for every step — \
     this is how long it waits for you to stop."
}

/// Zoom settle: what changing it costs.
#[must_use]
pub const fn settle_radius() -> &'static str {
    "Affects how zooming feels. Does not change the file."
}

/// The slider's own label.
#[must_use]
pub const fn settle_slider_label() -> &'static str {
    "Wait"
}

/// The slider's unit.
#[must_use]
pub const fn settle_suffix() -> &'static str {
    " ms"
}

/// How to choose one.
#[must_use]
pub const fn settle_note() -> &'static str {
    "Shorter feels more responsive and redraws the page more often, which on a \
     dense drawing can make zooming stutter. Longer leaves the stretched \
     picture on screen for a moment after you stop, which reads as blurry. \
     pdfcer ships 150."
}

// ===========================================================================
// Display — what you see when a document FIRST OPENS
// ===========================================================================
//
// Two settings, both `look` radius, and both saying the same thing in their
// radius line: **they apply to the next document, not to this one.**
//
// That sentence is not padding and it is not a limitation being apologised
// for. It is the answer to the question an operator asks the moment they
// change either of these with a document already on screen — *why did nothing
// happen?* — and it is a deliberate design decision rather than a shortcoming:
// applying them live would resize the page the operator is looking at and
// switch off overlays they turned on by hand, because of a preference about
// documents in general. `app::prefs::Prefs::opening_fit` carries the argument.

/// Opening fit: what it is.
#[must_use]
pub const fn opening_fit_title() -> &'static str {
    "How a page is sized when a document opens"
}

/// Opening fit: what is open.
#[must_use]
pub const fn opening_fit_silence() -> &'static str {
    "Not a question about the PDF standard — the page has a size, and this is \
     how much of it pdfcer shows you first."
}

/// Opening fit: what changing it costs.
#[must_use]
pub const fn opening_fit_radius() -> &'static str {
    "Applies to the next document you open, not to the one on screen. Does not \
     change the file."
}

/// One opening fit's name.
#[must_use]
pub const fn opening_fit_label(fit: crate::prefs::OpeningFit) -> &'static str {
    use crate::prefs::OpeningFit as F;
    match fit {
        F::Page => "The whole page (pdfcer's default)",
        F::Width => "The full width",
        F::Height => "The full height",
        F::ActualSize => "Actual size",
    }
}

/// One opening fit's description.
#[must_use]
pub const fn opening_fit_note(fit: crate::prefs::OpeningFit) -> &'static str {
    use crate::prefs::OpeningFit as F;
    match fit {
        F::Page => {
            "Always shows you the thing you just opened, whatever size it is. \
             On a large drawing that means the detail starts small."
        }
        F::Width => {
            "Fills the window edge to edge and lets the bottom run off screen. \
             Good for reading down a long sheet; on a wide drawing it is barely \
             different from the whole page."
        }
        F::Height => {
            "Fills the window top to bottom and lets the side run off screen. The one for a landscape drawing sheet in a tall window, where fitting the whole page leaves a band across the middle."
        }
        F::ActualSize => {
            "One dot on screen for one point on the page — the size it would \
             print at, near enough. On an A1 sheet you will be looking at a \
             corner of it."
        }
    }
}

/// What the wheel does on a single page: the setting's name.
#[must_use]
pub const fn wheel_paging_title() -> &'static str {
    "Mouse wheel on a single page"
}

/// What happens if you never touch it.
#[must_use]
pub const fn wheel_paging_silence() -> &'static str {
    "The wheel scrolls within the page, which is what pdfcer has always done."
}

/// What it costs, and what it does not affect.
#[must_use]
pub const fn wheel_paging_radius() -> &'static str {
    "Applies at once, to every open document. Has no effect under a continuous page display, where the wheel scrolls the whole document anyway, and none on Ctrl+wheel, which always zooms."
}

/// One wheel-paging option's name.
#[must_use]
pub const fn wheel_paging_label(paging: crate::prefs::WheelPaging) -> &'static str {
    use crate::prefs::WheelPaging as W;
    match paging {
        W::Scroll => "Scroll within the page (pdfcer's default)",
        W::FlipPages => "Turn to the next or previous page",
    }
}

/// One wheel-paging option's description.
#[must_use]
pub const fn wheel_paging_note(paging: crate::prefs::WheelPaging) -> &'static str {
    use crate::prefs::WheelPaging as W;
    match paging {
        W::Scroll => {
            "Moves around inside the sheet. On a page that already fits the window there is nothing to move, so the wheel does nothing."
        }
        W::FlipPages => {
            "One notch, one sheet. The one to pick for a drawing set you read a page at a time; the page buttons and Page Up / Page Down keep working either way."
        }
    }
}

/// Which paste chord means which, for a form field: the setting's name.
#[must_use]
pub const fn paste_chords_title() -> &'static str {
    "Copying a form field"
}

/// What happens if you never touch it.
#[must_use]
pub const fn paste_chords_silence() -> &'static str {
    "Ctrl+V pastes a separate field with its own value; Ctrl+Shift+V pastes another box that fills in step with the original."
}

/// What it costs, and what it does not affect.
#[must_use]
pub const fn paste_chords_radius() -> &'static str {
    "Applies at once. Both kinds of paste stay available either way — this only swaps which key does which, and both are on the Edit tab under their own names. Nothing already in a document changes."
}

/// One paste-order option's name.
#[must_use]
pub const fn paste_chords_label(order: crate::prefs::PasteChords) -> &'static str {
    use crate::prefs::PasteChords as P;
    match order {
        P::PdfcerOrder => "Ctrl+V makes a separate field (pdfcer's default)",
        P::AcrobatOrder => "Ctrl+V makes another box for the same field (matches Acrobat)",
    }
}

/// One paste-order option's description.
#[must_use]
pub const fn paste_chords_note(order: crate::prefs::PasteChords) -> &'static str {
    use crate::prefs::PasteChords as P;
    match order {
        P::PdfcerOrder => {
            "The copy is its own field: filling one leaves the other alone. Usually what you want going down a column of a title block. Hold Shift to link them instead."
        }
        P::AcrobatOrder => {
            "The copy shares the original's value, so typing in either shows in both — Acrobat's own behaviour, and how a form repeats one answer across sheets. Hold Shift to get a separate field instead."
        }
    }
}

/// Page overlays: what they are.
#[must_use]
pub const fn chrome_title() -> &'static str {
    "What is switched on when a document opens"
}

/// Page overlays: what is open.
#[must_use]
pub const fn chrome_silence() -> &'static str {
    "Also not a standards question. These are the three switches in the View \
     tab's Display group. pdfcer does not remember them per document, so \
     without this they start off every time."
}

/// Page overlays: what changing them costs.
#[must_use]
pub const fn chrome_radius() -> &'static str {
    "Applies to the next document you open, not to the one on screen. Does not \
     change the file."
}

/// The rulers switch.
#[must_use]
pub const fn chrome_rulers_label() -> &'static str {
    "Rulers"
}

/// What turning the rulers on costs.
#[must_use]
pub const fn chrome_rulers_note() -> &'static str {
    "A measuring strip down the top and left edges. It takes that strip out of \
     the space the drawing gets, on every page."
}

/// The grid switch.
#[must_use]
pub const fn chrome_grid_label() -> &'static str {
    "Grid"
}

/// What the grid is.
#[must_use]
pub const fn chrome_grid_note() -> &'static str {
    "A drafting grid drawn over the page. It is never part of the document and \
     never prints."
}

/// The guides switch.
#[must_use]
pub const fn chrome_guides_label() -> &'static str {
    "Guides"
}

/// What the guides switch does, and what it does not.
#[must_use]
pub const fn chrome_guides_note() -> &'static str {
    "Guide lines you drag onto the page to line things up. You drag them out \
     of a ruler, so switch the rulers on too or there is nothing to drag from."
}

/// What pdfcer does about guides a document already has.
#[must_use]
pub const fn chrome_guides_bound() -> &'static str {
    "Whichever you choose, a document you have already placed guides on opens \
     with them showing. Work you did outranks a default."
}

// ===========================================================================
// Drawing the page — how much is remembered
// ===========================================================================

/// Page cache: what it is.
#[must_use]
pub const fn page_cache_title() -> &'static str {
    "How many pages pdfcer keeps in memory"
}

/// Page cache: what is open.
#[must_use]
pub const fn page_cache_silence() -> &'static str {
    "Not a question about the PDF standard — a trade between memory and waiting. \
     Drawing a dense engineering sheet takes about two thirds of a second, so a \
     page pdfcer still remembers appears instantly and one it has forgotten does \
     not."
}

/// Page cache: what changing it costs.
///
/// Names the direction that can actually hurt. Too small is slow, which is
/// recoverable and obvious; too large is an allocation failure, which is not.
#[must_use]
pub const fn page_cache_radius() -> &'static str {
    "Affects memory and waiting, never the file. Setting it higher than this \
     machine can spare will make pdfcer fail to draw rather than run slowly."
}

/// One cache size's name — the step, and what it actually costs.
#[must_use]
pub fn page_cache_label(cache: crate::prefs::PageCache) -> String {
    use crate::prefs::PageCache as C;
    let mb = cache.megabytes();
    match cache {
        C::Small => format!("Small — about {mb} MB"),
        C::Medium => format!("Medium — about {mb} MB"),
        C::Large => format!("Large — about {mb} MB (pdfcer's default)"),
        C::Maximum => format!("Maximum — about {mb} MB"),
    }
}

/// One cache size's description.
#[must_use]
pub const fn page_cache_note(cache: crate::prefs::PageCache) -> &'static str {
    use crate::prefs::PageCache as C;
    match cache {
        C::Small => {
            "What pdfcer used before this release. Enough for a few large sheets; \
             scrolling across a drawing set will redraw them."
        }
        C::Medium => "A report, or a dozen large sheets at a time.",
        C::Large => {
            "About twenty-five large sheets at screen size. Enough that moving \
             back and forth through a drawing set does not redraw anything."
        }
        C::Maximum => {
            "A whole drawing set kept ready at once. Choose it if this machine \
             has memory to spare and you work across many sheets."
        }
    }
}

/// Title for the mesh patch-padding setting.
pub const fn mesh_padding_title() -> &'static str {
    "A gradient fill that comes out scrambled"
}

/// What the standard leaves open here.
pub const fn mesh_padding_silence() -> &'static str {
    "Smooth gradient fills come in several kinds. Two of them store their data in a way the standard describes only by pointing at the rule for a different kind — and that rule is written in terms of a part those two kinds do not have. Both readings survive the wording, and the 2.0 edition repeats it unchanged, so this is permanent rather than something waiting to be clarified."
}

/// The radius line.
pub const fn mesh_padding_radius() -> &'static str {
    "Affects what you see and what you print. Does not change the file."
}

/// Label for the per-record reading.
pub const fn mesh_padding_record_label() -> &'static str {
    "Start each patch on a fresh byte (pdfcer's default)"
}

/// Note for the per-record reading.
pub const fn mesh_padding_record_note() -> &'static str {
    "Reads the cross-reference as meaning something. Most files are unaffected either way: the two readings differ only when a file's numbers do not happen to fill whole bytes. When they do differ, they differ completely — one patch out of step scrambles every patch after it."
}

/// Label for the continuous reading.
pub const fn mesh_padding_none_label() -> &'static str {
    "Read straight through without a break"
}

/// Note for the continuous reading.
pub const fn mesh_padding_none_note() -> &'static str {
    "Reads the cross-reference as importing nothing. Try this if a gradient fill renders as noise, banding or the wrong shape and the rest of the page is fine."
}

/// The presets row's own heading.
pub const fn preset_title() -> &'static str {
    "Start from a known set of answers"
}

/// What the presets row is for.
pub const fn preset_silence() -> &'static str {
    "Sets everything below in one go. You can still change any of them afterwards, and nothing is applied until you press Save."
}

/// Label for pdfcer's own recommended answers.
pub const fn preset_pdfcer_label() -> &'static str {
    "pdfcer recommended"
}

/// Note for the same.
pub const fn preset_pdfcer_note() -> &'static str {
    "What pdfcer ships with, including the two answers you chose personally: neutral black for line art, and smoothing pictures that are shrunk to fit. Use this to get back after experimenting."
}

/// **This standard's render answers are the same as N others'.**
#[must_use]
pub fn preset_same_as_others(others: usize) -> String {
    format!(
        "The same render answers as {others} other preset(s) here. These standards differ in \
         what they require of the FILE — embedded fonts, an output intent, whether transparency \
         is allowed — which is a preflight question. What they ask of a renderer is the same, so \
         switching between them changes nothing on screen."
    )
}

/// What a standard does NOT specify, listed by name.
#[must_use]
pub fn preset_leaves_alone(keys: &str) -> String {
    format!("This standard says nothing about: {keys}. Those keep whatever you have set.")
}

/// How much weight a standard's answers can bear.
#[must_use]
pub fn preset_weight(sourced: usize, inferred: usize, chosen: usize) -> String {
    // A standard that specifies NOTHING gets a sentence, not three zeroes.
    //
    // PDF/UA is the case: nine rendering terms across all 197 of its rules,
    // zero hits, and it hands colour contrast explicitly to WCAG. "Of its
    // answers: 0 stated, 0 inferred, 0 chosen" is arithmetically true and reads
    // as a control that failed to load.
    //
    // The engine asked for exactly this and gave the reason: *"a variant that
    // answers 'nothing, and here is the measurement' cannot be mistaken for an
    // unfinished one. Please surface it that way rather than hiding the
    // entry."* An empty tally would have hidden it in plain sight.
    if sourced == 0 && inferred == 0 && chosen == 0 {
        return "This standard does not say how a page should be rendered — it sets nothing here, and that is its answer rather than a gap."
            .to_owned();
    }
    format!(
        "Of its answers: {sourced} stated by the standard, {inferred} inferred from it, {chosen} chosen by pdfcer where it is silent."
    )
}

/// **Getting the ribbon and the left strip out of the way** — his instruction
/// of 2026-09-05, *"we should also add the capability to auto hide the ribbon
/// until we hover over top of it… left rail should also have the option to auto
/// hide as well."*
#[must_use]
pub const fn auto_hide_title() -> &'static str {
    "Give the drawing more room"
}

/// What happens if you never touch it.
#[must_use]
pub const fn auto_hide_silence() -> &'static str {
    "Both strips stay where they are, all the time. That is how pdfcer has always worked and nothing about it changes unless you switch one of these on."
}

/// What it costs, and what it does not affect.
#[must_use]
pub const fn auto_hide_radius() -> &'static str {
    "With one of these on, the strip appears OVER the drawing when you move the pointer onto its edge, so nothing you were about to click moves. There is always somewhere to put the pointer: the ribbon keeps its row of tab names, and the left strip keeps a narrow marked band."
}

/// The ribbon toggle's own label.
#[must_use]
pub const fn auto_hide_ribbon_label() -> &'static str {
    "Hide the ribbon buttons until I point at the tabs"
}

/// The note under the ribbon toggle.
#[must_use]
pub const fn auto_hide_ribbon_note() -> &'static str {
    "The row of tab names — File, View, Pages and the rest — stays on screen. Only the buttons under it go, and they come back the moment the pointer reaches that row."
}

/// The rail toggle's own label.
#[must_use]
pub const fn auto_hide_rail_label() -> &'static str {
    "Shrink the left strip until I point at it"
}

/// The note under the rail toggle.
#[must_use]
pub const fn auto_hide_rail_note() -> &'static str {
    "The strip of panel and tool buttons down the left edge shrinks to a narrow band with a small arrow on it. Move the pointer onto the band and the strip comes back over the panel beside it."
}

// ===========================================================================
// Forms — the order Tab visits a page in
// ===========================================================================
//
// Two settings whose blast radius is a keystroke. Neither writes a byte, and
// both say so, because an operator changing how Tab behaves has every reason
// to wonder whether they are also changing the document.
//
// They are here rather than in a module of their own for the reason the four
// *Drawing the page* strings are here: this file holds every setting whose
// radius stops at what the operator sees or does.

/// Tab tail: the question.
#[must_use]
pub const fn tab_tail_title() -> &'static str {
    "Tab order on a page that puts form fields first"
}

/// Tab tail: what the standard leaves open.
#[must_use]
pub const fn tab_tail_silence() -> &'static str {
    "Some pages say, in their own data, that Tab should visit every form field first and everything else afterwards. The standard describes what happens on that second pass twice, in two different places, and the two descriptions disagree with each other. Neither one mentions the other."
}

/// Tab tail: what changing it costs.
#[must_use]
pub const fn tab_tail_radius() -> &'static str {
    "Changes the order Tab moves through a page like that, and nothing else. Does not change the file. Whichever way it is set, pdfcer still tells you that the standard contradicts itself on that page."
}

/// Tab tail: the shipped reading.
#[must_use]
pub const fn tab_tail_array_label() -> &'static str {
    "The order things were added to the page"
}

/// Tab tail: why the shipped reading is shipped.
#[must_use]
pub const fn tab_tail_array_note() -> &'static str {
    "What pdfcer ships. Both passes follow the page's own list from top to bottom. This is the reading that keeps a page's stated order meaning something: the whole reason a page can ask for fields first is to escape being ordered by where things happen to sit."
}

/// Tab tail: the other reading.
#[must_use]
pub const fn tab_tail_row_label() -> &'static str {
    "Where things sit on the page, left to right"
}

/// Tab tail: who the other reading is for.
#[must_use]
pub const fn tab_tail_row_note() -> &'static str {
    "Comments, stamps and everything that is not a form field get visited by position instead. Choose this if you have checked what the program your readers use actually does, and found it to be this."
}

/// Row tolerance: the question.
#[must_use]
pub const fn tab_tolerance_title() -> &'static str {
    "How close two things must be to count as the same row"
}

/// Row tolerance: what the standard leaves open.
#[must_use]
pub const fn tab_tolerance_silence() -> &'static str {
    "A page can ask for Tab to move in reading order, which means pdfcer has to work out which things are on the same line before it can go along that line. The standard describes no test for that at all, so this number is pdfcer's own."
}

/// Row tolerance: what changing it costs.
#[must_use]
pub const fn tab_tolerance_radius() -> &'static str {
    "Changes the order Tab moves through a page that asks for reading order. Does not change the file. Every order pdfcer works out tells you which number it used."
}

/// Row tolerance: the slider's own label.
#[must_use]
pub const fn tab_tolerance_slider_label() -> &'static str {
    "Same row within"
}

/// Row tolerance: the slider's unit.
#[must_use]
pub const fn point_suffix() -> &'static str {
    " pt"
}

/// Row tolerance: how to choose one.
///
/// Names both failure modes rather than recommending a number, the same way
/// the zoom settle note does, because which one bites depends on the drawing.
#[must_use]
pub const fn tab_tolerance_note() -> &'static str {
    "Too small and two boxes a hair apart become two separate rows, so Tab zig-zags down the page. Too large and a whole column folds into one row, so Tab runs across the page when you wanted it to run down. pdfcer ships 1."
}

/// Coloured icons: the title. `OPERATOR_REQUESTS.md` O232.
#[must_use]
pub const fn colour_icons_title() -> &'static str {
    "Coloured icons"
}

/// Coloured icons: what it does, and that off is how pdfcer ships.
#[must_use]
pub const fn colour_icons_silence() -> &'static str {
    "Draws one part of each toolbar icon in a quiet colour, the way Office and \
     SOLIDWORKS do: a green plus, a red cross, a blue arrow. On as pdfcer \
     ships; off draws every icon in one colour."
}

/// Coloured icons: what changing it costs.
#[must_use]
pub const fn colour_icons_radius() -> &'static str {
    "Applies as soon as you tick it. Cancel puts it back. Disabled buttons \
     stay grey either way. It never changes the page or the file — only the \
     icons around them."
}

/// The checkbox's own label.
#[must_use]
pub const fn colour_icons_label() -> &'static str {
    "Colour the icons"
}
