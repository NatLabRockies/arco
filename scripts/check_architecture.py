#!/usr/bin/env python3
from __future__ import annotations

import json
import subprocess
from pathlib import Path
import tomllib

ROOT = Path(__file__).resolve().parents[1]
POLICY_PATH = ROOT / "architecture-layers.toml"


LAYER_ORDER = {
    "primitives": 0,
    "application": 1,
    "wrappers": 2,
    "surfaces": 3,
}


def load_policy() -> tuple[dict[str, str], dict[str, set[str]], bool]:
    data = tomllib.loads(POLICY_PATH.read_text())
    layers = data["layers"]
    allow_same = bool(data.get("rules", {}).get("allow_same_layer", True))
    overrides_raw = data.get("overrides", {}).get("allow", {})
    overrides: dict[str, set[str]] = {
        src: set(targets) for src, targets in overrides_raw.items()
    }

    crate_to_layer: dict[str, str] = {}
    for layer_name, crates in layers.items():
        if layer_name not in LAYER_ORDER:
            raise SystemExit(f"Unknown layer in policy: {layer_name}")
        for crate in crates:
            if crate in crate_to_layer:
                raise SystemExit(f"Crate listed in multiple layers: {crate}")
            crate_to_layer[crate] = layer_name
    return crate_to_layer, overrides, allow_same


def workspace_edges() -> tuple[set[str], dict[str, set[str]]]:
    proc = subprocess.run(
        ["cargo", "metadata", "--format-version", "1", "--no-deps"],
        cwd=ROOT,
        check=True,
        capture_output=True,
        text=True,
    )
    meta = json.loads(proc.stdout)

    ws_ids = set(meta["workspace_members"])
    id_to_name: dict[str, str] = {}
    deps_by_name: dict[str, set[str]] = {}

    for pkg in meta["packages"]:
        if pkg["id"] not in ws_ids:
            continue
        name = pkg["name"]
        id_to_name[pkg["id"]] = name
        deps_by_name[name] = set(dep["name"] for dep in pkg.get("dependencies", []))

    ws_names = set(id_to_name.values())

    edges: dict[str, set[str]] = {}
    for src, deps in deps_by_name.items():
        edges[src] = {dep for dep in deps if dep in ws_names}

    return ws_names, edges


def main() -> int:
    crate_to_layer, overrides, allow_same = load_policy()
    ws_names, edges = workspace_edges()

    errors: list[str] = []

    missing = sorted(name for name in ws_names if name not in crate_to_layer)
    extra = sorted(name for name in crate_to_layer if name not in ws_names)

    if missing:
        errors.append("Unclassified workspace crates: " + ", ".join(missing))
    if extra:
        errors.append("Policy references non-workspace crates: " + ", ".join(extra))

    for src, targets in edges.items():
        if src not in crate_to_layer:
            continue
        src_layer = crate_to_layer[src]
        src_order = LAYER_ORDER[src_layer]
        allowed_override = overrides.get(src, set())

        for dst in sorted(targets):
            if dst not in crate_to_layer:
                continue
            if dst in allowed_override:
                continue
            dst_layer = crate_to_layer[dst]
            dst_order = LAYER_ORDER[dst_layer]

            if src_order < dst_order:
                errors.append(
                    f"Layer violation: {src}({src_layer}) -> {dst}({dst_layer})"
                )
            elif src_order == dst_order and not allow_same:
                errors.append(
                    f"Same-layer dependency not allowed: {src}({src_layer}) -> {dst}({dst_layer})"
                )

    if errors:
        print("Architecture check failed:")
        for err in errors:
            print(f"- {err}")
        return 1

    print("Architecture check passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
