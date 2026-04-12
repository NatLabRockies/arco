// Wrap the KDL scanner, adding an _implicit_terminator token
// that matches zero characters when the next non-whitespace is '}'.

#include "tree_sitter/parser.h"

// Rename KDL scanner functions to avoid symbol clashes
#define tree_sitter_kdl_external_scanner_create  kdl_scanner_create
#define tree_sitter_kdl_external_scanner_destroy kdl_scanner_destroy
#define tree_sitter_kdl_external_scanner_scan    kdl_scanner_scan
#define tree_sitter_kdl_external_scanner_serialize   kdl_scanner_serialize
#define tree_sitter_kdl_external_scanner_deserialize kdl_scanner_deserialize

#include "../node_modules/tree-sitter-kdl/src/scanner.c"

// The external token indices must match the order in grammar.js externals.
// externals: [_eof, multi_line_comment, _implicit_terminator]
enum {
  KDL_EOF = 0,
  KDL_MULTI_LINE_COMMENT = 1,
  IMPLICIT_TERMINATOR = 2,
};

void *tree_sitter_arco_kdl_external_scanner_create(void) {
  return kdl_scanner_create();
}

void tree_sitter_arco_kdl_external_scanner_destroy(void *payload) {
  kdl_scanner_destroy(payload);
}

unsigned tree_sitter_arco_kdl_external_scanner_serialize(void *payload, char *buffer) {
  return kdl_scanner_serialize(payload, buffer);
}

void tree_sitter_arco_kdl_external_scanner_deserialize(void *payload, const char *buffer, unsigned length) {
  kdl_scanner_deserialize(payload, buffer, length);
}

bool tree_sitter_arco_kdl_external_scanner_scan(
  void *payload,
  TSLexer *lexer,
  const bool *valid_symbols
) {
  // If the parser is looking for an implicit terminator, check if
  // the next non-whitespace character is '}'. If so, return true
  // without consuming any characters (zero-width match).
  if (valid_symbols[IMPLICIT_TERMINATOR]) {
    // Peek ahead past whitespace
    while (lexer->lookahead == ' ' || lexer->lookahead == '\t') {
      lexer->advance(lexer, true);  // skip whitespace
    }
    if (lexer->lookahead == '}') {
      lexer->result_symbol = IMPLICIT_TERMINATOR;
      return true;
    }
  }

  // Otherwise, delegate to the KDL scanner for _eof and multi_line_comment.
  return kdl_scanner_scan(payload, lexer, valid_symbols);
}
