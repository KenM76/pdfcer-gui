# `dialogs::settings::widgets` — the three shapes every setting is made of

Seven group modules draw thirteen settings, and every one of them is built
from the three functions here. That is deliberate: a settings window whose
entries are hand-laid-out drifts into thirteen slightly different layouts
within a year, and the reader notices the inconsistency before they notice
the content.

## [`header`]'s signature is where obligation 2 and 3 are enforced

`crate::dialogs::settings`' header names three things this window must show
that a conventional settings screen omits. Two of them are properties of
*every* setting, and rather than trusting each group module to remember
them, they are **required arguments**:

```text
header(ui, title, silence, radius)
              │       │       └── which way costs what
              │       └────────── what the standard leaves open
              └────────────────── what the setting is
```

A setting cannot be added without answering all three, because the code
does not compile otherwise. `crate::text::settings` mirrors the shape —
`*_title`, `*_silence`, `*_radius` for all thirteen — and its own tests
assert none of the answers is empty.

Obligation 1 — *whether the default is a guess* — is **not** enforceable
this way: it belongs to one option rather than to the setting, and only
some options have anything to say. It is pinned by a test over the catalog
instead.
