#!/usr/bin/env python3
"""Monte Carlo ray tracer reference for the WP6 light model (design plan §3.5, §8.1).

Takes a shoot of opaque leaf discs and, for each leaflet, estimates the probability that it is sunlit along a set of directions by
casting rays from random points on its disc and testing them against every other disc.

The shoot is either read with --leaves FILE (columns id,x,y,z,area_m2,nx,ny,nz, as
written by the ps-arch test `export_leaves_for_ray_tracer` from a 60-day sim plant;
plant60_*, decision 0018) or, without it, a synthetic bean-like shoot of
19 trifoliate leaves with spherical normals (the WP6 fixture, decision 0008).

Outputs, under crates/ps-light/tests/fixtures/ (plant60_* with --leaves):
  plant_leaves.csv     id, x, y, z, area_m2, nx, ny, nz
  plant_raytrace.csv   leaf, direction, elevation_deg, azimuth_deg, transmittance
The diffuse directions are the 16 sky-sector centers of ps-light (elevations given).

Run: python validation/fixtures/light_raytrace.py [--leaves FILE]   (numpy only; seed fixed)
"""

from __future__ import annotations

import argparse
import math
import pathlib

import numpy as np

SEED = 20261105
RAYS_PER_LEAF = 4000
ROOT = pathlib.Path(__file__).resolve().parents[2]
OUT = ROOT / "crates" / "ps-light" / "tests" / "fixtures"

# Sun directions to test (elevation, azimuth), degrees.
SUN = [(90.0, 0.0), (60.0, 135.0), (40.0, 90.0), (25.0, 250.0), (15.0, 300.0)]
# Sky sector elevations used by ps-light (irradiance-weighted band means); printed by
# the Rust test `fixture_sky_directions_match_sky_sectors` if they ever change.
SKY_ELEV = [30.495391, 62.005490]


def plant(rng: np.random.Generator) -> list[tuple[np.ndarray, float, np.ndarray]]:
    leaves = []
    golden = math.radians(137.5)

    def leaflet(center, area):
        v = rng.normal(size=3)
        v /= np.linalg.norm(v)
        if v[2] < 0:
            v = -v
        leaves.append((np.asarray(center, float), float(area), v))

    def add(center, area):
        # A trifoliate leaf: three leaflets of a third of the area each, the terminal
        # one 3.5 cm beyond the petiole tip and two laterals 2.5 cm to either side.
        c = np.asarray(center, float)
        out = np.array([c[0], c[1], 0.0])
        out /= max(np.linalg.norm(out), 1e-9)
        side = np.array([-out[1], out[0], 0.0])
        leaflet(c + 0.035 * out + [0, 0, rng.uniform(-0.01, 0.01)], area / 3)
        leaflet(c + 0.025 * side + [0, 0, rng.uniform(-0.01, 0.01)], area / 3)
        leaflet(c - 0.025 * side + [0, 0, rng.uniform(-0.01, 0.01)], area / 3)

    # Main stem: 7 nodes, internodes 4.5 cm, a trifoliate leaf per node on an 8 cm petiole.
    for k in range(7):
        z = 0.06 + 0.045 * k
        az = k * golden
        r = 0.08
        add([r * math.cos(az), r * math.sin(az), z + 0.02], rng.uniform(0.004, 0.008))
    # Four branches from nodes 1 to 4, each with 3 leaves.
    for b in range(4):
        z0 = 0.06 + 0.045 * (b + 1)
        az0 = (b + 1) * golden + math.pi / 2
        for j in range(3):
            d = 0.06 + 0.05 * j
            az = az0 + 0.6 * (j - 1)
            add([d * math.cos(az), d * math.sin(az), z0 + 0.03 * j + 0.01], rng.uniform(0.003, 0.007))
    return leaves


def unit(elev, az):
    e, a = math.radians(elev), math.radians(az)
    return np.array([math.cos(e) * math.sin(a), math.cos(e) * math.cos(a), math.sin(e)])


def disc_points(c, area, n, count, rng):
    r = math.sqrt(area / math.pi)
    # Orthonormal basis of the disc plane.
    a = np.array([1.0, 0.0, 0.0]) if abs(n[0]) < 0.9 else np.array([0.0, 1.0, 0.0])
    u = np.cross(n, a)
    u /= np.linalg.norm(u)
    v = np.cross(n, u)
    rad = r * np.sqrt(rng.uniform(size=count))
    th = rng.uniform(0, 2 * math.pi, size=count)
    return c + np.outer(rad * np.cos(th), u) + np.outer(rad * np.sin(th), v)


def transmittance(i, leaves, d, rng):
    c, area, n = leaves[i]
    pts = disc_points(c, area, n, RAYS_PER_LEAF, rng)
    blocked = np.zeros(len(pts), dtype=bool)
    for j, (cj, aj, nj) in enumerate(leaves):
        if j == i:
            continue
        denom = float(np.dot(d, nj))
        if abs(denom) < 1e-12:
            continue
        t = ((cj - pts) @ nj) / denom
        hit = pts + np.outer(t, d)
        rj2 = aj / math.pi
        inside = (t > 1e-9) & (np.sum((hit - cj) ** 2, axis=1) <= rj2)
        blocked |= inside
    return 1.0 - blocked.mean()


def read_leaves(path: pathlib.Path) -> list[tuple[np.ndarray, float, np.ndarray]]:
    leaves = []
    for line in path.read_text(encoding="utf-8").splitlines():
        if line.startswith("#") or line.startswith("id,") or not line.strip():
            continue
        v = [float(x) for x in line.split(",")]
        n = np.array(v[5:8])
        leaves.append((np.array(v[1:4]), v[4], n / np.linalg.norm(n)))
    return leaves


def main() -> None:
    ap = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    ap.add_argument("--leaves", type=pathlib.Path, help="leaf discs exported from the sim")
    args = ap.parse_args()
    rng = np.random.default_rng(SEED)
    leaves = read_leaves(args.leaves) if args.leaves else plant(rng)
    source = "a 60-day sim plant" if args.leaves else "the synthetic shoot"
    stem = "plant60" if args.leaves else "plant"
    OUT.mkdir(parents=True, exist_ok=True)
    head = (
        f"# generated by validation/fixtures/light_raytrace.py from {source}, "
        f"seed {SEED}, {RAYS_PER_LEAF} rays per leaf\n"
    )
    with open(OUT / f"{stem}_leaves.csv", "w", encoding="utf-8") as f:
        f.write(head + "id,x,y,z,area_m2,nx,ny,nz\n")
        for i, (c, a, n) in enumerate(leaves):
            f.write(f"{i},{c[0]:.6f},{c[1]:.6f},{c[2]:.6f},{a:.8f},{n[0]:.6f},{n[1]:.6f},{n[2]:.6f}\n")
    dirs = [("sun", e, a) for e, a in SUN]
    for e in SKY_ELEV:
        for k in range(8):
            dirs.append(("sky", e, (k + 0.5) * 45.0))
    with open(OUT / f"{stem}_raytrace.csv", "w", encoding="utf-8") as f:
        f.write(head + "leaf,kind,elevation_deg,azimuth_deg,transmittance\n")
        for i in range(len(leaves)):
            for kind, e, a in dirs:
                t = transmittance(i, leaves, unit(e, a), rng)
                f.write(f"{i},{kind},{e:.6f},{a:.3f},{t:.5f}\n")


if __name__ == "__main__":
    main()
