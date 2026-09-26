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
