# `pdfcer-gui-base/ribbontabs/security`

The **Security** tab — *who may open, change or trust this file, and what
must never leave it?*

`RIBBON_IA.md` §5.9. Two groups: Security, Protect. The operator's request,
`OPERATOR_REQUESTS.md` O289 item 10: one tab holding the security and the
protect tools.

# Two groups, because they are two kinds of verb

**Security** — Remove old passwords, Encrypt, Allow edits under RC4, Unlock,
Permissions, Sign, Add validation evidence, Add archive time-stamp — writes
something about the *file*: a save transform that rewrites the document and
enters nothing in the undo log.

**Protect** — Redact, Redact selection, Off-page, Apply redactions — removes
content from *pages*. Marking is undoable; applying is not, and both
tooltips say so. Mark comes before apply, and the pair stays in that order.

Keeping them in separate bands is what stops one tab from saying the two
are the same kind of verb.

# Which mode sees what

The tab is in all three modes, so Read can encrypt, sign and inspect
permissions — each produces a new document rather than changing the one on
screen, which Read permits.

Protect is `.shown_when("mode.edit_content")`: redaction changes page
content, and only Edit mode may. In Read and Review the Protect band is
absent, not greyed (R9), and the tab shows the Security band alone.

`Capabilities::for_mode` derives `edit_content` from whether a mode lists the
**edit** tab; listing **security** grants nothing, so adding this tab to Read
did not hand Read redaction.

# Collapse order

`ribbonladder` folds Protect first: Security is the band the tab is named
for.
