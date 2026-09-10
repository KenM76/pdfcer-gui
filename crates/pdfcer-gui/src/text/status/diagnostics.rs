//! # `text::status::diagnostics` — **the narrator's whole vocabulary**
//!
//! Every sentence the *Render notes* disclosure can say, and nothing else.
//! Split out of [`super`] on 2026-09-10 under standing rule **R2** when the
//! tenth finding — the appearance-less annotation — pushed the parent file to
//! 1,509 lines against a 1,500-line cap.
//!
//! ## ★ The seam is a consumer, not an alphabet
//!
//! A catalog area in this crate is keyed by **the surface it serves**, and
//! every function here is read by exactly one: `app::status::notes`, whose
//! `findings` table pairs a renderer counter with one of these sentences and
//! whose `notes_line` joins the results. Nothing else in the shell calls any
//! of them, and the Render-diagnostics dialog reaches them only *through*
//! `findings`, deliberately — two tables would agree on the day they were
//! written and disagree the first time an eleventh counter was added to one.
//!
//! ⇒ So the split cost no call site a character: every name is re-exported
//! from [`super`], and `t::diagnostics_images_skipped` still resolves exactly
//! where it always did. That is the difference between a structural fix and a
//! churn commit, and it is the same argument [`super::selection`] carries.
//!
//! ## What a sentence in here has to do
//!
//! **Name the consequence on the page, never the mechanism in pdfcer.** The
//! catalog's own worked example is [`diagnostics_fonts_skipped`], which says
//! *"text from 2 fonts not drawn"* rather than *"2 unsupported fonts"*: a
//! count of unsupported fonts is a fact about pdfcer, and missing text is a
//! fact about the picture in front of the operator. He can act on the second.
//!
//! **Singular and plural are written out.** *"1 images not drawn"* is the sort
//! of thing an operator screenshots, and every entry here pays six lines to
//! avoid it.
//!
//! **And it must not accuse the document.** These are things pdfcer had to
//! substitute or leave out; [`diagnostics_tooltip`] says so in as many words,
//! because the difference between *"pdfcer approximated something"* and
//! *"your document is damaged"* is the single most valuable thing this
//! surface can teach.

/// The disclosure control's label, closed and open.
///
/// ★ **Closed is the default, and the caption is still shown.** `DEFECTS.md`
/// records the old shell opening with a substitute-glyph census: *"The first
/// thing a user reads is the app talking about itself. Excellent
/// information, wrong prominence."* The fix is prominence, not deletion — so
/// the report is one click away and named, rather than hidden behind a bare
/// triangle nobody would think to press.
///
/// "Render notes" rather than "Diagnostics": the operator's question is
/// *"did pdfcer draw my page faithfully?"*, and "diagnostics" is the word an
/// application uses about itself.
///
/// ★ **The triangles are `⏵` (U+23F5) and `⏷` (U+23F7), and the choice was
/// forced by measurement rather than taste.** The obvious glyphs for a
/// disclosure — `▸` U+25B8 and `▾` U+25BE — are **absent from egui's
/// bundled font set** (Ubuntu-Light + NotoEmoji + emoji-icon-font), as are
/// `▶`/`◀`. They were in this file first and
/// [`crate::app::status::tests::every_glyph_the_status_bar_draws_has_a_glyph`]
/// caught them: on screen they would have been tofu boxes, which on a
/// disclosure means an operator cannot tell open from closed.
///
/// `⏵` is therefore also [`next_page`]'s glyph, which is a real (small)
/// collision and is accepted rather than worked around: the two controls sit
/// at **opposite ends** of the bar, this one always carries the word "Render
/// notes" beside it, and this one alternates while the page arrows never do.
/// Substituting a non-triangle here — `›`, `»` — would trade a resolvable
/// ambiguity for a control that no longer looks like a disclosure at all.
#[must_use]
pub fn diagnostics_toggle(open: bool) -> &'static str {
    if open {
        "⏷ Render notes"
    } else {
        "⏵ Render notes"
    }
}

/// Hover text for the disclosure.
///
/// Says what the report is *about*, because the difference between "pdfcer
/// approximated something" and "your document is damaged" is the single
/// most valuable thing this surface can teach.
#[must_use]
pub fn diagnostics_tooltip() -> &'static str {
    "What pdfcer had to substitute or leave out when it drew this page. \
     These are facts about the renderer, not faults in your document."
}

