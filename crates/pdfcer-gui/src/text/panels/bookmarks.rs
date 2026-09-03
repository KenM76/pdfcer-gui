//! # `text::panels::bookmarks` — the words for **moving** a bookmark and for
//! **expanding or collapsing** one
//!
//! The surface for `pdfcer-core` `Pass 161.0`'s two verbs,
//! `EditSession::move_outline_item` and `EditSession::set_outline_open`. See
//! [`crate::panels::bookmarks::reorder`] for the gesture; this module is only
//! the copy.
//!
//! ## Why a module of its own rather than more of [`super`]
//!
//! R2, arithmetic first: `super`'s `mod.rs` was 1,164 lines when this arrived
//! and carries three panels. Two hundred lines of disclosure would have put it
//! within a hundred of the 1,500-line ceiling, and the next bookmark verb over
//! the top of it. `fonts`, `objects` and `properties` are already split out of
//! that file on the same rule, and the subject boundary — *"the words for the
//! two verbs that change a bookmark's PLACE rather than its name"* — decides
//! where the cut falls.
//!
//! ## ★★★ The `/Count` sign runs through every sentence in this file
//!
//! §12.3.3 gives `/Count` two meanings and makes the **sign** carry
//! open-or-closed, because Table 153 defines no `/Open` key:
//!
//! | | root `/Outlines` (Table 152) | an item (Table 153) |
//! |---|---|---|
//! | counts | all visible items **including** the top level | visible **descendants**, excluding itself |
//! | sign | cannot be negative | **positive = open, negative = closed** |
//!
//! A **closed** item contributes exactly **1** to its ancestors, however large
//! its subtree. Three sentences below exist only because of that, and each one
//! says so in its own doc comment:
//!
//! 1. [`bookmark_moved`] takes `OutlineMove::visible_items`, which is the
//!    item plus its **visible** descendants — `1` for a collapsed chapter with
//!    forty sections in it. So the number alone would under-report a move on
//!    exactly the branch whose size the operator cannot see, and
//!    [`bookmark_move_took_hidden`] is the second sentence that covers it.
//! 2. [`bookmark_move_into_collapsed`] exists because a bookmark moved under a
//!    collapsed parent **disappears from the list**, and the panel is correct
//!    to show it that way. It is
//!    [`super::bookmark_add_under_collapsed`]'s sentence for the other verb,
//!    and it is deliberately worded to the same shape.
//! 3. [`bookmark_collapse_tooltip`] says that collapsing is **written into the
//!    document**, which is a surprise in a control that every other program
//!    treats as a view setting — and it is true here because `/Count`'s sign
//!    lives in the file.
//!
//! ## The two decline sentences do NOT name the bookmark, and that is a rule
//!
//! [`bookmark_move_declined_own_subtree`] and
//! [`bookmark_move_declined_engine`] are read by
//! `crate::app::status::decline`, whose `Declined` is **`Copy`** — a
//! deliberate property that a `String` payload would take away. So they carry
//! no title, exactly as
//! [`crate::text::forms::groups::field_group_preview_declined`] carries no
//! group name, and for the reason recorded there: the operator has just
//! released a row they were dragging, the row is still under their pointer,
//! and the sentence's job is to say **what happened**, not to re-identify what
//! they were looking at.

// ---------------------------------------------------------------------------
// Expanding and collapsing — `EditSession::set_outline_open`
// ---------------------------------------------------------------------------

