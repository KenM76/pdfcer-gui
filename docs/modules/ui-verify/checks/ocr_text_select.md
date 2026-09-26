# `ui-verify/checks/ocr_text_select`

`text_on_a_scan_can_still_be_swept_over_the_image` — **the OCR layer must
not lose to the picture it sits on.**

# The report

Ken, 2026-09-01: *"I can't seem to copy and paste text we have OCRed."*

## ★★★ It was caused by a feature that shipped seven hours earlier

His own earlier ask — *"select images so we can copy and paste them"* —
landed that morning as a Read-mode arm: a click on an image selects it, and
clears the text selection, because the operator has just said they mean the
picture.

It was narrowed to **images only**, on an argument that was right about the
document it was written against: a CAD sheet has a path under the pointer
almost everywhere, so admitting paths would have made text unreachable. The
narrowing does nothing for the case that broke, because **a scanned page IS
one image**, edge to edge — every click hits it, and an OCR layer is
invisible text lying exactly on top.

⇒ The one document class where selecting text matters most is the one where
the arm swallowed it.

## ★★ Why this check is not "does OCR work"

Three things were ruled out **before** any code was changed, by measurement
rather than by reading:

| | verdict |
|---|---|
| does the engine extract an OCR layer? | **yes** — 13 invisible codes off his own file |
| do the recognised words carry geometry? | **yes** — a 6 × 6 pt box on the first word |
| does the shell's own extraction differ? | no — same options, same funnel |

So the subject here is precisely the **precedence** between two things that
both claim the same pixel, and the oracle has to distinguish them: a click
that selects the image traces `via=read-image`, and one that sweeps text
traces a text selection. Both are "something was selected".

## The sequence

| # | step | oracle |
|---|---|---|
| A | open a recognised scan in Read mode | the page draws |
| B | click a point **on a recognised word** | a text selection, and NOT `via=read-image` |
| C | click a point on the same page with **no** word under it | `via=read-image` — the picture still wins where the words are not |

★★★ Step C is the control point and is the half a careless fix would break.
Making text win everywhere would take the image feature away again, which is
the same defect facing the other direction — and a check asserting only B
would pass against exactly that.
