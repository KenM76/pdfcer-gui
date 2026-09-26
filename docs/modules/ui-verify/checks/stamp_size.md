# `ui-verify/checks/stamp_size`

`stamp_size_reaches_the_engine` — the Size chooser in the stamp dialog is
pressed for real, and the number the operator picked is asserted to arrive
at the engine call.

# The report this closes


> *"I have to draw the size before it gets applied … still can't adjust the
> size of a stamp on the canvas, or by entering a different size in the
> properties box."*


⚠ **It shipped in `v0.5.0-dev.20260910.1` undriven**, because the release was
asked for immediately. `OPERATOR_REQUESTS.md` O168 records that as a debt in
those words rather than folding it into a green report. This file is the
payment.

# ★★★ Why unit tests could not have closed it, in this project's own words

*"Unit tests cannot see the chain in front of the verb"* — the standing
lesson from the day eight green tests sat in front of a feature that did 1
of its 14 steps. The chain here is four hops long and every hop has a
passing test:

| hop | its test | what that test cannot see |
|---|---|---|
| combo writes `self.stamp_size` | `a_fresh_dialog_is_empty_and_defaulted` | whether the combo is reachable with a pointer at all |
| dialog puts it on the action | `the_chosen_icon_reaches_the_commit_action` | it sets the field directly; no widget is pressed |
| action carries it to `spec` | the `Placement` tests | they construct a `Placement` by hand |
| `spec` builds the style | `canvas::textannot::tests` | it calls `spec` with the value already chosen |

Every one of those passes on a build whose combo is drawn **underneath the
Accept button**, or clipped off the bottom of a window that did not grow, or
whose popup opens off-screen. `each_kinds_window_is_as_tall_as_its_body_needs`
asserts the window grew — it does not assert the control inside it can be
hit. This project has shipped a panel that was unreachable in a real build
with every gate green.

# ★★ The oracle, and why the application grew a trace line for it


```text
pdfcer-diag stamp-style size=24 fit=grow rect_w=220.0 rect_h=220.0
```

`size=` is `StampSize::trace_token`'s output — the bare number for a
stated size, the literal word `derived` when the operator let the drawn box
decide. Its contract is asserted by
`canvas::textannot::tests::every_trace_token_is_parseable_and_distinct`,
because a token that drifts makes this check fail to **match** rather than
fail to build, and a check that cannot find `size=24` reports *"the
operator's choice did not reach the engine"* — a defect report about the
application, written by a defect in the harness.

# ★★★ The falsification, which is the part that makes this a test

**The size pressed is deliberately not the default.**

`DEFAULT_STAMP_SIZE` is `StampSize::FitTheBox`, which traces `derived`. If
this check pressed *Fit the box I drew* it would pass on a build that
ignores the combo entirely, ignores the whole `stamp_size` field, and hard
codes the old behaviour — which is precisely the silent decline
`canvas::textannot`'s header warns about, since `font_size: None` compiles
and means exactly that. So it presses **24 pt** and asserts `size=24`, a
value no build can produce without having read the operator's choice.

⚠ And it asserts the default **first**, before pressing anything, for the
other direction: a build that opened the chooser on 12 pt would already have
shrunk every stamp on the operator's drawings, and would still pass a check
that only looked at the end state after an explicit selection.


*"A check that cannot fail is not evidence."* This one passed on its first
run, which is worth exactly nothing until the passing has been shown to be
contingent. Each defect below was planted in a real release build, the build
was driven, and the file was restored **from a kept copy** rather than
through git.

| # | planted defect | one line | outcome |
|---|---|---|---|
| 1 | `dialogs::textannot` puts `DEFAULT_STAMP_SIZE` on the action instead of `self.stamp_size` — the dialog→action hop drops the pick | the exact silent decline the feature was built against | **FAIL**, at step 7, reporting `size=derived` and naming the hop |
| 2 | `TextAnnotDialog::open` starts the chooser at `StampSize::Points(12)` — the engine's flat default adopted | every stamp on his drawings shrinks, nobody presses anything | **FAIL**, at step 5, before any control is touched |

★★ **Defect 1 is the one that proves the check is not merely watching
itself.** The chooser still reported `24` under it — step 6 was green — and
only the commit disagreed. A check that had asserted the control's own state
and stopped there would have passed on a build that dropped the operator's
choice on the floor one hop later.

⚠ **Step 6 itself was NOT falsified**, and that is stated rather than left
for someone to assume from the table. Planting it means making an egui
`selectable_value` write a field without repainting its own combo, which is
not a defect this shell can express in one line — it would be a bug in the
framework. It is asserted because it is the operator's only confirmation,
not because it has been shown to be able to fail.

# What this check does NOT claim

It does not assert what the engine draws. `size=24` proves the shell handed
`StampStyle { font_size: Some(24.0), fit: GrowToText }` to
`add_text_annotation_with`; whether `pdfcer-core` then paints 24 pt text and
widens the `/Rect` is asserted by the engine's own tests. Stated because a
green line here must not be read as a claim about `pdfcer-core`'s renderer.
