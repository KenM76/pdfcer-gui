# `word_lists_reach_the_program_that_reads_them`

**For a Tesseract program add-on, the Recognise-text window offers *No word
lists* and *Add word file…*, and both choices reach the program.**

# Sequence

1. Plant the stand-in Tesseract add-on in `target/ui-verify/ocr-word-lists/`
   (as `a_program_ocr_addon_runs_and_is_disclosed` does) and write
   `project-words.txt` holding `alpha` and `beta`.
2. Launch with `PDFCER_DIAG_WORDS_PATH` naming that file, so the word-file
   picker answers without a native dialog; open File ▸ Recognise text.
3. `ocr-no-word-lists` and `ocr-add-words` are declared (★).
4. Click both, then Run. `ocr-started` says `builtin-lists=false
   user-words=1` (★★).
5. `ocr-applied` says `words=7` (★★★).

# Oracle

The stand-in answers five words when run plainly, one more (`nodawg`) when
both built-in lists are switched off, and one more (`words:alpha,beta`) when
given `--user-words`. Seven words can only come from the program having
received both choices — the request alone (step 4) is what the window sent,
not what the program got.

# Falsified

The dialog made to drop word files for every model: FAIL at ★★
(`user-words=0`).

# Not covered

A real Tesseract install. The in-process engines, which take no word files and
whose refusal is shown in their own sentence at Run.
