//! # `panels::properties::markup::textannot` — restyling the marks that carry
//! WORDS
//!
//! A sticky note's **icon and colour**, and a stamp's **colour**, on a mark
//! that is already on the page. `EditSession::set_text_annot_style`
//! (`pdfcer-core` `edit.rs:27124`), and this module is its only caller in the
//! shell.
//!
//! ## ★★★ What this closes — the oldest open row on this shell's list
//!
//! `request_a_sticky_notes_icon_and_colour_cannot_be_changed.md`, filed
//! 2026-09-05 against `pdfcer-core` v0.38.0, opening with the operator's own
//! instruction of the same day: ***"check that these are fully editable while
//! you are at it."*** The request's summary of the cost was three rows, and
//! two of them are this module's:
//!
//! > | the operator places a sticky and wants a different icon | **delete it
//! > and place another** |
//! > | a reviewer wants their comments in a different colour after the fact |
//! > **delete and replace**, losing `/M` and the object id |
//!
//! The engine answered on 2026-09-06 with `Pass 253.2`. This is the consuming
//! half.
//!
//! ## ★★★ Why a SECOND verb, and why the guard between them is a `match`
//!
//! `pdfcer-core` has two annotation-style verbs, and the split is not a
//! tidiness decision anybody took — `edit::TextAnnotStyle`'s own doc
//! (`edit.rs:15969`) says why:
//!
//! > `MarkupStyle` reaches its annotation through
//! > `annot_author::spec_from_dict`, whose arms are the geometric family and
//! > the four text markups. **There is no `/Text` arm** … So the two verbs are
//! > not a split anyone chose for tidiness — they read through different
//! > functions because the two families are modelled by different spec types.
//!
//! ⇒ Two readers, two spec types, two style structs, two verbs. The parent
//! module's [`super::section`] therefore routes on [`Reach`], an enum
//! whose arms the compiler makes exhaustive, and **not** on a `/Subtype`
//! string compared in an `if`. Sending a `/Stamp` to `set_markup_style` is the
//! defect this panel already shipped once — *"live controls, every press
//! refused"*, the module header's ★★★ of 2026-09-06 — and the fix for a
//! mis-route is not a better string comparison, it is making the wrong turn
//! fail to compile.
//!
//! ## ★★ Why a submodule and not more of `markup.rs`
//!
//! **R2.** The parent was at 1,348 lines before this and the rows below are
//! not a paragraph. The seam is the same one `markup/tests.rs` took the day
//! before, and it is a subject seam rather than a line count: **the parent
//! draws what one verb reaches, this file draws what the other one does.** A
//! file split down the middle of a routing decision would be the worse cut;
//! this one is split along it.
//!
//! ## ★★★ The three things this module does NOT offer, each for its own reason
//!
//! Every one of these is **absent**, not greyed. R9 reserves greying for a
//! capability that is *temporarily* unavailable and can explain itself on
//! hover, and none of these three can ever become available by anything the
//! operator does.
//!
//! 1. **No Clear beside the colour swatch.** `TextAnnotStyle::color` cannot
//!    clear, and the engine explains that rather than merely lacking it:
//!    `TextAnnotSpec`'s three variants each carry a **required** `Color` — a
//!    sticky's icon, a stamp's face and a free text's frame are each drawn in
//!    one — so *"no colour"* is not a state the authoring type can express,
//!    and a `StyleEdit::Clear` here *"would have to invent a fallback —
//!    silently picking yellow for a note whose colour an operator asked to
//!    remove"*. [`super::colour_row`]'s Clear is `MarkupStyle::stroke`'s and
//!    does not generalise.
//!
//! 2. **No opacity row.** `/CA` is `set_markup_style`'s, and that verb cannot
//!    reach these subtypes at all. So it is not that this verb declines the
//!    property — there is no route to it for a `/Text` or a `/Stamp` from any
//!    verb the engine publishes today. Recorded here because the parent's
//!    [`super::opacity_row`] is right above and its absence would otherwise
//!    read as an oversight.
//!
//! 3. **No width, fill, dash or endings.** Same reason, one level up: they are
//!    `MarkupStyleSupport`'s properties, asked of a verb that has no arm for
//!    these subtypes.
//!
//! ## ★★★ AND A FOURTH, WHICH IS THE FINDING OF THIS PASS: **a `/FreeText` is
//! ## deliberately NOT routed here, even though the verb accepts one**
//!
//! `set_text_annot_style` takes a `/FreeText` and will restyle its frame
//! colour. **This shell does not send it one**, and the reason is a defect the
//! engine's own documentation predicts three files apart without joining up:
//!
//! * `annot_author::text_spec_from_dict` returns `multiline: false` for
//!   **every** `/FreeText`, always, and says so at `annot_author.rs:896`:
//!   §12.5.6.6 gives the subtype no multiline key, `/Ff` is a form-field entry
//!   and a `/FreeText` is not a field, so *"a caller that needs the true value
//!   must **measure** it rather than believe this field"*.
//! * `set_text_annot_style` (`edit.rs:27124`) reads through that function,
//!   amends the spec, and re-bakes through `build_text_annotation` — **without
//!   measuring**. `set_markup_note` does measure, by baking the original text
//!   both ways and comparing bytes; this verb does not.
//!
//! ⇒ **Changing a text box's colour would re-bake its words as a single
//! unwrapped line.** `crate::canvas::textannot::spec` authors
//! `multiline: true` on every text box this shell has ever placed, so it is
//! not an edge case — it is *every* pdfcer callout, and the operator's second
//! sentence would leave the box with nothing on screen to say so. That module's
//! own comment named this exact hazard in advance: *"re-baking a wrapped
//! callout as unwrapped would push the operator's second sentence off the page
//! with nothing on screen to say so."*
//!
//! ★ So the text box gets [`crate::text::panels::textannotstyle::markup_text_box_not_restylable`]
//! — a sentence that is **true for a new reason** rather than the old one. The
//! capability is not missing; it is declined, by this shell, on this shell's
//! reading of what it would cost. That is a different claim and the copy says
//! so. **Filed for the engine**, whose fix is one call site: measure
//! `multiline` the way `set_markup_note` already does, or take it as an
//! argument.
//!
//! ## ✅ THE READER NO LONGER NORMALISES A FOREIGN ICON NAME AWAY — 2026-09-07
//!
//! This section used to read:
//!
//! > *"`text_spec_from_dict`'s `/Text` arm reads `/Name`, runs it through
//! > `StickyIcon::from_name`, and `.unwrap_or(StickyIcon::Note)`. §12.5.6.4's
//! > seven names are 'a standard set, not a closed one' — a producer's own icon
//! > name is conforming — so a note carrying `/Sparkle` reads back as `Note`,
//! > and a restyle of its **colour alone** rewrites its `/Name` to `/Note`.
//! > Silent, and not recoverable by looking. [`Reading::foreign_icon`] catches
//! > that by reading the raw `/Name` off the dictionary itself and comparing."*
//!
//! Every word of that was true against `pdfcer-core` v0.44.0. It was filed
//! (`request_set_text_annot_style_rewrites_a_foreign_icon_name.md`) rather than
//! worked around quietly, and **`Pass 253.5` answered it the same day**:
//! `StickyIcon::Other(Vec<u8>)` carries the bytes and
//! `StickyIcon::from_name_lossless` reads them, so the round trip is exact and
//! a colour change touches nothing else.
//!
//! ⇒ **What changed here, in three lines.** The raw-dictionary read
//! (`read_icon_name`) is **deleted** — its site carries the reason it existed.
//! [`Reading::icon`] now carries `Other` like any other value, and the chooser
//! shows the file's own name **in quotes as a selectable entry**, so an
//! operator who opens the list can get back to what his file said.
//! [`Reading::foreign_icon`] survives with a **narrower job**: it no longer
//! warns about destruction, it explains that pdfcer paints its own glyph
//! whatever the name says. The engine's note is the authority — *"`sticky_note`
//! paints the same glyph for all seven variants; the icon chooses the `/Name`
//! written, not the picture drawn."*
//!
//! ⚠ **The shelf life of the paragraph above was ONE DAY.** Do not quote this
//! module as a source about the engine; it is a record of what was true on a
//! date. Re-read the verb at the current pin before repeating any claim about
//! what it will or will not preserve.

