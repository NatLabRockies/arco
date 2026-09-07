# Release Python distributions

Choose the cutoff through the Release Please PR as described in
[Release Policy](../../RELEASE_POLICY.md). Release Please automatically opens and
updates the PR with `GITHUB_TOKEN`. Candidate runs remain pending and create no
artifacts until a maintainer selects Approve workflows to run in the PR merge box.
Approve the run for the exact PR revision chosen as the cutoff. It automatically
uses the PR's base branch and builds six wheels and one sdist before a version tag
exists.

Before publishing the first release, have an organization or repository
administrator enable GitHub Immutable Releases. The release workflows use the
workflow-provided `GITHUB_TOKEN`; they do not require an additional personal
access token.

Review the successful candidate, then manually run the promotion workflow from the
same base branch with the candidate run ID. Promotion squash merges the unchanged
release PR, lets Release Please create the tag and draft, and publishes the
original files to a GitHub Release.

Promotion verifies the release and every asset after GitHub publication. A failed
verification prevents promotion from dispatching `publish-pypi.yml`, but it does
not undo the published tag or release. After successful verification, promotion
dispatches `publish-pypi.yml` at the tag.
That workflow downloads the six wheels and sdist, verifies their GitHub release
attestations, and publishes through PyPI trusted publishing. Check its result
separately from the promotion run. The workflow accepts stable `vX.Y.Z` tags.

## Retry publication

Use Re-run failed jobs for candidate failures only while the triggering PR head and
base remain unchanged. A Release Please update supersedes the earlier candidate.
Wait for the new pending run, approve that revision, and review its artifacts.
Promotion rejects stale candidate approvals through its source checks.

For a verification failure, first confirm that the published release is immutable.
If it is immutable, resolve the verification problem and rerun the failed read-only
job. If it is mutable, stop the announcement and PyPI publication. Enabling
Immutable Releases now protects only future releases; it cannot change the
existing release. Preserve its tag and files, correct the setting, and issue a new
version.

For a PyPI failure, run `publish-pypi.yml` from the published tag with the same
`tag` input. It verifies the original GitHub files and skips distributions already
on PyPI. It never rebuilds packages. Ship code fixes as a new version.

## Distribution targets

| Platform                     | Wheels                      |
| ---------------------------- | --------------------------- |
| Linux x86_64, manylinux 2.28 | `cp310-cp310`, `cp311-abi3` |
| macOS arm64                  | `cp310-cp310`, `cp311-abi3` |
| Windows x86_64               | `cp310-cp310`, `cp311-abi3` |

The release also includes one source distribution. Python 3.10 uses its dedicated
wheel; Python 3.11 and later use the stable-ABI wheel. The compatibility job
installs and imports the original wheels on Python 3.10–3.14 on each platform,
after the complete source suite and package builds pass.

## Add a wheel platform

The supported wheel platforms are an explicit release contract. Adding a row to
the build matrix is only the first step: candidate assembly, promotion, and PyPI
publication each reject an inventory that does not match their own platform and
ABI checks. Update all pipeline stages and the release documentation in the same
pull request.

For example, to propose macOS Intel as a fourth platform, add this row to the
`platform` matrix in
`.github/workflows/build-packages.yml`:

```yaml
- label: macos-intel
  os: macos-15-intel
  manylinux: false
  wheel_python: python3
```

The platform list is shared with the compatibility job through a YAML anchor, so
the new row automatically tests the two Intel wheels across Python 3.10–3.14
without rebuilding them. The label becomes the build artifact prefix used by that
job.

GitHub documents `macos-15-intel` as an Intel hosted-runner label in
[Choosing the runner for a job](https://docs.github.com/en/actions/how-tos/write-workflows/choose-where-workflows-run/choose-the-runner-for-a-job).
The existing Python matrix combines every platform with two configurations:
`cp310`, built with the CPython 3.10 ABI, and `abi3`, built with
`pyo3/abi3-py311`. The new platform therefore adds two wheels. PyO3 documents
that an `abi3-py311` build supports Python 3.11 and later in its
[building and distribution guide](https://pyo3.rs/main/building-and-distribution#minimum-python-version-for-abi3).
The expected macOS Intel filenames end in `macosx_*_x86_64.whl`; the Python
Packaging User Guide defines `x86_64` as the tag for a single-architecture Intel
macOS wheel in its
[platform compatibility tag specification](https://packaging.python.org/en/latest/specifications/platform-compatibility-tags/#macos).

Keep the solver setup used by the other platform rows. The build environment
sets up the statically linked HiGHS dependency, the wheel features include
`xpress` and `scip-from-source`, and the `Set up Xpress runtime` step calls
`scripts/setup_solver_runtime_env.sh`. Before enabling the Intel row, confirm
that `XPRESS_SDK_MACOS_URL` provides an x86_64-compatible SDK. If that repository
variable is unset, the script falls back to the `xpress` and `xpresslibs` Python
packages, which must also provide an Intel runtime on the Intel runner.

Update the inventory checks after adding the platform:

| Release stage                                                 | Required macOS Intel update                                                                                                                                        |
| ------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| Candidate assembly in `.github/workflows/build-candidate.yml` | Change the total from 6 to 8 wheels and each ABI count from 3 to 4. Add checks for one `cp310-cp310-macosx_*_x86_64.whl` and one `cp311-abi3-macosx_*_x86_64.whl`. |
| Promotion in `.github/workflows/promote-release.yml`          | Change the total from 6 to 8 wheels and require the two x86_64 macOS filename patterns alongside the arm64 patterns.                                               |
| PyPI publication in `.github/workflows/publish-pypi.yml`      | Change the downloaded inventory from 6 to 8 wheels and require the same two x86_64 macOS filename patterns before attestation verification and publication.        |
| Release documentation                                         | Update the distribution target table above and the published-artifact table in `RELEASE_POLICY.md` after the platform is supported.                                |

Do not change the source-distribution count or the cargo-dist native artifact
count when adding a Python wheel platform. Those inventories describe different
release products.

A new Linux architecture also needs an architecture-specific manylinux container
image and wheel filename check in `build-packages.yml`. The current
`manylinux: true` branch always selects the x86_64 image and requires a
`manylinux_2_28_x86_64` wheel. Confirm that HiGHS, bundled SCIP, and the Xpress SDK
all support the new architecture before adding its matrix row.

Run the same checks used by the matrix job on an Intel macOS host or runner:

```bash
# Export XPRESSDIR and, when required, XPAUTH_PATH for the Intel SDK first.
rm -f dist/*.whl
PYTHON_WHEEL_INTERPRETER=python3.10 \
  PYTHON_WHEEL_FEATURES=pyo3/extension-module,xpress,scip-from-source \
  just ci-python-release-wheel "dist/*.whl"

rm -f dist/*.whl
PYTHON_WHEEL_INTERPRETER=python3.11 \
  PYTHON_WHEEL_FEATURES=pyo3/extension-module,pyo3/abi3-py311,xpress,scip-from-source \
  just ci-python-release-wheel "dist/*.whl"
```

Each command prepares the HiGHS and bundled SCIP build environment, builds one
release wheel, installs it into an isolated environment, and imports `arco`.
Run the helper-script tests and Python suite as separate regression checks:

```bash
just script-test
just py-test
```

After merging the matrix and inventory changes, approve the new pending candidate
run on the updated release PR and inspect its output before promotion. The expanded
inventory must contain exactly eight wheels: the six current targets in the table
above plus the two macOS Intel wheels, split into four `cp310-cp310` wheels and
four `cp311-abi3` wheels. It must still contain one source distribution.
