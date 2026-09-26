# `canvas::textedit` — **editing the page's own words**, and placing new ones

`DEFECTS.md` **D4** is the defect that began this project, reported as:

> *"text editing is weird and doesn't just edit the existing box and move the
> text correctly as you type plus flow to the next line doesn't work."*

This module is the shell half of the answer. It arms a tool, puts a caret in
a run, collects keystrokes into a draft, and commits the draft as **one**
`EditSession` command. What it fixes that the old shell did not is in
[`disposition`] — the two cases D4b names as *wrong on commit* — and what it
deliberately does not fix is listed under **What is out of scope** below,
in words, because a silently-disabled typing loop is how the old shell told
an operator it could not do something.

---

## §1. The admission argument `canvas::tool`'s header asked for

That header names one exclusion and one only:

> **Text *editing*** — Phase 5, the defect that began this project — remains
> outside, and for exactly the original reason: it is a caret in a
> re-laid-out box, it would drag a whole subsystem's state through this type
> […] **Whoever brings the second should have to make this argument again,
> in this file.**

It is made there, at [`CanvasTool::TextEdit`](crate::canvas::tool::CanvasTool::TextEdit),
and the short form is this: the objection was *"it would drag a whole
subsystem's state through this type"*, and the state does not go through the
type. What crosses the boundary is one [`TextEditKind`] — the same single
value `Markup` and `Measure` each carry — and the draft, the caret and the
anchor live in `egui::Memory` exactly where a half-finished measure pick
lives, for the reason `canvas::measure`'s header gives: *"a half-finished
pick is not part of the document and a document saved mid-gesture must not
carry one."* A half-typed word is the same category.

## §2. Two kinds, one tool — and why it is not two tools

`edit.text` edits a run that is already on the page; `edit.add_text` places
new page content. They are the same *gesture* — click somewhere, type, press
Enter — differing only in what the click resolves to and which engine verb
the commit calls. So they are one [`CanvasTool`] variant carrying a
[`TextEditKind`], which is `MarkupKind`'s argument restated: the operator is
doing exactly one of the two, and a type that could say both would need
discipline to keep honest.

## §3. It clicks AND it drags

> *"I should be able to make it multi line."*

**A PDF has no paragraph.** Each visual line is its own show operator at its
own absolute position, so something has to decide where the second line
starts: a width to wrap against and a leading to step by. That is
`AddTextRequest::with_box`, and it needs a rectangle — so the gesture is a
drag, and the `DragKind` is not a placeholder but the whole feature.

| gesture | anchor | what commits |
|---|---|---|
| click on existing text | [`Anchor::Run`] | `edit_text` |
| click on bare page | [`Anchor::Origin`] | `add_text`, one line at a point |
| **drag a rectangle** | [`Anchor::Box`] | `add_text` boxed, a wrapped paragraph |

The drag belongs to **this** tool and not to `CanvasTool::Text`. On the
sweep tool's rung the box would take the text sweep away in Edit, which
`text_tool_selects_and_marks_in_edit` depends on to make a selection the
markup verbs can act on. **Two features claiming one drag is a choice
somebody has to make, and taking a shipped gesture away to make room is the
wrong way to make it.** Add-text drags; the text tool goes on sweeping.

## §4. It does not disturb the text-selection gate

`canvas::textsel::gate`'s §3 warns that exclusivity between the text sweep
and the content marquee is now *by precedence*, not by construction. This
variant does not touch that: `takes_the_press` asks `tool.is_text()`, which
is `matches!(tool, CanvasTool::Text)` and is therefore **false** for
`TextEdit(_)` by construction rather than by an added condition. A press with
this tool armed is claimed by this module's own rung in `press_kind`, which
sits *above* the text-selection question for the same reason the measure rung
does — an armed tool takes the press — so the two can never both claim one
press. `gate.rs`'s tests carry a case asserting exactly that.

## §5. Capability: `edit_content`, and nothing wider

Unlike text *selection*, this authors. Every entry point is gated on
`Capabilities::edit_content`, which is Edit alone in the shipped manifest —
the dispatch arms decline by name and trace, and
[`crate::canvas::tool::retire_forbidden`] disarms the tool on the way into a
mode that cannot author, so a draft cannot survive into Read.

---

## §6. What is out of scope, said in words rather than by a dead key

