//! # `text::status` — every string the status bar shows
//!
//! One area of the catalog described in [`crate::text`]'s header, and the
//! sole consumer is [`crate::app::status`]. Nothing here is read by the
//! ribbon: the status bar **mirrors** three View-tab commands under
//! amendment P1a (`RIBBON_IA.md` §2), and a mirror is a second surface for
//! one command, not a second command.
//!
//! ## ★ The one place this file deliberately repeats the ribbon
//!
//! [`fit_actual_size`] and [`fit_actual_size_tooltip`] say what
//! `crate::text::commands::view_zoom_actual` says, in the same words. That
//! is not an oversight and it is not a copy-paste that should be
//! de-duplicated into a shared constant:
//!
//! - The two surfaces are mirrors of **one** command, so an operator who
//!   reads the ribbon's tooltip and then hovers the status bar's button
//!   must be told the same thing. Two paraphrases of one command is how a
//!   product acquires two different mental models of the same verb.
//! - They are nevertheless two *entries*, because the ribbon's catalog is
//!   keyed by command id and consumed by `crate::shell::commands`, while
//!   this one is keyed by control and consumed by a widget. Reaching across
//!   would make `text::status` depend on `text::commands`' `CommandText`
//!   type for no gain, and would put the first cross-area dependency in a
//!   catalog whose whole organising principle is one area per consumer.
//!
//! **Both entries are now true**, and the wording being identical to the
//! ribbon's is what made fixing them one edit rather than two. The action
//! behind them raises `Action::ZoomTo(1.0)` (see the ★ section of
//! [`crate::app::status`]'s module docs for that half), and the chord they
//! name has one owner (see [`crate::app::keyboard`]'s ★ section for this
//! one). The same holds for the three mirrored fit tooltips: each is now
//! word-for-word its `crate::text::commands` twin, chord included.
//!
//! ## Why the arrows and the minus sign are in the catalog
//!
//! `⏴`, `⏵`, `⏷`, `−`, `+`, `·` are *labels*: they are the entire visible
//! text of a control, and a control's visible text is exactly what rule R1
//! governs. The `check-ui-strings.sh` heuristic would never catch them —
//! it flags literals containing whitespace, and these contain none — so
//! they are here by the rule rather than by the gate, which is the
//! distinction that file's own header draws.
//!
//! They are also the reason
//! [`crate::app::status::tests::every_glyph_the_status_bar_draws_has_a_glyph`]
//! exists, and **that test has already paid for itself**. This file was
//! written with the obvious glyphs — `◀` `▶` for the page steps and `▸` `▾`
//! for the disclosure — and every one of the four is **missing from egui's
//! bundled font set** (Ubuntu-Light + NotoEmoji + emoji-icon-font). On
//! screen they would have rendered as tofu boxes: defect D2's shape, an
//! invisible label, with the operator's page position behind it. What the
//! font set does carry, measured rather than assumed, is `⏴ ⏵ ⏶ ⏷ ‹ › « »
//! ○ • · – — − + % /`.
//!
//! ## Conventions, restated from [`crate::text`] because they bind here
//!
//! - **Sentence case, no trailing period on labels; full sentences with
//!   punctuation for prose.** Every tooltip below is prose and ends in a
//!   full stop; every label is a name and does not.
//! - **Name the chord only when the chord works.** Actual size names
//!   `Ctrl+0` because the manifest keymap binds it there and
//!   `crate::app::keyboard::commands` enacts what the keymap says. Fit page
//!   and Fit width name **none**, because none reaches them: `Ctrl+0` is
//!   actual size's and `Ctrl+2` is `mode.review`'s. Both used to be named
//!   here, and both were half of a chord with two owners — the rule is what
//!   caught it, and the rule is why the correction is an omission rather
//!   than a substitution. Do not invent a replacement chord to fill the gap;
//!   bind one in the manifest first, and it may be named the same day.
//! - **Never state a capability the build does not have.** The Find toggle
//!   `RIBBON_IA.md` §6 specifies has **no strings here**, and that is now a
//!   filing decision rather than an absence: the toggle exists, and its label
//!   and tooltip live in [`crate::text::find`] beside the rest of the Find
//!   surface's copy. One area per consumer is this catalog's organising
//!   principle, and the toggle's consumer is a control the Find module owns.
//!   (It used to say the toggle had no strings *because it had no command*.
//!   That is no longer true, and the sentence is corrected rather than
//!   deleted, because "this used to be absent and is now built" is exactly
//!   what a catalog header should make legible.)

mod formdelete;
mod refused;
mod selection;

/// ★ Re-exported for the reason stated on [`selection`]'s block below, and it
/// earns a file of its own rather than a place in this catalog because its
/// argument is long: a **decline** that repeats a panel's **standing
/// description** almost word for word has to justify every word it does not
/// share with it, or the two surfaces become two paraphrases of one fact.
pub use formdelete::field_delete_declined_structural;

/// ★ Re-exported on [`formdelete`]'s precedent and for the same two reasons —
/// R2 forced a file, the subject decided which one — with one addition that is
/// this entry's whole difficulty: **it is the only sentence in this catalog
/// that names no cause**, because the engine exposes none this shell may
/// switch on. Its file carries the argument for every word it does not say,
/// including why it does not point the operator at Render diagnostics.
pub use refused::edit_declined_by_engine;

/// ★ Re-exported rather than moved-and-repathed.
///
/// A catalog area is keyed by the **consumer** it serves, not by the file it
/// happens to live in, and `crate::app::status::selected` is still the one
/// consumer of every one of these. Splitting the file to satisfy R2 while
/// leaving `t::selection_one` resolving exactly where it always did is the
/// difference between a structural fix and a churn commit — no call site in
/// this crate changed, and none should have to when a catalog area is
/// reorganised internally.
/// **Sentences about the program being busy** — a different species from
/// everything else in this catalog: a STATE rather than an event, with no
/// retirement rule. Its header carries the distinction.
pub mod waiting;
pub use waiting::page_catching_up;

pub use selection::{
    TextStyleRefusal,
    inside_container,
    selection_inside_form_declined,
    selection_many,
    selection_one,
    selection_one_in_form,
    selection_one_in_form_unsized,
    selection_one_unsized,
    selection_with_depth,
    text_style_faked_instead,
    text_style_faked_warning,
    text_style_multi,
    text_style_used_other_family,
    text_style_used_real_face,
    // ★ O69's sibling of `too_many_anchors` below. It lives in `selection`
    // rather than in this file because this file is at 1,482 lines against
    // R2's 1,500, and it is re-exported here so a caller says
    // `text::status::too_many_anchors_in_part` beside
    // `text::status::too_many_anchors` — two sentences about one limit, named
    // the same way, which is what stops the second being missed.
    too_many_anchors_in_part,
};

// ---------------------------------------------------------------------------
// The narrator — the render diagnostics disclosure
// ---------------------------------------------------------------------------

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

