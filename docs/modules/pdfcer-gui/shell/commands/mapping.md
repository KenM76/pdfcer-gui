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
