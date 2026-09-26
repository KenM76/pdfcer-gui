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

## ★ Why this is a file of its own

**R2** (no `.rs` file over 1,500 lines) forced a split when
`measure.finish` was registered and [`super`] reached 1,521 lines. The seam
is the one the file had already drawn for itself with a blank line and a
`★` banner: everything before it answers *"what commands exist, with what
label, icon and predicate"*, and everything here answers *"which id names
this value, and which value does this id name"*.

They change for genuinely different reasons. A registration changes when the
ribbon gains a control or a tooltip is rewritten; a mapping changes only
when an **enum** the canvas owns gains a variant — and when that happens the
test that fails is in this file, iterating that enum's `ALL`, which is
exactly where a reader looking for "why does my new kind not arm anything"
should land.

## ★ The forward direction is a `match`; the inverse is a search over it

Every inverse here is `ALL.iter().find(|k| forward(k) == id)`. That is not
an optimisation question — the lists are four to seven entries and the call
happens on an operator click, not per frame — it is a **correctness**
property: two hand-written tables can disagree, and one table plus a derived
search cannot. The failure the derivation removes is the worst kind
available here, because it is silent: a button that arms one tool while the
ribbon lights another.

## ★ The guard arms in `app::dispatch` are tried in order, so these must not
overlap

`PdfcerApp::dispatch_command` has several arms of the shape
`id if …_for_command(id).is_some()`, and `match` takes the first that
matches. If a `measure.*` id ever answered to [`markup_for_command`] the
measure arm would still win by being written first and the defect would be
invisible; if a `markup.*` id answered to [`measure_for_command`] the markup
arm would be **swallowed** and four ribbon buttons would silently stop
arming anything. [`tests::every_measure_kind_has_a_registered_command`]
asserts the disjointness in both directions.
