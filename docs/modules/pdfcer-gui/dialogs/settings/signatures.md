# `dialogs::settings::signatures` — may pdfcer read Acrobat's trust list,
and where is it

Two settings, and the group exists because of what they have to explain
rather than because of how many they are.

`ENGINE_BACKLOG.md`'s third trust row asked for exactly this and named the
two properties it had to have:

> it is a preference with a privacy shape — it reads a file belonging to
> another vendor's product. Belongs in Settings beside the other opt-ins,
> off by default, with the store's path and its **mtime** disclosed, because
> an anchor set that silently went stale is worse than one that was never
> imported.

Both are here: [`use_store`] draws the engine's own `Off`/`AtOwnRisk`
setting, and [`store_path`] draws the location, the resolved state and the
**date**.

## ★★★ Why this is not filed under the *Where Acrobat is* group

It is the obvious place — both settings are about Acrobat — and it is wrong,
for the reason this window's own header gives: **a setting filed under the
wrong heading is not untidy, it is unreachable**, because an operator opens
this window with a *symptom* and the headings are how a symptom finds its
setting.

The symptom that brings somebody here is *"the Signatures panel says it did
not check who signed this"*. Nobody carrying that symptom looks under a
group whose own header says it *"changes nothing at all except which program
a single button starts"* — and putting this there would make that sentence
false as well as making the setting hard to find.

## Placement: last of the document groups, before the program ones

The window's ordering rule runs from what the **program** looks like,
through what the **document** is made of, to what pdfcer **does with it**.
Reading a document's signatures is squarely the third, so this sits after
*Pages and printing* and before *Drawing the page*, which is where the
shell's own preferences begin.

## ★★ Two headers, not one, and the reason is the two stores

[`super::widgets::toggle`]'s own note says the sub-parts of ONE setting
share a header. These are two settings: a **permission**, persisted in
`pdfcer_core::settings` where the CLI reads the same answer, and a
**location**, persisted in `crate::app::prefs` because the engine has no
field for it and deliberately does not — its module header says *"locating
the file is the shell's job"*.

An operator has no business meeting that split, and does not: the two are
adjacent, one Cancel discards both, one Save writes both. But they are two
questions with two different blast radii, and a single `radius` line
covering both would have to be vague about the one that matters.

## ★★★ R9, and which control is absent

[`inspect`] — the button that reads the store and reports what is in it — is
drawn **only when a store was actually found**. An unavailable capability
renders nothing; greying is reserved for something *temporarily*
unavailable, and a person with no Acrobat store is not one press away from
having one.

The path field above it is drawn **always**, and that is R9's other half:
the remedy for an absent capability must be reachable, and an absent
capability whose remedy is also absent is a dead end. Somebody whose store
is on a redirected profile sees no inspect button, and the field that fixes
it is right there with a resolved-state line telling them what pdfcer
currently finds.

## The resolved line IS live, unlike the Acrobat group's

[`super::acrobat`]'s state line cannot update as the operator types, because
resolving an Acrobat spawns processes and a settings pane redraws every
frame. Locating a trust store does not: it is `Path::is_file` plus one
`metadata`, which is a stat rather than a read. So this line updates
keystroke by keystroke and a typo is visible at the place it was made,
rather than after a Save.

★ The *anchors* are a different matter — parsing 3 MB of COS and decoding
~1,800 certificates is not a per-frame act — which is exactly why reading
them is behind a button and the button caches its answer.
