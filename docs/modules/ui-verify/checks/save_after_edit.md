# `ui-verify/checks/save_after_edit`

`ctrl_s_after_an_edit_saves_and_the_program_is_still_running` — **the
operator pressed Ctrl+S and the window disappeared.**

# The report

Ken, 2026-09-01: *"can you try doing an edit and save? I did this and
pressed ctrl+s to save and it closed."*

## ★★★ Why the existing coverage could not see this

Three things already test parts of this and none of them tests **the thing
that happened**:

| what exists | what it does | why it misses this |
|---|---|---|
| `every_declared_chord_dispatches` | presses `Ctrl+S` among seven chords | on a **clean** document — nothing to write |
| `saving_writes_the_document` and friends | invoke `file.save` through `PDFCER_DIAG_INVOKE` | the seam, not the keyboard |
| 2,786 unit tests | call the verbs | a process that exits is not a returned `Err` |

⇒ The gap is the **conjunction**: a real keystroke, on a document that has
something to write. Each half was covered and the pair was not, which is the
shape of nearly every defect this project has found by driving.

## ★★ The oracle is that the process is ALIVE

Unusual, and the reason is what was reported. Every other check in this
harness asks *"did the right trace line appear?"*; a program that has exited
writes no line at all, and an absent line is this harness's most common
**false** signal — it is what a missed click, a stale coordinate and a
window that never focused all look like.

So this check asserts three things in order, and the order is what makes the
diagnosis unambiguous:

1. the edit landed (`pages-rotated`) — so there IS something to save;
2. the process is still running **after** the chord;
3. the save committed (`save-in-place outcome=ok`).

★ If (2) fails, (3)'s absence is explained and must not be reported as
*"the save did nothing"*. Getting that order wrong would turn a crash into a
silent-verb report and send a reader to the wrong module entirely.

## Why a page rotation is the edit

It is the only document change in this shell that needs **no pointer at
all** — one command id, no dialog, no selection, no picker. Every other edit
needs a click, and a click that missed would make this check report a save
failure that was really an aim failure.
