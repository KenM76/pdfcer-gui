"""Build the two walking figures (human, werewolf) for the 3D viewer.

Run headless from any directory:

    "C:\\Program Files\\Blender Foundation\\Blender 5.2\\blender.exe" --background
        --factory-startup --python tools/figures/make_figures.py --
        [--figures DIR] [--previews DIR] [--no-render]

--figures   where human.figure / werewolf.figure go
            (default: crates/pdfcer-gui/src/dialogs/model3d/figures beside this repo)
--previews  where the preview PNGs go (default D:\\scratch\\avatar): rest pose and
            mid-stride, each from front three-quarter and side, plus a back view
--no-render write the .figure files only

Reproducible: every shape is built procedurally with bmesh from the numbers in
this file, and coordinates are written rounded to 0.1 mm, so the same script
gives the same bytes.

.figure format (UTF-8, LF, no BOM; metres, Z up, facing +Y, soles at z=0):
    figure <name> eye <eye height>
    part <name> <parent|-> <pivot x y z>     pivot is absolute, in figure space
    v <x> <y> <z>                            relative to this part's pivot
    f <i> <j> <k> <r> <g> <b>                0-based into this part's v list,
                                             CCW seen from outside, flat colour
Every v/f belongs to the latest part line. The viewer animates by rotating
each rigid part about its pivot; parts overlap a little at every joint so a
~35 degree bend does not open a gap.
"""

import math
import os
import sys

import bmesh
import bpy
from mathutils import Matrix, Vector

TRI_BUDGET = 1500
X = Vector((1.0, 0.0, 0.0))


# ---------------------------------------------------------------- geometry --

class Part:
    def __init__(self, name, parent, pivot):
        self.name, self.parent, self.pivot = name, parent, Vector(pivot)
        self.verts = []      # absolute figure-space Vectors
        self.faces = []      # (i, j, k, (r, g, b))
        self.prims = 0
        self.centroid_misses = 0


def _ring_axis(cs, i):
    a = cs[max(i - 1, 0)]
    b = cs[min(i + 1, len(cs) - 1)]
    d = b - a
    return d.normalized()


def _ring_verts(bm, c, axis, rx, ry, n, rot):
    if rx == 0.0 and ry == 0.0:
        return [bm.verts.new(c)]
    u = X - axis * axis.dot(X)
    u.normalize()
    v = axis.cross(u)
    out = []
    for k in range(n):
        a = rot + 2.0 * math.pi * k / n
        out.append(bm.verts.new(c + u * (rx * math.cos(a)) + v * (ry * math.sin(a))))
    return out


def _bridge(bm, layer, a, b, band):
    if len(a) == 1 and len(b) == 1:
        raise ValueError("two apexes in a row")
    n = max(len(a), len(b))
    for k in range(n):
        k2 = (k + 1) % n
        if len(a) == 1:
            vs = [a[0], b[k], b[k2]]
        elif len(b) == 1:
            vs = [a[k], a[k2], b[0]]
        else:
            vs = [a[k], a[k2], b[k2], b[k]]
        bm.faces.new(vs)[layer] = band


def loft(part, rings, n, colour, rot=None):
    """Closed tube through rings (cx, cy, cz, rx, ry); rx=ry=0 is an apex.

    Each ring lies across the path; rx runs along the figure's X (projected),
    ry along axis x X. colour is an (r,g,b) or fn(band, centre, normal), where
    band is the ring gap index, -1 the first cap and -2 the last cap.
    """
    if rot is None:
        rot = math.pi / n if n % 2 == 0 and n != 6 else 0.0
    bm = bmesh.new()
    layer = bm.faces.layers.int.new("band")
    cs = [Vector(r[:3]) for r in rings]
    loops = [_ring_verts(bm, cs[i], _ring_axis(cs, i), r[3], r[4], n, rot)
             for i, r in enumerate(rings)]
    for i in range(len(loops) - 1):
        _bridge(bm, layer, loops[i], loops[i + 1], i)
    if len(loops[0]) > 1:
        bm.faces.new(list(reversed(loops[0])))[layer] = -1
    if len(loops[-1]) > 1:
        bm.faces.new(loops[-1])[layer] = -2
    _emit(part, bm, layer, colour)


