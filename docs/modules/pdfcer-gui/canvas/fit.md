# `canvas::fit` — **where the view goes when the viewport changes, or a fit
is pressed**

★★★ **The subject widened on 2026-08-31** (`OPERATOR_REQUESTS.md` O78) and
the old title — *"spending a fit command's request to place the view"* — is
kept above the new one because the widening is the finding.

The operator:

> *"when I change the size of the canvas window, whatever area was centered
> in the current canvas should stay centered, and unless I have manually
> changed the zoom after clicking one of the preset options, the pdf should
> maintain whichever option was selected."*

## ★★★ Preserving the centre SUBSUMES a fit's re-placement


On an axis a fit **pins**, the page is by construction no larger than the
viewport, so `margin = (v − d) / 2`, and holding the page's own centre at
the viewport centre gives
`(v − d)/2 + 0.5·d − v/2 = 0` — **exactly** what
[`crate::canvas::geometry::fit_placement_offset`] returns for a pinned
axis. So a fit-page document nobody has panned is re-centred by the general
rule for free, and `centring_agrees_with_the_pinned_fit_answer` pins that
equality so deleting the old path cannot silently change what Fit page
does.


## The original subject, unchanged below this line

# `canvas::fit` — spending a fit command's request to place the view

## The request


> *"If I press the Fit width or fit page button the view should center to
> the width as well or center the page."*

## ★★★ Why a fit is now a position as well as a scale

Before O23's pasteboard a page no larger than the viewport had nowhere to
be except the middle, so *fit* and *centred* were the same act and the
button never had to choose between them. The pasteboard added a whole
viewport of slack on every side — deliberately, so any corner of the page
can be brought to any point of the screen — and with it the state the
operator is reporting: **the scale is right and the page is not on
screen.**

## The two-frame handshake, and why it is the same one the zoom anchor uses

`Action::Fit` cannot place the view itself: the re-fitted zoom is computed
by `ViewState::apply_fit` from a viewport the action funnel cannot see, so
the page's new drawn size is not known until the canvas next runs. So the
action records the request on [`crate::app::state::OpenDoc::fit_placement`]
and this module spends it on the following frame, by which time
`apply_fit` has run near the top of `show_in` and `current_display` is the
page's **new** size.

That is exactly the shape [`crate::canvas::zoom`]'s anchor uses, for
exactly the same reason, and the resemblance is not a coincidence worth
collapsing: an anchor says *"hold this page point where it is"* and a fit
says *"decide where the page goes"*, which on a pinned axis is the
statement that there is no previous position worth holding.

## Where the rules live, and why not here

* *Which axes does this mode pin?* — [`crate::viewer::FitMode::pinned_axes`],
  beside `fit_scale`, because the two are one decision: an axis is pinned
  exactly when the fit has just decided its extent.
* *What offset does a pinned or unpinned axis get?* —
  [`crate::canvas::geometry::fit_placement_offset`], beside the other
  offset solves and the `margin` term whose definition makes the pinned
  answer a single constant.

This file is the **frame plumbing** between them: read where the view is
now, ask the two rules, hand back an offset. Keeping it separate is what
stops either rule being restated in `canvas::show`.