use egui::Ui;
use pdfcer_core::annot::{StampLabelParameters, StampSizeSource};
use pdfcer_core::annot_author::{Color, StickyIcon, TextAnnotSpec};
use pdfcer_core::edit::TextAnnotStyle;

use crate::app::actions::Action;
use crate::app::actions::annot::AnnotAction;
use crate::text::panels::properties as t;
use crate::text::panels::textannotstyle as ts;
use crate::text::textannot as tt;

/// The region this subsection publishes, so a driven check can find the icon
/// chooser on a placed note rather than only on the placing dialog.
pub(super) const REGION: &str = "properties.markup.textannot"; // ui-text-exempt: trace region name, never displayed

/// The **label-size spinner**'s own region, published only when the row is
/// actually on screen.
///
/// # ★★★ Why this constant exists at all, and what shipped without it
///
/// The operator asked twice — 2026-09-09 and again after the fix for the
/// authoring half landed — for the same thing in the same words:
///
/// > *"still can't adjust the size of a stamp on the canvas, **or by entering a
/// > different size in the properties box**."*
///
/// The second clause is this row. It was built with unit tests in front of
/// every hop and **no way for a driven check to find it**: a region is how
/// `tools/ui-verify` locates a control, and a control with no region can be
/// drawn under another widget, clipped off the bottom of the panel's scroller,
/// or not drawn at all, with every test in the crate still green. This project
/// has shipped exactly that — a panel that was unreachable in a real build with
/// every gate green — which is why R1 says a phase is not done until the
/// behaviour is asserted by driving the binary.
///
/// ⇒ Published through [`crate::diag::ui_rect_visible`] and not `ui_rect`, for
/// the reason the rail's header gives in full: this row lives inside a
/// scrolling panel, and a rectangle published for a row that is scrolled out of
/// view is a rectangle a driven check will click on — hitting whatever is
/// really there.
pub(super) const SIZE_REGION: &str = "properties.markup.textannot.size"; // ui-text-exempt: trace region name, never displayed

/// The **fit chooser**'s region — see [`SIZE_REGION`] for the argument.
///
/// Separate from the spinner's because the two are separate failure modes: a
/// build can draw the number and clip the chooser below it, and the operator
/// then has a size they can change and no way to say what should give when it
/// stops fitting.
pub(super) const FIT_REGION: &str = "properties.markup.textannot.fit"; // ui-text-exempt: trace region name, never displayed

/// The trace slot the label row reports **its own reading** through.
///
/// ```text
/// pdfcer-diag stamp-label-row size=24 source=declared-in-da fit=grow
/// ```
///
/// # ★★ Why a line, when the number is on the screen
///
/// Because a driven check cannot read a number off a screenshot, and the
/// alternative this project has been bitten by is a check that *describes* the
/// absence it never measured — an unevidenced excuse, which reads as an
/// answered question and stops anybody looking again. `stamp-size-chooser` was
/// added to the placing dialog for the identical reason on 2026-09-10; this is
/// its twin on the restyle side.
///
/// ★ It carries `source=` as well as `size=`, because the two answer different
/// questions. `size=` is what the operator sees. `source=` is where it came
/// from, and it is the only way a check can tell *"the file declared 24"* from
/// *"pdfcer read 24 off the picture because the file declares nothing"* — a
/// distinction no screenshot contains and the whole reason
/// `StampSizeSource` has three variants rather than being an `Option`.
const ROW_SLOT: &str = "stamp-label-row"; // ui-text-exempt: diagnostic trace slot, never displayed

