# `shell::commands::mapping` — the single binding between a command id and
the value it names

Four pairs of functions, each pair one *forward* mapping from a canvas or
view value to the `&'static str` id that names it, and one *inverse* derived
from the forward one rather than written out a second time:

| value | forward | inverse | who reads which |
|---|---|---|---|
| [`crate::viewer::PageDisplay`] | [`page_display_command`] | [`page_display_for_command`] | `conditions` publishes the pressed radio position; `dispatch` turns a click into a mode |
| [`crate::app::actions::ViewChrome`] | [`chrome_command`] | [`chrome_for_command`] | the same pair, for three independent toggles rather than a radio |
| [`crate::canvas::markup::MarkupKind`] | [`markup_command`] | [`markup_for_command`] | `dispatch` arms the pen; `conditions` lights exactly one shape button |
| [`crate::canvas::measure::MeasureKind`] | [`measure_command`] | [`measure_for_command`] | the same pair, for the dimension tools |

## Why this is a file of its own

**R2** (no `.rs` file over 1,500 lines) forced a split when
`measure.finish` was registered and [`super`] reached 1,521 lines. The seam
is the one the file had already drawn for itself with a blank line and a
`` banner: everything before it answers *"what commands exist, with what
label, icon and predicate"*, and everything here answers *"which id names
this value, and which value does this id name"*.

They change for genuinely different reasons. A registration changes when the
ribbon gains a control or a tooltip is rewritten; a mapping changes only
when an **enum** the canvas owns gains a variant — and when that happens the
test that fails is in this file, iterating that enum's `ALL`, which is
exactly where a reader looking for "why does my new kind not arm anything"
should land.

## The forward direction is a `match`; the inverse is a search over it

Every inverse here is `ALL.iter().find(|k| forward(k) == id)`. That is not
an optimisation question — the lists are four to seven entries and the call
happens on an operator click, not per frame — it is a **correctness**
property: two hand-written tables can disagree, and one table plus a derived
search cannot. The failure the derivation removes is the worst kind
available here, because it is silent: a button that arms one tool while the
ribbon lights another.

## The guard arms in `app::dispatch` are tried in order, so these must not
overlap

`PdfcerApp::dispatch_command` has several arms of the shape
`id if …_for_command(id).is_some()`, and `match` takes the first that
matches. If a `measure.*` id ever answered to [`markup_for_command`] the
measure arm would still win by being written first and the defect would be
invisible; if a `markup.*` id answered to [`measure_for_command`] the markup
arm would be **swallowed** and four ribbon buttons would silently stop
arming anything. [`tests::every_measure_kind_has_a_registered_command`]
asserts the disjointness in both directions.

## Item notes

### `fn registry`

Every test below asserts against **this** rather than against the
mapping's own table, which is the difference between asserting that the
code agrees with itself and asserting that the control exists.

### `fn every_chrome_toggle_has_a_registered_command`

The twin of [`every_page_display_mode_has_a_registered_command`], and
it catches the same failure: a fourth toggle added to
[`crate::app::actions::ViewChrome`] with no registration would be a
piece of chrome no operator could reach, and nothing else in the suite
would notice. Asserted against the **live registry** rather than
against the mapping's own table, which is the difference between the
code agreeing with itself and the control existing.

### `fn every_markup_kind_has_a_registered_command`

The third of this family, and the one with the most room to go wrong,
because the kinds and the commands were built by different hands: the
canvas enumerates what the *gesture* can draw, the manifest enumerates
what `RIBBON_IA.md` §5.5 *names*, and those two sets are deliberately
different sizes today — ten names, four kinds. This asserts the four
are a genuine subset and reach real controls, not that the sets match.

The failure it exists to catch is a fifth kind added to `MarkupKind`
with no registration: a tool an operator could not arm, which is
precisely the class of half-built surface `panels`' header is about.
Asserted against the **live registry** rather than the mapping's own
table — the difference between the code agreeing with itself and the
control existing.

### `fn every_text_mark_kind_has_a_registered_command`

The third `markup.*` mapping and the one with the most room to go wrong,
because both families share the id prefix and both are matched in
`PdfcerApp::dispatch_command` by guard arms tried in order. The two
crossings have different symptoms and neither is diagnosable from a
screenshot:

* a **shape** id answering here would author a text markup instead of
  arming the pen — a Rectangle button that marks the selected words;
