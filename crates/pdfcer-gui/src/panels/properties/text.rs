//! # `panels::properties::text` — how the selected text LOOKS, and changing it
//!
//! `RIBBON_IA.md` §5.8's Font controls — `format.font`, `format.font_size` —
//! built where that section says to build them:
//!
//! > Build order: **panel first, tab second.** The panel is the harder half
//! > and the tab's contents are a subset of it, so building the tab first
//! > would mean writing the property editors twice.
//!
//! ## The operator's ask, twice
//!
//! > *"We should also have all the font tools available that Word does."*
//! > — O37, 2026-08-25
//!
//! > *"…when I have an object selected like text the Tool tab doesn't switch
//! > to giving me the editable stuff for that object."* — O46, 2026-08-26
//!
//! ## ★★★ The operand is the TEXT SELECTION, not the object selection
//!
//! This is the decision most likely to be read as a shortcut, so it is argued
//! rather than asserted.
//!
//! `EditSession::format_text` locates its operand by a **pinned byte span into
//! a decoded content buffer**, obtained from `GlyphProvenance` — which is
//! keyed on a *run* of the page's text extraction. The canvas object selection
//! is a `TargetId`: a **paint-order index** into `PageObjects`. The two index
//! spaces are unrelated, and nothing in either crate maps between them.
//!
//! So an object-selection operand would have to be inferred — by bounding box
//! overlap, most plausibly — and an inference that picks the wrong run
//! restyles text the operator did not select, silently, in a file they then
//! send to somebody. The text sweep *is* a run range, exactly, by construction.
//!
//! ★ That is a real gap and it is named rather than hidden: clicking a text
//! object with the Select tool does not raise this section; sweeping across the
//! text does.
//!
//! ## ★★★ AMENDED 2026-09-05 — the paragraph above is still true of THIS
//! ## section, and it is no longer true of the panel
//!
//! `OPERATOR_REQUESTS.md` **O89**: *"I don't see where I am able to edit the
//! color of text, vectors, etc."* He had clicked the text and found nothing but
//! a sentence telling him to sweep.
//!
//! [`super::textobject`] now draws a **working colour control** for a clicked
//! text object, and it does not weaken one word of the argument above: its
//! operand is not inferred from geometry, it is the object's own `BT`…`ET`
//! **byte span** joined against each run's show-operator span — an exact
//! containment, in one buffer, of the same kind the pinned edit path already
//! stakes every restyle on. `crate::canvas::textedit::pin::object_text` is the
//! join and carries the argument.
//!
//! ⇒ Two consequences for a reader of this file. **(1)** The sentence formerly
//! ending this paragraph — *"the empty state says so in those words"* — has
//! moved with the state it describes; `route` and `ROUTE_REGION` are gone from
//! here, and the block where they stood says where they went and why the region
//! kept its old spelling. **(2)** This section is now exactly what its heading
//! claims: the editor for a **swept range**. With nothing swept it returns
//! `false` and says nothing, because something else is speaking.
//!
//! ★ Four of the five controls here are still sweep-only, and that is a
//! decision rather than a leftover: face, size, bold and italic each need a
//! reading of **one run** to be honest, and a whole object has no single answer
//! to any of them. Colour is the one property for which *"they disagree"* is
//! itself a displayable answer.
//!
//! ## ★★ Why the read-back is stamped and not re-read every frame
//!
//! The values shown — face, size, colour — come from `GlyphProvenance`, and
//! provenance is **off** in the shared page-text cache. Reading it means an
//! extraction with `capture_provenance` on, which is the expensive thing this
//! shell does: **392 ms on the operator's benchmark sheet.**
//!
//! A panel that did that per frame would take the application to under three
//! frames a second on exactly the drawings this program is for. So
//! [`TextStyleDraft`] carries a stamp — `(page, first run, edit epoch)` — and
//! re-reads only when it moves, which is the same shape as
//! [`super::geometry::GeometryDraft`] and for a much larger reason.
//!
//! ## ★★★ Bold and Italic are NEVER greyed, and that is the engine's ruling
//!
//! `set_font` selects a real face and refuses when the page carries none.
//! `gate_synthesis` refuses synthesis when a real face **is** available.
//!
//! ★★★ **They are NOT exact complements, and this paragraph used to say they
//! were.** It read *"so between them every page is covered and there is no
//! page on which bold is unreachable"*, quoting the engine — who withdrew the
//! claim in writing on 2026-08-27 after reproducing the counter-example.
//! `gate_synthesis` prefers a real face by *family*, and the face it prefers
//! may not map every glyph in the run, in which case `set_font` refuses it and
//! synthesis is already gated off. On `textedit/format_family.pdf` bold is
//! reachable by neither verb. Filed, confirmed, and queued first by the
//! engine; `crate::app::actions::textstyle`'s header carries the whole of it.
//!
//! ★★ The conclusion survives its premise, which is why the two buttons still
//! do not grey. Greying them would mean predicting a refusal that depends on a
//! per-run glyph-coverage test this shell cannot run without doing the
//! engine's work. The honest behaviour is to try and to show the engine's own
//! named refusal, which is what happens.
//!
//! pdfcer-core's instruction, verbatim: *"Do not grey out a bold button. Offer
//! it, and surface the disclosure when synthesis fires."*
//! `crate::app::actions::textstyle` takes whichever verb the page allows and
//! discloses which one it took.
//!
//! ## ★★★ AMENDED 2026-08-29 — the premise above is now half false, and the
//! conclusion still holds
//!
//! The paragraph two up says greying would mean *"predicting a refusal that
//! depends on a per-run glyph-coverage test this shell cannot run without doing
//! the engine's work."* That was true when it was written and it is **no longer
//! true**, which is worth saying plainly rather than leaving as a claim nobody
//! re-measured:
//!
//! * `EditSession::preview_style_resolution` (consumed here on 2026-08-29) says
//!   whether a real face resolves and hands back **the string to pass to
//!   `set_font` to reach it**;
//! * `preview_font_resources` — which [`TextStyleDraft::sync`] has called since
//!   2026-08-27 for the face chooser — says which resources `set_font` would
//!   accept **for this run's characters**, each with the same kind of string.
//!
//! Comparing the two is a string equality between two engine-issued selectors.
//! It is not the coverage test re-implemented, and it is not the family
//! heuristic re-derived — the line `StyleResolution`'s own invariant draws,
//! *"critically, in `pdfcer-gui`"*. So this shell **can** now predict the
//! `format_family.pdf` refusal, and [`StyleOutlook::FaceCannotCover`] is that
//! prediction.
//!
//! ⇒ ★★ **The buttons still do not grey**, and the reason has changed from *we
//! cannot know* to *knowing is not a reason to withhold*:
//!
//! 1. The engine's instruction is unconditional and unwithdrawn.
//! 2. The engine has a **queued fix** — `gate_synthesis` will treat a real face
//!    as available only if `set_font` would accept it for the run — which turns
//!    this case into ordinary synthesis. A control withheld on the strength of a
//!    defect that is about to be fixed is a control that stays withheld for
//!    months after it starts working. A sentence that goes stale is read once
//!    and corrected in one line.
//! 3. R9 reserves greying for the *temporarily* unavailable **and requires it to
//!    explain itself on hover**. The hover is where the explanation already is,
//!    and it now carries the whole answer — so greying would add a disabled
//!    control and no information.
//!
//! What changed instead is which sentence the hover carries, and that is exactly
//! R83's size of change: the operator learns before the gesture rather than from
//! a refusal after it. [`bold_hint`] carries the four-row table.
//!
//! ★ This is also why the two toggles do **not** show the run's current state.
//! There is no "is this run bold" bit in a PDF: weight is a property of the
//! *face* (`Helvetica-Bold` is a different font from `Helvetica`), and a
//! synthetic weight is a stroke width in the content stream. A toggle drawn
//! pressed-in would be claiming to have read a fact that is not recorded. They
//! are **buttons that apply**, not switches that reflect — and the face name
//! beside them is where an operator reads what the text actually is.
//!
//! ## ★★★ AMENDED 2026-08-29 — the face chooser offers faces the document does
//! NOT contain
//!
//! `pdfcer-core` v0.15.0 (`Pass 162.0`) closed the last of the four things the
//! operator named as not fully editable:
//!
//! > **FONTS** — text can be restyled to a face the document **DOES NOT
//! > CONTAIN**, for the fourteen faces every PDF reader is required to have.
//! > pdfcer authors the font resource on demand, with widths, embedding nothing.
//! > A face outside those fourteen still refuses by name — that needs a real
//! > font program.
//!
//! This section offered only what `preview_font_resources` returned, which by
//! construction enumerates *the page's own `/Font` resources* — so the new
//! capability was unreachable from any surface in the program.
//!
//! ⇒ The list now has **two groups** and they are two different acts:
//! rewriting one `Tf` operand, and rewriting it *plus writing a new `/Font`
//! object into the operator's file*. [`super::face`] owns the list, the
//! headings, and the disclosure the second group owes; [`face_row`] owns the
//! row it sits on. Both surfaces — this panel and the ribbon's Format ▸ Font
//! group — draw the identical body, because *"a face offered in one and not the
//! other"* is the divergence this project keeps finding.
//!
//! ★★ **The shell writes nothing.** `FormatPlan::created_font` puts the
//! resource write on the caller, and `EditSession::format_text` **is** that
//! caller: it folds the write into the same undo command on both its page and
//! form paths. [`super::face`]'s header carries the evidence. Nothing here
//! allocates an object or knows the shape of a font dictionary.
//!
//! ★ The refusal for a fifteenth face is a **sentence**, not a silence —
//! `crate::text::status::selection::TextStyleRefusal::FaceNotOnPage`, whose old
//! wording (*"pdfcer can only switch text to a font this page already carries"*)
//! stated a limit this engine no longer has and was corrected in the same
//! change.
//!
//! ## Rule 4: nothing here marks the canvas
//!
//! Every disclosure this section causes — a synthetic weight, a colour space
//! narrowed, a real face used instead — lands in the status bar through
//! `crate::app::actions::disclosure`. **The restyled text renders exactly as
//! the saved file will render it.** No badge, no tint, no "provisional"
//! styling: the one-line test is whether a screenshot of the canvas would
//! differ from a screenshot of the same document saved and reopened, and
//! nothing in this module can make it differ.

