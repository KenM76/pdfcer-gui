# `panels::objects::provider_tests` — the object model, proved against real content streams

Every test that was in `provider.rs`'s inline `mod tests`, moved out
unchanged, plus the eight that arrived with form-XObject descent.

## Why it is a file rather than an inline module

R2. `provider.rs` reached 1,846 lines when the deep hit test and its tests
landed, and the rule this project was founded on is that the limit is the
signal to find the seam, not to raise the limit — the GUI being replaced
reached 25,005 lines in one `main.rs`, and two independent regressions of
the same key landed two days apart without either noticing the other.

The seam here is the obvious one and the crate already uses it in three
other places (`canvas::selection::tests`, `app::state::tests`,
`app::actions::apply::tests`): a `#[cfg(test)] mod tests;` declared in the
parent, living in its own file, with `use super::*` giving it exactly the
access an inline module had. Nothing about visibility changes, so nothing
about what these tests can reach changes.

## The two fixtures, and the thing they exist to make possible

Most tests here build their model with
[`pdfcer_core::vector::decompose`] over a hand-written content stream,
whose resolver seam is `NoXObjects` — so `PageObjects::leaves` is **always
empty** and not one of them can see whether this provider descends into a
form.

That is not a hypothetical limitation. The deep hit test landed with the
entire workspace suite green, which was a suite reporting nothing about the
change that had just been made. The form tests use
[`ObjectModelProvider::build_or_reason`] against a real `Document`, which
is the only entry point that has a `DocumentView` to descend with, and they
were falsified in both directions before being believed: with the shallow
`hit_test_point_all` restored, three of them go red.

## `#![cfg(test)]` at the top, and why it is the marker rather than the name

Two gates recognise the **inner attribute** as meaning *"none of this is in
the shipped binary"* — `check-ui-strings.sh` and `check-theme-colors.sh`
— and both state why they match on that rather than on a filename: the
property that earns the exemption is not being in the binary, and a
filename is a restatement of it that goes stale the moment a third such
module is written.


**The line gate still counts these lines.** `check-file-size.sh` counts
total lines, tests included, on purpose — its own header says so — so
this split is not a way of hiding lines from R2. It is the split R2 asked
for, taken on the seam that was already there.

## Item notes

### `const PAGE_SIZED_FORM`

The engine's own reproduction of *"when I click on one of the objects
all I get is the page selected"*. The three squares sit at 10,10-50,50,
80,80-120,120 and 150,150-190,190 in PDF user space, and the gaps between
them matter as much as the squares: a click in a gap is a click *inside
the form* that must select nothing.

### `fn provider_over`

The transform is `page_device_geometry`'s, not the identity, because
that is what `build_or_reason` uses live -- so a canvas point in these
tests is a canvas point in the running program. The fixture pages are
200 x 200 with no `/Rotate`, so the map is the plain Y-flip and
`canvas_y = 200 - pdf_y`.

### `fn the_form_fixture_has_one_page_object_and_three_leaves`

Stated as its own test so that a fixture that stopped having a form
fails **here**, with a message about the fixture, rather than turning
every test under it into a confusing report about hit testing.

### `fn a_click_inside_a_page_sized_form_selects_what_is_drawn_there`

A click on a square inside the page-sized form selects **that square**,
as a leaf -- not the form, and not nothing.

Falsified before it was believed: with `hit_test_point_all` in place of
`hit_test_point_deep` this returns `TargetId::Object(0)`, the form,
which is the operator's report reproduced in one line.

### `fn a_click_on_blank_paper_inside_a_form_selects_nothing`

This is the assertion that forbids the tempting "fall back to the
shallow hit test when the deep one is empty" fix. That fallback would
answer this click with the page-sized form -- the operator's original
complaint, restored, for the case that produces it most often.

### `fn a_leaf_outlines_its_own_square_and_not_the_form`

The outline is the whole visible evidence of what got selected. A leaf
whose bounds fell back to its container would draw the page-edge
rectangle that made the original defect look like "the page is
selected", while having actually selected the right thing -- a fix that
is invisible is not a fix.

### `fn a_nested_leaf_reports_its_full_containment_chain`

`nested-forms.pdf` is form A holding form B holding one square. The
intermediate form is deliberately **not** a leaf: it is a container,
and counting it as content would make "how many objects are in here"
wrong by one per level.

### `fn selection_tolerance_is_honoured_per_query_not_baked_in`

This is what makes the fix meaningful rather than cosmetic. Before it,
the tolerance was hard-coded at 3.0 canvas units at every zoom, so at
"Fit page" (~0.5x on a letter page in a typical window) the operator's
real on-screen catch radius was ~1.5 px and thin geometry could not be
clicked at all. The tolerance now arrives from the caller, scaled by
`1 / zoom`, which keeps the on-screen radius constant.

The *other half* of that law — that the caller's conversion really is
`1 / zoom` — is asserted in `canvas/` at S4, where the conversion
lives. See this module's header, "What changed at salvage" §4.

### `fn part_kind_and_part_count_answer_for_every_object_kind`

This is what the Objects panel's tree builder relies on to decide
whether a row gets an expander. A path with subpaths expands, a text
object with lines expands, an image is a leaf — and the panel asks one
question rather than matching on `VectorObject` itself, which is the
duplicated-predicate drift [`ObjectModelProvider::part_hits`]'s own
docs warn about.
