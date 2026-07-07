from __future__ import annotations

import importlib.util
import json
import sys
import tarfile
from pathlib import Path


def load_bundle_module():
    module_path = Path(__file__).with_name("ci_bundle_scip_runtime.py")
    spec = importlib.util.spec_from_file_location("ci_bundle_scip_runtime", module_path)
    if spec is None or spec.loader is None:
        raise AssertionError(f"failed to load {module_path}")
    module = importlib.util.module_from_spec(spec)
    sys.modules[spec.name] = module
    spec.loader.exec_module(module)
    return module


def test_patch_shell_installer_sets_libs_and_fixes_chmod() -> None:
    bundle = load_bundle_module()
    text = """
        "arco-cli-x86_64-unknown-linux-gnu.tar.gz")
            _libs=""
            _libs_js_array=""
            ;;
    ensure chmod +x "$_lib_install_dir/$_lib_name"
"""

    patched = bundle.patch_shell_installer(
        text,
        {"arco-cli-x86_64-unknown-linux-gnu.tar.gz": ["libscip.so.10.0"]},
    )

    assert '_libs="libscip.so.10.0"' in patched
    assert "_libs_js_array='\"libscip.so.10.0\"'" in patched
    assert 'ensure chmod +x "$_lib_install_temp/$_lib_name"' in patched


def test_patch_powershell_installer_sets_libs_for_matching_archive() -> None:
    bundle = load_bundle_module()
    text = """
    "x86_64-pc-windows-msvc" = @{
      "artifact_name" = "arco-cli-x86_64-pc-windows-msvc.zip"
      "libs" = @()
    }
"""

    patched = bundle.patch_powershell_installer(
        text,
        {"arco-cli-x86_64-pc-windows-msvc.zip": ["libscip.dll", "ipopt-3.dll"]},
    )

    assert '"libs" = @("libscip.dll", "ipopt-3.dll")' in patched


def test_patch_workflow_is_idempotent() -> None:
    bundle = load_bundle_module()
    workflow = bundle.LOCAL_WORKFLOW_ANCHOR + bundle.GLOBAL_WORKFLOW_ANCHOR

    patched = bundle.insert_after(
        workflow,
        bundle.LOCAL_WORKFLOW_ANCHOR,
        bundle.LOCAL_WORKFLOW_COMMAND,
    )
    patched = bundle.insert_after(
        patched,
        bundle.GLOBAL_WORKFLOW_ANCHOR,
        bundle.GLOBAL_WORKFLOW_COMMAND,
    )

    assert (
        bundle.insert_after(
            patched,
            bundle.LOCAL_WORKFLOW_ANCHOR,
            bundle.LOCAL_WORKFLOW_COMMAND,
        )
        == patched
    )
    assert bundle.LOCAL_WORKFLOW_COMMAND in patched
    assert bundle.GLOBAL_WORKFLOW_COMMAND in patched


def test_rewrite_sha256_sum_updates_existing_entry(tmp_path: Path) -> None:
    bundle = load_bundle_module()
    path = tmp_path / "sha256.sum"
    path.write_text("0" * 64 + " *arco-cli-installer.sh\n", encoding="utf-8")

    bundle.rewrite_sha256_sum(path, {"arco-cli-installer.sh": "1" * 64})

    assert path.read_text(encoding="utf-8") == ("1" * 64 + " *arco-cli-installer.sh\n")


def test_bundle_local_adds_runtime_files_and_manifest_assets(
    tmp_path: Path,
    monkeypatch,
) -> None:
    bundle = load_bundle_module()
    distrib_dir = tmp_path / "target" / "distrib"
    distrib_dir.mkdir(parents=True)
    root = tmp_path / "arco-cli-x86_64-apple-darwin"
    root.mkdir()
    (root / "arco").write_text("fake", encoding="utf-8")

    archive = distrib_dir / "arco-cli-x86_64-apple-darwin.tar.gz"
    with tarfile.open(archive, "w:gz") as tar:
        tar.add(root, arcname=root.name)

    manifest_path = tmp_path / "dist-manifest.json"
    manifest_path.write_text(
        json.dumps(
            {
                "artifacts": {
                    archive.name: {
                        "kind": "executable-zip",
                        "target_triples": ["x86_64-apple-darwin"],
                        "assets": [{"name": "arco", "kind": "executable"}],
                    }
                }
            }
        ),
        encoding="utf-8",
    )

    scip_lib = tmp_path / "scip" / "lib"
    gcc_lib = tmp_path / "gcc"
    scip_lib.mkdir(parents=True)
    gcc_lib.mkdir()
    (scip_lib / "libscip.10.0.dylib").write_text("scip", encoding="utf-8")
    for name in bundle.MACOS_GCC_LIBS:
        (gcc_lib / name).write_text(name, encoding="utf-8")
    monkeypatch.setenv("ARCO_SCIP_LIBRARY_PATH_x86_64_apple_darwin", str(scip_lib))
    monkeypatch.setenv("ARCO_SCIP_GCC_RUNTIME_PATH_x86_64_apple_darwin", str(gcc_lib))

    bundle.bundle_local(manifest_path, distrib_dir)

    with tarfile.open(archive, "r:gz") as tar:
        names = set(tar.getnames())
    assert f"{root.name}/libscip.10.0.dylib" in names

    manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
    assets = manifest["artifacts"][archive.name]["assets"]
    assert {
        "name": "libscip.10.0.dylib",
        "path": "libscip.10.0.dylib",
        "kind": "dynamic_library",
    } in assets