`DEFECTS.md` D4a's **cross-run editing** is not built: it needs a multi-run
edit request in `pdfcer-core` that does not exist — `EditRequest` pins to one
show operator, and *"a `TJ` array is one operator"*.

What a caret landing where two runs meet does **not** do is go quiet. It
opens on the piece that was clicked and **discloses** the consequence on the
status bar — `crate::text::textedit::shares_the_line_note` — because a
control that takes keystrokes it will not honour is this module's defining
defect class, and so is a typing loop that is silently disabled.

D4c's **reflow gates** are out of scope and untouched.

---

## §7. The two costs an operator pays, and where they are disclosed

Rule 4 — *disclosure lives off-canvas* — so neither of these is drawn on the
page. Both reach the status bar through the disclosure list `vector_edit`
records, and one of them is a disclosure this shell adds because the engine
does not:

* **`Reflow`** — the engine already discloses that the line may now overrun
  its margin.
* **`Pin`** — the engine discloses nothing, because from its side pinning is
  what was asked for. But a pinned tail does not make room, so a longer
  replacement grows *into* it. [`plan`] adds
  `crate::text::textedit::pinned_tail_disclosure` for exactly this, which is
  why the pin is never silent.

## Item notes

### `const DRAFT_MEMORY_KEY`

One key for both kinds, because one draft can be in flight: arming the other
kind clears it, exactly as arming a different `MarkupKind` cannot reach a
drag already in flight.

### `fn an_unchanged_draft_pushes_no_action`

The no-op guard, and it is load-bearing because clicking away commits: an
operator who typed a letter and removed it again would otherwise get an
undo entry for having changed their mind.

### `fn a_box_draft_commits_with_its_wrap_rectangle`

The operator: *"I should be able to make it multi line."*

Asserted through the ACTION rather than through the engine, because the
action is where this shell's decision lives: `wrap: Some(..)` is what
becomes `AddTextRequest::with_box`, and a build that dropped it would
author the same characters as one long single line — plausible, silent,
and wrong in exactly the way the operator asked for.

### `fn a_point_draft_commits_without_one`

A build that gave every add-text a box would wrap a one-line label at
whatever width it invented — and the width would have to be invented,
because a click has no extent.

### `fn only_a_box_anchor_takes_a_paragraph_break`

Asserted on the ANCHOR, because that is the fact the keystroke handler
branches on. Driving the key itself needs a `Context` with focus and an
event queue, which `text_box_takes_a_paragraph` does in the real binary;
what is provable here is that the two anchors are distinguishable at all
— and a build that folded the box into `Origin` with an `Option<Rect>`
would fail this by construction.

### `fn an_emptied_run_draft_commits_the_emptying`

Deleting every character of a run and clicking away is ambiguous —
"remove this text" and "I changed my mind" are the same gesture — and
the recoverable reading is **undo**, not refusal, which is what Acrobat
does. The refusing reading raised no action at all, so there was no
plan, no engine call and no sentence: a declined edit and a failed save
looked the same from the operator's side.

The emptiness guard stays on the `Origin` and `Box` arms, where
`an_empty_add_text_draft_places_nothing` holds it.

### `fn an_empty_add_text_draft_places_nothing`

**Both** add anchors are asserted here, because this is the whole of
the emptiness guard: `commit_into`'s `Run` arm deliberately does not
carry one (`an_emptied_run_draft_commits_the_emptying`), so an assertion
covering `Origin` alone would leave the `Box` arm's guard held by
nothing while its header claimed otherwise.

### `mod lines`

Enter inserting a line break is one keystroke; a caret that can reach the
second line is four more. Its header carries the distinction that keeps it
separate from [`blocks`]: a line here is one the **operator typed**, and a
line there is one the **page draws** — different models, different costs
(nothing, against 336 ms), and confusing them moves the caret to another
part of the sheet mid-word.

### `const DIAG_TYPE`

**A diagnostic seam in the shape `app::files`' two already have** — see
`DIAG_OPEN_PATH` and `DIAG_SAVE_PATH`, which exist because a native modal
cannot be driven from a harness. This one exists because **this machine
cannot inject text**: `tools/ui-verify`'s `sys::vk` is a deliberately closed
list of eight non-character virtual keys, and its own comment refuses to
grow into `pub const A..Z` on the ground that *"a harness that can press any
key is a harness whose scripts stop being readable"*.

