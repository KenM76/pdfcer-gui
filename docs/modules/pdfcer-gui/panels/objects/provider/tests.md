# `provider::tests` — the object model, proved against real content streams

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

## ★ The two fixtures, and the thing they exist to make possible

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


★ **The line gate still counts these lines.** `check-file-size.sh` counts
total lines, tests included, on purpose — its own header says so — so
this split is not a way of hiding lines from R2. It is the split R2 asked
for, taken on the seam that was already there.
