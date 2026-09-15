//! # `panels::properties::markup` — restyling a markup that is already on the
//! page
//!
//! The panel half of `FEATURES.md`'s Phase 1 row *"Format tab contents —
//! colour, width, style, opacity for a **placed** markup"*: the surface that
//! reaches `pdfcer-core`'s `set_markup_style`.
//!
//! ## Why the PANEL and not the Format tab
//!
//! `RIBBON_IA.md` §5.8 settles it, in the operator's own words:
//!
//! > The division of labour: the **tab** carries what a user changes *while
//! > working* — colour, width, style, align, delete. The **panel** carries
//! > everything, including the read-only facts … The panel is also where the
//! > **editable geometry** lives.
//!
//! So the panel is where the complete set goes, and it is also the cheaper
//! surface by a wide margin: a ribbon band cannot hold a colour picker or a
//! slider without a new `Item::custom` kind and a renderer for it, which is
//! shell work in a crate that must never learn what a PDF is (R7). The tab's
//! slice is a smaller job that reads the same actions.
//!
//! ## Every control is `None` unless the operator touched it
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
//! ## Where the style verb can reach, and how that is decided
//!
//! `set_markup_style` begins by calling `annot_author::spec_from_dict`, whose
//! `match` reads a fixed set of `/Subtype`s — `Square`, `Circle`, `Line`,
//! `Ink`, `Polygon`, `PolyLine`, `Highlight`, `Underline`, `StrikeOut`,
//! `Squiggly` — with every other name answering
//! `SpecReadError::UnsupportedSubtype`. `AnnotKind::Markup` is wider than that:
//! its own doc covers *"a shape, a note, a stamp, a text markup"*. So the kind
//! check alone would draw a live colour swatch and a live opacity spinner over
//! a `/Text`, a `/FreeText` or a `/Stamp`, and every press would be refused
//! with `EditError::MarkupSpec` — the *visible control, silently inert* class
//! this project forbids by name.
//!
//! ### Reachability is asked of `spec_from_dict`, never of a subtype list
//!
//! A `matches!(subtype, "Square" | "Circle" | …)` beside the kind check goes
//! stale the day the engine learns an eleventh subtype, and it goes stale
//! **silently**, in the direction that withholds a control that would have
//! worked. [`Current::read`] already calls `spec_from_dict`, so
//! [`Current::reach`] carries that verdict forward: this section and the verbs
//! answer the same question through the same function, and an engine that grows
//! a subtype grows this panel with it and no shell change.
//!
//! ### Two verbs, one routing decision
//!
//! A `/Text`'s icon and colour and a `/Stamp`'s colour are changeable, but
//! through a **second engine verb with a second reader and a second style
//! struct** — `set_text_annot_style`. [`textannot`] is that route and its
//! header carries the account, including why `/FreeText` is refused and on what
//! grounds.
//!
//! ⇒ Nothing here goes through `set_markup_style` on their behalf. The two are
//! separated by [`Reach`], an enum whose arms the compiler makes exhaustive,
//! because a routing decision that can be got wrong silently is exactly the
//! defect this seam exists to prevent.
//!
//! ### The refusal SAYS something
//!
//! R9 makes an unavailable capability render nothing. It does not make the
//! panel go silent: the heading and the subtype line still draw, because
//! something *is* selected, and a heading over an empty space reads as a bug.
//! [`t::markup_not_restylable`] names what is still possible — move, resize,
//! delete, edit the note — and its doc comment records the engine verb each of
//! those four claims is checked against.
//!
//! ## Two properties this panel offers that the author-time path does not
//!
//! - **Fill (`/IC`) is offered on restyle and never at author time.**
//!   `canvas::markup::spec` writes `interior: None` on purpose — *"a filled
//!   comment shape hides the drawing it is a comment about, which on a CAD
//!   sheet is the whole content under it"* — and this module does not go near
//!   that. A default and a prohibition are different acts, and letting one
//!   stand in for the other is how a sensible default becomes a capability
//!   nobody can reach. Acrobat's shape tools all offer fill and all default it
//!   to none; that is the shape matched here.
//!
//! - **Line endings (`/LE`) are offered on a `/Line`.** A `/Line` with
//!   `/LE [/None /None]` is the same `/Subtype`, with the same geometry,
//!   reached by the same verb, and §12.5.6.7 treats its endings as *style* in
//!   exactly the way `/C` and `/BS` `/W` are style — which is why
//!   `MarkupStyle::endings` sits beside them in one struct rather than in a
//!   reshape. A leader with a head at one end only is among the commonest
//!   annotations on a drawing sheet.
//!
//! - **A ce dimension is NOT restyled here.** [`super::dimension`] owns those,
//!   through `set_dimension_style` — a different verb with a different model,
//!   and `AnnotKind` carries the distinction **in the type** so this section's
//!   guard is a `match` the compiler checks. Restyling a ce dimension as
//!   ordinary markup regenerates it as a bare line with its label and witness
//!   lines gone. Rule 15 in one sentence: never write a bare dimension.
//!
//! ## WHICH subtype takes WHICH property is the ENGINE's question
//!
//! This shell holds no copy of that list:
//!
//! > *"That list is the engine's to know. The first subtype that gains or loses
//! > a border is the day our copy is wrong and nothing tells us."*
//!
//! `pdfcer-core`'s `edit::MarkupStyleSupport::for_subtype` answers it, with
//! `takes_border`, `takes_interior` and `takes_endings`. [`Current::support`]
//! holds the answer and the rows read it. **A comment saying a list was checked
//! against the engine source is a comment that ages; a call cannot.**
//!
//! ⚠ **What this does NOT cover.** *"What IS this mark's width?"* is read off
//! the `MarkupSpec` arm, because only `MarkupSpec::Square` has a `border_width`
//! field and the engine publishes no API that would answer it. A **value** read
//! and a **capability** question are different questions with different owners;
//! `canvas::annotnodes`' header draws the same line for painting.
//!
//! ### The refusal, which is the other half of the same contract
//!
//! `EditError::StylePropertyNotApplicable { id, subtype, property }` is raised
//! by `set_markup_style` **before** anything is regenerated. So the predicate
//! above shapes this panel and the refusal catches a shell that drifted anyway
//! — belt and braces, and the reason [`Current::reach`]'s neighbours are not
//! enough on their own. It reaches the operator through the channel every
//! engine refusal uses, `app::actions::funnel::vector_edit`'s `Err` arm: the
//! decline sentence on screen, the engine's own words into `PDFCER_DIAG`.
//! Nothing here builds a second route, because `check-ui-strings.sh`'s
//! exclusion 3 forbids one in as many words.
//!

