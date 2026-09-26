# `ui-verify/checks/chunk_empty`

`emptying_a_chunk_commits_the_emptying` — **O216 ask 1, driven**: select
every character inside a text chunk, delete them, commit, and find out
whether anything was ever asked of the engine.

# The report and what it turned out to be

The operator: *"deleting all the text inside a chunk doesn't save."* Four
causes fit that sentence — the commit never fires, the shell's plan declines
it, the engine refuses and the decline is not shown, or it commits and the
save drops it — and they lead to four different pieces of work.

It is the first. `canvas::textedit::commit_into`'s `Anchor::Run` arm used to
carry `&& !draft.text.is_empty()`, so an emptied chunk raised **no action at
all**: no plan, no `edit_text`, no refusal to classify and no sentence
anywhere. Nothing failed to save because nothing was ever requested.

## Which is why this check's subject is an ABSENCE, and why that is hard

The defect's whole signature is that nothing happened. Four other things
produce exactly the same trace:

| what really happened | what the trace shows |
|---|---|
| the guard dropped the edit | no `text-edit-plan` |
| the click resolved no run, so there was no chunk to empty | no `text-edit-plan` |
| `Ctrl+A` never reached the draft, so the text was unchanged | no `text-edit-plan` |
| `Delete` never reached the draft, same | no `text-edit-plan` |
| `Ctrl+Enter` never reached the draft | no `text-edit-plan` |

So every step below the caret is **calibrated from a line the shell emits
for its own reasons**, and the check reports which of the five it met rather
than reporting the one it was written to find. A check whose failure message
names a defect it did not distinguish from four harness faults is worse than
no check: it sends a reader into `commit_into` over a missed keystroke.

## The calibration, and where it comes from

`canvas::textedit::keys` traces the draft's selection on every change —
`text-select from=… to=… n=…` while there is one, and `text-select none
caret=…` when there is not. That line exists for its own reasons and this
check reads it as an instrument:

* after `Ctrl+A`, `n` must equal the `len` the caret line reported. Less, and
  the chord was taken by something else; absent, and it never arrived.
* after `Delete`, `none caret=0` — the selection is gone and the caret is at
  the start, which on a draft whose whole contents were selected is the
  emptying, observed.

Only with both in hand does an absent `text-edit-plan` mean what this check
says it means.

# ⚠ The aim is the ENGINE's, not a guess

`--doc-point` is required and has no default, for the reason
`text_edit_on_a_real_drawing` records at length: a click on empty page is
symptom-identical to a broken hit test, and an Edit click that lands on bare
paper is silently converted into an **Add** draft, where every assertion
here would be describing something that is not a run.

```text
# ⚠ Drive a COPY — a run types into the document and may commit.
cp D:/dev/pdfTests/SW41177/SW41177.pdf target/scratch/docs/SW41177-empty.pdf

pdfcer find-text --needle SLOT target/scratch/docs/SW41177-empty.pdf
  match page=1 text="SLOT" rect=271.76,717.10,301.58,731.26
  …

# The aim is that rect's CENTRE. `find-text` numbers pages from 1 and
# `--doc-point` numbers them from 0, so the first match is page 0 here.

ui-verify --exe target/release/pdfcer-gui.exe \
          --pdf target/scratch/docs/SW41177-empty.pdf \
          --doc-point 0,286.67,724.18 \
          --check emptying_a_chunk_commits_the_emptying
```

## Falsified in both directions, which is what the calibration is for

Planting the old `!draft.text.is_empty()` guard turns this row **red** with
the O216 sentence — and the three calibration notes still pass, so the
failure is attributed to `commit_into` rather than to a missed keystroke.
Removing the `Delete` press instead turns it **SKIPPED**, naming the draft's
own `text-select` reading as the reason. A check on an absence that cannot
tell those two apart has not measured anything.

# What this check does NOT assert

**That the emptying survives a save.** That is
`ctrl_s_after_an_edit_saves_and_the_program_is_still_running`'s subject and
it is a different drive; splitting them keeps each failure naming one
module. This one ends at the engine's own answer, because the whole of O216
ask 1 was that the engine was never asked.
