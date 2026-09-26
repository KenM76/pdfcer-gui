# `app::status::page_box` — page navigation, and the box you type into


- Everything left in the parent answers *"how is the bar laid out, and
  what does each group show?"* — a fixed row, a demoted narrator, and two
  clusters of stateless buttons that read `doc.view` and push an
  [`Action`].
- Everything here answers *"what did the operator mean by what they
  typed?"* — which is a different question with its own state
  ([`PageBox`]), its own vocabulary ([`PageCommit`], [`Note`]), its own
  pure decision function ([`resolve`]) and its own hazard (defect D1's
  keyboard guard). None of that is shared with anything left behind.

The parent's module docs carry the *why* for the surface as a whole,
including the four properties this control is judged on and the R128
fixed-height argument. What follows is the part that only concerns the
box.

## Why an editable box at all

`GUI_ROADMAP.md` 3.3: *"Reaching page 37 of 42 currently means the
thumbnail rail or 36 keystrokes."* Type `37`, press Enter, arrive.

## The commit rule

**Enter or focus loss. Never a keystroke.** Someone typing `42` passes
through `4`; a box that navigated per keystroke would take them to page
4, rasterize a CAD sheet nobody asked for, and then take them to page 42.
[`Response::lost_focus`](egui::Response::lost_focus) covers both exits at
once, because egui's single-line `TextEdit` surrenders focus when its
return key is pressed (`egui-0.35.0/src/widgets/text_edit/builder.rs:1115`
— *"End input with enter"*). One commit path, so the two cannot disagree.

`Escape` is the third exit and it **cancels**: the draft is dropped and
the box goes back to showing the current page. Without it, an operator
who started typing a page number has no way out but to retype the one
they were already on.

## The three outcomes, and why none of them is silent

| typed | outcome | what the operator sees |
|---|---|---|
| `37` in a 42-page document | [`PageCommit::Go`] | the page changes; the box reads `37` |
| `99` in a 42-page document | [`PageCommit::Clamped`] | the page changes to 42, **and a note names the number that does not exist** |
| `abc` | [`PageCommit::NotANumber`] | nothing moves, **`abc` stays in the box**, and a note says why |

A silent clamp is indistinguishable from a box that ignored what was
typed, and an operator who cannot tell those apart stops trusting the
control. A refusal that *also* wiped the field would destroy the evidence
of what they meant — which is why the draft outlives a failed commit and
never outlives a successful one.

## Defect D1, from the other end

`crate::app::keyboard` guards the unmodified bindings — PageUp, PageDown,
Home, End, Delete — with `ctx.text_edit_focused()`, **not**
`egui_wants_keyboard_input()`. The latter means "any widget has focus",
the canvas takes focus on click, and the difference cost the operator
every one of those keys from the first click onward.

`text_edit_focused()` resolves the focused id and asks whether a
`TextEditState` exists **for that id**. So this control has to be a real
[`egui::TextEdit`] with a stable, explicit id, or the guard cannot see it
— and `PageDown` would step the page while the operator was halfway
through typing `42`. A `DragValue` in display mode, a painted field, or a
label with a popup would each typecheck and each re-open D1 in reverse.

[`tests::typing_a_digit_into_the_page_box_does_not_also_step_the_page`]
is the regression test, and it asserts the *failing condition is really
present* before asserting the fix — the shape the D1 post-mortem says the
original test was missing.

## Item notes

### `const PAGE_BOX_WIDTH_PTS`

Four digits of a proportional face plus the field's own margins: enough
for `9999` without eliding, which covers every document anyone has opened
in this application. A box that grew with the page count would move the
two step buttons every time a different document was opened.

### `const PAGE_BOX_MAX_CHARS`

Not a validation rule — [`resolve`] is what decides whether the text
names a page — but a guard against an operator pasting a paragraph into a
44-point-wide field and losing the control's shape. Nine digits is past
any real page count and past the point where the text is legible anyway.

