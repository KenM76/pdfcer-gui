# `a_refused_character_offers_a_face_that_can_type_it` — the two runs that decided O141

<!-- old-name-exempt-file: the captured harness output below quotes the engine's
own refusal, which names the fixture's embedded font `SUBSET+pdfceSubsetDemo` and
the `/Font` resource prefix pdfcer writes into a file. Those are identifiers
inside a PDF, reproduced verbatim; the engine deliberately stopped its rename at
the file-format boundary because changing a resource name would alter the bytes
of every document pdfcer has ever produced. This file cannot be edited to remove
them without ceasing to be a capture. See `fixtures/subset-font-floor.PROVENANCE.md`,
which carries the same exemption for the same identifier. -->

**Both runs are kept, and the FAILING one is the more valuable of the two**, so
it is first. It is what a driven check is for: everything was green — the unit
tests, the gates, the ribbon comparison — and ninety seconds of driving found
that the route did not arrive.

★★★ **And its own diagnosis was wrong, which is the finding to carry.** Read its
last paragraph: it says *"a failure here is the shell, and the likeliest cause is
the operand"*. It is not the shell. `crates/pdfcer-gui/src/canvas/textedit/facewall.rs`
settled it by measurement the next morning — one `EditSession`, `format_text`
then `edit_text`, located by find text alone so that **no operand this shell
computes is in the request**, and it is still refused; the same pair with a save
and a reopen between them succeeds. The defect is `pdfcer-core`'s, filed as
`request_edit_text_resolves_font_names_against_the_base_revision.md`.

⇒ A driven FAIL is a claim about the check too, *and about the paragraph the
check writes when it fails*. This one named a mechanism, sounded certain, and
pointed at the wrong crate.

---

## RUN 1 — 2026-09-05, commit `6098433`: FAIL

The route reached the last step and stopped there.

```text
ui-verify — profile `pdfcer-gui` (the application this project is building (crates/pdfcer-gui))
  NOTE: this harness drives the REAL cursor and keyboard. It raises the target window, moves the pointer, and types into it. The pointer is put back where it was when the run ends. Pass --no-input to disable (checks that need input then report SKIPPED, never PASS).

[FAIL] a_refused_character_offers_a_face_that_can_type_it
       detects: an edit is refused because the run's font has no code for the character just typed, and the operator is told only that the edit was refused — while the engine named the character, the shell already has a chooser that offers faces which carry it, and set_font already writes one; so the answer to "can we change to a different font?" is yes and is unreachable from the moment the question arises
       · launched target/release/pdfcer-gui.exe as pid 46564 on fixtures/subset-font-floor.pdf with PDFCER_DIAG_INVOKE=view.reset_layout,mode.edit,file.properties,edit.text and PDFCER_DIAG_TYPE=q
       · ★ the dock layout was reset: `pdfcer-diag layout-reset scope=all changed=false`
       · ★ the control point holds: nothing refused, no offer on screen
       · ★ the click placed a caret: `pdfcer-diag text-edit-caret kind=Edit page=0 run=0 len=3`
       · ★★ the engine refused the commit: `pdfcer-diag edit-text-refused page=0 n=1 detail=R-INV-1 (embedded-subset floor): character U+0071 'q' maps to code 113 which font 'SUBSET+pdfceSubsetDemo' (an embedded SUBSET) does not already carry on this page; embedding a new glyph is deferred to FF-C (font subsetting). This is exactly Acrobat's 'embedded-but-not-local' floor.`
       · ★★ and it was classified: character='q', sentence=FontLacksTheCharacter
       · ★★★ the offer drew and NAMED the character: `pdfcer-diag refused-char page=0 run=0 character='q' font=pdfceSubsetDemo faces=15 state=offer`
       · ★★★ rule 4 is discharged: the disclosure is on screen, in the panel, and shares no area with the page — nothing marks the canvas
       · the restyle answered after 511 ms of wall clock
       · ★★★ the offer was TAKEN and it reached the document: `pdfcer-diag text-style-applied page=0 change=face applied=1 runs=1` — `format_text` wrote the `/Font` resource itself, in the same undo command
       → ★★★ THE ROUTE DOES NOT ARRIVE. The face was swapped and the same
         character was committed again into the same run, and no `edit-text` line
         followed — it was refused AGAIN: `pdfcer-diag edit-text-refused page=0 n=1
         detail=this run cannot be edited in the first cut: the run's font resource
         is unresolvable in the target stream's resources`. Measured on this fixture
         with `pdfcer.exe` before this check was written: `format-text --set-font
         Helvetica` then `edit-text --replace "q"` succeeds and `extract-text` reads
         the character back. So a failure here is the shell, and the likeliest cause
         is the operand: the restyle must reach **the run the refusal named**, and a
         swap applied to a different run leaves the one under the caret in the font
         that refused. Trace: tools/ui-verify/out\refused-character-face.trace.txt.
       artifact: tools/ui-verify/out\refused-character-face.trace.txt

