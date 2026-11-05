#!/usr/bin/env python3
"""Plot report for a validation run (WP11, design plan §8).

Reads what `ps-cli validate` wrote (default out/validation): results.csv, report.md,
runs/<scenario>/daily.csv and organs_day45.csv, curves/<id>.csv. Writes PNG figures
and report.html next to them. Plots only; the pass/fail verdicts come from ps-cli.

    python3 validation/report.py [out/validation]
"""

from __future__ import annotations

import csv
import html
import sys
from pathlib import Path

import matplotlib

matplotlib.use("Agg")
import matplotlib.pyplot as plt  # noqa: E402

RESPONSE = ["response/control", "response/drought_from_flowering", "response/heat_at_flowering", "response/shade_50"]
SERIES = [
    ("total_dry_mass_g", "Total dry mass built (g)"),
    ("leaf_area_m2", "Leaf area (m²)"),
    ("seed_mass_g", "Seed mass (g)"),
    ("pods", "Pods"),
    ("root_mass_g", "Root mass (g)"),
    ("feedback_loss", "Sink feedback loss (share of gross)"),
    ("ftsw", "Root-zone FTSW"),
    ("height_m", "Height (m)"),
]
KIND_COLORS = {
    "Leaf": "#3a7d2c",
    "Cotyledon": "#9bbf4f",
    "Internode": "#6b4f2a",
    "Root": "#b08a5a",
    "Pod": "#c9a227",
    "SeedInPod": "#7a5c1f",
    "Flower": "#e8d5f0",
    "Bud": "#8fbc8f",
    "Seed": "#5a3d1e",
}


def read_csv(path: Path) -> list[dict[str, str]]:
    with path.open(newline="") as f:
        return list(csv.DictReader(f))


def num(v: str) -> float:
    try:
        return float(v)
    except ValueError:
        return float("nan")


def plot_responses(out: Path) -> Path | None:
    runs = {n: read_csv(out / "runs" / n / "daily.csv") for n in RESPONSE if (out / "runs" / n / "daily.csv").exists()}
    if not runs:
        return None
    fig, axes = plt.subplots(4, 2, figsize=(11, 13), constrained_layout=True)
    for ax, (col, label) in zip(axes.flat, SERIES):
        for name, rows in runs.items():
            days: dict[float, list[float]] = {}
            for r in rows:
                days.setdefault(num(r["day"]), []).append(num(r[col]))
            xs = sorted(days)
            ax.plot(xs, [sum(days[d]) / len(days[d]) for d in xs], label=name.split("/")[-1], lw=1.6)
        ax.set_title(label, fontsize=10)
        ax.set_xlabel("days after sowing", fontsize=8)
        ax.grid(alpha=0.3)
    axes.flat[0].legend(fontsize=8)
    fig.suptitle("§8.3 response scenarios, mean over the replicate plants", fontsize=12)
    path = out / "responses.png"
    fig.savefig(path, dpi=110)
    plt.close(fig)
    return path


def plot_curves(out: Path) -> list[Path]:
    paths = []
    for c in sorted((out / "curves").glob("*.csv")):
        rows = read_csv(c)
        if not rows:
            continue
        fig, ax = plt.subplots(figsize=(6, 4), constrained_layout=True)
        x = [num(r["x"]) for r in rows]
        y = [num(r["y"]) for r in rows]
        sd = [num(r["sd"]) if r["sd"] else 0.0 for r in rows]
        ax.errorbar(x, y, yerr=sd, fmt="o", label="reference", color="#333")
        ax.plot(x, [num(r["sim"]) for r in rows], "-", label="sim", color="#3a7d2c")
        ax.set_title(c.stem, fontsize=10)
        ax.legend(fontsize=8)
        ax.grid(alpha=0.3)
        p = out / f"curve_{c.stem}.png"
        fig.savefig(p, dpi=110)
        plt.close(fig)
        paths.append(p)
    return paths


def plot_organs(out: Path, scenario: str = "single_pot", day: int = 45) -> Path | None:
    src = out / "runs" / scenario / f"organs_day{day}.csv"
    if not src.exists():
        return None
    rows = [r for r in read_csv(src) if r["alive"] == "true" and r["plant"] == "0"]
    fig = plt.figure(figsize=(8, 9), constrained_layout=True)
    ax = fig.add_subplot(projection="3d")
    for r in rows:
        b = [num(r["base_x"]), num(r["base_y"]), num(r["base_z"])]
        t = [num(r["tip_x"]), num(r["tip_y"]), num(r["tip_z"])]
        kind = r["kind"]
        color = KIND_COLORS.get(kind, "#888")
        if kind in ("Leaf", "Cotyledon"):
            area = num(r["area_m2"])
            ax.scatter([t[0]], [t[1]], [t[2]], s=max(4.0, area * 4e4), color=color, alpha=0.7)
        else:
            width = 2.5 if kind == "Internode" else 0.6
            ax.plot([b[0], t[0]], [b[1], t[1]], [b[2], t[2]], color=color, lw=width)
    ax.set_xlabel("x (m)")
    ax.set_ylabel("y (m)")
    ax.set_zlabel("z (m)")
    ax.set_title(f"{scenario}, day {day}: organ graph (leaves as dots sized by area)", fontsize=10)
    path = out / f"organs_{scenario}_day{day}.png"
    fig.savefig(path, dpi=110)
    plt.close(fig)
    return path


def write_html(out: Path, figures: list[Path]) -> Path:
    md = (out / "report.md").read_text() if (out / "report.md").exists() else ""
    body = [f"<pre>{html.escape(md)}</pre>"]
    for f in figures:
        body.append(f'<figure><img src="{f.name}" style="max-width:100%"><figcaption>{f.stem}</figcaption></figure>')
    page = (
        "<!doctype html><meta charset='utf-8'><title>Validation report</title>"
        "<style>body{font-family:sans-serif;max-width:1100px;margin:auto;padding:16px}"
        "pre{white-space:pre-wrap;font-size:12px}</style>" + "\n".join(body)
    )
    path = out / "report.html"
    path.write_text(page)
    return path


def main() -> int:
    out = Path(sys.argv[1] if len(sys.argv) > 1 else "out/validation")
    if not out.is_dir():
        print(f"{out} does not exist; run `ps-cli validate` first", file=sys.stderr)
        return 1
    figures = [p for p in [plot_responses(out), plot_organs(out)] if p] + plot_curves(out)
    page = write_html(out, figures)
    print(f"wrote {len(figures)} figures and {page}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
