---
name: a-hand-written-list-inside-a-completeness-test-is-the-gap
description: A test that proves "the window draws everything the catalog describes" is only as complete as the hand-written file list inside it — a new module is invisible to the check built to find it
metadata:
  type: feedback
---

**When a completeness test enumerates its inputs by hand, adding a new input
makes the test silently narrower — and the count still adds up.**

**Why:** 2026-08-28, adding a Comments group to the Settings window.
`text::settings::tests::the_window_draws_exactly_the_settings_this_catalog_describes`
parses every settings module with `syn` and counts `widgets::header` calls
against `SETTINGS_COUNT`. The module list, `GROUP_SOURCES`, is a hand-written
`&[(&str, &str)]` of `include_str!`s.

A new `dialogs/settings/comments.rs` was not in it. So:

- its header was **not counted** → the drawn total stayed at the old number;
- its triple was **not in the catalog** → the described total also stayed;
- **both halves were wrong by one and the assertion passed.**

The whole suite went green with a setting that no completeness check had ever
looked at. The test is genuinely good — it caught two engine settings within
one `cargo update` each — and its blind spot is only reachable by *adding a
file*, which is rare enough that nothing routinely exercises it.

**How to apply:**

- **Before writing a new module that a sweeping test consumes, add it to that
  test's input list.** Not after — the green run in between is the trap.
- When you meet a test that walks "all the X", **grep for how it gets the
  list**. `include_str!` in a const array, a `match` over an enum, a `&[&str]`
  of ids — all of these are hand-written and all decay the same way. A
  `read_dir` or a build-script glob does not.
- The tell that a sweep is hand-listed: it compiles when a file it should cover
  is deleted.

★ This is the same family as [[a-check-that-cannot-fail-is-not-evidence]] and
[[a-long-green-check-can-be-aiming-at-nothing]], with one difference worth
holding: those two are about a check that never *fires*. This one **fires,
passes, and reports a number** — the arithmetic is internally consistent and
externally short.
