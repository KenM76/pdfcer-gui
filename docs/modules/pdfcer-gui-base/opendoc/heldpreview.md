# `pdfcer-gui-base/opendoc/heldpreview`

## Item notes

### `const HELD_PREVIEW_MAX`

A wall-clock backstop, because the epoch test alone can fail to fire at all:
a render that fails, a page that will not rasterise, or any path that leaves
`page_texture_epoch` behind strands the hold, and the operator is left
looking at a selection-coloured tracing of their drawing with no way to clear
it. A stuck preview is worse than a late one — it is indistinguishable from a
corrupted document.

Four seconds is roughly four times the measured whole-page raster on the
operator's hardest drawing, so it cannot fire on a render that is merely
slow.

### `const HELD_PREVIEW_GRACE`

This is the only thing separating *"not applied yet"* from *"refused"*.
Actions are drained after the frame that raised them, so there is a real
window — one frame, ~16 ms — in which a hold is legitimate and the epoch has
not moved; there is also a state in which the epoch never moves at all,
because the engine refused the edit. By epoch the two are identical; by
elapsed time they are not remotely alike. 250 ms is fifteen frames at 60 Hz,
far longer than the real window can be and far shorter than a refusal stays
wrong for.

Getting it wrong ships a preview of a move that did not happen, over a
document that disagrees with it, for the full [`HELD_PREVIEW_MAX`] — a
picture of a lie rather than a picture that is late.

### `const CATCHING_UP_AFTER`

# Why a threshold rather than "whenever it is behind"

The picture is behind after **every** edit — for a few milliseconds on a
simple page, for a second or two on a dense one. A sentence that appeared
every time would flash on and off on every keystroke, and a status line that
flickers is one the operator stops reading. That costs every *other*
sentence the bar carries, which is a far larger loss than this one is a gain.

400 ms is past the point where a person notices a wait and starts wondering
whether the program heard them. Below it, saying nothing is the correct
behaviour and not merely the cheap one.

### `fn held_preview_to_draw`

Three conditions, in order, and each rejects a different way for a hold
to be wrong:

| test | what it rejects |
|---|---|
| the epoch moved | a gesture that was **refused** — nothing committed, so there is nothing to preview |
| the raster has not caught up | a picture that is already correct — drawing over it would be strictly worse than the real thing |
| it is younger than [`HELD_PREVIEW_MAX`] | a raster that will never arrive, leaving a preview nobody can clear |

There is a fourth state that deliberately draws: the frame **between**
the release and the commit. Actions are drained after the frame that
raised them, so for exactly one frame `edit_epoch` still equals
`captured_at_epoch`. Rejecting that frame would blink the preview off and
on again, which is the flicker this feature exists to remove.

### `fn retire_held_preview`

Separate from [`Self::held_preview_to_draw`] because that one takes
`&self` — it is called from the painter, which holds the document
immutably. This is called once a frame from `canvas::interact`, which
does not, and it exists so a dead hold does not sit in memory carrying
thousands of segments until the next gesture replaces it.

### `fn hold_preview`

Called on release, with the geometry the gesture last drew. Replaces any
previous hold outright: a second edit supersedes the first, and two
previews on screen would be two claims about one document.

### `fn page_is_catching_up`

# What this answers, and why it is not the same question as the hold

[`Self::held_preview_to_draw`] asks *"should this particular geometry
still be drawn?"* and only ever has an answer for a **canvas gesture on
a path**. This asks *"is the page the operator is looking at out of
date?"* and has an answer for **every edit in the program** — a colour,
a Bold press, a delete, a redaction mark, a page rotation, an undo.

⇒ That is the whole reason it exists. `OPERATOR_REQUESTS.md` O63 is
*"live preview for everything we do"*, and a drawn preview is only
possible where the shell holds the geometry. Where it does not, the
honest substitute is not a worse picture — it is **saying that the
picture is not the answer yet**, which is the third of the three options
the operator chose between and the one with no failure mode.

Deliberately silent under [`CATCHING_UP_AFTER`]: see that constant.

**Both sides of the comparison must come from the same counter.**
`edit_epoch` is incremented by the action modules; `page_texture_epoch`
holds a `PageEpochs` value written by `app::settle`. They are issued
independently, so an `EditScope::Page(other)` edit advances `edit_epoch`
without advancing this page's entry and the two pass each other for good
— deleting a page guarantees it, because `actions::pages` calls
`bump_all` and a `resize` in the same breath. Comparing across the two
counters therefore does not flicker, it **sticks**: *"the picture is
catching up"* stays on the status bar for the rest of the session over a
picture that is perfectly correct.

A unit test that sets both fields by hand cannot see this — equal or
adjacent values hold under either model. Only a test that edits a
*different* page exercises the divergence.
