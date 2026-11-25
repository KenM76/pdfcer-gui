# `panels::preflight` — the document checked against a preflight profile

**Purpose.** Check the open document against one of the engine's built-in
profiles (`Profile::BUILTIN_NAMES`) or a profile file, list what fails grouped by rule, go to a
finding, copy the report, and fix a rule or every fixable rule (O458).
Acrobat's Preflight, beside Output Preview as Acrobat groups both under
Print Production (`RIBBON_IA.md`, Preflight amendment).

**The run (`job.rs`).** Run forks the session with
`EditSession::fork_for_print` — the same base, a copy of the overlay, no
history — and hands the fork to a thread named `pdfcer-preflight`, which
calls `preflight::run_with_meter` over `session.view()` with a
`pdfcer_render::TacMeter` (so the total-ink rules measure rendered coverage),
the panel's cancel flag and a progress callback. The worker never holds the
open document's `Arc<EditSession>`, so an edit made during the run proceeds
at once (`vector_edit` needs sole ownership) and only makes the answer
stale. Dropping the `Job` — another document, the panel state reset — sets
the cancel flag. The UI thread polls the channel once a frame and never
waits on it.

**State (`PreflightUi`).** The picked profile, the run in flight, and the
last result, each keyed on the document's `serial` (another document drops
it) and `edit_epoch` (an edit marks it stale). A stale result says so, its
Run button reads Run again, and its fixes are greyed with the reason: a fix
is computed against findings that no longer describe the document.

**Results (`results.rs`).**

- Grouped in profile rule order with `rule_matches`, each rule with its
  severity and count; a finding no rule claims gets its own group rather than
  vanishing.
- Each row names its kind — Violation, Needs a person, Incomplete — so none
  reads as a pass; then the page, the engine's message and the measured value
  against its limit.
- The headline: clean (`PreflightReport::is_clean`, which requires no
  findings, nothing dropped and not cancelled) or the kind counts; a
  cancelled run says how many pages it checked; `dropped` is disclosed.
- At most 200 rows are drawn per rule and the rest counted; Copy report
  (`report_text`) carries every finding.

**Picking a row.** Turns to the finding's page (`GoToPage`); a
`FindingTarget::PageObject(i)` finding is selected with the canvas's own
selection (`SelectionAction::SelectObject`, `TargetId::Object(i)`), so
Properties, Delete and the handles act on it; a finding with a rect is
revealed (`RevealRect`, why `preflight-finding`). The page is never marked:
no tint or badge (R8b); a selection is the cursor. A pick leaves Preflight in
front of its stack (`app::forward`), so the next finding is a click away;
Properties is one tab over.

**Fixes.** A rule's Fix appears when the rule names fixups, or
`Fix::suggested` offers one for its check (`preflightfix::rule_fixups`); Fix
all appears when any group is fixable (R9: no stub otherwise). Both raise
`Action::PreflightFix`, run by `app::actions::preflightfix`. Both are greyed
with a hover reason outside Edit mode (`capabilities().edit_content`, so
Read and Review: a fix rewrites content) and on a stale result. The
engine's `FixupReport::to_text` is shown under the results until a run
starts after it.

**Trace.**

- `preflight-started profile= pages= epoch=` (`profile=` is the `BUILTIN_NAMES` key)
- `preflight-progress done= of=` (on change)
- `preflight-finished profile= findings= dropped= pages=checked/count ms=
  cancelled=0|1 frames=` — `frames` counts UI frames drawn while the run
  was still going, the evidence that the window kept answering.
- `preflight-rows n= list=row:rule@page:target,…`
- `preflight-picked rule= page= target= selected=` (1-based page; `-` for
  none)
- `preflight-copied bytes=`
- `preflight-report-saved ok= bytes=`, `preflight-profile-saved ok= bytes=`,
  `preflight-profile-loaded ok= rules=`; a failure ends with `why=`.
- Regions: `preflight.body`, `preflight.run`, `preflight.cancel`,
  `preflight.copy`, `preflight.save_report`, `preflight.load_profile`,
  `preflight.save_profile`, `preflight.fix_all`, `preflight.fix.<rule>`,
  `preflight.row.<n>`.

**Files (`files.rs`).** Save report… writes `report_text`, the engine's only
report format, through the save dialog (`.txt`). Load profile… reads a file
with `Profile::from_json`; on success the profile joins the picker under the
key `custom`, labelled with its own name, and is picked. A refused file names
`ProfileError`'s sentence under the picker and leaves the picked profile as
it was. Save profile… writes the picked profile (built-in or loaded) with
`Profile::to_json`, the form Load reads back. A loaded profile carries no
standard verdict: the verdict is keyed on the built-in names. Seams for
`ui-verify`: `PDFCER_DIAG_PREFLIGHT_REPORT_SAVE`,
`PDFCER_DIAG_PREFLIGHT_PROFILE_LOAD`, `PDFCER_DIAG_PREFLIGHT_PROFILE_SAVE`;
an empty value is a cancel.

**Not built.** A profile editor; a structured (machine-read) report, which
the engine does not offer.

**Driven by.** `the_preflight_panel_runs_jumps_and_fixes`,
`the_preflight_panel_saves_its_report_and_loads_a_profile`.
