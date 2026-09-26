# `prefs::wheel` — what the mouse wheel does when the document is not a scroll

## The request


> *"when in single page view there should be an option on screen near the
> button to scroll or flip through pages, or the current way it is now when
> the scroll wheel is used."*

## Why this is a choice at all, rather than a behaviour

Under a **continuous** display mode there is nothing to decide: the whole
document is one scroll and the wheel scrolls it. Under
[`crate::viewer::PageDisplay::Single`] and
[`crate::viewer::PageDisplay::Facing`] the wheel is ambiguous, and the two
answers are both right for somebody:

* **Scroll the page.** The wheel moves within the sheet and never leaves
  it. This is what the build does with no preference set, so it is the
  default — an option that changes what the operator already has is not an
  option, it is a surprise.
* **Flip pages.** The wheel turns to the next or previous sheet. On a
  drawing set opened at fit-page — which is how this shell opens documents
  by default — there is *nothing to scroll*, so today's wheel does nothing
  at all and the operator reaches for the page buttons every time.

That last sentence is the whole case for the feature. The default
behaviour is not merely a matter of taste in the common configuration; it
is a dead control.

## The two rules that keep this honest

1. **Ctrl+wheel is untouched.** `egui` routes a modified wheel event to
   `zoom_delta` and contributes nothing to the scroll delta, so zoom is not
   a case this module has to exclude — it never arrives here. `crate::canvas`
   owns why that separation must not be broken.
2. **The control renders only where the choice exists** — R9. Under a
   continuous mode nothing is drawn, rather than a disabled stub explaining
   that the setting does not apply.
