# `canvas::pick` — WHAT a click is allowed to land on

## The question this module answers, and the one it deliberately does not

Every press on the page eventually asks two separate questions, and until
this module existed the shell only had a vocabulary for the second:

| question | answered by |
|---|---|
| **May a click land on this *class* of thing at all?** | here — [`PickFilter`] |
| Where, in the stack of things it *may* land on, does this particular click land? | [`crate::canvas::input::probe`], [`crate::canvas::selection::annot`] |

Those look similar and are not. The second is geometry: tolerance, paint
order, distance-to-segment, topmost-wins. The first is **operator intent**,
it does not vary with the pointer, and it is exactly the thing a CAD
package parks permanently on screen so it can be glanced at rather than
remembered.

## Why this replaced a pair of ribbon buttons, in the operator's words


> *"On the bottom bar I want a filter menu that pops up with all the
> options of what to enable selecting of — text, points, lines, etc — all
> the object types … This is to replace the wonky content edit text and
> edit objects menu at the top."*

The diagnosis of *wonky* is worth carrying in the source, because it is a
statement about gesture design rather than about two buttons. Edit ▸
Content asks the operator to **declare an intention before pointing at
anything** — *I am now editing text* — and the hit test then obeys the
declaration rather than the drawing. Three consequences follow, all of them
reported:

1. The same click on the same pixel means different things depending on a
   control the operator is not looking at while they click.
2. Making a class of thing reachable costs a mode change plus two levels of
   ribbon travel — `FEATURES.md` records the measured ritual for typing one
   character as **four steps**.
3. The state is invisible at the moment it matters. A ribbon button pressed
   thirty seconds ago is not on screen while you aim.

A filter on the status bar has none of those properties: it is always
visible, it is one click from anywhere, and it says what it is doing while
you do it.

## The invariant: a filter is SUBTRACTIVE, always

**A [`PickFilter`] can only ever take candidates away. It can never make
something pickable that was not pickable without it.**

This is the most important property here and it is worth being explicit
about why it was chosen, because the obvious alternative is tempting and
wrong.

The tempting version: *"Nodes ON means a click lands directly on the
nearest node, without descending into the object first."* That would be a
real convenience, and it would also re-open a defect this project already
measured and closed. One CAD export in the fixture set holds **6,681
anchors in a single path object** and **1,194 subpaths**; offering all of
them to every press is what made the old ungated gesture unpredictable,
because the nearest anchor to a press routinely belongs to a subpath the
operator was not pointing at, with nothing drawn beforehand to say which.
[`crate::canvas::selection::SelectionLevel`] exists to scope that, and
[`crate::canvas::input::probe`] is built around the scoping.

So the ladder is left exactly as it is, and the filter sits *outside* it:
it decides which rungs and which classes are **eligible**, and the existing
geometry decides which eligible thing wins. Direct one-click node picking
already has a home — the Node tool, `A`, which calls
[`crate::canvas::selection::SelectionState::click_direct`] and skips the
descent ritual by design. The filter governs whether that tool may pick,
not whether every press behaves like it.

Two properties fall out of subtractiveness, and both are load-bearing:

- **`PickFilter::default()` reproduces today's behaviour exactly**, so R6
  ("nothing regresses") holds by construction rather than by testing every
  path twice. The default is *everything the shell can currently pick*.
- **The filter can never contradict a capability.** It is an `AND`, not an
  override — see the next section.

## The filter sits ABOVE the mode, which is not the same as replacing it

O17 is explicit: *"In all three modes the filter is authoritative. A class
switched off in the filter is not selectable in Read, not selectable in
Review, not selectable in Edit."*

That is a statement about one direction only. The composition is:

```text
pickable(class) = capability_allows(class, mode) && filter.allows(class)
```

**Both must be true.** Switching a class ON in the filter does not grant
Read mode the ability to edit content, and nothing here can. Capabilities
(`crate::app::modes::capability::Capabilities`) remain the mode's answer to
*what may be authored*; the filter is the operator's answer to *what I am
currently interested in pointing at*. They are different questions with
different owners, and collapsing them is how a filter turns into a hole in
the mode system.

