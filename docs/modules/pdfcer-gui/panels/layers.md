# `panels::layers` — the document's optional-content groups

Salvaged from the old shell's `panels_structure.rs`. The **report** came
across whole at S3; the **checkbox did not**, and at S4 it is back, with
the `/RBGroups` radio behaviour and the Reset control that travelled with
it. This module's header is therefore in two halves: what the panel says,
and the history of the control it says it about.

# What it shows that a name cannot

Whether a reader opening this document with no interaction would DRAW
each layer. A "Confidential" watermark that is off by default is a
different document from one where it is on, and the two are
indistinguishable by name.

That value comes from `pdfcer-core`'s own `Layer::visible_by_default` —
the initial `/D` state, which is the same thing the renderer resolves —
so the panel cannot say "on" about content the page hides.

Six per-layer facts are surfaced as tooltips, and each exists because
without it the operator's only available reading is *"pdfcer got it
wrong"*:

| Fact | Why it must be said |
|---|---|
| the operator has changed this row | Names the document's own state, so the divergence can be seen without resetting to find out. |
| `/Intent` excludes `View` (§8.11.2.3) | The group does not participate in visibility **under the document's own configuration**, so its state in the document's `/OFF` array has no effect on what a reader draws — but a switch here does affect it, for the reason in [`crate::text::panels::layer_design_intent_tooltip`]'s docs. A layer marked visible that the file's own `/OFF` names looks like a defect otherwise. |
| `/Locked` (Table 101) | *"the UI shall not allow the visibility state to be changed"* — an interface lock, not a guarantee, since the specification's own table blesses JavaScript and `/AS` bypass. |
| not in the default configuration | Page content uses the layer and the document never registered it. Some readers will not show it at all. |
| in an `/RBGroups` radio group | At most one member visible at a time, so this layer's state is not independent of its siblings'. |
| that radio group contains a **locked** member | pdfcer will not switch that sibling off, so this group can end up with two members showing. See "`DA-A8`" below. |

§8.11.4.4's **auto-managed** groups get a line of their own, above the
list: some states are not the document's to state — a viewer recomputes
them from the magnification — so a zoom-banded layer can read "shown"
here while its content is off the page. Said out loud rather than left to
be discovered as a defect.

# The visibility control, and the three preconditions it waited on

**All three now hold.** The section is kept rather than deleted because
it is a worked example of `crate::render::worker`'s rule — *"the key
ships in the same commit as its control"* — actually preventing a defect
rather than merely being stated, and because the next person to blame a
control for redrawing nothing should be able to read how this one was
diagnosed.

The old panel let an operator toggle a layer for the session, honouring
`/RBGroups` radio semantics, with a Reset that returned to the document's
own configuration. All of that machinery is *good*. Three things had to be
true for the tick to change a pixel:

1. **The renderer must accept an override.** True, and always was —
   [`pdfcer_render::LayerVisibility`] exists and `pdfcer-render` honours
   content-stream `/OC`.
2. **The render worker's cache key must vary with it.** **True as of
   S4.** `crate::render::worker`'s `RenderKey` carries
   `layers_generation`; `crate::app::state::OpenDoc` carries the override
   the renderer takes, the counter that keys it, and the mutators a
   control calls — `hidden_layers`, `set_hidden_layers`,
   `set_layer_visible`, `reset_layers`. `OpenDoc::render_key` is compared
   against the cached texture's own key every frame, so a change to the
   override re-rasterizes at once rather than waiting out the zoom
   debounce. [`tests::the_render_key_no_longer_blocks_a_layer_toggle`]
   pins exactly that, against this panel's own data path.
3. **An action must carry the toggle.** **True as of S4**, and the last
   to land. [`crate::app::actions::Action`] gained `SetLayerVisible`,
   `ResetLayers` and `ToggleAnnotations`, each implemented in
   `PdfcerApp::apply`. Before that a panel body — handed `&OpenDoc`, a
   *shared* reference, precisely so that it cannot mutate — had nowhere
   to send a click.

## Why the last precondition was not simply worked around

Kept because the shortcut is still available and still wrong.

`OpenDoc` holds two caches behind a `RefCell`, so the shape of a shortcut
is visible from here: put the override behind one too, and the panel could
toggle it through the shared reference it already has.

That would be wrong, and the difference is not stylistic. The `RefCell`s
hold **derived caches** — filling one changes nothing an observer could
see, which is why a shared reference may do it. Layer visibility decides
**what appears on the page**. Routing it around the action funnel would
forfeit the fourth property `crate::app::actions`' header claims for that
funnel — *"every state change is greppable: what can change the zoom? has
a complete answer"* — and it would do so for the one class of state where
an operator can see the result and not find the cause. It would also put
the change outside the one place an undo log will be written.