use egui::Ui;
use pdfcer_core::annot_author::{Color, LineEnding, MarkupSpec};
use pdfcer_core::edit::MarkupStyleSupport;

use crate::app::actions::Action;
use crate::app::state::OpenDoc;
use crate::canvas::selection::annot::AnnotKind;
use crate::text::panels::properties as t;

// `Reach` lives in the SUBMODULE and is used here. See its own doc for why:
// it is the seam between two verbs rather than a property of either, and R2
// gave it the file with the room. The parent still owns the routing `match`.
use textannot::Reach;

use rows::{colour_row, dash_row, endings_row, fill_row, opacity_row, width_row};

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
/// Wider than the Format band's `DASH_WIDTH` (88), and deliberately so: this
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
    // Markup only. A ce dimension is `super::dimension`'s, and the
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

    // **Locked is R9's "temporarily unavailable", so it GREYS with a reason
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

    // Read from the SESSION every frame, never from a cache, and read
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

    // **WHICH VERB REACHES THIS MARK — one `match`, and the compiler checks
    // it.**
    //
    // Two verbs over two spec families, and the arms below are the whole
    // routing decision:
    //
    // | arm | verb | reader |
    // |---|---|---|
    // | `Markup` | `set_markup_style` | `annot_author::spec_from_dict` |
    // | `TextAnnot` | `set_text_annot_style` | `annot_author::text_spec_from_dict` |
    // | `TextBoxWithheld` | — | this shell declines; see [`Reach`] |
    // | `Neither` | — | both readers refused |
    //
    // A `match` rather than two `if`s, and that is a correctness requirement
    // rather than a style preference: a third verb, or a fourth face, arrives
    // here as a non-exhaustive-match error instead of as a mark that quietly
    // falls through to the refusal sentence. **A routing decision that can be
    // got wrong silently produces live controls whose every press is refused.**
    //
    // Both predicates are the engine's own readers succeeding — the SAME
    // calls the two verbs make — rather than subtype lists written here. A list
    // is correct today and wrong on the day the engine adds a subtype, and
    // wrong in the silent direction: withholding a control that had started
    // working.
    match &current.reach {
        Reach::Markup => markup_rows(ui, &current, &target, actions),
        Reach::TextAnnot(reading) => textannot::rows(ui, reading, &target, actions),
        // A `/FreeText`. `set_text_annot_style` would take it and this
        // shell will not send it — `textannot`'s header carries the
        // measurement, and the short form is that the engine's reader always
        // reports `multiline: false` and this verb, unlike `set_markup_note`,
        // does not measure the true value before re-baking. Every text box this
        // shell places is `multiline: true`, so a colour change would unwrap
        // the operator's callout with nothing on screen to say so.
        //
        // ⇒ A sentence of its own rather than the generic one, because the
        // claim is different: the capability is not missing, it is declined.
        Reach::TextBoxWithheld => {
            ui.label(
                egui::RichText::new(
                    crate::text::panels::textannotstyle::markup_text_box_not_restylable(),
                )
                .small()
                .weak(),
            );
        }
        Reach::Neither => {
            ui.label(
                egui::RichText::new(t::markup_not_restylable())
                    .small()
                    .weak(),
            );
        }
    }
    ui.separator();
    true
}

