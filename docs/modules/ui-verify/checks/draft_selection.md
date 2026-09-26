# `ui-verify/checks/draft_selection`

`shift_arrows_select_text` — **there is a selection inside a text draft**,
driven on a real drawing.

# What this is for


> **No selection inside a draft** — no Shift+arrow, no Ctrl+A, no
> drag-select.

Every text field the operator has ever used has all three. Without them,
replacing a word means pressing Backspace once per character, and replacing
a whole title-block cell means pressing it a dozen times.

# Why this check reads a TRACE and changes nothing

The honest way to prove that Shift+Right selected three characters is to
type over them and watch the text shrink. This check refuses to.

It drives **the operator's own drawing** — that is deliberate and it is what
`FEATURES.md` records as the lesson that cost three weeks: *"a check that
drives a document this project authored tests the shape this project
imagined, and the operator's documents are the only ones with the shape that
broke."* But proving a **selection** by making an **edit** is a bad trade:
it puts a real change on his document to observe something that was already
observable, and a check that has to mutate to measure will eventually mutate
and fail to clean up.


# The second half is the one nobody writes: the selection must GO AWAY

Rule 4 of the four in `canvas::textedit::caret`'s selection section: any
movement without Shift drops the selection. It is as important as the
selecting, and its absence is invisible until it bites — a highlight left on
screen after the caret has walked out of it, and the next keystroke deleting
text the operator is no longer looking at.

A build with no rule 4 passes every "does Shift+Right select" assertion.
This check presses Right afterwards and requires the shell to say `none`.


Step 6 sweeps the pointer across the editor box and requires a selection to
come out of it. It is here rather than in a check of its own because it
needs everything steps 1-3 establish — a mode, a tool, and a caret in a real
run — and a second check would be a second copy of all of it.

**Double-click-to-select-a-word is built and is NOT driven here.** Its
logic is unit-tested against a real galley, and a driven double click is a
gesture this harness has had trouble synthesising before (see
`a_synthetic_double_click_must_not_be_two_calls_to_a_settling_click_helper`
in the egui RAG). Named rather than quietly implied by a green run.
