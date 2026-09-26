# `dialogs::protect` — the window behind **Encrypt…** and **Permissions…**

`OPERATOR_REQUESTS.md` **O119**, approved 2026-09-04: *"yes add encryption
and permissions"*, under the standing instruction *"Always add new features.
never ask. just do."*

This is the `Ui` half of the feature. Everything that can be decided without
one — what the document says today, which jobs it may be offered, which
engine verb a choice reaches, and the atomic write — is
[`crate::protect`], and that split is the whole reason the rules on this
surface are asserted headlessly rather than by driving a window once and
hoping.

---

# 1. ★★★ The window's shape follows one sentence of the build brief

> *"Show the document's CURRENT state before offering to change it. A
> permissions dialog that opens with everything ticked, on a document that
> forbids printing, has told him a falsehood before he touches anything."*

So the body is **two sections in a fixed order**, and the order is not a
layout preference:

| § | heading | what it is | can the operator move it? |
|---|---|---|---|
| 1 | [`crate::text::protect::standing_heading`] — *"This document, as it is now"* | a **read-back**: the cipher, which password opened it, and every one of the eight permission bits with the document's own three-valued answer | no |
| 2 | [`crate::text::protect::change_heading`] — *"What to change"* | the job, the passwords, the permission ticks, the destination | yes |

Nothing in §1 is a control and nothing in §2 has a hard-coded default —
every tick is seeded from [`crate::protect::Standing::initial_ticks`], which
reads the file. On an unprotected document that read-back is *eight ticks*,
and [`crate::text::protect::permissions_start_open`] says so **in words**,
because eight ticks that are true and eight ticks that are a convenient
default look identical.

# 2. ★★★ The three disclosures O119 named, and where each one is

The operator listed three things he already knows and would notice missing.
All three are on screen, none of them is behind a hover, and none of them
waits for a press:

| # | the fact | drawn where | drawn when |
|---|---|---|---|
| 1 | **a permission is a request, not a lock** — [`crate::text::security::permissions_are_advisory`], the ENGINE's own sentence, in the danger role | at the head of the permission list, **above** the tick-boxes | on every job that writes permission bits |
| 2 | **a signed document is refused** — [`crate::text::protect::signed_refusal`] | **instead of** the entire form | whenever the document carries a signature |
| 3 | **re-permissioning needs the owner password** — [`crate::text::protect::owner_password_note`] | **above** the current-owner field | on every job that touches an already-protected file |

★ Disclosure 1 is not re-worded here and must not be. The engine supplies it
as `EncryptionSettings::PERMISSIONS_DISCLOSURE`, the CLI prints it, and
`crate::text::security` catalogued it. Two surfaces wording one limitation
differently is worse than either wording.

★★ Disclosure 2 is **R9** in its strongest form. The ribbon controls stay
present — whether *this* document is signed is not known when the registry is
built — so the window opens, states the refusal, names the count, explains
the mechanism (*it rewrites every byte the signature covers*) and offers
nothing. There is no greyed form behind it and no button that fails on press.

# 3. Saving: `dialogs::redact`'s answer, followed rather than re-invented

★★★ **This is deliberate and it is stated rather than left to be noticed.**
Protecting a document rewrites every byte, exactly as applying a redaction
does, so it raises exactly the question the operator settled hours earlier on
that surface, in his own words:

> *"why does it have to save to a new file right away? Why can't it just wait
> on saving until I choose to save over the existing file or save as a new
> file?"*

A second answer to a settled question would be the defect. So the mechanism
here is `crate::dialogs::redact`'s, part for part:

| | redaction | protection |
|---|---|---|
| default destination | a new file, chosen in the picker | **the same** |
| suggestion | never the source (`-redacted`) | **the same** (`-protected` / `-unprotected`) |
| replacing the original | offered, **one extra acknowledgement** naming the file, **no picker** | **the same** |
| the write | temp file, then rename — atomic | **the same** ([`crate::protect::Prepared::write_to`]) |
| confirm control | the label IS the consequence; an ellipsis promises a picker, naming the file promises none | **the same** |
| the open document afterwards | untouched, and the outcome sentence says so | **the same**, and it matters more — see §5 |

★ Replacing takes **no picker**, and that is the deliberate half. A picker
pre-filled with the source is a dialog whose safe answer is to change the
field, which is the shape of every accidental overwrite there has ever been.
The consent is taken before the click, in words, at a control the operator
had to select.

★★ The half of his request the engine cannot express is the same half as
there — *defer the write to a later Save*. All three encryption verbs
**return bytes**; none stages anything in a session, and `EditSession` has no
`replace_document`. Approximating it would mean swapping a second session
under the open document and silently discarding its undo log, which
`crate::app::save::save_as` refuses for the same reason.

# 4. Why this dialog does not push an `Action`

[`super`]'s rule: a dialog uses the action funnel when it edits **this**
document, and this one never does. Every job here produces *bytes on disk*.
The open session is not mutated — see [`crate::protect`] §2 for the argument,
which is load-bearing rather than cautious: two of the three engine verbs
take `&mut EditSession` and what they clear is the guard that stops the next
ordinary `Ctrl+S` writing plaintext objects into a file of AES ciphertext.

# 5. ★★ After a replace the window is deliberately STALE, and it says so

`crate::dialogs::redact`'s outcome, and the divergence matters more here
because it is **invisible**: a redacted page looks different, and a protected
file looks identical to the one it came from. So
[`crate::text::protect::written`]'s replace form names the file to re-open.
Rule 4: report separately, and do not pretend.

# ★ 6. The section headings are NOT `.strong()`, and that is deliberate

Every heading here was written `RichText::new(…).strong()` in the first
draft, and `tools/gates/check-strong-text.sh` caught all six. Its rule, and
`DEFECTS.md` D11 behind it: egui has no separate role for emphasised text,
so `.strong()` resolves to the **accent-filled widget** colour — which on an
ordinary panel is pale text on a pale background. Six labels have already
shipped that way once, and the Settings window repeated it three days after
the rule was written.

So the hierarchy here is carried by **layout and wording** instead: each
section is separated by a rule and a gap, the two headings are full phrases
(*"This document, as it is now"*, *"What to change"*) rather than one-word
captions, and the muted `.small().weak()` notes below them are what the
headings contrast against. A reader who wants to re-emphasise one of these
should read D11 first — in every observed case the label read **better**
without it.

# 7. Document-scoped, like every dialog that holds unsaved bytes

Closing the document discards them. A protected copy of a file nobody is
looking at any more, derived from a permission census that can no longer be
checked, is not a saving.
