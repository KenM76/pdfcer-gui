# `app::keyclaim` — Tab taken from egui's focus walk

## Contract

An `egui::Plugin` whose `input_hook` runs on every viewport's raw input,
after the scripted pointer's and before `Focus::begin_pass`. It calls, in
order:

1. `canvas::tabnav::claim` — a form-field or object ring that holds egui
   focus takes Tab and Shift+Tab as a ring step.
2. `canvas::textedit::claim_tab` — an open text draft, with no text field
   focused, takes a bare Tab as a typed tab (spaces to the next stop) and
   drops Shift+Tab and every Tab release.

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
