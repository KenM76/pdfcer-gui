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

use egui::Ui;
use pdfcer_core::annot_author::{Color, LineEnding, MarkupSpec};
use pdfcer_core::edit::{MarkupStyle, StyleEdit};

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
#[derive(Debug, Clone, Copy, Default)]
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
    /// `/C`, as a swatch can show it, and whether showing it cost a conversion.
    colour: Swatch,
    /// **Whether this shape has an interior to fill at all**, which is a
    /// property of its `MarkupSpec` arm rather than of its dictionary: a
    /// `/Square`, `/Circle`, `/Polygon` and a cloud carry `/IC`; a `/Line`, an
    /// `/Ink`, a `/PolyLine` and a text markup have no interior for one to
    /// mean anything in.
    ///
    /// The Fill row is absent when this is `false` — the same shape
    /// [`width_row`] already had for a highlight, and for the same R9 reason.
    has_interior: bool,
    /// `/IC`, when [`Self::has_interior`]. `None` inside means *no fill*, which
    /// is the state this shell authors and the state Acrobat defaults to.
    interior: Swatch,
    /// `/BS` `/W`, the border width in points.
    width: Option<f64>,
    /// `/CA`, the constant opacity.
    alpha: Option<f64>,
    /// `/LE`, the pair of line endings — `Some` for a `/Line` and nothing else,
    /// which is exactly the set §12.5.6.7 gives them meaning for.
    endings: Option<(LineEnding, LineEnding)>,
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
        Self::from_spec(spec_from_dict(&graph, dict).ok().as_ref(), alpha)
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
    fn from_spec(spec: Option<&MarkupSpec>, alpha: Option<f64>) -> Self {
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
            // `/QuadPoints` and there is nothing to stroke. The width row
            // still draws, with the engine's own default showing, because
            // `set_markup_style` accepts a width for it and simply has
            // nothing to apply it to; offering the control and having it do
            // nothing would be the inert control this project forbids, so
            // `width_row` asks this value and hides itself when it is
            // `None`.
            MarkupSpec::TextMarkup { color, .. } => (swatch_of(Some(color)), None),
            // `MarkupSpec` is `#[non_exhaustive]`. A kind this build does
            // not know the shape of gets no readback and no Clear, which is
            // the same answer a refused parse gets and for the same reason.
            _ => (Swatch::default(), None),
        };
        // ★★ The interior slot is read off the SPEC ARM, not off a subtype
        // string, and the two are not the same question: a revision cloud is a
        // `/Polygon` in the file and a `MarkupSpec::Cloud` here, and a fill row
        // hidden by a `"Polygon" | "Square" | "Circle"` string list would have
        // been correct for the polygon and wrong for the cloud drawn with the
        // same tool. `apply_markup_style` applies `style.interior` to exactly
        // these four arms — checked against the engine source, not assumed.
        let interior = match spec {
            MarkupSpec::Square { interior, .. }
            | MarkupSpec::Circle { interior, .. }
            | MarkupSpec::Polygon { interior, .. }
            | MarkupSpec::Cloud { interior, .. } => Some(swatch_of(interior.as_ref())),
            _ => None,
        };
        Self {
            restylable: true,
            colour,
            has_interior: interior.is_some(),
            interior: interior.unwrap_or_default(),
            width,
            alpha,
            // ★ `/Line` alone. §12.5.6.7 gives `/LE` a meaning on `/Line`,
            // `/PolyLine` and `/FreeText`; `MarkupStyle::endings` says "`Line`
            // only" and `apply_markup_style` sets it on the `Line` arm and on
            // no other, so offering it anywhere else would be a control the
            // engine drops on the floor.
            endings: match spec {
                MarkupSpec::Line { endings, .. } => Some(*endings),
                _ => None,
            },
        }
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
/// # ★★ Absent for a shape with no interior
///
/// A `/Line`, an `/Ink`, a `/PolyLine` and a text markup have no interior for
/// `/IC` to mean anything in, and `apply_markup_style` does not read
/// `style.interior` on those arms — so a Fill control there would be drawn,
/// live, and dropped on the floor. That is the same defect this session came to
/// fix, one control down, and R9's answer is the same: the row is absent, the
/// same way [`width_row`] is absent for a highlight.
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
    if !current.has_interior {
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
/// # ★ `/Line` only, and absent otherwise
///
/// [`Current::endings`] is `Some` for a `MarkupSpec::Line` and nothing else,
/// which matches `apply_markup_style` exactly: it reads `style.endings` on the
/// `Line` arm and ignores it everywhere else. A chooser on a polygon would be
/// live, committable, and would change nothing — R9 says absent.
fn endings_row(
    ui: &mut Ui,
    current: Current,
    target: &crate::canvas::selection::annot::AnnotTarget,
    actions: &mut Vec<Action>,
) {
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
                endings: Some(chosen),
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
mod tests {
    use super::*;

    // ★ `page_tree::Rect`, not `annot_author::Rect`. `annot_author` imports the
    // type privately, so the path that reads naturally is not a path that
    // resolves — the one place in this module where the engine's own module
    // layout leaks.
    use pdfcer_core::page_tree::Rect;

    /// A `MarkupSpec::Square`, with whatever interior the caller wants.
    fn square(interior: Option<Color>) -> MarkupSpec {
        MarkupSpec::Square {
            rect: Rect::from_corners(0.0, 0.0, 10.0, 10.0),
            border: Some(Color::Rgb(1.0, 0.0, 0.0)),
            interior,
            border_width: 2.0,
            border_effect: None,
        }
    }

    /// A `MarkupSpec::Line` with the given endings.
    fn line(endings: (LineEnding, LineEnding)) -> MarkupSpec {
        MarkupSpec::Line {
            start: (0.0, 0.0),
            end: (10.0, 10.0),
            color: Color::Gray(0.0),
            width: 1.0,
            endings,
        }
    }

    /// Grey resolves to an equal-component swatch, and RGB round-trips —
    /// neither of them flagged as narrowing.
    ///
    /// Grey is not flagged because it is **lossless** in both directions —
    /// `Gray(v)` and `Rgb(v, v, v)` are the same ink — where CMYK is not, which
    /// is the distinction `swatch_of`'s own docs draw.
    #[test]
    fn a_swatch_shows_grey_and_rgb_without_calling_them_converted() {
        let red = swatch_of(Some(&Color::Rgb(1.0, 0.0, 0.0)));
        assert_eq!(red.rgb, Some([255, 0, 0]));
        assert!(!red.narrowed);

        let black = swatch_of(Some(&Color::Gray(0.0)));
        assert_eq!(black.rgb, Some([0, 0, 0]));
        assert!(!black.narrowed);

        assert_eq!(
            swatch_of(Some(&Color::Gray(1.0))).rgb,
            Some([255, 255, 255])
        );
        assert_eq!(swatch_of(None).rgb, None);
    }

    /// ★★★ **A CMYK `/C` shows, and says it is a conversion.**
    ///
    /// The defect this pins: before 2026-09-06 a CMYK mark reached the panel as
    /// `None` — a default black swatch and, worse, **no Clear button**, so the
    /// one lossless operation available on it was the one withheld. Both halves
    /// are asserted, because fixing only the first would leave a converted
    /// colour presented as though it were the file's own value.
    ///
    /// Falsified twice: `narrowed: false` in the `Cmyk` arm turned the
    /// disclosure assertion red, and collapsing that arm to
    /// `Color::Cmyk(..) => Swatch::default()` — which is the behaviour that
    /// shipped until 2026-09-06 — turned the value assertions red.
    #[test]
    fn a_cmyk_colour_is_shown_as_a_conversion_and_is_flagged_as_one() {
        // Pure cyan: 1 - min(1, 1 + 0) = 0 red, 1 - 0 = 1 green and blue.
        let cyan = swatch_of(Some(&Color::Cmyk(1.0, 0.0, 0.0, 0.0)));
        assert_eq!(cyan.rgb, Some([0, 255, 255]));
        assert!(
            cyan.narrowed,
            "a converted colour must announce itself, or the swatch is a quiet lie"
        );
        // Registration black: every channel plus K, clamped, is black.
        assert_eq!(
            swatch_of(Some(&Color::Cmyk(1.0, 1.0, 1.0, 1.0))).rgb,
            Some([0, 0, 0])
        );
    }

    /// ★★★ **A mark the style verb cannot reach gets NO rows.**
    ///
    /// `spec_from_dict` answers `UnsupportedSubtype` for `/Text`, `/FreeText`
    /// and `/Stamp` — verified against the engine source — and `None` here is
    /// exactly what that refusal becomes on the way in. What this asserts is
    /// that the refusal reaches the panel as a *reachability* verdict rather
    /// than as "this mark has no colour", because the two produce different
    /// screens: nothing plus a sentence, versus three live controls that cannot
    /// commit.
    ///
    /// Falsified by setting `restylable: true` in the `None` arm of
    /// `from_spec`, which turned this red immediately.
    #[test]
    fn a_subtype_the_style_verb_refuses_offers_no_controls() {
        let refused = Current::from_spec(None, Some(0.5));
        assert!(!refused.restylable);
        assert_eq!(refused.colour.rgb, None);
        assert_eq!(refused.width, None);
        assert!(!refused.has_interior);
        assert_eq!(refused.endings, None);
        assert_eq!(
            refused.alpha, None,
            "an opacity under a heading whose controls are all withheld is decoration"
        );
    }

    /// …and a mark it CAN reach says so, on the same function.
    ///
    /// The companion assertion, and it is the one that stops the fix above from
    /// being "return false always", which would pass the test above and remove
    /// the whole section from the application.
    #[test]
    fn a_subtype_the_style_verb_reads_offers_its_controls() {
        let ok = Current::from_spec(Some(&square(None)), Some(0.5));
        assert!(ok.restylable);
        assert_eq!(ok.colour.rgb, Some([255, 0, 0]));
        assert_eq!(ok.width, Some(2.0));
        assert_eq!(ok.alpha, Some(0.5));
    }

    /// ★★ **The Fill row follows the shape, not the subtype string.**
    ///
    /// A square, a circle, a polygon and a cloud have an interior; a line, an
    /// ink stroke, a polyline and a text markup do not, and
    /// `apply_markup_style` reads `style.interior` on exactly the first four.
    /// The cloud is the case a subtype-string list would have got wrong — it is
    /// a `/Polygon` in the file and a `MarkupSpec::Cloud` here.
    ///
    /// Falsified by dropping the `Cloud` arm from `from_spec`'s interior match
    /// (the cloud assertion went red) and by adding `MarkupSpec::Line` to it
    /// (the line assertion went red).
    #[test]
    fn only_a_shape_with_an_interior_gets_a_fill_row() {
        assert!(Current::from_spec(Some(&square(None)), None).has_interior);
        assert!(
            Current::from_spec(
                Some(&MarkupSpec::Cloud {
                    vertices: vec![(0.0, 0.0), (10.0, 0.0), (5.0, 8.0)],
                    border: None,
                    interior: None,
                    width: 1.0,
                    intensity: 1.0,
                }),
                None
            )
            .has_interior,
            "a revision cloud is a /Polygon in the file and has an /IC"
        );
        assert!(
            !Current::from_spec(Some(&line((LineEnding::None, LineEnding::None))), None)
                .has_interior,
            "a line has no interior for /IC to mean anything in"
        );
    }

    /// The Fill swatch reads back the interior it was given, and offers Clear
    /// only when there is something to clear.
    ///
    /// `interior.rgb.is_some()` is precisely the condition `fill_row` puts the
    /// Clear button behind, so this is that button's guard asserted where a test
    /// can reach it.
    #[test]
    fn the_fill_swatch_reads_back_the_interior_and_knows_when_it_is_absent() {
        let filled = Current::from_spec(Some(&square(Some(Color::Rgb(0.0, 0.0, 1.0)))), None);
        assert_eq!(filled.interior.rgb, Some([0, 0, 255]));

        let unfilled = Current::from_spec(Some(&square(None)), None);
        assert!(unfilled.has_interior, "the row draws");
        assert_eq!(unfilled.interior.rgb, None, "…with no Clear beside it");
    }

    /// ★★ **The line-ending choosers appear for a `/Line` and for nothing
    /// else**, which is the set `apply_markup_style` acts on.
    ///
    /// Falsified by returning `Some((LineEnding::None, LineEnding::None))` for
    /// every arm, which turned the square assertion red.
    #[test]
    fn only_a_line_gets_the_ending_choosers() {
        assert_eq!(
            Current::from_spec(Some(&line((LineEnding::OpenArrow, LineEnding::None))), None)
                .endings,
            Some((LineEnding::OpenArrow, LineEnding::None)),
            "and it reads back what the file says, so the unchanged end is never stale"
        );
        assert_eq!(Current::from_spec(Some(&square(None)), None).endings, None);
    }

    /// ★★★ **The chooser's list covers every ending the engine can draw.**
    ///
    /// `LineEnding` has no `ALL` of its own, so [`ALL_ENDINGS`] is written by
    /// hand — and a hand-written list of an enum's variants is exactly the thing
    /// that goes stale. This `match` has **no wildcard**: an ending the engine
    /// gains fails to compile here, and the fix is to add it to both places at
    /// once.
    ///
    /// The count is asserted too, so a duplicated entry (three names, two
    /// distinct variants) is caught as well as a missing one.
    #[test]
    fn the_ending_list_covers_every_variant_the_engine_has() {
        for ending in ALL_ENDINGS {
            // Exhaustive by construction — no `_` arm.
            match ending {
                LineEnding::None | LineEnding::OpenArrow | LineEnding::ClosedArrow => {}
            }
        }
        let mut seen: Vec<LineEnding> = Vec::new();
        for ending in ALL_ENDINGS {
            assert!(!seen.contains(&ending), "an ending is offered twice");
            seen.push(ending);
        }
        assert_eq!(seen.len(), 3, "Table 176's three that pdfcer authors");
    }

    /// ★ The width range is the same one the markup pen offers.
    ///
    /// Two ranges for one quantity would let an operator author a 2 pt mark and
    /// then be unable to set 2 pt on it — or, worse, set a width here the pen
    /// could not have produced, so a document would carry marks the shell
    /// cannot make.
    #[test]
    fn the_width_range_matches_the_pen_that_authors() {
        assert!((MIN_WIDTH_PT - crate::canvas::markup::pen::MIN_WIDTH_PTS).abs() < f64::EPSILON);
        assert!((MAX_WIDTH_PT - crate::canvas::markup::pen::MAX_WIDTH_PTS).abs() < f64::EPSILON);
    }
}
