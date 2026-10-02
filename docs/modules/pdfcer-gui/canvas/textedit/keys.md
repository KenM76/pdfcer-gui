# `canvas::textedit::keys` — what every key means inside a draft

## What this is

The frame's entry point for a draft's input. `typing` runs the pointer
gestures, then hands each keyboard event to `edits::Keys`, which owns what a
key means: one method per key family, each under the function limit.

| key | inside a draft |
|---|---|
| Ctrl+Z | takes back the draft's last run of typing or deleting; with nothing typed, commits the draft and undoes the document |
| Ctrl+Y, Ctrl+Shift+Z | puts it back; with nothing to redo in the draft, commits and redoes the document |
| Ctrl+Backspace, Ctrl+Delete | removes the word before or after the caret |
| Ctrl+A | selects the whole draft |
| Ctrl+S | commits the draft, then saves (Save copy when the document has no file) |
| Tab | types spaces to the next half-inch stop and says so in the status bar. `claim_tab` (run by the `app::keyclaim` plugin) turns the press into a typed tab before egui sees it; otherwise egui's focus walk would also move focus to the ribbon and the next Enter would press a button there |
| paste | runs through the sieve; on a line already on the page, line breaks become spaces and the status bar says so |
| Escape | commits the draft (O223) |

Typing runs coalesce: consecutive typed characters are one undo entry until
the caret moves, the pointer acts, or the edit changes kind. The history
(`editmodel::history`) and the status note (`textedit::note`) are dropped with
the draft.


## Why it is its own file

R2. `textedit/mod.rs` reached 1,571 lines the day the selection landed, and
the seam was already drawn: everything else in that module is about *what a
draft is* and *where it came from*, and this is about *what happens when a
key goes down*. The old shell's 25,005-line `main.rs` is the argument, and
the rule that prevents it is to split at the seam rather than to raise the
limit.

## The four selection rules, and where each is enforced

| # | rule | enforced by |
|---|---|---|
| 1 | a selection is the range between the mark and the caret | [`caret::range`] |
| 2 | typing replaces it | the `Text` arm, via [`take_selection`] |
| 3 | Backspace and Delete remove it and nothing else | their two arms |
| 4 | any movement without Shift drops it | [`caret::moved`], called by every movement arm |

Rule 4 is the one that looks like a detail and is not: without it a
highlight stays on screen after the caret has walked out of it, and the next
keystroke deletes text the operator is no longer looking at.

## What is NOT here, named rather than left to be discovered

**Drag-select and double-click-to-select-a-word.** The draft is drawn in an
editor box in *screen* space by [`super::paint`], and hit-testing a pointer
into it needs that laid-out galley published where the click ladder can
reach it. Real work, not a line — and until it exists, a selection is made
with the keyboard only.

## Item notes

### `fn pointer`

Returns `true` when the draft changed and must be written back.

# The three gestures, and why they are one function

| gesture | result |
|---|---|
| press | the caret goes where the pointer is, and any selection is dropped |
| drag | the mark stays at the PRESS and the caret follows the pointer |
| double click | the word under the pointer is selected |

They share a hit test and they share the rule that all three are only
meaningful **inside** the box, so splitting them would be three copies of
the containment check and three chances to drift on which coordinate space
is being asked about.

# The press origin, not the current position, anchors the drag

`PointerState::press_origin` is where the button went down, so the mark is
recomputed from it every frame rather than stored. That is deliberate: a
stored mark would have to be cleared on every way a drag can end — released,
Escaped, interrupted by focus loss, interrupted by the space bar — and
`canvas::markup::ink`'s header records what forgetting one of those four
costs. Derived state cannot go stale.

# Why this reads raw pointer input rather than a `Response`

Because the draft is not a widget. It is painted into the canvas and the
canvas's own `Response` covers the whole page; a second `interact` over the
editor box would put an invisible widget in the canvas's hit-test order and
change what the page under it receives. The box is a rectangle this module
drew and this module knows where it is — asking egui to tell it back would
be a second derivation of a fact it already has.

### `fn take_selection`

Answers `draft.caret` unchanged when nothing is selected, so a caller may
call it unconditionally — which the `Text` arm does, because *"replace the
selection if there is one"* and *"insert here"* is one act.

It clears the mark, and that is not tidying: a mark left pointing into
text that no longer exists is an index past the end of the string, and the
next Shift+Left would select a range that is not there.

### `fn copy_selection`

The return value is what makes [`Event::Cut`](egui::Event::Cut) safe: cut is
copy-then-delete, and it must not delete when the copy found nothing to take.
Returning a `bool` rather than having the caller re-ask `caret::range` means
the two halves of a cut cannot disagree about whether a selection existed.

Routed through [`crate::canvas::textsel::clipboard::copy`] — **the** one
place this shell writes text to the clipboard — rather than calling
`egui::Context::copy_text` directly. That function's header carries why:
three verbs reach it, and routing all of them through one function is what
makes its trace line a complete record of what pdfcer has copied rather than
one of several partial ones. It also refuses an empty string there, which is
the guard that stops a copy silently destroying whatever the operator had on
their clipboard from another application.

### `fn publish_layout`

A REAL GALLEY, from the same font stack the shell draws with, because
the whole claim of `hit` is that the layout which is hit-tested is the
layout which was drawn. A stub that mapped x to an index would test the
arithmetic of a stub.

