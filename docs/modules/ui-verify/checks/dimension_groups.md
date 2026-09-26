# `ui-verify/checks/dimension_groups`

`dimension_groups_panel_makes_a_group` — the Dimension-groups **panel**
opens, a group made in it reaches the document and comes back joinable, and
the same group can then be renamed and removed.

# The surface is a DOCK PANEL, not a window

A window whose content outgrows the screen can push its own title bar — and
its only ✕ — off the desktop, leaving the operator no way to close it.
`crate::checks` has no way to notice that from a passing check: every
assertion below is satisfied by a window just as well as by a panel, so the
surface is a panel by decision and not by evidence.

Two things follow from the surface being a panel, and both are worth naming,
because they are what a driven check is *for*:

1. **It clicks fold headings.** Five of the panel's six sections start
   shut, so the regions this check aims at do not exist until a heading is
   pressed. That is not a cost — it means the check proves the folds work,
   which is half of what the operator asked for and the half a unit test
   cannot see.
2. **It asserts no window opened.** It asserts a panel *body*
   region appeared, which is a weaker claim about layout and a stronger one
   about reach: the panel is the last tab of a stack in Review's right dock,
   so its region appearing proves the ribbon control raised a tab that was
   behind another one.

# THE CONTROL IS A TOGGLE, AND THIS CHECK ESTABLISHES ITS OWN
PRECONDITION

`app::panels::toggle_panel` closes a panel that is already on screen and
raises one that is not. A check that presses the ribbon control
unconditionally therefore shuts the panel on any run where it was already
the active tab of its stack, and then reports *"the arm ran, traced neither
a decline nor an unimplemented line, and no panel appeared"* — a defect
named in a panel that was working and visible one frame earlier.

⇒ **A driven check may not press a toggle without first reading the state
it toggles.** The guard is three lines: ask whether the panel's own body
region is live, and press the control only when it is not. `declared`
answers that honestly because the application retires a region it stops
drawing (`ui-rect-gone`), so a live `panel:dimension-groups` means *drawing
now*, which is the same predicate `DockLayout::is_on_screen` answers inside
the application.

The convention is `properties_metadata`'s and `bookmark_add`'s, worded the
same way here rather than reinvented: press a panel toggle only if the panel
is not already up. Three panel checks, one rule, one wording — because three
wordings become three rules and then three behaviours.

What the guard costs, stated rather than hidden: on a run that takes the
already-showing branch, nothing presses the ribbon control, so *that* run
does not prove the control raises a tab. The check writes a note saying so
instead of passing quietly on the weaker claim. Review's default arrangement
mounts this panel LAST in its stack — behind Comments, Properties and Forms
— so the ordinary run still presses the control; only a persisted layout
that left it active takes the other branch.

# The gap this closes

A command can be registered, drawn on Measure ▸ Scale, and **inert**.
Nothing about a ribbon entry proves an arm stands behind it, which is why
this check presses the real control rather than calling the arm.

# Why this needs driving, and not a unit test

Because the chain has five links and **four of them are frame-level or
cross-process**, and every individual link already had tests while the
feature did not exist:

1. a ribbon press reaches `app::dispatch`'s arm;
2. the arm resolves the measure tool's active authoring group out of
   `egui::Memory` — which may hold no state at all — and builds the dialog;
3. the dialog's Add button raises an `Action`, which is applied **after**
   the frame it was raised in;
4. the apply calls `EditSession::add_dimension_group`, which writes the
   `/PieceInfo` sidecar;
5. the **next** frame re-reads `dimension_model()` and draws a row for it.

Link 5 is the one worth the whole check. A group that is created and does
not come back in the model is a group nothing can ever draw a dimension
into — and it looks *exactly* like success at links 1 through 4, because
the undo entry is there, the epoch moved, and the trace line says the verb
ran.

# The assertion it would be easy to leave out

The last one: **a second `draw_into` radio appears**. Asserting only the
`add-dimension-group` trace line would pass on a build where the panel
writes to the document and lists nothing — which is the shape of every
panel in this project's history that shipped with a body, a rail entry and
no control anyone could click.

The radio is also the *point of the feature*. `MeasureState::group` is the
group picker, and a build in which nothing writes to it is a build where a
second group can be created and joined by nothing. A row with a radio on it
is the only evidence that the group is reachable rather than merely
recorded.

# The round trip, and the verbs it exercises

Create, **rename**, **delete** — ending with the list exactly as long as it
started. Each verb alone would show only that its arm exists; together they
show that the panel's three controls act on **the same group**.

That is the failure a per-row surface actually has and no single-verb
assertion can see: a rename field bound to the *selected* row while Delete
acts on the *authoring* row would let every individual step report success
while the operator renamed one group and deleted another. The two are
deliberately different things in this panel — the radio chooses where the
next dimension goes, the name chooses whose settings are on screen — which
is exactly what makes confusing them possible.

**The populated-group branch is not covered**, and is named rather than left
unstated: deleting a group that still has members asks a destination
question, and reaching it needs a fixture whose dimensions already live in a
named group.

## Item notes

### `const PANEL`

The `panel:` prefix is load-bearing in the trace — a reader scanning for
what drew has to be able to tell a floating window from a docked body — so
the region names the kind of surface rather than being kept short for the
harness's convenience.

### `const PANEL_BODY`

The DOCK's name, not the panel's: `egui-shell` publishes
`dock.body.<panel command id>` for every mounted panel, and it is the one
rect that says where the panel actually is this frame. `PANEL` below is the
application's own region for the same surface, and the two are not
interchangeable — the application publishes its when the body draws, the
dock publishes its when the layout resolves, and the gap between those two
moments is exactly the hazard `fold` guards against.

### `const CLOSED_EVENT`

Read only to improve a failure message, and it is the one line that can
tell a press that SHUT the panel apart from a panel that never drew.
The two look identical from outside — no body region either way — and they
have opposite fixes: one is the harness's precondition, the other is the
panel's own body.

### `const APPLIED`

`apply::vector_edit` traces the label it was given on the success path, so
this string is `DimensionAction::AddGroup`'s label verbatim. A *refusal*
traces `add-dimension-group-refused`, which is a different event and is
reported separately below — the difference between "the arm never ran" and
"the engine declined" is the whole diagnosis.

### `const RENAME_KEY`

**Appended rather than retyped**, and that is what makes the rename
observable without reading the text back. The field opens seeded with the
group's current name, and the Rename button is drawn **only while the draft
differs from it** — so a single extra letter producing a commit is itself
evidence the field was seeded from the document rather than opening empty.

A rename box that opened empty would let an operator wipe a name by pressing
a button they took for a no-op, and nothing else in this check would notice.

### `fn fold`

# Why this exists, and why it returns `Option` rather than failing

Five of the panel's six sections start shut, so the regions this check aims
at — the name field, the Add button, Rename, Delete, the appearance block —
**do not exist** until their heading is pressed. That is the surface the
operator asked for (*"each section should be able to fold up like the
settings one"*) and it makes this check the only thing in the project that
proves the folds open at all.

It returns the heading's own rect rather than a bare `bool` so a caller can
press it a second time to shut the section again. Shutting matters more than
it looks: the panel lives in a dock column, `declared` retires a region that
stops being published, and a section left open pushes everything below it
down the scroll region — where the next `declared_center` would aim at a
point the operator cannot see. Closing what has been inspected is how this
check stays inside the visible column without a single hard-coded height.

`None` means the heading is not on screen at all, which is a real failure
and is reported by the caller in its own words — a fold whose heading cannot
be found is a section that is unreachable, not a section that is shut.
