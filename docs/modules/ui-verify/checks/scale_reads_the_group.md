# `ui-verify/checks/scale_reads_the_group`

`set_scale_reads_the_group_it_is_about_to_overwrite` — the Set-scale window
names its group, states the scale already set, and keeps what the operator
typed when they step out to point at the page.

# The three reports this is the oracle for


> **O192** — *"the set scale dialogue does not show me the scale that is
> already set"*

> **O193** — *"there is no group dropdown"*

…plus a third he had not got to yet, which this check asserts because it is
the one that could cost him work: pressing **Measure it on the drawing…**
destroyed everything already entered. The window was closed
(`DialogsState::close_scale`) and a fresh one built from
`ScaleEntryFields::default()` when the pick completed, so an operator who
chose metres, typed a ratio and set the number style before deciding to
measure a line came back to a window that had forgotten all four — and one
who pressed Escape mid-pick came back to no window at all.

The cause of all three was one sentence: `ScaleDialog::show` took a context
and an action queue, so the window **physically could not see the document
it was editing**, and `dialogs::open` destructured `status` only to prove a
document existed and then threw it away.

# ★★ Why every part of this needs DRIVING, and unit tests cannot stand in

`ScaleEntryFields::for_group` — the inversion that makes O192 possible — has
five unit tests including an independently calibrated one, and they would
all pass on a build where nothing ever calls it. That is this project's
oldest lesson in its sharpest form: *unit tests that call the verb cannot
see the chain in front of it.* The chain here is six links and four are
frame-level:

1. a ribbon press reaches `open_scale`, which must now **keep** the
   `OpenDoc` it proves it has;
2. `ScaleDialog::open` must seed from that document rather than from
   `for_group_panel`'s defaults;
3. `DialogsState::show` must pass `doc` through, which it did not before;
4. the picker and the current-scale line must be **drawn**, inside the
   window's body, not merely computed;
5. arming the pick must **hide** rather than close;
6. the pick completing must **deliver into** the surviving window rather
   than build a new one.

Every one of those is an edge read once per frame. Only a running window
sees them.

# ★★★ The assertion that carries the whole thing: ONE `scale-open`

Step 7 counts `scale-open` trace lines across the entire session and
requires exactly one. That is the difference between *hidden* and *closed*
stated as a number a machine can check, and it is the only assertion here
that a plausible-but-wrong build cannot satisfy by accident:

- a build that closes and rebuilds emits **two**;
- a build that closes and never comes back emits one, and fails step 6
  instead;
- a build that hides correctly emits one, because the constructor ran once.

An assertion on *"the dialog is on screen afterwards"* alone is satisfied by
both the fixed build and the broken one, and would have passed throughout
the defect's life. Naming what the wrong mechanism **cannot** produce is the
rule this check was written under.

# The O192 assertion is a ROUND TRIP through the real application

Asserting that `scale.current` is drawn proves a label exists; it does not
prove the label says anything true, and on a fresh fixture every group is
uncalibrated, so a window that invented `1:100` and a window that read the
document would print the same defaults. So this check **sets a scale first**
— through the two-point calibration, typing a real length, pressing Accept —
and then reopens the window and requires the seeded ratio to have **moved**.

That is the operator's exact complaint, driven: open it a second time and
see the number you set. A build that seeds from defaults reopens at the
same ratio it opened at the first time, which is what this measures.

# What this check deliberately does NOT assert

That the group picker lists more than one group. Creating a second group is
`dimension_groups_panel_makes_a_group`'s gesture and its fixture, and
duplicating it here would make two checks fail for one cause. What is
asserted is that the picker is **drawn inside the window's body** — the
failure mode `D:/dev/rag/egui/` records as panels that shipped unreachable
in real builds with every gate green.
