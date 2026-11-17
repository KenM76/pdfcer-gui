# `app::dispatch::group`

Group and Ungroup: four commands, two verbs, two kinds of thing.

| Command | Acts on | Engine verb |
|---|---|---|
| `edit.group` | two or more page objects | `EditSession::group_objects` |
| `edit.ungroup` | one form XObject | `EditSession::ungroup_form` |
| `markup.group` | two or more markups on one page | `EditSession::group_annotations` |
| `markup.ungroup` | a markup in a group | `EditSession::ungroup_annotations` |

Ctrl+G and Ctrl+Shift+G are bound to the `markup.*` ids and reach both kinds:
page objects win while the mode edits content and the selection holds them;
otherwise the selected markups are taken. The `edit.*` ids never fall through
to markups.

## Decisions

- **The markup selected last is the group's primary.** Other readers show the
  primary's comment and author for the whole group (ISO 32000-1 §12.5.6.2),
  and last-selected is the one the handles are already drawn from.
- **A click on one grouped markup selects the whole group**
  (`canvas::selection::groupmates`), so move, copy and delete act on it
  through the existing multi-markup paths. Shift-click still toggles one.
- **Grouping page objects that are not neighbours in paint order moves them
  to the front-most member's depth.** The page may look different; the status
  line says so (`text::grouping::group_gathered_at_the_front`).
- **Ungroup is offered for any form XObject on the page in view**, not only
  for one pdfcer grouped; the engine refuses a transparency group, a form on
  a layer and one it cannot inline, each with its own sentence.
- **Refusals** are `text::grouping::GroupRefusal`, recorded through
  `app::status::decline::record_group`.

## Known gap

With only a group's primary selected from a panel, Markup ▸ Ungroup is greyed:
`selection.markup_ungroupable` reads the member side (`/IRT`). The chord still
dispatches and the engine answers.
