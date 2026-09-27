# `ui-verify/checks/password_fill`

`a_typed_password_is_not_saved_unless_asked` — filling a password field stores
nothing and says so off-canvas, and the Forms panel stores it when asked.

# What it drives

Fixture `fixtures/password-field.pdf`: one text field with the Password flag
(`/Ff` bit 14) and no `/V`.

1. Read mode: click the field on the canvas, type `4711`, press Enter.
2. The `form-fill-text` line after the Enter must carry `password_withheld=true`.
   That is the engine's `FillOutcome::password_value_withheld` passed through;
   `false` means the value went into the file as plain text or the report was
   dropped.
3. The status bar's fill-disclosure group must be drawn after the fill.
4. Review mode: open the Forms panel and click `forms.fill.store_password`.
5. A `form-fill-storing-password` line must follow, with `commands=1` and
   `password_withheld=false`: `fill_text_field_storing_password` ran and stored.

# Why the canvas, then the panel

The canvas is where the operator types. The button lives only in the panel,
because it is a deliberate second act, not part of the fill.
