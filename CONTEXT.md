# Arco Solver Integration

This context defines the canonical terms for how Arco discovers, configures, validates, and invokes solver backends across CLI and Python.

## Language

**Solver registry**:
The catalog of solver families Arco knows how to configure, validate, and invoke through one common contract.
_Avoid_: plugin system, ad-hoc backend match table

**Solver family**:
The stable solver type in the registry that defines capabilities, option schema, and invocation semantics.
_Avoid_: installation, profile, endpoint

**Solver profile**:
A named installation and launch configuration for one solver family in a specific environment.
_Avoid_: backend type, capability set

**Solver selection**:
The user-facing choice of solver family or solver profile used for a solve.
_Avoid_: backend enum, endpoint string

**Embedded solver backend**:
A solver backend that Arco invokes through a linked library or crate in the same process.
_Avoid_: external solver, shell backend

**External-process solver backend**:
A solver backend that Arco invokes by locating and running a separately installed solver executable.
_Avoid_: embedded backend, built-in solver

**Solver transport**:
The invocation mode a solver family uses, such as embedded library or external process.
_Avoid_: treating installation details and invocation mode as the same thing

**Solver option schema**:
The typed description of configurable solver options, including common Arco options and optionally family-specific options.
_Avoid_: unstructured blob, backend-specific ad-hoc flags

**Generic solver API**:
The canonical public API centered on solver selections, profiles, and typed options rather than per-solver classes.
_Avoid_: backend-specific primary APIs, hardcoded solver enums

**Capability enforcement policy**:
The rule that unsupported solver capabilities should fail fast rather than degrade silently.
_Avoid_: implicit fallback, best-effort feature dropping

**Solver preflight**:
The central validation step that resolves a solver selection and checks model and option compatibility before backend invocation.
_Avoid_: backend-only late validation, trial-and-error solver startup

**Solver capability model**:
The structured description of solver support levels, using booleans only where the capability is truly binary.
_Avoid_: boolean-only capability maps for nuanced features

**Solver profile schema**:
The constrained typed shape of a solver profile, including executable location, arguments, environment, and option values without arbitrary shell logic.
_Avoid_: shell template DSL, programmable launcher config

**Solver requirements**:
The inferred and user-declared capability constraints a solve must satisfy before a solver can run.
_Avoid_: implicit backend guesses, model-embedded solver names

**Solver availability**:
The distinction between a solver family being registered, compiled into the build, and currently usable on a machine.
_Avoid_: treating unknown, unsupported, and unavailable as the same state

**Solver introspection model**:
The stable machine-readable representation of solver families, profiles, capabilities, schemas, availability, and preflight results.
_Avoid_: CLI-only tables, duplicated ad-hoc inspection formats

**Solver result envelope**:
The generic solve result that carries common solution data plus optional typed solver artifacts.
_Avoid_: backend-specific primary result objects, unstructured metadata dumps

**Solver config document**:
The persisted TOML configuration that stores solver profiles, defaults, and active selections.
_Avoid_: backend-only JSON stub, model-embedded solver config

**External-process protocol**:
The contract an external-process solver backend uses to exchange model inputs, options, and results with a solver executable.
_Avoid_: ad-hoc shell script integration, opaque subprocess glue

**Solver IR boundary**:
The canonical lowered representation Arco passes into solver backends before any backend-specific translation or export.
_Avoid_: high-level expression graph, duplicated backend inputs

## Relationships

- A **Solver registry** contains **Solver families**.
- A **Solver family** supports one or more **Solver transports**.
- A **Solver family** may be implemented as an **Embedded solver backend** or an **External-process solver backend**.
- A **Solver family** may have zero or more **Solver profiles**.
- A **Solver profile** chooses the **Solver transport** used for invocation.
- A **Solver profile** belongs to exactly one **Solver family**.
- A **Solver selection** resolves to either a **Solver family** directly or a **Solver profile** that belongs to one.
- A **Solver family** may publish a **Solver option schema**.
- A **Solver profile** supplies values against the **Solver option schema** of its **Solver family**.
- A **Generic solver API** resolves through the **Solver registry**.
- A **Capability enforcement policy** is evaluated against the capabilities of the resolved **Solver family** and **Solver profile**.
- A **Solver preflight** applies the **Capability enforcement policy** before solver invocation.
- A **Solver capability model** informs **Solver preflight** and user-facing introspection.
- A **Solver preflight** checks **Solver requirements** against the **Solver capability model**.
- A **Solver family** has a **Solver availability** state.
- A **Solver introspection model** reports **Solver availability**, capabilities, profiles, schemas, and preflight results.
- A **Solver result envelope** may include optional typed artifacts gated by the **Solver capability model**.
- A **Solver config document** stores **Solver profiles** and default **Solver selections**.
- An **External-process solver backend** uses an **External-process protocol**.
- A **Solver family** consumes the **Solver IR boundary**.

## Example dialogue

> **Dev:** "If I select `gurobi`, what happens when there are multiple Gurobi installs?"
> **Domain expert:** "Arco should auto-resolve only when exactly one **Solver profile** exists for that **Solver family**; otherwise the user must pick a profile or configure a default."

## Flagged ambiguities

