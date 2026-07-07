#!/usr/bin/env python3
from __future__ import annotations

import argparse
import hashlib
import json
import os
import re
import shutil
import subprocess
import tarfile
import tempfile
import zipfile
from pathlib import Path
from typing import Any


LOCAL_WORKFLOW_ANCHOR = (
    "          dist build ${{ needs.plan.outputs.tag-flag }} --print=linkage "
    "--output-format=json ${{ matrix.dist_args }} > dist-manifest.json\n"
    '          echo "dist ran successfully"\n'
)
LOCAL_WORKFLOW_COMMAND = (
    "          uv run python scripts/ci_bundle_scip_runtime.py local "
    "dist-manifest.json\n"
)
GLOBAL_WORKFLOW_ANCHOR = (
    "          dist build ${{ needs.plan.outputs.tag-flag }} --output-format=json "
    '"--artifacts=global" > dist-manifest.json\n'
    '          echo "dist ran successfully"\n\n'
)
GLOBAL_WORKFLOW_COMMAND = (
    "          python3 scripts/ci_bundle_scip_runtime.py global "
    "dist-manifest.json target/distrib\n"
)

MACOS_GCC_LIBS = (
    "libgcc_s.1.1.dylib",
    "libgfortran.5.dylib",
    "libquadmath.0.dylib",
)


def read_json(path: Path) -> dict[str, Any]:
    return json.loads(path.read_text(encoding="utf-8"))


def write_json(path: Path, data: dict[str, Any]) -> None:
    path.write_text(json.dumps(data, indent=2) + "\n", encoding="utf-8")