/// ★★★ **Which of `pdfcer-core`'s TWO annotation-style verbs reaches the
/// selected mark** — the guard between them, as a `match` the compiler
/// checks.
///
/// # Why an enum and not two booleans
///
/// Because two booleans have four states and only three of them mean anything,
/// and the fourth — *both verbs reach it* — is the one that would send a mark
/// down whichever branch happened to be tested first. The two engine readers
/// are disjoint by construction (`spec_from_dict` has no `/Text`, `/Stamp` or
/// `/FreeText` arm; `text_spec_from_dict` has **only** those three), so the
/// disjointness is a fact about the engine — and encoding it in a type is the
/// difference between a fact that holds and a fact that is relied upon.
///
/// # ⚠ [`Self::TextBoxWithheld`] is this SHELL's decision, not the engine's
///
/// Every other arm reports what an engine function answered.
/// This one reports a refusal of our own, and it is labelled so nobody reads it
/// as a capability gap and files it: `set_text_annot_style` restyles a
/// `/FreeText` perfectly well and would **unwrap its words** doing so.
/// This module's header carries the measurement, the two engine
/// doc comments it joins up, and what the engine's fix would be.
///
/// ⚠ **`Clone`, not `Copy`, since 2026-09-07** — [`Self::TextAnnot`] carries a
/// [`Reading`], which carries a [`StickyIcon`], which gained an owning
/// `Other(Vec<u8>)` variant. The `match` in `super::section` binds it by
/// reference now; nothing else changed.
#[derive(Debug, Clone)]
pub(super) enum Reach {
    /// `EditSession::set_markup_style` — the geometric family and the four text
    /// markups. `annot_author::spec_from_dict` read a spec.
    Markup,
    /// `EditSession::set_text_annot_style` — a `/Text` or a `/Stamp`.
    /// `annot_author::text_spec_from_dict` read a spec and
    /// [`Reading::of`] accepted the face.
    TextAnnot(Reading),
    /// A `/FreeText`. Reachable by the second verb and **declined here**.
    TextBoxWithheld,
    /// Neither reader could produce a spec: a `/Subtype` outside both families,
    /// or geometry pdfcer does not model.
    Neither,
}

/// **Which text-bearing face is selected** — the closed set
/// `set_text_annot_style` is offered for by this shell.
///
/// ★★ An enum rather than the `/Subtype` string, so that *which properties does
/// this face take?* is answered by a `match` the compiler checks. A `bool` pair
/// (`takes_icon`, `takes_colour`) would let a third face arrive and be given
/// both by whichever default the author typed first.
///
/// ⚠ **`/FreeText` is deliberately not a variant.** The engine's verb accepts
/// one; this shell declines to send it, and the module header carries the
/// measurement. Modelling it here as a face that takes only a colour would put
/// a live control in front of the operator whose press unwraps their callout —
/// which is the *visible control, silently destructive* case, one worse than
/// the *visible control, silently inert* case this panel already fixed once.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Face {
    /// `/Text` — a sticky note. Icon and colour.
    Sticky,
    /// `/Stamp` — a rubber stamp. Colour only: its face comes from its own
    /// `/Name` vocabulary (Table 181), which is not Table 172's, and the engine
    /// refuses a `StickyIcon` on one **by name** rather than swallowing it
    /// (`EditError::StylePropertyNotApplicable`).
    Stamp,
}

impl Face {
    /// Whether this face takes an icon.
    ///
    /// ★ The same question `set_text_annot_style` asks at `edit.rs:27152`
    /// (`style.icon.is_some() && target.subtype != b"Text"`), asked here so the
    /// chooser is **absent** rather than drawn-and-refused. Belt and braces:
    /// the engine's refusal is what catches a shell that drifted anyway, which
    /// is exactly the arrangement [`super::Current::restylable`] has with its
    /// own verb.
    pub(super) const fn takes_icon(self) -> bool {
        match self {
            Self::Sticky => true,
            Self::Stamp => false,
        }
    }
}

/// What the selected text-bearing mark's dictionary currently says, in the
/// terms this subsection can change.
///
/// ⚠ **`Clone`, not `Copy`, since 2026-09-07** — `StickyIcon` gained an
/// `Other(Vec<u8>)` variant that owns its bytes, so the whole struct lost its
/// implicit copies. Every borrow that used to be free is now explicit; the row
/// functions take `&Reading`.
#[derive(Debug, Clone)]
pub(super) struct Reading {
    /// Which face, and therefore which properties mean anything.
    pub(super) face: Face,
    /// `/C`, as a swatch can show it, and whether showing it cost a conversion.
    ///
    /// ★ Through [`super::swatch_of`], the parent's function, rather than a
    /// second conversion written here. The CMYK narrowing disclosure is a
    /// property of *showing a `/C` in an sRGB button* and has nothing to do
    /// with which verb writes it back — two copies of that arithmetic would be
    /// two chances to disagree about a colour on the operator's sheet.
    pub(super) colour: super::Swatch,
    /// `/Name`, for a `/Text` — **including a name pdfcer does not model**,
    /// which arrives as [`StickyIcon::Other`].
    ///
    /// ★★★ **`None` now means only "this face has no icon"** — a `/Stamp`,
    /// whose `/Name` is a stamp face and a different vocabulary altogether.
    /// Until 2026-09-07 it *also* meant *"the `/Name` is one pdfcer does not
    /// model"*, because the engine's reader normalised such a name to `Note`
    /// and this shell had to detect the loss by reading the raw dictionary
    /// beside it. `Pass 253.5` made the reader lossless, so the name is now a
    /// value like any other and the second meaning is gone.
    pub(super) icon: Option<StickyIcon>,
    /// ★★ **`true` when the file's `/Name` is a name pdfcer does not model.**
    ///
    /// §12.5.6.4's seven are *"a standard set, not a closed one"*, so a
    /// producer's own icon name is conforming and this is a legitimate state,
    /// not a defect.
    ///
    /// # What it is still FOR, now that nothing is lost
    ///
    /// The name round-trips, so this no longer warns about destruction. What
    /// it still says is that **pdfcer draws its own picture for it** — the
    /// engine's own note: *"`sticky_note` paints the same glyph for all seven
    /// variants; the icon chooses the `/Name` written, not the picture drawn"*
    /// — so an operator comparing this window with Acrobat's is entitled to
    /// know why the two differ.
    ///
    /// ⇒ **The old disclosure said something else and had become false.** It
    /// read *"changing its icon OR its colour here will replace that icon with
    /// one of the ones listed"*, which was exactly right against `v0.44.0`'s
    /// normalising reader and is exactly wrong against `v0.44.1`'s. See
    /// `text::panels::textannotstyle::markup_icon_foreign_note`.
    ///
    /// ★ Derived from the **spec** now, not from a second read of the
    /// dictionary. `read_icon_name` is deleted, with the reason it existed kept
    /// at its old site.
    pub(super) foreign_icon: bool,
    /// **What the stamp's own appearance says its label is** — the words, the
    /// size in points, and where that size came from (`pdfcer-core`
    /// `Pass 292.0`). `None` for a sticky note, and `None` for a stamp whose
    /// picture shows no text pdfcer can read a size off.
    ///
    /// # ★★★ Why this is NOT filled by [`Reading::of`]
    ///
    /// Because `of` is **pure** and takes the spec and nothing else, which is
    /// what lets six tests build a `Reading` in one expression. The label
    /// parameters are not in the spec at all — they are recovered by parsing
    /// the annotation's `/AP` `/N` stream, so reading them needs the session,
    /// the object graph and the R45 staging buffer behind it.
    ///
    /// ⇒ The read stays in [`super::Reach::read`], where the session already
    /// is, and arrives here through [`Reading::with_stamp_label`]. That keeps
    /// the impure half in the one function that was always impure, rather than
    /// making every test of the pure half construct a document.
    ///
    /// ⚠ A `Reading` built by `of` alone therefore has `label: None`, which is
    /// indistinguishable from *"this stamp has no describable label"*. That is
    /// deliberate and it is safe in the only direction that matters: the size
    /// row is **absent** rather than wrong. A test that means to assert the row
    /// appears must call `with_stamp_label`.
    pub(super) label: Option<StampLabelParameters>,
}

