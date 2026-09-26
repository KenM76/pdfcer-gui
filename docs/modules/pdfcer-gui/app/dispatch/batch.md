# `app::dispatch::batch` — the Tools ▸ Batch band's arms

One command today, `tools.merge_files`, and it is here rather than inline in
[`crate::app::dispatch`] under **R2**: that file is at 1,400 of the 1,500
ceiling and this codebase's comment density means an inline arm carrying its
own argument would consume most of the remaining headroom. It is the seventh
such split and the reasoning is the one the six before it recorded.

## What this closes, and what it says about the check that missed it

`OPERATOR_REQUESTS.md` row **O68**: *"the Merge files and Split files
buttons don't do anything."*

They were registered, drawn on the Tools tab with icons and tooltips, and
had no arm anywhere in the dispatcher. A press fell to the catch-all and
traced `command-unimplemented`, which no operator can see.

There **is** a gate for exactly this — `every_registered_command_is_routed_
or_argued` — and it did not fire, because both ids were entered on the
`SCAFFOLDED` allow-list with a written paragraph beside them. That is the
finding worth more than the fix:

> **An allow-list whose entries are prose can only ever force an
> explanation, never a fix.**


⇒ The runtime replacement is in `tools/ui-verify`: press every registered
id and fail on any `command-unimplemented` line. That is a claim about the
running program, and no paragraph can satisfy it.

## The two pickers, and why there is no dialog between them

Choose the sources, choose the destination, done. A *Combine Files* has
nothing left to ask — it takes every page of every source, in the order
given — which is the same argument `pages.merge_into` makes for opening no
dialog at all. A window offering options nobody has asked for would be
ceremony, and the two things an operator might eventually want (reorder the
sources, take a subset of a source's pages) are features with their own
designs rather than defaults this verb is missing.
