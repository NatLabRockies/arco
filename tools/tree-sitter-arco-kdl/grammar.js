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

  rules: {
    value: ($) =>
      seq(
        optional($.type),
        choice($.string, $.number, $.keyword, $._bare_identifier),
      ),

    node: ($) => choice($.arco_math_node, $.kdl_node),

    kdl_node: ($) => prec(1, nodeShape($, $.identifier, $.node_children)),

    arco_math_node: ($) =>
      prec(
        2,
        nodeShape(
          $,
          field(
            "name",
            choice(
              "constraint",
              "expression",
              "minimize",
              "maximize",
              "expr",
              "lower",
              "upper",
            ),
          ),
          $.arco_node_children,
        ),
      ),

    arco_node_children: ($) =>
      prec(
        2,
        seq(
          optional(
            seq(alias("/-", $.node_children_comment), repeat($._node_space)),
          ),
          "{",
          choice(
            prec(
              2,
              seq(
                repeat($._linespace),
                $.node,
                repeat(seq(repeat($._linespace), $.node)),
                repeat($._linespace),
              ),
            ),
            prec(1, field("math", $.arco_math_text)),
            seq(repeat($._linespace)),
          ),
          "}",
        ),
      ),

    arco_math_text: (_) => token(prec(10, /[^{}"']*[<>+\-*\/\[\]][^{}"']*/)),
  },
});
