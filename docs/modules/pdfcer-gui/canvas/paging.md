# `canvas::paging` — the wheel as a page turn

## The request


> *"when in single page view there should be an option on screen near the
> button to scroll or flip through pages, or the current way it is now when
> the scroll wheel is used."*

## The case for it, which is stronger than a preference

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

## Item notes

### `const POINTS_PER_PAGE`

# Why a distance and not an event count

The two devices that produce a wheel do not agree on what an event is. A
mouse delivers one detent as a single large delta; a trackpad delivers one
swipe as dozens of small ones. Counting events turns a trackpad gesture
into forty page turns. Thresholding an instantaneous delta makes a slow,
deliberate scroll do nothing at all. **Travel is the quantity both devices
agree on**, and a threshold on it behaves for both.

`egui`'s own default for one wheel line is 50 points at the time of
writing. This is deliberately a little under that, so a single detent
reliably turns exactly one page even if the platform reports slightly less
— and comfortably over the few points a trackpad delivers per frame, so a
swipe pages at the speed of the hand rather than of the frame rate.

### `fn travel_accumulates_and_one_threshold_buys_exactly_one_page`

Asserted through the accumulator rather than by driving `egui`,
because the arithmetic is the part that can be wrong. The gesture
itself is `zooming`-harness territory.

### `fn wheel_turns_pages`

Asked before the `ScrollArea` is built, to withhold the wheel from it, and
by the status bar, to decide whether the toggle exists. Independent of
the page count: see the module header's one-page section.

### `fn flip`

Call once per frame, after the canvas has drawn, with `hovered` saying
whether the pointer is over the canvas. Pushes at most one
[`Action::NextPage`] or [`Action::PrevPage`] per call.

# The sign, and why it is this way round

`egui`'s scroll delta is positive when the content should move **down** —
i.e. when the operator is scrolling **up**, toward the start. So a positive
delta is a *previous* page. Getting this backwards produces a viewer that
works and feels wrong, which is harder to notice than one that is broken;
[`tests::rolling_the_wheel_up_goes_back_and_down_goes_on`] pins it.

# Why the accumulator is zeroed rather than decremented

Subtracting the threshold and keeping the remainder would let a long
trackpad swipe page continuously at a rate set by the hand — which sounds
right and is not: the remainder carries across the gesture's end, so the
*next* small nudge lands a page turn it did not earn. Zeroing makes every
turn cost a full threshold of fresh travel, which is what "one notch, one
sheet" means.

The accumulator is also **reset on a direction change**, so a wheel
rolled half a notch forward and then back does not arrive at a page turn by
cancellation. Travel toward a page turn is travel in one direction.
