# `ui-verify/checks/text_box`

`text_box_takes_a_paragraph` — **multi-line text**, driven end to end.

# What this is for

The operator, 2026-08-21: *"I should be able to make it multi line."*

Until then a text draft was one line and Enter **committed** it, so there
was no keystroke that could produce a second line and no gesture that could
ask for one.

# Why multi-line needs a box, which is what this check is really about

**A PDF has no paragraph.** Each visual line is its own show operator at its
own absolute position, so something has to decide where the second line
starts — a width to wrap against and a leading to step by. `pdfcer-core`'s
`AddTextRequest::with_box` is that, and it needs a rectangle.

So the gesture is a **drag**, and the chain has four links no unit test can
reach:

| # | link | its own test |
|---|---|---|
| 1 | a drag with the text tool becomes `DragKind::TextBox` rather than a marquee or a sweep | `gesture::meaning` — the decision, not the routing |
| 2 | the release converts the band to a page rect and opens a draft anchored to it | nothing |
| 3 | **plain Enter INSERTS instead of committing** | nothing — and this is the link that would ship |
| 4 | Ctrl+Enter commits, and the wrap rectangle reaches the engine | `canvas::textedit` asserts the ACTION; nothing asserts the engine saw it |

**Link 3 is the one that would fail silently and plausibly.** If Enter
still commits inside a box, the first Enter ends the draft and everything
typed after it goes nowhere — which from a chair is *"multi-line does not
work"*, with a perfectly ordinary single-line run left on the page as
evidence that something happened.

# The oracle carries the LINE COUNT

`add-text … n=<hard newlines>`, and that number is the point: a build where
Enter committed authors `n=1` and one where the newline survived authors
`n=2`. A line saying only *"text was added"* is identical for both, which is
`DEFECTS.md` D14's rule — **a trace line must carry the number a wrong build
would get wrong.**

It counts **hard** newlines rather than laid-out lines, deliberately: the
engine wraps to the box's width using the face's own metrics and this
harness does not have them, so a wrapped count would be a number neither
side could check.

## Item notes

### `const EDIT_TAB`

A mode is not a tab. `click_mode_segment` puts the shell in Edit *mode*,
which is what decides `edit_content` — and leaves whichever tab was already
showing. The control has to be reached on its own tab, and the first run of
this check found that out by reporting the control as undeclared.

### `const TOOL`

**Add text, not Edit text**, and the distinction is what keeps two
features off one gesture. `edit.add_text` arms
`CanvasTool::TextEdit(TextEditKind::Add)`, whose drag draws a box; the
separate `view.tool_text` arms `CanvasTool::Text`, whose drag **sweeps** and
must go on sweeping, because `text_tool_selects_and_marks_in_edit` depends
on it to make a text selection the markup verbs can act on.

The box was briefly offered from the sweep tool's rung instead, and two unit
tests said no — correctly. Two features claiming one drag is a choice
somebody has to make, and taking a shipped gesture away to make room is the
wrong way to make it.
