# `ocr::offer` — which reading choices a model takes

The Recognise-text dialog draws a reading choice only when the chosen model can
honour it. `reading_offer(model)` asks `pdfcer_ocr_host::check_options` about
each choice alone, so the shell restates none of the engine's rules:

| Choice | Probe | Drawn when |
|---|---|---|
| *No word lists* | `Dictionaries::none()` | not `RunnerError::DictionariesUnsupported` |
| Word files | built-in lists plus one placeholder word file | not `DictionariesUnsupported` |
| *Read by layout* | built-in lists, `layout = true` | `Ok` |

The word-file probe names a file that does not exist. An in-process engine
refuses any user word list with `DictionariesUnsupported` and never reads the
path. A program add-on tries the path and answers with a file error, which
means it takes word files, so any other error counts as acceptance.

Other refusals, such as missing language data, do not hide a choice.
`check_runnable` reports those against the model itself.

`check_reading(model, dictionaries, by_layout)` asks the same question of the
whole choice the run will send. The dialog shows its refusal, the engine's
`Display` text, under the group before Run. `OcrRunner::load` makes the same
check, so the sentence at Run is the same one.

The probes use the worker's language choice and `MAX_DPI`. Neither changes the
answer for word lists or layout.