use egui::Ui;

use crate::app::actions::Action;
use crate::app::actions::textstyle::StyleChange;
use crate::app::state::OpenDoc;
use crate::text::panels::properties as t;

/// The trace region, so a driven check can find this section on screen.
// ui-text-exempt: trace region name, never displayed
pub const REGION: &str = "properties.text";
/// The Bold button's own region.
///
/// ★ Published per control rather than leaving a driven check to divide
/// [`REGION`] by eye — `geometry`'s own note on this is the precedent, and the
/// reason is that a check computing a control's position from a section's
/// bounds is a check that passes on a build where the controls moved.
// ui-text-exempt: trace region name, never displayed
pub const BOLD_REGION: &str = "properties.text.bold";
/// The **Italic** button's own region, published since 2026-08-29.
///
/// It had none while both weight buttons carried one shared sentence. They now
/// carry per-axis sentences derived from two separate
/// `preview_style_resolution` probes — a page holding a real `Arial-Bold` and no
/// `Arial-Italic` gives different answers for the two — so a driven check that
/// could point at Bold and not at Italic could assert half a per-axis answer,
/// which is the exact case the two probes exist to keep apart.
// ui-text-exempt: trace region name, never displayed
pub const ITALIC_REGION: &str = "properties.text.italic";
/// The size spinner's own region.
// ui-text-exempt: trace region name, never displayed
pub const SIZE_REGION: &str = "properties.text.size";
/// The face chooser's own region.
// ui-text-exempt: trace region name, never displayed
pub const FACE_REGION: &str = "properties.text.face";
// ★★★ `ROUTE_REGION` moved to [`super::textobject`] on 2026-09-05 with the
// sentence it names, and it kept its SPELLING — `properties.text.route`. The
// surface did not move, only the module that draws it, and
// `tools/ui-verify/src/checks/font_group.rs` finds it by that string. A rename
// would have been a harness break dressed as tidiness, and a driven check that
// stops finding a region reports the feature as missing.

