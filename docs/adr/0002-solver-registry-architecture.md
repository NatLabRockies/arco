# Solver registry architecture for all solver integrations

> [!NOTE]
> ADR 0003 supersedes only this ADR's lowered solver IR boundary language. The solver registry, profile, capability, preflight, transport, and result-envelope decisions remain in force.

Arco will replace backend-specific solver enums and dispatch paths with a single solver registry architecture that resolves solver families, profiles, selections, capabilities, availability, preflight checks, and result artifacts through one generic model shared by CLI and Python. The canonical solver boundary is superseded by ADR 0003's `ModelView` boundary; v1 supports embedded and external-process transports, models transport on profiles, enforces strict capability checks in central preflight, uses machine-readable introspection, and stores versioned layered TOML solver config with secret references only. We chose this full cutover now—repurposing issue #145 from a capability-matrix task into the umbrella solver-architecture issue—because future solver onboarding would otherwise keep hardening scattered custom code, and the new design must preserve Arco’s memory goals by avoiding extra full-model materialization beyond the model-view boundary unless a transport genuinely requires it.

## Considered Options

- Incremental capability-matrix-only changes on top of the current architecture
- Running old and new solver paths in parallel during migration
- Treating solver selection as hardcoded per-backend CLI/Python APIs instead of a generic registry/profile model

## Consequences

- Existing public solver APIs and config may break in one all-at-once cutover.
- All solver families exposed at cutover time must migrate to the new architecture before release.
- cuOpt v1 must use the embedded path; remote-service transport is deferred from the v1 architecture.
