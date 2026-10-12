# `panels::properties::link` — a selected link's destination and border

Drawn for a selected annotation whose subtype is `Link`, and nothing else; the
markup section stands aside for links.

# Contract

- A locked link shows a note and no controls.
- The form is seeded from `linkread::read` and kept in egui temporary memory,
  keyed by the link's id, beside the reading it was seeded from. When the file
  changes under it (an undo, another edit) the reading differs and the form is
  re-seeded, so it never shows a value the link no longer has.
- A link running an action this shell does not author says which action, and
  that applying a destination replaces it. A link with no target says so.
- Apply destination pushes `LinkAction::SetTarget`; it is absent while the
  form refuses, with the reason shown. Apply border pushes
  `LinkAction::SetBorder`. Each is one undo entry.
- Regions: `properties.link`, `properties.link.apply-target`,
  `properties.link.apply-border`, and the form's `properties.link.page`,
  `.web`, `.named`, `.field`, `.border`, `.colour`.

# Reaching a link

In Edit, a click on a link selects it (`canvas::clicking::annot_hit`). In Read
and Review a click follows it. The pick filter's Link row is on by default;
turning it off stops clicks selecting links.
