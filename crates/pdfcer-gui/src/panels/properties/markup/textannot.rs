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
//! ## ⚠ One more thing the file can say that the reader normalises away
//!
//! `text_spec_from_dict`'s `/Text` arm (`annot_author.rs:963`) reads `/Name`,
//! runs it through `StickyIcon::from_name`, and `.unwrap_or(StickyIcon::Note)`.
//! §12.5.6.4's seven names are *"a standard set, not a closed one"* — a
//! producer's own icon name is conforming — so a note carrying `/Sparkle`
//! reads back as `Note`, and a restyle of its **colour alone** rewrites its
//! `/Name` to `/Note`. Silent, and not recoverable by looking.
//!
//! ⇒ [`Reading::foreign_icon`] catches that by reading the raw `/Name` off the
//! dictionary itself and comparing, and [`rows`] discloses it **before** the
//! operator touches anything. This is the shell's own read rather than
//! `pdfcer_core::annot::Annotation::icon` (`annot.rs:449`), which reports the
//! same raw bytes for the same reason — that field is on the reader's
//! page-walk view and this section already has the dictionary in hand, so
//! asking for it would be a second walk for a value one `.get(b"Name")` away.
//! The engine's field is what a shell **without** the dictionary would use.

use egui::Ui;
use pdfcer_core::annot_author::{Color, StickyIcon, TextAnnotSpec};
use pdfcer_core::edit::TextAnnotStyle;
use pdfcer_core::graph::ObjectGraph;
use pdfcer_core::object::Object;

use crate::app::actions::Action;
use crate::app::actions::annot::AnnotAction;
use crate::text::panels::properties as t;
use crate::text::panels::textannotstyle as ts;
use crate::text::textannot as tt;