/// **The rows `set_markup_style` can commit**, behind one arm of [`section`]'s
/// `match`.
///
/// Extracted rather than left inline purely so the routing `match` above
/// reads as four one-line arms. A `match` whose first arm is forty lines and
/// whose others are three is a `match` a reader stops seeing as a routing
/// decision, which is the one thing this one has to remain.
fn markup_rows(
    ui: &mut Ui,
    current: &Current,
    target: &crate::canvas::selection::annot::AnnotTarget,
    actions: &mut Vec<Action>,
) {
    colour_row(ui, current, target, actions);
    fill_row(ui, current, target, actions);
    // The narrowing disclosure sits under BOTH swatches and above the rest,
    // because it qualifies them and a caveat below the thing it qualifies
    // arrives after the operator has drawn their conclusion. It is absent —
    // not greyed, not blank — for the overwhelming
    // majority of marks, whose `/C` is RGB or grey and costs no conversion.
    if current.colour.narrowed || current.interior.narrowed {
        ui.label(
            egui::RichText::new(t::markup_colour_narrowed())
                .small()
                .weak(),
        );
    }
    width_row(ui, current, target, actions);
    // Directly under the width, because the two are one subject — *what the
    // line looks like* — and the Format tab's band puts them adjacent for the
    // same reason. A panel is read top to bottom, and an operator setting a
    // mark's linework should not have to read past the arrowheads to finish.
    dash_row(ui, current, target, actions);
    endings_row(ui, current, target, actions);
    opacity_row(ui, current, target, actions);

    ui.label(egui::RichText::new(t::markup_note()).small().weak());
}