/// What the selected text looks like now, re-read only when it can have
/// changed.
///
/// # The stamp is three parts and every one is load-bearing
///
/// * **page** — a run ordinal means nothing without one;
/// * **first run** — the operator moved the selection to different text;
/// * **edit epoch** — the text is the same text and its style changed, which is
///   what happens on every press of a control in this section. Without this
///   term the panel would show the pre-edit size for ever after the first
///   change, which is the failure that makes a properties panel untrustworthy.
#[derive(Default)]
pub struct TextStyleDraft {
    /// `(page, first run, edit epoch)` the values below were read at.
    stamp: Option<(usize, usize, u64)>,
    /// The run's `/BaseFont`, subset tag and all, or `None` when the
    /// provenance carried no font resource.
    face: Option<String>,
    /// The `Tf` size in points.
    size: f64,
    /// The fill colour as sRGB bytes, or `None` when the run is painted in a
    /// space this control cannot round-trip.
    ///
    /// ★ `None` is shown as *"not a plain colour"* rather than as black. A
    /// swatch that renders a CMYK or Separation fill as its nearest RGB and
    /// then writes that back on the next press would silently convert the
    /// operator's ink — the exact narrowing pdfcer refuses to do on their
    /// behalf elsewhere.
    colour: Option<[u8; 3]>,
    /// The size the operator is typing, kept separate from [`Self::size`] so a
    /// half-typed number does not become an edit.
    typed_size: f64,
    /// ★★★ **Which faces on this page `set_font` would accept for this run**,
    /// and the string to pass for each — `Pass 142.1`, consumed 2026-08-27.
    ///
    /// Read behind the same stamp as everything else here, because it costs
    /// another extraction and answers a question that only changes when the
    /// selection or the document does.
    ///
    /// `Vec<(selector, label, ambiguous)>`, flattened at read time rather than
    /// holding the engine's `FontPreflight`: the chooser needs three strings
    /// per row and holding the whole preflight would mean this struct's
    /// lifetime being tied to a `pdfcer-core` type that will grow.
    ///
    /// **Empty is a real answer and is not the same as "not read".** A page
    /// whose every font refuses this run — which `format_family.pdf` was until
    /// `Pass 144.0` — has no face to offer, and the chooser says so rather than
    /// falling back to a list of entries that cannot work.
    faces: Vec<FaceChoice>,
    /// ★★★ **What pressing Bold would actually do to this run** —
    /// `EditSession::preview_style_resolution`, consumed 2026-08-29.
    ///
    /// Behind the same stamp as everything else here, because it costs a third
    /// content-stream plan and answers a question that changes only when the
    /// selection or the document does. `None` when the probe could not be run
    /// at all, which is a fourth state distinct from the three
    /// [`StyleOutlook`] carries and is rendered as the old conditional hint —
    /// the sentence that was there before this landed, and which is still the
    /// honest thing to say when nothing is known.
    bold_outlook: Option<StyleOutlook>,
    /// The italic twin of [`Self::bold_outlook`], probed **separately**.
    ///
    /// ★★ Two probes and not one, and it is not symmetry for its own sake.
    /// `gate_synthesis` is all-or-nothing per *combined* request: a page holding
    /// a real `Arial-Bold` but no `Arial-BoldItalic` answers a Bold+Italic
    /// request with *"no real face — synthesize both"*, silently passing over
    /// the real Bold. The two buttons here are two separate single-axis
    /// requests (`crate::app::dispatch::format` builds each with the other flag
    /// false — *"buttons that apply, not switches that reflect"*), so each one
    /// needs the answer for its own axis and neither may borrow the other's.
    ///
    /// ⇒ `StyleResolution::is_mixed` is deliberately not consulted anywhere in
    /// this module: it can only fire for a combined request, and this shell
    /// never issues one.
    italic_outlook: Option<StyleOutlook>,
}

/// ★★★ **What one of the two weight buttons would do to this run**, as three
/// distinguishable outcomes.
///
/// Derived entirely from two engine answers — `preview_style_resolution` and
/// `preview_font_resources` — and from no rule re-implemented here.
/// `StyleResolution`'s own invariant is explicit that a shell must not
/// re-derive the matching heuristics, *"critically, in `pdfcer-gui`"*, because a
/// second copy would drift from the commit path the first time the heuristic
/// changed and would be lost in the WASM fork besides.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum StyleOutlook {
    /// A real face resolves **and the retry will reach it**: one press gets a
    /// genuine typeface.
    ///
    /// `set_synthetic` is refused *because* a real face is available, and
    /// `crate::app::actions::textstyle` retries with the face the refusal names
    /// — so the two-verb path completes. The string is the human `/BaseFont`,
    /// shortened the way the face chooser shortens it, because it is going into
    /// a sentence a person reads.
    RealFace(String),
    /// A real face resolves and **`set_font` would refuse it for this run**, so
    /// neither verb reaches the weight and the press will be declined.
    ///
    /// ★★★ The shipped engine defect `crate::app::actions::textstyle`'s header
    /// retracts a claim over: `gate_synthesis` prefers a face by *family* and
    /// gates synthesis off, and the face it names may not map every character
    /// in the run. Reproduced on pdfcer's `textedit/format_family.pdf`,
    /// confirmed, and a fix is queued — which is precisely why this is a
    /// **sentence in the hover and not a greyed button**: when the fix lands
    /// this case becomes [`Self::Synthesized`] on its own, where a withheld
    /// control would stay withheld until somebody noticed.
    FaceCannotCover(String),
    /// No real face covers what was asked, so the letters will be thickened or
    /// slanted and the engine will say so afterwards.
    Synthesized,
}

// ★★★ `FaceChoice` was DEFINED HERE until 2026-08-29 and now lives in
// [`super::face`], beside the popup body that draws it and the [`FaceOrigin`]
// tag that says which of two acts a row performs.
//
// It moved because the control did. `Pass 162.0` let pdfcer restyle text to a
// face the document does NOT contain, so a row in this list is now either a
// `Tf` rewrite or a `Tf` rewrite plus a new `/Font` object in the operator's
// file — and a chooser presenting those as one list would hide a write behind a
// menu. The two surfaces that draw it (this section and the ribbon's Format ▸
// Font group) were already two copies of one loop; they are now one.

use super::face::{FaceChoice, FaceOrigin};

