//! # `panels::properties::markup` — restyling a markup that is already on the
//! page
//!
//! ## What this closes
//!
//! `FEATURES.md`'s Phase 1 row *"Format tab contents — colour, width, style,
//! opacity for a **placed** markup"*, and the row `pdfcer`'s own capability
//! register carried as ⬜ with a note this project wrote:
//!
//! > **`set_markup_style` shipped in the engine on 2026-08-18 and has zero GUI
//! > callers.** It appears only in doc comments.
//!
//! It was, until this landed, **the largest engine capability with no route
//! from this GUI**. Both blockers `shell::manifest::format`'s header recorded
//! are discharged — the verb landed 2026-08-18, and annotations became
//! selectable the same day — so what remained was work rather than a block, and
//! the operator's instruction of 2026-08-19 was to do the work.
//!
//! ## ★ Why the PANEL and not the Format tab
//!
//! `RIBBON_IA.md` §5.8 settles it and the wording is the operator's own
//! decision of 2026-08-12:
//!
//! > The division of labour: the **tab** carries what a user changes *while
//! > working* — colour, width, style, align, delete. The **panel** carries
//! > everything, including the read-only facts … The panel is also where the
//! > **editable geometry** lives.
//!
//! So the panel is where the complete set goes, and it is also the cheaper
//! surface by a wide margin: a ribbon band cannot hold a colour picker or a
//! slider without a new `Item::custom` kind and a renderer for it, which is
//! shell work in a crate that must never learn what a PDF is. The tab's slice
//! is a later, smaller job that reads the same actions.
//!
//! ## ★★ Every control is `None` unless the operator touched it
//!
//! `MarkupStyle`'s own doc comment is the rule and the reason:
//!
//! > Every field is `None` by default … That shape is deliberate: a Format tab
//! > whose colour picker also had to restate the current width would overwrite
//! > whatever the operator had set from the other control.
//!
//! So this section raises **one action per control that changed**, carrying one
//! field, and never a whole style struct assembled from what the widgets happen
//! to show. The failure that prevents is specific: two controls drawn from the
//! same annotation, one of them stale by a frame, and a colour change that
//! silently reverts a width the operator set a moment earlier.
//!
//! ## ★★★ Where the style verb cannot reach — the defect of 2026-09-06
//!
//! **This section used to draw live controls that could not commit.** The guard
//! was `AnnotKind::Markup` plus the locked flag, and `AnnotKind::Markup`'s own
//! doc says what it covers: *"a shape, a note, a stamp, a text markup"*. But
//! `set_markup_style` begins by calling `annot_author::spec_from_dict`, and that
//! function's `match` reads exactly ten `/Subtype`s — `Square`, `Circle`,
//! `Line`, `Ink`, `Polygon`, `PolyLine`, `Highlight`, `Underline`, `StrikeOut`,
//! `Squiggly` — with every other name falling to an `other =>` arm that answers
//! `SpecReadError::UnsupportedSubtype`. **Verified by reading the engine source
//! on 2026-09-06, not inferred.**
//!
//! So `/Text` (a sticky note), `/FreeText` (a text box) and `/Stamp` were
//! selectable, drew a live colour swatch and a live opacity spinner, and every
//! press was refused with `EditError::MarkupSpec`. That is the *visible
//! control, silently inert* class this project forbids by name.
//!
//! ### ★★ Reachability is asked of `spec_from_dict`, never of a subtype list
//!
//! The obvious fix — a `matches!(subtype, "Square" | "Circle" | …)` beside the
//! kind check — is the fix that goes stale the day the engine learns an
//! eleventh subtype, and it goes stale **silently**, in the direction that
//! withholds a control that would have worked. [`Current::read`] already called
//! `spec_from_dict`; all that was missing was carrying its verdict forward.
//! [`Current::restylable`] is that verdict, so this section and the verb are
//! answering the same question through the same function, and an engine that
//! grows a subtype grows this panel with it and no shell change.
//!
//! ### ★ The refusal SAYS something
//!
//! R9 makes an unavailable capability render nothing. It does not make the
//! panel go silent: the heading and the subtype line still draw, because
//! something *is* selected, and a heading over an empty space reads as a bug.
//! [`t::markup_not_restylable`] names what is still possible — move, resize,
//! delete, edit the note — and its doc comment records the engine verb each of
//! those four claims was checked against.
//!
//! ## ★★★ What WAS deliberately absent, and the two arguments that were wrong
//!
//! This header shipped on 2026-08-19 with three refusals written into it. On
//! 2026-09-06 the operator asked for **full editing of the markup tools**, and
//! two of the three did not survive contact with that. They are kept here with
//! the correction beside each rather than deleted, because a header that
//! quietly loses an argument teaches the next reader nothing — and because the
//! surviving half of the first one is still load-bearing elsewhere.
//!
//! - **Fill (`/IC`) — WAS refused, and is now offered on restyle.** What this
//!   header used to say:
//!
//!   > `canvas::markup::spec` authors `interior: None` on purpose — *"a filled
//!   > comment shape hides the drawing it is a comment about, which on a CAD
//!   > sheet is the whole content under it"* — and `NO_SURFACE.md` records that
//!   > reversing it is the operator's call, not this module's. A control here
//!   > would make the decision by offering it.
//!
//!   ★ **The author-time half of that stands and is untouched.**
//!   `canvas::markup::spec` still writes `interior: None`, and this module does
//!   not go near it: a shape this shell *places* is still unfilled, and still
//!   does not hide the drawing under it. What the argument never justified is
//!   the second thing it was being used for — refusing to fill a shape the
//!   operator has **already placed** and is looking at right now. A default and
//!   a prohibition are different acts, and letting one stand in for the other
//!   is how a sensible default becomes a capability nobody can reach. Acrobat's
//!   shape tools all offer fill and all default it to none; that is the shape
//!   matched here. **No fill at author time, fill available on restyle**, and
//!   the difference between the two is the whole point.
//!
//! - **Line endings (`/LE`) — WAS refused, and is now offered on a `/Line`.**
//!   What this header used to say:
//!
//!   > They are meaningful for `/Line` alone, and the one `/Line` an operator
//!   > of this application places is an arrow whose endings are what makes it
//!   > an arrow. A control that could turn an arrow into a plain line belongs
//!   > with a *kind* change, which nothing here does.
//!
//!   ★ **Wrong on its own terms.** "An arrow with no head is a different kind
//!   of mark" is a claim about this shell's tool palette, not about the file: a
//!   `/Line` with `/LE [/None /None]` is the same `/Subtype`, with the same
//!   geometry, reached by the same verb, and §12.5.6.7 treats its endings as
//!   *style* in exactly the way `/C` and `/BS` `/W` are style — which is why
//!   `MarkupStyle::endings` sits beside them in one struct rather than in a
//!   reshape. It is also a mark operators want and ask for: a leader with a
//!   head at one end only is among the commonest annotations on a drawing
//!   sheet, and Acrobat's line tool has carried this control for twenty years.
//!
//! - **A ce dimension — STANDS, unchanged.** [`super::dimension`] owns those,
//!   through `set_dimension_style` — a different verb with a different model,
//!   and `AnnotKind` carries the distinction **in the type** so this section's
//!   guard is a `match` the compiler checks. Restyling a ce dimension as
//!   ordinary markup regenerates it as a bare line with its label and witness
//!   lines gone. Rule 15 in one sentence: never write a bare dimension.
//!
//! ## ★★★ WHICH subtype takes WHICH property is the ENGINE's question — the
//! shell's copy of the list was deleted on 2026-09-06
//!
//! [`Current::from_spec`] used to answer three capability questions from
//! `MarkupSpec`'s arms: which shapes have an `/IC` to fill (four arms, with a
//! comment saying the list had been *"checked against the engine source"*),
//! which have a border to widen (the `TextMarkup` arm returning no width), and
//! which have `/LE` (the `Line` arm alone).
//!
//! Every one of those was correct on the day it was written, and this project
//! filed it as a boundary defect anyway:
//!
//! > *"That list is the engine's to know. The first subtype that gains or loses
//! > a border is the day our copy is wrong and nothing tells us."*
//!
//! ⇒ `pdfcer-core` shipped **`edit::MarkupStyleSupport::for_subtype`**
//! (`edit.rs:4493`; the type at `edit.rs:4460`) the same afternoon, with
//! `takes_border`, `takes_interior` and `takes_endings`, and quoted that
//! sentence into the type's doc comment as its justification. [`Current::support`]
//! holds the answer and the rows read it. **A comment saying a list was checked
//! against the engine source is a comment that ages; a call cannot.**
//!
//! ⚠ **What did NOT move.** *"What IS this mark's width?"* is still read off
//! the `MarkupSpec` arm, because only `MarkupSpec::Square` has a `border_width`
//! field and the engine publishes no API that would answer it. A **value** read
//! and a **capability** question are different questions with different owners;
//! `canvas::annotnodes`' header draws the same line for painting, and it is
//! right.
//!
//! ### ★ The refusal, which is the other half of the same Pass
//!
//! `EditError::StylePropertyNotApplicable { id, subtype, property }`
//! (`edit.rs:7353`) is raised at `edit.rs:26460`–`26483`, **before** anything is
//! regenerated. So the predicate above shapes this panel and the refusal
//! catches a shell that drifted anyway — belt and braces, and the reason
//! [`Current::restylable`]'s neighbours are not enough on their own. It reaches
//! the operator through the channel every engine refusal uses,
//! `app::actions::funnel::vector_edit`'s `Err` arm (`funnel.rs:276`): the
//! decline sentence on screen, the engine's own words into `PDFCER_DIAG`.
//! Nothing here builds a second route, because `check-ui-strings.sh`'s
//! exclusion 3 forbids one in as many words.
//!