// ---------------------------------------------------------------------------
// The edit disclosure — rule 4's surviving half for the vector verbs
//
// ★ Almost every word an operator reads here was written by `pdfcer-core`,
// and that is the point of this section rather than a shortcut through it.
//
// `EditSession`'s vector verbs return `Result<Vec<String>, EditError>`, and
// the `Vec<String>` is a **disclosure list**: sentences the surgery owes the
// operator because it had to change an operator's *form* to express the
// request — an `re` rectangle rewritten as four lines so one corner could
// move, an implicitly-started subpath's `m` materialised, a curve dropped
// with the point it ran into. They are already finished English prose,
// written where the fact is known, and they are passed through **verbatim**.
//
// So what belongs in this catalog is only the *framing this shell adds*:
// the warning mark, the lead-in that ties the sentences to the gesture that
// produced them, and the separator between two of them. Re-wording core's
// sentences here would put two descriptions of one surgery in the product,
// and the one further from the code would be the one on screen.
// ---------------------------------------------------------------------------

/// One line for the disclosures the last vector edit returned.
///
/// `notes` are `pdfcer-core`'s own sentences, in the order the planner pushed
/// them, unmodified. This function contributes three things and nothing else:
///
/// 1. **A mark**, `⚑` (U+2691). The status bar's other left-hand line is
///    *narration* — a census of what a raster contained — and this one is a
///    fact about the operator's own document that they cannot see by looking
///    at it. A mark is what tells them apart at a glance.
///
///    ★ **It is deliberately NOT `⚠`, and that is a measurement rather than
///    a preference.** `⚠` (U+26A0) is what
///    [`crate::text::forms::forms_fill_autosize_note`] and twelve other
///    forms sentences carry, and **egui's bundled font set cannot draw it**
///    — Ubuntu-Light + NotoEmoji + emoji-icon-font, which is the whole set,
///    because nothing in this workspace installs a font of its own. Every
///    one of those sentences renders `□` today, on the panel and in this
///    bar. That is defect D2's shape and it is
///    [`crate::app::status::tests::every_glyph_the_status_bar_draws_has_a_glyph`]
///    that caught it, on its third sighting of the same hazard.
///
///    The forms catalog is not corrected here because the convention is
///    thirteen sentences wide and one of its assertions lives in
///    `crate::panels::forms::tab_order`, outside this change's territory —
///    it is **reported**, which is what a boundary finding gets. What is in
///    this project's gift is not to add a fourteenth undrawable mark, and
///    `⚑` is the closest drawable neighbour: measured present in the same
///    bundled set, alongside `✱ ★ ☆ ! ○ ■ • · † ‡ ⊗ ◊ №`. It is also the
///    mark this file would recommend for the other thirteen, so a future
///    correction converges rather than adding a third spelling.
/// 2. **A lead-in naming the gesture**, because the sentence outlives the
///    gesture: it stands until the next edit or an undo retires it (see
///    [`crate::app::actions::last_edit_disclosure`]), and core's sentences
///    open with *"This shape…"* / *"This point…"* — deictic words that are
///    unambiguous at the moment of the drag and unanchored a minute later.
/// 3. **A single space between sentences**, matching the way
///    [`crate::app::status`] joins the two form-fill notes into its one row.
///    Not [`diagnostics_join`]'s `·`: that separator exists because the
///    render notes are independent *fragments*, and these are whole
///    sentences with their own full stops.
///
/// # Why the lead-in does not say what changed
///
/// The obvious wording — *"that edit changed how the page is written"* —
/// would be a claim, and it is **false of at least one note core can
/// return**: the clipping-region disclosure fires on a move that rewrote
/// operands in place and rewrote nothing's form, and says instead that the
/// shape controls what other content is visible elsewhere. A lead-in that
/// asserted a rewrite would be contradicted by the sentence immediately
/// after it. "About your last edit" is true of every note in the list,
/// which is the property a frame has to have when it does not know which
/// note it is framing.
#[must_use]
pub fn edit_disclosure_line(notes: &[String]) -> String {
    format!("⚑ About your last edit: {}", notes.join(" "))
}

// ---------------------------------------------------------------------------
// The worded decline — a framing zoom that had nothing to frame
//
// ★ A DECLINE IS NOT A DISCLOSURE, AND THE COPY HAS TO SAY SO
//
// The section above frames sentences `pdfcer-core` wrote about work that
// *happened*: a rectangle really was rewritten as four lines, and the operator
// is owed the part they cannot see. These two strings are the opposite speech
// act. Nothing happened. The command was invoked, it looked at what it had to
// work with, and it declined.
//
// One slot and one wording for both would make a completed gesture and a
// refused one wear the same sentence in the same place, which is worse than
// the trace-only state these strings replace — an operator who reads
// "About your last edit" after a gesture that did nothing has been told a
// small lie confidently. So the lead-in diverges (*"Nothing to zoom to"*, not
// *"About your last edit"*) and the mark diverges with it: `⊗` (U+2297) rather
// than `⚑` (U+2691).
//
// `⊗` was chosen because it reads as *"this did not happen"* rather than as
// *"look at this"*, and because it is drawable. That second half is measured,
// not assumed: `crate::icons::glyphs`' header records which codepoints egui's
// bundled proportional chain (Ubuntu-Light → NotoEmoji-Regular →
// emoji-icon-font) actually supplies, and `⊗` is among those confirmed
// present. It is checked twice on every run anyway — by
// `crate::app::status::tests::every_glyph_the_status_bar_draws_has_a_glyph`,
// which lists this bar's labels by hand, and by
// `crate::icons::glyphs::tests::every_glyph_the_catalog_draws_has_a_glyph`,
// which reads every literal in this directory from source. A tofu box on a
// decline would read as a rendering failure, which is exactly how an operator
// decides a surface is broken and stops reading it.
//
// # ★ What is deliberately NOT worded here
//
// **The raster-ceiling-clamped region zoom.** A framing zoom that asked for
// more magnification than the page's raster allows still zooms, still centres
// what was asked for, and raises `Action::ZoomTo` carrying the **clamped**
// scale — so the zoom readout three controls to the right states the scale
// actually pinned, on the same frame. That is a partial grant that already
// reports itself, and a sentence saying so would word a non-event. See
// `crate::canvas::zoom::ZoomOutcome::ceiling_changed_the_answer`, which
// carries the argument in full.
//
// # No trailing full stop
//
// These sit in the same slot as `page_clamped_note` and `page_rejected_note`
// — a short note beside a control, read at a glance in a small weak face —
// and they follow that precedent rather than the prose one. The tooltips in
// this file are prose and end in a full stop; these are notes and do not.
// ---------------------------------------------------------------------------

