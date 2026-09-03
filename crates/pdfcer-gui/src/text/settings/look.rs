//! # `text::settings::look` — what changing it makes you SEE
//!
//! One of three copy modules under [`crate::text::settings`], split on
//! 2026-08-17 at rule R2's 1,500-line ceiling.
//!
//! ## ★ The split is by BLAST RADIUS, which is the window's own taxonomy
//!
//! Not by dialog group, and not alphabetically. Every setting in this window
//! carries a `*_radius` line stating *which way costs what*, and that line is
//! one of exactly three things:
//!
//! | module | radius | settings |
//! |---|---|---|
//! | [`super::look`] | changes what you SEE; the file is untouched | theme, CMYK intent, CMYK JPEG polarity, mask resampling, minification |
//! | [`super::extract`] | changes what you GET OUT — copy, search, redaction-by-pattern, new dimensions | word gap, unmappable codes, replacement text, parallel tolerance |
//! | [`super::bytes`] | changes what pdfcer WRITES | separations, missing appearance state, index line endings, trailing newline |
//!
//! That taxonomy is load-bearing rather than a filing convenience: it is the
//! distinction the window exists to make legible, and a test in
//! [`super`] asserts that exactly the byte-changing settings say they change
//! the file — in both directions, so a preview setting cannot quietly claim a
//! consequence it does not have.
//!
//! One setting is filed by its radius rather than by its group and it is worth
//! naming: **CMYK JPEG polarity** appears above under *look*, and its radius
//! line also says *"and the saved file if pdfcer re-compresses the image"*. It
//! is the only setting whose radius spans two of the three. It sits with the
//! others in its dialog group, where an operator looks for it.

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
// Colour — CMYK intent
// ===========================================================================

/// CMYK intent: what it is.
#[must_use]
pub const fn cmyk_intent_title() -> &'static str {
    "How CMYK colour is shown on screen"
}

/// CMYK intent: what the standard leaves open.
#[must_use]
pub const fn cmyk_intent_silence() -> &'static str {
    "The standard (section 8.6.4.4) defines no conversion from CMYK ink to \
     screen colour at all — it depends on the device. Acrobat's answer is a \
     profile you can change; this is pdfcer's."
}

/// CMYK intent: what changing it costs.
#[must_use]
pub const fn cmyk_intent_radius() -> &'static str {
    "Affects what you see. Does not change the file."
}

/// The shipped default, listed first because it is the one in force.
#[must_use]
pub const fn cmyk_intent_neutral_label() -> &'static str {
    "Black ink is black (pdfcer's default)"
}

/// Why the shipped default exists.
///
/// The last sentence is the one that matters: the divergence is **narrow by
/// construction**. Only the pure-K axis moves and every mixed colour still
/// uses the measured table, so an operator worrying that pdfcer has invented
/// its own colour science can be answered from the window.
#[must_use]
pub const fn cmyk_intent_neutral_note() -> &'static str {
    "Pure black ink shows as true black. Right for CAD and engineering drawings, \
     where every line is drawn in black ink alone. Only pure black changes; \
     every mixed colour is still the measured one."
}

/// The option that agrees with other readers.
#[must_use]
pub const fn cmyk_intent_calibrated_label() -> &'static str {
    "Match other PDF viewers"
}

/// What matching costs.
#[must_use]
pub const fn cmyk_intent_calibrated_note() -> &'static str {
    "Colours are measured to match what Acrobat and most other viewers show. \
     Solid black ink appears as a very dark warm grey rather than true black, \
     because that is what those viewers do. Choose this when you want to see a \
     document the way someone else will."
}