/// The glyph on a row whose children are **hidden** — press to reveal them.
///
/// ★ A right-pointing triangle, which is the disclosure control every tree in
/// every operating system uses: Explorer's navigation pane, Finder's sidebar,
/// every IDE's project tree, Acrobat's own Bookmarks panel. The operator's
/// standing tie-breaker is *"make it work the way other programs do"*, and
/// there is no second answer to what a collapsed branch looks like.
///
/// It is in the catalog rather than inline because it is **drawn on screen**
/// and R1 admits no size threshold — a one-character label is still a label,
/// and the day somebody wants `+`/`−` instead there must be one place to
/// change it.
///
/// # ★★★ It is U+23F5, not U+25B6, and that was measured rather than chosen
///
/// The obvious pair is `▶` U+25B6 / `▼` U+25BC — the Geometric Shapes
/// triangles every style guide names. **The bundled font stack cannot draw
/// U+25BC.** [`crate::icons::glyphs`]' coverage gate reads the four `.ttf`
/// charmaps `epaint 0.35` ships, and it caught this the first time this module
/// compiled, with the sentence that gate exists to produce: *"each renders as a
/// substitution box in front of the operator."*
///
/// U+25B6 happens to draw and U+25BC does not, which is the worst possible
/// arrangement: a collapsed row would show a triangle and an expanded one a
/// hollow box, so the missing glyph would read as a *state* rather than as a
/// defect. Nothing in the source would say so, and nothing but the running
/// window would show it.
///
/// ⇒ Both halves therefore come from the **same** face — `emoji-icon-font`'s
/// `⏴⏵⏶⏷` U+23F4–U+23F7 block, which that module's coverage table records as
/// supplied. The rule generalises past this pair: **two glyphs that mean two
/// states of one control must come from one face**, or a substitution box and a
/// state are indistinguishable.
#[must_use]
pub const fn bookmark_collapsed_glyph() -> &'static str {
    "\u{23f5}"
}

/// The glyph on a row whose children are **showing** — press to hide them.
///
/// The same triangle turned down, from the same face. See
/// [`bookmark_collapsed_glyph`] for why the face matters more than the
/// codepoint.
#[must_use]
pub const fn bookmark_expanded_glyph() -> &'static str {
    "\u{23f7}"
}

/// Hover text on the triangle of a **collapsed** row.
///
/// ★★ It says the change is **saved into the document**, and that is the
/// non-obvious half. Every other outline panel an operator has used —
/// Explorer's tree, an IDE's file list, a spreadsheet's grouping — treats
/// expand and collapse as a view setting that belongs to the window. Here it
/// is a property of the file: §12.3.3 Table 153 carries open-or-closed as the
/// **sign** on `/Count` and defines no other key for it, so there is nowhere
/// to put a "just for me" answer. `EditSession::set_outline_open` writes it,
/// one undo entry, and the document is then modified.
///
/// Saying so costs one line of hover text and buys the operator the reason
/// their document went dirty from a gesture that looks like scrolling.
#[must_use]
pub const fn bookmark_expand_tooltip() -> &'static str {
    "Show the bookmarks filed under this one. Whether a bookmark is open or \
     closed is stored in the document, so this is a change you can undo."
}

/// Hover text on the triangle of an **expanded** row.
///
/// ★ It names the consequence the operator is about to create for themselves:
/// the rows go out of sight, and everything else in this panel that talks
/// about a collapsed bookmark — the add row's disclosure, the move's — is
/// about the state this button produces. See [`bookmark_expand_tooltip`] for
/// why both mention the document.
#[must_use]
pub const fn bookmark_collapse_tooltip() -> &'static str {
    "Hide the bookmarks filed under this one. They stay in the document; the \
     list stops showing them. Whether a bookmark is open or closed is stored \
     in the document, so this is a change you can undo."
}

// ---------------------------------------------------------------------------
// Moving one — `EditSession::move_outline_item`
// ---------------------------------------------------------------------------

/// The standing hint that says the rows can be dragged.
///
/// ★ Drawn once, above the list, beside the sentence that explains what
/// clicking a row does. R83 forbids offering a control that cannot work, and
/// its quieter twin is that a gesture nobody is told about is a capability the
/// program does not have. A drag has no widget to look at, which is exactly
/// why it is the one gesture in this panel that has to be **written down**.
///
/// It names all three landings, because the three-band split is the only part
/// of the gesture an operator cannot discover by trying it once — dropping on
/// the middle of a row and dropping on its edge look identical until you have
/// seen the caret move.
#[must_use]
pub const fn bookmark_drag_hint() -> &'static str {
    "Drag a bookmark to move it. Dropping on the top or bottom edge of another \
     bookmark puts it beside that one; dropping in the middle files it inside. \
     Whatever is filed under it comes with it."
}