/// Shown when the page drew with nothing substituted and nothing skipped.
///
/// Stated positively rather than left blank. An empty disclosure is
/// indistinguishable from a disclosure that failed to fill itself, and the
/// operator who opened it wanted an answer either way.
#[must_use]
pub fn diagnostics_clean() -> &'static str {
    "Drawn with nothing substituted or left out"
}

/// Glyphs painted from a **bundled** substitute face.
///
/// Positions are the document's own; the shapes are pdfcer's. Worth its own
/// line rather than being folded into [`diagnostics_glyphs_supplied`],
/// because the two have different remedies: a bundled substitute is fixed by
/// supplying the real font, and a supplied one is already the operator's own
/// deliberate choice.
#[must_use]
pub fn diagnostics_glyphs_substituted(n: usize) -> String {
    if n == 1 {
        "1 glyph drawn with a bundled substitute face".to_owned()
    } else {
        format!("{n} glyphs drawn with a bundled substitute face")
    }
}

/// Glyphs painted from an **operator-supplied** face.
#[must_use]
pub fn diagnostics_glyphs_supplied(n: usize) -> String {
    if n == 1 {
        "1 glyph drawn from a supplied font".to_owned()
    } else {
        format!("{n} glyphs drawn from a supplied font")
    }
}

/// Glyphs that had no shape at all — `.notdef`, or nothing painted.
#[must_use]
pub fn diagnostics_glyphs_notdef(n: usize) -> String {
    if n == 1 {
        "1 glyph with no shape available".to_owned()
    } else {
        format!("{n} glyphs with no shape available")
    }
}

/// Whole fonts whose machinery this build does not implement; their text was
/// **skipped**, not approximated.
///
/// Worded as "text not drawn" rather than "fonts unsupported" because the
/// consequence is what the operator can see on the page. A count of
/// unsupported fonts is a fact about pdfcer; missing text is a fact about the
/// picture in front of them.
#[must_use]
pub fn diagnostics_fonts_skipped(n: usize) -> String {
    if n == 1 {
        "text from 1 font not drawn".to_owned()
    } else {
        format!("text from {n} fonts not drawn")
    }
}

/// Images that could not be drawn at all.
#[must_use]
pub fn diagnostics_images_skipped(n: usize) -> String {
    if n == 1 {
        "1 image not drawn".to_owned()
    } else {
        format!("{n} images not drawn")
    }
}

/// Annotations the file carries that pdfcer drew **nothing** for.
///
/// ## ★★★ Why this one is worded as an absence rather than as a fault
///
/// Every other sentence in this catalog describes something the operator can
/// look at and find wrong — a substituted glyph, an image-shaped hole. This
/// one describes a thing that is **not on the page at all**, and the operator
/// has no way to know it was ever there. A colleague's sticky note or approval
/// stamp that arrived without a baked appearance stream renders as clean
/// paper, and clean paper is exactly what an unannotated drawing looks like.
///
/// ⇒ That is why it is reported at all, and why it is reported **off-canvas**
/// per **R8b rule 4**: the finding is not that pdfcer failed, it is that the
/// operator is looking at less than the file contains. Nothing is drawn onto
/// the page to mark the absence, because a marker would need a position and
/// the whole problem is that pdfcer has an appearance-less annotation whose
/// `/Rect` it will not presume to fill.
///
/// ## Why *"carries no appearance"* and not *"is broken"*
///
/// An annotation with no `/AP` is **legal**. §12.5.2 makes `/AP` optional and
/// puts the drawing duty on the reader for some subtypes and on nobody for
/// others; a `/Square` with no appearance stream is a conforming file that
/// most readers show as nothing. Calling it broken would accuse a document
/// that is fine, which is the same error
/// `text::stamps::properties_stamp_no_page` was written to avoid.
///
/// ## ⚠ What the number is, and the one way it could lie
///
/// `annotations_without_ap` summed over its subtypes, **minus**
/// `annotations_icon_painted` — the ones pdfcer nevertheless drew from its own
/// standard-icon artwork (`Pass 289.0`). The engine keeps those two apart
/// deliberately: the map is a fact about the *file* and the counter is a fact
/// about what the *operator saw*, and folding them together would make one of
/// the two a lie. This sentence wants the second question, so it subtracts.
///
/// The lie it could tell is a **narrowed annotation scope**: the census is
/// taken under every scope but the icon counter only increments in scope, so
/// under `--no-annotations` the subtraction would report *"not drawn"* about
/// content that was **withheld on request**. `app::status::notes::findings`
/// therefore suppresses the whole finding when `annotations_out_of_scope` is
/// non-zero, which is the distinction the engine's own row insists on:
/// *"withheld"* and *"tried and failed"* must stay distinguishable.
#[must_use]
pub fn diagnostics_annots_no_appearance(n: usize) -> String {
    if n == 1 {
        "1 annotation carries no appearance and is not drawn".to_owned()
    } else {
        format!("{n} annotations carry no appearance and are not drawn")
    }
}

