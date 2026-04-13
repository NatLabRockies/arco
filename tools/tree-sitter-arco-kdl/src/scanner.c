// Vendored tree-sitter-kdl scanner with arco_kdl extensions
// This file combines tree-sitter-kdl's scanner with arco-specific additions
// to remove the node_modules dependency for nvim-treesitter installation

#include "tree_sitter/parser.h"
#include "wctype.h"
#include <ctype.h>

// === Begin vendored tree-sitter-kdl scanner (with renamed symbols) ===

// Rename KDL scanner symbols to avoid clashes when we wrap them
#define tree_sitter_kdl_external_scanner_create  kdl_scanner_create
#define tree_sitter_kdl_external_scanner_destroy kdl_scanner_destroy
#define tree_sitter_kdl_external_scanner_scan    kdl_scanner_scan
#define tree_sitter_kdl_external_scanner_serialize   kdl_scanner_serialize
#define tree_sitter_kdl_external_scanner_deserialize kdl_scanner_deserialize

// KDL scanner internal enums (renamed)
enum { KDL_INTERNAL_EOF = 0, KDL_INTERNAL_MULTI_LINE_COMMENT = 1 };

static void kdl_advance(TSLexer *lexer) { lexer->advance(lexer, false); }

static void *kdl_scanner_create() { return NULL; }

static void kdl_scanner_destroy(void *payload) {}

static unsigned kdl_scanner_serialize(void *payload, char *buffer) {
    return 0;
}

static void kdl_scanner_deserialize(void *payload, const char *buffer, unsigned length) {}

static bool kdl_scanner_scan(void *payload, TSLexer *lexer, const bool *valid_symbols) {
    // check for End-of-file
    if (valid_symbols[KDL_INTERNAL_EOF] && lexer->lookahead == 0) {
        lexer->result_symbol = KDL_INTERNAL_EOF;
        kdl_advance(lexer);
        return true;
    }

    // multi-line-comment := '/*' commented-block
    if (lexer->lookahead == '/') {
        kdl_advance(lexer);
        if (lexer->lookahead != '*')
            return false;
        kdl_advance(lexer);

        bool     after_star = false;
        unsigned nesting_depth = 1;
        for (;;) {
            switch (lexer->lookahead) {
                case '\0':
                    return false;
                case '*':
                    kdl_advance(lexer);
                    after_star = true;
                    break;
                case '/':
                    if (after_star) {
                        kdl_advance(lexer);
                        after_star = false;
                        nesting_depth--;
                        if (nesting_depth == 0) {
                            lexer->result_symbol = KDL_INTERNAL_MULTI_LINE_COMMENT;
                            return true;
                        }
                    } else {
                        kdl_advance(lexer);
                        after_star = false;
                        if (lexer->lookahead == '*') {
                            nesting_depth++;
                            kdl_advance(lexer);
                        }
                    }
                    break;
                default:
                    kdl_advance(lexer);
                    after_star = false;
                    break;
            }
        }
    }

    return false;
}

// === End vendored tree-sitter-kdl scanner ===

// === Begin arco_kdl-specific extensions ===

// The external token indices must match the order in grammar.js externals.
// externals: [_eof, multi_line_comment, _implicit_terminator]
enum {
  KDL_EOF = 0,
  KDL_MULTI_LINE_COMMENT = 1,
  IMPLICIT_TERMINATOR = 2,
};

static bool scan_implicit_terminator(TSLexer *lexer) {
  while (lexer->lookahead == ' ' || lexer->lookahead == '\t') {
    lexer->advance(lexer, true);
  }

  if (lexer->lookahead != '}') {
    return false;
  }

  lexer->result_symbol = IMPLICIT_TERMINATOR;
  return true;
}

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
  if (valid_symbols[IMPLICIT_TERMINATOR] && scan_implicit_terminator(lexer)) {
    return true;
  }

  return kdl_scanner_scan(payload, lexer, valid_symbols);
}
