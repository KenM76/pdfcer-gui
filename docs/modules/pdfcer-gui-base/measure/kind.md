# `measure::kind` — which dimensioning tool is armed

## Item notes

### `fn every_variant_is_either_offered_or_deliberately_excluded`

`MeasureKind::ALL` stopped being exhaustive over the enum when
[`MeasureKind::Scale`] arrived — it is armed from the Set-scale dialog,
not from a ribbon control, so listing it there would fail
`every_measure_kind_has_a_registered_command` for a kind that correctly
has no command.

An inventory with a silent exception is how a future kind ships armed by
nothing, which is the exact failure `ALL` was written to prevent. So the
exhaustiveness is moved here and made a **compile-time** obligation: the
`match` below has no wildcard, so a new variant does not build until
somebody decides which list it belongs in.

The run-time half then checks the two lists are disjoint and complete,
so a kind cannot be quietly in both or in neither.

### `enum MeasureKind`

Carried on one [`crate::canvas::tool::CanvasTool::Measure`] variant rather
than becoming four `CanvasTool` entries, for the argument
[`crate::canvas::markup::MarkupKind`] settled and which applies here
unchanged: the operator is placing exactly one kind of dimension, so a type
that can say `Linear` and `Circular` at once — which four variants plus a
"which is active" rule spread across call sites can — is a type whose
illegal states must be prevented by discipline. Carrying the kind makes them
unrepresentable, and it makes every rule this enum owns
([`crate::canvas::tool::CanvasTool::cursor`], the press decision in
[`crate::canvas::gesture::press_kind`]) written once for measure as a whole.

**The old shell had four separate `CanvasTool` variants for these**
(`MeasureLinear`, `MeasureCircular`, `MeasureScale`) plus an `is_measure()`
predicate and three `tool_builds_measure_*` functions to ask which. That is
the shape this enum exists to avoid, and it is the one place this salvage
deliberately departs from the source: the old arrangement needed five
helpers to answer questions the kind answers by being a value.

### `const ALL`

A kind listed here and not given a command fails a test rather than
shipping as a tool nothing can arm — the same contract
[`crate::canvas::markup::MarkupKind::ALL`] carries.

# It is no longer exhaustive over the enum, and that is deliberate

[`Self::Scale`] is absent. It is armed from a button inside the
Set-scale dialog rather than from the ribbon, because the dialog is
where an operator is already standing when they discover they need it —
they opened it to set a scale, and the button says *there is a better
way to do this than typing a ratio*. A second ribbon control would put
the two halves of one decision on two different surfaces.

The cost of the exception is that `ALL` stops being a complete
inventory, and an inventory with a silent exception is how a future
kind ships armed by nothing.
`tests::every_variant_is_either_offered_or_deliberately_excluded` pays
that cost: it matches **exhaustively** over the enum, so a new variant
does not compile until it is either put in `ALL` or added to the
excluded list with a reason.