/// ★★ **What the move did**, said after the press from the engine's own report.
///
/// # The number is `OutlineMove::visible_items`, and it is not a subtree size
///
/// `move_outline_item` returns the count *"the item plus its **visible**
/// descendants"*, which is the quantity `/Count` propagation actually moved
/// between the two branches. Its own doc comment is explicit that a shell must
/// not recompute it:
///
/// > *"A shell can say 'moved 1 bookmark (7 nested)' only if the core tells
/// > it; recomputing it shell-side would be a second implementation of the
/// > sign convention."*
///
/// ⇒ So this sentence quotes it and nothing else. What it deliberately does
/// **not** do is dress `1` up as *"and the 0 bookmarks under it"*: a collapsed
/// chapter reports `1` however large it is, and a sentence claiming nothing
/// travelled would be flatly false on exactly the branch the operator can
/// least see. That case has its own sentence — [`bookmark_move_took_hidden`] —
/// which the panel adds when it knows the item was collapsed.
///
/// # `reparented` chooses the verb, and it comes from the engine too
///
/// `OutlineMove::reparented` is carried *"separately from comparing the two
/// ids because it is the fact a disclosure sentence turns on — 'moved' versus
/// 'nested under' — and a shell deriving it independently is a second place
/// for the two to disagree."* This is that sentence, and it turns on exactly
/// that field.
#[must_use]
pub fn bookmark_moved(visible_items: usize, reparented: bool) -> String {
    match (visible_items, reparented) {
        (0 | 1, false) => "Bookmark moved.".to_owned(),
        (0 | 1, true) => "Bookmark moved, and it now sits under a different one.".to_owned(),
        (n, false) => format!(
            "Bookmark moved, and the {} shown under it moved with it.",
            n - 1
        ),
        (n, true) => format!(
            "Bookmark moved under a different one, and the {} shown under it went too.",
            n - 1
        ),
    }
}

/// ★★★ **The subtree that travelled and was never counted**, because it was
/// collapsed.
///
/// # Why the engine's number cannot say this
///
/// `OutlineMove::visible_items` counts *visible* descendants, per §12.3.3
/// Table 153's sign convention: a **closed** bookmark reports `1` however many
/// items are filed under it. So a chapter with forty sections, collapsed,
/// moves forty-one bookmarks and reports one.
///
/// The engine's own doc says the count is the shell's to *report*, never to
/// recompute — and this sentence does not recompute it. It reports a
/// **different quantity**: the size of the subtree, from the tree the panel
/// already drew, which is the same number
/// [`super::bookmark_delete_takes_subtree`] quotes before a delete and the same
/// walk (`crate::panels::bookmarks::tree::descendants`) produces it.
///
/// ⇒ Two numbers, two sources, two questions — *how many rows moved on screen*
/// and *how many bookmarks are in the branch* — and the operator is given the
/// second only when it differs from the first, which is exactly when the item
/// was collapsed.
///
/// ★ It is worded as a fact about the branch, not as a warning: the move
/// worked, everything went, and the only thing the operator could not see is
/// how much.
#[must_use]
pub fn bookmark_move_took_hidden(descendants: usize) -> String {
    if descendants == 1 {
        "It was collapsed, so the 1 bookmark hidden inside it moved as well.".to_owned()
    } else {
        format!("It was collapsed, so the {descendants} bookmarks hidden inside it moved as well.")
    }
}

