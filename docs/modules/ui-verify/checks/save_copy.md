# `ui-verify/checks/save_copy`

`save_copy_round_trip` — **the check for the one thing this application
could not do**: put an operator's work on disk and get it back.

# What was wrong, and why nothing noticed

`file.save_copy` has been registered since the ribbon landed. It is drawn on
File ▸ Save, it is on the quick-access toolbar, it is bound to `Ctrl+S`, and
its tooltip prints "(Ctrl+S)". It had **no dispatch arm**, so every press
traced `command-unimplemented` and did nothing — which meant that every
feature this project has shipped (dimensions, markup, text marks, form
fills, page operations, a document made by `file.new`) was **unwritable**.

The whole suite was green throughout, and it had to be: the engine's write
verbs are tested in `pdfcer-core`, the command's registration is tested in
`shell::commands`, the picker's seam is tested in `app::files`. What has no
test anywhere in the workspace is the **join** — that pressing the control
reaches a dispatch arm, that the arm reaches a picker, that the picker's
answer reaches a write, and that what is written is the document the
operator was looking at rather than the one they opened.

# The five links, and where each is otherwise covered

| # | Link | Its own test |
|---|---|---|
| 1 | the ribbon click reports the command | `egui-shell`'s `band::render_command` — yes |
| 2 | dispatch raises `Action::SaveCopy` | `app::files::tests::the_save_copy_command_raises_the_save_action` — yes |
| 3 | the apply reaches the picker and the picker's answer reaches the write | **nothing** |
| 4 | the bytes are an **incremental update**, not a full rewrite | `app::save`'s unit tests — yes, on a synthetic edit |
| 5 | the operator's **canvas-authored** annotation is in the file that comes out | **nothing** |