def slab(part, rings, colour):
    """Four-sided loft whose rings are true rectangles of half-size rx, ry."""
    r2 = math.sqrt(2.0)
    loft(part, [(x, y, z, a * r2, b * r2) for x, y, z, a, b in rings], 4, colour,
         rot=math.pi / 4)


def box(part, centre, size, colour):
    """Axis-aligned box from bmesh's cube primitive."""
    bm = bmesh.new()
    layer = bm.faces.layers.int.new("band")
    m = Matrix.Translation(Vector(centre)) @ Matrix.Diagonal((*size, 1.0))
    bmesh.ops.create_cube(bm, size=1.0, matrix=m)
    _emit(part, bm, layer, colour)


def cone(part, base, tip, r, colour, n=3):
    """Spike / claw / tuft: n-sided cone from a base disc to a tip point."""
    loft(part, [(*base, r, r), (*tip, 0.0, 0.0)], n, colour, rot=0.0)


def _signed_volume(bm):
    vol = 0.0
    for f in bm.faces:
        a, b, c = (v.co for v in f.verts)
        vol += a.dot(b.cross(c))
    return vol / 6.0


def _emit(part, bm, layer, colour):
    """Triangulate, orient outward, verify, and append one closed primitive."""
    bmesh.ops.triangulate(bm, faces=bm.faces[:], quad_method='BEAUTY',
                          ngon_method='BEAUTY')
    bmesh.ops.recalc_face_normals(bm, faces=bm.faces[:])
    if _signed_volume(bm) < 0.0:
        bmesh.ops.reverse_faces(bm, faces=bm.faces[:])
    bm.normal_update()
    if not all(e.is_manifold for e in bm.edges):
        raise RuntimeError(f"{part.name}: primitive is not closed")
    if _signed_volume(bm) <= 0.0:
        raise RuntimeError(f"{part.name}: primitive faces inward")
    centre = sum((v.co for v in bm.verts), Vector()) / len(bm.verts)
    base = len(part.verts)
    for v in bm.verts:
        v.index = len(part.verts) - base
        part.verts.append(v.co.copy())
    for f in bm.faces:
        fc = f.calc_center_median()
        if f.normal.dot(fc - centre) <= 0.0:
            part.centroid_misses += 1
        rgb = colour(f[layer], fc, f.normal) if callable(colour) else colour
        i, j, k = (base + v.index for v in f.verts)
        part.faces.append((i, j, k, tuple(rgb)))
    part.prims += 1
    bm.free()


def limb(part, rings, n, colour, end=True):
    """Loft from a joint: rounds the start into a ball about rings[0] (the
    pivot) and, if end, plugs a short stub past the last ring so the next
    part's ball swallows it. Bands keep the indices of the given rings."""
    p0, p1 = Vector(rings[0][:3]), Vector(rings[1][:3])
    d0 = (p1 - p0).normalized()
    r0 = rings[0]
    c0 = p0 - d0 * (0.6 * max(r0[3], r0[4]))
    out = [(*c0, .7 * r0[3], .7 * r0[4])] + list(rings)
    if end:
        pl, pk = Vector(rings[-1][:3]), Vector(rings[-2][:3])
        rl = rings[-1]
        cl = pl + (pl - pk).normalized() * (0.35 * max(rl[3], rl[4]))
        out.append((*cl, .75 * rl[3], .75 * rl[4]))
    fn = colour if callable(colour) else (lambda b, c, nn: colour)
    loft(part, out, n, lambda b, c, nn: fn(max(b - 1, 0) if b >= 0 else b, c, nn))


def _bands(table, caps):
    def fn(band, c, n):
        if band < 0:
            return caps[band]
        return table[min(band, len(table) - 1)]
    return fn


# ------------------------------------------------------------------ human --

SKIN = (230, 176, 134)
SKIN_D = (196, 140, 102)
HAIR = (104, 60, 28)
TUNIC = (40, 78, 160)
TUNIC_D = (26, 50, 112)
TABARD = (176, 34, 34)
TABARD_D = (128, 22, 24)
GOLD = (232, 186, 64)
GOLD_D = (170, 126, 38)
STEEL = (170, 176, 186)
BELT = (90, 54, 26)
TROUS = (112, 90, 64)
BOOT = (74, 46, 26)
BOOT_L = (128, 84, 46)
SOLE = (40, 26, 16)
EYE_H = (30, 40, 80)


