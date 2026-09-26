# `ui-verify/checks/reaching`

`checks::reaching` — **putting a control where the pointer can hit it.**

# Why this is its own file


The seam is a real subject rather than an arbitrary cut, and the tell is
that all three functions here answer the **same question** —

> *the application says this control is at that rectangle. Can the pointer
> actually reach it there, and if not, what has to happen first?*

— where the rest of `driving` answers *"where is it"*, *"what frame is it
in"*, and *"press it"*. This project has spent real sessions on the gap
between those two questions, and every incident is written up on the
function that closed it:

| function | the failure it exists to prevent |
|---|---|
| [`scroll_to`] | a control below a pane's fold, never declared, reported as missing |
| [`raise_dock_tab`] | a docked pane that is **not in front publishes nothing**, which is indistinguishable from a panel with nothing to say |
| [`bring_into_body`] | a control declared through the ungated `diag::ui_rect`, laid out **past the bottom of its panel**, clicked at a centre that is outside it |

The three are stated as one family because getting the wrong one produces
the same class of report every time: **a confident, articulate failure about
a mechanism the trace can rule out.** [`bring_into_body`]'s header carries
the worked example — a Paste button nineteen points inside its panel, and a
failure message naming `paste_outline_item`, which had never been asked.

They are re-exported from [`super::driving`], so `driving::scroll_to` and
the rest keep working at every existing call site. The file is what R2
limits; the module path a check reads is a separate question and moving it
would have been churn with no reader served.