/// What the selected mark's dictionary currently says, in the terms this
/// section can change — **and whether it can change any of them at all**.
///
/// Five terms — colour, fill, width, opacity and the two line endings — plus a
/// sixth field, [`Self::reach`], which is not a term at all but the answer to
/// WHICH VERB the other five are reachable through.
///
/// # Why it is read through `spec_from_dict` and not from `annot::Annotation`
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
/// Its refusals are `None` here rather than an error, and that is honest
/// rather than lax. `SpecReadError`'s own doc says every variant is *"a refusal
/// to guess"* — an unsupported `/Subtype`, or geometry that is missing or is
/// not something pdfcer models.
///
/// ⚠ **A refusal here means the mark cannot be given a colour either**, not
/// merely that the one it has cannot be shown. `set_markup_style` opens by
/// calling this same function and propagating its error with `?`. So a swatch
/// drawn over a refused mark would not be uninformative — it would be unable to
/// commit. See [`Self::reach`] and the module header.
/// ⚠ **`Clone`, not `Copy`** — [`Self::reach`] carries a
/// [`textannot::Reading`] on its `TextAnnot` arm, which carries a `StickyIcon`,
/// which has an owning `Other(Vec<u8>)` variant. The frame reads one of these
/// and hands it out by reference.
#[derive(Debug, Clone)]
struct Current {
    /// **Which style verb reaches this mark, if either does.**
    ///
    /// ⚠ **Not a `bool`, and the reason is the whole of this field.** A `bool`
    /// can only answer *"is it the verb I know about?"*, and a `false` there
    /// means both *"nothing is possible"* and *"nothing THIS verb can do" —
    /// with nothing in the type to say which. For a sticky note the second is
    /// true and the first is false, so a `bool` would produce the refusal
    /// sentence over a mark whose icon and colour are changeable.
    ///
    /// [`Reach`] says which, in a type whose arms the compiler makes
    /// exhaustive.
    ///
    /// It is a field rather than a recomputation because the answer is already
    /// in hand — the read below has to call the engine's readers regardless —
    /// and two calls to one function is how the panel and the verb come to
    /// disagree about the same annotation.
    reach: Reach,
    /// **Which of these properties this `/Subtype` can take at all — the
    /// ENGINE's answer, not this module's.**
    ///
    /// `pdfcer-core`'s `MarkupStyleSupport::for_subtype` is asked once, off the
    /// annotation's own `/Subtype`, and [`fill_row`], [`width_row`] and
    /// [`endings_row`] consult it before drawing anything.
    ///
    /// ⚠ **This shell keeps no subtype list of its own.** Which shapes carry
    /// an `/IC`, which have a border to widen and which have `/LE` is four
    /// subtypes' worth of the engine's knowledge, and a copy of it here is
    /// correct until the first subtype gains or loses one. The row is
    /// **absent** when the answer is `false`, for the R9 reason, and a cloud is
    /// handled correctly because `for_subtype(b"Polygon")` says so rather than
    /// because this module
    /// remembered that a revision cloud is a `/Polygon` in the file.
    ///
    /// It is not the same question as [`Self::reach`] and neither
    /// subsumes the other: `reach` asks *which verb can read this mark at all*,
    /// this asks *which of that verb's properties mean anything*. A `/Highlight` answers **yes** to the first
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
    /// A **value**, from the `MarkupSpec` arm that has one. Whether the row is
    /// offered is `support.takes_border`; see the module header's distinction
    /// between a value read and a capability question.
    width: Option<f64>,
    /// `/CA`, the constant opacity.
    alpha: Option<f64>,
    /// **`/BS` `/S` and `/D` — the border's line style**, as the chooser shows
    /// it.
    ///
    /// Read off the **dictionary**, not off the spec, and that is the same
    /// exception `/CA` is rather than a departure from this struct's rule. A
    /// dash cuts across `MarkupSpec`'s variants rather than belonging to any
    /// one of them, so `pdfcer-core` carries it in `AppearanceOptions` beside
    /// the spec instead of inside it and `spec_from_dict` returns none. The
    /// engine's own reader is `pub(crate)`, so
    /// [`crate::canvas::markup::linestyle::read`] is this shell's copy of it —
    /// declared as a copy in that function's header, with the bound on what a
    /// divergence can cost written down beside it.
    ///
    /// It therefore travels **through** `from_spec` rather than being derived
    /// in it, exactly as `alpha` and `endings_key_present` do, and for the same
    /// reason: it is a fact about the dictionary that the spec reader does not
    /// carry.
    dash: crate::canvas::markup::linestyle::DashReading,
    /// `/LE`, the pair of line endings the mark currently draws.
    ///
    /// Also a value, and `MarkupSpec::Line` is the only arm carrying one — a
    /// fact the compiler checks. `support.takes_endings` is what decides
    /// whether the choosers appear.
    endings: Option<(LineEnding, LineEnding)>,
    /// **Whether `/LE` is actually IN the dictionary**, as distinct from
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
/// The second field is the whole reason this is a struct rather than an
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

/// Even "nothing to show" asks the engine what the properties are.
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
            reach: Reach::Neither,
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
        use pdfcer_core::annot_author::{spec_from_dict, text_spec_from_dict};
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
        // `ObjectGraph::resolve` comes from the TRAIT, so it has to be in
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

