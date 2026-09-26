# `canvas::markup::ink` — freehand, and the hundreds of points nobody asked
for

`/Ink` (§12.5.6.12): **press, follow the pointer, release.** Drag-shaped, so
it fits the existing `DragKind` path with no new gesture — and *not*
band-shaped, because the thing being authored is the whole path the pointer
took and a band is two points. That difference is the whole of this module:
the trail, its lifetime, its simplification, and a preview that draws the
simplified trail rather than the raw one.

---

## 1. One drag is one annotation, and one annotation is one stroke

`/InkList` is a list *of* strokes, so the format permits several drags to
accumulate into a single ink comment. This shell authors **one stroke per
drag, one annotation per drag**, and the reference applications decide it:

| | freehand tool | what a second drag does |
|---|---|---|
| **Inkscape** | the pencil | a second, separate path object |
| **Acrobat** | Comment ▸ Draw free form | a separate comment; the strokes are not merged unless the operator explicitly groups them |
| **SolidWorks** | — | no freehand surface at all, so no vote — *ask which of the three has the surface* |

Two of two applicable references say one drag, one mark. It is also the answer
that costs nothing to explain: the release commits, exactly as it does for the
four band kinds, so the whole family shares one sentence — *let go and it is
drawn* — and `Ctrl+Z` removes exactly the stroke the operator just made rather
than a session's worth of them.

[`super::Geometry::Strokes`] is still a list of lists, because that is the
engine's shape and this module's job is not to editorialise about it. What is
deliberately absent is a *gesture* that fills more than one entry.

---

## 2. THE TRAIL IS DERIVED, NEVER STORED — which is what makes its lifetime
## impossible to get wrong

A trail is state that outlives a frame, and this codebase has a standing
argument about that shape of state. [`crate::canvas::tool`]'s header makes it
about the space bar: the obvious implementation — remember on key-down, restore
on key-up — *"is the one that fails"*, because the restore step can be missed
and the failure is **sticky**.

The trail has the identical hazard. A drag can end four ways — released,
Escaped, interrupted by focus loss, interrupted by the space bar borrowing the
hand — and only the first two are events this module could hook. An
implementation that cleared the trail on release and on Escape would leave a
stale trail after an interruption, and the operator's *next* freehand stroke
would begin with a segment jumping from wherever they last let go.

So the trail is not cleared by an event. [`sync`] is called once per frame with
the gesture machine's own answer to *"is a freehand markup drag in flight?"*
([`crate::canvas::gesture::GestureState::active`]), and the trail exists
exactly while that answer is yes. Every one of the four endings is covered by
the same line, because all four make `active()` answer `None` — there is no
restore step to miss, and a dropped frame costs nothing.

### It is asked BEFORE the machine advances, and that is not a detail

`GestureState::update` clears its own drag on the frame it reports
`Phase::Complete`. An `active()` read *after* it therefore answers `None` on
exactly the frame the release arrives — so the trail would be discarded a few
lines before the arm that commits it, and every stroke would author only the
two points `egui` happened to report on that last frame.

**The symptom is specific and only the running binary shows it.** The
harness walks a drag in eight increments across a 1584 x 1224 pt sheet; read
in the wrong order the trace says `markup-commit kind=Ink raw=2 kept=2` — a
two-point stroke, which is a straight line between the ends of a gesture the
operator drew freehand. Every unit test in this file passes either way,
because they all call [`drag`] directly and none of them can see the order
`canvas::interact` calls two functions in. That is a whole defect class: a
wiring order a green suite cannot express. `ui-verify`'s `raw=` / `kept=`
assertion is what catches it.

Read first, the answer is the state the *previous* frame left, which is what
*"is a stroke in progress?"* actually means.

Cheap, too: one `egui::Id` lookup per frame, and an early return when there is
no trail, which is every frame nobody is drawing on.

---

## 3. SIMPLIFICATION — what was measured, and the rule the tolerance comes
## from

A raw pointer trail is hundreds of points. Every one of them is written into
`/InkList` as two `Real`s **and** into the appearance stream as a `l` operator,
so the cost is paid twice in the file and again on every render.