------------------------------------------------------------------------
  0 passed, 1 failed, 0 skipped

RESULT: FAIL — a check drove the application and the assertion did not hold.
```

---

## RUN 2 — 2026-09-05, after the one-gesture route and the engine request: PASS

Two things changed between the runs and neither of them is a loosened
assertion. **The offer now re-applies the operator's own edit itself** — the
words he typed travel with the refusal (`canvas::textedit::Committing`), so
there is no second click and no second `Ctrl+Enter`; and **the block has a third
state**, `blocked`, which is the sentence naming the measured engine limit and
the remedy that was actually run.

The check accepts two outcomes at the last step and refuses every other one: the
character landing (which is what happens the day the engine ships the fix), or
the base-revision refusal *with* the block having moved to `state=blocked`. A
second embedded-subset refusal — the shell defect the file was written to catch
— is failed by name.

```text
ui-verify — profile `pdfcer-gui` (the application this project is building (crates/pdfcer-gui))
  NOTE: this harness drives the REAL cursor and keyboard. It raises the target window, moves the pointer, and types into it. The pointer is put back where it was when the run ends. Pass --no-input to disable (checks that need input then report SKIPPED, never PASS).

[PASS] a_refused_character_offers_a_face_that_can_type_it
       detects: an edit is refused because the run's font has no code for the character just typed, and the operator is told only that the edit was refused — while the engine named the character, the shell already has a chooser that offers faces which carry it, and set_font already writes one; so the answer to "can we change to a different font?" is yes and is unreachable from the moment the question arises
       · launched target/release/pdfcer-gui.exe as pid 22484 on fixtures/subset-font-floor.pdf with PDFCER_DIAG_INVOKE=view.reset_layout,mode.edit,file.properties,edit.text and PDFCER_DIAG_TYPE=q
       · ★ the dock layout was reset: `pdfcer-diag layout-reset scope=all changed=false`
       · ★ the control point holds: nothing refused, no offer on screen
       · ★ the click placed a caret: `pdfcer-diag text-edit-caret kind=Edit page=0 run=0 len=3`
       · ★★ the engine refused the commit: `pdfcer-diag edit-text-refused page=0 n=1 detail=R-INV-1 (embedded-subset floor): character U+0071 'q' maps to code 113 which font 'SUBSET+pdfceSubsetDemo' (an embedded SUBSET) does not already carry on this page; embedding a new glyph is deferred to FF-C (font subsetting). This is exactly Acrobat's 'embedded-but-not-local' floor.`
       · ★★ and it was classified: character='q', sentence=FontLacksTheCharacter
       · ★★★ the offer drew and NAMED the character: `pdfcer-diag refused-char page=0 run=0 character='q' font=pdfceSubsetDemo faces=15 state=offer`
       · ★★★ rule 4 is discharged: the disclosure is on screen, in the panel, and shares no area with the page — nothing marks the canvas
       · the restyle answered after 103 ms of wall clock
       · ★★★ the offer was TAKEN and it reached the document: `pdfcer-diag text-style-applied page=0 change=face applied=1 runs=1` — `format_text` wrote the `/Font` resource itself, in the same undo command
       · ★★★ the offer RE-APPLIED the operator's own edit with no second gesture — a commit followed the face swap after 508 ms and the harness pressed nothing
       · ★★ the retype was refused by the ENGINE — `pdfcer-diag edit-text-refused page=0 n=1 detail=this run cannot be edited in the first cut: the run's font resource is unresolvable in the target stream's resources` — and the block moved to `state=blocked`, which is the sentence naming the measured cause and the remedy (save, reopen, type it once more). This is `request_edit_text_resolves_font_names_against_the_base_revision.md`, filed 2026-09-05 with a copy-pasteable reproduction; `pdfcer-gui`'s `canvas::textedit::facewall` asserts all three of its measurements, so the day it ships this check takes the branch above instead
       · ★★★ and the instrument has dynamic range: the block walked ["offer", "swapped", "blocked"] across the run, three states each caused by a different thing happening to the document — it is not a region drawn unconditionally
       artifact: tools/ui-verify/out\refused-character-face.trace.txt

------------------------------------------------------------------------
  1 passed, 0 failed, 0 skipped

RESULT: PASS — every check drove the application and every assertion held.
```

## How run 2 was falsified

Neither mutation is a rewording; each cuts a link the check claims to hold, and
each was built into `target/release/pdfcer-gui.exe` and driven.

| mutation | verdict |
|---|---|
| the `Action::CommitTextEdit` push removed from `refusedchar::section`'s `swapped` arm | **FAIL** — *"THE OFFER DOES NOT FINISH THE JOB. The face was swapped 20495 ms ago and no commit of any kind followed"* |
| `state.retried` left `false` instead of being set | **FAIL** — *"THE RETYPE WAS REFUSED AND THE BLOCK DOES NOT SAY SO … the last `refused-char` state after the swap is Some(\"swapped\") rather than `blocked`"* |
