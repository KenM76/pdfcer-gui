# `text::settings::shell` — the settings that change pdfcer's own window

The fourth row of the blast-radius taxonomy in [`super::look`]'s header.
Every setting here stops at what the operator sees or does: none of them
writes a byte, none of them changes a page, and each one's `_radius` line
says so, because an operator changing how Tab behaves or how big the
buttons are drawn has every reason to wonder whether they are also changing
the document.

## Why this is a module and not a section of [`super::look`]

The window's opening paragraph promises that everything in it exists
*because the standard declines to have an opinion*. That promise is true of
the first three modules and false of this one, and the difference is not
cosmetic: a `_silence` line has to answer *"what does the standard leave
open?"*, and for a setting the standard has never heard of the honest
answer is **"nothing — this is not a standards question"**. Those lines say
exactly that rather than inventing a clause to sit under, and keeping them
in one place is what stops the next one being written to match its
neighbours instead of the truth.

Two settings here answer a real ambiguity whose blast radius still stops at
a keystroke — the Tab-order pair, where the standard describes the second
pass twice and the two descriptions disagree. They are filed by radius like
everything else in this window, so they are here.

## The theme and UI scale are twins and must stay together

They are the only two settings in the whole window that take effect **before
Save** — both apply the moment they are picked so the operator can see them,
and both are put back by Cancel. That is an exception to the window's
draft-until-Save contract, and it is stated in two `_radius` lines that have
to keep saying the same thing. Split across files, one of them drifts.

## Item notes

### `fn theme_silence`

Nothing — and saying so is the point. This is the one setting in the window
that is not a spec ambiguity, and letting it silently share the shape of
the twelve that are would imply pdfcer thinks the standard has an opinion
about window colours.

### `fn theme_preset_label`

# The catch-all arm is required, and it is not a `todo!()`

`egui_shell::theme::Preset` is `#[non_exhaustive]` — deliberately, because
the whole point of the shell crate is that another application may ship
presets pdfcer has never heard of. So this catalog cannot be exhaustive over
it and the compiler says so.

The fallback returns the preset's own **key** rather than a placeholder.
That is the honest answer: a theme this catalog has no prose for still has
a name the operator can recognise, since the key is what they would have
typed into the settings file. A `todo!()` would crash the settings window
on a preset the *shell* is entitled to add, and a literal like "Other"
would tell them nothing and would be indistinguishable between two such
presets.

### `fn theme_preset_note`

Each says what it looks like *and* what it costs, because "which theme do I
want" is not answerable from three adjectives. Airy's *"uses more screen"*
and Dark's *"as CAD tools do it"* are the two facts that actually decide it
for this audience.
The catch-all returns an **empty** description rather than inventing one,
and the window omits the line entirely when it is empty — an absent note is
truthful about a preset this catalog cannot describe, whereas a generic one
would be prose pdfcer made up about somebody else's theme.

### `fn theme_unknown`

# Why this is said out loud, with the name quoted

Without it the operator sees none of the three radios selected and no
explanation, which reads as a rendering fault. And the likeliest cause is
benign and worth knowing: a settings file written by a **newer** pdfcer,
whose token this build is **preserving rather than overwriting**. Quoting
the name is what makes that legible — and telling them it is kept is what
stops them "fixing" it by picking one of the three, which would discard it.

### `fn ui_scale_title`

**"pdfcer's own"** does the work in this title. The word an operator is most
likely to arrive with is *"zoom"*, and zoom in this application means the
page — so the title has to draw the line before the operator has read a
word of the body, or they will set this expecting the document to change.

### `fn ui_scale_silence`

Says what it multiplies, because that is the fact that stops it being
misread as an override. An operator who has already set Windows display
scaling to 150 % needs to know this stacks on top rather than replacing it.

### `fn ui_scale_radius`

Two disclosures in one line, and both are needed. It takes effect
immediately — the exception to the whole window's draft-until-Save contract
that this setting shares with the theme — and it does **not** resize the
page, which is the thing an operator will most reasonably expect it to do
given that the word "size" is in the title.

### `fn ui_scale_percent`

A percentage rather than the stored multiplier, because *"125 %"* is a
quantity an operator can hold against the Windows display setting they
already know, and *"1.25"* is one they have to interpret. Same value, and
the unit is doing the explaining.