impl Reading {
    /// **Read one, or answer `None` for a mark this subsection does not serve.**
    ///
    /// `None` covers three different situations and they are worth telling
    /// apart in the head even though the answer is the same:
    ///
    /// 1. a `/Subtype` outside `/Text`, `/Stamp` and `/FreeText` —
    ///    `text_spec_from_dict` refuses it and so does the verb;
    /// 2. a `/FreeText` — the verb accepts it and **this shell declines**, per
    ///    the module header's multiline finding;
    /// 3. a `/Text` or `/Stamp` whose `/Rect` is missing or unreadable —
    ///    `SpecReadError::BadGeometry`, the same refusal the verb would make.
    ///
    /// [`Reach`] separates (2) from the other two, because
    /// only (2) has a sentence of its own to show.
    ///
    /// ★★ It takes the spec the caller already read, and **nothing else** since
    /// 2026-09-07. It used to take the raw `/Name` bytes as a second argument,
    /// because the engine's reader normalised an unmodelled name to `Note` and
    /// the loss was invisible in the spec. `Pass 253.5` made that reader
    /// lossless (`StickyIcon::from_name_lossless`), so the fact is in the value
    /// and the second argument — and the dictionary read behind it — are gone.
    pub(super) fn of(spec: &TextAnnotSpec) -> Option<Self> {
        match spec {
            TextAnnotSpec::Sticky { color, icon, .. } => Some(Self {
                face: Face::Sticky,
                colour: super::swatch_of(Some(color)),
                // ★ An ABSENT `/Name` is not foreign, and the engine's reader
                // already draws that line for us: Table 172's default is
                // `Note`, so a note carrying no `/Name` arrives as `Note` and
                // showing `Note` for it is reporting the standard rather than
                // inventing anything.
                foreign_icon: matches!(icon, StickyIcon::Other(_)),
                icon: Some(icon.clone()),
                // A sticky note draws an ICON, not text. `set_text_annot_style`
                // refuses a label size on one BY NAME
                // (`StylePropertyNotApplicable`, property "a label font
                // size"), so this is not "we did not read it" — there is
                // nothing to read.
                label: None,
            }),
            TextAnnotSpec::Stamp { color, .. } => Some(Self {
                face: Face::Stamp,
                colour: super::swatch_of(Some(color)),
                icon: None,
                foreign_icon: false,
                // Filled by [`Self::with_stamp_label`] from the session; see the
                // field's own note on why `of` cannot.
                label: None,
            }),
            // (2) above. The verb would take it; this shell will not send it.
            TextAnnotSpec::FreeText { .. } => None,
            // ★ `TextAnnotSpec` is `#[non_exhaustive]`. A fourth text-bearing
            // face this build does not know the shape of gets no rows and the
            // withheld sentence — the same answer a `/FreeText` gets, and for a
            // compatible reason: this shell cannot say what a control over it
            // would do. R9 in the conservative direction, which is also
            // `MarkupStyleSupport::for_subtype`'s own stated posture.
            _ => None,
        }
    }

    /// **Carry the stamp's label parameters in**, read from the session by
    /// [`super::Reach::read`] (`pdfcer-core` `Pass 292.0`).
    ///
    /// # ★★ Why a builder and not a second argument to [`Self::of`]
    ///
    /// Because `of` used to take a second argument — the raw `/Name` bytes —
    /// and this module's own history says what that cost. It was deleted on
    /// 2026-09-07 when `Pass 253.5` put the fact in the value, and the note
    /// left at its site is the general form: a second read beside the spec is
    /// a second chance to disagree with the engine about an operator's file,
    /// and it makes the pure function impure for every caller including the
    /// six tests.
    ///
    /// ⇒ A builder keeps `of`'s signature at *one spec in, one reading out*
    /// and puts the session-shaped read where the session already is. The
    /// tests that assert the reachability verdict never see it; the one test
    /// that means to assert the size row appears calls this.
    ///
    /// ⚠ It takes an `Option` rather than a value, and passes it straight
    /// through, because `EditSession::stamp_label_parameters` answers
    /// `Ok(None)` for a stamp whose appearance shows no text — Acrobat's own
    /// custom stamps are artwork — and the engine is explicit that this is
    /// *"the honest answer … not a failure"*. Collapsing it to a default here
    /// would invent a size for a picture of a signature.
    #[must_use]
    pub(super) fn with_stamp_label(mut self, label: Option<StampLabelParameters>) -> Self {
        self.label = label;
        self
    }
}