impl TextStyleDraft {
    /// Re-read from the document when the stamp has moved; otherwise keep what
    /// is on screen.
    ///
    /// Returns `true` when there is something to draw — i.e. the run resolved.
    pub(crate) fn sync(&mut self, doc: &OpenDoc, page: usize, run: usize) -> bool {
        let stamp = (page, run, doc.edit_epoch);
        if self.stamp == Some(stamp) {
            return self.face.is_some();
        }
        self.stamp = Some(stamp);
        self.face = None;
        self.size = 0.0;
        self.colour = None;
        self.bold_outlook = None;
        self.italic_outlook = None;

        // ★ The expensive call, made exactly here and nowhere else in this
        // module. See the module header on the 392 ms.
        let Some(read) = crate::canvas::textedit::pin::inspect(doc, page, run) else {
            return false;
        };
        self.size = f64::from(read.style.size);
        self.typed_size = self.size;
        // ★ The join. `GlyphProvenance` records the RESOURCE KEY the content
        // stream used — `F1` — and an operator needs the `/BaseFont`. The
        // document's font inventory is the only place both appear, so this is
        // the one hop that turns a machine name into a human one.
        //
        // `None` when the key resolves to nothing, which is a real state on a
        // malformed page and is shown as such rather than as a blank combo.
        self.face = read.style.font_resource.as_ref().and_then(|key| {
            doc.font_inventory()
                .fonts
                .iter()
                .find(|record| record.resource_names.iter().any(|name| name == key))
                .and_then(|record| record.base_font.clone())
        });
        self.colour = read.style.fill.and_then(rgb_of);
        // ★★ The pre-flight, in the same stamped read as everything else. It
        // costs a second extraction, which is why it is here and not in the
        // chooser: a combo is drawn every frame it is open.
        //
        // ★★★ The list is built by [`super::face::choices`] and is **two lists
        // joined**, since 2026-08-29: the page's own accepted resources, and the
        // standard-14 faces pdfcer would author on demand (`Pass 162.0`). That
        // function's doc comment carries the whole of why the second half is
        // filtered on the pre-flight's `entries` rather than on `accepted()`,
        // and why the fourteen are offered without being coverage-tested first.
        //
        // ★ `accepted()` for the page half. A refused entry is deliberately
        // **not** offered greyed-with-a-reason, though the engine hands us the
        // reason: the refusals are per-character encoding facts (*"'o' has no
        // code in Times-Bold's encoding"*), and a list of twelve faces with nine
        // greyed rows each carrying a sentence about a character is a control an
        // operator cannot read. R9's absent-rather-than-greyed case: for THIS
        // run those faces are not a capability that is temporarily unavailable,
        // they are not applicable.
        self.faces = super::face::choices(
            crate::canvas::textedit::pin::font_preflight(doc, page, &read).as_ref(),
        );
        // ★★★ **What the two weight buttons would do**, asked here for the
        // reason everything else in this function is asked here: it costs a
        // content-stream read and a plan, and the answer changes only when the
        // stamp does. Twice — once per axis — because the two buttons issue two
        // separate single-axis requests; see `Self::italic_outlook`.
        //
        // ★★ AFTER `self.faces`, and the order is load-bearing rather than
        // incidental: `Self::outlook` joins the preview's `selector` against
        // the pre-flight's accepted list to tell `StyleOutlook::RealFace` from
        // `StyleOutlook::FaceCannotCover`, so the pre-flight has to be in hand
        // before the probes are read.
        self.bold_outlook = self.outlook(doc, page, &read, true);
        self.italic_outlook = self.outlook(doc, page, &read, false);
        self.face.is_some()
    }

    /// One axis's [`StyleOutlook`], or `None` when the probe did not answer.
    ///
    /// # ★★★ The join, and why it is not a rule re-implemented here
    ///
    /// `preview_style_resolution` says whether a real face resolves and, when
    /// one does, hands back **the string to pass to `set_font` to reach it**.
    /// `preview_font_resources` says which resources `set_font` would accept
    /// **for this run's characters**, each with the same kind of string. Both
    /// are the engine's own selectors, issued by the engine for the same
    /// purpose, and comparing them is a string equality rather than a second
    /// implementation of the family heuristic or of the per-glyph coverage
    /// test.
    ///
    /// ⇒ That distinction is the whole licence for this function.
    /// `StyleResolution`'s invariant forbids `pdfcer-gui` re-deriving
    /// `family_stem`, `name_claims_bold` or `name_claims_italic`, and nothing
    /// here does: the shell asks two questions and notices when the answers
    /// disagree.
    ///
    /// ★ `combined` rather than the per-axis probe, even though the request is
    /// single-axis. For a one-axis request the two are the same answer by
    /// construction — `combined` **is** `set_synthetic(want)`'s verdict, which
    /// is what the button will do — and reading the field that describes the
    /// act rather than the field that describes an axis is what keeps this
    /// right if a caller ever asks for both at once.
    fn outlook(
        &self,
        doc: &OpenDoc,
        page: usize,
        read: &crate::canvas::textedit::pin::Inspected,
        bold: bool,
    ) -> Option<StyleOutlook> {
        use pdfcer_core::text_edit::{StyleOutcome, StyleSynthesis};
        let want = StyleSynthesis::new(bold, !bold);
        let resolution = doc
            .session
            .preview_style_resolution(page, "", Some(read.pin.span), want)
            .ok()?;
        match resolution.combined? {
            StyleOutcome::WouldSynthesize => Some(StyleOutlook::Synthesized),
            StyleOutcome::RealFaceResolves {
                real_font,
                selector,
                ..
            } => {
                let label = shorten(&real_font).to_owned();
                // ★★★ **The page's own faces only**, and the filter is not
                // cosmetic. `gate_synthesis` names a real face it found by
                // surveying THIS PAGE's resources, so the question this join
                // asks is *"is the face it named one `set_font` would accept for
                // this run?"* — and the answer has to be looked for in the same
                // population the engine looked in.
                //
                // Since 2026-08-29 `self.faces` also carries the fourteen
                // standard faces pdfcer would ADD, which are by construction not
                // on the page. Matching against those would answer
                // `StyleOutlook::RealFace` — *"this page carries Helvetica-Bold,
                // so pdfcer will use that real typeface"* — about a page that
                // carries no such thing, and the press would then be refused
                // exactly as `FaceCannotCover` predicted. A hover made confidently
                // wrong by a list gaining rows for an unrelated reason is the
                // shape of defect this project spends its time on.
                if self
                    .faces
                    .iter()
                    .filter(|face| face.origin == FaceOrigin::OnThisPage)
                    .any(|face| face.selector == selector)
                {
                    Some(StyleOutlook::RealFace(label))
                } else {
                    Some(StyleOutlook::FaceCannotCover(label))
                }
            }
            // ★ A named catch-all rather than a fall-through, because
            // `StyleOutcome` is `#[non_exhaustive]`: a variant the engine adds
            // later must land somewhere honest, and "nothing is known" is the
            // one answer that is true of an outcome this build has never seen.
            // It renders the conditional hint, which is what was said before
            // any of this existed.
            _ => None,
        }
    }

