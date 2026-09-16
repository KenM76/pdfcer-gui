---
name: a-launch-failure-blamed-on-a-resource-count-needs-a-control-binary
description: "Not enough memory resources" at launch was attributed to OneDrive's handle leak, then to Outlook's handles, then to nothing — each a count that correlated once. A twelve-hour-old release build failing the same way settled it in four launches. Launch the control before blaming a count.
metadata:
  type: feedback
---

**When a launch dies before its first frame with a resource-flavoured error, launch a binary that is NOT yours (an older release build) before attributing it to any resource count.**

**Why:** 2026-09-09, 00:15–01:30. `accesskit_windows` `SetPropW` died with `0x80070008`. The packager's header blamed the OneDrive mirror's handle leak (measured once, 2026-08-26). I measured 398 k handles, saw Outlook holding 203 k, and amended the header to blame Outlook — a second attribution from a single correlation. Outlook closed, handles at 157 k, launches kept dying. USER/GDI objects, both atom tables, commit and free RAM were then each measured and ruled out. What settled the SUBJECT in four launches was the 13:05 release build in `D:\builds` failing 2 of 4 the same way: the session, not the build, and not any count I had. The only monotone correlate was ~60 force-killed harness instances; the remedy is a logoff. Two wrong attributions reached print (one in the packager's own header) before the control ran.

**How to apply:**
- A resource count that correlates with one failure is a hypothesis; the control launch is the test. Run it first — it costs a minute and it decides the subject.
- Any environment note that names a cause ("this is the handle leak") must name what it compared against and what a control did. The packager header now carries two amendments because it did not.
- Force-killing dozens of GUI instances degrades the session in a way no count I could read shows; a driven sweep on a many-day session is itself a treatment.

## The same class on a layout number — 2026-09-16

A `ScrollArea` body reported `egui_content` past its `egui_view` and the number
alone could not say whether it was the regression or the normal state, because
the only thing available to reason from was the code just changed. No unit test
produces those numbers, and a screenshot only repeats the complaint.

**The control was Ken's own running exe**, copied to scratch and launched
off-screen: `egui_content=[760.0 534.0]` in `egui_view=[776.0 550.0]`. Three
launches — control, regressed, candidate — settled the subject, and the fix was
accepted only when it **reproduced the control exactly**. "The scrollbars are
gone" is satisfied by any number under the viewport, including a wrong one that
the next feature exposes; equality with the control is satisfied by one.

⇒ The generalisation over both instances: **a number with no control beside it
is an attribution, not a measurement** — and the shipped build is the only
control that is independent of the hypothesis, because it was built before there
was one. Full mechanism in
`D:/dev/rag/egui/the_last_shipped_build_is_the_only_calibrated_baseline_for_a_layout_number.md`.

★ Copy the exe to scratch first — never drive the published build, see
[[never-drive-the-published-build]] — and kill it by PID verified against its
path, never by image name, see [[never-kill-pdfcer-gui-by-name-he-uses-it-all-day]].
