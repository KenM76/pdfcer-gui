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

## Item notes

### `const SETTLE_FRAMES`

The 48 `find_bar` waits, for the same reason: a search run against a
document still rastering reports zero hits for a reason that is not Find's,
and zero hits is what both checks below treat as "no evidence".

### `fn gesture`

`tag` names the artefact, so a failing run leaves both launches' traces
side by side — which is the first thing anybody reading the failure will
want, because every assertion in this file is a comparison between two
runs.

### `fn write_prefs`

Through `sandbox::write_prefs`, never `fs::write`. The header it
prepends carries `ask_default_app = false`, and three checks that wrote the
file directly re-enabled the O173 startup offer in front of their own
launches — one of which then measured the *dialog's* client area and
reported a working preference as broken.

# Errors

The directory could not be created or the file could not be written. A SKIP
at the call site: a preference that could not be written means the check
never began, and reporting that as a Find failure would name the wrong
subsystem.

### `struct RestorePrefs`

A guard rather than a line at the end, because there are a dozen returns
above and the one that gets forgotten is the one that leaves
`find_zoom_on_jump = false` behind for every check that runs afterwards. A
suite that shares state measures the order it ran in.

Reset rather than deleted, for the reason `ui_scale` records: a *missing*
file exercises the absent-file path, which is a different state and not the
one the other checks were written against.

Failure to restore is warned about rather than fatal — this type runs during
unwinding as well as on the ordinary path, and a harness that turned its own
housekeeping problem into a verdict would be reporting itself as a defect in
the program.

### `fn the_needle_is_the_letter_e`

Pinned because the failure text quotes it and the SKIP messages tell the
operator to *"point the check at a document containing the letter `e`"*.
A key code that drifted from that sentence would send somebody looking
for the wrong character in their own file.

### `fn the_blank_is_the_space_bar`

A wrong code here does not fail loudly: it types some other character,
the search legitimately finds nothing like it, and the check reports
that trimming is broken. That reads as an application defect and is a
harness bug.

### `fn the_trailing_space_is_off_until_a_check_asks_for_it`

O179's check never sets the flag, and if the default were `true` it
would be silently driving O180's gesture instead — and still passing,
because a trimmed query searches for the same needle. A wrong default
here is invisible in every outcome, which is exactly the class this
project pins.