### 3.1 What is actually captured

One point per frame in which the pointer **moved**. Frames in which it did not
are dropped at capture, and that is not an optimisation: `egui` reports
`dragged` on every frame the button is down, so a pointer held still for two
seconds contributes ~120 identical points. That is the `canvas-pointer`
lesson — *fifty identical trace lines in nine seconds from a stationary
pointer* — arriving as file bytes instead of log lines, and a run of identical
points is also what turns [`super::action`]'s zero-extent guard into the only
thing standing between a held button and a 1-point blob.

### 3.2 The tolerance is derived from the pen, not chosen

The tolerance is a **quarter of the stroke width**, and it is not a feel:
Ramer–Douglas–Peucker guarantees that no removed point lay further than ε from
the line that replaced it, so ε is a bound on how far the *drawn centreline*
can move. A stroke drawn at width *w* has a half-width of *w*/2. Setting ε to
**half of that half-width** means the simplified centreline stays strictly
inside the body of the stroke the raw trail would have drawn: no pixel of the
mark can move outside the mark. That is the strongest statement available
about a lossy simplification.

At the shipped [`super::PEN_WIDTH_PTS`] of 2 pt that is
[`SIMPLIFY_TOLERANCE_PTS`] = **0.5 pt**, which is what §3.3 measures.

#### The tolerance FOLLOWS the pen, and a `const` cannot follow anything

The pen width is an operator control from 0.25 to 12 pt, so the derivation
has to be read per stroke: [`drag`] calls
[`super::pen::Pen::simplify_tolerance_pts`], the derivation lives on `Pen`
beside the width it derives from, and
`tests::the_guarantee_holds_at_every_width_the_operator_can_set` asserts the
bound at both ends of the operator's range rather than at the shipped middle.

What a fixed ε costs, at the thin end, is silent and is not a rounding
difference: a 0.25 pt pen — the width that exists to match a CAD sheet's own
linework — has a 0.125 pt half-width, against which a fixed 0.5 pt ε is
**four times** too loose. The centreline is then free to leave the stroke
entirely and the operator gets a curve they did not draw, while the claim
*"no pixel of the mark can move outside the mark"* stays true for exactly one
pen.

**The rule is not about ink.** A constant derived from another value is safe
exactly as long as that value is also a constant. The moment the input
becomes settable, the derivation stays pinned to the old input and *nothing
about it looks wrong* — the expression still names the right thing. Writing
the rule in a comment is not enough; only a test that varies the input can
notice.

### 3.3 What it measures out at

Two measurements, both reproducible, and neither of them a feel.

**Synthetic**, from `tests::the_measured_retention_at_the_shipped_tolerance` —
a 240-point trail sampled at 60 Hz along a hand-drawn-shaped curve: a sweeping
arc carrying a 1.2 pt lateral wander (the hand leaving the line it meant to
draw) and a +/-0.3 pt per-sample jitter (pointer quantisation between one frame
and the next). Run it with `--nocapture` and it prints the table it asserts:

| tolerance (pt) | points kept, of 240 | measured max deviation (pt) |
|---:|---:|---:|
| 0.125 | 154 (64 %) | 0.125 |
| 0.25 | 91 (38 %) | 0.249 |
| **0.5 — shipped** | **31 (13 %)** | **0.477** |
| 1.0 | 16 (7 %) | 0.934 |
| 2.0 | 10 (4 %) | 1.767 |

So the shipped tolerance discards **87 %** of a realistic trail, and the worst
any point of the original moves is **0.477 pt** — inside the 0.5 pt bound, and
well inside the 1 pt half-width the bound was derived from. That is the whole
claim of section 3.2 measured rather than argued.

The same test walks the sweep and asserts the two properties that make the
table mean something: **retention falls monotonically** as the tolerance rises,
and **the measured deviation never exceeds the tolerance** — which is RDP's
guarantee, checked rather than assumed, and which is what licenses deriving the
tolerance from the pen at all.

