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

## Item notes

### `fn folders`

Takes `&mut Prefs` rather than the settings draft, like
`appearance::ui_scale`: this is a **shell preference**, not a
`pdfcer_core::settings` entry, and the window's own header explains that the
two live side by side in one dialog because an operator does not care which
file a setting lands in.

### `fn style_policy`

# What this setting is, in this shell

The engine calls it `StylePolicy` and gives it three values. Its own reading
of them is narrower than this window's, and the difference is deliberate and
worth stating rather than papering over.

`pdfcer-core`'s gate asks exactly one question: *"could a real face have done
this instead?"* — so its `Refuse` refuses a fake **only when a real face was
available and would have been passed over**. On a page carrying no bold at
all there is nothing to refuse in favour of, and the engine thickens the
strokes under all three postures.

That is the right contract for a crate whose caller might genuinely mean
*"fake it"*. It is not what this shell's Bold button means. Here the button
means **"make this bold"**, and `crate::app::actions::textstyle` already
prefers a real face under every posture — it asks the gate with `Refuse`
pinned, purely to learn which real face is on offer, and then takes it.

⇒ So the only question left for the operator is the one this control asks:
**when no real face will do, may pdfcer fake one?** All three engine values
map onto it, and every one of them is observable:

| choice | what happens |
|---|---|
| Fake it quietly (`Auto`, the default) | the letters are thickened or slanted, and it is reported with the edit's other disclosures |
| Fake it and say so (`Warn`) | the same, plus a sentence of its own on the status bar |
| Never fake it (`Refuse`) | nothing changes, and pdfcer says no real face on the page can show that text |

# Why it lives in Fonts and not in Text

The window's rule is *"whichever group matches the SYMPTOM that would send
somebody looking"*. The symptom here is **"my bold looks wrong / pdfcer did
not use the bold font"** — a statement about faces. The Text group is about
what comes out when you copy, which this never touches: a synthesised weight
changes how a run is painted, not what it extracts as.

# Never fake it is NOT the same as an error

The third option changes nothing and says so, which makes it the only
setting in this window that can make a control appear not to work. The note
under it says that in advance, because an operator who ticks it in January
and presses Bold in March will otherwise file a bug.
