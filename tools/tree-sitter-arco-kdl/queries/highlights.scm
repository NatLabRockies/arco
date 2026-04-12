; Arco KDL Tree-sitter Highlight Queries
; Extends base KDL with Arco-specific node highlighting

; Types
(node (identifier) @type)
(type) @type
(annotation_type) @type.builtin

; Properties
(prop (identifier) @property)

; Variables
(identifier) @variable

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

; Arco-specific nodes (arco_kdl extension over base KDL)
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
  ) @keyword.function)

(arco_constraint_node
  name: "constraint" @keyword.function)

(arco_math_text) @string.special
