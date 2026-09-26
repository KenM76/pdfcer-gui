# `ui-verify/checks/clipboard_text`

`ctrl_c_copies_text_to_the_os_clipboard` — the one assertion no unit test in
this workspace can make.

# Why this check exists, and why nothing else could have caught the defect


> *"if I select text in read mode … and press ctrl+c to copy, then try to
> paste in notepad, it doesn't work. I get a notice to paste it back into
> pdfc to place it."*

Sweeping text on the page and pressing `Ctrl+C` had **never once copied it**,
in any mode, since the day that code was written. `egui-winit` intercepts the
three clipboard chords and pushes `Event::Copy` *instead of* an `Event::Key`,
and the handler was looking for the key. The object clipboard answered
instead and left its own marker sentence on the clipboard.

**1,628 unit tests were green throughout**, and no number of them would have
helped, because the failure is not in a function's return value — it is in
*which of two handlers reached the operating system last*. Even a trace is
not enough: `text-copy source=selection` can be emitted, truthfully, by a
frame in which a later handler then overwrites the clipboard. The trace says
what a function did; it cannot say what the operator would get.

**The only oracle for "what does the operator get when they paste" is the
operating system's clipboard, read from outside the process.** That is what
this check does, and it is the entire justification for
[`crate::sys::clipboard_text`] existing.

# ★★ It clears the clipboard first, and that is not hygiene

It is the difference between a check and a coin toss. Three outcomes are
otherwise indistinguishable:

| the application | the clipboard afterwards | without clearing first |
|---|---|---|
| copies the text correctly | the swept text | passes |
| writes the object marker (the defect) | `"…copied from pdfcer…"` | fails |
| **does nothing at all** | whatever was there before | **passes, if a previous run left the right text** |

Row three is the one that matters. A check that passes when the application
does nothing is worse than no check, because it certifies the silence. So:
clear, drive, read — and if the clear itself fails, report SKIPPED rather
than asserting against a clipboard this process never controlled.

# What it asserts, in both directions

A positive and a negative, and neither alone is sufficient:

* the clipboard holds **something**, and
* what it holds is **not the object clipboard's marker sentence**.

The negative is the one that names the defect. Without it, a build that
wrote *"1 object copied from pdfcer"* would satisfy "the clipboard is not
empty" and pass while doing exactly the wrong thing.

It deliberately does **not** assert the exact string. What a sweep across a
CAD drawing's title block yields depends on the fixture, on where
`--doc-point` aims and on how the producer split its show operators; pinning
it would make the check a statement about the file rather than about the
program, and it would be re-pinned rather than believed the first time it
failed.

# Read mode, deliberately

The operator reported it there first, and it is the sharper case: Read has no
content selection at all, so the *only* thing a click can produce is a text
sweep. If the clipboard comes back holding an object marker in Read, an
object clipboard answered a chord in a mode that cannot select an object —
which is a second defect wearing the first one's clothes.
