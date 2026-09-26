# `ui-verify/checks/signing/reaching`

## Item notes

### `fn click_tab_tolerant`

[`click_tab`] asserts a new `ribbon-tab-activated` line, which is the right
test when a check is switching away from a tab it knows is active. It is the
wrong test for *"make sure this tab is on top"*: a tab that is already active
emits nothing when clicked, and the strict form then reports a perfectly good
click as a failure.

A missing tab is still an error. This tolerates *no change*, never *no tab*.

### `fn press`

Through [`declared_or_in_overflow`] rather than a bare rect lookup, and
for `checks::protect`'s reason word for word: at the harness's window width
the File band runs out of room, and `file.sign` is the **third** control in a
Security group that was already the last group added to a full band. A plain
`declared` would report *"the application declared no
`ribbon.item.file.sign` region"* — which would be true, and would be
reported as a missing feature when what is missing is a scroll.

### `fn click`

Through [`driving::frame_of`] rather than `session.frame()`, and the
first driven run of this check is why. **A dialog is an OS window**
(`ui-conventions/dialogs.md` G1), so every region inside the Sign window is
declared in a CHILD viewport with its own origin; `session.frame()` is the
application window's, and clicking `declared_center` against it aims the
real pointer hundreds of points away — at whatever happens to be there.

The symptom was silence: the trace showed `certificate-picked chosen=1` and
then nothing at all, because the press on *Open certificate* landed outside
the button. ⇒ **Ask what the check AIMED AT**, which is the same finding
this project has recorded about the rotation buttons and about
`panning_past_the_overscan`, arriving a third way.

`frame_of` is safe on a main-window region too — an untagged one answers
with `session.frame()`, unchanged — so there is no reason for a call site to
use the other form.

### `fn click_scrolled`

Written for phase E, and the run that forced it is the point. The form
grew by one section when `Pass 10.12`'s certification option landed, and the
placement radios went below the fold: the harness aimed at
`(943, 889)` — a point **outside the application's window entirely** — and
reported it, correctly, as *"there is simply nothing of the application
there."*

⇒ **A region declared inside a `ScrollArea` is a position in the scrolled
CONTENT, not on screen.** [`click`] is right for anything above the fold and
silently wrong below it; a check written against today's form length is a
check that breaks the next time a section is added, which is exactly what
happened here.

The wheel goes at the window's own centre rather than at a fraction of
it, and that is a difference from `reaching::bring_into_body`, which aims
three-quarters down because a dock body carries fixed furniture above its
scroll area. This window's furniture is **below** — the separator and the
button row — so its centre is inside the scrolling region, and aiming lower
would risk the wheel landing on the footer.

Everything goes through [`driving::frame_of`], never `session.frame()`: a
dialog is an OS window with its own origin, and this module's header records
what aiming at the wrong frame cost the first time.

### `fn field_name_of`

⚠⚠ **`panels::signatures` writes `field={:?}` over an `Option<String>`, so
the value arrives as the literal text `Some("SignHere")`** — and on a
document with no field name, as `None`. That is the *"never Debug-format a
value a check parses"* trap in the trace this check's whole verdict rests
on, and it was found by reading phase D's own note, which had been printing
`field=Some("Signature1")` since the day it was written without anybody
noticing that the quotes and the wrapper were not the field's name.

It is normalised **here** rather than fixed at the emitter, and the reason
is ownership rather than preference: `crates/pdfcer-gui/src/panels/` belongs
to no track this session and a concurrent edit would lose one of them. The
one-line fix — a bare token, `none` for the absent case, spelled by a `const
fn` the way `dialogs::sign::refusal_token` is — is reported to the operator
instead. **This function is the workaround, not the remedy**, and it is
deliberately tolerant of the fixed form so that it keeps working the day the
emitter is corrected.

### `fn engine_fixture`

The path is derived, not configured. `D:\Dev\pdfcer` is READ-ONLY to this
project and its corpus is the only place these shapes exist, so the check
reads from it and writes nowhere near it — `checks::adopt_widget`'s
precedent, unchanged.

A missing corpus is a hard error naming the path, not a SKIP: a SKIP reads
as *"this build does not have the feature"*, and this is a fact about the
checkout rather than about the program.

### `fn raise_signatures`

The fallback is not optional, and the first full re-run of this check is
why. `raise_dock_tab` succeeded once purely because a PREVIOUS launch had
left the panel selected and the shell had saved that layout — so the verdict
was resting on inherited state. On a machine whose saved layout has it behind
another tab, the phase would have had no oracle and would have SKIPPED,
reporting nothing, in green.

The tab click is TOLERANT, unlike [`click_tab`]: the View tab may already
be active, in which case a correct click emits no new
`ribbon-tab-activated` line and the strict form reports a click that landed
as one that did not.

### `fn disclosed_appearance`

# What this measures that no test in the process can

The engine composes a visible signature's appearance itself — signer, time,
and whatever reason and location were typed — so it is content the operator
never wrote and cannot read back: the document still open is the unsigned
one. Rule 4 therefore owes it a sentence, and the shell composes that
sentence from `SignReport::appearance_lines`.

⇒ `pdfcer_core::sign::apply::SignReport` is `#[non_exhaustive]`, so no test
outside the engine crate can build one. Every unit test of the composing
function passes over a build whose call site hands it an empty slice, and
the operator sees a report with the page's own text missing from it. This
is the only oracle for that link.

The `sign-disclosed` line carries both halves — how many lines the engine
composed, and how many of them the sentence actually contains, counted
against the sentence rather than against the report.

⚠ **A count of zero composed lines is a note, never a finding.** An
invisible signature owes no sentence, and whether the fixture's field is
large enough for the engine to compose one at all is the engine's question,
not the disclosure's. The finding fires on the disagreement only.
