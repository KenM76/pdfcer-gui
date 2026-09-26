# `canvas::tool` — which pointer tool the canvas is in, and the space bar that borrows it

## What this module is for

`GUI_ROADMAP.md` Phase 3.2: *"There is no hand tool at all; panning is
middle-drag only."* This is the hand tool, and the space bar that borrows
it for as long as it is held.

It owns exactly one question — **what does the primary button mean right
now?** — and answers it as a pure function of two inputs: the tool the
operator *chose* ([`selected`]) and whether the space bar is *down*
([`space_held`]). Everything else in `canvas/` reads [`active`] and
branches on the answer.

## Why the space override is derived and never stored

The requirement is *"space held = temporary pan, releasing returns to the
previous tool"*, and the obvious implementation — remember the previous
tool on key-down, restore it on key-up — is the one that fails. It fails
in the ordinary way (an interrupted key-up: the window loses focus mid-pan,
the operator alt-tabs, a dialog steals the release) and the failure is
*sticky*: the canvas is left in a hand tool the operator never chose and
cannot leave except by choosing something else. Every application that has
ever shipped a modal space-pan has shipped that bug at least once.

So there is **no stored override and nothing to restore**. [`selected`] is
the only persistent value; the space bar is read fresh from
[`egui::InputState`] on every frame and composed with it by [`resolve`].
"Returning to the previous tool" is then not an action that can be missed —
it is what the next frame computes when the key is no longer down. A lost
key-up costs one frame of pan, not a stuck mode.

## The text-field guard is not optional

Space is a *character*. A canvas that panned on any Space keypress would
pan while the operator typed a page number into the status bar's page box
or a value into the Properties panel. The guard is
[`egui::Context::text_edit_focused`] — the same predicate, for the same
reason, as `DEFECTS.md` D1's Delete-key fix, and deliberately **not**
`egui_wants_keyboard_input()`, which is true whenever *any* widget has
focus and would therefore disable space-pan after a single click on the
canvas (the canvas takes focus on click, which is exactly how D1 happened).

## The bar a new variant has to clear

This is **not** a general "tool" enum with one member per authoring surface.
It answers a narrow question — **does a primary drag select, move the paper,
or draw?** — and a candidate is a *mode* that arms a whole surface until it
clears one bar: it must **arrive with its own state**, and it must need this
enum to hold no more than *which kind is armed*.

[`CanvasTool::Markup`] is the shape every later admission is measured
against. It arrives with [`markup::MarkupKind`], with a `DragKind` and a
`GestureOutcome` of its own in [`crate::canvas::gesture`], with a rubber
band, a commit path and an `Action` — and nothing of it outlives a frame
except **which kind is armed**, which is precisely one enum value and
exactly the kind of thing this module already stores. So the enum grows by
one variant *carrying* the kind, rather than by four.

Two rules keep the growth cheap, and both are enforced here rather than at
call sites: [`CanvasTool::pans_with_primary`] is the single predicate the
pan and gesture-suppression paths share, and [`CanvasTool::cursor`] is the
single place a tool's cursor is decided.

## What each later variant had to bring

**[`CanvasTool::Measure`]** reads like a counter-example — a two-point pick
with a snap indicator and a live readout — and is not one: that machinery is
[`crate::canvas::measure::pick`]'s, and none of it has to be held here. What
crosses the boundary is one [`MeasureKind`], exactly as markup's one
[`markup::MarkupKind`] does.

**Text selection** ([`CanvasTool::Text`]) **clears the bar more cleanly than
either**, and the set it arrives with is:

| it arrives with | where |
|---|---|
| a selection type, with its own staleness rule | [`crate::canvas::textsel::TextSelection`] |
| a [`PressMeaning`](crate::canvas::gesture::PressMeaning) and a `DragKind` | [`crate::canvas::gesture::DragKind::TextSelect`] |
| a resolver — one pass producing the string, the canvas boxes and the page quads | `canvas::textsel::resolve`, reached through `drag` / `click` / `select_all` |
| a commit path, in the only sense it has one: three markup kinds whose operand is the selection | [`crate::canvas::markup::text`] |
| two keyboard verbs of its own | [`crate::canvas::textsel::clipboard`] |

