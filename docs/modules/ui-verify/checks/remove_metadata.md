# `remove_metadata_takes_only_the_entries_ticked`

**Defect it guards.** Security ▸ Protect ▸ Remove metadata (`OPERATOR_REQUESTS.md` O289
item 21) fails to list the document's description entries, or removes entries
other than the ones ticked — none, all, or the wrong ones.

**Fixture.** `fixtures/four-pages.pdf`, which carries all four of Title,
Author, Subject and Keywords, copied and driven off the desktop with the
scripted pointer.

**Steps.**

1. Security tab ▸ Protect ▸ Remove metadata: require `remove-metadata-listed
   fields=Title,Author,Subject,Keywords`.
2. Tick Author and Keywords; press Remove chosen.
3. Open the window again: the new `remove-metadata-listed` must name exactly
   `Title,Subject`. Removing nothing, removing everything and removing the
   unticked pair each produce a different list.

The reopened list is read from the edit session, so it is the engine's
answer, not the window's memory.

**Falsified** in two ways:

- Pushing the removal for every entry rather than the ticked ones: the window
  reopens listing nothing.
- Dropping the `SetInfoField` push: the window reopens listing all four.

**What it does not prove.** That the saved file lacks the keys; the save path
is the same for every edit and is not this window's.