### `fn ctrl_c_in_a_text_box_copies_the_selection_and_changes_nothing`

The event injected is `Event::Copy`, which is what `egui-winit` actually
sends, and injecting anything else is how this defect shipped: a test
feeding `Event::Key { key: C }` certifies a path the running application
can never take.

It asserts the draft is UNCHANGED as well. A copy that quietly moved
the caret or dropped the selection would pass a "did it copy" check and
still be wrong — the operator's next Shift+Left would select the wrong
range.

### `fn ctrl_x_with_no_selection_destroys_nothing`

Some editors cut the whole current line when nothing is selected. A text
box on a drawing is not a code editor, and a stray Ctrl+X silently
removing everything the operator had typed is not a behaviour worth
borrowing — they would have to notice it to undo it.

### `fn a_drag_across_the_editor_box_selects_what_it_crossed`

Driven through the same function the keyboard goes through, with a real
galley published the way `paint` publishes one, so the hit test is the
inverse of the caret painter rather than a second guess at it.

### `fn a_real_text_event_lands_in_the_draft`

Every existing text-edit check seeds the draft through `PDFCER_DIAG_TYPE`,
which is the ONE path that bypasses the event loop — so all of them pass
on a build where real typing is dead. This one drives a real
`egui::Context` with a real `Event::Text` and asserts the draft grew.

### `fn shift_right_selects_and_a_plain_right_drops_it`

A unit test rather than only a driven one, because the driven check
cannot tell "the shell ignored Shift" from "the harness never sent it",
and those live in different repositories. This one is unambiguous: the
event carries `shift: true` by construction.

### `fn enter_makes_a_new_line_in_both_authoring_drafts`

`OPERATOR_REQUESTS.md` **O127**, defect 2, and the whole of the answer to
*"can the enter key create new lines when we are editing or creating
text?"* Both authoring anchors take a break: the dragged box always did,
and the clicked point — which used to commit — now does too.

### `fn control_enter_commits_whatever_the_draft_is`

O127's brief in one assertion: *"commit must not be reachable only by
mouse."* Asserted across all three anchors rather than on the one that
changed, because the property being claimed is universality — an operator
must not have to know which gesture started the draft they are in to know
how to finish it.

### `fn enter_in_an_existing_run_declines_instead_of_committing`

It is the FILE's rule: `edit_text` re-encodes into the run's own font,
and a line-feed has no code in any standard encoding — so the engine
refuses it by name rather than dropping it. A show operator is one line
by construction.

### `fn the_modifier_is_the_only_thing_that_changes_a_run_draft`

A build that collapsed `CannotSplit` into `Commit` would compile, would
pass a test that only checked the two authoring anchors, and would put
the shell back exactly where the operator found it.

### `fn enter_means`

| draft | Enter | Ctrl+Enter |
|---|---|---|
| a dragged **box** | a line break | commit |
| a clicked **point** | a line break | commit |
| an existing **run** | a worded decline | commit |

# Why this is a function rather than three branches in the arm

Because *"decide the whole interaction, not half of it"* is a claim that has
to be checkable, and a `match` buried inside a 200-line event loop is not.
Every rule in the table above is proved by [`tests`] with no window, no
document, no `egui::Context` and no keyboard — which is the same discipline
`canvas::moving::eligible` and `SelectionState::click` are written to, and
the reason those two have rules a reader can trust.

It is also the trace's operand. `text-edit-enter means=NewLine` says which
branch was taken; the line it replaced said `boxed=1 command=0` and left the
reader to re-run the rule in their head against a build they were debugging.

# What changed, and the argument that was retired to change it


That argument is now retired rather than ignored. Enter means **one** thing:
a new line. The anchors therefore no longer differ on this key at all, and
what still separates them — the WIDTH, which a drag chooses and a click does
not — is settled at the commit in `app::actions::addtext`, where the page's
geometry is in scope.

⇒ The variant stays, for the reason that outlived the one it was introduced
with: `Anchor::Run` must be distinguishable *here*, and asking the TEXT
(*"does it already contain a newline?"*) would make the first Enter behave
differently from every one after it, which is the worst available answer.

# [`EnterMeans::CannotSplit`] is the FILE's rule, not this shell's

`EditSession::edit_text` replaces the string inside **one show operator**,
re-encoding it into that run's own font. `\n` has no code in any standard
encoding, so the engine refuses it by name — `Refusal { trigger:
TargetAbsent, character: Some('\n') }` — rather than dropping it. A PDF has
no paragraph: each visible line is its own operator at its own absolute
position, so splitting a line in two is not an edit to that line, it is
authoring a second one somewhere.

Committing quietly, which is what this did before, hid that from an operator
who had just asked the question with his fingers.

### `fn typing`

Returns `true` when the draft was committed — by Enter, Ctrl+S, or an undo or
redo that reached past the draft into the document — so the caller knows the
caret is gone.

# Why the events are read raw rather than through a `TextEdit` widget

Because the caret is painted in PDF space, on the page, at the glyphs' own
scale — which is what *"just edit the existing box"* means. An `egui`
`TextEdit` would be a second box floating over the first, and the old shell's
one virtue here is worth keeping: it had a real caret in the page, and no
widget in the typing path.
