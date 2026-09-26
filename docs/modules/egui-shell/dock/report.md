# `egui-shell/dock/report`

## Item notes

### `fn a_report_carries_the_clip_in_force_not_the_region_itself`

Worth a test of its own because the failure mode is invisible: a
reporter that handed back `rect` as its own clip, or the screen
rectangle as everybody's clip, would make every consumer's
visibility fraction come out at exactly 1.0 and every check built
on it green forever.