/// ★★★ **The bookmark landed somewhere it cannot be seen**, and the panel is
/// right to show it that way.
///
/// # The trap this closes, in the operator's own terms
///
/// It is [`super::bookmark_add_under_collapsed`]'s trap for the other verb, and
/// that string's doc comment states the shape once for both:
///
/// > *"Getting the count right is the low bar. The operator's actual problem is
/// > that they will add a bookmark, look at the panel, and not see it — and the
/// > panel will be correct."*
///
/// A move into a collapsed parent is worse than an add into one, because the
/// operator watched the row leave. It vanished from where it was and did not
/// appear where they put it, and every reading available to them —
/// *"the drag missed"*, *"pdfcer deleted it"* — is wrong.
///
/// # Why it is said AFTER the press rather than before
///
/// The add row says its version *before*, and the difference is measured
/// rather than preferred. A sentence that appears while a drag is in flight
/// changes the height of the surface the operator is aiming at:
/// `crate::panels::pages`'s own header records the trace — a wrapping caption
/// above the grid moved the target tile 49 points, then 34, then back, because
/// its wording (and so its line count) changed as the pointer crossed gaps.
/// The rule it yields is more general than the caption that produced it:
/// **a surface may not change size in response to a gesture aimed at it.**
///
/// So the disclosure goes where every other after-the-fact disclosure goes —
/// the status bar, which has a fixed height by construction — and the remedy
/// is one click away, on the triangle the sentence names.
#[must_use]
pub const fn bookmark_move_into_collapsed() -> &'static str {
    "Its new parent is collapsed, so the bookmark will not show in the list \
     until you open that parent with its triangle. It is in the document."
}

/// **The move was asked for and changed nothing**, because the bookmark was
/// already there.
///
/// # ★ Why this is a disclosure and not a silence
///
/// `OutlineMove::moved` is `false` for a placement the bookmark already
/// occupies, and the engine is explicit that this is *"a legitimate request
/// with a legitimate answer — nothing"*: no objects are written and **no undo
/// entry is created**.
///
/// The panel dims its caret over a landing it can see is a no-op, so an
/// operator who reads the caret never reaches this sentence. Reaching it means
/// the shell's forecast and the engine's answer **disagreed** — the panel drew
/// a live caret and nothing happened — and that is a fact worth one line
/// rather than a shrug. It is the same posture
/// [`super::bookmark_deleted`] takes about its own two numbers being allowed to
/// differ: when the shell's read of the tree and the engine's read of the file
/// part company, say so.
#[must_use]
pub const fn bookmark_move_no_change() -> &'static str {
    "That bookmark was already in that place, so nothing changed and there is \
     nothing to undo."
}

/// **A bookmark cannot be filed inside itself** — the decline for a drop that
/// landed on the dragged row or somewhere in its own subtree.
///
/// # ★★★ Why this is a sentence and not a dimmed caret alone
///
/// The caret **is** dimmed over such a landing, before the press, which is the
/// disclosure this panel prefers. But the operator can release anyway, and
/// what they have then done is *ask*. R83's rule is not *gate the control*, it
/// is **a refusal must be a sentence, never a silence** — and the two things a
/// silence would be confused with are both wrong: *"the drag did not
/// register"* and *"pdfcer moved it somewhere I cannot see"*, the second of
/// which is a real state this very feature can produce (see
/// [`bookmark_move_into_collapsed`]).
///
/// # Why the shell refuses it rather than letting the engine
///
/// `EditError::OutlineMoveIntoOwnSubtree` exists and would refuse the call,
/// *"refused unconditionally; the Acrobat reference could not source what
/// Acrobat does here, and a cycle is a defect whatever Acrobat does."* The
/// shell answers first because it is a question about the tree it has already
/// drawn and can answer exactly — and because answering it here is what lets
/// the caret be dimmed **during** the drag, which is worth more than the
/// sentence.
///
/// ⇒ The engine's guard stays the authority; this is a forecast of it, in the
/// same relationship `panels::properties::formfield::refuses_delete` has with
/// `EditSession::deletion_refusal`. If the two ever disagree, the engine wins
/// and [`bookmark_move_declined_engine`] is what the operator reads.
///
/// ★ The bookmark is **not named**, deliberately. See the module header.
#[must_use]
pub const fn bookmark_move_declined_own_subtree() -> &'static str {
    "A bookmark cannot be filed inside itself, or inside anything filed under \
     it, so nothing moved. Drop it on a bookmark outside that branch."
}