def human_core(parts):
    hips, torso, head = parts["hips"], parts["torso"], parts["head"]
    loft(hips, [(0, 0, .84, .175, .125), (0, 0, .92, .2, .135),
                (0, 0, .985, .215, .145), (0, 0, 1.065, .21, .14)], 8,
         _bands([TROUS, TROUS, BELT], {-1: TROUS, -2: BELT}))
    box(hips, (0, .14, 1.025), (.10, .04, .075), GOLD)
    box(hips, (0, .155, 1.025), (.05, .02, .04), GOLD_D)
    slab(hips, [(0, .125, 1.0, .09, .022), (0, .145, .78, .085, .02)],
         lambda b, c, n: TABARD if n.y > .6 else GOLD)

    def tunic(band, c, n):
        return TUNIC_D if band in (-1, -2, 0) or n.z > .7 else TUNIC
    loft(torso, [(0, 0, 1.03, .19, .13), (0, 0, 1.16, .215, .14),
                 (0, .01, 1.32, .29, .18), (0, .005, 1.45, .305, .175),
                 (0, 0, 1.545, .13, .095)], 8, tunic)
    slab(torso, [(0, .15, 1.46, .115, .025), (0, .155, 1.3, .105, .025),
                 (0, .12, 1.05, .1, .025)],
         lambda b, c, n: (TABARD if b == 1 else TABARD_D) if n.y > .6 else GOLD)
    box(torso, (0, .178, 1.34), (.07, .02, .09), GOLD)

    loft(head, [(0, 0, 1.46, .068, .068), (0, 0, 1.60, .068, .068)], 6, SKIN_D)

    def face(band, c, n):
        if band == -1:
            return SKIN_D
        if band == 0:
            return HAIR if n.y < -.2 else SKIN
        if band == 1:
            return HAIR if n.y < -.3 else SKIN
        if band == 2:
            return HAIR if n.y < .5 or c.z > 1.745 else SKIN
        return HAIR
    loft(head, [(0, .02, 1.545, .085, .08), (0, .012, 1.615, .135, .128),
                (0, .003, 1.70, .148, .145), (0, -.006, 1.765, .128, .13),
                (0, -.01, 1.80, 0, 0)], 8, face)
    for s in (-1, 1):
        box(head, (s * .052, .13, 1.69), (.04, .016, .026), EYE_H)
        box(head, (s * .052, .128, 1.718), (.06, .02, .014), HAIR)
        cone(head, (s * .135, -.005, 1.675), (s * .172, -.018, 1.70), .032,
             SKIN_D, n=4)
    cone(head, (0, .12, 1.69), (0, .165, 1.66), .025, SKIN_D, n=4)
    slab(head, [(0, .085, 1.615, .075, .04), (0, .1, 1.575, .05, .03)], HAIR)


