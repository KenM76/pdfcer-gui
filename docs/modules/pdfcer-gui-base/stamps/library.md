# `stamps::library` — the operator's OWN stamps, found and offered

`OPERATOR_REQUESTS.md` **O172**: *"We also need to make it easy to add our
own custom stamps and use them, preferrably exactly the same way acrobat
does."* — and the load-bearing half of that sentence is **use them**.

[`super`] already authors a collection and [`super::folder`] already knows
where Acrobat keeps one. What neither could do was put a stamp the operator
made onto a drawing, because the engine had no verb that carried one page's
artwork onto another. `pdfcer-core` `Pass 293.0` shipped
`EditSession::place_page_artwork`, and this module is the half that has to
exist above it: **which stamps are there, what are they called, and which
page of which file is each one.**

## What a "library" is here, concretely

A flat scan of one directory, turned into a list a gallery can draw:

```text
%APPDATA%\Adobe\Acrobat\DC\Stamps\*.pdf
    |- each file  = one CATEGORY   (its /Info /Title)
        |- each named page = one STAMP  (/Names -> /Pages, `internal=display`)
```

Nothing is cached across a scan and nothing is watched: [`scan`] reads the
folder each time it is called, and its callers call it when a stamp gallery
opens. That is deliberate — the alternative is a cache that is stale
exactly when it matters, on the run after the operator has just made a new
stamp in Acrobat and gone looking for it here.

## Why the SHIPPED collections are excluded, and it is argued

Acrobat's install carries four more collections at
`…/Acrobat DC/Acrobat/plug_ins/Annotations/Stamps/ENU/` — `Standard`,
`StandardBusiness`, `Dynamic`, `SignHere` — and this module does not scan
them. Three reasons, in decreasing order of how much they matter:

1. **`Standard` would DOUBLE the gallery and demote the better entry.**
   pdfcer already offers Table 181's stamp faces as real `/Name`
   annotations ([`crate::canvas::textannot::STAMPS`]) — which is what
   Acrobat itself writes for those, and what any other reader understands.
   Importing Adobe's artwork instead would author a *nameless* form XObject
   where a named annotation belongs, and the gallery would show `Approved`
   twice with no way to tell which was which.
2. **`Dynamic` is a promise pdfcer cannot keep.** Its text comes from
   AcroForm calculation JavaScript that Acrobat runs at placement time.
   `place_page_artwork` imports the artwork and *not* the machinery, so a
   dynamic stamp arrives frozen at its design-time text — a stamp reading
   `Received 10 Sep 2026` forever. See [`CustomStamp::dynamic`], which is
   how a dynamic stamp in the operator's OWN folder is disclosed rather
   than hidden.
3. **They are not "our own custom stamps"**, which is what the request
   says.

⚠ The one that is genuinely arguable is `SignHere`: five faces pdfcer does
not offer, in a category a drafting workflow uses. It is excluded here only
because it arrives bundled with the other three, and the exclusion is one
constant away from being reversed. This paragraph exists so that reversal
is a decision rather than a discovery.

## R8b rule 4 — what this module discloses, and where

Everything here is **off-canvas by construction**: it produces names for a
gallery and counts for a status line, and touches no painter. Two
inferences are made and both are reported rather than silently applied:

| inference | reported as |
|---|---|
| a collection with no `/Info` `/Title` is labelled from its FILENAME | [`Category::named_from_file`] |
| a stamp file that will not open, or names no page, is left out | [`Library::unreadable`] / [`Library::unplaceable`] |

The second is the one that would otherwise be invisible: a folder of six
stamps that shows five is indistinguishable from a folder of five.

## Item notes

### `fn read_collection`

`None` when the file will not open at all, or opens and has no stamp name
tree — the two cases a caller counts identically because to the operator
they are the same thing: a file in the folder that is not a stamp file.

### `fn label_for`

The display half when there is one, the internal name when there is not —
with the `#` that marks a dynamic stamp stripped, because that character is
a *marker in the format*, not part of what the stamp is called. Acrobat
shows `Received`, not `#Received`.

⚠ Stripping it here is presentation only. [`CustomStamp::dynamic`] carries
the fact, so nothing downstream has to re-derive it from a string.

### `struct CustomStamp`

# Why the whole thing travels rather than an index

This is carried on `Action::CommitTextAnnot` and therefore has to survive
the dialog that produced it. An index into a [`Library`] would be a handle
into a list that is rescanned on the next gallery, which is the exact shape
of bug where the operator places *Ken* and gets *Savy* because a file
landed in the folder in between. Three `String`s and a `PathBuf` per
placement is not a cost worth that risk.

### `fn scan`

Never fails: a missing folder, an unreadable file and a PDF that is not a
stamp collection are all ordinary outcomes, and each is *counted* rather
than raised. The caller draws whatever came back.

# Cost, because this runs when a dialog opens

One `read_dir` plus one [`Document::load`] per `.pdf` in the folder. A
stamp collection is a handful of kilobytes; the operator's own is one file
with two stamps. If that folder ever holds something large this is the
place that will show it, and the answer then is a cache with an mtime, not
a background thread.