Typing is this feature's entire input, so without a seam the only honest
verification would be *"the tool armed"* — an assertion pointing in the
right direction that measures the wrong thing.

**It is not load-bearing and it is not a second input path.** It is read at
exactly one place, [`typing`], on the frame a caret is set, and what it does
is push characters through **the same** [`insert`] every `egui::Event::Text`
goes through. A build with the variable unset cannot tell it exists; a build
with it set still has to route the click, resolve the anchor, plan the
disposition and reach the engine, which is every link the check is about.

### `enum TextEditKind`

One value carried on the tool, for [`MarkupKind`](crate::canvas::markup::MarkupKind)'s
argument: the operator is doing exactly one of these, so a type that could
express both would have illegal states to prevent by discipline.

### `fn command_id`

The single binding between an id and a kind, in the shape
`shell::commands::markup_for_command` has — read from both directions by
`crate::shell::commands::text_edit_for_command` and by the label the
status bar shows, so the two cannot drift.

### `enum Refusal`

Every variant is a *sentence to show*, not a state to be silent in. That is
the whole difference from the old shell, which set a boolean and stopped
responding to the keyboard.

**There is no variant for "the line is made of several runs"**, and
there must not be: on a CAD sheet that describes nearly every click, and it
answers a question about the *line* when the operator is editing a *run*.
`place::resolve_run` carries the measurement. That case is a disclosure —
`text::textedit::shares_the_line_note` — not a refusal.

Every variant here is a genuine absence of a thing to edit, or a run that
nothing could be typed into.

### `fn composing`

# Two claimants, one predicate

This shell composes text in two different places, and both have to be asked
about:

1. [`egui::Context::text_edit_focused`] — a real `egui::TextEdit`: a form
   field, the page-number box, a dialog's box, the Find bar. **D1's
   predicate**, never `egui_wants_keyboard_input()`, for the reason
   `app::keyboard::collect` gives at length.
2. **A canvas text draft** — the caret this shell paints on the page, which
   is deliberately *not* a `TextEdit` (this module's header says why: the
   caret sits in PDF space at the glyphs' own scale, which a floating widget
   cannot do). **egui therefore reports no focused text field for an
   operator who is visibly mid-word.**

A call site that asks only the first is **defect D1 one rung along**: D1 was
`egui_wants_keyboard_input()` where `text_edit_focused()` was meant; this is
`text_edit_focused()` where *"anybody is composing"* was meant. Same shape,
same invisibility to a harness that builds a bare `Context` with no draft in
it, same silent loss of a key the operator is plainly pressing — the space
bar is the hand tool's modifier, so an operator typing on the canvas gets a
pan instead of a space. The operator: *"it doesn't accept spaces. Like how?"*

⇒ **A predicate with two claimants must exist once**, and
`tools/gates/check-typing-guard.sh` fails the build on any bare
`text_edit_focused()` outside this function.

# Why "is a draft in flight" and not "is a caret tool armed"

An armed tool that has not been clicked yet owns no keystrokes — the page
keys must keep working right up until the caret is placed.

### `fn abandon`

The teardown half of an exit, never an exit in itself. Returns `bool` for
the reason `measure::abandon` does: the ladder rung above it needs to know
whether this rung consumed the key.

**Every caller runs `commit_into` immediately before it**, so nothing in
the program reaches this without writing what was typed — Escape included,
which takes the [`settle`] route. Read the name as *forget*, not as *throw
away*: a caller that meant to discard would be the first, and would be
contradicting the ruling recorded on [`settle`].

# `text-edit-abandon` is not evidence that an edit was lost

A successful commit runs `keys::commit_into` and then tears the draft down
through this same path, so the trace line is emitted on the happy path too.
The evidence that a commit happened is the published action — `add-text` or
`edit-text`. A driven check must assert on that, never on the absence of
`text-edit-abandon`.

### `fn commit_into`

A draft byte-identical to what it replaces is **not a write**. Without this,
an operator who typed a character and deleted it again would put a no-op
entry on the undo stack every time they clicked away — the old shell's own
finding, and it matters more here because clicking out commits.

## An emptied RUN is a write; an empty ADD caret is not

