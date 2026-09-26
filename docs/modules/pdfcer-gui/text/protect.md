# `text::protect` — every operator-facing string on the two Security
controls that **write** protection into a file

`OPERATOR_REQUESTS.md` **O119**, approved 2026-09-04 with the instruction
that ended the question: *"yes add encryption and permissions … Always add
new features. never ask. just do."*

## Why this is a second module rather than more of [`crate::text::security`]

Three reasons, and the first is the one that decides it.

1. **The two modules make opposite kinds of claim.** `text::security` is the
   READ side: it reports what a document already says about itself, and its
   header's standing rule is that *"it makes claims about what protects a
   document, and a wrong one is worse than silence."* This module is the
   WRITE side: every sentence here describes something pdfcer is about to
   **do to a file**, and the failure mode is not a wrong report, it is a
   wrong file. Mixing an "it is like this" catalogue with a "this is what
   will happen" catalogue would put the two under one reviewer's eye with
   one standard, and they need two.
2. **R2.** `text::security` is 375 lines of dense argument and several of
   its neighbours in `text/` are at or near the 1,500-line ceiling. The
   build brief for this work names a new module for exactly that reason.
3. **Everything the read side already wrote is REUSED rather than
   restated.** [`crate::text::security::permissions_are_advisory`],
   [`crate::text::security::permission_name`],
   [`crate::text::security::permission_state`],
   [`crate::text::security::cipher_line`],
   [`crate::text::security::auth_line`] and
   [`crate::text::security::not_encrypted`] are all called from the protect
   dialog verbatim. **A second wording of one fact is the defect**, and it
   is the defect `text::security`'s own header names about the engine and
   the CLI. So the split is by *kind of claim*, not by subject, and nothing
   is duplicated across it.

## ★★★ The three disclosures, and why they are three

`OPERATOR_REQUESTS.md` O119 lists three things the operator said "change the
answer" before he gave one. All three are on screen, and the surface may not
ship without them:

| # | fact | where it comes from | where it is drawn |
|---|---|---|---|
| 1 | **a permission is a request, not a lock** | the ENGINE's own sentence, already catalogued at [`crate::text::security::permissions_are_advisory`] | at the top of the permission list, in the danger role, on every job that writes permission bits |
| 2 | **a signed document is refused** | [`signed_refusal`] here | instead of the whole form — the dialog opens, states it, and offers nothing |
| 3 | **re-permissioning needs the owner password** | [`owner_password_note`] here | above the current-owner-password field, on every job that touches an already-protected file |

★ Number 1 is the important one and it is **not re-worded here**. The engine
supplied it, the CLI prints it, `text::security` catalogued it, and
`EncryptionSettings::PERMISSIONS_DISCLOSURE` is the same sentence in the
engine's own source. A UI that presented permissions as enforcement would be
lying, and softening the clause *"a request, not a lock"* is exactly the edit
a marketing instinct makes.

## What is deliberately NOT said

No sentence here promises that a protected document is safe, secure, or
locked. The only true version of that claim is about the **user password**
and nothing else, and [`crate::text::security::permissions_are_advisory`]
already says it in the engine's own words.
