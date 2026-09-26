# `app::modes::defaults` — what a mode's arrangement *is*

One subject: the **default arrangement per mode**. Which panels Read,
Review and Edit mount, on which side, stacked with which others, and how
wide those docks start. It is a pure function from a mode id to a
[`DockLayout`], and nothing in it can reach a document, a dock, a layout
file or a [`super::Modes`].

[`super`] owns the other half of the same feature — how an arrangement is
**remembered** once the operator has rearranged it.

## Why this is its own file

Split from `app/modes.rs` when that file reached 1,512 lines against the
1,500-line gate (R2). `app/mod.rs` has been split twice under the same
rule, into `crate::app::dispatch` and `crate::app::conditions`, and the
standing instruction for this gate is the one written into the gate
itself: *"the right response to this gate firing is to SPLIT THE MODULE,
not to shrink the prose."* This project's documentation is the logic, so
trimming it to fit is the one response that would make the file smaller
and the program less well specified.

**The seam is a real one rather than arithmetic**, and the test for that
is whether the two halves change for different reasons. They demonstrably
do:

* This file changes when the **information architecture** changes — when
  `MODES_AND_PANELS.md`'s table is amended, when a panel is invented, when
  the operator answers a taxonomy question. Read carrying Forms on its
  right — the operator's own answer to a question this module had been
  holding open — is exactly such a change.
* `super` changes when **persistence** changes — a new workspace naming
  rule, a new upgrade-reconciliation case, a different start-up order. The
  `Unseen` stamp, so a panel added in a new release is not born invisible,
  is exactly such a change.

The two are also readable at different times. Someone asking *"why does
Read have no Objects panel"* never needs to know how a workspace is
named; someone asking *"why did my arrangement come back wrong after an
upgrade"* never needs the taxonomy. And the dependency runs one way only:
`super` calls [`layout_for_build`], while this file calls nothing in
`super` at all.

## What `egui-shell` cannot supply

`SHELL_FRAMEWORK.md` §4 and `egui-shell`'s workspace store between them
make Read/Review/Edit a *configuration* rather than a built-in — see
[`super`]'s header for that rule and for how [`super::Modes`] honours it.

What *is* pdfcer's business, and therefore is here, is the **default
arrangement per mode** — see [`layout_for`]. `egui-shell` cannot supply
that: it does not know what a panel is for, so a default arrangement
invented there would be the framework inventing an application's
information architecture.

## The three defaults, and where they come from

`MODES_AND_PANELS.md` Part 1's table, reduced to what the *dock* does:

| Mode | Left | Right |
|---|---|---|
| **Read** | Pages, Bookmarks | Comments, Forms, **Document properties** |
| **Review** | Pages, Bookmarks | Comments, Properties, Forms, Dimension groups, **Document properties** |
| **Edit** | Pages, Bookmarks, Layers, Signatures, Fonts | Objects / Properties, Comments, Forms, Redact, Dimension groups, Attachments, **Document properties** |

**Document properties is in all three, and it is the only panel besides
Pages that is** — the operator: *"the document properties are
still always visible in the properties tab. it needs to get out of there and
be in its own document properties tab."* Reading a document's title is
reading, so no mode is withheld from it; its command sits on File ▸ Document
and every mode is shown the `file` tab, so no mode can mount it without
being able to reopen it. The arms below carry the placement within each
stack.

Read is the point of the whole feature — *"a PDF viewer, with pdfcer's
inspection panels available but nothing that authors anything"* — so its
default mounts the two surfaces that answer *where am I* and nothing that
merely describes an object you are not allowed to edit. **Forms on its
right is the one amendment to that sentence**, on the operator's own
answer to the open question; the arms of `spec` carry the full reasoning
and it is not repeated here. Review adds the two surfaces
markup work needs. Edit is everything, with **Objects on the right**,
opposite the navigators, because an inspector and a navigator are
consulted in different directions.

A mode this module has never heard of gets the **full** arrangement: a
mode with no opinion recorded about it should not have panels taken
away, because removing is the opinionated act.

## Panels this build does not have

**None — [`ABSENT_PANELS`] is empty.** The list is kept anyway, because
there are two opposite ways to get it wrong and the same mechanism catches
both.

**A blocker recorded from the wrong end** holds back a surface nothing is
blocking. *"Annotation authoring does not exist yet, so neither does the
panel that lists comments"* is false on its merits: listing what a document
already carries needs no authoring, and the Comments panel works against
`pdfcer_core::annot` while this shell still cannot place a single markup.
An **id** recorded the same way is the same failure — `view.panel_comments`
is a guess, while `RIBBON_IA.md` §7's migration map puts the control on
Markup ▸ Comments, and an id no code has ever resolved is a guess by
definition.

**A panel whose body exists and whose COMMAND does not** goes the other
way: it is correctly not an absent panel and is still not reachable.

Both are the `SHELL_FRAMEWORK.md` §5b mechanism rather than an oversight:
[`layout_for_build`] filters every default through the live
[`PanelCatalog`], so an id nothing registers is simply not mounted —
whether what is missing is the body or the command.

Writing the *intended* arrangement and filtering it is strictly better
than writing only what exists today, because the alternative is that the
intent lives in a document nobody re-reads when the panel lands. The
Pages panel is the worked proof of that: it was built long after these
defaults named it, and the day its command is registered it appears in
all three with **no edit in this file at all**.
`every_default_panel_is_registered_or_declared_absent` is what keeps
[`ABSENT_PANELS`] honest in both directions.
