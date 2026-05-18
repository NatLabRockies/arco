local plugin_dir = assert(
  vim.env.ARCO_KDL_NVIM_PLUGIN_DIR,
  "ARCO_KDL_NVIM_PLUGIN_DIR must be set"
)

local parser_configs = {}

package.loaded["nvim-treesitter.parsers"] = {
  get_parser_configs = function()
    return parser_configs
  end,
}

local registered_language
local registered_filetype
local original_register = vim.treesitter.language.register
vim.treesitter.language.register = function(language, filetype)
  registered_language = language
  registered_filetype = filetype
end

local plugin = require("arco-kdl")
assert(type(plugin.setup) == "function", "setup function is exported")

plugin.setup()

assert(parser_configs.arco_kdl, "arco_kdl parser config is registered")
assert(parser_configs.arco_kdl.filetype == "kdl", "default filetype is kdl")
assert(
  parser_configs.arco_kdl.install_info.url:match("/tools/arco%-kdl%-nvim$"),
  "default install url points at tools/arco-kdl-nvim"
)
assert(
  parser_configs.arco_kdl.install_info.files[1] == "src/parser.c",
  "parser.c is registered"
)
assert(
  parser_configs.arco_kdl.install_info.files[2] == "src/scanner.c",
  "scanner.c is registered"
)
assert(
  parser_configs.arco_kdl.install_info.requires_generate_from_grammar == false,
  "install uses generated parser sources"
)
assert(registered_language == "arco_kdl", "tree-sitter language is registered")
assert(registered_filetype == "kdl", "tree-sitter language maps to kdl")

plugin.setup({ filetype = "arco-kdl-test", url = "/tmp/arco-kdl-parser" })
assert(
  parser_configs.arco_kdl.filetype == "arco-kdl-test",
  "custom filetype is honored"
)
assert(
  parser_configs.arco_kdl.install_info.url == "/tmp/arco-kdl-parser",
  "custom parser url is honored"
)
assert(registered_filetype == "arco-kdl-test", "custom filetype is registered")

vim.treesitter.language.register = original_register

local highlight_files = vim.treesitter.query.get_files("arco_kdl", "highlights")
local injection_files = vim.treesitter.query.get_files("arco_kdl", "injections")
assert(#highlight_files > 0, "highlight query is discoverable")
assert(#injection_files > 0, "injection query is discoverable")

dofile(plugin_dir .. "/ftdetect/kdl.lua")
assert(
  vim.filetype.match({ filename = "example.kdl" }) == "kdl",
  "kdl extension filetype detection is registered"
)
