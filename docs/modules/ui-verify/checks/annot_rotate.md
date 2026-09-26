# `ui-verify/checks/annot_rotate`

`rotating_a_markup_turns_it` — **draw a shape, grab its ninth handle, and it
is turned.**

# What this is for


## Why this cannot be a unit test, and what it is really guarding

`canvas::rotating`'s arithmetic is pure and has eight unit tests;
`gesture::meaning` has four more for the new rung. What none of them can
reach is **which verb the release lands on**, and that is the whole risk
here:

| # | link | its own test |
|---|---|---|
| 1 | a markup is selected and a **ninth** affordance is painted | `handles::GripSet` — the flags, not the paint |
| 2 | a press above the box finds `Grip::Rotate` over `grabbable`'s box | `handles::grip_at` — the geometry, given a box |
| 3 | it becomes `DragKind::Rotate` and not `Move` | `gesture::meaning` — given `annot_rotate`, which `pressing` computes |
| 4 | `rotating::Frame::bounds` is `grabbable`'s and not `overlay::grip_box`'s | **nothing** |
| 4b | **no CONTENT-shaped guard stands in front of the annotation branch** | **nothing** |
| 5 | the commit routes to the **annotation** verb, not `transform_objects` | **nothing** |
| 6 | the sign survives the screen → page crossing | **nothing** |

**Link 4 is the one that would ship, and it would ship as silence.**
`overlay::grip_box` derives from the selection's cached *content* outlines,
which `select_annot` clears — so over an annotation it answers `None`,
`rotating::drag` returns at its first line, and the whole gesture is a
no-op with nothing said anywhere. That is this project's founding defect
shape exactly: a grip that is dragged, released, and does nothing.


It was added to the table *after* that run, and it is here because the run
is the only reason anybody knows it exists. Link 4 was **already correct**:
`canvas::interact` had passed `pressing::grabbable`'s box since the day the
module landed, and this check's failure message said otherwise — a
confident, specific, wrong accusation, which `checks::rotate` already warns
is worse than a vague one.

The real break was a
`selection.object_indices_on(page_index).is_empty()` guard sitting *above*
`rotating::drag`'s annotation branch. It counts page **content**, which
`select_annot` clears, so it returned `None` on every markup before the
routing decision was ever reached. **Identical symptom, different line, and
the same silence** — which is why the failure message below now names both
candidates and names this one first.

⇒ Three destinations share this gesture. The durable form of the lesson is
that **a guard written in one destination's vocabulary belongs after the
branch that picks the destination**, and `canvas::rotating`'s header carries
it as the sixth instance of this canvas's recurring hazard.

**Link 5 is the one that would ship and look deliberate.** If the press
fell through to `caps.edit_content`'s branch, the commit would call
`transform_objects` on the *page content* selection — a working gesture
aimed at the wrong verb, which is the failure this canvas has produced four
times. From a chair it is invisible: on an empty content selection nothing
happens, and on a non-empty one *something moves*.

⇒ So this check does not only assert that a rotation happened. It asserts
**which line the trace carries**, and it asserts that the content verb's
line did **not** appear. Those two together are the only place link 5 is
visible from outside the process.

# The oracle carries DEGREES, and a signed number

`rotate-annot-commit id=… deg=… px=… py=…`, then
`rotate-annotation-applied … from=WxH to=WxH`. A line saying only *"a
rotation committed"* would be identical for a build that turned the other
way, pivoted about a corner instead of the centre, or left the appearance
`/Matrix` alone. This project's standing rule, earned by `DEFECTS.md` D14:
**a trace line must carry the number a wrong build would get wrong.**

`from=`/`to=` are **reported and not asserted**, deliberately. `/Rect`
grows at any angle that is not a quarter turn (§12.5.2 requires it upright)
and this check drives exactly a quarter turn, where it must **not** grow —
so asserting growth would be asserting the opposite of what this gesture
produces. What the two numbers are for is the reader of a failed run.

# Where it aims, and why it no longer mirrors a constant

`checks::rotate` — the page-content sibling — derives the handle's position
from the selection outline plus its own copy of `ROTATE_STEM_PX`, and says
in its own comment that it *"does not aim at this number directly"*. It had
no choice: the handle published no region of its own.

It does now. `canvas.rotate-handle` is published by `overlay::draw_grips`
**only inside the `offer.rotate` branch**, so its presence in the trace is
the application's own statement that the affordance exists — and this check
aims at its declared centre. That removes the last mirrored number from the
aiming path, and it makes step 4 below a *direct* observation of link 1
rather than an inference from a failed press.

## Item notes

### `const INVOKE`

`mode.review` because markup is authored there — and because it is the
mode where `caps.edit_content` is **false**, which is precisely the mode the
new rung had to fire in. Driving this in Edit would pass on a build whose
rotate handle only worked where the content branch could catch it.

### `const CONTENT_ROTATE_EVENT`

Asserted **absent**. See the module header, link 5: a press that fell
through to `caps.edit_content` produces this line instead, and the resulting
build has a rotate handle that works perfectly on the wrong subject.

### `const ROTATED_EVENT`

`-applied`, per the convention this project adopted after making the
same-name mistake twice: `vector_edit` writes its own bare
`rotate-annotation …` line for the identical edit, and `.last()` on the bare
name reads that one.

### `const MOVE_EVENT`

The other half of link 3. A build whose `annot_rotate` was computed from
the live pointer rather than from `press_origin` finds `None` on every real
drag (egui does not call an interaction a drag until the pointer has
travelled a threshold, by which time it is ~20 pt from an 8 pt handle) — and
the press then means whatever the rungs below say. If that came out as a
MOVE, this line appears and the shape slides across the sheet.

### `const ANGLE_EVENT`

# Why a trace line and not a screenshot

Because the two builds this has to tell apart are *"drew the upright box"*
and *"drew a quad that happens to coincide with the upright box"*, and on an
unturned mark those are the same picture - so a pixel oracle is blind
exactly where the regression would land. The line carries `turned=` **and**
the corners, because a check reading only the flag would pass on a build
that took the turned branch and then drew the upright box's corners.

### `const PROPERTIES_TAB_REGION`

A dock draws only its ACTIVE tab, and in Review the right dock opens on
Comments. Reading the trace without bringing this forward reports "the panel
published no angle" about a build whose panel is correct.

### `const CANVAS_REGION`

Read so a failure can tell *the handle is off-canvas* from *the handle is on
the canvas and the press was routed wrongly*. Those are different defects in
different files and they produce the identical symptom. `checks::rotate`
records the run where that distinction was missing and three confident,
specific, wrong causes were named instead.

### `const SHAPE`

**Well below the top of the sheet**, which is this check's own version of
`checks::rotate`'s O22 hazard: the rotate handle sits `ROTATE_STEM_PX` above
the selection box, so a shape near the top of the viewport has its handle
clipped away by the painter and the press lands on the ribbon. 0.35 down the
page is comfortably clear of it on any sheet size.

And away from the edges, so the drag in step 5 — which swings out to a
radius of half the box plus the stem — has somewhere to go.
