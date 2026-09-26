# `canvas::paging` — the wheel as a page turn

## The request


> *"when in single page view there should be an option on screen near the
> button to scroll or flip through pages, or the current way it is now when
> the scroll wheel is used."*

## ★★★ The case for it, which is stronger than a preference

This shell opens documents at **fit page** by default. Under
[`crate::viewer::PageDisplay::Single`] that means the whole sheet is on
screen and there is *nothing to scroll* — so the plain wheel, the most
reached-for gesture on a mouse, **does nothing at all**. Not "does
something the operator did not want": nothing. Every page change costs a
trip to the status bar or a key the operator has to know about.

So this is not a taste setting with two equally good answers. It is the
difference between a live control and a dead one, in the configuration the
program ships in.

## What is deliberately NOT here

* **Continuous modes.** The wheel scrolls the whole document there, by
  definition, and [`flip`] declines before it reads anything. The status
  bar's toggle is not drawn there either — R9, and the two decisions are
  made from the same predicate so they cannot disagree.
* **Ctrl+wheel.** `egui` routes a modified wheel event into `zoom_delta`
  and contributes nothing to the scroll delta, so a zoom gesture never
  reaches this module and does not have to be excluded by it. See
  `canvas`'s header: keeping those two apart is *"the single most common
  way a from-scratch viewer feels wrong"*.
* **Deciding whether the scroll area also sees the wheel.** That is the
  caller's, one line above the call, because it has to be decided *before*
  the `ScrollArea` is built and this runs after. It asks
  [`wheel_turns_pages`], of which [`flips_pages`] is the narrowing, so a
  frame can never both scroll and page.

## A one-page document

The wheel is still taken from the scroll area, and turns nothing: the
fitted page stays put rather than sliding over the pasteboard (O239). The
scroll bars and the middle-button pan still move a zoomed-in page, as they
do on a longer document in this mode.
