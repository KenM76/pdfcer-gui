# `ui-verify/checks/dropped_text`

`a_dropped_text_file_becomes_pages_after_this_one` — a `.txt` dragged onto
the window becomes pages directly after the page on screen, set exactly as
File ▸ Import text as pages sets them with its controls untouched.

It drives the same scripted `drop` step as `dropped_file`, for that check's
reason: a real drop cannot be synthesised without the operator's mouse.

# The oracles

A two-line file must trace `text-dropped` and then
`import-text-applied pages=1 first=1`: one page, at index 1, after page 0
where the document opened. Ctrl+Z must trace `undo-applied`. An empty file
must trace `import-text-refused` and add no `import-text-applied`, because
the engine refuses rather than write a blank page.