        // **The one call**, and its verdict is carried rather than
        // recomputed. `.ok().as_ref()` turns the refusal into the `None` that
        // [`Self::from_spec`] reads as *"the style verb does not reach this
        // mark"* — which is precisely what a `SpecReadError` means to
        // `set_markup_style`, since that verb's next line after this same call
        // is `?`.
        // **The capability question, asked of the engine, off the same key
        // the engine itself reads.** `set_markup_style` derives its
        // `MarkupStyleSupport` from `/Subtype` on the annotation dictionary
        // and refuses a property the answer excludes before anything is
        // regenerated. Reading the same key through the same
        // function is what makes a row drawn here and a call refused there
        // impossible to disagree.
        let subtype = dict
            .get(b"Subtype")
            .map(|o| graph.resolve(o))
            .and_then(Object::as_name)
            .map_or_else(Vec::new, |n| n.as_bytes().to_vec());
        let support = MarkupStyleSupport::for_subtype(&subtype);

        // Presence, not value — see `Self::endings_key_present`. This is
        // the one fact `spec_from_dict` erases, and the *Clear the setting*
        // button exists to act on it.
        let endings_key_present = dict
            .get(b"LE")
            .map(|o| graph.resolve(o))
            .is_some_and(|o| !matches!(o, Object::Null));

        // Read BEFORE `spec_from_dict` and carried across its refusal is not
        // needed here — a mark the spec reader refuses gets no rows at all
        // (`Self::reach`) — but it is read off the dictionary for the same
        // reason `/CA` is: the spec has no dash in it to read.
        let dash = crate::canvas::markup::linestyle::read(&graph, dict);

        // **Both readers, in order, and the second only when the first
        // refuses.** `spec_from_dict`'s arms and `text_spec_from_dict`'s are
        // disjoint — no `/Subtype` is read by both — so the order is a saving
        // rather than a precedence rule, and the `?`-shaped fallback below
        // reads as one because of it.
        //
        // The second call is what makes [`Reach::TextAnnot`] reachable, and it
        // is the same call `set_text_annot_style` opens with. The panel and the
        // verb ask the same function about the same dictionary, which is what
        // makes a row drawn here and a call refused there impossible to
        // disagree.
        let markup = spec_from_dict(&graph, dict).ok();
        let text = markup
            .is_none()
            .then(|| text_spec_from_dict(&graph, dict).ok())
            .flatten();