// ★★★ `cmyk_intent_naive_label`, `cmyk_intent_naive_note` and
// `cmyk_intent_divergence` were DELETED here on 2026-08-28
// (`OPERATOR_REQUESTS.md` O52), and the deletion is recorded because two of the
// three carried arguments worth keeping even though the strings are gone.
//
// **The naive option** was offered for one stated purpose — *"only useful for
// comparing against something pdfcer produced earlier"* — which was true while
// pdfcer had recently produced such files. ⇒ A control whose entire
// justification **expires with time** is one nothing will ever prompt anybody
// to remove: no test fails, no gate fires, and the copy still reads sensibly.
// The operator asked for it and that is what it took.
//
// **The divergence note** said pdfcer's default deliberately differed from
// Acrobat, and existed so a future session would not investigate a
// render-parity difference as a defect. The default now matches, so the
// sentence is not redundant but **backwards**. ⇒ A note explaining a divergence
// must die with the divergence; keeping it "for context" is how a window comes
// to describe a program that no longer exists.

// ===========================================================================
// Colour — CMYK JPEG polarity
// ===========================================================================

/// Polarity: what it is.
#[must_use]
pub const fn polarity_title() -> &'static str {
    "Reading a CMYK JPEG that does not say which way round it is"
}

/// Polarity: what the standard leaves open.
#[must_use]
pub const fn polarity_silence() -> &'static str {
    "Some CMYK JPEGs store their ink values inverted and nothing in the file says \
     so. No document defines how to tell — the marker people point at carries no \
     such flag."
}

/// Polarity: what changing it costs.
///
/// **The only preview setting that can also change saved bytes**, and it is
/// the second half of the sentence that says so. A re-encode under the wrong
/// polarity bakes the inversion in permanently.
#[must_use]
pub const fn polarity_radius() -> &'static str {
    "Affects what you see, and the saved file if pdfcer re-compresses the image."
}

/// The default.
#[must_use]
pub const fn polarity_never_label() -> &'static str {
    "Take the values as stored (pdfcer's default)"
}

/// ★ **The one positively-sourced default in the window, and it says so.**
///
/// Every other note either says "this is a guess" or says nothing about
/// provenance. This one claims the opposite, and it is entitled to: `"invert"`
/// occurs **zero times** in the Adobe technical note the standard makes
/// normative, the marker carries no polarity flag at all, and all four
/// reference engines make the same choice.
///
/// The distinction is the point. "pdfcer matched every other implementation"
/// and "pdfcer guessed" must not read alike, or the operator has no way to tell
/// which of thirteen defaults to trust.
#[must_use]
pub const fn polarity_never_note() -> &'static str {
    "The best-supported answer here: the reference document never mentions \
     inverting, the marker carries no flag to test, and all four major PDF \
     engines make the same choice."
}

/// The opt-in.
#[must_use]
pub const fn polarity_invert_label() -> &'static str {
    "Invert when an Adobe marker is present"
}

/// Why getting this wrong is safe to discover.
#[must_use]
pub const fn polarity_invert_note() -> &'static str {
    "For a library of old Photoshop-authored images that genuinely do store \
     inverted ink. Getting this wrong renders a photographic negative — an \
     obvious failure rather than a subtle one, so it is easy to tell which way \
     your files need."
}

// ===========================================================================
// Colour — where a page's blending space comes from
// ===========================================================================

/// ★★ **The ceiling on the buffer pages are blended in** — the answer to his
/// 2026-08-26 question.
///
/// > *"can the size of the buffer be increased? Allow the user to set the size
/// > up to the maximum possible?"*
///
/// # Why this is worded as a SYMPTOM and not as a buffer
///
/// Nobody arrives at this window looking for a compositing buffer. They arrive
/// because *"the colours in this file change when I zoom"* — which is exactly
/// how it was reported, and which is what the title says. The mechanism is in
/// the note, underneath, for the reader who wants it.
///
/// # The three numbers this copy owes, and why each is here
///
/// All measured by the engine, not estimated:
///
/// * **20 bytes a pixel**, which is what makes the memory figure predictable;
/// * **about 50 % slower** than blending on screen at the same pixel count
///   (1.4 s against 0.9 s at the boundary) — the trade is *correct colours are
///   slower*, and an operator raising a ceiling deserves to know that before
///   they notice it;
/// * ★ **up to about four times the number chosen**, because the ceiling bounds
///   ONE buffer and a page with nested transparency holds several page-sized
///   ones at once — the page buffer, a group's child, a retained spare, and a
///   whole backdrop copy for a knockout group.
///
/// The third is the one that must not be left out. The first version of the
/// advice given to the operator said "about 5 GB is the maximum possible",
/// which is right per buffer and **four times too low as a memory figure** —
/// and too low is the direction he could act on.
#[must_use]
pub const fn cmyk_ceiling_title() -> &'static str {
    "Colours changing when you zoom"
}

