# `egui-shell/ribbon/trailing`

## Item notes

### `fn shown`

One helper rather than the same `filter` written twice, because
[`measure`] and [`render`] disagreeing about which items exist is exactly
the class of defect this region's reservation is supposed to make
impossible.
