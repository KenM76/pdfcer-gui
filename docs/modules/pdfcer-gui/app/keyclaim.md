# `app::keyclaim` — Tab and the arrows taken from egui's focus walk

## Contract

An `egui::Plugin` whose `input_hook` runs on every viewport's raw input,
after the scripted pointer's and before `Focus::begin_pass`. It calls, in
order:

1. `canvas::tabnav::claim` — a form-field or object ring that holds egui
   focus takes Tab and Shift+Tab as a ring step.
2. `canvas::textedit::claim_tab` — an open text draft, with no text field
   focused, takes a bare Tab as a typed tab (spaces to the next stop) and
   drops Shift+Tab and every Tab release.
3. `canvas::textedit::hold_arrows` — while a draft is open and no text field
   is focused, locks the arrow keys on the widget that holds focus (a page's
   `pagefocus` seat), so `begin_pass` turns none of them into a directional
   focus move. The draft's own key handling still reads them. The lock is
   released on the frame the draft closes, if that widget still holds focus.

A press neither claims reaches egui and walks its focus ring as usual.

## Why a plugin

`Focus::begin_pass` latches a Tab into a focus move straight from
`RawInput.events`, before any application `ui` code runs. Consuming Tab
later removes the evidence and not the effect: focus still walks, onto the
first focusable widget of the frame — a Quick Access Toolbar button — and the
next Enter presses it. Inside a text draft that opened the file dialog.

`eframe::App::raw_input_hook` also runs before the latch, but it runs before
every plugin too, so it never sees the scripted pointer's events. A plugin
registered after the script sees a driven Tab exactly as it sees an OS one,
which is what lets `ui-verify` measure the operator's route.

It knows nothing about forms, objects or text: whether a surface owns the
press is decided inside the two claim functions.

## Why the arrows too

`Memory::begin_pass` turns any unmodified arrow press into a directional
focus move unless the focused widget's `EventFilter` claims it. A click on a
page gives that page's seat focus, so in a draft the arrows walked focus to
the next page's seat or a ribbon button (the highlight Ken saw), and the next
Enter pressed whatever it landed on — Save As, in the driven reproduction.
`set_focus_lock_filter` is the only per-widget claim egui offers, and it
applies only to a widget that held focus last frame and still does, which
the seat does from the frame after the click.

## Trace

| line | when |
|---|---|
| `keyboard-focus to=<hex id|none> draft=<bool>` | egui's focused widget changed, root viewport, read at end of pass |

## Verified by

`ui-verify` check `arrows_in_a_text_draft_stay_with_the_caret`
(`docs/modules/ui-verify/checks/draft_arrows.md`).
