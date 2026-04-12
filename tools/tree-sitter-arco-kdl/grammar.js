const kdl = require("tree-sitter-kdl/grammar");

const nodeShape = ($, nameRule, childrenRule) =>
  seq(
    alias(optional(seq("/-", repeat($._node_space))), $.node_comment),
    optional($.type),
    nameRule,
    repeat(seq(repeat1($._node_space), $.node_field)),
    optional(
      seq(
        repeat($._node_space),
        field("children", childrenRule),
        repeat($._ws),
      ),
    ),
    repeat($._node_space),
    $._node_terminator,
  );

module.exports = grammar(kdl, {
  name: "arco_kdl",

  externals: ($, previous) => [...previous, $._implicit_terminator],

  rules: {
    // Allow nodes to be implicitly terminated before `}`.
    _node_terminator: ($, previous) =>
      choice(previous, $._implicit_terminator),

    value: ($) =>
      seq(
        optional($.type),
        choice($.string, $.number, $.keyword, $.bare_identifier),
      ),

    bare_identifier: ($) => $._bare_identifier,

    node: ($) =>
      choice($.arco_pure_math_node, $.arco_constraint_node, $.kdl_node),

    kdl_node: ($) => prec(1, nodeShape($, $.identifier, $.node_children)),

    // Nodes whose { } body is always algebra text.
    arco_pure_math_node: ($) =>
      prec(
        2,
        nodeShape(
          $,
          field(
            "name",
            choice(
              "expression",
              "minimize",
              "maximize",
              "expr",
              "filter",
              "if",
              "lower",
              "upper",
            ),
          ),
          $.arco_pure_math_children,
        ),
      ),

    // Constraint nodes can have either KDL children or a math body.
    arco_constraint_node: ($) =>
      prec(
        2,
        nodeShape(
          $,
          field("name", "constraint"),
          choice($.arco_constraint_math_children, $.node_children),
        ),
      ),

    // Math body for nodes whose braces are always algebra text.
    arco_pure_math_children: ($) =>
      prec(
        2,
        seq(
          optional(
            seq(alias("/-", $.node_children_comment), repeat($._node_space)),
          ),
          "{",
          choice(field("math", $.arco_math_text), seq(repeat($._linespace))),
          "}",
        ),
      ),

    // Constraint math body remains stricter so child-node bodies keep parsing
    // as KDL instead of being swallowed as free-form math text.
    arco_constraint_math_children: ($) =>
      prec(
        2,
        seq(
          optional(
            seq(alias("/-", $.node_children_comment), repeat($._node_space)),
          ),
          "{",
          choice(
            field("math", $.arco_constraint_math_text),
            seq(repeat($._linespace)),
          ),
          "}",
        ),
      ),

    // Single opaque token for free-form algebra text in expression/minimize/
    // maximize/filter/if/lower/upper nodes.
    arco_math_text: (_) => token(prec(10, /[^{}"']+/)),

    // Constraint math must include an operator or bracket so bare KDL child
    // nodes like `if { ... }` still parse through node_children.
    arco_constraint_math_text: (_) =>
      token(prec(10, /[^{}"']*[<>=!+\-*\/\[\]][^{}"']*/)),
  },
});
