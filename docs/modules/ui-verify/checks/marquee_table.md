# `ui-verify/checks/marquee_table`

`a_marquee_over_a_table_takes_its_text_as_well_as_its_lines` — **the
operator drew a box round a table and could not move it.**

# The report


> *"I can't box select the tables in the left or right top corners using the
> mouse — it only picks up the lines of each table, so I can't drag the
> entire thing and move it somewhere else, or cut/copy and paste it
> elsewhere."*

## What "only the lines" would mean, and why it needs measuring

A CAD-exported table is two kinds of object drawn in one place: **paths**
(the rules and the border) and **text** (every cell's contents). They are
separate objects in the content stream and nothing in the file says they
belong together.

So *"it only picks up the lines"* has three candidate causes and they want
opposite fixes:

| | what would be wrong | how this check tells |
|---|---|---|
| the marquee excludes **text objects** | the hit test, or a filter above it | the selection has paths and no text |
| the marquee is **`Enclosed`** and the table touches the page edge, so it cannot be surrounded | the gesture, not the hit test | a marquee that fits INSIDE the page selects both |
| the selection is right and the **drag** refuses a mixed set | `canvas::moving`, not selection at all | both kinds selected, and no move line |

⇒ This check settles the first two by drawing a band that fits comfortably
inside the page around a table that does **not** touch the edge, and asking
what came back. A green result here moves the investigation to the third,
which is a different module and a different report.

It is deliberately NOT a screenshot. Two objects selected and one object
selected draw the same blue outline round the same table; the distinguishing
fact is the census, and `canvas-selection` carries it.

## The fixture is the operator's own drawing

Copied to scratch first. The check writes nothing — a marquee is a read —
but the application persists layout and recent-file state beside whatever it
opens, and this project's standing rule is that the suite's side effects do
not land in the operator's own folder.

## Item notes

### `const INVOKE`

The first run of this check drove a band to screen y=2 — above the canvas
entirely — because the file is ten pages shown continuously and the view had
been scrolled by the layout it inherited. `aim` faithfully computed where the
table WOULD be and the drag went there, off the canvas, selecting nothing.

A region off the top of the canvas looks exactly like a hit test that
excluded everything, which is the fourth instance of that shape in this
harness. Fitting the page first makes the aim a statement about the document
rather than about the scroll position the run happened to start from.

### `const BAND_FROM`

The previous origin was `(14, 618)` — just outside the table, "comfortably
inside the sheet", and **on top of an object**. Three runs of this check
reported *"THE BAND SELECTED NOTHING AT ALL"*, which reads as a hit test
that excluded everything. It was not. The trace carried
`selection-set page=0 object=23 via=press` and **no marquee line of any
kind**: the press had selected the object under it and the drag had become a
MOVE.

`canvas::presspick` documents that behaviour and its first stated
non-disturbance is the rule this check broke: *"A press on empty paper still
marquees."* Pressing on ink does not.

And "empty" is a much larger radius than it looks. The pick tolerance is
`SELECT_SCREEN_TOLERANCE_PX` converted to page units, so at the fitted zoom
this check drives (about 0.38×) a **4-pixel** screen tolerance is over **ten
page points**. The old origin sat 6 pt from the sheet border — visually in
the margin, and inside the catch radius.

Chosen by rendering page 1 at scale 1.0 (1 px = 1 pt) and looking: this
point is in the blank field below the INSPECTION STATUS table and left of
the isometric view, about **80 pt** from the nearest ink in any direction.

### `const BAND_TO`

Right-to-left, so this is a **crossing window** and takes anything it
touches. That is the point: it is the gesture `OPERATOR_REQUESTS.md` O88
added, and it is the only one that can reach this table at all.

An **enclosing** band cannot be driven here and the reason is the operator's
own complaint rather than a harness limitation: to surround a table hard
against the sheet edge the band must start outside the page, and every
corner from which it could be started is on ink. So the window direction is
deliberately **not** driven by this check, and that absence is reported
rather than left for a reader of a PASS to assume away.

### `const MODE`

Asserted as well as the selection census, and the pair is the point. A
build that ignored the drag direction and ran `Enclosed` for everything
would select nothing here and fail on the count — but so would a build
whose hit test was broken outright, and the two want opposite fixes. This
line separates them.