## The pair that must move together, and the record of it not doing so

The old panel's own header records this exact doc comment being **wrong
for three commits** in the opposite direction: it said the checkbox could
not exist, after the commit that added it. It was found by a person
reading the file, not by any check — nothing compiles a doc comment
against the behaviour it describes.

So the control and the sentence that describes it are stated together, and
`the_layers_note_says_a_toggle_changes_the_view_and_not_the_document` in
`crate::text::panels` is the check that at least the sentence cannot
silently revert to describing a program that does not exist. That test
replaced `the_layers_note_states_that_switching_is_unavailable` **in this
commit**, which is the discipline working in the other direction: the
assertion that pinned the absence came out with the absence.

# `/RBGroups`: how the radio behaviour survives an action funnel

Table 101's `/RBGroups` are "radio button" groups — at most one member ON
— and `pdfcer_core::layers` hands the panel everything needed to honour
them *before the first click*: [`pdfcer_core::layers::Layer::radio_group`]
is an index into [`pdfcer_core::layers::Layers::radio_groups`], which
carries the full membership.

The interesting part is that [`crate::app::state::OpenDoc::set_layer_visible`]
deliberately does **not** implement radio semantics, and says so: the
sibling list is *"the control's reading, not this type's"*, and a
half-implementation there would be a second visibility algebra beside the
engine's. So the panel composes the whole gesture itself — and it does so
**as a list of actions**, one per layer that has to move, rather than by
reaching for a `SetHiddenLayers` variant that carries a set.

That composes correctly for a reason worth stating, because it is the
only thing making the simple variant sufficient: `apply_actions` applies
in the order raised, and each `set_layer_visible` recomputes from
`hidden_layers()` — the *current* answer, including the effect of the
actions applied a moment ago in the same frame. So N actions in one frame
settle to exactly the set one composed call would have produced, they bump
the generation N times (which costs one re-render, since the raster
settles once per frame), and the funnel keeps its grep property: every
layer that moved has its own `Action` naming it.

[`toggle_actions`] is that composition, extracted as a pure function so
the radio rule is tested against real fixtures without an egui context.

## Turning one OFF does not turn a sibling ON

"At most one" permits none. Picking a replacement would be pdfcer choosing
which alternate the operator meant, which is exactly the class of
invention rule 4 forbids.

## `DA-A8`: a locked layer inside a radio group

`pdfcer_core::layers` names this as a genuine gap in the standard and hands
the decision here verbatim: a locked group's state *"cannot be changed
through the user interface"*, while a sibling being turned ON means all
others *"shall be turned OFF"*. Both clauses address the **user
interface**, so nothing in the specification breaks the tie.

**pdfcer lets the lock win**: [`toggle_actions`] skips a locked sibling, so
a group can end up showing two members. The full argument is on
[`crate::text::panels::layer_radio_locked_sibling_tooltip`], which is the
disclosure — in one line, the alternative would switch off a locked layer
as a side effect of clicking a *different* row, which is a lock bypass
nobody watching the screen would see, and this way the violation is on the
page where the operator already is.

# Reset means "the document's own default", not "show everything"

[`crate::app::actions::Action::ResetLayers`] drops the override, which
restores `/D` (§8.11.4.3). It is emphatically **not**
`set_hidden_layers(BTreeSet::new())`, which would reveal every layer the
document turns off — on a drawing with a "Confidential" watermark that is
a disclosure event rather than a cosmetic one. The distinction is core API
trap T-12.9 and it is the whole reason
[`crate::text::panels::layers_reset_tooltip`] names what it returns *to*.

The control is drawn **only when something differs**, which is the old
shell's decision kept: a Reset sitting there permanently implies a change
that has not happened (`RIBBON_IA.md` P3 — an unavailable capability
renders nothing).

"Differs" is computed by comparing [`crate::app::state::OpenDoc::hidden_layers`]
against `pdfcer_core::annot::optional_content_default_off`, **not** by
counting clicks the way the old shell did. That is a deliberate
improvement: a layer switched off and back on again agrees with the
document, and an operator asked "how many did you change?" would say
none. `OpenDoc` exposes no "is an override in force?" predicate, so this
is also the only answer reachable from a panel — and it happens to be the
better one.

# A note on `/Locked` rows

Drawn as a **disabled** control, not as an absent one, which is the old
shell's call and survives review: the widget is a *state display* that
happens not to be interactive, not a stub for a capability the build
lacks. Every other row has a tick, and a locked row with nothing where the
tick goes reads as a rendering fault.

One defect from the salvage source is fixed here: it attached the
explanation with `on_hover_text` alone, and **egui does not show the
ordinary hover text of a disabled widget**. So the one row whose whole
problem is that it looks broken was silent about why. Both are attached
now, the same fix [`crate::panels::bookmarks`] made for its disabled rows.

