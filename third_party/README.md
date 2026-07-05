# Third-Party Patches

This directory contains small Cargo patches for upstream crates when the
workspace needs a behavior-preserving build fix before the change is available
from crates.io.

## `zip-extract-0.1.3`

`scip-sys` uses `zip-extract` only in its build script to unpack SCIP release
zip files. The upstream `zip-extract` default feature set also enables zstd,
AES, and bzip2 archive support, which forces expensive native build
dependencies that are not needed for SCIP's release archives. The vendored
patch keeps the crate API and version unchanged while narrowing the default
feature set to deflate extraction.

## `zip-0.5.13`

`scip-sys` also declares the older `zip` crate as a build dependency under its
bundled feature, but its build script extracts SCIP release zips through
`zip-extract`. The upstream `zip` default feature set enables bzip2 support,
which compiles `bzip2-sys` even though the bundled SCIP archives do not need
it. The vendored patch keeps the crate API and version unchanged while removing
bzip2 from the default feature set.