// ★★★ `read_icon_name` WAS HERE, AND IT IS DELETED — 2026-09-07.
//
// It read the `/Name` bytes straight out of the annotation dictionary, beside
// the spec, because `text_spec_from_dict` normalised a name outside the seven
// to `Note` — so the spec could not tell *"the file says Note"* from *"the file
// says Sparkle and the reader flattened it"*, and this panel would have shown
// the wrong entry as selected and written it back on a colour change.
//
// It was filed rather than kept quiet
// (`request_set_text_annot_style_rewrites_a_foreign_icon_name.md`), and
// `Pass 253.5` answered it with `StickyIcon::Other(Vec<u8>)` and
// `from_name_lossless`. The fact is now in the value the reader returns, so a
// second reader of the same key would be a second chance to disagree with the
// engine about an operator's file.
//
// ⇒ **Delete the workaround when the cause is removed.** Its own header said
// this was a second reader of a structure `pdfcer-core` owns; leaving it in
// place because it still compiles is how a shell accumulates a private,
// diverging model of somebody else's format.

/// **Draw the rows `set_text_annot_style` can commit.**
///
/// Called from [`super::section`]'s `Reach::TextAnnot` arm and from nowhere
/// else, on a mark the parent has already established is neither locked nor a
/// ce dimension.
///
/// # ★ The order: what it says, then what it looks like
///
/// The colour first, because it is the property both faces have and the one an
/// operator reaches for; the icon second, because it exists on one face only
/// and a row that appears and disappears between selections should not be the
/// one that sets the panel's vertical rhythm.
pub(super) fn rows(
    ui: &mut Ui,
    current: &Reading,
    target: &crate::canvas::selection::annot::AnnotTarget,
    actions: &mut Vec<Action>,
) {
    crate::diag::ui_rect(REGION, ui.max_rect());
    // ★★★ **What this subsection is showing, on the trace channel** — added
    // 2026-09-07 with the lossless icon name.
    //
    // The published region says *the rows drew*; it cannot say **what they
    // say**. That distinction is the whole reason this line exists: a build
    // that flattened a producer's `/Sparkle` to `Note` and one that carried it
    // draw the same rectangle, in the same place, with the same number of
    // controls — and differ only in the words inside the combo, which no rect
    // carries.
    //
    // ★ `icon=` is the NAME as the file spells it, lossily decoded, not a
    // variant label. A check reading `icon=Note` cannot tell the flattened case
    // from a note that genuinely says `Note`; reading `icon=Sparkle` on a
    // fixture planted with `/Sparkle` can. `foreign=` is the panel's own
    // verdict beside it, so a build that carried the name and forgot to
    // disclose is a different line from one that did neither.
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed.
        format!(
            "textannot-rows face={:?} icon={} foreign={}",
            current.face,
            current.icon.as_ref().map_or_else(
                || "none".to_owned(),
                |i| String::from_utf8_lossy(i.name()).into_owned()
            ),
            u8::from(current.foreign_icon),
        )
    });
    colour_row(ui, current, target, actions);
    // ★ The narrowing disclosure sits directly under the swatch it qualifies,
    // which is `REVIEW_TRIAGE.md`'s rule and the parent's placement: a caveat
    // below the thing it qualifies arrives after the operator has drawn their
    // conclusion, so it must arrive before the next control instead.
    if current.colour.narrowed {
        ui.label(
            egui::RichText::new(t::markup_colour_narrowed())
                .small()
                .weak(),
        );
    }
    // ★★ The size sits between the colour and the icon, and the two never
    // appear together: a stamp takes a size and no icon, a sticky note takes an
    // icon and no size. It is placed here rather than last so that the ORDER a
    // reader sees is stable across the two faces — colour, then whatever the
    // face's own property is — rather than the icon jumping above the size on
    // one selection and below it on the next.
    size_row(ui, current, target, actions);
    icon_row(ui, current, target, actions);
    ui.label(
        egui::RichText::new(ts::markup_text_annot_note())
            .small()
            .weak(),
    );
}

/// The smallest and largest label size the spinner offers, in points.
///
/// # ⚠ These are THIS SHELL's bounds. The engine has none.
///
/// `set_text_annot_style` does not clamp `font_size`, and `StampStyle` does
/// not either — a caller may ask for 0.1 pt or 900 pt and get it. So these two
/// numbers are a usability judgement made here, not a limit reported from
/// anywhere, and this comment exists so that nobody later quotes them as an
/// engine fact. Below about four points Helvetica Bold is a smudge on paper;
/// above about a gross the box `GrowToText` produces is wider than a letter
/// page, so the stamp leaves the sheet.
///
/// ★★★ **And a range is a hazard, which [`size_row`] handles rather than
/// ignores.** A control narrower than the values its subject accepts *silently
/// rewrites a value the operator never touched*: open a stamp declaring 200 pt
/// under a spinner capped at 144, and the spinner shows 144 — then one
/// keystroke anywhere commits it. That has happened on this project before, in
/// the settings window, and the fix taken there is the one taken here: the
/// range is **widened to admit whatever the file said**, and the action is
/// pushed only when the number actually differs from what was read.
const MIN_LABEL_PT: f64 = 4.0;
/// See [`MIN_LABEL_PT`].
const MAX_LABEL_PT: f64 = 144.0;

