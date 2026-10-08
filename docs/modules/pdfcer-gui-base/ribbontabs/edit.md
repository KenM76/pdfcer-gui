# `pdfcer-gui/shell/manifest/edit`

The **Edit** tab — *what am I changing about content that is already
there?*

`RIBBON_IA.md` §5.4. Three groups: Content, Insert, Forms. Redaction is
on the Security tab's Protect group (`security.md`).


# Three renames that are the point of the tab

The salvage source's Content group carried three buttons labelled
`Aa`, `I⁺ Aa` and `Obj`. `Obj` is not a word, and the first two
returned the *same string literal* — two adjacent buttons
distinguishable only by icon and tooltip. These are the primary
content-editing tools and they were the least legible controls in the
application. They are now **Edit text**, **Add text** and **Edit
objects**, with the icons kept.

# The `Editing on` master toggle is gone

Operator decision, 2026-08-12: *"make it work the same way other
programs do."*

No mainstream editor has a global editing switch. Acrobat, Bluebeam,
Word and Illustrator all work the same way: selection and Delete are
always live, and picking a tool arms *that tool* until Escape or
another tool. There is no state in which a click does nothing without
the application saying so.

So there is no `Mode` group on this tab and no `edit.editing_enabled`
command. `RIBBON_IA.md` §7's migration map has a row for it — `Edit ▸
ContentTools ▸ Editing on` → `Edit ▸ Mode` — which §5.4 then
supersedes; §5.4 is the later and more specific statement and it is the
one implemented. The command is deliberately **not** in
[`super::PLANNED`] either, because it is not planned: it is deleted.

This matters for how the Read/Review/Edit modes must behave. The rule
that makes those safe — *a mode changes what is **visible**; it never
makes a visible control silently inert* — is precisely the rule the
master toggle broke. A mode **removes** the tools it disables, so
there is no click that mysteriously fails. Reintroducing a global
enable flag under any name would undo that.

# What is absent

The whole **Arrange** group (align, distribute, bring forward, send
backward, group, ungroup, flip) is **N**, so it is not here. So is the
object clipboard — cut, copy, paste, paste in place — and with the two
text-copy commands gone to File ▸ Export there is nothing left for a
Clipboard band to hold, so the band is absent rather than empty.
`Shape ⌄` and `Sanitise…` are **N**.
