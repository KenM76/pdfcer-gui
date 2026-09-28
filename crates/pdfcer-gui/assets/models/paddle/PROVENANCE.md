# PROVENANCE — PaddleOCR PP-OCRv5 Latin models (English and French)

Three files that pdfcer-gui redistributes in its portable folder as
`models/paddle/`, where the PaddleOCR recogniser
(`pdfcer_core::ocr::engine_paddle`) loads them. They are not a Cargo dependency,
so `package-portable.py` copies this directory and the attribution is written
by hand in `about.hbs` and in the About dialog.

- **Creator:** the PaddleOCR authors (PaddlePaddle,
  <https://github.com/PaddlePaddle/PaddleOCR>), who trained PP-OCRv5 and
  published the ONNX exports.
- **Source:** Hugging Face, retrieved 2026-09-28:
  - `PaddlePaddle/PP-OCRv5_mobile_det_onnx`, file `inference.onnx`;
  - `PaddlePaddle/latin_PP-OCRv5_mobile_rec_onnx`, files `inference.onnx` and
    `inference.yml`.
- **Licence: `Apache-2.0`**, as both model cards declare. `LICENSE` in this
  directory is PaddleOCR's copy of the licence text.
- **Languages:** the Latin-script recogniser — English (US, UK and Canadian
  spelling are the same alphabet) and French, every accent and ligature
  Québécois text uses included (é è ê ë à â î ï ô û ù ü ÿ ç œ æ), with digits and
  punctuation. 836 dictionary entries.

## Changes made by pdfcer (Apache-2.0 §4(b))

- `det.onnx` — **unchanged**, renamed only.
- `rec.onnx` — **converted from ONNX opset 7 to opset 13** with
  `onnx.version_converter.convert_version(model, 13)` and checked with
  `onnx.checker.check_model`. The weights are untouched. The ONNX runtime the
  engine links (`rten`) refuses the opset-7 form: `BatchNormalization` carries
  the removed `spatial` attribute and `Slice` takes its bounds as attributes.
- `dict.txt` — **extracted** from `inference.yml`,
  `PostProcess.character_dict`, one entry per line, LF endings. The engine's
  CTC decoder reads it; the export embeds no `character` metadata.

| Shipped as | Upstream | Bytes | SHA-256 |
|---|---|---:|---|
| `det.onnx` | `PP-OCRv5_mobile_det_onnx/inference.onnx` | 4,826,518 | `a431985659dc921974177a95adcfbb90fd9e51989a5e04d70d0b75f597b6e61d` |
| `rec.onnx` | `latin_PP-OCRv5_mobile_rec_onnx/inference.onnx` (8,042,023 B, `7888113072263cb471b93f66dd5e2ad70548dc526fa1ace760d0d973dd121498`), converted | 8,075,678 | `54d7e99d58eed786f0d5647f4121e889ef08f3a9671ef74b64a77c6d3cc076ad` |
| `dict.txt` | `latin_PP-OCRv5_mobile_rec_onnx/inference.yml`, extracted | 2,616 | `ccbcc45730b3fbbd9050c5bc74db6a99067141ef1035e3d14889a84a6b9b1aff` |

Shipping, and the choice of English and French over the engine's Chinese
models, are the operator's rulings.