/// **The stamp's label size, and what to do when it stops fitting** —
/// `pdfcer-core` `Pass 292.0`.
///
/// # ★★★ What this closes, in the operator's own words — asked TWICE
///
/// 2026-09-09, at the machine: ***"I STILL can't adjust the size of a stamp on
/// the canvas, or by entering a different size in the properties box."***
///
/// The first half of that sentence was answered the same day — `annots::resize`
/// now sets `scale_stroke_width` and `allow_appearance_distortion` for a
/// `/Stamp` target, so the canvas grips and the Properties width/height fields
/// scale the picture. **The second half was not**, and the reason was a real
/// gap rather than an oversight: until `Pass 292.0` a stamp already on the page
/// had a label size that could be neither read nor written. There was no verb
/// to call. This row is the consuming half of the two the engine shipped.
///
/// # ★★ Why it is a size ROW and not another entry in the placing dialog's list
///
/// Because the two controls answer different questions. `canvas::textannot::
/// StampSize` offers a *list* — `Fit the box I drew`, then a ladder of stated
/// sizes — because at placing time the operator has no stamp to look at and a
/// ladder is how every tool they own presents a font size. Here the stamp
/// exists, its size is a **number that came out of the file**, and the act is
/// *change this number*. A combo box would have to invent an entry for a stamp
/// whose file says 17 pt.
///
/// ⚠ **`Fit the box I drew` has no counterpart here, deliberately.** That
/// choice means *derive the size from the box*, i.e. `font_size: None`, and
/// `None` on this verb means *leave the size alone* — the field's contract, and
/// the same contract that keeps a colour change from touching the icon. The two
/// meanings collide, and the engine's field cannot express the first. Offering
/// it would be a control whose press does nothing, which is the defect this
/// panel already shipped once (*"live controls, every press refused"*).
///
/// # ★ Absent, not greyed, when the stamp has no label pdfcer can describe
///
/// `stamp_label_parameters` answers `None` for a stamp whose appearance shows
/// no text — **Acrobat's own custom stamps are artwork**, not a laid-out
/// label — and the engine is explicit that this is *"the honest answer … not a
/// failure"*. R9: nothing is drawn. A greyed spinner would imply that a size
/// could appear if something were different, and for a picture of a signature
/// nothing can be different.
fn size_row(
    ui: &mut Ui,
    current: &Reading,
    target: &crate::canvas::selection::annot::AnnotTarget,
    actions: &mut Vec<Action>,
) {
    // ★ Two guards, and they are not the same guard twice. The first is about
    // the FACE — a sticky note draws an icon and has no label to size, which is
    // the refusal `set_text_annot_style` makes by name
    // (`StylePropertyNotApplicable`, property "a label font size"). The second
    // is about this PARTICULAR stamp — the face is right and its appearance
    // still shows no text pdfcer can read a size off.
    if current.face != Face::Stamp {
        return;
    }
    let Some(label) = current.label.as_ref() else {
        return;
    };

    let read_size = label.size;
    let mut size = read_size;
    let ctx = ui.ctx().clone();
    let mut fit = crate::canvas::stampfit::read(&ctx);

    // ★★ The row's own reading, before anything is pressed. `trace_changed`
    // rather than `trace`, so a panel drawn at sixty frames a second writes one
    // line per actual change; see [`ROW_SLOT`] for why the line exists at all
    // and why it carries `source=`.
    crate::diag::trace_changed(ROW_SLOT, || {
        format!(
            // ui-text-exempt: diagnostic trace, never displayed. The exemption sits
            // HERE and not above `format!` because check-ui-strings reads the comment
            // block immediately above the LITERAL; one intervening line of code and
            // the reason is invisible to it.
            "{ROW_SLOT} size={read_size} source={} fit={}",
            source_token(label.size_source),
            crate::canvas::stampfit::trace_token(fit)
        )
    });

    ui.horizontal(|ui| {
        ui.label(ts::stamp_text_size_label());
        let response = ui.add(
            egui::DragValue::new(&mut size)
                // ★★★ The range ADMITS whatever the file said. See
                // [`MIN_LABEL_PT`] — a spinner that clamps a value it did not
                // author is a spinner that edits documents nobody asked it to.
                .range(MIN_LABEL_PT.min(read_size)..=MAX_LABEL_PT.max(read_size))
                .speed(0.5)
                .suffix(ts::stamp_text_size_suffix()),
        );
        // ★ `drag_stopped` and `lost_focus`, never `changed` — the parent's
        // `width_row` carries the full argument. A `DragValue` reports a change
        // on every pixel of a drag and each one here is an appearance re-bake
        // plus an undo entry, so one drag across the control would leave forty
        // entries on the stack.
        //
        // ★★ **And `size != read_size` beside it**, which `width_row` does not
        // need and this one does. A `lost_focus` fires when the operator clicks
        // away having changed nothing, and on a stamp whose declared size lies
        // outside this shell's range the displayed number is not the file's
        // number — so an unconditional commit here would rewrite a `/DA` the
        // operator never touched, on a mark they only looked at.
        // ★★★ `ui_rect_visible`, and the clip rect is the panel's — not the
        // window's. A row scrolled below the properties panel's viewport is
        // still laid out and still has a rectangle; publishing it would hand a
        // driven check a coordinate that lands on whatever is drawn over it,
        // and the resulting report would name this feature for a defect in the
        // panel above it.
        crate::diag::ui_rect_visible(SIZE_REGION, response.rect, ui.clip_rect());
        if (response.drag_stopped() || response.lost_focus()) && size != read_size {
            push_size(target, size, fit, actions);
        }
    });

    // ★★★ The fit chooser sits UNDER the number it qualifies, because it is
    // read at the moment the operator has just typed a larger one and is
    // wondering what will happen. `REVIEW_TRIAGE.md`'s placement rule cuts the
    // other way for a *caveat* — a warning below the thing it warns about
    // arrives after the conclusion has been drawn — but this is not a caveat,
    // it is the second half of one instruction, and an operator reads the two
    // in the order they are committed.
    ui.horizontal(|ui| {
        ui.label(ts::stamp_fit_label());
        let combo = egui::ComboBox::from_id_salt("properties-stamp-fit") // ui-text-exempt: widget id salt, never displayed.
            .selected_text(ts::stamp_fit_option(fit))
            .show_ui(ui, |ui| {
                // ★ `stampfit::FITS`, never a hand-written list here. A
                // completeness test keys on that constant, and a second list
                // written out at a call site is invisible to it — the exact
                // shape of defect this project has a standing rule about.
                for policy in crate::canvas::stampfit::FITS.iter().copied() {
                    ui.selectable_value(&mut fit, policy, ts::stamp_fit_option(policy));
                }
            });
        crate::diag::ui_rect_visible(FIT_REGION, combo.response.rect, ui.clip_rect());
    });
    if fit != crate::canvas::stampfit::read(&ctx) {
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed.
            format!(
                "stamp-fit-chosen {}",
                crate::canvas::stampfit::trace_token(fit)
            )
        });
        crate::canvas::stampfit::store(&ctx, fit);
    }

    // ⚠ **One source of the three owes a sentence, and it is not the one a
    // shell reaches for first.** `RecoveredFromAppearance` — no `/DA` at all,
    // the size read off the baked `Tf` — covers *every stamp authored before
    // `Pass 287.0` and everything another producer wrote*, and the engine is
    // explicit that it is "not an anomaly and owes no warning … the number is
    // exactly what is on the page". A caution there would fire on the majority
    // of stamps in the world and teach the operator to ignore the one that
    // matters. `DaUnreadable` is the one that matters: the file states a size,
    // pdfcer cannot parse it, and setting one here overwrites it.
    if label.size_source == StampSizeSource::DaUnreadable {
        ui.label(
            egui::RichText::new(ts::stamp_size_da_unreadable())
                .small()
                .weak(),
        );
    }
}

