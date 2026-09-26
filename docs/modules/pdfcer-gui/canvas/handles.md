# `canvas::handles` — eight grips plus move, and the cursor over each

`GUI_ROADMAP.md` Phase 1.3: *"Eight handles plus move, per the convention
every drawing tool shares. Cursor changes over a handle, over a movable
object, over the canvas."*

## Rule 4 says these are welcome, and says exactly why

`D:\Dev\FeatureRequests\pdfce_FeatureRequests\README.md`, fourth clause of
the disclosure rule:

> **A pre-commit affordance is not content marking.** A snap indicator, a
> hover highlight, a rubber-band, a selection handle — these are the
> *cursor*; they describe what is about to happen and they are welcome.
> What is forbidden is styling content that has **already been applied**
> as though it were pending.

So the grips are drawn, and nothing else is. No badge, no tint, no dashed
"provisional" layer over content, nothing that would make a screenshot of
the editing canvas differ from a screenshot of the same document saved and
reopened. The grips vanish with the selection because they are the
cursor's statement about the selection, not a property of the page.

## ★ These are SCREEN-space rects, deliberately, and it is the one place

Everything else in `canvas/` past [`crate::canvas::mapping`] is page
space. A grip is the exception and must be: it is a **fixed number of
screen pixels**, because it is something the operator has to hit with a
mouse, and a grip sized in page units would be a 3-pixel speck at fit-page
and a slab the size of the object at 800%. It sits on the *output* side of
the boundary — the selection's bounds are converted to screen once, by
[`crate::canvas::mapping::PageMapping::rect_to_screen`], and the grips are
laid out on the result.

## What a grip drag does today, stated so it is not mistaken for an oversight

[`Grip::Move`] is live: a drag on the selection's body moves it, through
`EditSession::move_objects`.

The **eight resize grips change the cursor and consume the drag, and
perform no edit yet.** That is not a placeholder left in by accident, and
the reason is worth writing down rather than rediscovering: `pdfcer-core`
has `move_object`, `move_objects`, `move_subpath`, `move_node`,
`move_nodes` and `move_handle` — and **no scale or resize verb for a
vector object at all**. `GUI_ROADMAP.md` 1.2 (*"move and resize anything
carrying a `/Rect`"*, `FEATURES.md:208`) is the row that gives them one,
and it covers annotations, form widgets, redaction marks, links and ce
dimensions — objects whose size is a rectangle in the file rather than a
consequence of their path data.

Consuming the drag is the deliberate part. Without it, a drag that started
on a grip would fall through and become a **marquee**, so aiming at a
resize handle would silently replace the selection the operator was trying
to resize. Swallowing the gesture is the honest behaviour until the verb
exists.
