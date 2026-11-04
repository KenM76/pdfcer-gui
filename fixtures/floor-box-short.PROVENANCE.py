# -*- coding: utf-8 -*-
"""Build `fixtures/floor-box-short.prc`: `fixtures/floor-box.prc` whose
FileHeader states a model-file end one byte past the data (ISO 14739-1
WD 6.1.1), a damaged file every triangle of which still reads.

The end is the little-endian u32 equal to the file's length within its first
120 bytes, as the engine's own salvage test finds it. The engine reads to the
data's end and counts one damaged unit in `AssembledModel::damaged`.

Run: python fixtures/floor-box-short.PROVENANCE.py
"""
import pathlib

here = pathlib.Path(__file__).parent
data = bytearray((here / "floor-box.prc").read_bytes())
length = len(data).to_bytes(4, "little")
at = [i for i in range(120) if data[i : i + 4] == length]
if len(at) != 1:
    raise SystemExit(f"expected one stated end in the header, found {at}")
data[at[0] : at[0] + 4] = (len(data) + 1).to_bytes(4, "little")
(here / "floor-box-short.prc").write_bytes(bytes(data))
print(f"wrote floor-box-short.prc: end at byte {at[0]} now {len(data) + 1}")
