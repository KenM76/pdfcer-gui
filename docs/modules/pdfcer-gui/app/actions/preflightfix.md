# `app::actions::preflightfix` — a Preflight fix through the edit funnel

**Purpose.** Run a Preflight panel Fix or Fix all click as one undo step.

**Contract.**

- `rule: None` (Fix all) calls `EditSession::apply_profile_fixups` with
  `suggest = true`, so a rule naming no fixups gets the engine's suggestion.
- `rule: Some(id)` with one fixup calls `apply_fixup` with
  `FixupScope::Rule { rule_id, findings }`; several fixups on the rule run
  `apply_profile_fixups` over a one-rule copy of the profile, which keeps
  them one step.
- Both go through `funnel::vector_edit`, which cancels the render worker,
  bumps `edit_epoch` and re-syncs the pages. A session another thread still
  holds is the funnel's refusal; nothing runs and no note is left.
- The engine's step disclosures become the edit's disclosures; the report's
  `to_text()` is parked on `OpenDoc::preflight_fixed` with the new epoch.

**Tools.** `FixupTools` carries `IcccerConverter` and
`pdfcer_emf::RecordingFlattener`, so the convert-colours and flatten fixups
can apply. A fixup still missing what it needs (an ICC profile for
convert-colours on a document with no output intent) reports itself
unavailable in the shown report.

**Trace.** `preflight-fixed scope=<rule|all> steps= changed=0|1 undo=
outcomes=fixup:outcome,…`.
