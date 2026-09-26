# `ui-verify/checks/restyle_text`

`restyling_selected_text_reaches_the_document` — **sweep text, press Bold,
and the file changes.**

# What this is for

O37 — *"We should also have all the font tools available that Word does"*.
Without this check every claim about that feature rests on unit tests over
the engine, and R1 is explicit that a green suite is not a report of working
software: a defect can be invisible to every test and obvious within thirty
seconds of using the program.

## The specific way this feature can pass its unit tests and be dead

`app::actions::textstyle` is tested against a document. It calls
`format_text`, reads the file back, and asserts the size changed. **Every
one of those tests would still pass on a build where the panel never draws**
— where the section's guard is wrong, where the operator's sweep produces no
`text_selection`, where the panel is not in the dock, or where the Bold
button is below the fold of a scroll area.

Each of those is a state a running window shows in two seconds and no unit
test can construct, because what is under test is *the chain of surfaces*
rather than the function at the end of it. That chain is:

| # | link | its own test |
|---|---|---|
| 1 | a sweep on the canvas produces a `TextSelection` | shared with `clipboard_text` |
| 2 | `TextSelection::runs` answers with run ordinals | unit-tested |
| 3 | the Properties panel draws a Text section for it | **nothing** |
| 4 | the section's read-back stamp does not wipe itself every frame | **nothing** |
| 5 | Bold is on screen and clickable | **nothing** |
| 6 | the press reaches `format_text` | unit-tested, eight ways |

Links 3, 4 and 5 have no other instrument — link 4 because `sync` runs
per frame and a stale stamp is invisible to a unit test. Link 4 is also the
one that would fail
most plausibly: `sync` runs every frame, and a stamp recomputed too eagerly
re-reads the document between the operator's press and the button's read.

## There is a link 0, and this check spent a red run learning it

**The panel has to be on screen before any of the six can be observed**, and
the dock arrangement is *persisted per machine* — so it is not a constant
this check may assume. A check that assumes it finds no `properties.text`
region and reports the panel as saying nothing about the selection. The
panel is not saying nothing; it is not there. [`INVOKE`] carries the
mechanism and the evidence.

The general form, and it is worth stating because it will apply again:
**a driven check must put the surface it is about on screen itself.** An
arrangement that happens to be right on the machine the check was written on
is not a precondition, it is a coincidence, and the failure it produces
accuses the program of exactly the defect the check exists to find.

# Why Bold and not the size field

Because it is **one click on a button** and the size field is an
`egui::DragValue`, which takes a number by double-click-then-type or by
scrub. `geometry_fields` scrubs, and its own header gives the reason to
prefer that over typing; but a scrub also has to be arithmetically
reconciled against a speed constant, and a check whose failure could be
either the program or its own arithmetic is a worse check.

Bold has no such ambiguity: pressed or not pressed. It is also the control
with the most machinery behind it — the two-verb retry — so it is the one
whose silence would be most expensive.

# The oracle, and why it is two lines rather than one

`text-style … applied=N of=N`, **plus** the `format-text` label that
`vector_edit` writes when the edit reached the engine.

One alone is not enough in either direction. The first without the second is
a module that decided to act and whose action never landed — the exact shape
of the grips' year-long defect. The second without the first cannot happen,
and asserting it alone would let a check pass on a build where some *other*
verb wrote that label in the same window.

A `text-style-declined` is reported as a **failure with its reason**, not
as a skip. A refusal here is the program answering, and the sentence it
answers with is the thing worth reading — a `FaceLacksCharacters` on this
fixture would mean the two-verb retry took the offer and the offer was bad,
which is a real finding about the engine and not a reason to stay quiet.

## Item notes

### `const INVOKE`

# The Properties panel has to be ASKED FOR, and not asking for it cost
a red run

`file.properties` is the command that mounts and activates the Properties
panel from **any** arrangement (`app::panels::show_panel` — it activates the
panel if it is already mounted and otherwise looks its home up in the `edit`
default, and it is idempotent rather than a toggle). Two sibling checks —
`field_delete_gate` and `annot_delete_gate` — already launch this way, and
the reason they give is the one that applies here:

> so the check does not have to know what dock layout the machine it runs
> on happens to have persisted.

Without it the check fails with *"266 character(s) are selected and the
Properties panel says nothing about them"* and names three candidates, of
which the first — the panel is not mounted — is the real one. A trace from
that state settles it without ambiguity:

| evidence | in `restyle-text.trace.txt` |
|---|---|
| six panel **bodies** were on screen | `dock.body.view.panel_tool`, `.panel_objects`, `.panel_pages`, `.panel_layers`, `.panel_forms`, `dock.body.measure.manage_groups` |
| ten panel **tabs** were declared | none of them `dock.tab.file.properties` |
| the Properties panel | **no tab, no body, and no `panel-shown` line** |

The section is then neither guarded out nor failing to pin — there is no
panel for `panels::properties::text::section` to draw into at all, and the
other two candidates cannot be reached, let alone judged, until this
command has run.

`mode.edit` is first and is load-bearing: `dispatch`'s mode arm sets the
ribbon mode and *"the dock follows on the same frame"*, so a panel mounted
before the mode moved would be mounted into the workspace the check is about
to leave. Ordering them here rather than clicking afterwards is what makes
that impossible.

The Edit-mode segment is still **clicked** below, and that is not
redundant: `docks` compares `ribbon.mode()` against `modes.active()` and
does nothing when they agree, so the click cannot disturb the panel, and it
keeps the check driving the operator's own gesture rather than trusting an
environment variable to have taken.

### `const STYLE_EVENT`

Named `-applied` rather than plain `text-style` because `vector_edit`'s
label for the same edit is a sibling event, and trace matching is on the
exact event name. Two lines sharing a name is how a check reads the wrong
one and then reports `applied=0` about a gesture that worked.

### `const SWEEP_PT`

Sixty. The fixture's title-block labels are six-point text a few tens of
points wide, so this crosses a whole label and a little blank paper. It does
**not** need to stop inside the run: a sweep that overshoots selects the run
and stops, which is the operator's own gesture.

### `const VK_T`

Pressing it is not optional, and the first run of this check is why.

`textsel::gate::takes_the_press` reads
`tool.is_text() || (Select && !caps.edit_content)`. In **Edit** the second
disjunct is false, so a drag with the Select tool is an object marquee — the
first run of this check produced `sel=1` on the trace and reported the panel
as broken, when what had actually happened is that it never swept any text.

That is a discoverability finding about the product, not only about the
harness, and it is written up rather than absorbed: an operator who wants to
restyle text in Edit mode has to know to arm a tool first, and nothing on
screen tells them so.

### `fn wait_for_verdict`

A bounded poll rather than a fixed sleep. **A wait the harness owns is a
failure mode the harness owns**: a sleep long enough for the worst case
makes every run slow, and one short enough to be pleasant fails on the
operator's own drawings — which is where this check is aimed.

Returns the elapsed milliseconds. Reaching the ceiling is **not** an error
here: the caller then reads a trace with neither line in it and reports the
"nothing happened" verdict, which is the right answer if a restyle really
can take twenty seconds.