/// **Where the displayed label size came from**, as one word for the trace.
///
/// # ★★ Why the shell writes this token and the engine does not
///
/// `StampLabelFit` publishes `token()` because a *driven check reads it* and
/// the engine tests that contract. `StampSizeSource` publishes no such thing —
/// it is a panel-facing distinction, and the engine's own doc says as much:
/// *"the three cases are kept apart because a panel owes different things to
/// each"*. So the vocabulary is this shell's, and it is written here, once,
/// rather than at the format string, so a check and a reader are looking at the
/// same list.
///
/// ⚠ **`StampSizeSource` is `#[non_exhaustive]`**, so the `_` arm is reachable
/// by nothing but a pin bump — and it is a **tripwire**, not a fallback. A
/// driven run showing `source=unknown` means the engine grew a fourth answer to
/// *"where did this number come from?"*, and this row's disclosure rule (only
/// `DaUnreadable` owes a sentence) was written against three. Seeing it is the
/// signal to go and read the new variant before deciding whether it owes one.
pub(super) fn source_token(source: StampSizeSource) -> &'static str {
    // ui-text-exempt: diagnostic trace tokens, never displayed.
    match source {
        StampSizeSource::DeclaredInDa => "declared-in-da",
        StampSizeSource::RecoveredFromAppearance => "recovered-from-appearance",
        StampSizeSource::DaUnreadable => "da-unreadable",
        _ => "unknown",
    }
}

/// Raise the restyle that carries a new label size — and **nothing else**.
///
/// ★ Split out from [`size_row`] so the struct literal that names every field
/// of `TextAnnotStyle` sits in one place per act rather than inside a closure
/// three levels deep. The `None`s are the contract, not a formality: *a field
/// left `None` is left alone*, so a size change does not touch the colour and
/// cannot touch the icon.
fn push_size(
    target: &crate::canvas::selection::annot::AnnotTarget,
    size: f64,
    fit: pdfcer_core::annot_author::StampFit,
    actions: &mut Vec<Action>,
) {
    actions.push(Action::Annot(AnnotAction::SetTextAnnotStyle {
        id: target.id,
        style: TextAnnotStyle {
            font_size: Some(size),
            // ★★ Named even though it equals the engine's default, because the
            // operator has an opinion about it and a `None` here would hide
            // that the chooser above had been read at all. `stamp_fit` is
            // "ignored unless `font_size` is set" — which is exactly the call
            // this is.
            stamp_fit: Some(fit),
            color: None,
            icon: None,
        },
    }));
}

/// The annotation's colour, `/C`.
///
/// # ★★ A swatch and NO Clear, unlike [`super::colour_row`]
///
/// The one structural difference between this row and the parent's, and it is
/// the engine's decision rather than a control left out: `TextAnnotStyle::color`
/// has no `Clear` arm to raise. Its doc gives the reason in full — every
/// `TextAnnotSpec` variant carries a **required** `Color`, so the authoring type
/// cannot express *no colour*, and a Clear would have to invent a fallback.
///
/// ⇒ **Absent, not greyed.** R9: a greyed Clear here would imply that clearing
/// could become possible if something were different, and nothing can be
/// different — the limit is in the shape of the engine's spec type. The
/// sentence under the rows says so once rather than a tooltip saying it per
/// press.
fn colour_row(
    ui: &mut Ui,
    current: &Reading,
    target: &crate::canvas::selection::annot::AnnotTarget,
    actions: &mut Vec<Action>,
) {
    // ★ The fallback is the mark's own default rather than black, and it
    // matters here in a way it does not in the parent: `text_spec_from_dict`
    // supplies a default `/C` when the key is absent (yellow for a note, black
    // for a stamp), so `current.colour.rgb` is `None` only when the value is
    // present and names no device space §8.6.3 defines. Black is then as good
    // an answer as any and the operator's first pick replaces it.
    let mut rgb = current.colour.rgb.unwrap_or([0, 0, 0]);
    ui.horizontal(|ui| {
        ui.label(t::markup_colour_label());
        if ui.color_edit_button_srgb(&mut rgb).changed() {
            actions.push(Action::Annot(AnnotAction::SetTextAnnotStyle {
                id: target.id,
                style: TextAnnotStyle {
                    color: Some(Color::Rgb(
                        f64::from(rgb[0]) / 255.0,
                        f64::from(rgb[1]) / 255.0,
                        f64::from(rgb[2]) / 255.0,
                    )),
                    // ★★ Explicit `None` on EVERY other field, and it is
                    // the contract rather than a formality: "a field left
                    // `None` is left alone", so a call that names only the
                    // colour does not touch the icon, the label size or the
                    // fit policy.
                    //
                    // ★★★ **Spelt out rather than reached through
                    // `..Default::default()`, and `Pass 292.0` is why that is
                    // load-bearing rather than tidy.** This literal named two
                    // fields for months; the engine then grew `font_size` and
                    // `stamp_fit`, and the build broke here — which is the
                    // outcome we wanted. `..Default::default()` would have
                    // compiled silently and taken `font_size: None`, i.e. it
                    // would have DECLINED a new capability on the operator's
                    // behalf without a single word appearing anywhere. A
                    // compile error is an invitation to read the engine's
                    // reply; a default is a way of not receiving it.
                    icon: None,
                    font_size: None,
                    stamp_fit: None,
                },
            }));
        }
    });
}

