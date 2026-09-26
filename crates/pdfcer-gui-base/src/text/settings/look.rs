//! # `text::settings::look` — what changing it makes you see in the DOCUMENT
//!
//! ## The split is by BLAST RADIUS, which is the window's own taxonomy
//!
//! Not by dialog group, and not alphabetically. Every setting in this window
//! carries a `*_radius` line stating *which way costs what*, and that line is
//! one of exactly four things:
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/text/settings/look.md`.

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
#[must_use]
pub const fn polarity_radius() -> &'static str {
    "Affects what you see, and the saved file if pdfcer re-compresses the image."
}

/// The default.
#[must_use]
pub const fn polarity_never_label() -> &'static str {
    "Take the values as stored (pdfcer's default)"
}

/// **The one positively-sourced default in the window, and it says so.**
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

/// **The ceiling on the buffer pages are blended in** — the answer to his
/// 2026-08-26 question.
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
#[must_use]
pub fn parsed_as(canonical: &str) -> String {
    format!("= {canonical}")
}

/// Shown beside a free-text setting whose contents do not parse.
#[must_use]
pub const fn unparsed_value_note() -> &'static str {
    "Not a size pdfcer recognises yet — the saved value is unchanged. Try `default`, `512mib`, `1.5gb`, or a plain number of bytes."
}

/// Where a page's blending colour space comes from: the setting's name.
///
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
        // `PageBlendSpaceSource` is `#[non_exhaustive]`, so the engine may
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
#[must_use]
pub const fn author_name_title() -> &'static str {
    "Your name on the comments you write"
}

/// What happens if you never touch it.
#[must_use]
pub const fn author_name_silence() -> &'static str {
    "Comments you write are anonymous — the author column in Acrobat and in pdfcer's own Comments panel is blank."
}

/// What it costs, and what it does not affect.
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

/// **The guess disclosure the old window omitted.**
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