/// Shown when zoom-to-selection was invoked with nothing it could frame.
///
/// # The three causes, and why they get one sentence
///
/// `crate::canvas::zoom::zoom_to_selection` raises
/// `ZoomOutcome::NoBounds` in three situations — nothing is selected, the
/// selection is on another page, or it no longer resolves against the current
/// decomposition after an edit. That function's own docs rule that from the
/// operator's side those are **one** situation: *"there is nothing on screen
/// for this command to act on."* Three sentences would ask the operator to
/// care about a distinction that has one remedy.
///
/// # ★ Why it describes the state and does not instruct
///
/// `view.zoom_selection` is greyed on `selection.bounds`, so this is *mostly*
/// unreachable from the ribbon. The two ways it is reached are both cases in
/// which blaming the operator would be wrong:
///
/// 1. **By chord.** A keymap reaches any command from any state, and the
///    manifest binds this one; nobody who presses a chord has clicked a
///    control that promised anything.
/// 2. **★ In the race.** The condition is evaluated on the frame that *draws*
///    the control and the verb runs on the frame that *applies* it, so a
///    selection that evaporates in between — a mode change that clears it, an
///    edit that dissolves what it named — leaves the operator having clicked
///    an enabled control and been declined. That is the case an operator finds
///    most confusing, and a sentence reading *"select something first"* would
///    tell them to do the thing they just did.
///
/// So it reports the state, at the moment the command ran: *nothing on this
/// page is selected right now*. "Right now" is doing work — it dates the
/// claim to the gesture rather than asserting a standing fact about an
/// operator who may already have fixed it.
#[must_use]
pub fn zoom_declined_no_selection() -> &'static str {
    "⊗ Nothing to zoom to — nothing on this page is selected right now"
}

/// Shown when a framing zoom was invoked before the canvas had drawn a page.
///
/// `ZoomOutcome::NoCanvas`: there is no viewport, no page rect and no scroll
/// offset yet, so there is nothing to frame *into*. Kept separate from
/// [`zoom_declined_no_selection`] because the remedy is different and the
/// operator has to do nothing at all to reach it — it resolves itself on the
/// next raster, which is what "yet" and "has not finished" promise.
///
/// Reachable in practice only by a chord fired at a document that has just
/// opened, or on a very slow first raster of a dense CAD sheet, where ~99 % of
/// the render cost is resolution-independent and the first frame can take
/// most of a second.
#[must_use]
pub fn zoom_declined_not_drawn() -> &'static str {
    "⊗ Nothing to zoom to yet — the page has not finished drawing"
}

/// Shown when `file.save_copy` asked where to write, was told, and could not.
///
/// # Why a decline needs wording here more than anywhere else on this bar
///
/// The other two sentences beside it describe a command that was refused
/// *before* the operator invested anything. This one arrives after they opened
/// a dialog, chose a folder and typed a name — and the only other evidence they
/// would get is a file that is not there. Silence would make a save that failed
/// indistinguishable from a save that never ran, which is precisely the "the
/// button does nothing" state this project exists to remove.
///
/// # ★ Why it does not carry the engine's reason
///
/// `crate::app::save::SaveError`'s `Display` output goes to the trace, and
/// `check-ui-strings.sh`'s exclusion 3 states in as many words that a `Display`
/// impl "is not permission to route UI text through an error type". A
/// cross-reference form that could not express an entry is a true sentence and
/// not one an operator can act on.
///
/// # Why it names the two things they CAN act on
///
/// The folder and the permission, because between them they are almost every
/// real instance: a path typed into the dialog whose parent does not exist, a
/// network share that went away, a read-only volume, a file open in another
/// program. It reports the check to make rather than blaming the operator —
/// [`zoom_declined_no_selection`]'s rule, which
/// [`tests::the_decline_reports_the_state_rather_than_instructing_the_operator`]
/// enforces for that sentence — because the commonest cause is not something
/// they did.
#[must_use]
pub fn save_copy_failed() -> &'static str {
    "⊗ The copy was not written — check that the folder exists and can be written to"
}

/// Shown when the Settings window's Save reached no disk.
///
/// # ★ Why this is not [`save_copy_failed`]'s sentence, though both are writes
///
/// Because the two have to say **opposite things about the operator's work**,
/// and getting that backwards costs them either their trust or their time.
///
/// A failed save-a-copy produced no file: nothing happened, and the operator
/// should try again. A failed settings save is the reverse — pdfcer **adopted
/// the configuration anyway**, deliberately, because a disk that refuses should
/// not cost somebody a choice they deliberately made. So what is true is *"this
/// is in force now, and it will be gone when you restart"*, and the sentence
/// has to carry both halves or it is misleading in one direction or the other:
///
/// - Say only "settings were not saved" and the operator makes the choice
///   again, or concludes the setting does not work.
/// - Say only "settings applied" and they restart and lose it silently, which
///   is the failure the whole store exists to prevent.
///
/// # Why the reason is not in the sentence
///
/// The store's `SaveError` has a `Display` — *"no writable location"*, *"could
/// not write settings to {path}: {reason}"* — and it is a developer's sentence,
/// not an operator's. It goes to the trace beside the store kind, from
/// `crate::app::settings_window`. The operator's actionable half is *the
/// folder*, and the settings window itself states which folder that is, on a
/// line it draws every time it opens.
#[must_use]
pub fn settings_not_saved() -> &'static str {
    "⊗ Your choices are in use now but could not be written down — they will be \
     gone when pdfcer restarts"
}

/// Shown when `edit.undo` was invoked and the command log was empty.
///
/// # Why this sentence exists at all, when the control is greyed
///
/// Because the route that reaches it is **the keyboard**, and the keyboard is
/// the one route on which the greyed control explains nothing. `edit.undo` is
/// gated on `undo.available`, so its quick-access button is un-pressable with an
/// empty log — but it is also bound to `Ctrl+Z`, and
/// `app::modes::capability::offers_command` lets it through in every mode
/// because it sits on no tab. An operator who presses `Ctrl+Z` is looking at the
/// page, not at an 18 pt icon in the title bar, and silence there is
/// indistinguishable from a chord that never arrived — which is the state
/// `HANDOFF.md` §2 exists to remove.
///
/// It is the same argument [`save_copy_failed`] makes about *its* route, one
/// step earlier: that one arrives after the operator invested a dialog, this one
/// after they invested the single most reflexive keystroke in any editor.
///
/// # Why it says *this document* rather than *nothing*
///
/// The log is per-[`EditSession`](pdfcer_core::edit::EditSession), which is
/// per-document: closing a document and opening another empties it. "Nothing to
/// undo" alone would read as a claim about the application, and an operator who
/// had just undone six things in the file they closed would read it as a defect.
///
/// # Why it does not name the remedy
///
/// There is none — nothing has been changed, so there is nothing to take back,
/// and the sentence is a complete report of the state. It reports rather than
/// instructs, which is [`zoom_declined_no_selection`]'s rule and the one
/// [`tests::the_decline_reports_the_state_rather_than_instructing_the_operator`]
/// pins for that sentence.
#[must_use]
pub fn undo_declined_empty() -> &'static str {
    "⊗ Nothing to undo — this document has no changes to take back"
}

