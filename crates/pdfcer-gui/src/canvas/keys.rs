//! # `canvas::keys` — the keys the canvas owns, and who gets Escape
//!
//! Escape, Delete and **Tab**, and nothing else. They are split out of
//! `canvas/mod.rs`
//! along a real seam rather than for line count: everything else in that file
//! is *wiring* — it needs an `egui::Ui`, a laid-out scroll area and a live
//! document, and cannot be exercised without a window — whereas this is a
//! decision about keys that a headless `egui::Context` can drive end to end.
//! Its tests came with it, which is the test for whether a split was along a
//! seam.
//!
//! ## Tab, and why it is guarded on the armed tool rather than on a mode
//!
//! Tab advances the **snap cycle** while a measure tool is armed
//! ([`crate::canvas::measure::cycle_snap`]) — the operator's way of saying
//! *"the other candidate"* when an endpoint and a midpoint are a few pixels
//! apart and pointing cannot separate them.
//!
//! Tab is `egui`'s focus key, so the guard matters more here than for the
//! other two. It is the **armed tool**, not a mode and not a capability:
//! with no measure tool armed the branch is false, the key is untouched, and
//! keyboard navigation of every panel and dialog behaves exactly as it did.
//! A mode-shaped guard would have been wrong in both directions — Review and
//! Edit both offer the measure tools, and in neither of them should Tab stop
//! moving focus when the operator is not measuring.
//!
//! ## ★ Six claimants for Escape, one press, one effect
//!
//! Decision 025's L1 is that Escape ascends **exactly one rung** rather than
//! collapsing the ladder, and the same discipline governs everything else that
//! would like the key. By Phase 6 there are six claimants, and the precedence
//! is *"retire the most transient thing first"*:
//!
//! | # | claimant | who decides | how it says it took the key |
//! |---|---|---|---|
//! | 0 | a **form field being typed into** | [`crate::canvas::forms`] | [`crate::canvas::forms::escape_spent`]: `true` when a draft was abandoned |
//! | 1 | a **drag in flight**, including a markup band | [`crate::canvas::gesture::GestureState::update`] — the only thing that knows whether there is one | [`crate::canvas::gesture::GestureOutcome::Cancelled`], arriving here as `escape_consumed` |
//! | 2 | a **guide drag in flight** | [`crate::canvas::guides::cancel_drag`] | its return value: `true` when there was one |
//! | 3a | a **measure pick** or a **markup vertex run** in progress | [`crate::canvas::measure::abandon`] / [`crate::canvas::markup::vertex::abandon`] | its return value: `true` when there was one |
//! | 3b | an **armed markup or measure tool** | [`crate::canvas::tool::disarm_markup`] / [`crate::canvas::tool::disarm_measure`] | its return value: `true` when there was one armed |
//! | — | ★ the **armed text tool** is deliberately **not** a claimant — see below | — | — |
//! | 4 | an **armed region zoom** | [`crate::canvas::zoom::disarm_region_zoom`] | its return value: `true` when there was something to retire |
//! | 5 | the **selection ladder**, or the **text selection** | [`crate::canvas::selection::SelectionState::escape`] / clearing [`crate::canvas::textsel::TextSelection`] | it is last, so it acts only when none above did |
//!
//! ### ★ Why the text selection shares rung 5 rather than taking a sixth
//!
//! The original argument was that it is not a precedence decision at all:
//!
//! > **The two occupants of rung 5 can never both be present.**
//! > `canvas::textsel::takes_the_press` gives a press its text meaning exactly
//! > when `Capabilities::edit_content` is absent, and the content selection is
//! > reachable exactly when it is present — one flag, two mutually exclusive
//! > branches […] there is no frame in which both could claim the key.
//!
//! ★ **That is no longer true, and the rung is right anyway.** Since
//! [`crate::canvas::tool::CanvasTool::Text`] landed (2026-08-14) an operator in
//! **Edit** can marquee some objects with the select tool, arm the text tool,
//! sweep a line, and hold both selections at once — `takes_the_press` gained a
//! disjunct, so the two are no longer decided by one flag in opposite senses.
//! `canvas::textsel` §3 records that move from exclusivity-by-construction to
//! exclusivity-by-precedence in full.
//!
//! So there **is** an ordering now, and it is the one the code already had:
//! **the text selection first.** Three reasons, in weight order:
//!
//! 1. **It is the more transient**, which is this table's own rule. A text range
//!    is made by the gesture the operator is performing right now and is
//!    destroyed by the very next edit anyway (`textsel` §7's epoch rule); an
//!    object selection survives edits, navigation, zoom and the mode change that
//!    hid it.
//! 2. **It is what the operator just did.** Reaching this state requires arming
//!    a tool and sweeping, and the press that follows means the thing that press
//!    could plausibly be about.
//! 3. **The content ladder has rungs and a text range has none**, which is the
//!    asymmetry the paragraph below already describes: clearing the text
//!    selection costs the operator one re-sweep, while ascending the object
//!    ladder loses the sub-path or node they had descended into. Given a
//!    choice, spend the press on the cheaper loss.
//!
//! A sixth rung is **still** not the answer, and now for a different reason than
//! before. It would put a *precedence* between two things that are the same act
//! — "clear what is selected" — expressed twice because this shell has two kinds
//! of selectable thing; and it would make a mode in which only one of them can
//! exist (Read, Review) look as though it had two rungs to climb. The rung is
//! "the selection"; which selection is answered by what is actually there.
//!
//! Both branches obey L1 identically: one press clears, and a **second** press
//! then does nothing, because there is nothing left. That is deliberately
//! unlike the content ladder, whose first press *ascends* and whose second
//! clears — a text range has no rungs to ascend, since there is no larger unit
//! than the sweep the operator made and no smaller one they have descended into.
//!
//! ### ★ Why the armed TEXT tool takes no rung at all
//!
//! Rung 3b retires an armed markup or measure tool, so the obvious symmetry is a
//! third call beside them. It is deliberately absent, and the decision is
//! recorded here because its absence looks exactly like an omission.
//!
//! **What rung 3b is actually for.** Both tools it retires paint a **crosshair**
//! that promises a gesture which *writes to the document*, and a mis-armed pen is
//! costly enough that the operator needs a universal way out of it. The text tool
//! paints an I-beam and promises a selection; it authors nothing (see
//! [`crate::canvas::tool::retire_forbidden`], which permits it in every mode for
//! the same reason). Nothing about it needs an emergency exit.
//!
//! **What the rung would collide with.** Escape at rung 5 already means *clear
//! the selection* while this tool is armed, because clearing a text selection is
//! what rung 5's first branch does. A rung *below* that — which is where the
//! transience rule would put it, the tool being less transient than the range —
//! would make the second press silently move the operator from **sweeping text**
//! to **marqueeing objects** in Edit: a change of what the primary button means,
//! delivered by the key they pressed to clear something. One press, one effect
//! is satisfied; *one press, one **expected** effect* is not.
//!
//! **What the reference applications do**, under `HANDOFF.md` §3's standing
//! instruction: Inkscape's Escape in the text tool deselects and **stays in the
//! tool**; Acrobat's Escape changes no tool; only SolidWorks exits the active
//! command. Two of three keep the tool, which is what ships — and it is also the
//! answer that needs no code.
//!
//! The route out is the control that armed it: `view.tool_text` is a toggle, so
//! pressing it again returns to the select tool
//! ([`crate::canvas::tool::toggle_text`]). That is the same *the button is
//! pressed, so pressing it un-presses it* rule the four markup buttons follow —
//! rung 3b is the **extra** affordance those tools get, not their only one.
//!
//! ### ★ Why a measure pick is TWO rungs rather than one
//!
//! A markup **band** is a drag, so abandoning it and retiring the pen are
//! already separated by the table: the drag is claimant 1, the tool is claimant
//! 3b. A measure pick is a sequence of **clicks**, so there is no drag for
//! claimant 1 to cancel — and yet a linear dimension with point A taken and
//! point B not is unmistakably a gesture in flight. Without 3a, one Escape
//! would put the tool down *and* silently discard that pick: two effects from
//! one press, which is exactly what decision 025's L1 forbids.
//!
//! ★ **The same argument admitted a second occupant to 3a on 2026-08-14**, and
//! the fact that it needed no new reasoning is the point. PolyLine and Polygon
//! are also gestured by clicks
//! ([`crate::canvas::markup::vertex`]), so a run of three vertices with the
//! fourth not yet placed is in exactly the position a half-taken linear pick is.
//! [`crate::canvas::markup::vertex::abandon`] therefore sits beside
//! [`crate::canvas::measure::abandon`] rather than taking a rung of its own:
//! the two cannot both be in progress, because a measure tool and a markup tool
//! cannot both be armed, so this is **one claimant expressed as two calls** —
//! which is precisely the arrangement rung 3b already uses for the two `disarm`
//! functions.
//!
//! It also means the sentence *"a markup is a drag"* is now only three-quarters
//! true, and the quarter that is not is why the rung was needed. The band kinds
//! and Ink are drags; the two vertex kinds are not.
//!
//! So the pick is retired first and the tool second, which is also the order
//! the transience rule gives — a half-taken pick is the more transient of the
//! two, and it is the thing the operator is most likely to have meant.
//! Pressing Escape twice puts the tool down; pressing it once corrects a
//! mis-aimed first click without leaving the tool.
//!
//! ### ★ Why a focused form field is rung 0, and why its rung is unlike every
//! other
//!
//! It is numbered 0 rather than 1 because it does not merely *outrank* the
//! others — while it is live, **none of them can see the key at all**. A
//! focused field is an `egui::TextEdit`, so `Context::text_edit_focused` is
//! true, and that predicate is the first line of this very function
//! (`DEFECTS.md` D1's guard) as well as the gate `interact` builds the gesture
//! machine's `cancel` flag from. So the exclusion is **mechanical**, not
//! ordered, and there is no version of this table in which a marquee and a
//! half-typed field compete for one press.
//!
//! What the rung is actually for is the *other* direction, and it is a real
//! hazard rather than a formality. `egui`'s own `TextEdit` surrenders focus on
//! Escape, and it does so **before** this function runs — so by the time the
//! guard above is asked, `text_edit_focused` is already false and the key
//! falls straight through to the selection ladder. One press would abandon the
//! draft *and* ascend a rung: exactly the double effect L1 forbids, and exactly
//! the shape the `escape_consumed` flag was invented for one row down.
//! [`crate::canvas::forms::escape_spent`] is the same report-rather-than-
//! re-derive contract, read once and cleared by the reading.
//!
//! ### ★ Where the markup tool sits, and why the transience rule does not
//! settle it
//!
//! Row 1 needed **no change at all**, and that is the first thing to notice: a
//! markup band is a [`DragKind`](crate::canvas::gesture::DragKind), so a markup
//! drag in flight is already the *drag in flight* claimant, cancelled by the
//! existing branch, with no new mechanism and no second rule. Abandoning it
//! authors nothing — the annotation is only written by
//! [`crate::app::actions::Action::CommitMarkup`], which the release raises and
//! a cancellation never does.
//!
//! Retiring the armed **tool** is a different act, and it needed a row. Its
//! placement is the one judgement call here, because the table's own rule does
//! not decide it: an armed markup tool is *less* transient than an armed region
//! zoom, not more. The zoom arming is a one-shot, spent by the very next drag;
//! the markup tool is a mode the operator stays in while they draw five
//! rectangles. On transience alone it would sit **below** the zoom.
//!
//! It sits above it, and the deciding argument is the one the guide-versus-zoom
//! row already makes in the paragraph below: *"there is no reading of that press
//! under which they meant a zoom they armed earlier and have not used."* That
//! is true here twice over, and the second reason is mechanical rather than a
//! matter of intent — **while the markup tool is armed, the region zoom cannot
//! be reached at all.** [`crate::canvas::gesture::press_kind`] gives an armed
//! markup tool the primary drag unconditionally, so the armed zoom is inert for
//! as long as the pen is down. Spending the operator's Escape on retiring
//! something inert, while they are looking at an armed Rectangle button that
//! does not un-press, is one press with no visible effect — and the effect it
//! *does* have is on a control that lives on a different ribbon tab, where they
//! cannot see it.
//!
//! So the ordering rule is unchanged and is simply not the one that applies to
//! this pair. What applies is the rule underneath it: **retire the thing the
//! operator could have meant.**
//!
//! ### Why a guide drag outranks an armed region zoom
//!
//! Both are "something in flight", so the tie is broken by the rule itself:
//! **retire the most transient thing first.** A guide drag ends the moment
//! the pointer is released, and it is following the pointer *now*; an armed
//! region zoom persists across frames waiting for a drag that has not started.
//! An operator dragging a guide who presses Escape means the guide, and there
//! is no reading of that press under which they meant a zoom they armed
//! earlier and have not used.
//!
//! A guide drag also does **not** reach [`crate::canvas::gesture`] — see that
//! module's header for why: a drag that did would move a guide *and* the
//! selection. So claimant 1 cannot speak for it, which is exactly why it needs
//! a row of its own rather than folding into `escape_consumed`.
//!
//! Each claimant reports whether it took the key rather than the caller
//! guessing, because the caller cannot know: whether a drag exists is the
//! gesture machine's private state and whether a zoom is armed is the zoom
//! module's. A version that re-derived either here would be the version that
//! cancels a drag **and** ascends a rung — which is the defect the whole
//! arrangement exists to prevent, and which an operator experiences as losing
//! the sub-path they were editing every time they abandon a mis-aimed drag.

