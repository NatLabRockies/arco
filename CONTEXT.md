# Arco DSL Authoring

This context defines canonical terms for Arco KDL syntax around tuple-domain indexing and constraint generation.

## Language

**Tuple-domain set**:
A set declared with multiple `index` children representing valid tuples, not Cartesian products.
_Avoid_: relation set, table set

**Set unpacking index**:
`index <set_name>` binds canonical component variables from the referenced set into expression scope. For tuple-domain sets, all tuple components are exposed; for non-tuple sets, one component is exposed.
_Avoid_: treating tuple and non-tuple shorthand as unrelated semantics

**Index binding alias**:
`index <var> { in <set> }` introduces `<var>` as a local alias for iterating `<set>` within the current declaration scope.
_Avoid_: reading `<var>` as a new set name

**Canonical component names**:
When tuple unpacking is used, exposed variables must match the tuple-domain set component names exactly (for example `a,i,g,b`) and cannot be locally renamed at constraint site.
_Avoid_: local axis renaming, ad-hoc aliases inside constraint

**Tuple-key indexing**:
For tuple-domain indexed symbols, both expanded indexing (`investment[a,i,g,b]`) and tuple-key indexing (`investment[feasible_links]`) are valid, with tuple-key indexing as the preferred entrypoint. In v1, tuple-key tokens are set-name based (not alias based).
_Avoid_: forcing only expanded arity access

**Access preference rule**:
When `index <var> { in <set> }` is used, prefer `<var>` in indexed expressions for non-tuple sets. When shorthand `index <set>` is used, prefer `<set>`.
_Avoid_: mixing alias and set-name access styles within the same declaration without reason

**Subset-compatible tuple key**:
A symbol indexed by a parent tuple-domain set may be accessed with a child tuple-domain subset key when the subset is declared `in <parent>` and has the same component signature/order.
_Avoid_: requiring parent-key-only access for subset-filtered constraints

**Acceptance example**:
A file-backed `.kdl` model in `examples/` intended to run through CLI end-to-end flows and demonstrate supported authoring patterns.
_Avoid_: treating narrow test fixtures as user-facing examples

**Spec-negative example**:
A file-backed `.kdl` case intentionally designed to fail parser/semantic/compile validation with asserted diagnostics.
_Avoid_: encoding invalid cases as inline string literals inside Rust tests

**Test fixture**:
A test-scoped file-backed artifact used by Rust tests to exercise one precise behavior contract.
_Avoid_: labeling fixture-only models as living examples

## Relationships

- A **Set unpacking index** references one declared set.
- A **Set unpacking index** exposes canonical component variables for that set in expression scope.
- An **Index binding alias** maps local variable names to declared set domains at declaration scope.
- **Canonical component names** are sourced from the referenced **Tuple-domain set** declaration.
- **Tuple-key indexing** and expanded indexing are equivalent access forms over the same tuple-domain row.
- A **Subset-compatible tuple key** is valid only when subset-parent tuple signatures are structurally compatible.
- An **Acceptance example** should be runnable in CLI smoke/e2e flows.
- A **Spec-negative example** is consumed by Rust tests that assert typed failure diagnostics.
- A **Test fixture** is consumed by Rust tests for contract coverage and is distinct from acceptance examples.

## Example dialogue

> **Dev:** "If I write `index priority_links`, can I use `a,i,g,b` directly?"
> **Domain expert:** "Yes — for tuple-domain sets, unpacking exposes each component variable in scope."

## Flagged ambiguities

- `index priority_links` previously read as one scalar index variable. Resolved: `index <set_name>` uses unpacking semantics.
- Non-tuple disambiguation resolved: unpacking also applies to non-tuple sets (single canonical component).
- Alias semantics resolved: `index t { in time }` means local alias `t` for set `time`.
- Tuple unpacking scope resolved: `index <tuple_set>` unpacking applies in generated constraints, `control`, and `param` declarations.
- Access form resolved: tuple-domain indexed symbols accept both `symbol[tuple_set]` and `symbol[a,i,g,b]`, with tuple-key form preferred.
- Non-tuple style resolved: `index <var> { in <set> }` prefers `<var>`; shorthand `index <set>` prefers `<set>`.
- Tuple alias decision: v1 tuple-key form uses set name only (`symbol[feasible_links]`), not binding alias (`symbol[link]`).
- Subset access resolved: symbols indexed by parent tuple-domain sets accept child subset tuple keys (Option A).
- Rollout strategy resolved: unpacking semantics are always-on (no compatibility gate); rationale captured in `docs/adr/0001-set-unpacking-always-on.md`.
- Migration diagnostics resolved: do not emit special legacy scalar-intent warnings for tuple-set shorthand.
- `example` naming ambiguity resolved: use **Acceptance example** for public runnable models and **Test fixture** for test-scoped artifacts.
- Validation boundary resolved: fixture correctness is asserted by Rust fixture tests, while acceptance examples are asserted by separate CLI e2e flows.
- CI scope resolved: acceptance e2e runs only when related Rust crates or `.kdl` files change.
- Migration approach resolved: inline-KDL removal will be executed as a sweeping migration, not incremental slices.
- Living-example scope resolved: only `examples/` models are treated as living examples; fixture-only `.kdl` files are contract fixtures.
- Enforcement policy resolved: inline-KDL violations are rejected in review, with behavior guidance codified in `AGENTS.md` rather than a dedicated CI guard.