- Solver integration ambiguity resolved: Arco should use a **Solver registry** rather than scattered backend-specific dispatch.
- Backend transport ambiguity resolved: a solver backend may be either **Embedded** or **External-process**.
- Solver identity ambiguity resolved: the registry should model **Solver families**, while installations should be captured as **Solver profiles**.
- Naming ambiguity resolved: use **Solver profile** rather than "endpoint" for a configured solver installation.
- Selection ambiguity resolved: a **Solver selection** may name either a **Solver family** or a **Solver profile**.
- Name resolution ambiguity resolved: **Solver profile** names must not collide with **Solver family** names.
- Family selection ambiguity resolved: selecting a **Solver family** should auto-resolve when exactly one **Solver profile** exists for that family; otherwise Arco should require explicit selection or a configured default.
- Option modeling ambiguity resolved: use a hybrid **Solver option schema** with typed common options, optional typed family-specific options, and passthrough as an escape hatch.
- Process integration ambiguity resolved: use a file-first **External-process protocol** with optional custom hooks.
- Solver input boundary resolved: the **Solver IR boundary** should be the lowered algebraic representation rather than `arco_core::Model`.
- Delivery scope resolved: the first implementation slice should be a full end-to-end solver architecture redesign rather than an incremental capability-only or registry-only change.
- Compatibility scope resolved: this redesign may make a clean break across CLI, Python, and Rust internal solver APIs.
- Public API direction resolved: use a **Generic solver API** as the canonical surface, with family-specific helpers as optional sugar.
- Capability behavior resolved: use a strict **Capability enforcement policy** that errors on unsupported requested or required features.
- Validation placement resolved: use central **Solver preflight** with backend defensive validation.
- Capability modeling resolved: use a layered **Solver capability model** with structured values where nuance matters.
- Profile modeling resolved: use a constrained **Solver profile schema** rather than programmable launcher logic.
- Requirement sourcing resolved: **Solver requirements** should combine inferred model/solve needs with explicit user-declared constraints.
- Availability modeling resolved: distinguish registered, compiled, and currently usable **Solver availability** states.
- Introspection direction resolved: use a machine-readable-first **Solver introspection model** and render CLI views from it.
- Result modeling resolved: use a **Solver result envelope** with optional typed artifacts.
- Persistence format resolved: use a TOML **Solver config document** for solver profiles and defaults.
- Config scope resolved: support both user-scoped and project-scoped **Solver config documents** with layered resolution.
- Cross-scope profile resolution resolved: same-name **Solver profiles** should deep-merge across project and user config, with user values winning.
- Profile composition resolved: do not support named profile inheritance beyond layered cross-scope merging.
- CLI validation UX resolved: do not add standalone preflight/probe commands; compatibility errors should surface through existing solve and inspection flows.
- Profile validation timing resolved: availability and compatibility checks should happen at solve time rather than during profile save/edit flows.
- Inspection UX resolved: solver show/list views should include best-effort live availability status inline.
- Transport constraint resolved: **Solver requirements** may include transport-level constraints such as embedded-only or external-process-only.
- Default selection persistence resolved: store the exact **Solver selection** the user set rather than an eagerly resolved profile.
- Profile naming scope resolved: **Solver profile** names should be globally unique.
- Alias policy resolved: do not support solver family or profile aliases in v1.
- Documentation source resolved: the **Solver option schema** should be the canonical source for CLI help, Python help, and docs.
- Config migration policy resolved: do not migrate legacy solver JSON config; require explicit adoption of the new TOML **Solver config document**.
- Python API exposure resolved: expose registry and profile concepts as first-class Python types rather than primarily strings and dicts.
- Cross-interface semantics resolved: CLI and Python should share the same canonical **Solver selection** semantics and resolution rules.
- Capability source ambiguity resolved: capability and availability data should come from both family-level declarations and profile/runtime state.
- cuOpt transport ambiguity resolved: cuOpt is a multi-transport **Solver family** with an embedded-library path and a server path; v1 should target the embedded path first.
- Transport scope resolved: v1 should model only embedded and external-process **Solver transports**; remote-service transport is deferred.
- Roadmap scope resolved: any v1 solver integration must fit the v1 transport model, so cuOpt v1 must use the embedded path and remote-service support must be deferred.
- Issue scope resolved: repurpose issue #145 from a narrow capability-matrix task into the umbrella solver architecture redesign issue.
- Cutover strategy resolved: switch to the new solver architecture all at once rather than running old and new paths in parallel.
- Migration scope resolved: all solver families currently exposed by Arco at cutover time must migrate to the new architecture.
- Architectural intent resolved: the redesign is optimized for future **Solver family** onboarding even at some short-term implementation cost.
- Performance constraint resolved: the new solver architecture must not add an extra full-model materialization beyond the chosen **Solver IR boundary** unless a transport genuinely requires it.
- Secret handling resolved: **Solver config documents** should store only references to credentials or license material, not raw secret values.
- Config evolution resolved: **Solver config documents** should carry an explicit schema version from day one.
- Transport selection resolved: **Solver transport** is chosen by the **Solver profile**, not fixed solely by the **Solver family**.
- Built-in runnable default resolved: a **Solver family** may be runnable without an explicit **Solver profile** when Arco provides an implicit built-in default.
- Built-in default modeling resolved: implicit runnable defaults should be represented internally as synthesized **Solver profiles**.
- Introspection visibility resolved: synthesized built-in **Solver profiles** should appear in the **Solver introspection model** and user-facing listings.
- Implementation sequencing resolved: fully implement the core registry architecture milestone before public-surface redesign work.
- Transitional compatibility resolved: during local-first implementation, temporary user-facing breakage is acceptable while milestone A is completed.
- Planning scope resolved: keep decisions at ADR/context level for now without decomposing into child implementation issues yet.