def human_side(parts, sd):
    s = -1 if sd == "l" else 1
    ua, fa, hd = parts["upper_arm_" + sd], parts["fore_arm_" + sd], parts["hand_" + sd]
    th, sh, ft = parts["thigh_" + sd], parts["shin_" + sd], parts["foot_" + sd]
    loft(ua, [(s * .3, 0, 1.36, .12, .12), (s * .315, 0, 1.44, .175, .165),
              (s * .31, 0, 1.51, .135, .125), (s * .295, 0, 1.56, 0, 0)], 8,
         _bands([GOLD_D, GOLD, GOLD], {-1: GOLD_D, -2: GOLD}))
    cone(ua, (s * .34, 0, 1.52), (s * .38, -.01, 1.60), .04, STEEL)
    limb(ua, [(s * .29, 0, 1.43, .09, .09), (s * .31, 0, 1.29, .085, .085),
              (s * .33, 0, 1.15, .075, .075)], 6,
         _bands([TUNIC, SKIN], {-1: TUNIC, -2: SKIN}))
    limb(fa, [(s * .33, 0, 1.15, .08, .08), (s * .335, .005, 1.07, .085, .085),
              (s * .34, .01, 1.05, .1, .1), (s * .35, .02, .91, .09, .09),
              (s * .35, .02, .89, .07, .07)], 6,
         _bands([SKIN, BELT, BELT, BELT], {-1: SKIN, -2: BELT}))
    limb(hd, [(s * .35, .02, .89, .06, .065), (s * .355, .025, .84, .06, .088),
              (s * .355, .03, .75, .055, .08), (s * .355, .035, .715, .03, .05)], 6,
         SKIN, end=False)
    cone(hd, (s * .33, .07, .85), (s * .32, .105, .78), .024, SKIN_D, n=4)
    limb(th, [(s * .12, 0, .92, .13, .13), (s * .125, 0, .74, .118, .118),
              (s * .125, 0, .51, .095, .095)], 6, TROUS)
    limb(sh, [(s * .125, 0, .51, .1, .1), (s * .125, 0, .43, .09, .09),
              (s * .125, 0, .42, .125, .125), (s * .125, -.004, .32, .118, .118),
              (s * .125, -.006, .30, .09, .09), (s * .125, -.01, .10, .08, .08)], 6,
         _bands([TROUS, BOOT_L, BOOT_L, BOOT, BOOT], {-1: TROUS, -2: BOOT}))

    def boot(band, c, n):
        return SOLE if n.z < -.7 else (BOOT_L if band == 3 else BOOT)
    slab(ft, [(s * .125, -.11, .07, .07, .07), (s * .125, -.06, .085, .08, .085),
              (s * .125, .09, .075, .085, .075), (s * .125, .24, .045, .07, .045)],
         boot)


# --------------------------------------------------------------- werewolf --

FUR_D = (86, 74, 64)
FUR = (124, 108, 92)
FUR_M = (158, 140, 116)
FUR_L = (214, 198, 168)
PAW = (58, 50, 46)
CLAW = (28, 24, 22)
NOSE = (22, 18, 18)
EYE_W = (255, 206, 30)
EAR_IN = (150, 98, 92)
MOUTH = (120, 36, 40)
TEETH = (240, 234, 214)


def fur(band, c, n):
    if n.z > .5 or n.y < -.45:
        return FUR_D
    return FUR_M if n.y > .45 else FUR


def shard(part, base, tip, r=.045, colour=FUR_D):
    cone(part, base, tip, r, colour)


def wolf_core(parts):
    hips, torso, head, tail = (parts[k] for k in ("hips", "torso", "head", "tail"))
    loft(hips, [(0, -.06, .92, .15, .13), (0, -.06, 1.0, .2, .165),
                (0, -.05, 1.1, .19, .155), (0, -.03, 1.21, .15, .13)], 8,
         lambda b, c, n: FUR_D if n.z > .5 or n.y < -.45 else FUR)

    def chest(band, c, n):
        if n.y > .3 and n.z < .55 and 1 <= band <= 5 and abs(c.x) < .22:
            return FUR_L
        return fur(band, c, n)
    loft(torso, [(0, -.05, 1.08, .17, .14), (0, -.02, 1.2, .145, .125),
                 (0, .07, 1.35, .23, .22), (0, .15, 1.47, .29, .265),
                 (0, .24, 1.57, .3, .225), (0, .33, 1.65, .2, .175),
                 (0, .42, 1.71, .15, .145), (0, .49, 1.75, .115, .115)], 8, chest)
    wolf_mane(torso)
    loft(tail, [(0, -.14, 1.07, .055, .055), (0, -.25, 1.05, .1, .09),
                (0, -.41, .96, .13, .12), (0, -.55, .80, .12, .11),
                (0, -.63, .63, .085, .08), (0, -.66, .50, 0, 0)], 8,
         lambda b, c, n: FUR_L if b >= 3 or b == -2 else fur(b, c, n))
    for base, tip in (((0, -.3, 1.1), (0, -.44, 1.13)), ((0, -.45, 1.03), (0, -.6, .98)),
                      ((.09, -.38, .99), (.2, -.5, .96)), ((-.09, -.38, .99), (-.2, -.5, .96)),
                      ((.08, -.53, .82), (.18, -.66, .76)), ((-.08, -.53, .82), (-.18, -.66, .76))):
        shard(tail, base, tip, .05, FUR)
    wolf_head(head)


