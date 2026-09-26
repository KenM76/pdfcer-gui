# `panels::properties::geometry::annot` — the **annotation** arm of the
geometry section: X, Y, W, H and, since 2026-09-07, **Angle**

## Why this is a separate file

R2, and the seam was already drawn in prose before it was drawn in the file
system. `geometry.rs` served two subjects — a page-content object and a
markup annotation — behind one heading and one draft type, and its own
header describes them as *"two engine verbs that take different shapes"*.
The shared parts (the draft, the seed, the arithmetic, [`super::plan`] and
[`super::annot_plan`]) stay in the parent, where both arms can reach them
and where their tests already live. What moved is only the half that talks
to `EditSession`'s **annotation** verbs.

⇒ The split is deliberately NOT "UI here, arithmetic there". The parent
keeps `annot_plan`, because the whole reason that function is pure is so it
can be tested without a document, and moving it beside its caller would
have put it back in a file that needs `OpenDoc` to compile a test.

## What this arm does that the content arm does not

| | content | annotation |
|---|---|---|
| move | `move_nodes` on the object's anchors | `move_annotation(id, dx, dy)` |
| resize | `resizing::action` factors | `resize_annotation(id, anchor, sx, sy, opts)` |
| **turn** | — | `rotate_annotation(id, pivot, degrees)` |
| lock | no such thing in a content stream | §12.5.3 bit 8, and every field is greyed |

The **turn** row is the one this file is newest for, and the operator asked
for it in as many words: *"the angle should be editable from the
properties."* Read [`super::GeometryDraft::angle_delta`] before changing
anything about it — the field is absolute, the verb is a delta, and the
conversion has a normalisation in it that is not decoration.
