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

## Item notes

### `const MODE`

Marking text needs two things that do not overlap the way anyone expects:
the ability to *select* text, which `canvas::textsel::takes_the_press`
grants where the mode cannot select content (Read and Review), and
`author_markup`, which the mode's tab list grants where it contains
`markup` (Review and Edit). The intersection is **Review alone** — Read has
no Markup tab, and in Edit the primary button is the content marquee so no
text selection can be made and the three controls are permanently greyed.

See `canvas::markup::text` §2. That inversion is a known gap with a known
fix (`CanvasTool::Text`), and it is the reason this check does not carry an
Edit control phase the way [`crate::checks::text_selection`] does: in Edit
the controls are correctly dead, and a phase asserting so would be asserting
the gap rather than the feature.

### `const SUBJECT`

Underline rather than Strikeout or Squiggly because it is the one an
operator reaches for first and because the three are the same code path with
one `match` arm between them — `shell::commands::text_mark_command` maps all
three and the mapping's own test walks `TextMarkKind::ALL`. Driving all three
here would cost three more clicks and three more edits to prove what that
test already proves, while the *join* this check exists for is per-command
only in its id.

### `const SIBLING`

Not a pixel differential here (see the module header on why the invoke is
the better oracle for enablement), but its presence is still worth
asserting: a build that registered one of the three and not the others would
otherwise pass this check completely.

### `const DECLINE_EVENT`

Read to *improve failure messages*: `reason=Stale` and `reason=NoSelection`
send a reader to two different places, and both are different again from a
command that never reached dispatch at all.

### `const APPLY_EVENT`

The line that makes this check about a document rather than about an intent.
[`COMMIT_EVENT`] says the shell decided to author one; this says
`EditSession::add_markup` returned `Ok` and the revision moved.

### `fn invokes`

A **count**, not a presence, and for the reason
`driving::click_mode_segment` counts its mode events: this check clicks the
same control three times, and "has it ever been invoked?" would be answered
`true` by a click made ten seconds earlier.

### `fn the_selectors_match_the_shells_own_spelling`

Pinned here for the reason [`crate::checks::markup_rectangle`]'s twin
test states: the two crates are joined by a **string** and nothing else,
so a rename would leave both sides compiling while every assertion here
quietly stopped matching — and a check that matches nothing passes
vacuously.

### `fn a_sweep_that_ends_cleared_has_selected_nothing`

The two traces differ only in which line is last, and that is the whole
point: reading the last *non-empty* line instead of the last line is
what made this check report a correctly-greyed control as a dead
feature.