/// Shown when `edit.redo` was invoked and the redo stack was empty.
///
/// Kept separate from [`undo_declined_empty`] because the two states are
/// reached differently and a reader has one line to tell them apart. An empty
/// **undo** log means nothing has been changed; an empty **redo** stack is the
/// ordinary state of a document that has been edited and never undone — and it
/// is also what a *new edit after an undo* produces, because the engine clears
/// the redo stack when a fresh command is recorded. One sentence for both would
/// tell an operator who just pressed `Ctrl+Y` after ten edits that their
/// document has no changes, which is false.
#[must_use]
pub fn redo_declined_empty() -> &'static str {
    "⊗ Nothing to redo — nothing has been undone, or a new change replaced it"
}

// ---------------------------------------------------------------------------
// Zoom
// ---------------------------------------------------------------------------

/// The zoom-out button's label — `−` (U+2212 MINUS SIGN).
///
/// Not the ASCII hyphen. A hyphen next to a `+` reads as a dash rather than
/// as an operator, and the two controls are meant to be seen as a pair.
#[must_use]
pub fn zoom_out() -> &'static str {
    "−"
}

/// Hover text for zoom out.
#[must_use]
pub fn zoom_out_tooltip() -> &'static str {
    "Zoom out one step (Ctrl+Minus)."
}

/// The zoom-in button's label.
#[must_use]
pub fn zoom_in() -> &'static str {
    "+"
}

/// Hover text for zoom in.
#[must_use]
pub fn zoom_in_tooltip() -> &'static str {
    "Zoom in one step (Ctrl+Plus)."
}

/// The current zoom, as a whole percentage.
///
/// A **readout**, not a control: this build has no way to set an arbitrary
/// zoom by typing, so an editable box here would be an affordance for
/// something that cannot happen. The page number beside it *is* editable
/// because `Action::GoToPage` exists; there is no `Action` that sets a zoom
/// to a named value, and inventing a text box in front of one would be the
/// placeholder the project's invariants forbid.
#[must_use]
pub fn zoom_percent(percent: f64) -> String {
    // ★ `{:.0}` rather than an integer cast — O24j. The value now spans 10 % to
    // a trillion percent and no integer type covers it without either
    // saturating or being wider than the thing it describes. Rounding at the
    // formatter keeps the readout an exact whole number of percent at every
    // magnitude, which is what it always showed.
    format!("{percent:.0}%")
}

/// Hover text for the zoom readout.
///
/// Explains the ladder, because "why did 137% become 150%?" is the question
/// the readout provokes and the answer is a deliberate design choice
/// (`crate::viewer`'s module docs: a fixed ladder makes zoom-in-then-out
/// exactly reversible).
#[must_use]
pub fn zoom_percent_tooltip() -> &'static str {
    "The current zoom. The − and + buttons step a fixed ladder of familiar \
     percentages, so zooming in and back out returns to exactly where you \
     started."
}

// ---------------------------------------------------------------------------
// Fit — the three View-tab mirrors (amendment P1a)
// ---------------------------------------------------------------------------

/// The Actual size button's label.
///
/// **Identical to `crate::text::commands::view_zoom_actual`'s label**, on
/// purpose — see this module's header for why a mirror repeats rather than
/// paraphrases, and [`crate::app::status`]'s ★ section for why the claim it
/// makes is not yet true.
#[must_use]
pub fn fit_actual_size() -> &'static str {
    "Actual size"
}

/// Hover text for Actual size.
///
/// ★ **It names `Ctrl+0` again, and that sentence is now true.** The chord
/// had two owners — the manifest keymap bound it to `view.zoom_actual` while
/// `crate::app::keyboard` bound it to Fit page and reached it first — so this
/// tooltip had to advertise no chord at all, with a test pinning the
/// omission. `crate::app::keyboard`'s ★ section has the whole account; the
/// outcome is that the manifest is the only place a chord is bound, and it
/// binds this one here.
///
/// Word for word `crate::text::commands::view_zoom_actual`'s tooltip,
/// including the chord — see this module's header on why a mirror repeats
/// rather than paraphrases.
#[must_use]
pub fn fit_actual_size_tooltip() -> &'static str {
    "Show the page at actual size — one PDF point per screen point (Ctrl+0)."
}

/// The Fit width button's label.
#[must_use]
pub fn fit_width() -> &'static str {
    "Fit width"
}

/// Hover text for Fit width.
///
/// Says "and keep it fitted", because a fit is a **mode** here rather than a
/// one-shot: resizing the window re-fits. A viewer that stopped fitting on
/// the first resize would be conspicuously wrong, and the tooltip is where
/// the operator learns which of the two this is.
///
/// ★ **It no longer names `Ctrl+2`.** That chord belongs to `mode.review`
/// (`MODES_AND_PANELS.md` Part 1 §6, and `crate::text::commands::mode_review`
/// names it), and `crate::app::keyboard` bound it here as well — one chord,
/// two owners, of which this was the half nothing but this string admitted
/// to. Fit width keeps this button, its View ▸ Zoom control and its
/// `canvas.empty` context-menu entry; what it does not have is a chord, and
/// the rule in this module's header is to say so by omission rather than to
/// name one that does something else.
#[must_use]
pub fn fit_width_tooltip() -> &'static str {
    "Scale the page so its full width is visible, and keep it fitted as the \
     window resizes."
}

/// The Fit page button's label.
#[must_use]
pub fn fit_page() -> &'static str {
    "Fit page"
}

/// Hover text for Fit page.
///
/// ★ **It no longer names `Ctrl+0`.** See [`fit_actual_size_tooltip`]: that
/// chord now has one owner, the manifest keymap, and the manifest binds it to
/// actual size. Fit page is reached from this button, from View ▸ Zoom and
/// from the `canvas.empty` context menu.
///
/// Word for word `crate::text::commands::view_zoom_fit_page`'s tooltip, which
/// has never named a chord — the two mirrors of one command now say exactly
/// the same thing, which is what the header claims they should.
#[must_use]
pub fn fit_page_tooltip() -> &'static str {
    "Scale the page so all of it is visible, and keep it fitted as the \
     window resizes."
}

/// The Fit height button's label.
#[must_use]
pub fn fit_height() -> &'static str {
    "Fit height"
}

/// Hover text for Fit height.
///
/// Word for word `crate::text::commands::view_zoom_fit_height`'s tooltip, as
/// this module's header requires of every status-bar mirror of a ribbon
/// command — and, like its two siblings, it names no chord, because it has
/// none.
///
/// ★ It says nothing about the page overflowing sideways, deliberately. That
/// is what *"its full height is visible"* already means on a sheet wider than
/// the window, and a tooltip that warned about it would be describing the
/// operator's own document back at them.
#[must_use]
pub fn fit_height_tooltip() -> &'static str {
    "Scale the page so its full height is visible, and keep it fitted as the window resizes."
}

// ---------------------------------------------------------------------------
// Page navigation, and the editable page box
// ---------------------------------------------------------------------------