def sha256(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as file:
        for chunk in iter(lambda: file.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def target_env_path(name: str, target: str) -> Path:
    suffix = target.replace("-", "_")
    value = os.environ.get(f"{name}_{suffix}") or os.environ.get(name)
    if not value:
        raise SystemExit(f"{name}_{suffix} or {name} is required for {target}")
    return Path(value)


def required(path: Path) -> Path:
    if not path.is_file():
        raise SystemExit(f"required runtime file does not exist: {path}")
    return path


def ldd_path(binary: Path, library: str) -> Path | None:
    result = subprocess.run(
        ["ldd", str(binary)],
        check=False,
        stdout=subprocess.PIPE,
        stderr=subprocess.DEVNULL,
        text=True,
    )
    for line in result.stdout.splitlines():
        match = re.search(rf"\b{re.escape(library)}\s+=>\s+(\S+)", line)
        if match and Path(match.group(1)).is_file():
            return Path(match.group(1))
    return None


def runtime_files(target: str) -> list[Path]:
    scip_lib = target_env_path("ARCO_SCIP_LIBRARY_PATH", target)

    if target.endswith("-unknown-linux-gnu"):
        fortran_lib = target_env_path("ARCO_SCIP_FORTRAN_RUNTIME_PATH", target)
        files = [
            required(scip_lib / "libscip.so.10.0"),
            required(fortran_lib / "libgfortran.so.5"),
        ]
        quadmath = fortran_lib / "libquadmath.so.0"
        if not quadmath.is_file():
            quadmath = ldd_path(files[1], "libquadmath.so.0")
        if quadmath is not None:
            files.append(required(quadmath))
        return files

    if target.endswith("-apple-darwin"):
        gcc_lib = target_env_path("ARCO_SCIP_GCC_RUNTIME_PATH", target)
        return [required(scip_lib / "libscip.10.0.dylib")] + [
            required(gcc_lib / name) for name in MACOS_GCC_LIBS
        ]

    if "-pc-windows-" in target:
        runtime_dir = scip_lib.parent / "bin"
        if not runtime_dir.is_dir():
            runtime_dir = scip_lib
        files = sorted(runtime_dir.glob("*.dll"), key=lambda path: path.name.casefold())
        if not any(path.name.casefold() == "libscip.dll" for path in files):
            raise SystemExit(f"{runtime_dir} does not contain libscip.dll")
        return files

    raise SystemExit(f"unsupported SCIP runtime target: {target}")


def archive_root(archive: Path) -> str:
    if archive.name.endswith(".tar.gz"):
        return archive.name.removesuffix(".tar.gz")
    if archive.name.endswith(".zip"):
        return archive.name.removesuffix(".zip")
    raise SystemExit(f"unsupported archive format: {archive}")


def extract_tar(archive: Path, dest: Path) -> None:
    with tarfile.open(archive, "r:gz") as tar:
        try:
            tar.extractall(dest, filter="data")
        except TypeError:
            tar.extractall(dest)


def copy_runtime_files(dest: Path, files: list[Path]) -> list[str]:
    names: list[str] = []
    for source in files:
        shutil.copy2(source, dest / source.name)
        names.append(source.name)
    return names


def repack_archive(archive: Path, files: list[Path]) -> list[str]:
    if archive.name.endswith(".zip"):
        names = [path.name for path in files]
        replacement = archive.with_suffix(f"{archive.suffix}.tmp")
        with zipfile.ZipFile(archive, "r") as existing:
            with zipfile.ZipFile(replacement, "w", zipfile.ZIP_DEFLATED) as updated:
                for info in existing.infolist():
                    if info.filename not in names:
                        updated.writestr(info, existing.read(info.filename))
                for source in files:
                    updated.write(source, source.name)
        replacement.replace(archive)
        return names

    with tempfile.TemporaryDirectory(prefix="arco-scip-") as temp_name:
        temp = Path(temp_name)
        extract_tar(archive, temp)
        root = temp / archive_root(archive)
        if not root.is_dir():
            raise SystemExit(f"{archive} did not unpack to {root.name}")
        names = copy_runtime_files(root, files)
        replacement = archive.with_name(f"{archive.name}.tmp")
        with tarfile.open(replacement, "w:gz") as tar:
            tar.add(root, arcname=root.name)
        replacement.replace(archive)
        return names


def validate_linux_rpath(archive: Path) -> None:
    if "linux" not in archive.name or not archive.name.endswith(".tar.gz"):
        return
    with tempfile.TemporaryDirectory(prefix="arco-scip-check-") as temp_name:
        temp = Path(temp_name)
        extract_tar(archive, temp)
        binary = temp / archive_root(archive) / "arco"
        result = subprocess.run(
            ["readelf", "-d", str(binary)],
            check=False,
            stdout=subprocess.PIPE,
            stderr=subprocess.DEVNULL,
            text=True,
        )
    if "(RPATH)" not in result.stdout or "$ORIGIN" not in result.stdout:
        raise SystemExit(f"{binary} is missing transitive $ORIGIN RPATH")


def bundle_local(manifest_path: Path, distrib_dir: Path) -> None:
    manifest = read_json(manifest_path)
    for name, artifact in manifest.get("artifacts", {}).items():
        if artifact.get("kind") != "executable-zip":
            continue
        target = artifact["target_triples"][0]
        archive = distrib_dir / name
        runtime_names = repack_archive(archive, runtime_files(target))
        validate_linux_rpath(archive)

        checksum = sha256(archive)
        (distrib_dir / f"{archive.name}.sha256").write_text(
            f"{checksum} *{archive.name}\n",
            encoding="utf-8",
        )
        artifact["checksums"] = artifact.get("checksums") or {}
        artifact["checksums"]["sha256"] = checksum
        artifact["assets"] = [
            asset
            for asset in artifact.get("assets", [])
            if asset.get("kind") != "dynamic_library"
        ] + [
            {"name": runtime_name, "path": runtime_name, "kind": "dynamic_library"}
            for runtime_name in runtime_names
        ]
    write_json(manifest_path, manifest)


def runtime_assets_by_archive(distrib_dir: Path) -> dict[str, list[str]]:
    by_archive: dict[str, list[str]] = {}
    for manifest_path in sorted(distrib_dir.glob("*-dist-manifest.json")):
        for name, artifact in read_json(manifest_path).get("artifacts", {}).items():
            libs = [
                asset["name"]
                for asset in artifact.get("assets", [])
                if asset.get("kind") == "dynamic_library"
            ]
            if libs:
                by_archive[name] = libs
    return by_archive


def shell_array(libs: list[str]) -> str:
    return " ".join(libs)


def json_array(libs: list[str]) -> str:
    return ",".join(json.dumps(lib) for lib in libs)


def replace_shell_var(block: str, name: str, value: str, quote: str = '"') -> str:
    pattern = re.compile(rf'(?m)^(\s*{re.escape(name)}=)(?:"[^"]*"|\'[^\']*\')$')
    updated, count = pattern.subn(
        lambda match: f"{match.group(1)}{quote}{value}{quote}", block
    )
    if count != 1:
        raise SystemExit(f"missing {name} in shell installer block")
    return updated


def patch_shell_installer(text: str, libs_by_archive: dict[str, list[str]]) -> str:
    for archive, libs in libs_by_archive.items():
        pattern = re.compile(rf'(?ms)^(\s*"{re.escape(archive)}"\)\n)(.*?^\s*;;)')
        match = pattern.search(text)
        if not match:
            continue
        block = replace_shell_var(match.group(0), "_libs", shell_array(libs))
        block = replace_shell_var(block, "_libs_js_array", json_array(libs), "'")
        text = text[: match.start()] + block + text[match.end() :]
    return text.replace(
        'ensure chmod +x "$_lib_install_dir/$_lib_name"',
        'ensure chmod +x "$_lib_install_temp/$_lib_name"',
    )


def ps_array(libs: list[str]) -> str:
    return "@(" + ", ".join(json.dumps(lib) for lib in libs) + ")"


def patch_powershell_installer(text: str, libs_by_archive: dict[str, list[str]]) -> str:
    def patch_block(match: re.Match[str]) -> str:
        block = match.group(0)
        archive = re.search(r'"artifact_name"\s*=\s*"([^"]+)"', block)
        if archive is None or archive.group(1) not in libs_by_archive:
            return block
        return re.sub(
            r'(?m)^(\s*"libs"\s*=\s*)@\([^)]*\)$',
            lambda libs_match: (
                f"{libs_match.group(1)}{ps_array(libs_by_archive[archive.group(1)])}"
            ),
            block,
            count=1,
        )

    return re.sub(r'(?ms)^    "[^"]+" = @\{\n.*?^    \}', patch_block, text)


def rewrite_sha256_sum(path: Path, checksums: dict[str, str]) -> None:
    if not path.exists():
        return
    lines = []
    for line in path.read_text(encoding="utf-8").splitlines():
        match = re.match(r"^[0-9a-fA-F]{64} \*(.+)$", line)
        if match and match.group(1) in checksums:
            lines.append(f"{checksums[match.group(1)]} *{match.group(1)}")
        else:
            lines.append(line)
    path.write_text("\n".join(lines) + "\n", encoding="utf-8")


def bundle_global(manifest_path: Path, distrib_dir: Path) -> None:
    libs_by_archive = runtime_assets_by_archive(distrib_dir)
    if not libs_by_archive:
        raise SystemExit(f"no bundled SCIP runtime assets found in {distrib_dir}")

    changed: list[Path] = []
    for filename, patcher in {
        "arco-cli-installer.sh": patch_shell_installer,
        "arco-cli-installer.ps1": patch_powershell_installer,
    }.items():
        path = distrib_dir / filename
        if not path.exists():
            continue
        original = path.read_text(encoding="utf-8")
        updated = patcher(original, libs_by_archive)
        if updated != original:
            path.write_text(updated, encoding="utf-8")
            changed.append(path)

    checksums = {path.name: sha256(path) for path in changed}
    rewrite_sha256_sum(distrib_dir / "sha256.sum", checksums)

    manifest = read_json(manifest_path)
    for name, checksum in checksums.items():
        artifact = manifest.get("artifacts", {}).get(name)
        if artifact and artifact.get("checksums"):
            artifact["checksums"]["sha256"] = checksum
    write_json(manifest_path, manifest)


def insert_after(text: str, anchor: str, insertion: str) -> str:
    if insertion in text:
        return text
    index = text.find(anchor)
    if index == -1:
        raise SystemExit(f"missing workflow anchor: {anchor.strip()}")
    return text[: index + len(anchor)] + insertion + text[index + len(anchor) :]


def patch_workflow(path: Path) -> None:
    text = path.read_text(encoding="utf-8")
    text = insert_after(text, LOCAL_WORKFLOW_ANCHOR, LOCAL_WORKFLOW_COMMAND)
    text = insert_after(text, GLOBAL_WORKFLOW_ANCHOR, GLOBAL_WORKFLOW_COMMAND)
    path.write_text(text, encoding="utf-8")


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    subcommands = parser.add_subparsers(dest="command", required=True)

    local = subcommands.add_parser("local")
    local.add_argument("manifest", type=Path)
    local.add_argument("--distrib-dir", type=Path, default=Path("target/distrib"))

    global_cmd = subcommands.add_parser("global")
    global_cmd.add_argument("manifest", type=Path)
    global_cmd.add_argument("distrib_dir", type=Path)

    workflow = subcommands.add_parser("workflow")
    workflow.add_argument(
        "path", type=Path, default=Path(".github/workflows/v-release.yml")
    )
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    if args.command == "local":
        bundle_local(args.manifest, args.distrib_dir)
    elif args.command == "global":
        bundle_global(args.manifest, args.distrib_dir)
    elif args.command == "workflow":
        patch_workflow(args.path)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