/// **The sticky note's icon, `/Name`** (§12.5.6.4, Table 172).
///
/// # ★★★ The row the request was filed for
///
/// > *the operator places a sticky and wants a different icon* → **delete it
/// > and place another**
///
/// That was the state until 2026-09-06, and the cost was never the icon: it was
/// the object identity, the `/M` stamp and any reply thread hung off the note,
/// all lost to a delete-and-replace.
///
/// # ★ A combo, where the placing dialog uses radios
///
/// Deliberately different, and the difference is the surface rather than the
/// choice. The dialog is a **transaction** with room to spare and one question
/// to ask, so seven radios read at a glance. The properties panel is a narrow
/// column shared with every other section, and seven rows here would push the
/// colour swatch and the delete control off the visible part of it. §5.8's
/// division of labour says the panel *carries everything*, which is an argument
/// for the control existing, not for it being the tallest thing on screen.
///
/// # ★ `/Text` only, and absent otherwise
///
/// [`Face::takes_icon`]. A `/Stamp`'s face comes from Table 181's own
/// vocabulary and the engine refuses a `StickyIcon` on one **by name** rather
/// than ignoring it — `EditError::StylePropertyNotApplicable`, raised before
/// anything is written, because *"silently ignoring this would be the
/// swallowed-`width` defect `Pass 258.0` closed, one family along."*
fn icon_row(
    ui: &mut Ui,
    current: &Reading,
    target: &crate::canvas::selection::annot::AnnotTarget,
    actions: &mut Vec<Action>,
) {
    if !current.face.takes_icon() {
        return;
    }
    ui.horizontal(|ui| {
        ui.label(tt::sticky_icon_heading());
        let mut chosen = current.icon.clone();
        egui::ComboBox::from_id_salt("properties-textannot-icon") // ui-text-exempt: internal widget id, never displayed
            .selected_text(match &current.icon {
                // ★★★ **The file's own name, shown as the file spells it** —
                // 2026-09-07. This arm used to be unreachable for an unmodelled
                // name (`icon` was forced to `None` and the combo read *"Not
                // one of these"*), because the engine's reader flattened such a
                // name to `Note` and this shell could only detect the loss, not
                // carry it. `Pass 253.5` carries it, so the honest thing is to
                // print it.
                Some(StickyIcon::Other(name)) => {
                    ts::markup_icon_foreign_named(&String::from_utf8_lossy(name))
                }
                Some(icon) => tt::sticky_icon_label(icon).to_owned(),
                // A face with no icon at all reaches this only through a build
                // error — `takes_icon` returned above — so it says nothing
                // rather than inventing an entry.
                None => String::new(),
            })
            .show_ui(ui, |ui| {
                // ★★ **The file's own name is the FIRST entry when it is not
                // one of the seven**, and this is not decoration. A combo whose
                // current value is absent from its own list is a one-way door:
                // the operator opens it to look, picks something to see what it
                // does, and cannot get back to what the file said. Offering it
                // costs one row and it is the only row that can restore the
                // document's own state.
                if let Some(other @ StickyIcon::Other(name)) = &current.icon {
                    ui.selectable_value(
                        &mut chosen,
                        Some(other.clone()),
                        ts::markup_icon_foreign_named(&String::from_utf8_lossy(name)),
                    );
                    ui.separator();
                }
                for icon in crate::canvas::textannot::STICKY_ICONS {
                    ui.selectable_value(
                        &mut chosen,
                        Some(icon.clone()),
                        tt::sticky_icon_label(icon),
                    );
                }
            });
        if let Some(icon) = chosen
            && Some(&icon) != current.icon.as_ref()
        {
            actions.push(Action::Annot(AnnotAction::SetTextAnnotStyle {
                id: target.id,
                style: TextAnnotStyle {
                    icon: Some(icon),
                    // Left alone — see [`colour_row`]'s note on naming every
                    // other field explicitly, and on why a `..Default::default()`
                    // here would be a silent decline rather than a shorthand.
                    color: None,
                    font_size: None,
                    stamp_fit: None,
                },
            }));
        }
    });
    // ★★★ The two sentences under the chooser, and they are about different
    // things.
    //
    // The first is always true and says the icon changes the FILE and not
    // pdfcer's own picture.
    //
    // ★★ **The second was REWRITTEN on 2026-09-07 because it had become
    // false.** It used to warn that *"changing its icon OR its colour here will
    // replace that icon with one of the ones listed"*, which was exactly right
    // against the engine that normalised an unmodelled `/Name` to `Note` — a
    // colour-only restyle really did rewrite the icon. `Pass 253.5` made the
    // reader lossless, so a colour change now carries the name through
    // untouched and the warning was describing a destruction that no longer
    // happens.
    //
    // ⇒ A limitation sentence on this project has a shelf life measured in
    // hours. What survives is the part that was never about loss: pdfcer draws
    // its own glyph whatever the name says.
    //
    // Neither is a hover: an operator who has to hover to find out what a
    // control does has already been given the chance not to.
    ui.label(egui::RichText::new(tt::sticky_icon_bound()).small().weak());
    if current.foreign_icon {
        ui.label(
            egui::RichText::new(ts::markup_icon_foreign_note())
                .small()
                .weak(),
        );
    }
}
