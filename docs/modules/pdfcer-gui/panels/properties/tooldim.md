# `panels::properties::tooldim` — the dimension tools' group and direction

`OPERATOR_REQUESTS.md` O288 items 4–6. Drawn by `tool::section_in` for
`Block::DimensionTool` (every measure tool except Scale) and above the
circular tool's pick list.

## What it offers

| control | what it sets |
|---|---|
| *Add to group* combo | `canvas::measure::set_active_group` — the group the next ce dimension's `DimensionAction::Commit` names |
| *New group…* entry, name field, *Create* | `DimensionAction::AddGroup { scale_from: Some(active), author_into: true }` |
| *Direction* radios (linear tool only) | `canvas::measure::set_linear_constraint` — `LinearPick.constraint`, which `placing_kind` and `commit_point` already honour |

The groups panel offers the same *draw into* choice; both write the one
store, so neither can disagree with the other.

## Contracts

- **The combo never names a group the commit cannot reach.** A group deleted
  from under the tool resets the store to `DEFAULT_GROUP_ID`, the one group
  that always exists.
- **A group made here starts from the current group** — its unit, and its
  scale converted into that unit (`actions::dimensions::add_group`). An
  empty group with no scale would make the next ce dimension read raw page
  units, which is not what an operator making a group *for this drawing*
  means.
- **The new group becomes the authoring group after the edit applies.** The
  apply phase has no `egui::Context`, so `add_group` hands the id to
  `canvas::measure::queue_active_group`, and the next `measure::read` adopts
  it (`dimension-authoring-group via=created`).
- **The draft name lives in egui memory** (`DRAFT_KEY`, `Option<String>`):
  `Some` while the row is open, cleared on Create or on picking a group.
- **Every region is published with `ui_rect_visible`**, for the reason
  `tool.md` gives: this panel scrolls.

## Trace

| line | when |
|---|---|
| `dimension-authoring-group id= via=tool` | a group picked in the combo |
| `dimension-group-add via=tool unit= chars= scale_from=` | Create pressed |
| `measure-direction constraint=` | a direction radio changed |

## Verified by

`ui-verify` checks `a_horizontal_dimension_measures_only_the_run` and
`a_dimension_joins_the_group_made_in_tool_options`
(`docs/modules/ui-verify/checks/dimension_tool_options.md`).