/// **The engine refused the move** — the residue the shell's own forecast
/// cannot cover.
///
/// # ★★ What is actually left after the forecast
///
/// `crate::panels::bookmarks::reorder` refuses a drop into the dragged item's
/// own subtree before raising anything, so `OutlineMoveIntoOwnSubtree` should
/// not arrive here. What can:
///
/// | `EditError` | when |
/// |---|---|
/// | `DocumentEncrypted` | the file carries `/Encrypt` — every editing verb refuses |
/// | the certification gate | a signature forbids the change |
/// | `OutlineItemNotFound` | the id stopped resolving between the frame and the apply — the ordinary state after an undo |
/// | `NotADictionary` | the same, one step further gone |
/// | `OutlineRootIsNotAnItem` | unreachable from this surface: `read_outline` reports the root's *children* as its top-level items, so no id the panel can hold is the root's |
///
/// None of those is guessable from the canvas: a certified drawing looks
/// exactly like an uncertified one, and an encrypted one opens and renders
/// normally. That is the same argument
/// [`crate::text::status::field_delete_declined_structural`] makes for its own
/// verb, and it is why a catch-all sentence is honest here — the operator can
/// act on none of them, so splitting them would be detail without a remedy.
///
/// ★ The engine's own `Display` prose is **not** printed. It is written for a
/// log, it reaches the trace through `vector_edit`, and the operator gets a
/// sentence that says what happened and what state the document is in.
///
/// ★ The bookmark is **not named**. See the module header.
#[must_use]
pub const fn bookmark_move_declined_engine() -> &'static str {
    "That bookmark was not moved \u{2014} pdfcer declined the change, and the \
     outline is exactly as it was. The document may be encrypted or signed in \
     a way that forbids changing it."
}

#[cfg(test)]
mod tests {
    use super::*;

    /// ★★ **The move disclosure never says "0 bookmarks".**
    ///
    /// `OutlineMove::visible_items` counts the item itself, so `1` means *just
    /// this one* and the sentence must not subtract its way into a template.
    /// `0` is not a value the engine produces for a successful move — it counts
    /// the item — and it is folded into the same arm rather than being left to
    /// underflow, which is the one arithmetic error this function could make.
    #[test]
    fn the_move_disclosure_never_reads_as_a_template() {
        for reparented in [false, true] {
            for visible in [0usize, 1] {
                let said = bookmark_moved(visible, reparented);
                assert!(!said.contains('0'), "{said}");
                assert!(!said.contains(" 1 "), "{said}");
            }
        }
        // And with a real subtree it names the number, one lower than the
        // engine's count because that count includes the bookmark itself.
        assert!(bookmark_moved(8, false).contains('7'));
        assert!(bookmark_moved(8, true).contains('7'));
    }

    /// ★ **Re-parenting and reordering do not read the same.**
    ///
    /// `OutlineMove::reparented` is carried by the engine precisely so a shell
    /// does not derive it, *"because it is the fact a disclosure sentence turns
    /// on"*. A build that ignored the flag would produce one sentence for two
    /// different acts, and the operator could not tell a chapter that changed
    /// place from one that changed owner.
    #[test]
    fn a_reparent_and_a_reorder_are_worded_differently() {
        for n in [1usize, 2, 9] {
            assert_ne!(bookmark_moved(n, false), bookmark_moved(n, true));
        }
    }

    /// ★★★ **The hidden-subtree sentence is about the branch, not about the
    /// screen**, and it is the one place the two `/Count` quantities are
    /// visibly different.
    ///
    /// The fixture makes the two answers unconfusable: a collapsed bookmark
    /// reports `visible_items = 1` and may hold any number, so the sentence
    /// pair for a collapsed chapter of forty sections must name **40** and must
    /// not read as though one thing moved.
    #[test]
    fn the_hidden_subtree_sentence_names_the_branch_size() {
        let moved = bookmark_moved(1, true);
        let hidden = bookmark_move_took_hidden(40);
        assert!(hidden.contains("40"), "{hidden}");
        assert!(hidden.contains("collapsed"), "{hidden}");
        assert!(
            !moved.contains("40"),
            "the engine's count knows nothing about the hidden branch: {moved}"
        );
        // One is not spelled as a plural.
        assert!(bookmark_move_took_hidden(1).contains("1 bookmark hidden"));
    }