/// Operators recognised but not yet implemented.
#[must_use]
pub fn diagnostics_ops_deferred(n: usize) -> String {
    if n == 1 {
        "1 drawing operator not yet implemented".to_owned()
    } else {
        format!("{n} drawing operators not yet implemented")
    }
}

/// Operators not recognised at all.
///
/// Distinct from [`diagnostics_ops_deferred`]: "not implemented" is a gap in
/// pdfcer with a name, and "unrecognised" means the content stream contained
/// something no version of pdfcer expects — which is usually a fact about the
/// file.
#[must_use]
pub fn diagnostics_ops_unknown(n: usize) -> String {
    if n == 1 {
        "1 unrecognised drawing operator".to_owned()
    } else {
        format!("{n} unrecognised drawing operators")
    }
}

/// Optional-content sections that were hidden and therefore not drawn.
///
/// Reported even though hiding a layer is usually the operator's own doing,
/// because the alternative reading of a suddenly-emptier page is "the render
/// failed". Naming the cause is the difference between a control working and
/// a control looking broken.
#[must_use]
pub fn diagnostics_layers_hidden(n: usize) -> String {
    if n == 1 {
        "1 hidden layer section not drawn".to_owned()
    } else {
        format!("{n} hidden layer sections not drawn")
    }
}

/// `/Contents` entries that named an object the file does not contain.
///
/// The one entry here that is a statement about the **document** rather than
/// about the renderer, and it is worded that way: the page is incomplete
/// because part of it is missing from the file, not because pdfcer declined
/// to draw it.
#[must_use]
pub fn diagnostics_contents_missing(n: usize) -> String {
    if n == 1 {
        "1 content stream missing from the file".to_owned()
    } else {
        format!("{n} content streams missing from the file")
    }
}

/// The page carried no `/Resources` dictionary, and pdfcer supplied one.
///
/// # ★★★ Why a repair that changes nothing on the page is still disclosed
///
/// Engine `Pass 290.0` (2026-09-10) stopped refusing a page whose
/// `/Resources` is absent on itself **and every ancestor**, substituting the
/// empty dictionary ISO 32000 Table 30 names for exactly this case. The page
/// then renders normally — rule 4, no mark on the canvas, nothing tinted —
/// and this line is the off-canvas half of that.
///
/// ⚠ **It is not cosmetic bookkeeping, and the reason is easy to get
/// backwards.** The obvious reading is *"a page with no content stream has
/// nothing to resolve resources for, so an empty dictionary costs nothing"*.
/// That reading is **false**: §7.8.3 lets a form XObject or a Type 3 font omit
/// its own `/Resources` and inherit **the page's**, and the ISO 32000-2
/// erratum extends that to **annotation appearance streams**. A page whose
/// only marks are annotations — the shape of the operator's own
/// Acrobat-written signature file — can genuinely need the dictionary that was
/// not there.
///
/// So this line is drawn **first** in the findings list, because a font or an
/// image reported missing below it may be missing *because of* it. Read the
/// other way round, an operator goes looking for a font that was never the
/// problem.
///
/// Terse because [`diagnostics_join`] puts it in the status bar beside other
/// findings; the argument above lives here and in the Render-diagnostics
/// dialog's ordering, not in the operator's one-line disclosure.
#[must_use]
pub const fn diagnostics_resources_defaulted() -> &'static str {
    "this page names no resources of its own — an empty set was assumed"
}

/// Join the notes into the single line the disclosure shows.
///
/// The separator lives here rather than at the call site because it is
/// operator-visible punctuation, and because putting it in the widget would
/// be the first crack in "every string a human can read is defined here".
///
/// `·` (U+00B7) rather than a comma: the parts are independent facts, not a
/// list in a sentence, and a middle dot survives being read at a glance in a
/// small weak font better than a comma does.
#[must_use]
pub fn diagnostics_join(parts: &[String]) -> String {
    parts.join(" · ")
}
