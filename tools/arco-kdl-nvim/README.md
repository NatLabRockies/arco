# arco KDL for Neovim

Lazy.nvim-ready Neovim plugin for Arco KDL tree-sitter highlighting.

## Install With lazy.nvim

From this repository checkout:

```lua
{
  dir = "/absolute/path/to/arco/tools/arco-kdl-nvim",
  dependencies = { "nvim-treesitter/nvim-treesitter" },
  config = function()
    require("arco-kdl").setup()
  end,
}
```

From GitHub, lazy.nvim clones the repository root, so add the plugin
subdirectory to `runtimepath` before requiring it:

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

Open a `.kdl` file and verify:

```vim
:set filetype?
:lua print(vim.treesitter.language.get_lang(vim.bo.filetype))
:lua print(vim.inspect(vim.treesitter.query.get_files("arco_kdl", "highlights")))
```

Expected:

- `filetype=kdl`
- language resolves to `arco_kdl`
- highlights query list is non-empty

This plugin includes generated parser sources and Arco highlight/injection
queries, so parser installation does not require npm or grammar generation.

## Test

From the repository root:

```sh
just kdl-nvim-test
```