use egui::Ui;
use pdfcer_core::annot_author::{Color, LineEnding, MarkupSpec};
use pdfcer_core::edit::{MarkupStyle, MarkupStyleSupport, StyleEdit};

use crate::app::actions::Action;
use crate::app::state::OpenDoc;
use crate::canvas::selection::annot::AnnotKind;
use crate::text::panels::properties as t;

/// The region this section publishes.
pub const REGION: &str = "properties.markup"; // ui-text-exempt: trace region name, never displayed

/// The narrowest border this shell offers.
///
/// Zero is excluded and it is a decision rather than an oversight: §8.4.3.2
/// gives `0` a defined meaning — *the thinnest line the device can render* —
/// which on a 600 dpi plot is a hairline and on screen at 25 % is invisible.
/// An operator who wants a mark they cannot see has the visibility toggle;
/// what they must not get is a mark whose weight depends on the output device
/// without being told.
const MIN_WIDTH_PT: f64 = 0.25;

/// The widest.
///
/// Beyond about twelve points a border stops reading as a border and starts
/// reading as a filled shape, which is the thing this shell deliberately does
/// not author. The same ceiling `canvas::markup::pen` uses, for the same
/// reason and from the same argument.
const MAX_WIDTH_PT: f64 = 12.0;

/// The width of the Line style chooser, in points.
///
/// ★ Wider than the Format band's `DASH_WIDTH` (88), and deliberately so: this
/// is the surface with room for the whole of
/// [`crate::text::markup::line_style_foreign`] — *"Dashed (the file's own
/// pattern)"* — which the band clips. §5.8's division of labour is that the tab
/// carries what an operator changes while working and the panel carries
/// everything; a reading that needs a sentence belongs on the second.
const DASH_WIDTH: f32 = 180.0;

/// **Draw the selected markup's style controls, or nothing.**
///
/// Returns whether it drew, so [`super::body`] knows the panel is already
/// saying something — the same contract [`super::dimension::section`] has, and
/// for the same reason: *"nothing is selected"* under a section describing the
/// thing that is selected would be the panel contradicting itself.
pub fn section(ui: &mut Ui, doc: &OpenDoc, actions: &mut Vec<Action>) -> bool {
    let Some(selection) = doc.selection.annot() else {
        return false;
    };
    // ★ Markup only. A ce dimension is `super::dimension`'s, and the
    // distinction is in the type rather than in a string comparison so that
    // routing one to the wrong verb is a compile error. Restyling a ce
    // dimension through `set_markup_style` regenerates it as a bare line with
    // its label and witness lines gone.
    if selection.target.kind != AnnotKind::Markup {
        return false;
    }
    // Cloned rather than borrowed: `AnnotTarget` carries the `/Subtype` as an
    // owned `String`, so it is not `Copy`, and the three rows below each need
    // the page and the id. One clone per frame of one small record is cheaper
    // than threading a borrow through a section that also reads the session.
    let target = selection.target.clone();

    crate::diag::ui_rect(REGION, ui.max_rect());
    // No `.strong()` — R84 / DEFECTS.md D11: no theme this project ships
    // renders it legibly on a panel.
    ui.label(t::markup_heading());
    ui.label(
        egui::RichText::new(t::markup_subtype(&target.subtype))
            .small()
            .weak(),
    );

    // ★★ **Locked is R9's "temporarily unavailable", so it GREYS with a reason
    // rather than vanishing.**
    //
    // §12.5.3 Table 165 bit 8 says a locked annotation's properties "shall not
    // be changed by the user interface", and the engine refuses
    // `set_markup_style` for one by name. That is a property of *this*
    // annotation rather than of this build — click a different mark and the
    // controls work — which is exactly the case R9 reserves greying for, and
    // exactly the case where making the controls absent would read as pdfcer
    // being unable to restyle anything.
    if target.locked {
        ui.label(egui::RichText::new(t::markup_locked()).small().weak());
        ui.separator();
        return true;
    }

    // ★★ Read from the SESSION every frame, never from a cache, and read
    // through the SAME function the selection was made with.
    //
    // The verb this section raises rewrites the very values it displays, and an
    // action is applied *after* the frame that raised it — so a cached copy
    // would be stale for exactly the frame the operator is looking at, which is
    // the frame they judge the result on.
    //
    // `page_annotations` is `canvas::selection::annot::selectable_on`'s own
    // source, so a mark this section can restyle is by construction a mark the
    // canvas could select. A second reader — a `/Annots` walk of this module's
    // own — would eventually disagree about which annotations exist, and the
    // symptom would be controls drawn for a selection that no verb could name.
    let current = Current::read(doc, target.id);

    // ★★★ **The style verb cannot reach every mark this section can be shown
    // for**, and until 2026-09-06 the rows below were drawn anyway. See the
    // module header: `AnnotKind::Markup` covers a sticky note, a text box and a
    // stamp; `spec_from_dict` refuses all three; `set_markup_style` calls it
    // first. Live controls, every press refused.
    //
    // ★ The predicate is `spec_from_dict` succeeding — the SAME call the verb
    // makes — rather than a subtype list written here. A list would be correct
    // today and wrong on the day the engine adds a subtype, and wrong in the
    // silent direction: withholding a control that had started working.
    if !current.restylable {
        ui.label(
            egui::RichText::new(t::markup_not_restylable())
                .small()
                .weak(),
        );
        ui.separator();
        return true;
    }

    colour_row(ui, current, &target, actions);
    fill_row(ui, current, &target, actions);
    // ★ The narrowing disclosure sits under BOTH swatches and above the rest,
    // because it qualifies them and `REVIEW_TRIAGE.md`'s rule is that a caveat
    // below the thing it qualifies arrives after the operator has drawn their
    // conclusion. It is absent — not greyed, not blank — for the overwhelming
    // majority of marks, whose `/C` is RGB or grey and costs no conversion.
    if current.colour.narrowed || current.interior.narrowed {
        ui.label(
            egui::RichText::new(t::markup_colour_narrowed())
                .small()
                .weak(),
        );
    }
    width_row(ui, current, &target, actions);
    // ★ Directly under the width, because the two are one subject — *what the
    // line looks like* — and the Format tab's band puts them adjacent for the
    // same reason. A panel is read top to bottom, and an operator setting a
    // mark's linework should not have to read past the arrowheads to finish.
    dash_row(ui, current, &target, actions);
    endings_row(ui, current, &target, actions);
    opacity_row(ui, current, &target, actions);

    ui.label(egui::RichText::new(t::markup_note()).small().weak());
    ui.separator();
    true
}