Link 3 cannot be tested off the binary at all: applying `Action::SaveCopy`
in a unit test opens a **real modal dialog** and hangs `cargo test` behind an
invisible window (`app::files`' rule 3). Link 5 cannot either, because the
only thing that authors an annotation from a *drag* is the running canvas.

# The six phases

| Phase | Does | Expected |
|---|---|---|
| A | Review, Comments panel brought to the front | a `comments-panel listed=N` baseline, published **after** the `mode-changed` line |
| B | arm Rectangle, drag on the page | `markup-tool`, `markup-commit kind=Rectangle`, `add-markup`, and a census published **after** `add-markup` reading `listed=N+1` with `with_note` and `authors` unmoved |
| C | File tab, **Save a copy** | `save-copy … appended=>0`, and a file at the named path |
| D | re-read the **source** | byte-identical to before the run |
| E | compare the copy's prefix with the source | the source's bytes are the copy's prefix, verbatim |
| F | **launch a second process on the copy** | `comments-panel listed=N+1`, again with `with_note` and `authors` unmoved |

# Three falsifying phases, and the build each one catches

Phases D, E and F each fail against a *different* plausible wrong
implementation, and **no two of them catch the same one**. That is what makes
them worth the run time; a check whose phases all fail together is one
assertion wearing six hats.

## D catches: *the save wrote over the file that was opened*

`dialogs::ocr`'s check already makes this assertion for the OCR write, and it
matters more here, not less: this command's own tooltip promises *"The
original is never overwritten unless you pick it"*, and the operator does not
pick it — `PDFCER_DIAG_SAVE_PATH` names somewhere else. A build whose write
ignored the picker's answer and used `doc.path` would pass A, B, C, E and F
(E vacuously, F because the annotation really would be in the file) and would
have destroyed the operator's drawing. The digest is what survives a reviewer
who did not look.

## E catches: *the save was a full rewrite*

`EditSession::to_full_bytes` produces a perfectly good PDF that carries the
annotation. It passes A, B, C, D and **F**. What it does is destroy every
digital signature in the file (§12.8.1) and discard the previous revision
that `file.save_copy`'s shipped tooltip promises *"stays intact inside the
file"* — neither of which any oracle in this harness can see except this one.

A §7.5.6 incremental update cannot rewrite the original bytes by
construction: it leaves the base revision alone and appends. So
`copy[..source.len()] == source` is a **property of the save mode**, and it
is the cheapest possible test that tells the two modes apart.

## F catches: *a file was written and the edit is not in it*

The one that gives the check its name. A build that wrote the **base
document's** bytes — `Document::bytes()`, the file as opened — instead of the
session's update produces a file that exists, opens, has the right page count
and is byte-identical to the source. It passes A, B, C, D **and E**, and E
passes *trivially*, because the copy simply is the source.

So the copy is re-opened **in a second process**, through the same
`Document::load` an operator's own File ▸ Open would use, and the Comments
panel is asked how many annotations it can see. *A save that writes a file is
not the same claim as a save that writes the edit*, and this is the second
one.


[`crate::checks`]' rule for a new check is that *"it must fail against a
build where the wiring is absent"*, and that every check here "has been run
against such a build and seen to fail". Two builds were planted, one per
falsifier, against `fixtures/a1-titleblock.pdf` (12 annotations, page 1
2384 × 1684 pt):

| Plant | One line changed | Result |
|---|---|---|
| `to_full_bytes` in place of `to_incremental_bytes` | phase E: *"they agree for the first 152 bytes and then diverge"*, copy 39,932 B against a source of 39,509 B | **FAIL at E**, having passed A–D |
| write `std::fs::read(&doc.path)` — the file as opened — instead of the session's bytes | phase F: the second process listed **12**, not 13 | **FAIL at F**, having passed A–E, *including E* |

The second plant is the more instructive of the two, and it is the reason
phase F exists rather than being trusted to the trace: **the planted build's
own `save-copy` line still read `appended=1002`**, because it computed a
perfectly correct `SaveReport` and then wrote different bytes. A trace line is
written by the code under test, about itself. Only the two phases that read
the *file* — and, for F, read it back through a second process — could tell
the difference.

The unedited build passes both. That pair is what makes a green result here
evidence rather than an absence of evidence.

# Why the Comments panel is the oracle, and not a pixel

Because a 2 pt rectangle over a drawing is a few hundred antialiased pixels
among a page already full of thin dark lines, and no threshold this crate has
separates the two — the same reason [`crate::checks::markup_shapes`] saves its
capture as evidence rather than using it as an oracle.

`panels::comments` traces `comments-panel … listed=N` once per frame, and `N`
is derived by walking the **session's own annotation list** through
`pdfcer-core`. In the second process that session was built by loading the
saved file from disk, so `listed=` there is a statement about the *file*,
made by the engine, in a process that never saw the first one.

## Every census here is ANCHORED, and the day that started mattering

This paragraph used to say that in Review the panel is the first tab of the
right stack and *"is therefore active on the first frame"*, with a click on
`markup.comments` as belt and braces. Both halves went wrong at once on
2026-09-05. The belt-and-braces click had already been removed on a finding
about band overflow that has since stopped being true; and a persisted
`userdata/layout.ron` put Document properties in front of Comments, so the
panel **stopped tracing entirely** — a dock draws only its active tab.

The check then read the census the panel had published in the *previous
mode*, twice, and subtracted it from itself: *"it listed 12 before the drag
and 12 after it."* It reported a working panel as broken, and
`undo_redo_round_trip` — carrying a copy of the same helper — reported the
same thing in the same words on the same sweep, which read as corroboration.

⇒ Every census this check takes now comes from
[`crate::checks::comments_census`], which (a) requires the line to have been
published **after** a named cause — the mode change, or the engine's own
`add-markup` — so a fossil cannot answer, (b) **brings the panel forward**
when it has gone quiet, by its dock tab or by `markup.comments`, which
`app/panels.rs` makes a *show* rather than a toggle, and (c) reports SKIP,
never FAIL, when it cannot: *"the panel said nothing"* is a layout fact and
*"the panel said the wrong number"* is a defect, and they are not the same
verdict. Driven against a deliberately seeded hostile layout on 2026-09-05
and seen to recover; driven against a planted frozen census and seen to
fail.

# The picker is answered, not driven

`PDFCER_DIAG_SAVE_PATH` supplies the save dialog's result and the dialog is
never opened — `app::files`' established pattern for a native picker, and the
RAG note it quotes: *"Don't try to script the dialog."* That is what makes
phase C an assertion about **a file on disk** rather than about a button
having been pressed.

**Say plainly which path was driven, because it is not the whole of the
operator's one.** Everything from the ribbon click to the write is the real
code path: the click, the dispatch arm, `Action::SaveCopy`, the apply phase,
`app::save::save_copy`, `pick_save_path`, `to_incremental_bytes`,
`std::fs::write`. The **one** substituted call is the `rfd` dialog itself,
and it is substituted because the alternative is not "test it a harder way" —
it is a modal top-level window owned by the OS shell that no synthetic input
can reach. What is therefore NOT covered here: that the dialog opens with the
suggested name pre-filled and in the right folder. That is
`app::save::suggested_path`'s unit tests plus two `rfd` calls, and it is
stated rather than implied by a green result.

# Mouse only

Every gesture is a real `SetCursorPos` + `mouse_event`. **`Ctrl+S` is not
driven here**, and the keymap binding is covered only by


`Ctrl+S` was one of the fourteen dead chords. It is now dispatchable and
this check SHOULD drive it — that is unwritten work, not a limitation.
Continuing the original note:
`shell::manifest`'s keymap test and by the one dispatcher every route shares.

# Every way this reports SKIP, and why none of them is a pass

* no binary, no `--pdf`, `--no-input` — the harness never began;
* the diagnostic switches did not reach the process;
* the page size could not be read and no `--page-size` was given;
* a mode segment, a tab or a control was never declared, or took no click;
* the canvas is not showing page 1;
* **the fixture already carries so many annotations that the panel excludes
  some** — the count would then not move by exactly one and the check would
  be measuring the panel's editorial rules rather than the save.
