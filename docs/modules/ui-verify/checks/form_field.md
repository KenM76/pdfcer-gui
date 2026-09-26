# `ui-verify/checks/form_field`

`form_field` — **place a form field on the page, then click an existing one
and get its properties.**

The driven assertion for the operator's request, in both its halves:

> *"when I click one I should be able to click on the canvas to place the
> position or drag a box for size then a pop up lets me set the details for
> the feature"* … *"when I click on an existing form field on the page it's
> properties should come up in our side pane for editing it's properties."*

# Why this check is the only oracle for most of the feature

Everything between a click and an authored field crosses boundaries a unit
test cannot: an armed tool in `egui::Memory`, a gesture resolved from a real
pointer, a canvas→page transform against a real page, a **second OS window**,
and five `pdfcer-core` verbs. The unit tests cover each rule; not one of them
covers the sequence, and the sequence is where this project's defects live.

The precedent is the shell's own founding defect: the Delete key's guard was
*"analysis-confirmed, NOT empirically verified"*, its unit test built a bare
context with no widgets, and the condition that broke the real application
could not occur in the harness.

# The dialog is answered by a seam, and that is not a shortcut

`PDFCER_DIAG_FORM_ACCEPT=1` makes the placement dialog press its own Add on
the first frame it is authorable. This harness drives **one** window — the
one `Session::launch` found — and the dialog is a deferred viewport with a
window of its own, so without the seam everything downstream of placing is
unreachable: the five engine verbs, the narrowing in
`app::actions::forms::author`, and all four rule-4 disclosures.

Two seams already exist for exactly this shape — `PDFCER_DIAG_OPEN_PATH` and
`PDFCER_DIAG_INSERT_PATH`, both substituting the answer to a native picker.
What this one substitutes is the **operator's press**, not the authoring:
it sets the same flag the Add button sets, so the readiness guard, the
action, the remembering and the engine call are all the path an operator
takes.

# The two clicks aim at deliberately different places

The first must land on **empty page** — a click on an existing widget would
place a field on top of one, which is legal and would make the second phase
ambiguous. The second must land on a widget whose canvas rect the
application itself published in a `form-box` line, so the check aims at
where the program says the box is rather than at where the fixture author
thought it would be. **A click that hits the field next to the one it
aimed at produces the same screenshot as a click that worked**, so the only
safe target is a rect the application itself published this frame.

# Phases

| Phase | Does | Expected |
|---|---|---|
| A | launch with `mode.edit,edit.form_text_field` | `form-tool-armed kind=Text` |
| B | click empty page | `form-field-open kind=Text`, then `add-form-field` succeeded |
| C | Escape, then click a published `form-box` | `form-field-selected field=…` |
| D | read the properties region | `properties.form_field` declared |

## Item notes

### `const INVOKE`

The list form of `PDFCER_DIAG_INVOKE`, which exists because arming a form
tool takes two commands: the arm declines without `edit_content`, so Edit
mode has to be entered first. Using it here rather than clicking a mode
segment also removes a whole class of flake from this check — a mode segment
click that misses is a failure about the ribbon, not about forms.

**`file.properties` is in the middle of the list, and phases D–F cannot
run without it.** Edit mode's default dock puts Properties in a TABBED stack
with Comments, Forms, Redact, Dimension groups and Attachments
(`app::modes::defaults`), and a tabbed stack draws only its **active** tab.
If Properties is not the active tab then
`panels::properties::formfield::section` never runs on a single frame, no
`properties.form_field` region is ever declared, and phase D reports a
properties pane that "did not draw" about a pane that was never asked to
draw. `file.properties` is `show_panel`, not a toggle — it mounts
the panel and brings it to the front of whatever stack holds it, from any
mode (`app::tests` asserts exactly that), so ringing it is idempotent.

It goes AFTER `mode.edit`, because a mode change re-applies that mode's
default arrangement and would undo it, and BEFORE `edit.form_text_field`, so
that nothing runs after the tool is armed that could put it down.

### `const BOX_LINE`

`form-target`, not `form-box`. The two censuses describe different sets
and the difference is exactly what form authoring added: `form-box` lists
what a click can FILL, which excludes a drop-down, a push button and any
widget with no appearance. Aiming at that list would make this check unable
to reach three of the five kinds it exists to verify — and on a fixture whose
only text field is undrawn, unable to reach anything at all.

### `const VIEWPORT`

The harness's default window gives the Properties panel about **180 points**
of dock slot — it shares the right column with the Tool and Objects panels —
and a selected form field now draws about **450 points** of content there:
the read-only facts, the rename box, seven editable properties, the two
delete buttons, and the box's own four numbers.

