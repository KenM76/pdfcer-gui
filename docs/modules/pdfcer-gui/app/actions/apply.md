# `app::actions::apply` — the other end of the funnel: what an [`Action`]
actually does

[`super`] declares the **vocabulary** — one variant per thing an operator
can ask for, each carrying a complete statement of that intent. This file is
the **interpreter**: [`PdfcerApp::apply_actions`] drains the frame's queue,
[`PdfcerApp::apply`] routes one intent to one state transition, and
[`vector_edit`] is the four-step protocol every arm that changes a document
goes through.

## Why this is its own file

`app/actions.rs` crossed the 1,500-line gate (standing rule **R2**) when
`file.save_copy` was wired, and the rule's own justification decides where
the cut goes rather than the line count: *"the value of the limit is that
the file has to have a single subject."*

Two subjects were sharing one file, and they change for entirely different
reasons:

| | subject | what changes it |
|---|---|---|
| [`super`] | **what an operator can ask for**, and what each request must carry to remain resolvable after the frame that raised it | a new command with a new operand |
| here | **what happens when one is granted**, and the ordering that makes a mutation safe | a new engine verb, or a change to the cancel-mutate-bump-invalidate protocol |


## What did NOT move, and why

The **edit-disclosure store** stayed in [`super`], beside the type it holds,
even though [`vector_edit`] is its only writer. Two reasons: it is read by
`crate::app::status` through [`super::last_edit_disclosure`], which makes it
part of this module tree's published surface rather than an implementation
detail of applying; and Rust's own visibility rule means a child module can
reach an ancestor's private items, so `record_edit_disclosure` is callable
here without being made `pub` to anybody else. Splitting a store from its
type to follow its writer would have been the tidier-looking edit and the
one that widened a private thing's visibility for no gain.

## Item notes

### `fn apply`

Every arm is a state transition on [`crate::viewer::ViewState`],
which is where the clamping and the ladder arithmetic live and are
tested. This function decides *which* transition, never *what it
means* — a zoom that saturates, a page step that stops at the last
page and a NaN that falls back to actual size are all decided in
`viewer`, under unit test.

### `fn apply_actions`

Applied in the order raised. `pixels_per_point` is passed in rather
than read from a context because the per-page zoom ceiling depends
on it — see [`viewer::max_zoom_for_page`] — and threading it makes
this function pure with respect to egui, which is what keeps it
reviewable.
