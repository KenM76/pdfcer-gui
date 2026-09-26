# `text::panels::bookmarks` — the words for **moving** a bookmark and for
**expanding or collapsing** one

The surface for `pdfcer-core` `Pass 161.0`'s two verbs,
`EditSession::move_outline_item` and `EditSession::set_outline_open`. See
[`crate::panels::bookmarks::reorder`] for the gesture; this module is only
the copy.

## Why a module of its own rather than more of [`super`]

R2, arithmetic first: `super`'s `mod.rs` was 1,164 lines when this arrived
and carries three panels. Two hundred lines of disclosure would have put it
within a hundred of the 1,500-line ceiling, and the next bookmark verb over
the top of it. `fonts`, `objects` and `properties` are already split out of
that file on the same rule, and the subject boundary — *"the words for the
two verbs that change a bookmark's PLACE rather than its name"* — decides
where the cut falls.

## The `/Count` sign runs through every sentence in this file

§12.3.3 gives `/Count` two meanings and makes the **sign** carry
open-or-closed, because Table 153 defines no `/Open` key:

| | root `/Outlines` (Table 152) | an item (Table 153) |
|---|---|---|
| counts | all visible items **including** the top level | visible **descendants**, excluding itself |
| sign | cannot be negative | **positive = open, negative = closed** |

A **closed** item contributes exactly **1** to its ancestors, however large
its subtree. Three sentences below exist only because of that, and each one
says so in its own doc comment:

1. [`bookmark_moved`] takes `OutlineMove::visible_items`, which is the
   item plus its **visible** descendants — `1` for a collapsed chapter with
   forty sections in it. So the number alone would under-report a move on
   exactly the branch whose size the operator cannot see, and
   [`bookmark_move_took_hidden`] is the second sentence that covers it.
2. [`bookmark_move_into_collapsed`] exists because a bookmark moved under a
   collapsed parent **disappears from the list**, and the panel is correct
   to show it that way. It is
   [`super::bookmark_add_under_collapsed`]'s sentence for the other verb,
   and it is deliberately worded to the same shape.
3. [`bookmark_collapse_tooltip`] says that collapsing is **written into the
   document**, which is a surprise in a control that every other program
   treats as a view setting — and it is true here because `/Count`'s sign
   lives in the file.

## The two decline sentences do NOT name the bookmark, and that is a rule

[`bookmark_move_declined_own_subtree`] and
[`bookmark_move_declined_engine`] are read by
`crate::app::status::decline`, whose `Declined` is **`Copy`** — a
deliberate property that a `String` payload would take away. So they carry
no title, exactly as
[`crate::text::forms::groups::field_group_preview_declined`] carries no
group name, and for the reason recorded there: the operator has just
released a row they were dragging, the row is still under their pointer,
and the sentence's job is to say **what happened**, not to re-identify what
they were looking at.

## Item notes

### `fn the_move_disclosure_never_reads_as_a_template`

`OutlineMove::visible_items` counts the item itself, so `1` means *just
this one* and the sentence must not subtract its way into a template.
`0` is not a value the engine produces for a successful move — it counts
the item — and it is folded into the same arm rather than being left to
underflow, which is the one arithmetic error this function could make.

### `fn a_reparent_and_a_reorder_are_worded_differently`

`OutlineMove::reparented` is carried by the engine precisely so a shell
does not derive it, *"because it is the fact a disclosure sentence turns
on"*. A build that ignored the flag would produce one sentence for two
different acts, and the operator could not tell a chapter that changed
place from one that changed owner.

### `fn the_hidden_subtree_sentence_names_the_branch_size`

The fixture makes the two answers unconfusable: a collapsed bookmark
reports `visible_items = 1` and may hold any number, so the sentence
pair for a collapsed chapter of forty sections must name **40** and must
not read as though one thing moved.

### `fn the_collapsed_destination_sentence_matches_the_add_rows_posture`

Its whole purpose is that the operator watched a row leave and did not
see it arrive. A sentence that only stated the fact would leave them
where they started; the triangle is the way out and it is on screen
beside them.

Deliberately pinned against `bookmark_add_under_collapsed`'s two
properties, because the two sentences are the same disclosure for two
verbs and must not drift apart.

### `fn the_declines_carry_no_title`

The field-group pair made this rule: a `String` payload on that enum
would take away a deliberate property, and the loss is small because the
operator's pointer is still on the row they dropped. Asserted as a
property of the *signature* — both are `&'static str` and neither takes
an argument — because that is the thing a future edit would break.

### `fn the_disclosure_triangles_are_two_different_glyphs`

A build that returned the same glyph for both states would give the
operator a control that never appears to respond — the row would open
and the triangle would not turn — and every unit test about the *tree*
would still pass.

### `fn both_triangle_tooltips_disclose_that_the_state_is_saved`

The one genuinely surprising fact about this control. Every other tree
an operator has used treats expand and collapse as a window setting;
here it is `/Count`'s sign in the file, so the gesture marks the
document modified and lands on the undo stack. A tooltip that omitted it
would leave them hunting for what dirtied their file.

### `fn the_drag_hint_teaches_the_three_landings`

The three-band split is the only part of the gesture that cannot be
discovered by trying it once: an edge drop and a middle drop look
identical until the caret has been seen to move. A hint that said only
*"drag to move"* would leave re-parenting undiscoverable, which is half
the feature.
