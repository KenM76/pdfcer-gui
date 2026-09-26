# `ui-verify/checks/about`

`about` — **About opens, reports the build, and shows its attributions.**

The first driven check of the About window, which until now had no declared
region and so could not be found by anything.

# Why About is worth driving at all

It is the one surface in this program with a **legal** obligation behind it.
`dialogs::about`'s own header sets it out: the third-party attributions are
not decoration, they are the discharge of the notice requirements the
bundled licences impose. A build where that list silently stopped rendering
would look completely normal — nobody opens About — and would be a licence
breach shipping unnoticed. Unit tests assert the *strings*; only a driven
run asserts they reached a window.

# What it asserts, and why the build block is a trace rather than pixels

The operator asked on 2026-08-18 for About to carry *"the date and time of
the build … and the date and time of the builds of the used pdfcer and
iccce"*. Those values come from `build.rs` through `env!`, so the thing that
can go wrong is not the wording — that is unit-tested — but a value arriving
**empty**, which renders as `built  from abc1234` and reads as a layout
glitch rather than as a missing stamp.

Reading four fields out of a PNG is not something this harness can do, so
the application traces them alongside drawing them and this check asserts on
the trace. The window is captured too, and the capture is attached, because
the layout question — does the block fit, is it legible — has exactly one
oracle and it is a rendered pixel.

# Phases

| Phase | Does | Expected |
|---|---|---|
| A | click **File ▸ About** | `dialog:about` declared |
| B | read the provenance trace | `about-build stamp=… rev=… engine=…`, none of them empty |
| C | capture the window | attached as evidence |