The two look like one rule and they are opposite ones, so the emptiness
guard sits on `Origin` and `Box` and not on `Run`.

Deleting every character of a run and clicking away is ambiguous — *remove
this text* and *I changed my mind* are the same gesture — and this shell
takes the operator's reading, leaving **undo** as the recovery, which is
what Acrobat does with the same gesture. The other reading wrote nothing,
and its cost was not the lost edit: with no action raised there is no
plan, no `edit_text` call, no refusal to classify and **no sentence
anywhere the operator could read one**, so an edit that was declined was
indistinguishable from one that failed to save. O216.

An `Add` caret with nothing typed is genuinely not a write — there is no
content to remove and nothing to undo — so those two arms keep the guard.

### `fn settle`

# The rule it enacts: typed text is not thrown away by a navigation gesture

`OPERATOR_REQUESTS.md` **O222** and **O223**, in the operator's words:

> *"I think for adding and editing text when using any tool that has text
> escape should also save changes to the text. The user can always undo if
> they want, but it is easy to accidentally press escape and lose a lot of
> text that has been entered."*

That is a ruling about **asymmetric cost**. A draft written by mistake is
one `Ctrl+Z`; a draft discarded by mistake is minutes of typing with no
recovery anywhere, because a draft lives in `egui::Memory` and never
reaches the undo stack. So every exit writes first and tears down second,
and there is no exit that does the second without the first: [`abandon`] is
reached only immediately after a [`commit_into`], here and at the two exits
that inline the same pair (`Ctrl+Enter`, and a click that starts a new
draft). An exit that meant *throw this away* would be the first, and there
is none.

## Why it is safe for the emptied-run case

[`commit_into`] treats an emptied `Run` as a deletion and an empty `Add`
caret as nothing at all, so settling a draft the operator never typed into
raises no action and puts nothing on the undo stack. Settling is therefore
unconditional at the call sites: they do not have to ask whether the draft
says anything, which is the question that would get asked differently in
each of them.

### `struct Committing`

## Why this exists, and it is O141's second half rather than a cache

When the engine refuses a commit because the run's font has no code for the
character just typed, the shell offers a face that carries it
(`panels::properties::refusedchar`). Taking that offer must be **one**
gesture, and cannot be without this: `Ctrl+Enter` calls `commit_into` and
then `abandon` unconditionally, so by the time the offer is on screen the
draft the operator wrote is gone — leaving them to click back into the text
and type the character a second time.

The replacement text is the one operand of the retry that cannot be recovered
from anywhere else: the page still holds the *original* words (the refusal
changed nothing), and the page index and run index travel on the refusal
already — but **what the operator typed exists only in a draft that has been
abandoned**. So it is kept here, at the one function every text commit goes
through, and the offer re-applies the edit itself.

## Why a slot here rather than a parameter on the refusal recorder

`app::status::decline::textedit::record_edit_text_refusal` is called from
**inside** `vector_edit`'s closure in `app::actions::apply`, where the only
things in scope are the session and the engine's error. Widening its
parameter list means widening the router's call, and `app/actions/apply.rs`
is a file whose whole job is to route: it decides nothing, and an operand it
carries only to hand on is a decision it would then appear to have made.

More importantly the datum is not the router's. *What this edit is trying to
write* is a fact about the edit, and [`plan`] is the one function that has
it: every `CommitTextEdit` passes through it, exactly once, immediately
before `EditSession::edit_text` is called with the request it built. There is
no path from a caret to the engine that does not cross this line.

## Why staleness cannot bite, stated rather than assumed

The slot is written on **every** call and read only by
`panels::properties::refusedchar::record`, which is itself called only from
the refusal arm of the very `edit_text` this plan was built for. So the value
read is always the one written microseconds earlier in the same call stack.
It is a thread-local for the reason `refusedchar::PENDING` is one — the
writer is the dispatcher and the reader is a panel body handed `&OpenDoc`
shared — and the shell is single-threaded at the UI.

`canvas::textedit::last_commit_is_the_one_just_planned` asserts the round
trip, so a build that stopped writing it, or that wrote it before the
operands were known, goes red rather than silently retyping the wrong words.

### `fn last_commit`

See [`Committing`] for the whole argument. Cloned rather than borrowed
because the caller stores it: the offer outlives the frame it was recorded
on, which is the entire point of it.