Rounded to whole percent: the step is 0.05, so every value the control can
produce is a whole number of percent and no precision is lost. A decimal
place would show `100.0 %` and imply a fineness the control does not have.

### `fn ui_scale_note`

Names the two failure modes rather than recommending a number, as the
zoom-settle note does — and for a stronger version of the same reason:
which value is right depends on the operator's eyes and their monitor, so
there is no number to recommend. What can be said is what going too far in
each direction looks like, and an operator who knows that can find their
value in two drags.

### `fn quality_silence`

Nothing, and it says so. This is a **preference**, a trade between sharpness
and speed that depends on the machine and on how big the drawings are — not
a question the standard leaves unanswered. Saying "the standard does not
define…" here would be inventing a clause to fit a template, which is the
dishonest version of consistency.

### `fn quality_note`

Each names **what it costs**, not just what it does — which is the whole
content of the choice. "Faster" without "softer" is half a sentence, and it
is the half that makes the setting look free.

### `fn settle_suffix`

A catalog entry rather than a literal, for the reason the degree sign and
the point abbreviation are: the ui-strings gate looks for exactly this, and
a translator has to be able to see that a unit exists.

### `fn settle_note`

Names both failure modes rather than recommending a number, because which
one bites depends on the machine — and an operator who knows what going too
far in each direction looks like can find their own value in two tries.

### `fn opening_fit_silence`

As the two settings above it — nothing. The PDF standard has an opinion
about page *size*; it has none about how a viewer chooses to fit that size
to a window, which is why this is a preference rather than an ambiguity.

### `fn opening_fit_note`

Each names what it costs on a **large sheet**, because that is the case
where they differ and it is the case this shell exists for. On a letter page
at a normal window size all three look much the same, and copy written
against that case would tell the operator nothing about the choice they are
actually making.

### `fn wheel_paging_radius`

The second sentence is the one that matters. Under a continuous display
mode the wheel scrolls the whole document by definition, so this setting
has nothing to change — and an operator who tried it there and saw no
difference would reasonably conclude it was broken.

### `fn wheel_paging_note`

The first names the case where today's behaviour is a **dead control**,
which is the whole reason the choice exists: this shell opens documents at
fit page, and a page that already fits has nothing to scroll.

### `fn paste_chords_title`

It names the SUBJECT, not the keys. An operator scanning the pane for
*"why did my copied field come out linked?"* is thinking about fields, not
about `V`.

### `fn paste_chords_radius`

Three things, and the second is the one that stops the support question.
Both pastes always exist — this only exchanges the keys — so an operator who
picks the Acrobat order has lost nothing and can still reach either from the
Edit tab. The third sentence forestalls the other reasonable worry: it is a
keyboard preference, not a document one, so nothing already pasted changes.

### `fn paste_chords_label`

The pdfcer entry does not say *"pdfcer's default"* the way `wheel_paging`'s
does, because here the alternative is named after a **product** and the pair
would read as an endorsement contest. Each names what its Ctrl+V does, which
is the fact being chosen between.

### `fn paste_chords_note`

Both notes lead with the CONSEQUENCE — whether typing in one box shows
in the other — because that is the only difference an operator can observe,
and it is invisible on the page. Two linked boxes and two independent boxes
are pixel-identical until somebody types.

The Acrobat note says *why* Acrobat does it, rather than only that it does.
An operator picking a compatibility setting deserves to know it is a real
convention with a purpose — repeated page-number and date fields that must
agree — and not merely a quirk being mimicked.

### `fn chrome_silence`

Names the thing an operator is most likely to have come here about — that
these are switches they have to flick on every single document — because the
group headings are how a symptom finds its setting and this is the symptom.

### `fn chrome_rulers_note`

It states the cost, and the cost is real rather than rhetorical: the
gutters come off the drawing area, on every document, for as long as the
preference is set. `ViewState::default`'s own comment calls this *"the one
default that has a measurable cost"*, which is why it ships off — and an
operator turning it on permanently deserves to be told what they are
spending.

### `fn chrome_guides_note`

