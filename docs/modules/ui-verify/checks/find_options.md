# `ui-verify/checks/find_options`

Two driven checks over the find bar's two preferences — `OPERATOR_REQUESTS.md`
**O179** and **O180**, both reported by the operator on 2026-09-12 and both
fixed the same day.

# Why both live in one file

They share every expensive part: a sandboxed preference file, a launch, a
Ctrl+F, a typed needle, an Enter, and the same paragraph of reasoning about
why a keystroke that never arrived must be reported as a SKIP rather than as
a Find defect. Splitting them would leave two copies of the control-chord
probe to drift apart. R2 has room: the shared machinery is about a hundred
lines and each check's own assessment is well under that.

# Both preferences had a passing test suite and both were wrong in the
running program

This is the project's founding defect shape and these two are its newest
instances, which is why these checks exist at all rather than only the unit
tests that already cover the predicates.

**O179.** `zoom_on_jump` had a long unit suite and every test passed. They
tested `hold_the_zoom_if_asked` — the half of the mechanism the flag was
read in. The other half, arming `OpenDoc::find_reveal` so the canvas scrolls
the hit to the middle, never consulted the flag and no test asked it to,
because the suite's subject was the function rather than the operator's
sentence. He described it exactly: *"instead of … leaving the page in its
current position on the canvas it still zooms and repositions the page."*

**O180.** Every unit test of search passed too, because the engine was doing
precisely what it was asked. Nothing between the clipboard and
`EditSession::search_text` had ever looked at the query, so a character that
is invisible in a one-line text box decided the answer.

# Both checks run a CONTROL launch, and that is not optional

An assertion that a trace line is absent is satisfied by every build in
which the feature never ran at all — a fixture with no hits, a needle that
matched nothing, a keystroke that missed the window. Each check below
therefore drives the same gesture twice, once in the state where the
mechanism must fire and once in the state where it must not, and reports a
SKIP unless the control produced its evidence. The absence only means
something once the presence has been observed on the same fixture in the
same run.