/// The region this subsection publishes, so a driven check can find the icon
/// chooser on a placed note rather than only on the placing dialog.
pub(super) const REGION: &str = "properties.markup.textannot"; // ui-text-exempt: trace region name, never displayed

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
#[derive(Debug, Clone, Copy)]
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
/// `Copy`, like [`super::Current`], and for the same reason: it is read fresh
/// every frame off the session and passed by value into the row functions, so
/// there is no borrow to thread through a section that also reads the session.
#[derive(Debug, Clone, Copy)]
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
    /// `/Name`, for a `/Text`, as one of the seven pdfcer models.
    ///
    /// `None` for a `/Stamp` (whose `/Name` is a stamp face, a different
    /// vocabulary) and for a `/Text` whose `/Name` is one pdfcer does not
    /// model — see [`Self::foreign_icon`].
    pub(super) icon: Option<StickyIcon>,
    /// ★★★ **`true` when the file's `/Name` is a name pdfcer does not model.**
    ///
    /// §12.5.6.4's seven are *"a standard set, not a closed one"*, so a
    /// producer's own icon name is conforming. `text_spec_from_dict`
    /// normalises one to `Note` on the way past, which means the chooser would
    /// show *Note* for a note that says something else — and a restyle of the
    /// **colour alone** would write that `Note` into the file.
    ///
    /// ⇒ It is read off the dictionary rather than off the spec for exactly the
    /// reason [`super::Current::endings_key_present`] is: this is the one fact
    /// the engine's reader deliberately erases, and the disclosure is the whole
    /// point of catching it.
    pub(super) foreign_icon: bool,
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
    /// ★ It takes the spec the caller already read rather than reading one
    /// itself, which is [`super::Current::from_spec`]'s rule: **one call to the
    /// engine's reader per frame**, its verdict carried, so this panel and the
    /// verb cannot come to disagree about the same annotation.
    ///
    /// ★★ And it takes the raw `/Name` **bytes** rather than a graph and a
    /// dictionary, for the same reason `from_spec` takes a spec rather than an
    /// `OpenDoc`: this is the part with the decisions in it, and a test that
    /// wants to ask *"what does a `/Sparkle` do?"* should not have to build a
    /// document to ask. [`read_icon_name`] is the one-line dictionary read that
    /// feeds it, and it is the part a test cannot reach and does not need to.
    pub(super) fn of(spec: &TextAnnotSpec, raw_icon: Option<&[u8]>) -> Option<Self> {
        match spec {
            TextAnnotSpec::Sticky { color, icon, .. } => {
                // ★ Foreign means **present and unmodelled**. An ABSENT
                // `/Name` is not foreign: Table 172's own default is `Note`, so
                // showing `Note` for a note that carries no `/Name` is
                // reporting the standard rather than inventing anything, and a
                // restyle that writes `/Note` writes what was already in
                // effect. The distinction is the same one
                // `Annotation::color` draws between an absent `/C` and an
                // explicitly empty one.
                let foreign_icon = raw_icon.is_some_and(|n| StickyIcon::from_name(n).is_none());
                Some(Self {
                    face: Face::Sticky,
                    colour: super::swatch_of(Some(color)),
                    icon: (!foreign_icon).then_some(*icon),
                    foreign_icon,
                })
            }
            TextAnnotSpec::Stamp { color, .. } => Some(Self {
                face: Face::Stamp,
                colour: super::swatch_of(Some(color)),
                icon: None,
                foreign_icon: false,
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
}

/// **The file's own `/Name`, before the engine's reader normalises it** — the
/// one-line dictionary read that feeds [`Reading::of`]'s `raw_icon`.
///
/// ★ Separated from [`Reading::of`] so that everything with a decision in it is
/// reachable from a unit test and everything needing a document is one
/// `.get()`. `pdfcer_core::annot::Annotation::icon` (`annot.rs:449`) reports
/// exactly these bytes, for exactly this reason — the module header records why
/// this section reads the dictionary it already holds instead.
pub(super) fn read_icon_name<G: ObjectGraph + ?Sized>(
    graph: &G,
    dict: &pdfcer_core::object::Dict,
) -> Option<Vec<u8>> {
    match dict.get(b"Name").map(|o| graph.resolve(o)) {
        Some(Object::Name(n)) => Some(n.as_bytes().to_vec()),
        _ => None,
    }
}

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
    current: Reading,
    target: &crate::canvas::selection::annot::AnnotTarget,
    actions: &mut Vec<Action>,
) {
    crate::diag::ui_rect(REGION, ui.max_rect());
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
    icon_row(ui, current, target, actions);
    ui.label(
        egui::RichText::new(ts::markup_text_annot_note())
            .small()
            .weak(),
    );
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
    current: Reading,
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
                    // ★ Explicit `None`, and it is the contract rather than a
                    // formality: "a field left `None` is left alone", so a call
                    // that names only the colour does not touch the icon. Spelt
                    // out rather than reached through `..Default::default()`
                    // because `TextAnnotStyle` has exactly two fields and
                    // naming both is what makes the omission visible.
                    icon: None,
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
    current: Reading,
    target: &crate::canvas::selection::annot::AnnotTarget,
    actions: &mut Vec<Action>,
) {
    if !current.face.takes_icon() {
        return;
    }
    ui.horizontal(|ui| {
        ui.label(tt::sticky_icon_heading());
        let mut chosen = current.icon;
        egui::ComboBox::from_id_salt("properties-textannot-icon") // ui-text-exempt: internal widget id, never displayed
            .selected_text(match current.icon {
                Some(icon) => tt::sticky_icon_label(icon),
                // ★ The file's `/Name` is one pdfcer does not model, so there
                // is no entry to show as selected. The word says that rather
                // than the combo silently showing the first entry, which would
                // be the panel asserting a value the file does not carry.
                None => ts::markup_icon_foreign(),
            })
            .show_ui(ui, |ui| {
                for icon in crate::canvas::textannot::STICKY_ICONS {
                    ui.selectable_value(&mut chosen, Some(*icon), tt::sticky_icon_label(*icon));
                }
            });
        if let Some(icon) = chosen
            && chosen != current.icon
        {
            actions.push(Action::Annot(AnnotAction::SetTextAnnotStyle {
                id: target.id,
                style: TextAnnotStyle {
                    icon: Some(icon),
                    // Left alone — see [`colour_row`]'s note on the same field.
                    color: None,
                },
            }));
        }
    });
    // ★★★ The two sentences under the chooser, and they are about different
    // things. The first is always true and says the icon changes the FILE and
    // not pdfcer's own picture; the second appears only for a note whose
    // `/Name` pdfcer does not model, and warns that ANY change here — the
    // colour included — replaces it. Neither is a hover: an operator who has
    // to hover to find out that a control will discard something has already
    // been given the chance not to.
    ui.label(egui::RichText::new(tt::sticky_icon_bound()).small().weak());
    if current.foreign_icon {
        ui.label(
            egui::RichText::new(ts::markup_icon_foreign_note())
                .small()
                .weak(),
        );
    }
}
