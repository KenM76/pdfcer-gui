# `ui-verify/checks/custom_stamp`

`custom_stamp_reaches_the_page` — **one of the operator's OWN stamps is
picked out of the gallery with the pointer, dragged onto a drawing, and the
artwork arrives.**

# The request this closes


> *"add our own custom stamps and use them, preferrably exactly the same way
> acrobat does"*

*Use them* is the half this file measures. Reading his stamps folder is
covered by unit tests over `stamps::library`; **placing** one is a chain
nine hops long, and the standing lesson of this project is that unit tests
cannot see the chain in front of the verb:

| hop | its unit test | what that test cannot see |
|---|---|---|
| the folder is scanned when the dialog opens | `only_the_stamp_kind_scans_the_stamps_folder` | it is vacuous on a machine with no stamps |
| the gallery draws his half | `the_custom_half_appears_only_when_there_is_something_in_it` | whether it is reachable with a pointer |
| a click selects one | `exactly_one_of_the_two_galleries_holds_the_selection` | it calls `select_custom` directly |
| the selection reaches the action | `the_operators_own_stamp_reaches_the_commit_action` | it sets `accept_requested` by hand |
| `apply` forks on `custom: Some(..)` | the `apply` arm tests | they build the action by hand |
| the collection is reopened off disk | — | nothing in-process opens a real file |
| the engine imports the artwork | the engine's own tests | they never went through a dialog |
| the mark is turned upright on a rotated page | — | |
| four facts become sentences | `customstamp::tests` | they hand-build the outcome, because `PlacedArtwork` is `#[non_exhaustive]` |

Every one of those passes on a build whose gallery is drawn below the
window's bottom edge, or whose radio never fires, or whose fork reads
`stamp` first and puts `Approved` on the sheet instead of his signature.

# ★★★ The collection is PLANTED, and that is the whole reason this check
can fail

The obvious version of this check reads the operator's own Acrobat stamps
folder and skips when it is empty. That version is worthless on any machine
but his — and, worse, it goes **vacuous on his own machine the day he
deletes a stamp**, silently, behind a SKIP nobody reads. A SKIP is not red.

So this check builds a stamps folder of its own under `CheckContext::out_dir`
and points the application at it by overriding **`%APPDATA%`** for the
launched process. `stamps::folder::user_stamps_dir` resolves
`%APPDATA%\Adobe\Acrobat\<generation>\Stamps` and reads the variable from
the environment on every call, so a child process with a redirected
`APPDATA` sees exactly the collection this file put there and nothing else.

⚠ **What else moves when `APPDATA` moves**, measured rather than assumed:

* `stamps::folder::user_stamps_dir` — the point of the exercise.
* `trust::…` — Acrobat's address book, which becomes empty. Nothing this
  check drives reads it.
* `pdfcer_core::settings::resolve_store`'s **fallback** only. Its first
  choice is `<directory of the running executable>/userdata`, which exists
  and is writable for every way this suite launches a binary, so the
  preferences, the layout and the recent-file list do **not** move. That is
  a property of the engine's resolver, and it is named here because if it
  ever changes this check silently starts running against a blank profile
  and its failures stop meaning what they say.

⇒ The redirect is also why this check leaves **nothing** in the operator's
own Acrobat folder. It never writes there and never reads there.

# The fixture, and the two things it was already built to prove

[`COLLECTION`] is this repository's `fixtures/stamp-collection.pdf`, whose
`.PROVENANCE.py` states its properties. Two are load-bearing here and
neither was chosen for this check — they were already true:

1. **The name-tree order is not the page order.** §7.9.6 sorts a name tree
   lexicographically, and `#SRIssued` (`#` is 0x23) sorts before
   `SRApproved` (`S` is 0x53) while its page is last. So the **first** entry
   in the gallery is *Issued*, on source page **2**. A build that enumerated
   pages instead of tree entries would put *Approved* there, look entirely
   plausible, and mislabel every stamp the operator owns. Phase E asserts
   the ordinal-0 stamp by name *and* by source page for exactly that reason.
2. **Exactly one of the three is dynamic**, and it is that same first entry.
   So pressing ordinal 0 also drives the dynamic disclosure — the sentence
   saying the date baked into the artwork will not recompute — without
   needing a second placement.

And one property this check adds: the collection's pages are **200 × 60 pt**
and the box dragged here is [`BOX_PT`] **square**. `place_page_artwork` maps
the form `/BBox` onto `/Rect` with independent horizontal and vertical
factors (§12.5.5), so the scales come out ≈1.1 and ≈3.7 and the engine's
`distorted` flag — `(scale_x - scale_y).abs() > 1e-6` — is true **by
construction rather than by luck**. The stretch disclosure is therefore
guaranteed to be owed, which is what lets phase G assert that a sentence is
on the status bar rather than merely hope one is.

# R8b rule 4 — what is asserted, and what is deliberately NOT

The placed stamp renders **exactly as a saved-and-reopened one would**: no
badge, no tint, no dashed outline, nothing on the canvas saying *"this was
stretched"*. This check therefore asserts the disclosure **off-canvas**, by
the presence of the `status-group:edit-disclosure` region, and asserts
nothing at all about the pixels of the mark. An edit that added a
provisional style to the canvas would leave every assertion here green, and
that is correct: it would be a defect for a different check to catch.

# Rule 15


# Phases

| Phase | Does | Expected |
|---|---|---|
| A | plant `fixtures/stamp-collection.pdf` into a scratch `%APPDATA%` | the file is on disk before launch |
| B | launch with `APPDATA` redirected, Review mode, Markup tab, arm **Stamp** | `markup-tool tool=TextAnnot(..)` |
| C | drag a square box | `text-annot-open`, and `custom-stamp-library categories=1 stamps=3` |
| D | read the gallery's custom half | `text-annot.custom-stamps` and `…custom-stamps.0` declared |
| E | click ordinal 0 | `custom-stamp-chosen name=Issued page=2 dynamic=true` |
| F | press **Add** | `custom-stamp-requested`, then `custom-stamp-placed distorted=true` |
| G | read the answer | the funnel line carries TWO joined sentences, and the status bar declares its disclosure region |
| H | arm **Stamp** again, drag a second box elsewhere | `stamp-gallery-opens restored=custom remembered="custom:Site Review/Issued"` |
| I | press **Add** without touching the gallery | a SECOND `custom-stamp-requested name=Issued` -- the memory reached the page |

★ **Phases H and I have been seen to fail.** On their first driven run
`REMEMBERED_TOKEN` carried the quotes the shell writes, `trace::parse_fields`
had already stripped them, and the check refused -- printing the line it
refused on, which is why the mistake cost thirty seconds. A phase that has
only ever been seen green is indistinguishable from one that cannot go red;
these two have a red to their name and it was the check's fault, not the
application's.
