# `ui-verify/checks/marquee_table`

`a_marquee_over_a_table_takes_its_text_as_well_as_its_lines` — **the
operator drew a box round a table and could not move it.**

# The report


> *"I can't box select the tables in the left or right top corners using the
> mouse — it only picks up the lines of each table, so I can't drag the
> entire thing and move it somewhere else, or cut/copy and paste it
> elsewhere."*

## ★★★ What "only the lines" would mean, and why it needs measuring

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

★★ It is deliberately NOT a screenshot. Two objects selected and one object
selected draw the same blue outline round the same table; the distinguishing
fact is the census, and `canvas-selection` carries it.

## The fixture is the operator's own drawing

Copied to scratch first. The check writes nothing — a marquee is a read —
but the application persists layout and recent-file state beside whatever it
opens, and this project's standing rule is that the suite's side effects do
not land in the operator's own folder.
