# tree-sitter-arco-kdl

Thin overlay grammar on top of `tree-sitter-kdl` for Arco surface syntax.

## Why this exists

Arco accepts math blocks like:

```kdl
constraint balance {
  sum(generation[a,t] for a in assets) + unserved_energy[t] = demand[t]
}
```

This is not plain KDL node content, so stock `tree-sitter-kdl` flags it.
Arco rewrites these blocks before parsing (`normalize_surface_syntax`), but editor tooling needs to parse the source form directly.

## Overlay behavior

- Keeps normal KDL parsing from `tree-sitter-kdl`.
- Adds a fallback for block bodies containing algebra text instead of child nodes.
- Exposes algebra text as `arco_math_text` so editor injections can apply a math grammar.

## Files

- `grammar.js`: KDL overlay grammar.
- `queries/injections.scm`: marks `arco_math_text` for language injection.
- `test/corpus/arco_math.txt`: corpus examples for algebra-body parsing.

## Notes

- This is intentionally thin and non-invasive.
- It does not define a full math grammar. Pair it with a `tree-sitter-arco-math` (or equivalent) grammar via injection.