…and **the only thing it needs to persist is that it is armed.** Not a range,
not a caret, not an anchor: the range lives on the document beside the object
selection, the anchor is re-derived from the press origin on every frame of a
sweep (`textsel::drag`'s own header says why that is exact rather than lazy),
and there is no caret at all (`textsel` §1.2 — a caret promises an insertion
point, and there is nothing to insert). So the variant carries **nothing**,
where `Markup` and `Measure` each carry a kind. That is the smallest thing
this enum can be asked to hold and still be worth holding.

What the variant *buys* is two things at once. Without it, `canvas::textsel`
§3 gives a press its text meaning *"when the select tool is active and the
mode cannot select content"*, which yields Read ✓, Review ✓, **Edit ✗** — a
reviewer can sweep text and an editor cannot, and, worse, the three
text-markup controls drawn on Edit's Markup tab can **never enable**, because
`selection.text` is never true there. That is a live tension with
`RIBBON_IA.md` P3, which reserves greying for *temporarily* unavailable, and
it cannot be closed by hiding the controls, because a command lives on
exactly one tab and the Markup tab is in both Review and Edit. One variant
closes both.

### The reference applications DISAGREE here, and Inkscape wins

The standing instruction is to match Inkscape, Acrobat and SolidWorks, and
to say which won where they disagree. On this question they genuinely do:

* **Acrobat and SolidWorks resolve text-versus-object *contextually*, within
  one tool** — hover text, get an I-beam; hover an object, get an arrow.
* **Inkscape uses a separate Text tool**, distinct from its Selector.

**Inkscape wins, and the reason is not a head-count.** An object marquee over
*vector content* is a surface Acrobat does not have at all: its "objects" are
annotations and form fields, never the page's own path and text operators, so
its contextual answer is not an answer to this conflict — it never has the
conflict. The conflict exists only in the Inkscape-shaped mode, which is what
makes Inkscape's resolution the applicable one rather than merely the
outvoted one.

The concrete failure a contextual press would produce is the deciding
argument. In Edit the primary drag is the content marquee, and the commonest
gesture on a drawing sheet is a marquee over a *region* — which on any real
sheet contains text. Under a contextual rule that drag would mean "sweep
text" or "marquee objects" depending on whether the pixel under the button-
down happened to be inside a glyph's box, a distinction the operator cannot
see and cannot aim at. A tool makes the answer a thing they chose.

### Text EDITING is a separate admission, and it made the argument again

Selecting text and editing text are different features with different state.
The second is [`CanvasTool::TextEdit`], and it was admitted against this same
bar and against a sharper objection — that a caret in a re-laid-out box would
drag a whole subsystem's state through this type. That variant's own
documentation is where the argument is made and where the objection is
answered; it carries one [`TextEditKind`], and the draft, the caret, the
anchor and the original text live in `egui::Memory`. Whoever brings the next
authoring surface should have to make the argument again, in this file, in
the same place.

## Where the state lives, and why `egui::Memory` is right here when it was
wrong for the selection

`canvas/mod.rs`'s seam 1 records the selection being *moved out* of
`egui::Memory` because it is **document-scoped**: closing a document must
forget it, and `Memory` outlives documents. A tool is the opposite — it is
**application-scoped**, like the ribbon tab or the theme. An operator who
picks the hand tool, opens another drawing and finds themselves back in the
select tool would report that as a bug. So the tool stays in `Memory`
precisely *because* `Memory` outlives documents, which is the property that
disqualified it for the selection.

## Item notes

### `fn space_borrows_the_hand_and_releasing_returns_the_previous_tool`

The third case is the one that matters: releasing space returns to
`Select`, and it does so without anything having been stored, so there
is no restore step that can be skipped.

### `fn only_the_hand_pans_and_each_tool_paints_its_own_cursor`

The markup rows are the ones that matter now: a markup tool that
answered `true` to `pans_with_primary` would be handed a blank pointer
frame by `canvas::interact` and could never draw anything at all — a
tool that arms, shows a crosshair and does nothing, which is the exact
shape of an affordance that looks available and is inert.

### `fn the_text_tool_sweeps_rather_than_pans_and_says_so_with_the_pointer`

All three halves, because each has a distinct and plausible failure. A
text tool that answered `true` to `pans_with_primary` would be handed a
**blank** pointer frame by `canvas::interact` and could never sweep
anything at all — a tool that arms, shows an I-beam and does nothing,
which is the exact shape of an affordance that looks available and is
inert. A tool with no cursor would be indistinguishable from the select
tool it replaces, on a control whose entire visible effect is the pointer.
And an `is_text` that answered `true` for a *markup* tool would hand an
armed pen's press to the text gesture.

### `fn toggling_the_text_tool_arms_it_retires_it_and_takes_it_from_another_tool`

`arm_markup`'s two halves for a tool with no kind, and both matter for the
reason that test gives: a build that only ever armed would pass a test of
the first press alone, and the operator's complaint would be that the tool
cannot be put down.

The third case is the one `toggle_text`'s own docs argue: arriving from a
markup tool must **take** the text tool rather than dropping to Select,
or one press would mean "put the pen down" and a second one "pick up the
I-beam".

### `fn space_borrows_the_hand_out_of_the_text_tool_and_escape_leaves_it_alone`

The first is the property the derived-never-stored design exists for,
asserted for the new tool exactly as it is for markup above.

The second is a **deliberate absence**, asserted so that adding an Escape
rung later is a decision rather than an accident. `canvas::keys`' rung 3b
retires an armed markup or measure tool, and the text tool is
deliberately not on it: those two paint a crosshair promising a gesture
that *writes to the document*, while this one promises a selection, and —
the deciding half — Escape's rung 5 already means "clear the selection"
in this tool. A further press that silently moved the operator from
sweeping text to marqueeing objects would be a change of gesture they did
not ask for, on the key they pressed to clear something. See
`canvas::keys`' header.

### `fn the_text_tool_is_permitted_in_every_mode_where_a_pen_is_not`

The Edit row (`FULL`) is the one that would break the feature outright: a
capability check copied from the markup arm would fail on the frame the
operator entered the one mode this tool exists for. The Read row
(`NONE`) is the one that would break it quietly, by taking a reading tool
away from the reading mode.

Asserted beside the *markup* tool in the same loop, so this is a statement
about the difference rather than about the text tool alone: a build that
stopped retiring anything would pass the first half and fail the second.

### `fn the_cursor_precedence_runs_tool_then_gesture_then_grip`

This rule was four `if`s in the middle of `canvas::interact` and had no
test at all — it needed a window to reach. Moving it here is what makes
it assertable, and the rungs are asserted **against each other**: each
case supplies a lower rung that would answer differently, so a build
that consulted them in the wrong order fails rather than merely
producing *a* cursor.

### `fn arming_a_markup_kind_toggles_that_kind_and_switches_between_kinds`

Both halves, because a build that only armed would pass a test of the
first press alone — and the operator's complaint would be that the tool
cannot be put down.

### `fn disarming_markup_reports_whether_there_was_anything_to_disarm`

`false` with nothing armed is the load-bearing half: without it Escape
would be consumed by a tool that was not armed, and the selection ladder
would need two presses to leave a rung.

### `fn space_borrows_the_hand_out_of_the_markup_tool_and_returns_the_kind`

The property the whole "derived, never stored" design exists for,
asserted for the new tool: an operator drawing a rectangle who holds
space to reposition the page must get the rectangle tool back on
release, with its kind intact.

### `fn a_focused_text_field_keeps_the_space_bar`

Built against a real `TextEdit` for the same reason
`canvas::tests::a_focused_text_field_keeps_delete_for_itself` is:
`text_edit_focused()` resolves the focused id and looks for a
`TextEditState` under it, so a hand-requested focus on a bare id would
pass vacuously.

### `enum CanvasTool`

**Does a primary drag select, move the paper, or draw?** — the only question
the pan, marquee and markup paths need settled, and settling it here keeps
them from inventing three different answers.

Four variants, not nine: [`Self::Markup`] and [`Self::Measure`] each carry
**which** kind is armed rather than there being one variant per shape or per
dimension. See those variants' docs for the argument, and the module header
for what changed since this enum said it would stay at two.

### `fn pans_with_primary`

The whole branch, in one predicate, so the pan path and the
gesture-suppression path cannot disagree about which tool pans — a
disagreement whose symptom would be a drag that pans **and** marquees,
which is one of the two things this stage must not ship.

The markup tool answers `false`, which is what makes a markup drag reach
the gesture machine at all: `canvas::interact` hands that machine a
**blank** frame whenever this is `true`. Space-to-pan still works over
the markup tool, because [`resolve`] composes the held space bar *before*
this is asked — so a held space bar borrows the hand out of the markup
tool exactly as it does out of the select tool, and releasing it hands
the markup tool back with nothing stored and nothing to restore.

### `fn cursor`

`Grab` when the hand is available and `Grabbing` while it is closed, in
the direction every browser, CAD package and image editor uses. The
pair matters: the requirement is that the cursor *changes and changes
back*, and a single hand cursor for both states would leave an operator
unable to tell a hand tool that is working from one that has run out of
scroll range — the exact ambiguity the middle-drag path's own
`Grabbing` was added to remove.

`Select` returns `None` rather than `Default`: returning a cursor here
would overwrite the grip cursors that [`crate::canvas::handles`] sets
for the eight resize handles, and a resize grip that loses its cursor
is a grip nobody can find.

`Markup` returns `Crosshair` in **both** states, and the sameness is
deliberate where the hand's pair is deliberately different. The hand
needs to distinguish "available" from "closed" because a pan that has
run out of scroll range is otherwise indistinguishable from a pan that
is not working; a markup drag has no such failure — the band under the
pointer is the feedback, and a cursor that changed under it would
compete with the thing it is describing. What the crosshair says is
*"this canvas draws now"*, which is true from the moment the tool is
armed until it is retired, and returning it also **suppresses the grip
cursors** — correctly, because a markup drag over a selected object
draws a shape rather than resizing anything.

`Text` returns `CursorIcon::Text` in both states, on the same argument
the crosshair makes and with one extra consequence worth naming. The
I-beam is what Acrobat, Inkscape and SolidWorks all show over selectable
text, and [`cursor_for`]'s own note records why this shell would not
paint it on **hover** — answering *"is there a glyph under the pointer?"*
per frame is a hit test against the page's extraction on every frame the
pointer moves, paid on canvases nobody is selecting on. Armed, the
question does not arise: the tool is a statement about what the next drag
means, so the cursor is constant while it is armed and costs nothing.
That is precisely the "becomes free on the day a `CanvasTool::Text`
lands" this pair of comments anticipated.

It suppresses the grip cursors too, and here that is load-bearing rather
than incidental: in Edit a content selection can be on the page *while*
this tool is armed, and [`crate::canvas::gesture::press_kind`] gives the
armed tool the press. A grip that still showed its resize cursor would be
promising a gesture the press rule has already decided against — the
exact mismatch [`retire_forbidden`] exists to prevent at the other end of
a tool's life.

### `fn markup_kind`

The accessor `crate::app::PdfcerApp::conditions` needs in order to render
exactly one Markup button pressed, and the accessor
[`crate::canvas::gesture::press_kind`] needs in order to decide what a
press means. Both would otherwise write the same `if let` — which is how
a canvas ends up drawing one shape while the ribbon says another.

### `fn measure_kind`

[`Self::markup_kind`]'s twin, and it exists for the identical two
callers: `crate::app::PdfcerApp::conditions`, so exactly one Measure
button renders pressed, and
[`crate::canvas::gesture::press_kind`], so a click is offered to the
pick machines instead of to the selection.

### `fn place_kind`

[`Self::markup_kind`]'s and [`Self::measure_kind`]'s sibling, with the
same contract and one fewer caller: nothing publishes a `selected:`
condition for it, because a placement is armed from **inside a dialog**
and has no ribbon control to render pressed. `panels::tool::armed`
records that absence rather than inheriting it.

### `fn is_text`

[`Self::markup_kind`]'s and [`Self::measure_kind`]'s third sibling,
answering `bool` rather than `Option<Kind>` because [`Self::Text`] carries
no kind — see that variant's docs for why it carries nothing at all.

It exists for the same reason the other two do, and it has the same three
callers, which is what stops them writing three `matches!` that could
drift: [`crate::canvas::textsel::takes_the_press`], which decides what a
press means; `crate::app::PdfcerApp::conditions`, which decides whether the
ribbon control renders **pressed**; and [`crate::canvas::gesture::press_kind`],
which reads it through `takes_the_press` rather than directly, so the
drag's meaning and the click's routing cannot disagree.

Deliberately `selected`-agnostic: like its siblings it is a question about
a [`CanvasTool`] value, and *which* value — the chosen one or the one a
held space bar composes — is the caller's decision. [`active`] and
[`selected`] answer differently on purpose.

### `fn is_node`

True for [`Self::Node`] alone. Read by the paint pass — which draws every
anchor of the selected object while it is armed, rather than only after a
descent — and by the click router, which selects an anchor directly.

### `fn text_edit_kind`

[`Self::markup_kind`]'s and [`Self::measure_kind`]'s fourth sibling, with
the same three-caller contract: [`crate::canvas::gesture::press_kind`],
which decides that the press is a caret placement and not a marquee;
`crate::app::PdfcerApp::conditions`, so exactly one of the two Edit
controls renders pressed; and `canvas::interact`, which routes the
resulting click to `canvas::textedit::click`. Three `matches!` written
separately is how a canvas comes to place a caret while the ribbon says
Add text.

Note what it is **not** a sibling of: [`Self::is_text`]. That answers
*"is the text SWEEP armed"* and this answers *"is the text CARET armed"*,
and they are false at the same time and true at different times. They are
two questions with confusingly similar names, so each names the other
here — a reader who calls the wrong one gets a compile error only if the
return types differ, and they do not.
