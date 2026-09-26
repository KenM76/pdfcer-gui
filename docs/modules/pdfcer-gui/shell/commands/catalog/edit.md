# `shell::commands::catalog::edit` — the Edit tab — changing content that is already there

One band of [`super::all`]'s catalogue: the registrations for the Edit tab,
and the argument behind each one.

## Why the band is a file

A tab's registrations and the prose that justifies them are one subject, and
they are re-read together. Splitting the catalogue per tab does **not** hide
a handler-token collision — `super::super::tests::every_handler_token_is_unique`
sweeps the whole registry and `every_handler_token_is_in_its_tabs_block`
asserts each token sits in its own tab's hundred, so a collision is a red
test in any arrangement.

## One picture per control, and where that rule bites hardest

[`super`]'s header permits a shared icon key across a *family* — `copy` on
both copy verbs, `redact` on both redaction verbs — because a family sharing
a glyph is how a ribbon reads as grouped. **That convention does not reach
controls whose difference is the thing the picture would have to show**, and
this band holds the two worst cases in the application:

* **The five form-field placers.** `edit.form_text_field`,
  `edit.form_check_box`, `edit.form_radio_button`, `edit.form_choice` and
  `edit.form_push_button` sit in ONE ribbon group, drawn side by side. A
  shared `form-field` key gives five different words under five identical
  pictures — and the field TYPE that the picture is supposed to show is
  exactly what distinguishes them. Each therefore owns a key:
  `check-box`, `radio-button`, `drop-down`, `push-button`.
* **The three redaction commands.** `edit.redact` arms a marking tool,
  `edit.redact_selection` marks what is already picked, and
  `edit.redact_apply` destroys content irreversibly. Three verbs, not one
  verb over three operands, so `redact-selection` and `apply-redactions`
  are separate from `redact`.

## What a discharged refusal leaves behind

Several registrations below record that a control **refused** a glyph and
now has one. Those paragraphs are rewritten in place rather than deleted,
and the part that outlives the refusal is the load-bearing half: **which
neighbouring glyph the control must never be redrawn to resemble.** That
constraint is exactly what a later "make this group look consistent" pass
would break, and the refusal is where it is written down.

⚠ **Who wrote a refusal is part of the refusal.** A build session's argument
quoted back often enough begins to read as an operator ruling, and a
refusal attributed to the operator is one nobody revisits. Where a
registration below names an author, the attribution is doing work.

⚠ **This band's registrations feed an arithmetic that lives elsewhere.**
`super::super::tests::the_icon_coverage_split_adds_up_to_the_registry` pins
named-versus-refused as two literals over the whole registry; adding or
discharging a glyph here moves them. The count is never restated in prose,
here or in [`super`]'s header — the test is the only copy.

## What is here, and what is not

The `Command` entries and the argument for each one's label, tooltip,
handler token, icon and enable predicate. **The prose is the point** — most
of this file is the record of decisions that would otherwise be
re-litigated.

Not here: the registration itself ([`super::super::register`]), the
command-id-to-behaviour mapping ([`super::super::mapping`]), and the
reachability register ([`super::super::reach`]).
