# tree-sitter-arco-kdl

Thin overlay grammar on top of `tree-sitter-kdl` for Arco surface syntax.

## Why this exists

Arco extends KDL 2.0 with algebra blocks — bare math expressions inside `{ }`:

```kdl
constraint balance {
  sum(generation[a,t] for a in assets) + unserved_energy[t] = demand[t]
}

set thermal {
  in gen
  filter { type == thermal }
}
```

This is not valid KDL node content, so stock `tree-sitter-kdl` flags it as an
error. This grammar extends `tree-sitter-kdl` to recognize algebra blocks in the
correct contexts, giving editors clean parses with no red squiggles.

## Overlay behavior

- Keeps normal KDL parsing from `tree-sitter-kdl` for structural nodes (`data`,
  `model`, `scenario`, `control`, `set`, `param`, etc.).
- **Pure math nodes** (`expression`, `minimize`, `maximize`, `filter`, `if`,
  `lower`, `upper`) always parse their `{ }` body as algebra text.
- **Mixed math nodes** (`constraint`) try KDL children first (generated form
  with `index`, `if`, `expression` children), falling back to algebra text
  (simple form).
- Exposes algebra text as `arco_math_text` so editor injections can apply a math
  grammar for syntax highlighting.

## Highlight capture map (theme tuning)

Use `examples/highlight_demo.kdl` as the visual fixture when tuning colors.

| Semantic intent                         | Tree-sitter capture(s)                           |
| --------------------------------------- | ------------------------------------------------ |
| Predicates / declaration keywords       | `@keyword`                                       |
| Node names (`set thermal`, `model m`)   | `@variable.parameter`                            |
| Properties (`from=...`, `index=...`)    | `@property`                                      |
| Operators (`=`, `+`, `-`)               | `@operator`                                      |
| Literals (`"..."`, numbers, booleans)   | `@string`, `@number`, `@boolean`                 |
| Braces/parens/semicolons                | `@punctuation.bracket`, `@punctuation.delimiter` |
| Comments                                | `@comment`                                       |
| Algebra payload text (before injection) | `@string.special`                                |

If your editor supports capture inspection (e.g. Neovim `:Inspect`), open the
fixture and verify these captures line-by-line before taking screenshots.

## Installation

### Neovim

Add this parser to your Neovim tree-sitter config. With `nvim-treesitter`:

```lua
local parser_config = require("nvim-treesitter.parsers").get_parser_configs()

parser_config.arco_kdl = {
  install_info = {
    url = "path/to/tools/tree-sitter-arco-kdl",
    files = { "src/parser.c" },
    requires_generate_from_grammar = false,
  },
  filetype = "kdl",
}
```

Then run `:TSInstall arco_kdl`.

### VS Code

Use the
[Tree-sitter for VS Code](https://marketplace.visualstudio.com/items?itemName=piotrminkowski.tree-sitter-syntax)
extension or package this grammar as a VS Code extension with a `syntaxes/`
contribution.

### Helix

Add to `languages.toml`:

```toml
[[language]]
name = "arco-kdl"
scope = "source.arco_kdl"
file-types = ["kdl"]
grammar = "arco_kdl"

[[grammar]]
name = "arco_kdl"
source = { path = "path/to/tools/tree-sitter-arco-kdl" }
```

Then run `hx --grammar build`.

## Files

- `grammar.js`: KDL overlay grammar.
- `queries/injections.scm`: marks `arco_math_text` for language injection.
- `examples/highlight_demo.kdl`: semantic highlight fixture for theme tuning.
- `test/corpus/arco_math.txt`: corpus examples for algebra-body parsing.

## Notes

- This is intentionally thin and non-invasive.
- It does not define a full math grammar. Pair it with a `tree-sitter-arco-math`
  (or equivalent) grammar via injection for full algebra highlighting.
