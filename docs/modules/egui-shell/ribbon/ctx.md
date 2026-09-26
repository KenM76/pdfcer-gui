# `egui-shell/ribbon/ctx`

## Item notes

### `fn the_local_condition_evaluator_agrees_with_enable`

There are two implementations of one rule: the canonical one in
`commands`, and this allocation-free copy used for contextual
tab visibility. That is a drift hazard with a nasty failure mode —
a `Format` tab that appears under conditions its author's enable
predicate would have refused, so the tab is present and every
control on it is disabled.

The copy is justified (it runs once per contextual tab per frame
and `Enable::When` would allocate a `String` to be dropped), but
it is only safe while something checks the two agree. This is that
something.

### `fn an_empty_condition_says_nothing_rather_than_never`

The one deliberate divergence from `Enable::When`, which has no
empty case because a command always carries a real predicate. A
manifest that spells `visible_when: ""` has said nothing, and
reading "nothing" as "never" would silently delete a tab from the
interface with no message anywhere.
