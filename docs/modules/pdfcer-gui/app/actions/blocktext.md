# `app::actions::blocktext`

`commit` is the body of `TextAction::CommitBlock`: one
`EditSession::edit_block_text` through `funnel::vector_edit` (document scope,
as the rewrite may add glyphs to a font), so it is one undo entry and its
disclosures reach the status line.

A refusal is worded by its family. `BlockEditError::Text` is a caret commit's
refusal and goes to `record_edit_text_refusal` with the run the draft was opened
from; `BlockEditError::Block` is a reflow's and goes to `record_reflow` through
`textstyle::reflow_refusal`. Anything else keeps the funnel's generic sentence.

On success it traces `edit-block-text-applied page= block= lines=A->B
paragraphs= glyphs_added=`, `-applied` because the funnel writes its own
bare-named line for the same edit. `paragraphs` counts the committed text's
`\n`-separated paragraphs.
