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

# The second launch is the check

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

# Why the row count is three and not "at least one"

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

## Item notes

### `const COLLECTION`

Built by `fixtures/stamp-collection.PROVENANCE.py`, which computes its own
xref offsets. Its name tree is deliberately **not** in page order — `#`
sorts before `S`, so the dynamic stamp is tree entry 0 and page 3 — because
the rows are indexed by tree position and a fixture sorted the lazy way
would let a build that enumerated pages pass.

### `const PLAIN`

⚠ Asserted through the engine by the shell's
`the_driven_checks_fixtures_are_what_the_check_believes_they_are`. Do not
swap it for another fixture without adding the new name there.

### `const ROW_PREFIX`

⚠ It is `SECTION_REGION` plus a dot, and the shell asserts that relationship
in its own test: several checks dump every region under a prefix when they
cannot find the one they wanted, and a row prefix that drifted out from
under the section's would vanish from that dump.

### `const SOURCE_PAGES`

⚠ Every row is included by default (`Plan::new` sets `include: true`), so a
build that silently dropped a page would still write a valid collection —
one sheet short, with no error anywhere. That is why this number is
asserted three times over: on the window's `pages=`, on the request's
`stamps=`, and on the row count when the written file is read back.

### `const WROTE`

⚠ Exactly `stamp-collection`, and the trace parser matches event names by
**equality** — so this does not also match `stamp-collection-open`,
`-requested`, `-failed`, `-declined`, `-cancelled` or `-unavailable`. That
is load bearing rather than incidental: a prefix match here would let the
window merely *opening* satisfy the assertion that a file was written.

### `fn launch_on`

Takes a `&Path` rather than a fixture name because the author half's third
launch opens a file that did not exist when the check started — the
collection pdfcer itself just wrote, under `--out`. A helper that could only
open things in `fixtures/` would have made the round trip unwritable, and the
round trip is the only assertion in this file that reads the produced bytes
with the reader Acrobat's own structure demands.

### `fn open_properties`

Reuses `properties_metadata`'s opener rather than spelling the two clicks
again. It is the same ribbon item and the same toggle hazard — pressing
`file.document_properties` while the panel is up CLOSES it — and two copies
of that guard would be two places for the next ribbon move to be applied.

### `struct StampCollectionReachesTheEngine`

# The method: the reader is the oracle for the writer

This check writes a collection and then **reopens it in pdfcer**, asserting
that Document properties discloses it as a collection with one row per page.
That looks circular at first reading and is not, for one specific reason:

> The reader was calibrated against a file pdfcer did not write.

`a_stamp_collection_discloses_itself`, immediately above, proves the same
reader on `fixtures/stamp-collection.pdf` — a file assembled byte by byte by
`fixtures/stamp-collection.PROVENANCE.py` from §7.9.6 and §12.5, with its own
xref computed by hand and not one line of pdfcer involved. A reader that
agrees with an independently-built artifact and then agrees with pdfcer's
output is evidence about the output; a reader that had only ever been shown
pdfcer's own files would be evidence about nothing.

⚠ Which is why the two checks must not be separated, and why neither may be
deleted while the other stands. If `a_stamp_collection_discloses_itself` is
ever removed, this check's final assertion silently loses its meaning while
continuing to pass.

# Why the round trip and not a byte scan

The obvious cheaper oracle — grep the written file for the stamp names — is
wrong here and would be *quietly* wrong. `pdfcer-core`'s writer may place the
catalog and the name tree in an object stream, in which case the names are
inside a Flate-compressed blob and a scan finds nothing on a perfectly good
file. And even uncompressed, finding the bytes `SRApproved` somewhere in a
PDF says nothing about whether they are reachable from the catalog's
`/Names` → `/Pages` tree, which is the only thing Acrobat looks at. The
round trip asks the question Acrobat asks.

# What the same fixture proves twice

The source document is `four-pages.pdf`, which is also the *control* in the
read half — the file whose whole job there is to be provably **not** a
collection, with the panel open and the section absent. So one fixture
carries both ends of the claim: before pdfcer touches it, Document
properties says nothing about stamps; after, the file pdfcer wrote from it
discloses four of them. Neither end is asserted about a document whose
nature was assumed — the shell's
`stamps::tests::the_driven_checks_fixtures_are_what_the_check_believes_they_are`
pins it through the engine on every `cargo test`.

# What this does NOT prove

**That Acrobat shows the stamps.** Nothing in this repository can prove
that; Acrobat scans its stamps folder once at startup and reports nothing
either way. What is provable — that the file carries a `/Names` → `/Pages`
tree with the planned names resolving to the planned pages — is what this
asserts, and the remaining gap is closed by the operator opening Acrobat
once. The receipt the shell prints says *restart Acrobat* for that reason.

It also does not prove the **naming** is right: which display name landed on
which page, and how a duplicate was made unique, are `crate::stamps`' unit
tests, against the same plan type, deterministically, without a window.
