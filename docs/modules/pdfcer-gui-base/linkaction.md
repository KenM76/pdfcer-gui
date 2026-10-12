# `linkaction` — the link verbs as actions

`LinkAction` is the `Action::Link` family: one variant per `EditSession` link
verb, carrying that verb's arguments as engine types.

- `Add{page, rect, target, border}` → `add_link`. `page` is 0-based; `rect`
  is in page space.
- `SetTarget{id, target}` → `set_link_target`.
- `SetBorder{id, border}` → `set_link_border`; `None` removes the border.

Applied by `pdfcer-gui`'s `app::actions::links`.