* one of **these** answering to [`markup_for_command`] would be swallowed
  by the arm below it, and Underline would do nothing at all.

Asserted against the **live registry** rather than the mapping's own
table, which is the difference between the code agreeing with itself and
the control existing.

### `fn every_measure_kind_has_a_registered_command`

[`tests::every_markup_kind_has_a_registered_command`]'s twin, catching
the same failure: a fifth `MeasureKind` added with no registration is a
tool an operator cannot arm. Asserted against the **live registry**,
which is the difference between the code agreeing with itself and the
control existing.

The guard-arm overlap check at the end matters more here than it did
for markup, because there are now **two** guard arms in
`PdfcerApp::dispatch_command` matching on `*_for_command(id).is_some()`
and they are tried in order. If a measure id ever answered to
`markup_for_command`, the measure arm would still win by being first and
the defect would be invisible; if a markup id answered to
`measure_for_command`, the markup arm would be **swallowed** and four
ribbon buttons would silently stop arming anything.

### `fn finish_is_registered_and_is_not_a_tool`

If it ever answered to [`measure_for_command`] the guard arm in
`PdfcerApp::dispatch_command` would claim it before its own arm, and
`arm_measure`'s same-kind-retires rule would turn a press of Finish into
*putting the tool down* — the exact opposite of what the control says it
does, with the pick set left standing and nothing authored. The failure
would look like a Finish button that does nothing, which is the hardest
kind to diagnose from a screenshot.

### `fn every_page_display_mode_has_a_registered_command`

Both directions, against the **live registry** rather than against the
mapping's own table — which is the difference between asserting the
code agrees with itself and asserting that the control exists. The
failure this catches is a fifth mode added to the enum with no
registration: the ribbon would draw three buttons, the fourth would be
unreachable, and nothing else in the suite would notice.

### `fn page_display_command`

One binding between [`crate::viewer::PageDisplay`] and the ribbon, written
down once. The two directions are used by different surfaces and would drift
apart if each spelled the mapping for itself:

* `crate::app::dispatch` turns an invoked command into a mode;
* `PdfcerApp::conditions` turns the active mode into the `selected:`
  condition that makes its ribbon button render pressed.

It lives here rather than on the enum for the reason the enum's own
`id`/`from_id` pair lives *there*: `viewer` must not know what a ribbon is.
`viewer::PageDisplay::id` is the **on-disk** spelling and this is the
**command** spelling, and keeping them separate is what lets either change
without silently rewriting the other's files.

[`tests::every_page_display_mode_has_a_registered_command`] asserts both
directions against the live registry, so a fifth mode that is added and not
registered fails the suite rather than becoming a mode with no control.

### `fn chrome_command`

Exactly the shape [`page_display_command`] has, and here for exactly the
same reasons: two surfaces need the mapping in opposite directions —
`crate::app::dispatch` turns an invoked command into a
[`crate::app::actions::ViewChrome`], and `PdfcerApp::conditions` turns each
toggle's state into the `selected:` condition that renders its button
pressed — and a mapping spelled twice is a mapping that drifts.

The difference from the page-display pair is that these are
**independent toggles rather than a radio**: all, none or any subset may
be on at once, so `conditions` publishes between zero and six of these
conditions where it publishes exactly one page-display condition. That is
the whole of what makes them read as a row of switches instead of one
multi-position control.

### `fn markup_command`

The **single** binding between a `markup.*` id and a
[`crate::canvas::markup::MarkupKind`], in the shape [`chrome_command`]
established and for the same reason: a match here plus a derived inverse
cannot disagree, where two hand-written tables can.

## Why these seven and not the ten `RIBBON_IA.md` §5.5 names

[`crate::canvas::markup::MarkupKind`] enumerates the kinds a `markup.*`
command **arms the canvas tool with** — nothing else. What is still outside
it, and each for its own reason:


**This said "these four" until 2026-08-14**, and the sentence it rested on
was *"polygon, polyline and ink are not drag-shaped"*. That was true and it
stopped being a reason the day those gestures were built: two of the three
are now clicked (`canvas::markup::vertex`) and one is dragged freehand
(`canvas::markup::ink`). The wording is kept in this note rather than
deleted, because the boundary it was proxying for — *a variant nothing can
arm is a dead state* — is the real rule and is unchanged. See
`canvas::markup`'s header, where the boundary is restated as the property the
tests below actually assert.

