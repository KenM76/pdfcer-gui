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

## ★★★ THIRD RECURRENCE — 2026-09-03, and it hid FOUR operator-visible defects

`ui-verify`'s `dialogs_open_in_their_own_window` sweeps every command-reachable
dialog for *"is this a real OS window"*. Its subject list is a `const DIALOGS:
&[(&str, &str)]` typed by hand, and **Print was not in it** — the dialog whose
report (*"Print dialogue box doesn't pop up in its own movable window"*) started
that entire piece of work.

The header even rationalised the omission: *"Print was fixed that evening and
`print_dialog` asserts it."* And `print_dialog` asserts the job reaches the
**spooler**. That is not a claim about the window, its margins, its scrollbars
or its buttons — and all four of those were broken, for weeks, in the one
command-reachable dialog with no headless check.

⇒ The tell to look for: **a completeness sweep that names an exception in
prose.** "X is covered elsewhere" inside the list's own documentation is the
sentence that decays, because "elsewhere" asserts something about another test
that nobody re-reads. If it is genuinely covered elsewhere, the list costs one
line to include it anyway and the duplicate proves the claim.
