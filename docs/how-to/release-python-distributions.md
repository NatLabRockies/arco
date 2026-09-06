# Release Python distributions

Choose the cutoff through the Release Please PR as described in
[Release Policy](../../RELEASE_POLICY.md). Update the release branch if needed,
then run the candidate workflow with the release PR number. It builds and tests
six wheels and one sdist before a version tag exists.

Review the successful candidate and approve its run through the promotion workflow.
Promotion merges the unchanged release PR, lets Release Please create the tag and
draft, and publishes the original files to an immutable GitHub Release.

After release verification, promotion dispatches `publish-pypi.yml` at the tag.
That workflow downloads the six wheels and sdist, verifies their GitHub release
attestations, and publishes through PyPI trusted publishing. Check its result
separately from the promotion run. The workflow accepts stable `vX.Y.Z` tags.

## Retry publication

Use Re-run failed jobs for candidate or release verification failures. If the
candidate source changes before approval, build and review a new candidate.

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
wheel; Python 3.11 and later use the stable-ABI wheel.
