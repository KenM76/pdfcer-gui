# `ui-verify/checks/text_markup`

`text_markup_marks_a_selection` — the regression test for **a ribbon command
whose operand is not the pointer**.

Every other driving check in this crate asserts that a control *arms* a tool:
[`crate::checks::markup_rectangle`] and [`crate::checks::measure_linear`]
both end at `…-tool tool=…`, and what the operator does next is a gesture.
Underline, Strikeout and Squiggly are the first commands in this shell that
**author an annotation on the press**, from the text selection already on the
document — which is Acrobat's model and is argued at
`canvas::markup::text`'s header §1.


# The five-link chain, and which links no unit test observes

| # | Link | Where | Its own test |
|---|---|---|---|
| 1 | a sweep makes a text selection on the document | `canvas::textsel` + `canvas::interact` | partly — link 4 of [`crate::checks::text_selection`] |
| 2 | that selection publishes `selection.text` | `app::conditions` | yes |
| 3 | the condition **enables the control** | `egui_shell`'s `Enable::When` | yes, on each side |
| 4 | the click routes to a `TextMarkKind` and builds the action | `app::dispatch` via `shell::commands::text_mark_for_command` | yes |
| 5 | the action reaches `add_markup` and the engine authors it | `app::actions` | yes |
|   | **2→3 and 1→4 joined, in one running window** | — | **no** |

Link 3 is the interesting one and it is asserted here in a way the workspace
cannot: **a greyed `egui` control does not emit
`ribbon-command-invoked`**. So a click that produces no invoke is positive
evidence that the control was disabled, and a click that does produce one is
positive evidence that it was enabled — from outside the process, with no
knowledge of the condition's name. That is a *better* oracle here than pixels
would be: a disabled control differs from an enabled one mostly in its
**text** colour, which is a few dozen antialiased pixels inside a fill that
does not change, and `MIN_PRESSED_DELTA` is a fill measurement. Using it here
would have measured the wrong thing and passed.

# The four phases, and why the first is a negative

| Phase | State | Action | Expected | If it does not hold |
|---|---|---|---|---|
| A | nothing selected | click Underline | **no** `ribbon-command-invoked` | FAIL — a control that can only act on a selection was live without one, which is what `RIBBON_IA.md` P3 forbids |
| B | — | sweep a band | `canvas-text-selection chars>0 quads=N` | SKIP — this band had no text; try the next |
| C | text selected | click Underline | invoke **and** `text-markup-commit quads=N` **and** `add-text-markup n=1` | FAIL, with the missing line naming the link |
| D | selection now stale | click Underline again | **no second** `add-text-markup` | FAIL — a stale selection authored a second annotation |

Phase A is the half that would be easy to omit, and omitting it would leave
the check unable to distinguish *"the condition works"* from *"the control is
always live and the click happened to land after a sweep"*. It is also the
only assertion in the suite that a control is **correctly disabled**, which
is the direction P3 is usually violated in.

Phase D is the one that documents a real, deliberate consequence rather than
a defect: authoring a markup is an edit, `vector_edit` bumps `edit_epoch`,
and `canvas::textsel`'s §7 staleness rule therefore retires the selection
that authored it. Acrobat keeps its selection across a markup and this does
not. The check pins the behaviour so that a future change to the staleness
rule is a decision rather than an accident.

# The assertion that spans the process boundary

`canvas-text-selection … quads=N` and `text-markup-commit … quads=N` are
written by two different modules about two different values — the boxes the
**wash** was painted from, and the boxes the `/QuadPoints` was **authored**
from. `canvas::textsel` §5.1 claims they are the same list from one pass.
Comparing the two numbers is the only way to test that claim from outside the
process, and a build that re-derived the authoring quads from anything else
would have to reproduce the line grouping exactly to pass it.

# Mouse only

Every gesture here is a real `SetCursorPos` + `mouse_event`, and **nothing in
this check needs a key**


Continuing: — which is itself a consequence of the interaction
model chosen: select-then-press is two clicks and a drag.

# Every way this reports SKIP, and why none of them is a pass

* no binary, no `--pdf`, `--no-input` — the harness never began;
* the diagnostic switches did not reach the process;
* the page size could not be read and no `--page-size` was given;
* the mode segment, the Markup tab, or the Text markup group's controls were
  never declared;
* the canvas is not showing page 1, so the harness's one known page size does
  not describe the page it would be sweeping;
* **no band had text under it** — phase B never succeeded, so there is no
  selection for phase C to mark and phase A's silence would prove nothing.
