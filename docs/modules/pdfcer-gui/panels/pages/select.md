# `panels::pages::select` — which pages the operator has picked

A page selection, and the three-modifier click rule that builds it. Pure:
no `egui`, no document, no rendering. That is deliberate and it is what
makes the rule testable — the interesting part of a multi-select is the
*policy* (what does Shift extend from? what does a plain click discard?),
and the policy is the part that can be wrong in a way an operator would
notice.

## This is a SECOND selection in the application, and it is not the
canvas's

`crate::panels::ObjectTreeUi::focus`'s own docs refuse to become a second
selection, and `the_panel_focus_has_not_quietly_become_a_selection`
defends that refusal — so a new selection type arriving two modules over
needs its reason stated rather than assumed.

The reason is that this one selects a **different kind of thing**. The
canvas's [`crate::canvas::selection::SelectionState`] selects *objects on
a page*; this selects *pages in a document*. They cannot be confused,
they cannot drift into each other, and no command reads both:

| | canvas selection | page selection |
|---|---|---|
| operand | paint-order indices on one page | 0-based page indices |
| condition | `selection.any` | — (the `pages.*` commands are gated on `doc.pages`) |
| verbs | `format.delete`, the move verbs | `pages.delete`, `pages.extract`, `pages.move_*`, `pages.rotate_*` |
| survives a page change | yes | yes — it *is* about pages |
| survives an edit | resolved against the new decomposition | **cleared**, see [`PageSelection::retain_below`] |

The hazard the Objects panel's focus was guarding against — a *report*
surface arming a destructive command that acts on something else — does
not arise, because the destructive commands that read this selection are
the ones whose tooltips already say *"the selected pages"*. There is no
other candidate for what that phrase means.

## The three-modifier rule

Standard, and standard on purpose: this is the idiom every file list, every
layer palette and every other PDF reader's page rail uses, so an operator
arrives already knowing it. Deviating would be a novelty tax paid on every
click.

| Gesture | Selection | Navigates |
|---|---|---|
| click | exactly this page | **yes** |
| Ctrl+click | toggles this page's membership | no |
| Shift+click | every page between the anchor and this one | no |

**Only a plain click navigates**, and that is the load-bearing half. A
Ctrl+click that also moved the canvas would make building a set of five
pages a five-page journey through the document, re-rendering the canvas
each time — on the benchmark drawing, five renders of ~0.8 s to perform a
gesture that changes nothing about what the operator is looking at.

## The anchor, and why Shift+click needs one

A range needs two ends. The anchor is the last page the operator named
*deliberately* — by a plain click or a Ctrl+click — and it is **not**
moved by a Shift+click, so a second Shift+click adjusts the same range
rather than growing it from wherever the first one landed. That is the
behaviour of every list that gets this right, and the difference is only
visible when someone overshoots and corrects, which is exactly when it
matters.

## Item notes

### `fn a_plain_click_replaces_the_selection_and_navigates`

The navigation half is asserted because it is the only gesture that
has it, and a regression that made every click navigate would cost a
canvas re-render per click — ~0.8 s each on the benchmark drawing.

### `fn correcting_an_overshoot_shrinks_the_range`

The property the anchor rule exists for, and the only one that is
invisible until somebody overshoots. If the anchor moved with each
Shift+click, correcting an overshoot from 5→20 back to 5→8 would leave
8–20 selected with no gesture that removes them.

### `fn a_right_click_over_an_unpicked_page_picks_it`

Without this, right-clicking page 9 while 1–3 are picked and choosing
Delete destroys 1–3 — the pointer and the operand list disagreeing,
with an irreversible verb between them.

### `fn shrinking_the_document_drops_the_picks_that_fell_off_the_end`

The document shrinking must drop the picks that no longer name a
page. Keeping them would leave a selection pointing at a *different*
sheet, and the next `pages.delete` would remove one nobody chose.

### `fn a_reorder_carries_the_picked_pages_with_it`

The property that makes the reorder arrows usable more than once: move
four sheets up, and they are still the four sheets that are picked, so
the next press moves the same four. A build that cleared here — or, far
worse, one that left the indices alone — would leave the second press
acting on whichever sheets happen to sit at those positions now, which
is a *destructive* verb pointed at pages nobody chose the moment the
operator reaches for Delete instead.
