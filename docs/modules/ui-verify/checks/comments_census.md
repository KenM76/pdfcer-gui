# `ui-verify/checks/comments_census`

`checks::comments_census` — reading the Comments panel's count **honestly**,
for every check that uses it as an oracle.


`save_copy_round_trip` and `undo_redo_round_trip` both use the Comments
panel's per-frame census as their proof that an annotation reached the
document. On the driven sweep of 2026-09-05 **both failed, in the same
words**, and the sweep report treated them as *"one application defect with
two independent witnesses"*:

> *"THE COMMENTS PANEL DOES NOT SEE THE ANNOTATION THAT WAS JUST AUTHORED:
> it listed 12 before the drag and 12 after it. The engine traced
> `add-markup`, so the annotation IS on the session."*

The panel was fine. The witnesses were not independent — each carried a
**copy** of the same eight-line helper:

```ignore
fn listed(trace: &Trace) -> Option<usize> {
    trace.last(COMMENTS_EVENT)?.get_usize(LISTED_FIELD)
}
```

`Trace::last` searches the whole capture. A docked pane that is not the
front tab **draws nothing and therefore traces nothing** — this project's
own `RESUME.md` records that as a finding — so when a persisted
`userdata/layout.ron` put Document properties in front of Comments, the
panel fell silent three hundred frames before the drag and its last census
stood for ever. Both checks read that fossil, twice, and subtracted it from
itself.

⇒ **Two checks sharing a helper can share a defect; two checks sharing a
COPY of a helper are worse, because the duplication is what makes the two
failures look like corroboration.** One module now, imported by both, so a
future repair cannot land in one and miss the other.

# What a caller gets instead

| | |
|---|---|
| [`Census::since`] | a census the panel published **after** a named cause, or `None` |
| [`refresh`] | the same, and if the panel is silent, **puts it back in front** and asks again |
| [`baseline`] | enter the mode, front the panel, and report the starting census |

`None` is never folded into a number. *"The panel said nothing since the
edit"* and *"the panel said the same number"* are different verdicts about
different subjects — the first is a layout fact and reports SKIP, the second
is a defect and reports FAIL — and the whole of the 2026-09-05 misreport was
the first being printed as the second.

# What a census asserts, and what it does not

The line carries no annotation identity, so a caller cannot name the object
it is looking for. What it can do, and what [`Census::describes_one_more`]
does, is assert the **shape** of the change a freshly drawn markup makes:

* `listed` rises by exactly one — a row appeared;
* `with_note` does **not** move — the new row has no `/Contents`, because
  nothing in this shell can write words onto a shape at the moment it is
  drawn;
* `authors` does **not** move — nor a `/T`.

A build whose census moved because something *else* changed (a widget
stopped being excluded, a reply was counted, a ce dimension was
reclassified) fails that conjunction, which is what stops the assertion
being the vacuous *"a number went up"*. The caller states the fixture's
starting census in its own report so the arithmetic is auditable from the
output alone.


| run | condition | result |
|---|---|---|
| 1 | `userdata/` cleared, unmodified build | `save_copy_round_trip` **PASS**, 12 → 13 |
| 2 | the same | `undo_redo_round_trip` **PASS**, 12 → 13 → 12 → 13 |
| 3 | **the hostile layout seeded by hand** — `mode:review`'s right stack rewritten with `active: 4`, so `file.document_properties` is the front tab and Comments is in the overflow, which is the exact state of the sweep that produced the two false reports | both **PASS**, each printing *"the Comments panel published no census … Bringing it forward"* and then *"the panel came forward on a press of `markup.comments`"* |
| 4 | **a planted application defect** — the panel's listing truncated to the row count of its first frame, a census frozen on something that never moves | both **FAIL**, naming the arithmetic: *"`listed` 12 → 12, `with_note` 12 → 12, `authors` 12 → 12 (expected 12 → 13, 12 → 12, 12 → 12)"*, exit 1 |

Run 3 is the one that matters most, because it is the only one that could
have shown the repair to be cosmetic. Run 4 is what stops the repair being
*"make the check pass"*: with the panel genuinely blind, both checks still
say so, and they say it about the panel rather than about the save or the
undo.

Run 3 also corrected the repair **while it was being made**. The first
version anchored the baseline on a mark taken before the mode click, and
that was still wrong: a launch restores its remembered mode, the harness
clicks a segment out from under it, and a perfectly fresh census from the
*previous* mode satisfies the anchor. The driven run showed the baseline
coming from a Read frame while the check believed it had measured Review.
The anchor is `mode-changed … to=<mode>` because of that run and not
because of any reasoning that preceded it.

# `filtered=` is checked, and the reason is this panel's own rule

`panels::comments` gained filtering by author, type and has-words on
2026-09-05. Its founding discipline is that *"nothing is silently
omitted"*, and a filter is an omission the **operator** caused. The panel
states it on screen; since the same day it states it on the trace as
`filtered=1`, and every function here refuses a filtered census rather than
comparing it. A narrowed list is not a census of the document, and reading
one as though it were is exactly how a check reports a document as having
lost annotations it still has.
