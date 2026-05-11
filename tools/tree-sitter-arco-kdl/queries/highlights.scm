; Arco KDL Tree-sitter Highlight Queries
; Keep structural elements distinct:
; - Predicates/keywords (`data`, `param`, `set`, ...)
; - Node names (positional values)
; - Properties (`from=generator`, etc.)

; Types
(type) @type
(annotation_type) @type.builtin

; Node predicate / declaration keywords (default KDL nodes)
(kdl_node (identifier) @function)

((kdl_node
  (identifier) @keyword)
 (#any-of? @keyword
  "include"
  "set"
  "data"
  "param"
  "model"
  "scenario"
  "map"
  "index"
  "in"
  "filter"
  "control"
  "expression"
  "constraint"
  "minimize"
  "maximize"
  "if"
  "lower"
  "upper"
  "bounds"
  "slack"
  "report"
  "use"
  "use_data"
  "reduce"))

; Arco-specific literal-name nodes
[
  "constraint"
  "expression"
  "expr"
  "minimize"
  "maximize"
  "filter"
  "if"
  "lower"
  "upper"
] @keyword

(arco_pure_math_node
  name: (
    "expression"
    "minimize"
    "maximize"
    "expr"
    "filter"
    "if"
    "lower"
    "upper"
  ) @keyword)

(arco_constraint_node
  name: "constraint" @keyword)

; Node names (e.g. `set thermal`, `model dispatch_model`)
(node_field
  (value
    [
      (bare_identifier)
      (string)
    ] @variable.parameter))

; Properties (`from=...`, `index=...`, etc.)
(prop (identifier) @property)
(prop (value (bare_identifier) @property))
(prop (value (string) @property))

; Operators
[
  "="
  "+"
  "-"
] @operator

; Literals
(string) @string
(escape) @string.escape
(number) @number
(boolean) @boolean
"null" @constant.builtin

; Punctuation
[
  "{"
  "}"
  "("
  ")"
] @punctuation.bracket

[
  ";"
] @punctuation.delimiter

; Comments
(single_line_comment) @comment
(multi_line_comment) @comment

; Opaque algebra payloads (for injected math grammar)
(arco_math_text) @string.special
(arco_constraint_math_text) @string.special
