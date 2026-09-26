# `text::commands::annotate` — the labels and tooltips of the **Markup** and
**Measure** tabs

## Why this is a file of its own, and what the seam actually is

**R2** (no `.rs` file over 1,500 lines) forced a split when the three
unblocked Phase 6 markup kinds arrived: [`super`] reached 1,520 lines. But a
line count only says *that* something had to move; it does not say what, and
`tools/gates/check-file-size.sh` says in its own header that shaving prose to
fit a threshold is the behaviour it exists to refuse. So the question was
which subject was separable, and this one is:

> **Markup and Measure are the tabs about what you ADD ON TOP of the page.
> Everything left in [`super`] is about the file, the view, the pages, or the
> content that is already there.**

That is not a line drawn here for convenience. It is the line
[`crate::app::modes::Capabilities`] already draws — `edit_content` on one
side, `author_markup` and `author_measure` on the other — and it is why
Review mode exists at all: a reviewer may add a comment and a dimension to a
drawing they may not otherwise touch. It is also the line
[`crate::shell::manifest`] already draws, which keeps `markup.rs` and
`measure.rs` as files of their own beside `edit.rs` and `pages.rs`. A reader
adding a Markup command now edits `text/commands/annotate.rs` and
`manifest/markup.rs`, which sit one directory apart and describe the same
band.

## Nothing else changed


The tests stay in [`super`], with the list they walk. They are about the
catalog as a whole — no two labels alike, every tooltip a sentence, every
command reachable — and splitting them across the two files would let a
duplicate label pass by being in the other half.