/// What the selected mark's dictionary currently says, in the terms this
/// section can change — **and whether it can change any of them at all**.
///
/// It described three terms until 2026-09-06 (colour, width, opacity) and now
/// describes five, having gained the fill and the two line endings; the sixth
/// field, [`Self::restylable`], is not a term at all but the answer to whether
/// the other five are reachable.
///
/// # ★★ Why it is read through `spec_from_dict` and not from `annot::Annotation`
///
/// `pdfcer_core::annot::Annotation` is the **reader's** view — id, subtype,
/// rect, flags, `/CA`, appearance — and it deliberately carries no `/C` and no
/// `/BS /W`, because nothing that renders a page needs them: the picture comes
/// from the baked `/AP`.
///
/// `annot_author::spec_from_dict` is the **author's** view, and it exists for
/// exactly this: *"so an existing annotation can be restyled by regenerating
/// its appearance from its own declared geometry"*. Reading through it means
/// the values these controls show are the values `set_markup_style` will read
/// when it plans — one derivation, not two.
///
/// ★ Its refusals are `None` here rather than an error, and that is honest
/// rather than lax. `SpecReadError`'s own doc says every variant is *"a refusal
/// to guess"* — an unsupported `/Subtype`, or geometry that is missing or is
/// not something pdfcer models.
///
/// ⚠ **What a refusal MEANS here changed on 2026-09-06, and the old reading was
/// the defect.** This paragraph used to continue:
///
/// > A mark like that can still be **given** a colour; what cannot be done is
/// > show the one it has, so the swatch falls back to its default and offers no
/// > Clear. Nothing is destroyed by touching nothing.
///
/// The first clause is **false**, and it was the whole mistake:
/// `set_markup_style` opens by calling this same function and propagating its
/// error with `?`, so a mark it refuses cannot be given a colour either. The
/// swatch was not merely uninformative — it could not commit. See
/// [`Self::restylable`] and the module header.
#[derive(Debug, Clone, Copy)]
struct Current {
    /// ★★★ **Whether `spec_from_dict` could read a spec out of this annotation
    /// at all** — and therefore whether `set_markup_style` will do anything but
    /// refuse.
    ///
    /// `false` is not "this mark has no colour". It is *"the style verb does not
    /// reach this `/Subtype`"*, which is a different fact with a different
    /// consequence: no rows at all, plus a sentence. See the module header.
    ///
    /// It is a field rather than a recomputation because the answer is already
    /// in hand — the read below has to call `spec_from_dict` regardless — and
    /// two calls to one function is how the panel and the verb come to disagree
    /// about the same annotation.
    restylable: bool,
    /// ★★★ **Which of these properties this `/Subtype` can take at all — the
    /// ENGINE's answer, not this module's.**
    ///
    /// `MarkupStyleSupport::for_subtype` (`pdfcer-core` `edit.rs:4493`) is
    /// asked once, off the annotation's own `/Subtype`, and [`fill_row`],
    /// [`width_row`] and [`endings_row`] consult it before drawing anything.
    ///
    /// ⚠ **This field replaced a `has_interior: bool` on 2026-09-06**, whose
    /// doc comment read:
    ///
    /// > **Whether this shape has an interior to fill at all**, which is a
    /// > property of its `MarkupSpec` arm rather than of its dictionary: a
    /// > `/Square`, `/Circle`, `/Polygon` and a cloud carry `/IC`; a `/Line`, an
    /// > `/Ink`, a `/PolyLine` and a text markup have no interior for one to
    /// > mean anything in.
    ///
    /// Every word of that was true, and it was still four subtypes' worth of
    /// the engine's knowledge kept in a shell — the boundary defect this
    /// project filed and the engine answered. Note what has *not* changed: the
    /// row is still **absent** when the answer is `false`, for the same R9
    /// reason, and the cloud is still handled correctly, now because
    /// `for_subtype(b"Polygon")` says so rather than because this module
    /// remembered that a revision cloud is a `/Polygon` in the file.
    ///
    /// ★ It is not the same question as [`Self::restylable`] and neither
    /// subsumes the other: `restylable` asks *can `set_markup_style` read this
    /// mark at all* (`spec_from_dict` succeeding), this asks *which of its
    /// properties mean anything*. A `/Highlight` answers **yes** to the first
    /// and **no** to `takes_border` — which is exactly the mark the engine now
    /// refuses a width for.
    support: MarkupStyleSupport,
    /// `/C`, as a swatch can show it, and whether showing it cost a conversion.
    colour: Swatch,
    /// `/IC`, when [`Self::support`] says the shape has one. `None` inside means
    /// *no fill*, which is the state this shell authors and the state Acrobat
    /// defaults to.
    interior: Swatch,
    /// `/BS` `/W`, the border width in points.
    ///
    /// ★ A **value**, from the `MarkupSpec` arm that has one. Whether the row is
    /// offered is `support.takes_border`; see the module header's distinction
    /// between a value read and a capability question.
    width: Option<f64>,
    /// `/CA`, the constant opacity.
    alpha: Option<f64>,
    /// **`/BS` `/S` and `/D` — the border's line style**, as the chooser shows
    /// it.
    ///
    /// ★★ Read off the **dictionary**, not off the spec, and that is the same
    /// exception `/CA` is rather than a departure from this struct's rule. A
    /// dash cuts across `MarkupSpec`'s variants rather than belonging to any one
    /// of them, so the engine carries it in `AppearanceOptions` beside the spec
    /// instead of inside it (`pdfcer-core` `annot_author.rs:1633-1673`) and
    /// `spec_from_dict` returns none. The engine's own reader is `pub(crate)`
    /// (`annot_author.rs:840`), so [`crate::canvas::markup::linestyle::read`] is
    /// this shell's copy of it — declared as a copy in that function's header,
    /// with the bound on what a divergence can cost written down beside it.
    ///
    /// ★ It therefore travels **through** `from_spec` rather than being derived
    /// in it, exactly as `alpha` and `endings_key_present` do, and for the same
    /// reason: it is a fact about the dictionary that the spec reader does not
    /// carry.
    dash: crate::canvas::markup::linestyle::DashReading,
    /// `/LE`, the pair of line endings the mark currently draws.
    ///
    /// ★ Also a value, and `MarkupSpec::Line` is the only arm carrying one — a
    /// fact the compiler checks. `support.takes_endings` is what decides
    /// whether the choosers appear.
    endings: Option<(LineEnding, LineEnding)>,
    /// ★★ **Whether `/LE` is actually IN the dictionary**, as distinct from
    /// being supplied by Table 176's default on the way through
    /// `spec_from_dict`.
    ///
    /// The one thing [`Self::endings`] cannot tell anybody: the spec reader
    /// hands back `(None, None)` both for a `/Line` with no `/LE` and for one
    /// carrying `/LE [/None /None]`, because those two draw the same picture.
    /// Correct for a reader whose job is the picture; useless to the *Clear the
    /// setting* control, whose whole subject is the difference. So the key is
    /// looked for on the dictionary itself.
    endings_key_present: bool,
}