def wolf_mane(torso):
    """Shards round the neck base and shoulders, pointing out, back and up."""
    for row, (cy, cz, rad, lift) in enumerate(((.32, 1.64, .19, .12), (.24, 1.55, .26, .06))):
        for k in range(9):
            t = math.radians(-125 + 250 * k / 8)
            sx, cy_ = math.sin(t), math.cos(t)
            base = (.85 * rad * sx, cy - .14 * cy_, cz + .03 * cy_)
            tip = (base[0] + .15 * sx, base[1] - .12 * cy_ - .09, base[2] + lift + .05 * cy_)
            shard(torso, base, tip, .055, FUR_D if (k + row) % 2 else FUR)
    for dx in (-.07, .07):
        shard(torso, (dx, .43, 1.66), (dx * 1.4, .5, 1.5), .06, FUR_L)
        shard(torso, (dx * 2.2, .38, 1.66), (dx * 2.8, .45, 1.52), .06, FUR_M)


def wolf_head(head):
    def skull(band, c, n):
        return FUR_D if n.z > .35 else (FUR_L if n.z < -.5 else FUR)
    loft(head, [(0, .27, 1.88, 0, 0), (0, .33, 1.91, .13, .12),
                (0, .43, 1.92, .15, .13), (0, .52, 1.9, .12, .11),
                (0, .565, 1.865, .075, .07)], 8, skull)
    box(head, (0, .525, 1.968), (.2, .07, .04), FUR_D)
    loft(head, [(0, .5, 1.885, .085, .07), (0, .64, 1.87, .065, .052),
                (0, .76, 1.86, .042, .036)], 6,
         lambda b, c, n: FUR if n.z > .5 else FUR_L)
    box(head, (0, .762, 1.875), (.06, .04, .042), NOSE)
    loft(head, [(0, .48, 1.825, .072, .03), (0, .66, 1.755, .042, .02)], 4,
         FUR_L, rot=math.pi / 4)
    loft(head, [(0, .49, 1.835, .055, .028), (0, .68, 1.79, .03, .014)], 4,
         MOUTH, rot=math.pi / 4)
    for s in (-1, 1):
        cone(head, (s * .033, .70, 1.838), (s * .035, .712, 1.77), .012, TEETH)
        cone(head, (s * .032, .615, 1.77), (s * .034, .62, 1.825), .01, TEETH)
        box(head, (s * .07, .55, 1.925), (.05, .04, .022), EYE_W)
        loft(head, [(s * .085, .4, 1.985, .055, .028), (s * .11, .33, 2.08, .04, .02),
                    (s * .13, .25, 2.2, 0, 0)], 3,
             lambda b, c, n: EAR_IN if n.y > .3 else FUR_D)
        shard(head, (s * .1, .46, 1.87), (s * .21, .37, 1.82), .045, FUR)
        shard(head, (s * .09, .42, 1.82), (s * .19, .33, 1.73), .045, FUR_L)
        shard(head, (s * .1, .36, 1.9), (s * .2, .26, 1.9), .045, FUR_D)


def wolf_side(parts, sd):
    s = -1 if sd == "l" else 1
    ua, fa, hd = parts["upper_arm_" + sd], parts["fore_arm_" + sd], parts["hand_" + sd]
    limb(ua, [(s * .36, .17, 1.62, .13, .12), (s * .4, .15, 1.42, .1, .095),
              (s * .42, .12, 1.26, .075, .075)], 6, fur)
    limb(fa, [(s * .42, .12, 1.26, .085, .085), (s * .44, .17, 1.08, .105, .095),
              (s * .455, .23, .92, .068, .068)], 6, fur)
    for base, tip in (((s * .43, .1, 1.19), (s * .45, -.05, 1.12)),
                      ((s * .445, .15, 1.04), (s * .47, .0, .97)),
                      ((s * .5, .15, 1.13), (s * .59, .09, 1.05))):
        shard(fa, base, tip, .04, FUR_D)
    limb(hd, [(s * .455, .23, .92, .072, .072), (s * .46, .25, .84, .07, .105),
              (s * .465, .28, .75, .06, .095)], 6,
         lambda b, c, n: FUR_D if b == 0 else PAW, end=False)
    for dy in (-.06, -.02, .02, .06):
        cone(hd, (s * .465, .28 + dy, .77), (s * .47, .29 + dy, .665), .028, PAW, n=4)
        cone(hd, (s * .47, .29 + dy, .69), (s * .475, .325 + dy, .58), .015, CLAW)
    cone(hd, (s * .43, .3, .83), (s * .41, .37, .76), .02, CLAW)
    wolf_leg(parts, sd, s)


