---
name: parallelism-is-the-default-and-it-needs-an-ownership-map
description: Ken wants as many tracks running at once as will not collide; what makes nine agents in one repo work is a written file-ownership map plus a no-racing rule for shared counters.
metadata:
  type: feedback
---

**Run as many tracks in parallel as will not collide, and do not ration them.**

Ken, 2026-09-04, unprompted and mid-turn: ***"run as many tasks in parallel
that you can without colliding. you have full use of the PC."*** Earlier the
same day: *"run multiple agents if you can"*, then *"run agents in parallel
where you can."* Three statements of the same instruction in one session.

**Why:** the machine is configured for it (`CLAUDE_CODE_MAX_SUBAGENTS_PER_SESSION`
is 2000) and he is usually away from the keyboard while it runs. A session that
does one thing at a time is spending his wall-clock, which is the only resource
here that cannot be bought back. On the day he said it, **nine tracks ran
concurrently in one repository** — the glyph audit workflow, five defect tracks,
three ribbon-band wiring tracks — and every one landed.

**How to apply — the four things that actually made it work.** Parallelism in
one working tree is not free, and none of these is optional:

1. **★★★ Write an explicit ownership map into every brief, naming the OTHER
   tracks' files.** Not "don't break things" — *"another track owns
   `crates/pdfcer-gui/src/icons/`, `src/shell/`, `tools/ui-verify/`; if your fix
   needs one of them, STOP and report instead."* Every agent honoured it, and
   three of them stopped and reported rather than reaching across.

2. **★★★ Name the shared counters and forbid racing on them.** Three tracks each
   wired a different ribbon band and each would have had to bump the same
   `assert_eq!(named, 107)`. The brief told them not to, and all three reported
   their delta instead. The coordinating session moved it **once**, from 107 to
   119, with the arithmetic. A shared counter bumped three times in a race
   records the last writer's number and none of the reasoning.

3. **★★ Tell them the tree will TRANSIENTLY not compile, and that it is not
   theirs to fix.** With nine tracks writing, `cargo check` fails constantly for
   reasons in files an agent does not own. Without that sentence they either
   "fix" someone else's half-written function or give up. With it, one agent
   waited through **fourteen** retries and reported cleanly.

4. **★ Commit at reconciliation points, not per track.** Several tracks' changes
   are in the same file (`canvas/forms.rs` had two owners' work), so per-track
   commits would not compile. Wait for green, commit once, and put every track's
   findings in the message.

**★★★ AND THE MOST VALUABLE THING THEY DID WAS DISAGREE WITH THEIR BRIEF.**
Three of the eight found the premise they were given was factually wrong and
said so instead of implementing it: the "distinguishable only by icon and
tooltip" sentence was about different commands entirely; the sticky-note
dialog's position was never click-relative; the whole "there is no theme-only
fix, the work is at ~19 call sites" analysis was wrong on both halves and the
real fix touched **zero** call sites. So write briefs that invite the
contradiction — *"if you find this is wrong, stop and report"* — and read the
report before believing your own analysis.

Related: [[feedback_a_backlog_row_is_a_record_not_evidence]] — the same lesson
one level up. A brief is a backlog row with a deadline.
