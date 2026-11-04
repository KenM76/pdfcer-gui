# `pdfcer-gui-base/ribbontabs/clipboard`

The **Clipboard** tab — *what am I taking out, or putting in?*

`RIBBON_IA.md` §5.0. Second in every mode, after File, so the verbs an
operator uses most are one click from any mode. Every command is Large —
icon above its label — because the tab holds few commands and is visited
often (O380).

# Two groups

- **Clipboard** — Cut, Copy, Paste, Paste as duplicate, Duplicate. Within
  this document, or between two pdfcer documents.
- **Copy for other programs** — what another program pastes: the selection
  or page as drawing geometry (`edit.copy_as_vector`), and the snapshot box
  with its text as outlines (`edit.copy_snapshot_outlines`) or as editable
  text (`edit.copy_snapshot_text`). Both snapshot copies override the
  Snapshot setting's *Copy as*; Copy itself follows it.

# What Read mode shows

Copy alone. Cut, the two pastes and Duplicate change the document, so they
carry `visible_when: mode.author_markup` and are absent in Read rather than
greyed: Read cannot change the document at all, which is a property of the
mode, not a temporary state (R9). In Review they act on markup; on page
content they refuse with the dispatcher's reason, because content editing is
Edit's (`app::dispatch::clipboard`).

# The ids keep `edit.`

`edit.cut`, `edit.copy` and the rest keep their prefix rather than becoming
`clipboard.`. A keymap, a saved layout and the ui-verify checks name them by
id, so the id does not follow the tab — the rule the Forms tab's
`edit.form_` ids follow.

# Pages ▸ Clipboard stays on Pages

Page cut, copy and paste move whole pages and are part of organising a
document, so they stay with the other page verbs. One command, one tab (P1).