    // -----------------------------------------------------------------------
    // Accessors, added 2026-08-27 when the Format ▸ Font group shipped.
    //
    // ★★ **One draft, two surfaces, and that is the whole reason these exist.**
    //
    // The ribbon's Font group and this panel's *This text* section show the
    // same four values and both need them read back from the document. The
    // read costs 392 ms on the operator's benchmark sheet — a text extraction
    // with provenance capture on — so a second draft would double it on every
    // selection change, on exactly the drawings this program is for.
    //
    // So `PanelsState` owns the one draft, `app::surfaces` borrows it for the
    // ribbon and `panels::properties` borrows it for the panel, and whichever
    // draws first in the frame pays for the read while the other gets a stamp
    // hit. These accessors are what let the ribbon read it without the fields
    // becoming public and without `app::fontband` learning how the stamp works.
    // -----------------------------------------------------------------------

    /// The run's `/BaseFont`, subset tag and all, or `None` when the
    /// provenance carried no font resource.
    #[must_use]
    pub(crate) fn face(&self) -> Option<&str> {
        self.face.as_deref()
    }

    /// The `Tf` size the document currently holds, in points.
    ///
    /// ★ Not [`Self::typed_size`]. This is what was **read**; that is what the
    /// operator is **typing**, and the difference between them is what decides
    /// whether a release is an edit or a no-op.
    #[must_use]
    pub(crate) fn size(&self) -> f64 {
        self.size
    }

    /// The size the operator is typing.
    #[must_use]
    pub(crate) fn typed_size(&self) -> f64 {
        self.typed_size
    }

    /// The size the operator is typing, for a widget to write into.
    pub(crate) fn typed_size_mut(&mut self) -> &mut f64 {
        &mut self.typed_size
    }

    /// The fill colour as sRGB bytes, or `None` when the run is painted in a
    /// space this control cannot round-trip. See [`rgb_of`].
    #[must_use]
    pub(crate) fn colour(&self) -> Option<[u8; 3]> {
        self.colour
    }

    /// The faces `set_font` would accept for this run, in the order the engine
    /// listed the page's resources.
    ///
    /// **Empty means no face on this page can show this run's characters** —
    /// a real state, and one a chooser must render as a sentence rather than as
    /// an empty list. It is not the same as "the pre-flight was not read":
    /// [`Self::sync`] clears it on every re-read and fills it from the engine.
    #[must_use]
    pub(crate) fn faces(&self) -> &[FaceChoice] {
        &self.faces
    }

    /// What pressing **Bold** would do to this run, or `None` when the probe
    /// did not answer.
    #[must_use]
    pub(crate) fn bold_outlook(&self) -> Option<&StyleOutlook> {
        self.bold_outlook.as_ref()
    }

    /// What pressing **Italic** would do to this run, or `None` when the probe
    /// did not answer.
    #[must_use]
    pub(crate) fn italic_outlook(&self) -> Option<&StyleOutlook> {
        self.italic_outlook.as_ref()
    }
}

/// Draw the section, or nothing.
///
/// Returns whether it drew, so [`super::body_sections`] knows the panel is
/// already saying something about a selection.
pub fn section(
    ui: &mut Ui,
    doc: &OpenDoc,
    draft: &mut TextStyleDraft,
    actions: &mut Vec<Action>,
) -> bool {
    // ★ The staleness gate is inside `runs`, not here — a stale run ordinal
    // restyles the WRONG text, so the check lives with the data rather than
    // with each of its readers.
    // ★ `false`, not a sentence. The object-selection state belongs to
    // [`super::textobject`] since 2026-09-05 — see the block below where
    // `route` used to be.
    let Some(selection) = doc.text_selection.as_ref() else {
        return false;
    };
    let runs = selection.runs(doc.edit_epoch);
    let Some(&first) = runs.first() else {
        return false;
    };
    let page = selection.page;

    if !draft.sync(doc, page, first) {
        // The selection is real and the run would not pin. Saying nothing here
        // would be the "control that is silently missing" defect: the operator
        // has text selected and the section they saw last time is gone. One
        // sentence, no controls.
        ui.heading(t::text_heading());
        ui.label(t::text_unreadable());
        crate::diag::ui_rect_visible(REGION, ui.min_rect(), ui.clip_rect());
        return true;
    }

    ui.heading(t::text_heading());
    ui.label(t::text_covers(runs.len()));

    face_row(ui, doc, draft, page, &runs, actions);
    size_row(ui, draft, page, &runs, actions);
    weight_row(ui, draft, page, &runs, actions);
    colour_row(ui, draft, page, &runs, actions);

    crate::diag::ui_rect_visible(REGION, ui.min_rect(), ui.clip_rect());
    ui.separator();
    true
}

// ===========================================================================
// ★★★ `route` LIVED HERE UNTIL 2026-09-05, AND IT WAS ONLY HALF AN ANSWER
//
// It drew one sentence — *"press T for the Text tool and sweep across them"* —
// whenever a text OBJECT was selected and nothing had been swept, and it was
// `OPERATOR_REQUESTS.md` O89's second candidate, *"the Properties panel naming
// the missing step where the swatch would be."* Built 2026-08-29, correct, and
// still not what he asked for: he wanted the colour, and the panel told him
// where to go and get it.
//
// It has moved WHOLE — sentence, region name, `object_kind` gate and the
// one-object rule — into [`super::textobject`], which draws a **working colour
// control** for the clicked object and keeps the sentence underneath it for the
// four properties that genuinely still need the sweep. Moved rather than
// duplicated: two sections that both claim the object-selection state would
// draw two headings, and the argument for the `object_kind` gate is worth more
// than a retyped copy of the code it justifies.
//
// ⇒ This section is now exactly what its own header always said it was: the
// editor for a **swept range**. With nothing swept it returns `false` and says
// nothing, and `super::textobject` speaks instead.
// ===========================================================================

