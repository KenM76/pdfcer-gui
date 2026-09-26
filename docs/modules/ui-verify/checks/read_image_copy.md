# `ui-verify/checks/read_image_copy`

`read_mode_copies_a_picture_other_programs_can_paste` — **click a picture
while reading, press Ctrl+C, and it is on the Windows clipboard.**

# The request


> *"In read mode the regular pointer should also allow us to select images
> so we can copy and paste them as well as text outside of the pdfcergui."*

## The two halves, and each has its own way of failing

| # | half | how it failed before |
|---|---|---|
| 1 | a click in **Read** selects an image | the click was swallowed by the text sweep; content selection needs `edit_content`, which Read does not have |
| 2 | `Ctrl+C` puts a **picture** on the OS clipboard | the clipboard got a marker sentence and an in-memory clip; Word got the sentence |

★★ Half 1 alone would be a feature nobody could use — a selection in a mode
with nothing to do with it. Half 2 alone was already reachable in Edit and
is not what he asked for. The check drives both in one sequence for that
reason.

## ★★★ Why the oracle is a trace line and not the clipboard

**Because reading the clipboard back would be testing Windows.** The
clipboard is one system-wide resource that any process can take at any
moment, so a harness that opened it would introduce a failure mode that has
nothing to do with pdfcer and would report it as a defect — and this suite's
own rule is that a harness assertion is a claim about the program *and*
about the harness, so the route with fewer harness-owned failure modes is
the honest one.

`clipboard-image w=… h=…` is written **after** `set_image_and_text` returns
true, which is after `SetClipboardData` accepted both payloads. So the line
is the application reporting what the operating system told it, which is the
strongest claim available from inside the process.

⇒ And it carries the **size**, which is the part a wrong build gets wrong: a
picture rendered at the wrong scale, or of the wrong thing, is still a
picture. This asserts that the pixels are at least as many as the selection
is points — the floor `canvas::clipimage::MIN_SCALE` exists to enforce,
because a copy that pastes smaller than the thing on screen is one the
operator cannot use.

## The sequence

| # | step | oracle |
|---|---|---|
| A | Read mode, click the picture | `selection-set … via=read-image` |
| B | `Ctrl+C` | `clipboard-copy kind=content objects=1` |
| C | …and a picture went with it | `clipboard-image w=… h=…`, at least 1:1 |
| D | **right-click it** | `canvas-menu context=canvas.read-object` |

★★ Step D is the route somebody finds without being told, and it was added
after the first three shipped. A chord is a feature for an operator who has
read a release note; Acrobat Reader puts *Copy Image* on the right-click,
and until 2026-09-01 a right-click anywhere in Read produced **no menu at
all** — the gate asked `caps.edit_content` before asking which menu, so
even the view menu that file's own comment calls *"the correct menu for a
reader"* was unreachable.
