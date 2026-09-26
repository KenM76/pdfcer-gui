# `pdfcer-gui/shell/commands/reach/guards`

## Item notes

### `fn guard_claiming`

This is the half a shell script could not have written. Each name returned
is the same string [`read_arms`] extracts from the guard arm that consults
it, so the two halves can be compared as sets; and each answer comes from
the real mapping rather than from a re-derivation of it, so there is no
second table to drift. [`super::super::mapping`]'s header states the property this
preserves: *"two hand-written tables can disagree, and one table plus a
derived search cannot."*

Order is irrelevant here even though it is load-bearing in the dispatcher,
where `match` takes the first arm that matches. Reachability asks only
whether **some** arm claims the id; which one wins is asserted, in both
directions, by the disjointness tests in [`super::super::mapping`].

### `const EVALUATED_GUARDS`

**Not a mirror of the dispatcher**, and the distinction is the one `D5`
turns on: this list is *asserted equal* to the set read out of
`dispatch.rs`'s syntax tree by
[`tests::the_guards_the_checker_evaluates_are_the_guards_the_dispatcher_has`],
so it cannot drift without a named failure. A hand-maintained list that
nothing checks is the defect; a hand-written list that a test pins against
the source is a declaration.
