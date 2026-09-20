//! # `text::settings::shell` — the settings that change pdfcer's own window
//!
//! The fourth row of the blast-radius taxonomy in [`super::look`]'s header.
//! Every setting here stops at what the operator sees or does: none of them
//! writes a byte, none of them changes a page, and each one's `_radius` line
//! says so, because an operator changing how Tab behaves or how big the
//! buttons are drawn has every reason to wonder whether they are also changing
//! the document.
//!
//! ## ★ Why this is a module and not a section of [`super::look`]
//!
//! The window's opening paragraph promises that everything in it exists
//! *because the standard declines to have an opinion*. That promise is true of
//! the first three modules and false of this one, and the difference is not
//! cosmetic: a `_silence` line has to answer *"what does the standard leave
//! open?"*, and for a setting the standard has never heard of the honest
//! answer is **"nothing — this is not a standards question"**. Those lines say
//! exactly that rather than inventing a clause to sit under, and keeping them
//! in one place is what stops the next one being written to match its
//! neighbours instead of the truth.
//!
//! Two settings here answer a real ambiguity whose blast radius still stops at
//! a keystroke — the Tab-order pair, where the standard describes the second
//! pass twice and the two descriptions disagree. They are filed by radius like
//! everything else in this window, so they are here.
//!
//! ## The theme and UI scale are twins and must stay together
//!
//! They are the only two settings in the whole window that take effect **before
//! Save** — both apply the moment they are picked so the operator can see them,
//! and both are put back by Cancel. That is an exception to the window's
//! draft-until-Save contract, and it is stated in two `_radius` lines that have
//! to keep saying the same thing. Split across files, one of them drifts.

use super::*;

// ===========================================================================
// Appearance — theme
// ===========================================================================

/// Theme: what it is.
#[must_use]
pub const fn theme_title() -> &'static str {
    "Theme"
}

/// Theme: what the standard leaves open.
///
/// Nothing — and saying so is the point. This is the one setting in the window
/// that is not a spec ambiguity, and letting it silently share the shape of
/// the twelve that are would imply pdfcer thinks the standard has an opinion
/// about window colours.
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
///
/// # The catch-all arm is required, and it is not a `todo!()`
///
/// `egui_shell::theme::Preset` is `#[non_exhaustive]` — deliberately, because
/// the whole point of the shell crate is that another application may ship
/// presets pdfcer has never heard of. So this catalog cannot be exhaustive over
/// it and the compiler says so.
///
/// The fallback returns the preset's own **key** rather than a placeholder.
/// That is the honest answer: a theme this catalog has no prose for still has
/// a name the operator can recognise, since the key is what they would have
/// typed into the settings file. A `todo!()` would crash the settings window
/// on a preset the *shell* is entitled to add, and a literal like "Other"
/// would tell them nothing and would be indistinguishable between two such
/// presets.
#[must_use]
pub fn theme_preset_label(preset: Preset) -> &'static str {
    match preset {
        Preset::Quiet => "Quiet",
        Preset::Airy => "Airy",
        Preset::Dark => "Dark",
        // ui-text-exempt: not a literal — the shell's own key for a preset this
        // catalog predates. See the doc comment.
        other => other.key(),
    }
}

/// One preset's description.
///
/// Each says what it looks like *and* what it costs, because "which theme do I
/// want" is not answerable from three adjectives. Airy's *"uses more screen"*
/// and Dark's *"as CAD tools do it"* are the two facts that actually decide it
/// for this audience.
/// The catch-all returns an **empty** description rather than inventing one,
/// and the window omits the line entirely when it is empty — an absent note is
/// truthful about a preset this catalog cannot describe, whereas a generic one
/// would be prose pdfcer made up about somebody else's theme.
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
        _ => "",
    }
}

