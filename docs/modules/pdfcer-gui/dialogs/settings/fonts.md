# `dialogs::settings::fonts` — the Settings window's Fonts group

One control: the list of folders pdfcer searches when it has to embed a font
a document names but does not carry.

## Why it is HERE and not on a batch pane

Nothing about a list of directories needs the host it was first drawn in.
This window is the surface whose whole subject is *"settings that persist
across documents"*, which is exactly what a font search path is.

⇒ Worth stating because the shape recurs: a blocker naming a **missing
host** is weaker than one naming a missing capability, and it goes stale the
moment any other host will do — without anyone noticing, because a
host-shaped blocker never prompts the question of whether some other host
would already serve.

## The command that points here

`tools.font_folders` on the Tools tab opens this window **landed on this
group**. It is not a route to `file.settings`: both ids are owned by
[`crate::app::dispatch::settings`] and share one window, one draft and one
Save, differing only in where the window lands. They are still one
implementation — what separates them is that *"where do font folders live"*
is a different **request** from *"show me the settings"*, and a route's
target is a bare id with nowhere to carry that.

## What this group deliberately does NOT do

**It does not check that a folder exists**, and it does not list the faces
in one. The first is [`crate::app::prefs::fonts::add`]'s stated position —
an unmounted drive is still where the fonts live. The second is a font
census, which is `panels::fonts`' subject and would be a second inventory
with a second way of going stale.
