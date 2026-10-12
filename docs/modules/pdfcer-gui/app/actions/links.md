# `app::actions::links` — applying `LinkAction`

Each variant is one `EditSession` link verb through the vector-edit funnel:
one undo entry, a trace line, and the verb's disclosures on the status line.

| Action | Verb | Trace |
|---|---|---|
| `Add` | `add_link` | `add-link-applied page= id= target= border=` |
| `SetTarget` | `set_link_target` | `set-link-target-applied id= target= replaced=` |
| `SetBorder` | `set_link_border` | `set-link-border-applied id= visible= width= appearance_replaced=` |

`target=` is `page:N`, `named:NAME` or `uri:ADDRESS`. `replaced=` is the `/S`
of the action that re-pointing removed, or `none`.

Disclosures:

- `LinkTargetChange::replaced_action`: a link that ran JavaScript or launched
  a file loses that action when re-pointed, and the status line names it.
- `LinkBorderChange::appearance_replaced`: the previous drawn appearance, made
  by another program, was replaced.

Refusals the engine words in its own terms are re-worded here
(`NamedDestinationNotFound`, `LinkUriInvalid`, `LinkBorderWidthInvalid`,
`EmptyGeometry`); the rest are shown as the engine words them.