/// The wheel-paging toggle's label — `OPERATOR_REQUESTS.md` O30.
///
/// Two words, because it shares a 24-point bar with the page buttons, the zoom
/// readout and three fit controls. It names the state the control **turns on**
/// rather than the state it is in, which is what a pressed/unpressed toggle
/// already reports: *Flip pages*, lit, means the wheel flips pages.
#[must_use]
pub fn wheel_flip_pages() -> &'static str {
    "Flip pages"
}

/// Hover text for the wheel-paging toggle.
///
/// ★ It states **both** answers, because the label can only state one and the
/// operator needs to know what turning it off gives them back.
///
/// ★★ And it names the two things the setting does **not** touch. Ctrl+wheel
/// always zooms, and a continuous display always scrolls — an operator who
/// tried the toggle in a continuous mode and saw no difference would
/// reasonably conclude it was broken, which is why the control is not drawn
/// there at all and why this sentence says so.
#[must_use]
pub fn wheel_flip_pages_tooltip() -> &'static str {
    "Turn the mouse wheel into a page turn: one notch, one sheet. Switch it off and the wheel scrolls within the page instead. Ctrl+wheel always zooms, and a continuous page display always scrolls."
}

/// The previous-page button's label — `⏴` (U+23F4).
///
/// **Not `◀` (U+25C0)**, which `RIBBON_IA.md` §6 spells the control with and
/// which egui's bundled fonts cannot draw — see [`diagnostics_toggle`] for
/// the measurement and the test that caught it. `⏴`/`⏵` are the same shape
/// at a slightly smaller optical size, and they are what this font set has.
#[must_use]
pub fn prev_page() -> &'static str {
    "⏴"
}

/// Hover text for the previous-page button.
#[must_use]
pub fn prev_page_tooltip() -> &'static str {
    "Previous page (Page Up)."
}

/// The next-page button's label — `⏵` (U+23F5). See [`prev_page`].
#[must_use]
pub fn next_page() -> &'static str {
    "⏵"
}

/// Hover text for the next-page button.
#[must_use]
pub fn next_page_tooltip() -> &'static str {
    "Next page (Page Down)."
}

/// The page number, as the editable box shows it.
///
/// **1-based.** `crate::viewer::ViewState::page_index` is 0-based and the
/// conversion happens here, once, exactly as that module's own docs
/// prescribe: *"The UI displays it 1-based; the conversion happens once, in
/// the string catalog."*
#[must_use]
pub fn page_number(page_1_based: usize) -> String {
    format!("{page_1_based}")
}

/// The total, shown to the right of the editable box.
///
/// `/ 42` rather than `of 42`: `RIBBON_IA.md` §6 spells the control
/// `page ◀ n/N ▶`, and the slash is narrower — which matters on a control
/// that sits between two buttons in a fixed-height bar.
#[must_use]
pub fn page_of_total(total: usize) -> String {
    format!("/ {total}")
}

/// Hover text for the editable page box.
///
/// States the commit rule, because it is the one thing about this control
/// that is not visible: nothing happens per keystroke, so an operator typing
/// `42` must be able to trust that passing through `4` did not move them.
#[must_use]
pub fn page_box_tooltip() -> &'static str {
    "Type a page number and press Enter. Nothing moves while you type, and \
     a number past the end of the document goes to the nearest page and \
     says so."
}

/// Shown beside the box when a committed number was outside the document.
///
/// ★ **The point of this string is that the clamp is not silent.** Typing
/// `99` into a 42-page document and landing on 42 with no explanation is
/// indistinguishable from the box ignoring what was typed — and an operator
/// who cannot tell those apart stops trusting the control. Naming the number
/// that does not exist, and the page they got instead, makes the clamp a
/// *report* rather than a shrug.
///
/// `asked` is the 1-based number typed; `landed` and `total` are 1-based
/// page numbers.
#[must_use]
pub fn page_clamped_note(asked: usize, landed: usize, total: usize) -> String {
    format!("No page {asked} — went to {landed} of {total}")
}

/// Shown beside the box when the committed text was not a page number.
///
/// The operator's text is deliberately **left in the box** when this
/// appears, so the note explains something still visible rather than
/// describing a value that has already been thrown away.
#[must_use]
pub fn page_rejected_note() -> &'static str {
    "Not a page number — type digits, then Enter"
}

// --- Registering an unclaimed form control ---------------------------------

/// `adopt_widget` refused: the name is already another field's.
///
/// # ★ Why the sentence explains the standard rather than just refusing
///
/// Because the refusal looks arbitrary otherwise. Every other program the
/// operator uses will happily hold two things with one name in one file, and
/// "that name is taken" reads as pdfcer being fussy about a namespace it made
/// up.
///
/// It is not pdfcer's namespace. ISO 32000-2 SS12.7.3.1 makes the fully
/// qualified name the field's **identity**: two top-level fields called
/// `Address` are one field with two boxes, and filling either fills both. So
/// the second half of the sentence is the part that does the work — it says
/// what would happen if pdfcer allowed it, which is the only thing that makes
/// the refusal obviously right rather than obviously annoying.
#[must_use]
pub const fn adopt_declined_name_taken() -> &'static str {
    "Another field in this document already uses that name. In a PDF, two fields with the same \
     name are one field with two boxes — filling either would fill both — so pdfcer needs a \
     different name."
}

/// `adopt_widget` refused: the widget carries no name and none was typed.
///
/// # ★ The word this sentence must not use is "restore"
///
/// The operator's mental model at this moment is *"something was lost, and I
/// am putting it back"*, and for the common case that is exactly right — a
/// merged field-widget carries its own name, type and value, and registering
/// it recovers the field as it was.
///
/// This is the other case, and it is not that. The box was a **bare kid**: its
/// name, its field type, its radio flags and its value all lived in a field
/// dictionary that is not in this document. Naming it here **creates a new
/// field** with no type and no value. That is a legitimate thing to want, and
/// it is not a recovery — an operator told they had restored a radio button
/// would go looking for its group, and there is no group.
///
/// So the sentence offers the name box and says what naming it will produce,
/// and it names the only route that gets the original back.
#[must_use]
pub const fn adopt_declined_no_name() -> &'static str {
    "This box carries no name of its own, so pdfcer has nothing to register it under. Type a name \
     to make it a new, empty field — its original name, type and any value it had are not in \
     this file. To get those back, insert the pages again from the document they came from."
}

