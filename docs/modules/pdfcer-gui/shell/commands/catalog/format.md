# `shell::commands::catalog::format` — the Format contextual tab — what changes about the selection


## The split is per TAB, and the reason it was refused before is gone

[`super`]'s header argued against exactly this cut:

> a per-tab split would put the handler-token blocks in eight files where a
> collision between two of them is invisible.

**That objection was already false when it was written.**
`super::super::tests::every_handler_token_is_unique` sweeps the whole
registry, and `every_handler_token_is_in_its_tabs_block` asserts each token
sits in its own tab's hundred. A collision is not invisible — it is a red
test, in either arrangement — so the argument that kept 120 commands in one
file rested on a property two tests had already taken over.

⇒ Recorded rather than quietly reversed, because it is the same shape this
project keeps finding: **a reason that was true when written, is checked by
nobody, and outlives what made it true.**

## What is here, and what is not

The `Command` entries and the argument for each one's label, tooltip,
handler token, icon and enable predicate. **The prose is the point** — most
of this file is the record of decisions that would otherwise be re-litigated,
which is also why the byte count grew past a limit in the first place.

Not here: the registration itself ([`super::super::register`]), the
command-id-to-behaviour mapping ([`super::super::mapping`]), and the
reachability register ([`super::super::reach`]).
