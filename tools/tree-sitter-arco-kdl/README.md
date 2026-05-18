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
  `lower`, `upper`) always parse their `{ }` body as algebra text, including
  nested projection-reduce blocks (`reduce ... { ... }`).
- **Mixed math nodes** (`constraint`) try KDL children first (generated form
  with `index`, `if`, `expression` children), falling back to algebra text
  (simple form).
- Exposes algebra text as `arco_math_text` so editor injections can apply a math
  grammar for syntax highlighting. Simple constraint bodies are exposed as
  `arco_constraint_math_text` and use the same injection language.
- Adding a future algebra-bearing predicate should be a small grammar-table
  change in `grammar.js` followed by `npm run generate` and `npm test`.

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

## Source of truth

Authored files:

- `grammar.js`: overlay grammar source of truth
- `package.json`: local Tree-sitter scripts and dependency pins
- `src/scanner.c`: thin Arco wrapper for external tokens
- `src/vendor/tree_sitter_kdl_external_scanner.inc`: vendored upstream KDL scanner
- `queries/*.scm`: editor queries
- `test/corpus/arco_math.txt`: parser regression corpus

Generated files:

- `src/parser.c`
- `src/grammar.json`
- `src/node-types.json`
- `src/tree_sitter/parser.h` (vendored tree-sitter runtime header, pinned to the current toolchain)

When `grammar.js` changes, regenerate the parser artifacts with:

```sh
npm install
npm run generate
```

Do not hand-edit `src/parser.c`. It is generated code.
Do not hand-edit `src/tree_sitter/parser.h` either. Treat it as a vendored,
mostly frozen header that only changes when we intentionally bump the
Tree-sitter CLI/runtime version.

## Installation

### Neovim

With lazy.nvim:

```lua
{
  "NatLabRockies/arco",
  name = "arco-kdl-nvim",
  dependencies = { "nvim-treesitter/nvim-treesitter" },
  config = function(plugin)
    vim.opt.runtimepath:prepend(plugin.dir .. "/tools/arco-kdl-nvim")
    require("arco-kdl").setup()
  end,
}
```

Then install the parser:

```vim
:TSInstall arco_kdl
```

For a local checkout, use:

```lua
{
  dir = "/absolute/path/to/arco/tools/arco-kdl-nvim",
  dependencies = { "nvim-treesitter/nvim-treesitter" },
  config = function()
    require("arco-kdl").setup()
  end,
}
```

Release/local artifact packaging also includes the same plugin:

```sh
just kdl-editor-artifacts
tar -tf target/editor-artifacts/arco-kdl-nvim-0.1.0.tar.gz | head
```

For direct repository installs with `nvim-treesitter`, add this parser
registration to your config:

```lua
local function register_arco_kdl()
  local parser_config = require("nvim-treesitter.parsers")
  parser_config.arco_kdl = {
    install_info = {
      url = "https://github.com/NatLabRockies/arco.git",
      location = "tools/tree-sitter-arco-kdl",
      revision = "3325d6f772077397b858ae2c54af24dd61aeefe9", -- pinned commit from fix/treesitter
      files = { "src/parser.c", "src/scanner.c" },
      queries = "queries",
      requires_generate_from_grammar = false,
    },
    filetype = "kdl",
  }
end

register_arco_kdl()
vim.api.nvim_create_autocmd("User", {
  pattern = "TSUpdate",
  callback = register_arco_kdl,
})
vim.treesitter.language.register("arco_kdl", "kdl")
```

Then install/update:

```vim
:TSInstall arco_kdl
:TSUpdate
```

#### Notes

- No custom `ftdetect` is needed.
- No local query copies are needed.
- Keep your colorscheme/theme mappings for captures like `@keyword`,
  `@variable.parameter`, and `@property`.

#### Quick verification in Neovim

```vim
:set filetype?
:lua print(vim.treesitter.language.get_lang(vim.bo.filetype))
:lua print(vim.inspect(vim.treesitter.query.get_files("arco_kdl", "highlights")))
```

Expected:

- `filetype=kdl`
- language resolves to `arco_kdl`
- highlights query list is non-empty

#### Optional sanity check

```vim
:Inspect
```

Place cursor on `set` or `constraint`; you should see `@keyword.arco_kdl`.

> [!TIP]
> If `:TSInstall arco_kdl` says "unsupported language", your registration
> likely ran too late. Ensure the snippet above executes during startup
> (before running `TSInstall`).

### VS Code

Ready-to-install local artifact:

```sh
just kdl-editor-artifacts
code --install-extension target/editor-artifacts/arco-kdl-vscode-0.1.0.vsix --force
```

The VS Code artifact packages the repository extension in
`tools/vscode-arco-kdl`. VS Code uses TextMate grammars for built-in syntax
coloring, so the extension ships a TextMate grammar aligned with the
tree-sitter capture intent plus canonical CLI diagnostics.

For either option, use these parser assets from this directory:

- parser sources: `src/parser.c`, `src/scanner.c`
- highlight queries: `queries/highlights.scm`
- injections: `queries/injections.scm`

If you are building your own extension, make sure `.kdl` files are mapped to
this grammar (language id/scope of your choice) so Arco files use `arco_kdl`
instead of generic KDL highlighting.

## Files

- `grammar.js`: KDL overlay grammar, source of truth.
- `src/scanner.c`: thin Arco-specific external scanner shim.
- `src/vendor/tree_sitter_kdl_external_scanner.inc`: vendored upstream KDL scanner implementation.
- `src/parser.c`: generated parser output.
- `queries/injections.scm`: marks `arco_math_text` for language injection.
- `examples/highlight_demo.kdl`: semantic highlight fixture for theme tuning.
- `test/corpus/arco_math.txt`: corpus examples for algebra-body parsing.

## Checks

```sh
npm install
npm test
npm run query:highlights
npm run query:injections
```

## Notes

- This is intentionally thin and non-invasive.
- It does not define a full math grammar. Pair it with a `tree-sitter-arco-math`
  (or equivalent) grammar via injection for full algebra highlighting.