/// The disclosure after a widget was registered.
///
/// # ★ Three facts, each conditional, and none of them is "done"
///
/// `AdoptOutcome` carries three things the operator cannot see and would not
/// guess, and each is dropped when it is not true rather than being reported as
/// a negative:
///
/// - **the name it went in under** — always said, because for a blank box it is
///   the name the file already carried, which the operator has never seen;
/// - **`field_type: None`** — legal (`/FT` is inheritable) and useless, because
///   a top-level field has nothing left to inherit from. No viewer knows how to
///   render or fill it. This is the fuzzy-never-sneaky half that would
///   otherwise be invisible: the registration **succeeded** and the box is
///   still not fillable;
/// - **`acroform_created`** — the document had no interactive form at all and
///   now has one, which changes what other software does with the file.
#[must_use]
pub fn adopted(name: &str, typed: bool, acroform_created: bool) -> String {
    let mut line = format!("Registered as \u{201c}{name}\u{201d}.");
    if !typed {
        line.push_str(
            " It has no field type, so no viewer knows how to fill it — pdfcer cannot give it one \
             without the field definition it lost.",
        );
    }
    if acroform_created {
        line.push_str(" This document had no interactive form before; it has one now.");
    }
    line
}
/// **`edit.form_flatten` was invoked on a document whose certification forbids
/// it.**
///
/// # ★★ Why the ribbon control is live at all, when the panel's is greyed
///
/// The Forms panel asks `EditSession::flatten_refusal` every frame and greys
/// its own Flatten with the reason on hover, because it is already reading the
/// session to draw the field list. A **ribbon** `enabled_when` is a condition
/// name evaluated against a published set, and publishing this one would mean
/// a certification query per frame for a control that is almost never pressed.
///
/// So the ribbon control is `enabled_when("doc.pages")` and the arm declines in
/// words. That is this project's standing division and `app::dispatch::forms`'
/// own header states it: *greying is a hint; the worded decline is the answer.*
///
/// # What the sentence has to carry
///
/// **Which gate refused**, because flatten and fill take *different* ones and
/// an operator who has just successfully typed into the form will otherwise
/// conclude the button is broken. On the ordinary real-world shape — a
/// certified fillable form at `/P 2` — filling is permitted and flattening is
/// refused, by design and by the standard.
///
/// **What it would cost**, because "the signature would be broken" is the fact
/// that makes the refusal reasonable rather than arbitrary.
///
/// ★ It does **not** offer a way round. There is one — remove the signature —
/// and pdfcer will not suggest defeating a certification as a workaround for a
/// convenience.
#[must_use]
pub const fn flatten_declined_certified() -> &'static str {
    "This document is certified, and flattening its fields would break the signature. Filling \
     is still allowed; turning the values into page content is not."
}

/// **A resize was refused because the artwork cannot be rebuilt.**
///
/// `OPERATOR_REQUESTS.md` O51. Two sentences, chosen by whether the drag was
/// proportional, and the split is the whole value of the message: **only one of
/// the two switches helps in each case**, and naming the wrong one would send
/// the operator to a control that changes nothing.
///
/// | drag | what fixes it |
/// |---|---|
/// | proportional | *Scale line weight* — the resize then comes out **exact** |
/// | not proportional | nothing fixes it; only *Allow the artwork to distort* proceeds |
///
/// ★★★ **It does not say "cannot".** The operator resized a shape and got
/// nothing; what they need is the next click, not a diagnosis. Both sentences
/// name a switch by the words on it, and the non-uniform one is honest that the
/// result will be imperfect rather than dressing the option up.
///
/// ★★ Neither sentence mentions appearance streams, placement matrices or
/// §12.5.5. The *reason* is real and is written down in `canvas::scaling`; what
/// belongs in a status bar is what to do. A sentence that explained the matrix
/// would be correct, unactionable, and too long to read where it appears.
///
/// ★ *"pdfcer did not draw this shape"* is in the uniform sentence because it is
/// the part an operator can verify and act on — shapes pdfcer drew resize
/// perfectly, so the message quietly tells them the difference between the two
/// kinds of object on their page.
#[must_use]
pub const fn resize_not_rebuildable(uniform: bool) -> &'static str {
    if uniform {
        "pdfcer did not draw this shape, so its border will thicken as the shape grows. Turn on Scale line weight in the Tool panel and the resize comes out exactly right."
    } else {
        "pdfcer did not draw this shape, and stretching it more in one direction than the other would leave its border uneven — no PDF can describe that. Resize it proportionally, or turn on Allow the artwork to distort in the Tool panel to go ahead anyway."
    }
}

/// Shown when the renderer reports `cmyk_buffer_refused` — the page's raster
/// grew past the size the engine will composite in subtractive CMYK, so
/// blending fell back to sRGB and the colours moved.
///
/// # ★★ Every word of this was chosen against a specific misreading
///
/// **"at this zoom"**, not "on this page". The operator's report was
/// *"different results depending on Zoom level"*, and the thing that must land
/// is that the page has not changed — the view has. A sentence blaming the
/// document would send him looking at the file.
///
/// **"zoom out"** rather than "reduce the zoom", because it is the instruction,
/// and it is the opposite of what somebody chasing a colour difference tries.
/// Measured on an A4 page the boundary is 534 %; naming a number here would be
/// worse than useless, because it depends on the page size and on the display
/// density, and would be wrong on the next document.
///
/// **"approximate"**, not "wrong". The fallback is a known, counted
/// approximation that pdfcer has shipped for its whole life and that most pages
/// never reach; calling it wrong would overstate it and invite a bug report
/// about a page that is fine.
///
/// ★ It does not apologise and does not promise a fix. What it owes the
/// operator is the fact and the remedy, and it gives both in one line that fits
/// a status bar.
#[must_use]
pub fn blend_space_status_line() -> String {
    "Colours are approximate at this zoom \u{2014} the page is too large to blend in print \
     colours here. Zoom out to see the exact colours."
        .to_owned()
}

/// The status-bar line for a document whose index pdfcer had to rebuild.
///
/// ★★★ One sentence, stating the fact and where to look, and stopping. It does
/// not warn, does not instruct, and carries no counters — the numbers live in
/// Properties, and a status line long enough to hold three of them would push
/// the zoom and page controls off a narrow window.
///
/// ★ It says **"rebuilt"** rather than "repaired" or "fixed". Repaired implies
/// the file is now correct; rebuilt says what actually happened — pdfcer
/// reconstructed the index by scanning, which is a best reading of damaged
/// bytes and may or may not be the one the author intended. The operator's
/// trust in the page should follow the weaker word.
#[must_use]
pub const fn recovered_status_line() -> &'static str {
    "This file's index was damaged — pdfcer rebuilt it to open the document. The Properties panel says what was recovered."
}