**The second sentence is the whole reason this control has notes at all.**
`canvas::guides::ruler_drag` registers nothing when the rulers are hidden,
so an operator who switches guides on and cannot place one has met a
coupling the program never told them about. Saying it here costs one line
and saves the conclusion that the feature is broken.

### `fn chrome_guides_bound`

A [`crate::dialogs::settings::widgets::disclosure`] rather than a note under
the guides switch, because it is true **whichever way that switch is set** —
which is exactly the distinction that widget documents, and the same reason
the replacement-text bound is one.

It exists because the alternative is silent surprise in the honest
direction: an operator who sets this off will still see guides appear on the
documents they placed guides on, and with nothing said that reads as the
preference not working.

### `fn page_cache_silence`

Nothing about the standard, and it says so — the same honesty
[`quality_silence`] applies. What it says instead is the **symptom**, because
that is how an operator finds this control: they came here because scrolling
back to a sheet made them wait.

### `fn page_cache_label`

The megabyte figure is **computed from the budget**, never written beside
it. Two spellings of one quantity drift, and the drift here would be a
settings window promising 512 MB while the cache spent 2 GB —
`NO_SURFACE.md` §1's finding with a number instead of a colour.

"Large" is not something anybody can budget against. An operator with 8 GB
and one with 64 GB are making different decisions and neither can make theirs
from an adjective.

### `fn page_cache_note`

Each says **how much work it saves**, in sheets rather than in bytes, because
a drawing set is what this operator has and "25 sheets" is a thing he can
picture where "1 GB" is not.

### `fn mesh_padding_title`

Filed by the SYMPTOM, not the mechanism. Nobody goes looking for
*"type 6/7 mesh shading patch record byte alignment"*. Somebody whose
gradient came out as garbage goes looking for *gradient*, so that is the
first word.

### `fn preset_silence`

States the two facts an operator needs before clicking something that
changes several settings at once: what it will do, and that it is not a
lock. The second is the one that makes it safe to try.

### `fn preset_pdfcer_note`

Worded for the operator who has been experimenting and wants out. That is
the reported use — *"touching some of our presets caused some test to show
up as failed"* — and it is a person looking for a way back, not a person
choosing a philosophy.

### `fn preset_same_as_others`

It exists because the operator asked for the control in order to *"see how
far we are along with matching the [conformance suite's] tests"*, and
switching to PDF/X-4
will change nothing on screen. Finding that out by comparing two identical
renders costs an hour and reads as the setting being broken.

It says **why**, and the why is the part that stops it sounding like a
bug: the standards differ in what they demand of a *file* — fonts embedded,
an output intent present, transparency allowed or not — and those are
preflight questions. What they ask of a **renderer** is the same, so pdfcer
giving them the same answers is agreement rather than laziness.

The number is counted at the moment of drawing, so if a standard's answers
ever diverge this sentence corrects itself. See
`crate::dialogs::settings::preset`'s `identical_siblings`.

### `fn preset_leaves_alone`

Named rather than left blank. Roughly a third of the grid is axes a
standard does not reach — no PDF/X part contains a shading clause at all —
and a blank cell reads as missing data, while a value would assert a
requirement that does not exist.

### `fn preset_weight`

The sentence that stops this feature being a dropdown. The engine grades
every value it supplies, and its own framing is that *the interesting column
is not the value, it is how much weight the value can bear.* For PDF/X-4,
exactly one of six answers is a claim about the standard at all.

Worded so the weakest category is unmistakable. "pdfcer chose" is not a
hedge — it is the honest description of an axis the standard is silent on,
and an operator reading it should understand that switching standards will
not change that answer for a reason the standard requires.

### `fn auto_hide_title`

The radius line carries the fact that decides whether an operator dares
turn this on: **the drawing does not move.** Every program in the class that
gets this wrong reflows the document as the strip comes and goes, and an
operator who has met that once will not try it again. Saying it here is what
makes the setting choosable.

The title says *"getting out of the way"* rather than *"auto-hide"*,
because the operator is looking for room on their drawing, not for a feature
name.

### `fn point_suffix`

A catalog entry rather than a literal, for the reason the millisecond and
degree suffixes are: the ui-strings gate looks for exactly this, and a
translator has to be able to see that a unit exists.
