# `canvas::cutgate` — **would a cut survive the round trip?**

One question, asked **before the press**, because a cut of something the
clipboard cannot carry is a deletion wearing a clipboard's clothes.

## Where the question comes from


> **Do not offer Cut as enabled and let it fail.** A **copy** of something
> pdfcer cannot carry costs nothing — the original stays, the clip carries an
> `Unsupported` marker, the paste declines by name. A **cut** of the same
> thing is a deletion wearing a clipboard's clothes, so it is refused
> *before anything is removed*: `EditError::CutWouldNotSurvive { subtype }`.
> Copy the selection first, look for an `Unsupported` entry, grey the
> control with the subtype named.

## Why this MIRRORS the engine's rule instead of calling it

Their advice — *copy the selection first, then look* — is right about the
**oracle** and wrong about the **budget**, and the difference only shows on
this operator's documents.

`copy_selection` decomposes the page: it resolves every `/Contents` stream,
inflates, concatenates, tokenizes and walks the whole token stream resolving
fonts as it goes, **with no cache anywhere in `pdfcer-core`**. A ribbon
condition is rebuilt **every frame**. On the benchmark drawing — 129,758
objects — that is a full decomposition per frame to decide whether one
button is grey, and the answer changes only when the selection does.

⇒ So this asks the same question from the **cheap side**: what the engine
refuses is decided by the annotation's `/Subtype` and by one sidecar lookup,
and the selection already carries the object id. One dictionary read, the
same shape `panels::properties::annotdelete::gate` already uses for the
delete gate, on the same cadence.

## The engine remains the authority, and that is not a formality

This gate greys a control. It does **not** decide whether the cut happens —
`EditSession::cut_selection` copies first and refuses on its own
`Unsupported` scan, so a case this mirror does not know about is still
caught, still refuses, and still deletes nothing.

That matters because the two can drift: the engine's carryable set **grew**
on the day this was written (sticky notes, text boxes, stamps and links all
became carryable via the new `Raw` carrier), and a mirror that had been
written a day earlier would have been greying Cut over annotations that had
since become perfectly cuttable. A mirror that is *too permissive* costs a
refusal sentence; one that is *too strict* costs a capability, silently.

⇒ **So this mirror is deliberately permissive.** It names only what the
engine refuses *by policy* and is documented as refusing — the four cases in
§3 of their note — and says nothing about anything else. Every doubt
resolves to *"let them press it"*, and the engine answers.

## The four, and why two of them cannot arrive here at all

| subtype | why the engine refuses | reachable from the canvas? |
|---|---|---|
| `/Widget` | has its own clipboard — `canvas::fieldclip` | **no**: excluded from `AnnotSelection` |
| `/Popup` | §12.5.6.14 — belongs to the comment that opens it, and travels with it | **no**: excluded from `AnnotSelection` |
| `/Redact` | a pending destructive operation; pasting one arms a redaction nobody reviewed | **yes** |
| a ce dimension with a missing sidecar record | R204 — the record is what makes it a ce dimension rather than lines | **yes** |

The two unreachable rows are checked anyway. `canvas::selection::annot`'s
exclusion table is a *current* fact about one surface, and this gate is
consulted by the ribbon, the context menu and the keyboard — three doors,
and only one of them is that surface. A gate that assumed the exclusion
would be correct today and wrong the first time a widget became selectable
anywhere.

## Item notes

### `fn the_three_policy_refusals_are_spelled_as_the_engine_spells_them`

Asserted as strings rather than by building a document, because the
claim under test is that this shell's spelling matches the engine's
`CutWouldNotSurvive { subtype }` — a wording agreement across a crate
boundary, which no fixture can check and a typo would silently break.
