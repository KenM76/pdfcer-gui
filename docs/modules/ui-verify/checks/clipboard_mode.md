# `ui-verify/checks/clipboard_mode`

`a_paste_review_may_not_do_says_so` — **the other side of the driven
sweep's finding A1: the chord gets through, and the refusal is a sentence.**


Said here, in its own header, rather than left for an absent result to
imply. That session worked headlessly by instruction: another track held the
machine's pointer and keyboard, and this harness drives both. **No line
below has been observed against a running binary.** It is registered so the
next sweep picks it up, and its first run should be treated as calibration
rather than as a verdict — this project has three recorded cases of an
articulate, plausible failure message being about nothing at all.

## What this is for


```text
chord-command      chord="Ctrl+C" id=edit.copy  via=clipboard-event
chord-command      chord="Ctrl+V" id=edit.paste via=clipboard-event
chord-not-offered  id=edit.paste mode=review
```

**Copy was offered in Review and paste was not.** The cause was
`app::modes::capability::offers_command`, which gates a chord on *"does this
mode show the tab that owns this command?"* — a proxy for *"may this mode do
this?"* that is wrong for a verb whose answer depends on what the operator
is pointing at. Paste lives on the Edit tab; Review is not shown it; so the
mode whose entire purpose is marking up somebody else's drawing could copy a
comment and had nowhere to put it.

The fix pushes the four clipboard chords through blind and lets
`app::dispatch::clipboard` decide per press, which it was already doing
correctly and had never been reached from Review to do.

## ★★★ Why THIS check, and not just the repaired sticky-note one

`copying_a_sticky_note_carries_the_whole_comment` owns the **grant**: a
comment copied in Review pastes in Review. It cannot own the **refusal**,
and the refusal is where opening a gate does its damage:

| | before the fix | after it, if nobody wrote the sentence |
|---|---|---|
| Review, comment on the clipboard | chord refused; `chord-not-offered` traced | pastes — the grant |
| Review, **page geometry** on the clipboard | chord refused; `chord-not-offered` traced | dispatcher returns; **nothing traced on any surface** |

The second row is strictly worse than the state it replaced. A chord stopped
at the gate at least left a line in the trace; a chord that reaches a
dispatcher and silently returns leaves the operator with a keystroke that
does nothing, no sentence, and — since `command-declined` is a diagnostic
channel — nothing they could show anybody either. That is this project's
founding defect class, and it is exactly what a fix for A1 would introduce
if it stopped at the gate.

⇒ So this check asserts **three things that must all hold at once**, and
none of the three is redundant:


★ Assertion 2 is the one that would be tempting to drop as "internal". It is
not: without it, assertion 1 alone passes on a build where **paste
succeeded**, because a successful paste also emits no `chord-not-offered`.
The pair is what pins *reached the dispatcher AND was refused there*.

## ★★ How it gets a content clip into Review without Review copying one

Review cannot select page content — that is `edit_content`, and Review does
not have it. So the operand is prepared **in Edit**, with `Ctrl+A`, and the
mode is changed afterwards. That is not a contrivance for the harness; it is
the operator's own path. Copy a detail out of a drawing, switch to Review to
mark up the sheet you were sent, press `Ctrl+V` out of habit.

`Ctrl+A` rather than a canvas click, deliberately: a click needs a document
whose geometry is under a known point, and `RESUME.md`'s fixture table
records three separate occasions on which a coordinate aimed at the wrong
document produced an articulate failure about nothing. `edit.select_all`
needs only that the page has content, and its own trace line says whether it
got any.

## What it does NOT assert, said rather than implied

**Which sentence is drawn.** The harness cannot read the text a panel
renders — no AccessKit reader, no OCR, no text extraction from a capture —
so it asserts that the decline REGION was published, which is the strongest
claim available from out here. The wording is pinned headlessly by
`text::clipboard::tests::every_mode_refusal_names_a_mode_the_selector_actually_offers`
and by
`app::status::decline::tests::a_mode_refusal_reads_like_no_other_decline`.

**The Read case.** Read refuses all four verbs and does so through the same
two gates, so it would be a second run of the same code path for no new
fact. Review is the mode the defect was reported in and the mode where the
grant and the refusal differ, which makes it the one worth a launch.
