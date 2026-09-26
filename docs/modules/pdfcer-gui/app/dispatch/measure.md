# `app::dispatch::measure` — arming a measure tool, and the two windows
behind it

## Why this is a module

Every `measure.*` command is here, and they share two rules that must be
stated once: the capability they all gate on, and the way an authoring
group is resolved. Split across `super`'s match they were restated per arm,
and restated rules agree only by inspection.

[`active_group`] takes the **command id** so its trace line names the
command that fell back. That is the one thing a per-arm copy of the
resolution got right, and it is why the shared helper is not argument-free.

## What the fallback is for, and why it is traced rather than silent

`canvas::measure::active_group` returns `None` when the measure tool has
never been armed this session — there is no state in `egui::Memory` and no
group has been chosen. Substituting the default group is the **right**
answer for an operator who has drawn nothing, and the **wrong** one for
anybody whose state was somehow lost.

Both look identical afterwards — *"a group got a scale"* — so the fallback
says so on the trace rather than being silent. That is the whole reason a
`None` is not quietly turned into a default at the source.

## The capability gate is one sentence, repeated per arm

Every command here declines in a mode that cannot author a ce dimension, and
they must decline **alike**: a mode that cannot place a dimension has no
business calibrating the group they live in, creating one, or ending a fit.
Differing refusals for one capability read as arbitrary, so every arm below
traces `reason=mode-cannot-author-measure` and nothing else.

## Item notes

### `fn active_group`

See the module header: `None` means the measure tool has never been armed
this session, and substituting the default group is right for an operator
who has drawn nothing and wrong for anybody whose state was lost. The two
are indistinguishable afterwards, so the substitution says so.

`id` is in the trace line so a reader can tell which command fell back.
Without it the trace says a substitution happened and not what asked for it.
