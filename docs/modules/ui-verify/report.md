# `ui-verify/report`

PASS, FAIL, SKIPPED — and why the third one has to exist.

## The rule

> **A missing precondition SKIPs. A missing postcondition FAILs.**

A check that could not run has learned nothing, and "learned nothing" must
never be printed as green. That is the same defect as the gate documented in
`PROJECT_PLAN.md` §4.1 — a string checker that scanned three files out of
forty and reported `clean`, because finding nothing looks exactly like
finding no violations.

It is a live hazard here rather than a theoretical one, because **the
application these checks target is still being built**. Checks in this
crate report SKIPPED for weeks at a time — as of S2 the application runs
and traces, and two of the three surfaces under test (the ribbon, the
Settings dialog) have not been written. If SKIPPED rendered as a pass, the
suite would be green for its entire construction period and would go on
being green after the first check silently broke.

Which is why a SKIP reason is held to the same standard as a FAIL reason,
and audited when the application changes. A reason that names the wrong
blocked component — "the application traces no `ui-rect` regions", said of
a binary that traces three of them on every frame — sends its reader to a
finished module to look for a defect that is not there. That is strictly
worse than no reason at all, because they will believe it before they
disbelieve it.

## Where the line falls, concretely

| Situation | Verdict | Why |
|---|---|---|
| no binary at the given path | SKIP | the harness never began |
| no window appeared | SKIP | ditto |
| the trace has no canvas rect | SKIP | nothing to aim at; the click was never made |
| the click was made and selected nothing | **FAIL** | the harness did its job and the application did not do its own |
| Delete was pressed and no deletion was traced | **FAIL** | see [`crate::checks::delete_key`] — this is the D1 verdict |
| the region set is calibrated for a different surface | SKIP | measuring the wrong pixels is worse than measuring none |
| the caption's contrast is 1.1:1 | **FAIL** | this is the D2 verdict |

The interesting row is the fifth, and it is the row that makes this suite
evidence rather than decoration. The old binary is perfectly capable of
emitting its deletion event — the code path exists and the event is in its
vocabulary — it simply never gets there, because the key is suppressed. So
the absence of that event, *after* the harness has established that the
click landed and something was selected, is a postcondition failure and not
a missing feature.
