# `a_picture_pastes_onto_a_selected_push_button`

**Defect it guards:** a picture copied in another program and pasted while a
push button is selected lands on the page as loose content instead of
becoming the button's picture.

## What it drives

Off-screen, scripted pointer, `PDFCER_DIAG_INVOKE=mode.edit,file.properties`
and `PDFCER_DIAG_SELECT_FIELD=PushOne` on a copy of `all-field-kinds.pdf`.
The OS clipboard is snapshotted, set to a 64×32 bitmap, and restored after
the verdict (`os_image_paste::ClipGuard`). Ctrl+V, then Ctrl+Z.

## Oracles

| step | requires |
|---|---|
| before | `button-icon-shown field=PushOne icon=absent` |
| Ctrl+V | `clip-pasted source=os kind=image as=button-icon field=PushOne`, a new `edit-widget-applied` with `redrawn=yes`, then `icon=present`; no `as=content` line |
| Ctrl+Z | `undo-applied`, then `icon=absent` |

## Falsification

Removing the push-button branch from `dispatch::ospaste::picture` makes the
paste trace `as=content` and the check fails on the Ctrl+V row.
