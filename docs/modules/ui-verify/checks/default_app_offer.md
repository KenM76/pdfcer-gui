# `ui-verify/checks/default_app_offer`

`default_app_offer` — **the ask-once offer reaches the screen, both its
answers are reachable, and it does not come back once it has been answered.**

`OPERATOR_REQUESTS.md` **O173**, his words of 2026-09-10: *"we should have
an easy way to make pdfce-gui our default opener for pdfs. Ask once with a (old-name-exempt: HIS words, quoted verbatim from O173 — correcting an operator's own sentence would stop this being a quotation)
don't show me again check box option."*

# ★★★ The one thing this check must do before it launches anything

**Delete the preference the sandbox seeds.**

Every ui-verify sandbox is by construction a fresh profile, so the offer
would otherwise open in front of all two hundred-odd driven checks and take
their pointer presses — the standing lesson that *a window over the thing it
describes takes that thing's gestures*. [`crate::sandbox`] therefore writes
one key into every sandbox that declines the offer in advance, and its own
doc comment states the cost in the same paragraph as the benefit:

> **A check written to drive that offer must delete this file first**, or it
> will assert against a starting state that defeats the very thing it
> measures.

That is this check, and [`clear_seed`] is the deletion. It is not a
convenience: without it every assertion below is still *evaluable* — the
window simply never opens — and the check would report *"the offer did not
appear"* about a build in which the offer works perfectly. A fixture that
defeats a default does not defeat a starting state; the starting state has
to be planted.

# ★★ What this check will NOT do, deliberately

**It never presses the affirmative button.** That button writes ten values
under `HKCU` on the machine running the sweep and then opens Windows' own
Default-apps page in front of whatever the operator was doing. A driven
check is not entitled to either. The registration mechanism is unit-tested
in `crate::app::assoc`; what only a driven run can prove is that the window
**opens, and that its answer row is on the screen** — which is precisely the
class of defect O171 was, one row above this one in the same file.

⇒ So the button is asserted **present and visible** and then left alone, and
the window is answered through *Not now*, which is the one route out that
changes nothing outside pdfcer.

# ⚠ When this check legitimately cannot run, and why that is a SKIP

The shell decides whether to ask by **reading Windows**, not by reading a
flag — deliberately, so that an operator who set the default by hand is not
asked about something already true. ⇒ On a machine where pdfcer is
**already** the registered `.pdf` handler the offer never opens, and there
is nothing here to drive.

That is reported as a skip carrying the list of regions the run actually
declared, rather than as a failure. A check that went red on a
correctly-behaving build would be edited away inside a week, and the edit
would take the four real assertions with it.

★ It is worth knowing that this makes the check's coverage a function of the
machine it runs on: the day the operator accepts the offer for real, this
check stops exercising anything on his desktop and keeps exercising
everything on a clean one. The standing lesson *a SKIP is not red, so a
check can stop running unnoticed* applies directly — **diff the sweep's SKIP
set**, and if this name appears in it, read the reason before assuming the
sweep covered O173.

# Phases

| Phase | Does | Expected |
|---|---|---|
| A | delete the seeded preference, launch with **no document** | the offer opens |
| B | read the regions | `defaultapp.body`, `.action`, `.dont-ask` and `.later` all declared |
| C | capture the window | attached as evidence |
| D | tick the box, press *Not now* | `default-app-settled dont_ask=true saved=true` |
| E | relaunch the same profile | **no** `defaultapp.body` this time |

★ Phase E is the half of *"ask once"* that no unit test can reach. The unit
tests assert that the dialog **writes** the preference. Only a second launch
against the same profile directory proves the written preference is **read
back** on the path that decides whether to ask — and two files and a round
trip through disk sit between those two facts.

# Rule 15

No dimension of either kind appears in this module.
