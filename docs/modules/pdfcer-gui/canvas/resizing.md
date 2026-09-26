# `canvas::resizing` — what the eight resize grips commit

## ★★★ The verb is `transform_objects`, and it is kind-agnostic

`EditSession::transform_objects` wraps each object's operator run in
`q <cm> … Q`. **That never looks at an operand**, which is what makes it
kind-agnostic — not a match arm per kind that somebody has to remember to
extend. A text run and an image therefore resize exactly as a path does
(neither has nodes to move, and neither needs any), and a selection of N
objects is **one** call, one command, one undo entry.

**Stroke width is not scaled on page content**, and that is a decision
rather than a consequence: on a CAD drawing a line weight is a *drafting
standard* — 0.25 mm is 0.25 mm whatever size the detail is — so keeping it
is right far more often than scaling it would be, and it is what every
drafting package does. It is nonetheless something pdfcer decided and the
operator did not, so it is **disclosed** ([`crate::text::resizing`]) rather
than assumed.

★★ **The matrix is PAGE space and nothing else.** `cm` composes into the CTM
in force at that point in the stream — the object's *user* space — so the
engine emits `X = CTM × M × CTM⁻¹` per object from that object's own captured
CTM. A selection spanning two local spaces gets two different `cm` operands
for one gesture and both land where the operator pointed. Passing anything
but page space from here would be right only where an object's CTM happens to
be the identity and **silently wrong at every scale the producer left in
force**.

---

## The arithmetic, and why it is not written out here

**Scaling about an anchor is moving every point.** For an anchor `a` and
factors `(sx, sy)`:

```text
p' = a + (p - a) * (sx, sy)
```

`Matrix::scale(sx, sy).about(a)` is `translate(a) × M × translate(-a)`,
which is that expression exactly — so the map is stated once, by the crate
that owns matrices, rather than once per point here. A shell keeping its own
copy would be a second derivation of one answer in coordinate space, which
is the shape every silent defect this project has met there has had.

One gesture is **one call, one command, one undo entry** — this project's
standing rule for a gesture (`canvas::moving`'s §1), and the thing a
per-object loop would break both by producing N undo entries and by planning
each edit against byte offsets the previous one invalidated.

The operator's instruction: *"finish off phase 1 and phase 5. Get everything
unblocked on phase 5 — no excuses about slowness of feature from pdfcer as a
reason not to implement."*

## Why the arithmetic is here and not in `moving`

[`crate::canvas::moving`] is about a **displacement** — one delta applied to
whatever the rung named. This is about a **map**: every node goes somewhere
different, and the somewhere depends on where it started. Folding it in
would put two different shapes of answer behind one `MoveSubject`, and the
module that owns the ghost preview would have to branch on which.

## The ghost, and rule 4

An in-flight resize draws its **new outline**, not a tint over the old one —
`canvas::overlay`'s existing move ghost with a different transform. It is a
pre-commit affordance and therefore the *cursor*, which R8b's fourth clause
welcomes explicitly. Nothing is drawn onto the applied content, and a
screenshot of the page after a commit is a screenshot of the page as it will
save.