/// The face: what this page carries, and what pdfcer can add to the document.
///
/// # ★★★ The list was the page's fonts and no longer only is
///
/// This doc comment used to open *"`set_font` **selects** an existing resource;
/// it does not **create** one. Offering Helvetica on a page that carries only
/// Arial would produce a refusal on press."* That was true, it was the reason
/// the chooser only ever offered what the page already had, and `Pass 162.0`
/// ended it: pdfcer now authors a standard-14 `/Font` resource on demand, so
/// Helvetica on a page built from Arial is a change that works rather than a
/// refusal.
///
/// ★ The old sentence is kept above rather than deleted because the *rule* it
/// states has not changed — a chooser must not offer entries that cannot work —
/// only the set of entries that can. `super::face::choices` is where that set is
/// computed and it carries the argument.
///
/// # What is drawn here, and what is not
///
/// This function owns the **row**: the label, the combo, the current face and
/// the region. The **popup body** — two groups, their headings, the disclosure
/// the standard-14 half owes and every clickable row — is
/// [`super::face::popup_body`], shared verbatim with the ribbon's Format ▸ Font
/// chooser in [`crate::app::fontband`].
///
/// ★★ Shared rather than copied, and that is the change this project keeps
/// having to make: the two were two copies of one loop, and *"a face offered in
/// one surface and not the other"* is the divergence found here more than once.
/// A disclosure added to one copy and not the other would be worse than either.
fn face_row(
    ui: &mut Ui,
    doc: &OpenDoc,
    draft: &TextStyleDraft,
    page: usize,
    runs: &[usize],
    actions: &mut Vec<Action>,
) {
    let current = draft.face.clone().unwrap_or_default();
    let _ = doc;
    ui.horizontal(|ui| {
        ui.label(crate::text::panels::face::text_face_label());
        let mut chosen = None;
        let combo = egui::ComboBox::from_id_salt("properties-text-face")
            .selected_text(shorten(&current))
            .show_ui(ui, |ui| {
                chosen = super::face::popup_body(ui, FACE_REGION, draft.faces(), shorten(&current));
            });
        crate::diag::ui_rect_visible(FACE_REGION, combo.response.rect, ui.clip_rect());
        // ★ The action is raised HERE, outside the popup closure, because
        // nothing mutates from a widget — `app::actions`' founding invariant.
        // The closure reports which row was pressed and this row turns that into
        // one `Action`, exactly as the ribbon's copy turns it into one parked
        // `StyleChange`.
        if let Some(selector) = chosen {
            actions.push(Action::TextStyle {
                page,
                runs: runs.to_vec(),
                change: StyleChange::Face(selector),
            });
        }
    });
}

/// The size, in points.
///
/// ★ Committed on `drag_stopped` or `lost_focus`, never on `.changed()`. Each
/// commit is a content-stream rewrite and an undo entry, so a drag across the
/// spinner would author one edit per pixel — the same rule
/// [`super::markup`]'s width and opacity rows follow, for the same reason.
fn size_row(
    ui: &mut Ui,
    draft: &mut TextStyleDraft,
    page: usize,
    runs: &[usize],
    actions: &mut Vec<Action>,
) {
    ui.horizontal(|ui| {
        ui.label(t::text_size_label());
        let response = ui.add(
            egui::DragValue::new(&mut draft.typed_size)
                .speed(0.25)
                .range(1.0..=1440.0)
                .suffix(t::text_size_suffix()),
        );
        crate::diag::ui_rect_visible(SIZE_REGION, response.rect, ui.clip_rect());
        if (response.drag_stopped() || response.lost_focus())
            && (draft.typed_size - draft.size).abs() > f64::EPSILON
        {
            actions.push(Action::TextStyle {
                page,
                runs: runs.to_vec(),
                change: StyleChange::Size(draft.typed_size),
            });
        }
    });
}

/// Bold and Italic — buttons that apply, not switches that reflect.
///
/// See the module header: there is no "is this run bold" bit in a PDF, so a
/// pressed-in toggle would claim to have read a fact that is not recorded.
/// Neither is ever greyed; the engine's two verbs cover every page between
/// them.
fn weight_row(
    ui: &mut Ui,
    draft: &TextStyleDraft,
    page: usize,
    runs: &[usize],
    actions: &mut Vec<Action>,
) {
    ui.horizontal(|ui| {
        ui.label(t::text_weight_label());
        let bold = ui.button(t::text_bold());
        crate::diag::ui_rect_visible(BOLD_REGION, bold.rect, ui.clip_rect());
        if bold.on_hover_text(bold_hint(draft)).clicked() {
            actions.push(Action::TextStyle {
                page,
                runs: runs.to_vec(),
                change: StyleChange::Weight {
                    bold: true,
                    italic: false,
                },
            });
        }
        let italic = ui.button(t::text_italic());
        // ★ Published, since 2026-08-29, and it was the only control in this
        // section without a region. The two buttons now carry *different*
        // sentences derived from two separate engine probes, so a driven check
        // that could point at Bold and not at Italic could assert half the
        // feature — and half a per-axis answer is exactly the case the two
        // probes exist to separate.
        crate::diag::ui_rect_visible(ITALIC_REGION, italic.rect, ui.clip_rect());
        if italic.on_hover_text(italic_hint(draft)).clicked() {
            actions.push(Action::TextStyle {
                page,
                runs: runs.to_vec(),
                change: StyleChange::Weight {
                    bold: false,
                    italic: true,
                },
            });
        }
    });
}

/// The Bold button's hover text, given what the engine says would happen.
///
/// # ★★★ Four sentences, and the fourth is the one that was there before
///
/// | outlook | what the operator reads |
/// |---|---|
/// | [`StyleOutlook::RealFace`] | *this page carries **Arial-Bold**, so pdfcer will use that real typeface* |
/// | [`StyleOutlook::Synthesized`] | *no real bold face covers this text, so pdfcer will thicken the letters* |
/// | [`StyleOutlook::FaceCannotCover`] | *bold is not available for this text*, naming the face and the reason |
/// | `None` — the probe did not answer | the conditional hint, unchanged |
///
/// The fourth row is not a fallback that should have been designed away. A
/// probe returns `None` for a page whose content cannot be planned, for an
/// `#[non_exhaustive]` outcome this build has never seen, and for an encrypted
/// document — and in every one of those the honest thing to say is the
/// mechanism rather than a prediction. That is exactly what
/// [`crate::text::panels::properties::text_bold_hint`] already said, which is
/// why it stays.
///
/// # ★★ None of the four greys the button, and that is the engine's ruling
///
/// *"Do not grey out a bold button. Offer it, and surface the disclosure when
/// synthesis fires."* The third row is the one where greying could now be
/// argued — the shell can predict that refusal for the first time, because
/// `preview_font_resources` runs the per-run coverage test the old argument
/// said it could not — and it is still a sentence, because the engine has a
/// **queued fix** that turns that case into ordinary synthesis. A control
/// withheld on the strength of a defect that is about to be fixed is a control
/// that stays withheld for months; a sentence that goes stale is read once and
/// corrected in one line.
fn bold_hint(draft: &TextStyleDraft) -> String {
    match draft.bold_outlook() {
        Some(StyleOutlook::RealFace(face)) => t::text_bold_hint_real_face(face),
        Some(StyleOutlook::FaceCannotCover(face)) => t::text_bold_hint_face_cannot_cover(face),
        Some(StyleOutlook::Synthesized) => t::text_bold_hint_synthetic().to_owned(),
        None => t::text_bold_hint().to_owned(),
    }
}

