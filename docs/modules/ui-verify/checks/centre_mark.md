# `centre_mark` — a circle gets a centre mark, with or without its measurement

Drives Measure ▸ Radius/diameter on `fixtures/hole-in-a-big-object.pdf`
(the r=30 hole at (306, 500)).

1. Ticks *Add centre mark*, unticks *Add measurement*; each press must trace
   `measure-circular-outputs` with the wanted pair.
2. Picks three rim points, the last by double-click. The next
   `dimension-added` must carry `shape=centre-mark`, with no placing click.
3. Ticks *Add measurement* again and repeats the picks. Nothing may commit
   before the placing click; after it, `shape=circular+mark`.

Falsified by disabling the direct commit of the bare mark: step 2 then
reports `mark alone committed: None` and the check fails.
