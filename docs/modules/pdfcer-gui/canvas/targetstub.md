# `canvas::targetstub` — a target provider assembled from plain rectangles

A `CanvasTargetProvider` built from rectangles, so selection tests run without
a document. It lists objects and form-interior leaves separately because the
two index spaces are different things, and only an object is an edit operand.