/// The Italic button's hover text. See [`bold_hint`] for the whole argument;
/// this is the same four rows with *slant* in place of *thicken*, from the
/// draft's separately-probed italic axis.
///
/// ★ It reads [`TextStyleDraft::italic_outlook`] and never the bold one. The
/// two are genuinely different answers on an ordinary page — one holding a real
/// `Arial-Bold` and no `Arial-Italic` gives `RealFace` for one button and
/// `Synthesized` for the other — and a shared sentence would be wrong on
/// exactly the pages an operator is most likely to be working on.
fn italic_hint(draft: &TextStyleDraft) -> String {
    match draft.italic_outlook() {
        Some(StyleOutlook::RealFace(face)) => t::text_italic_hint_real_face(face),
        Some(StyleOutlook::FaceCannotCover(face)) => t::text_italic_hint_face_cannot_cover(face),
        Some(StyleOutlook::Synthesized) => t::text_italic_hint_synthetic().to_owned(),
        None => t::text_italic_hint().to_owned(),
    }
}

/// The fill colour.
///
/// ★ `None` renders a sentence, not a swatch. A run painted in DeviceCMYK, a
/// Separation or an ICC space has no faithful `[u8; 3]`, and a swatch showing
/// its nearest RGB would write that RGB back on the next press — converting
/// the operator's ink without being asked. `pdfcer-core` deliberately does not
/// force-convert to DeviceRGB the way Acrobat does, and this control must not
/// undo that on its behalf.
fn colour_row(
    ui: &mut Ui,
    draft: &TextStyleDraft,
    page: usize,
    runs: &[usize],
    actions: &mut Vec<Action>,
) {
    ui.horizontal(|ui| {
        ui.label(t::text_colour_label());
        let Some(current) = draft.colour else {
            ui.label(t::text_colour_not_plain());
            return;
        };
        let mut rgb = current;
        if ui.color_edit_button_srgb(&mut rgb).changed() && rgb != current {
            let components = vec![
                f64::from(rgb[0]) / 255.0,
                f64::from(rgb[1]) / 255.0,
                f64::from(rgb[2]) / 255.0,
            ];
            if let Ok(fill) = pdfcer_core::text_edit::NewFill::new(
                pdfcer_core::text_edit::FillModel::Rgb,
                components,
            ) {
                actions.push(Action::TextStyle {
                    page,
                    runs: runs.to_vec(),
                    change: StyleChange::Fill(fill),
                });
            }
        }
    });
}

// ★★★ `faces_on_page` was HERE until 2026-08-27, and it is **deleted rather
// than kept**, which is this project's standing rule the moment a workaround's
// cause is removed: a mechanism with no caller rots, and the next reader cannot
// tell a deliberate fallback from a forgotten one.
//
// It built the face chooser's list from `fontinfo::FontInventory`, filtered to
// the records naming this page, and its own doc comment named the flaw
// honestly: *"this is a superset that is usually exactly right, and when it is
// not, the press earns a named refusal rather than silence. A proper pre-flight
// is filed with the engine as `Pass 142.1`."*
//
// `Pass 142.1` shipped, the same night it was asked for.
// `canvas::textedit::pin::font_preflight` is the replacement and it closes
// **two** holes, not one: the entries that could not work (offered, pressed,
// refused), and — much worse — the page carrying two subsets of one
// `/BaseFont`, where a name match reached one of the twins arbitrarily and
// applied the wrong font with no refusal to show for it. The Fonts panel's own
// survey puts that at 87 % of embedding files.
//
// ★ The filed request said the superset was "usually right". It was, and
// "usually right" about which font is applied to an operator's drawing is not
// a standard this program should have been holding itself to. The pre-flight is
// not an optimisation of it; it is the correct answer where that was a guess.

/// A `/BaseFont` without its §9.6.4 subset tag, for display only.
///
/// ★ Display only, and the distinction matters: the **value** pushed on the
/// action is the full name, because `set_font` accepts either and handing it
/// the full one keeps the shell from having to know the stripping rule. What
/// an operator gains from `ABCDEF+ArialMT` being shown as `ArialMT` is the
/// ability to read the list at all.
pub(crate) fn shorten(base_font: &str) -> &str {
    match base_font.split_once('+') {
        // A subset tag is exactly six uppercase letters (§9.6.4). Anything else
        // before a `+` is part of the name and is kept — `Foo+Bar` is a legal,
        // if unusual, font name.
        Some((tag, rest)) if tag.len() == 6 && tag.bytes().all(|b| b.is_ascii_uppercase()) => rest,
        _ => base_font,
    }
}

/// sRGB bytes for a fill colour, or `None` when the space cannot round-trip.
///
/// # ★★ Why CMYK is `None` rather than converted
///
/// A conversion here would be a **one-way** trip the operator never asked for.
/// The swatch would show DeviceCMYK ink as its nearest RGB; the next press
/// would write that RGB back through `set_fill`; and the run would leave its
/// original space for ever, on a document heading for a printer that cares.
///
/// `pdfcer-core` deliberately does not force-convert to DeviceRGB the way
/// Acrobat does — it stores the space the caller chose — and a control that
/// undid that on the operator's behalf would make the engine's care pointless.
/// Gray round-trips exactly, so it is offered.
///
/// # ★★★ `pub(super)` since 2026-09-05, and the widening is the point
///
/// [`super::textobject`] asks the same question about a **whole text object**,
/// and it asks it **here** rather than deciding for itself which spaces are
/// safe. This function IS the spot-ink guard for text: it is what makes a
/// `TextColor::Other` or a `TextColor::Cmyk` produce no swatch, on either
/// surface.
///
/// A second copy would be two answers to *"may pdfcer overwrite this ink with a
/// screen colour?"*, and the two would drift the first time a space was added
/// to the safe list — with the drift showing up as a colour picker opening over
/// a `/Separation` on one surface and not the other. There is one answer, in one
/// place, with one doc comment stating why.
pub(super) fn rgb_of(colour: pdfcer_core::text_extract::TextColor) -> Option<[u8; 3]> {
    use pdfcer_core::text_extract::TextColor;
    let byte = |v: f32| (v.clamp(0.0, 1.0) * 255.0).round() as u8;
    match colour {
        TextColor::Rgb(r, g, b) => Some([byte(r), byte(g), byte(b)]),
        // DeviceGray IS a subset of DeviceRGB with r == g == b, so this is a
        // faithful reading. The write-back is a widening, disclosed by the
        // engine, and the operator asked for a colour.
        TextColor::Gray(v) => Some([byte(v), byte(v), byte(v)]),
        TextColor::Cmyk(..) | TextColor::Other => None,
        // `TextColor` is `#[non_exhaustive]`: a space added later is unknown,
        // and unknown means do not guess.
        _ => None,
    }
}

