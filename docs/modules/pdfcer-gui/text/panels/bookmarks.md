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

## ★★★ The `/Count` sign runs through every sentence in this file

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