/// What happens if you never touch it.
#[must_use]
pub const fn cmyk_ceiling_silence() -> &'static str {
    "A print-ready page is blended in ink up to about 500 % zoom on A4, and on screen colours above it — so its colours shift slightly as you zoom past that. The status bar says when it has happened."
}

/// What it costs, and what it does not affect.
#[must_use]
pub const fn cmyk_ceiling_radius() -> &'static str {
    "Changes how pages are drawn and printed. It never changes the file, and it has no effect at all on a page that is not blended in ink — which is nearly every page that is not print-ready, including CAD line work."
}

/// The field's own label.
#[must_use]
pub const fn cmyk_ceiling_label() -> &'static str {
    "Most memory to use for ink blending"
}

/// The field's note — the mechanism, the cost, and the four-times warning.
#[must_use]
pub const fn cmyk_ceiling_note() -> &'static str {
    "Type `default`, or a size such as `512mib` or `1.5gb`. Blending in ink costs 20 bytes for every pixel of the page image, so a bigger allowance buys exact colours at higher zoom: about 640 MB reaches 800 % on A4, and 1.5 GB reaches about 1200 %. Two things worth knowing before you raise it — blending in ink takes roughly half again as long, and a page with layered transparency can need up to about four times the figure you set here, because it holds several of these at once. Nothing stops you setting a number this machine cannot supply; if that happens the page simply blends on screen instead and the status bar says so, exactly as it does today."
}

/// Shown beside a free-text setting whose typed form and canonical form differ.
///
/// The whole line is `= 256 MiB`, and it appears **only** when the operator's
/// spelling is not the canonical one — `256 MiB` echoed back as `256 MiB` is
/// noise, while `0.25gb` answered with `256 MiB` is the reason the line exists.
/// It is how somebody learns, without being told, that this field speaks binary
/// megabytes.
#[must_use]
pub fn parsed_as(canonical: &str) -> String {
    format!("= {canonical}")
}

/// Shown beside a free-text setting whose contents do not parse.
///
/// Deliberately says what is still true rather than what is wrong: the stored
/// value stands, nothing has been lost, and the operator is very likely
/// mid-keystroke. A message that read like a rejection would be false — the
/// field accepts every keystroke and always will.
#[must_use]
pub const fn unparsed_value_note() -> &'static str {
    "Not a size pdfcer recognises yet — the saved value is unchanged. Try `default`, `512mib`, `1.5gb`, or a plain number of bytes."
}

/// Where a page's blending colour space comes from: the setting's name.
///
/// ★ Worded as the **symptom**, not the mechanism. Nobody arrives at this
/// window looking for a "page group blending colour space"; they arrive
/// because a print file's overprinted areas look wrong, or because a file
/// renders differently than it used to. The title is the sentence that makes
/// the person with that problem stop scrolling.
#[must_use]
pub const fn blend_space_title() -> &'static str {
    "Overprint in print-ready files"
}

/// What happens if you never touch it.
#[must_use]
pub const fn blend_space_silence() -> &'static str {
    "A file that declares itself destined for ink renders its overprinted areas the way a print-oriented viewer would."
}

/// What it costs, and what it does not affect.
#[must_use]
pub const fn blend_space_radius() -> &'static str {
    "Changes how pages are drawn and printed, everywhere. It never changes the file. It has no effect at all on a file with no output intent, which is most files that are not print-ready."
}

