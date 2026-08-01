"""AXIOM COMMON PROTOCOL v1.1.0 implementation (zlib-expanded)."""
from __future__ import annotations
import base64, sys, types, zlib
from pathlib import Path

_dir = Path(__file__).resolve().parent
_P0 = (_dir / "_p0.b64").read_text(encoding="utf-8").strip()
_P1 = (_dir / "_p1.b64").read_text(encoding="utf-8").strip()

def _expand():
    code = zlib.decompress(base64.b64decode(_P0 + _P1)).decode("utf-8")
    mod = types.ModuleType(__name__)
    mod.__file__ = __file__
    mod.__package__ = __package__
    sys.modules[__name__] = mod
    exec(compile(code, __file__, "exec"), mod.__dict__)
    g = globals()
    for k, v in mod.__dict__.items():
        if not k.startswith("_") or k in ("__all__",):
            g[k] = v
    g.update({k: v for k, v in mod.__dict__.items() if k not in g})

_expand()
del _expand, _P0, _P1, base64, zlib, types, sys, Path
