local M = {}

local function plugin_root()
  local source = debug.getinfo(1, "S").source
  if source:sub(1, 1) == "@" then
    source = source:sub(2)
  end
  return source:gsub("/lua/arco%-kdl/init%.lua$", "")
end

function M.setup(opts)
  opts = opts or {}

  local ok, parsers = pcall(require, "nvim-treesitter.parsers")
  if not ok then
    vim.notify("arco-kdl requires nvim-treesitter", vim.log.levels.ERROR)
    return
  end

  local parser_config = parsers.get_parser_configs()
  local filetype = opts.filetype or "kdl"

  parser_config.arco_kdl = {
    install_info = {
      url = opts.url or plugin_root(),
      files = { "src/parser.c", "src/scanner.c" },
      requires_generate_from_grammar = false,
    },
    filetype = filetype,
  }

  vim.treesitter.language.register("arco_kdl", filetype)
end

return M
