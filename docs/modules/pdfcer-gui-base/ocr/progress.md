# `ocr::progress` — **what the recogniser is doing, and the two ways to end
it early**


> *"can you make it so the recognizing ocr gives feedback on what it is
> doing when it is running (pages done, words/characters detected, etc) so
> that the user can see that it is doing something and hasn't frozen on
> large documents? Maybe a cancel and stop button too. The cancel throws
> away what was done, and the stop finished the page it is on and keeps the
> work it has done."*

Three things in one sentence, and the third is the one with a sharp edge.

## Cancel and Stop are DIFFERENT, and the difference is the whole design

| | the current page | what survives |
|---|---|---|
| **Cancel** | abandoned | **nothing** — the document is untouched |
| **Stop** | **finished** | every page recognised so far, offered for review as usual |

They are not two names for one act and must never collapse into one. An
operator who presses Stop on page 40 of 200 has *asked for* those forty
pages; one who presses Cancel has asked for none of them. Getting that
backwards either throws away twenty minutes of work or writes a partial
layer somebody did not want.

**Stop finishes the page it is on**, which is his wording and is also the
only coherent reading: a page is recognised as a unit — rendered, run
through the model, converted to page space — and half of one is not a thing
that can be kept. The wait is bounded by one page, which on a scanned sheet
is a second or two.

## Why a flag and not a channel message

The worker is a plain loop over pages; it does not select on anything. A
shared flag it reads at the top of each iteration costs one atomic load per
page and needs no runtime, no timeout and no second thread to deliver it.

The flag is checked **between** pages and never inside one, which is what
makes "Stop keeps the finished pages" true by construction rather than by
care: there is no point in the loop where a half-recognised page exists.

## Why progress is a channel message and not a shared counter

A counter would need the UI to poll a lock, and — more importantly — a
counter cannot carry *what was found*. The operator asked for words and
characters, which are per-page facts the worker computes and then folds
into a total; publishing them as they happen is free, and reconstructing
them from a shared number afterwards is impossible.

It also means the UI thread never blocks on the worker: `try_recv` in a
loop, drain what is there, draw what arrived.

## Item notes

### `fn from_u8`

An unrecognised value answers `Continue`, which is the safe direction:
the failure mode of a corrupt read is a job that keeps going and can be
asked again, not one that silently discards an operator's work.

### `fn cancel_wins_whichever_order_the_two_arrive_in`

Two adjacent buttons and a run the operator has decided against: the
order the clicks land in must not decide whether a partial layer is
written. Abandonment wins in both orders.

### `enum Wish`

One atomic rather than two booleans, because the states are **ordered** and
mutually exclusive: a job cannot be both cancelled and stopped, and a Cancel
arriving after a Stop must win. An enum in a `u8` makes that a single
compare-and-set instead of two loads whose order a reader has to reason
about.

### `fn wish`

`Relaxed` is correct and deliberate. There is no other memory being
published alongside this flag — the results travel by channel, which
carries its own ordering — so the only requirement is that the value
eventually arrives, and a page of OCR is several orders of magnitude
longer than any plausible propagation delay.

### `fn stop`

Refuses to downgrade a Cancel. An operator who cancelled and then hit
Stop — two clicks in the same second on adjacent buttons — must not have
the abandonment quietly turned into a partial write.

### `fn cancel`

Unconditional: Cancel outranks Stop, because it is the one that cannot
be undone by waiting and because it is what an operator reaches for when
they have realised the whole run was a mistake.

### `enum Update`

`Page` is sent **after** the page is recognised, carrying that page's own
counts. The dialog accumulates; the worker does not send running totals,
because a message that is a total rather than an event cannot be dropped
safely and this channel is allowed to be drained in batches.

### `enum Outcome`

`Stopped` is a distinct outcome and NOT a successful run with fewer
pages. The disclosure has to say the run ended early, or an operator who
stopped at page 40 of 200 is left believing the whole document was
recognised — which they will discover months later, searching for a word
that is on page 150 and is not in the layer.
