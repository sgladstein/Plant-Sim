#!/usr/bin/env python3
"""Dependency rule (design plan §10.5, CLAUDE.md).

Sim crates (ps-world and everything below it, plus the headless tools) must never
depend, directly or transitively, on Bevy, egui, wgpu, winit or anything with a GPU.
ps-mesh may depend only on ps-graph, ps-units and glam among non-std crates it names.
Only ps-view, ps-ui and ps-app may depend on Bevy.

Reads `cargo metadata`; exits non-zero with a list of violations.
"""

from __future__ import annotations

import json
import re
import subprocess
import sys

RENDER_CRATES = {"ps-view", "ps-ui", "ps-app"}
MESH_CRATE = "ps-mesh"
MESH_ALLOWED_DIRECT = {"ps-graph", "ps-units", "glam"}
FORBIDDEN = re.compile(r"^(bevy.*|egui.*|eframe|wgpu.*|winit|naga|gilrs.*|glow|ash|vulkano|gpu-.*)$")


def main() -> int:
    meta = json.loads(
        subprocess.check_output(["cargo", "metadata", "--format-version", "1", "--locked"])
    )
    pkgs = {p["id"]: p for p in meta["packages"]}
    members = set(meta["workspace_members"])
    nodes = {n["id"]: n for n in meta["resolve"]["nodes"]}

    def name(pid: str) -> str:
        return pkgs[pid]["name"]

    def closure(root: str) -> set[str]:
        seen: set[str] = set()
        stack = [root]
        while stack:
            pid = stack.pop()
            for dep in nodes[pid]["deps"]:
                # Only normal and build dependencies ship; dev-dependencies are test-only.
                kinds = {k.get("kind") for k in dep["dep_kinds"]}
                if kinds <= {"dev"}:
                    continue
                if dep["pkg"] not in seen:
                    seen.add(dep["pkg"])
                    stack.append(dep["pkg"])
        return seen

    errors: list[str] = []
    for mid in sorted(members, key=name):
        crate = name(mid)
        if crate in RENDER_CRATES:
            continue
        for dep in sorted(closure(mid), key=name):
            if FORBIDDEN.match(name(dep)):
                errors.append(f"{crate} depends on GPU/UI crate {name(dep)}")
        if crate == MESH_CRATE:
            for dep in nodes[mid]["deps"]:
                kinds = {k.get("kind") for k in dep["dep_kinds"]}
                if kinds <= {"dev"}:
                    continue
                if name(dep["pkg"]) not in MESH_ALLOWED_DIRECT:
                    errors.append(f"{crate} may depend only on {sorted(MESH_ALLOWED_DIRECT)}, found {name(dep['pkg'])}")

    if errors:
        print("Dependency rule violations:")
        for e in errors:
            print(f"  - {e}")
        return 1
    print(f"Dependency rules OK for {len(members)} workspace crates.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
