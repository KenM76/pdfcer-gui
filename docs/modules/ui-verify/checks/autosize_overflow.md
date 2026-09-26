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

# ★★★ Why this check exists when two other tests already cover it

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

★ `bound=` is spelled by this shell's own `bound_token`, never `{:?}`.
`Debug` is another crate's unstable rendering; a check keyed on it goes
quiet — or reports the opposite of the truth while quoting the truth — the
day upstream renames a variant.
