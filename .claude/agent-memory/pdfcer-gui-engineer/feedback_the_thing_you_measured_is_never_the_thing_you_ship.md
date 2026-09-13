---
name: the-thing-you-measured-is-never-the-thing-you-ship
description: A driven sweep measures the binary on disk before the release commit exists; committing rewrites the embedded stamp and relinks the executable, so "this exact binary" is false by eight million bytes — re-drive the stamp-reading checks against the shipped file instead of arguing the difference is cosmetic
metadata:
  type: feedback
---

**A measurement of an executable is a measurement of the executable that existed
when the measurement started. The commit that names the release comes after, and
it changes the file.**

**Why:** 2026-09-13, cutting `v0.5.0-dev.20260913.2`. `FEATURES.md` said *"the
entire driven suite was run against this exact binary"* — and so had the two
release notes before it. `sweep-full.sh` copies `target/release/pdfcer-gui.exe`
to a frozen scratch path before its first check, deliberately, so the whole
ninety-five-minute run measures one unchanging program. That copy is taken
*before the release commit exists*. `build.rs` compiles `PDFCER_BUILD_TIME`, the
short git rev and a `-dirty` flag into the program and declares `.git/HEAD` an
input, so `git commit` alone forces a relink.

Measured rather than assumed, which is the only reason this is worth keeping:

    same size          29,746,176 bytes
    differing bytes     7,998,303
    differing runs         25,508

**Not one of those is a change to what the program does.** The stamp string got
shorter when `-dirty` fell off the end of it, everything after it slid, and
every relative call target in the binary was rewritten. The *behaviour* claim
was sound the whole time. The claim that was written down was about **bytes**,
and that one was false, three releases running, in the document the operator
reads to know what he has.

**How to apply:**

1. **Write the claim you can defend.** "The suite ran against these sources, from
   this commit's tree" is true, is what anyone actually wants to know, and costs
   nothing to say.
2. **Then close the gap with two minutes of work rather than a paragraph of
   argument.** After the release commit, rebuild and re-drive the checks that
   read the version stamp against the executable that ships, plus the off-screen
   smoke launch. An argument that a difference is cosmetic is exactly the shape
   of thing this project keeps finding to be wrong; a re-drive is a measurement.
3. **Do not try to make the two files identical.** `PDFCER_BUILD_STAMP` can pin
   the time, but nothing can pin the git rev of a commit to a value known before
   that commit exists. A build that reports a revision it was not built from is
   a far worse defect than a relink.
4. **The general shape:** any artifact that embeds its own identity cannot be
   measured under that identity before it has one. It applies to anything
   stamped with a version, a tag, a build number or a content hash — and the
   tell is a sentence in a report using the word *exact* about something the
   report itself caused to change.

Related: [[feedback_a_measured_limit_belongs_to_a_revision_not_a_design]],
[[feedback_a_still_broken_report_is_first_a_question_about_which_build_and_which_pin]],
[[feedback_a_capability_claim_in_product_copy_needs_the_same_citation_as_a_limitation_claim]],
[[feedback_the_tell_for_running_the_wrong_binary_is_an_absent_line]].
