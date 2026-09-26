# `text::unembed` — what the Remove-fonts window says before it takes
something out

The copy for [`crate::dialogs::unembed`], and the destructive twin of
[`crate::text::embed`].

## This is the window `tools.unembed_fonts` was blocked on, and the
blocker was TRUE

Nine of the project's scaffolded commands turned out to be sitting behind
reasons that had expired. **This one was not.** Its recorded reason said:

> `panels/fonts.rs` records that the old shell's confirmation window exists
> because *"three of unembedding's four consequences are invisible on the
> canvas (a broken PDF/A claim, an invalidated signature, a renamed font)"*.
> That disclosure surface is rule 4 work and is not built.

It is built here. Every sentence below exists because of one of those four,
and the module is worth reading as the argument for why the window could not
have been skipped.

## The FOURTH consequence, which nobody had written down

**Unembedding does not make the file smaller when pdfcer saves it.**

`UnembedPlan::bytes_reclaimable` is the number an operator is chasing, and
the engine is explicit about the trap: §7.5.6's update section is *appended*,
so the deleted objects get free cross-reference entries in a new section and
**their bytes remain in the prior revision, which is still in the file.** An
incremental save after an unembed produces a *larger* file. Only a full
rewrite drops the bytes.

`crate::app::save` writes **incrementally, always**, by design and by a
promise in a tooltip that has been on an operator-visible surface since the
command was registered. So this shell cannot deliver the reclaimed bytes at
all today.

⇒ The engine's own rule is *"this number must never be reported without the
save mode that delivers it"*, and the honest reading of that here is not to
soften the number — it is to **state the number and then state that pdfcer's
Save will not deliver it**. Filed as an operator question in
`OPERATOR_REQUESTS.md`; hiding it would make the window a sales pitch.