## Item notes

### `fn row_name`

An undeclared name shows as a placeholder, never as an invented one.
`/Name` is Required (Table 98), so its absence is a real malformation and
a synthesised "Layer 3" would disguise it as data from the file.

# Why this is a function and not two lines at the row

It was two lines at the row until the search landed. The search matches
**the name the row shows** — [`search`]'s Decision 1 — and the only way to
hold that claim is for the row and the predicate to call the same
function. Two copies of the same three-line `if` would compile, pass every
test written about either half, and leave a layer with no `/Name` drawn as
its placeholder while being searchable only by the empty string: a row on
screen that nothing the operator can type will find.

The general form is `DEFECTS.md` D5's — *the same concept implemented in
two places, and the copies drift* — arriving in a feature small enough
that it would have been assumed safe.

### `fn is_locked`

A linear scan rather than a map, deliberately. It runs only when a radio
group is being examined, over at most [`pdfcer_core::layers::MAX_LAYERS`]
rows, and a map built per frame to serve a handful of lookups would cost
more than it saved. Measure before trading back.

A member id that is not in `read.layers` answers `false` — it is a
dangling reference inside `/RBGroups`, which cannot be locked because it
is not a group. Failing closed here would silently refuse a legal toggle.

### `fn toggle_actions`

# What it does

- Always ends with `SetLayerVisible { group: id, visible }`.
- When `visible` is `true` **and** `id` is in an `/RBGroups` array, it is
  preceded by one `SetLayerVisible { …, visible: false }` per **unlocked**
  sibling. Table 101: at most one member of a radio group is ON.

# What it deliberately does NOT do

- **Turning a layer off does not turn a sibling on.** "At most one"
  permits none, and choosing a replacement would be pdfcer deciding which
  alternate the operator meant.
- **It does not switch off a locked sibling** — `DA-A8`, argued in the
  module docs and disclosed by
  [`crate::text::panels::layer_radio_locked_sibling_tooltip`].
- **It does not chase a group's *other* radio arrays.**
  `Layer::radio_group` reports the **first** array a group belongs to and
  `LayerDiagnostics::overlapping_radio_groups` counts the rest, because
  the constraints are not jointly satisfiable and the standard is
  permanently silent on the case (`DA-N1`). Honouring the first array is
  the engine's own reported answer; inventing a resolution for the others
  would be pdfcer deciding something ISO declined to.

# Why a `Vec<Action>` rather than one action carrying a set

See the module docs: they compose correctly because `apply` runs them in
order and each one recomputes from the *current* hidden set, and keeping
one `Action` per layer that moved is what keeps "what changed this layer?"
answerable by grep.

The clicked layer goes **last** so that a group whose array (legally)
lists the clicked group among its own members cannot end with the
sibling sweep switching off the very layer being switched on.

### `fn row_caveats`

A pure function over one [`pdfcer_core::layers::Layer`] and the effective
state, so the *set* of caveats a row carries is testable without an egui
context — which matters because each one exists to stop an operator
concluding "pdfcer got it wrong", and a row that silently loses its
explanation looks exactly like a row that never needed one.

Order is meaningful: the operator's own change comes first, because if
they changed this row that is the explanation they are looking for.

### `trait RadioGroupLookup`

[`pdfcer_core::layers::Layer::radio_group`] is an *index* into
[`pdfcer_core::layers::Layers::radio_groups`] rather than a member list, so
every use of it is this two-step. Written once, in a trait, so the two
places that need it (the tooltip decision and the toggle composition)
cannot come to disagree about which array a layer belongs to.

### `fn settle`

Test-only, and it exists so the radio tests can assert on the *outcome* of
a click rather than on the shape of the action list. Mirrors
`OpenDoc::set_layer_visible`'s arithmetic exactly — insert to hide, remove
to show — which is the thing being modelled.

### `fn a_composed_click_changes_both_the_hidden_set_and_the_render_key`

The end-to-end statement the other tests only approach: compose the
click the way [`super::body`] does, hand the result to the real
[`crate::app::actions::Action`] machinery via the mutator `apply`
calls, and assert the effective hidden set moved AND the render key
with it.

Without this, every piece could be individually correct and the panel
still inert — which is precisely the S3 state, and it took a person
reading the file to notice.

### `fn a_reset_restores_the_document_rather_than_revealing_everything`

Core API trap T-12.9 in one assertion, on a fixture that actually
declares a layer off. `Action::ResetLayers` maps to
`OpenDoc::reset_layers`, and the failure it guards against is a
plausible one — `set_hidden_layers(BTreeSet::new())` reads like
"clear the override" and means "reveal every layer the document
turns off", which on a drawing with a "Confidential" watermark is a
disclosure event.

