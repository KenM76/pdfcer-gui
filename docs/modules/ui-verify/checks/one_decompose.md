# `an_edit_reads_the_page_from_the_session`

**Defect it guards.** Every edit decomposes the page twice: once in the
engine's verb and once in the window's own `decompose_page`, so a dense page
holds the next input for twice the parse it needs. The window now reads the
edit session's memoised model (`opendoc::cache::sessionmodel`), which the
verbs share.

**Steps.** Edit mode on a copy of `fixtures/clear-image-over-ink.pdf`. Click
the black square through the image's clear half (`selection-set`), press
ArrowRight (`canvas-move`), and read every `page-objects-built` line traced
after the key. Each must carry `source=session`.

**Falsified** by not reading the session's model in
`OpenDoc::read_session_model`: the rebuild after the move traces
`source=shell` and the check fails.

**What it does not prove.** That the verb itself skipped its decomposition:
the engine traces nothing for a memo hit. That follows from the engine's
`page_objects` contract, read from its source at the pin.