**That is a real finding about the product and it is recorded here rather
than absorbed.** An operator on a 1,100 × 800 window has to scroll a
180-point window through 450 points of pane to reach the controls that move
a box, and *"I clicked the field and there is nothing there"* is what that
looks like from a chair. It is written up in `FEATURES.md`; the remedy is a
layout decision (collapsible groups, a taller default slot, or the dialog
route O39 already uses for placement) and it is the operator's call, not
this check's.

What this check does about it is drive at the biggest window the desktop
will actually show, which is `read_mode_chrome`'s precedent and its
reasoning: a check's job is to exercise the feature, and a check that failed
because the *window* was small would be reporting the wrong subject. **The
scroll loop below is what carries the step regardless** — a taller window
makes it need fewer notches, not none, and on the display this is driven on
it needs them all. The measurement below says why the room cannot be had.

# A viewport is a request; a PLACED window is an arithmetic obligation

`PDFCER_DIAG_VIEWPORT` states a size, not a position:
`launch::Session::place` moves **every** launched window to desktop
`(780, 40)`, deliberately, to clear the top-left corner where the Windows
on-screen keyboard docks. So the real constraint is
`SAFE_ORIGIN + size ≤ desktop`, and asking for more does not fail — it hangs
the far edge of the window off the screen, which is precisely where the
Properties panel lives.

**The consequence is invisible and total.** `SetCursorPos` **clamps** an
off-desktop coordinate rather than refusing it, so a wheel aimed past the
right edge still lands over the panel and scrolls, and the step looks
healthy — while a click aimed past the bottom edge lands a few points above
its target and the check reports that the control reached nothing. That is
an accusation against the application for a pixel the harness could not
deliver. `Driver::confirm_uncovered` refuses such a click by name rather
than clamping it, so the next check to overreach is told.

This constant is what keeps this one inside the screen:
780 + 1100 + 16 px of border = 1896, and 40 + 980 + 39 px of title bar =
1059 — both inside the 1920 × 1080 this is driven on, with room to
spare. **A margin of one pixel is a margin that a theme change takes
away.**

# And the height it CAN get is still not enough, which is the real point

1000 points of window is ~400 points of Properties slot once the tab bar,
the Objects panel above it and the status bar have taken theirs — against
~1100 points of content for a selected field. **No window on this display
puts these controls above the fold.** So the scroll loop in phase E is not a
fallback for a small screen, it is the mechanism; this constant only decides
how many notches it spends.

### `const PANE_REGION`

`egui_shell::dock` publishes `dock.body.<panel command id>` for the body of
every mounted pane, and that rect is the visible slot by construction: it is
what the dock gave the panel, before the panel scrolled anything inside it.
See [`scroll_to`] for the three content rects that were tried first and how
each of them failed.

### `const EDITABLE_REGION`

Its own region, distinct from [`PROPERTIES_REGION`], and the separation
is the point. The section above it — the read-only facts, the rename box,
the delete buttons — draws perfectly well on a build whose pane is entirely
READ-ONLY, so a check asserting only `properties.form_field` passes there,
correctly, because what it asserts is true.

The two regions are therefore two separate claims: *"clicking a field
describes it"* and *"clicking a field lets you change it"*. Only the second
distinguishes an editor from a viewer.

### `const DEFAULT_VALUE_REGION`

Asserted here rather than in a check of its own because the expensive
part is already paid: this check authors a text field, selects it, and
scrolls the properties pane to its editable section. Adding a second launch
to look at one more control in the same section would cost thirty seconds
per run to assert something this one is already looking at.

⚠ **Text fields only.** `/DV` is a text string on a `/Tx` and a NAME on a
`/Btn`, so `panels::properties::fieldedit` gates the row on the field type —
and this check authors a text field, which is why the assertion is
unconditional here and would not be in a check that authored a checkbox.

### `const ALIGNMENT_REGION`

Asserted in the SAME late block as [`DEFAULT_VALUE_REGION`] and for the
same reason: reaching it scrolls the pane, and anything that moves the pane
belongs after every phase that clicks at a computed point. That ordering was
learned the expensive way — see the block's own comment.

### `const EDIT_APPLIED`

Named after the ENGINE verb, so the line says which crate did the work —
the convention `format-text` follows, and the one that was learned the hard
way when a module's summary line and `vector_edit`'s label shared a name and
a check read the wrong one.

### `fn placed_boxes`

The application's numbers, not the fixture's. `canvas/forms.rs` publishes
one line per widget precisely so a harness can aim at where the program says
the box is; a check that computed the rect from the PDF would be asserting
that two independent derivations agree, and would report a disagreement as a
hit-test failure.
