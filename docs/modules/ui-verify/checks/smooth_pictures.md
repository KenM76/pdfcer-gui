# `checks::smooth_pictures` — Smooth enlarged pictures follows the setting

Opens `fixtures/checker-image.pdf` (a 4 × 4 black-and-white checkerboard drawn
300 pt wide) twice: once with the preferences file bare, which is the shipped
default, and once with `smooth_enlarged_pictures = false`. It measures the
share of the box, inset 10 pt from its edges, whose pixels are mid-grey
(every channel 40..215).

Smoothed, a 4-pixel image ramps between black and white across most of each
square, so the share is high (at least 0.20). Point-sampled, only the square
edges can be grey, so it is near zero (at most 0.05). The two thresholds are
far apart on purpose: the property is "the setting changes the picture", and
a check that needs a precise grey count would measure the filter, not the
setting. The sandbox's preferences are reset afterwards.
