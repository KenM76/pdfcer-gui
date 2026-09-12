//! # `app::status::decline::line` — one decline, one sentence
//!
//! Split out of [`super`] on 2026-09-10, when `OPERATOR_REQUESTS.md` O172's
//! custom-stamp decline took that file past R2's 1500-line ceiling. The seam is
//! the one the module already used for `record`, `textedit` and `clipboard`:
//! the parent keeps the **state**, a child keeps one **job** done with it.
//!
//! ## Why THIS was the half to move
//!
//! Because it is the only part of `impl Declined` that is a pure mapping.
//! [`super::Declined::still_true`] is a *ruling* — every arm of it is an
//! argument about whether a sentence has gone stale, and several of those
//! arguments are about the enum's shape rather than about any one variant, so
//! moving it would separate a ruling from the thing it rules on. This function
//! is a table: variant in, catalog call out, no state read, no decision made.
//!
//! ⇒ Which is also why the two must stay in different files rather than the
//! same one split by a comment. A reviewer asking *"why does this sentence not
//! go away?"* reads `still_true`; a reviewer asking *"what does the operator
//! see?"* reads this. Neither question has ever needed the other's answer.
//!
//! ## The rule every arm here obeys
//!
//! **No string literal.** Every word an operator reads comes from a
//! `crate::text::…` catalog function, under R1 and `check-ui-strings.sh`. Most
//! come from [`crate::text::status`]; the arms that reach further afield say in
//! their own comments why — a sentence lives with the surface that owns its
//! subject, so that a second wording of the same refusal cannot grow up beside
//! the first without somebody noticing.

use super::Declined;
use crate::text::status as t;

