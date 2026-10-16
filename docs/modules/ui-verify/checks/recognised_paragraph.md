# `checks::recognised_paragraph`

`a_recognised_paragraph_edits_word_by_word` drives `ocr-layer.pdf` in Edit
mode with the text layer shown and the Text tool armed.

1. A click on the first recognised word must trace no `text-edit-widened`:
   the word opens alone, keeping its in-place preview in its own font, which
   a recognised paragraph has no engine layout for. A build that opens the
   paragraph on the click fails here.
2. *This paragraph* on the caret's menu must then trace
   `text-edit-widened … recognised=true`: the paragraph's own answer opens
   it (`promote::recognised::widen`).
3. Home, one typed letter, Ctrl+Enter must trace
   `edit-recognised-text-applied` with `replaced=1 deleted=0 kept>=1`: the
   commit went through `edit_recognised_block_text`, which changes only the
   edited word. A commit through `edit_block_text` traces a different event,
   and a rewrite that dropped or re-made the other words fails the counts.

Typing at the paragraph's start changes exactly its first word, so the
expected counts do not depend on the fixture's word count.
