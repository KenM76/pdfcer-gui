# `app::dispatch::compare` — `tools.compare`

Shows the Compare panel, asks for the older version through
`app::files::pick_compare_source`, and starts the comparison with the
panel's current options. Nothing happens without an open document (the
command is greyed then, `doc.pages`). A cancelled pick leaves the panel as it
was: the panel is shown first so that the operator who cancels still sees
where the feature lives.

Routed before the dispatcher's panel guard, which would only show the panel.
