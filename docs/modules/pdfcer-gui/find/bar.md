# `find::bar` — the Find overlay's controls, and the keys they own

One compact box **floating over the top-right of the page**, which is
where Acrobat Reader, Chrome's PDF viewer and Edge's all put theirs:

```text
                         ┌────────────────────────────────────────────┐
                         │ Find [ total       ] ⏴ ⏵ 3 of 47  Options ⏷  × │
                         └────────────────────────────────────────────┘
```

[`super`]'s header carries the placement note. The short version is that an
overlay consumes **no layout space**, so opening Find does not resize the
canvas and therefore does not re-fit the page under the operator's eyes.
That is not a theoretical advantage: a docked version of this bar was built
first and driven, and pressing Ctrl+F under Fit page took the zoom from
85 % to 81 % and back again on close — a page that jumps every time you go
looking for a word on it.

## ★ The four search options live in a menu, not on the bar

Match case, Whole word, Wildcards and the whole-word **rule** are behind
the `Options` button. Three reasons, in order of weight:

1. **An overlay has to be narrow**, because it covers the page. Laid out in
   a row, those four controls are wider than the search field, the step
   buttons and the readout put together; the box would span most of the
   window and hide the thing being searched for.
2. **It is what the reference product does.** Reader's Ctrl+F box is a
   field, two arrows and a settings dropdown holding *Whole words only* and
   *Case-Sensitive*. An operator arriving from Acrobat finds the options
   where they left them.
3. **The rule chooser can then appear and disappear without moving
   anything.** It is meaningful only while Whole word is on (see
   [`options_menu`]), and inside a menu its arrival costs a row of a popup
   rather than shifting every control to its right — which, on a bar the
   operator is aiming at, is the difference between a tidy layout and a
   mis-click.

## ★ The width is fixed, and nothing on the bar may move

The box is anchored by its **top-right corner** to the canvas viewport, so
its left edge is `right − width`: any change of width moves every control
on it. The search field, the readout and the buttons therefore all have
reserved widths ([`FIELD_WIDTH_PTS`], [`READOUT_WIDTH_PTS`]) and the
options are in a menu — so `3 of 47` becoming `No matches`, or a search
finding nothing at all, cannot slide the ⏴ ⏵ buttons out from under the
pointer between two clicks. That is the same reason the status bar reserves
a width for its zoom readout.

The height is fixed too, at [`ROW_HEIGHT_PTS`], but for a much weaker
reason: layout tidiness. **R128 does not reach this surface.** That rule is
*a panel whose size feeds a fit-to-viewport computation has a fixed size*,
and an `egui::Area` feeds no such computation because it consumes none of
the layout. Docking the bar is what would have made R128 bind, and that is
one of the reasons it is not docked.

## ★ The three keys, and the one that is shared

| key | while the field has focus | otherwise |
|---|---|---|
| Enter | search, or step to the next hit | belongs to whatever has focus |
| Shift+Enter | search, or step to the previous hit | as above |
| **Escape** | close the bar | **belongs to the canvas** |

Escape is the interesting one, because three surfaces want it: a canvas
drag in flight wants to abandon itself, the selection ladder wants to
ascend a rung, and this bar wants to close. There is no arbitration code,
and there does not need to be — `crate::canvas::interact` already reads
Escape as `!ctx.text_edit_focused() && …`, so while the operator is typing
here the canvas is not offered the key at all. This file takes it only
under the same condition, from the other side.

The consequence is worth stating because it looks like a gap: **Escape does
not close the bar after the operator has clicked on the page.** That is
deliberate. At that moment Escape is the selection ladder's, and a bar that
stole it would cost the operator the rung they were working in — the same
one-press-one-effect rule `canvas::interact` applies between the gesture
machine and the ladder. The close button and Ctrl+F are the routes out from
there, and both are visible.

## ★ What Enter does depends on whether the answer is still current

[`super::FindState::readout`] is the single test, and [`enter_intent`] is
the whole decision as a pure function of it:

| readout | Enter | Shift+Enter |
|---|---|---|
| `Idle` — nothing searched for what is in the box | **search** | **search** |
| `Stale` — the document has been edited since | **search** | **search** |
| `At` — there are hits | step **next** | step **previous** |
| `Empty` — searched, nothing found | **nothing** | **nothing** |

The last row is the one that needs defending. Re-running a search that just
returned nothing would re-extract the whole document's text — **350 ms on
the benchmark drawing, measured**; see [`super`]'s cost section — to
produce the same empty answer, and an operator leaning on Enter would do it
once per press. Nothing has changed since the search ran; if something had,
the readout would be `Stale` and the first row would apply.

## Actions, not mutations

Every commit leaves here as an [`Action::Find`]. The two exceptions are the
ones `crate::app::status`'s page box already takes and for the same reason:
the **text buffer** and the **option flags** are widget state, they describe
the control rather than the document, and deferring a keystroke to after
the frame would make typing lag by a frame.

## Where the strings are

[`crate::text::find`], all of them. Nothing here is a literal an operator
can read; `tools/gates/check-ui-strings.sh` is the mechanical half of that
rule and [`crate::text`]'s header is the reason for it.
