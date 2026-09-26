# `app::prefs::chrome` — how big the program's own controls are drawn

One preference, and it is the only one in this store that is an
**accessibility** control rather than a taste or a speed trade.

## Why the shell has to provide this itself

`egui` ships a *built-in* `Ctrl` + `+`/`-`/`0` handler that drives
`Context::set_zoom_factor`, so on a stock `eframe` application UI scaling
appears to exist without anyone building it. This shell **switches that
handler off**, in [`crate::app::configure_context`], because in a document
viewer those chords mean *page* zoom, as they do in every browser, in
Acrobat and in every other PDF reader. Nothing else in this crate calls
`set_zoom_factor`, so without this preference every control, label, icon and
panel would be drawn at exactly one size on every machine.

**A framework default you switch off may be carrying a capability you never
decided to have.** Check what goes with the handler before removing one.

## Where the reference applications put it, and where the chords went

The standing tie-breaker is to match Inkscape, Acrobat and SolidWorks — but
first to ask which of them actually has the surface:

| application | UI scale control | chord |
|---|---|---|
| **Inkscape** | Preferences ▸ Interface ▸ *Interface scale* | none |
| **SolidWorks** | not a scale control; it follows the Windows display setting | none |
| **Acrobat** | no UI-scale control at all | — |

So: **a settings control and no chord**. None of the three binds a chord to
it, and no chord is available here anyway — `Ctrl` + `+`/`-`/`0` are taken
by page zoom and `Ctrl+1`/`2`/`3` are the mode selector, so an invented
third family would be a chord nobody's muscle memory has that collides with
the two that do.

## It lives in the *Appearance* group, beside the theme

Not in *Drawing the page*, where most of this store's preferences are
presented. The window's groups are a **navigation model** — an operator
arrives with a symptom and the heading is how the symptom finds its
setting — and the symptom here is *"the program's text is too small to
read"*, which is a question about the window, not about the page.

Theme and UI scale are the two settings that change **the program's own
appearance and nothing about the document**, and they belong together for
that reason. It is also the only group whose two members share the live
preview below.

## Item notes

### `fn the_shipped_scale_is_the_identity_and_is_reachable`

Both halves matter. Identity is the "the shipped default reproduces a
build without the choice" rule; reachability is the recurring one that a
default must sit inside its own widget's range, or the first operator to
open the window has their value rewritten without touching anything.

### `fn normalising_twice_changes_nothing`

The property that makes the load path safe to run on its own output —
which it is, every time pdfcer saves and reloads. A rounding that moved
a value it had already produced would make the preference drift a step
per restart, which is the kind of defect that takes a fortnight to be
noticed and is then very hard to attribute.

### `fn every_value_the_control_offers_round_trips`

The weld between the widget's step and the file's grammar. If the
slider could land on a value the loader would round away, the operator
would set a scale, restart, and find a different one — with the file
on disk holding what they chose and the program showing something else.