#[cfg(test)]
mod outlook_tests {
    use super::*;

    /// A draft carrying the two outlooks and nothing else.
    ///
    /// A constructor rather than three field assignments after
    /// `Default::default()`, which is what clippy's `field_reassign_with_default`
    /// asks for and is better here anyway: what these tests vary is the pair of
    /// outlooks, and a helper that takes exactly the pair says so.
    fn drafted(bold: Option<StyleOutlook>, italic: Option<StyleOutlook>) -> TextStyleDraft {
        TextStyleDraft {
            bold_outlook: bold,
            italic_outlook: italic,
            ..Default::default()
        }
    }

    /// ★★★ **Four outlooks, four different sentences**, per axis.
    ///
    /// If any two collapsed, the probe would be decoration: an operator whose
    /// page carries `Arial-Bold` and one whose page carries nothing would read
    /// the same words and learn nothing either way. The `None` row is included
    /// deliberately — it is the sentence that was there before
    /// `preview_style_resolution` was consumed, and it must remain
    /// distinguishable from the three predictions rather than being absorbed
    /// into one of them.
    #[test]
    fn every_outlook_earns_its_own_sentence() {
        let mut seen: Vec<String> = Vec::new();
        for outlook in [
            None,
            Some(StyleOutlook::RealFace("Arial-Bold".to_owned())),
            Some(StyleOutlook::FaceCannotCover("Times-Bold".to_owned())),
            Some(StyleOutlook::Synthesized),
        ] {
            let draft = drafted(outlook, None);
            let line = bold_hint(&draft);
            assert!(
                !seen.contains(&line),
                "two outlooks produced the same hover text: {line}"
            );
            seen.push(line);
        }
    }

    /// ★★ **The two axes read their own probes**, and never each other's.
    ///
    /// The state this pins is the ordinary one, not an exotic one: a page
    /// carrying a real `Arial-Bold` and no `Arial-Italic` gives `RealFace` for
    /// one button and `Synthesized` for the other. A shared sentence — or a
    /// copy-paste that read `bold_outlook` in both helpers — would be wrong on
    /// exactly the pages an operator is most likely to be working on, and would
    /// be invisible on every page where the two answers happen to agree.
    #[test]
    fn the_two_buttons_do_not_borrow_each_others_answer() {
        let draft = drafted(
            Some(StyleOutlook::RealFace("Arial-Bold".to_owned())),
            Some(StyleOutlook::Synthesized),
        );
        assert!(bold_hint(&draft).contains("Arial-Bold"));
        assert!(!italic_hint(&draft).contains("Arial-Bold"));
        assert!(italic_hint(&draft).contains("slant"));
    }

    /// ★ **Bold thickens and italic slants**, and neither sentence borrows the
    /// other's verb.
    ///
    /// They are different synthetic operations — a weight is the regular face
    /// stroked, a slant is the upright face sheared — and an operator who has
    /// read one should not have to guess that the other means something else.
    #[test]
    fn the_synthetic_sentences_name_the_right_operation() {
        let draft = drafted(
            Some(StyleOutlook::Synthesized),
            Some(StyleOutlook::Synthesized),
        );
        let bold = bold_hint(&draft);
        let italic = italic_hint(&draft);
        assert!(bold.contains("thicken"), "{bold}");
        assert!(!bold.contains("slant"), "{bold}");
        assert!(italic.contains("slant"), "{italic}");
        assert!(!italic.contains("thicken"), "{italic}");
    }

    /// ★★★ **The unreachable case names the face and says the press will be
    /// refused**, which is the whole point of predicting it.
    ///
    /// `crate::app::actions::textstyle`'s header retracts the claim that the
    /// two verbs cover every page: `gate_synthesis` prefers a face by *family*
    /// and gates synthesis off, and the face it names may map none of the run's
    /// characters. Before this, the operator pressed Bold, watched nothing
    /// happen, and read a font name in the status bar afterwards.
    ///
    /// ★ It must NOT tell them to pick a different font. The face chooser is
    /// two rows up and would often work, but naming that remedy would be this
    /// shell second-guessing pdfcer's font selection — decision 058's exact
    /// case. Saying what will happen is the honest half.
    #[test]
    fn the_unreachable_case_names_the_face_and_not_a_remedy() {
        let draft = drafted(
            Some(StyleOutlook::FaceCannotCover("Times-Bold".to_owned())),
            None,
        );
        let line = bold_hint(&draft);
        assert!(line.contains("Times-Bold"), "{line}");
        assert!(line.contains("refused"), "{line}");
        assert!(
            !line.to_lowercase().contains("choose another"),
            "the sentence must not prescribe a font: {line}"
        );
    }

    /// ★★ **A fresh draft says the conditional**, not a prediction.
    ///
    /// `TextStyleDraft::default()` has never been synced, so both outlooks are
    /// `None` — and the honest thing to say about a run nothing has been read
    /// from is the mechanism, which is exactly what the hint said before any of
    /// this landed. A build that guessed `Synthesized` here would tell an
    /// operator their letters are about to be thickened on a page that carries
    /// a real bold face.
    #[test]
    fn an_unsynced_draft_promises_nothing() {
        let draft = drafted(None, None);
        assert_eq!(bold_hint(&draft), t::text_bold_hint());
        assert_eq!(italic_hint(&draft), t::text_italic_hint());
    }
}