    /// ★ **The collapsed-destination sentence names the remedy and does not
    /// read as a failure.**
    ///
    /// Its whole purpose is that the operator watched a row leave and did not
    /// see it arrive. A sentence that only stated the fact would leave them
    /// where they started; the triangle is the way out and it is on screen
    /// beside them.
    ///
    /// Deliberately pinned against `bookmark_add_under_collapsed`'s two
    /// properties, because the two sentences are the same disclosure for two
    /// verbs and must not drift apart.
    #[test]
    fn the_collapsed_destination_sentence_matches_the_add_rows_posture() {
        let said = bookmark_move_into_collapsed();
        assert!(said.contains("collapsed"), "{said}");
        assert!(said.contains("triangle"), "the remedy is named: {said}");
        assert!(
            said.contains("in the document"),
            "the move WORKED, and the sentence must not read as a failure: {said}"
        );
        // The add row's sentence, on the same two counts.
        let add = super::super::bookmark_add_under_collapsed();
        assert!(add.contains("collapsed"), "{add}");
        assert!(add.contains("still be in the file"), "{add}");
    }

    /// ★★ **Neither decline names a bookmark**, which is what keeps
    /// `Declined` `Copy`.
    ///
    /// The field-group pair made this rule: a `String` payload on that enum
    /// would take away a deliberate property, and the loss is small because the
    /// operator's pointer is still on the row they dropped. Asserted as a
    /// property of the *signature* — both are `&'static str` and neither takes
    /// an argument — because that is the thing a future edit would break.
    #[test]
    fn the_declines_carry_no_title() {
        let own: &'static str = bookmark_move_declined_own_subtree();
        let engine: &'static str = bookmark_move_declined_engine();
        assert_ne!(own, engine, "two moments, two remedies, two sentences");
        // Each says that nothing happened, which is the whole speech act.
        assert!(own.contains("nothing moved"), "{own}");
        assert!(own.contains("Drop it"), "the remedy is named: {own}");
        assert!(engine.contains("exactly as it was"), "{engine}");
    }

    /// ★ **The two triangles differ**, and both are one character.
    ///
    /// A build that returned the same glyph for both states would give the
    /// operator a control that never appears to respond — the row would open
    /// and the triangle would not turn — and every unit test about the *tree*
    /// would still pass.
    #[test]
    fn the_disclosure_triangles_are_two_different_glyphs() {
        assert_ne!(bookmark_collapsed_glyph(), bookmark_expanded_glyph());
        assert_eq!(bookmark_collapsed_glyph().chars().count(), 1);
        assert_eq!(bookmark_expanded_glyph().chars().count(), 1);
    }

    /// ★★ **Both triangle tooltips say the state is stored in the document.**
    ///
    /// The one genuinely surprising fact about this control. Every other tree
    /// an operator has used treats expand and collapse as a window setting;
    /// here it is `/Count`'s sign in the file, so the gesture marks the
    /// document modified and lands on the undo stack. A tooltip that omitted it
    /// would leave them hunting for what dirtied their file.
    #[test]
    fn both_triangle_tooltips_disclose_that_the_state_is_saved() {
        for tip in [bookmark_expand_tooltip(), bookmark_collapse_tooltip()] {
            assert!(tip.contains("stored in the document"), "{tip}");
            assert!(tip.contains("undo"), "{tip}");
        }
        assert_ne!(bookmark_expand_tooltip(), bookmark_collapse_tooltip());
    }

    /// ★ **The drag hint names all three landings.**
    ///
    /// The three-band split is the only part of the gesture that cannot be
    /// discovered by trying it once: an edge drop and a middle drop look
    /// identical until the caret has been seen to move. A hint that said only
    /// *"drag to move"* would leave re-parenting undiscoverable, which is half
    /// the feature.
    #[test]
    fn the_drag_hint_teaches_the_three_landings() {
        let hint = bookmark_drag_hint();
        assert!(hint.contains("Drag"), "{hint}");
        assert!(hint.contains("edge"), "{hint}");
        assert!(hint.contains("middle"), "{hint}");
        assert!(
            hint.contains("comes with it"),
            "the subtree travels, and the hint is where that is said before any \
             press: {hint}"
        );
    }
}