/// A colour a swatch can show, and the honesty that goes with it.
///
/// ★★ The second field is the whole reason this is a struct rather than an
/// `Option<[u8; 3]>`. `/C` and `/IC` may be grey, RGB **or CMYK** (§12.5.2), and
/// the three are not equally showable: grey is the same ink as its equal-
/// component RGB and converts losslessly in both directions, where CMYK does
/// not. Carrying *whether a conversion happened* beside the converted value is
/// what lets [`section`] disclose it rather than the operator discovering it
/// from a changed file. See [`t::markup_colour_narrowed`] for the argument that
/// replaced the old refuse-to-show behaviour.
#[derive(Debug, Clone, Copy, Default)]
struct Swatch {
    /// What to show, `None` when there is no such key or it names no device
    /// space §8.6.3 defines.
    rgb: Option<[u8; 3]>,
    /// `true` when [`Self::rgb`] is a **conversion** rather than the file's own
    /// value, so a change made through it narrows the colour space.
    narrowed: bool,
}

/// ★ Even "nothing to show" asks the engine what the properties are.
///
/// `MarkupStyleSupport` is `#[non_exhaustive]` and has no `Default`, so the
/// derive had to go — and that is worth keeping rather than working around.
/// The honest default for *"the dictionary could not be read"* is **not** a
/// hand-written all-`false` literal; it is what the engine answers for a
/// subtype it does not recognise, which `for_subtype`'s own doc calls *"the
/// conservative direction: a caller is told a property is unavailable rather
/// than being told one is available on a shape pdfcer cannot restyle at all."*
/// Asking for it removes the last place a `false` about a subtype could have
/// been written by hand in this module.
impl Default for Current {
    fn default() -> Self {
        Self {
            restylable: false,
            support: MarkupStyleSupport::for_subtype(b""),
            colour: Swatch::default(),
            interior: Swatch::default(),
            width: None,
            alpha: None,
            // Solid, which is what `linestyle::read` answers for an annotation
            // with no `/BS` at all — so an unreadable dictionary and a plainly
            // solid one show the same chooser, and neither invents a dash.
            dash: crate::canvas::markup::linestyle::DashReading::Solid,
            endings: None,
            endings_key_present: false,
        }
    }
}

impl Current {
    /// Read it out of the session, this frame.
    fn read(doc: &OpenDoc, id: pdfcer_core::object::ObjId) -> Self {
        use pdfcer_core::annot_author::spec_from_dict;
        use pdfcer_core::object::Object;

        let graph = doc.session.graph();
        let Some(Object::Dict(dict)) = doc.session.value(id) else {
            return Self::default();
        };
        // `/CA` straight off the dictionary rather than through the spec: it is
        // not part of `MarkupSpec` at all — the engine's own note says it
        // composites the annotation onto the page rather than affecting what
        // the appearance draws, which is why `set_markup_style` applies it to
        // the dictionary directly.
        // ★ `ObjectGraph::resolve` comes from the TRAIT, so it has to be in
        // scope. Reaching for the inherent method — there is none — is the
        // error a reader will hit first, and importing the trait beside the use
        // is what makes the call read as what it is: an indirect reference
        // followed through the session's overlay rather than through the base
        // file, so an unsaved edit is visible.
        use pdfcer_core::graph::ObjectGraph;
        let alpha = dict
            .get(b"CA")
            .map(|o| graph.resolve(o))
            .and_then(Object::as_number);

        // ★★★ **The one call**, and its verdict is carried rather than
        // recomputed. `.ok().as_ref()` turns the refusal into the `None` that
        // [`Self::from_spec`] reads as *"the style verb does not reach this
        // mark"* — which is precisely what a `SpecReadError` means to
        // `set_markup_style`, since that verb's next line after this same call
        // is `?`.
        // ★★★ **The capability question, asked of the engine, off the same key
        // the engine itself reads.** `set_markup_style` derives its
        // `MarkupStyleSupport` from `/Subtype` on the annotation dictionary
        // (`edit.rs:26453`–`26460`) and refuses a property the answer excludes
        // before anything is regenerated. Reading the same key through the same
        // function is what makes a row drawn here and a call refused there
        // impossible to disagree.
        let subtype = dict
            .get(b"Subtype")
            .map(|o| graph.resolve(o))
            .and_then(Object::as_name)
            .map_or_else(Vec::new, |n| n.as_bytes().to_vec());
        let support = MarkupStyleSupport::for_subtype(&subtype);

        // ★★ Presence, not value — see `Self::endings_key_present`. This is
        // the one fact `spec_from_dict` erases, and the *Clear the setting*
        // button exists to act on it.
        let endings_key_present = dict
            .get(b"LE")
            .map(|o| graph.resolve(o))
            .is_some_and(|o| !matches!(o, Object::Null));

        // ★ Read BEFORE `spec_from_dict` and carried across its refusal is not
        // needed here — a mark the spec reader refuses gets no rows at all
        // (`Self::restylable`) — but it is read off the dictionary for the same
        // reason `/CA` is: the spec has no dash in it to read.
        let dash = crate::canvas::markup::linestyle::read(&graph, dict);

        Self::from_spec(
            spec_from_dict(&graph, dict).ok().as_ref(),
            support,
            alpha,
            dash,
            endings_key_present,
        )
    }

