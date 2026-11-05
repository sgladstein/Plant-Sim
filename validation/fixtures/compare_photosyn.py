#!/usr/bin/env python3
"""Checks the committed gas-exchange fixture against a fresh plantecophys run.

Usage: python validation/fixtures/compare_photosyn.py committed.csv fresh.csv
Fails when any value differs by more than 1e-6 relative (plus 1e-9 absolute).
"""

import csv
import sys


def load(path):
    with open(path, encoding="utf-8") as f:
        rows = [r for r in csv.DictReader(line for line in f if not line.startswith("#"))]
    return {(float(r["par"]), float(r["tleaf"]), float(r["vpd"]), float(r["ca"])): r for r in rows}


def main() -> int:
    a, b = load(sys.argv[1]), load(sys.argv[2])
    if a.keys() != b.keys():
        print(f"grids differ: {len(a)} vs {len(b)} rows")
        return 1
    worst = {}
    bad = 0
    for k in sorted(a):
        for col in ("a_net", "gs", "e", "ci"):
            x, y = float(a[k][col]), float(b[k][col])
            rel = abs(x - y) / max(abs(y), 1e-12)
            worst[col] = max(worst.get(col, 0.0), rel)
            if abs(x - y) > 1e-6 * abs(y) + 1e-9:
                bad += 1
                if bad <= 20:
                    print(f"{k} {col}: committed {x} plantecophys {y}")
    print("worst relative difference:", {c: f"{v:.2e}" for c, v in worst.items()})
    return 1 if bad else 0


if __name__ == "__main__":
    sys.exit(main())