/// ★★★ **Show points was switched on and the object has more anchors than the
/// canvas will draw.**
///
/// # The state this exists for, and why silence was the wrong answer
///
/// `overlay::MAX_UNSELECTED_ANCHORS` is 400, and the cap is right: five
/// thousand hollow squares over a CAD path is noise rather than an answer, and
/// the cap's own note argues that at length.
///
/// But it means `view.show_points` **does nothing visible on exactly the
/// drawings this program is for**. A 5,000-node path toggled on and off looks
/// identical, and the operator's report would be *"Show points is broken"* —
/// which is what the toggle was wired to stop happening in the first place,
/// arriving through a different door.
///
/// Rule 4's half that survives: *an inference the operator cannot see still
/// owes an off-canvas report.* The canvas is not marked — nothing is drawn on
/// the page to indicate suppression — and the status bar carries the number.
///
/// ★ It names **both** numbers. The count alone would not say the cap is the
/// reason; the cap alone would not say how far past it they are. An operator
/// who sees *"5,903 … 400"* knows immediately that no setting is going to help
/// and that the answer is to enter a part.
///
/// ★★ It also names the remedy, and the remedy is real: descending into a
/// subpath narrows the anchor list to that subpath, which is nearly always
/// under the cap. That is the route the Points tool takes and it is one click.
#[must_use]
pub fn too_many_anchors(count: usize, cap: usize) -> String {
    format!(
        "This object has {count} points and pdfcer draws at most {cap} at once, so none are \
         shown. Double-click into a part of it, or use the Points tool, to see that part's."
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The disclosure must say which state it is in.
    ///
    /// A toggle whose two states read identically is a toggle whose state
    /// the operator has to discover by clicking it, which is exactly the
    /// affordance a disclosure triangle exists to remove.
    #[test]
    fn the_disclosure_reads_differently_open_and_closed() {
        assert_ne!(diagnostics_toggle(false), diagnostics_toggle(true));
    }

    /// ★ **`pdfcer-core`'s sentences reach the operator unaltered, and on one
    /// line.**
    ///
    /// Two properties, and both are load-bearing:
    ///
    /// - **Verbatim.** The disclosure prose is written where the fact is known
    ///   — inside the planner that decided to rewrite a rectangle as four
    ///   lines. A shell that paraphrased it would put two descriptions of one
    ///   surgery into the product, and the one further from the code is the
    ///   one on screen. So this asserts *containment*, which is the mechanical
    ///   form of "we added framing and changed nothing".
    /// - **One line.** `crate::app::status` draws this inside a row whose
    ///   height may not vary (R128), eliding what does not fit. A newline
    ///   would defeat that from the string side, where no layout assertion is
    ///   looking — the label wraps, the row grows, and the page re-fits itself
    ///   at the moment the operator finishes a drag.
    #[test]
    fn the_edit_disclosure_frames_cores_sentences_without_altering_them() {
        let notes = vec![
            "This shape was stored as a rectangle, so it has been rewritten as four lines."
                .to_owned(),
            "This point had no coordinates of its own — the file re-used an earlier start."
                .to_owned(),
        ];
        let line = edit_disclosure_line(&notes);

        for note in &notes {
            assert!(
                line.contains(note.as_str()),
                "core's sentence was altered on the way to the bar: {line}"
            );
        }
        assert!(
            !line.contains('\n'),
            "the bar gets one line, and a newline in the string wraps the label \
             regardless of what the layout asks for: {line}"
        );
        assert!(
            line.starts_with('⚑'),
            "the mark is what tells a disclosure apart from the narration beside it: {line}"
        );
    }

    /// ★ **A decline does not read like a disclosure.**
    ///
    /// The whole reason the worded decline got its own strings — rather than
    /// borrowing the line the vector verbs already put in this bar — is that
    /// *"this happened, and here is the part you cannot see"* and *"this did
    /// not happen"* are different speech acts. One wording in one slot would
    /// make a completed gesture and a refused one indistinguishable, in the
    /// same place, which is worse than the trace-only state these replace.
    ///
    /// Four properties, and each would be silently lost by an edit that looked
    /// harmless:
    ///
    /// 1. **The lead-in diverges.** Neither decline may open with the edit
    ///    disclosure's *"About your last edit"*.
    /// 2. **The mark diverges.** `⊗` and not `⚑` — the mark is what tells the
    ///    two apart at a glance, before either sentence is read.
    /// 3. **The two declines read differently from each other.** "Nothing is
    ///    selected" and "the page is still drawing" have different remedies,
    ///    and the operator gets one line to tell them apart — the same
    ///    property [`the_two_page_box_notes_read_differently`] defends for the
    ///    page box's pair.
    /// 4. **One line.** `crate::app::status` draws these inside a row whose
    ///    height may not vary (R128), eliding what does not fit. A newline
    ///    would defeat that from the string side, where no layout assertion is
    ///    looking.
    #[test]
    fn a_decline_does_not_read_like_a_disclosure() {
        let declines = [
            zoom_declined_no_selection(),
            zoom_declined_not_drawn(),
            save_copy_failed(),
        ];
        let disclosure = edit_disclosure_line(&["x".to_owned()]);

        for line in declines {
            assert!(
                line.starts_with('⊗'),
                "a decline carries the mark that says nothing happened: {line}"
            );
            assert!(
                !line.starts_with('⚑'),
                "`⚑` is the disclosure's mark; a decline wearing it is a \
                 refused gesture dressed as a completed one: {line}"
            );
            assert!(
                !line.contains("About your last edit"),
                "a decline must not borrow the disclosure's lead-in — nothing \
                 happened, so there is no last edit to be about: {line}"
            );
            assert_ne!(line, disclosure);
            assert!(!line.contains('\n'), "the bar gets one line: {line}");
        }
        // Pairwise rather than one comparison, because the operator gets ONE
        // line to tell these apart and each has a different remedy: select
        // something, wait, or fix the destination. A third sentence that
        // duplicated either of the first two would be a decline that says
        // nothing about which command declined.
        for (i, a) in declines.iter().enumerate() {
            for b in &declines[i + 1..] {
                assert_ne!(
                    a, b,
                    "two declines with different remedies must not share a sentence"
                );
            }
        }
    }

    /// ★ **The decline does not tell the operator they did something wrong.**
    ///
    /// `view.zoom_selection` is greyed on `selection.bounds`, so the sentence
    /// is reached by a chord — or, worse, in the race where the selection
    /// evaporates between the frame that drew the enabled control and the
    /// frame that applied it. In that second case the operator clicked a
    /// control that was offered to them, and an imperative telling them to
    /// select something first would be instructing them to repeat what they
    /// just did.
    ///
    /// Asserted as an absence of imperatives rather than against a fixed
    /// string, so the copy can be rewritten and the property survives.
    #[test]
    fn the_decline_reports_the_state_rather_than_instructing_the_operator() {
        let line = zoom_declined_no_selection();
        for imperative in ["Select ", "select something", "first", "try again"] {
            assert!(
                !line.contains(imperative),
                "the control was enabled and then declined; {imperative:?} \
                 blames the operator for a race they cannot see: {line}"
            );
        }
        assert!(
            line.contains("right now"),
            "the sentence has to date its claim to the gesture rather than \
             assert a standing fact about a selection the operator may \
             already have made: {line}"
        );
    }

    /// Every counted note in this module, so a new one cannot be added
    /// without inheriting the checks below.
    const COUNTERS: [fn(usize) -> String; 9] = [
        diagnostics_contents_missing,
        diagnostics_fonts_skipped,
        diagnostics_images_skipped,
        diagnostics_glyphs_notdef,
        diagnostics_glyphs_substituted,
        diagnostics_glyphs_supplied,
        diagnostics_layers_hidden,
        diagnostics_ops_deferred,
        diagnostics_ops_unknown,
    ];

    /// ★ **One is singular everywhere it can be.**
    ///
    /// Not pedantry: these lines are read in a small weak font at the edge
    /// of the window, and "1 glyphs" is the kind of thing a reader notices
    /// *instead of* the number, which is the part that matters.
    ///
    /// The property is asserted structurally rather than against a table of
    /// expected sentences: **the singular must not be the plural with the
    /// digit swapped.** That catches a missing branch on every entry,
    /// including the ones whose noun is not the first word ("text from 1
    /// font not drawn"), and it keeps working when the copy is edited.
    #[test]
    fn every_counted_note_is_singular_at_one() {
        for f in COUNTERS {
            let one = f(1);
            let two = f(2);
            assert!(one.contains('1'), "a count must show its number: {one}");
            assert!(two.contains('2'), "a count must show its number: {two}");
            assert_ne!(
                one,
                two.replacen('2', "1", 1),
                "the singular form is missing — this note reads as a plural at \
                 a count of one: {one}"
            );
        }
    }

    /// The join is what makes several notes one line.
    #[test]
    fn notes_join_into_one_line() {
        let joined = diagnostics_join(&[
            diagnostics_glyphs_substituted(3),
            diagnostics_images_skipped(1),
        ]);
        assert!(joined.contains('·'));
        assert!(
            !joined.contains('\n'),
            "the bar has exactly one row: {joined}"
        );
    }

    /// **★ The clamp note names both numbers.**
    ///
    /// The whole value of the note is that it distinguishes "your number was
    /// out of range" from "the box ignored you". A note that named only the
    /// page landed on could not do that.
    #[test]
    fn the_clamp_note_names_what_was_asked_and_what_was_given() {
        let note = page_clamped_note(99, 42, 42);
        assert!(note.contains("99"), "{note}");
        assert!(note.contains("42"), "{note}");
    }

    /// The two failure notes must not read alike.
    ///
    /// "I typed a page that does not exist" and "I typed something that is
    /// not a page number" are different mistakes with different fixes, and
    /// the operator gets one line to tell them apart.
    #[test]
    fn the_two_page_box_notes_read_differently() {
        assert_ne!(page_clamped_note(99, 42, 42), page_rejected_note());
    }

    /// The page number shown is 1-based, and the total reads as a total.
    #[test]
    fn the_page_number_is_one_based_and_the_total_is_labelled() {
        assert_eq!(page_number(1), "1");
        assert_eq!(page_number(42), "42");
        assert!(page_of_total(42).contains("42"));
        assert!(
            page_of_total(42).starts_with('/'),
            "the total must read as a denominator, not as a second page number"
        );
    }

    /// A zoom readout is a percentage.
    #[test]
    fn the_zoom_readout_carries_its_unit() {
        assert_eq!(zoom_percent(100.0), "100%");
        assert_eq!(zoom_percent(8.0), "8%");
    }

    /// **The three fit labels are distinct, and so are their tooltips.**
    ///
    /// Three controls in a row that read alike is the failure the ribbon's
    /// own salvage notes record (two adjacent Content buttons both reading
    /// `Aa`, distinguished only by their tooltips). Here both halves are
    /// asserted.
    #[test]
    fn the_three_fit_controls_are_distinguishable() {
        let labels = [fit_actual_size(), fit_width(), fit_page()];
        let tooltips = [
            fit_actual_size_tooltip(),
            fit_width_tooltip(),
            fit_page_tooltip(),
        ];
        for i in 0..3 {
            for j in (i + 1)..3 {
                assert_ne!(labels[i], labels[j]);
                assert_ne!(tooltips[i], tooltips[j]);
            }
        }
    }

    /// **★ Each fit tooltip names exactly the chord that reaches it.**
    ///
    /// This test used to pin the *opposite* facts — Actual size naming no
    /// chord, Fit page naming `Ctrl+0`, Fit width naming `Ctrl+2` — because
    /// `Ctrl+0` and `Ctrl+2` each had two owners and this file was the
    /// surface that had to keep quiet about it. With one owner per chord
    /// (`crate::app::keyboard`'s ★ section) the three claims invert, and the
    /// test inverts with them rather than being deleted: the property worth
    /// defending was never "Actual size is silent", it was **"a status-bar
    /// tooltip names a chord if and only if that chord reaches the control"**.
    ///
    /// The three assertions below are the direct expression of that, and the
    /// last two are the ones that matter most — a chord *removed* from the
    /// keyboard leaves no compile error behind, so the only thing standing
    /// between the operator and a tooltip that lies is an assertion that the
    /// string stayed silent.
    #[test]
    fn each_fit_tooltip_names_exactly_the_chord_that_reaches_it() {
        assert!(
            fit_actual_size_tooltip().contains("Ctrl+0"),
            "the manifest binds Ctrl+0 to view.zoom_actual and the keyboard enacts it: {}",
            fit_actual_size_tooltip()
        );
        assert!(
            !fit_page_tooltip().contains("Ctrl"),
            "no chord reaches Fit page in this build: {}",
            fit_page_tooltip()
        );
        assert!(
            !fit_width_tooltip().contains("Ctrl"),
            "Ctrl+2 belongs to mode.review, not to Fit width: {}",
            fit_width_tooltip()
        );
    }

    /// **★ The mirrors say exactly what the ribbon says.**
    ///
    /// The header's claim, asserted rather than trusted. Three status-bar
    /// controls mirror three View ▸ Zoom commands under amendment P1a, and a
    /// mirror that paraphrases is how a product acquires two mental models of
    /// one verb. It is also what let the chord defect live on one surface and
    /// not the other for as long as it did.
    #[test]
    fn the_fit_mirrors_repeat_the_ribbon_word_for_word() {
        use crate::text::commands as c;
        assert_eq!(fit_actual_size(), c::view_zoom_actual().label);
        assert_eq!(fit_actual_size_tooltip(), c::view_zoom_actual().tooltip);
        assert_eq!(fit_page(), c::view_zoom_fit_page().label);
        assert_eq!(fit_page_tooltip(), c::view_zoom_fit_page().tooltip);
        assert_eq!(fit_width(), c::view_zoom_fit_width().label);
        assert_eq!(fit_width_tooltip(), c::view_zoom_fit_width().tooltip);
    }

    /// The page-box tooltip must state the commit rule.
    ///
    /// It is the only place the operator can learn that typing does not
    /// navigate, and that property is the reason the control is usable at
    /// all — see `crate::app::status`.
    #[test]
    fn the_page_box_tooltip_states_the_commit_rule() {
        let t = page_box_tooltip();
        assert!(t.contains("Enter"), "{t}");
        assert!(t.contains("while you type"), "{t}");
    }
}