    /// The pure half: everything this section shows, derived from the spec the
    /// engine read (or from its absence).
    ///
    /// # ★ Why it is split out from [`Self::read`]
    ///
    /// Because it is the part with the decisions in it, and it is the part a
    /// test can reach. `read` needs an `OpenDoc`, a session and a real
    /// annotation dictionary; `from_spec` needs a `MarkupSpec`, which is a value
    /// a test constructs in one expression. The tests at the foot of this module
    /// assert the reachability verdict, the interior slot and the endings slot
    /// through this function, and each of them was falsified by breaking the arm
    /// it guards and watching it go red.
    ///
    /// `None` means the read refused — an unsupported `/Subtype`, or geometry
    /// pdfcer does not model. Both produce the same answer here for the same
    /// reason: `set_markup_style` would refuse the same call.
    fn from_spec(
        spec: Option<&MarkupSpec>,
        support: MarkupStyleSupport,
        alpha: Option<f64>,
        dash: crate::canvas::markup::linestyle::DashReading,
        endings_key_present: bool,
    ) -> Self {
        let Some(spec) = spec else {
            // ★ Note what is NOT carried across: `alpha`. `/CA` reads fine off
            // any annotation dictionary, so it would be easy to keep — and it
            // would be a value shown under a heading whose every control is
            // about to be withheld. The opacity row cannot commit on a mark
            // `set_markup_style` refuses, so the value it would display is
            // decoration.
            return Self::default();
        };
        let (colour, width) = match spec {
            MarkupSpec::Square {
                border,
                border_width,
                ..
            }
            | MarkupSpec::Circle {
                border,
                border_width,
                ..
            } => (swatch_of(border.as_ref()), Some(*border_width)),
            MarkupSpec::Polygon { border, width, .. } | MarkupSpec::Cloud { border, width, .. } => {
                (swatch_of(border.as_ref()), Some(*width))
            }
            MarkupSpec::Line { color, width, .. }
            | MarkupSpec::PolyLine { color, width, .. }
            | MarkupSpec::Ink { color, width, .. } => (swatch_of(Some(color)), Some(*width)),
            // A text markup has a colour and no border at all — its shape is
            // `/QuadPoints` and there is nothing to stroke, so the arm has no
            // width to hand over.
            //
            // ⚠ **Corrected 2026-09-06.** This comment used to continue:
            //
            //   > The width row still draws, with the engine's own default
            //   > showing, because `set_markup_style` accepts a width for it
            //   > and simply has nothing to apply it to.
            //
            // Two things about that are now wrong, and neither was wrong when
            // it was written. `set_markup_style` no longer *accepts* a width
            // here — it answers `EditError::StylePropertyNotApplicable`
            // (`edit.rs:26462`) before touching the file, which is the request
            // this project filed against the silent no-op. And whether the row
            // draws is no longer decided by this arm handing over `None`; it is
            // `support.takes_border`, which is the engine's to say.
            MarkupSpec::TextMarkup { color, .. } => (swatch_of(Some(color)), None),
            // `MarkupSpec` is `#[non_exhaustive]`. A kind this build does
            // not know the shape of gets no readback and no Clear, which is
            // the same answer a refused parse gets and for the same reason.
            _ => (Swatch::default(), None),
        };
        // ★★ The interior VALUE, read off the spec arm because that is where a
        // value lives — only these four arms have an `interior` field and the
        // compiler checks which.
        //
        // ⚠ **Corrected 2026-09-06.** This comment used to close with the
        // sentence that made it a capability decision:
        //
        //   > `apply_markup_style` applies `style.interior` to exactly these
        //   > four arms — checked against the engine source, not assumed.
        //
        // A comment recording that a list was checked against the engine source
        // is a comment that goes stale the first time the engine changes and
        // nothing says so. `support.takes_interior` is the answer now, and
        // [`fill_row`] is what asks it; this `match` supplies the colour and
        // stops there. The cloud case the old comment was proud of still works
        // and now works for a better reason: `for_subtype(b"Polygon")` is what
        // says a revision cloud has an `/IC`, rather than this module
        // remembering that a cloud is a `/Polygon` in the file.
        let interior = match spec {
            MarkupSpec::Square { interior, .. }
            | MarkupSpec::Circle { interior, .. }
            | MarkupSpec::Polygon { interior, .. }
            | MarkupSpec::Cloud { interior, .. } => swatch_of(interior.as_ref()),
            _ => Swatch::default(),
        };
        Self {
            restylable: true,
            support,
            colour,
            interior,
            width,
            alpha,
            // ★ Carried through untouched, like `alpha`. There is no arm to
            // derive it from — a dash is not in `MarkupSpec` at all — and no
            // decision to take about it here: the chooser's absence for a
            // borderless subtype is `support.takes_border`'s answer, and the
            // reading itself is the dictionary's.
            dash,
            // ★ The pair `/Line` draws, from the one arm that has one. That
            // `/Line` is the only subtype the control is *offered* for is
            // `support.takes_endings`' answer, and the engine's own words for
            // it are on `MarkupStyleSupport::takes_endings`: Table 176 declares
            // `/LE` for `/PolyLine` too, "but pdfcer authors endings on a line
            // alone."
            endings: match spec {
                MarkupSpec::Line { endings, .. } => Some(*endings),
                _ => None,
            },
            endings_key_present,
        }
    }

    // -----------------------------------------------------------------------
    // ★★★ WHETHER a row is drawn — one question, one place, testable
    //
    // The three rows take a `Ui` and can only be exercised by driving the
    // binary; these take nothing and are reachable from a unit test, which is
    // what lets `the_engines_answer_is_what_hides_a_row_not_the_spec_arm`
    // falsify the claim in both directions. Two call sites spelling
    // `takes_border && width.is_some()` slightly differently is the same drift
    // the whole request was about, one level down.
    // -----------------------------------------------------------------------

    /// Whether the Fill row draws.
    ///
    /// Purely the engine's answer: *no fill* is a legitimate current state and
    /// [`fill_row`] shows it as a default swatch with [`t::markup_fill_none`]
    /// beside it, so there is no value whose absence should withhold the row.
    const fn offers_fill(self) -> bool {
        self.support.takes_interior
    }

    /// Whether the width row draws.
    ///
    /// Two terms meaning two different things. `takes_border` false is *this
    /// subtype has no border* — the engine's, and permanent. A `None` width
    /// under a `takes_border` that is true is *this build cannot read this
    /// arm's width*, which `MarkupSpec` being `#[non_exhaustive]` makes
    /// reachable; a spinner with no value to show is what R9 and
    /// `app::markupband::placeholder` both forbid.
    const fn offers_width(self) -> bool {
        self.support.takes_border && self.width.is_some()
    }

    /// Whether the Line style row draws.
    ///
    /// ★★ **Purely the engine's answer, with no second term** — unlike
    /// [`Self::offers_width`], which also asks whether a width was read. The
    /// asymmetry is real: a width has to be *shown* in a spinner, so a mark
    /// whose width this build could not read has nothing to put in one; a line
    /// style always has a value, because *solid* is a state rather than an
    /// absence and [`crate::canvas::markup::linestyle::read`] is total — every
    /// dictionary answers it, including one carrying no `/BS`.
    ///
    /// ⇒ So the only question left is the engine's *does this subtype have a
    /// border?*, which is the same predicate `set_markup_style` guards
    /// `style.dash` with (`pdfcer-core` `edit.rs:26463-26476`). A row drawn here
    /// cannot produce that refusal.
    const fn offers_dash(self) -> bool {
        self.support.takes_border
    }

    /// Whether the two ending choosers draw.
    const fn offers_endings(self) -> bool {
        self.support.takes_endings && self.endings.is_some()
    }

    /// Whether the *Clear the setting* button draws under them.
    ///
    /// ★ Strictly narrower than [`Self::offers_endings`]: there has to be a
    /// chooser to sit under **and** a `/LE` in the file to take out.
    const fn offers_endings_clear(self) -> bool {
        self.offers_endings() && self.endings_key_present
    }
}

/// The border colour.
///
/// ★ A **swatch plus a Clear**, not a swatch alone. `StyleEdit` has two arms
/// and they mean different things in the file: `Set` writes `/C`, and `Clear`
/// removes it, restoring the standard's default. A control that could only set
/// would make `/C` a one-way door — once an operator gave a mark a colour there
/// would be no way back to the file's own, and the difference is visible in
/// another viewer even when it is not visible here.
fn colour_row(
    ui: &mut Ui,
    current: Current,
    target: &crate::canvas::selection::annot::AnnotTarget,
    actions: &mut Vec<Action>,
) {
    let existing = current.colour.rgb;
    let mut rgb = existing.unwrap_or([0, 0, 0]);
    ui.horizontal(|ui| {
        ui.label(t::markup_colour_label());
        if ui.color_edit_button_srgb(&mut rgb).changed() {
            actions.push(Action::SetMarkupStyle {
                page: target.page,
                id: target.id,
                style: MarkupStyle {
                    stroke: Some(StyleEdit::Set(Color::Rgb(
                        f64::from(rgb[0]) / 255.0,
                        f64::from(rgb[1]) / 255.0,
                        f64::from(rgb[2]) / 255.0,
                    ))),
                    ..MarkupStyle::default()
                },
            });
        }
        // Absent when there is nothing to clear, rather than greyed: a Clear
        // beside a mark that has no `/C` is a control whose only possible
        // effect is an undo entry the operator did not earn.
        if existing.is_some() && ui.button(t::markup_clear()).clicked() {
            actions.push(Action::SetMarkupStyle {
                page: target.page,
                id: target.id,
                style: MarkupStyle {
                    stroke: Some(StyleEdit::Clear),
                    ..MarkupStyle::default()
                },
            });
        }
    });
}

