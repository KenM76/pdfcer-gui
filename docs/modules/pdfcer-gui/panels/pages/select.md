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