**The first version of that fixture was wrong and its numbers were too
good**, which is worth recording because it is how a measured claim goes bad.
It offset both disturbances along `(sin, -cos)` — the arc's **tangent** — so
they only re-spaced the samples along a path whose shape they never changed,
and RDP removed them almost for free: 17 points kept at 0.5 pt and only 33 at
0.125 pt, which is a suspiciously flat response to a sixteen-fold change in
tolerance. It was caught by computing the retention independently and noticing
the answer was better than the input deserved. The fixture now offsets
radially, which for a circular arc is the normal, and the test carries a
`worst > tolerance / 2` assertion so a future fixture that stops exercising the
bound fails rather than flattering it.

**Real, through the driven binary** — `ui-verify`'s
`markup_freehand_and_vertex_kinds` reads `markup-commit … raw=N kept=M` off the
trace of an actual OS-injected drag and reports both. A synthetic curve can be
argued with; a number the running program printed cannot. The trace line is the
reason that measurement is possible at all: without `raw=` beside `kept=`, a
build whose simplification did nothing would emit a line indistinguishable from
one whose simplification worked perfectly.

### 3.4 Why the decision is made in CANVAS space

The trail is captured in canvas space, simplified there, and only the surviving
points are converted to PDF user space. That is deliberate and it is what makes
*"the preview describes what will commit"* exact rather than approximate: the
preview draws the kept points and the file receives the same kept points, so
the two cannot disagree about which points survived.

It is also numerically the same decision as simplifying in page space, because
[`crate::viewer::canvas_to_pdf_space`] at scale 1.0 is an **isometry** — a
translation, a multiple-of-90° rotation and a Y flip, all of which preserve
distance — so a tolerance in canvas units is a tolerance in PDF points. If that
ever stopped being true, the identity of the kept set would still hold, which
is the property rule 4 actually asks for.

### 3.5 What is deliberately NOT done

**No smoothing, and no curve fitting.** Acrobat's ink is a polyline and so is
`pdfcer-core`'s builder (`ink()` emits `move_to` then `line_to`, §12.5.6.12), so
a Bézier fit here would be a shape the appearance stream cannot express and the
preview would be the only place it existed. **No minimum-step filter in screen
space**, either: it would make the captured detail depend on the magnification
at the moment of drawing, so the same gesture at 8× and at 0.5× would author
different geometry — the zoom-dependence
[`crate::canvas::mapping`]'s header exists to keep out of this crate.

## Item notes

### `struct Trail`

Canvas space rather than page space, for §3.4's reason: the simplification
decides *which points survive* and it decides in the space the pointer lives
in, so the preview and the file receive the same surviving points rather than
two derivations that agree by construction until they do not.

### `fn simplify`

Returns the subsequence of `points` that survives at `tolerance`, always
including the first and last. The guarantee — and the reason
[`SIMPLIFY_TOLERANCE_PTS`] can be derived from the pen rather than tuned — is
that **no removed point lay further than `tolerance` from the segment that
replaced it**, which bounds how far the drawn centreline can move.

Iterative rather than recursive on purpose. The recursive form is shorter and
its depth is the *number of retained points*, which for a long slow stroke is
in the hundreds; a canvas that overflowed its stack because somebody drew a
spiral would be a spectacular way to lose an unsaved document. The explicit
stack costs four lines.

`tolerance <= 0` returns the input unchanged rather than looping, and a run of
fewer than three points has nothing to remove.

### `fn distance_to_segment`

Segment, not infinite line: RDP's guarantee is about the polyline that
replaces the removed run, and a point beyond an endpoint is further from the
*segment* than from the line it lies on. Using the line would under-report
exactly at a hairpin, which is where a hand-drawn stroke doubles back and is
the one place the operator can see the difference.

A degenerate segment (`a == b`) falls back to the distance to `a`, which is
the correct answer and avoids a division by zero.

### `fn hand_drawn_trail`

The two disturbances are the fixture, and they are chosen to sit on
**opposite sides of the tolerance** so that it is asked a question it
could get wrong in either direction:

| component | amplitude | what it stands for | what must happen to it |
|---|---|---|---|
| a slow undulation, 7½ cycles across the stroke | **1.2 pt** | the hand wandering off the line it meant to draw | **kept** — it is above the pen's 1 pt half-width, so it is visible in the mark and is detail the simplification is obliged to preserve |
| a fast per-sample jitter | **±0.3 pt** | pointer quantisation and tremor between one frame and the next | **removed** — it is below the 0.5 pt tolerance, so no pixel of the drawn stroke moves when it goes |

A smooth arc alone would flatter the tolerance: it would simplify to
almost nothing and prove only that RDP works on lines. The jitter is what
makes the retention figure in §3.3 mean something, because it is the
component a real trail is mostly made of.

The jitter is a deterministic hash of the sample index rather than a
random number, so the measured figures in §3.3 are reproducible: a
measurement quoted in prose that changes between runs is a measurement
nobody can check.

### `fn the_measured_retention_at_the_shipped_tolerance`

§3.3's synthetic measurement, asserted rather than quoted, plus the two
properties that make the tolerance a *rule* instead of a number:

1. **The deviation never exceeds the tolerance**, at any tolerance in the
   sweep. That is RDP's guarantee, and it is what licenses deriving the
   tolerance from the pen's half-width — if it did not hold, "the
   centreline stays inside the stroke" would be an unsupported claim.
2. **Retention falls monotonically** as the tolerance rises. A build whose
   simplification was subtly wrong — comparing against the infinite line
   rather than the segment, say — can still pass a single-point retention
   assertion and fails this one.

The exact retention figure is printed in the assertion message so that a
reader who changes the pen can see what it did rather than only that it
broke a bound.

### `fn a_hairpin_keeps_its_apex_where_an_infinite_line_would_lose_it`

A stroke that doubles back on itself has its apex *on* the line through
its two ends, so a build measuring against the infinite line would compute
a deviation of zero and delete the apex — turning a fold into a straight
line the operator never drew, and doing it silently.

### `fn a_stationary_pointer_contributes_one_point`

§3.1, at the capture end. Without the duplicate filter a held button emits
~60 identical points a second into `/InkList`, and the resulting run of
identical coordinates is also what [`super::action`]'s zero-extent guard
would then be the only defence against.

### `fn every_way_a_drag_can_end_discards_the_trail`

§2's argument, asserted through the one line that implements it. The
interruption row is the one an event-hooked implementation gets wrong: the
window loses focus, `egui` stops reporting the drag without ever reporting
a stop, and the next stroke would begin with a segment jumping from
wherever the operator last let go.

### `fn the_preview_is_the_simplified_trail_that_will_be_authored`

Rule 4's honesty requirement, asserted where it can be: the value handed
back for painting is the *same* `simplify` output the release converts and
authors. A build that previewed the raw trail would show a mark that
visibly changed shape at the moment of release, which is precisely what a
pre-commit affordance exists to prevent.

### `fn the_guarantee_holds_at_every_width_the_operator_can_set`

The test above is true and insufficient: it asserts a relation between
two *constants*, so it passes unchanged whatever the operator's pen is
set to, including a build in which the tolerance stays welded to the
default 2 pt while the width ranges from 0.25 to 12 pt.

At the thin end that is not a rounding difference: a 0.25 pt pen has a
0.125 pt half-width, and a fixed 0.5 pt ε is **four times** it — so
Ramer–Douglas–Peucker is free to move the centreline clean outside the
stroke, and the operator gets a curve they did not draw. §3.2's whole
claim is *"no pixel of the mark can move outside the mark"*, and a
constants-only assertion leaves that claim true for exactly one pen.

This asserts it across the range, which is the only form that can notice
the input being settable.

### `fn the_shipped_constant_matches_the_shipped_pen`

The weld between the two halves of this module's tolerance story:
[`SIMPLIFY_TOLERANCE_PTS`] is the value §3.3's measurement table was
measured at, and [`super::super::pen::Pen::simplify_tolerance_pts`] is
what the running code reads. If they ever disagreed, the table would be
documenting a tolerance the shipped build does not use — a measurement
that is still *reproducible* and no longer *about* anything.

The two are separate on purpose (the constant names a value, the method
derives one), so this is what stops them being separate in effect.
