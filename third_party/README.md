# Third-Party Patches

This directory contains small Cargo patches for upstream crates when the
workspace needs a behavior-preserving build fix before the change is available
from crates.io.

## `russcip-0.9.1`

`russcip` enables `scip-sys` default features on its dependency edge even when
Arco uses the bundled SCIP path. The bundled path has prebuilt bindings, so
compiling `bindgen` for SCIP is unnecessary. The vendored patch keeps the crate
API and version unchanged while disabling default features on the `scip-sys`
dependency; `russcip/bundled` still enables `scip-sys/bundled`.

## `scip-sys-0.1.28`

`scip-sys/bundled` downloads a pinned SCIP release and uses prebuilt bindings,
but the upstream feature compiles Rust HTTP/TLS/ZIP dependencies only to fetch
and unpack that build-time archive. The vendored patch keeps bundled SCIP
enabled and version-pinned while:

- accepting `SCIP_SYS_BUNDLED_DIR_<target>` or `SCIP_SYS_BUNDLED_DIR` for a
  pre-extracted SCIP install,
- preserving an external `curl` + Python fallback download for local builds,
- removing the unused library dependency on `cmake`, and
- removing Rust download/archive crates from the bundled feature graph.