/// One blend-space source's name.
#[must_use]
pub const fn blend_space_label(src: pdfcer_core::settings::PageBlendSpaceSource) -> &'static str {
    use pdfcer_core::settings::PageBlendSpaceSource as S;
    match src {
        S::DeviceNative => "What the standard literally says",
        S::OutputIntentIfSubtractive => {
            "Follow the file's print intent, when it has one (pdfcer's default)"
        }
        S::OutputIntentAlways => "Always follow the file's output intent",
        // ★ `PageBlendSpaceSource` is `#[non_exhaustive]`, so the engine may
        // add a source without breaking this build. A new one must not render
        // as a blank radio label, and it must not be silently mapped onto a
        // neighbour either -- both would be a control lying about what it
        // selects. Naming it as unknown is the honest answer and it is
        // self-reporting: the operator can quote it.
        _ => "A newer pdfcer added this option; this build cannot describe it",
    }
}

/// One blend-space source's description.
#[must_use]
pub const fn blend_space_note(src: pdfcer_core::settings::PageBlendSpaceSource) -> &'static str {
    use pdfcer_core::settings::PageBlendSpaceSource as S;
    match src {
        S::DeviceNative => {
            "Strictly conforming, and it cannot show overprint at all -- on screen an overprinted area simply takes the topmost colour. Pick it to reproduce how pdfcer drew pages before this setting existed."
        }
        S::OutputIntentIfSubtractive => {
            "Only files that declare an ink destination are affected; an RGB or greyscale intent is ignored. On the industry's own test suite, 24 of 51 overprint patches come out right this way and wrong the other."
        }
        S::OutputIntentAlways => {
            "Honours the output intent whatever it says. The most literal reading of the standard's annex, and a larger change than the evidence supports -- an RGB intent would start moving pages too."
        }
        _ => "Not described by this build.",
    }
}

// ===========================================================================
// Comments — who signs them
// ===========================================================================

/// Author name: what it is.
///
/// ★★ The title says **your comments**, not *"annotation title"* and not
/// *"the /T entry"*. §12.5.6.4 calls the field a title, which is a word from
/// the format rather than a word about the job, and an operator scanning
/// headings for *"why do my comments say nobody"* would not stop at it.
#[must_use]
pub const fn author_name_title() -> &'static str {
    "Your name on the comments you write"
}

/// What happens if you never touch it.
///
/// ★★★ It states the consequence plainly rather than calling the empty value
/// a problem. An anonymous comment is legal, is what pdfcer wrote before this
/// existed, and is a reasonable choice for a drawing leaving the firm. The
/// window's job is to say what silence does, not to nag.
#[must_use]
pub const fn author_name_silence() -> &'static str {
    "Comments you write are anonymous — the author column in Acrobat and in pdfcer's own Comments panel is blank."
}

/// What it costs, and what it does not affect.
///
/// ★ It names the two boundaries that matter: it goes into the file, and it
/// does not apply retroactively. Somebody who sets it after a review session
/// should not expect yesterday's comments to be signed.
#[must_use]
pub const fn author_name_radius() -> &'static str {
    "Written into every comment you make from now on, and it travels with the file to whoever you send it to. Comments already in the document are not changed."
}

/// The field's own label.
#[must_use]
pub const fn author_name_label() -> &'static str {
    "Your name"
}

/// The note under the field.
///
/// ★ It answers the question the empty box provokes — *"why does pdfcer not
/// already know this?"* — because the answer is a decision rather than an
/// omission, and one an operator would agree with if told.
#[must_use]
pub const fn author_name_note() -> &'static str {
    "Leave it blank to stay anonymous. pdfcer does not take your Windows login for this: the name goes into files you send to other people, so it is yours to choose."
}

// ===========================================================================
// Images and transparency — mask resampling
// ===========================================================================

/// Mask resampling: what it is.
#[must_use]
pub const fn mask_title() -> &'static str {
    "Smoothing a transparency mask that is a different size"
}

/// Mask resampling: what the standard leaves open.
#[must_use]
pub const fn mask_silence() -> &'static str {
    "An image's transparency mask can be a different size from the image it \
     applies to. The standard says the two are stretched over the same area, and \
     says nothing at all about how to fill in the in-between values."
}

/// Mask resampling: what changing it costs.
#[must_use]
pub const fn mask_radius() -> &'static str {
    "Affects what you see. Does not change the file."
}

