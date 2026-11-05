#!/usr/bin/env python3
"""Determinism lint for sim crates (design plan §3.14, §11.5, CLAUDE.md).

Fails on, in the source of any sim crate:
  - HashMap / HashSet (iteration order is random; use Vec, BTreeMap or indexmap),
  - thread_rng / rand::random / OsRng / SystemTime / Instant (hidden state, wall clock),
  - std f64 transcendental methods (x.sin(), x.exp(), x.powf() ...; use libm),
  - f32 (sim math is f64).

A line may opt out with a trailing `// determinism: allow (<reason>)` comment, which a
reviewer should question.
"""

from __future__ import annotations

import pathlib
import re
import sys

ROOT = pathlib.Path(__file__).resolve().parent.parent
NON_SIM = {"ps-mesh", "ps-view", "ps-ui", "ps-app", "ps-py"}
ALLOW = "determinism: allow"

TRANSCENDENTALS = (
    "sin|cos|tan|asin|acos|atan|atan2|sin_cos|sinh|cosh|tanh|asinh|acosh|atanh|"
    "exp|exp2|exp_m1|ln|log|log2|log10|ln_1p|powf|powi|cbrt|hypot"
)
RULES = [
    (re.compile(r"\bHash(Map|Set)\b"), "HashMap/HashSet in sim code; use Vec, BTreeMap or indexmap"),
    (re.compile(r"\b(thread_rng|rand::random|OsRng|from_entropy)\b"), "global or entropy RNG; use ps-rng"),
    (re.compile(r"\b(SystemTime|Instant)\b"), "wall-clock time in sim code"),
    (re.compile(rf"\.({TRANSCENDENTALS})\s*\("), "std transcendental method; use libm"),
    (re.compile(r"\bf32\b"), "f32 in sim code; sim math is f64"),
]


def strip_comment(line: str) -> str:
    # Good enough for this lint: drop everything after `//` that is not inside a string.
    in_str = False
    for i, ch in enumerate(line):
        if ch == '"' and (i == 0 or line[i - 1] != "\\"):
            in_str = not in_str
        if not in_str and line.startswith("//", i):
            return line[:i]
    return line


def main() -> int:
    errors: list[str] = []
    crates = sorted(p for p in (ROOT / "crates").iterdir() if p.is_dir() and p.name not in NON_SIM)
    for crate in crates:
        for path in sorted((crate / "src").rglob("*.rs")):
            for no, raw in enumerate(path.read_text(encoding="utf-8").splitlines(), start=1):
                if ALLOW in raw:
                    continue
                code = strip_comment(raw)
                for pattern, why in RULES:
                    if pattern.search(code):
                        errors.append(f"{path.relative_to(ROOT)}:{no}: {why}\n    {raw.strip()}")
    if errors:
        print("Determinism lint failed:")
        print("\n".join(errors))
        return 1
    print(f"Determinism lint OK for {len(crates)} sim crates.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
