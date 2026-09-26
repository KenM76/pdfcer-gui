# `ui-verify/checks/comment_note`

`a_note_can_be_written_onto_a_shape_that_exists` — **the Comments panel
stopped being a viewer, and this is what says it stayed that way.**

# The gap this closes, in the engine's own words

`pdfcer-core`'s reply to this shell's blocker (`Pass 154.0`) lists what a
read-only comment list costs a reviewer, and none of the four is an edge
case:

> comment a shape you just drew, comment a highlight you just swept, fix a
> typo in your own comment, answer someone else's.


# ★★★ Why this cannot be a unit test, in the specific

The chain is five links and each has its own passing test:

| # | Link | Its own test |
|---|---|---|
| 1 | the row decides the annotation is note-editable | yes (`note_controls` is a `match` over `CommentRow`) |
| 2 | *Add note* opens the draft | yes (`NoteDraft`'s suite) |
| 3 | Save raises `AnnotAction::SetNote` | no — that is a widget, and a widget's effect is observable only in a window |
| 4 | the apply arm resolves the author and calls the engine | partially |
| 5 | the engine writes `/Contents` and the panel reads it back | yes, on both sides separately |

Link 3 is the one that has burned this project repeatedly, most recently
**on 2026-08-28 itself**: the O51 scale switches were written into an arm
that never runs, compiled, read correctly, and drew nothing, with every unit
test green. *"Nothing tested that the control is on screen."* This check is
that test for this control.

# What it does

`PDFCER_DIAG_INVOKE` supplies the two commands at launch rather than clicking
for them — Review mode, the Comments panel, and the rectangle tool — because
the subject here is the note, not the ribbon, and three extra clicks are
three extra ways for the check to fail at something it is not testing.
`markup_rectangle` is the check that proves those controls are reachable by
mouse.

| Phase | Does | Expected |
|---|---|---|
| A | drag a rectangle on the page | `add-markup` — there is now one annotation |
| B | read the panel's census | `comments-panel listed=1 with_note=0` |
| C | click *Add note* | `comments.note_box` and `comments.note_save` appear |
| D | type four letters into the box | — |
| E | click *Save note* | `set-markup-note-applied … keys=…Contents…` |
| F | read the census again | `with_note=1` |
| G | select the shape, Ctrl+C, Ctrl+V | `paste-markup … note=true` |

# ★★★ Phase G exists because THIS CHECK CREATED THE DEFECT IT GUARDS

The object clipboard copies a markup by reading it into a `MarkupSpec` and
authoring a new one. That is lossless only for what a spec can express — and
the note this check writes in phase E **is not expressible in a spec**. So
on the day the note editor shipped, copying a commented cloud and pasting it
produced an anonymous one, and **nothing on the page would show it**: the
words live in a pop-up this shell does not draw.

⇒ The general form, worth carrying: **a copy implemented as a re-author
loses ground every time the authoring side gains a key**, silently, in a
direction no screenshot can see. This phase is the tripwire on that, and it
is here rather than in `object_clipboard` because this is the check that can
produce an annotation with a note to copy in the first place.

# ★★ Phase F is the assertion that matters, and B is what makes it mean
anything

`set-markup-note-applied` says the engine accepted the call. `with_note=1`
says **the panel read the words back out of the document**, which is the
only evidence that a reviewer would see anything. Both are needed and
neither is sufficient: a build that wrote `/Contents` into a session the
panel does not read from would pass the first and fail the second, and that
is not a hypothetical — the Comments panel reads `doc.session.view()`
precisely because reading the file on disk showed nothing until a save.

Phase B pins `with_note=0` first, so F cannot be satisfied by a fixture that
arrived with a commented annotation already on it.

# ★ The word typed is TAIL

Four letters, all of them already in the closed `vk` list (`T`, `A`, `I`,
`L`), and a word a drafter would recognise in a failure message. The list is
deliberately closed and grown one key at a time with a reason; spelling a
word out of what is already there is cheaper than adding two constants to
type something prettier.
