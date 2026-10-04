# `text::digital_id`

Every operator-facing string on the Sign window's *Create a digital ID* form
(`pdfcer_gui::dialogs::sign::create_id`). Compiled only with `signing`.

`refusal` gives each `IdError` variant a sentence that says what to change;
the engine's `field` names map to this form's labels (*common name* is
*name*). `IdError` is `#[non_exhaustive]`, so a variant added later falls to a
generic sentence carrying the engine's own words rather than to silence.

`disclosure` is a claim about what a signature proves, and is worded from the
engine's own description of a self-signed ID: integrity and possession, not
identity until trusted. Do not strengthen it.
