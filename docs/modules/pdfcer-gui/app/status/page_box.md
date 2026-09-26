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

## ★ Why an editable box at all

`GUI_ROADMAP.md` 3.3: *"Reaching page 37 of 42 currently means the
thumbnail rail or 36 keystrokes."* Type `37`, press Enter, arrive.

## ★ The commit rule

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

## ★ The three outcomes, and why none of them is silent

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

## ★ Defect D1, from the other end

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