/// The settings file names a theme this build does not have.
///
/// # Why this is said out loud, with the name quoted
///
/// Without it the operator sees none of the three radios selected and no
/// explanation, which reads as a rendering fault. And the likeliest cause is
/// benign and worth knowing: a settings file written by a **newer** pdfcer,
/// whose token this build is **preserving rather than overwriting**. Quoting
/// the name is what makes that legible — and telling them it is kept is what
/// stops them "fixing" it by picking one of the three, which would discard it.
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
///
/// **"pdfcer's own"** does the work in this title. The word an operator is most
/// likely to arrive with is *"zoom"*, and zoom in this application means the
/// page — so the title has to draw the line before the operator has read a
/// word of the body, or they will set this expecting the document to change.
#[must_use]
pub const fn ui_scale_title() -> &'static str {
    "Size of pdfcer's own menus, buttons and text"
}

/// UI scale: what is open.
///
/// Says what it multiplies, because that is the fact that stops it being
/// misread as an override. An operator who has already set Windows display
/// scaling to 150 % needs to know this stacks on top rather than replacing it.
#[must_use]
pub const fn ui_scale_silence() -> &'static str {
    "Not a standards question. Windows already tells pdfcer how big to draw \
     things; this adjusts that up or down for pdfcer alone, without changing \
     any other program."
}

/// UI scale: what changing it costs.
///
/// ★ Two disclosures in one line, and both are needed. It takes effect
/// immediately — the exception to the whole window's draft-until-Save contract
/// that this setting shares with the theme — and it does **not** resize the
/// page, which is the thing an operator will most reasonably expect it to do
/// given that the word "size" is in the title.
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
///
/// A percentage rather than the stored multiplier, because *"125 %"* is a
/// quantity an operator can hold against the Windows display setting they
/// already know, and *"1.25"* is one they have to interpret. Same value, and
/// the unit is doing the explaining.
///
/// Rounded to whole percent: the step is 0.05, so every value the control can
/// produce is a whole number of percent and no precision is lost. A decimal
/// place would show `100.0 %` and imply a fineness the control does not have.
#[must_use]
pub fn ui_scale_percent(multiplier: f64) -> String {
    format!("{:.0} %", multiplier * 100.0)
}

/// How to choose one.
///
/// Names the two failure modes rather than recommending a number, as the
/// zoom-settle note does — and for a stronger version of the same reason:
/// which value is right depends on the operator's eyes and their monitor, so
/// there is no number to recommend. What can be said is what going too far in
/// each direction looks like, and an operator who knows that can find their
/// value in two drags.
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
// ★ These two are not spec ambiguities and their `_silence` lines say so
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
///
/// Nothing, and it says so. This is a **preference**, a trade between sharpness
/// and speed that depends on the machine and on how big the drawings are — not
/// a question the standard leaves unanswered. Saying "the standard does not
/// define…" here would be inventing a clause to fit a template, which is the
/// dishonest version of consistency.
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
pub const fn quality_label(quality: crate::app::prefs::RenderQuality) -> &'static str {
    use crate::app::prefs::RenderQuality as Q;
    match quality {
        Q::Faster => "Faster",
        Q::Normal => "Normal (pdfcer's default)",
        Q::Sharper => "Sharper",
    }
}

/// One quality's description.
///
/// Each names **what it costs**, not just what it does — which is the whole
/// content of the choice. "Faster" without "softer" is half a sentence, and it
/// is the half that makes the setting look free.
#[must_use]
pub const fn quality_note(quality: crate::app::prefs::RenderQuality) -> &'static str {
    use crate::app::prefs::RenderQuality as Q;
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
///
/// A catalog entry rather than a literal, for the reason the degree sign and
/// the point abbreviation are: the ui-strings gate looks for exactly this, and
/// a translator has to be able to see that a unit exists.
#[must_use]
pub const fn settle_suffix() -> &'static str {
    " ms"
}