Declaring the remaining kinds here early would put dead arms in a type whose
job is to say what the tool is doing — the same argument the old shell made
about its own tool enum, applied at the gesture boundary. They arrive with
the gestures that can draw them.

### `fn text_mark_command`

[`markup_command`]'s sibling and deliberately a separate pair, because the
two families do different things to a press: a `markup.*` shape id *arms a
tool*, and one of these *authors an annotation immediately* from the text
selection already on the document. See [`crate::canvas::markup::text`]'s
header §1 for the interaction decision that makes them different, and §3 for
why the three kinds are not [`crate::canvas::markup::MarkupKind`] variants.

## The disjointness is load-bearing, not tidy

All six ids begin `markup.`, and `app::dispatch` matches both families with
guard arms of the shape `id if …_for_command(id).is_some()`, tried in order.
If a shape id ever answered here, pressing Rectangle would author an
annotation over whatever text was selected and never arm the pen; if one of
these answered to [`markup_for_command`], pressing Underline would arm a
**shape tool** — `arm_markup` would take a `MarkupKind` from an id that does
not name one, which it cannot, so the arm would swallow the command and
Underline would do nothing at all. Both directions are asserted by
[`tests::every_text_mark_kind_has_a_registered_command`].

## Why Highlight is not here

`markup.highlight` authors the same `MarkupSpec::TextMarkup` these three do,
with `TextMarkupKind::Highlight` — and it is a **drag** across an area, not a
mark on a selection, so it stays a `MarkupKind`. That is a genuine seam
rather than an inconsistency: a highlight over an image or a title-block cell
is a thing operators want and a text markup cannot express. See
`canvas::markup::text` §3, which also records what it would take to offer
*both* and why that is the operator's taxonomy decision rather than this
file's.

### `fn measure_command`

The **single** binding between a `measure.*` id and a
[`crate::canvas::measure::MeasureKind`] — [`markup_command`]'s twin, in the
same shape and for the same reason: a match here plus a derived inverse
cannot disagree, where two hand-written tables can.

## Which are absent, and why each

`measure.aligned` is a *constraint* on a linear pick rather than a tool
(the old shell's `LinearPick` carries an `AxisConstraint`), so it belongs on
a property control, not here. `measure.angular`, `measure.area`,
`measure.distance`, `measure.perimeter` and `measure.count` need engine
verbs that do not exist — `RIBBON_IA.md` §5.6 marks them **N** — and
`measure.calibrate` is a second entry path into the scale dialog rather than
a fifth tool. All remain in [`super::manifest::PLANNED`], which is where an
absent command is supposed to be.

`measure.set_scale` is registered and is deliberately **not** a kind: it
changes what measurements are read against rather than placing one. (This
sentence used to end *"and its dialog does not exist yet"*. It does —
`crate::dialogs::scale`, since 2026-08-17 — and the reason for not being a
kind never depended on that clause.)

`measure.finish` is registered and is not a kind either, for a sharper
reason: it does not *arm* anything. It **ends** the radius/diameter
gesture, and it is dispatched through its own arm to
[`crate::canvas::measure::finish`] rather than through `arm_measure`. If it
answered here, pressing Finish would toggle the tool off — see
`crate::canvas::tool::arm_measure`'s same-kind-retires rule — which is the
opposite of what it is for.


Until 2026-08-14 it carried a paragraph explaining that `Circular` was
absent because *"the gesture has no natural end, and the only place to say
so was an accept box decision 024 retired"*. That was accurate while it
stood and it is now false: the operator decided the tool should have **two**
endings, neither of them a floating box, and both are built. The paragraph
is not merely deleted — a reader who finds `measure.radius_diameter`
reaching a real tool and remembers the old note should be able to see what
replaced it. What shipped is a **double-click** on the canvas and this
command, both routed through one commit path
(`canvas::measure::circular::commit`), with the ribbon control
enabled only while there is a non-degenerate fit to commit.

### `fn form_for_command`

A NAMED function taking `id`, deliberately, rather than an inline closure
at the dispatch site. `shell::commands::reach` *reads* `app/dispatch.rs` to
prove every registered command is routed, and it can follow a call like
`form_for_command(id)` while a closure is opaque to it — the first version
of this arm used `.iter().any(|k| ...)` and the reader rejected the whole
dispatcher as unreadable rather than guessing.

So the convention `markup_for_command` established is not style: it is what
keeps the routing table machine-checkable.
