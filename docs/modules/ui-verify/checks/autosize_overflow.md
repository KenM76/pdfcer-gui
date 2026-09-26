# `ui-verify/checks/autosize_overflow`

`a_field_too_small_for_its_text_says_so` — the one auto-size outcome that
is not an answer, driven through the real window.

# What this asserts, and why the size alone was never enough

A PDF text field can ask the viewer to pick the point size (`/DA … 0 Tf`).
`pdfcer-core` picks one and reports **which constraint decided it**:
`AutoFitBound::Height`, `::Width`, or `::Floor`. The first two mean *it
fits*. The third does not — the engine's own comment at the branch that
returns it reads *"the one case where the returned size does NOT fit the
constraint that produced it"* — so the text is going to spill out of the
box, and pdfcer stopped shrinking only to keep it readable.


> *"⚠ "FullName" asks for an automatic text size and pdfcer chose 4.0 pt.
> Another program filling this field may choose differently."*

A sentence about interoperability, when the fact was that his text does not
fit. ⚠ Meanwhile `OPERATOR_REQUESTS.md` **O86** told him, under a ✅, that
*"pdfcer now tells you which way it decided … the box is too small for this
text, which will overflow"* — true of the engine and the CLI, and **false of
this shell for three days**.

# Why this check exists when two other tests already cover it

Because neither of them is the operator.

| test | proves | cannot see |
|---|---|---|
| `app::status::tests` | the three sentences differ and only one claims overflow | that anything ever *reaches* the overflow arm |
| `tests/autosize_floor_says_it_will_overflow.rs` | a real fill through `EditSession` reports `Floor`, and the sentence is built from it | that a **click and a keystroke** get there, or that the bar draws it |

This project's standing lesson is that a chain can be green at every link
and broken end to end — *"eight green tests while the feature did 1 of 14"*.
So this check presses the keys.

# The trace oracle, and why the application had to grow one

`form-fill-text` carried `commands=` and `epoch=` and nothing else, so a
build that dropped the bound produced a **byte-identical** trace line to one
that honoured it. `panels::forms::edit` now appends `autosize=` and
`bound=`, for `place.rs`'s standing reason: *a trace line must carry the
number a wrong build would get wrong.*

`bound=` is spelled by this shell's own `bound_token`, never `{:?}`.
`Debug` is another crate's unstable rendering; a check keyed on it goes
quiet — or reports the opposite of the truth while quoting the truth — the
day upstream renames a variant.

## Item notes

### `const MODE`

Filling a form is not editing a document, and this shell has said so
since canvas filling landed: a field is fillable in Read mode because
filling in a reading stance is what forms are *for*. Driving it here rather
than in Edit also proves the disclosure reaches the one mode whose dock
does not mount the Forms panel — which is the whole argument for putting the
sentence on the status bar rather than in the panel alone.

### `const BAR_REGION`

Asserting this is what turns the check from *"the engine reported a
floor"* into *"the operator was told"*. The bound reaching the trace proves
the value survived the shell's own plumbing; it does **not** prove the bar
drew anything, and a build whose status row never called `fill_disclosure`
would satisfy every assertion above it.

⚠ It proves the line was DRAWN, not which sentence it holds. The wording is
pinned by `app::status::tests` and by
`tests/autosize_floor_says_it_will_overflow.rs`; a pixel oracle for the
glyphs would be the only way to close that last inch and is not worth it
here, because the branch that chooses the sentence is a `match` on the very
value this check has already read out of the trace.

### `const TOO_LONG`

Long enough that the **width** bound drives the size under the engine's 4 pt
legibility floor, and the length is **measured rather than reasoned**:

a first draft of 84 characters drove this fixture to `autosize=5.1
bound=width` — close, and not there. The chosen size scales inversely with
the text's width, so the 4.0 pt floor needs about `84 x 5.1 / 4.0 = 107`
characters; this is 132, which leaves room for a font-metric change without
leaving so much that the check stops resembling anything an operator types.

That first run is why the SKIP arm below quotes `autosize=` as well as
`bound=`. *"It came out at 5.1"* names the next edit; *"it was not floor"*
does not.

Letters and spaces only, and the constraint is the instrument rather than
the subject: `type_ascii` refuses punctuation because `-`, `.` and `/` are
`VK_OEM_*` codes whose meaning is keyboard-layout specific, so a check typing
them would pass here and type something else on another machine. A hyphen in
this name would have been a silent layout dependency for no gain.

Plain ASCII on purpose. `type_ascii` sends key events, and a check that
also exercised the `WinAnsi` substitution path would be testing two
disclosures at once — and the other one has its own sentence, which would
then be concatenated onto this one and break the assertion for a reason that
has nothing to do with auto-size.

### `struct PlacedBox`

The application's numbers, not the fixture's — the same rule
`form_field`'s `placed_boxes` states: a check that computed the rect from
the PDF would be asserting that two independent derivations agree, and would
report a disagreement as a hit-test failure.
One widget, as the application says it drew it.
