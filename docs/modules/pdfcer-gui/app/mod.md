# app — the one owner of state, and the shape of a frame

[`PdfcerApp`] is the single owner of everything the shell knows. There is
no global, no `thread_local`, no second source of truth: if it is state,
it is reachable from here, and if it is reachable from here it has one
owner. That is what makes the action funnel in [`actions`] enforceable —
a widget cannot mutate a document it has no path to.

## The order of a frame, and why the order is load-bearing

[`PdfcerApp::update`] runs four steps, in this order, every frame:

1. **Collect keyboard actions** ([`keyboard::collect`]) and dispatch the
   frame's **manifest chords** ([`keyboard::commands`] →
   [`PdfcerApp::dispatch_command`]) — before any widget is built, so the
   map sees the frame's raw key presses rather than whatever survived a
   widget consuming them. The split between the two is the subject of
   [`keyboard`]'s ★ section: chords the manifest keymap binds arrive as
   command ids and go through the same dispatcher a ribbon click does,
   and chords the viewer owns outright arrive as actions.
2. **Compose the panels** — draw, and let each surface push more
   actions, then the Find overlay over the top of them. Nothing mutates.
3. **Apply the actions** ([`PdfcerApp::apply_actions`]) — after the frame
   is drawn, in one place, in the order raised.
4. **Settle and rasterize** ([`PdfcerApp::settle_and_rasterize`]) — decide
   whether the cached texture still matches the (now updated) view
   state, and start a render if not.

Step 4 must come after step 3 or every zoom would be rasterized one
frame late — visible as a page that always lags the operator's last
action by a frame. Step 3 must come after step 2 because that is the
actions-not-mutations invariant itself.

## Panel composition order is load-bearing (layout *and* focus order)

egui resolves panels in the order they are added, and that order
determines both the rectangles they get and the order the Tab key visits
their widgets. **S0 adds exactly one panel** — the `CentralPanel` — so
there is nothing to order yet. The rule is written down here anyway,
where the composition lives, because it is the thing the old shell got
bitten by and it constrains every panel S2 and S3 add:

> A full-width bar (toolbar, status bar) must be added **before** any
> side panel, or it starts at the side panel's edge instead of spanning
> the window. A status bar that does not span the window is not a status
> bar.

That constraint conflicts with the Tab order a reviewer would prefer
(toolbar → canvas → footer), and in the old shell the layout property
won, deliberately. The same trade will be made at S2, and it should be
made with this paragraph in front of whoever makes it.

## What this build does not have, stated so it is not mistaken for an
oversight

The list S0 opened with — *"no ribbon, no QAT, no dock, no status bar, no
find bar, no dialogs, no Open command"* — is down to **the save**. The find
bar is [`crate::find`], reached by Ctrl+F and by the status bar's Find
toggle. **Open, Close and Recent are wired**: the picker and its
diagnostics seam are [`files`], the list is [`recent`], and both arrive
through [`actions::Action::Open`] like everything else. What is left of
that sentence is scheduled in `PROJECT_PLAN.md` §4, and the
"no placeholders" invariant still applies to all of it: unavailable
renders **nothing** rather than a greyed-out control that explains itself
badly.