/// **The interior colour, `/IC` — the Fill row.**
///
/// # ★★★ Why this exists at all, when the header used to argue against it
///
/// `MarkupStyle::interior` shipped in the engine with `set_markup_style` and had
/// **zero GUI callers** until 2026-09-06. The module header carries the argument
/// that kept it that way, and carries the correction beside it; the short form
/// is that *"a filled comment shape hides the drawing it is a comment about"* is
/// a sound reason to author `interior: None` and not a reason to refuse an
/// operator the ability to fill a shape they have already placed.
///
/// `canvas::markup::spec` is untouched by this. **No fill at author time, fill
/// available on restyle.**
///
/// # ★★ Absent for a shape with no interior — and the ENGINE says which
///
/// A `/Line`, an `/Ink`, a `/PolyLine` and a text markup have no interior for
/// `/IC` to mean anything in, so a Fill control there would be drawn, live, and
/// dropped on the floor. That is the same defect this session came to fix, one
/// control down, and R9's answer is the same: the row is absent, the way
/// [`width_row`] is absent for a highlight.
///
/// ⚠ **Corrected 2026-09-06.** The paragraph above used to justify the list
/// with *"`apply_markup_style` does not read `style.interior` on those arms"* —
/// a fact about the engine's source, restated here, where nothing checks it.
/// The list is now asked for: `MarkupStyleSupport::takes_interior`, through
/// [`Current::offers_fill`]. The old sentence was not wrong; it was a copy, and
/// a copy is what this project filed a request to be rid of.
///
/// # ★ The swatch shape mirrors [`colour_row`] exactly, including the Clear
///
/// Set writes `/IC`; Clear removes it and the shape is unfilled again. The one
/// addition is the word beside the swatch when there is no fill — a swatch
/// cannot show *absence*, and a black square next to "Fill" says the opposite of
/// the truth. See [`t::markup_fill_none`].
fn fill_row(
    ui: &mut Ui,
    current: Current,
    target: &crate::canvas::selection::annot::AnnotTarget,
    actions: &mut Vec<Action>,
) {
    if !current.offers_fill() {
        return;
    }
    let existing = current.interior.rgb;
    let mut rgb = existing.unwrap_or([0, 0, 0]);
    ui.horizontal(|ui| {
        ui.label(t::markup_fill_label());
        if ui.color_edit_button_srgb(&mut rgb).changed() {
            actions.push(Action::SetMarkupStyle {
                page: target.page,
                id: target.id,
                style: MarkupStyle {
                    interior: Some(StyleEdit::Set(Color::Rgb(
                        f64::from(rgb[0]) / 255.0,
                        f64::from(rgb[1]) / 255.0,
                        f64::from(rgb[2]) / 255.0,
                    ))),
                    ..MarkupStyle::default()
                },
            });
        }
        if existing.is_some() {
            // Absent when there is nothing to clear, for `colour_row`'s reason:
            // a Clear beside a shape with no `/IC` is a control whose only
            // possible effect is an undo entry the operator did not earn.
            if ui.button(t::markup_clear()).clicked() {
                actions.push(Action::SetMarkupStyle {
                    page: target.page,
                    id: target.id,
                    style: MarkupStyle {
                        interior: Some(StyleEdit::Clear),
                        ..MarkupStyle::default()
                    },
                });
            }
        } else {
            ui.label(egui::RichText::new(t::markup_fill_none()).small().weak());
        }
    });
}

/// The border width.
///
/// ⚠ **This moves `/Rect` for every subtype except `Square` and `Circle`**, and
/// the engine says so in its own doc: the rectangle is derived from the
/// geometry plus a margin that contains the stroke and any arrowheads, so a
/// wider pen needs a bigger box. That is disclosed in [`t::markup_note`]
/// rather than here, because it is true of the section and not of this control
/// alone.
/// **The border line style, `/BS` `/S` and `/D` — the Line style row.**
///
/// # ★★★ Why this exists, and what it took to make it SAFE
///
/// `RIBBON_IA.md` §5.8's Markup row lists eight controls and this was the
/// eighth. It read **⛔ no engine verb exists**, and that was true: `MarkupStyle`
/// carried colour, interior, width, opacity and endings and had no dash field at
/// all, so there was nothing for a control to reach.
///
/// The verb arrived on the afternoon of 2026-09-06 with two others, and the
/// **other two are what make this one safe to offer**:
///
/// * `/BS` `/S` and `/D` are read back on the way IN, so a restyle that does not
///   mention the dash **preserves** it — including a dash pdfcer never authored
///   (`pdfcer-core` `edit.rs:4396-4425`);
/// * `MarkupOptions::dash` authors one, so a shape can be *drawn* dashed rather
///   than drawn and then corrected.
///
/// ⇒ Before that, a dashed mark in the operator's file was silently converted to
/// a solid one the first time anything re-baked its appearance. The engine's
/// reply records that this was **wider than this shell reported**: the recolour
/// path was named, and `resize_annotation`, `reshape_annotation` and authoring
/// solidified a dash too — so it was reachable by dragging a resize handle or a
/// vertex, not only by pressing the colour swatch. All four carry it now. A
/// Line style control over the old engine would have been a control its
/// neighbours undid.
///
/// # ★★ The "way back to the default" is an ENTRY, not a Clear button
///
/// [`colour_row`] and [`fill_row`] each put `StyleEdit::Clear` behind its own
/// button, because in both cases the cleared state is *the absence of a key* and
/// has no name in a list: a swatch cannot show *no colour*. A border's cleared
/// state does have a name. `Clear` makes it **solid**, solid is Table 166's own
/// `/S` default, and *Solid* is the chooser's first entry — so a separate button
/// would be a second spelling of one act, and the two would eventually be
/// pressed expecting different things.
///
/// ★ It is also why the button's absence rule does not apply. A Clear beside a
/// mark with nothing to clear is *"a control whose only possible effect is an
/// undo entry the operator did not earn"*; a **Solid** entry beside a mark that
/// is already solid is simply the entry that is currently selected, and
/// [`crate::canvas::markup::linestyle::chooser`] reports nothing when the
/// current entry is picked again.
///
/// # ★ Absent for a subtype with no border
///
/// [`Current::offers_dash`], which is `MarkupStyleSupport::takes_border` and
/// nothing else — a highlight is a colour wash and has no `/BS` to dash. R9, and
/// the same answer [`width_row`] gives.
fn dash_row(
    ui: &mut Ui,
    current: Current,
    target: &crate::canvas::selection::annot::AnnotTarget,
    actions: &mut Vec<Action>,
) {
    if !current.offers_dash() {
        return;
    }
    ui.horizontal(|ui| {
        ui.label(t::markup_line_style_label());
        let picked = crate::canvas::markup::linestyle::chooser(
            ui,
            // ui-text-exempt: internal widget id, never displayed
            "properties-markup-line-style",
            current.dash,
            DASH_WIDTH,
        );
        // ★ The `Option` from `LineStyle::style_edit` is answered by raising
        // NOTHING — no action, no undo entry, no substituted pattern. It is
        // unreachable for the four offered styles
        // (`linestyle::tests::every_offered_pattern_is_one_the_engine_accepts`),
        // and writing Table 166's default in its place would be this panel
        // choosing a pattern the operator did not.
        if let Some(edit) = picked.and_then(crate::canvas::markup::linestyle::LineStyle::style_edit)
        {
            actions.push(Action::SetMarkupStyle {
                page: target.page,
                id: target.id,
                style: MarkupStyle {
                    dash: Some(edit),
                    ..MarkupStyle::default()
                },
            });
        }
    });
}