        // **The stamp's label parameters, and the ONE call that needs the
        // session rather than the spec.**
        //
        // A stamp's label size is not in `TextAnnotSpec` and cannot be — it is
        // recovered by parsing the annotation's `/AP` `/N` content stream for
        // its `Tf`, or by reading `/DA` when one is present. So it needs the
        // object graph *and* the R45 staging buffer behind it, which is why
        // this is the session method and not `annot::stamp_label_parameters_in`
        // with a `StreamSource::Contiguous(doc.bytes())`: a stamp authored this
        // session has its appearance staged, not on disk, and the contiguous
        // form would answer `None` for exactly the stamp the operator just
        // placed and is now looking at.
        //
        // `.ok().flatten()` collapses two different `None`s that mean the
        // same thing HERE and nothing else: `Err(AnnotationNotFound)` — the id
        // is not an annotation on any page of this session — and `Ok(None)`,
        // which is the engine's honest answer for a stamp whose appearance
        // shows no text (Acrobat's custom stamps are artwork). Both produce an
        // absent size row, which is the correct outcome for both, and the panel
        // has nothing different to say about them.
        let stamp_label = doc.session.stamp_label_parameters(id).ok().flatten();

        Self::from_spec(
            markup.as_ref(),
            text.as_ref()
                .map(|spec| textannot::Reading::of(spec).map(|r| r.with_stamp_label(stamp_label))),
            support,
            alpha,
            dash,
            endings_key_present,
        )
    }

    /// The pure half: everything this section shows, derived from the spec the
    /// engine read (or from its absence).
    ///
    /// # Why it is split out from [`Self::read`]
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
        text: Option<Option<textannot::Reading>>,
        support: MarkupStyleSupport,
        alpha: Option<f64>,
        dash: crate::canvas::markup::linestyle::DashReading,
        endings_key_present: bool,
    ) -> Self {
        let Some(spec) = spec else {
            // Note what is NOT carried across: `alpha`. `/CA` reads fine off
            // any annotation dictionary, so it would be easy to keep — and it
            // would be a value shown under a heading whose every control is
            // about to be withheld. Neither style verb writes `/CA` for a mark
            // `set_markup_style` refuses, so the value it would display is
            // decoration.
            return Self {
                // The three-way answer the second reader gives, and the
                // nesting is load-bearing rather than awkward: the OUTER
                // `Option` is *"did `text_spec_from_dict` produce a spec?"* and
                // the INNER is *"does this shell serve that face?"*. Collapsing
                // them to one `Option` would merge a `/FreeText` — which the
                // engine reads perfectly and this shell declines — with a
                // `/Link`, which neither reader touches, and the two owe the
                // operator different sentences.
                reach: match text {
                    Some(Some(reading)) => Reach::TextAnnot(reading),
                    Some(None) => Reach::TextBoxWithheld,
                    None => Reach::Neither,
                },
                ..Self::default()
            };
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
            // ⚠ **The `None` here is not what withholds the width row.**
            // `support.takes_border` decides that, and it is the engine's to
            // say. `set_markup_style` answers
            // `EditError::StylePropertyNotApplicable` for a width on a text
            // markup before touching the file, so a row drawn from this arm
            // could not commit anyway.
            MarkupSpec::TextMarkup { color, .. } => (swatch_of(Some(color)), None),
            // `MarkupSpec` is `#[non_exhaustive]`. A kind this build does
            // not know the shape of gets no readback and no Clear, which is
            // the same answer a refused parse gets and for the same reason.
            _ => (Swatch::default(), None),
        };
        // The interior VALUE, read off the spec arm because that is where a
        // value lives — only these four arms have an `interior` field and the
        // compiler checks which.
        //
        // ⚠ **This `match` is not the capability answer.** Whether a subtype
        // has an `/IC` at all is `support.takes_interior`, and [`fill_row`] is
        // what asks it; this arm list supplies the colour and stops there. A
        // comment recording that a list was checked against the engine source
        // is a comment that goes stale the first time the engine changes and
        // nothing says so — which includes the cloud: `for_subtype(b"Polygon")`
        // is what says a revision cloud has an `/IC`, rather than this module
        // remembering that a cloud is a `/Polygon` in the file.
        let interior = match spec {
            MarkupSpec::Square { interior, .. }
            | MarkupSpec::Circle { interior, .. }
            | MarkupSpec::Polygon { interior, .. }
            | MarkupSpec::Cloud { interior, .. } => swatch_of(interior.as_ref()),
            _ => Swatch::default(),
        };
        Self {
            reach: Reach::Markup,
            support,
            colour,
            interior,
            width,
            alpha,
            // Carried through untouched, like `alpha`. There is no arm to
            // derive it from — a dash is not in `MarkupSpec` at all — and no
            // decision to take about it here: the chooser's absence for a
            // borderless subtype is `support.takes_border`'s answer, and the
            // reading itself is the dictionary's.
            dash,
            // The pair `/Line` draws, from the one arm that has one. That
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
    // WHETHER a row is drawn — one question, one place, testable
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
    const fn offers_fill(&self) -> bool {
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
    const fn offers_width(&self) -> bool {
        self.support.takes_border && self.width.is_some()
    }

    /// Whether the Line style row draws.
    ///
    /// **Purely the engine's answer, with no second term** — unlike
    /// [`Self::offers_width`], which also asks whether a width was read. The
    /// asymmetry is real: a width has to be *shown* in a spinner, so a mark
    /// whose width this build could not read has nothing to put in one; a line
    /// style always has a value, because *solid* is a state rather than an
    /// absence and [`crate::canvas::markup::linestyle::read`] is total — every
    /// dictionary answers it, including one carrying no `/BS`.
    ///
    /// ⇒ So the only question left is the engine's *does this subtype have a
    /// border?*, which is the same predicate `set_markup_style` guards
    /// `style.dash` with. A row drawn here cannot produce that refusal.
    const fn offers_dash(&self) -> bool {
        self.support.takes_border
    }

    /// Whether the two ending choosers draw.
    const fn offers_endings(&self) -> bool {
        self.support.takes_endings && self.endings.is_some()
    }

    /// Whether the *Clear the setting* button draws under them.
    ///
    /// Strictly narrower than [`Self::offers_endings`]: there has to be a
    /// chooser to sit under **and** a `/LE` in the file to take out.
    const fn offers_endings_clear(&self) -> bool {
        self.offers_endings() && self.endings_key_present
    }
}

/// An annotation's `/C` or `/IC` as something a swatch can show, plus whether
/// showing it cost a conversion.
///
/// # Why CMYK gets a converted swatch rather than none
///
/// §12.5.2 lets `/C` be a 0-, 1-, 3- or 4-component array, so the tempting
/// answer is `None` for anything that is not RGB — on the grounds that showing
/// a CMYK mark's *converted* colour is a readback the operator never asked for.
///
/// **That answer is worse than the thing it avoids.** A CMYK mark is not rare
/// on a CAD sheet, where a plotter-bound producer writes process colour, and
/// `None` gives it a **default black swatch and no Clear**: the panel would
/// tell the operator their coloured mark has no colour, which is not a smaller
/// misstatement than an approximate one, and it would withhold Clear, the one
/// operation on a CMYK `/C` that loses nothing at all.
///
/// **And the feared round trip is not a thing this control can do.** egui's
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
    // EXHAUSTIVE, with no wildcard, and deliberately so. `Color` is not
    // `#[non_exhaustive]`, so a fourth device space would fail to compile here
    // rather than fall into a catch-all that shows the operator a default black
    // square. A wildcard arm here is what lets a colour space sit unhandled and
    // unnoticed.
}

// The seven per-property rows. `rows.rs`'s header carries the seam and the
// contract; the short version is that a row knows its own control and nothing
// else, and everything that decides WHICH rows exist lives here.
mod rows;

mod textannot;

#[cfg(test)]
mod tests;
