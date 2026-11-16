# `ui-verify/checks/markup_text_tools`

Two checks on Markup ▸ Content, both off-screen with the scripted pointer on
`fixtures/dimension-scaled.pdf`.

| check | mode | asserts |
|---|---|---|
| `the_markup_tab_arms_the_text_tools` | Edit | Add text traces `text-edit-tool tool=TextEdit(Add)` and draws pressed; a second press traces `tool=Select` and releases it. Paragraphs flips `open-paragraphs enabled=…` on each of two presses, and `markup.paragraphs` draws pressed exactly when it is on. |
| `the_markup_text_tools_are_absent_in_review` | Review | the Markup tab draws its Notes group and no `ribbon.group.markup.content` |

`text-edit-tool` is the line `canvas::tool::arm_text_edit` emits, so the
first check proves the Markup control reaches the same arm as Edit ▸ Add
text rather than a look-alike. An armed caret changes only the cursor, which
a capture does not carry, so the trace and the drawn pressed state are the
oracles, not the pixels.

The Paragraphs half presses twice so the preference it persists ends where it
started.

The Review check's witness is the Notes group: an absent Content group means
nothing unless the tab was drawn.