### `fn the_reset_control_appears_exactly_when_the_view_differs_from_the_document`

Pins the predicate [`super::body`] uses, since the control's presence
is the only signal that an override is in force at all — `OpenDoc`
exposes no "is it overridden?" accessor.

The second half is the part the old shell got subtly wrong: it
counted *clicks*, so a layer switched off and back on left the panel
claiming a difference that no longer existed. Comparing sets says
what an operator would say.

### `fn turning_on_a_radio_member_turns_its_unlocked_siblings_off`

Table 101's whole content, against the fixture built for it. The
failure this prevents is not cosmetic: on a CAD drawing the members of
an `/RBGroups` array are mutually exclusive alternates, so two of them
on means two title blocks painted over each other, and the operator
has no way to know pdfcer did that rather than the document.

### `fn a_locked_radio_sibling_is_never_switched_off_by_a_click_elsewhere`

pdfcer's answer to a conflict the standard leaves open, asserted rather
than merely written down — see the module docs for why the lock wins.

Note what it asserts: not that the group ends up legal, but that no
action *names* the locked layer. A click on one row must never change
a layer the document told the interface not to touch, and the failure
mode is a silent one: it would look like the radio rule working.

### `fn turning_a_radio_member_off_leaves_its_siblings_alone`

"At most one" permits none. Choosing a replacement would be pdfcer
deciding which alternate the operator meant — the class of invention
rule 4 forbids — and it would do so at the exact moment the operator
asked to see *less*.

### `fn a_click_never_reaches_outside_the_radio_array_core_reported`

`DA-N1`: a group may legally appear in more than one `/RBGroups`
array, the standard never says what a reader does with it, and the
constraints are not jointly satisfiable. `pdfcer_core` answers with
"the first array, plus a count of the overlaps", and this panel
carries that answer through rather than inventing a resolution.

The fixture is built for exactly this — two inner arrays sharing a
member. The invariant asserted is the strong, general one: **no click
ever names a layer outside the array core reported for it**, checked
for every clickable row rather than for a hand-picked one. A version
that hunted for the shared member and asserted about that row alone
would pass on a fixture whose shared member happened to be the locked
one, and prove nothing.

### `fn a_layers_state_is_carried_by_words_not_by_a_cue`

R84 — never colour alone. With the checkbox back these are no longer
the *only* state cue, which weakens the argument not at all: a tick is
a glyph, so a panel whose state was carried by the tick alone would be
exactly the colour-only failure R84 names, one substitution along.

### `fn each_per_layer_caveat_explains_a_different_surprise`

Eight of them now: the six per-layer caveats, both arms of the
override tooltip, and the ordinary toggle tooltip they share a row
with. Each exists because without it the operator's only available
reading of a surprising row is "pdfcer got it wrong". Two that read
alike would send them looking for the wrong cause — and the
design-intent one in particular explains a row that *contradicts the
file's own `/OFF` array*, which is the most alarming thing this panel
can show.

### `fn only_a_diverging_row_says_the_operator_changed_it`

The caveat set is what the operator reads to find out why a row looks
wrong, so a row that explains something that did not happen is as bad
as one that explains nothing.

Driven off `Layer::visible_by_default` rather than off the hidden set,
because that is the field [`super::row_caveats`] actually branches on
— a test that reconstructs the same answer by another route is one
more thing that can disagree.

### `fn the_locked_sibling_warning_is_not_shown_to_every_radio_row`

A blanket warning on every radio row would train the operator to
ignore it, and the row it matters on is the one where two members of a
mutually exclusive group can end up showing at once.

### `fn an_unnamed_layer_is_not_given_an_invented_name`

`/Name` is Required, so its absence is a real malformation. A
synthesised "Layer 3" would disguise a defect in the file as data
from it.

### `fn layer_name_for`

`None` when the document's registered groups contain no such id — an OCMD
(§8.11.2.2), or an OCG page content refers to and `/OCProperties` never
listed. A caller must say something *different* in that case, never
`on layer ""`: an empty pair of quotes is the placeholder R9 forbids, and
[`crate::text::panels::layers::layer_clause`] has words for it.

# Why the status bar comes here rather than reading `/Name` itself

[`row_name`]'s own header states the rule: **one spelling of what a layer
is called.** It was written when the search needed to match what the row
showed, and the reason generalises to every second surface. A bar that read
`Layer::name` directly would print the empty string for an undeclared
`/Name` where the panel prints its placeholder — the same layer, two names,
on two surfaces the operator sees at once.

`DEFECTS.md` D5 in a feature small enough to have been assumed safe. That
is the second time this function has absorbed a would-be copy.