fn width_row(
    ui: &mut Ui,
    current: Current,
    target: &crate::canvas::selection::annot::AnnotTarget,
    actions: &mut Vec<Action>,
) {
    // ★ ABSENT rather than greyed when the mark has no border to widen — a
    // highlight is `/QuadPoints` and has nothing to stroke. R9: an unavailable
    // capability renders nothing. A greyed spinner here would be pdfcer
    // implying that a highlight could have a line width if only something were
    // different, and nothing is.
    if !current.offers_width() {
        return;
    }
    let Some(mut width) = current.width else {
        return;
    };
    ui.horizontal(|ui| {
        ui.label(t::markup_width_label());
        let response = ui.add(
            egui::DragValue::new(&mut width)
                .range(MIN_WIDTH_PT..=MAX_WIDTH_PT)
                .speed(0.1)
                .suffix(t::markup_width_suffix()),
        );
        // ★ `drag_stopped` and `lost_focus`, not `changed`. A `DragValue` reports
        // a change on every pixel of a drag, and each one here is a
        // content-stream rewrite plus an undo entry — so a single drag across
        // the control would leave forty entries on the stack and re-plan the
        // annotation forty times. The colour swatch above needs no such guard:
        // it opens a popup and reports once, on the operator's pick.
        if response.drag_stopped() || response.lost_focus() {
            actions.push(Action::SetMarkupStyle {
                page: target.page,
                id: target.id,
                style: MarkupStyle {
                    width: Some(width),
                    ..MarkupStyle::default()
                },
            });
        }
    });
}

/// **The two line endings, `/LE` — what makes an arrow an arrow.**
///
/// # ★★★ Why this exists, when the header used to argue against it
///
/// `MarkupStyle::endings` shipped with `set_markup_style` and had **zero GUI
/// callers** until 2026-09-06. The header carries the overturned argument in
/// full; the short form is that a `/Line` with `/LE [/None /None]` is not a
/// different *kind* of mark — same `/Subtype`, same geometry, same verb — and
/// §12.5.6.7 treats the endings as style, which is why the engine put them in
/// the style struct rather than in a reshape.
///
/// # ★★ Two controls, ONE dictionary property — and why that is not a breach of
/// this module's "one field per action" rule
///
/// The rule at the top of this file forbids assembling a whole `MarkupStyle`
/// from what the widgets happen to show, because two controls read a frame apart
/// will disagree and the later action will silently revert the earlier one.
/// `/LE` is a **single two-element array** (Table 176) and
/// `MarkupStyle::endings` is a single `Option<(LineEnding, LineEnding)>`, so
/// changing one end necessarily sends both — there is no field that carries half
/// of it.
///
/// ★ That is still one field of one property, and it is still safe, for the
/// reason the rule actually rests on: the unchanged half comes from
/// [`Current::endings`], which was read **from the session this frame** through
/// `spec_from_dict`. It is not a widget's remembered value and cannot be stale.
/// The failure the rule prevents needs a second control holding a copy of a
/// value it set earlier, and neither of these two holds anything.
///
/// # ★ `/Line` only, and absent otherwise — the engine's answer, since
/// 2026-09-06
///
/// `MarkupStyleSupport::takes_endings` is `true` for `/Line` and for nothing
/// else, and [`Current::offers_endings`] is what asks it. A chooser on a
/// polygon would be live and would be **refused** —
/// `EditError::StylePropertyNotApplicable` at `edit.rs:26478` — so R9 says
/// absent and the engine agrees in writing.
///
/// ⚠ This paragraph used to read *"[`Current::endings`] is `Some` for a
/// `MarkupSpec::Line` and nothing else, which matches `apply_markup_style`
/// exactly."* True, and a restatement of the engine's list inside a shell. The
/// spec arm still supplies the **pair**, because that is a value; it no longer
/// decides whether the control exists.
fn endings_row(
    ui: &mut Ui,
    current: Current,
    target: &crate::canvas::selection::annot::AnnotTarget,
    actions: &mut Vec<Action>,
) {
    if !current.offers_endings() {
        return;
    }
    let Some((start, end)) = current.endings else {
        return;
    };
    let mut chosen = (start, end);
    ending_chooser(
        ui,
        t::markup_line_start_label(),
        "properties-markup-line-start", // ui-text-exempt: internal widget id, never displayed
        &mut chosen.0,
    );
    ending_chooser(
        ui,
        t::markup_line_end_label(),
        "properties-markup-line-end", // ui-text-exempt: internal widget id, never displayed
        &mut chosen.1,
    );
    // ★★★ **The fifth state — and yes, it belongs here as well as on the tab.**
    //
    // Three arguments, and the third is the one that settles it:
    //
    // 1. **§5.8's division of labour.** The panel *"carries everything"*; the
    //    tab carries what is reached for mid-gesture. A capability the tab has
    //    and the panel lacks is the one direction that rule forbids outright.
    // 2. **Consistency inside this section.** `/C`, `/IC` and `/CA` each offer
    //    a Clear on the same terms — present only when there is a key to
    //    remove. `/LE` was the odd one out **solely** because
    //    `MarkupStyle::endings` was a bare `Option` and the removal could not
    //    be expressed. That reason is gone, so the exception should go with it.
    // 3. **This is the surface an operator is on when the question arises.**
    //    *"Does this file still match the one my client sent me"* is asked
    //    while looking at an annotation's properties, not while reaching across
    //    a ribbon mid-drag.
    //
    // ★ It is a **button on its own row** rather than an entry in the two
    // choosers, and the reason is the tab's reason one level down: the choosers
    // answer *what shape at this end*, and `LineEnding::None` is already an
    // answer to that. A removal offered as a fourth shape would be a second
    // entry drawing exactly what *No end* draws, which is a distinction a
    // drafter cannot check by looking.
    //
    // ★ Absent when there is no `/LE` to remove — `Current::offers_endings_clear`
    // — which is `colour_row`'s rule and the same sentence: a Clear beside a
    // mark that has nothing to clear is a control whose only possible effect is
    // an undo entry the operator did not earn.
    if current.offers_endings_clear()
        && ui
            .button(t::markup_endings_clear())
            .on_hover_text(t::markup_endings_clear_hint())
            .clicked()
    {
        actions.push(Action::SetMarkupStyle {
            page: target.page,
            id: target.id,
            style: MarkupStyle {
                endings: Some(StyleEdit::Clear),
                ..MarkupStyle::default()
            },
        });
    }
    ui.label(
        egui::RichText::new(t::markup_line_ending_note())
            .small()
            .weak(),
    );
    if chosen != (start, end) {
        actions.push(Action::SetMarkupStyle {
            page: target.page,
            id: target.id,
            style: MarkupStyle {
                // ★ `StyleEdit::Set` since 2026-09-06 — the arm that WRITES
                // `/LE`, including `Set((None, None))` when the operator sets
                // both ends to *No end*. That is deliberately not the same act
                // as the Clear above: it states "no arrowheads" in the file
                // where Clear takes the statement out. Same picture, different
                // bytes; see `t::markup_endings_clear`.
                endings: Some(StyleEdit::Set(chosen)),
                ..MarkupStyle::default()
            },
        });
    }
}