def wolf_leg(parts, sd, s):
    th, sh, ft = parts["thigh_" + sd], parts["shin_" + sd], parts["foot_" + sd]
    x = s * .17
    hip, knee, hock, ball = (Vector((s * p[0], p[1], p[2]))
                             for p in (WOLF_HIP, WOLF_KNEE, WOLF_HOCK, WOLF_BALL))
    mid = lambda a, b, t=.5: tuple(a.lerp(b, t))
    limb(th, [(*hip, .135, .135), (*mid(hip, knee, .45), .125, .12),
              (*knee, .088, .088)], 6, fur)
    limb(sh, [(*knee, .095, .095), (*mid(knee, hock, .45), .07, .078),
              (*hock, .052, .052)], 6, fur)
    back = Vector((0, -(knee - hock).z, (knee - hock).y)).normalized()
    for t, out in ((.25, 0), (.5, 0), (.38, .5)):
        b = knee.lerp(hock, t) + Vector((s * .03 * out, 0, 0))
        shard(sh, tuple(b), tuple(b + back * .15 + Vector((s * .1 * out, 0, -.03))),
              .04, FUR_D)
    limb(ft, [(*hock, .058, .058), (*mid(hock, ball), .05, .05),
              (*ball, .05, .055)], 6,
         lambda b, c, n: FUR_D if b == 0 else PAW, end=False)
    x, y = ball.x, ball.y
    slab(ft, [(x, y - .06, .055, .06, .055), (x, y + .03, .055, .075, .055),
              (x, y + .11, .04, .07, .04)], PAW)
    for dx in (-.05, -.017, .017, .05):
        cone(ft, (x + dx, y + .08, .038), (x + dx, y + .155, .024), .03, PAW, n=4)
        cone(ft, (x + dx, y + .14, .026), (x + dx, y + .205, 0.0), .015, CLAW)


# ---------------------------------------------------------------- figures --

def skeleton(spec):
    return {name: Part(name, parent, pivot) for name, parent, pivot in spec}


def _limbs(hip, knee, ankle, sh, el, wr):
    out = []
    for sd, s in (("l", -1), ("r", 1)):
        out += [("upper_arm_" + sd, "torso", (s * sh[0], sh[1], sh[2])),
                ("fore_arm_" + sd, "upper_arm_" + sd, (s * el[0], el[1], el[2])),
                ("hand_" + sd, "fore_arm_" + sd, (s * wr[0], wr[1], wr[2]))]
    for sd, s in (("l", -1), ("r", 1)):
        out += [("thigh_" + sd, "hips", (s * hip[0], hip[1], hip[2])),
                ("shin_" + sd, "thigh_" + sd, (s * knee[0], knee[1], knee[2])),
                ("foot_" + sd, "shin_" + sd, (s * ankle[0], ankle[1], ankle[2]))]
    return out


def build_human():
    spec = [("hips", "-", (0, 0, .98)), ("torso", "hips", (0, 0, 1.06)),
            ("head", "torso", (0, 0, 1.54))]
    spec += _limbs((.12, 0, .92), (.125, 0, .51), (.125, -.01, .10),
                   (.29, 0, 1.43), (.33, 0, 1.15), (.35, .02, .89))
    parts = skeleton(spec)
    human_core(parts)
    for sd in "lr":
        human_side(parts, sd)
    return "human", 1.69, parts


# Digitigrade leg, pivot to pivot: hip -> knee runs forward and down, knee ->
# hock back and down, hock -> paw ball forward and down; the paw is on its toes.
WOLF_HIP, WOLF_KNEE, WOLF_HOCK = (.15, -.05, 1.02), (.17, .2, .64), (.17, -.17, .34)
WOLF_BALL = (.17, .04, .08)
# The head and arms are modelled upright, then carried forward and down onto
# the stooped torso; the head is also scaled up about its pivot.
HEAD_MOVE, HEAD_SCALE = Vector((0, .13, -.08)), 1.1
ARM_MOVE = Vector((0, .09, -.04))