### `const REGION_PAGE_BOX`

Named separately from [`REGION_PAGE`] because it is the control this
stage exists to build: a legibility or hit-target check wants the field
itself, not the field plus two buttons and a possible note.

### `const PAGE_BOX_ID`

**Fixed and explicit, not auto-generated.** Three things depend on it
being stable across frames: `TextEdit` finds its own cursor and selection
under it, `ctx.text_edit_focused()` finds the `TextEditState` under it
(defect D1's guard — see the module docs), and [`tests`] request focus on
it directly. An auto id derived from widget order would change the moment
a note appeared beside the box.

### `fn wheel_toggle`

> *"when in single page view there should be an option on screen near the
> button to scroll or flip through pages, or the current way it is now when
> the scroll wheel is used."*

# It renders NOTHING under a continuous display mode

R9: an unavailable capability renders nothing, and greying is reserved for
*temporarily* unavailable. Under
[`crate::viewer::PageDisplay::Continuous`] the wheel scrolls the whole
document **by definition** — there is no second answer to offer, so there
is no control. A disabled toggle there would be a permanent apology for a
choice that does not exist.

It sits immediately to the left of `⏴`, inside the page group's own
right-to-left scope, so it is adjacent to the two buttons it is an
alternative to. Placing it in the empty middle of the bar would put the
question a hand's width from its subject.

# A toggle, not a pair of labels

The two answers are not peers: one is what the build has always done and
the other is the departure from it. A pressed/unpressed control says that
— *flipping is on* — where two `selectable_label`s would present them as
equals and cost twice the width in a 24-point bar. The tooltip carries
both sentences, and the settings window carries the full argument for each.

Drawn wherever the choice changes what the wheel does: every
non-continuous display, a one-page document included, where flipping on
stops the wheel scrolling the fitted page
([`crate::canvas::paging::wheel_turns_pages`]).

### `fn field`

See the module docs for the commit rule, the three outcomes, and why this
must be a real [`egui::TextEdit`].

While a draft exists the box shows the draft and **not** the page, which
is what makes rejection non-destructive: refusing `abc` leaves `abc` in
the field with a note beside it, rather than silently restoring the
current page and leaving the operator to wonder whether the field is
broken or they mistyped.

### `enum Note`

There is no `Ok` variant: a commit that went exactly where it was asked
needs no explanation, and a note beside every successful navigation would
train the operator to stop reading the ones that matter.

### `enum PageCommit`

A separate type from [`Note`] because the two answer different questions:
this one says *what to do*, including the successful cases; `Note` says
*what to tell the operator*, which is only the surprising subset.

### `fn resolve`

**Pure, and that is the point.** `crate::viewer`'s header states the
project's split — *"this module is unit-testable and the widget code is
not"* — and every property this control is judged on lives here rather
than inside an `egui` closure: what counts as a number, what happens at
the ends of the document, what happens to nonsense.

# The rules, and why each is what it is

- **Surrounding whitespace is ignored.** `" 37 "` is a pasted page
  number, not a typo, and refusing it would be pedantry the operator has
  to work around by hand.
- **A leading `+` is accepted, a leading `-` is not.** `+37` is a number;
  `-37` is a *different* number, one no document has, and silently
  reading it as 37 would be inventing an intent. It falls to
  [`PageCommit::NotANumber`], which keeps the text so the operator can
  see what they typed.
- **Only ASCII digits count.** A full-width `３` or an Arabic-Indic `٣`
  is refused rather than guessed at; pdfcer has no locale model, and a
  half-implemented one that worked for three scripts would be worse than
  an honest refusal.
- **An absurdly long run of digits is a number, not nonsense.**
  `99999999999999999999` overflows `usize`, and `parse` would report that
  as an error indistinguishable from `abc`. It is saturated to
  [`usize::MAX`] instead, so it clamps to the last page and *reports the
  clamp* — which is the honest answer to "go to page ten quintillion".
- **Page 0 does not exist.** The box is 1-based (see
  [`crate::text::status::page_number`]), so `0` is out of range at the
  near end and clamps to page 1 with a note, exactly as `99` clamps to
  the far end with one.
- **The clamp is `crate::viewer::clamp_page_index`**, not a second
  spelling of the same arithmetic. There is one rule for "which page is
  this really", it is already tested against the empty-document case, and
  a private copy here is how two clamps drift apart.

`page_count == 0` yields [`PageCommit::Empty`]: there is nowhere to go.
The caller never asks — [`group`] draws nothing for a document with no
pages — but a decision function that panicked or invented an answer for
an input its caller happens not to produce is one refactor away from
being wrong.

### `fn the_displayed_number_and_the_action_index_differ_by_exactly_one`

An off-by-one here is the most likely defect this control can carry
and the least likely to be noticed in review: typing `37` and landing
on 36 looks like a rendering delay until you check twice.

### `fn an_out_of_range_number_clamps_and_says_what_it_asked_for`

The verdict carries the number that was asked for as well as the one
that was given, because that is what
`crate::text::status::page_clamped_note` needs to distinguish "your
number was out of range" from "the box ignored you".

### `fn an_absurdly_large_number_clamps_rather_than_being_refused`

`parse` reports an overflow the same way it reports `abc`, and
treating "go to page ten quintillion" as a typing error would refuse
an input whose meaning is perfectly clear. It clamps to the last page
and reports the clamp.

### `fn non_numeric_input_is_refused`

The other half of the requirement — that the text survives the
refusal — is a property of the widget and is asserted in
[`a_refused_commit_keeps_what_the_operator_typed`].

### `fn a_document_with_no_pages_resolves_to_nothing`

Unreachable through the widget — [`group`] draws nothing for
`/Count 0` — and pinned anyway, because a decision function that
invented an answer for an input its caller happens not to produce is
one refactor away from being wrong.

### `fn typing_a_digit_into_the_page_box_does_not_also_step_the_page`

`crate::app::keyboard`'s guard is `ctx.text_edit_focused()`, and it
only protects a control that egui recognises as a text edit. This
asserts, in order:

1. `egui_wants_keyboard_input()` is genuinely `true` — so the test is
   known to be exercising the condition rather than passing
   vacuously. Its absence from the original D1 test is exactly why
   that defect shipped.
2. `text_edit_focused()` is `true`, i.e. the box really is a
   `TextEdit` under the id the guard resolves. Replace it with a
   `DragValue` in display mode or a painted field and this fails.
3. With the box focused, `keyboard::collect` installs **no**
   unmodified binding — so a digit typed on the way to `42` cannot
   also page the document.
4. Typing does not commit. Nothing is raised until Enter or focus
   loss.

### `fn the_page_keys_come_back_the_moment_the_box_loses_focus`

The mirror of the test above, and the reason it matters is D1 itself
— a guard that is always on is exactly as broken as one that is
always off, and it is much harder to notice.

### `fn a_refused_commit_keeps_what_the_operator_typed`

Wiping the field to "helpfully" restore the current page destroys the
evidence of what the operator meant, and leaves them unable to tell a
rejection from a control that does nothing.

### `fn a_clamp_note_is_forgotten_once_the_operator_moves_away`

The note explains where *this* commit put them. Left in place it
would attach that explanation to a page they reached with the ⏴
button, which is a small lie told confidently.

### `fn the_step_buttons_raise_the_shared_navigation_actions`

Not a tautology: the whole value of a mirror surface is that it
invokes the *same* command, and a status bar that raised its own
page-stepping arithmetic would be a second navigation model to keep
in step with the keyboard's.

Asserted through the action type rather than by clicking, which would
need synthesized pointer input at a rect this test has to predict.
What is worth pinning is that the variants exist and are the ones
`keyboard::collect` produces.
