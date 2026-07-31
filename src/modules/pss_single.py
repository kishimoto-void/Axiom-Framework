#!/usr/bin/env python3
"""PSS v1.0.0-rc1 — loads gzip source from pss_data/ chunks.

Behavior matches the plain single-file edition:
Mission / Gate / PredictionPolicy / compile_for_generic(mode=balanced|strict).
"""
from pathlib import Path
import base64, gzip
root = Path(__file__).resolve().parent
data = "".join((root / "pss_data" / f"chunk_{i}.txt").read_text() for i in range(3))
exec(compile(gzip.decompress(base64.b64decode(data)), "pss_single.py", "exec"), globals())
if __name__ == "__main__":
    main()
