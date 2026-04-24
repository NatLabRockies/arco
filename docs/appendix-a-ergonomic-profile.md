# Appendix A. Ergonomic syntax profile

This section defines a supported ergonomic authoring profile. Implementations
that claim support for this profile MUST accept the forms in this section and
lower them to canonical
[§1](arco-spec.md#1-conformance)–[§12](arco-spec.md#12-algebra-expression-summary)
forms before model execution.

All forms in this section are valid KDL 2.0 nodes, properties, and child blocks.

## A.1 Capability summary

| Capability                                     | Form                                | Canonical expansion target                     |
| ---------------------------------------------- | ----------------------------------- | ---------------------------------------------- |
| Data imports                                   | `use_data` in `model`               | model-local `set` and `param` declarations     |
| Edge domains _(usage pattern, not new syntax)_ | network-topology naming conventions | reusable filtered domains for graph structures |

> [!NOTE] The `set { in <parent>; filter { ... } }` syntax itself is canonical
> (§5.2). The "Edge domains" capability here refers to recommended patterns for
> using that syntax to model network topology (e.g., active branches, connected
> buses), not additional syntax.

## A.2 `use_data` model imports

Syntax:

```kdl
model <name> {
  use_data <data_name> <data_name> ...
}
```

Semantics:

- Each referenced `<data_name>` MUST resolve to a top-level `data` block.
- Imported symbols are restricted to eligible `set` and `param` declarations.
- Explicit model-local declarations with the same name override imported
  declarations.
- If two `use_data` references import declarations with the same name, the
  conflict MUST fail validation unless an explicit model-local declaration
  overrides both. "Eligible" declarations are all `set` and `param` nodes
  declared directly inside the referenced `data` block (including subsets
  declared with `in`).

## A.3 Edge-domain patterns

This section documents recommended patterns for modeling network topology (graph
structures such as transmission lines, pipelines, or transport arcs) using the
canonical `set { in ...; filter { ... } }` syntax defined in
[§5.2](arco-spec.md#52-set-inside-data). No additional syntax is introduced.
This is a usage guide, not new grammar.

```kdl
data branch_data source="data/branches.csv" {
  set edge
  set active_edge { in edge; filter { conex == 1 } }
  param conex index=edge
}
```

> [!TIP]
>
> - Edge-domain sets SHOULD produce reusable domain members for constraint
>   generation (e.g., iterating over active branches in a power flow
>   constraint).
> - Use `filter` to distinguish connected vs. disconnected edges, directional
>   subsets, or capacity tiers.
>
> Tuple-domain feasible-set pattern for nodal allocation:
>
> ```kdl
> data links source="data/links.csv" {
>   set area as=a
>   set tech as=i
>   alias generators column=gen
>   alias buses column=bus
>   set generators as=g
>   set buses as=b
>
>   set feasible_links {
>     index a { in area }
>     index i { in tech }
>     index g { in generators }
>     index b { in buses }
>     filter { feasible > 0 }
>   }
> }
>
> set priority_links {
>   in feasible_links
>   index a { in area }
>   index i { in tech }
>   index g { in generators }
>   index b { in buses }
>   filter { area == "south" }
> }
>
> model nodal_allocation {
>   control dispatch lower=0 {
>     index a { in feasible_links }
>     index i { in feasible_links }
>     index g { in feasible_links }
>     index b { in feasible_links }
>   }
>
>   constraint priority_floor {
>     index a { in priority_links }
>     index i { in priority_links }
>     index g { in priority_links }
>     index b { in priority_links }
>     expression { dispatch[a,i,g,b] >= 0 }
>   }
> }
> ```
>
> In V1, tuple-domain variables mean membership in `feasible_links`, not
> Cartesian expansion over `area × tech × generators × buses`. Projection stays
> explicit: define a named subset and iterate that subset directly. If the index
> order is wrong, diagnostics report both the scoped source ID and the canonical
> tuple domain. Empty-subset diagnostics list every offending key in
> deterministic tuple order.

## A.4 Ergonomic grammar

The following EBNF productions define the ergonomic forms introduced in this
appendix. These desugar into the canonical grammar defined in
[§11](arco-spec.md#11-grammar-low-level-profile).

```ebnf
use_data_decl     := "use_data" name { name }
```

> [!NOTE] `use_data_decl` is valid only inside `model_block`. When the ergonomic
> profile is active, `model_block` accepts `use_data_decl` in addition to the
> canonical children defined in
> [§11](arco-spec.md#11-grammar-low-level-profile).

## A.5 Lowering and diagnostics requirements

- Ergonomic forms MUST preserve semantics of canonical expansion.
- Diagnostics SHOULD reference the original ergonomic source span.
- `arco print-model` and solver export operate on canonical lowered output.
- If ergonomic and explicit canonical declarations conflict in the same scope,
  explicit canonical declarations take precedence.