/// How to choose one.
///
/// Names both failure modes rather than recommending a number, because which
/// one bites depends on the machine — and an operator who knows what going too
/// far in each direction looks like can find their own value in two tries.
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
// ★ Two settings, both `look` radius, and both saying the same thing in their
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
///
/// As the two settings above it — nothing. The PDF standard has an opinion
/// about page *size*; it has none about how a viewer chooses to fit that size
/// to a window, which is why this is a preference rather than an ambiguity.
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
pub const fn opening_fit_label(fit: crate::app::prefs::OpeningFit) -> &'static str {
    use crate::app::prefs::OpeningFit as F;
    match fit {
        F::Page => "The whole page (pdfcer's default)",
        F::Width => "The full width",
        F::Height => "The full height",
        F::ActualSize => "Actual size",
    }
}

/// One opening fit's description.
///
/// Each names what it costs on a **large sheet**, because that is the case
/// where they differ and it is the case this shell exists for. On a letter page
/// at a normal window size all three look much the same, and copy written
/// against that case would tell the operator nothing about the choice they are
/// actually making.
#[must_use]
pub const fn opening_fit_note(fit: crate::app::prefs::OpeningFit) -> &'static str {
    use crate::app::prefs::OpeningFit as F;
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
///
/// ★ The second sentence is the one that matters. Under a continuous display
/// mode the wheel scrolls the whole document by definition, so this setting
/// has nothing to change — and an operator who tried it there and saw no
/// difference would reasonably conclude it was broken.
#[must_use]
pub const fn wheel_paging_radius() -> &'static str {
    "Applies at once, to every open document. Has no effect under a continuous page display, where the wheel scrolls the whole document anyway, and none on Ctrl+wheel, which always zooms."
}

/// One wheel-paging option's name.
#[must_use]
pub const fn wheel_paging_label(paging: crate::app::prefs::WheelPaging) -> &'static str {
    use crate::app::prefs::WheelPaging as W;
    match paging {
        W::Scroll => "Scroll within the page (pdfcer's default)",
        W::FlipPages => "Turn to the next or previous page",
    }
}

/// One wheel-paging option's description.
///
/// ★ The first names the case where today's behaviour is a **dead control**,
/// which is the whole reason the choice exists: this shell opens documents at
/// fit page, and a page that already fits has nothing to scroll.
#[must_use]
pub const fn wheel_paging_note(paging: crate::app::prefs::WheelPaging) -> &'static str {
    use crate::app::prefs::WheelPaging as W;
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
///
/// ★ It names the SUBJECT, not the keys. An operator scanning the pane for
/// *"why did my copied field come out linked?"* is thinking about fields, not
/// about `V`.
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
///
/// ★★ Three things, and the second is the one that stops the support question.
/// Both pastes always exist — this only exchanges the keys — so an operator who
/// picks the Acrobat order has lost nothing and can still reach either from the
/// Edit tab. The third sentence forestalls the other reasonable worry: it is a
/// keyboard preference, not a document one, so nothing already pasted changes.
#[must_use]
pub const fn paste_chords_radius() -> &'static str {
    "Applies at once. Both kinds of paste stay available either way — this only swaps which key does which, and both are on the Edit tab under their own names. Nothing already in a document changes."
}

/// One paste-order option's name.
///
/// ★ The pdfcer entry does not say *"pdfcer's default"* the way `wheel_paging`'s
/// does, because here the alternative is named after a **product** and the pair
/// would read as an endorsement contest. Each names what its Ctrl+V does, which
/// is the fact being chosen between.
#[must_use]
pub const fn paste_chords_label(order: crate::app::prefs::PasteChords) -> &'static str {
    use crate::app::prefs::PasteChords as P;
    match order {
        P::PdfcerOrder => "Ctrl+V makes a separate field (pdfcer's default)",
        P::AcrobatOrder => "Ctrl+V makes another box for the same field (matches Acrobat)",
    }
}