use egui::Key;

use crate::app::actions::Action;
use crate::app::modes::Capabilities;
use crate::canvas::selection::SelectionState;
use crate::canvas::zoom;

/// Escape and Delete, for the canvas selection.
///
/// # ★ `DEFECTS.md` D1, from the other end
///
/// D1 is *"I can't even click on an object and delete it by hitting the
/// delete key."* Its cause was `ctx.egui_wants_keyboard_input()` — which means
/// *any* widget has focus, not *a text field has focus* — combined with a
/// canvas that takes focus on click. `app::keyboard` already carries the fix
/// (`ctx.text_edit_focused()`) and the regression test for it. This is the
/// **verb** the fixed key now reaches: without it, D1 would be fixed and
/// Delete would still do nothing, because there was no selection to delete
/// and nothing to delete it with.
///
/// The same guard is applied here rather than inherited, because this reads
/// the key itself. It has to: `app::keyboard::collect` runs before any widget
/// is built, and although the selection now lives on `OpenDoc` and is
/// therefore *reachable* from there (module docs, seam 1), moving these two
/// bindings into the keymap is a change to `app::keyboard`'s key table and its
/// tests — a separate change with its own argument about chord precedence,
/// not a consequence of this one. What the move already bought is the ribbon's
/// Delete: `PdfcerApp::dispatch_token` can now read the selection, so
/// `format.delete` raises the same action this does, from the same rule
/// ([`SelectionState::deletable_objects_on`]).
///
/// Backspace is bound alongside Delete because a laptop keyboard without a
/// dedicated Delete key is the common case, and every editor accepts both.
///
/// # `escape_consumed`, and why the gesture gets first refusal on the key
///
/// A drag in flight may be abandoned with Escape ([`gesture::GestureOutcome::Cancelled`]),
/// and that is the *same press* the ladder would otherwise read. One press must
/// have one effect — decision 025's L1, which is why Escape ascends exactly one
/// rung rather than collapsing the ladder — so when the gesture machine has
/// already spent the key, this is told and leaves it alone. The flag travels as
/// an argument rather than being re-derived here because the machine is the only
/// thing that knows whether there *was* a drag under the press; an Escape with an
/// idle pointer arrives here untouched and ascends, as it always did.
///
/// # `text_selection` — rung 5's other occupant
///
/// Passed as `&mut Option<_>` rather than being read back off the document,
/// because `canvas::interact` has taken it out by value for the duration of the
/// frame (the same move it makes for the object selection, and for the same
/// borrow reason). Clearing it needs nothing but the field: the *making* of a
/// text selection needs the page's extraction, which is why Ctrl+A and Ctrl+C
/// live in `canvas::textsel::keys` and only Escape lives here. See this module's
/// header on why it shares a rung with the ladder instead of taking a sixth.
/// Everything the key rungs need that is not the keyboard.
///
/// ★★ A struct because the list reached eight when the form-field Delete landed
/// (`OPERATOR_REQUESTS.md` **O53**), and eight positional parameters is a call
/// nobody can read — three of them are `bool`-ish and transposing two would
/// compile. `gesture::Press`, `resizing::Frame` and `dragroute::Frame` all took
/// the same shape for the same reason, so this is the local convention.
pub(super) struct Keys<'a> {
    /// The egui context.
    pub ctx: &'a egui::Context,
    /// The page on screen.
    pub page_index: usize,
    /// What the mode permits.
    pub caps: Capabilities,
    /// The selected form field, when one is selected.
    ///
    /// ★★★ On the DOCUMENT, not on `SelectionState` — `canvas::selection::annot`
    /// excludes `/Widget` so the form surface owns those presses. That is why it
    /// arrives as its own parameter rather than being read off the selection,
    /// and why Delete could not reach one until it did: the ladder below never
    /// had the fact in front of it.
    pub selected_field: Option<&'a crate::app::state::SelectedField>,
    /// ★★★ **Whether deleting the selected annotation would be refused** —
    /// `crate::panels::properties::annotdelete::refuses`, computed by the
    /// caller.
    ///
    /// ★★★ **`refuses`, taking the live selection — never `refuses_selected`.**
    /// The caller is `canvas::interact`, and that function opens by moving the
    /// selection off the document (`std::mem::take(&mut doc.selection)`), so a
    /// query that reads `doc.selection` sees an empty one and answers `false`
    /// for every document there is. That is precisely what this field carried
    /// from 2026-08-28 to 2026-08-29: the gate below was written, reviewed and
    /// unit-tested, and the flag feeding it was a constant `false` in the
    /// running program. See `annotdelete::refuses_selected`'s header for what
    /// that looked like from a chair, and note that no unit test in this file
    /// could have caught it — every one of them sets this field by hand.
    ///
    /// # Why the ANSWER arrives here and not the document
    ///
    /// [`canvas_keys`] takes no `&OpenDoc` and must not start. It is a pure
    /// function over what it is handed, which is what lets its eleven unit tests
    /// exercise the whole Delete/Escape ladder **without opening a file** — and
    /// those tests are the only thing standing between this ladder and the
    /// rung-order regressions its header catalogues. A `&OpenDoc` parameter
    /// would make every one of them need a real document.
    ///
    /// So the caller asks and passes a `bool`, exactly as it already does for
    /// [`Self::selected_field`], and for the same reason stated one field up:
    /// the fact lives somewhere this function cannot see.
    ///
    /// # ★★ What it costs to get wrong, which is why the ladder consults it
    ///
    /// Before 2026-08-29 this rung asked `annot.target.locked` and nothing else
    /// — one of the **three** things that refuse a delete. `/Encrypt` and a
    /// certification signature were not asked, so on a certified drawing the key
    /// raised the action, `delete_annotation` refused into
    /// `actions::apply::vector_edit`'s `Err` arm — a trace line and nothing to
    /// the operator — **and `actions::annots::delete` then cleared the selection
    /// anyway**, unconditionally, because it clears after the funnel rather than
    /// on success.
    ///
    /// ⇒ The operator pressed Delete, the comment stayed, the selection vanished
    /// with the Properties panel's explanation inside it, and nothing was said.
    /// A silence that also destroys the sentence explaining it is the worst
    /// shape this refusal could have taken, and it is what makes this a rung
    /// rather than a nicety.
    ///
    /// ★ `bool` rather than the `Refusal` itself: this rung only decides whether
    /// to proceed. **The wording is not this file's** — it is already on screen
    /// in `panels::properties::annotdelete`, drawn from the moment the
    /// annotation was selected, which is the R83 half a keystroke cannot
    /// provide. Carrying the reason here would invite a second sentence in a
    /// second place, and two wordings of one fact is how they come to disagree.
    pub annot_delete_refused: bool,
    /// **Whether the document refuses to delete the selected FORM FIELD**, from
    /// `panels::properties::formfield::refuses_delete`.
    ///
    /// [`Self::annot_delete_refused`]'s twin for rung 0, and it is a separate
    /// field rather than a widening of that one because the two are answers to
    /// **different engine queries** — `EditSession::deletion_refusal` here,
    /// `annotation_deletion_refusal` there. `refuses_delete`'s own doc argues
    /// why borrowing one for the other is a silent failure waiting on a spec
    /// nuance; the short form is that an annotation gate additionally consults
    /// §12.5.3 Table 165's per-annotation `Locked` bit, which no form field
    /// has.
    ///
    /// # ★★★ What its absence cost, and it is the day-before defect one rung up
    ///
    /// Rung 0 was added on 2026-08-28 with **no gate at all** — `caps.edit_content
    /// && selected_field`, push `DeleteWidget`, `return` — and it returns six
    /// lines above the annotation branch that *does* ask one. So the R83 work
    /// that closed the annotation rung on 2026-08-29 walked straight past the
    /// form rung sitting above it.
    ///
    /// On an ordinary certified fillable form the key raised `DeleteWidget`,
    /// `delete_widget` refused into `actions::apply::vector_edit`'s `Err` arm —
    /// a trace line and nothing to the operator — **and
    /// `actions::forms::delete_widget` had already cleared
    /// `doc.selected_field` before the call**, so the Properties panel's
    /// sentence explaining the refusal went blank on the same frame. The box
    /// stayed, the selection vanished, nothing was said.
    ///
    /// ★ `bool` rather than the refusal, for [`Self::annot_delete_refused`]'s
    /// reason verbatim: this rung decides only whether to proceed, and the
    /// wording is already on screen in
    /// `panels::properties::formfield`'s delete row, drawn from the moment the
    /// field was selected — which is the R83 half a keystroke cannot provide,
    /// and which now survives the press.
    pub field_delete_refused: bool,
    /// Whether Escape was already spent by a drag this frame.
    pub escape_consumed: bool,
    /// ★★★ **The page's object model**, so Delete can reach the deeper rungs.
    ///
    /// # Why this field exists at all
    ///
    /// Until 2026-09-05 Delete acted at the Object rung only and needed nothing
    /// but the selection: an entry already holds a resolved `TargetId`, and the
    /// operand list is a filter over four integers. The Part and Node rungs are
    /// different in kind — a subpath and a text run wear the *same*
    /// `subpath: Some(n)` field on a [`crate::canvas::selection::Selection`] and
    /// reach **different engine verbs** (`delete_subpath` and
    /// `delete_text_run`) — so something has to say which kind of part it is,
    /// and only the decomposition knows.
    ///
    /// # ★★ It is an `Option`, and the `None` case is not a formality
    ///
    /// `canvas::interact` builds the provider only when the frame needs one
    /// (`needs_targets`), because `decompose_page` walks every content stream on
    /// the page with no cache anywhere in `pdfcer-core`. On a frame that did not
    /// need it this is `None`, and [`crate::canvas::deleting::subject`] declines
    /// the deeper rungs by name rather than guessing at a part kind — while the
    /// **Object rung still works**, because it never needed the model. A
    /// signature that demanded one would have made a page that will not
    /// decompose un-deletable at the rung where deletion needs no decomposition
    /// at all.
    ///
    /// ★ It does not violate this struct's *"no `&OpenDoc`"* rule: a provider is
    /// a decomposition, not the application state, and the eleven unit tests
    /// below pass `None` and still exercise every rung of the ladder.
    pub targets: Option<&'a crate::panels::objects::provider::ObjectModelProvider>,
    /// The revision currently on screen — [`crate::app::state::OpenDoc::edit_epoch`].
    ///
    /// Carried for one purpose: a refusal that owes the operator a sentence is
    /// stamped with it, so the sentence stands from now until the next real edit
    /// moves past it and is retired without anything having to remember to. See
    /// `crate::app::actions::disclosure::record_note`, whose contract is that
    /// the epoch passed for a **non-edit** is the current one and not a new one.
    ///
    /// A plain integer, so it costs the unit tests nothing.
    pub edit_epoch: u64,
    /// ★★★ **Whether this frame ASKED for the decomposition** — the tripwire
    /// half of [`Self::targets`], and it exists because those two facts are
    /// different and were for one commit indistinguishable.
    ///
    /// `targets: None` has two causes and only one of them is honest:
    ///
    /// | cause | `model_attempted` | what it means |
    /// |---|---|---|
    /// | the page would not decompose | `true` | a real limit; `Refusal::NoObjectModel` is the correct answer and the operator is owed a sentence |
    /// | **nobody asked for it** | `false` | the 2026-09-05 defect, four times over: a working verb reachable by nothing, and a key that silently does nothing |
    ///
    /// Carried so the second can be made **loud** rather than being reported
    /// in the first's words. See the `debug_assert` at the decline site below,
    /// and `canvas::modelneed` for why a frame might not have asked.
    ///
    /// A plain `bool`, so the unit tests below pass `false` and still exercise
    /// every rung of the ladder — they never reach the assert, because they
    /// never supply a selection at a deeper rung without a provider.
    pub model_attempted: bool,
    /// ★★★ **The page on screen**, for the arrow-key nudge and for nothing else.
    ///
    /// # Why a `&Page` does not break this struct's "no `&OpenDoc`" rule
    ///
    /// [`Self::targets`]' doc states the rule and the reason: this function is
    /// pure over what it is handed, which is what lets its unit tests drive the
    /// whole ladder **without opening a file**. A `&Page` is a page
    /// dictionary — the same thing [`crate::canvas::annotdrag::drag`] takes as
    /// a parameter, and for the identical stated reason: *"it is what lets every
    /// rule in this module be tested without a window or a file."* It is not
    /// the application state, and the tests below pass `None`.
    ///
    /// # ★★ Why it is needed at all, when a nudge is a fixed step
    ///
    /// Because *up* is a screen fact and `dy` is a page fact, and the two are
    /// related by the page's own device transform — the Y flip **and**
    /// `/Rotate`. [`crate::canvas::moving::page_delta`] is the one function in
    /// `canvas/` that crosses between them, it takes a `&Page`, and routing
    /// through it is what stops this feature writing a second derivation of the
    /// page transform. A nudge that hard-coded `dy = +1` would be right on an
    /// unrotated page and would move a mark sideways on a landscape drawing
    /// exported with `/Rotate 90` — silently, and correctly-looking in every
    /// test that asserted the sign.
    ///
    /// `None` for a frame with no page on screen, in which case the nudge
    /// declines with a sentence rather than fabricating a delta.
    pub page: Option<&'a pdfcer_core::page_tree::Page>,
}

