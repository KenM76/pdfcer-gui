# -*- coding: utf-8 -*-
"""Build `fixtures/floor-box.prc`: a 4 x 4 grey floor with a 1-unit orange cube
standing on its centre, so one part casts a shadow on another and their
contact is a crease.

The glTF is written here and converted by the engine's CLI:
`pdfcer 3d-convert -o fixtures/floor-box.prc <the glTF>` (12 triangles,
flat normals inferred, +Y up turned to +Z up). Rendered with
`pdfcer 3d-render`, the default picture and the `--no-shadows` one differ in
about 23,000 pixels and the `--no-ambient-occlusion` one in about 3,300.

With `lamp` as a second argument it writes `fixtures/floor-lamp.prc`
instead: the same scene with one white glTF point light
(`KHR_lights_punctual`) above a corner of the floor, which the engine
keeps as the model's own lamp.

Run: python fixtures/floor-box.PROVENANCE.py <path to pdfcer.exe> [lamp]
"""
import pathlib
import subprocess
import sys
import tempfile
import base64
import json
import struct


pos, idx = [], []


def quad(a, b, c, d):
    base = len(pos)
    pos.extend([a, b, c, d])
    idx.extend([base, base + 1, base + 2, base, base + 2, base + 3])


quad((-2, 0, -2), (-2, 0, 2), (2, 0, 2), (2, 0, -2))  # floor, normal +y
floor_tris = len(idx)
x0, x1, y0, y1, z0, z1 = -0.5, 0.5, 0, 1, -0.5, 0.5
quad((x0, y1, z0), (x0, y1, z1), (x1, y1, z1), (x1, y1, z0))  # top
quad((x0, y0, z1), (x1, y0, z1), (x1, y1, z1), (x0, y1, z1))  # +z
quad((x1, y0, z0), (x0, y0, z0), (x0, y1, z0), (x1, y1, z0))  # -z
quad((x1, y0, z1), (x1, y0, z0), (x1, y1, z0), (x1, y1, z1))  # +x
quad((x0, y0, z0), (x0, y0, z1), (x0, y1, z1), (x0, y1, z0))  # -x

pb = b''.join(struct.pack('<3f', *p) for p in pos)
ib = b''.join(struct.pack('<H', i) for i in idx)
pad = (-len(ib)) % 4
data = pb + ib + b'\0' * pad
fl = idx[:floor_tris]
cb = idx[floor_tris:]
g = {
    "asset": {"version": "2.0"},
    "buffers": [{"byteLength": len(data),
                 "uri": "data:application/octet-stream;base64," + base64.b64encode(data).decode()}],
    "bufferViews": [
        {"buffer": 0, "byteLength": len(pb)},
        {"buffer": 0, "byteOffset": len(pb), "byteLength": len(fl) * 2},
        {"buffer": 0, "byteOffset": len(pb) + len(fl) * 2, "byteLength": len(cb) * 2},
    ],
    "accessors": [
        {"bufferView": 0, "componentType": 5126, "count": len(pos), "type": "VEC3",
         "min": [-2, 0, -2], "max": [2, 1, 2]},
        {"bufferView": 1, "componentType": 5123, "count": len(fl), "type": "SCALAR"},
        {"bufferView": 2, "componentType": 5123, "count": len(cb), "type": "SCALAR"},
    ],
    "materials": [
        {"name": "floor", "pbrMetallicRoughness": {"baseColorFactor": [0.8, 0.8, 0.8, 1]}},
        {"name": "box", "pbrMetallicRoughness": {"baseColorFactor": [1, 0.4, 0, 1]}},
    ],
    "meshes": [
        {"primitives": [{"attributes": {"POSITION": 0}, "indices": 1, "material": 0}]},
        {"primitives": [{"attributes": {"POSITION": 0}, "indices": 2, "material": 1}]},
    ],
    "nodes": [{"name": "floor", "mesh": 0}, {"name": "box", "mesh": 1}],
    "scenes": [{"nodes": [0, 1]}],
    "scene": 0,
}

lamp = sys.argv[2:3] == ['lamp']
if lamp:
    g["extensionsUsed"] = ["KHR_lights_punctual"]
    g["extensions"] = {"KHR_lights_punctual": {"lights": [
        {"type": "point", "color": [1, 1, 1], "intensity": 20}]}}
    g["nodes"].append({"name": "lamp", "translation": [1.5, 2.5, 1.5],
                       "extensions": {"KHR_lights_punctual": {"light": 0}}})
    g["scenes"][0]["nodes"].append(2)

out = pathlib.Path(__file__).with_name('floor-lamp.prc' if lamp else 'floor-box.prc')
with tempfile.TemporaryDirectory() as tmp:
    src = pathlib.Path(tmp) / 'floor-box.gltf'
    src.write_text(json.dumps(g, separators=(',', ':')), encoding='utf-8')
    subprocess.run([sys.argv[1], '3d-convert', '-o', str(out), str(src)], check=True)
