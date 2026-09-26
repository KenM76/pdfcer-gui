# `ui-verify/checks/embed_bundled`

`embedding_works_with_no_font_folder_at_all` — **pdfcer's own fourteen faces
answer when nothing of the operator's can, AND only when he asks.**



# What this is for

`OPERATOR_REQUESTS.md` **O47** asked the operator whether pdfcer should embed
the standard-14 faces it ships when none of their folders holds the font a
document names.


The 2026-08-28 answer was *"always, disclosed loudly"* — no control, the
bundled rung simply on. It is now **the disclosed opt-in**: a checkbox in
the Embed window, **off** when the window opens, with the fonts it would
stand in for named beside it. The deciding reason is not the letterforms,
which were always disclosed; it is that pdfcer's fourteen faces are
BSD-3-Clause and embedding one puts that licence inside a file the operator
then distributes. `pdfcer`'s own CLI keeps `--use-bundled-fonts` off for
exactly that, in those words.

## WHY THIS CHECK NOW DRIVES BOTH POSITIONS

> *A check that the switch is off by default passes on a build that ignores
> the switch entirely.*

That is the shape this project keeps meeting, and it is why *"assert the box
is unticked"* is not a check. Both positions are driven, in one run, in
order:

| position | asserted |
|---|---|
| as the window opens | `own_fonts_on=false`, and `own_fonts_offered` is **greater than zero** — the offer was made and declined |
| after ticking the box | the commit reaches the engine with `substituted=true` |

The first half alone passes on a build that has lost the bundled faces
altogether (nothing to offer, nothing offered, box off — all true). The
second half alone is the check as it stood before today, and passes on a
build with no switch at all. Neither is worth anything without the other,
and `own_fonts_offered=` exists on the trace line so that the first half can
be made at all.

## Why it is a SEPARATE check and not a parameter of the other one

Because it asserts the opposite premise. `embedding_fonts_puts_a_program_in_
the_document` supplies a real font folder and would pass identically with the
bundled rung ripped out — the folder answers first, every time, by design.
Only a run with **no folder at all** can distinguish *"pdfcer ships faces and
will use them"* from *"pdfcer ships faces and never reaches them"*.

That is also why it is worth the extra process launch. Two checks over one
feature, differing in one environment variable, is the shape that catches a
rung being unreachable — which is the same failure the whole resolver had on
the day it was written, when only the exact rung worked and every test
registered a name and then asked for it.

## The oracle is `substituted=true`, and it is the point of the row

The operator's *"yes"* came with a condition: **disclosed loudly**. A build
that embedded a bundled face and reported it as an ordinary match would
satisfy the letter of the request and lose the half he can act on — the
document goes out with pdfcer's stand-in in it, and nothing on the canvas says
which face went in.

`substituted=` is `EmbedPlan::substitutes_any`, computed by the engine from
`FontMatch::is_substitute` on every target. It is `true` here **only if** the
shell reported the rung honestly on the way in — a shell that claimed `Exact`
for a bundled face would produce a green `missing_after=0` and a false
`substituted=false`, so this one field checks the disclosure and the
correctness together.

⇒ Reporting a bundled donor as `Exact` would also walk it past the engine's
**symbolic-font guard**, which turns on exactly that predicate. The
disclosure and the guard are the same flag, which is why understating it is a
correctness defect rather than a cosmetic one.

## A DECLINE IS A SKIP, NOT A FAILURE — and reading it the other way
cost an afternoon

The 2026-08-28 sweep ran this check twice. On `fixtures\a1-titleblock.pdf`
it **passed**, with `targets=3 … substituted=true` and no font folder
configured — the bundled rung firing exactly as O47 asked. On
`D:\Dev\pdfTests\SW41177\SW41177.pdf` it **failed**, on
`embed-fonts-declined folders=0 detail=nothing-to-open`, and that failure
was carried into the handoff as *"the strongest candidate for a real
defect"*. It was neither. SW41177 carries six fonts and **six
`/FontFile2` streams** — every face it names is already embedded, so
*"nothing to do"* is the correct and only honest answer.

⇒ The old message claimed a decline meant *"the bundled rung was not
reached"*. **That inference is not available**, and has not been since O47.
`DialogsState::open_embed_fonts` has exactly ONE decline sentence left —
`text::embed::nothing_missing` — because O47 falsified the other branch
(*"pdfcer has no font folders, so it cannot embed anything"*) and it was
deleted. `EmbedDialog::open` answers `None` only when `plan.targets` is
empty **and** every `plan.blocked` row is `AlreadyEmbedded`; a missing font
nothing can answer for is a `NoSourceFont` row, which is `shown`, which
opens the window. So a decline is a statement about the FIXTURE and
carries no information about the resolver at all.

**The oracle for a broken bundled rung is the `targets=0` branch
below, and it always was.** If `allow_bundled` stopped reaching
`resolve_for_embedding`, this run would still open a window — full of
`NoSourceFont` rows — and that branch would fail it by name. Turning the
decline into a skip therefore gives up nothing: it removes a false failure
and leaves every true one standing.

The general lesson, and it is the expensive half: **a check whose
failure message names a suspect can teach a reader the wrong suspect.**
This one named `Library::scan_with(folders, true)` — a call that was
correct, tested (`app::fonts::a_bundled_face_answers_only_when_it_is_
allowed_to`) and proven in the very same sweep by the sibling run. Two runs
of one check disagreeing is a fact about their INPUTS first; check that
before believing either one's diagnosis.

## What this does NOT establish

**Which face was substituted, or that it looks right.** pdfcer's standard-14
substitutes are the engine's to choose and its tests cover the choice. This
establishes that the shell reaches them, at the right moment, and says so.