impl Declined {
    /// The sentence, from the catalog.
    ///
    /// The mapping is the whole of this module's contribution to the copy;
    /// every word an operator reads is [`crate::text::status`]'s, under rule
    /// R1.
    ///
    /// # ★★★ Why this returns a [`Cow`] rather than `&'static str`
    ///
    /// Because exactly one decline in this enum has a **subject the operator can
    /// see** — `OPERATOR_REQUESTS.md` O141's *"pdfcer cannot type a `q` into
    /// this text"* — and while the return type was `&'static str` it could not
    /// say which character. The sentence was written generically for that
    /// reason, and the reason was recorded in three separate doc comments as
    /// though it were a design choice; it was a return type.
    ///
    /// The cost is one arm. Every other sentence here is still borrowed static
    /// prose returned through `fixed` below, so the bar — which redraws every
    /// frame — allocates nothing except on the frames that are reporting that
    /// one refusal.
    #[must_use]
    pub(super) fn line(&self) -> std::borrow::Cow<'static, str> {
        // ★ Bound through a `&'static str` so only the arms that interpolate
        // carry machinery. They `return`; the catalog below is unchanged.
        let fixed: &'static str = match self {
            Self::NothingToFrame => t::zoom_declined_no_selection(),
            Self::CanvasNotDrawn => t::zoom_declined_not_drawn(),
            // ★ The variant carries WHICH fact refused; the catalog owns the
            // words for each. Before 2026-09-11 this arm named one flat string
            // that was wrong for both of its two call sites in different
            // directions — see `text::status::InsideFormRefusal`.
            Self::InsideForm(reason) => reason.line(),
            Self::SaveFailed => t::save_copy_failed(),
            Self::SettingsNotSaved => t::settings_not_saved(),
            Self::NothingToUndo => t::undo_declined_empty(),
            Self::NothingToRedo => t::redo_declined_empty(),
            // ★★ Reaches across to `crate::text::fieldclip` on the rule this
            // module's header states — *a string lives with the surface that
            // owns its subject* — and this sentence's subject is a field NAME,
            // which is the copy `fieldclip` already holds.
            //
            // ★★★ And it INTERPOLATES, making it the second arm in this table
            // ever to need the `Cow`. That is worth a note because the arm
            // directly below argues the opposite for itself and both are right:
            // `FieldNameTaken` does not carry its name because the name is the
            // one the operator just typed and is still in the box in front of
            // him; this one carries its name because the name is a DIFFERENT
            // field — he typed `Order.Total` and the field in the way is
            // `Order` — and nothing on screen tells him which prefix offended.
            Self::FieldPathCrossesTerminal(terminal) => {
                return std::borrow::Cow::Owned(crate::text::fieldclip::name_crosses_a_field(
                    terminal,
                ));
            }
            // ★★ The third interpolating arm, and the one whose payload is the
            // operator's OWN string rather than a fact the engine resolved.
            // Echoed back because the surface that can reach this refusal —
            // the Tab-order register panel's adopt rows — shows a name box per
            // unclaimed widget, and the bar has one sentence to spend.
            //
            // ★★★ Worded by `fieldclip`, for the field surfaces only, and
            // there is deliberately NO sign-surface wording beside it. The
            // engine raises the same variant from `sign`, and this shell
            // cannot reach it there: the signature window offers a list of
            // field names that already exist, and an existing FQN takes the
            // engine's reuse branch — where a period is CORRECT, because
            // `Approvals.Engineer` is a real nested placeholder to sign into.
            // A sentence for it would be a sentence no operator can provoke.
            // See `crate::app::actions::sign::worded`, which records that
            // absence where the missing arm would be.
            Self::DottedPartialName(supplied) => {
                return std::borrow::Cow::Owned(crate::text::fieldclip::name_is_a_path(supplied));
            }
            Self::FieldNameTaken => t::adopt_declined_name_taken(),
            Self::WidgetHasNoName => t::adopt_declined_no_name(),
            Self::ResizeNotRebuildable { uniform } => t::resize_not_rebuildable(*uniform),
            Self::ResizeFixedSizeMarker { by_flag } => t::resize_fixed_size_marker(*by_flag),
            Self::FlattenCertified => t::flatten_declined_certified(),
            Self::FieldDeleteRefused => t::field_delete_declined_structural(),
            // ★ Stays in `crate::text::status` rather than reaching across the
            // way the five arms below do, and the reach-across rule is what
            // decides it rather than what is bent for it: *a string lives with
            // the surface that owns its subject*, and this sentence's subject
            // is not a tool, a panel or a verb — it is any edit at all,
            // arriving from ~78 call sites through one funnel. The only surface
            // that owns it is this bar's `⊗` slot, whose catalog area is
            // `text::status`. It is in a FILE of its own there for the reason
            // `field_delete_declined_structural` above is: `text::status`'
            // `mod.rs` stands two dozen lines from R2's ceiling.
            Self::EditRefused => t::edit_declined_by_engine(),
            // ★ Reaches across to `crate::text::textedit` on the same rule the
            // arms below use: a string lives with the surface that owns its
            // subject, and every one of these eight sentences is about the text
            // caret and the paragraph under it. `ReflowRefusal::line` is the
            // one mapping, so the shell-side causes and the engine-side ones
            // cannot drift into two voices.
            Self::Reflow(why) => (*why).line(),
            // ★ Same catalog and same subject as the reflow family above: the
            // text caret and what the page under it will accept.
            Self::EnterCannotSplit => crate::text::textedit::enter_cannot_split_existing_text(),
            // ★ Reaches across to `crate::text::tool` rather than adding an
            // entry to `crate::text::status`, on the precedent the two field
            // -group sentences below already set: a string lives with the
            // surface that owns its subject, and `text::status` is at 1,482
            // lines against R2's 1,500.
            Self::NodeToolNeedsEditMode => crate::text::tool::node_tool_needs_edit_mode(),
            // ★ These two reach across to `text::forms::groups` rather than
            // adding entries here, and that is the catalog rule honoured rather
            // than bent: a string lives in `crate::text::…`, and the module
            // that owns this surface's other twenty sentences is the one that
            // owns these. `crate::text::status` is also two dozen lines from
            // R2's ceiling, which is a reason to notice the seam and not a
            // reason to choose it.
            Self::FieldGroupPreviewRefused => {
                crate::text::forms::groups::field_group_preview_declined()
            }
            Self::FieldGroupDeleteRefused => {
                crate::text::forms::groups::field_group_delete_declined()
            }
            // ★ Reaches across to `text::stamps` on the same catalog rule the
            // two above record: the module that owns this surface's other
            // sentences owns this one.
            Self::CustomStampUnavailable(why) => crate::text::stamps::place_declined(*why),
            // ★ These two reach across to `text::panels::bookmarks` on the
            // identical argument the field-group pair above records: a string
            // lives in `crate::text::…`, and the module that owns this
            // surface's other sentences owns these. `crate::text::status` is
            // two dozen lines from R2's ceiling, which is a reason to notice
            // the seam and not a reason to choose it.
            Self::BookmarkMoveIntoOwnSubtree => {
                crate::text::panels::bookmarks::bookmark_move_declined_own_subtree()
            }
            Self::BookmarkMoveRefused => {
                crate::text::panels::bookmarks::bookmark_move_declined_engine()
            }
            // ★ Returns early like `EditText` below, because this sentence can
            // name the faces that WOULD show the run — `Refusal::remedy_faces`,
            // consumed 2026-09-11. Without a list it is still borrowed static
            // prose and still allocates nothing.
            Self::TextStyle(why) => return why.line(),
            Self::Rotate(why) => (*why).line(),
            Self::Unshare(why) => (*why).line(),
            // ★ Reaches across to `crate::text::clipboard` on the same rule the
            // arms above use: a string lives with the surface that owns its
            // subject, and this one's subject is the clipboard — where the
            // other three clipboard refusals already live, so a fourth wording
            // of "that did not happen" cannot grow up beside them.
            Self::ClipboardMode(why) => (*why).line(),
            // ★★ Same catalog as `Reflow` and `EnterCannotSplit` above, and the
            // same rule: this enum owns which sentence, `crate::text::textedit`
            // owns the words. `EditRefusal::Unstated` forwards to
            // `t::edit_declined_by_engine` — the line `Self::EditRefused` shows
            // — so the un-categorised case is the *same string*, in one place,
            // and cannot drift into two voices for one condition.
            //
            // ★★★ **And it is the one arm that leaves through a `return`.**
            // `EditRefusal::line` answers a [`Cow`] since 2026-09-05 because
            // its `FontLacksTheCharacter` sentence names the character the
            // engine refused. Everything else in this match is fixed prose and
            // is unaffected.
            Self::EditText(why) => return why.line(),
            // ★ Reaches across to `crate::text::measure` on the same rule: a
            // string lives with the surface that owns its subject, and this
            // one's subject is what a ce dimension measures — where the
            // vertex-move disclosure it is the refusal twin of already lives.
            Self::VertexEditRefused(why) => (*why).line(),
            // ★ Reaches across to `crate::text::markup` on the same rule
            // every arm above uses: a string lives with the surface that owns
            // its subject, and this one's subject is a markup shape — where the
            // other twenty sentences about markup already live, so a second
            // wording of "that shape did not change" cannot grow up beside
            // them.
            Self::MarkupNodeRefused(why) => (*why).line(),
        };
        std::borrow::Cow::Borrowed(fixed)
    }
}
