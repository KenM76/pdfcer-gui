# -*- coding: utf-8 -*-
"""Build `fixtures/glow-room.prc`: a grey floor and an orange box lit by one
point lamp, with three parts that give off light themselves: `bulb`, loose
beside the box, and `tube-a` and `tube-b`, alone together in the folder
`Fittings`. The viewer's light switches are driven on it: a glowing part
outside a folder of its own, a folder holding only glowing parts, and a lamp
that is no part at all.

The glTF is written here and converted by the engine's CLI:
`pdfcer 3d-convert -o fixtures/glow-room.prc <the glTF>`. The glowing parts
carry a glTF `emissiveFactor`; the lamp is a `KHR_lights_punctual` point
light, which the engine keeps as the model's own lamp.

Run: python fixtures/glow-room.PROVENANCE.py <path to pdfcer.exe>
"""
import base64
import json
import pathlib
import struct
import subprocess
import sys
import tempfile

pos, parts = [], []


def quad(idx, a, b, c, d):
    base = len(pos)
    pos.extend([a, b, c, d])
    idx.extend([base, base + 1, base + 2, base, base + 2, base + 3])


def cube(x0, x1, y0, y1, z0, z1):
    idx = []
    quad(idx, (x0, y1, z0), (x0, y1, z1), (x1, y1, z1), (x1, y1, z0))
    quad(idx, (x0, y0, z0), (x1, y0, z0), (x1, y0, z1), (x0, y0, z1))
    quad(idx, (x0, y0, z1), (x1, y0, z1), (x1, y1, z1), (x0, y1, z1))
    quad(idx, (x1, y0, z0), (x0, y0, z0), (x0, y1, z0), (x1, y1, z0))
    quad(idx, (x1, y0, z1), (x1, y0, z0), (x1, y1, z0), (x1, y1, z1))
    quad(idx, (x0, y0, z0), (x0, y0, z1), (x0, y1, z1), (x0, y1, z0))
    return idx


floor = []
quad(floor, (-2, 0, -2), (-2, 0, 2), (2, 0, 2), (2, 0, -2))
parts.append(('floor', floor, [0.8, 0.8, 0.8, 1], None))
parts.append(('box', cube(-0.5, 0.5, 0, 1, -0.5, 0.5), [1, 0.4, 0, 1], None))
parts.append(('bulb', cube(1.0, 1.4, 0, 0.4, 1.0, 1.4), [0.2, 0.2, 0.2, 1], [1, 0.9, 0.5]))
parts.append(('tube-a', cube(-1.6, -1.4, 0, 0.2, -1.5, 0.5), [0.2, 0.2, 0.2, 1], [0.4, 0.8, 1]))
parts.append(('tube-b', cube(-1.0, -0.8, 0, 0.2, -1.5, 0.5), [0.2, 0.2, 0.2, 1], [0.4, 0.8, 1]))

pb = b''.join(struct.pack('<3f', *p) for p in pos)
data = bytearray(pb)
views = [{"buffer": 0, "byteLength": len(pb)}]
accessors = [{"bufferView": 0, "componentType": 5126, "count": len(pos), "type": "VEC3",
              "min": [min(p[i] for p in pos) for i in range(3)],
              "max": [max(p[i] for p in pos) for i in range(3)]}]
materials, meshes, nodes = [], [], []
for name, idx, colour, glow in parts:
    ib = b''.join(struct.pack('<H', i) for i in idx)
    views.append({"buffer": 0, "byteOffset": len(data), "byteLength": len(ib)})
    data += ib + b'\0' * ((-len(ib)) % 4)
    accessors.append({"bufferView": len(views) - 1, "componentType": 5123,
                      "count": len(idx), "type": "SCALAR"})
    material = {"name": name, "pbrMetallicRoughness": {"baseColorFactor": colour}}
    if glow:
        material["emissiveFactor"] = glow
    materials.append(material)
    meshes.append({"primitives": [{"attributes": {"POSITION": 0},
                                   "indices": len(accessors) - 1,
                                   "material": len(materials) - 1}]})
    nodes.append({"name": name, "mesh": len(meshes) - 1})

nodes.append({"name": "Fittings", "children": [3, 4]})
nodes.append({"name": "lamp", "translation": [1.5, 2.5, -1.5],
              "extensions": {"KHR_lights_punctual": {"light": 0}}})
g = {
    "asset": {"version": "2.0"},
    "buffers": [{"byteLength": len(data),
                 "uri": "data:application/octet-stream;base64,"
                        + base64.b64encode(bytes(data)).decode()}],
    "bufferViews": views,
    "accessors": accessors,
    "materials": materials,
    "meshes": meshes,
    "nodes": nodes,
    "scenes": [{"nodes": [0, 1, 2, 5, 6]}],
    "scene": 0,
    "extensionsUsed": ["KHR_lights_punctual"],
    "extensions": {"KHR_lights_punctual": {"lights": [
        {"type": "point", "color": [1, 1, 1], "intensity": 20}]}},
}

out = pathlib.Path(__file__).with_name('glow-room.prc')
with tempfile.TemporaryDirectory() as tmp:
    src = pathlib.Path(tmp) / 'glow-room.gltf'
    src.write_text(json.dumps(g, separators=(',', ':')), encoding='utf-8')
    subprocess.run([sys.argv[1], '3d-convert', '-o', str(out), str(src)], check=True)