pub(super) fn canvas_keys(
    keys: Keys<'_>,
    selection: &mut SelectionState,
    text_selection: &mut Option<crate::canvas::textsel::TextSelection>,
    actions: &mut Vec<Action>,
) {
    let Keys {
        ctx,
        page_index,
        caps,
        selected_field,
        annot_delete_refused,
        field_delete_refused,
        escape_consumed,
        targets,
        edit_epoch,
        model_attempted,
        page,
    } = keys;
    // ★ Claimant 0, and it is read BEFORE the D1 guard rather than after it.
    //
    // That order is the whole point: `egui`'s `TextEdit` has already
    // surrendered focus by the time this runs, so the guard below no longer
    // sees a focused field and would let the press reach the ladder. Reading
    // the flag here — and clearing it, which `escape_spent` does — is what
    // makes one press one effect. See the header's own section on this rung.
    //
    // With nothing focused it is `false` and costs one map lookup, exactly as
    // an un-armed `disarm_region_zoom` costs one.
    let form_abandoned = crate::canvas::forms::escape_spent(ctx);

    // ★ D1: `text_edit_focused()`, NEVER `egui_wants_keyboard_input()`.
    //
    // ★★ And deliberately NOT `textedit::composing` here, which is the wider
    // predicate this project otherwise insists on. A canvas draft must still
    // reach rung 4 of the Escape ladder below — that rung is what ABANDONS the
    // draft, and a draft in flight is exactly the state in which Escape has the
    // most to do. Widening this guard would make Escape stop working for the
    // one gesture that needs it most.
    //
    // Delete is the key that must yield to a draft, and it is guarded on its
    // own branch further down rather than at this door. See there.
    //
    // typing-guard-exempt: this asks whether a WIDGET holds the keyboard, so
    // that Escape can still reach the rung that abandons a canvas draft.
    if ctx.text_edit_focused() {
        // typing-guard-exempt: Escape must reach the draft-abandon rung below;
        // Delete yields to a draft on its own branch. See the note above.
        return;
    }
    let (escape, delete, tab) = ctx.input(|i| {
        (
            i.key_pressed(Key::Escape),
            i.key_pressed(Key::Delete) || i.key_pressed(Key::Backspace),
            i.key_pressed(Key::Tab),
        )
    });

    // ★ Tab advances the snap cycle, and ONLY while a measure tool is armed.
    //
    // Tab is egui's focus key, so taking it unconditionally would break
    // keyboard navigation of every panel and every dialog. The guard is the
    // armed tool rather than a mode or a capability: with no measure tool
    // armed, `cycle_snap` reports `false` and the key falls through untouched,
    // so this costs one map lookup on a canvas that is not measuring.
    //
    // It returns early rather than falling through to Escape and Delete
    // because a frame carrying Tab is not carrying either of those, and
    // continuing would only re-read two keys that cannot be pressed.
    if tab && crate::canvas::tool::active(ctx).measure_kind().is_some() {
        crate::canvas::measure::cycle_snap(ctx);
        return;
    }

    // ★★★ **The arrow keys nudge the selected markup** — the gesture every
    // drawing program has and this one did not.
    //
    // It is read here rather than in `app::keyboard` for the reason Delete and
    // Escape are: a nudge acts on the **canvas selection**, and the keymap's
    // dispatcher reaches commands rather than selections. It is read here rather
    // than in `canvas::interact` because this is the function that already
    // asked `text_edit_focused` — an arrow key is the key most easily stolen
    // from somebody who is typing, and asking that question twice in one frame
    // is how the two spellings of it come to disagree (`DEFECTS.md` D1).
    //
    // ★ It does NOT return early, and the omission is deliberate: a frame
    // carrying an arrow carries neither Escape nor Delete, so there is nothing
    // below to protect from it, and a `return` here would be a claim about key
    // exclusivity that this function would then rely on without checking.
    //
    // Everything about the step, the modifiers it refuses, the Y sign and the
    // four refusals is `canvas::moving::nudge`. This line is routing.
    crate::canvas::moving::nudge::keys(
        ctx,
        &crate::canvas::moving::nudge::Frame {
            page,
            caps,
            edit_epoch,
        },
        selection,
        actions,
    );

    // ★ Escape retires the most transient thing first, and exactly one thing.
    //
    // The precedence is: a focused form field (spent above) → a drag in flight
    // (spent at step 3, and reported here as `escape_consumed`) → a guide drag
    // → an armed markup tool → an armed region zoom → the selection ladder.
    // Every rung obeys the same rule,
    // decision 025's L1: **one press, one effect.** An operator who
    // arms a marquee zoom and changes their mind presses Escape once and is
    // back in the select tool — with the rung they were working in intact,
    // because this returns before the ladder is touched.
    //
    // `disarm_region_zoom` reports whether there was anything armed, so an
    // Escape on an un-armed canvas falls straight through and still ascends,
    // exactly as it did before this branch existed.
    let escape_available = escape && !escape_consumed && !form_abandoned;
    if form_abandoned {
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed in the UI
            "canvas-escape outcome=AbandonedFormDraft".to_owned()
        });
    }

    // Claimant 2: a guide being dragged right now. Ahead of the region zoom
    // because it is the more transient of the two — it is following the
    // pointer this frame, while an armed zoom is waiting for a drag that has
    // not started. Abandoning it leaves nothing behind: the drag holds a
    // *proposed* position and the committed set only changes on release, so
    // there is no half-applied state to undo.
    let guide_cancelled = escape_available && crate::canvas::guides::cancel_drag(ctx);
    if guide_cancelled {
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed in the UI
            "canvas-escape outcome=CancelledGuideDrag".to_owned()
        });
    }

    // Claimant 3a: a measure pick in progress. Above the tool rung below it,
    // so one Escape corrects a mis-aimed pick and a second puts the tool down
    // — see the header's own section on why this needs two rungs where markup
    // needs one.
    let measure_abandoned =
        escape_available && !guide_cancelled && crate::canvas::measure::abandon(ctx);

    // …and a **markup vertex run** in progress, on the same rung and for the
    // identical reason: PolyLine and Polygon are gestured by clicks, so there is
    // no drag for claimant 1 to cancel, and yet a polygon with three vertices
    // taken and the fourth not is unmistakably a gesture in flight. Without this,
    // one Escape would discard the run **and** put the pen down — two effects
    // from one press, which is decision 025's L1 broken.
    //
    // One claimant expressed as two calls rather than two claimants, exactly as
    // 3b below is: a measure tool and a markup tool cannot both be armed, so a
    // measure pick and a vertex run cannot both be in progress.
    let vertex_abandoned = escape_available
        && !guide_cancelled
        && !measure_abandoned
        && crate::canvas::markup::vertex::abandon(ctx);

    // …and a **text draft** in progress, third on the same rung and for the
    // third time the same reason. A caret with characters typed into it is a
    // gesture in flight that no drag represents, and it is the one where Escape
    // matters most: it is the only way to say *"throw this away"*, because
    // clicking elsewhere COMMITS (`textedit::click`'s own note on why). One
    // Escape must therefore discard the draft and leave the tool armed, or an
    // operator correcting a typo would also be putting the pen down.
    //
    // Three calls on one rung rather than three rungs, on 3a's existing
    // argument: one tool is armed at a time, so a measure pick, a vertex run and
    // a text draft cannot two of them be in progress.
    let draft_abandoned = escape_available
        && !guide_cancelled
        && !measure_abandoned
        && !vertex_abandoned
        && crate::canvas::textedit::abandon(ctx);
    let vertex_abandoned = vertex_abandoned || draft_abandoned;

    // ★★★ **…and a PENDING PLACEMENT, fourth on the same rung** —
    // `OPERATOR_REQUESTS.md` O66.
    //
    // A window has stepped aside and is waiting for the operator to point at
    // the page. Escape is the way back, and it is the ONLY way back: the window
    // is not on screen, so there is no Cancel button to press.
    //
    // ★★ **Decision 025 L1 is not broken by this, and a reader will reach for
    // the opposite conclusion.** L1 forbids one Escape having two effects.
    // Abandoning a placement is ONE effect; the window reappearing is not a
    // second act but the *undo of the hide*, because being hidden was never a
    // fact of its own — `dialogs::placing` derives it from the record this
    // clears. Nothing is reopened, because nothing was closed.
    //
    // Guarded into the same `!` chain as its three neighbours for their reason:
    // one tool is armed at a time, so a placement cannot be pending while a
    // measure pick, a vertex run or a text draft is in progress. The chain is
    // belt to that braces, and it is what stops one press both cancelling a
    // placement and putting a pen down.
    //
    // ★ ABOVE the disarm claimants below. `disarm_any` would put the placement
    // tool down and leave the pending record set — a hidden window with its
    // tool retired, which is the worst of the reachable states. Cancelling
    // first clears both, because `placing::cancel` does the disarm itself.
    let placement_cancelled = escape_available
        && !guide_cancelled
        && !measure_abandoned
        && !vertex_abandoned
        && crate::canvas::placing::cancel(ctx);
    let vertex_abandoned = vertex_abandoned || placement_cancelled;

    // Claimant 3b: an armed markup tool. Above the region zoom deliberately —
    // see the header's own section on why the transience rule does not settle
    // that pair and what does. Note this is `disarm`, not "cancel a markup
    // drag": a drag in flight was already spent at claimant 1, and by the time
    // control reaches here `escape_available` is false in that case, so one
    // press can never both abandon the band AND put the pen down.
    let markup_disarmed = escape_available
        && !guide_cancelled
        && !measure_abandoned
        && !vertex_abandoned
        && crate::canvas::tool::disarm_markup(ctx);
    if markup_disarmed {
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed in the UI
            "canvas-escape outcome=DisarmedMarkupTool".to_owned()
        });
    }

    // …and the measure tool, on the same rung: they cannot both be armed, so
    // this is one claimant expressed as two calls rather than two claimants.
    let measure_disarmed = escape_available
        && !guide_cancelled
        && !measure_abandoned
        && !vertex_abandoned
        && !markup_disarmed
        && crate::canvas::tool::disarm_measure(ctx);
    if measure_disarmed {
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed in the UI
            "canvas-escape outcome=DisarmedMeasureTool".to_owned()
        });
    }

    // ★★★ …and ANY OTHER ARMED TOOL, last on this rung — 2026-08-20, on the
    // operator: *"Escape should get me out of a tool."*
    //
    // The two calls above covered a pen and a measure tool. They left the
    // caret, the node tool, the text tool and the hand, so the answer to *"how
    // do I stop doing this?"* depended on which tool had been picked. The
    // convention has no exceptions: Escape returns you to the pointer.
    //
    // Below the two specific calls rather than replacing them, because they
    // trace distinct outcomes that driven checks read — and a general disarm
    // that swallowed those would make a harness unable to tell which tool went
    // down. It reports its own outcome for the same reason.
    let tool_disarmed = escape_available
        && !guide_cancelled
        && !measure_abandoned
        && !vertex_abandoned
        && !markup_disarmed
        && !measure_disarmed
        && crate::canvas::tool::disarm_any(ctx);
    if tool_disarmed {
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed in the UI
            "canvas-escape outcome=DisarmedTool".to_owned()
        });
    }

    // Claimant 4.
    let disarmed = escape_available
        && !guide_cancelled
        && !measure_abandoned
        && !vertex_abandoned
        && !markup_disarmed
        && !measure_disarmed
        // …and not if the general tool disarm above already spent this press.
        // Decision 025's L1: ONE PRESS, ONE EFFECT. Without this line an
        // operator with the node tool armed and a region zoom pending would
        // lose both to a single Escape and have no way to keep either.
        && !tool_disarmed
        && zoom::disarm_region_zoom(ctx);
    if disarmed {
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed in the UI
            "canvas-escape outcome=DisarmedRegionZoom".to_owned()
        });
    }

    // Claimant 5, and only if none of the rungs above took the key.
    if escape_available
        && !guide_cancelled
        && !measure_abandoned
        && !vertex_abandoned
        && !markup_disarmed
        && !measure_disarmed
        && !tool_disarmed
        && !disarmed
    {
        // ★ Rung 5's two occupants, and they cannot both be here — see the
        // header. The text branch is tested first because it is the one that
        // can be non-empty in a mode where the other is *structurally* empty:
        // in Read the ladder has nothing to ascend, so calling `escape()` first
        // would consume the press on a no-op and leave the wash on the page.
        if text_selection.take().is_some() {
            crate::canvas::trace::text_selection(page_index, None, "escape");
        } else {
            let outcome = selection.escape();
            crate::diag::trace(|| {
                format!(
                    // ui-text-exempt: diagnostic trace, never displayed in the UI
                    "canvas-escape outcome={outcome:?} sel={}",
                    selection.len()
                )
            });
            // ★★★ **Claimant 6 — LEAVE THE CONTAINER**, and only when the rung
            // above found nothing left to do. `OPERATOR_REQUESTS.md` O70.
            //
            // Below the selection rather than above it, which is the opposite
            // of a first guess and follows this table's own rule: **retire the
            // most transient thing first**. A selection inside a title block is
            // made and remade by every click; the fact that the operator is
            // WORKING inside that title block survives all of them, and is what
            // they would be most annoyed to lose to a stray press.
            //
            // ⇒ So one Escape clears what is selected, and a second steps back
            // out into the drawing. That is one rung per press — decision 025's
            // L1 — with the scope as the outermost rung of the same ladder,
            // rather than a sixth thing competing for the key.
            if matches!(outcome, crate::canvas::selection::EscapeOutcome::Nothing)
                && crate::canvas::smart::leave(ctx)
            {
                crate::diag::trace(|| {
                    // ui-text-exempt: diagnostic trace, never displayed in the UI
                    "canvas-escape outcome=LeftContainer".to_owned()
                });
            }
        }
    }

    if !delete {
        return;
    }

    // ★★★ **A CANVAS DRAFT TAKES DELETE AND BACKSPACE**, 2026-08-20.
    //
    // Above every rung below it, and it has to be: with a caret on the page the
    // operator is typing, and Delete means "eat the character in front of me".
    // Reaching the rungs below would delete the SELECTED OBJECT instead —
    // silently, destructively, while they were mid-word — which is defect D1's
    // family in its worst form. D1 lost a keystroke; this would lose a drawing.
    //
    // ★ Note where the guard is, and why it is not at this function's entry.
    // The entry test is deliberately `text_edit_focused()` alone, because
    // **Escape must still reach rung 4**, which is what abandons the draft. A
    // draft in flight is precisely the state in which Escape has the most to
    // do. So the two keys diverge here rather than at the door: Escape belongs
    // to the ladder, Delete belongs to whoever is composing.
    //
    // `canvas::textedit::typing` has already consumed the key this frame — it
    // runs from `canvas::interact` before this — so this is a guard against
    // double handling rather than the thing that makes Delete work in a draft.
    if crate::canvas::textedit::composing(ctx) {
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed in the UI
            "canvas-delete-declined reason=composing-text".to_owned()
        });
        return;
    }

    // ★★ An ANNOTATION takes the key, on its OWN capability, above the
    // content gate.
    //
    // # Why it is above, and what being below it cost
    //
    // The gate below is `!caps.edit_content`, and it used to be the only one.
    // With the annotation branch beneath it, **Review** — `edit_content:
    // false`, `author_markup: true` — returned before ever reaching it. Review
    // is the markup stance: it is the mode an operator is in *because* they
    // are working on stamps and dimensions, and it was the one mode where
    // Delete could not remove one.
    //
    // That is the second predicate in this feature that was answering a
    // question wider than the one it was written for, and both were invisible
    // in the same way: the mechanism below worked perfectly and never ran.
    // **One predicate per capability** — `author_markup` guards the annotation
    // verb, `edit_content` guards the content verb, and neither stands in for
    // the other.
    //
    // The `caps` check is belt and braces here for the reason the old comment
    // below gives about its own: entering a mode without `author_markup`
    // already clears the annotation selection (`app::gating`), so this is
    // unreachable in practice. It is written anyway, because *"Delete is safe
    // because nothing can be selected"* holds only for as long as its other
    // half does, and the other half is in a different file.
    //
    // Locked (§12.5.3 bit 8) declines and **says which silence it is**: a
    // reader of the trace is entitled to tell "the file forbids it" from
    // "nothing was selected", because only one of those is something the
    // operator can act on.
    // ★★★ A FORM FIELD, and it is checked FIRST of the three Delete claimants.
    //
    // `OPERATOR_REQUESTS.md` **O53**: *"if the engine is capable, I should be
    // able to select the object and do all of the ordinary editing one would
    // expect a GUI editor to be able to do."* Delete is the most ordinary of
    // them and it did not reach a selected field at all -- this ladder never
    // had `doc.selected_field` in front of it, because a widget is deliberately
    // not an annotation selection.
    //
    // ★★ First rather than last, and the order is not arbitrary: a form
    // selection and an annotation selection are mutually exclusive by
    // construction (the form surface owns `/Widget` presses and
    // `selection::annot` excludes them), so this is a statement of that rather
    // than a precedence. Putting it first means a reader meets the narrowest
    // claim before the general ones.
    //
    // ★ `edit_content` guards it, matching where the SELECTION is offered:
    // `canvas::forms` gives the selection surface to Edit and the fill surface
    // to Read and Review, because *"the same click cannot both type a value and
    // select the box to rename it."* One predicate per capability -- the same
    // rule the annotation branch below had to learn the hard way.
    if caps.edit_content
        && let Some(field) = selected_field
    {
        // ★★★ **The gate, asked through the one function that also withholds
        // the menu item and draws the sentence** —
        // `panels::properties::formfield::refuses_delete`, arriving as
        // [`Keys::field_delete_refused`] because this function takes no
        // `&OpenDoc` and must not start.
        //
        // This rung had NO gate between 2026-08-28 and 2026-08-29 — it pushed
        // the action on `caps.edit_content` alone and returned six lines above
        // the annotation branch that does ask one. The R83 pass that closed
        // the annotation rung read this one and did not see it, because a rung
        // that asks nothing looks like a rung with nothing to ask.
        //
        // ★★ Declines to the TRACE, not to `app::status::decline`, and that is
        // the same ruling the annotation rung below makes for the same reason:
        // a key cannot be undrawn, so R9's remedy for a permanently
        // unavailable capability has nowhere to render — but the sentence is
        // **already on screen**, in the Properties panel's delete row, from the
        // moment the field was selected. The operator was told before the
        // press rather than after it, which is what R83 asks for, and the
        // press no longer erases the telling.
        //
        // ⇒ Deliberately NOT `decline`: a decline reports *a gesture just
        // failed* and must be repeatable, whereas this is a **standing property
        // of the open document** — true from the moment it was opened, true
        // whether or not anything was pressed.
        if field_delete_refused {
            crate::diag::trace(|| {
                format!(
                    // ui-text-exempt: diagnostic trace, never displayed in the UI
                    "canvas-delete-declined field={} widget={} reason=field-delete-refused",
                    field.field, field.widget
                )
            });
            return;
        }
        actions.push(Action::Field(
            crate::app::actions::forms::FieldAction::DeleteWidget {
                field: field.field.clone(),
                widget: field.widget,
            },
        ));
        return;
    }

    if caps.author_markup
        && let Some(annot) = selection.annot()
    {
        // ★★★ **The gate, asked through the one function that also withholds the
        // control** — `panels::properties::annotdelete::gate`.
        //
        // This was `if annot.target.locked` and nothing else, which is one of
        // three refusals. §12.5.3 Table 165 bit 8 was caught; `/Encrypt` and a
        // certification signature were not, so on a certified drawing the key
        // raised the action, `delete_annotation` refused, and
        // `actions::apply::vector_edit`'s `Err` arm wrote a line to the trace
        // and — by that arm's own recorded decision — said nothing at all to the
        // operator. Discovery by pressing, on a press the program already knew
        // would fail: R83's whole subject.
        //
        // ★★ Why the key declines to the TRACE while the ribbon's Delete is
        // withheld outright, and why that is not two policies
        //
        // Because a key cannot be undrawn. R9's remedy for a permanently
        // unavailable capability is *render nothing, or say where the thing
        // lives*; a keystroke renders nothing by nature, so the only thing left
        // to decide is where the sentence goes — and it is already on screen,
        // in the Properties panel's `annotdelete` section, which draws it the
        // moment the annotation is selected and keeps it there. The operator has
        // been told **before** the press rather than after it, which is the
        // outcome R83 asks for and a strictly better one than a status line that
        // arrives with the failure and retires on the next command.
        //
        // ⇒ Deliberately NOT `app::status::decline`. That module's own header
        // draws the line this sits on: a decline reports *a gesture just failed*
        // and must be repeatable, whereas this is a **standing property of the
        // open document** — true from the moment it was opened, true whether or
        // not anything was pressed. A sentence that only appears after a press
        // delivers that fact at the one moment R83 exists to get ahead of.
        if annot_delete_refused {
            crate::diag::trace(|| {
                format!(
                    // ui-text-exempt: diagnostic trace, never displayed in the UI
                    "canvas-delete-declined id={:?} reason=annot-delete-refused",
                    annot.target.id
                )
            });
            return;
        }
        actions.push(Action::Annot(
            crate::app::actions::annot::AnnotAction::Delete {
                page: annot.target.page,
                id: annot.target.id,
            },
        ));
        return;
    }

    // ★ A mode that cannot edit CONTENT has no Delete for content.
    //
    // Escape is deliberately **above** this line and ungated: every one of its
    // claimants — a form draft, a guide drag, an armed region zoom — is
    // reachable in Read, and a mode that swallowed Escape would trap the
    // operator inside the gesture it had just let them start.
    //
    // In practice this is unreachable, because entering such a mode clears the
    // selection (`PdfcerApp`'s mode-change arm) and no gesture can build a new
    // one, so `deletable_objects_on` would return an empty list two lines
    // below and refuse anyway. It is written explicitly regardless: *"a test
    // that checks a relation rather than a magnitude is satisfied by any
    // absurdity in the right direction"* (`HANDOFF.md` §2), and "Delete is
    // safe because nothing can be selected" is exactly that shape of argument
    // — it holds only for as long as the other half does, and the other half
    // is in a different file.
    if !caps.edit_content {
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed in the UI
            "canvas-delete-declined reason=mode-cannot-edit-content".to_owned()
        });
        return;
    }

    // ★★★ **DELETE REACHES THE RUNG THE OPERATOR IS ON**, 2026-09-05.
    //
    // What stood here decided the whole question itself: `deletable_objects_on`
    // (Object rung only), a leaf fallback, and — for every deeper rung —
    //
    //     canvas-delete-declined level=Part reason=no-verb-for-rung
    //
    // and nothing else. No sentence, no sound, nothing on screen. The refusal
    // was honest when it was written and stopped being honest the moment the
    // engine shipped `delete_subpath`, `delete_text_run` and `delete_node`,
    // whose MOVE twins this shell had wired: on a CAD export a line could be
    // entered, selected and **dragged**, and could not be removed.
    //
    // ★★ The decision moved to [`crate::canvas::deleting::subject`] and did not
    // merely move — it is now asked by the ribbon's `format.delete` as well, so
    // Delete-the-key and Delete-the-command cannot act on different things.
    // That divergence is not hypothetical: it happened once already over form
    // fields (`app::dispatch::format`'s own arm records it), and it is what
    // `app::keyboard`'s header calls the defect the single dispatcher exists to
    // make impossible. A destructive rule stated in two places is a rule that
    // drifts, and the drift here removes a drawing view instead of a line.
    //
    // ★ `deletable_objects_on` is NOT deleted. It is still the answer to *"what
    // may a Delete act on at the Object rung"* and `app::conditions` and the
    // tests read it; what changed is that this ladder no longer treats its
    // empty answer as the end of the question.
    match crate::canvas::deleting::subject(selection, page_index, targets) {
        // The selection is NOT cleared on any of these arms. The delete is an
        // action, applied after this frame; the epoch it bumps makes
        // `SelectionState::resolve` drop exactly the entries whose objects no
        // longer exist, on the next frame. Clearing here as well would be a
        // second mechanism for the same outcome, and the two would disagree the
        // first time the engine refused the edit.
        Ok(subject) => actions.push(crate::canvas::deleting::action(subject).into()),
        // ★★ A silent decline IS the defect, and this is where it ends. Three
        // of the eleven refusals put a sentence on the status row — the three an
        // operator meets having done nothing wrong — and every one of the eleven
        // is named on the trace. `deleting::decline` owns which is which and
        // `text::deleting` owns the words; neither decision belongs in a key
        // handler.
        Err(reason) => {
            // ★★★ **THE TRIPWIRE FOR THE FIFTH RECURRENCE**, and it is here
            // rather than in `deleting::decline` because this is the only
            // place that knows both halves of the question.
            //
            // `Refusal::NoObjectModel` has two causes that produce the
            // identical refusal, and telling them apart is the whole point:
            //
            // * **the page would not decompose** — honest, a real limit, the
            //   operator is owed the sentence `text::deleting` now gives it;
            // * **this frame never asked for the decomposition** — a defect
            //   that has now shipped four times, presenting each time as a
            //   gesture or a key that silently does nothing while a working
            //   engine verb sat one call away.
            //
            // `canvas::modelneed` is what makes the second answerable, and
            // this assert is what makes it *loud*. If it fires, do not widen
            // anything here — go and add the missing term to that module,
            // where every other reason a frame needs the model already lives.
            //
            // ★ Release builds are not left silent either: `decline` carries
            // `model_attempted` onto the trace as `asked=`, so a driven check
            // reads the same distinction the assert makes.
            debug_assert!(
                reason != crate::canvas::deleting::Refusal::NoObjectModel || model_attempted,
                "Delete was declined `NoObjectModel` on a frame that never ASKED for the \
                 page's decomposition. That is not a limit of the document — it is \
                 `canvas::modelneed` failing to name a reason this frame needs the model, \
                 which is the same defect as `Resize` (2026-08-19), `Handle` (2026-08-19), \
                 `DimensionVertex` (2026-08-20) and the Delete key (2026-09-05). Add the \
                 term there, not a fallback here."
            );
            crate::canvas::deleting::decline(selection, reason, edit_epoch, model_attempted);
        }
    }
}

#[cfg(test)]
mod tests;