/// One paste-order option's description.
///
/// ★★★ Both notes lead with the CONSEQUENCE — whether typing in one box shows
/// in the other — because that is the only difference an operator can observe,
/// and it is invisible on the page. Two linked boxes and two independent boxes
/// are pixel-identical until somebody types.
///
/// ★ The Acrobat note says *why* Acrobat does it, rather than only that it does.
/// An operator picking a compatibility setting deserves to know it is a real
/// convention with a purpose — repeated page-number and date fields that must
/// agree — and not merely a quirk being mimicked.
#[must_use]
pub const fn paste_chords_note(order: crate::app::prefs::PasteChords) -> &'static str {
    use crate::app::prefs::PasteChords as P;
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
///
/// Names the thing an operator is most likely to have come here about — that
/// these are switches they have to flick on every single document — because the
/// group headings are how a symptom finds its setting and this is the symptom.
///
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
///
/// ★ It states the cost, and the cost is real rather than rhetorical: the
/// gutters come off the drawing area, on every document, for as long as the
/// preference is set. `ViewState::default`'s own comment calls this *"the one
/// default that has a measurable cost"*, which is why it ships off — and an
/// operator turning it on permanently deserves to be told what they are
/// spending.
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
///
/// ★ **The second sentence is the whole reason this control has notes at all.**
/// `canvas::guides::ruler_drag` registers nothing when the rulers are hidden,
/// so an operator who switches guides on and cannot place one has met a
/// coupling the program never told them about. Saying it here costs one line
/// and saves the conclusion that the feature is broken.
#[must_use]
pub const fn chrome_guides_note() -> &'static str {
    "Guide lines you drag onto the page to line things up. You drag them out \
     of a ruler, so switch the rulers on too or there is nothing to drag from."
}

/// What pdfcer does about guides a document already has.
///
/// A [`crate::dialogs::settings::widgets::disclosure`] rather than a note under
/// the guides switch, because it is true **whichever way that switch is set** —
/// which is exactly the distinction that widget documents, and the same reason
/// the replacement-text bound is one.
///
/// It exists because the alternative is silent surprise in the honest
/// direction: an operator who sets this off will still see guides appear on the
/// documents they placed guides on, and with nothing said that reads as the
/// preference not working.
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
///
/// Nothing about the standard, and it says so — the same honesty
/// [`quality_silence`] applies. What it says instead is the **symptom**, because
/// that is how an operator finds this control: they came here because scrolling
/// back to a sheet made them wait.
#[must_use]
pub const fn page_cache_silence() -> &'static str {
    "Not a question about the PDF standard — a trade between memory and waiting. \
     Drawing a dense engineering sheet takes about two thirds of a second, so a \
     page pdfcer still remembers appears instantly and one it has forgotten does \
     not."
}

/// Page cache: what changing it costs.
///
/// ★ Names the direction that can actually hurt. Too small is slow, which is
/// recoverable and obvious; too large is an allocation failure, which is not.
#[must_use]
pub const fn page_cache_radius() -> &'static str {
    "Affects memory and waiting, never the file. Setting it higher than this \
     machine can spare will make pdfcer fail to draw rather than run slowly."
}

/// One cache size's name — the step, and what it actually costs.
///
/// ★★ The megabyte figure is **computed from the budget**, never written beside
/// it. Two spellings of one quantity drift, and the drift here would be a
/// settings window promising 512 MB while the cache spent 2 GB —
/// `NO_SURFACE.md` §1's ★★ finding with a number instead of a colour.
///
/// "Large" is not something anybody can budget against. An operator with 8 GB
/// and one with 64 GB are making different decisions and neither can make theirs
/// from an adjective.
#[must_use]
pub fn page_cache_label(cache: crate::app::prefs::PageCache) -> String {
    use crate::app::prefs::PageCache as C;
    let mb = cache.megabytes();
    match cache {
        C::Small => format!("Small — about {mb} MB"),
        C::Medium => format!("Medium — about {mb} MB"),
        C::Large => format!("Large — about {mb} MB (pdfcer's default)"),
        C::Maximum => format!("Maximum — about {mb} MB"),
    }
}