## Why the class list is derived and not invented

Every variant of [`PickClass`] corresponds to something the existing hit
test can already **distinguish**. That constraint is what keeps the popup
honest: a row the operator can switch off but which nothing consults is a
lie told once per session, and a class the hit test can separate but which
has no row is a thing the operator cannot reach.

| [`PickClass`] | derived from | distinguished by |
|---|---|---|
| [`PickClass::Text`] | `VectorObject::Text` | `panels::objects::summary::object_kind` |
| [`PickClass::Path`] | `VectorObject::Path` | same |
| [`PickClass::Image`] | `ImageSource::Inline` / `ImageSource::XObject` | same |
| [`PickClass::FormXObject`] | `ImageSource::Form` | same |
| [`PickClass::Part`] | the `Part` rung | `ObjectModelProvider::part_kind` |
| [`PickClass::Node`] | the `Node` rung | `CanvasTargetProvider::nearest_node` |
| [`PickClass::Markup`] | `AnnotKind::Markup` | `canvas::selection::annot` |
| [`PickClass::CeDimension`] | `AnnotKind::CeDimension` | same |
| [`PickClass::FormField`] | `/Widget` annotations | `annot::selectable_on`'s exclusions |
| [`PickClass::Link`] | `/Link` annotations | same |
| [`PickClass::Characters`] | the character sweep | `canvas::textsel` |

### Two rows are RUNGS, not object kinds, and they belong here anyway

`Part` and `Node` are levels of
[`crate::canvas::selection::SelectionLevel`], not variants of any object
enum. Mixing them into one list with `Text` and `Image` is a category error
on the implementation's terms and is nonetheless correct on the operator's,
which is the side that matters for a control they read.

From the pointing end, *"can I click a corner point?"* is the same shape of
question as *"can I click a piece of text?"* — both are *"is this kind of
thing live right now"*. The operator asked for them in one breath and in
one list: *"text, points, lines, etc"*. Splitting them across two popups to
honour an internal distinction would be the shell explaining its own
architecture to somebody who is trying to click a corner.

### One class is OFF by default, and it is not an oversight

[`PickClass::Link`]. `annot::selectable_on` excludes `/Link` from selection
today, so there is no path by which a link can be picked, and a filter row
defaulting to ON would claim a capability that does not exist. It has a row
because links are a thing on the page the operator can see and will
eventually want to reach; it defaults OFF because subtractiveness means the
default must describe what the shell *does*, not what it should.

When link picking lands, the default flips here and nowhere else.

### Four more classes are INERT, and their tooltips do not say so

[`PickClass::Markup`], [`PickClass::CeDimension`], [`PickClass::FormField`]
and [`PickClass::Characters`] are read by nothing on the pick path, so
switching one off changes no click. [`PickFilter::allows`] is consulted at
exactly three points — the `Part` rung, the `Node` rung, and the object
class of the topmost hit, all in [`crate::canvas::input`]. The annotation
pick (`selection::annot::under_pointer`) is called with no filter argument
at all, and the character sweep's gate (`canvas::textsel::gate`) is decided
by the armed tool and the mode's capabilities alone. Unlike
[`PickClass::Link`], whose tooltip is honest about being inert, these four
promise that clicks will pass through. `DEFECTS.md` D49.

The rule: **a row is inert until an `allows` call reads it.** Wire the
consumer in the same change that adds the row, or the popup becomes the
*"visible control, silently inert"* failure it exists to replace.

## What this module does NOT do

It never draws, never touches egui, never reads a pointer, never reaches a
document, and never decides which class won a click. It is a set of
booleans over a closed enum, plus the two mappings that connect that enum
to the classifiers that already exist. Every claim in this header is
therefore assertable in a unit test rather than hoped for in a running
window — which, per R1, is the *floor*, not the ceiling: the popup that
drives this has to be driven before any of it counts as working.