def place(part, d, k=1.0):
    part.verts = [part.pivot + (v - part.pivot) * k + d for v in part.verts]
    part.pivot = part.pivot + d


def build_werewolf():
    spec = [("hips", "-", (0, -.05, 1.1)), ("torso", "hips", (0, -.03, 1.18)),
            ("head", "torso", (0, .33, 1.81))]
    spec += _limbs(WOLF_HIP, WOLF_KNEE, WOLF_HOCK,
                   (.36, .17, 1.62), (.42, .12, 1.26), (.455, .23, .92))
    spec.append(("tail", "hips", (0, -.2, 1.04)))
    parts = skeleton(spec)
    wolf_core(parts)
    for sd in "lr":
        wolf_side(parts, sd)
    place(parts["head"], HEAD_MOVE, HEAD_SCALE)
    for sd in "lr":
        for name in ("upper_arm_", "fore_arm_", "hand_"):
            place(parts[name + sd], ARM_MOVE)
    eye = 1.81 + (1.925 - 1.81) * HEAD_SCALE + HEAD_MOVE.z
    return "werewolf", round(eye, 4), parts


# ----------------------------------------------------------------- output --

def num(x):
    s = f"{x:.4f}".rstrip("0").rstrip(".")
    return "0" if s in ("-0", "") else s


def check(name, parts):
    tris = sum(len(p.faces) for p in parts.values())
    zs = [v.z for p in parts.values() for v in p.verts]
    if tris > TRI_BUDGET:
        raise RuntimeError(f"{name}: {tris} triangles > {TRI_BUDGET}")
    if abs(min(zs)) > 5e-5:
        raise RuntimeError(f"{name}: lowest point {min(zs):.5f}, soles must sit at 0")
    misses = sum(p.centroid_misses for p in parts.values())
    print(f"[figure] {name}: {tris} tris, height {max(zs):.4f}, "
          f"{misses} faces fail the per-primitive centroid test (non-convex lofts)")
    for p in parts.values():
        print(f"[figure]   {p.name:12s} {p.parent:12s} pivot "
              f"{num(p.pivot.x)} {num(p.pivot.y)} {num(p.pivot.z)}  {len(p.faces)} tris")
    return tris


def write_figure(path, name, eye, parts):
    lines = [f"# {name}: rigid-part walking figure, made by tools/figures/make_figures.py",
             "# metres, Z up, facing +Y, soles at z=0; v relative to the part pivot",
             f"figure {name} eye {num(eye)}"]
    for p in parts.values():
        lines.append(f"part {p.name} {p.parent} {num(p.pivot.x)} {num(p.pivot.y)} "
                     f"{num(p.pivot.z)}")
        for v in p.verts:
            d = v - p.pivot
            lines.append(f"v {num(d.x)} {num(d.y)} {num(d.z)}")
        for i, j, k, (r, g, b) in p.faces:
            lines.append(f"f {i} {j} {k} {r} {g} {b}")
    with open(path, "w", encoding="utf-8", newline="\n") as fh:
        fh.write("\n".join(lines) + "\n")


# ---------------------------------------------------------------- preview --

def _mesh_object(name, verts, faces, parent=None, loc=(0, 0, 0)):
    me = bpy.data.meshes.new(name)
    me.from_pydata([tuple(v) for v in verts], [], [f[:3] for f in faces])
    me.update()
    attr = me.color_attributes.new("Col", 'BYTE_COLOR', 'CORNER')
    for poly, f in zip(me.polygons, faces):
        r, g, b = f[3]
        for li in poly.loop_indices:
            attr.data[li].color_srgb = (r / 255, g / 255, b / 255, 1.0)
    me.color_attributes.active_color = attr
    ob = bpy.data.objects.new(name, me)
    bpy.context.scene.collection.objects.link(ob)
    ob.parent = parent
    ob.location = loc
    return ob


