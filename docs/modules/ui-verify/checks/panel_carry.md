# `ui-verify/checks/panel_carry`

`panel_carried_home_lands_where_it_was_aimed` — **a floating panel dragged
by its header onto a dock compartment joins THAT compartment.**

# What this is for, and what it is not

`panel_float` proves a panel tears out into a real window, draws in it, and
can be put back — by a **command**. Putting it back by command always
returns it to the compartment it came from, because that is the only
address a command has. This check is about the other route: the operator
picks the window up and drops it somewhere *else*.

The two routes are worth separating because they fail apart. A build can
have a perfect `view.panel_dock` and no carry at all — and the symptom is a
window whose only way home is a menu row.

# The oracle is *which* compartment, not *whether it docked*

A carry that landed nowhere falls back to leaving the window where it was
let go; a carry that landed *home* is indistinguishable from the command
route. So the aim is deliberately a compartment the panel is **not** from:
Layers is a left-dock panel in Edit's default arrangement, and this drops
it onto the right dock's Objects stack. The assertion is that the two tabs
end up in the **same tab bar**, found by containment — which is false for a
drop that went home, false for a drop that was ignored, and false for a
drop that resolved against the wrong compartment's rectangle.

★ Containment rather than a compartment address, for `panel_tab_reorder`'s
reason: a tab region is named for its panel and carries no compartment, so
the only thing that says which bar a tab is in is where it was drawn.

# Why the gesture needs its own verb

`Driver::drag` raises the application's main window first, which on this
gesture would bury the float window the press is aimed at. `Driver::carry`
raises the window the gesture *begins* in, and rests on the destination
before releasing — see its documentation for why a release on the arrival
frame lands nothing.

# Where the aim points come from

Both are regions the application published on the frame it drew them, which
is region source 1 and the only one that survives a layout change.

| end | region |
|---|---|
| from | `float.header.<panel>`, tagged with the float window's viewport |
| to | `dock.body.<target>`, in the application window |

The `from` region exists **because of this check**. The header strip is the
shell's own geometry, derived from `floatwin::BODY_MARGIN_PTS` and
`floatwin::split_header`, and the shell has no diagnostic channel and must
not grow one (R7). A harness that re-derived the strip from those two
constants would be holding a copy of the shell's layout, and would go on
aiming confidently at the old place the day either constant moved. The
application publishes it instead, from the tab-menu handler it already
supplies for that strip.

# What this deliberately does NOT assert

* **That the compass was visible while the window was over it.** It is
  drawn in the application window and the carried window is on top of it,
  which is a disclosure shortfall recorded in `GUI_ROADMAP.md` rather than
  a correctness one — the drop still resolves and still lands. Asserting it
  needs a screenshot taken mid-gesture and a rule about what "visible
  enough" means.
* **Every zone.** One drop into one compartment. The five-zone grammar and
  its previews are `dock::overlay`'s unit tests; what those cannot reach is
  whether a real pointer crossing a real window boundary arrives at the
  grammar at all, and that is this file's single question.
