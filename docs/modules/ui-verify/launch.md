# `ui-verify/launch`

Start the built binary, capture its diagnostic trace, find its window, and
never leave it running.

## The binary, not a test harness

This module launches `pdfcer-gui.exe` — the actual artefact, built in
release, opening an actual file. That is the entire premise of the crate:
D1 and D2 both live in the space between our code and the framework, and
neither is reachable from inside a test binary that constructs an
`egui::Context` by hand.

## Two guarantees this module owes the operator

**1. It never kills a process it did not start.** The operator may well
have the application open for their own work — this harness drives the real
desktop, which is precisely the situation where that is most likely — and a
harness that tidied up by killing "all pdfcer-gui processes" would close
their document. So [`Session`] holds a child handle and kills exactly that.

**2. It never leaks the process it did start.** pdfcer's predecessor script
killed its child on its last line, so any error before that line left a
window running: parked off-screen, invisible, and still consuming pointer
input on the operator's desktop. The operator reported it as *"do you have
some gui processes leftover that are interfering with my mouse?"* — twice
in one session, which is what made it a defect in the tool rather than an
operating mistake. Here the kill is in [`Drop`], so it happens on every
path including a panic.

## The staleness gate

[`Session::launch`] refuses a binary older than the newest source file
under `crates/`, unless explicitly told not to. The failure this prevents
is the worst kind: the traces a developer expects are simply **absent**,
which reads as "the feature does not work" rather than "the feature was
never compiled". pdfcer recorded an agent nearly concluding a panel did not
render, when the binary predated every change it had made.

An absence is only evidence when the thing that would have produced it was
actually built.