/// The default.
#[must_use]
pub const fn mask_nearest_label() -> &'static str {
    "Use the nearest value (pdfcer's default)"
}

/// Why, and that the why is pdfcer's own.
#[must_use]
pub const fn mask_nearest_note() -> &'static str {
    "Never invents a transparency value that is not in the mask. Matters for a \
     hard-edged cut-out, where blending would fabricate soft edges that were \
     never there. Edges can look slightly stepped. This is pdfcer's own \
     reasoning, not a rule from anywhere — no standard or reference program \
     defines it."
}

/// The middle option.
#[must_use]
pub const fn mask_box_label() -> &'static str {
    "Average the surrounding values"
}

/// When it is the right answer.
///
/// The second sentence is not in the source and is the case that actually
/// decides it: a mask at **higher** resolution than the base read one sample
/// per texel discards fifteen sixteenths of what the producer supplied.
#[must_use]
pub const fn mask_box_note() -> &'static str {
    "Smoother on photographic masks, at the cost of softening edges that were \
     meant to be sharp. It is also the better answer when the mask is finer than \
     the image it covers, where taking one value per pixel throws most of it away."
}

/// The smoothest option.
#[must_use]
pub const fn mask_bilinear_label() -> &'static str {
    "Blend smoothly"
}

/// What it risks.
#[must_use]
pub const fn mask_bilinear_note() -> &'static str {
    "The smoothest result, and the most likely to invent transparency values the \
     mask never contained."
}

// ===========================================================================
// Images and transparency — minification
// ===========================================================================

/// Minification: what it is.
#[must_use]
pub const fn minify_title() -> &'static str {
    "Shrinking a large image to fit"
}

/// Minification: what the standard leaves open.
#[must_use]
pub const fn minify_silence() -> &'static str {
    "The standard describes smoothing only for images being ENLARGED. It says \
     nothing about images being shrunk, which is what happens whenever a \
     high-resolution scan is displayed at page size."
}

/// Minification: what changing it costs.
#[must_use]
pub const fn minify_radius() -> &'static str {
    "Affects what you see. Does not change the file."
}

/// The default.
#[must_use]
pub const fn minify_point_label() -> &'static str {
    "Take one pixel in each area"
}

/// ★ **The guess disclosure the old window omitted.**
///
/// `pdfcer-core` grades this default tier (d) — reasoned inference — as
/// explicitly as it grades the mask filter beside it, and the old note read as
/// a confident recommendation with no such admission. Obligation 1 was
/// therefore unmet for this setting for the whole of its shipped life.
///
/// The final sentence is more than a disclosure: it names the **specific
/// observation that would change the default**, which is what turns a
/// confessed guess into a piece of work somebody can do.
#[must_use]
pub const fn minify_point_note() -> &'static str {
    "Fast, and follows the standard's wording literally. Fine detail such as thin \
     lines or small text in a scan can shimmer or disappear entirely. This is \
     pdfcer's own reading rather than a rule from anywhere: what other viewers \
     actually do when shrinking has not been checked, and if it turns out they \
     smooth, this default should change."
}

/// The alternative.
#[must_use]
pub const fn minify_smooth_label() -> &'static str {
    "Average the area (pdfcer's default)"
}

/// What it buys and costs.
#[must_use]
pub const fn minify_smooth_note() -> &'static str {
    "Better-looking on scanned pages and photographs — detail is averaged rather \
     than dropped. Slower, and goes beyond what the standard describes."
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
/// ★ **It says "the View tab" rather than "View ▸ Display", and that is not a
/// style choice.** `icons::glyphs`' coverage gate refused the first draft: the
/// font stack this shell ships cannot draw `U+25B8 ▸`, so the separator this
/// project's *documentation* uses everywhere would have rendered as a
/// substitution box in front of the operator. The gate found it on the first
/// run of the new copy, which is the second real tofu it has caught. Internal
/// notation is not operator copy.
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
/// Shown under a selected conformance preset when its six values match another
/// preset's exactly — which, measured on 2026-08-26, is true of all eight of
/// the PDF/X and PDF/A presets.
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
