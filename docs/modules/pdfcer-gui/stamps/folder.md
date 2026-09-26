# `stamps::folder` — where Acrobat looks for the operator's own stamps

One question, and it is the difference between a feature and a chore: after
pdfcer writes a stamp collection, **does the operator have to go and put it
somewhere?** If yes, the feature is "pdfcer can write a PDF", which he
already had. If pdfcer suggests the folder Acrobat scans, his stamps appear
in Acrobat's own menu the next time he starts it, which is what he asked
for in `OPERATOR_REQUESTS.md` **O169**.

## What is measured, on this machine, today

```text
%APPDATA%\Adobe\Acrobat\DC\Stamps\YTV_yyfVN1TzJ0_6oei-GB.pdf
```

That is **his** file — category `Signatures`, stamps `Ken` and `Savy` — put
there by Acrobat when he made them, and it is what pins every claim below.
Acrobat's *shipped* collections live elsewhere entirely, under the install
at `…/Acrobat DC/Acrobat/plug_ins/Annotations/Stamps/ENU/`, and are not
writable and not our business.

⚠ Note the filename: **Acrobat generates a key, not a name.** The category
that shows in its menu comes from `/Info` `/Title` and nothing reads the
filename. So pdfcer is free to write `Signatures.pdf`, which is better for
a human browsing the folder, and Acrobat is indifferent.

## Why the version segment is discovered rather than hard-coded

`%APPDATA%\Adobe\Acrobat\` holds one folder per Acrobat generation, and on
this machine it also holds three that are **not** generations —
`Preflight Acrobat Continuous`, `Privileged`, `TypeQuest`. A hard-coded
`DC` works until Adobe renames the generation, and a naive "first
subdirectory" picks `Preflight Acrobat Continuous` because it sorts first.
So the rule is: **prefer a folder that already has a `Stamps` directory**,
and fall back to the shortest plausible generation name.

"Already has `Stamps`" is a strong signal precisely because Acrobat
creates that folder the first time a user makes a custom stamp — its
presence means *this is the generation the operator actually uses*, which
is a better answer than any version ordering could give.

## What this module deliberately does NOT do

**It does not create the folder, and it does not write anything.** It
returns a suggestion for a file picker. The operator confirms a path in a
native dialog like every other write in this shell, and if he saves the
collection to his desktop instead that is a complete and correct outcome.
A feature that silently drops files into another application's preferences
directory is a feature nobody can uninstall.

## Platform

Windows-shaped by construction, via `%APPDATA%`. On a platform without it
every function here returns `None` and the caller falls back to the
document's own directory — the same graceful nothing
`crate::acrobat::resolve` produces when Acrobat is not installed, and for
the same reason: **R9**, an unavailable capability renders nothing rather
than a broken suggestion.

## Item notes

### `const NOT_A_GENERATION`

A deny-list rather than an allow-list of known generations, because a new
generation name is a thing that will happen and a new sibling utility
folder is a thing that already has. Getting the deny-list wrong costs a
wrong suggestion in a picker the operator is looking at; getting an
allow-list wrong costs the feature entirely on a future Acrobat.

### `fn pick_generation`

The rule, in order:

1. **A folder that already holds `Stamps` wins outright.** Acrobat made it,
   which means the operator has made a stamp with that generation.
2. Otherwise, the shortest name that is not on [`NOT_A_GENERATION`].
   Generation keys are short (`DC`, `11.0`, `2020`); the sibling utility
   folders are long descriptive phrases. Shortest-wins is a heuristic and
   is admitted as one — it only ever decides a *suggestion* in a picker the
   operator can overrule.

⚠ Ties are broken by name order so the answer is deterministic. A
suggestion that differs between two runs on an unchanged machine is a
defect even when both answers are defensible.

### `fn is_path_hostile`

The set is §"Naming Files, Paths, and Namespaces"'s reserved list. A
category name is free text an operator typed and `Approved / Rejected` is
an entirely reasonable thing to type.
