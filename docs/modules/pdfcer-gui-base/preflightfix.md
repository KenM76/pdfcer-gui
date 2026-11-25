# `preflightfix` — a Preflight fix, from the panel to the action handler

**Purpose.** The Preflight panel is handed `&OpenDoc` and cannot edit, so a
Fix click raises `Action::PreflightFix(PreflightFix)`: the profile, the
report the fix is computed against, and the rule (`None` for Fix all). The
handler parks the engine's report as `FixNote { epoch, text }` on
`OpenDoc::preflight_fixed`.

**`rule_fixups(rule)`.** The fixups a rule offers: its own when it names
any, else `Fixup::suggested(&rule.check)`. The panel shows Fix only where
this is non-empty, and the handler runs exactly these, so what is offered
and what runs cannot disagree.
