# `ui-verify/checks/text_tool`

`text_tool_selects_and_marks_in_edit` — the regression test for **a tool
whose entire visible effect is the mouse pointer**, and for the `RIBBON_IA.md`
P3 tension it was built to close.

# What this is about

Until 2026-08-14, `canvas::textsel::takes_the_press` gave a press its text
meaning only *"when the select tool is active and the mode cannot select
content"* — Read ✓, Review ✓, **Edit ✗**. Two things followed, and the second
is the one that made a fix urgent:

1. a reviewer could sweep text and an editor could not, which is an
   inversion;
2. the three text-markup controls (`markup.underline`, `markup.strikeout`,
   `markup.squiggly`) are **drawn** on the Markup tab in Edit and could
   **never enable** there, because `selection.text` was never true. P3
   reserves greying for *temporarily* unavailable and says an absent
   capability should render nothing; a control that is greyed for the whole
   life of a build is neither, and it could not be fixed by hiding, because a
   command lives on exactly one tab and the Markup tab is in both Review and
   Edit.

Both close with `CanvasTool::Text`, armed by **`view.tool_text`** in
View ▸ Navigate. This check is the evidence that they actually did, **in one
mode, in one run, with the same control observed dead and then live.**

# Why the trace is the only possible oracle here

Two independent reasons, and either alone would be enough:

* **An armed text tool changes the cursor and nothing else on the canvas.** A
  captured window does not carry the pointer, so a screenshot of an armed
  canvas and an un-armed one are not merely similar — they are *the same
  bytes*. `canvas::tool::toggle_text` emits `text-tool tool=…` for exactly
  this reason, as `markup-tool` and `measure-tool` already do.
* **A text selection's whole feedback is a translucent wash** at
  `overlay::TEXT_SELECTION_ALPHA`, deliberately low so the operator can read
  through it. [`crate::checks::text_selection`]'s header carries that
  argument in full: on a drawing sheet, selected and unselected are the same
  picture to any threshold.

The one thing a pixel oracle *could* have answered — is the control enabled?
— is answered better by the invoke, and for the reason
[`crate::checks::text_markup`] gives: a greyed `egui` control never reports
itself invoked, so the absence of `ribbon-command-invoked` is positive
evidence of disablement from outside the process, where the visible
difference is a few dozen antialiased pixels of *text* colour inside a fill
that does not change.

# The five phases, and why phase A is a negative and phase E is a falsifier

| Phase | State | Action | Expected | If it does not hold |
|---|---|---|---|---|
| A | Edit, nothing armed, nothing selected | click Underline | **no** `ribbon-command-invoked` | FAIL — the control is live with no operand, which is the P3 violation in the *other* direction |
| B | Edit | click `view.tool_text` | `text-tool tool=Text` | FAIL — the ribbon control does not arm the canvas tool |
| C | Edit, tool armed | sweep a band | `canvas-text-selection` with `chars` > 0 **and** `quads` > 0 | SKIP if no band has text; FAIL if a band selects characters and draws no boxes |
| D | Edit, selection live | click Underline | invoke **and** `text-markup-commit` **and** `add-text-markup` | FAIL, with the missing line naming the link |
| E | Edit, tool retired | sweep the **same** band | `text-tool tool=Select`, and **no** new `canvas-text-selection` | FAIL — the sweep works in Edit without the tool, so the mode gate has been deleted rather than the tool added |

**Phase A is what makes phase D mean anything.** Without it, a build whose
three text-markup controls were simply always live would pass D perfectly —
and the whole point of the fix is that the controls become *reachable*, not
that they become unconditional.

**Phase E is what makes phase C mean anything**, and it is the rule this
crate states as *"never treat an absence as evidence unless you have shown the
thing that would have produced it was working"* run backwards: here the
*presence* in phase C is the claim, and E establishes that the presence is
caused by the tool rather than by the mode gate having been removed. A build
that had deleted `takes_the_press`'s second half — the `!caps.edit_content`
clause — would sweep text in Edit with nothing armed, would pass A, B, C and
D, and would have silently replaced the marquee, which is the only
content-selection gesture the product has. E is the only phase that catches
it.

`text_selection`'s own phase C asserts the same property from the other side
(Read sweeps, Edit does not) and is deliberately **not** removed by this
check's arrival: that one runs with no tool ever armed in the process, so it
is evidence about the shipped default, while this one is evidence about the
default surviving beside the new tool.

# Mouse only

Every gesture here is a real `SetCursorPos` + `mouse_event`. Nothing in this
check needs a key, which is a consequence of the interaction model rather than a
coincidence: the tool is armed from the ribbon and the mark is a ribbon press,
so the whole feature is clicks and one drag.

What is therefore **not** covered here, and is on the record rather than
implied by a green result: Escape's behaviour with the tool armed (rung 5
clears the text selection and the tool is deliberately not an Escape
claimant — `canvas::keys`' header), and Ctrl+A / Ctrl+C in Edit with the tool
armed. All three are covered by unit test alone.

# Every way this reports SKIP, and why none of them is a pass

* no binary, no `--pdf`, `--no-input` — the harness never began;
* the diagnostic switches did not reach the process;
* the page size could not be read and no `--page-size` was given;
* the Edit segment, the View or Markup tab, or one of the two controls was
  never declared;
* the canvas is not showing page 1, so the harness's one known page size does
  not describe the page it would be sweeping;
* **no band had text under it** — phase C never succeeded, so phase A's
  silence proves nothing and phase D has no operand.

## Item notes

### `const MODE`

Read and Review already sweep text with the select tool, so arming the tool
there is a no-op an operator cannot see; the two gaps it closes are both in
Edit, whose primary button is the content marquee. A check aimed at Review
would pass against a build where `view.tool_text` did nothing at all.

### `const TOOL_SIBLING`

The two pointer-tool toggles are one group and one idea, and a build that
registered the new one while losing the old would otherwise pass this check
completely — the same half-done-registration guard
[`crate::checks::text_markup`] applies to Strikeout and Squiggly.

### `const MARK_ITEM`

Underline rather than Strikeout or Squiggly for
[`crate::checks::text_markup`]'s reason: the three are one dispatch arm with
one `match` between them, `shell::commands::mapping` walks all three, and the
*join* under test here is per-command only in its id.

### `fn selections`

Filtered on `chars > 0` for the reason both sibling checks record: a *clear*
is traced too, with `chars=0`, so counting the event would be satisfied by
the gesture that ends a selection.

### `fn invokes`

A **count**, never a presence: this check clicks the same control twice and
two different controls in one run, so "has it ever been invoked?" would be
answered `true` by a click made ten seconds earlier.

### `fn click_tab`

Three tab clicks in one run is what this check costs — View to arm, Markup to
mark, and Markup again after the sweep — so the move is written once rather
than three times. It is not in [`driving`] because the two existing tab
clicks in the suite are inline and folding them in would rewrite checks that
are already known to detect their defects, which is the argument that
module's own header makes about `markup_rectangle`.

### `fn the_selectors_match_the_shells_own_spelling`

Pinned for the reason both sibling checks pin theirs: the two crates are
joined by a **string** and nothing else, so a rename would leave both
sides compiling while every assertion here quietly stopped matching — and
a check that matches nothing passes vacuously.

### `fn arming_and_retiring_are_told_apart_by_the_tool_field`

`canvas::tool::toggle_text` traces the tool it moved *to*, so arming and
retiring differ only in that one word. A check that grepped for the event
name alone would be satisfied by either, and phase E — the falsifier —
would then pass on the frame the tool was armed.