def instantiate(parts, x_offset=0.0):
    obs = {}
    for p in parts.values():
        par = obs.get(p.parent)
        base = parts[p.parent].pivot if par else Vector((-x_offset, 0, 0))
        verts = [v - p.pivot for v in p.verts]
        obs[p.name] = _mesh_object(p.name, verts, p.faces, par, p.pivot - base)
    return obs


def _scene_setup():
    bpy.ops.wm.read_factory_settings(use_empty=True)
    sc = bpy.context.scene
    sc.render.engine = 'BLENDER_WORKBENCH'
    sc.display.shading.light = 'STUDIO'
    sc.display.shading.color_type = 'VERTEX'
    sc.display.shading.show_shadows = True
    sc.display.shading.show_cavity = False
    sc.view_settings.view_transform = 'Standard'
    sc.render.resolution_x, sc.render.resolution_y = 900, 1200
    sc.world = bpy.data.worlds.new("bg")
    sc.world.color = (0.30, 0.36, 0.44)
    g = [(-2, -2, 0), (2, -2, 0), (2, 2, 0), (-2, 2, 0)]
    _mesh_object("ground", g, [(0, 1, 2, (150, 150, 140)), (0, 2, 3, (150, 150, 140))])
    cam = bpy.data.objects.new("cam", bpy.data.cameras.new("cam"))
    sc.collection.objects.link(cam)
    sc.camera = cam
    return sc, cam


def _shoot(sc, cam, eye, target, lens, path):
    cam.data.lens = lens
    cam.location = eye
    d = Vector(target) - Vector(eye)
    cam.rotation_euler = d.to_track_quat('-Z', 'Y').to_euler()
    sc.render.filepath = path
    bpy.ops.render.render(write_still=True)


STRIDE = {
    "human": {"thigh_l": 35, "shin_l": -35, "foot_l": 15, "thigh_r": -30, "shin_r": -30,
              "upper_arm_l": -35, "fore_arm_l": 35, "upper_arm_r": 35, "fore_arm_r": 35,
              "head": -8, "torso": -6},
    "werewolf": {"thigh_l": 25, "shin_l": -25, "foot_l": 15, "thigh_r": -20, "shin_r": 0,
                 "foot_r": -12, "upper_arm_l": -35, "fore_arm_l": 30, "upper_arm_r": 35,
                 "fore_arm_r": 30, "tail": 15, "head": -10, "torso": -6},
}


def render_previews(name, parts, outdir):
    sc, cam = _scene_setup()
    obs = instantiate(parts)
    tz = 1.0 if name == "human" else 1.1
    q, side = (2.3, 3.6, 1.6), (4.6, 0.1, tz)
    _shoot(sc, cam, q, (0, 0, tz), 62, os.path.join(outdir, f"{name}.png"))
    _shoot(sc, cam, side, (0, 0.05, tz), 62, os.path.join(outdir, f"{name}_side.png"))
    _shoot(sc, cam, (-2.0, -3.6, 1.7), (0, 0, tz), 62,
           os.path.join(outdir, f"{name}_back.png"))
    for part, deg in STRIDE[name].items():
        obs[part].rotation_euler = (math.radians(deg), 0, 0)
    _shoot(sc, cam, q, (0, 0, tz), 62, os.path.join(outdir, f"{name}_stride.png"))
    _shoot(sc, cam, side, (0, 0.05, tz), 62,
           os.path.join(outdir, f"{name}_stride_side.png"))


def main():
    argv = sys.argv[sys.argv.index("--") + 1:] if "--" in sys.argv else []
    here = os.path.dirname(os.path.abspath(__file__))
    figures = os.path.normpath(os.path.join(
        here, "..", "..", "crates", "pdfcer-gui", "src", "dialogs", "model3d", "figures"))
    previews, render = r"D:\scratch\avatar", True
    for i, a in enumerate(argv):
        if a == "--figures":
            figures = argv[i + 1]
        elif a == "--previews":
            previews = argv[i + 1]
        elif a == "--no-render":
            render = False
    os.makedirs(figures, exist_ok=True)
    built = [build_human(), build_werewolf()]
    for name, eye, parts in built:
        check(name, parts)
        write_figure(os.path.join(figures, f"{name}.figure"), name, eye, parts)
    if render:
        os.makedirs(previews, exist_ok=True)
        for name, eye, parts in built:
            render_previews(name, parts, previews)


main()
