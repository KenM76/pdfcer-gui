# `ui-verify/checks/stamp_collection`

**Acrobat custom stamps, driven end to end** — both halves of
`OPERATOR_REQUESTS.md` **O169** against the running binary.

Two checks live here, and they are deliberately neighbours:

| Check | Asks |
|---|---|
| `a_stamp_collection_discloses_itself` | somebody else's collection, opened — does pdfcer **say** what the file is? |
| `stamp_collection_reaches_the_engine` | an ordinary document, saved — do **bytes Acrobat will accept** reach the disk? |

They share their fixtures, their region names and their plumbing, and the
second one uses the first one's surface as its oracle. Splitting them
across two files would put that dependency out of sight.

# What this file is for

The operator asked for Acrobat's custom stamps *"with the same
import/export"*. The finding that shaped the feature is that there is **no
interchange format to match**: a stamp collection simply IS an ordinary PDF
— one file per category, one page per stamp, the category in `/Info`
`/Title`, the names in the catalog's `/Names` → `/Pages` name tree. Handing
somebody the file is the export; `file.open` is the import.

Which is exactly why the read half needed building. Without it, an operator
opening a collection somebody sent him sees a document of unrelated
pictures, and every fact that makes it a collection — the category, the
names, which entries are dynamic — is invisible in the page view. pdfcer now
discloses all of it in Document properties, off-canvas, under **R8b rule 4**.

`panels::docprops::stamps` unit-tests every sentence in that section. What no
unit test can see is whether the section is **connected to a running
window**: whether it draws at all, whether it draws its rows where an
operator can read them, and — the half that matters most — whether it stays
away from the files that are not collections. That is R1, and that is this
file.

# ★★★ The second launch is the check

This launches **twice**: once on `fixtures/stamp-collection.pdf`, asserting
the section is THERE with a row per stamp, and once on
`fixtures/four-pages.pdf` with the panel provably open, asserting it is NOT.

Without the control launch, "the region is declared" is satisfied by a build
that declares it on every document — a *Stamp collection* heading pinned
permanently into Document properties, telling the operator that every CAD
drawing he opens is an Acrobat stamp set. That is a worse defect than the
one this check is for, it is the shape rule 4 is most alert to, and it wears
exactly the same green tick.

⚠ Neither fixture's suitability is assumed. The shell's own
`stamps::tests::the_driven_checks_fixtures_are_what_the_check_believes_they_are`
asserts through the engine that `stamp-collection.pdf` really carries three
stamps with one dynamic under the category *Site Review*, **and** that
`four-pages.pdf` really is not a collection. Both halves live in the crate
that runs on every `cargo test`, because a driven check runs only when
somebody has the machine's pointer to spare — and an absence assertion
against an unverified control is an assertion about nothing.

# ★★ Why the row count is three and not "at least one"

Because the failure this check is most likely to catch is a section that
draws its heading and its count and then nothing — the shape a regression
takes when the loop over `collection.stamps` stops publishing, or when the
engine's name-tree walk returns empty on a file it used to read. A heading
reading *"Stamps: 3 stamps, 1 of them dynamic"* over an empty space is worse
than no section, because the count is then the only number on screen and it
is contradicted by what is under it. Only counting the rows sees that.

# What this does NOT prove

That the *wording* is right, or that the rows are in tree order. The trace
publishes a region name and a rectangle, not a string, so a build that
printed the internal name where the display name belongs, or listed the
stamps by page instead of by name-tree position, would pass here. Both are
asserted in the shell's own tests against the same fixture through the same
engine call — deliberately, because a string and an ordering are exactly
what a unit test CAN see. What it cannot see is the window.
