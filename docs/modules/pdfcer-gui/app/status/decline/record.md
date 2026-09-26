# `app::status::decline::record` — every writer of the decline slot

**The recording half of [`super`].**

## Why this is the seam, and not a cut anywhere convenient

[`super`] answers two questions that share a store and nothing else:

| question | where it is now answered |
|---|---|
| *what is a decline, and how long does it owe its sentence?* | [`super`] — the [`Declined`] enum, `still_true`, `line`, `retire`, `live`, `show` |
| *who says one, and from which phase?* | **here** — one constructor per source of truth |

The second is a family of twenty-odd functions whose whole content is a
one-line write plus the argument for **where it is called from** — dispatch
or apply, before the verb or inside it — and that argument is the same
argument twenty times over. It reads better as a file about recording than
as a tail on a file about retirement.

`decline/floor.rs` and `decline/textedit.rs` are recorders too, so the
arrangement is regular rather than invented for this file. Those two keep
files of their own because each carries an argument of its own (the floor's
ordering rule; the text caret's channel argument) that would be buried in a
file of twenty siblings.

## Re-exported, so no call site names this module

[`super`] carries `pub(crate) use record::*;`, so every `decline::record_*`
call in the crate resolves through the parent. A split that also renamed
its callers could not be reviewed as a split.

**Written unconditionally, overwriting whatever was live**, everywhere in
this file. `LAST` is a slot rather than a queue precisely so the most recent
answer is the visible one — see [`super::record_text_style`]'s rule, and
`decline/textedit.rs`'s restatement of why it matters for a control an
operator presses twice.