/// One line-ending chooser, labelled.
///
/// ★ The list is [`ALL_ENDINGS`] rather than a literal written at each call
/// site, so the two choosers cannot come to offer different sets — and so that
/// an ending the engine learns to draw appears in both by editing one constant
/// whose exhaustiveness the compiler checks.
fn ending_chooser(ui: &mut Ui, label: &str, id: &str, value: &mut LineEnding) {
    ui.horizontal(|ui| {
        ui.label(label);
        egui::ComboBox::from_id_salt(id)
            .selected_text(t::markup_line_ending_name(*value))
            .show_ui(ui, |ui| {
                for ending in ALL_ENDINGS {
                    ui.selectable_value(value, ending, t::markup_line_ending_name(ending));
                }
            });
    });
}

/// Every line ending pdfcer can draw, in the order the choosers offer them.
///
/// ★★ `annot_author::LineEnding` has no `ALL` of its own — unlike `ArrowForm`,
/// which the ce-dimension panel iterates — so this list is written here, and
/// `the_ending_list_covers_every_variant_the_engine_has` is what stops it
/// drifting: that test `match`es an exhaustive set of variants with **no
/// wildcard**, so an ending the engine gains fails to compile here rather than
/// quietly going missing from the chooser.
///
/// `None` first because it is the state an operator reaches for when they want
/// a plain line, and because Table 176 lists it first.
const ALL_ENDINGS: [LineEnding; 3] = [
    LineEnding::None,
    LineEnding::OpenArrow,
    LineEnding::ClosedArrow,
];

/// The constant opacity, `/CA`.
///
/// ★★ **This is the control `NO_SURFACE.md` recorded as "blocked on the engine"
/// for weeks, and the blocker was false.** `set_markup_style` has taken an
/// opacity since it shipped and writes `/CA` clamped to `0.0..=1.0`; the row
/// that said otherwise was a claim about a repository this project does not
/// build, and it could not fail a test. See `NO_SURFACE.md` §1b.
///
/// Shown as a **percentage**, because that is the unit every other application
/// an operator has used states opacity in, and `/CA`'s own `0.0..=1.0` is a
/// file-format detail they should never meet.
fn opacity_row(
    ui: &mut Ui,
    current: Current,
    target: &crate::canvas::selection::annot::AnnotTarget,
    actions: &mut Vec<Action>,
) {
    let existing = current.alpha;
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let mut percent = (existing.unwrap_or(1.0) * 100.0).round().clamp(0.0, 100.0) as u8;
    ui.horizontal(|ui| {
        ui.label(t::markup_opacity_label());
        let response = ui.add(
            egui::DragValue::new(&mut percent)
                .range(0..=100)
                .speed(1.0)
                .suffix(t::markup_opacity_suffix()),
        );
        if response.drag_stopped() || response.lost_focus() {
            actions.push(Action::SetMarkupStyle {
                page: target.page,
                id: target.id,
                style: MarkupStyle {
                    opacity: Some(StyleEdit::Set(f64::from(percent) / 100.0)),
                    ..MarkupStyle::default()
                },
            });
        }
        if existing.is_some() && ui.button(t::markup_clear()).clicked() {
            actions.push(Action::SetMarkupStyle {
                page: target.page,
                id: target.id,
                style: MarkupStyle {
                    opacity: Some(StyleEdit::Clear),
                    ..MarkupStyle::default()
                },
            });
        }
    });
}

/// An annotation's `/C` or `/IC` as something a swatch can show, plus whether
/// showing it cost a conversion.
///
/// # ★★★ The CMYK arm, and the position it replaced
///
/// This function used to answer `None` for CMYK, with an argument worth keeping
/// because half of it is still right:
///
/// > `None` for anything that is not RGB, and that is honest rather than lossy:
/// > §12.5.2 lets `/C` be a 0-, 1-, 3- or 4-component array, and a swatch
/// > showing a CMYK mark's *converted* colour would be a control whose readback
/// > is a conversion the operator never asked for — pick it up, put it down
/// > unchanged, and the file now says something different.
///
/// ★ **What that shipped was worse than the thing it avoided.** A CMYK mark —
/// not rare on a CAD sheet, where a plotter-bound producer writes process colour
/// — got a **default black swatch and no Clear**. So the panel told the operator
/// their coloured mark had no colour, which is not a smaller misstatement than
/// an approximate one; and it withheld Clear, which is the one operation on a
/// CMYK `/C` that loses nothing at all.
///
/// ★ **And the feared round trip is not a thing this control can do.** egui's
/// colour button reports `changed()` only when the value actually moves, so
/// *pick it up and put it down unchanged* raises no action and writes no byte.
///
/// # ⚠ The narrowing, disclosed rather than hidden
///
/// A restyle raised from a swatch fed by this function writes
/// `Color::Rgb`, because `color_edit_button_srgb` produces sRGB and nothing
/// else. **On a mark whose `/C` or `/IC` was CMYK that NARROWS the colour
/// space** — four components in the file become three, and a colour-managed
/// consumer downstream will separate the result differently than the original
/// process values. The engine's own posture is that a narrowing conversion is
/// disclosed rather than performed quietly, so [`Swatch::narrowed`] carries the
/// fact up to [`section`] and [`t::markup_colour_narrowed`] is the sentence the
/// operator reads **before** they pick, not after.
///
/// Grey is **not** narrowing and is not flagged: `Gray(v)` and `Rgb(v, v, v)`
/// are the same ink, exactly, in both directions.
///
/// # The conversion itself
///
/// The naïve `1 - min(1, x + k)` per channel — the same one §8.6.4.4 states as
/// the default `DeviceCMYK` → `DeviceRGB` transform when no colour management is
/// in play. It is an approximation and this shell says so; it is not a place to
/// invent an ICC pipeline for a 16-pixel square.
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
fn swatch_of(color: Option<&Color>) -> Swatch {
    let byte = |v: f64| (v.clamp(0.0, 1.0) * 255.0).round() as u8;
    let Some(color) = color else {
        return Swatch::default();
    };
    match *color {
        Color::Rgb(r, g, b) => Swatch {
            rgb: Some([byte(r), byte(g), byte(b)]),
            narrowed: false,
        },
        Color::Gray(v) => Swatch {
            rgb: Some([byte(v), byte(v), byte(v)]),
            narrowed: false,
        },
        Color::Cmyk(c, m, y, k) => Swatch {
            rgb: Some([
                byte(1.0 - (c + k).min(1.0)),
                byte(1.0 - (m + k).min(1.0)),
                byte(1.0 - (y + k).min(1.0)),
            ]),
            narrowed: true,
        },
    }
    // ★ EXHAUSTIVE, with no wildcard, and deliberately so. `Color` is not
    // `#[non_exhaustive]`, so a fourth device space would fail to compile here
    // rather than fall into a catch-all that shows the operator a default black
    // square. The old wildcard was what let CMYK sit unhandled and unnoticed
    // for the life of this module.
}

#[cfg(test)]
mod tests;
