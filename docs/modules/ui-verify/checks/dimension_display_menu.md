# `a_radius_switches_to_a_diameter_from_its_right_click_menu`

**Subject.** A circular ce dimension's right-click menu offers the measure it
does not currently show, and picking it redraws the dimension. The menu then
offers the way back.

**Gesture.**

1. In Review, arm Measure ▸ Radius / Diameter and pick three rim points of a
   circle on the blank half of `blank-overhang.pdf`. Finish.
2. Select it by clicking its leader at 0°.
3. Right-click the same point. The `canvas.dimension` menu must declare
   `format.dimension_diameter` and must not declare `format.dimension_radius`.
4. Click the row, then assert `dimension-display … diameter=1` follows the mark.
5. Repeat with the pair swapped, expecting `diameter=0`.

**Pixel oracle.** A thin box across the leader at 180°, half-way between the
centre and the rim. A radius leader runs from the centre to 0°, so the box is
blank. A diameter leader crosses the circle through it. The box is blank
before the switch, inked after it, and blank again after the switch back. A
non-blank box before the switch is reported as SKIPPED, because ink there
afterwards would then prove nothing.

**Control.** A build without the `canvas.dimension` menu fails at step 3 with
no rows at all: the right-click there opens the view menu.
