# `ui-verify/checks/roster`

`checks::roster` — **which checks exist, and the order the suite runs
them in.**

# Why this is its own file

## The seam, and the argument for it

`checks/mod.rs` holds **the index**: the module declarations that make the
tree, each carrying the argument for why that check exists.
`harness.rs` holds **the contract** — the [`Check`] trait and the
[`CheckContext`]. This file holds **the roster**: one `Box::new(..)` per
check, in run order, each with the note explaining why it sits where it
does.

Those are genuinely three subjects, and the tell is who edits them:

| | `harness.rs` | `mod.rs` | `roster.rs` |
|---|---|---|---|
| changes when | the harness gains a capability every check can use | a check is added or removed | **any** check is added, removed or re-ordered |
| how often | rarely | every landing that ships a driven check | the same, plus re-orderings |
| reviewed for | is the contract still right? | does this check exist, and why? | is it in the right place, and does its note say why? |

The second row is the whole argument. This list grows with every feature
the project ships; the trait beside it barely moves. Keeping an unboundedly
growing list in the same file as a stable contract makes every landing touch
the contract's file, and makes the file's size somebody's problem at random
rather than the problem of whoever owns the growth.

## The `pub mod` declarations stayed behind, deliberately

They look like roster material and they are not: a `mod` declaration
*defines the module path*, so moving `pub mod layers_search;` here would
rename the check to `checks::roster::layers_search` and break every
reference in the crate. They stay in `mod.rs`, which is also where a
reader looking for "does a check for X exist?" will look first.

## The ordering notes are content, not decoration

Several entries carry a paragraph about **why they are adjacent to the one
above** — a dependency (`unshare_form` SKIPs on what `form_selection`
asserts), a pairing (two checks that differ only in the document they open),
or a diagnosis that only reads correctly when two verdicts sit together in
the summary. Those notes travelled with the entries. Re-ordering this list
without reading them has cost this project a misdiagnosis before.