/// One cache size's description.
///
/// Each says **how much work it saves**, in sheets rather than in bytes, because
/// a drawing set is what this operator has and "25 sheets" is a thing he can
/// picture where "1 GB" is not.
#[must_use]
pub const fn page_cache_note(cache: crate::app::prefs::PageCache) -> &'static str {
    use crate::app::prefs::PageCache as C;
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
///
/// ★ Filed by the SYMPTOM, not the mechanism. Nobody goes looking for
/// *"type 6/7 mesh shading patch record byte alignment"*. Somebody whose
/// gradient came out as garbage goes looking for *gradient*, so that is the
/// first word.
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
///
/// ★ States the two facts an operator needs before clicking something that
/// changes several settings at once: what it will do, and that it is not a
/// lock. The second is the one that makes it safe to try.
pub const fn preset_silence() -> &'static str {
    "Sets everything below in one go. You can still change any of them afterwards, and nothing is applied until you press Save."
}

/// Label for pdfcer's own recommended answers.
pub const fn preset_pdfcer_label() -> &'static str {
    "pdfcer recommended"
}

/// Note for the same.
///
/// ★ Worded for the operator who has been experimenting and wants out. That is
/// the reported use — *"touching some of our presets caused some test to show
/// up as failed"* — and it is a person looking for a way back, not a person
/// choosing a philosophy.
pub const fn preset_pdfcer_note() -> &'static str {
    "What pdfcer ships with, including the two answers you chose personally: neutral black for line art, and smoothing pictures that are shrunk to fit. Use this to get back after experimenting."
}

/// ★★★ **This standard's render answers are the same as N others'.**
///
///
/// It exists because the operator asked for the control in order to *"see how
/// far we are along with matching the [conformance suite's] tests"*, and
/// switching to PDF/X-4
/// will change nothing on screen. Finding that out by comparing two identical
/// renders costs an hour and reads as the setting being broken.
///
/// ★★ It says **why**, and the why is the part that stops it sounding like a
/// bug: the standards differ in what they demand of a *file* — fonts embedded,
/// an output intent present, transparency allowed or not — and those are
/// preflight questions. What they ask of a **renderer** is the same, so pdfcer
/// giving them the same answers is agreement rather than laziness.
///
/// ★ The number is counted at the moment of drawing, so if a standard's answers
/// ever diverge this sentence corrects itself. See
/// `crate::dialogs::settings::preset`'s `identical_siblings`.
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
///
/// ★ Named rather than left blank. Roughly a third of the grid is axes a
/// standard does not reach — no PDF/X part contains a shading clause at all —
/// and a blank cell reads as missing data, while a value would assert a
/// requirement that does not exist.
#[must_use]
pub fn preset_leaves_alone(keys: &str) -> String {
    format!("This standard says nothing about: {keys}. Those keep whatever you have set.")
}

/// How much weight a standard's answers can bear.
///
/// ★★★ The sentence that stops this feature being a dropdown. The engine grades
/// every value it supplies, and its own framing is that *the interesting column
/// is not the value, it is how much weight the value can bear.* For PDF/X-4,
/// exactly one of six answers is a claim about the standard at all.
///
/// Worded so the weakest category is unmistakable. "pdfcer chose" is not a
/// hedge — it is the honest description of an axis the standard is silent on,
/// and an operator reading it should understand that switching standards will
/// not change that answer for a reason the standard requires.
#[must_use]
pub fn preset_weight(sourced: usize, inferred: usize, chosen: usize) -> String {
    // ★★★ A standard that specifies NOTHING gets a sentence, not three zeroes.
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
///
/// ★★ The radius line carries the fact that decides whether an operator dares
/// turn this on: **the drawing does not move.** Every program in the class that
/// gets this wrong reflows the document as the strip comes and goes, and an
/// operator who has met that once will not try it again. Saying it here is what
/// makes the setting choosable.
///
/// ★ The title says *"getting out of the way"* rather than *"auto-hide"*,
/// because the operator is looking for room on their drawing, not for a feature
/// name.
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
///
/// A catalog entry rather than a literal, for the reason the millisecond and
/// degree suffixes are: the ui-strings gate looks for exactly this, and a
/// translator has to be able to see that a unit exists.
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
