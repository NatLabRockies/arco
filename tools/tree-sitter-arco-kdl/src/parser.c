#include "tree_sitter/parser.h"

#if defined(__GNUC__) || defined(__clang__)
#pragma GCC diagnostic push
#pragma GCC diagnostic ignored "-Wmissing-field-initializers"
#endif

#define LANGUAGE_VERSION 14
#define STATE_COUNT 610
#define LARGE_STATE_COUNT 2
#define SYMBOL_COUNT 150
#define ALIAS_COUNT 4
#define TOKEN_COUNT 99
#define EXTERNAL_TOKEN_COUNT 3
#define FIELD_COUNT 3
#define MAX_ALIAS_SEQUENCE_LENGTH 10
#define PRODUCTION_ID_COUNT 40

enum ts_symbol_identifiers {
  sym__normal_bare_identifier = 1,
  anon_sym_SLASH_DASH = 2,
  anon_sym_LBRACE = 3,
  anon_sym_RBRACE = 4,
  anon_sym_SEMI = 5,
  sym__identifier_char = 6,
  sym___identifier_char_no_digit = 7,
  anon_sym_null = 8,
  anon_sym_i8 = 9,
  anon_sym_i16 = 10,
  anon_sym_i32 = 11,
  anon_sym_i64 = 12,
  anon_sym_u8 = 13,
  anon_sym_u16 = 14,
  anon_sym_u32 = 15,
  anon_sym_u64 = 16,
  anon_sym_isize = 17,
  anon_sym_usize = 18,
  anon_sym_f32 = 19,
  anon_sym_f64 = 20,
  anon_sym_decimal64 = 21,
  anon_sym_decimal128 = 22,
  anon_sym_date_DASHtime = 23,
  anon_sym_time = 24,
  anon_sym_date = 25,
  anon_sym_duration = 26,
  anon_sym_decimal = 27,
  anon_sym_currency = 28,
  anon_sym_country_DASH2 = 29,
  anon_sym_country_DASH3 = 30,
  anon_sym_country_DASHsubdivision = 31,
  anon_sym_email = 32,
  anon_sym_idn_DASHemail = 33,
  anon_sym_hostname = 34,
  anon_sym_idn_DASHhostname = 35,
  anon_sym_ipv4 = 36,
  anon_sym_ipv6 = 37,
  anon_sym_url = 38,
  anon_sym_url_DASHreference = 39,
  anon_sym_irl = 40,
  anon_sym_iri_DASHreference = 41,
  anon_sym_url_DASHtemplate = 42,
  anon_sym_uuid = 43,
  anon_sym_regex = 44,
  anon_sym_base64 = 45,
  anon_sym_EQ = 46,
  anon_sym_LPAREN = 47,
  anon_sym_RPAREN = 48,
  anon_sym_DQUOTE = 49,
  aux_sym__escaped_string_token1 = 50,
  sym_escape = 51,
  sym__hex_digit = 52,
  aux_sym__raw_string_token1 = 53,
  aux_sym__raw_string_token2 = 54,
  anon_sym_POUND = 55,
  aux_sym__raw_string_token3 = 56,
  aux_sym__raw_string_token4 = 57,
  anon_sym_DOT = 58,
  anon_sym_e = 59,
  anon_sym_E = 60,
  anon_sym__ = 61,
  sym__digit = 62,
  anon_sym_PLUS = 63,
  anon_sym_DASH = 64,
  anon_sym_0x = 65,
  anon_sym_0o = 66,
  aux_sym__octal_token1 = 67,
  anon_sym_0b = 68,
  anon_sym_0 = 69,
  anon_sym_1 = 70,
  anon_sym_true = 71,
  anon_sym_false = 72,
  anon_sym_BSLASH = 73,
  aux_sym__newline_token1 = 74,
  aux_sym__newline_token2 = 75,
  aux_sym__newline_token3 = 76,
  aux_sym__newline_token4 = 77,
  aux_sym__newline_token5 = 78,
  aux_sym__newline_token6 = 79,
  aux_sym__newline_token7 = 80,
  sym__bom = 81,
  sym__unicode_space = 82,
  anon_sym_SLASH_SLASH = 83,
  aux_sym_single_line_comment_token1 = 84,
  anon_sym_expression = 85,
  anon_sym_minimize = 86,
  anon_sym_maximize = 87,
  anon_sym_expr = 88,
  anon_sym_filter = 89,
  anon_sym_if = 90,
  anon_sym_lower = 91,
  anon_sym_upper = 92,
  anon_sym_constraint = 93,
  sym_arco_math_text = 94,
  sym_arco_constraint_math_text = 95,
  sym__eof = 96,
  sym_multi_line_comment = 97,
  sym__implicit_terminator = 98,
  sym_document = 99,
  sym_node = 100,
  sym_node_field = 101,
  sym__node_field_comment = 102,
  sym__node_field = 103,
  sym_node_children = 104,
  sym__node_space = 105,
  sym__node_terminator = 106,
  sym_identifier = 107,
  sym__bare_identifier = 108,
  sym_keyword = 109,
  sym_annotation_type = 110,
  sym_prop = 111,
  sym_value = 112,
  sym_type = 113,
  sym_string = 114,
  sym__escaped_string = 115,
  sym__raw_string = 116,
  sym_number = 117,
  sym__decimal = 118,
  sym__exponent = 119,
  sym__integer = 120,
  sym__sign = 121,
  sym__hex = 122,
  sym__octal = 123,
  sym__binary = 124,
  sym_boolean = 125,
  sym__escline = 126,
  sym__linespace = 127,
  sym__newline = 128,
  sym__ws = 129,
  sym_single_line_comment = 130,
  sym_bare_identifier = 131,
  sym_kdl_node = 132,
  sym_arco_pure_math_node = 133,
  sym_arco_constraint_node = 134,
  sym_arco_pure_math_children = 135,
  sym_arco_constraint_math_children = 136,
  aux_sym_document_repeat1 = 137,
  aux_sym_document_repeat2 = 138,
  aux_sym__node_field_comment_repeat1 = 139,
  aux_sym__node_space_repeat1 = 140,
  aux_sym__bare_identifier_repeat1 = 141,
  aux_sym__escaped_string_repeat1 = 142,
  aux_sym__raw_string_repeat1 = 143,
  aux_sym__integer_repeat1 = 144,
  aux_sym__hex_repeat1 = 145,
  aux_sym__octal_repeat1 = 146,
  aux_sym__binary_repeat1 = 147,
  aux_sym_single_line_comment_repeat1 = 148,
  aux_sym_kdl_node_repeat1 = 149,
  alias_sym_decimal = 150,
  alias_sym_node_children_comment = 151,
  alias_sym_node_field_comment = 152,
  alias_sym_string_fragment = 153,
};

static const char * const ts_symbol_names[] = {
  [ts_builtin_sym_end] = "end",
  [sym__normal_bare_identifier] = "_normal_bare_identifier",
  [anon_sym_SLASH_DASH] = "node_comment",
  [anon_sym_LBRACE] = "{",
  [anon_sym_RBRACE] = "}",
  [anon_sym_SEMI] = ";",
  [sym__identifier_char] = "_identifier_char",
  [sym___identifier_char_no_digit] = "__identifier_char_no_digit",
  [anon_sym_null] = "null",
  [anon_sym_i8] = "i8",
  [anon_sym_i16] = "i16",
  [anon_sym_i32] = "i32",
  [anon_sym_i64] = "i64",
  [anon_sym_u8] = "u8",
  [anon_sym_u16] = "u16",
  [anon_sym_u32] = "u32",
  [anon_sym_u64] = "u64",
  [anon_sym_isize] = "isize",
  [anon_sym_usize] = "usize",
  [anon_sym_f32] = "f32",
  [anon_sym_f64] = "f64",
  [anon_sym_decimal64] = "decimal64",
  [anon_sym_decimal128] = "decimal128",
  [anon_sym_date_DASHtime] = "date-time",
  [anon_sym_time] = "time",
  [anon_sym_date] = "date",
  [anon_sym_duration] = "duration",
  [anon_sym_decimal] = "decimal",
  [anon_sym_currency] = "currency",
  [anon_sym_country_DASH2] = "country-2",
  [anon_sym_country_DASH3] = "country-3",
  [anon_sym_country_DASHsubdivision] = "country-subdivision",
  [anon_sym_email] = "email",
  [anon_sym_idn_DASHemail] = "idn-email",
  [anon_sym_hostname] = "hostname",
  [anon_sym_idn_DASHhostname] = "idn-hostname",
  [anon_sym_ipv4] = "ipv4",
  [anon_sym_ipv6] = "ipv6",
  [anon_sym_url] = "url",
  [anon_sym_url_DASHreference] = "url-reference",
  [anon_sym_irl] = "irl",
  [anon_sym_iri_DASHreference] = "iri-reference",
  [anon_sym_url_DASHtemplate] = "url-template",
  [anon_sym_uuid] = "uuid",
  [anon_sym_regex] = "regex",
  [anon_sym_base64] = "base64",
  [anon_sym_EQ] = "=",
  [anon_sym_LPAREN] = "(",
  [anon_sym_RPAREN] = ")",
  [anon_sym_DQUOTE] = "\"",
  [aux_sym__escaped_string_token1] = "_escaped_string_token1",
  [sym_escape] = "escape",
  [sym__hex_digit] = "_hex_digit",
  [aux_sym__raw_string_token1] = "_raw_string_token1",
  [aux_sym__raw_string_token2] = "_raw_string_token2",
  [anon_sym_POUND] = "#",
  [aux_sym__raw_string_token3] = "_raw_string_token3",
  [aux_sym__raw_string_token4] = "_raw_string_token4",
  [anon_sym_DOT] = ".",
  [anon_sym_e] = "e",
  [anon_sym_E] = "E",
  [anon_sym__] = "_",
  [sym__digit] = "_digit",
  [anon_sym_PLUS] = "+",
  [anon_sym_DASH] = "-",
  [anon_sym_0x] = "0x",
  [anon_sym_0o] = "0o",
  [aux_sym__octal_token1] = "_octal_token1",
  [anon_sym_0b] = "0b",
  [anon_sym_0] = "0",
  [anon_sym_1] = "1",
  [anon_sym_true] = "true",
  [anon_sym_false] = "false",
  [anon_sym_BSLASH] = "\\",
  [aux_sym__newline_token1] = "_newline_token1",
  [aux_sym__newline_token2] = "_newline_token2",
  [aux_sym__newline_token3] = "_newline_token3",
  [aux_sym__newline_token4] = "_newline_token4",
  [aux_sym__newline_token5] = "_newline_token5",
  [aux_sym__newline_token6] = "_newline_token6",
  [aux_sym__newline_token7] = "_newline_token7",
  [sym__bom] = "_bom",
  [sym__unicode_space] = "_unicode_space",
  [anon_sym_SLASH_SLASH] = "//",
  [aux_sym_single_line_comment_token1] = "single_line_comment_token1",
  [anon_sym_expression] = "expression",
  [anon_sym_minimize] = "minimize",
  [anon_sym_maximize] = "maximize",
  [anon_sym_expr] = "expr",
  [anon_sym_filter] = "filter",
  [anon_sym_if] = "if",
  [anon_sym_lower] = "lower",
  [anon_sym_upper] = "upper",
  [anon_sym_constraint] = "constraint",
  [sym_arco_math_text] = "arco_math_text",
  [sym_arco_constraint_math_text] = "arco_constraint_math_text",
  [sym__eof] = "_eof",
  [sym_multi_line_comment] = "multi_line_comment",
  [sym__implicit_terminator] = "_implicit_terminator",
  [sym_document] = "document",
  [sym_node] = "node",
  [sym_node_field] = "node_field",
  [sym__node_field_comment] = "_node_field_comment",
  [sym__node_field] = "_node_field",
  [sym_node_children] = "node_children",
  [sym__node_space] = "_node_space",
  [sym__node_terminator] = "_node_terminator",
  [sym_identifier] = "identifier",
  [sym__bare_identifier] = "_bare_identifier",
  [sym_keyword] = "keyword",
  [sym_annotation_type] = "annotation_type",
  [sym_prop] = "prop",
  [sym_value] = "value",
  [sym_type] = "type",
  [sym_string] = "string",
  [sym__escaped_string] = "_escaped_string",
  [sym__raw_string] = "_raw_string",
  [sym_number] = "number",
  [sym__decimal] = "_decimal",
  [sym__exponent] = "exponent",
  [sym__integer] = "_integer",
  [sym__sign] = "_sign",
  [sym__hex] = "_hex",
  [sym__octal] = "_octal",
  [sym__binary] = "_binary",
  [sym_boolean] = "boolean",
  [sym__escline] = "_escline",
  [sym__linespace] = "_linespace",
  [sym__newline] = "_newline",
  [sym__ws] = "_ws",
  [sym_single_line_comment] = "single_line_comment",
  [sym_bare_identifier] = "bare_identifier",
  [sym_kdl_node] = "kdl_node",
  [sym_arco_pure_math_node] = "arco_pure_math_node",
  [sym_arco_constraint_node] = "arco_constraint_node",
  [sym_arco_pure_math_children] = "arco_pure_math_children",
  [sym_arco_constraint_math_children] = "arco_constraint_math_children",
  [aux_sym_document_repeat1] = "document_repeat1",
  [aux_sym_document_repeat2] = "document_repeat2",
  [aux_sym__node_field_comment_repeat1] = "_node_field_comment_repeat1",
  [aux_sym__node_space_repeat1] = "_node_space_repeat1",
  [aux_sym__bare_identifier_repeat1] = "_bare_identifier_repeat1",
  [aux_sym__escaped_string_repeat1] = "_escaped_string_repeat1",
  [aux_sym__raw_string_repeat1] = "_raw_string_repeat1",
  [aux_sym__integer_repeat1] = "_integer_repeat1",
  [aux_sym__hex_repeat1] = "_hex_repeat1",
  [aux_sym__octal_repeat1] = "_octal_repeat1",
  [aux_sym__binary_repeat1] = "_binary_repeat1",
  [aux_sym_single_line_comment_repeat1] = "single_line_comment_repeat1",
  [aux_sym_kdl_node_repeat1] = "kdl_node_repeat1",
  [alias_sym_decimal] = "decimal",
  [alias_sym_node_children_comment] = "node_children_comment",
  [alias_sym_node_field_comment] = "node_field_comment",
  [alias_sym_string_fragment] = "string_fragment",
};

static const TSSymbol ts_symbol_map[] = {
  [ts_builtin_sym_end] = ts_builtin_sym_end,
  [sym__normal_bare_identifier] = sym__normal_bare_identifier,
  [anon_sym_SLASH_DASH] = anon_sym_SLASH_DASH,
  [anon_sym_LBRACE] = anon_sym_LBRACE,
  [anon_sym_RBRACE] = anon_sym_RBRACE,
  [anon_sym_SEMI] = anon_sym_SEMI,
  [sym__identifier_char] = sym__identifier_char,
  [sym___identifier_char_no_digit] = sym___identifier_char_no_digit,
  [anon_sym_null] = anon_sym_null,
  [anon_sym_i8] = anon_sym_i8,
  [anon_sym_i16] = anon_sym_i16,
  [anon_sym_i32] = anon_sym_i32,
  [anon_sym_i64] = anon_sym_i64,
  [anon_sym_u8] = anon_sym_u8,
  [anon_sym_u16] = anon_sym_u16,
  [anon_sym_u32] = anon_sym_u32,
  [anon_sym_u64] = anon_sym_u64,
  [anon_sym_isize] = anon_sym_isize,
  [anon_sym_usize] = anon_sym_usize,
  [anon_sym_f32] = anon_sym_f32,
  [anon_sym_f64] = anon_sym_f64,
  [anon_sym_decimal64] = anon_sym_decimal64,
  [anon_sym_decimal128] = anon_sym_decimal128,
  [anon_sym_date_DASHtime] = anon_sym_date_DASHtime,
  [anon_sym_time] = anon_sym_time,
  [anon_sym_date] = anon_sym_date,
  [anon_sym_duration] = anon_sym_duration,
  [anon_sym_decimal] = anon_sym_decimal,
  [anon_sym_currency] = anon_sym_currency,
  [anon_sym_country_DASH2] = anon_sym_country_DASH2,
  [anon_sym_country_DASH3] = anon_sym_country_DASH3,
  [anon_sym_country_DASHsubdivision] = anon_sym_country_DASHsubdivision,
  [anon_sym_email] = anon_sym_email,
  [anon_sym_idn_DASHemail] = anon_sym_idn_DASHemail,
  [anon_sym_hostname] = anon_sym_hostname,
  [anon_sym_idn_DASHhostname] = anon_sym_idn_DASHhostname,
  [anon_sym_ipv4] = anon_sym_ipv4,
  [anon_sym_ipv6] = anon_sym_ipv6,
  [anon_sym_url] = anon_sym_url,
  [anon_sym_url_DASHreference] = anon_sym_url_DASHreference,
  [anon_sym_irl] = anon_sym_irl,
  [anon_sym_iri_DASHreference] = anon_sym_iri_DASHreference,
  [anon_sym_url_DASHtemplate] = anon_sym_url_DASHtemplate,
  [anon_sym_uuid] = anon_sym_uuid,
  [anon_sym_regex] = anon_sym_regex,
  [anon_sym_base64] = anon_sym_base64,
  [anon_sym_EQ] = anon_sym_EQ,
  [anon_sym_LPAREN] = anon_sym_LPAREN,
  [anon_sym_RPAREN] = anon_sym_RPAREN,
  [anon_sym_DQUOTE] = anon_sym_DQUOTE,
  [aux_sym__escaped_string_token1] = aux_sym__escaped_string_token1,
  [sym_escape] = sym_escape,
  [sym__hex_digit] = sym__hex_digit,
  [aux_sym__raw_string_token1] = aux_sym__raw_string_token1,
  [aux_sym__raw_string_token2] = aux_sym__raw_string_token2,
  [anon_sym_POUND] = anon_sym_POUND,
  [aux_sym__raw_string_token3] = aux_sym__raw_string_token3,
  [aux_sym__raw_string_token4] = aux_sym__raw_string_token4,
  [anon_sym_DOT] = anon_sym_DOT,
  [anon_sym_e] = anon_sym_e,
  [anon_sym_E] = anon_sym_E,
  [anon_sym__] = anon_sym__,
  [sym__digit] = sym__digit,
  [anon_sym_PLUS] = anon_sym_PLUS,
  [anon_sym_DASH] = anon_sym_DASH,
  [anon_sym_0x] = anon_sym_0x,
  [anon_sym_0o] = anon_sym_0o,
  [aux_sym__octal_token1] = aux_sym__octal_token1,
  [anon_sym_0b] = anon_sym_0b,
  [anon_sym_0] = anon_sym_0,
  [anon_sym_1] = anon_sym_1,
  [anon_sym_true] = anon_sym_true,
  [anon_sym_false] = anon_sym_false,
  [anon_sym_BSLASH] = anon_sym_BSLASH,
  [aux_sym__newline_token1] = aux_sym__newline_token1,
  [aux_sym__newline_token2] = aux_sym__newline_token2,
  [aux_sym__newline_token3] = aux_sym__newline_token3,
  [aux_sym__newline_token4] = aux_sym__newline_token4,
  [aux_sym__newline_token5] = aux_sym__newline_token5,
  [aux_sym__newline_token6] = aux_sym__newline_token6,
  [aux_sym__newline_token7] = aux_sym__newline_token7,
  [sym__bom] = sym__bom,
  [sym__unicode_space] = sym__unicode_space,
  [anon_sym_SLASH_SLASH] = anon_sym_SLASH_SLASH,
  [aux_sym_single_line_comment_token1] = aux_sym_single_line_comment_token1,
  [anon_sym_expression] = anon_sym_expression,
  [anon_sym_minimize] = anon_sym_minimize,
  [anon_sym_maximize] = anon_sym_maximize,
  [anon_sym_expr] = anon_sym_expr,
  [anon_sym_filter] = anon_sym_filter,
  [anon_sym_if] = anon_sym_if,
  [anon_sym_lower] = anon_sym_lower,
  [anon_sym_upper] = anon_sym_upper,
  [anon_sym_constraint] = anon_sym_constraint,
  [sym_arco_math_text] = sym_arco_math_text,
  [sym_arco_constraint_math_text] = sym_arco_constraint_math_text,
  [sym__eof] = sym__eof,
  [sym_multi_line_comment] = sym_multi_line_comment,
  [sym__implicit_terminator] = sym__implicit_terminator,
  [sym_document] = sym_document,
  [sym_node] = sym_node,
  [sym_node_field] = sym_node_field,
  [sym__node_field_comment] = sym__node_field_comment,
  [sym__node_field] = sym__node_field,
  [sym_node_children] = sym_node_children,
  [sym__node_space] = sym__node_space,
  [sym__node_terminator] = sym__node_terminator,
  [sym_identifier] = sym_identifier,
  [sym__bare_identifier] = sym__bare_identifier,
  [sym_keyword] = sym_keyword,
  [sym_annotation_type] = sym_annotation_type,
  [sym_prop] = sym_prop,
  [sym_value] = sym_value,
  [sym_type] = sym_type,
  [sym_string] = sym_string,
  [sym__escaped_string] = sym__escaped_string,
  [sym__raw_string] = sym__raw_string,
  [sym_number] = sym_number,
  [sym__decimal] = sym__decimal,
  [sym__exponent] = sym__exponent,
  [sym__integer] = sym__integer,
  [sym__sign] = sym__sign,
  [sym__hex] = sym__hex,
  [sym__octal] = sym__octal,
  [sym__binary] = sym__binary,
  [sym_boolean] = sym_boolean,
  [sym__escline] = sym__escline,
  [sym__linespace] = sym__linespace,
  [sym__newline] = sym__newline,
  [sym__ws] = sym__ws,
  [sym_single_line_comment] = sym_single_line_comment,
  [sym_bare_identifier] = sym_bare_identifier,
  [sym_kdl_node] = sym_kdl_node,
  [sym_arco_pure_math_node] = sym_arco_pure_math_node,
  [sym_arco_constraint_node] = sym_arco_constraint_node,
  [sym_arco_pure_math_children] = sym_arco_pure_math_children,
  [sym_arco_constraint_math_children] = sym_arco_constraint_math_children,
  [aux_sym_document_repeat1] = aux_sym_document_repeat1,
  [aux_sym_document_repeat2] = aux_sym_document_repeat2,
  [aux_sym__node_field_comment_repeat1] = aux_sym__node_field_comment_repeat1,
  [aux_sym__node_space_repeat1] = aux_sym__node_space_repeat1,
  [aux_sym__bare_identifier_repeat1] = aux_sym__bare_identifier_repeat1,
  [aux_sym__escaped_string_repeat1] = aux_sym__escaped_string_repeat1,
  [aux_sym__raw_string_repeat1] = aux_sym__raw_string_repeat1,
  [aux_sym__integer_repeat1] = aux_sym__integer_repeat1,
  [aux_sym__hex_repeat1] = aux_sym__hex_repeat1,
  [aux_sym__octal_repeat1] = aux_sym__octal_repeat1,
  [aux_sym__binary_repeat1] = aux_sym__binary_repeat1,
  [aux_sym_single_line_comment_repeat1] = aux_sym_single_line_comment_repeat1,
  [aux_sym_kdl_node_repeat1] = aux_sym_kdl_node_repeat1,
  [alias_sym_decimal] = alias_sym_decimal,
  [alias_sym_node_children_comment] = alias_sym_node_children_comment,
  [alias_sym_node_field_comment] = alias_sym_node_field_comment,
  [alias_sym_string_fragment] = alias_sym_string_fragment,
};

static const TSSymbolMetadata ts_symbol_metadata[] = {
  [ts_builtin_sym_end] = {
    .visible = false,
    .named = true,
  },
  [sym__normal_bare_identifier] = {
    .visible = false,
    .named = true,
  },
  [anon_sym_SLASH_DASH] = {
    .visible = true,
    .named = true,
  },
  [anon_sym_LBRACE] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_RBRACE] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_SEMI] = {
    .visible = true,
    .named = false,
  },
  [sym__identifier_char] = {
    .visible = false,
    .named = true,
  },
  [sym___identifier_char_no_digit] = {
    .visible = false,
    .named = true,
  },
  [anon_sym_null] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_i8] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_i16] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_i32] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_i64] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_u8] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_u16] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_u32] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_u64] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_isize] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_usize] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_f32] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_f64] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_decimal64] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_decimal128] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_date_DASHtime] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_time] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_date] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_duration] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_decimal] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_currency] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_country_DASH2] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_country_DASH3] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_country_DASHsubdivision] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_email] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_idn_DASHemail] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_hostname] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_idn_DASHhostname] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_ipv4] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_ipv6] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_url] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_url_DASHreference] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_irl] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_iri_DASHreference] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_url_DASHtemplate] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_uuid] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_regex] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_base64] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_EQ] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_LPAREN] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_RPAREN] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_DQUOTE] = {
    .visible = true,
    .named = false,
  },
  [aux_sym__escaped_string_token1] = {
    .visible = false,
    .named = false,
  },
  [sym_escape] = {
    .visible = true,
    .named = true,
  },
  [sym__hex_digit] = {
    .visible = false,
    .named = true,
  },
  [aux_sym__raw_string_token1] = {
    .visible = false,
    .named = false,
  },
  [aux_sym__raw_string_token2] = {
    .visible = false,
    .named = false,
  },
  [anon_sym_POUND] = {
    .visible = true,
    .named = false,
  },
  [aux_sym__raw_string_token3] = {
    .visible = false,
    .named = false,
  },
  [aux_sym__raw_string_token4] = {
    .visible = false,
    .named = false,
  },
  [anon_sym_DOT] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_e] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_E] = {
    .visible = true,
    .named = false,
  },
  [anon_sym__] = {
    .visible = true,
    .named = false,
  },
  [sym__digit] = {
    .visible = false,
    .named = true,
  },
  [anon_sym_PLUS] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_DASH] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_0x] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_0o] = {
    .visible = true,
    .named = false,
  },
  [aux_sym__octal_token1] = {
    .visible = false,
    .named = false,
  },
  [anon_sym_0b] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_0] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_1] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_true] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_false] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_BSLASH] = {
    .visible = true,
    .named = false,
  },
  [aux_sym__newline_token1] = {
    .visible = false,
    .named = false,
  },
  [aux_sym__newline_token2] = {
    .visible = false,
    .named = false,
  },
  [aux_sym__newline_token3] = {
    .visible = false,
    .named = false,
  },
  [aux_sym__newline_token4] = {
    .visible = false,
    .named = false,
  },
  [aux_sym__newline_token5] = {
    .visible = false,
    .named = false,
  },
  [aux_sym__newline_token6] = {
    .visible = false,
    .named = false,
  },
  [aux_sym__newline_token7] = {
    .visible = false,
    .named = false,
  },
  [sym__bom] = {
    .visible = false,
    .named = true,
  },
  [sym__unicode_space] = {
    .visible = false,
    .named = true,
  },
  [anon_sym_SLASH_SLASH] = {
    .visible = true,
    .named = false,
  },
  [aux_sym_single_line_comment_token1] = {
    .visible = false,
    .named = false,
  },
  [anon_sym_expression] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_minimize] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_maximize] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_expr] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_filter] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_if] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_lower] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_upper] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_constraint] = {
    .visible = true,
    .named = false,
  },
  [sym_arco_math_text] = {
    .visible = true,
    .named = true,
  },
  [sym_arco_constraint_math_text] = {
    .visible = true,
    .named = true,
  },
  [sym__eof] = {
    .visible = false,
    .named = true,
  },
  [sym_multi_line_comment] = {
    .visible = true,
    .named = true,
  },
  [sym__implicit_terminator] = {
    .visible = false,
    .named = true,
  },
  [sym_document] = {
    .visible = true,
    .named = true,
  },
  [sym_node] = {
    .visible = true,
    .named = true,
  },
  [sym_node_field] = {
    .visible = true,
    .named = true,
  },
  [sym__node_field_comment] = {
    .visible = false,
    .named = true,
  },
  [sym__node_field] = {
    .visible = false,
    .named = true,
  },
  [sym_node_children] = {
    .visible = true,
    .named = true,
  },
  [sym__node_space] = {
    .visible = false,
    .named = true,
  },
  [sym__node_terminator] = {
    .visible = false,
    .named = true,
  },
  [sym_identifier] = {
    .visible = true,
    .named = true,
  },
  [sym__bare_identifier] = {
    .visible = false,
    .named = true,
  },
  [sym_keyword] = {
    .visible = true,
    .named = true,
  },
  [sym_annotation_type] = {
    .visible = true,
    .named = true,
  },
  [sym_prop] = {
    .visible = true,
    .named = true,
  },
  [sym_value] = {
    .visible = true,
    .named = true,
  },
  [sym_type] = {
    .visible = true,
    .named = true,
  },
  [sym_string] = {
    .visible = true,
    .named = true,
  },
  [sym__escaped_string] = {
    .visible = false,
    .named = true,
  },
  [sym__raw_string] = {
    .visible = false,
    .named = true,
  },
  [sym_number] = {
    .visible = true,
    .named = true,
  },
  [sym__decimal] = {
    .visible = false,
    .named = true,
  },
  [sym__exponent] = {
    .visible = true,
    .named = true,
  },
  [sym__integer] = {
    .visible = false,
    .named = true,
  },
  [sym__sign] = {
    .visible = false,
    .named = true,
  },
  [sym__hex] = {
    .visible = false,
    .named = true,
  },
  [sym__octal] = {
    .visible = false,
    .named = true,
  },
  [sym__binary] = {
    .visible = false,
    .named = true,
  },
  [sym_boolean] = {
    .visible = true,
    .named = true,
  },
  [sym__escline] = {
    .visible = false,
    .named = true,
  },
  [sym__linespace] = {
    .visible = false,
    .named = true,
  },
  [sym__newline] = {
    .visible = false,
    .named = true,
  },
  [sym__ws] = {
    .visible = false,
    .named = true,
  },
  [sym_single_line_comment] = {
    .visible = true,
    .named = true,
  },
  [sym_bare_identifier] = {
    .visible = true,
    .named = true,
  },
  [sym_kdl_node] = {
    .visible = true,
    .named = true,
  },
  [sym_arco_pure_math_node] = {
    .visible = true,
    .named = true,
  },
  [sym_arco_constraint_node] = {
    .visible = true,
    .named = true,
  },
  [sym_arco_pure_math_children] = {
    .visible = true,
    .named = true,
  },
  [sym_arco_constraint_math_children] = {
    .visible = true,
    .named = true,
  },
  [aux_sym_document_repeat1] = {
    .visible = false,
    .named = false,
  },
  [aux_sym_document_repeat2] = {
    .visible = false,
    .named = false,
  },
  [aux_sym__node_field_comment_repeat1] = {
    .visible = false,
    .named = false,
  },
  [aux_sym__node_space_repeat1] = {
    .visible = false,
    .named = false,
  },
  [aux_sym__bare_identifier_repeat1] = {
    .visible = false,
    .named = false,
  },
  [aux_sym__escaped_string_repeat1] = {
    .visible = false,
    .named = false,
  },
  [aux_sym__raw_string_repeat1] = {
    .visible = false,
    .named = false,
  },
  [aux_sym__integer_repeat1] = {
    .visible = false,
    .named = false,
  },
  [aux_sym__hex_repeat1] = {
    .visible = false,
    .named = false,
  },
  [aux_sym__octal_repeat1] = {
    .visible = false,
    .named = false,
  },
  [aux_sym__binary_repeat1] = {
    .visible = false,
    .named = false,
  },
  [aux_sym_single_line_comment_repeat1] = {
    .visible = false,
    .named = false,
  },
  [aux_sym_kdl_node_repeat1] = {
    .visible = false,
    .named = false,
  },
  [alias_sym_decimal] = {
    .visible = true,
    .named = true,
  },
  [alias_sym_node_children_comment] = {
    .visible = true,
    .named = true,
  },
  [alias_sym_node_field_comment] = {
    .visible = true,
    .named = true,
  },
  [alias_sym_string_fragment] = {
    .visible = true,
    .named = true,
  },
};

enum ts_field_identifiers {
  field_children = 1,
  field_math = 2,
  field_name = 3,
};

static const char * const ts_field_names[] = {
  [0] = NULL,
  [field_children] = "children",
  [field_math] = "math",
  [field_name] = "name",
};

static const TSFieldMapSlice ts_field_map_slices[PRODUCTION_ID_COUNT] = {
  [1] = {.index = 0, .length = 1},
  [2] = {.index = 1, .length = 1},
  [4] = {.index = 2, .length = 2},
  [5] = {.index = 4, .length = 1},
  [6] = {.index = 5, .length = 2},
  [7] = {.index = 7, .length = 1},
  [8] = {.index = 8, .length = 1},
  [9] = {.index = 8, .length = 1},
  [12] = {.index = 9, .length = 1},
  [14] = {.index = 10, .length = 2},
  [15] = {.index = 12, .length = 2},
  [16] = {.index = 14, .length = 1},
  [17] = {.index = 15, .length = 2},
  [18] = {.index = 15, .length = 2},
  [19] = {.index = 14, .length = 1},
  [20] = {.index = 17, .length = 1},
  [21] = {.index = 18, .length = 1},
  [24] = {.index = 19, .length = 2},
  [25] = {.index = 21, .length = 2},
  [26] = {.index = 23, .length = 1},
  [27] = {.index = 24, .length = 2},
  [28] = {.index = 24, .length = 2},
  [29] = {.index = 23, .length = 1},
  [30] = {.index = 26, .length = 2},
  [31] = {.index = 28, .length = 1},
  [33] = {.index = 29, .length = 2},
  [34] = {.index = 31, .length = 1},
  [35] = {.index = 29, .length = 2},
  [36] = {.index = 31, .length = 1},
  [37] = {.index = 32, .length = 2},
  [38] = {.index = 34, .length = 2},
  [39] = {.index = 36, .length = 1},
};

static const TSFieldMapEntry ts_field_map_entries[] = {
  [0] =
    {field_name, 0},
  [1] =
    {field_name, 1},
  [2] =
    {field_children, 1},
    {field_name, 0},
  [4] =
    {field_children, 1},
  [5] =
    {field_children, 2},
    {field_name, 1},
  [7] =
    {field_children, 2},
  [8] =
    {field_name, 2},
  [9] =
    {field_math, 1},
  [10] =
    {field_children, 2},
    {field_name, 0},
  [12] =
    {field_children, 3},
    {field_name, 1},
  [14] =
    {field_children, 3},
  [15] =
    {field_children, 3},
    {field_name, 2},
  [17] =
    {field_name, 3},
  [18] =
    {field_math, 2},
  [19] =
    {field_children, 3},
    {field_name, 0},
  [21] =
    {field_children, 4},
    {field_name, 1},
  [23] =
    {field_children, 4},
  [24] =
    {field_children, 4},
    {field_name, 2},
  [26] =
    {field_children, 4},
    {field_name, 3},
  [28] =
    {field_math, 3},
  [29] =
    {field_children, 5},
    {field_name, 2},
  [31] =
    {field_children, 5},
  [32] =
    {field_children, 5},
    {field_name, 3},
  [34] =
    {field_children, 6},
    {field_name, 3},
  [36] =
    {field_children, 6},
};

static const TSSymbol ts_alias_sequences[PRODUCTION_ID_COUNT][MAX_ALIAS_SEQUENCE_LENGTH] = {
  [0] = {0},
  [3] = {
    [1] = alias_sym_string_fragment,
  },
  [9] = {
    [1] = anon_sym_SLASH_DASH,
  },
  [10] = {
    [1] = anon_sym_SLASH_DASH,
  },
  [11] = {
    [0] = alias_sym_node_children_comment,
  },
  [13] = {
    [0] = alias_sym_node_field_comment,
    [1] = alias_sym_node_field_comment,
  },
  [18] = {
    [1] = anon_sym_SLASH_DASH,
  },
  [19] = {
    [1] = anon_sym_SLASH_DASH,
  },
  [20] = {
    [1] = anon_sym_SLASH_DASH,
  },
  [21] = {
    [0] = alias_sym_node_children_comment,
  },
  [22] = {
    [0] = alias_sym_node_field_comment,
    [1] = alias_sym_node_field_comment,
    [2] = alias_sym_node_field_comment,
  },
  [23] = {
    [2] = alias_sym_decimal,
  },
  [28] = {
    [1] = anon_sym_SLASH_DASH,
  },
  [29] = {
    [1] = anon_sym_SLASH_DASH,
  },
  [30] = {
    [1] = anon_sym_SLASH_DASH,
  },
  [31] = {
    [0] = alias_sym_node_children_comment,
  },
  [32] = {
    [3] = alias_sym_decimal,
  },
  [35] = {
    [1] = anon_sym_SLASH_DASH,
  },
  [36] = {
    [1] = anon_sym_SLASH_DASH,
  },
  [37] = {
    [1] = anon_sym_SLASH_DASH,
  },
  [38] = {
    [1] = anon_sym_SLASH_DASH,
  },
  [39] = {
    [1] = anon_sym_SLASH_DASH,
  },
};

static const uint16_t ts_non_terminal_alias_map[] = {
  sym__node_field, 2,
    sym__node_field,
    alias_sym_node_field_comment,
  sym__integer, 2,
    sym__integer,
    alias_sym_decimal,
  aux_sym__node_field_comment_repeat1, 3,
    aux_sym__node_field_comment_repeat1,
    alias_sym_node_field_comment,
    anon_sym_SLASH_DASH,
  aux_sym__escaped_string_repeat1, 2,
    aux_sym__escaped_string_repeat1,
    alias_sym_string_fragment,
  0,
};

static const TSStateId ts_primary_state_ids[STATE_COUNT] = {
  [0] = 0,
  [1] = 1,
  [2] = 2,
  [3] = 3,
  [4] = 4,
  [5] = 5,
  [6] = 6,
  [7] = 7,
  [8] = 8,
  [9] = 9,
  [10] = 10,
  [11] = 11,
  [12] = 12,
  [13] = 13,
  [14] = 14,
  [15] = 15,
  [16] = 16,
  [17] = 17,
  [18] = 18,
  [19] = 19,
  [20] = 20,
  [21] = 21,
  [22] = 22,
  [23] = 23,
  [24] = 24,
  [25] = 25,
  [26] = 26,
  [27] = 27,
  [28] = 28,
  [29] = 29,
  [30] = 30,
  [31] = 31,
  [32] = 32,
  [33] = 33,
  [34] = 34,
  [35] = 35,
  [36] = 36,
  [37] = 37,
  [38] = 38,
  [39] = 39,
  [40] = 40,
  [41] = 41,
  [42] = 42,
  [43] = 43,
  [44] = 44,
  [45] = 45,
  [46] = 46,
  [47] = 47,
  [48] = 48,
  [49] = 49,
  [50] = 50,
  [51] = 51,
  [52] = 52,
  [53] = 53,
  [54] = 54,
  [55] = 55,
  [56] = 56,
  [57] = 57,
  [58] = 58,
  [59] = 59,
  [60] = 60,
  [61] = 61,
  [62] = 62,
  [63] = 63,
  [64] = 64,
  [65] = 65,
  [66] = 66,
  [67] = 67,
  [68] = 68,
  [69] = 69,
  [70] = 70,
  [71] = 71,
  [72] = 72,
  [73] = 73,
  [74] = 74,
  [75] = 75,
  [76] = 76,
  [77] = 77,
  [78] = 78,
  [79] = 79,
  [80] = 80,
  [81] = 81,
  [82] = 82,
  [83] = 83,
  [84] = 84,
  [85] = 85,
  [86] = 81,
  [87] = 87,
  [88] = 88,
  [89] = 89,
  [90] = 79,
  [91] = 91,
  [92] = 92,
  [93] = 93,
  [94] = 94,
  [95] = 84,
  [96] = 96,
  [97] = 91,
  [98] = 93,
  [99] = 78,
  [100] = 100,
  [101] = 101,
  [102] = 102,
  [103] = 89,
  [104] = 80,
  [105] = 105,
  [106] = 92,
  [107] = 107,
  [108] = 108,
  [109] = 109,
  [110] = 110,
  [111] = 111,
  [112] = 112,
  [113] = 113,
  [114] = 114,
  [115] = 115,
  [116] = 116,
  [117] = 117,
  [118] = 118,
  [119] = 119,
  [120] = 120,
  [121] = 121,
  [122] = 122,
  [123] = 123,
  [124] = 124,
  [125] = 125,
  [126] = 126,
  [127] = 127,
  [128] = 128,
  [129] = 129,
  [130] = 130,
  [131] = 131,
  [132] = 132,
  [133] = 133,
  [134] = 134,
  [135] = 135,
  [136] = 136,
  [137] = 137,
  [138] = 138,
  [139] = 139,
  [140] = 140,
  [141] = 141,
  [142] = 142,
  [143] = 143,
  [144] = 144,
  [145] = 145,
  [146] = 146,
  [147] = 147,
  [148] = 148,
  [149] = 149,
  [150] = 150,
  [151] = 151,
  [152] = 152,
  [153] = 153,
  [154] = 154,
  [155] = 155,
  [156] = 156,
  [157] = 157,
  [158] = 158,
  [159] = 159,
  [160] = 160,
  [161] = 161,
  [162] = 162,
  [163] = 163,
  [164] = 164,
  [165] = 165,
  [166] = 166,
  [167] = 167,
  [168] = 168,
  [169] = 169,
  [170] = 170,
  [171] = 171,
  [172] = 172,
  [173] = 173,
  [174] = 174,
  [175] = 175,
  [176] = 176,
  [177] = 177,
  [178] = 178,
  [179] = 179,
  [180] = 180,
  [181] = 181,
  [182] = 182,
  [183] = 183,
  [184] = 184,
  [185] = 185,
  [186] = 186,
  [187] = 187,
  [188] = 188,
  [189] = 189,
  [190] = 190,
  [191] = 191,
  [192] = 192,
  [193] = 193,
  [194] = 194,
  [195] = 195,
  [196] = 196,
  [197] = 197,
  [198] = 198,
  [199] = 199,
  [200] = 200,
  [201] = 201,
  [202] = 202,
  [203] = 203,
  [204] = 204,
  [205] = 205,
  [206] = 206,
  [207] = 207,
  [208] = 208,
  [209] = 209,
  [210] = 210,
  [211] = 211,
  [212] = 212,
  [213] = 213,
  [214] = 214,
  [215] = 215,
  [216] = 216,
  [217] = 217,
  [218] = 218,
  [219] = 219,
  [220] = 220,
  [221] = 221,
  [222] = 222,
  [223] = 223,
  [224] = 224,
  [225] = 225,
  [226] = 226,
  [227] = 227,
  [228] = 228,
  [229] = 229,
  [230] = 230,
  [231] = 231,
  [232] = 232,
  [233] = 233,
  [234] = 234,
  [235] = 235,
  [236] = 236,
  [237] = 237,
  [238] = 238,
  [239] = 239,
  [240] = 240,
  [241] = 241,
  [242] = 242,
  [243] = 243,
  [244] = 244,
  [245] = 245,
  [246] = 246,
  [247] = 247,
  [248] = 248,
  [249] = 249,
  [250] = 250,
  [251] = 251,
  [252] = 252,
  [253] = 253,
  [254] = 254,
  [255] = 255,
  [256] = 256,
  [257] = 257,
  [258] = 258,
  [259] = 259,
  [260] = 260,
  [261] = 261,
  [262] = 262,
  [263] = 263,
  [264] = 264,
  [265] = 100,
  [266] = 105,
  [267] = 102,
  [268] = 268,
  [269] = 269,
  [270] = 270,
  [271] = 271,
  [272] = 272,
  [273] = 273,
  [274] = 274,
  [275] = 275,
  [276] = 276,
  [277] = 277,
  [278] = 278,
  [279] = 279,
  [280] = 280,
  [281] = 281,
  [282] = 282,
  [283] = 283,
  [284] = 284,
  [285] = 285,
  [286] = 286,
  [287] = 287,
  [288] = 288,
  [289] = 289,
  [290] = 290,
  [291] = 291,
  [292] = 292,
  [293] = 293,
  [294] = 294,
  [295] = 295,
  [296] = 296,
  [297] = 297,
  [298] = 298,
  [299] = 299,
  [300] = 300,
  [301] = 301,
  [302] = 302,
  [303] = 303,
  [304] = 304,
  [305] = 305,
  [306] = 306,
  [307] = 307,
  [308] = 308,
  [309] = 309,
  [310] = 310,
  [311] = 311,
  [312] = 312,
  [313] = 313,
  [314] = 314,
  [315] = 315,
  [316] = 316,
  [317] = 317,
  [318] = 318,
  [319] = 319,
  [320] = 320,
  [321] = 321,
  [322] = 322,
  [323] = 323,
  [324] = 324,
  [325] = 325,
  [326] = 326,
  [327] = 327,
  [328] = 328,
  [329] = 329,
  [330] = 330,
  [331] = 331,
  [332] = 332,
  [333] = 333,
  [334] = 334,
  [335] = 335,
  [336] = 336,
  [337] = 337,
  [338] = 338,
  [339] = 339,
  [340] = 340,
  [341] = 341,
  [342] = 342,
  [343] = 343,
  [344] = 344,
  [345] = 345,
  [346] = 346,
  [347] = 347,
  [348] = 348,
  [349] = 349,
  [350] = 350,
  [351] = 351,
  [352] = 352,
  [353] = 353,
  [354] = 354,
  [355] = 355,
  [356] = 356,
  [357] = 357,
  [358] = 358,
  [359] = 359,
  [360] = 360,
  [361] = 361,
  [362] = 362,
  [363] = 363,
  [364] = 364,
  [365] = 365,
  [366] = 366,
  [367] = 367,
  [368] = 368,
  [369] = 369,
  [370] = 370,
  [371] = 371,
  [372] = 372,
  [373] = 373,
  [374] = 374,
  [375] = 375,
  [376] = 376,
  [377] = 377,
  [378] = 378,
  [379] = 379,
  [380] = 380,
  [381] = 381,
  [382] = 382,
  [383] = 383,
  [384] = 384,
  [385] = 385,
  [386] = 386,
  [387] = 387,
  [388] = 388,
  [389] = 389,
  [390] = 390,
  [391] = 391,
  [392] = 392,
  [393] = 393,
  [394] = 394,
  [395] = 395,
  [396] = 396,
  [397] = 397,
  [398] = 398,
  [399] = 399,
  [400] = 400,
  [401] = 401,
  [402] = 402,
  [403] = 403,
  [404] = 404,
  [405] = 405,
  [406] = 406,
  [407] = 407,
  [408] = 408,
  [409] = 409,
  [410] = 410,
  [411] = 411,
  [412] = 412,
  [413] = 413,
  [414] = 414,
  [415] = 415,
  [416] = 416,
  [417] = 417,
  [418] = 418,
  [419] = 419,
  [420] = 420,
  [421] = 421,
  [422] = 422,
  [423] = 423,
  [424] = 424,
  [425] = 425,
  [426] = 426,
  [427] = 427,
  [428] = 428,
  [429] = 429,
  [430] = 430,
  [431] = 431,
  [432] = 432,
  [433] = 433,
  [434] = 434,
  [435] = 435,
  [436] = 436,
  [437] = 437,
  [438] = 438,
  [439] = 439,
  [440] = 440,
  [441] = 441,
  [442] = 442,
  [443] = 443,
  [444] = 444,
  [445] = 445,
  [446] = 446,
  [447] = 447,
  [448] = 448,
  [449] = 449,
  [450] = 450,
  [451] = 451,
  [452] = 452,
  [453] = 453,
  [454] = 454,
  [455] = 455,
  [456] = 456,
  [457] = 457,
  [458] = 458,
  [459] = 459,
  [460] = 460,
  [461] = 461,
  [462] = 462,
  [463] = 463,
  [464] = 464,
  [465] = 465,
  [466] = 466,
  [467] = 467,
  [468] = 468,
  [469] = 469,
  [470] = 470,
  [471] = 471,
  [472] = 472,
  [473] = 473,
  [474] = 474,
  [475] = 475,
  [476] = 476,
  [477] = 477,
  [478] = 478,
  [479] = 479,
  [480] = 480,
  [481] = 481,
  [482] = 482,
  [483] = 483,
  [484] = 484,
  [485] = 485,
  [486] = 486,
  [487] = 487,
  [488] = 488,
  [489] = 489,
  [490] = 490,
  [491] = 491,
  [492] = 492,
  [493] = 493,
  [494] = 494,
  [495] = 495,
  [496] = 496,
  [497] = 497,
  [498] = 498,
  [499] = 499,
  [500] = 500,
  [501] = 501,
  [502] = 502,
  [503] = 503,
  [504] = 504,
  [505] = 505,
  [506] = 506,
  [507] = 507,
  [508] = 508,
  [509] = 509,
  [510] = 510,
  [511] = 511,
  [512] = 512,
  [513] = 513,
  [514] = 514,
  [515] = 515,
  [516] = 516,
  [517] = 517,
  [518] = 518,
  [519] = 519,
  [520] = 520,
  [521] = 521,
  [522] = 522,
  [523] = 523,
  [524] = 524,
  [525] = 525,
  [526] = 526,
  [527] = 527,
  [528] = 528,
  [529] = 529,
  [530] = 530,
  [531] = 531,
  [532] = 532,
  [533] = 533,
  [534] = 529,
  [535] = 535,
  [536] = 536,
  [537] = 537,
  [538] = 538,
  [539] = 539,
  [540] = 540,
  [541] = 541,
  [542] = 542,
  [543] = 543,
  [544] = 544,
  [545] = 545,
  [546] = 543,
  [547] = 547,
  [548] = 548,
  [549] = 549,
  [550] = 550,
  [551] = 551,
  [552] = 552,
  [553] = 553,
  [554] = 554,
  [555] = 555,
  [556] = 554,
  [557] = 555,
  [558] = 558,
  [559] = 559,
  [560] = 560,
  [561] = 561,
  [562] = 562,
  [563] = 563,
  [564] = 564,
  [565] = 565,
  [566] = 566,
  [567] = 567,
  [568] = 566,
  [569] = 569,
  [570] = 567,
  [571] = 477,
  [572] = 487,
  [573] = 474,
  [574] = 574,
  [575] = 482,
  [576] = 485,
  [577] = 577,
  [578] = 578,
  [579] = 579,
  [580] = 580,
  [581] = 578,
  [582] = 582,
  [583] = 493,
  [584] = 584,
  [585] = 585,
  [586] = 496,
  [587] = 587,
  [588] = 588,
  [589] = 589,
  [590] = 590,
  [591] = 591,
  [592] = 491,
  [593] = 593,
  [594] = 594,
  [595] = 494,
  [596] = 510,
  [597] = 498,
  [598] = 598,
  [599] = 599,
  [600] = 600,
  [601] = 601,
  [602] = 602,
  [603] = 603,
  [604] = 604,
  [605] = 605,
  [606] = 606,
  [607] = 603,
  [608] = 601,
  [609] = 600,
};

static inline bool sym__normal_bare_identifier_character_set_1(int32_t c) {
  return (c < 8490
    ? (c < 3285
      ? (c < 2579
        ? (c < 1552
          ? (c < 886
            ? (c < 185
              ? (c < 'g'
                ? (c < '?'
                  ? (c < ':'
                    ? (c >= '!' && c <= '*')
                    : c <= ':')
                  : (c <= 'Z' || c == '^'))
                : (c <= '~' || (c < 178
                  ? (c < 174
                    ? (c >= 169 && c <= 170)
                    : c <= 174)
                  : (c <= 179 || c == 181))))
              : (c <= 186 || (c < 710
                ? (c < 216
                  ? (c < 192
                    ? (c >= 188 && c <= 190)
                    : c <= 214)
                  : (c <= 246 || (c >= 248 && c <= 705)))
                : (c <= 721 || (c < 750
                  ? (c < 748
                    ? (c >= 736 && c <= 740)
                    : c <= 748)
                  : (c <= 750 || (c >= 768 && c <= 884)))))))
            : (c <= 887 || (c < 1329
              ? (c < 908
                ? (c < 902
                  ? (c < 895
                    ? (c >= 890 && c <= 893)
                    : c <= 895)
                  : (c <= 902 || (c >= 904 && c <= 906)))
                : (c <= 908 || (c < 1015
                  ? (c < 931
                    ? (c >= 910 && c <= 929)
                    : c <= 1013)
                  : (c <= 1153 || (c >= 1155 && c <= 1327)))))
              : (c <= 1366 || (c < 1473
                ? (c < 1425
                  ? (c < 1376
                    ? c == 1369
                    : c <= 1416)
                  : (c <= 1469 || c == 1471))
                : (c <= 1474 || (c < 1488
                  ? (c < 1479
                    ? (c >= 1476 && c <= 1477)
                    : c <= 1479)
                  : (c <= 1514 || (c >= 1519 && c <= 1522)))))))))
          : (c <= 1562 || (c < 2417
            ? (c < 2042
              ? (c < 1770
                ? (c < 1749
                  ? (c < 1646
                    ? (c >= 1568 && c <= 1641)
                    : c <= 1747)
                  : (c <= 1756 || (c >= 1759 && c <= 1768)))
                : (c <= 1788 || (c < 1869
                  ? (c < 1808
                    ? c == 1791
                    : c <= 1866)
                  : (c <= 1969 || (c >= 1984 && c <= 2037)))))
              : (c <= 2042 || (c < 2160
                ? (c < 2112
                  ? (c < 2048
                    ? c == 2045
                    : c <= 2093)
                  : (c <= 2139 || (c >= 2144 && c <= 2154)))
                : (c <= 2183 || (c < 2275
                  ? (c < 2200
                    ? (c >= 2185 && c <= 2190)
                    : c <= 2273)
                  : (c <= 2403 || (c >= 2406 && c <= 2415)))))))
            : (c <= 2435 || (c < 2519
              ? (c < 2482
                ? (c < 2451
                  ? (c < 2447
                    ? (c >= 2437 && c <= 2444)
                    : c <= 2448)
                  : (c <= 2472 || (c >= 2474 && c <= 2480)))
                : (c <= 2482 || (c < 2503
                  ? (c < 2492
                    ? (c >= 2486 && c <= 2489)
                    : c <= 2500)
                  : (c <= 2504 || (c >= 2507 && c <= 2510)))))
              : (c <= 2519 || (c < 2556
                ? (c < 2534
                  ? (c < 2527
                    ? (c >= 2524 && c <= 2525)
                    : c <= 2531)
                  : (c <= 2545 || (c >= 2548 && c <= 2553)))
                : (c <= 2556 || (c < 2565
                  ? (c < 2561
                    ? c == 2558
                    : c <= 2563)
                  : (c <= 2570 || (c >= 2575 && c <= 2576)))))))))))
        : (c <= 2600 || (c < 2918
          ? (c < 2748
            ? (c < 2649
              ? (c < 2620
                ? (c < 2613
                  ? (c < 2610
                    ? (c >= 2602 && c <= 2608)
                    : c <= 2611)
                  : (c <= 2614 || (c >= 2616 && c <= 2617)))
                : (c <= 2620 || (c < 2635
                  ? (c < 2631
                    ? (c >= 2622 && c <= 2626)
                    : c <= 2632)
                  : (c <= 2637 || c == 2641))))
              : (c <= 2652 || (c < 2703
                ? (c < 2689
                  ? (c < 2662
                    ? c == 2654
                    : c <= 2677)
                  : (c <= 2691 || (c >= 2693 && c <= 2701)))
                : (c <= 2705 || (c < 2738
                  ? (c < 2730
                    ? (c >= 2707 && c <= 2728)
                    : c <= 2736)
                  : (c <= 2739 || (c >= 2741 && c <= 2745)))))))
            : (c <= 2757 || (c < 2835
              ? (c < 2790
                ? (c < 2768
                  ? (c < 2763
                    ? (c >= 2759 && c <= 2761)
                    : c <= 2765)
                  : (c <= 2768 || (c >= 2784 && c <= 2787)))
                : (c <= 2799 || (c < 2821
                  ? (c < 2817
                    ? (c >= 2809 && c <= 2815)
                    : c <= 2819)
                  : (c <= 2828 || (c >= 2831 && c <= 2832)))))
              : (c <= 2856 || (c < 2887
                ? (c < 2869
                  ? (c < 2866
                    ? (c >= 2858 && c <= 2864)
                    : c <= 2867)
                  : (c <= 2873 || (c >= 2876 && c <= 2884)))
                : (c <= 2888 || (c < 2908
                  ? (c < 2901
                    ? (c >= 2891 && c <= 2893)
                    : c <= 2903)
                  : (c <= 2909 || (c >= 2911 && c <= 2915)))))))))
          : (c <= 2927 || (c < 3090
            ? (c < 2984
              ? (c < 2962
                ? (c < 2949
                  ? (c < 2946
                    ? (c >= 2929 && c <= 2935)
                    : c <= 2947)
                  : (c <= 2954 || (c >= 2958 && c <= 2960)))
                : (c <= 2965 || (c < 2974
                  ? (c < 2972
                    ? (c >= 2969 && c <= 2970)
                    : c <= 2972)
                  : (c <= 2975 || (c >= 2979 && c <= 2980)))))
              : (c <= 2986 || (c < 3024
                ? (c < 3014
                  ? (c < 3006
                    ? (c >= 2990 && c <= 3001)
                    : c <= 3010)
                  : (c <= 3016 || (c >= 3018 && c <= 3021)))
                : (c <= 3024 || (c < 3072
                  ? (c < 3046
                    ? c == 3031
                    : c <= 3058)
                  : (c <= 3084 || (c >= 3086 && c <= 3088)))))))
            : (c <= 3112 || (c < 3192
              ? (c < 3157
                ? (c < 3142
                  ? (c < 3132
                    ? (c >= 3114 && c <= 3129)
                    : c <= 3140)
                  : (c <= 3144 || (c >= 3146 && c <= 3149)))
                : (c <= 3158 || (c < 3168
                  ? (c < 3165
                    ? (c >= 3160 && c <= 3162)
                    : c <= 3165)
                  : (c <= 3171 || (c >= 3174 && c <= 3183)))))
              : (c <= 3198 || (c < 3242
                ? (c < 3214
                  ? (c < 3205
                    ? (c >= 3200 && c <= 3203)
                    : c <= 3212)
                  : (c <= 3216 || (c >= 3218 && c <= 3240)))
                : (c <= 3251 || (c < 3270
                  ? (c < 3260
                    ? (c >= 3253 && c <= 3257)
                    : c <= 3268)
                  : (c <= 3272 || (c >= 3274 && c <= 3277)))))))))))))
      : (c <= 3286 || (c < 5792
        ? (c < 3864
          ? (c < 3535
            ? (c < 3412
              ? (c < 3328
                ? (c < 3302
                  ? (c < 3296
                    ? (c >= 3293 && c <= 3294)
                    : c <= 3299)
                  : (c <= 3311 || (c >= 3313 && c <= 3314)))
                : (c <= 3340 || (c < 3398
                  ? (c < 3346
                    ? (c >= 3342 && c <= 3344)
                    : c <= 3396)
                  : (c <= 3400 || (c >= 3402 && c <= 3406)))))
              : (c <= 3427 || (c < 3482
                ? (c < 3457
                  ? (c < 3450
                    ? (c >= 3430 && c <= 3448)
                    : c <= 3455)
                  : (c <= 3459 || (c >= 3461 && c <= 3478)))
                : (c <= 3505 || (c < 3520
                  ? (c < 3517
                    ? (c >= 3507 && c <= 3515)
                    : c <= 3517)
                  : (c <= 3526 || c == 3530))))))
            : (c <= 3540 || (c < 3718
              ? (c < 3585
                ? (c < 3558
                  ? (c < 3544
                    ? c == 3542
                    : c <= 3551)
                  : (c <= 3567 || (c >= 3570 && c <= 3571)))
                : (c <= 3642 || (c < 3713
                  ? (c < 3664
                    ? (c >= 3648 && c <= 3662)
                    : c <= 3673)
                  : (c <= 3714 || c == 3716))))
              : (c <= 3722 || (c < 3782
                ? (c < 3751
                  ? (c < 3749
                    ? (c >= 3724 && c <= 3747)
                    : c <= 3749)
                  : (c <= 3773 || (c >= 3776 && c <= 3780)))
                : (c <= 3782 || (c < 3804
                  ? (c < 3792
                    ? (c >= 3784 && c <= 3789)
                    : c <= 3801)
                  : (c <= 3807 || c == 3840))))))))
          : (c <= 3865 || (c < 4696
            ? (c < 4038
              ? (c < 3902
                ? (c < 3895
                  ? (c < 3893
                    ? (c >= 3872 && c <= 3891)
                    : c <= 3893)
                  : (c <= 3895 || c == 3897))
                : (c <= 3911 || (c < 3974
                  ? (c < 3953
                    ? (c >= 3913 && c <= 3948)
                    : c <= 3972)
                  : (c <= 3991 || (c >= 3993 && c <= 4028)))))
              : (c <= 4038 || (c < 4301
                ? (c < 4256
                  ? (c < 4176
                    ? (c >= 4096 && c <= 4169)
                    : c <= 4253)
                  : (c <= 4293 || c == 4295))
                : (c <= 4301 || (c < 4682
                  ? (c < 4348
                    ? (c >= 4304 && c <= 4346)
                    : c <= 4680)
                  : (c <= 4685 || (c >= 4688 && c <= 4694)))))))
            : (c <= 4696 || (c < 4824
              ? (c < 4786
                ? (c < 4746
                  ? (c < 4704
                    ? (c >= 4698 && c <= 4701)
                    : c <= 4744)
                  : (c <= 4749 || (c >= 4752 && c <= 4784)))
                : (c <= 4789 || (c < 4802
                  ? (c < 4800
                    ? (c >= 4792 && c <= 4798)
                    : c <= 4800)
                  : (c <= 4805 || (c >= 4808 && c <= 4822)))))
              : (c <= 4880 || (c < 4992
                ? (c < 4957
                  ? (c < 4888
                    ? (c >= 4882 && c <= 4885)
                    : c <= 4954)
                  : (c <= 4959 || (c >= 4969 && c <= 4988)))
                : (c <= 5007 || (c < 5121
                  ? (c < 5112
                    ? (c >= 5024 && c <= 5109)
                    : c <= 5117)
                  : (c <= 5740 || (c >= 5743 && c <= 5786)))))))))))
        : (c <= 5866 || (c < 7296
          ? (c < 6448
            ? (c < 6108
              ? (c < 5984
                ? (c < 5919
                  ? (c < 5888
                    ? (c >= 5870 && c <= 5880)
                    : c <= 5909)
                  : (c <= 5940 || (c >= 5952 && c <= 5971)))
                : (c <= 5996 || (c < 6016
                  ? (c < 6002
                    ? (c >= 5998 && c <= 6000)
                    : c <= 6003)
                  : (c <= 6099 || c == 6103))))
              : (c <= 6109 || (c < 6176
                ? (c < 6155
                  ? (c < 6128
                    ? (c >= 6112 && c <= 6121)
                    : c <= 6137)
                  : (c <= 6157 || (c >= 6159 && c <= 6169)))
                : (c <= 6264 || (c < 6400
                  ? (c < 6320
                    ? (c >= 6272 && c <= 6314)
                    : c <= 6389)
                  : (c <= 6430 || (c >= 6432 && c <= 6443)))))))
            : (c <= 6459 || (c < 6800
              ? (c < 6608
                ? (c < 6528
                  ? (c < 6512
                    ? (c >= 6470 && c <= 6509)
                    : c <= 6516)
                  : (c <= 6571 || (c >= 6576 && c <= 6601)))
                : (c <= 6618 || (c < 6752
                  ? (c < 6688
                    ? (c >= 6656 && c <= 6683)
                    : c <= 6750)
                  : (c <= 6780 || (c >= 6783 && c <= 6793)))))
              : (c <= 6809 || (c < 7019
                ? (c < 6912
                  ? (c < 6832
                    ? c == 6823
                    : c <= 6862)
                  : (c <= 6988 || (c >= 6992 && c <= 7001)))
                : (c <= 7027 || (c < 7232
                  ? (c < 7168
                    ? (c >= 7040 && c <= 7155)
                    : c <= 7223)
                  : (c <= 7241 || (c >= 7245 && c <= 7293)))))))))
          : (c <= 7304 || (c < 8150
            ? (c < 8025
              ? (c < 7424
                ? (c < 7376
                  ? (c < 7357
                    ? (c >= 7312 && c <= 7354)
                    : c <= 7359)
                  : (c <= 7378 || (c >= 7380 && c <= 7418)))
                : (c <= 7957 || (c < 8008
                  ? (c < 7968
                    ? (c >= 7960 && c <= 7965)
                    : c <= 8005)
                  : (c <= 8013 || (c >= 8016 && c <= 8023)))))
              : (c <= 8025 || (c < 8118
                ? (c < 8031
                  ? (c < 8029
                    ? c == 8027
                    : c <= 8029)
                  : (c <= 8061 || (c >= 8064 && c <= 8116)))
                : (c <= 8124 || (c < 8134
                  ? (c < 8130
                    ? c == 8126
                    : c <= 8132)
                  : (c <= 8140 || (c >= 8144 && c <= 8147)))))))
            : (c <= 8155 || (c < 8400
              ? (c < 8265
                ? (c < 8182
                  ? (c < 8178
                    ? (c >= 8160 && c <= 8172)
                    : c <= 8180)
                  : (c <= 8188 || c == 8252))
                : (c <= 8265 || (c < 8319
                  ? (c < 8308
                    ? (c >= 8304 && c <= 8305)
                    : c <= 8313)
                  : (c <= 8329 || (c >= 8336 && c <= 8348)))))
              : (c <= 8432 || (c < 8473
                ? (c < 8458
                  ? (c < 8455
                    ? c == 8450
                    : c <= 8455)
                  : (c <= 8467 || c == 8469))
                : (c <= 8477 || (c < 8486
                  ? (c < 8484
                    ? c == 8482
                    : c <= 8484)
                  : (c <= 8486 || c == 8488))))))))))))))
    : (c <= 8493 || (c < 43744
      ? (c < 10175
        ? (c < 9854
          ? (c < 9728
            ? (c < 9167
              ? (c < 8528
                ? (c < 8517
                  ? (c < 8508
                    ? (c >= 8495 && c <= 8505)
                    : c <= 8511)
                  : (c <= 8521 || c == 8526))
                : (c <= 8585 || (c < 8986
                  ? (c < 8617
                    ? (c >= 8596 && c <= 8601)
                    : c <= 8618)
                  : (c <= 8987 || c == 9000))))
              : (c <= 9167 || (c < 9450
                ? (c < 9312
                  ? (c < 9208
                    ? (c >= 9193 && c <= 9203)
                    : c <= 9210)
                  : (c <= 9371 || c == 9410))
                : (c <= 9471 || (c < 9664
                  ? (c < 9654
                    ? (c >= 9642 && c <= 9643)
                    : c <= 9654)
                  : (c <= 9664 || (c >= 9723 && c <= 9726)))))))
            : (c <= 9732 || (c < 9774
              ? (c < 9757
                ? (c < 9748
                  ? (c < 9745
                    ? c == 9742
                    : c <= 9745)
                  : (c <= 9749 || c == 9752))
                : (c <= 9757 || (c < 9766
                  ? (c < 9762
                    ? c == 9760
                    : c <= 9763)
                  : (c <= 9766 || c == 9770))))
              : (c <= 9775 || (c < 9823
                ? (c < 9794
                  ? (c < 9792
                    ? (c >= 9784 && c <= 9786)
                    : c <= 9792)
                  : (c <= 9794 || (c >= 9800 && c <= 9811)))
                : (c <= 9824 || (c < 9832
                  ? (c < 9829
                    ? c == 9827
                    : c <= 9830)
                  : (c <= 9832 || c == 9851))))))))
          : (c <= 9855 || (c < 9992
            ? (c < 9928
              ? (c < 9895
                ? (c < 9883
                  ? (c < 9881
                    ? (c >= 9874 && c <= 9879)
                    : c <= 9881)
                  : (c <= 9884 || (c >= 9888 && c <= 9889)))
                : (c <= 9895 || (c < 9917
                  ? (c < 9904
                    ? (c >= 9898 && c <= 9899)
                    : c <= 9905)
                  : (c <= 9918 || (c >= 9924 && c <= 9925)))))
              : (c <= 9928 || (c < 9968
                ? (c < 9939
                  ? (c < 9937
                    ? (c >= 9934 && c <= 9935)
                    : c <= 9937)
                  : (c <= 9940 || (c >= 9961 && c <= 9962)))
                : (c <= 9973 || (c < 9986
                  ? (c < 9981
                    ? (c >= 9975 && c <= 9978)
                    : c <= 9981)
                  : (c <= 9986 || c == 9989))))))
            : (c <= 9997 || (c < 10055
              ? (c < 10013
                ? (c < 10004
                  ? (c < 10002
                    ? c == 9999
                    : c <= 10002)
                  : (c <= 10004 || c == 10006))
                : (c <= 10013 || (c < 10035
                  ? (c < 10024
                    ? c == 10017
                    : c <= 10024)
                  : (c <= 10036 || c == 10052))))
              : (c <= 10055 || (c < 10083
                ? (c < 10067
                  ? (c < 10062
                    ? c == 10060
                    : c <= 10062)
                  : (c <= 10069 || c == 10071))
                : (c <= 10084 || (c < 10145
                  ? (c < 10133
                    ? (c >= 10102 && c <= 10131)
                    : c <= 10135)
                  : (c <= 10145 || c == 10160))))))))))
        : (c <= 10175 || (c < 12881
          ? (c < 11720
            ? (c < 11559
              ? (c < 11093
                ? (c < 11035
                  ? (c < 11013
                    ? (c >= 10548 && c <= 10549)
                    : c <= 11015)
                  : (c <= 11036 || c == 11088))
                : (c <= 11093 || (c < 11517
                  ? (c < 11499
                    ? (c >= 11264 && c <= 11492)
                    : c <= 11507)
                  : (c <= 11517 || (c >= 11520 && c <= 11557)))))
              : (c <= 11559 || (c < 11680
                ? (c < 11631
                  ? (c < 11568
                    ? c == 11565
                    : c <= 11623)
                  : (c <= 11631 || (c >= 11647 && c <= 11670)))
                : (c <= 11686 || (c < 11704
                  ? (c < 11696
                    ? (c >= 11688 && c <= 11694)
                    : c <= 11702)
                  : (c <= 11710 || (c >= 11712 && c <= 11718)))))))
            : (c <= 11726 || (c < 12445
              ? (c < 12293
                ? (c < 11744
                  ? (c < 11736
                    ? (c >= 11728 && c <= 11734)
                    : c <= 11742)
                  : (c <= 11775 || c == 11823))
                : (c <= 12295 || (c < 12353
                  ? (c < 12344
                    ? (c >= 12321 && c <= 12341)
                    : c <= 12349)
                  : (c <= 12438 || (c >= 12441 && c <= 12442)))))
              : (c <= 12447 || (c < 12690
                ? (c < 12549
                  ? (c < 12540
                    ? (c >= 12449 && c <= 12538)
                    : c <= 12543)
                  : (c <= 12591 || (c >= 12593 && c <= 12686)))
                : (c <= 12693 || (c < 12832
                  ? (c < 12784
                    ? (c >= 12704 && c <= 12735)
                    : c <= 12799)
                  : (c <= 12841 || (c >= 12872 && c <= 12879)))))))))
          : (c <= 12895 || (c < 42994
            ? (c < 42512
              ? (c < 13312
                ? (c < 12953
                  ? (c < 12951
                    ? (c >= 12928 && c <= 12937)
                    : c <= 12951)
                  : (c <= 12953 || (c >= 12977 && c <= 12991)))
                : (c <= 13312 || (c < 42192
                  ? (c < 19968
                    ? c == 19903
                    : c <= 42124)
                  : (c <= 42237 || (c >= 42240 && c <= 42508)))))
              : (c <= 42539 || (c < 42786
                ? (c < 42623
                  ? (c < 42612
                    ? (c >= 42560 && c <= 42610)
                    : c <= 42621)
                  : (c <= 42737 || (c >= 42775 && c <= 42783)))
                : (c <= 42888 || (c < 42963
                  ? (c < 42960
                    ? (c >= 42891 && c <= 42954)
                    : c <= 42961)
                  : (c <= 42963 || (c >= 42965 && c <= 42969)))))))
            : (c <= 43047 || (c < 43360
              ? (c < 43216
                ? (c < 43072
                  ? (c < 43056
                    ? c == 43052
                    : c <= 43061)
                  : (c <= 43123 || (c >= 43136 && c <= 43205)))
                : (c <= 43225 || (c < 43261
                  ? (c < 43259
                    ? (c >= 43232 && c <= 43255)
                    : c <= 43259)
                  : (c <= 43309 || (c >= 43312 && c <= 43347)))))
              : (c <= 43388 || (c < 43584
                ? (c < 43488
                  ? (c < 43471
                    ? (c >= 43392 && c <= 43456)
                    : c <= 43481)
                  : (c <= 43518 || (c >= 43520 && c <= 43574)))
                : (c <= 43597 || (c < 43642
                  ? (c < 43616
                    ? (c >= 43600 && c <= 43609)
                    : c <= 43638)
                  : (c <= 43714 || (c >= 43739 && c <= 43741)))))))))))))
      : (c <= 43759 || (c < 67424
        ? (c < 65482
          ? (c < 64285
            ? (c < 44012
              ? (c < 43808
                ? (c < 43785
                  ? (c < 43777
                    ? (c >= 43762 && c <= 43766)
                    : c <= 43782)
                  : (c <= 43790 || (c >= 43793 && c <= 43798)))
                : (c <= 43814 || (c < 43868
                  ? (c < 43824
                    ? (c >= 43816 && c <= 43822)
                    : c <= 43866)
                  : (c <= 43881 || (c >= 43888 && c <= 44010)))))
              : (c <= 44013 || (c < 55243
                ? (c < 55203
                  ? (c < 44032
                    ? (c >= 44016 && c <= 44025)
                    : c <= 44032)
                  : (c <= 55203 || (c >= 55216 && c <= 55238)))
                : (c <= 55291 || (c < 64256
                  ? (c < 64112
                    ? (c >= 63744 && c <= 64109)
                    : c <= 64217)
                  : (c <= 64262 || (c >= 64275 && c <= 64279)))))))
            : (c <= 64296 || (c < 65008
              ? (c < 64323
                ? (c < 64318
                  ? (c < 64312
                    ? (c >= 64298 && c <= 64310)
                    : c <= 64316)
                  : (c <= 64318 || (c >= 64320 && c <= 64321)))
                : (c <= 64324 || (c < 64848
                  ? (c < 64467
                    ? (c >= 64326 && c <= 64433)
                    : c <= 64829)
                  : (c <= 64911 || (c >= 64914 && c <= 64967)))))
              : (c <= 65019 || (c < 65296
                ? (c < 65136
                  ? (c < 65056
                    ? (c >= 65024 && c <= 65039)
                    : c <= 65071)
                  : (c <= 65140 || (c >= 65142 && c <= 65276)))
                : (c <= 65305 || (c < 65382
                  ? (c < 65345
                    ? (c >= 65313 && c <= 65338)
                    : c <= 65370)
                  : (c <= 65470 || (c >= 65474 && c <= 65479)))))))))
          : (c <= 65487 || (c < 66432
            ? (c < 65799
              ? (c < 65576
                ? (c < 65536
                  ? (c < 65498
                    ? (c >= 65490 && c <= 65495)
                    : c <= 65500)
                  : (c <= 65547 || (c >= 65549 && c <= 65574)))
                : (c <= 65594 || (c < 65616
                  ? (c < 65599
                    ? (c >= 65596 && c <= 65597)
                    : c <= 65613)
                  : (c <= 65629 || (c >= 65664 && c <= 65786)))))
              : (c <= 65843 || (c < 66208
                ? (c < 66045
                  ? (c < 65930
                    ? (c >= 65856 && c <= 65912)
                    : c <= 65931)
                  : (c <= 66045 || (c >= 66176 && c <= 66204)))
                : (c <= 66256 || (c < 66349
                  ? (c < 66304
                    ? (c >= 66272 && c <= 66299)
                    : c <= 66339)
                  : (c <= 66378 || (c >= 66384 && c <= 66426)))))))
            : (c <= 66461 || (c < 66928
              ? (c < 66720
                ? (c < 66513
                  ? (c < 66504
                    ? (c >= 66464 && c <= 66499)
                    : c <= 66511)
                  : (c <= 66517 || (c >= 66560 && c <= 66717)))
                : (c <= 66729 || (c < 66816
                  ? (c < 66776
                    ? (c >= 66736 && c <= 66771)
                    : c <= 66811)
                  : (c <= 66855 || (c >= 66864 && c <= 66915)))))
              : (c <= 66938 || (c < 66979
                ? (c < 66964
                  ? (c < 66956
                    ? (c >= 66940 && c <= 66954)
                    : c <= 66962)
                  : (c <= 66965 || (c >= 66967 && c <= 66977)))
                : (c <= 66993 || (c < 67072
                  ? (c < 67003
                    ? (c >= 66995 && c <= 67001)
                    : c <= 67004)
                  : (c <= 67382 || (c >= 67392 && c <= 67413)))))))))))
        : (c <= 67431 || (c < 128371
          ? (c < 127358
            ? (c < 67672
              ? (c < 67592
                ? (c < 67506
                  ? (c < 67463
                    ? (c >= 67456 && c <= 67461)
                    : c <= 67504)
                  : (c <= 67514 || (c >= 67584 && c <= 67589)))
                : (c <= 67592 || (c < 67644
                  ? (c < 67639
                    ? (c >= 67594 && c <= 67637)
                    : c <= 67640)
                  : (c <= 67644 || (c >= 67647 && c <= 67669)))))
              : (c <= 67702 || (c < 67835
                ? (c < 67808
                  ? (c < 67751
                    ? (c >= 67705 && c <= 67742)
                    : c <= 67759)
                  : (c <= 67826 || (c >= 67828 && c <= 67829)))
                : (c <= 67867 || (c < 127183
                  ? (c < 126980
                    ? (c >= 67872 && c <= 67883)
                    : c <= 126980)
                  : (c <= 127183 || (c >= 127344 && c <= 127345)))))))
            : (c <= 127359 || (c < 127780
              ? (c < 127514
                ? (c < 127462
                  ? (c < 127377
                    ? c == 127374
                    : c <= 127386)
                  : (c <= 127487 || (c >= 127489 && c <= 127490)))
                : (c <= 127514 || (c < 127568
                  ? (c < 127538
                    ? c == 127535
                    : c <= 127546)
                  : (c <= 127569 || (c >= 127744 && c <= 127777)))))
              : (c <= 127891 || (c < 127991
                ? (c < 127902
                  ? (c < 127897
                    ? (c >= 127894 && c <= 127895)
                    : c <= 127899)
                  : (c <= 127984 || (c >= 127987 && c <= 127989)))
                : (c <= 128253 || (c < 128336
                  ? (c < 128329
                    ? (c >= 128255 && c <= 128317)
                    : c <= 128334)
                  : (c <= 128359 || (c >= 128367 && c <= 128368)))))))))
          : (c <= 128378 || (c < 128725
            ? (c < 128465
              ? (c < 128420
                ? (c < 128400
                  ? (c < 128394
                    ? c == 128391
                    : c <= 128397)
                  : (c <= 128400 || (c >= 128405 && c <= 128406)))
                : (c <= 128421 || (c < 128444
                  ? (c < 128433
                    ? c == 128424
                    : c <= 128434)
                  : (c <= 128444 || (c >= 128450 && c <= 128452)))))
              : (c <= 128467 || (c < 128495
                ? (c < 128483
                  ? (c < 128481
                    ? (c >= 128476 && c <= 128478)
                    : c <= 128481)
                  : (c <= 128483 || c == 128488))
                : (c <= 128495 || (c < 128640
                  ? (c < 128506
                    ? c == 128499
                    : c <= 128591)
                  : (c <= 128709 || (c >= 128715 && c <= 128722)))))))
            : (c <= 128727 || (c < 129351
              ? (c < 128755
                ? (c < 128747
                  ? (c < 128745
                    ? (c >= 128733 && c <= 128741)
                    : c <= 128745)
                  : (c <= 128748 || c == 128752))
                : (c <= 128764 || (c < 129292
                  ? (c < 129008
                    ? (c >= 128992 && c <= 129003)
                    : c <= 129008)
                  : (c <= 129338 || (c >= 129340 && c <= 129349)))))
              : (c <= 129535 || (c < 129712
                ? (c < 129664
                  ? (c < 129656
                    ? (c >= 129648 && c <= 129652)
                    : c <= 129660)
                  : (c <= 129670 || (c >= 129680 && c <= 129708)))
                : (c <= 129722 || (c < 129760
                  ? (c < 129744
                    ? (c >= 129728 && c <= 129733)
                    : c <= 129753)
                  : (c <= 129767 || (c >= 129776 && c <= 129782)))))))))))))))));
}

static inline bool sym__normal_bare_identifier_character_set_2(int32_t c) {
  return (c < 8488
    ? (c < 3274
      ? (c < 2575
        ? (c < 1519
          ? (c < 768
            ? (c < 181
              ? (c < 'a'
                ? (c < ':'
                  ? (c < '.'
                    ? (c >= '#' && c <= '&')
                    : c <= '.')
                  : (c <= ':' || (c < '^'
                    ? (c >= '?' && c <= 'Z')
                    : c <= '_')))
                : (c <= 'z' || (c < 174
                  ? (c < 169
                    ? (c >= '|' && c <= '~')
                    : c <= 170)
                  : (c <= 174 || (c >= 178 && c <= 179)))))
              : (c <= 181 || (c < 248
                ? (c < 192
                  ? (c < 188
                    ? (c >= 185 && c <= 186)
                    : c <= 190)
                  : (c <= 214 || (c >= 216 && c <= 246)))
                : (c <= 705 || (c < 748
                  ? (c < 736
                    ? (c >= 710 && c <= 721)
                    : c <= 740)
                  : (c <= 748 || c == 750))))))
            : (c <= 884 || (c < 1155
              ? (c < 904
                ? (c < 895
                  ? (c < 890
                    ? (c >= 886 && c <= 887)
                    : c <= 893)
                  : (c <= 895 || c == 902))
                : (c <= 906 || (c < 931
                  ? (c < 910
                    ? c == 908
                    : c <= 929)
                  : (c <= 1013 || (c >= 1015 && c <= 1153)))))
              : (c <= 1327 || (c < 1471
                ? (c < 1376
                  ? (c < 1369
                    ? (c >= 1329 && c <= 1366)
                    : c <= 1369)
                  : (c <= 1416 || (c >= 1425 && c <= 1469)))
                : (c <= 1471 || (c < 1479
                  ? (c < 1476
                    ? (c >= 1473 && c <= 1474)
                    : c <= 1477)
                  : (c <= 1479 || (c >= 1488 && c <= 1514)))))))))
          : (c <= 1522 || (c < 2406
            ? (c < 1984
              ? (c < 1759
                ? (c < 1646
                  ? (c < 1568
                    ? (c >= 1552 && c <= 1562)
                    : c <= 1641)
                  : (c <= 1747 || (c >= 1749 && c <= 1756)))
                : (c <= 1768 || (c < 1808
                  ? (c < 1791
                    ? (c >= 1770 && c <= 1788)
                    : c <= 1791)
                  : (c <= 1866 || (c >= 1869 && c <= 1969)))))
              : (c <= 2037 || (c < 2144
                ? (c < 2048
                  ? (c < 2045
                    ? c == 2042
                    : c <= 2045)
                  : (c <= 2093 || (c >= 2112 && c <= 2139)))
                : (c <= 2154 || (c < 2200
                  ? (c < 2185
                    ? (c >= 2160 && c <= 2183)
                    : c <= 2190)
                  : (c <= 2273 || (c >= 2275 && c <= 2403)))))))
            : (c <= 2415 || (c < 2507
              ? (c < 2474
                ? (c < 2447
                  ? (c < 2437
                    ? (c >= 2417 && c <= 2435)
                    : c <= 2444)
                  : (c <= 2448 || (c >= 2451 && c <= 2472)))
                : (c <= 2480 || (c < 2492
                  ? (c < 2486
                    ? c == 2482
                    : c <= 2489)
                  : (c <= 2500 || (c >= 2503 && c <= 2504)))))
              : (c <= 2510 || (c < 2548
                ? (c < 2527
                  ? (c < 2524
                    ? c == 2519
                    : c <= 2525)
                  : (c <= 2531 || (c >= 2534 && c <= 2545)))
                : (c <= 2553 || (c < 2561
                  ? (c < 2558
                    ? c == 2556
                    : c <= 2558)
                  : (c <= 2563 || (c >= 2565 && c <= 2570)))))))))))
        : (c <= 2576 || (c < 2911
          ? (c < 2741
            ? (c < 2641
              ? (c < 2616
                ? (c < 2610
                  ? (c < 2602
                    ? (c >= 2579 && c <= 2600)
                    : c <= 2608)
                  : (c <= 2611 || (c >= 2613 && c <= 2614)))
                : (c <= 2617 || (c < 2631
                  ? (c < 2622
                    ? c == 2620
                    : c <= 2626)
                  : (c <= 2632 || (c >= 2635 && c <= 2637)))))
              : (c <= 2641 || (c < 2693
                ? (c < 2662
                  ? (c < 2654
                    ? (c >= 2649 && c <= 2652)
                    : c <= 2654)
                  : (c <= 2677 || (c >= 2689 && c <= 2691)))
                : (c <= 2701 || (c < 2730
                  ? (c < 2707
                    ? (c >= 2703 && c <= 2705)
                    : c <= 2728)
                  : (c <= 2736 || (c >= 2738 && c <= 2739)))))))
            : (c <= 2745 || (c < 2831
              ? (c < 2784
                ? (c < 2763
                  ? (c < 2759
                    ? (c >= 2748 && c <= 2757)
                    : c <= 2761)
                  : (c <= 2765 || c == 2768))
                : (c <= 2787 || (c < 2817
                  ? (c < 2809
                    ? (c >= 2790 && c <= 2799)
                    : c <= 2815)
                  : (c <= 2819 || (c >= 2821 && c <= 2828)))))
              : (c <= 2832 || (c < 2876
                ? (c < 2866
                  ? (c < 2858
                    ? (c >= 2835 && c <= 2856)
                    : c <= 2864)
                  : (c <= 2867 || (c >= 2869 && c <= 2873)))
                : (c <= 2884 || (c < 2901
                  ? (c < 2891
                    ? (c >= 2887 && c <= 2888)
                    : c <= 2893)
                  : (c <= 2903 || (c >= 2908 && c <= 2909)))))))))
          : (c <= 2915 || (c < 3086
            ? (c < 2979
              ? (c < 2958
                ? (c < 2946
                  ? (c < 2929
                    ? (c >= 2918 && c <= 2927)
                    : c <= 2935)
                  : (c <= 2947 || (c >= 2949 && c <= 2954)))
                : (c <= 2960 || (c < 2972
                  ? (c < 2969
                    ? (c >= 2962 && c <= 2965)
                    : c <= 2970)
                  : (c <= 2972 || (c >= 2974 && c <= 2975)))))
              : (c <= 2980 || (c < 3018
                ? (c < 3006
                  ? (c < 2990
                    ? (c >= 2984 && c <= 2986)
                    : c <= 3001)
                  : (c <= 3010 || (c >= 3014 && c <= 3016)))
                : (c <= 3021 || (c < 3046
                  ? (c < 3031
                    ? c == 3024
                    : c <= 3031)
                  : (c <= 3058 || (c >= 3072 && c <= 3084)))))))
            : (c <= 3088 || (c < 3174
              ? (c < 3146
                ? (c < 3132
                  ? (c < 3114
                    ? (c >= 3090 && c <= 3112)
                    : c <= 3129)
                  : (c <= 3140 || (c >= 3142 && c <= 3144)))
                : (c <= 3149 || (c < 3165
                  ? (c < 3160
                    ? (c >= 3157 && c <= 3158)
                    : c <= 3162)
                  : (c <= 3165 || (c >= 3168 && c <= 3171)))))
              : (c <= 3183 || (c < 3218
                ? (c < 3205
                  ? (c < 3200
                    ? (c >= 3192 && c <= 3198)
                    : c <= 3203)
                  : (c <= 3212 || (c >= 3214 && c <= 3216)))
                : (c <= 3240 || (c < 3260
                  ? (c < 3253
                    ? (c >= 3242 && c <= 3251)
                    : c <= 3257)
                  : (c <= 3268 || (c >= 3270 && c <= 3272)))))))))))))
      : (c <= 3277 || (c < 5743
        ? (c < 3840
          ? (c < 3530
            ? (c < 3402
              ? (c < 3313
                ? (c < 3296
                  ? (c < 3293
                    ? (c >= 3285 && c <= 3286)
                    : c <= 3294)
                  : (c <= 3299 || (c >= 3302 && c <= 3311)))
                : (c <= 3314 || (c < 3346
                  ? (c < 3342
                    ? (c >= 3328 && c <= 3340)
                    : c <= 3344)
                  : (c <= 3396 || (c >= 3398 && c <= 3400)))))
              : (c <= 3406 || (c < 3461
                ? (c < 3450
                  ? (c < 3430
                    ? (c >= 3412 && c <= 3427)
                    : c <= 3448)
                  : (c <= 3455 || (c >= 3457 && c <= 3459)))
                : (c <= 3478 || (c < 3517
                  ? (c < 3507
                    ? (c >= 3482 && c <= 3505)
                    : c <= 3515)
                  : (c <= 3517 || (c >= 3520 && c <= 3526)))))))
            : (c <= 3530 || (c < 3716
              ? (c < 3570
                ? (c < 3544
                  ? (c < 3542
                    ? (c >= 3535 && c <= 3540)
                    : c <= 3542)
                  : (c <= 3551 || (c >= 3558 && c <= 3567)))
                : (c <= 3571 || (c < 3664
                  ? (c < 3648
                    ? (c >= 3585 && c <= 3642)
                    : c <= 3662)
                  : (c <= 3673 || (c >= 3713 && c <= 3714)))))
              : (c <= 3716 || (c < 3776
                ? (c < 3749
                  ? (c < 3724
                    ? (c >= 3718 && c <= 3722)
                    : c <= 3747)
                  : (c <= 3749 || (c >= 3751 && c <= 3773)))
                : (c <= 3780 || (c < 3792
                  ? (c < 3784
                    ? c == 3782
                    : c <= 3789)
                  : (c <= 3801 || (c >= 3804 && c <= 3807)))))))))
          : (c <= 3840 || (c < 4688
            ? (c < 3993
              ? (c < 3897
                ? (c < 3893
                  ? (c < 3872
                    ? (c >= 3864 && c <= 3865)
                    : c <= 3891)
                  : (c <= 3893 || c == 3895))
                : (c <= 3897 || (c < 3953
                  ? (c < 3913
                    ? (c >= 3902 && c <= 3911)
                    : c <= 3948)
                  : (c <= 3972 || (c >= 3974 && c <= 3991)))))
              : (c <= 4028 || (c < 4295
                ? (c < 4176
                  ? (c < 4096
                    ? c == 4038
                    : c <= 4169)
                  : (c <= 4253 || (c >= 4256 && c <= 4293)))
                : (c <= 4295 || (c < 4348
                  ? (c < 4304
                    ? c == 4301
                    : c <= 4346)
                  : (c <= 4680 || (c >= 4682 && c <= 4685)))))))
            : (c <= 4694 || (c < 4808
              ? (c < 4752
                ? (c < 4704
                  ? (c < 4698
                    ? c == 4696
                    : c <= 4701)
                  : (c <= 4744 || (c >= 4746 && c <= 4749)))
                : (c <= 4784 || (c < 4800
                  ? (c < 4792
                    ? (c >= 4786 && c <= 4789)
                    : c <= 4798)
                  : (c <= 4800 || (c >= 4802 && c <= 4805)))))
              : (c <= 4822 || (c < 4969
                ? (c < 4888
                  ? (c < 4882
                    ? (c >= 4824 && c <= 4880)
                    : c <= 4885)
                  : (c <= 4954 || (c >= 4957 && c <= 4959)))
                : (c <= 4988 || (c < 5112
                  ? (c < 5024
                    ? (c >= 4992 && c <= 5007)
                    : c <= 5109)
                  : (c <= 5117 || (c >= 5121 && c <= 5740)))))))))))
        : (c <= 5786 || (c < 7245
          ? (c < 6432
            ? (c < 6103
              ? (c < 5952
                ? (c < 5888
                  ? (c < 5870
                    ? (c >= 5792 && c <= 5866)
                    : c <= 5880)
                  : (c <= 5909 || (c >= 5919 && c <= 5940)))
                : (c <= 5971 || (c < 6002
                  ? (c < 5998
                    ? (c >= 5984 && c <= 5996)
                    : c <= 6000)
                  : (c <= 6003 || (c >= 6016 && c <= 6099)))))
              : (c <= 6103 || (c < 6159
                ? (c < 6128
                  ? (c < 6112
                    ? (c >= 6108 && c <= 6109)
                    : c <= 6121)
                  : (c <= 6137 || (c >= 6155 && c <= 6157)))
                : (c <= 6169 || (c < 6320
                  ? (c < 6272
                    ? (c >= 6176 && c <= 6264)
                    : c <= 6314)
                  : (c <= 6389 || (c >= 6400 && c <= 6430)))))))
            : (c <= 6443 || (c < 6783
              ? (c < 6576
                ? (c < 6512
                  ? (c < 6470
                    ? (c >= 6448 && c <= 6459)
                    : c <= 6509)
                  : (c <= 6516 || (c >= 6528 && c <= 6571)))
                : (c <= 6601 || (c < 6688
                  ? (c < 6656
                    ? (c >= 6608 && c <= 6618)
                    : c <= 6683)
                  : (c <= 6750 || (c >= 6752 && c <= 6780)))))
              : (c <= 6793 || (c < 6992
                ? (c < 6832
                  ? (c < 6823
                    ? (c >= 6800 && c <= 6809)
                    : c <= 6823)
                  : (c <= 6862 || (c >= 6912 && c <= 6988)))
                : (c <= 7001 || (c < 7168
                  ? (c < 7040
                    ? (c >= 7019 && c <= 7027)
                    : c <= 7155)
                  : (c <= 7223 || (c >= 7232 && c <= 7241)))))))))
          : (c <= 7293 || (c < 8144
            ? (c < 8016
              ? (c < 7380
                ? (c < 7357
                  ? (c < 7312
                    ? (c >= 7296 && c <= 7304)
                    : c <= 7354)
                  : (c <= 7359 || (c >= 7376 && c <= 7378)))
                : (c <= 7418 || (c < 7968
                  ? (c < 7960
                    ? (c >= 7424 && c <= 7957)
                    : c <= 7965)
                  : (c <= 8005 || (c >= 8008 && c <= 8013)))))
              : (c <= 8023 || (c < 8064
                ? (c < 8029
                  ? (c < 8027
                    ? c == 8025
                    : c <= 8027)
                  : (c <= 8029 || (c >= 8031 && c <= 8061)))
                : (c <= 8116 || (c < 8130
                  ? (c < 8126
                    ? (c >= 8118 && c <= 8124)
                    : c <= 8126)
                  : (c <= 8132 || (c >= 8134 && c <= 8140)))))))
            : (c <= 8147 || (c < 8336
              ? (c < 8252
                ? (c < 8178
                  ? (c < 8160
                    ? (c >= 8150 && c <= 8155)
                    : c <= 8172)
                  : (c <= 8180 || (c >= 8182 && c <= 8188)))
                : (c <= 8252 || (c < 8308
                  ? (c < 8304
                    ? c == 8265
                    : c <= 8305)
                  : (c <= 8313 || (c >= 8319 && c <= 8329)))))
              : (c <= 8348 || (c < 8469
                ? (c < 8455
                  ? (c < 8450
                    ? (c >= 8400 && c <= 8432)
                    : c <= 8450)
                  : (c <= 8455 || (c >= 8458 && c <= 8467)))
                : (c <= 8469 || (c < 8484
                  ? (c < 8482
                    ? (c >= 8473 && c <= 8477)
                    : c <= 8482)
                  : (c <= 8484 || c == 8486))))))))))))))
    : (c <= 8488 || (c < 43744
      ? (c < 10175
        ? (c < 9854
          ? (c < 9728
            ? (c < 9167
              ? (c < 8528
                ? (c < 8508
                  ? (c < 8495
                    ? (c >= 8490 && c <= 8493)
                    : c <= 8505)
                  : (c <= 8511 || (c < 8526
                    ? (c >= 8517 && c <= 8521)
                    : c <= 8526)))
                : (c <= 8585 || (c < 8986
                  ? (c < 8617
                    ? (c >= 8596 && c <= 8601)
                    : c <= 8618)
                  : (c <= 8987 || c == 9000))))
              : (c <= 9167 || (c < 9450
                ? (c < 9312
                  ? (c < 9208
                    ? (c >= 9193 && c <= 9203)
                    : c <= 9210)
                  : (c <= 9371 || c == 9410))
                : (c <= 9471 || (c < 9664
                  ? (c < 9654
                    ? (c >= 9642 && c <= 9643)
                    : c <= 9654)
                  : (c <= 9664 || (c >= 9723 && c <= 9726)))))))
            : (c <= 9732 || (c < 9774
              ? (c < 9757
                ? (c < 9748
                  ? (c < 9745
                    ? c == 9742
                    : c <= 9745)
                  : (c <= 9749 || c == 9752))
                : (c <= 9757 || (c < 9766
                  ? (c < 9762
                    ? c == 9760
                    : c <= 9763)
                  : (c <= 9766 || c == 9770))))
              : (c <= 9775 || (c < 9823
                ? (c < 9794
                  ? (c < 9792
                    ? (c >= 9784 && c <= 9786)
                    : c <= 9792)
                  : (c <= 9794 || (c >= 9800 && c <= 9811)))
                : (c <= 9824 || (c < 9832
                  ? (c < 9829
                    ? c == 9827
                    : c <= 9830)
                  : (c <= 9832 || c == 9851))))))))
          : (c <= 9855 || (c < 9992
            ? (c < 9928
              ? (c < 9895
                ? (c < 9883
                  ? (c < 9881
                    ? (c >= 9874 && c <= 9879)
                    : c <= 9881)
                  : (c <= 9884 || (c >= 9888 && c <= 9889)))
                : (c <= 9895 || (c < 9917
                  ? (c < 9904
                    ? (c >= 9898 && c <= 9899)
                    : c <= 9905)
                  : (c <= 9918 || (c >= 9924 && c <= 9925)))))
              : (c <= 9928 || (c < 9968
                ? (c < 9939
                  ? (c < 9937
                    ? (c >= 9934 && c <= 9935)
                    : c <= 9937)
                  : (c <= 9940 || (c >= 9961 && c <= 9962)))
                : (c <= 9973 || (c < 9986
                  ? (c < 9981
                    ? (c >= 9975 && c <= 9978)
                    : c <= 9981)
                  : (c <= 9986 || c == 9989))))))
            : (c <= 9997 || (c < 10055
              ? (c < 10013
                ? (c < 10004
                  ? (c < 10002
                    ? c == 9999
                    : c <= 10002)
                  : (c <= 10004 || c == 10006))
                : (c <= 10013 || (c < 10035
                  ? (c < 10024
                    ? c == 10017
                    : c <= 10024)
                  : (c <= 10036 || c == 10052))))
              : (c <= 10055 || (c < 10083
                ? (c < 10067
                  ? (c < 10062
                    ? c == 10060
                    : c <= 10062)
                  : (c <= 10069 || c == 10071))
                : (c <= 10084 || (c < 10145
                  ? (c < 10133
                    ? (c >= 10102 && c <= 10131)
                    : c <= 10135)
                  : (c <= 10145 || c == 10160))))))))))
        : (c <= 10175 || (c < 12881
          ? (c < 11720
            ? (c < 11559
              ? (c < 11093
                ? (c < 11035
                  ? (c < 11013
                    ? (c >= 10548 && c <= 10549)
                    : c <= 11015)
                  : (c <= 11036 || c == 11088))
                : (c <= 11093 || (c < 11517
                  ? (c < 11499
                    ? (c >= 11264 && c <= 11492)
                    : c <= 11507)
                  : (c <= 11517 || (c >= 11520 && c <= 11557)))))
              : (c <= 11559 || (c < 11680
                ? (c < 11631
                  ? (c < 11568
                    ? c == 11565
                    : c <= 11623)
                  : (c <= 11631 || (c >= 11647 && c <= 11670)))
                : (c <= 11686 || (c < 11704
                  ? (c < 11696
                    ? (c >= 11688 && c <= 11694)
                    : c <= 11702)
                  : (c <= 11710 || (c >= 11712 && c <= 11718)))))))
            : (c <= 11726 || (c < 12445
              ? (c < 12293
                ? (c < 11744
                  ? (c < 11736
                    ? (c >= 11728 && c <= 11734)
                    : c <= 11742)
                  : (c <= 11775 || c == 11823))
                : (c <= 12295 || (c < 12353
                  ? (c < 12344
                    ? (c >= 12321 && c <= 12341)
                    : c <= 12349)
                  : (c <= 12438 || (c >= 12441 && c <= 12442)))))
              : (c <= 12447 || (c < 12690
                ? (c < 12549
                  ? (c < 12540
                    ? (c >= 12449 && c <= 12538)
                    : c <= 12543)
                  : (c <= 12591 || (c >= 12593 && c <= 12686)))
                : (c <= 12693 || (c < 12832
                  ? (c < 12784
                    ? (c >= 12704 && c <= 12735)
                    : c <= 12799)
                  : (c <= 12841 || (c >= 12872 && c <= 12879)))))))))
          : (c <= 12895 || (c < 42994
            ? (c < 42512
              ? (c < 13312
                ? (c < 12953
                  ? (c < 12951
                    ? (c >= 12928 && c <= 12937)
                    : c <= 12951)
                  : (c <= 12953 || (c >= 12977 && c <= 12991)))
                : (c <= 13312 || (c < 42192
                  ? (c < 19968
                    ? c == 19903
                    : c <= 42124)
                  : (c <= 42237 || (c >= 42240 && c <= 42508)))))
              : (c <= 42539 || (c < 42786
                ? (c < 42623
                  ? (c < 42612
                    ? (c >= 42560 && c <= 42610)
                    : c <= 42621)
                  : (c <= 42737 || (c >= 42775 && c <= 42783)))
                : (c <= 42888 || (c < 42963
                  ? (c < 42960
                    ? (c >= 42891 && c <= 42954)
                    : c <= 42961)
                  : (c <= 42963 || (c >= 42965 && c <= 42969)))))))
            : (c <= 43047 || (c < 43360
              ? (c < 43216
                ? (c < 43072
                  ? (c < 43056
                    ? c == 43052
                    : c <= 43061)
                  : (c <= 43123 || (c >= 43136 && c <= 43205)))
                : (c <= 43225 || (c < 43261
                  ? (c < 43259
                    ? (c >= 43232 && c <= 43255)
                    : c <= 43259)
                  : (c <= 43309 || (c >= 43312 && c <= 43347)))))
              : (c <= 43388 || (c < 43584
                ? (c < 43488
                  ? (c < 43471
                    ? (c >= 43392 && c <= 43456)
                    : c <= 43481)
                  : (c <= 43518 || (c >= 43520 && c <= 43574)))
                : (c <= 43597 || (c < 43642
                  ? (c < 43616
                    ? (c >= 43600 && c <= 43609)
                    : c <= 43638)
                  : (c <= 43714 || (c >= 43739 && c <= 43741)))))))))))))
      : (c <= 43759 || (c < 67424
        ? (c < 65482
          ? (c < 64285
            ? (c < 44012
              ? (c < 43808
                ? (c < 43785
                  ? (c < 43777
                    ? (c >= 43762 && c <= 43766)
                    : c <= 43782)
                  : (c <= 43790 || (c >= 43793 && c <= 43798)))
                : (c <= 43814 || (c < 43868
                  ? (c < 43824
                    ? (c >= 43816 && c <= 43822)
                    : c <= 43866)
                  : (c <= 43881 || (c >= 43888 && c <= 44010)))))
              : (c <= 44013 || (c < 55243
                ? (c < 55203
                  ? (c < 44032
                    ? (c >= 44016 && c <= 44025)
                    : c <= 44032)
                  : (c <= 55203 || (c >= 55216 && c <= 55238)))
                : (c <= 55291 || (c < 64256
                  ? (c < 64112
                    ? (c >= 63744 && c <= 64109)
                    : c <= 64217)
                  : (c <= 64262 || (c >= 64275 && c <= 64279)))))))
            : (c <= 64296 || (c < 65008
              ? (c < 64323
                ? (c < 64318
                  ? (c < 64312
                    ? (c >= 64298 && c <= 64310)
                    : c <= 64316)
                  : (c <= 64318 || (c >= 64320 && c <= 64321)))
                : (c <= 64324 || (c < 64848
                  ? (c < 64467
                    ? (c >= 64326 && c <= 64433)
                    : c <= 64829)
                  : (c <= 64911 || (c >= 64914 && c <= 64967)))))
              : (c <= 65019 || (c < 65296
                ? (c < 65136
                  ? (c < 65056
                    ? (c >= 65024 && c <= 65039)
                    : c <= 65071)
                  : (c <= 65140 || (c >= 65142 && c <= 65276)))
                : (c <= 65305 || (c < 65382
                  ? (c < 65345
                    ? (c >= 65313 && c <= 65338)
                    : c <= 65370)
                  : (c <= 65470 || (c >= 65474 && c <= 65479)))))))))
          : (c <= 65487 || (c < 66432
            ? (c < 65799
              ? (c < 65576
                ? (c < 65536
                  ? (c < 65498
                    ? (c >= 65490 && c <= 65495)
                    : c <= 65500)
                  : (c <= 65547 || (c >= 65549 && c <= 65574)))
                : (c <= 65594 || (c < 65616
                  ? (c < 65599
                    ? (c >= 65596 && c <= 65597)
                    : c <= 65613)
                  : (c <= 65629 || (c >= 65664 && c <= 65786)))))
              : (c <= 65843 || (c < 66208
                ? (c < 66045
                  ? (c < 65930
                    ? (c >= 65856 && c <= 65912)
                    : c <= 65931)
                  : (c <= 66045 || (c >= 66176 && c <= 66204)))
                : (c <= 66256 || (c < 66349
                  ? (c < 66304
                    ? (c >= 66272 && c <= 66299)
                    : c <= 66339)
                  : (c <= 66378 || (c >= 66384 && c <= 66426)))))))
            : (c <= 66461 || (c < 66928
              ? (c < 66720
                ? (c < 66513
                  ? (c < 66504
                    ? (c >= 66464 && c <= 66499)
                    : c <= 66511)
                  : (c <= 66517 || (c >= 66560 && c <= 66717)))
                : (c <= 66729 || (c < 66816
                  ? (c < 66776
                    ? (c >= 66736 && c <= 66771)
                    : c <= 66811)
                  : (c <= 66855 || (c >= 66864 && c <= 66915)))))
              : (c <= 66938 || (c < 66979
                ? (c < 66964
                  ? (c < 66956
                    ? (c >= 66940 && c <= 66954)
                    : c <= 66962)
                  : (c <= 66965 || (c >= 66967 && c <= 66977)))
                : (c <= 66993 || (c < 67072
                  ? (c < 67003
                    ? (c >= 66995 && c <= 67001)
                    : c <= 67004)
                  : (c <= 67382 || (c >= 67392 && c <= 67413)))))))))))
        : (c <= 67431 || (c < 128371
          ? (c < 127358
            ? (c < 67672
              ? (c < 67592
                ? (c < 67506
                  ? (c < 67463
                    ? (c >= 67456 && c <= 67461)
                    : c <= 67504)
                  : (c <= 67514 || (c >= 67584 && c <= 67589)))
                : (c <= 67592 || (c < 67644
                  ? (c < 67639
                    ? (c >= 67594 && c <= 67637)
                    : c <= 67640)
                  : (c <= 67644 || (c >= 67647 && c <= 67669)))))
              : (c <= 67702 || (c < 67835
                ? (c < 67808
                  ? (c < 67751
                    ? (c >= 67705 && c <= 67742)
                    : c <= 67759)
                  : (c <= 67826 || (c >= 67828 && c <= 67829)))
                : (c <= 67867 || (c < 127183
                  ? (c < 126980
                    ? (c >= 67872 && c <= 67883)
                    : c <= 126980)
                  : (c <= 127183 || (c >= 127344 && c <= 127345)))))))
            : (c <= 127359 || (c < 127780
              ? (c < 127514
                ? (c < 127462
                  ? (c < 127377
                    ? c == 127374
                    : c <= 127386)
                  : (c <= 127487 || (c >= 127489 && c <= 127490)))
                : (c <= 127514 || (c < 127568
                  ? (c < 127538
                    ? c == 127535
                    : c <= 127546)
                  : (c <= 127569 || (c >= 127744 && c <= 127777)))))
              : (c <= 127891 || (c < 127991
                ? (c < 127902
                  ? (c < 127897
                    ? (c >= 127894 && c <= 127895)
                    : c <= 127899)
                  : (c <= 127984 || (c >= 127987 && c <= 127989)))
                : (c <= 128253 || (c < 128336
                  ? (c < 128329
                    ? (c >= 128255 && c <= 128317)
                    : c <= 128334)
                  : (c <= 128359 || (c >= 128367 && c <= 128368)))))))))
          : (c <= 128378 || (c < 128725
            ? (c < 128465
              ? (c < 128420
                ? (c < 128400
                  ? (c < 128394
                    ? c == 128391
                    : c <= 128397)
                  : (c <= 128400 || (c >= 128405 && c <= 128406)))
                : (c <= 128421 || (c < 128444
                  ? (c < 128433
                    ? c == 128424
                    : c <= 128434)
                  : (c <= 128444 || (c >= 128450 && c <= 128452)))))
              : (c <= 128467 || (c < 128495
                ? (c < 128483
                  ? (c < 128481
                    ? (c >= 128476 && c <= 128478)
                    : c <= 128481)
                  : (c <= 128483 || c == 128488))
                : (c <= 128495 || (c < 128640
                  ? (c < 128506
                    ? c == 128499
                    : c <= 128591)
                  : (c <= 128709 || (c >= 128715 && c <= 128722)))))))
            : (c <= 128727 || (c < 129351
              ? (c < 128755
                ? (c < 128747
                  ? (c < 128745
                    ? (c >= 128733 && c <= 128741)
                    : c <= 128745)
                  : (c <= 128748 || c == 128752))
                : (c <= 128764 || (c < 129292
                  ? (c < 129008
                    ? (c >= 128992 && c <= 129003)
                    : c <= 129008)
                  : (c <= 129338 || (c >= 129340 && c <= 129349)))))
              : (c <= 129535 || (c < 129712
                ? (c < 129664
                  ? (c < 129656
                    ? (c >= 129648 && c <= 129652)
                    : c <= 129660)
                  : (c <= 129670 || (c >= 129680 && c <= 129708)))
                : (c <= 129722 || (c < 129760
                  ? (c < 129744
                    ? (c >= 129728 && c <= 129733)
                    : c <= 129753)
                  : (c <= 129767 || (c >= 129776 && c <= 129782)))))))))))))))));
}

static inline bool sym__normal_bare_identifier_character_set_3(int32_t c) {
  return (c < 8490
    ? (c < 3285
      ? (c < 2579
        ? (c < 1552
          ? (c < 886
            ? (c < 185
              ? (c < 'a'
                ? (c < '?'
                  ? (c < '.'
                    ? (c >= '!' && c <= '*')
                    : c <= ':')
                  : (c <= 'Z' || (c >= '^' && c <= '_')))
                : (c <= '~' || (c < 178
                  ? (c < 174
                    ? (c >= 169 && c <= 170)
                    : c <= 174)
                  : (c <= 179 || c == 181))))
              : (c <= 186 || (c < 710
                ? (c < 216
                  ? (c < 192
                    ? (c >= 188 && c <= 190)
                    : c <= 214)
                  : (c <= 246 || (c >= 248 && c <= 705)))
                : (c <= 721 || (c < 750
                  ? (c < 748
                    ? (c >= 736 && c <= 740)
                    : c <= 748)
                  : (c <= 750 || (c >= 768 && c <= 884)))))))
            : (c <= 887 || (c < 1329
              ? (c < 908
                ? (c < 902
                  ? (c < 895
                    ? (c >= 890 && c <= 893)
                    : c <= 895)
                  : (c <= 902 || (c >= 904 && c <= 906)))
                : (c <= 908 || (c < 1015
                  ? (c < 931
                    ? (c >= 910 && c <= 929)
                    : c <= 1013)
                  : (c <= 1153 || (c >= 1155 && c <= 1327)))))
              : (c <= 1366 || (c < 1473
                ? (c < 1425
                  ? (c < 1376
                    ? c == 1369
                    : c <= 1416)
                  : (c <= 1469 || c == 1471))
                : (c <= 1474 || (c < 1488
                  ? (c < 1479
                    ? (c >= 1476 && c <= 1477)
                    : c <= 1479)
                  : (c <= 1514 || (c >= 1519 && c <= 1522)))))))))
          : (c <= 1562 || (c < 2417
            ? (c < 2042
              ? (c < 1770
                ? (c < 1749
                  ? (c < 1646
                    ? (c >= 1568 && c <= 1641)
                    : c <= 1747)
                  : (c <= 1756 || (c >= 1759 && c <= 1768)))
                : (c <= 1788 || (c < 1869
                  ? (c < 1808
                    ? c == 1791
                    : c <= 1866)
                  : (c <= 1969 || (c >= 1984 && c <= 2037)))))
              : (c <= 2042 || (c < 2160
                ? (c < 2112
                  ? (c < 2048
                    ? c == 2045
                    : c <= 2093)
                  : (c <= 2139 || (c >= 2144 && c <= 2154)))
                : (c <= 2183 || (c < 2275
                  ? (c < 2200
                    ? (c >= 2185 && c <= 2190)
                    : c <= 2273)
                  : (c <= 2403 || (c >= 2406 && c <= 2415)))))))
            : (c <= 2435 || (c < 2519
              ? (c < 2482
                ? (c < 2451
                  ? (c < 2447
                    ? (c >= 2437 && c <= 2444)
                    : c <= 2448)
                  : (c <= 2472 || (c >= 2474 && c <= 2480)))
                : (c <= 2482 || (c < 2503
                  ? (c < 2492
                    ? (c >= 2486 && c <= 2489)
                    : c <= 2500)
                  : (c <= 2504 || (c >= 2507 && c <= 2510)))))
              : (c <= 2519 || (c < 2556
                ? (c < 2534
                  ? (c < 2527
                    ? (c >= 2524 && c <= 2525)
                    : c <= 2531)
                  : (c <= 2545 || (c >= 2548 && c <= 2553)))
                : (c <= 2556 || (c < 2565
                  ? (c < 2561
                    ? c == 2558
                    : c <= 2563)
                  : (c <= 2570 || (c >= 2575 && c <= 2576)))))))))))
        : (c <= 2600 || (c < 2918
          ? (c < 2748
            ? (c < 2649
              ? (c < 2620
                ? (c < 2613
                  ? (c < 2610
                    ? (c >= 2602 && c <= 2608)
                    : c <= 2611)
                  : (c <= 2614 || (c >= 2616 && c <= 2617)))
                : (c <= 2620 || (c < 2635
                  ? (c < 2631
                    ? (c >= 2622 && c <= 2626)
                    : c <= 2632)
                  : (c <= 2637 || c == 2641))))
              : (c <= 2652 || (c < 2703
                ? (c < 2689
                  ? (c < 2662
                    ? c == 2654
                    : c <= 2677)
                  : (c <= 2691 || (c >= 2693 && c <= 2701)))
                : (c <= 2705 || (c < 2738
                  ? (c < 2730
                    ? (c >= 2707 && c <= 2728)
                    : c <= 2736)
                  : (c <= 2739 || (c >= 2741 && c <= 2745)))))))
            : (c <= 2757 || (c < 2835
              ? (c < 2790
                ? (c < 2768
                  ? (c < 2763
                    ? (c >= 2759 && c <= 2761)
                    : c <= 2765)
                  : (c <= 2768 || (c >= 2784 && c <= 2787)))
                : (c <= 2799 || (c < 2821
                  ? (c < 2817
                    ? (c >= 2809 && c <= 2815)
                    : c <= 2819)
                  : (c <= 2828 || (c >= 2831 && c <= 2832)))))
              : (c <= 2856 || (c < 2887
                ? (c < 2869
                  ? (c < 2866
                    ? (c >= 2858 && c <= 2864)
                    : c <= 2867)
                  : (c <= 2873 || (c >= 2876 && c <= 2884)))
                : (c <= 2888 || (c < 2908
                  ? (c < 2901
                    ? (c >= 2891 && c <= 2893)
                    : c <= 2903)
                  : (c <= 2909 || (c >= 2911 && c <= 2915)))))))))
          : (c <= 2927 || (c < 3090
            ? (c < 2984
              ? (c < 2962
                ? (c < 2949
                  ? (c < 2946
                    ? (c >= 2929 && c <= 2935)
                    : c <= 2947)
                  : (c <= 2954 || (c >= 2958 && c <= 2960)))
                : (c <= 2965 || (c < 2974
                  ? (c < 2972
                    ? (c >= 2969 && c <= 2970)
                    : c <= 2972)
                  : (c <= 2975 || (c >= 2979 && c <= 2980)))))
              : (c <= 2986 || (c < 3024
                ? (c < 3014
                  ? (c < 3006
                    ? (c >= 2990 && c <= 3001)
                    : c <= 3010)
                  : (c <= 3016 || (c >= 3018 && c <= 3021)))
                : (c <= 3024 || (c < 3072
                  ? (c < 3046
                    ? c == 3031
                    : c <= 3058)
                  : (c <= 3084 || (c >= 3086 && c <= 3088)))))))
            : (c <= 3112 || (c < 3192
              ? (c < 3157
                ? (c < 3142
                  ? (c < 3132
                    ? (c >= 3114 && c <= 3129)
                    : c <= 3140)
                  : (c <= 3144 || (c >= 3146 && c <= 3149)))
                : (c <= 3158 || (c < 3168
                  ? (c < 3165
                    ? (c >= 3160 && c <= 3162)
                    : c <= 3165)
                  : (c <= 3171 || (c >= 3174 && c <= 3183)))))
              : (c <= 3198 || (c < 3242
                ? (c < 3214
                  ? (c < 3205
                    ? (c >= 3200 && c <= 3203)
                    : c <= 3212)
                  : (c <= 3216 || (c >= 3218 && c <= 3240)))
                : (c <= 3251 || (c < 3270
                  ? (c < 3260
                    ? (c >= 3253 && c <= 3257)
                    : c <= 3268)
                  : (c <= 3272 || (c >= 3274 && c <= 3277)))))))))))))
      : (c <= 3286 || (c < 5792
        ? (c < 3864
          ? (c < 3535
            ? (c < 3412
              ? (c < 3328
                ? (c < 3302
                  ? (c < 3296
                    ? (c >= 3293 && c <= 3294)
                    : c <= 3299)
                  : (c <= 3311 || (c >= 3313 && c <= 3314)))
                : (c <= 3340 || (c < 3398
                  ? (c < 3346
                    ? (c >= 3342 && c <= 3344)
                    : c <= 3396)
                  : (c <= 3400 || (c >= 3402 && c <= 3406)))))
              : (c <= 3427 || (c < 3482
                ? (c < 3457
                  ? (c < 3450
                    ? (c >= 3430 && c <= 3448)
                    : c <= 3455)
                  : (c <= 3459 || (c >= 3461 && c <= 3478)))
                : (c <= 3505 || (c < 3520
                  ? (c < 3517
                    ? (c >= 3507 && c <= 3515)
                    : c <= 3517)
                  : (c <= 3526 || c == 3530))))))
            : (c <= 3540 || (c < 3718
              ? (c < 3585
                ? (c < 3558
                  ? (c < 3544
                    ? c == 3542
                    : c <= 3551)
                  : (c <= 3567 || (c >= 3570 && c <= 3571)))
                : (c <= 3642 || (c < 3713
                  ? (c < 3664
                    ? (c >= 3648 && c <= 3662)
                    : c <= 3673)
                  : (c <= 3714 || c == 3716))))
              : (c <= 3722 || (c < 3782
                ? (c < 3751
                  ? (c < 3749
                    ? (c >= 3724 && c <= 3747)
                    : c <= 3749)
                  : (c <= 3773 || (c >= 3776 && c <= 3780)))
                : (c <= 3782 || (c < 3804
                  ? (c < 3792
                    ? (c >= 3784 && c <= 3789)
                    : c <= 3801)
                  : (c <= 3807 || c == 3840))))))))
          : (c <= 3865 || (c < 4696
            ? (c < 4038
              ? (c < 3902
                ? (c < 3895
                  ? (c < 3893
                    ? (c >= 3872 && c <= 3891)
                    : c <= 3893)
                  : (c <= 3895 || c == 3897))
                : (c <= 3911 || (c < 3974
                  ? (c < 3953
                    ? (c >= 3913 && c <= 3948)
                    : c <= 3972)
                  : (c <= 3991 || (c >= 3993 && c <= 4028)))))
              : (c <= 4038 || (c < 4301
                ? (c < 4256
                  ? (c < 4176
                    ? (c >= 4096 && c <= 4169)
                    : c <= 4253)
                  : (c <= 4293 || c == 4295))
                : (c <= 4301 || (c < 4682
                  ? (c < 4348
                    ? (c >= 4304 && c <= 4346)
                    : c <= 4680)
                  : (c <= 4685 || (c >= 4688 && c <= 4694)))))))
            : (c <= 4696 || (c < 4824
              ? (c < 4786
                ? (c < 4746
                  ? (c < 4704
                    ? (c >= 4698 && c <= 4701)
                    : c <= 4744)
                  : (c <= 4749 || (c >= 4752 && c <= 4784)))
                : (c <= 4789 || (c < 4802
                  ? (c < 4800
                    ? (c >= 4792 && c <= 4798)
                    : c <= 4800)
                  : (c <= 4805 || (c >= 4808 && c <= 4822)))))
              : (c <= 4880 || (c < 4992
                ? (c < 4957
                  ? (c < 4888
                    ? (c >= 4882 && c <= 4885)
                    : c <= 4954)
                  : (c <= 4959 || (c >= 4969 && c <= 4988)))
                : (c <= 5007 || (c < 5121
                  ? (c < 5112
                    ? (c >= 5024 && c <= 5109)
                    : c <= 5117)
                  : (c <= 5740 || (c >= 5743 && c <= 5786)))))))))))
        : (c <= 5866 || (c < 7296
          ? (c < 6448
            ? (c < 6108
              ? (c < 5984
                ? (c < 5919
                  ? (c < 5888
                    ? (c >= 5870 && c <= 5880)
                    : c <= 5909)
                  : (c <= 5940 || (c >= 5952 && c <= 5971)))
                : (c <= 5996 || (c < 6016
                  ? (c < 6002
                    ? (c >= 5998 && c <= 6000)
                    : c <= 6003)
                  : (c <= 6099 || c == 6103))))
              : (c <= 6109 || (c < 6176
                ? (c < 6155
                  ? (c < 6128
                    ? (c >= 6112 && c <= 6121)
                    : c <= 6137)
                  : (c <= 6157 || (c >= 6159 && c <= 6169)))
                : (c <= 6264 || (c < 6400
                  ? (c < 6320
                    ? (c >= 6272 && c <= 6314)
                    : c <= 6389)
                  : (c <= 6430 || (c >= 6432 && c <= 6443)))))))
            : (c <= 6459 || (c < 6800
              ? (c < 6608
                ? (c < 6528
                  ? (c < 6512
                    ? (c >= 6470 && c <= 6509)
                    : c <= 6516)
                  : (c <= 6571 || (c >= 6576 && c <= 6601)))
                : (c <= 6618 || (c < 6752
                  ? (c < 6688
                    ? (c >= 6656 && c <= 6683)
                    : c <= 6750)
                  : (c <= 6780 || (c >= 6783 && c <= 6793)))))
              : (c <= 6809 || (c < 7019
                ? (c < 6912
                  ? (c < 6832
                    ? c == 6823
                    : c <= 6862)
                  : (c <= 6988 || (c >= 6992 && c <= 7001)))
                : (c <= 7027 || (c < 7232
                  ? (c < 7168
                    ? (c >= 7040 && c <= 7155)
                    : c <= 7223)
                  : (c <= 7241 || (c >= 7245 && c <= 7293)))))))))
          : (c <= 7304 || (c < 8150
            ? (c < 8025
              ? (c < 7424
                ? (c < 7376
                  ? (c < 7357
                    ? (c >= 7312 && c <= 7354)
                    : c <= 7359)
                  : (c <= 7378 || (c >= 7380 && c <= 7418)))
                : (c <= 7957 || (c < 8008
                  ? (c < 7968
                    ? (c >= 7960 && c <= 7965)
                    : c <= 8005)
                  : (c <= 8013 || (c >= 8016 && c <= 8023)))))
              : (c <= 8025 || (c < 8118
                ? (c < 8031
                  ? (c < 8029
                    ? c == 8027
                    : c <= 8029)
                  : (c <= 8061 || (c >= 8064 && c <= 8116)))
                : (c <= 8124 || (c < 8134
                  ? (c < 8130
                    ? c == 8126
                    : c <= 8132)
                  : (c <= 8140 || (c >= 8144 && c <= 8147)))))))
            : (c <= 8155 || (c < 8400
              ? (c < 8265
                ? (c < 8182
                  ? (c < 8178
                    ? (c >= 8160 && c <= 8172)
                    : c <= 8180)
                  : (c <= 8188 || c == 8252))
                : (c <= 8265 || (c < 8319
                  ? (c < 8308
                    ? (c >= 8304 && c <= 8305)
                    : c <= 8313)
                  : (c <= 8329 || (c >= 8336 && c <= 8348)))))
              : (c <= 8432 || (c < 8473
                ? (c < 8458
                  ? (c < 8455
                    ? c == 8450
                    : c <= 8455)
                  : (c <= 8467 || c == 8469))
                : (c <= 8477 || (c < 8486
                  ? (c < 8484
                    ? c == 8482
                    : c <= 8484)
                  : (c <= 8486 || c == 8488))))))))))))))
    : (c <= 8493 || (c < 43744
      ? (c < 10175
        ? (c < 9854
          ? (c < 9728
            ? (c < 9167
              ? (c < 8528
                ? (c < 8517
                  ? (c < 8508
                    ? (c >= 8495 && c <= 8505)
                    : c <= 8511)
                  : (c <= 8521 || c == 8526))
                : (c <= 8585 || (c < 8986
                  ? (c < 8617
                    ? (c >= 8596 && c <= 8601)
                    : c <= 8618)
                  : (c <= 8987 || c == 9000))))
              : (c <= 9167 || (c < 9450
                ? (c < 9312
                  ? (c < 9208
                    ? (c >= 9193 && c <= 9203)
                    : c <= 9210)
                  : (c <= 9371 || c == 9410))
                : (c <= 9471 || (c < 9664
                  ? (c < 9654
                    ? (c >= 9642 && c <= 9643)
                    : c <= 9654)
                  : (c <= 9664 || (c >= 9723 && c <= 9726)))))))
            : (c <= 9732 || (c < 9774
              ? (c < 9757
                ? (c < 9748
                  ? (c < 9745
                    ? c == 9742
                    : c <= 9745)
                  : (c <= 9749 || c == 9752))
                : (c <= 9757 || (c < 9766
                  ? (c < 9762
                    ? c == 9760
                    : c <= 9763)
                  : (c <= 9766 || c == 9770))))
              : (c <= 9775 || (c < 9823
                ? (c < 9794
                  ? (c < 9792
                    ? (c >= 9784 && c <= 9786)
                    : c <= 9792)
                  : (c <= 9794 || (c >= 9800 && c <= 9811)))
                : (c <= 9824 || (c < 9832
                  ? (c < 9829
                    ? c == 9827
                    : c <= 9830)
                  : (c <= 9832 || c == 9851))))))))
          : (c <= 9855 || (c < 9992
            ? (c < 9928
              ? (c < 9895
                ? (c < 9883
                  ? (c < 9881
                    ? (c >= 9874 && c <= 9879)
                    : c <= 9881)
                  : (c <= 9884 || (c >= 9888 && c <= 9889)))
                : (c <= 9895 || (c < 9917
                  ? (c < 9904
                    ? (c >= 9898 && c <= 9899)
                    : c <= 9905)
                  : (c <= 9918 || (c >= 9924 && c <= 9925)))))
              : (c <= 9928 || (c < 9968
                ? (c < 9939
                  ? (c < 9937
                    ? (c >= 9934 && c <= 9935)
                    : c <= 9937)
                  : (c <= 9940 || (c >= 9961 && c <= 9962)))
                : (c <= 9973 || (c < 9986
                  ? (c < 9981
                    ? (c >= 9975 && c <= 9978)
                    : c <= 9981)
                  : (c <= 9986 || c == 9989))))))
            : (c <= 9997 || (c < 10055
              ? (c < 10013
                ? (c < 10004
                  ? (c < 10002
                    ? c == 9999
                    : c <= 10002)
                  : (c <= 10004 || c == 10006))
                : (c <= 10013 || (c < 10035
                  ? (c < 10024
                    ? c == 10017
                    : c <= 10024)
                  : (c <= 10036 || c == 10052))))
              : (c <= 10055 || (c < 10083
                ? (c < 10067
                  ? (c < 10062
                    ? c == 10060
                    : c <= 10062)
                  : (c <= 10069 || c == 10071))
                : (c <= 10084 || (c < 10145
                  ? (c < 10133
                    ? (c >= 10102 && c <= 10131)
                    : c <= 10135)
                  : (c <= 10145 || c == 10160))))))))))
        : (c <= 10175 || (c < 12881
          ? (c < 11720
            ? (c < 11559
              ? (c < 11093
                ? (c < 11035
                  ? (c < 11013
                    ? (c >= 10548 && c <= 10549)
                    : c <= 11015)
                  : (c <= 11036 || c == 11088))
                : (c <= 11093 || (c < 11517
                  ? (c < 11499
                    ? (c >= 11264 && c <= 11492)
                    : c <= 11507)
                  : (c <= 11517 || (c >= 11520 && c <= 11557)))))
              : (c <= 11559 || (c < 11680
                ? (c < 11631
                  ? (c < 11568
                    ? c == 11565
                    : c <= 11623)
                  : (c <= 11631 || (c >= 11647 && c <= 11670)))
                : (c <= 11686 || (c < 11704
                  ? (c < 11696
                    ? (c >= 11688 && c <= 11694)
                    : c <= 11702)
                  : (c <= 11710 || (c >= 11712 && c <= 11718)))))))
            : (c <= 11726 || (c < 12445
              ? (c < 12293
                ? (c < 11744
                  ? (c < 11736
                    ? (c >= 11728 && c <= 11734)
                    : c <= 11742)
                  : (c <= 11775 || c == 11823))
                : (c <= 12295 || (c < 12353
                  ? (c < 12344
                    ? (c >= 12321 && c <= 12341)
                    : c <= 12349)
                  : (c <= 12438 || (c >= 12441 && c <= 12442)))))
              : (c <= 12447 || (c < 12690
                ? (c < 12549
                  ? (c < 12540
                    ? (c >= 12449 && c <= 12538)
                    : c <= 12543)
                  : (c <= 12591 || (c >= 12593 && c <= 12686)))
                : (c <= 12693 || (c < 12832
                  ? (c < 12784
                    ? (c >= 12704 && c <= 12735)
                    : c <= 12799)
                  : (c <= 12841 || (c >= 12872 && c <= 12879)))))))))
          : (c <= 12895 || (c < 42994
            ? (c < 42512
              ? (c < 13312
                ? (c < 12953
                  ? (c < 12951
                    ? (c >= 12928 && c <= 12937)
                    : c <= 12951)
                  : (c <= 12953 || (c >= 12977 && c <= 12991)))
                : (c <= 13312 || (c < 42192
                  ? (c < 19968
                    ? c == 19903
                    : c <= 42124)
                  : (c <= 42237 || (c >= 42240 && c <= 42508)))))
              : (c <= 42539 || (c < 42786
                ? (c < 42623
                  ? (c < 42612
                    ? (c >= 42560 && c <= 42610)
                    : c <= 42621)
                  : (c <= 42737 || (c >= 42775 && c <= 42783)))
                : (c <= 42888 || (c < 42963
                  ? (c < 42960
                    ? (c >= 42891 && c <= 42954)
                    : c <= 42961)
                  : (c <= 42963 || (c >= 42965 && c <= 42969)))))))
            : (c <= 43047 || (c < 43360
              ? (c < 43216
                ? (c < 43072
                  ? (c < 43056
                    ? c == 43052
                    : c <= 43061)
                  : (c <= 43123 || (c >= 43136 && c <= 43205)))
                : (c <= 43225 || (c < 43261
                  ? (c < 43259
                    ? (c >= 43232 && c <= 43255)
                    : c <= 43259)
                  : (c <= 43309 || (c >= 43312 && c <= 43347)))))
              : (c <= 43388 || (c < 43584
                ? (c < 43488
                  ? (c < 43471
                    ? (c >= 43392 && c <= 43456)
                    : c <= 43481)
                  : (c <= 43518 || (c >= 43520 && c <= 43574)))
                : (c <= 43597 || (c < 43642
                  ? (c < 43616
                    ? (c >= 43600 && c <= 43609)
                    : c <= 43638)
                  : (c <= 43714 || (c >= 43739 && c <= 43741)))))))))))))
      : (c <= 43759 || (c < 67424
        ? (c < 65482
          ? (c < 64285
            ? (c < 44012
              ? (c < 43808
                ? (c < 43785
                  ? (c < 43777
                    ? (c >= 43762 && c <= 43766)
                    : c <= 43782)
                  : (c <= 43790 || (c >= 43793 && c <= 43798)))
                : (c <= 43814 || (c < 43868
                  ? (c < 43824
                    ? (c >= 43816 && c <= 43822)
                    : c <= 43866)
                  : (c <= 43881 || (c >= 43888 && c <= 44010)))))
              : (c <= 44013 || (c < 55243
                ? (c < 55203
                  ? (c < 44032
                    ? (c >= 44016 && c <= 44025)
                    : c <= 44032)
                  : (c <= 55203 || (c >= 55216 && c <= 55238)))
                : (c <= 55291 || (c < 64256
                  ? (c < 64112
                    ? (c >= 63744 && c <= 64109)
                    : c <= 64217)
                  : (c <= 64262 || (c >= 64275 && c <= 64279)))))))
            : (c <= 64296 || (c < 65008
              ? (c < 64323
                ? (c < 64318
                  ? (c < 64312
                    ? (c >= 64298 && c <= 64310)
                    : c <= 64316)
                  : (c <= 64318 || (c >= 64320 && c <= 64321)))
                : (c <= 64324 || (c < 64848
                  ? (c < 64467
                    ? (c >= 64326 && c <= 64433)
                    : c <= 64829)
                  : (c <= 64911 || (c >= 64914 && c <= 64967)))))
              : (c <= 65019 || (c < 65296
                ? (c < 65136
                  ? (c < 65056
                    ? (c >= 65024 && c <= 65039)
                    : c <= 65071)
                  : (c <= 65140 || (c >= 65142 && c <= 65276)))
                : (c <= 65305 || (c < 65382
                  ? (c < 65345
                    ? (c >= 65313 && c <= 65338)
                    : c <= 65370)
                  : (c <= 65470 || (c >= 65474 && c <= 65479)))))))))
          : (c <= 65487 || (c < 66432
            ? (c < 65799
              ? (c < 65576
                ? (c < 65536
                  ? (c < 65498
                    ? (c >= 65490 && c <= 65495)
                    : c <= 65500)
                  : (c <= 65547 || (c >= 65549 && c <= 65574)))
                : (c <= 65594 || (c < 65616
                  ? (c < 65599
                    ? (c >= 65596 && c <= 65597)
                    : c <= 65613)
                  : (c <= 65629 || (c >= 65664 && c <= 65786)))))
              : (c <= 65843 || (c < 66208
                ? (c < 66045
                  ? (c < 65930
                    ? (c >= 65856 && c <= 65912)
                    : c <= 65931)
                  : (c <= 66045 || (c >= 66176 && c <= 66204)))
                : (c <= 66256 || (c < 66349
                  ? (c < 66304
                    ? (c >= 66272 && c <= 66299)
                    : c <= 66339)
                  : (c <= 66378 || (c >= 66384 && c <= 66426)))))))
            : (c <= 66461 || (c < 66928
              ? (c < 66720
                ? (c < 66513
                  ? (c < 66504
                    ? (c >= 66464 && c <= 66499)
                    : c <= 66511)
                  : (c <= 66517 || (c >= 66560 && c <= 66717)))
                : (c <= 66729 || (c < 66816
                  ? (c < 66776
                    ? (c >= 66736 && c <= 66771)
                    : c <= 66811)
                  : (c <= 66855 || (c >= 66864 && c <= 66915)))))
              : (c <= 66938 || (c < 66979
                ? (c < 66964
                  ? (c < 66956
                    ? (c >= 66940 && c <= 66954)
                    : c <= 66962)
                  : (c <= 66965 || (c >= 66967 && c <= 66977)))
                : (c <= 66993 || (c < 67072
                  ? (c < 67003
                    ? (c >= 66995 && c <= 67001)
                    : c <= 67004)
                  : (c <= 67382 || (c >= 67392 && c <= 67413)))))))))))
        : (c <= 67431 || (c < 128371
          ? (c < 127358
            ? (c < 67672
              ? (c < 67592
                ? (c < 67506
                  ? (c < 67463
                    ? (c >= 67456 && c <= 67461)
                    : c <= 67504)
                  : (c <= 67514 || (c >= 67584 && c <= 67589)))
                : (c <= 67592 || (c < 67644
                  ? (c < 67639
                    ? (c >= 67594 && c <= 67637)
                    : c <= 67640)
                  : (c <= 67644 || (c >= 67647 && c <= 67669)))))
              : (c <= 67702 || (c < 67835
                ? (c < 67808
                  ? (c < 67751
                    ? (c >= 67705 && c <= 67742)
                    : c <= 67759)
                  : (c <= 67826 || (c >= 67828 && c <= 67829)))
                : (c <= 67867 || (c < 127183
                  ? (c < 126980
                    ? (c >= 67872 && c <= 67883)
                    : c <= 126980)
                  : (c <= 127183 || (c >= 127344 && c <= 127345)))))))
            : (c <= 127359 || (c < 127780
              ? (c < 127514
                ? (c < 127462
                  ? (c < 127377
                    ? c == 127374
                    : c <= 127386)
                  : (c <= 127487 || (c >= 127489 && c <= 127490)))
                : (c <= 127514 || (c < 127568
                  ? (c < 127538
                    ? c == 127535
                    : c <= 127546)
                  : (c <= 127569 || (c >= 127744 && c <= 127777)))))
              : (c <= 127891 || (c < 127991
                ? (c < 127902
                  ? (c < 127897
                    ? (c >= 127894 && c <= 127895)
                    : c <= 127899)
                  : (c <= 127984 || (c >= 127987 && c <= 127989)))
                : (c <= 128253 || (c < 128336
                  ? (c < 128329
                    ? (c >= 128255 && c <= 128317)
                    : c <= 128334)
                  : (c <= 128359 || (c >= 128367 && c <= 128368)))))))))
          : (c <= 128378 || (c < 128725
            ? (c < 128465
              ? (c < 128420
                ? (c < 128400
                  ? (c < 128394
                    ? c == 128391
                    : c <= 128397)
                  : (c <= 128400 || (c >= 128405 && c <= 128406)))
                : (c <= 128421 || (c < 128444
                  ? (c < 128433
                    ? c == 128424
                    : c <= 128434)
                  : (c <= 128444 || (c >= 128450 && c <= 128452)))))
              : (c <= 128467 || (c < 128495
                ? (c < 128483
                  ? (c < 128481
                    ? (c >= 128476 && c <= 128478)
                    : c <= 128481)
                  : (c <= 128483 || c == 128488))
                : (c <= 128495 || (c < 128640
                  ? (c < 128506
                    ? c == 128499
                    : c <= 128591)
                  : (c <= 128709 || (c >= 128715 && c <= 128722)))))))
            : (c <= 128727 || (c < 129351
              ? (c < 128755
                ? (c < 128747
                  ? (c < 128745
                    ? (c >= 128733 && c <= 128741)
                    : c <= 128745)
                  : (c <= 128748 || c == 128752))
                : (c <= 128764 || (c < 129292
                  ? (c < 129008
                    ? (c >= 128992 && c <= 129003)
                    : c <= 129008)
                  : (c <= 129338 || (c >= 129340 && c <= 129349)))))
              : (c <= 129535 || (c < 129712
                ? (c < 129664
                  ? (c < 129656
                    ? (c >= 129648 && c <= 129652)
                    : c <= 129660)
                  : (c <= 129670 || (c >= 129680 && c <= 129708)))
                : (c <= 129722 || (c < 129760
                  ? (c < 129744
                    ? (c >= 129728 && c <= 129733)
                    : c <= 129753)
                  : (c <= 129767 || (c >= 129776 && c <= 129782)))))))))))))))));
}

static inline bool sym__normal_bare_identifier_character_set_4(int32_t c) {
  return (c < 8488
    ? (c < 3274
      ? (c < 2575
        ? (c < 1519
          ? (c < 768
            ? (c < 181
              ? (c < '|'
                ? (c < '?'
                  ? (c < '.'
                    ? (c >= '#' && c <= '&')
                    : c <= ':')
                  : (c <= 'Z' || (c < 'a'
                    ? (c >= '^' && c <= '_')
                    : c <= 'z')))
                : (c <= '|' || (c < 174
                  ? (c < 169
                    ? c == '~'
                    : c <= 170)
                  : (c <= 174 || (c >= 178 && c <= 179)))))
              : (c <= 181 || (c < 248
                ? (c < 192
                  ? (c < 188
                    ? (c >= 185 && c <= 186)
                    : c <= 190)
                  : (c <= 214 || (c >= 216 && c <= 246)))
                : (c <= 705 || (c < 748
                  ? (c < 736
                    ? (c >= 710 && c <= 721)
                    : c <= 740)
                  : (c <= 748 || c == 750))))))
            : (c <= 884 || (c < 1155
              ? (c < 904
                ? (c < 895
                  ? (c < 890
                    ? (c >= 886 && c <= 887)
                    : c <= 893)
                  : (c <= 895 || c == 902))
                : (c <= 906 || (c < 931
                  ? (c < 910
                    ? c == 908
                    : c <= 929)
                  : (c <= 1013 || (c >= 1015 && c <= 1153)))))
              : (c <= 1327 || (c < 1471
                ? (c < 1376
                  ? (c < 1369
                    ? (c >= 1329 && c <= 1366)
                    : c <= 1369)
                  : (c <= 1416 || (c >= 1425 && c <= 1469)))
                : (c <= 1471 || (c < 1479
                  ? (c < 1476
                    ? (c >= 1473 && c <= 1474)
                    : c <= 1477)
                  : (c <= 1479 || (c >= 1488 && c <= 1514)))))))))
          : (c <= 1522 || (c < 2406
            ? (c < 1984
              ? (c < 1759
                ? (c < 1646
                  ? (c < 1568
                    ? (c >= 1552 && c <= 1562)
                    : c <= 1641)
                  : (c <= 1747 || (c >= 1749 && c <= 1756)))
                : (c <= 1768 || (c < 1808
                  ? (c < 1791
                    ? (c >= 1770 && c <= 1788)
                    : c <= 1791)
                  : (c <= 1866 || (c >= 1869 && c <= 1969)))))
              : (c <= 2037 || (c < 2144
                ? (c < 2048
                  ? (c < 2045
                    ? c == 2042
                    : c <= 2045)
                  : (c <= 2093 || (c >= 2112 && c <= 2139)))
                : (c <= 2154 || (c < 2200
                  ? (c < 2185
                    ? (c >= 2160 && c <= 2183)
                    : c <= 2190)
                  : (c <= 2273 || (c >= 2275 && c <= 2403)))))))
            : (c <= 2415 || (c < 2507
              ? (c < 2474
                ? (c < 2447
                  ? (c < 2437
                    ? (c >= 2417 && c <= 2435)
                    : c <= 2444)
                  : (c <= 2448 || (c >= 2451 && c <= 2472)))
                : (c <= 2480 || (c < 2492
                  ? (c < 2486
                    ? c == 2482
                    : c <= 2489)
                  : (c <= 2500 || (c >= 2503 && c <= 2504)))))
              : (c <= 2510 || (c < 2548
                ? (c < 2527
                  ? (c < 2524
                    ? c == 2519
                    : c <= 2525)
                  : (c <= 2531 || (c >= 2534 && c <= 2545)))
                : (c <= 2553 || (c < 2561
                  ? (c < 2558
                    ? c == 2556
                    : c <= 2558)
                  : (c <= 2563 || (c >= 2565 && c <= 2570)))))))))))
        : (c <= 2576 || (c < 2911
          ? (c < 2741
            ? (c < 2641
              ? (c < 2616
                ? (c < 2610
                  ? (c < 2602
                    ? (c >= 2579 && c <= 2600)
                    : c <= 2608)
                  : (c <= 2611 || (c >= 2613 && c <= 2614)))
                : (c <= 2617 || (c < 2631
                  ? (c < 2622
                    ? c == 2620
                    : c <= 2626)
                  : (c <= 2632 || (c >= 2635 && c <= 2637)))))
              : (c <= 2641 || (c < 2693
                ? (c < 2662
                  ? (c < 2654
                    ? (c >= 2649 && c <= 2652)
                    : c <= 2654)
                  : (c <= 2677 || (c >= 2689 && c <= 2691)))
                : (c <= 2701 || (c < 2730
                  ? (c < 2707
                    ? (c >= 2703 && c <= 2705)
                    : c <= 2728)
                  : (c <= 2736 || (c >= 2738 && c <= 2739)))))))
            : (c <= 2745 || (c < 2831
              ? (c < 2784
                ? (c < 2763
                  ? (c < 2759
                    ? (c >= 2748 && c <= 2757)
                    : c <= 2761)
                  : (c <= 2765 || c == 2768))
                : (c <= 2787 || (c < 2817
                  ? (c < 2809
                    ? (c >= 2790 && c <= 2799)
                    : c <= 2815)
                  : (c <= 2819 || (c >= 2821 && c <= 2828)))))
              : (c <= 2832 || (c < 2876
                ? (c < 2866
                  ? (c < 2858
                    ? (c >= 2835 && c <= 2856)
                    : c <= 2864)
                  : (c <= 2867 || (c >= 2869 && c <= 2873)))
                : (c <= 2884 || (c < 2901
                  ? (c < 2891
                    ? (c >= 2887 && c <= 2888)
                    : c <= 2893)
                  : (c <= 2903 || (c >= 2908 && c <= 2909)))))))))
          : (c <= 2915 || (c < 3086
            ? (c < 2979
              ? (c < 2958
                ? (c < 2946
                  ? (c < 2929
                    ? (c >= 2918 && c <= 2927)
                    : c <= 2935)
                  : (c <= 2947 || (c >= 2949 && c <= 2954)))
                : (c <= 2960 || (c < 2972
                  ? (c < 2969
                    ? (c >= 2962 && c <= 2965)
                    : c <= 2970)
                  : (c <= 2972 || (c >= 2974 && c <= 2975)))))
              : (c <= 2980 || (c < 3018
                ? (c < 3006
                  ? (c < 2990
                    ? (c >= 2984 && c <= 2986)
                    : c <= 3001)
                  : (c <= 3010 || (c >= 3014 && c <= 3016)))
                : (c <= 3021 || (c < 3046
                  ? (c < 3031
                    ? c == 3024
                    : c <= 3031)
                  : (c <= 3058 || (c >= 3072 && c <= 3084)))))))
            : (c <= 3088 || (c < 3174
              ? (c < 3146
                ? (c < 3132
                  ? (c < 3114
                    ? (c >= 3090 && c <= 3112)
                    : c <= 3129)
                  : (c <= 3140 || (c >= 3142 && c <= 3144)))
                : (c <= 3149 || (c < 3165
                  ? (c < 3160
                    ? (c >= 3157 && c <= 3158)
                    : c <= 3162)
                  : (c <= 3165 || (c >= 3168 && c <= 3171)))))
              : (c <= 3183 || (c < 3218
                ? (c < 3205
                  ? (c < 3200
                    ? (c >= 3192 && c <= 3198)
                    : c <= 3203)
                  : (c <= 3212 || (c >= 3214 && c <= 3216)))
                : (c <= 3240 || (c < 3260
                  ? (c < 3253
                    ? (c >= 3242 && c <= 3251)
                    : c <= 3257)
                  : (c <= 3268 || (c >= 3270 && c <= 3272)))))))))))))
      : (c <= 3277 || (c < 5761
        ? (c < 3864
          ? (c < 3535
            ? (c < 3412
              ? (c < 3328
                ? (c < 3296
                  ? (c < 3293
                    ? (c >= 3285 && c <= 3286)
                    : c <= 3294)
                  : (c <= 3299 || (c < 3313
                    ? (c >= 3302 && c <= 3311)
                    : c <= 3314)))
                : (c <= 3340 || (c < 3398
                  ? (c < 3346
                    ? (c >= 3342 && c <= 3344)
                    : c <= 3396)
                  : (c <= 3400 || (c >= 3402 && c <= 3406)))))
              : (c <= 3427 || (c < 3482
                ? (c < 3457
                  ? (c < 3450
                    ? (c >= 3430 && c <= 3448)
                    : c <= 3455)
                  : (c <= 3459 || (c >= 3461 && c <= 3478)))
                : (c <= 3505 || (c < 3520
                  ? (c < 3517
                    ? (c >= 3507 && c <= 3515)
                    : c <= 3517)
                  : (c <= 3526 || c == 3530))))))
            : (c <= 3540 || (c < 3718
              ? (c < 3585
                ? (c < 3558
                  ? (c < 3544
                    ? c == 3542
                    : c <= 3551)
                  : (c <= 3567 || (c >= 3570 && c <= 3571)))
                : (c <= 3642 || (c < 3713
                  ? (c < 3664
                    ? (c >= 3648 && c <= 3662)
                    : c <= 3673)
                  : (c <= 3714 || c == 3716))))
              : (c <= 3722 || (c < 3782
                ? (c < 3751
                  ? (c < 3749
                    ? (c >= 3724 && c <= 3747)
                    : c <= 3749)
                  : (c <= 3773 || (c >= 3776 && c <= 3780)))
                : (c <= 3782 || (c < 3804
                  ? (c < 3792
                    ? (c >= 3784 && c <= 3789)
                    : c <= 3801)
                  : (c <= 3807 || c == 3840))))))))
          : (c <= 3865 || (c < 4696
            ? (c < 4038
              ? (c < 3902
                ? (c < 3895
                  ? (c < 3893
                    ? (c >= 3872 && c <= 3891)
                    : c <= 3893)
                  : (c <= 3895 || c == 3897))
                : (c <= 3911 || (c < 3974
                  ? (c < 3953
                    ? (c >= 3913 && c <= 3948)
                    : c <= 3972)
                  : (c <= 3991 || (c >= 3993 && c <= 4028)))))
              : (c <= 4038 || (c < 4301
                ? (c < 4256
                  ? (c < 4176
                    ? (c >= 4096 && c <= 4169)
                    : c <= 4253)
                  : (c <= 4293 || c == 4295))
                : (c <= 4301 || (c < 4682
                  ? (c < 4348
                    ? (c >= 4304 && c <= 4346)
                    : c <= 4680)
                  : (c <= 4685 || (c >= 4688 && c <= 4694)))))))
            : (c <= 4696 || (c < 4824
              ? (c < 4786
                ? (c < 4746
                  ? (c < 4704
                    ? (c >= 4698 && c <= 4701)
                    : c <= 4744)
                  : (c <= 4749 || (c >= 4752 && c <= 4784)))
                : (c <= 4789 || (c < 4802
                  ? (c < 4800
                    ? (c >= 4792 && c <= 4798)
                    : c <= 4800)
                  : (c <= 4805 || (c >= 4808 && c <= 4822)))))
              : (c <= 4880 || (c < 4992
                ? (c < 4957
                  ? (c < 4888
                    ? (c >= 4882 && c <= 4885)
                    : c <= 4954)
                  : (c <= 4959 || (c >= 4969 && c <= 4988)))
                : (c <= 5007 || (c < 5121
                  ? (c < 5112
                    ? (c >= 5024 && c <= 5109)
                    : c <= 5117)
                  : (c <= 5740 || (c >= 5743 && c <= 5759)))))))))))
        : (c <= 5786 || (c < 7245
          ? (c < 6432
            ? (c < 6103
              ? (c < 5952
                ? (c < 5888
                  ? (c < 5870
                    ? (c >= 5792 && c <= 5866)
                    : c <= 5880)
                  : (c <= 5909 || (c >= 5919 && c <= 5940)))
                : (c <= 5971 || (c < 6002
                  ? (c < 5998
                    ? (c >= 5984 && c <= 5996)
                    : c <= 6000)
                  : (c <= 6003 || (c >= 6016 && c <= 6099)))))
              : (c <= 6103 || (c < 6159
                ? (c < 6128
                  ? (c < 6112
                    ? (c >= 6108 && c <= 6109)
                    : c <= 6121)
                  : (c <= 6137 || (c >= 6155 && c <= 6157)))
                : (c <= 6169 || (c < 6320
                  ? (c < 6272
                    ? (c >= 6176 && c <= 6264)
                    : c <= 6314)
                  : (c <= 6389 || (c >= 6400 && c <= 6430)))))))
            : (c <= 6443 || (c < 6783
              ? (c < 6576
                ? (c < 6512
                  ? (c < 6470
                    ? (c >= 6448 && c <= 6459)
                    : c <= 6509)
                  : (c <= 6516 || (c >= 6528 && c <= 6571)))
                : (c <= 6601 || (c < 6688
                  ? (c < 6656
                    ? (c >= 6608 && c <= 6618)
                    : c <= 6683)
                  : (c <= 6750 || (c >= 6752 && c <= 6780)))))
              : (c <= 6793 || (c < 6992
                ? (c < 6832
                  ? (c < 6823
                    ? (c >= 6800 && c <= 6809)
                    : c <= 6823)
                  : (c <= 6862 || (c >= 6912 && c <= 6988)))
                : (c <= 7001 || (c < 7168
                  ? (c < 7040
                    ? (c >= 7019 && c <= 7027)
                    : c <= 7155)
                  : (c <= 7223 || (c >= 7232 && c <= 7241)))))))))
          : (c <= 7293 || (c < 8144
            ? (c < 8016
              ? (c < 7380
                ? (c < 7357
                  ? (c < 7312
                    ? (c >= 7296 && c <= 7304)
                    : c <= 7354)
                  : (c <= 7359 || (c >= 7376 && c <= 7378)))
                : (c <= 7418 || (c < 7968
                  ? (c < 7960
                    ? (c >= 7424 && c <= 7957)
                    : c <= 7965)
                  : (c <= 8005 || (c >= 8008 && c <= 8013)))))
              : (c <= 8023 || (c < 8064
                ? (c < 8029
                  ? (c < 8027
                    ? c == 8025
                    : c <= 8027)
                  : (c <= 8029 || (c >= 8031 && c <= 8061)))
                : (c <= 8116 || (c < 8130
                  ? (c < 8126
                    ? (c >= 8118 && c <= 8124)
                    : c <= 8126)
                  : (c <= 8132 || (c >= 8134 && c <= 8140)))))))
            : (c <= 8147 || (c < 8336
              ? (c < 8252
                ? (c < 8178
                  ? (c < 8160
                    ? (c >= 8150 && c <= 8155)
                    : c <= 8172)
                  : (c <= 8180 || (c >= 8182 && c <= 8188)))
                : (c <= 8252 || (c < 8308
                  ? (c < 8304
                    ? c == 8265
                    : c <= 8305)
                  : (c <= 8313 || (c >= 8319 && c <= 8329)))))
              : (c <= 8348 || (c < 8469
                ? (c < 8455
                  ? (c < 8450
                    ? (c >= 8400 && c <= 8432)
                    : c <= 8450)
                  : (c <= 8455 || (c >= 8458 && c <= 8467)))
                : (c <= 8469 || (c < 8484
                  ? (c < 8482
                    ? (c >= 8473 && c <= 8477)
                    : c <= 8482)
                  : (c <= 8484 || c == 8486))))))))))))))
    : (c <= 8488 || (c < 43744
      ? (c < 10175
        ? (c < 9854
          ? (c < 9728
            ? (c < 9167
              ? (c < 8528
                ? (c < 8508
                  ? (c < 8495
                    ? (c >= 8490 && c <= 8493)
                    : c <= 8505)
                  : (c <= 8511 || (c < 8526
                    ? (c >= 8517 && c <= 8521)
                    : c <= 8526)))
                : (c <= 8585 || (c < 8986
                  ? (c < 8617
                    ? (c >= 8596 && c <= 8601)
                    : c <= 8618)
                  : (c <= 8987 || c == 9000))))
              : (c <= 9167 || (c < 9450
                ? (c < 9312
                  ? (c < 9208
                    ? (c >= 9193 && c <= 9203)
                    : c <= 9210)
                  : (c <= 9371 || c == 9410))
                : (c <= 9471 || (c < 9664
                  ? (c < 9654
                    ? (c >= 9642 && c <= 9643)
                    : c <= 9654)
                  : (c <= 9664 || (c >= 9723 && c <= 9726)))))))
            : (c <= 9732 || (c < 9774
              ? (c < 9757
                ? (c < 9748
                  ? (c < 9745
                    ? c == 9742
                    : c <= 9745)
                  : (c <= 9749 || c == 9752))
                : (c <= 9757 || (c < 9766
                  ? (c < 9762
                    ? c == 9760
                    : c <= 9763)
                  : (c <= 9766 || c == 9770))))
              : (c <= 9775 || (c < 9823
                ? (c < 9794
                  ? (c < 9792
                    ? (c >= 9784 && c <= 9786)
                    : c <= 9792)
                  : (c <= 9794 || (c >= 9800 && c <= 9811)))
                : (c <= 9824 || (c < 9832
                  ? (c < 9829
                    ? c == 9827
                    : c <= 9830)
                  : (c <= 9832 || c == 9851))))))))
          : (c <= 9855 || (c < 9992
            ? (c < 9928
              ? (c < 9895
                ? (c < 9883
                  ? (c < 9881
                    ? (c >= 9874 && c <= 9879)
                    : c <= 9881)
                  : (c <= 9884 || (c >= 9888 && c <= 9889)))
                : (c <= 9895 || (c < 9917
                  ? (c < 9904
                    ? (c >= 9898 && c <= 9899)
                    : c <= 9905)
                  : (c <= 9918 || (c >= 9924 && c <= 9925)))))
              : (c <= 9928 || (c < 9968
                ? (c < 9939
                  ? (c < 9937
                    ? (c >= 9934 && c <= 9935)
                    : c <= 9937)
                  : (c <= 9940 || (c >= 9961 && c <= 9962)))
                : (c <= 9973 || (c < 9986
                  ? (c < 9981
                    ? (c >= 9975 && c <= 9978)
                    : c <= 9981)
                  : (c <= 9986 || c == 9989))))))
            : (c <= 9997 || (c < 10055
              ? (c < 10013
                ? (c < 10004
                  ? (c < 10002
                    ? c == 9999
                    : c <= 10002)
                  : (c <= 10004 || c == 10006))
                : (c <= 10013 || (c < 10035
                  ? (c < 10024
                    ? c == 10017
                    : c <= 10024)
                  : (c <= 10036 || c == 10052))))
              : (c <= 10055 || (c < 10083
                ? (c < 10067
                  ? (c < 10062
                    ? c == 10060
                    : c <= 10062)
                  : (c <= 10069 || c == 10071))
                : (c <= 10084 || (c < 10145
                  ? (c < 10133
                    ? (c >= 10102 && c <= 10131)
                    : c <= 10135)
                  : (c <= 10145 || c == 10160))))))))))
        : (c <= 10175 || (c < 12881
          ? (c < 11720
            ? (c < 11559
              ? (c < 11093
                ? (c < 11035
                  ? (c < 11013
                    ? (c >= 10548 && c <= 10549)
                    : c <= 11015)
                  : (c <= 11036 || c == 11088))
                : (c <= 11093 || (c < 11517
                  ? (c < 11499
                    ? (c >= 11264 && c <= 11492)
                    : c <= 11507)
                  : (c <= 11517 || (c >= 11520 && c <= 11557)))))
              : (c <= 11559 || (c < 11680
                ? (c < 11631
                  ? (c < 11568
                    ? c == 11565
                    : c <= 11623)
                  : (c <= 11631 || (c >= 11647 && c <= 11670)))
                : (c <= 11686 || (c < 11704
                  ? (c < 11696
                    ? (c >= 11688 && c <= 11694)
                    : c <= 11702)
                  : (c <= 11710 || (c >= 11712 && c <= 11718)))))))
            : (c <= 11726 || (c < 12445
              ? (c < 12293
                ? (c < 11744
                  ? (c < 11736
                    ? (c >= 11728 && c <= 11734)
                    : c <= 11742)
                  : (c <= 11775 || c == 11823))
                : (c <= 12295 || (c < 12353
                  ? (c < 12344
                    ? (c >= 12321 && c <= 12341)
                    : c <= 12349)
                  : (c <= 12438 || (c >= 12441 && c <= 12442)))))
              : (c <= 12447 || (c < 12690
                ? (c < 12549
                  ? (c < 12540
                    ? (c >= 12449 && c <= 12538)
                    : c <= 12543)
                  : (c <= 12591 || (c >= 12593 && c <= 12686)))
                : (c <= 12693 || (c < 12832
                  ? (c < 12784
                    ? (c >= 12704 && c <= 12735)
                    : c <= 12799)
                  : (c <= 12841 || (c >= 12872 && c <= 12879)))))))))
          : (c <= 12895 || (c < 42994
            ? (c < 42512
              ? (c < 13312
                ? (c < 12953
                  ? (c < 12951
                    ? (c >= 12928 && c <= 12937)
                    : c <= 12951)
                  : (c <= 12953 || (c >= 12977 && c <= 12991)))
                : (c <= 13312 || (c < 42192
                  ? (c < 19968
                    ? c == 19903
                    : c <= 42124)
                  : (c <= 42237 || (c >= 42240 && c <= 42508)))))
              : (c <= 42539 || (c < 42786
                ? (c < 42623
                  ? (c < 42612
                    ? (c >= 42560 && c <= 42610)
                    : c <= 42621)
                  : (c <= 42737 || (c >= 42775 && c <= 42783)))
                : (c <= 42888 || (c < 42963
                  ? (c < 42960
                    ? (c >= 42891 && c <= 42954)
                    : c <= 42961)
                  : (c <= 42963 || (c >= 42965 && c <= 42969)))))))
            : (c <= 43047 || (c < 43360
              ? (c < 43216
                ? (c < 43072
                  ? (c < 43056
                    ? c == 43052
                    : c <= 43061)
                  : (c <= 43123 || (c >= 43136 && c <= 43205)))
                : (c <= 43225 || (c < 43261
                  ? (c < 43259
                    ? (c >= 43232 && c <= 43255)
                    : c <= 43259)
                  : (c <= 43309 || (c >= 43312 && c <= 43347)))))
              : (c <= 43388 || (c < 43584
                ? (c < 43488
                  ? (c < 43471
                    ? (c >= 43392 && c <= 43456)
                    : c <= 43481)
                  : (c <= 43518 || (c >= 43520 && c <= 43574)))
                : (c <= 43597 || (c < 43642
                  ? (c < 43616
                    ? (c >= 43600 && c <= 43609)
                    : c <= 43638)
                  : (c <= 43714 || (c >= 43739 && c <= 43741)))))))))))))
      : (c <= 43759 || (c < 67424
        ? (c < 65482
          ? (c < 64285
            ? (c < 44012
              ? (c < 43808
                ? (c < 43785
                  ? (c < 43777
                    ? (c >= 43762 && c <= 43766)
                    : c <= 43782)
                  : (c <= 43790 || (c >= 43793 && c <= 43798)))
                : (c <= 43814 || (c < 43868
                  ? (c < 43824
                    ? (c >= 43816 && c <= 43822)
                    : c <= 43866)
                  : (c <= 43881 || (c >= 43888 && c <= 44010)))))
              : (c <= 44013 || (c < 55243
                ? (c < 55203
                  ? (c < 44032
                    ? (c >= 44016 && c <= 44025)
                    : c <= 44032)
                  : (c <= 55203 || (c >= 55216 && c <= 55238)))
                : (c <= 55291 || (c < 64256
                  ? (c < 64112
                    ? (c >= 63744 && c <= 64109)
                    : c <= 64217)
                  : (c <= 64262 || (c >= 64275 && c <= 64279)))))))
            : (c <= 64296 || (c < 65008
              ? (c < 64323
                ? (c < 64318
                  ? (c < 64312
                    ? (c >= 64298 && c <= 64310)
                    : c <= 64316)
                  : (c <= 64318 || (c >= 64320 && c <= 64321)))
                : (c <= 64324 || (c < 64848
                  ? (c < 64467
                    ? (c >= 64326 && c <= 64433)
                    : c <= 64829)
                  : (c <= 64911 || (c >= 64914 && c <= 64967)))))
              : (c <= 65019 || (c < 65296
                ? (c < 65136
                  ? (c < 65056
                    ? (c >= 65024 && c <= 65039)
                    : c <= 65071)
                  : (c <= 65140 || (c >= 65142 && c <= 65276)))
                : (c <= 65305 || (c < 65382
                  ? (c < 65345
                    ? (c >= 65313 && c <= 65338)
                    : c <= 65370)
                  : (c <= 65470 || (c >= 65474 && c <= 65479)))))))))
          : (c <= 65487 || (c < 66432
            ? (c < 65799
              ? (c < 65576
                ? (c < 65536
                  ? (c < 65498
                    ? (c >= 65490 && c <= 65495)
                    : c <= 65500)
                  : (c <= 65547 || (c >= 65549 && c <= 65574)))
                : (c <= 65594 || (c < 65616
                  ? (c < 65599
                    ? (c >= 65596 && c <= 65597)
                    : c <= 65613)
                  : (c <= 65629 || (c >= 65664 && c <= 65786)))))
              : (c <= 65843 || (c < 66208
                ? (c < 66045
                  ? (c < 65930
                    ? (c >= 65856 && c <= 65912)
                    : c <= 65931)
                  : (c <= 66045 || (c >= 66176 && c <= 66204)))
                : (c <= 66256 || (c < 66349
                  ? (c < 66304
                    ? (c >= 66272 && c <= 66299)
                    : c <= 66339)
                  : (c <= 66378 || (c >= 66384 && c <= 66426)))))))
            : (c <= 66461 || (c < 66928
              ? (c < 66720
                ? (c < 66513
                  ? (c < 66504
                    ? (c >= 66464 && c <= 66499)
                    : c <= 66511)
                  : (c <= 66517 || (c >= 66560 && c <= 66717)))
                : (c <= 66729 || (c < 66816
                  ? (c < 66776
                    ? (c >= 66736 && c <= 66771)
                    : c <= 66811)
                  : (c <= 66855 || (c >= 66864 && c <= 66915)))))
              : (c <= 66938 || (c < 66979
                ? (c < 66964
                  ? (c < 66956
                    ? (c >= 66940 && c <= 66954)
                    : c <= 66962)
                  : (c <= 66965 || (c >= 66967 && c <= 66977)))
                : (c <= 66993 || (c < 67072
                  ? (c < 67003
                    ? (c >= 66995 && c <= 67001)
                    : c <= 67004)
                  : (c <= 67382 || (c >= 67392 && c <= 67413)))))))))))
        : (c <= 67431 || (c < 128371
          ? (c < 127358
            ? (c < 67672
              ? (c < 67592
                ? (c < 67506
                  ? (c < 67463
                    ? (c >= 67456 && c <= 67461)
                    : c <= 67504)
                  : (c <= 67514 || (c >= 67584 && c <= 67589)))
                : (c <= 67592 || (c < 67644
                  ? (c < 67639
                    ? (c >= 67594 && c <= 67637)
                    : c <= 67640)
                  : (c <= 67644 || (c >= 67647 && c <= 67669)))))
              : (c <= 67702 || (c < 67835
                ? (c < 67808
                  ? (c < 67751
                    ? (c >= 67705 && c <= 67742)
                    : c <= 67759)
                  : (c <= 67826 || (c >= 67828 && c <= 67829)))
                : (c <= 67867 || (c < 127183
                  ? (c < 126980
                    ? (c >= 67872 && c <= 67883)
                    : c <= 126980)
                  : (c <= 127183 || (c >= 127344 && c <= 127345)))))))
            : (c <= 127359 || (c < 127780
              ? (c < 127514
                ? (c < 127462
                  ? (c < 127377
                    ? c == 127374
                    : c <= 127386)
                  : (c <= 127487 || (c >= 127489 && c <= 127490)))
                : (c <= 127514 || (c < 127568
                  ? (c < 127538
                    ? c == 127535
                    : c <= 127546)
                  : (c <= 127569 || (c >= 127744 && c <= 127777)))))
              : (c <= 127891 || (c < 127991
                ? (c < 127902
                  ? (c < 127897
                    ? (c >= 127894 && c <= 127895)
                    : c <= 127899)
                  : (c <= 127984 || (c >= 127987 && c <= 127989)))
                : (c <= 128253 || (c < 128336
                  ? (c < 128329
                    ? (c >= 128255 && c <= 128317)
                    : c <= 128334)
                  : (c <= 128359 || (c >= 128367 && c <= 128368)))))))))
          : (c <= 128378 || (c < 128725
            ? (c < 128465
              ? (c < 128420
                ? (c < 128400
                  ? (c < 128394
                    ? c == 128391
                    : c <= 128397)
                  : (c <= 128400 || (c >= 128405 && c <= 128406)))
                : (c <= 128421 || (c < 128444
                  ? (c < 128433
                    ? c == 128424
                    : c <= 128434)
                  : (c <= 128444 || (c >= 128450 && c <= 128452)))))
              : (c <= 128467 || (c < 128495
                ? (c < 128483
                  ? (c < 128481
                    ? (c >= 128476 && c <= 128478)
                    : c <= 128481)
                  : (c <= 128483 || c == 128488))
                : (c <= 128495 || (c < 128640
                  ? (c < 128506
                    ? c == 128499
                    : c <= 128591)
                  : (c <= 128709 || (c >= 128715 && c <= 128722)))))))
            : (c <= 128727 || (c < 129351
              ? (c < 128755
                ? (c < 128747
                  ? (c < 128745
                    ? (c >= 128733 && c <= 128741)
                    : c <= 128745)
                  : (c <= 128748 || c == 128752))
                : (c <= 128764 || (c < 129292
                  ? (c < 129008
                    ? (c >= 128992 && c <= 129003)
                    : c <= 129008)
                  : (c <= 129338 || (c >= 129340 && c <= 129349)))))
              : (c <= 129535 || (c < 129712
                ? (c < 129664
                  ? (c < 129656
                    ? (c >= 129648 && c <= 129652)
                    : c <= 129660)
                  : (c <= 129670 || (c >= 129680 && c <= 129708)))
                : (c <= 129722 || (c < 129760
                  ? (c < 129744
                    ? (c >= 129728 && c <= 129733)
                    : c <= 129753)
                  : (c <= 129767 || (c >= 129776 && c <= 129782)))))))))))))))));
}

static inline bool sym__normal_bare_identifier_character_set_5(int32_t c) {
  return (c < 8486
    ? (c < 3270
      ? (c < 2561
        ? (c < 1479
          ? (c < 748
            ? (c < 174
              ? (c < '^'
                ? (c < '-'
                  ? (c < '*'
                    ? (c >= '!' && c <= '\'')
                    : c <= '+')
                  : (c <= '.' || (c < '?'
                    ? (c >= '0' && c <= ':')
                    : c <= 'Z')))
                : (c <= '_' || (c < '~'
                  ? (c < '|'
                    ? (c >= 'a' && c <= 'z')
                    : c <= '|')
                  : (c <= '~' || (c >= 169 && c <= 170)))))
              : (c <= 174 || (c < 192
                ? (c < 185
                  ? (c < 181
                    ? (c >= 178 && c <= 179)
                    : c <= 181)
                  : (c <= 186 || (c >= 188 && c <= 190)))
                : (c <= 214 || (c < 710
                  ? (c < 248
                    ? (c >= 216 && c <= 246)
                    : c <= 705)
                  : (c <= 721 || (c >= 736 && c <= 740)))))))
            : (c <= 748 || (c < 931
              ? (c < 895
                ? (c < 886
                  ? (c < 768
                    ? c == 750
                    : c <= 884)
                  : (c <= 887 || (c >= 890 && c <= 893)))
                : (c <= 895 || (c < 908
                  ? (c < 904
                    ? c == 902
                    : c <= 906)
                  : (c <= 908 || (c >= 910 && c <= 929)))))
              : (c <= 1013 || (c < 1376
                ? (c < 1329
                  ? (c < 1155
                    ? (c >= 1015 && c <= 1153)
                    : c <= 1327)
                  : (c <= 1366 || c == 1369))
                : (c <= 1416 || (c < 1473
                  ? (c < 1471
                    ? (c >= 1425 && c <= 1469)
                    : c <= 1471)
                  : (c <= 1474 || (c >= 1476 && c <= 1477)))))))))
          : (c <= 1479 || (c < 2200
            ? (c < 1808
              ? (c < 1646
                ? (c < 1552
                  ? (c < 1519
                    ? (c >= 1488 && c <= 1514)
                    : c <= 1522)
                  : (c <= 1562 || (c >= 1568 && c <= 1641)))
                : (c <= 1747 || (c < 1770
                  ? (c < 1759
                    ? (c >= 1749 && c <= 1756)
                    : c <= 1768)
                  : (c <= 1788 || c == 1791))))
              : (c <= 1866 || (c < 2048
                ? (c < 2042
                  ? (c < 1984
                    ? (c >= 1869 && c <= 1969)
                    : c <= 2037)
                  : (c <= 2042 || c == 2045))
                : (c <= 2093 || (c < 2160
                  ? (c < 2144
                    ? (c >= 2112 && c <= 2139)
                    : c <= 2154)
                  : (c <= 2183 || (c >= 2185 && c <= 2190)))))))
            : (c <= 2273 || (c < 2492
              ? (c < 2447
                ? (c < 2417
                  ? (c < 2406
                    ? (c >= 2275 && c <= 2403)
                    : c <= 2415)
                  : (c <= 2435 || (c >= 2437 && c <= 2444)))
                : (c <= 2448 || (c < 2482
                  ? (c < 2474
                    ? (c >= 2451 && c <= 2472)
                    : c <= 2480)
                  : (c <= 2482 || (c >= 2486 && c <= 2489)))))
              : (c <= 2500 || (c < 2527
                ? (c < 2519
                  ? (c < 2507
                    ? (c >= 2503 && c <= 2504)
                    : c <= 2510)
                  : (c <= 2519 || (c >= 2524 && c <= 2525)))
                : (c <= 2531 || (c < 2556
                  ? (c < 2548
                    ? (c >= 2534 && c <= 2545)
                    : c <= 2553)
                  : (c <= 2556 || c == 2558))))))))))
        : (c <= 2563 || (c < 2908
          ? (c < 2738
            ? (c < 2635
              ? (c < 2613
                ? (c < 2579
                  ? (c < 2575
                    ? (c >= 2565 && c <= 2570)
                    : c <= 2576)
                  : (c <= 2600 || (c < 2610
                    ? (c >= 2602 && c <= 2608)
                    : c <= 2611)))
                : (c <= 2614 || (c < 2622
                  ? (c < 2620
                    ? (c >= 2616 && c <= 2617)
                    : c <= 2620)
                  : (c <= 2626 || (c >= 2631 && c <= 2632)))))
              : (c <= 2637 || (c < 2689
                ? (c < 2654
                  ? (c < 2649
                    ? c == 2641
                    : c <= 2652)
                  : (c <= 2654 || (c >= 2662 && c <= 2677)))
                : (c <= 2691 || (c < 2707
                  ? (c < 2703
                    ? (c >= 2693 && c <= 2701)
                    : c <= 2705)
                  : (c <= 2728 || (c >= 2730 && c <= 2736)))))))
            : (c <= 2739 || (c < 2821
              ? (c < 2768
                ? (c < 2759
                  ? (c < 2748
                    ? (c >= 2741 && c <= 2745)
                    : c <= 2757)
                  : (c <= 2761 || (c >= 2763 && c <= 2765)))
                : (c <= 2768 || (c < 2809
                  ? (c < 2790
                    ? (c >= 2784 && c <= 2787)
                    : c <= 2799)
                  : (c <= 2815 || (c >= 2817 && c <= 2819)))))
              : (c <= 2828 || (c < 2869
                ? (c < 2858
                  ? (c < 2835
                    ? (c >= 2831 && c <= 2832)
                    : c <= 2856)
                  : (c <= 2864 || (c >= 2866 && c <= 2867)))
                : (c <= 2873 || (c < 2891
                  ? (c < 2887
                    ? (c >= 2876 && c <= 2884)
                    : c <= 2888)
                  : (c <= 2893 || (c >= 2901 && c <= 2903)))))))))
          : (c <= 2909 || (c < 3072
            ? (c < 2974
              ? (c < 2949
                ? (c < 2929
                  ? (c < 2918
                    ? (c >= 2911 && c <= 2915)
                    : c <= 2927)
                  : (c <= 2935 || (c >= 2946 && c <= 2947)))
                : (c <= 2954 || (c < 2969
                  ? (c < 2962
                    ? (c >= 2958 && c <= 2960)
                    : c <= 2965)
                  : (c <= 2970 || c == 2972))))
              : (c <= 2975 || (c < 3014
                ? (c < 2990
                  ? (c < 2984
                    ? (c >= 2979 && c <= 2980)
                    : c <= 2986)
                  : (c <= 3001 || (c >= 3006 && c <= 3010)))
                : (c <= 3016 || (c < 3031
                  ? (c < 3024
                    ? (c >= 3018 && c <= 3021)
                    : c <= 3024)
                  : (c <= 3031 || (c >= 3046 && c <= 3058)))))))
            : (c <= 3084 || (c < 3168
              ? (c < 3142
                ? (c < 3114
                  ? (c < 3090
                    ? (c >= 3086 && c <= 3088)
                    : c <= 3112)
                  : (c <= 3129 || (c >= 3132 && c <= 3140)))
                : (c <= 3144 || (c < 3160
                  ? (c < 3157
                    ? (c >= 3146 && c <= 3149)
                    : c <= 3158)
                  : (c <= 3162 || c == 3165))))
              : (c <= 3171 || (c < 3214
                ? (c < 3200
                  ? (c < 3192
                    ? (c >= 3174 && c <= 3183)
                    : c <= 3198)
                  : (c <= 3203 || (c >= 3205 && c <= 3212)))
                : (c <= 3216 || (c < 3253
                  ? (c < 3242
                    ? (c >= 3218 && c <= 3240)
                    : c <= 3251)
                  : (c <= 3257 || (c >= 3260 && c <= 3268)))))))))))))
      : (c <= 3272 || (c < 5743
        ? (c < 3840
          ? (c < 3530
            ? (c < 3402
              ? (c < 3313
                ? (c < 3293
                  ? (c < 3285
                    ? (c >= 3274 && c <= 3277)
                    : c <= 3286)
                  : (c <= 3294 || (c < 3302
                    ? (c >= 3296 && c <= 3299)
                    : c <= 3311)))
                : (c <= 3314 || (c < 3346
                  ? (c < 3342
                    ? (c >= 3328 && c <= 3340)
                    : c <= 3344)
                  : (c <= 3396 || (c >= 3398 && c <= 3400)))))
              : (c <= 3406 || (c < 3461
                ? (c < 3450
                  ? (c < 3430
                    ? (c >= 3412 && c <= 3427)
                    : c <= 3448)
                  : (c <= 3455 || (c >= 3457 && c <= 3459)))
                : (c <= 3478 || (c < 3517
                  ? (c < 3507
                    ? (c >= 3482 && c <= 3505)
                    : c <= 3515)
                  : (c <= 3517 || (c >= 3520 && c <= 3526)))))))
            : (c <= 3530 || (c < 3716
              ? (c < 3570
                ? (c < 3544
                  ? (c < 3542
                    ? (c >= 3535 && c <= 3540)
                    : c <= 3542)
                  : (c <= 3551 || (c >= 3558 && c <= 3567)))
                : (c <= 3571 || (c < 3664
                  ? (c < 3648
                    ? (c >= 3585 && c <= 3642)
                    : c <= 3662)
                  : (c <= 3673 || (c >= 3713 && c <= 3714)))))
              : (c <= 3716 || (c < 3776
                ? (c < 3749
                  ? (c < 3724
                    ? (c >= 3718 && c <= 3722)
                    : c <= 3747)
                  : (c <= 3749 || (c >= 3751 && c <= 3773)))
                : (c <= 3780 || (c < 3792
                  ? (c < 3784
                    ? c == 3782
                    : c <= 3789)
                  : (c <= 3801 || (c >= 3804 && c <= 3807)))))))))
          : (c <= 3840 || (c < 4688
            ? (c < 3993
              ? (c < 3897
                ? (c < 3893
                  ? (c < 3872
                    ? (c >= 3864 && c <= 3865)
                    : c <= 3891)
                  : (c <= 3893 || c == 3895))
                : (c <= 3897 || (c < 3953
                  ? (c < 3913
                    ? (c >= 3902 && c <= 3911)
                    : c <= 3948)
                  : (c <= 3972 || (c >= 3974 && c <= 3991)))))
              : (c <= 4028 || (c < 4295
                ? (c < 4176
                  ? (c < 4096
                    ? c == 4038
                    : c <= 4169)
                  : (c <= 4253 || (c >= 4256 && c <= 4293)))
                : (c <= 4295 || (c < 4348
                  ? (c < 4304
                    ? c == 4301
                    : c <= 4346)
                  : (c <= 4680 || (c >= 4682 && c <= 4685)))))))
            : (c <= 4694 || (c < 4808
              ? (c < 4752
                ? (c < 4704
                  ? (c < 4698
                    ? c == 4696
                    : c <= 4701)
                  : (c <= 4744 || (c >= 4746 && c <= 4749)))
                : (c <= 4784 || (c < 4800
                  ? (c < 4792
                    ? (c >= 4786 && c <= 4789)
                    : c <= 4798)
                  : (c <= 4800 || (c >= 4802 && c <= 4805)))))
              : (c <= 4822 || (c < 4969
                ? (c < 4888
                  ? (c < 4882
                    ? (c >= 4824 && c <= 4880)
                    : c <= 4885)
                  : (c <= 4954 || (c >= 4957 && c <= 4959)))
                : (c <= 4988 || (c < 5112
                  ? (c < 5024
                    ? (c >= 4992 && c <= 5007)
                    : c <= 5109)
                  : (c <= 5117 || (c >= 5121 && c <= 5740)))))))))))
        : (c <= 5759 || (c < 7232
          ? (c < 6400
            ? (c < 6016
              ? (c < 5919
                ? (c < 5870
                  ? (c < 5792
                    ? (c >= 5761 && c <= 5786)
                    : c <= 5866)
                  : (c <= 5880 || (c >= 5888 && c <= 5909)))
                : (c <= 5940 || (c < 5998
                  ? (c < 5984
                    ? (c >= 5952 && c <= 5971)
                    : c <= 5996)
                  : (c <= 6000 || (c >= 6002 && c <= 6003)))))
              : (c <= 6099 || (c < 6155
                ? (c < 6112
                  ? (c < 6108
                    ? c == 6103
                    : c <= 6109)
                  : (c <= 6121 || (c >= 6128 && c <= 6137)))
                : (c <= 6157 || (c < 6272
                  ? (c < 6176
                    ? (c >= 6159 && c <= 6169)
                    : c <= 6264)
                  : (c <= 6314 || (c >= 6320 && c <= 6389)))))))
            : (c <= 6430 || (c < 6752
              ? (c < 6528
                ? (c < 6470
                  ? (c < 6448
                    ? (c >= 6432 && c <= 6443)
                    : c <= 6459)
                  : (c <= 6509 || (c >= 6512 && c <= 6516)))
                : (c <= 6571 || (c < 6656
                  ? (c < 6608
                    ? (c >= 6576 && c <= 6601)
                    : c <= 6618)
                  : (c <= 6683 || (c >= 6688 && c <= 6750)))))
              : (c <= 6780 || (c < 6912
                ? (c < 6823
                  ? (c < 6800
                    ? (c >= 6783 && c <= 6793)
                    : c <= 6809)
                  : (c <= 6823 || (c >= 6832 && c <= 6862)))
                : (c <= 6988 || (c < 7040
                  ? (c < 7019
                    ? (c >= 6992 && c <= 7001)
                    : c <= 7027)
                  : (c <= 7155 || (c >= 7168 && c <= 7223)))))))))
          : (c <= 7241 || (c < 8134
            ? (c < 8008
              ? (c < 7376
                ? (c < 7312
                  ? (c < 7296
                    ? (c >= 7245 && c <= 7293)
                    : c <= 7304)
                  : (c <= 7354 || (c >= 7357 && c <= 7359)))
                : (c <= 7378 || (c < 7960
                  ? (c < 7424
                    ? (c >= 7380 && c <= 7418)
                    : c <= 7957)
                  : (c <= 7965 || (c >= 7968 && c <= 8005)))))
              : (c <= 8013 || (c < 8031
                ? (c < 8027
                  ? (c < 8025
                    ? (c >= 8016 && c <= 8023)
                    : c <= 8025)
                  : (c <= 8027 || c == 8029))
                : (c <= 8061 || (c < 8126
                  ? (c < 8118
                    ? (c >= 8064 && c <= 8116)
                    : c <= 8124)
                  : (c <= 8126 || (c >= 8130 && c <= 8132)))))))
            : (c <= 8140 || (c < 8319
              ? (c < 8182
                ? (c < 8160
                  ? (c < 8150
                    ? (c >= 8144 && c <= 8147)
                    : c <= 8155)
                  : (c <= 8172 || (c >= 8178 && c <= 8180)))
                : (c <= 8188 || (c < 8304
                  ? (c < 8265
                    ? c == 8252
                    : c <= 8265)
                  : (c <= 8305 || (c >= 8308 && c <= 8313)))))
              : (c <= 8329 || (c < 8458
                ? (c < 8450
                  ? (c < 8400
                    ? (c >= 8336 && c <= 8348)
                    : c <= 8432)
                  : (c <= 8450 || c == 8455))
                : (c <= 8467 || (c < 8482
                  ? (c < 8473
                    ? c == 8469
                    : c <= 8477)
                  : (c <= 8482 || c == 8484))))))))))))))
    : (c <= 8486 || (c < 43739
      ? (c < 10160
        ? (c < 9851
          ? (c < 9723
            ? (c < 9000
              ? (c < 8526
                ? (c < 8495
                  ? (c < 8490
                    ? c == 8488
                    : c <= 8493)
                  : (c <= 8505 || (c < 8517
                    ? (c >= 8508 && c <= 8511)
                    : c <= 8521)))
                : (c <= 8526 || (c < 8617
                  ? (c < 8596
                    ? (c >= 8528 && c <= 8585)
                    : c <= 8601)
                  : (c <= 8618 || (c >= 8986 && c <= 8987)))))
              : (c <= 9000 || (c < 9410
                ? (c < 9208
                  ? (c < 9193
                    ? c == 9167
                    : c <= 9203)
                  : (c <= 9210 || (c >= 9312 && c <= 9371)))
                : (c <= 9410 || (c < 9654
                  ? (c < 9642
                    ? (c >= 9450 && c <= 9471)
                    : c <= 9643)
                  : (c <= 9654 || c == 9664))))))
            : (c <= 9726 || (c < 9770
              ? (c < 9752
                ? (c < 9745
                  ? (c < 9742
                    ? (c >= 9728 && c <= 9732)
                    : c <= 9742)
                  : (c <= 9745 || (c >= 9748 && c <= 9749)))
                : (c <= 9752 || (c < 9762
                  ? (c < 9760
                    ? c == 9757
                    : c <= 9760)
                  : (c <= 9763 || c == 9766))))
              : (c <= 9770 || (c < 9800
                ? (c < 9792
                  ? (c < 9784
                    ? (c >= 9774 && c <= 9775)
                    : c <= 9786)
                  : (c <= 9792 || c == 9794))
                : (c <= 9811 || (c < 9829
                  ? (c < 9827
                    ? (c >= 9823 && c <= 9824)
                    : c <= 9827)
                  : (c <= 9830 || c == 9832))))))))
          : (c <= 9851 || (c < 9989
            ? (c < 9924
              ? (c < 9888
                ? (c < 9881
                  ? (c < 9874
                    ? (c >= 9854 && c <= 9855)
                    : c <= 9879)
                  : (c <= 9881 || (c >= 9883 && c <= 9884)))
                : (c <= 9889 || (c < 9904
                  ? (c < 9898
                    ? c == 9895
                    : c <= 9899)
                  : (c <= 9905 || (c >= 9917 && c <= 9918)))))
              : (c <= 9925 || (c < 9961
                ? (c < 9937
                  ? (c < 9934
                    ? c == 9928
                    : c <= 9935)
                  : (c <= 9937 || (c >= 9939 && c <= 9940)))
                : (c <= 9962 || (c < 9981
                  ? (c < 9975
                    ? (c >= 9968 && c <= 9973)
                    : c <= 9978)
                  : (c <= 9981 || c == 9986))))))
            : (c <= 9989 || (c < 10052
              ? (c < 10006
                ? (c < 10002
                  ? (c < 9999
                    ? (c >= 9992 && c <= 9997)
                    : c <= 9999)
                  : (c <= 10002 || c == 10004))
                : (c <= 10006 || (c < 10024
                  ? (c < 10017
                    ? c == 10013
                    : c <= 10017)
                  : (c <= 10024 || (c >= 10035 && c <= 10036)))))
              : (c <= 10052 || (c < 10071
                ? (c < 10062
                  ? (c < 10060
                    ? c == 10055
                    : c <= 10060)
                  : (c <= 10062 || (c >= 10067 && c <= 10069)))
                : (c <= 10071 || (c < 10133
                  ? (c < 10102
                    ? (c >= 10083 && c <= 10084)
                    : c <= 10131)
                  : (c <= 10135 || c == 10145))))))))))
        : (c <= 10160 || (c < 12872
          ? (c < 11712
            ? (c < 11520
              ? (c < 11088
                ? (c < 11013
                  ? (c < 10548
                    ? c == 10175
                    : c <= 10549)
                  : (c <= 11015 || (c >= 11035 && c <= 11036)))
                : (c <= 11088 || (c < 11499
                  ? (c < 11264
                    ? c == 11093
                    : c <= 11492)
                  : (c <= 11507 || c == 11517))))
              : (c <= 11557 || (c < 11647
                ? (c < 11568
                  ? (c < 11565
                    ? c == 11559
                    : c <= 11565)
                  : (c <= 11623 || c == 11631))
                : (c <= 11670 || (c < 11696
                  ? (c < 11688
                    ? (c >= 11680 && c <= 11686)
                    : c <= 11694)
                  : (c <= 11702 || (c >= 11704 && c <= 11710)))))))
            : (c <= 11718 || (c < 12441
              ? (c < 11823
                ? (c < 11736
                  ? (c < 11728
                    ? (c >= 11720 && c <= 11726)
                    : c <= 11734)
                  : (c <= 11742 || (c >= 11744 && c <= 11775)))
                : (c <= 11823 || (c < 12344
                  ? (c < 12321
                    ? (c >= 12293 && c <= 12295)
                    : c <= 12341)
                  : (c <= 12349 || (c >= 12353 && c <= 12438)))))
              : (c <= 12442 || (c < 12593
                ? (c < 12540
                  ? (c < 12449
                    ? (c >= 12445 && c <= 12447)
                    : c <= 12538)
                  : (c <= 12543 || (c >= 12549 && c <= 12591)))
                : (c <= 12686 || (c < 12784
                  ? (c < 12704
                    ? (c >= 12690 && c <= 12693)
                    : c <= 12735)
                  : (c <= 12799 || (c >= 12832 && c <= 12841)))))))))
          : (c <= 12879 || (c < 42965
            ? (c < 42240
              ? (c < 12977
                ? (c < 12951
                  ? (c < 12928
                    ? (c >= 12881 && c <= 12895)
                    : c <= 12937)
                  : (c <= 12951 || c == 12953))
                : (c <= 12991 || (c < 19968
                  ? (c < 19903
                    ? c == 13312
                    : c <= 19903)
                  : (c <= 42124 || (c >= 42192 && c <= 42237)))))
              : (c <= 42508 || (c < 42775
                ? (c < 42612
                  ? (c < 42560
                    ? (c >= 42512 && c <= 42539)
                    : c <= 42610)
                  : (c <= 42621 || (c >= 42623 && c <= 42737)))
                : (c <= 42783 || (c < 42960
                  ? (c < 42891
                    ? (c >= 42786 && c <= 42888)
                    : c <= 42954)
                  : (c <= 42961 || c == 42963))))))
            : (c <= 42969 || (c < 43312
              ? (c < 43136
                ? (c < 43056
                  ? (c < 43052
                    ? (c >= 42994 && c <= 43047)
                    : c <= 43052)
                  : (c <= 43061 || (c >= 43072 && c <= 43123)))
                : (c <= 43205 || (c < 43259
                  ? (c < 43232
                    ? (c >= 43216 && c <= 43225)
                    : c <= 43255)
                  : (c <= 43259 || (c >= 43261 && c <= 43309)))))
              : (c <= 43347 || (c < 43520
                ? (c < 43471
                  ? (c < 43392
                    ? (c >= 43360 && c <= 43388)
                    : c <= 43456)
                  : (c <= 43481 || (c >= 43488 && c <= 43518)))
                : (c <= 43574 || (c < 43616
                  ? (c < 43600
                    ? (c >= 43584 && c <= 43597)
                    : c <= 43609)
                  : (c <= 43638 || (c >= 43642 && c <= 43714)))))))))))))
      : (c <= 43741 || (c < 67424
        ? (c < 65482
          ? (c < 64285
            ? (c < 44012
              ? (c < 43808
                ? (c < 43777
                  ? (c < 43762
                    ? (c >= 43744 && c <= 43759)
                    : c <= 43766)
                  : (c <= 43782 || (c < 43793
                    ? (c >= 43785 && c <= 43790)
                    : c <= 43798)))
                : (c <= 43814 || (c < 43868
                  ? (c < 43824
                    ? (c >= 43816 && c <= 43822)
                    : c <= 43866)
                  : (c <= 43881 || (c >= 43888 && c <= 44010)))))
              : (c <= 44013 || (c < 55243
                ? (c < 55203
                  ? (c < 44032
                    ? (c >= 44016 && c <= 44025)
                    : c <= 44032)
                  : (c <= 55203 || (c >= 55216 && c <= 55238)))
                : (c <= 55291 || (c < 64256
                  ? (c < 64112
                    ? (c >= 63744 && c <= 64109)
                    : c <= 64217)
                  : (c <= 64262 || (c >= 64275 && c <= 64279)))))))
            : (c <= 64296 || (c < 65008
              ? (c < 64323
                ? (c < 64318
                  ? (c < 64312
                    ? (c >= 64298 && c <= 64310)
                    : c <= 64316)
                  : (c <= 64318 || (c >= 64320 && c <= 64321)))
                : (c <= 64324 || (c < 64848
                  ? (c < 64467
                    ? (c >= 64326 && c <= 64433)
                    : c <= 64829)
                  : (c <= 64911 || (c >= 64914 && c <= 64967)))))
              : (c <= 65019 || (c < 65296
                ? (c < 65136
                  ? (c < 65056
                    ? (c >= 65024 && c <= 65039)
                    : c <= 65071)
                  : (c <= 65140 || (c >= 65142 && c <= 65276)))
                : (c <= 65305 || (c < 65382
                  ? (c < 65345
                    ? (c >= 65313 && c <= 65338)
                    : c <= 65370)
                  : (c <= 65470 || (c >= 65474 && c <= 65479)))))))))
          : (c <= 65487 || (c < 66432
            ? (c < 65799
              ? (c < 65576
                ? (c < 65536
                  ? (c < 65498
                    ? (c >= 65490 && c <= 65495)
                    : c <= 65500)
                  : (c <= 65547 || (c >= 65549 && c <= 65574)))
                : (c <= 65594 || (c < 65616
                  ? (c < 65599
                    ? (c >= 65596 && c <= 65597)
                    : c <= 65613)
                  : (c <= 65629 || (c >= 65664 && c <= 65786)))))
              : (c <= 65843 || (c < 66208
                ? (c < 66045
                  ? (c < 65930
                    ? (c >= 65856 && c <= 65912)
                    : c <= 65931)
                  : (c <= 66045 || (c >= 66176 && c <= 66204)))
                : (c <= 66256 || (c < 66349
                  ? (c < 66304
                    ? (c >= 66272 && c <= 66299)
                    : c <= 66339)
                  : (c <= 66378 || (c >= 66384 && c <= 66426)))))))
            : (c <= 66461 || (c < 66928
              ? (c < 66720
                ? (c < 66513
                  ? (c < 66504
                    ? (c >= 66464 && c <= 66499)
                    : c <= 66511)
                  : (c <= 66517 || (c >= 66560 && c <= 66717)))
                : (c <= 66729 || (c < 66816
                  ? (c < 66776
                    ? (c >= 66736 && c <= 66771)
                    : c <= 66811)
                  : (c <= 66855 || (c >= 66864 && c <= 66915)))))
              : (c <= 66938 || (c < 66979
                ? (c < 66964
                  ? (c < 66956
                    ? (c >= 66940 && c <= 66954)
                    : c <= 66962)
                  : (c <= 66965 || (c >= 66967 && c <= 66977)))
                : (c <= 66993 || (c < 67072
                  ? (c < 67003
                    ? (c >= 66995 && c <= 67001)
                    : c <= 67004)
                  : (c <= 67382 || (c >= 67392 && c <= 67413)))))))))))
        : (c <= 67431 || (c < 128371
          ? (c < 127358
            ? (c < 67672
              ? (c < 67592
                ? (c < 67506
                  ? (c < 67463
                    ? (c >= 67456 && c <= 67461)
                    : c <= 67504)
                  : (c <= 67514 || (c >= 67584 && c <= 67589)))
                : (c <= 67592 || (c < 67644
                  ? (c < 67639
                    ? (c >= 67594 && c <= 67637)
                    : c <= 67640)
                  : (c <= 67644 || (c >= 67647 && c <= 67669)))))
              : (c <= 67702 || (c < 67835
                ? (c < 67808
                  ? (c < 67751
                    ? (c >= 67705 && c <= 67742)
                    : c <= 67759)
                  : (c <= 67826 || (c >= 67828 && c <= 67829)))
                : (c <= 67867 || (c < 127183
                  ? (c < 126980
                    ? (c >= 67872 && c <= 67883)
                    : c <= 126980)
                  : (c <= 127183 || (c >= 127344 && c <= 127345)))))))
            : (c <= 127359 || (c < 127780
              ? (c < 127514
                ? (c < 127462
                  ? (c < 127377
                    ? c == 127374
                    : c <= 127386)
                  : (c <= 127487 || (c >= 127489 && c <= 127490)))
                : (c <= 127514 || (c < 127568
                  ? (c < 127538
                    ? c == 127535
                    : c <= 127546)
                  : (c <= 127569 || (c >= 127744 && c <= 127777)))))
              : (c <= 127891 || (c < 127991
                ? (c < 127902
                  ? (c < 127897
                    ? (c >= 127894 && c <= 127895)
                    : c <= 127899)
                  : (c <= 127984 || (c >= 127987 && c <= 127989)))
                : (c <= 128253 || (c < 128336
                  ? (c < 128329
                    ? (c >= 128255 && c <= 128317)
                    : c <= 128334)
                  : (c <= 128359 || (c >= 128367 && c <= 128368)))))))))
          : (c <= 128378 || (c < 128725
            ? (c < 128465
              ? (c < 128420
                ? (c < 128400
                  ? (c < 128394
                    ? c == 128391
                    : c <= 128397)
                  : (c <= 128400 || (c >= 128405 && c <= 128406)))
                : (c <= 128421 || (c < 128444
                  ? (c < 128433
                    ? c == 128424
                    : c <= 128434)
                  : (c <= 128444 || (c >= 128450 && c <= 128452)))))
              : (c <= 128467 || (c < 128495
                ? (c < 128483
                  ? (c < 128481
                    ? (c >= 128476 && c <= 128478)
                    : c <= 128481)
                  : (c <= 128483 || c == 128488))
                : (c <= 128495 || (c < 128640
                  ? (c < 128506
                    ? c == 128499
                    : c <= 128591)
                  : (c <= 128709 || (c >= 128715 && c <= 128722)))))))
            : (c <= 128727 || (c < 129351
              ? (c < 128755
                ? (c < 128747
                  ? (c < 128745
                    ? (c >= 128733 && c <= 128741)
                    : c <= 128745)
                  : (c <= 128748 || c == 128752))
                : (c <= 128764 || (c < 129292
                  ? (c < 129008
                    ? (c >= 128992 && c <= 129003)
                    : c <= 129008)
                  : (c <= 129338 || (c >= 129340 && c <= 129349)))))
              : (c <= 129535 || (c < 129712
                ? (c < 129664
                  ? (c < 129656
                    ? (c >= 129648 && c <= 129652)
                    : c <= 129660)
                  : (c <= 129670 || (c >= 129680 && c <= 129708)))
                : (c <= 129722 || (c < 129760
                  ? (c < 129744
                    ? (c >= 129728 && c <= 129733)
                    : c <= 129753)
                  : (c <= 129767 || (c >= 129776 && c <= 129782)))))))))))))))));
}

static inline bool sym__normal_bare_identifier_character_set_6(int32_t c) {
  return (c < 8484
    ? (c < 3260
      ? (c < 2558
        ? (c < 1476
          ? (c < 736
            ? (c < 169
              ? (c < '?'
                ? (c < '*'
                  ? (c < '#'
                    ? c == '!'
                    : c <= '\'')
                  : (c <= '+' || (c < '0'
                    ? (c >= '-' && c <= '.')
                    : c <= ':')))
                : (c <= 'Z' || (c < '|'
                  ? (c < 'a'
                    ? (c >= '^' && c <= '_')
                    : c <= 'z')
                  : (c <= '|' || c == '~'))))
              : (c <= 170 || (c < 188
                ? (c < 181
                  ? (c < 178
                    ? c == 174
                    : c <= 179)
                  : (c <= 181 || (c >= 185 && c <= 186)))
                : (c <= 190 || (c < 248
                  ? (c < 216
                    ? (c >= 192 && c <= 214)
                    : c <= 246)
                  : (c <= 705 || (c >= 710 && c <= 721)))))))
            : (c <= 740 || (c < 910
              ? (c < 890
                ? (c < 768
                  ? (c < 750
                    ? c == 748
                    : c <= 750)
                  : (c <= 884 || (c >= 886 && c <= 887)))
                : (c <= 893 || (c < 904
                  ? (c < 902
                    ? c == 895
                    : c <= 902)
                  : (c <= 906 || c == 908))))
              : (c <= 929 || (c < 1369
                ? (c < 1155
                  ? (c < 1015
                    ? (c >= 931 && c <= 1013)
                    : c <= 1153)
                  : (c <= 1327 || (c >= 1329 && c <= 1366)))
                : (c <= 1369 || (c < 1471
                  ? (c < 1425
                    ? (c >= 1376 && c <= 1416)
                    : c <= 1469)
                  : (c <= 1471 || (c >= 1473 && c <= 1474)))))))))
          : (c <= 1477 || (c < 2185
            ? (c < 1791
              ? (c < 1568
                ? (c < 1519
                  ? (c < 1488
                    ? c == 1479
                    : c <= 1514)
                  : (c <= 1522 || (c >= 1552 && c <= 1562)))
                : (c <= 1641 || (c < 1759
                  ? (c < 1749
                    ? (c >= 1646 && c <= 1747)
                    : c <= 1756)
                  : (c <= 1768 || (c >= 1770 && c <= 1788)))))
              : (c <= 1791 || (c < 2045
                ? (c < 1984
                  ? (c < 1869
                    ? (c >= 1808 && c <= 1866)
                    : c <= 1969)
                  : (c <= 2037 || c == 2042))
                : (c <= 2045 || (c < 2144
                  ? (c < 2112
                    ? (c >= 2048 && c <= 2093)
                    : c <= 2139)
                  : (c <= 2154 || (c >= 2160 && c <= 2183)))))))
            : (c <= 2190 || (c < 2486
              ? (c < 2437
                ? (c < 2406
                  ? (c < 2275
                    ? (c >= 2200 && c <= 2273)
                    : c <= 2403)
                  : (c <= 2415 || (c >= 2417 && c <= 2435)))
                : (c <= 2444 || (c < 2474
                  ? (c < 2451
                    ? (c >= 2447 && c <= 2448)
                    : c <= 2472)
                  : (c <= 2480 || c == 2482))))
              : (c <= 2489 || (c < 2524
                ? (c < 2507
                  ? (c < 2503
                    ? (c >= 2492 && c <= 2500)
                    : c <= 2504)
                  : (c <= 2510 || c == 2519))
                : (c <= 2525 || (c < 2548
                  ? (c < 2534
                    ? (c >= 2527 && c <= 2531)
                    : c <= 2545)
                  : (c <= 2553 || c == 2556))))))))))
        : (c <= 2558 || (c < 2901
          ? (c < 2730
            ? (c < 2631
              ? (c < 2610
                ? (c < 2575
                  ? (c < 2565
                    ? (c >= 2561 && c <= 2563)
                    : c <= 2570)
                  : (c <= 2576 || (c < 2602
                    ? (c >= 2579 && c <= 2600)
                    : c <= 2608)))
                : (c <= 2611 || (c < 2620
                  ? (c < 2616
                    ? (c >= 2613 && c <= 2614)
                    : c <= 2617)
                  : (c <= 2620 || (c >= 2622 && c <= 2626)))))
              : (c <= 2632 || (c < 2662
                ? (c < 2649
                  ? (c < 2641
                    ? (c >= 2635 && c <= 2637)
                    : c <= 2641)
                  : (c <= 2652 || c == 2654))
                : (c <= 2677 || (c < 2703
                  ? (c < 2693
                    ? (c >= 2689 && c <= 2691)
                    : c <= 2701)
                  : (c <= 2705 || (c >= 2707 && c <= 2728)))))))
            : (c <= 2736 || (c < 2817
              ? (c < 2763
                ? (c < 2748
                  ? (c < 2741
                    ? (c >= 2738 && c <= 2739)
                    : c <= 2745)
                  : (c <= 2757 || (c >= 2759 && c <= 2761)))
                : (c <= 2765 || (c < 2790
                  ? (c < 2784
                    ? c == 2768
                    : c <= 2787)
                  : (c <= 2799 || (c >= 2809 && c <= 2815)))))
              : (c <= 2819 || (c < 2866
                ? (c < 2835
                  ? (c < 2831
                    ? (c >= 2821 && c <= 2828)
                    : c <= 2832)
                  : (c <= 2856 || (c >= 2858 && c <= 2864)))
                : (c <= 2867 || (c < 2887
                  ? (c < 2876
                    ? (c >= 2869 && c <= 2873)
                    : c <= 2884)
                  : (c <= 2888 || (c >= 2891 && c <= 2893)))))))))
          : (c <= 2903 || (c < 3046
            ? (c < 2972
              ? (c < 2946
                ? (c < 2918
                  ? (c < 2911
                    ? (c >= 2908 && c <= 2909)
                    : c <= 2915)
                  : (c <= 2927 || (c >= 2929 && c <= 2935)))
                : (c <= 2947 || (c < 2962
                  ? (c < 2958
                    ? (c >= 2949 && c <= 2954)
                    : c <= 2960)
                  : (c <= 2965 || (c >= 2969 && c <= 2970)))))
              : (c <= 2972 || (c < 3006
                ? (c < 2984
                  ? (c < 2979
                    ? (c >= 2974 && c <= 2975)
                    : c <= 2980)
                  : (c <= 2986 || (c >= 2990 && c <= 3001)))
                : (c <= 3010 || (c < 3024
                  ? (c < 3018
                    ? (c >= 3014 && c <= 3016)
                    : c <= 3021)
                  : (c <= 3024 || c == 3031))))))
            : (c <= 3058 || (c < 3165
              ? (c < 3132
                ? (c < 3090
                  ? (c < 3086
                    ? (c >= 3072 && c <= 3084)
                    : c <= 3088)
                  : (c <= 3112 || (c >= 3114 && c <= 3129)))
                : (c <= 3140 || (c < 3157
                  ? (c < 3146
                    ? (c >= 3142 && c <= 3144)
                    : c <= 3149)
                  : (c <= 3158 || (c >= 3160 && c <= 3162)))))
              : (c <= 3165 || (c < 3205
                ? (c < 3192
                  ? (c < 3174
                    ? (c >= 3168 && c <= 3171)
                    : c <= 3183)
                  : (c <= 3198 || (c >= 3200 && c <= 3203)))
                : (c <= 3212 || (c < 3242
                  ? (c < 3218
                    ? (c >= 3214 && c <= 3216)
                    : c <= 3240)
                  : (c <= 3251 || (c >= 3253 && c <= 3257)))))))))))))
      : (c <= 3268 || (c < 5121
        ? (c < 3804
          ? (c < 3520
            ? (c < 3398
              ? (c < 3302
                ? (c < 3285
                  ? (c < 3274
                    ? (c >= 3270 && c <= 3272)
                    : c <= 3277)
                  : (c <= 3286 || (c < 3296
                    ? (c >= 3293 && c <= 3294)
                    : c <= 3299)))
                : (c <= 3311 || (c < 3342
                  ? (c < 3328
                    ? (c >= 3313 && c <= 3314)
                    : c <= 3340)
                  : (c <= 3344 || (c >= 3346 && c <= 3396)))))
              : (c <= 3400 || (c < 3457
                ? (c < 3430
                  ? (c < 3412
                    ? (c >= 3402 && c <= 3406)
                    : c <= 3427)
                  : (c <= 3448 || (c >= 3450 && c <= 3455)))
                : (c <= 3459 || (c < 3507
                  ? (c < 3482
                    ? (c >= 3461 && c <= 3478)
                    : c <= 3505)
                  : (c <= 3515 || c == 3517))))))
            : (c <= 3526 || (c < 3713
              ? (c < 3558
                ? (c < 3542
                  ? (c < 3535
                    ? c == 3530
                    : c <= 3540)
                  : (c <= 3542 || (c >= 3544 && c <= 3551)))
                : (c <= 3567 || (c < 3648
                  ? (c < 3585
                    ? (c >= 3570 && c <= 3571)
                    : c <= 3642)
                  : (c <= 3662 || (c >= 3664 && c <= 3673)))))
              : (c <= 3714 || (c < 3751
                ? (c < 3724
                  ? (c < 3718
                    ? c == 3716
                    : c <= 3722)
                  : (c <= 3747 || c == 3749))
                : (c <= 3773 || (c < 3784
                  ? (c < 3782
                    ? (c >= 3776 && c <= 3780)
                    : c <= 3782)
                  : (c <= 3789 || (c >= 3792 && c <= 3801)))))))))
          : (c <= 3807 || (c < 4682
            ? (c < 3974
              ? (c < 3895
                ? (c < 3872
                  ? (c < 3864
                    ? c == 3840
                    : c <= 3865)
                  : (c <= 3891 || c == 3893))
                : (c <= 3895 || (c < 3913
                  ? (c < 3902
                    ? c == 3897
                    : c <= 3911)
                  : (c <= 3948 || (c >= 3953 && c <= 3972)))))
              : (c <= 3991 || (c < 4256
                ? (c < 4096
                  ? (c < 4038
                    ? (c >= 3993 && c <= 4028)
                    : c <= 4038)
                  : (c <= 4169 || (c >= 4176 && c <= 4253)))
                : (c <= 4293 || (c < 4304
                  ? (c < 4301
                    ? c == 4295
                    : c <= 4301)
                  : (c <= 4346 || (c >= 4348 && c <= 4680)))))))
            : (c <= 4685 || (c < 4802
              ? (c < 4746
                ? (c < 4698
                  ? (c < 4696
                    ? (c >= 4688 && c <= 4694)
                    : c <= 4696)
                  : (c <= 4701 || (c >= 4704 && c <= 4744)))
                : (c <= 4749 || (c < 4792
                  ? (c < 4786
                    ? (c >= 4752 && c <= 4784)
                    : c <= 4789)
                  : (c <= 4798 || c == 4800))))
              : (c <= 4805 || (c < 4957
                ? (c < 4882
                  ? (c < 4824
                    ? (c >= 4808 && c <= 4822)
                    : c <= 4880)
                  : (c <= 4885 || (c >= 4888 && c <= 4954)))
                : (c <= 4959 || (c < 5024
                  ? (c < 4992
                    ? (c >= 4969 && c <= 4988)
                    : c <= 5007)
                  : (c <= 5109 || (c >= 5112 && c <= 5117)))))))))))
        : (c <= 5740 || (c < 7168
          ? (c < 6320
            ? (c < 6002
              ? (c < 5888
                ? (c < 5792
                  ? (c < 5761
                    ? (c >= 5743 && c <= 5759)
                    : c <= 5786)
                  : (c <= 5866 || (c >= 5870 && c <= 5880)))
                : (c <= 5909 || (c < 5984
                  ? (c < 5952
                    ? (c >= 5919 && c <= 5940)
                    : c <= 5971)
                  : (c <= 5996 || (c >= 5998 && c <= 6000)))))
              : (c <= 6003 || (c < 6128
                ? (c < 6108
                  ? (c < 6103
                    ? (c >= 6016 && c <= 6099)
                    : c <= 6103)
                  : (c <= 6109 || (c >= 6112 && c <= 6121)))
                : (c <= 6137 || (c < 6176
                  ? (c < 6159
                    ? (c >= 6155 && c <= 6157)
                    : c <= 6169)
                  : (c <= 6264 || (c >= 6272 && c <= 6314)))))))
            : (c <= 6389 || (c < 6688
              ? (c < 6512
                ? (c < 6448
                  ? (c < 6432
                    ? (c >= 6400 && c <= 6430)
                    : c <= 6443)
                  : (c <= 6459 || (c >= 6470 && c <= 6509)))
                : (c <= 6516 || (c < 6608
                  ? (c < 6576
                    ? (c >= 6528 && c <= 6571)
                    : c <= 6601)
                  : (c <= 6618 || (c >= 6656 && c <= 6683)))))
              : (c <= 6750 || (c < 6832
                ? (c < 6800
                  ? (c < 6783
                    ? (c >= 6752 && c <= 6780)
                    : c <= 6793)
                  : (c <= 6809 || c == 6823))
                : (c <= 6862 || (c < 7019
                  ? (c < 6992
                    ? (c >= 6912 && c <= 6988)
                    : c <= 7001)
                  : (c <= 7027 || (c >= 7040 && c <= 7155)))))))))
          : (c <= 7223 || (c < 8130
            ? (c < 7968
              ? (c < 7357
                ? (c < 7296
                  ? (c < 7245
                    ? (c >= 7232 && c <= 7241)
                    : c <= 7293)
                  : (c <= 7304 || (c >= 7312 && c <= 7354)))
                : (c <= 7359 || (c < 7424
                  ? (c < 7380
                    ? (c >= 7376 && c <= 7378)
                    : c <= 7418)
                  : (c <= 7957 || (c >= 7960 && c <= 7965)))))
              : (c <= 8005 || (c < 8029
                ? (c < 8025
                  ? (c < 8016
                    ? (c >= 8008 && c <= 8013)
                    : c <= 8023)
                  : (c <= 8025 || c == 8027))
                : (c <= 8029 || (c < 8118
                  ? (c < 8064
                    ? (c >= 8031 && c <= 8061)
                    : c <= 8116)
                  : (c <= 8124 || c == 8126))))))
            : (c <= 8132 || (c < 8308
              ? (c < 8178
                ? (c < 8150
                  ? (c < 8144
                    ? (c >= 8134 && c <= 8140)
                    : c <= 8147)
                  : (c <= 8155 || (c >= 8160 && c <= 8172)))
                : (c <= 8180 || (c < 8265
                  ? (c < 8252
                    ? (c >= 8182 && c <= 8188)
                    : c <= 8252)
                  : (c <= 8265 || (c >= 8304 && c <= 8305)))))
              : (c <= 8313 || (c < 8455
                ? (c < 8400
                  ? (c < 8336
                    ? (c >= 8319 && c <= 8329)
                    : c <= 8348)
                  : (c <= 8432 || c == 8450))
                : (c <= 8455 || (c < 8473
                  ? (c < 8469
                    ? (c >= 8458 && c <= 8467)
                    : c <= 8469)
                  : (c <= 8477 || c == 8482))))))))))))))
    : (c <= 8484 || (c < 43739
      ? (c < 10145
        ? (c < 9832
          ? (c < 9664
            ? (c < 8986
              ? (c < 8517
                ? (c < 8490
                  ? (c < 8488
                    ? c == 8486
                    : c <= 8488)
                  : (c <= 8493 || (c < 8508
                    ? (c >= 8495 && c <= 8505)
                    : c <= 8511)))
                : (c <= 8521 || (c < 8596
                  ? (c < 8528
                    ? c == 8526
                    : c <= 8585)
                  : (c <= 8601 || (c >= 8617 && c <= 8618)))))
              : (c <= 8987 || (c < 9312
                ? (c < 9193
                  ? (c < 9167
                    ? c == 9000
                    : c <= 9167)
                  : (c <= 9203 || (c >= 9208 && c <= 9210)))
                : (c <= 9371 || (c < 9642
                  ? (c < 9450
                    ? c == 9410
                    : c <= 9471)
                  : (c <= 9643 || c == 9654))))))
            : (c <= 9664 || (c < 9766
              ? (c < 9748
                ? (c < 9742
                  ? (c < 9728
                    ? (c >= 9723 && c <= 9726)
                    : c <= 9732)
                  : (c <= 9742 || c == 9745))
                : (c <= 9749 || (c < 9760
                  ? (c < 9757
                    ? c == 9752
                    : c <= 9757)
                  : (c <= 9760 || (c >= 9762 && c <= 9763)))))
              : (c <= 9766 || (c < 9794
                ? (c < 9784
                  ? (c < 9774
                    ? c == 9770
                    : c <= 9775)
                  : (c <= 9786 || c == 9792))
                : (c <= 9794 || (c < 9827
                  ? (c < 9823
                    ? (c >= 9800 && c <= 9811)
                    : c <= 9824)
                  : (c <= 9827 || (c >= 9829 && c <= 9830)))))))))
          : (c <= 9832 || (c < 9986
            ? (c < 9917
              ? (c < 9883
                ? (c < 9874
                  ? (c < 9854
                    ? c == 9851
                    : c <= 9855)
                  : (c <= 9879 || c == 9881))
                : (c <= 9884 || (c < 9898
                  ? (c < 9895
                    ? (c >= 9888 && c <= 9889)
                    : c <= 9895)
                  : (c <= 9899 || (c >= 9904 && c <= 9905)))))
              : (c <= 9918 || (c < 9939
                ? (c < 9934
                  ? (c < 9928
                    ? (c >= 9924 && c <= 9925)
                    : c <= 9928)
                  : (c <= 9935 || c == 9937))
                : (c <= 9940 || (c < 9975
                  ? (c < 9968
                    ? (c >= 9961 && c <= 9962)
                    : c <= 9973)
                  : (c <= 9978 || c == 9981))))))
            : (c <= 9986 || (c < 10035
              ? (c < 10004
                ? (c < 9999
                  ? (c < 9992
                    ? c == 9989
                    : c <= 9997)
                  : (c <= 9999 || c == 10002))
                : (c <= 10004 || (c < 10017
                  ? (c < 10013
                    ? c == 10006
                    : c <= 10013)
                  : (c <= 10017 || c == 10024))))
              : (c <= 10036 || (c < 10067
                ? (c < 10060
                  ? (c < 10055
                    ? c == 10052
                    : c <= 10055)
                  : (c <= 10060 || c == 10062))
                : (c <= 10069 || (c < 10102
                  ? (c < 10083
                    ? c == 10071
                    : c <= 10084)
                  : (c <= 10131 || (c >= 10133 && c <= 10135)))))))))))
        : (c <= 10145 || (c < 12872
          ? (c < 11712
            ? (c < 11520
              ? (c < 11088
                ? (c < 10548
                  ? (c < 10175
                    ? c == 10160
                    : c <= 10175)
                  : (c <= 10549 || (c < 11035
                    ? (c >= 11013 && c <= 11015)
                    : c <= 11036)))
                : (c <= 11088 || (c < 11499
                  ? (c < 11264
                    ? c == 11093
                    : c <= 11492)
                  : (c <= 11507 || c == 11517))))
              : (c <= 11557 || (c < 11647
                ? (c < 11568
                  ? (c < 11565
                    ? c == 11559
                    : c <= 11565)
                  : (c <= 11623 || c == 11631))
                : (c <= 11670 || (c < 11696
                  ? (c < 11688
                    ? (c >= 11680 && c <= 11686)
                    : c <= 11694)
                  : (c <= 11702 || (c >= 11704 && c <= 11710)))))))
            : (c <= 11718 || (c < 12441
              ? (c < 11823
                ? (c < 11736
                  ? (c < 11728
                    ? (c >= 11720 && c <= 11726)
                    : c <= 11734)
                  : (c <= 11742 || (c >= 11744 && c <= 11775)))
                : (c <= 11823 || (c < 12344
                  ? (c < 12321
                    ? (c >= 12293 && c <= 12295)
                    : c <= 12341)
                  : (c <= 12349 || (c >= 12353 && c <= 12438)))))
              : (c <= 12442 || (c < 12593
                ? (c < 12540
                  ? (c < 12449
                    ? (c >= 12445 && c <= 12447)
                    : c <= 12538)
                  : (c <= 12543 || (c >= 12549 && c <= 12591)))
                : (c <= 12686 || (c < 12784
                  ? (c < 12704
                    ? (c >= 12690 && c <= 12693)
                    : c <= 12735)
                  : (c <= 12799 || (c >= 12832 && c <= 12841)))))))))
          : (c <= 12879 || (c < 42965
            ? (c < 42240
              ? (c < 12977
                ? (c < 12951
                  ? (c < 12928
                    ? (c >= 12881 && c <= 12895)
                    : c <= 12937)
                  : (c <= 12951 || c == 12953))
                : (c <= 12991 || (c < 19968
                  ? (c < 19903
                    ? c == 13312
                    : c <= 19903)
                  : (c <= 42124 || (c >= 42192 && c <= 42237)))))
              : (c <= 42508 || (c < 42775
                ? (c < 42612
                  ? (c < 42560
                    ? (c >= 42512 && c <= 42539)
                    : c <= 42610)
                  : (c <= 42621 || (c >= 42623 && c <= 42737)))
                : (c <= 42783 || (c < 42960
                  ? (c < 42891
                    ? (c >= 42786 && c <= 42888)
                    : c <= 42954)
                  : (c <= 42961 || c == 42963))))))
            : (c <= 42969 || (c < 43312
              ? (c < 43136
                ? (c < 43056
                  ? (c < 43052
                    ? (c >= 42994 && c <= 43047)
                    : c <= 43052)
                  : (c <= 43061 || (c >= 43072 && c <= 43123)))
                : (c <= 43205 || (c < 43259
                  ? (c < 43232
                    ? (c >= 43216 && c <= 43225)
                    : c <= 43255)
                  : (c <= 43259 || (c >= 43261 && c <= 43309)))))
              : (c <= 43347 || (c < 43520
                ? (c < 43471
                  ? (c < 43392
                    ? (c >= 43360 && c <= 43388)
                    : c <= 43456)
                  : (c <= 43481 || (c >= 43488 && c <= 43518)))
                : (c <= 43574 || (c < 43616
                  ? (c < 43600
                    ? (c >= 43584 && c <= 43597)
                    : c <= 43609)
                  : (c <= 43638 || (c >= 43642 && c <= 43714)))))))))))))
      : (c <= 43741 || (c < 67424
        ? (c < 65482
          ? (c < 64285
            ? (c < 44012
              ? (c < 43808
                ? (c < 43777
                  ? (c < 43762
                    ? (c >= 43744 && c <= 43759)
                    : c <= 43766)
                  : (c <= 43782 || (c < 43793
                    ? (c >= 43785 && c <= 43790)
                    : c <= 43798)))
                : (c <= 43814 || (c < 43868
                  ? (c < 43824
                    ? (c >= 43816 && c <= 43822)
                    : c <= 43866)
                  : (c <= 43881 || (c >= 43888 && c <= 44010)))))
              : (c <= 44013 || (c < 55243
                ? (c < 55203
                  ? (c < 44032
                    ? (c >= 44016 && c <= 44025)
                    : c <= 44032)
                  : (c <= 55203 || (c >= 55216 && c <= 55238)))
                : (c <= 55291 || (c < 64256
                  ? (c < 64112
                    ? (c >= 63744 && c <= 64109)
                    : c <= 64217)
                  : (c <= 64262 || (c >= 64275 && c <= 64279)))))))
            : (c <= 64296 || (c < 65008
              ? (c < 64323
                ? (c < 64318
                  ? (c < 64312
                    ? (c >= 64298 && c <= 64310)
                    : c <= 64316)
                  : (c <= 64318 || (c >= 64320 && c <= 64321)))
                : (c <= 64324 || (c < 64848
                  ? (c < 64467
                    ? (c >= 64326 && c <= 64433)
                    : c <= 64829)
                  : (c <= 64911 || (c >= 64914 && c <= 64967)))))
              : (c <= 65019 || (c < 65296
                ? (c < 65136
                  ? (c < 65056
                    ? (c >= 65024 && c <= 65039)
                    : c <= 65071)
                  : (c <= 65140 || (c >= 65142 && c <= 65276)))
                : (c <= 65305 || (c < 65382
                  ? (c < 65345
                    ? (c >= 65313 && c <= 65338)
                    : c <= 65370)
                  : (c <= 65470 || (c >= 65474 && c <= 65479)))))))))
          : (c <= 65487 || (c < 66432
            ? (c < 65799
              ? (c < 65576
                ? (c < 65536
                  ? (c < 65498
                    ? (c >= 65490 && c <= 65495)
                    : c <= 65500)
                  : (c <= 65547 || (c >= 65549 && c <= 65574)))
                : (c <= 65594 || (c < 65616
                  ? (c < 65599
                    ? (c >= 65596 && c <= 65597)
                    : c <= 65613)
                  : (c <= 65629 || (c >= 65664 && c <= 65786)))))
              : (c <= 65843 || (c < 66208
                ? (c < 66045
                  ? (c < 65930
                    ? (c >= 65856 && c <= 65912)
                    : c <= 65931)
                  : (c <= 66045 || (c >= 66176 && c <= 66204)))
                : (c <= 66256 || (c < 66349
                  ? (c < 66304
                    ? (c >= 66272 && c <= 66299)
                    : c <= 66339)
                  : (c <= 66378 || (c >= 66384 && c <= 66426)))))))
            : (c <= 66461 || (c < 66928
              ? (c < 66720
                ? (c < 66513
                  ? (c < 66504
                    ? (c >= 66464 && c <= 66499)
                    : c <= 66511)
                  : (c <= 66517 || (c >= 66560 && c <= 66717)))
                : (c <= 66729 || (c < 66816
                  ? (c < 66776
                    ? (c >= 66736 && c <= 66771)
                    : c <= 66811)
                  : (c <= 66855 || (c >= 66864 && c <= 66915)))))
              : (c <= 66938 || (c < 66979
                ? (c < 66964
                  ? (c < 66956
                    ? (c >= 66940 && c <= 66954)
                    : c <= 66962)
                  : (c <= 66965 || (c >= 66967 && c <= 66977)))
                : (c <= 66993 || (c < 67072
                  ? (c < 67003
                    ? (c >= 66995 && c <= 67001)
                    : c <= 67004)
                  : (c <= 67382 || (c >= 67392 && c <= 67413)))))))))))
        : (c <= 67431 || (c < 128371
          ? (c < 127358
            ? (c < 67672
              ? (c < 67592
                ? (c < 67506
                  ? (c < 67463
                    ? (c >= 67456 && c <= 67461)
                    : c <= 67504)
                  : (c <= 67514 || (c >= 67584 && c <= 67589)))
                : (c <= 67592 || (c < 67644
                  ? (c < 67639
                    ? (c >= 67594 && c <= 67637)
                    : c <= 67640)
                  : (c <= 67644 || (c >= 67647 && c <= 67669)))))
              : (c <= 67702 || (c < 67835
                ? (c < 67808
                  ? (c < 67751
                    ? (c >= 67705 && c <= 67742)
                    : c <= 67759)
                  : (c <= 67826 || (c >= 67828 && c <= 67829)))
                : (c <= 67867 || (c < 127183
                  ? (c < 126980
                    ? (c >= 67872 && c <= 67883)
                    : c <= 126980)
                  : (c <= 127183 || (c >= 127344 && c <= 127345)))))))
            : (c <= 127359 || (c < 127780
              ? (c < 127514
                ? (c < 127462
                  ? (c < 127377
                    ? c == 127374
                    : c <= 127386)
                  : (c <= 127487 || (c >= 127489 && c <= 127490)))
                : (c <= 127514 || (c < 127568
                  ? (c < 127538
                    ? c == 127535
                    : c <= 127546)
                  : (c <= 127569 || (c >= 127744 && c <= 127777)))))
              : (c <= 127891 || (c < 127991
                ? (c < 127902
                  ? (c < 127897
                    ? (c >= 127894 && c <= 127895)
                    : c <= 127899)
                  : (c <= 127984 || (c >= 127987 && c <= 127989)))
                : (c <= 128253 || (c < 128336
                  ? (c < 128329
                    ? (c >= 128255 && c <= 128317)
                    : c <= 128334)
                  : (c <= 128359 || (c >= 128367 && c <= 128368)))))))))
          : (c <= 128378 || (c < 128725
            ? (c < 128465
              ? (c < 128420
                ? (c < 128400
                  ? (c < 128394
                    ? c == 128391
                    : c <= 128397)
                  : (c <= 128400 || (c >= 128405 && c <= 128406)))
                : (c <= 128421 || (c < 128444
                  ? (c < 128433
                    ? c == 128424
                    : c <= 128434)
                  : (c <= 128444 || (c >= 128450 && c <= 128452)))))
              : (c <= 128467 || (c < 128495
                ? (c < 128483
                  ? (c < 128481
                    ? (c >= 128476 && c <= 128478)
                    : c <= 128481)
                  : (c <= 128483 || c == 128488))
                : (c <= 128495 || (c < 128640
                  ? (c < 128506
                    ? c == 128499
                    : c <= 128591)
                  : (c <= 128709 || (c >= 128715 && c <= 128722)))))))
            : (c <= 128727 || (c < 129351
              ? (c < 128755
                ? (c < 128747
                  ? (c < 128745
                    ? (c >= 128733 && c <= 128741)
                    : c <= 128745)
                  : (c <= 128748 || c == 128752))
                : (c <= 128764 || (c < 129292
                  ? (c < 129008
                    ? (c >= 128992 && c <= 129003)
                    : c <= 129008)
                  : (c <= 129338 || (c >= 129340 && c <= 129349)))))
              : (c <= 129535 || (c < 129712
                ? (c < 129664
                  ? (c < 129656
                    ? (c >= 129648 && c <= 129652)
                    : c <= 129660)
                  : (c <= 129670 || (c >= 129680 && c <= 129708)))
                : (c <= 129722 || (c < 129760
                  ? (c < 129744
                    ? (c >= 129728 && c <= 129733)
                    : c <= 129753)
                  : (c <= 129767 || (c >= 129776 && c <= 129782)))))))))))))))));
}

static inline bool sym___identifier_char_no_digit_character_set_1(int32_t c) {
  return (c < 6002
    ? (c < 2949
      ? (c < 2437
        ? (c < 1329
          ? (c < 248
            ? (c < '~'
              ? (c < '-'
                ? (c < '#'
                  ? c == '!'
                  : (c <= '\'' || (c >= '*' && c <= '+')))
                : (c <= ':' || (c < '^'
                  ? (c >= '?' && c <= 'Z')
                  : (c <= '_' || (c >= 'a' && c <= '|')))))
              : (c <= '~' || (c < 185
                ? (c < 178
                  ? c == 170
                  : (c <= 179 || c == 181))
                : (c <= 186 || (c < 192
                  ? (c >= 188 && c <= 190)
                  : (c <= 214 || (c >= 216 && c <= 246)))))))
            : (c <= 705 || (c < 895
              ? (c < 750
                ? (c < 736
                  ? (c >= 710 && c <= 721)
                  : (c <= 740 || c == 748))
                : (c <= 750 || (c < 886
                  ? (c >= 768 && c <= 884)
                  : (c <= 887 || (c >= 890 && c <= 893)))))
              : (c <= 895 || (c < 910
                ? (c < 904
                  ? c == 902
                  : (c <= 906 || c == 908))
                : (c <= 929 || (c < 1015
                  ? (c >= 931 && c <= 1013)
                  : (c <= 1153 || (c >= 1155 && c <= 1327)))))))))
          : (c <= 1366 || (c < 1791
            ? (c < 1488
              ? (c < 1471
                ? (c < 1376
                  ? c == 1369
                  : (c <= 1416 || (c >= 1425 && c <= 1469)))
                : (c <= 1471 || (c < 1476
                  ? (c >= 1473 && c <= 1474)
                  : (c <= 1477 || c == 1479))))
              : (c <= 1514 || (c < 1646
                ? (c < 1552
                  ? (c >= 1519 && c <= 1522)
                  : (c <= 1562 || (c >= 1568 && c <= 1641)))
                : (c <= 1747 || (c < 1759
                  ? (c >= 1749 && c <= 1756)
                  : (c <= 1768 || (c >= 1770 && c <= 1788)))))))
            : (c <= 1791 || (c < 2144
              ? (c < 2042
                ? (c < 1869
                  ? (c >= 1808 && c <= 1866)
                  : (c <= 1969 || (c >= 1984 && c <= 2037)))
                : (c <= 2042 || (c < 2048
                  ? c == 2045
                  : (c <= 2093 || (c >= 2112 && c <= 2139)))))
              : (c <= 2154 || (c < 2275
                ? (c < 2185
                  ? (c >= 2160 && c <= 2183)
                  : (c <= 2190 || (c >= 2200 && c <= 2273)))
                : (c <= 2403 || (c < 2417
                  ? (c >= 2406 && c <= 2415)
                  : c <= 2435)))))))))
        : (c <= 2444 || (c < 2662
          ? (c < 2561
            ? (c < 2507
              ? (c < 2482
                ? (c < 2451
                  ? (c >= 2447 && c <= 2448)
                  : (c <= 2472 || (c >= 2474 && c <= 2480)))
                : (c <= 2482 || (c < 2492
                  ? (c >= 2486 && c <= 2489)
                  : (c <= 2500 || (c >= 2503 && c <= 2504)))))
              : (c <= 2510 || (c < 2534
                ? (c < 2524
                  ? c == 2519
                  : (c <= 2525 || (c >= 2527 && c <= 2531)))
                : (c <= 2545 || (c < 2556
                  ? (c >= 2548 && c <= 2553)
                  : (c <= 2556 || c == 2558))))))
            : (c <= 2563 || (c < 2620
              ? (c < 2602
                ? (c < 2575
                  ? (c >= 2565 && c <= 2570)
                  : (c <= 2576 || (c >= 2579 && c <= 2600)))
                : (c <= 2608 || (c < 2613
                  ? (c >= 2610 && c <= 2611)
                  : (c <= 2614 || (c >= 2616 && c <= 2617)))))
              : (c <= 2620 || (c < 2641
                ? (c < 2631
                  ? (c >= 2622 && c <= 2626)
                  : (c <= 2632 || (c >= 2635 && c <= 2637)))
                : (c <= 2641 || (c < 2654
                  ? (c >= 2649 && c <= 2652)
                  : c <= 2654)))))))
          : (c <= 2677 || (c < 2821
            ? (c < 2748
              ? (c < 2707
                ? (c < 2693
                  ? (c >= 2689 && c <= 2691)
                  : (c <= 2701 || (c >= 2703 && c <= 2705)))
                : (c <= 2728 || (c < 2738
                  ? (c >= 2730 && c <= 2736)
                  : (c <= 2739 || (c >= 2741 && c <= 2745)))))
              : (c <= 2757 || (c < 2784
                ? (c < 2763
                  ? (c >= 2759 && c <= 2761)
                  : (c <= 2765 || c == 2768))
                : (c <= 2787 || (c < 2809
                  ? (c >= 2790 && c <= 2799)
                  : (c <= 2815 || (c >= 2817 && c <= 2819)))))))
            : (c <= 2828 || (c < 2891
              ? (c < 2866
                ? (c < 2835
                  ? (c >= 2831 && c <= 2832)
                  : (c <= 2856 || (c >= 2858 && c <= 2864)))
                : (c <= 2867 || (c < 2876
                  ? (c >= 2869 && c <= 2873)
                  : (c <= 2884 || (c >= 2887 && c <= 2888)))))
              : (c <= 2893 || (c < 2918
                ? (c < 2908
                  ? (c >= 2901 && c <= 2903)
                  : (c <= 2909 || (c >= 2911 && c <= 2915)))
                : (c <= 2927 || (c < 2946
                  ? (c >= 2929 && c <= 2935)
                  : c <= 2947)))))))))))
      : (c <= 2954 || (c < 3585
        ? (c < 3218
          ? (c < 3086
            ? (c < 2990
              ? (c < 2972
                ? (c < 2962
                  ? (c >= 2958 && c <= 2960)
                  : (c <= 2965 || (c >= 2969 && c <= 2970)))
                : (c <= 2972 || (c < 2979
                  ? (c >= 2974 && c <= 2975)
                  : (c <= 2980 || (c >= 2984 && c <= 2986)))))
              : (c <= 3001 || (c < 3024
                ? (c < 3014
                  ? (c >= 3006 && c <= 3010)
                  : (c <= 3016 || (c >= 3018 && c <= 3021)))
                : (c <= 3024 || (c < 3046
                  ? c == 3031
                  : (c <= 3058 || (c >= 3072 && c <= 3084)))))))
            : (c <= 3088 || (c < 3165
              ? (c < 3142
                ? (c < 3114
                  ? (c >= 3090 && c <= 3112)
                  : (c <= 3129 || (c >= 3132 && c <= 3140)))
                : (c <= 3144 || (c < 3157
                  ? (c >= 3146 && c <= 3149)
                  : (c <= 3158 || (c >= 3160 && c <= 3162)))))
              : (c <= 3165 || (c < 3200
                ? (c < 3174
                  ? (c >= 3168 && c <= 3171)
                  : (c <= 3183 || (c >= 3192 && c <= 3198)))
                : (c <= 3203 || (c < 3214
                  ? (c >= 3205 && c <= 3212)
                  : c <= 3216)))))))
          : (c <= 3240 || (c < 3412
            ? (c < 3296
              ? (c < 3270
                ? (c < 3253
                  ? (c >= 3242 && c <= 3251)
                  : (c <= 3257 || (c >= 3260 && c <= 3268)))
                : (c <= 3272 || (c < 3285
                  ? (c >= 3274 && c <= 3277)
                  : (c <= 3286 || (c >= 3293 && c <= 3294)))))
              : (c <= 3299 || (c < 3342
                ? (c < 3313
                  ? (c >= 3302 && c <= 3311)
                  : (c <= 3314 || (c >= 3328 && c <= 3340)))
                : (c <= 3344 || (c < 3398
                  ? (c >= 3346 && c <= 3396)
                  : (c <= 3400 || (c >= 3402 && c <= 3406)))))))
            : (c <= 3427 || (c < 3520
              ? (c < 3461
                ? (c < 3450
                  ? (c >= 3430 && c <= 3448)
                  : (c <= 3455 || (c >= 3457 && c <= 3459)))
                : (c <= 3478 || (c < 3507
                  ? (c >= 3482 && c <= 3505)
                  : (c <= 3515 || c == 3517))))
              : (c <= 3526 || (c < 3544
                ? (c < 3535
                  ? c == 3530
                  : (c <= 3540 || c == 3542))
                : (c <= 3551 || (c < 3570
                  ? (c >= 3558 && c <= 3567)
                  : c <= 3571)))))))))
        : (c <= 3642 || (c < 4304
          ? (c < 3872
            ? (c < 3751
              ? (c < 3716
                ? (c < 3664
                  ? (c >= 3648 && c <= 3662)
                  : (c <= 3673 || (c >= 3713 && c <= 3714)))
                : (c <= 3716 || (c < 3724
                  ? (c >= 3718 && c <= 3722)
                  : (c <= 3747 || c == 3749))))
              : (c <= 3773 || (c < 3792
                ? (c < 3782
                  ? (c >= 3776 && c <= 3780)
                  : (c <= 3782 || (c >= 3784 && c <= 3789)))
                : (c <= 3801 || (c < 3840
                  ? (c >= 3804 && c <= 3807)
                  : (c <= 3840 || (c >= 3864 && c <= 3865)))))))
            : (c <= 3891 || (c < 3993
              ? (c < 3902
                ? (c < 3895
                  ? c == 3893
                  : (c <= 3895 || c == 3897))
                : (c <= 3911 || (c < 3953
                  ? (c >= 3913 && c <= 3948)
                  : (c <= 3972 || (c >= 3974 && c <= 3991)))))
              : (c <= 4028 || (c < 4256
                ? (c < 4096
                  ? c == 4038
                  : (c <= 4169 || (c >= 4176 && c <= 4253)))
                : (c <= 4293 || (c < 4301
                  ? c == 4295
                  : c <= 4301)))))))
          : (c <= 4346 || (c < 4888
            ? (c < 4752
              ? (c < 4696
                ? (c < 4682
                  ? (c >= 4348 && c <= 4680)
                  : (c <= 4685 || (c >= 4688 && c <= 4694)))
                : (c <= 4696 || (c < 4704
                  ? (c >= 4698 && c <= 4701)
                  : (c <= 4744 || (c >= 4746 && c <= 4749)))))
              : (c <= 4784 || (c < 4802
                ? (c < 4792
                  ? (c >= 4786 && c <= 4789)
                  : (c <= 4798 || c == 4800))
                : (c <= 4805 || (c < 4824
                  ? (c >= 4808 && c <= 4822)
                  : (c <= 4880 || (c >= 4882 && c <= 4885)))))))
            : (c <= 4954 || (c < 5792
              ? (c < 5024
                ? (c < 4969
                  ? (c >= 4957 && c <= 4959)
                  : (c <= 4988 || (c >= 4992 && c <= 5007)))
                : (c <= 5109 || (c < 5121
                  ? (c >= 5112 && c <= 5117)
                  : (c <= 5740 || (c >= 5743 && c <= 5786)))))
              : (c <= 5866 || (c < 5952
                ? (c < 5888
                  ? (c >= 5870 && c <= 5880)
                  : (c <= 5909 || (c >= 5919 && c <= 5940)))
                : (c <= 5971 || (c < 5998
                  ? (c >= 5984 && c <= 5996)
                  : c <= 6000)))))))))))))
    : (c <= 6003 || (c < 42623
      ? (c < 8455
        ? (c < 7245
          ? (c < 6528
            ? (c < 6176
              ? (c < 6112
                ? (c < 6103
                  ? (c >= 6016 && c <= 6099)
                  : (c <= 6103 || (c >= 6108 && c <= 6109)))
                : (c <= 6121 || (c < 6155
                  ? (c >= 6128 && c <= 6137)
                  : (c <= 6157 || (c >= 6159 && c <= 6169)))))
              : (c <= 6264 || (c < 6432
                ? (c < 6320
                  ? (c >= 6272 && c <= 6314)
                  : (c <= 6389 || (c >= 6400 && c <= 6430)))
                : (c <= 6443 || (c < 6470
                  ? (c >= 6448 && c <= 6459)
                  : (c <= 6509 || (c >= 6512 && c <= 6516)))))))
            : (c <= 6571 || (c < 6823
              ? (c < 6688
                ? (c < 6608
                  ? (c >= 6576 && c <= 6601)
                  : (c <= 6618 || (c >= 6656 && c <= 6683)))
                : (c <= 6750 || (c < 6783
                  ? (c >= 6752 && c <= 6780)
                  : (c <= 6793 || (c >= 6800 && c <= 6809)))))
              : (c <= 6823 || (c < 7019
                ? (c < 6912
                  ? (c >= 6832 && c <= 6862)
                  : (c <= 6988 || (c >= 6992 && c <= 7001)))
                : (c <= 7027 || (c < 7168
                  ? (c >= 7040 && c <= 7155)
                  : (c <= 7223 || (c >= 7232 && c <= 7241)))))))))
          : (c <= 7293 || (c < 8118
            ? (c < 7968
              ? (c < 7376
                ? (c < 7312
                  ? (c >= 7296 && c <= 7304)
                  : (c <= 7354 || (c >= 7357 && c <= 7359)))
                : (c <= 7378 || (c < 7424
                  ? (c >= 7380 && c <= 7418)
                  : (c <= 7957 || (c >= 7960 && c <= 7965)))))
              : (c <= 8005 || (c < 8027
                ? (c < 8016
                  ? (c >= 8008 && c <= 8013)
                  : (c <= 8023 || c == 8025))
                : (c <= 8027 || (c < 8031
                  ? c == 8029
                  : (c <= 8061 || (c >= 8064 && c <= 8116)))))))
            : (c <= 8124 || (c < 8182
              ? (c < 8144
                ? (c < 8130
                  ? c == 8126
                  : (c <= 8132 || (c >= 8134 && c <= 8140)))
                : (c <= 8147 || (c < 8160
                  ? (c >= 8150 && c <= 8155)
                  : (c <= 8172 || (c >= 8178 && c <= 8180)))))
              : (c <= 8188 || (c < 8336
                ? (c < 8308
                  ? (c >= 8304 && c <= 8305)
                  : (c <= 8313 || (c >= 8319 && c <= 8329)))
                : (c <= 8348 || (c < 8450
                  ? (c >= 8400 && c <= 8432)
                  : c <= 8450)))))))))
        : (c <= 8455 || (c < 11728
          ? (c < 11264
            ? (c < 8495
              ? (c < 8484
                ? (c < 8469
                  ? (c >= 8458 && c <= 8467)
                  : (c <= 8469 || (c >= 8473 && c <= 8477)))
                : (c <= 8484 || (c < 8488
                  ? c == 8486
                  : (c <= 8488 || (c >= 8490 && c <= 8493)))))
              : (c <= 8505 || (c < 8528
                ? (c < 8517
                  ? (c >= 8508 && c <= 8511)
                  : (c <= 8521 || c == 8526))
                : (c <= 8585 || (c < 9450
                  ? (c >= 9312 && c <= 9371)
                  : (c <= 9471 || (c >= 10102 && c <= 10131)))))))
            : (c <= 11492 || (c < 11647
              ? (c < 11559
                ? (c < 11517
                  ? (c >= 11499 && c <= 11507)
                  : (c <= 11517 || (c >= 11520 && c <= 11557)))
                : (c <= 11559 || (c < 11568
                  ? c == 11565
                  : (c <= 11623 || c == 11631))))
              : (c <= 11670 || (c < 11704
                ? (c < 11688
                  ? (c >= 11680 && c <= 11686)
                  : (c <= 11694 || (c >= 11696 && c <= 11702)))
                : (c <= 11710 || (c < 11720
                  ? (c >= 11712 && c <= 11718)
                  : c <= 11726)))))))
          : (c <= 11734 || (c < 12704
            ? (c < 12353
              ? (c < 12293
                ? (c < 11744
                  ? (c >= 11736 && c <= 11742)
                  : (c <= 11775 || c == 11823))
                : (c <= 12295 || (c < 12337
                  ? (c >= 12321 && c <= 12335)
                  : (c <= 12341 || (c >= 12344 && c <= 12348)))))
              : (c <= 12438 || (c < 12540
                ? (c < 12445
                  ? (c >= 12441 && c <= 12442)
                  : (c <= 12447 || (c >= 12449 && c <= 12538)))
                : (c <= 12543 || (c < 12593
                  ? (c >= 12549 && c <= 12591)
                  : (c <= 12686 || (c >= 12690 && c <= 12693)))))))
            : (c <= 12735 || (c < 19903
              ? (c < 12881
                ? (c < 12832
                  ? (c >= 12784 && c <= 12799)
                  : (c <= 12841 || (c >= 12872 && c <= 12879)))
                : (c <= 12895 || (c < 12977
                  ? (c >= 12928 && c <= 12937)
                  : (c <= 12991 || c == 13312))))
              : (c <= 19903 || (c < 42512
                ? (c < 42192
                  ? (c >= 19968 && c <= 42124)
                  : (c <= 42237 || (c >= 42240 && c <= 42508)))
                : (c <= 42539 || (c < 42612
                  ? (c >= 42560 && c <= 42610)
                  : c <= 42621)))))))))))
      : (c <= 42737 || (c < 65296
        ? (c < 43793
          ? (c < 43312
            ? (c < 43052
              ? (c < 42960
                ? (c < 42786
                  ? (c >= 42775 && c <= 42783)
                  : (c <= 42888 || (c >= 42891 && c <= 42954)))
                : (c <= 42961 || (c < 42965
                  ? c == 42963
                  : (c <= 42969 || (c >= 42994 && c <= 43047)))))
              : (c <= 43052 || (c < 43216
                ? (c < 43072
                  ? (c >= 43056 && c <= 43061)
                  : (c <= 43123 || (c >= 43136 && c <= 43205)))
                : (c <= 43225 || (c < 43259
                  ? (c >= 43232 && c <= 43255)
                  : (c <= 43259 || (c >= 43261 && c <= 43309)))))))
            : (c <= 43347 || (c < 43616
              ? (c < 43488
                ? (c < 43392
                  ? (c >= 43360 && c <= 43388)
                  : (c <= 43456 || (c >= 43471 && c <= 43481)))
                : (c <= 43518 || (c < 43584
                  ? (c >= 43520 && c <= 43574)
                  : (c <= 43597 || (c >= 43600 && c <= 43609)))))
              : (c <= 43638 || (c < 43762
                ? (c < 43739
                  ? (c >= 43642 && c <= 43714)
                  : (c <= 43741 || (c >= 43744 && c <= 43759)))
                : (c <= 43766 || (c < 43785
                  ? (c >= 43777 && c <= 43782)
                  : c <= 43790)))))))
          : (c <= 43798 || (c < 64285
            ? (c < 44032
              ? (c < 43868
                ? (c < 43816
                  ? (c >= 43808 && c <= 43814)
                  : (c <= 43822 || (c >= 43824 && c <= 43866)))
                : (c <= 43881 || (c < 44012
                  ? (c >= 43888 && c <= 44010)
                  : (c <= 44013 || (c >= 44016 && c <= 44025)))))
              : (c <= 44032 || (c < 63744
                ? (c < 55216
                  ? c == 55203
                  : (c <= 55238 || (c >= 55243 && c <= 55291)))
                : (c <= 64109 || (c < 64256
                  ? (c >= 64112 && c <= 64217)
                  : (c <= 64262 || (c >= 64275 && c <= 64279)))))))
            : (c <= 64296 || (c < 64848
              ? (c < 64320
                ? (c < 64312
                  ? (c >= 64298 && c <= 64310)
                  : (c <= 64316 || c == 64318))
                : (c <= 64321 || (c < 64326
                  ? (c >= 64323 && c <= 64324)
                  : (c <= 64433 || (c >= 64467 && c <= 64829)))))
              : (c <= 64911 || (c < 65056
                ? (c < 65008
                  ? (c >= 64914 && c <= 64967)
                  : (c <= 65019 || (c >= 65024 && c <= 65039)))
                : (c <= 65071 || (c < 65142
                  ? (c >= 65136 && c <= 65140)
                  : c <= 65276)))))))))
        : (c <= 65305 || (c < 66736
          ? (c < 65856
            ? (c < 65536
              ? (c < 65474
                ? (c < 65345
                  ? (c >= 65313 && c <= 65338)
                  : (c <= 65370 || (c >= 65382 && c <= 65470)))
                : (c <= 65479 || (c < 65490
                  ? (c >= 65482 && c <= 65487)
                  : (c <= 65495 || (c >= 65498 && c <= 65500)))))
              : (c <= 65547 || (c < 65599
                ? (c < 65576
                  ? (c >= 65549 && c <= 65574)
                  : (c <= 65594 || (c >= 65596 && c <= 65597)))
                : (c <= 65613 || (c < 65664
                  ? (c >= 65616 && c <= 65629)
                  : (c <= 65786 || (c >= 65799 && c <= 65843)))))))
            : (c <= 65912 || (c < 66384
              ? (c < 66208
                ? (c < 66045
                  ? (c >= 65930 && c <= 65931)
                  : (c <= 66045 || (c >= 66176 && c <= 66204)))
                : (c <= 66256 || (c < 66304
                  ? (c >= 66272 && c <= 66299)
                  : (c <= 66339 || (c >= 66349 && c <= 66378)))))
              : (c <= 66426 || (c < 66513
                ? (c < 66464
                  ? (c >= 66432 && c <= 66461)
                  : (c <= 66499 || (c >= 66504 && c <= 66511)))
                : (c <= 66517 || (c < 66720
                  ? (c >= 66560 && c <= 66717)
                  : c <= 66729)))))))
          : (c <= 66771 || (c < 67463
            ? (c < 66967
              ? (c < 66928
                ? (c < 66816
                  ? (c >= 66776 && c <= 66811)
                  : (c <= 66855 || (c >= 66864 && c <= 66915)))
                : (c <= 66938 || (c < 66956
                  ? (c >= 66940 && c <= 66954)
                  : (c <= 66962 || (c >= 66964 && c <= 66965)))))
              : (c <= 66977 || (c < 67072
                ? (c < 66995
                  ? (c >= 66979 && c <= 66993)
                  : (c <= 67001 || (c >= 67003 && c <= 67004)))
                : (c <= 67382 || (c < 67424
                  ? (c >= 67392 && c <= 67413)
                  : (c <= 67431 || (c >= 67456 && c <= 67461)))))))
            : (c <= 67504 || (c < 67672
              ? (c < 67594
                ? (c < 67584
                  ? (c >= 67506 && c <= 67514)
                  : (c <= 67589 || c == 67592))
                : (c <= 67637 || (c < 67644
                  ? (c >= 67639 && c <= 67640)
                  : (c <= 67644 || (c >= 67647 && c <= 67669)))))
              : (c <= 67702 || (c < 67828
                ? (c < 67751
                  ? (c >= 67705 && c <= 67742)
                  : (c <= 67759 || (c >= 67808 && c <= 67826)))
                : (c <= 67829 || (c < 67872
                  ? (c >= 67835 && c <= 67867)
                  : c <= 67883)))))))))))))));
}

static bool ts_lex(TSLexer *lexer, TSStateId state) {
  START_LEXER();
  eof = lexer->eof(lexer);
  switch (state) {
    case 0:
      if (eof) ADVANCE(24);
      if (lookahead == '\n') ADVANCE(70);
      if (lookahead == '\f') ADVANCE(76);
      if (lookahead == '\r') ADVANCE(8);
      if (lookahead == '"') ADVANCE(39);
      if (lookahead == '#') ADVANCE(46);
      if (lookahead == '(') ADVANCE(36);
      if (lookahead == ')') ADVANCE(38);
      if (lookahead == '+') ADVANCE(59);
      if (lookahead == '-') ADVANCE(60);
      if (lookahead == '.') ADVANCE(50);
      if (lookahead == '/') ADVANCE(88);
      if (lookahead == '0') ADVANCE(66);
      if (lookahead == '1') ADVANCE(67);
      if (lookahead == ';') ADVANCE(28);
      if (lookahead == '=') ADVANCE(35);
      if (lookahead == 'E') ADVANCE(54);
      if (lookahead == '\\') ADVANCE(68);
      if (lookahead == '_') ADVANCE(56);
      if (lookahead == 'e') ADVANCE(52);
      if (lookahead == 'r') ADVANCE(30);
      if (lookahead == '{') ADVANCE(26);
      if (lookahead == '}') ADVANCE(27);
      if (lookahead == 133) ADVANCE(74);
      if (lookahead == 8232) ADVANCE(78);
      if (lookahead == 8233) ADVANCE(80);
      if (lookahead == 65279) ADVANCE(82);
      if (lookahead == '8' ||
          lookahead == '9') ADVANCE(43);
      if (('2' <= lookahead && lookahead <= '7')) ADVANCE(43);
      if (('A' <= lookahead && lookahead <= 'F') ||
          ('a' <= lookahead && lookahead <= 'f')) ADVANCE(32);
      if (lookahead == '\t' ||
          lookahead == ' ' ||
          lookahead == 160 ||
          lookahead == 5760 ||
          (8192 <= lookahead && lookahead <= 8202) ||
          lookahead == 8239 ||
          lookahead == 8287 ||
          lookahead == 12288) ADVANCE(84);
      if (sym__normal_bare_identifier_character_set_1(lookahead)) ADVANCE(32);
      if (lookahead != 0) ADVANCE(87);
      END_STATE();
    case 1:
      if (lookahead == '\n') ADVANCE(70);
      if (lookahead == '\f') ADVANCE(76);
      if (lookahead == '\r') ADVANCE(8);
      if (lookahead == '#') ADVANCE(45);
      if (lookahead == ')') ADVANCE(38);
      if (lookahead == '.') ADVANCE(49);
      if (lookahead == '/') ADVANCE(13);
      if (lookahead == ';') ADVANCE(28);
      if (lookahead == '=') ADVANCE(35);
      if (lookahead == 'E') ADVANCE(53);
      if (lookahead == '\\') ADVANCE(68);
      if (lookahead == '_') ADVANCE(55);
      if (lookahead == 'e') ADVANCE(51);
      if (lookahead == '{') ADVANCE(26);
      if (lookahead == 133) ADVANCE(74);
      if (lookahead == 8232) ADVANCE(78);
      if (lookahead == 8233) ADVANCE(80);
      if (lookahead == 65279) ADVANCE(82);
      if (('0' <= lookahead && lookahead <= '9')) ADVANCE(57);
      if (lookahead == '\t' ||
          lookahead == ' ' ||
          lookahead == 160 ||
          lookahead == 5760 ||
          (8192 <= lookahead && lookahead <= 8202) ||
          lookahead == 8239 ||
          lookahead == 8287 ||
          lookahead == 12288) ADVANCE(84);
      END_STATE();
    case 2:
      if (lookahead == '\n') ADVANCE(70);
      if (lookahead == '\f') ADVANCE(76);
      if (lookahead == '\r') ADVANCE(8);
      if (lookahead == ')') ADVANCE(38);
      if (lookahead == '/') ADVANCE(13);
      if (lookahead == '0') ADVANCE(58);
      if (lookahead == ';') ADVANCE(28);
      if (lookahead == '=') ADVANCE(35);
      if (lookahead == '\\') ADVANCE(68);
      if (lookahead == '{') ADVANCE(26);
      if (lookahead == 133) ADVANCE(74);
      if (lookahead == 8232) ADVANCE(78);
      if (lookahead == 8233) ADVANCE(80);
      if (lookahead == 65279) ADVANCE(82);
      if (('1' <= lookahead && lookahead <= '9')) ADVANCE(57);
      if (lookahead == '\t' ||
          lookahead == ' ' ||
          lookahead == 160 ||
          lookahead == 5760 ||
          (8192 <= lookahead && lookahead <= 8202) ||
          lookahead == 8239 ||
          lookahead == 8287 ||
          lookahead == 12288) ADVANCE(84);
      if (sym___identifier_char_no_digit_character_set_1(lookahead)) ADVANCE(34);
      END_STATE();
    case 3:
      if (lookahead == '\n') ADVANCE(70);
      if (lookahead == '\f') ADVANCE(76);
      if (lookahead == '\r') ADVANCE(8);
      if (lookahead == ')') ADVANCE(38);
      if (lookahead == '/') ADVANCE(13);
      if (lookahead == ';') ADVANCE(28);
      if (lookahead == '=') ADVANCE(35);
      if (lookahead == '\\') ADVANCE(68);
      if (lookahead == '{') ADVANCE(26);
      if (lookahead == 133) ADVANCE(74);
      if (lookahead == 8232) ADVANCE(78);
      if (lookahead == 8233) ADVANCE(80);
      if (lookahead == 65279) ADVANCE(82);
      if (lookahead == '\t' ||
          lookahead == ' ' ||
          lookahead == 160 ||
          lookahead == 5760 ||
          (8192 <= lookahead && lookahead <= 8202) ||
          lookahead == 8239 ||
          lookahead == 8287 ||
          lookahead == 12288) ADVANCE(84);
      if (sym___identifier_char_no_digit_character_set_1(lookahead)) ADVANCE(33);
      END_STATE();
    case 4:
      if (lookahead == '\n') ADVANCE(70);
      if (lookahead == '\f') ADVANCE(76);
      if (lookahead == '\r') ADVANCE(8);
      if (lookahead == '/') ADVANCE(13);
      if (lookahead == '0') ADVANCE(65);
      if (lookahead == '1') ADVANCE(67);
      if (lookahead == ';') ADVANCE(28);
      if (lookahead == '\\') ADVANCE(68);
      if (lookahead == '_') ADVANCE(55);
      if (lookahead == '{') ADVANCE(26);
      if (lookahead == 133) ADVANCE(74);
      if (lookahead == 8232) ADVANCE(78);
      if (lookahead == 8233) ADVANCE(80);
      if (lookahead == 65279) ADVANCE(82);
      if (lookahead == '\t' ||
          lookahead == ' ' ||
          lookahead == 160 ||
          lookahead == 5760 ||
          (8192 <= lookahead && lookahead <= 8202) ||
          lookahead == 8239 ||
          lookahead == 8287 ||
          lookahead == 12288) ADVANCE(84);
      END_STATE();
    case 5:
      if (lookahead == '\n') ADVANCE(70);
      if (lookahead == '\f') ADVANCE(76);
      if (lookahead == '\r') ADVANCE(8);
      if (lookahead == '/') ADVANCE(13);
      if (lookahead == ';') ADVANCE(28);
      if (lookahead == '\\') ADVANCE(68);
      if (lookahead == '_') ADVANCE(55);
      if (lookahead == '{') ADVANCE(26);
      if (lookahead == 133) ADVANCE(74);
      if (lookahead == 8232) ADVANCE(78);
      if (lookahead == 8233) ADVANCE(80);
      if (lookahead == 65279) ADVANCE(82);
      if (('0' <= lookahead && lookahead <= '7')) ADVANCE(63);
      if (lookahead == '\t' ||
          lookahead == ' ' ||
          lookahead == 160 ||
          lookahead == 5760 ||
          (8192 <= lookahead && lookahead <= 8202) ||
          lookahead == 8239 ||
          lookahead == 8287 ||
          lookahead == 12288) ADVANCE(84);
      END_STATE();
    case 6:
      if (lookahead == '\n') ADVANCE(70);
      if (lookahead == '\f') ADVANCE(76);
      if (lookahead == '\r') ADVANCE(8);
      if (lookahead == '/') ADVANCE(13);
      if (lookahead == ';') ADVANCE(28);
      if (lookahead == '\\') ADVANCE(68);
      if (lookahead == '_') ADVANCE(55);
      if (lookahead == '{') ADVANCE(26);
      if (lookahead == 133) ADVANCE(74);
      if (lookahead == 8232) ADVANCE(78);
      if (lookahead == 8233) ADVANCE(80);
      if (lookahead == 65279) ADVANCE(82);
      if (lookahead == '\t' ||
          lookahead == ' ' ||
          lookahead == 160 ||
          lookahead == 5760 ||
          (8192 <= lookahead && lookahead <= 8202) ||
          lookahead == 8239 ||
          lookahead == 8287 ||
          lookahead == 12288) ADVANCE(84);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'F') ||
          ('a' <= lookahead && lookahead <= 'f')) ADVANCE(43);
      END_STATE();
    case 7:
      if (lookahead == '\n') ADVANCE(70);
      if (lookahead == '\f') ADVANCE(76);
      if (lookahead == '\r') ADVANCE(8);
      if (lookahead == 133) ADVANCE(74);
      if (lookahead == 8232) ADVANCE(78);
      if (lookahead == 8233) ADVANCE(80);
      if (lookahead != 0) ADVANCE(87);
      END_STATE();
    case 8:
      if (lookahead == '\n') ADVANCE(72);
      if (lookahead == '\'') ADVANCE(69);
      END_STATE();
    case 9:
      if (lookahead == '\n') ADVANCE(71);
      if (lookahead == '\f') ADVANCE(77);
      if (lookahead == '\r') ADVANCE(10);
      if (lookahead == '"') ADVANCE(39);
      if (lookahead == '\'') ADVANCE(32);
      if (lookahead == '(') ADVANCE(37);
      if (lookahead == '+') ADVANCE(94);
      if (lookahead == '-') ADVANCE(94);
      if (lookahead == '/') ADVANCE(92);
      if (lookahead == 'r') ADVANCE(29);
      if (lookahead == '}') ADVANCE(27);
      if (lookahead == 133) ADVANCE(75);
      if (lookahead == 8232) ADVANCE(79);
      if (lookahead == 8233) ADVANCE(81);
      if (lookahead == 65279) ADVANCE(83);
      if (lookahead == '!' ||
          lookahead == '*') ADVANCE(93);
      if (('<' <= lookahead && lookahead <= '>') ||
          lookahead == '[' ||
          lookahead == ']') ADVANCE(94);
      if (lookahead == '\t' ||
          lookahead == ' ' ||
          lookahead == 160 ||
          lookahead == 5760 ||
          (8192 <= lookahead && lookahead <= 8202) ||
          lookahead == 8239 ||
          lookahead == 8287 ||
          lookahead == 12288) ADVANCE(85);
      if (sym__normal_bare_identifier_character_set_2(lookahead)) ADVANCE(31);
      if (lookahead != 0 &&
          lookahead != '{') ADVANCE(21);
      END_STATE();
    case 10:
      if (lookahead == '\n') ADVANCE(73);
      if (lookahead == '\'') ADVANCE(69);
      if (lookahead == '!' ||
          lookahead == '*' ||
          lookahead == '+' ||
          lookahead == '-' ||
          lookahead == '/' ||
          ('<' <= lookahead && lookahead <= '>') ||
          lookahead == '[' ||
          lookahead == ']') ADVANCE(94);
      if (lookahead != 0 &&
          lookahead != '"' &&
          lookahead != '{' &&
          lookahead != '}') ADVANCE(21);
      END_STATE();
    case 11:
      if (lookahead == '\n') ADVANCE(91);
      if (lookahead == '\f') ADVANCE(91);
      if (lookahead == '\r') ADVANCE(89);
      if (lookahead == '/') ADVANCE(90);
      if (lookahead == '}') ADVANCE(27);
      if (lookahead == 133) ADVANCE(91);
      if (lookahead == 8232) ADVANCE(91);
      if (lookahead == 8233) ADVANCE(91);
      if (lookahead == 65279) ADVANCE(91);
      if (lookahead == '\t' ||
          lookahead == ' ' ||
          lookahead == 160 ||
          lookahead == 5760 ||
          (8192 <= lookahead && lookahead <= 8202) ||
          lookahead == 8239 ||
          lookahead == 8287 ||
          lookahead == 12288) ADVANCE(91);
      if (lookahead != 0 &&
          lookahead != '"' &&
          lookahead != '\'' &&
          lookahead != '{') ADVANCE(91);
      END_STATE();
    case 12:
      if (lookahead == '"') ADVANCE(39);
      if (lookahead == '\\') ADVANCE(41);
      if (lookahead != 0) ADVANCE(40);
      END_STATE();
    case 13:
      if (lookahead == '-') ADVANCE(25);
      if (lookahead == '/') ADVANCE(86);
      END_STATE();
    case 14:
      if (lookahead == '{') ADVANCE(22);
      END_STATE();
    case 15:
      if (lookahead == '}') ADVANCE(42);
      END_STATE();
    case 16:
      if (lookahead == '}') ADVANCE(42);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'F') ||
          ('a' <= lookahead && lookahead <= 'f')) ADVANCE(15);
      END_STATE();
    case 17:
      if (lookahead == '}') ADVANCE(42);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'F') ||
          ('a' <= lookahead && lookahead <= 'f')) ADVANCE(16);
      END_STATE();
    case 18:
      if (lookahead == '}') ADVANCE(42);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'F') ||
          ('a' <= lookahead && lookahead <= 'f')) ADVANCE(17);
      END_STATE();
    case 19:
      if (lookahead == '}') ADVANCE(42);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'F') ||
          ('a' <= lookahead && lookahead <= 'f')) ADVANCE(18);
      END_STATE();
    case 20:
      if (lookahead == '}') ADVANCE(42);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'F') ||
          ('a' <= lookahead && lookahead <= 'f')) ADVANCE(19);
      END_STATE();
    case 21:
      if (lookahead == '!' ||
          lookahead == '*' ||
          lookahead == '+' ||
          lookahead == '-' ||
          lookahead == '/' ||
          ('<' <= lookahead && lookahead <= '>') ||
          lookahead == '[' ||
          lookahead == ']') ADVANCE(94);
      if (lookahead != 0 &&
          lookahead != '"' &&
          lookahead != '\'' &&
          lookahead != '{' &&
          lookahead != '}') ADVANCE(21);
      END_STATE();
    case 22:
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'F') ||
          ('a' <= lookahead && lookahead <= 'f')) ADVANCE(20);
      END_STATE();
    case 23:
      if (eof) ADVANCE(24);
      if (lookahead == '\n') ADVANCE(70);
      if (lookahead == '\f') ADVANCE(76);
      if (lookahead == '\r') ADVANCE(8);
      if (lookahead == '"') ADVANCE(39);
      if (lookahead == '(') ADVANCE(36);
      if (lookahead == ')') ADVANCE(38);
      if (lookahead == '+') ADVANCE(59);
      if (lookahead == '-') ADVANCE(60);
      if (lookahead == '/') ADVANCE(13);
      if (lookahead == '0') ADVANCE(58);
      if (lookahead == ';') ADVANCE(28);
      if (lookahead == '=') ADVANCE(35);
      if (lookahead == '\\') ADVANCE(68);
      if (lookahead == 'r') ADVANCE(30);
      if (lookahead == '{') ADVANCE(26);
      if (lookahead == '}') ADVANCE(27);
      if (lookahead == 133) ADVANCE(74);
      if (lookahead == 8232) ADVANCE(78);
      if (lookahead == 8233) ADVANCE(80);
      if (lookahead == 65279) ADVANCE(82);
      if (('1' <= lookahead && lookahead <= '9')) ADVANCE(57);
      if (lookahead == '\t' ||
          lookahead == ' ' ||
          lookahead == 160 ||
          lookahead == 5760 ||
          (8192 <= lookahead && lookahead <= 8202) ||
          lookahead == 8239 ||
          lookahead == 8287 ||
          lookahead == 12288) ADVANCE(84);
      if (sym__normal_bare_identifier_character_set_3(lookahead)) ADVANCE(32);
      END_STATE();
    case 24:
      ACCEPT_TOKEN(ts_builtin_sym_end);
      END_STATE();
    case 25:
      ACCEPT_TOKEN(anon_sym_SLASH_DASH);
      END_STATE();
    case 26:
      ACCEPT_TOKEN(anon_sym_LBRACE);
      END_STATE();
    case 27:
      ACCEPT_TOKEN(anon_sym_RBRACE);
      END_STATE();
    case 28:
      ACCEPT_TOKEN(anon_sym_SEMI);
      END_STATE();
    case 29:
      ACCEPT_TOKEN(sym__normal_bare_identifier);
      if (lookahead == '"') ADVANCE(47);
      if (lookahead == '\'') ADVANCE(32);
      if (lookahead == '!' ||
          lookahead == '*' ||
          lookahead == '+' ||
          lookahead == '-') ADVANCE(93);
      if (lookahead == '/' ||
          ('<' <= lookahead && lookahead <= '>') ||
          lookahead == '[' ||
          lookahead == ']') ADVANCE(94);
      if (sym__normal_bare_identifier_character_set_4(lookahead)) ADVANCE(31);
      if (lookahead != 0 &&
          (lookahead < '{' || '}' < lookahead)) ADVANCE(21);
      END_STATE();
    case 30:
      ACCEPT_TOKEN(sym__normal_bare_identifier);
      if (lookahead == '"') ADVANCE(47);
      if (sym__normal_bare_identifier_character_set_5(lookahead)) ADVANCE(32);
      END_STATE();
    case 31:
      ACCEPT_TOKEN(sym__normal_bare_identifier);
      if (lookahead == '\'') ADVANCE(32);
      if (lookahead == '!' ||
          lookahead == '*' ||
          lookahead == '+' ||
          lookahead == '-') ADVANCE(93);
      if (lookahead == '/' ||
          ('<' <= lookahead && lookahead <= '>') ||
          lookahead == '[' ||
          lookahead == ']') ADVANCE(94);
      if (sym__normal_bare_identifier_character_set_4(lookahead)) ADVANCE(31);
      if (lookahead != 0 &&
          lookahead != '"' &&
          (lookahead < '{' || '}' < lookahead)) ADVANCE(21);
      END_STATE();
    case 32:
      ACCEPT_TOKEN(sym__normal_bare_identifier);
      if (sym__normal_bare_identifier_character_set_6(lookahead)) ADVANCE(32);
      END_STATE();
    case 33:
      ACCEPT_TOKEN(sym__identifier_char);
      END_STATE();
    case 34:
      ACCEPT_TOKEN(sym___identifier_char_no_digit);
      END_STATE();
    case 35:
      ACCEPT_TOKEN(anon_sym_EQ);
      END_STATE();
    case 36:
      ACCEPT_TOKEN(anon_sym_LPAREN);
      END_STATE();
    case 37:
      ACCEPT_TOKEN(anon_sym_LPAREN);
      if (lookahead == '!' ||
          lookahead == '*' ||
          lookahead == '+' ||
          lookahead == '-' ||
          lookahead == '/' ||
          ('<' <= lookahead && lookahead <= '>') ||
          lookahead == '[' ||
          lookahead == ']') ADVANCE(94);
      if (lookahead != 0 &&
          lookahead != '"' &&
          lookahead != '\'' &&
          lookahead != '{' &&
          lookahead != '}') ADVANCE(21);
      END_STATE();
    case 38:
      ACCEPT_TOKEN(anon_sym_RPAREN);
      END_STATE();
    case 39:
      ACCEPT_TOKEN(anon_sym_DQUOTE);
      END_STATE();
    case 40:
      ACCEPT_TOKEN(aux_sym__escaped_string_token1);
      END_STATE();
    case 41:
      ACCEPT_TOKEN(aux_sym__escaped_string_token1);
      if (lookahead == '"' ||
          lookahead == '/' ||
          lookahead == '\\' ||
          lookahead == 'b' ||
          lookahead == 'f' ||
          lookahead == 'n' ||
          lookahead == 'r' ||
          lookahead == 't') ADVANCE(42);
      if (lookahead == 'u') ADVANCE(14);
      END_STATE();
    case 42:
      ACCEPT_TOKEN(sym_escape);
      END_STATE();
    case 43:
      ACCEPT_TOKEN(sym__hex_digit);
      END_STATE();
    case 44:
      ACCEPT_TOKEN(aux_sym__raw_string_token2);
      if (lookahead != 0 &&
          lookahead != '#') ADVANCE(44);
      END_STATE();
    case 45:
      ACCEPT_TOKEN(anon_sym_POUND);
      END_STATE();
    case 46:
      ACCEPT_TOKEN(anon_sym_POUND);
      if (sym__normal_bare_identifier_character_set_6(lookahead)) ADVANCE(32);
      END_STATE();
    case 47:
      ACCEPT_TOKEN(aux_sym__raw_string_token3);
      END_STATE();
    case 48:
      ACCEPT_TOKEN(aux_sym__raw_string_token4);
      if (lookahead != 0 &&
          lookahead != '"') ADVANCE(48);
      END_STATE();
    case 49:
      ACCEPT_TOKEN(anon_sym_DOT);
      END_STATE();
    case 50:
      ACCEPT_TOKEN(anon_sym_DOT);
      if (sym__normal_bare_identifier_character_set_6(lookahead)) ADVANCE(32);
      END_STATE();
    case 51:
      ACCEPT_TOKEN(anon_sym_e);
      END_STATE();
    case 52:
      ACCEPT_TOKEN(anon_sym_e);
      if (sym__normal_bare_identifier_character_set_6(lookahead)) ADVANCE(32);
      END_STATE();
    case 53:
      ACCEPT_TOKEN(anon_sym_E);
      END_STATE();
    case 54:
      ACCEPT_TOKEN(anon_sym_E);
      if (sym__normal_bare_identifier_character_set_6(lookahead)) ADVANCE(32);
      END_STATE();
    case 55:
      ACCEPT_TOKEN(anon_sym__);
      END_STATE();
    case 56:
      ACCEPT_TOKEN(anon_sym__);
      if (sym__normal_bare_identifier_character_set_6(lookahead)) ADVANCE(32);
      END_STATE();
    case 57:
      ACCEPT_TOKEN(sym__digit);
      END_STATE();
    case 58:
      ACCEPT_TOKEN(sym__digit);
      if (lookahead == 'b') ADVANCE(64);
      if (lookahead == 'o') ADVANCE(62);
      if (lookahead == 'x') ADVANCE(61);
      END_STATE();
    case 59:
      ACCEPT_TOKEN(anon_sym_PLUS);
      END_STATE();
    case 60:
      ACCEPT_TOKEN(anon_sym_DASH);
      END_STATE();
    case 61:
      ACCEPT_TOKEN(anon_sym_0x);
      END_STATE();
    case 62:
      ACCEPT_TOKEN(anon_sym_0o);
      END_STATE();
    case 63:
      ACCEPT_TOKEN(aux_sym__octal_token1);
      END_STATE();
    case 64:
      ACCEPT_TOKEN(anon_sym_0b);
      END_STATE();
    case 65:
      ACCEPT_TOKEN(anon_sym_0);
      END_STATE();
    case 66:
      ACCEPT_TOKEN(anon_sym_0);
      if (lookahead == 'o') ADVANCE(62);
      if (lookahead == 'x') ADVANCE(61);
      END_STATE();
    case 67:
      ACCEPT_TOKEN(anon_sym_1);
      END_STATE();
    case 68:
      ACCEPT_TOKEN(anon_sym_BSLASH);
      END_STATE();
    case 69:
      ACCEPT_TOKEN(aux_sym__newline_token1);
      END_STATE();
    case 70:
      ACCEPT_TOKEN(aux_sym__newline_token2);
      END_STATE();
    case 71:
      ACCEPT_TOKEN(aux_sym__newline_token2);
      if (lookahead == '!' ||
          lookahead == '*' ||
          lookahead == '+' ||
          lookahead == '-' ||
          lookahead == '/' ||
          ('<' <= lookahead && lookahead <= '>') ||
          lookahead == '[' ||
          lookahead == ']') ADVANCE(94);
      if (lookahead != 0 &&
          lookahead != '"' &&
          lookahead != '\'' &&
          lookahead != '{' &&
          lookahead != '}') ADVANCE(21);
      END_STATE();
    case 72:
      ACCEPT_TOKEN(aux_sym__newline_token3);
      END_STATE();
    case 73:
      ACCEPT_TOKEN(aux_sym__newline_token3);
      if (lookahead == '!' ||
          lookahead == '*' ||
          lookahead == '+' ||
          lookahead == '-' ||
          lookahead == '/' ||
          ('<' <= lookahead && lookahead <= '>') ||
          lookahead == '[' ||
          lookahead == ']') ADVANCE(94);
      if (lookahead != 0 &&
          lookahead != '"' &&
          lookahead != '\'' &&
          lookahead != '{' &&
          lookahead != '}') ADVANCE(21);
      END_STATE();
    case 74:
      ACCEPT_TOKEN(aux_sym__newline_token4);
      END_STATE();
    case 75:
      ACCEPT_TOKEN(aux_sym__newline_token4);
      if (lookahead == '!' ||
          lookahead == '*' ||
          lookahead == '+' ||
          lookahead == '-' ||
          lookahead == '/' ||
          ('<' <= lookahead && lookahead <= '>') ||
          lookahead == '[' ||
          lookahead == ']') ADVANCE(94);
      if (lookahead != 0 &&
          lookahead != '"' &&
          lookahead != '\'' &&
          lookahead != '{' &&
          lookahead != '}') ADVANCE(21);
      END_STATE();
    case 76:
      ACCEPT_TOKEN(aux_sym__newline_token5);
      END_STATE();
    case 77:
      ACCEPT_TOKEN(aux_sym__newline_token5);
      if (lookahead == '!' ||
          lookahead == '*' ||
          lookahead == '+' ||
          lookahead == '-' ||
          lookahead == '/' ||
          ('<' <= lookahead && lookahead <= '>') ||
          lookahead == '[' ||
          lookahead == ']') ADVANCE(94);
      if (lookahead != 0 &&
          lookahead != '"' &&
          lookahead != '\'' &&
          lookahead != '{' &&
          lookahead != '}') ADVANCE(21);
      END_STATE();
    case 78:
      ACCEPT_TOKEN(aux_sym__newline_token6);
      END_STATE();
    case 79:
      ACCEPT_TOKEN(aux_sym__newline_token6);
      if (lookahead == '!' ||
          lookahead == '*' ||
          lookahead == '+' ||
          lookahead == '-' ||
          lookahead == '/' ||
          ('<' <= lookahead && lookahead <= '>') ||
          lookahead == '[' ||
          lookahead == ']') ADVANCE(94);
      if (lookahead != 0 &&
          lookahead != '"' &&
          lookahead != '\'' &&
          lookahead != '{' &&
          lookahead != '}') ADVANCE(21);
      END_STATE();
    case 80:
      ACCEPT_TOKEN(aux_sym__newline_token7);
      END_STATE();
    case 81:
      ACCEPT_TOKEN(aux_sym__newline_token7);
      if (lookahead == '!' ||
          lookahead == '*' ||
          lookahead == '+' ||
          lookahead == '-' ||
          lookahead == '/' ||
          ('<' <= lookahead && lookahead <= '>') ||
          lookahead == '[' ||
          lookahead == ']') ADVANCE(94);
      if (lookahead != 0 &&
          lookahead != '"' &&
          lookahead != '\'' &&
          lookahead != '{' &&
          lookahead != '}') ADVANCE(21);
      END_STATE();
    case 82:
      ACCEPT_TOKEN(sym__bom);
      END_STATE();
    case 83:
      ACCEPT_TOKEN(sym__bom);
      if (lookahead == '!' ||
          lookahead == '*' ||
          lookahead == '+' ||
          lookahead == '-' ||
          lookahead == '/' ||
          ('<' <= lookahead && lookahead <= '>') ||
          lookahead == '[' ||
          lookahead == ']') ADVANCE(94);
      if (lookahead != 0 &&
          lookahead != '"' &&
          lookahead != '\'' &&
          lookahead != '{' &&
          lookahead != '}') ADVANCE(21);
      END_STATE();
    case 84:
      ACCEPT_TOKEN(sym__unicode_space);
      END_STATE();
    case 85:
      ACCEPT_TOKEN(sym__unicode_space);
      if (lookahead == '!' ||
          lookahead == '*' ||
          lookahead == '+' ||
          lookahead == '-' ||
          lookahead == '/' ||
          ('<' <= lookahead && lookahead <= '>') ||
          lookahead == '[' ||
          lookahead == ']') ADVANCE(94);
      if (lookahead != 0 &&
          lookahead != '"' &&
          lookahead != '\'' &&
          lookahead != '{' &&
          lookahead != '}') ADVANCE(21);
      END_STATE();
    case 86:
      ACCEPT_TOKEN(anon_sym_SLASH_SLASH);
      END_STATE();
    case 87:
      ACCEPT_TOKEN(aux_sym_single_line_comment_token1);
      END_STATE();
    case 88:
      ACCEPT_TOKEN(aux_sym_single_line_comment_token1);
      if (lookahead == '-') ADVANCE(25);
      if (lookahead == '/') ADVANCE(86);
      END_STATE();
    case 89:
      ACCEPT_TOKEN(sym_arco_math_text);
      if (lookahead == '\n') ADVANCE(91);
      if (lookahead != 0 &&
          lookahead != '"' &&
          lookahead != '\'' &&
          lookahead != '{' &&
          lookahead != '}') ADVANCE(91);
      END_STATE();
    case 90:
      ACCEPT_TOKEN(sym_arco_math_text);
      if (lookahead == '/') ADVANCE(91);
      if (lookahead != 0 &&
          lookahead != '"' &&
          lookahead != '\'' &&
          lookahead != '{' &&
          lookahead != '}') ADVANCE(91);
      END_STATE();
    case 91:
      ACCEPT_TOKEN(sym_arco_math_text);
      if (lookahead != 0 &&
          lookahead != '"' &&
          lookahead != '\'' &&
          lookahead != '{' &&
          lookahead != '}') ADVANCE(91);
      END_STATE();
    case 92:
      ACCEPT_TOKEN(sym_arco_constraint_math_text);
      if (lookahead == '-') ADVANCE(94);
      if (lookahead == '/') ADVANCE(94);
      if (lookahead == '!' ||
          lookahead == '*' ||
          lookahead == '+' ||
          ('<' <= lookahead && lookahead <= '>') ||
          lookahead == '[' ||
          lookahead == ']') ADVANCE(94);
      if (lookahead != 0 &&
          lookahead != '"' &&
          lookahead != '\'' &&
          lookahead != '{' &&
          lookahead != '}') ADVANCE(94);
      END_STATE();
    case 93:
      ACCEPT_TOKEN(sym_arco_constraint_math_text);
      if (lookahead == '!' ||
          lookahead == '*' ||
          lookahead == '+' ||
          lookahead == '-') ADVANCE(93);
      if (lookahead == '/' ||
          ('<' <= lookahead && lookahead <= '>') ||
          lookahead == '[' ||
          lookahead == ']') ADVANCE(94);
      if (sym__normal_bare_identifier_character_set_4(lookahead)) ADVANCE(93);
      if (lookahead != 0 &&
          (lookahead < '"' || '\'' < lookahead) &&
          (lookahead < '{' || '}' < lookahead)) ADVANCE(94);
      END_STATE();
    case 94:
      ACCEPT_TOKEN(sym_arco_constraint_math_text);
      if (lookahead == '!' ||
          lookahead == '*' ||
          lookahead == '+' ||
          lookahead == '-' ||
          lookahead == '/' ||
          ('<' <= lookahead && lookahead <= '>') ||
          lookahead == '[' ||
          lookahead == ']') ADVANCE(94);
      if (lookahead != 0 &&
          lookahead != '"' &&
          lookahead != '\'' &&
          lookahead != '{' &&
          lookahead != '}') ADVANCE(94);
      END_STATE();
    default:
      return false;
  }
}

static bool ts_lex_keywords(TSLexer *lexer, TSStateId state) {
  START_LEXER();
  eof = lexer->eof(lexer);
  switch (state) {
    case 0:
      if (lookahead == 'b') ADVANCE(1);
      if (lookahead == 'c') ADVANCE(2);
      if (lookahead == 'd') ADVANCE(3);
      if (lookahead == 'e') ADVANCE(4);
      if (lookahead == 'f') ADVANCE(5);
      if (lookahead == 'h') ADVANCE(6);
      if (lookahead == 'i') ADVANCE(7);
      if (lookahead == 'l') ADVANCE(8);
      if (lookahead == 'm') ADVANCE(9);
      if (lookahead == 'n') ADVANCE(10);
      if (lookahead == 'r') ADVANCE(11);
      if (lookahead == 't') ADVANCE(12);
      if (lookahead == 'u') ADVANCE(13);
      END_STATE();
    case 1:
      if (lookahead == 'a') ADVANCE(14);
      END_STATE();
    case 2:
      if (lookahead == 'o') ADVANCE(15);
      if (lookahead == 'u') ADVANCE(16);
      END_STATE();
    case 3:
      if (lookahead == 'a') ADVANCE(17);
      if (lookahead == 'e') ADVANCE(18);
      if (lookahead == 'u') ADVANCE(19);
      END_STATE();
    case 4:
      if (lookahead == 'm') ADVANCE(20);
      if (lookahead == 'x') ADVANCE(21);
      END_STATE();
    case 5:
      if (lookahead == '3') ADVANCE(22);
      if (lookahead == '6') ADVANCE(23);
      if (lookahead == 'a') ADVANCE(24);
      if (lookahead == 'i') ADVANCE(25);
      END_STATE();
    case 6:
      if (lookahead == 'o') ADVANCE(26);
      END_STATE();
    case 7:
      if (lookahead == '1') ADVANCE(27);
      if (lookahead == '3') ADVANCE(28);
      if (lookahead == '6') ADVANCE(29);
      if (lookahead == '8') ADVANCE(30);
      if (lookahead == 'd') ADVANCE(31);
      if (lookahead == 'f') ADVANCE(32);
      if (lookahead == 'p') ADVANCE(33);
      if (lookahead == 'r') ADVANCE(34);
      if (lookahead == 's') ADVANCE(35);
      END_STATE();
    case 8:
      if (lookahead == 'o') ADVANCE(36);
      END_STATE();
    case 9:
      if (lookahead == 'a') ADVANCE(37);
      if (lookahead == 'i') ADVANCE(38);
      END_STATE();
    case 10:
      if (lookahead == 'u') ADVANCE(39);
      END_STATE();
    case 11:
      if (lookahead == '#') ADVANCE(40);
      if (lookahead == 'e') ADVANCE(41);
      END_STATE();
    case 12:
      if (lookahead == 'i') ADVANCE(42);
      if (lookahead == 'r') ADVANCE(43);
      END_STATE();
    case 13:
      if (lookahead == '1') ADVANCE(44);
      if (lookahead == '3') ADVANCE(45);
      if (lookahead == '6') ADVANCE(46);
      if (lookahead == '8') ADVANCE(47);
      if (lookahead == 'p') ADVANCE(48);
      if (lookahead == 'r') ADVANCE(49);
      if (lookahead == 's') ADVANCE(50);
      if (lookahead == 'u') ADVANCE(51);
      END_STATE();
    case 14:
      if (lookahead == 's') ADVANCE(52);
      END_STATE();
    case 15:
      if (lookahead == 'n') ADVANCE(53);
      if (lookahead == 'u') ADVANCE(54);
      END_STATE();
    case 16:
      if (lookahead == 'r') ADVANCE(55);
      END_STATE();
    case 17:
      if (lookahead == 't') ADVANCE(56);
      END_STATE();
    case 18:
      if (lookahead == 'c') ADVANCE(57);
      END_STATE();
    case 19:
      if (lookahead == 'r') ADVANCE(58);
      END_STATE();
    case 20:
      if (lookahead == 'a') ADVANCE(59);
      END_STATE();
    case 21:
      if (lookahead == 'p') ADVANCE(60);
      END_STATE();
    case 22:
      if (lookahead == '2') ADVANCE(61);
      END_STATE();
    case 23:
      if (lookahead == '4') ADVANCE(62);
      END_STATE();
    case 24:
      if (lookahead == 'l') ADVANCE(63);
      END_STATE();
    case 25:
      if (lookahead == 'l') ADVANCE(64);
      END_STATE();
    case 26:
      if (lookahead == 's') ADVANCE(65);
      END_STATE();
    case 27:
      if (lookahead == '6') ADVANCE(66);
      END_STATE();
    case 28:
      if (lookahead == '2') ADVANCE(67);
      END_STATE();
    case 29:
      if (lookahead == '4') ADVANCE(68);
      END_STATE();
    case 30:
      ACCEPT_TOKEN(anon_sym_i8);
      END_STATE();
    case 31:
      if (lookahead == 'n') ADVANCE(69);
      END_STATE();
    case 32:
      ACCEPT_TOKEN(anon_sym_if);
      END_STATE();
    case 33:
      if (lookahead == 'v') ADVANCE(70);
      END_STATE();
    case 34:
      if (lookahead == 'i') ADVANCE(71);
      if (lookahead == 'l') ADVANCE(72);
      END_STATE();
    case 35:
      if (lookahead == 'i') ADVANCE(73);
      END_STATE();
    case 36:
      if (lookahead == 'w') ADVANCE(74);
      END_STATE();
    case 37:
      if (lookahead == 'x') ADVANCE(75);
      END_STATE();
    case 38:
      if (lookahead == 'n') ADVANCE(76);
      END_STATE();
    case 39:
      if (lookahead == 'l') ADVANCE(77);
      END_STATE();
    case 40:
      ACCEPT_TOKEN(aux_sym__raw_string_token1);
      if (lookahead == '#') ADVANCE(40);
      END_STATE();
    case 41:
      if (lookahead == 'g') ADVANCE(78);
      END_STATE();
    case 42:
      if (lookahead == 'm') ADVANCE(79);
      END_STATE();
    case 43:
      if (lookahead == 'u') ADVANCE(80);
      END_STATE();
    case 44:
      if (lookahead == '6') ADVANCE(81);
      END_STATE();
    case 45:
      if (lookahead == '2') ADVANCE(82);
      END_STATE();
    case 46:
      if (lookahead == '4') ADVANCE(83);
      END_STATE();
    case 47:
      ACCEPT_TOKEN(anon_sym_u8);
      END_STATE();
    case 48:
      if (lookahead == 'p') ADVANCE(84);
      END_STATE();
    case 49:
      if (lookahead == 'l') ADVANCE(85);
      END_STATE();
    case 50:
      if (lookahead == 'i') ADVANCE(86);
      END_STATE();
    case 51:
      if (lookahead == 'i') ADVANCE(87);
      END_STATE();
    case 52:
      if (lookahead == 'e') ADVANCE(88);
      END_STATE();
    case 53:
      if (lookahead == 's') ADVANCE(89);
      END_STATE();
    case 54:
      if (lookahead == 'n') ADVANCE(90);
      END_STATE();
    case 55:
      if (lookahead == 'r') ADVANCE(91);
      END_STATE();
    case 56:
      if (lookahead == 'e') ADVANCE(92);
      END_STATE();
    case 57:
      if (lookahead == 'i') ADVANCE(93);
      END_STATE();
    case 58:
      if (lookahead == 'a') ADVANCE(94);
      END_STATE();
    case 59:
      if (lookahead == 'i') ADVANCE(95);
      END_STATE();
    case 60:
      if (lookahead == 'r') ADVANCE(96);
      END_STATE();
    case 61:
      ACCEPT_TOKEN(anon_sym_f32);
      END_STATE();
    case 62:
      ACCEPT_TOKEN(anon_sym_f64);
      END_STATE();
    case 63:
      if (lookahead == 's') ADVANCE(97);
      END_STATE();
    case 64:
      if (lookahead == 't') ADVANCE(98);
      END_STATE();
    case 65:
      if (lookahead == 't') ADVANCE(99);
      END_STATE();
    case 66:
      ACCEPT_TOKEN(anon_sym_i16);
      END_STATE();
    case 67:
      ACCEPT_TOKEN(anon_sym_i32);
      END_STATE();
    case 68:
      ACCEPT_TOKEN(anon_sym_i64);
      END_STATE();
    case 69:
      if (lookahead == '-') ADVANCE(100);
      END_STATE();
    case 70:
      if (lookahead == '4') ADVANCE(101);
      if (lookahead == '6') ADVANCE(102);
      END_STATE();
    case 71:
      if (lookahead == '-') ADVANCE(103);
      END_STATE();
    case 72:
      ACCEPT_TOKEN(anon_sym_irl);
      END_STATE();
    case 73:
      if (lookahead == 'z') ADVANCE(104);
      END_STATE();
    case 74:
      if (lookahead == 'e') ADVANCE(105);
      END_STATE();
    case 75:
      if (lookahead == 'i') ADVANCE(106);
      END_STATE();
    case 76:
      if (lookahead == 'i') ADVANCE(107);
      END_STATE();
    case 77:
      if (lookahead == 'l') ADVANCE(108);
      END_STATE();
    case 78:
      if (lookahead == 'e') ADVANCE(109);
      END_STATE();
    case 79:
      if (lookahead == 'e') ADVANCE(110);
      END_STATE();
    case 80:
      if (lookahead == 'e') ADVANCE(111);
      END_STATE();
    case 81:
      ACCEPT_TOKEN(anon_sym_u16);
      END_STATE();
    case 82:
      ACCEPT_TOKEN(anon_sym_u32);
      END_STATE();
    case 83:
      ACCEPT_TOKEN(anon_sym_u64);
      END_STATE();
    case 84:
      if (lookahead == 'e') ADVANCE(112);
      END_STATE();
    case 85:
      ACCEPT_TOKEN(anon_sym_url);
      if (lookahead == '-') ADVANCE(113);
      END_STATE();
    case 86:
      if (lookahead == 'z') ADVANCE(114);
      END_STATE();
    case 87:
      if (lookahead == 'd') ADVANCE(115);
      END_STATE();
    case 88:
      if (lookahead == '6') ADVANCE(116);
      END_STATE();
    case 89:
      if (lookahead == 't') ADVANCE(117);
      END_STATE();
    case 90:
      if (lookahead == 't') ADVANCE(118);
      END_STATE();
    case 91:
      if (lookahead == 'e') ADVANCE(119);
      END_STATE();
    case 92:
      ACCEPT_TOKEN(anon_sym_date);
      if (lookahead == '-') ADVANCE(120);
      END_STATE();
    case 93:
      if (lookahead == 'm') ADVANCE(121);
      END_STATE();
    case 94:
      if (lookahead == 't') ADVANCE(122);
      END_STATE();
    case 95:
      if (lookahead == 'l') ADVANCE(123);
      END_STATE();
    case 96:
      ACCEPT_TOKEN(anon_sym_expr);
      if (lookahead == 'e') ADVANCE(124);
      END_STATE();
    case 97:
      if (lookahead == 'e') ADVANCE(125);
      END_STATE();
    case 98:
      if (lookahead == 'e') ADVANCE(126);
      END_STATE();
    case 99:
      if (lookahead == 'n') ADVANCE(127);
      END_STATE();
    case 100:
      if (lookahead == 'e') ADVANCE(128);
      if (lookahead == 'h') ADVANCE(129);
      END_STATE();
    case 101:
      ACCEPT_TOKEN(anon_sym_ipv4);
      END_STATE();
    case 102:
      ACCEPT_TOKEN(anon_sym_ipv6);
      END_STATE();
    case 103:
      if (lookahead == 'r') ADVANCE(130);
      END_STATE();
    case 104:
      if (lookahead == 'e') ADVANCE(131);
      END_STATE();
    case 105:
      if (lookahead == 'r') ADVANCE(132);
      END_STATE();
    case 106:
      if (lookahead == 'm') ADVANCE(133);
      END_STATE();
    case 107:
      if (lookahead == 'm') ADVANCE(134);
      END_STATE();
    case 108:
      ACCEPT_TOKEN(anon_sym_null);
      END_STATE();
    case 109:
      if (lookahead == 'x') ADVANCE(135);
      END_STATE();
    case 110:
      ACCEPT_TOKEN(anon_sym_time);
      END_STATE();
    case 111:
      ACCEPT_TOKEN(anon_sym_true);
      END_STATE();
    case 112:
      if (lookahead == 'r') ADVANCE(136);
      END_STATE();
    case 113:
      if (lookahead == 'r') ADVANCE(137);
      if (lookahead == 't') ADVANCE(138);
      END_STATE();
    case 114:
      if (lookahead == 'e') ADVANCE(139);
      END_STATE();
    case 115:
      ACCEPT_TOKEN(anon_sym_uuid);
      END_STATE();
    case 116:
      if (lookahead == '4') ADVANCE(140);
      END_STATE();
    case 117:
      if (lookahead == 'r') ADVANCE(141);
      END_STATE();
    case 118:
      if (lookahead == 'r') ADVANCE(142);
      END_STATE();
    case 119:
      if (lookahead == 'n') ADVANCE(143);
      END_STATE();
    case 120:
      if (lookahead == 't') ADVANCE(144);
      END_STATE();
    case 121:
      if (lookahead == 'a') ADVANCE(145);
      END_STATE();
    case 122:
      if (lookahead == 'i') ADVANCE(146);
      END_STATE();
    case 123:
      ACCEPT_TOKEN(anon_sym_email);
      END_STATE();
    case 124:
      if (lookahead == 's') ADVANCE(147);
      END_STATE();
    case 125:
      ACCEPT_TOKEN(anon_sym_false);
      END_STATE();
    case 126:
      if (lookahead == 'r') ADVANCE(148);
      END_STATE();
    case 127:
      if (lookahead == 'a') ADVANCE(149);
      END_STATE();
    case 128:
      if (lookahead == 'm') ADVANCE(150);
      END_STATE();
    case 129:
      if (lookahead == 'o') ADVANCE(151);
      END_STATE();
    case 130:
      if (lookahead == 'e') ADVANCE(152);
      END_STATE();
    case 131:
      ACCEPT_TOKEN(anon_sym_isize);
      END_STATE();
    case 132:
      ACCEPT_TOKEN(anon_sym_lower);
      END_STATE();
    case 133:
      if (lookahead == 'i') ADVANCE(153);
      END_STATE();
    case 134:
      if (lookahead == 'i') ADVANCE(154);
      END_STATE();
    case 135:
      ACCEPT_TOKEN(anon_sym_regex);
      END_STATE();
    case 136:
      ACCEPT_TOKEN(anon_sym_upper);
      END_STATE();
    case 137:
      if (lookahead == 'e') ADVANCE(155);
      END_STATE();
    case 138:
      if (lookahead == 'e') ADVANCE(156);
      END_STATE();
    case 139:
      ACCEPT_TOKEN(anon_sym_usize);
      END_STATE();
    case 140:
      ACCEPT_TOKEN(anon_sym_base64);
      END_STATE();
    case 141:
      if (lookahead == 'a') ADVANCE(157);
      END_STATE();
    case 142:
      if (lookahead == 'y') ADVANCE(158);
      END_STATE();
    case 143:
      if (lookahead == 'c') ADVANCE(159);
      END_STATE();
    case 144:
      if (lookahead == 'i') ADVANCE(160);
      END_STATE();
    case 145:
      if (lookahead == 'l') ADVANCE(161);
      END_STATE();
    case 146:
      if (lookahead == 'o') ADVANCE(162);
      END_STATE();
    case 147:
      if (lookahead == 's') ADVANCE(163);
      END_STATE();
    case 148:
      ACCEPT_TOKEN(anon_sym_filter);
      END_STATE();
    case 149:
      if (lookahead == 'm') ADVANCE(164);
      END_STATE();
    case 150:
      if (lookahead == 'a') ADVANCE(165);
      END_STATE();
    case 151:
      if (lookahead == 's') ADVANCE(166);
      END_STATE();
    case 152:
      if (lookahead == 'f') ADVANCE(167);
      END_STATE();
    case 153:
      if (lookahead == 'z') ADVANCE(168);
      END_STATE();
    case 154:
      if (lookahead == 'z') ADVANCE(169);
      END_STATE();
    case 155:
      if (lookahead == 'f') ADVANCE(170);
      END_STATE();
    case 156:
      if (lookahead == 'm') ADVANCE(171);
      END_STATE();
    case 157:
      if (lookahead == 'i') ADVANCE(172);
      END_STATE();
    case 158:
      if (lookahead == '-') ADVANCE(173);
      END_STATE();
    case 159:
      if (lookahead == 'y') ADVANCE(174);
      END_STATE();
    case 160:
      if (lookahead == 'm') ADVANCE(175);
      END_STATE();
    case 161:
      ACCEPT_TOKEN(anon_sym_decimal);
      if (lookahead == '1') ADVANCE(176);
      if (lookahead == '6') ADVANCE(177);
      END_STATE();
    case 162:
      if (lookahead == 'n') ADVANCE(178);
      END_STATE();
    case 163:
      if (lookahead == 'i') ADVANCE(179);
      END_STATE();
    case 164:
      if (lookahead == 'e') ADVANCE(180);
      END_STATE();
    case 165:
      if (lookahead == 'i') ADVANCE(181);
      END_STATE();
    case 166:
      if (lookahead == 't') ADVANCE(182);
      END_STATE();
    case 167:
      if (lookahead == 'e') ADVANCE(183);
      END_STATE();
    case 168:
      if (lookahead == 'e') ADVANCE(184);
      END_STATE();
    case 169:
      if (lookahead == 'e') ADVANCE(185);
      END_STATE();
    case 170:
      if (lookahead == 'e') ADVANCE(186);
      END_STATE();
    case 171:
      if (lookahead == 'p') ADVANCE(187);
      END_STATE();
    case 172:
      if (lookahead == 'n') ADVANCE(188);
      END_STATE();
    case 173:
      if (lookahead == '2') ADVANCE(189);
      if (lookahead == '3') ADVANCE(190);
      if (lookahead == 's') ADVANCE(191);
      END_STATE();
    case 174:
      ACCEPT_TOKEN(anon_sym_currency);
      END_STATE();
    case 175:
      if (lookahead == 'e') ADVANCE(192);
      END_STATE();
    case 176:
      if (lookahead == '2') ADVANCE(193);
      END_STATE();
    case 177:
      if (lookahead == '4') ADVANCE(194);
      END_STATE();
    case 178:
      ACCEPT_TOKEN(anon_sym_duration);
      END_STATE();
    case 179:
      if (lookahead == 'o') ADVANCE(195);
      END_STATE();
    case 180:
      ACCEPT_TOKEN(anon_sym_hostname);
      END_STATE();
    case 181:
      if (lookahead == 'l') ADVANCE(196);
      END_STATE();
    case 182:
      if (lookahead == 'n') ADVANCE(197);
      END_STATE();
    case 183:
      if (lookahead == 'r') ADVANCE(198);
      END_STATE();
    case 184:
      ACCEPT_TOKEN(anon_sym_maximize);
      END_STATE();
    case 185:
      ACCEPT_TOKEN(anon_sym_minimize);
      END_STATE();
    case 186:
      if (lookahead == 'r') ADVANCE(199);
      END_STATE();
    case 187:
      if (lookahead == 'l') ADVANCE(200);
      END_STATE();
    case 188:
      if (lookahead == 't') ADVANCE(201);
      END_STATE();
    case 189:
      ACCEPT_TOKEN(anon_sym_country_DASH2);
      END_STATE();
    case 190:
      ACCEPT_TOKEN(anon_sym_country_DASH3);
      END_STATE();
    case 191:
      if (lookahead == 'u') ADVANCE(202);
      END_STATE();
    case 192:
      ACCEPT_TOKEN(anon_sym_date_DASHtime);
      END_STATE();
    case 193:
      if (lookahead == '8') ADVANCE(203);
      END_STATE();
    case 194:
      ACCEPT_TOKEN(anon_sym_decimal64);
      END_STATE();
    case 195:
      if (lookahead == 'n') ADVANCE(204);
      END_STATE();
    case 196:
      ACCEPT_TOKEN(anon_sym_idn_DASHemail);
      END_STATE();
    case 197:
      if (lookahead == 'a') ADVANCE(205);
      END_STATE();
    case 198:
      if (lookahead == 'e') ADVANCE(206);
      END_STATE();
    case 199:
      if (lookahead == 'e') ADVANCE(207);
      END_STATE();
    case 200:
      if (lookahead == 'a') ADVANCE(208);
      END_STATE();
    case 201:
      ACCEPT_TOKEN(anon_sym_constraint);
      END_STATE();
    case 202:
      if (lookahead == 'b') ADVANCE(209);
      END_STATE();
    case 203:
      ACCEPT_TOKEN(anon_sym_decimal128);
      END_STATE();
    case 204:
      ACCEPT_TOKEN(anon_sym_expression);
      END_STATE();
    case 205:
      if (lookahead == 'm') ADVANCE(210);
      END_STATE();
    case 206:
      if (lookahead == 'n') ADVANCE(211);
      END_STATE();
    case 207:
      if (lookahead == 'n') ADVANCE(212);
      END_STATE();
    case 208:
      if (lookahead == 't') ADVANCE(213);
      END_STATE();
    case 209:
      if (lookahead == 'd') ADVANCE(214);
      END_STATE();
    case 210:
      if (lookahead == 'e') ADVANCE(215);
      END_STATE();
    case 211:
      if (lookahead == 'c') ADVANCE(216);
      END_STATE();
    case 212:
      if (lookahead == 'c') ADVANCE(217);
      END_STATE();
    case 213:
      if (lookahead == 'e') ADVANCE(218);
      END_STATE();
    case 214:
      if (lookahead == 'i') ADVANCE(219);
      END_STATE();
    case 215:
      ACCEPT_TOKEN(anon_sym_idn_DASHhostname);
      END_STATE();
    case 216:
      if (lookahead == 'e') ADVANCE(220);
      END_STATE();
    case 217:
      if (lookahead == 'e') ADVANCE(221);
      END_STATE();
    case 218:
      ACCEPT_TOKEN(anon_sym_url_DASHtemplate);
      END_STATE();
    case 219:
      if (lookahead == 'v') ADVANCE(222);
      END_STATE();
    case 220:
      ACCEPT_TOKEN(anon_sym_iri_DASHreference);
      END_STATE();
    case 221:
      ACCEPT_TOKEN(anon_sym_url_DASHreference);
      END_STATE();
    case 222:
      if (lookahead == 'i') ADVANCE(223);
      END_STATE();
    case 223:
      if (lookahead == 's') ADVANCE(224);
      END_STATE();
    case 224:
      if (lookahead == 'i') ADVANCE(225);
      END_STATE();
    case 225:
      if (lookahead == 'o') ADVANCE(226);
      END_STATE();
    case 226:
      if (lookahead == 'n') ADVANCE(227);
      END_STATE();
    case 227:
      ACCEPT_TOKEN(anon_sym_country_DASHsubdivision);
      END_STATE();
    default:
      return false;
  }
}

static const TSLexMode ts_lex_modes[STATE_COUNT] = {
  [0] = {.lex_state = 0, .external_lex_state = 1},
  [1] = {.lex_state = 23, .external_lex_state = 2},
  [2] = {.lex_state = 23, .external_lex_state = 1},
  [3] = {.lex_state = 23, .external_lex_state = 1},
  [4] = {.lex_state = 23, .external_lex_state = 1},
  [5] = {.lex_state = 23, .external_lex_state = 1},
  [6] = {.lex_state = 23, .external_lex_state = 1},
  [7] = {.lex_state = 23, .external_lex_state = 1},
  [8] = {.lex_state = 23, .external_lex_state = 1},
  [9] = {.lex_state = 23, .external_lex_state = 1},
  [10] = {.lex_state = 23, .external_lex_state = 1},
  [11] = {.lex_state = 23, .external_lex_state = 1},
  [12] = {.lex_state = 23, .external_lex_state = 1},
  [13] = {.lex_state = 23, .external_lex_state = 1},
  [14] = {.lex_state = 23, .external_lex_state = 1},
  [15] = {.lex_state = 23, .external_lex_state = 1},
  [16] = {.lex_state = 23, .external_lex_state = 1},
  [17] = {.lex_state = 23, .external_lex_state = 1},
  [18] = {.lex_state = 23, .external_lex_state = 1},
  [19] = {.lex_state = 23, .external_lex_state = 1},
  [20] = {.lex_state = 23, .external_lex_state = 1},
  [21] = {.lex_state = 23, .external_lex_state = 1},
  [22] = {.lex_state = 23, .external_lex_state = 1},
  [23] = {.lex_state = 23, .external_lex_state = 1},
  [24] = {.lex_state = 23, .external_lex_state = 1},
  [25] = {.lex_state = 23, .external_lex_state = 1},
  [26] = {.lex_state = 23, .external_lex_state = 1},
  [27] = {.lex_state = 23, .external_lex_state = 1},
  [28] = {.lex_state = 23, .external_lex_state = 1},
  [29] = {.lex_state = 23, .external_lex_state = 2},
  [30] = {.lex_state = 23, .external_lex_state = 2},
  [31] = {.lex_state = 23, .external_lex_state = 2},
  [32] = {.lex_state = 23, .external_lex_state = 2},
  [33] = {.lex_state = 23, .external_lex_state = 2},
  [34] = {.lex_state = 23, .external_lex_state = 2},
  [35] = {.lex_state = 23, .external_lex_state = 2},
  [36] = {.lex_state = 23, .external_lex_state = 2},
  [37] = {.lex_state = 23, .external_lex_state = 2},
  [38] = {.lex_state = 23, .external_lex_state = 2},
  [39] = {.lex_state = 23, .external_lex_state = 2},
  [40] = {.lex_state = 23, .external_lex_state = 2},
  [41] = {.lex_state = 23, .external_lex_state = 2},
  [42] = {.lex_state = 9, .external_lex_state = 2},
  [43] = {.lex_state = 23, .external_lex_state = 2},
  [44] = {.lex_state = 9, .external_lex_state = 2},
  [45] = {.lex_state = 9, .external_lex_state = 2},
  [46] = {.lex_state = 23, .external_lex_state = 2},
  [47] = {.lex_state = 23, .external_lex_state = 2},
  [48] = {.lex_state = 23, .external_lex_state = 2},
  [49] = {.lex_state = 23, .external_lex_state = 2},
  [50] = {.lex_state = 23, .external_lex_state = 2},
  [51] = {.lex_state = 23, .external_lex_state = 2},
  [52] = {.lex_state = 23, .external_lex_state = 2},
  [53] = {.lex_state = 23, .external_lex_state = 2},
  [54] = {.lex_state = 23, .external_lex_state = 2},
  [55] = {.lex_state = 23, .external_lex_state = 2},
  [56] = {.lex_state = 23, .external_lex_state = 2},
  [57] = {.lex_state = 23, .external_lex_state = 2},
  [58] = {.lex_state = 23, .external_lex_state = 2},
  [59] = {.lex_state = 23, .external_lex_state = 2},
  [60] = {.lex_state = 23, .external_lex_state = 2},
  [61] = {.lex_state = 23, .external_lex_state = 2},
  [62] = {.lex_state = 23, .external_lex_state = 2},
  [63] = {.lex_state = 23, .external_lex_state = 2},
  [64] = {.lex_state = 23, .external_lex_state = 2},
  [65] = {.lex_state = 23, .external_lex_state = 2},
  [66] = {.lex_state = 23, .external_lex_state = 2},
  [67] = {.lex_state = 23, .external_lex_state = 2},
  [68] = {.lex_state = 23, .external_lex_state = 2},
  [69] = {.lex_state = 23, .external_lex_state = 2},
  [70] = {.lex_state = 23, .external_lex_state = 2},
  [71] = {.lex_state = 23, .external_lex_state = 2},
  [72] = {.lex_state = 23, .external_lex_state = 2},
  [73] = {.lex_state = 23, .external_lex_state = 2},
  [74] = {.lex_state = 23, .external_lex_state = 2},
  [75] = {.lex_state = 23, .external_lex_state = 2},
  [76] = {.lex_state = 23, .external_lex_state = 2},
  [77] = {.lex_state = 23, .external_lex_state = 2},
  [78] = {.lex_state = 23, .external_lex_state = 2},
  [79] = {.lex_state = 23, .external_lex_state = 2},
  [80] = {.lex_state = 23, .external_lex_state = 2},
  [81] = {.lex_state = 23, .external_lex_state = 1},
  [82] = {.lex_state = 23, .external_lex_state = 2},
  [83] = {.lex_state = 23, .external_lex_state = 2},
  [84] = {.lex_state = 23, .external_lex_state = 1},
  [85] = {.lex_state = 23, .external_lex_state = 2},
  [86] = {.lex_state = 23, .external_lex_state = 2},
  [87] = {.lex_state = 23, .external_lex_state = 2},
  [88] = {.lex_state = 23, .external_lex_state = 2},
  [89] = {.lex_state = 23, .external_lex_state = 1},
  [90] = {.lex_state = 23, .external_lex_state = 1},
  [91] = {.lex_state = 23, .external_lex_state = 1},
  [92] = {.lex_state = 23, .external_lex_state = 1},
  [93] = {.lex_state = 23, .external_lex_state = 1},
  [94] = {.lex_state = 23, .external_lex_state = 2},
  [95] = {.lex_state = 23, .external_lex_state = 2},
  [96] = {.lex_state = 23, .external_lex_state = 2},
  [97] = {.lex_state = 23, .external_lex_state = 2},
  [98] = {.lex_state = 23, .external_lex_state = 2},
  [99] = {.lex_state = 23, .external_lex_state = 1},
  [100] = {.lex_state = 23, .external_lex_state = 1},
  [101] = {.lex_state = 23, .external_lex_state = 2},
  [102] = {.lex_state = 23, .external_lex_state = 1},
  [103] = {.lex_state = 23, .external_lex_state = 2},
  [104] = {.lex_state = 23, .external_lex_state = 1},
  [105] = {.lex_state = 23, .external_lex_state = 1},
  [106] = {.lex_state = 23, .external_lex_state = 2},
  [107] = {.lex_state = 23, .external_lex_state = 2},
  [108] = {.lex_state = 23, .external_lex_state = 2},
  [109] = {.lex_state = 23, .external_lex_state = 2},
  [110] = {.lex_state = 23, .external_lex_state = 2},
  [111] = {.lex_state = 23, .external_lex_state = 2},
  [112] = {.lex_state = 23, .external_lex_state = 2},
  [113] = {.lex_state = 23, .external_lex_state = 2},
  [114] = {.lex_state = 23, .external_lex_state = 2},
  [115] = {.lex_state = 23, .external_lex_state = 2},
  [116] = {.lex_state = 23, .external_lex_state = 2},
  [117] = {.lex_state = 23, .external_lex_state = 2},
  [118] = {.lex_state = 23, .external_lex_state = 2},
  [119] = {.lex_state = 23, .external_lex_state = 2},
  [120] = {.lex_state = 23, .external_lex_state = 2},
  [121] = {.lex_state = 23, .external_lex_state = 2},
  [122] = {.lex_state = 23, .external_lex_state = 2},
  [123] = {.lex_state = 23, .external_lex_state = 2},
  [124] = {.lex_state = 23, .external_lex_state = 2},
  [125] = {.lex_state = 23, .external_lex_state = 2},
  [126] = {.lex_state = 23, .external_lex_state = 2},
  [127] = {.lex_state = 23, .external_lex_state = 2},
  [128] = {.lex_state = 23, .external_lex_state = 2},
  [129] = {.lex_state = 23, .external_lex_state = 2},
  [130] = {.lex_state = 23, .external_lex_state = 2},
  [131] = {.lex_state = 23, .external_lex_state = 2},
  [132] = {.lex_state = 23, .external_lex_state = 2},
  [133] = {.lex_state = 23, .external_lex_state = 2},
  [134] = {.lex_state = 23, .external_lex_state = 2},
  [135] = {.lex_state = 23, .external_lex_state = 2},
  [136] = {.lex_state = 23, .external_lex_state = 2},
  [137] = {.lex_state = 23, .external_lex_state = 2},
  [138] = {.lex_state = 23, .external_lex_state = 2},
  [139] = {.lex_state = 23, .external_lex_state = 2},
  [140] = {.lex_state = 23, .external_lex_state = 2},
  [141] = {.lex_state = 23, .external_lex_state = 2},
  [142] = {.lex_state = 23, .external_lex_state = 2},
  [143] = {.lex_state = 23, .external_lex_state = 2},
  [144] = {.lex_state = 23, .external_lex_state = 2},
  [145] = {.lex_state = 23, .external_lex_state = 2},
  [146] = {.lex_state = 23, .external_lex_state = 2},
  [147] = {.lex_state = 23, .external_lex_state = 2},
  [148] = {.lex_state = 23, .external_lex_state = 2},
  [149] = {.lex_state = 23, .external_lex_state = 2},
  [150] = {.lex_state = 23, .external_lex_state = 2},
  [151] = {.lex_state = 23, .external_lex_state = 2},
  [152] = {.lex_state = 23, .external_lex_state = 2},
  [153] = {.lex_state = 23, .external_lex_state = 2},
  [154] = {.lex_state = 23, .external_lex_state = 2},
  [155] = {.lex_state = 23, .external_lex_state = 2},
  [156] = {.lex_state = 23, .external_lex_state = 2},
  [157] = {.lex_state = 23, .external_lex_state = 2},
  [158] = {.lex_state = 23, .external_lex_state = 2},
  [159] = {.lex_state = 23, .external_lex_state = 2},
  [160] = {.lex_state = 23, .external_lex_state = 2},
  [161] = {.lex_state = 23, .external_lex_state = 2},
  [162] = {.lex_state = 23, .external_lex_state = 2},
  [163] = {.lex_state = 23, .external_lex_state = 2},
  [164] = {.lex_state = 23, .external_lex_state = 2},
  [165] = {.lex_state = 23, .external_lex_state = 2},
  [166] = {.lex_state = 23, .external_lex_state = 2},
  [167] = {.lex_state = 23, .external_lex_state = 2},
  [168] = {.lex_state = 23, .external_lex_state = 2},
  [169] = {.lex_state = 23, .external_lex_state = 2},
  [170] = {.lex_state = 23, .external_lex_state = 2},
  [171] = {.lex_state = 23, .external_lex_state = 2},
  [172] = {.lex_state = 23, .external_lex_state = 2},
  [173] = {.lex_state = 23, .external_lex_state = 2},
  [174] = {.lex_state = 23, .external_lex_state = 2},
  [175] = {.lex_state = 23, .external_lex_state = 2},
  [176] = {.lex_state = 23, .external_lex_state = 2},
  [177] = {.lex_state = 23, .external_lex_state = 2},
  [178] = {.lex_state = 23, .external_lex_state = 2},
  [179] = {.lex_state = 23, .external_lex_state = 2},
  [180] = {.lex_state = 23, .external_lex_state = 2},
  [181] = {.lex_state = 23, .external_lex_state = 2},
  [182] = {.lex_state = 23, .external_lex_state = 2},
  [183] = {.lex_state = 23, .external_lex_state = 2},
  [184] = {.lex_state = 23, .external_lex_state = 2},
  [185] = {.lex_state = 23, .external_lex_state = 2},
  [186] = {.lex_state = 23, .external_lex_state = 2},
  [187] = {.lex_state = 23, .external_lex_state = 2},
  [188] = {.lex_state = 23, .external_lex_state = 2},
  [189] = {.lex_state = 23, .external_lex_state = 2},
  [190] = {.lex_state = 23, .external_lex_state = 2},
  [191] = {.lex_state = 23, .external_lex_state = 2},
  [192] = {.lex_state = 23, .external_lex_state = 2},
  [193] = {.lex_state = 23, .external_lex_state = 2},
  [194] = {.lex_state = 23, .external_lex_state = 2},
  [195] = {.lex_state = 23, .external_lex_state = 2},
  [196] = {.lex_state = 23, .external_lex_state = 2},
  [197] = {.lex_state = 23, .external_lex_state = 2},
  [198] = {.lex_state = 23, .external_lex_state = 2},
  [199] = {.lex_state = 23, .external_lex_state = 2},
  [200] = {.lex_state = 23, .external_lex_state = 2},
  [201] = {.lex_state = 23, .external_lex_state = 2},
  [202] = {.lex_state = 23, .external_lex_state = 2},
  [203] = {.lex_state = 23, .external_lex_state = 2},
  [204] = {.lex_state = 23, .external_lex_state = 2},
  [205] = {.lex_state = 23, .external_lex_state = 2},
  [206] = {.lex_state = 23, .external_lex_state = 2},
  [207] = {.lex_state = 23, .external_lex_state = 2},
  [208] = {.lex_state = 23, .external_lex_state = 2},
  [209] = {.lex_state = 23, .external_lex_state = 2},
  [210] = {.lex_state = 23, .external_lex_state = 2},
  [211] = {.lex_state = 23, .external_lex_state = 2},
  [212] = {.lex_state = 23, .external_lex_state = 2},
  [213] = {.lex_state = 23, .external_lex_state = 2},
  [214] = {.lex_state = 23, .external_lex_state = 2},
  [215] = {.lex_state = 23, .external_lex_state = 2},
  [216] = {.lex_state = 23, .external_lex_state = 2},
  [217] = {.lex_state = 23, .external_lex_state = 2},
  [218] = {.lex_state = 23, .external_lex_state = 2},
  [219] = {.lex_state = 23, .external_lex_state = 2},
  [220] = {.lex_state = 23, .external_lex_state = 2},
  [221] = {.lex_state = 23, .external_lex_state = 2},
  [222] = {.lex_state = 23, .external_lex_state = 2},
  [223] = {.lex_state = 23, .external_lex_state = 2},
  [224] = {.lex_state = 23, .external_lex_state = 2},
  [225] = {.lex_state = 23, .external_lex_state = 2},
  [226] = {.lex_state = 23, .external_lex_state = 2},
  [227] = {.lex_state = 23, .external_lex_state = 2},
  [228] = {.lex_state = 23, .external_lex_state = 2},
  [229] = {.lex_state = 23, .external_lex_state = 2},
  [230] = {.lex_state = 23, .external_lex_state = 2},
  [231] = {.lex_state = 23, .external_lex_state = 2},
  [232] = {.lex_state = 23, .external_lex_state = 2},
  [233] = {.lex_state = 23, .external_lex_state = 2},
  [234] = {.lex_state = 23, .external_lex_state = 2},
  [235] = {.lex_state = 23, .external_lex_state = 2},
  [236] = {.lex_state = 23, .external_lex_state = 2},
  [237] = {.lex_state = 23, .external_lex_state = 2},
  [238] = {.lex_state = 23, .external_lex_state = 2},
  [239] = {.lex_state = 23, .external_lex_state = 2},
  [240] = {.lex_state = 23, .external_lex_state = 2},
  [241] = {.lex_state = 23, .external_lex_state = 2},
  [242] = {.lex_state = 23, .external_lex_state = 2},
  [243] = {.lex_state = 23, .external_lex_state = 2},
  [244] = {.lex_state = 23, .external_lex_state = 2},
  [245] = {.lex_state = 23, .external_lex_state = 2},
  [246] = {.lex_state = 23, .external_lex_state = 2},
  [247] = {.lex_state = 23, .external_lex_state = 2},
  [248] = {.lex_state = 23, .external_lex_state = 2},
  [249] = {.lex_state = 23, .external_lex_state = 2},
  [250] = {.lex_state = 23, .external_lex_state = 2},
  [251] = {.lex_state = 23, .external_lex_state = 2},
  [252] = {.lex_state = 23, .external_lex_state = 2},
  [253] = {.lex_state = 23, .external_lex_state = 2},
  [254] = {.lex_state = 23, .external_lex_state = 2},
  [255] = {.lex_state = 23, .external_lex_state = 2},
  [256] = {.lex_state = 23, .external_lex_state = 2},
  [257] = {.lex_state = 23, .external_lex_state = 2},
  [258] = {.lex_state = 23, .external_lex_state = 2},
  [259] = {.lex_state = 23, .external_lex_state = 2},
  [260] = {.lex_state = 23, .external_lex_state = 2},
  [261] = {.lex_state = 23, .external_lex_state = 2},
  [262] = {.lex_state = 23, .external_lex_state = 2},
  [263] = {.lex_state = 23, .external_lex_state = 2},
  [264] = {.lex_state = 23, .external_lex_state = 2},
  [265] = {.lex_state = 23, .external_lex_state = 2},
  [266] = {.lex_state = 23, .external_lex_state = 2},
  [267] = {.lex_state = 23, .external_lex_state = 2},
  [268] = {.lex_state = 23, .external_lex_state = 1},
  [269] = {.lex_state = 23, .external_lex_state = 1},
  [270] = {.lex_state = 23, .external_lex_state = 2},
  [271] = {.lex_state = 23, .external_lex_state = 1},
  [272] = {.lex_state = 23, .external_lex_state = 1},
  [273] = {.lex_state = 23, .external_lex_state = 1},
  [274] = {.lex_state = 23, .external_lex_state = 1},
  [275] = {.lex_state = 23, .external_lex_state = 1},
  [276] = {.lex_state = 23, .external_lex_state = 1},
  [277] = {.lex_state = 23, .external_lex_state = 1},
  [278] = {.lex_state = 23, .external_lex_state = 1},
  [279] = {.lex_state = 23, .external_lex_state = 1},
  [280] = {.lex_state = 23, .external_lex_state = 1},
  [281] = {.lex_state = 23, .external_lex_state = 1},
  [282] = {.lex_state = 23, .external_lex_state = 1},
  [283] = {.lex_state = 23, .external_lex_state = 1},
  [284] = {.lex_state = 23, .external_lex_state = 1},
  [285] = {.lex_state = 23, .external_lex_state = 1},
  [286] = {.lex_state = 23, .external_lex_state = 1},
  [287] = {.lex_state = 23, .external_lex_state = 1},
  [288] = {.lex_state = 23, .external_lex_state = 1},
  [289] = {.lex_state = 23, .external_lex_state = 1},
  [290] = {.lex_state = 23, .external_lex_state = 1},
  [291] = {.lex_state = 23, .external_lex_state = 1},
  [292] = {.lex_state = 23, .external_lex_state = 1},
  [293] = {.lex_state = 23, .external_lex_state = 1},
  [294] = {.lex_state = 23, .external_lex_state = 1},
  [295] = {.lex_state = 23, .external_lex_state = 1},
  [296] = {.lex_state = 23, .external_lex_state = 1},
  [297] = {.lex_state = 23, .external_lex_state = 1},
  [298] = {.lex_state = 23, .external_lex_state = 1},
  [299] = {.lex_state = 2, .external_lex_state = 1},
  [300] = {.lex_state = 23, .external_lex_state = 1},
  [301] = {.lex_state = 23, .external_lex_state = 1},
  [302] = {.lex_state = 23, .external_lex_state = 1},
  [303] = {.lex_state = 23, .external_lex_state = 1},
  [304] = {.lex_state = 23, .external_lex_state = 1},
  [305] = {.lex_state = 23, .external_lex_state = 1},
  [306] = {.lex_state = 23, .external_lex_state = 1},
  [307] = {.lex_state = 23, .external_lex_state = 1},
  [308] = {.lex_state = 23, .external_lex_state = 1},
  [309] = {.lex_state = 23, .external_lex_state = 1},
  [310] = {.lex_state = 23, .external_lex_state = 1},
  [311] = {.lex_state = 23, .external_lex_state = 1},
  [312] = {.lex_state = 23, .external_lex_state = 1},
  [313] = {.lex_state = 23, .external_lex_state = 1},
  [314] = {.lex_state = 23, .external_lex_state = 1},
  [315] = {.lex_state = 23, .external_lex_state = 1},
  [316] = {.lex_state = 23, .external_lex_state = 1},
  [317] = {.lex_state = 23, .external_lex_state = 1},
  [318] = {.lex_state = 23, .external_lex_state = 1},
  [319] = {.lex_state = 23, .external_lex_state = 1},
  [320] = {.lex_state = 23, .external_lex_state = 1},
  [321] = {.lex_state = 23, .external_lex_state = 1},
  [322] = {.lex_state = 23, .external_lex_state = 1},
  [323] = {.lex_state = 23, .external_lex_state = 1},
  [324] = {.lex_state = 23, .external_lex_state = 1},
  [325] = {.lex_state = 23, .external_lex_state = 1},
  [326] = {.lex_state = 23, .external_lex_state = 1},
  [327] = {.lex_state = 23, .external_lex_state = 1},
  [328] = {.lex_state = 1, .external_lex_state = 1},
  [329] = {.lex_state = 23, .external_lex_state = 1},
  [330] = {.lex_state = 23, .external_lex_state = 1},
  [331] = {.lex_state = 23, .external_lex_state = 1},
  [332] = {.lex_state = 23, .external_lex_state = 1},
  [333] = {.lex_state = 23, .external_lex_state = 1},
  [334] = {.lex_state = 23, .external_lex_state = 1},
  [335] = {.lex_state = 23, .external_lex_state = 1},
  [336] = {.lex_state = 23, .external_lex_state = 1},
  [337] = {.lex_state = 23, .external_lex_state = 1},
  [338] = {.lex_state = 23, .external_lex_state = 1},
  [339] = {.lex_state = 23, .external_lex_state = 2},
  [340] = {.lex_state = 23, .external_lex_state = 1},
  [341] = {.lex_state = 23, .external_lex_state = 1},
  [342] = {.lex_state = 23, .external_lex_state = 1},
  [343] = {.lex_state = 23, .external_lex_state = 1},
  [344] = {.lex_state = 23, .external_lex_state = 1},
  [345] = {.lex_state = 23, .external_lex_state = 1},
  [346] = {.lex_state = 23, .external_lex_state = 1},
  [347] = {.lex_state = 23, .external_lex_state = 1},
  [348] = {.lex_state = 23, .external_lex_state = 1},
  [349] = {.lex_state = 23, .external_lex_state = 1},
  [350] = {.lex_state = 23, .external_lex_state = 1},
  [351] = {.lex_state = 23, .external_lex_state = 1},
  [352] = {.lex_state = 23, .external_lex_state = 1},
  [353] = {.lex_state = 23, .external_lex_state = 1},
  [354] = {.lex_state = 23, .external_lex_state = 1},
  [355] = {.lex_state = 23, .external_lex_state = 1},
  [356] = {.lex_state = 23, .external_lex_state = 1},
  [357] = {.lex_state = 23, .external_lex_state = 1},
  [358] = {.lex_state = 23, .external_lex_state = 1},
  [359] = {.lex_state = 23, .external_lex_state = 1},
  [360] = {.lex_state = 23, .external_lex_state = 1},
  [361] = {.lex_state = 23, .external_lex_state = 1},
  [362] = {.lex_state = 23, .external_lex_state = 1},
  [363] = {.lex_state = 23, .external_lex_state = 1},
  [364] = {.lex_state = 23, .external_lex_state = 1},
  [365] = {.lex_state = 23, .external_lex_state = 1},
  [366] = {.lex_state = 23, .external_lex_state = 1},
  [367] = {.lex_state = 23, .external_lex_state = 1},
  [368] = {.lex_state = 23, .external_lex_state = 1},
  [369] = {.lex_state = 23, .external_lex_state = 1},
  [370] = {.lex_state = 23, .external_lex_state = 1},
  [371] = {.lex_state = 23, .external_lex_state = 1},
  [372] = {.lex_state = 23, .external_lex_state = 1},
  [373] = {.lex_state = 23, .external_lex_state = 1},
  [374] = {.lex_state = 23, .external_lex_state = 1},
  [375] = {.lex_state = 1, .external_lex_state = 1},
  [376] = {.lex_state = 23, .external_lex_state = 1},
  [377] = {.lex_state = 23, .external_lex_state = 1},
  [378] = {.lex_state = 23, .external_lex_state = 1},
  [379] = {.lex_state = 23, .external_lex_state = 1},
  [380] = {.lex_state = 23, .external_lex_state = 1},
  [381] = {.lex_state = 23, .external_lex_state = 1},
  [382] = {.lex_state = 23, .external_lex_state = 1},
  [383] = {.lex_state = 23, .external_lex_state = 1},
  [384] = {.lex_state = 23, .external_lex_state = 1},
  [385] = {.lex_state = 23, .external_lex_state = 1},
  [386] = {.lex_state = 23, .external_lex_state = 1},
  [387] = {.lex_state = 23, .external_lex_state = 1},
  [388] = {.lex_state = 23, .external_lex_state = 1},
  [389] = {.lex_state = 23, .external_lex_state = 1},
  [390] = {.lex_state = 23, .external_lex_state = 1},
  [391] = {.lex_state = 23, .external_lex_state = 1},
  [392] = {.lex_state = 23, .external_lex_state = 1},
  [393] = {.lex_state = 23, .external_lex_state = 1},
  [394] = {.lex_state = 23, .external_lex_state = 1},
  [395] = {.lex_state = 23, .external_lex_state = 1},
  [396] = {.lex_state = 23, .external_lex_state = 1},
  [397] = {.lex_state = 23, .external_lex_state = 1},
  [398] = {.lex_state = 23, .external_lex_state = 1},
  [399] = {.lex_state = 23, .external_lex_state = 1},
  [400] = {.lex_state = 23, .external_lex_state = 1},
  [401] = {.lex_state = 23, .external_lex_state = 1},
  [402] = {.lex_state = 23, .external_lex_state = 1},
  [403] = {.lex_state = 23, .external_lex_state = 1},
  [404] = {.lex_state = 23, .external_lex_state = 1},
  [405] = {.lex_state = 23, .external_lex_state = 1},
  [406] = {.lex_state = 23, .external_lex_state = 1},
  [407] = {.lex_state = 23, .external_lex_state = 1},
  [408] = {.lex_state = 23, .external_lex_state = 1},
  [409] = {.lex_state = 23, .external_lex_state = 1},
  [410] = {.lex_state = 23, .external_lex_state = 1},
  [411] = {.lex_state = 23, .external_lex_state = 1},
  [412] = {.lex_state = 23, .external_lex_state = 1},
  [413] = {.lex_state = 23, .external_lex_state = 1},
  [414] = {.lex_state = 23, .external_lex_state = 1},
  [415] = {.lex_state = 23, .external_lex_state = 1},
  [416] = {.lex_state = 23, .external_lex_state = 1},
  [417] = {.lex_state = 23, .external_lex_state = 1},
  [418] = {.lex_state = 23, .external_lex_state = 1},
  [419] = {.lex_state = 23, .external_lex_state = 1},
  [420] = {.lex_state = 1, .external_lex_state = 1},
  [421] = {.lex_state = 23, .external_lex_state = 1},
  [422] = {.lex_state = 23, .external_lex_state = 1},
  [423] = {.lex_state = 23, .external_lex_state = 1},
  [424] = {.lex_state = 23, .external_lex_state = 1},
  [425] = {.lex_state = 23, .external_lex_state = 1},
  [426] = {.lex_state = 23, .external_lex_state = 1},
  [427] = {.lex_state = 23, .external_lex_state = 1},
  [428] = {.lex_state = 23, .external_lex_state = 1},
  [429] = {.lex_state = 23, .external_lex_state = 1},
  [430] = {.lex_state = 23, .external_lex_state = 1},
  [431] = {.lex_state = 23, .external_lex_state = 1},
  [432] = {.lex_state = 23, .external_lex_state = 1},
  [433] = {.lex_state = 23, .external_lex_state = 1},
  [434] = {.lex_state = 23, .external_lex_state = 1},
  [435] = {.lex_state = 23, .external_lex_state = 1},
  [436] = {.lex_state = 23, .external_lex_state = 1},
  [437] = {.lex_state = 23, .external_lex_state = 1},
  [438] = {.lex_state = 23, .external_lex_state = 1},
  [439] = {.lex_state = 23, .external_lex_state = 1},
  [440] = {.lex_state = 23, .external_lex_state = 1},
  [441] = {.lex_state = 23, .external_lex_state = 1},
  [442] = {.lex_state = 23, .external_lex_state = 1},
  [443] = {.lex_state = 23, .external_lex_state = 1},
  [444] = {.lex_state = 23, .external_lex_state = 1},
  [445] = {.lex_state = 23, .external_lex_state = 1},
  [446] = {.lex_state = 23, .external_lex_state = 1},
  [447] = {.lex_state = 23, .external_lex_state = 1},
  [448] = {.lex_state = 23, .external_lex_state = 1},
  [449] = {.lex_state = 23, .external_lex_state = 1},
  [450] = {.lex_state = 23, .external_lex_state = 1},
  [451] = {.lex_state = 23, .external_lex_state = 1},
  [452] = {.lex_state = 23, .external_lex_state = 1},
  [453] = {.lex_state = 23, .external_lex_state = 1},
  [454] = {.lex_state = 23, .external_lex_state = 1},
  [455] = {.lex_state = 23, .external_lex_state = 1},
  [456] = {.lex_state = 23, .external_lex_state = 1},
  [457] = {.lex_state = 23, .external_lex_state = 1},
  [458] = {.lex_state = 23, .external_lex_state = 1},
  [459] = {.lex_state = 23, .external_lex_state = 1},
  [460] = {.lex_state = 23, .external_lex_state = 1},
  [461] = {.lex_state = 23, .external_lex_state = 2},
  [462] = {.lex_state = 23, .external_lex_state = 2},
  [463] = {.lex_state = 23, .external_lex_state = 2},
  [464] = {.lex_state = 4, .external_lex_state = 1},
  [465] = {.lex_state = 4, .external_lex_state = 1},
  [466] = {.lex_state = 1, .external_lex_state = 1},
  [467] = {.lex_state = 1, .external_lex_state = 1},
  [468] = {.lex_state = 4, .external_lex_state = 1},
  [469] = {.lex_state = 4, .external_lex_state = 1},
  [470] = {.lex_state = 4, .external_lex_state = 1},
  [471] = {.lex_state = 5, .external_lex_state = 1},
  [472] = {.lex_state = 5, .external_lex_state = 1},
  [473] = {.lex_state = 1, .external_lex_state = 1},
  [474] = {.lex_state = 3, .external_lex_state = 1},
  [475] = {.lex_state = 5, .external_lex_state = 1},
  [476] = {.lex_state = 6, .external_lex_state = 1},
  [477] = {.lex_state = 3, .external_lex_state = 1},
  [478] = {.lex_state = 6, .external_lex_state = 1},
  [479] = {.lex_state = 1, .external_lex_state = 1},
  [480] = {.lex_state = 5, .external_lex_state = 1},
  [481] = {.lex_state = 5, .external_lex_state = 1},
  [482] = {.lex_state = 1, .external_lex_state = 1},
  [483] = {.lex_state = 6, .external_lex_state = 1},
  [484] = {.lex_state = 6, .external_lex_state = 1},
  [485] = {.lex_state = 3, .external_lex_state = 1},
  [486] = {.lex_state = 6, .external_lex_state = 1},
  [487] = {.lex_state = 1, .external_lex_state = 1},
  [488] = {.lex_state = 11, .external_lex_state = 2},
  [489] = {.lex_state = 23, .external_lex_state = 1},
  [490] = {.lex_state = 11, .external_lex_state = 2},
  [491] = {.lex_state = 23, .external_lex_state = 1},
  [492] = {.lex_state = 23, .external_lex_state = 1},
  [493] = {.lex_state = 2, .external_lex_state = 1},
  [494] = {.lex_state = 23, .external_lex_state = 1},
  [495] = {.lex_state = 11, .external_lex_state = 2},
  [496] = {.lex_state = 23, .external_lex_state = 1},
  [497] = {.lex_state = 23, .external_lex_state = 1},
  [498] = {.lex_state = 23, .external_lex_state = 1},
  [499] = {.lex_state = 23, .external_lex_state = 1},
  [500] = {.lex_state = 23, .external_lex_state = 1},
  [501] = {.lex_state = 23, .external_lex_state = 1},
  [502] = {.lex_state = 23, .external_lex_state = 1},
  [503] = {.lex_state = 23, .external_lex_state = 1},
  [504] = {.lex_state = 23, .external_lex_state = 2},
  [505] = {.lex_state = 23, .external_lex_state = 1},
  [506] = {.lex_state = 23, .external_lex_state = 1},
  [507] = {.lex_state = 23, .external_lex_state = 1},
  [508] = {.lex_state = 23, .external_lex_state = 1},
  [509] = {.lex_state = 23, .external_lex_state = 1},
  [510] = {.lex_state = 23, .external_lex_state = 1},
  [511] = {.lex_state = 23, .external_lex_state = 1},
  [512] = {.lex_state = 23, .external_lex_state = 1},
  [513] = {.lex_state = 23, .external_lex_state = 1},
  [514] = {.lex_state = 23, .external_lex_state = 1},
  [515] = {.lex_state = 23, .external_lex_state = 1},
  [516] = {.lex_state = 23, .external_lex_state = 1},
  [517] = {.lex_state = 23, .external_lex_state = 1},
  [518] = {.lex_state = 23, .external_lex_state = 1},
  [519] = {.lex_state = 23, .external_lex_state = 1},
  [520] = {.lex_state = 23, .external_lex_state = 2},
  [521] = {.lex_state = 23, .external_lex_state = 2},
  [522] = {.lex_state = 23, .external_lex_state = 1},
  [523] = {.lex_state = 23, .external_lex_state = 1},
  [524] = {.lex_state = 23, .external_lex_state = 1},
  [525] = {.lex_state = 23, .external_lex_state = 1},
  [526] = {.lex_state = 23, .external_lex_state = 1},
  [527] = {.lex_state = 23, .external_lex_state = 1},
  [528] = {.lex_state = 23, .external_lex_state = 1},
  [529] = {.lex_state = 23, .external_lex_state = 2},
  [530] = {.lex_state = 23, .external_lex_state = 1},
  [531] = {.lex_state = 23, .external_lex_state = 1},
  [532] = {.lex_state = 23, .external_lex_state = 1},
  [533] = {.lex_state = 23, .external_lex_state = 1},
  [534] = {.lex_state = 23, .external_lex_state = 2},
  [535] = {.lex_state = 23, .external_lex_state = 1},
  [536] = {.lex_state = 23, .external_lex_state = 1},
  [537] = {.lex_state = 23, .external_lex_state = 1},
  [538] = {.lex_state = 23, .external_lex_state = 1},
  [539] = {.lex_state = 23, .external_lex_state = 1},
  [540] = {.lex_state = 23, .external_lex_state = 1},
  [541] = {.lex_state = 23, .external_lex_state = 1},
  [542] = {.lex_state = 23, .external_lex_state = 1},
  [543] = {.lex_state = 23, .external_lex_state = 2},
  [544] = {.lex_state = 23, .external_lex_state = 1},
  [545] = {.lex_state = 23, .external_lex_state = 1},
  [546] = {.lex_state = 23, .external_lex_state = 2},
  [547] = {.lex_state = 23, .external_lex_state = 1},
  [548] = {.lex_state = 23, .external_lex_state = 1},
  [549] = {.lex_state = 23, .external_lex_state = 1},
  [550] = {.lex_state = 23, .external_lex_state = 1},
  [551] = {.lex_state = 23, .external_lex_state = 1},
  [552] = {.lex_state = 23, .external_lex_state = 1},
  [553] = {.lex_state = 23, .external_lex_state = 1},
  [554] = {.lex_state = 7, .external_lex_state = 3},
  [555] = {.lex_state = 7, .external_lex_state = 3},
  [556] = {.lex_state = 7, .external_lex_state = 3},
  [557] = {.lex_state = 7, .external_lex_state = 3},
  [558] = {.lex_state = 7, .external_lex_state = 3},
  [559] = {.lex_state = 23, .external_lex_state = 2},
  [560] = {.lex_state = 23, .external_lex_state = 2},
  [561] = {.lex_state = 23, .external_lex_state = 2},
  [562] = {.lex_state = 23, .external_lex_state = 2},
  [563] = {.lex_state = 23, .external_lex_state = 2},
  [564] = {.lex_state = 23, .external_lex_state = 2},
  [565] = {.lex_state = 23, .external_lex_state = 2},
  [566] = {.lex_state = 12, .external_lex_state = 2},
  [567] = {.lex_state = 12, .external_lex_state = 2},
  [568] = {.lex_state = 12, .external_lex_state = 2},
  [569] = {.lex_state = 12, .external_lex_state = 2},
  [570] = {.lex_state = 12, .external_lex_state = 2},
  [571] = {.lex_state = 3, .external_lex_state = 2},
  [572] = {.lex_state = 1, .external_lex_state = 2},
  [573] = {.lex_state = 3, .external_lex_state = 2},
  [574] = {.lex_state = 12, .external_lex_state = 2},
  [575] = {.lex_state = 1, .external_lex_state = 2},
  [576] = {.lex_state = 3, .external_lex_state = 2},
  [577] = {.lex_state = 23, .external_lex_state = 2},
  [578] = {.lex_state = 1, .external_lex_state = 2},
  [579] = {.lex_state = 23, .external_lex_state = 2},
  [580] = {.lex_state = 23, .external_lex_state = 2},
  [581] = {.lex_state = 1, .external_lex_state = 2},
  [582] = {.lex_state = 4, .external_lex_state = 2},
  [583] = {.lex_state = 2, .external_lex_state = 2},
  [584] = {.lex_state = 4, .external_lex_state = 2},
  [585] = {.lex_state = 23, .external_lex_state = 2},
  [586] = {.lex_state = 23, .external_lex_state = 2},
  [587] = {.lex_state = 23, .external_lex_state = 2},
  [588] = {.lex_state = 23, .external_lex_state = 2},
  [589] = {.lex_state = 23, .external_lex_state = 2},
  [590] = {.lex_state = 23, .external_lex_state = 2},
  [591] = {.lex_state = 23, .external_lex_state = 2},
  [592] = {.lex_state = 23, .external_lex_state = 2},
  [593] = {.lex_state = 23, .external_lex_state = 2},
  [594] = {.lex_state = 0, .external_lex_state = 2},
  [595] = {.lex_state = 23, .external_lex_state = 2},
  [596] = {.lex_state = 23, .external_lex_state = 2},
  [597] = {.lex_state = 23, .external_lex_state = 2},
  [598] = {.lex_state = 23, .external_lex_state = 2},
  [599] = {.lex_state = 5, .external_lex_state = 2},
  [600] = {.lex_state = 48, .external_lex_state = 2},
  [601] = {.lex_state = 44, .external_lex_state = 2},
  [602] = {.lex_state = 6, .external_lex_state = 2},
  [603] = {.lex_state = 23, .external_lex_state = 2},
  [604] = {.lex_state = 5, .external_lex_state = 2},
  [605] = {.lex_state = 23, .external_lex_state = 2},
  [606] = {.lex_state = 6, .external_lex_state = 2},
  [607] = {.lex_state = 23, .external_lex_state = 2},
  [608] = {.lex_state = 44, .external_lex_state = 2},
  [609] = {.lex_state = 48, .external_lex_state = 2},
};

static const uint16_t ts_parse_table[LARGE_STATE_COUNT][SYMBOL_COUNT] = {
  [0] = {
    [ts_builtin_sym_end] = ACTIONS(1),
    [sym__normal_bare_identifier] = ACTIONS(1),
    [anon_sym_SLASH_DASH] = ACTIONS(1),
    [anon_sym_LBRACE] = ACTIONS(1),
    [anon_sym_RBRACE] = ACTIONS(1),
    [anon_sym_SEMI] = ACTIONS(1),
    [anon_sym_null] = ACTIONS(1),
    [anon_sym_i8] = ACTIONS(1),
    [anon_sym_i16] = ACTIONS(1),
    [anon_sym_i32] = ACTIONS(1),
    [anon_sym_i64] = ACTIONS(1),
    [anon_sym_u8] = ACTIONS(1),
    [anon_sym_u16] = ACTIONS(1),
    [anon_sym_u32] = ACTIONS(1),
    [anon_sym_u64] = ACTIONS(1),
    [anon_sym_isize] = ACTIONS(1),
    [anon_sym_usize] = ACTIONS(1),
    [anon_sym_f32] = ACTIONS(1),
    [anon_sym_f64] = ACTIONS(1),
    [anon_sym_decimal64] = ACTIONS(1),
    [anon_sym_decimal128] = ACTIONS(1),
    [anon_sym_date_DASHtime] = ACTIONS(1),
    [anon_sym_time] = ACTIONS(1),
    [anon_sym_date] = ACTIONS(1),
    [anon_sym_duration] = ACTIONS(1),
    [anon_sym_decimal] = ACTIONS(1),
    [anon_sym_currency] = ACTIONS(1),
    [anon_sym_country_DASH2] = ACTIONS(1),
    [anon_sym_country_DASH3] = ACTIONS(1),
    [anon_sym_country_DASHsubdivision] = ACTIONS(1),
    [anon_sym_email] = ACTIONS(1),
    [anon_sym_idn_DASHemail] = ACTIONS(1),
    [anon_sym_hostname] = ACTIONS(1),
    [anon_sym_idn_DASHhostname] = ACTIONS(1),
    [anon_sym_ipv4] = ACTIONS(1),
    [anon_sym_ipv6] = ACTIONS(1),
    [anon_sym_url] = ACTIONS(1),
    [anon_sym_url_DASHreference] = ACTIONS(1),
    [anon_sym_irl] = ACTIONS(1),
    [anon_sym_iri_DASHreference] = ACTIONS(1),
    [anon_sym_url_DASHtemplate] = ACTIONS(1),
    [anon_sym_uuid] = ACTIONS(1),
    [anon_sym_regex] = ACTIONS(1),
    [anon_sym_base64] = ACTIONS(1),
    [anon_sym_EQ] = ACTIONS(1),
    [anon_sym_LPAREN] = ACTIONS(1),
    [anon_sym_RPAREN] = ACTIONS(1),
    [anon_sym_DQUOTE] = ACTIONS(1),
    [sym__hex_digit] = ACTIONS(1),
    [aux_sym__raw_string_token1] = ACTIONS(1),
    [anon_sym_POUND] = ACTIONS(1),
    [aux_sym__raw_string_token3] = ACTIONS(1),
    [anon_sym_DOT] = ACTIONS(1),
    [anon_sym_e] = ACTIONS(1),
    [anon_sym_E] = ACTIONS(1),
    [anon_sym__] = ACTIONS(1),
    [sym__digit] = ACTIONS(1),
    [anon_sym_PLUS] = ACTIONS(1),
    [anon_sym_DASH] = ACTIONS(1),
    [anon_sym_0x] = ACTIONS(1),
    [anon_sym_0o] = ACTIONS(1),
    [aux_sym__octal_token1] = ACTIONS(1),
    [anon_sym_0] = ACTIONS(1),
    [anon_sym_1] = ACTIONS(1),
    [anon_sym_true] = ACTIONS(1),
    [anon_sym_false] = ACTIONS(1),
    [anon_sym_BSLASH] = ACTIONS(1),
    [aux_sym__newline_token1] = ACTIONS(1),
    [aux_sym__newline_token2] = ACTIONS(1),
    [aux_sym__newline_token3] = ACTIONS(1),
    [aux_sym__newline_token4] = ACTIONS(1),
    [aux_sym__newline_token5] = ACTIONS(1),
    [aux_sym__newline_token6] = ACTIONS(1),
    [aux_sym__newline_token7] = ACTIONS(1),
    [sym__bom] = ACTIONS(1),
    [sym__unicode_space] = ACTIONS(1),
    [anon_sym_SLASH_SLASH] = ACTIONS(1),
    [aux_sym_single_line_comment_token1] = ACTIONS(1),
    [anon_sym_expression] = ACTIONS(1),
    [anon_sym_minimize] = ACTIONS(1),
    [anon_sym_maximize] = ACTIONS(1),
    [anon_sym_expr] = ACTIONS(1),
    [anon_sym_filter] = ACTIONS(1),
    [anon_sym_if] = ACTIONS(1),
    [anon_sym_lower] = ACTIONS(1),
    [anon_sym_upper] = ACTIONS(1),
    [anon_sym_constraint] = ACTIONS(1),
    [sym__eof] = ACTIONS(1),
    [sym_multi_line_comment] = ACTIONS(3),
    [sym__implicit_terminator] = ACTIONS(1),
  },
  [1] = {
    [sym_document] = STATE(594),
    [sym_node] = STATE(34),
    [sym_identifier] = STATE(294),
    [sym__bare_identifier] = STATE(510),
    [sym_type] = STATE(461),
    [sym_string] = STATE(510),
    [sym__escaped_string] = STATE(491),
    [sym__raw_string] = STATE(491),
    [sym__sign] = STATE(493),
    [sym__linespace] = STATE(64),
    [sym__newline] = STATE(64),
    [sym__ws] = STATE(64),
    [sym_single_line_comment] = STATE(64),
    [sym_kdl_node] = STATE(149),
    [sym_arco_pure_math_node] = STATE(149),
    [sym_arco_constraint_node] = STATE(149),
    [aux_sym_document_repeat1] = STATE(64),
    [ts_builtin_sym_end] = ACTIONS(5),
    [sym__normal_bare_identifier] = ACTIONS(7),
    [anon_sym_SLASH_DASH] = ACTIONS(9),
    [anon_sym_LPAREN] = ACTIONS(11),
    [anon_sym_DQUOTE] = ACTIONS(13),
    [aux_sym__raw_string_token1] = ACTIONS(15),
    [aux_sym__raw_string_token3] = ACTIONS(17),
    [anon_sym_PLUS] = ACTIONS(19),
    [anon_sym_DASH] = ACTIONS(19),
    [aux_sym__newline_token1] = ACTIONS(21),
    [aux_sym__newline_token2] = ACTIONS(21),
    [aux_sym__newline_token3] = ACTIONS(21),
    [aux_sym__newline_token4] = ACTIONS(21),
    [aux_sym__newline_token5] = ACTIONS(21),
    [aux_sym__newline_token6] = ACTIONS(21),
    [aux_sym__newline_token7] = ACTIONS(21),
    [sym__bom] = ACTIONS(21),
    [sym__unicode_space] = ACTIONS(21),
    [anon_sym_SLASH_SLASH] = ACTIONS(23),
    [anon_sym_expression] = ACTIONS(25),
    [anon_sym_minimize] = ACTIONS(25),
    [anon_sym_maximize] = ACTIONS(25),
    [anon_sym_expr] = ACTIONS(25),
    [anon_sym_filter] = ACTIONS(25),
    [anon_sym_if] = ACTIONS(25),
    [anon_sym_lower] = ACTIONS(25),
    [anon_sym_upper] = ACTIONS(25),
    [anon_sym_constraint] = ACTIONS(27),
    [sym_multi_line_comment] = ACTIONS(21),
  },
};

static const uint16_t ts_small_parse_table[] = {
  [0] = 37,
    ACTIONS(11), 1,
      anon_sym_LPAREN,
    ACTIONS(13), 1,
      anon_sym_DQUOTE,
    ACTIONS(15), 1,
      aux_sym__raw_string_token1,
    ACTIONS(17), 1,
      aux_sym__raw_string_token3,
    ACTIONS(23), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(29), 1,
      sym__normal_bare_identifier,
    ACTIONS(31), 1,
      anon_sym_SLASH_DASH,
    ACTIONS(33), 1,
      anon_sym_LBRACE,
    ACTIONS(37), 1,
      anon_sym_null,
    ACTIONS(39), 1,
      sym__digit,
    ACTIONS(43), 1,
      anon_sym_0x,
    ACTIONS(45), 1,
      anon_sym_0o,
    ACTIONS(47), 1,
      anon_sym_0b,
    ACTIONS(51), 1,
      anon_sym_BSLASH,
    STATE(81), 1,
      aux_sym__node_field_comment_repeat1,
    STATE(93), 1,
      sym__escline,
    STATE(102), 1,
      sym__node_space,
    STATE(270), 1,
      sym_type,
    STATE(299), 1,
      sym__sign,
    STATE(467), 1,
      sym__integer,
    STATE(492), 1,
      sym__bare_identifier,
    STATE(497), 1,
      sym_string,
    STATE(507), 1,
      sym_boolean,
    STATE(518), 1,
      sym_node_field,
    STATE(587), 1,
      sym_identifier,
    ACTIONS(41), 2,
      anon_sym_PLUS,
      anon_sym_DASH,
    ACTIONS(49), 2,
      anon_sym_true,
      anon_sym_false,
    STATE(84), 2,
      sym__ws,
      aux_sym__node_space_repeat1,
    STATE(426), 2,
      sym_node_children,
      sym_arco_constraint_math_children,
    STATE(491), 2,
      sym__escaped_string,
      sym__raw_string,
    STATE(506), 2,
      sym_prop,
      sym_value,
    STATE(516), 2,
      sym__node_field_comment,
      sym__node_field,
    ACTIONS(53), 3,
      sym_multi_line_comment,
      sym__bom,
      sym__unicode_space,
    STATE(201), 3,
      sym__node_terminator,
      sym__newline,
      sym_single_line_comment,
    STATE(508), 3,
      sym_keyword,
      sym_number,
      sym_bare_identifier,
    STATE(501), 4,
      sym__decimal,
      sym__hex,
      sym__octal,
      sym__binary,
    ACTIONS(35), 10,
      sym__eof,
      sym__implicit_terminator,
      anon_sym_SEMI,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
  [137] = 37,
    ACTIONS(11), 1,
      anon_sym_LPAREN,
    ACTIONS(13), 1,
      anon_sym_DQUOTE,
    ACTIONS(15), 1,
      aux_sym__raw_string_token1,
    ACTIONS(17), 1,
      aux_sym__raw_string_token3,
    ACTIONS(23), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(29), 1,
      sym__normal_bare_identifier,
    ACTIONS(31), 1,
      anon_sym_SLASH_DASH,
    ACTIONS(33), 1,
      anon_sym_LBRACE,
    ACTIONS(37), 1,
      anon_sym_null,
    ACTIONS(39), 1,
      sym__digit,
    ACTIONS(43), 1,
      anon_sym_0x,
    ACTIONS(45), 1,
      anon_sym_0o,
    ACTIONS(47), 1,
      anon_sym_0b,
    ACTIONS(51), 1,
      anon_sym_BSLASH,
    STATE(81), 1,
      aux_sym__node_field_comment_repeat1,
    STATE(93), 1,
      sym__escline,
    STATE(102), 1,
      sym__node_space,
    STATE(270), 1,
      sym_type,
    STATE(299), 1,
      sym__sign,
    STATE(467), 1,
      sym__integer,
    STATE(492), 1,
      sym__bare_identifier,
    STATE(497), 1,
      sym_string,
    STATE(507), 1,
      sym_boolean,
    STATE(518), 1,
      sym_node_field,
    STATE(587), 1,
      sym_identifier,
    ACTIONS(41), 2,
      anon_sym_PLUS,
      anon_sym_DASH,
    ACTIONS(49), 2,
      anon_sym_true,
      anon_sym_false,
    STATE(84), 2,
      sym__ws,
      aux_sym__node_space_repeat1,
    STATE(348), 2,
      sym_node_children,
      sym_arco_constraint_math_children,
    STATE(491), 2,
      sym__escaped_string,
      sym__raw_string,
    STATE(506), 2,
      sym_prop,
      sym_value,
    STATE(516), 2,
      sym__node_field_comment,
      sym__node_field,
    ACTIONS(53), 3,
      sym_multi_line_comment,
      sym__bom,
      sym__unicode_space,
    STATE(229), 3,
      sym__node_terminator,
      sym__newline,
      sym_single_line_comment,
    STATE(508), 3,
      sym_keyword,
      sym_number,
      sym_bare_identifier,
    STATE(501), 4,
      sym__decimal,
      sym__hex,
      sym__octal,
      sym__binary,
    ACTIONS(55), 10,
      sym__eof,
      sym__implicit_terminator,
      anon_sym_SEMI,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
  [274] = 37,
    ACTIONS(11), 1,
      anon_sym_LPAREN,
    ACTIONS(13), 1,
      anon_sym_DQUOTE,
    ACTIONS(15), 1,
      aux_sym__raw_string_token1,
    ACTIONS(17), 1,
      aux_sym__raw_string_token3,
    ACTIONS(23), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(29), 1,
      sym__normal_bare_identifier,
    ACTIONS(31), 1,
      anon_sym_SLASH_DASH,
    ACTIONS(33), 1,
      anon_sym_LBRACE,
    ACTIONS(37), 1,
      anon_sym_null,
    ACTIONS(39), 1,
      sym__digit,
    ACTIONS(43), 1,
      anon_sym_0x,
    ACTIONS(45), 1,
      anon_sym_0o,
    ACTIONS(47), 1,
      anon_sym_0b,
    ACTIONS(51), 1,
      anon_sym_BSLASH,
    STATE(81), 1,
      aux_sym__node_field_comment_repeat1,
    STATE(93), 1,
      sym__escline,
    STATE(102), 1,
      sym__node_space,
    STATE(270), 1,
      sym_type,
    STATE(299), 1,
      sym__sign,
    STATE(467), 1,
      sym__integer,
    STATE(492), 1,
      sym__bare_identifier,
    STATE(497), 1,
      sym_string,
    STATE(507), 1,
      sym_boolean,
    STATE(518), 1,
      sym_node_field,
    STATE(587), 1,
      sym_identifier,
    ACTIONS(41), 2,
      anon_sym_PLUS,
      anon_sym_DASH,
    ACTIONS(49), 2,
      anon_sym_true,
      anon_sym_false,
    STATE(84), 2,
      sym__ws,
      aux_sym__node_space_repeat1,
    STATE(437), 2,
      sym_node_children,
      sym_arco_constraint_math_children,
    STATE(491), 2,
      sym__escaped_string,
      sym__raw_string,
    STATE(506), 2,
      sym_prop,
      sym_value,
    STATE(516), 2,
      sym__node_field_comment,
      sym__node_field,
    ACTIONS(53), 3,
      sym_multi_line_comment,
      sym__bom,
      sym__unicode_space,
    STATE(134), 3,
      sym__node_terminator,
      sym__newline,
      sym_single_line_comment,
    STATE(508), 3,
      sym_keyword,
      sym_number,
      sym_bare_identifier,
    STATE(501), 4,
      sym__decimal,
      sym__hex,
      sym__octal,
      sym__binary,
    ACTIONS(57), 10,
      sym__eof,
      sym__implicit_terminator,
      anon_sym_SEMI,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
  [411] = 37,
    ACTIONS(11), 1,
      anon_sym_LPAREN,
    ACTIONS(13), 1,
      anon_sym_DQUOTE,
    ACTIONS(15), 1,
      aux_sym__raw_string_token1,
    ACTIONS(17), 1,
      aux_sym__raw_string_token3,
    ACTIONS(23), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(29), 1,
      sym__normal_bare_identifier,
    ACTIONS(31), 1,
      anon_sym_SLASH_DASH,
    ACTIONS(33), 1,
      anon_sym_LBRACE,
    ACTIONS(37), 1,
      anon_sym_null,
    ACTIONS(39), 1,
      sym__digit,
    ACTIONS(43), 1,
      anon_sym_0x,
    ACTIONS(45), 1,
      anon_sym_0o,
    ACTIONS(47), 1,
      anon_sym_0b,
    ACTIONS(51), 1,
      anon_sym_BSLASH,
    STATE(81), 1,
      aux_sym__node_field_comment_repeat1,
    STATE(93), 1,
      sym__escline,
    STATE(102), 1,
      sym__node_space,
    STATE(270), 1,
      sym_type,
    STATE(299), 1,
      sym__sign,
    STATE(467), 1,
      sym__integer,
    STATE(492), 1,
      sym__bare_identifier,
    STATE(497), 1,
      sym_string,
    STATE(507), 1,
      sym_boolean,
    STATE(518), 1,
      sym_node_field,
    STATE(587), 1,
      sym_identifier,
    ACTIONS(41), 2,
      anon_sym_PLUS,
      anon_sym_DASH,
    ACTIONS(49), 2,
      anon_sym_true,
      anon_sym_false,
    STATE(84), 2,
      sym__ws,
      aux_sym__node_space_repeat1,
    STATE(365), 2,
      sym_node_children,
      sym_arco_constraint_math_children,
    STATE(491), 2,
      sym__escaped_string,
      sym__raw_string,
    STATE(506), 2,
      sym_prop,
      sym_value,
    STATE(516), 2,
      sym__node_field_comment,
      sym__node_field,
    ACTIONS(53), 3,
      sym_multi_line_comment,
      sym__bom,
      sym__unicode_space,
    STATE(245), 3,
      sym__node_terminator,
      sym__newline,
      sym_single_line_comment,
    STATE(508), 3,
      sym_keyword,
      sym_number,
      sym_bare_identifier,
    STATE(501), 4,
      sym__decimal,
      sym__hex,
      sym__octal,
      sym__binary,
    ACTIONS(59), 10,
      sym__eof,
      sym__implicit_terminator,
      anon_sym_SEMI,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
  [548] = 37,
    ACTIONS(11), 1,
      anon_sym_LPAREN,
    ACTIONS(13), 1,
      anon_sym_DQUOTE,
    ACTIONS(15), 1,
      aux_sym__raw_string_token1,
    ACTIONS(17), 1,
      aux_sym__raw_string_token3,
    ACTIONS(23), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(29), 1,
      sym__normal_bare_identifier,
    ACTIONS(31), 1,
      anon_sym_SLASH_DASH,
    ACTIONS(33), 1,
      anon_sym_LBRACE,
    ACTIONS(37), 1,
      anon_sym_null,
    ACTIONS(39), 1,
      sym__digit,
    ACTIONS(43), 1,
      anon_sym_0x,
    ACTIONS(45), 1,
      anon_sym_0o,
    ACTIONS(47), 1,
      anon_sym_0b,
    ACTIONS(51), 1,
      anon_sym_BSLASH,
    STATE(81), 1,
      aux_sym__node_field_comment_repeat1,
    STATE(93), 1,
      sym__escline,
    STATE(102), 1,
      sym__node_space,
    STATE(270), 1,
      sym_type,
    STATE(299), 1,
      sym__sign,
    STATE(467), 1,
      sym__integer,
    STATE(492), 1,
      sym__bare_identifier,
    STATE(497), 1,
      sym_string,
    STATE(507), 1,
      sym_boolean,
    STATE(518), 1,
      sym_node_field,
    STATE(587), 1,
      sym_identifier,
    ACTIONS(41), 2,
      anon_sym_PLUS,
      anon_sym_DASH,
    ACTIONS(49), 2,
      anon_sym_true,
      anon_sym_false,
    STATE(84), 2,
      sym__ws,
      aux_sym__node_space_repeat1,
    STATE(408), 2,
      sym_node_children,
      sym_arco_constraint_math_children,
    STATE(491), 2,
      sym__escaped_string,
      sym__raw_string,
    STATE(506), 2,
      sym_prop,
      sym_value,
    STATE(516), 2,
      sym__node_field_comment,
      sym__node_field,
    ACTIONS(53), 3,
      sym_multi_line_comment,
      sym__bom,
      sym__unicode_space,
    STATE(211), 3,
      sym__node_terminator,
      sym__newline,
      sym_single_line_comment,
    STATE(508), 3,
      sym_keyword,
      sym_number,
      sym_bare_identifier,
    STATE(501), 4,
      sym__decimal,
      sym__hex,
      sym__octal,
      sym__binary,
    ACTIONS(61), 10,
      sym__eof,
      sym__implicit_terminator,
      anon_sym_SEMI,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
  [685] = 37,
    ACTIONS(11), 1,
      anon_sym_LPAREN,
    ACTIONS(13), 1,
      anon_sym_DQUOTE,
    ACTIONS(15), 1,
      aux_sym__raw_string_token1,
    ACTIONS(17), 1,
      aux_sym__raw_string_token3,
    ACTIONS(23), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(29), 1,
      sym__normal_bare_identifier,
    ACTIONS(31), 1,
      anon_sym_SLASH_DASH,
    ACTIONS(33), 1,
      anon_sym_LBRACE,
    ACTIONS(37), 1,
      anon_sym_null,
    ACTIONS(39), 1,
      sym__digit,
    ACTIONS(43), 1,
      anon_sym_0x,
    ACTIONS(45), 1,
      anon_sym_0o,
    ACTIONS(47), 1,
      anon_sym_0b,
    ACTIONS(51), 1,
      anon_sym_BSLASH,
    STATE(81), 1,
      aux_sym__node_field_comment_repeat1,
    STATE(93), 1,
      sym__escline,
    STATE(102), 1,
      sym__node_space,
    STATE(270), 1,
      sym_type,
    STATE(299), 1,
      sym__sign,
    STATE(467), 1,
      sym__integer,
    STATE(492), 1,
      sym__bare_identifier,
    STATE(497), 1,
      sym_string,
    STATE(507), 1,
      sym_boolean,
    STATE(518), 1,
      sym_node_field,
    STATE(587), 1,
      sym_identifier,
    ACTIONS(41), 2,
      anon_sym_PLUS,
      anon_sym_DASH,
    ACTIONS(49), 2,
      anon_sym_true,
      anon_sym_false,
    STATE(84), 2,
      sym__ws,
      aux_sym__node_space_repeat1,
    STATE(310), 2,
      sym_node_children,
      sym_arco_constraint_math_children,
    STATE(491), 2,
      sym__escaped_string,
      sym__raw_string,
    STATE(506), 2,
      sym_prop,
      sym_value,
    STATE(516), 2,
      sym__node_field_comment,
      sym__node_field,
    ACTIONS(53), 3,
      sym_multi_line_comment,
      sym__bom,
      sym__unicode_space,
    STATE(180), 3,
      sym__node_terminator,
      sym__newline,
      sym_single_line_comment,
    STATE(508), 3,
      sym_keyword,
      sym_number,
      sym_bare_identifier,
    STATE(501), 4,
      sym__decimal,
      sym__hex,
      sym__octal,
      sym__binary,
    ACTIONS(63), 10,
      sym__eof,
      sym__implicit_terminator,
      anon_sym_SEMI,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
  [822] = 37,
    ACTIONS(11), 1,
      anon_sym_LPAREN,
    ACTIONS(13), 1,
      anon_sym_DQUOTE,
    ACTIONS(15), 1,
      aux_sym__raw_string_token1,
    ACTIONS(17), 1,
      aux_sym__raw_string_token3,
    ACTIONS(23), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(29), 1,
      sym__normal_bare_identifier,
    ACTIONS(31), 1,
      anon_sym_SLASH_DASH,
    ACTIONS(33), 1,
      anon_sym_LBRACE,
    ACTIONS(37), 1,
      anon_sym_null,
    ACTIONS(39), 1,
      sym__digit,
    ACTIONS(43), 1,
      anon_sym_0x,
    ACTIONS(45), 1,
      anon_sym_0o,
    ACTIONS(47), 1,
      anon_sym_0b,
    ACTIONS(51), 1,
      anon_sym_BSLASH,
    STATE(81), 1,
      aux_sym__node_field_comment_repeat1,
    STATE(93), 1,
      sym__escline,
    STATE(102), 1,
      sym__node_space,
    STATE(270), 1,
      sym_type,
    STATE(299), 1,
      sym__sign,
    STATE(467), 1,
      sym__integer,
    STATE(492), 1,
      sym__bare_identifier,
    STATE(497), 1,
      sym_string,
    STATE(507), 1,
      sym_boolean,
    STATE(518), 1,
      sym_node_field,
    STATE(587), 1,
      sym_identifier,
    ACTIONS(41), 2,
      anon_sym_PLUS,
      anon_sym_DASH,
    ACTIONS(49), 2,
      anon_sym_true,
      anon_sym_false,
    STATE(84), 2,
      sym__ws,
      aux_sym__node_space_repeat1,
    STATE(392), 2,
      sym_node_children,
      sym_arco_constraint_math_children,
    STATE(491), 2,
      sym__escaped_string,
      sym__raw_string,
    STATE(506), 2,
      sym_prop,
      sym_value,
    STATE(516), 2,
      sym__node_field_comment,
      sym__node_field,
    ACTIONS(53), 3,
      sym_multi_line_comment,
      sym__bom,
      sym__unicode_space,
    STATE(218), 3,
      sym__node_terminator,
      sym__newline,
      sym_single_line_comment,
    STATE(508), 3,
      sym_keyword,
      sym_number,
      sym_bare_identifier,
    STATE(501), 4,
      sym__decimal,
      sym__hex,
      sym__octal,
      sym__binary,
    ACTIONS(65), 10,
      sym__eof,
      sym__implicit_terminator,
      anon_sym_SEMI,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
  [959] = 37,
    ACTIONS(11), 1,
      anon_sym_LPAREN,
    ACTIONS(13), 1,
      anon_sym_DQUOTE,
    ACTIONS(15), 1,
      aux_sym__raw_string_token1,
    ACTIONS(17), 1,
      aux_sym__raw_string_token3,
    ACTIONS(23), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(29), 1,
      sym__normal_bare_identifier,
    ACTIONS(31), 1,
      anon_sym_SLASH_DASH,
    ACTIONS(33), 1,
      anon_sym_LBRACE,
    ACTIONS(37), 1,
      anon_sym_null,
    ACTIONS(39), 1,
      sym__digit,
    ACTIONS(43), 1,
      anon_sym_0x,
    ACTIONS(45), 1,
      anon_sym_0o,
    ACTIONS(47), 1,
      anon_sym_0b,
    ACTIONS(51), 1,
      anon_sym_BSLASH,
    STATE(81), 1,
      aux_sym__node_field_comment_repeat1,
    STATE(93), 1,
      sym__escline,
    STATE(102), 1,
      sym__node_space,
    STATE(270), 1,
      sym_type,
    STATE(299), 1,
      sym__sign,
    STATE(467), 1,
      sym__integer,
    STATE(492), 1,
      sym__bare_identifier,
    STATE(497), 1,
      sym_string,
    STATE(507), 1,
      sym_boolean,
    STATE(518), 1,
      sym_node_field,
    STATE(587), 1,
      sym_identifier,
    ACTIONS(41), 2,
      anon_sym_PLUS,
      anon_sym_DASH,
    ACTIONS(49), 2,
      anon_sym_true,
      anon_sym_false,
    STATE(84), 2,
      sym__ws,
      aux_sym__node_space_repeat1,
    STATE(380), 2,
      sym_node_children,
      sym_arco_constraint_math_children,
    STATE(491), 2,
      sym__escaped_string,
      sym__raw_string,
    STATE(506), 2,
      sym_prop,
      sym_value,
    STATE(516), 2,
      sym__node_field_comment,
      sym__node_field,
    ACTIONS(53), 3,
      sym_multi_line_comment,
      sym__bom,
      sym__unicode_space,
    STATE(116), 3,
      sym__node_terminator,
      sym__newline,
      sym_single_line_comment,
    STATE(508), 3,
      sym_keyword,
      sym_number,
      sym_bare_identifier,
    STATE(501), 4,
      sym__decimal,
      sym__hex,
      sym__octal,
      sym__binary,
    ACTIONS(67), 10,
      sym__eof,
      sym__implicit_terminator,
      anon_sym_SEMI,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
  [1096] = 37,
    ACTIONS(11), 1,
      anon_sym_LPAREN,
    ACTIONS(13), 1,
      anon_sym_DQUOTE,
    ACTIONS(15), 1,
      aux_sym__raw_string_token1,
    ACTIONS(17), 1,
      aux_sym__raw_string_token3,
    ACTIONS(23), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(29), 1,
      sym__normal_bare_identifier,
    ACTIONS(31), 1,
      anon_sym_SLASH_DASH,
    ACTIONS(33), 1,
      anon_sym_LBRACE,
    ACTIONS(37), 1,
      anon_sym_null,
    ACTIONS(39), 1,
      sym__digit,
    ACTIONS(43), 1,
      anon_sym_0x,
    ACTIONS(45), 1,
      anon_sym_0o,
    ACTIONS(47), 1,
      anon_sym_0b,
    ACTIONS(51), 1,
      anon_sym_BSLASH,
    STATE(81), 1,
      aux_sym__node_field_comment_repeat1,
    STATE(93), 1,
      sym__escline,
    STATE(102), 1,
      sym__node_space,
    STATE(270), 1,
      sym_type,
    STATE(299), 1,
      sym__sign,
    STATE(467), 1,
      sym__integer,
    STATE(492), 1,
      sym__bare_identifier,
    STATE(497), 1,
      sym_string,
    STATE(507), 1,
      sym_boolean,
    STATE(518), 1,
      sym_node_field,
    STATE(587), 1,
      sym_identifier,
    ACTIONS(41), 2,
      anon_sym_PLUS,
      anon_sym_DASH,
    ACTIONS(49), 2,
      anon_sym_true,
      anon_sym_false,
    STATE(84), 2,
      sym__ws,
      aux_sym__node_space_repeat1,
    STATE(316), 2,
      sym_node_children,
      sym_arco_constraint_math_children,
    STATE(491), 2,
      sym__escaped_string,
      sym__raw_string,
    STATE(506), 2,
      sym_prop,
      sym_value,
    STATE(516), 2,
      sym__node_field_comment,
      sym__node_field,
    ACTIONS(53), 3,
      sym_multi_line_comment,
      sym__bom,
      sym__unicode_space,
    STATE(120), 3,
      sym__node_terminator,
      sym__newline,
      sym_single_line_comment,
    STATE(508), 3,
      sym_keyword,
      sym_number,
      sym_bare_identifier,
    STATE(501), 4,
      sym__decimal,
      sym__hex,
      sym__octal,
      sym__binary,
    ACTIONS(69), 10,
      sym__eof,
      sym__implicit_terminator,
      anon_sym_SEMI,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
  [1233] = 37,
    ACTIONS(11), 1,
      anon_sym_LPAREN,
    ACTIONS(13), 1,
      anon_sym_DQUOTE,
    ACTIONS(15), 1,
      aux_sym__raw_string_token1,
    ACTIONS(17), 1,
      aux_sym__raw_string_token3,
    ACTIONS(23), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(29), 1,
      sym__normal_bare_identifier,
    ACTIONS(31), 1,
      anon_sym_SLASH_DASH,
    ACTIONS(33), 1,
      anon_sym_LBRACE,
    ACTIONS(37), 1,
      anon_sym_null,
    ACTIONS(39), 1,
      sym__digit,
    ACTIONS(43), 1,
      anon_sym_0x,
    ACTIONS(45), 1,
      anon_sym_0o,
    ACTIONS(47), 1,
      anon_sym_0b,
    ACTIONS(51), 1,
      anon_sym_BSLASH,
    STATE(81), 1,
      aux_sym__node_field_comment_repeat1,
    STATE(93), 1,
      sym__escline,
    STATE(102), 1,
      sym__node_space,
    STATE(270), 1,
      sym_type,
    STATE(299), 1,
      sym__sign,
    STATE(467), 1,
      sym__integer,
    STATE(492), 1,
      sym__bare_identifier,
    STATE(497), 1,
      sym_string,
    STATE(507), 1,
      sym_boolean,
    STATE(518), 1,
      sym_node_field,
    STATE(587), 1,
      sym_identifier,
    ACTIONS(41), 2,
      anon_sym_PLUS,
      anon_sym_DASH,
    ACTIONS(49), 2,
      anon_sym_true,
      anon_sym_false,
    STATE(84), 2,
      sym__ws,
      aux_sym__node_space_repeat1,
    STATE(449), 2,
      sym_node_children,
      sym_arco_constraint_math_children,
    STATE(491), 2,
      sym__escaped_string,
      sym__raw_string,
    STATE(506), 2,
      sym_prop,
      sym_value,
    STATE(516), 2,
      sym__node_field_comment,
      sym__node_field,
    ACTIONS(53), 3,
      sym_multi_line_comment,
      sym__bom,
      sym__unicode_space,
    STATE(160), 3,
      sym__node_terminator,
      sym__newline,
      sym_single_line_comment,
    STATE(508), 3,
      sym_keyword,
      sym_number,
      sym_bare_identifier,
    STATE(501), 4,
      sym__decimal,
      sym__hex,
      sym__octal,
      sym__binary,
    ACTIONS(71), 10,
      sym__eof,
      sym__implicit_terminator,
      anon_sym_SEMI,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
  [1370] = 37,
    ACTIONS(11), 1,
      anon_sym_LPAREN,
    ACTIONS(13), 1,
      anon_sym_DQUOTE,
    ACTIONS(15), 1,
      aux_sym__raw_string_token1,
    ACTIONS(17), 1,
      aux_sym__raw_string_token3,
    ACTIONS(23), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(29), 1,
      sym__normal_bare_identifier,
    ACTIONS(37), 1,
      anon_sym_null,
    ACTIONS(39), 1,
      sym__digit,
    ACTIONS(43), 1,
      anon_sym_0x,
    ACTIONS(45), 1,
      anon_sym_0o,
    ACTIONS(47), 1,
      anon_sym_0b,
    ACTIONS(51), 1,
      anon_sym_BSLASH,
    ACTIONS(73), 1,
      anon_sym_SLASH_DASH,
    ACTIONS(75), 1,
      anon_sym_LBRACE,
    STATE(81), 1,
      aux_sym__node_field_comment_repeat1,
    STATE(93), 1,
      sym__escline,
    STATE(102), 1,
      sym__node_space,
    STATE(270), 1,
      sym_type,
    STATE(299), 1,
      sym__sign,
    STATE(377), 1,
      sym_node_children,
    STATE(467), 1,
      sym__integer,
    STATE(492), 1,
      sym__bare_identifier,
    STATE(497), 1,
      sym_string,
    STATE(507), 1,
      sym_boolean,
    STATE(518), 1,
      sym_node_field,
    STATE(587), 1,
      sym_identifier,
    ACTIONS(41), 2,
      anon_sym_PLUS,
      anon_sym_DASH,
    ACTIONS(49), 2,
      anon_sym_true,
      anon_sym_false,
    STATE(84), 2,
      sym__ws,
      aux_sym__node_space_repeat1,
    STATE(491), 2,
      sym__escaped_string,
      sym__raw_string,
    STATE(506), 2,
      sym_prop,
      sym_value,
    STATE(516), 2,
      sym__node_field_comment,
      sym__node_field,
    ACTIONS(53), 3,
      sym_multi_line_comment,
      sym__bom,
      sym__unicode_space,
    STATE(114), 3,
      sym__node_terminator,
      sym__newline,
      sym_single_line_comment,
    STATE(508), 3,
      sym_keyword,
      sym_number,
      sym_bare_identifier,
    STATE(501), 4,
      sym__decimal,
      sym__hex,
      sym__octal,
      sym__binary,
    ACTIONS(77), 10,
      sym__eof,
      sym__implicit_terminator,
      anon_sym_SEMI,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
  [1506] = 37,
    ACTIONS(11), 1,
      anon_sym_LPAREN,
    ACTIONS(13), 1,
      anon_sym_DQUOTE,
    ACTIONS(15), 1,
      aux_sym__raw_string_token1,
    ACTIONS(17), 1,
      aux_sym__raw_string_token3,
    ACTIONS(23), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(29), 1,
      sym__normal_bare_identifier,
    ACTIONS(37), 1,
      anon_sym_null,
    ACTIONS(39), 1,
      sym__digit,
    ACTIONS(43), 1,
      anon_sym_0x,
    ACTIONS(45), 1,
      anon_sym_0o,
    ACTIONS(47), 1,
      anon_sym_0b,
    ACTIONS(51), 1,
      anon_sym_BSLASH,
    ACTIONS(73), 1,
      anon_sym_SLASH_DASH,
    ACTIONS(75), 1,
      anon_sym_LBRACE,
    STATE(81), 1,
      aux_sym__node_field_comment_repeat1,
    STATE(93), 1,
      sym__escline,
    STATE(102), 1,
      sym__node_space,
    STATE(270), 1,
      sym_type,
    STATE(299), 1,
      sym__sign,
    STATE(319), 1,
      sym_node_children,
    STATE(467), 1,
      sym__integer,
    STATE(492), 1,
      sym__bare_identifier,
    STATE(497), 1,
      sym_string,
    STATE(507), 1,
      sym_boolean,
    STATE(518), 1,
      sym_node_field,
    STATE(587), 1,
      sym_identifier,
    ACTIONS(41), 2,
      anon_sym_PLUS,
      anon_sym_DASH,
    ACTIONS(49), 2,
      anon_sym_true,
      anon_sym_false,
    STATE(84), 2,
      sym__ws,
      aux_sym__node_space_repeat1,
    STATE(491), 2,
      sym__escaped_string,
      sym__raw_string,
    STATE(506), 2,
      sym_prop,
      sym_value,
    STATE(516), 2,
      sym__node_field_comment,
      sym__node_field,
    ACTIONS(53), 3,
      sym_multi_line_comment,
      sym__bom,
      sym__unicode_space,
    STATE(188), 3,
      sym__node_terminator,
      sym__newline,
      sym_single_line_comment,
    STATE(508), 3,
      sym_keyword,
      sym_number,
      sym_bare_identifier,
    STATE(501), 4,
      sym__decimal,
      sym__hex,
      sym__octal,
      sym__binary,
    ACTIONS(79), 10,
      sym__eof,
      sym__implicit_terminator,
      anon_sym_SEMI,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
  [1642] = 37,
    ACTIONS(11), 1,
      anon_sym_LPAREN,
    ACTIONS(13), 1,
      anon_sym_DQUOTE,
    ACTIONS(15), 1,
      aux_sym__raw_string_token1,
    ACTIONS(17), 1,
      aux_sym__raw_string_token3,
    ACTIONS(23), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(29), 1,
      sym__normal_bare_identifier,
    ACTIONS(37), 1,
      anon_sym_null,
    ACTIONS(39), 1,
      sym__digit,
    ACTIONS(43), 1,
      anon_sym_0x,
    ACTIONS(45), 1,
      anon_sym_0o,
    ACTIONS(47), 1,
      anon_sym_0b,
    ACTIONS(51), 1,
      anon_sym_BSLASH,
    ACTIONS(73), 1,
      anon_sym_SLASH_DASH,
    ACTIONS(75), 1,
      anon_sym_LBRACE,
    STATE(81), 1,
      aux_sym__node_field_comment_repeat1,
    STATE(93), 1,
      sym__escline,
    STATE(102), 1,
      sym__node_space,
    STATE(270), 1,
      sym_type,
    STATE(299), 1,
      sym__sign,
    STATE(458), 1,
      sym_node_children,
    STATE(467), 1,
      sym__integer,
    STATE(492), 1,
      sym__bare_identifier,
    STATE(497), 1,
      sym_string,
    STATE(507), 1,
      sym_boolean,
    STATE(518), 1,
      sym_node_field,
    STATE(587), 1,
      sym_identifier,
    ACTIONS(41), 2,
      anon_sym_PLUS,
      anon_sym_DASH,
    ACTIONS(49), 2,
      anon_sym_true,
      anon_sym_false,
    STATE(84), 2,
      sym__ws,
      aux_sym__node_space_repeat1,
    STATE(491), 2,
      sym__escaped_string,
      sym__raw_string,
    STATE(506), 2,
      sym_prop,
      sym_value,
    STATE(516), 2,
      sym__node_field_comment,
      sym__node_field,
    ACTIONS(53), 3,
      sym_multi_line_comment,
      sym__bom,
      sym__unicode_space,
    STATE(166), 3,
      sym__node_terminator,
      sym__newline,
      sym_single_line_comment,
    STATE(508), 3,
      sym_keyword,
      sym_number,
      sym_bare_identifier,
    STATE(501), 4,
      sym__decimal,
      sym__hex,
      sym__octal,
      sym__binary,
    ACTIONS(81), 10,
      sym__eof,
      sym__implicit_terminator,
      anon_sym_SEMI,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
  [1778] = 37,
    ACTIONS(11), 1,
      anon_sym_LPAREN,
    ACTIONS(13), 1,
      anon_sym_DQUOTE,
    ACTIONS(15), 1,
      aux_sym__raw_string_token1,
    ACTIONS(17), 1,
      aux_sym__raw_string_token3,
    ACTIONS(23), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(29), 1,
      sym__normal_bare_identifier,
    ACTIONS(37), 1,
      anon_sym_null,
    ACTIONS(39), 1,
      sym__digit,
    ACTIONS(43), 1,
      anon_sym_0x,
    ACTIONS(45), 1,
      anon_sym_0o,
    ACTIONS(47), 1,
      anon_sym_0b,
    ACTIONS(51), 1,
      anon_sym_BSLASH,
    ACTIONS(83), 1,
      anon_sym_SLASH_DASH,
    ACTIONS(85), 1,
      anon_sym_LBRACE,
    STATE(81), 1,
      aux_sym__node_field_comment_repeat1,
    STATE(93), 1,
      sym__escline,
    STATE(102), 1,
      sym__node_space,
    STATE(270), 1,
      sym_type,
    STATE(299), 1,
      sym__sign,
    STATE(410), 1,
      sym_arco_pure_math_children,
    STATE(467), 1,
      sym__integer,
    STATE(492), 1,
      sym__bare_identifier,
    STATE(497), 1,
      sym_string,
    STATE(507), 1,
      sym_boolean,
    STATE(518), 1,
      sym_node_field,
    STATE(587), 1,
      sym_identifier,
    ACTIONS(41), 2,
      anon_sym_PLUS,
      anon_sym_DASH,
    ACTIONS(49), 2,
      anon_sym_true,
      anon_sym_false,
    STATE(84), 2,
      sym__ws,
      aux_sym__node_space_repeat1,
    STATE(491), 2,
      sym__escaped_string,
      sym__raw_string,
    STATE(506), 2,
      sym_prop,
      sym_value,
    STATE(516), 2,
      sym__node_field_comment,
      sym__node_field,
    ACTIONS(53), 3,
      sym_multi_line_comment,
      sym__bom,
      sym__unicode_space,
    STATE(115), 3,
      sym__node_terminator,
      sym__newline,
      sym_single_line_comment,
    STATE(508), 3,
      sym_keyword,
      sym_number,
      sym_bare_identifier,
    STATE(501), 4,
      sym__decimal,
      sym__hex,
      sym__octal,
      sym__binary,
    ACTIONS(87), 10,
      sym__eof,
      sym__implicit_terminator,
      anon_sym_SEMI,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
  [1914] = 37,
    ACTIONS(11), 1,
      anon_sym_LPAREN,
    ACTIONS(13), 1,
      anon_sym_DQUOTE,
    ACTIONS(15), 1,
      aux_sym__raw_string_token1,
    ACTIONS(17), 1,
      aux_sym__raw_string_token3,
    ACTIONS(23), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(29), 1,
      sym__normal_bare_identifier,
    ACTIONS(37), 1,
      anon_sym_null,
    ACTIONS(39), 1,
      sym__digit,
    ACTIONS(43), 1,
      anon_sym_0x,
    ACTIONS(45), 1,
      anon_sym_0o,
    ACTIONS(47), 1,
      anon_sym_0b,
    ACTIONS(51), 1,
      anon_sym_BSLASH,
    ACTIONS(83), 1,
      anon_sym_SLASH_DASH,
    ACTIONS(85), 1,
      anon_sym_LBRACE,
    STATE(81), 1,
      aux_sym__node_field_comment_repeat1,
    STATE(93), 1,
      sym__escline,
    STATE(102), 1,
      sym__node_space,
    STATE(270), 1,
      sym_type,
    STATE(299), 1,
      sym__sign,
    STATE(385), 1,
      sym_arco_pure_math_children,
    STATE(467), 1,
      sym__integer,
    STATE(492), 1,
      sym__bare_identifier,
    STATE(497), 1,
      sym_string,
    STATE(507), 1,
      sym_boolean,
    STATE(518), 1,
      sym_node_field,
    STATE(587), 1,
      sym_identifier,
    ACTIONS(41), 2,
      anon_sym_PLUS,
      anon_sym_DASH,
    ACTIONS(49), 2,
      anon_sym_true,
      anon_sym_false,
    STATE(84), 2,
      sym__ws,
      aux_sym__node_space_repeat1,
    STATE(491), 2,
      sym__escaped_string,
      sym__raw_string,
    STATE(506), 2,
      sym_prop,
      sym_value,
    STATE(516), 2,
      sym__node_field_comment,
      sym__node_field,
    ACTIONS(53), 3,
      sym_multi_line_comment,
      sym__bom,
      sym__unicode_space,
    STATE(223), 3,
      sym__node_terminator,
      sym__newline,
      sym_single_line_comment,
    STATE(508), 3,
      sym_keyword,
      sym_number,
      sym_bare_identifier,
    STATE(501), 4,
      sym__decimal,
      sym__hex,
      sym__octal,
      sym__binary,
    ACTIONS(89), 10,
      sym__eof,
      sym__implicit_terminator,
      anon_sym_SEMI,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
  [2050] = 37,
    ACTIONS(11), 1,
      anon_sym_LPAREN,
    ACTIONS(13), 1,
      anon_sym_DQUOTE,
    ACTIONS(15), 1,
      aux_sym__raw_string_token1,
    ACTIONS(17), 1,
      aux_sym__raw_string_token3,
    ACTIONS(23), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(29), 1,
      sym__normal_bare_identifier,
    ACTIONS(37), 1,
      anon_sym_null,
    ACTIONS(39), 1,
      sym__digit,
    ACTIONS(43), 1,
      anon_sym_0x,
    ACTIONS(45), 1,
      anon_sym_0o,
    ACTIONS(47), 1,
      anon_sym_0b,
    ACTIONS(51), 1,
      anon_sym_BSLASH,
    ACTIONS(73), 1,
      anon_sym_SLASH_DASH,
    ACTIONS(75), 1,
      anon_sym_LBRACE,
    STATE(81), 1,
      aux_sym__node_field_comment_repeat1,
    STATE(93), 1,
      sym__escline,
    STATE(102), 1,
      sym__node_space,
    STATE(270), 1,
      sym_type,
    STATE(299), 1,
      sym__sign,
    STATE(394), 1,
      sym_node_children,
    STATE(467), 1,
      sym__integer,
    STATE(492), 1,
      sym__bare_identifier,
    STATE(497), 1,
      sym_string,
    STATE(507), 1,
      sym_boolean,
    STATE(518), 1,
      sym_node_field,
    STATE(587), 1,
      sym_identifier,
    ACTIONS(41), 2,
      anon_sym_PLUS,
      anon_sym_DASH,
    ACTIONS(49), 2,
      anon_sym_true,
      anon_sym_false,
    STATE(84), 2,
      sym__ws,
      aux_sym__node_space_repeat1,
    STATE(491), 2,
      sym__escaped_string,
      sym__raw_string,
    STATE(506), 2,
      sym_prop,
      sym_value,
    STATE(516), 2,
      sym__node_field_comment,
      sym__node_field,
    ACTIONS(53), 3,
      sym_multi_line_comment,
      sym__bom,
      sym__unicode_space,
    STATE(217), 3,
      sym__node_terminator,
      sym__newline,
      sym_single_line_comment,
    STATE(508), 3,
      sym_keyword,
      sym_number,
      sym_bare_identifier,
    STATE(501), 4,
      sym__decimal,
      sym__hex,
      sym__octal,
      sym__binary,
    ACTIONS(91), 10,
      sym__eof,
      sym__implicit_terminator,
      anon_sym_SEMI,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
  [2186] = 37,
    ACTIONS(11), 1,
      anon_sym_LPAREN,
    ACTIONS(13), 1,
      anon_sym_DQUOTE,
    ACTIONS(15), 1,
      aux_sym__raw_string_token1,
    ACTIONS(17), 1,
      aux_sym__raw_string_token3,
    ACTIONS(23), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(29), 1,
      sym__normal_bare_identifier,
    ACTIONS(37), 1,
      anon_sym_null,
    ACTIONS(39), 1,
      sym__digit,
    ACTIONS(43), 1,
      anon_sym_0x,
    ACTIONS(45), 1,
      anon_sym_0o,
    ACTIONS(47), 1,
      anon_sym_0b,
    ACTIONS(51), 1,
      anon_sym_BSLASH,
    ACTIONS(83), 1,
      anon_sym_SLASH_DASH,
    ACTIONS(85), 1,
      anon_sym_LBRACE,
    STATE(81), 1,
      aux_sym__node_field_comment_repeat1,
    STATE(93), 1,
      sym__escline,
    STATE(102), 1,
      sym__node_space,
    STATE(270), 1,
      sym_type,
    STATE(299), 1,
      sym__sign,
    STATE(396), 1,
      sym_arco_pure_math_children,
    STATE(467), 1,
      sym__integer,
    STATE(492), 1,
      sym__bare_identifier,
    STATE(497), 1,
      sym_string,
    STATE(507), 1,
      sym_boolean,
    STATE(518), 1,
      sym_node_field,
    STATE(587), 1,
      sym_identifier,
    ACTIONS(41), 2,
      anon_sym_PLUS,
      anon_sym_DASH,
    ACTIONS(49), 2,
      anon_sym_true,
      anon_sym_false,
    STATE(84), 2,
      sym__ws,
      aux_sym__node_space_repeat1,
    STATE(491), 2,
      sym__escaped_string,
      sym__raw_string,
    STATE(506), 2,
      sym_prop,
      sym_value,
    STATE(516), 2,
      sym__node_field_comment,
      sym__node_field,
    ACTIONS(53), 3,
      sym_multi_line_comment,
      sym__bom,
      sym__unicode_space,
    STATE(119), 3,
      sym__node_terminator,
      sym__newline,
      sym_single_line_comment,
    STATE(508), 3,
      sym_keyword,
      sym_number,
      sym_bare_identifier,
    STATE(501), 4,
      sym__decimal,
      sym__hex,
      sym__octal,
      sym__binary,
    ACTIONS(93), 10,
      sym__eof,
      sym__implicit_terminator,
      anon_sym_SEMI,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
  [2322] = 37,
    ACTIONS(11), 1,
      anon_sym_LPAREN,
    ACTIONS(13), 1,
      anon_sym_DQUOTE,
    ACTIONS(15), 1,
      aux_sym__raw_string_token1,
    ACTIONS(17), 1,
      aux_sym__raw_string_token3,
    ACTIONS(23), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(29), 1,
      sym__normal_bare_identifier,
    ACTIONS(37), 1,
      anon_sym_null,
    ACTIONS(39), 1,
      sym__digit,
    ACTIONS(43), 1,
      anon_sym_0x,
    ACTIONS(45), 1,
      anon_sym_0o,
    ACTIONS(47), 1,
      anon_sym_0b,
    ACTIONS(51), 1,
      anon_sym_BSLASH,
    ACTIONS(83), 1,
      anon_sym_SLASH_DASH,
    ACTIONS(85), 1,
      anon_sym_LBRACE,
    STATE(81), 1,
      aux_sym__node_field_comment_repeat1,
    STATE(93), 1,
      sym__escline,
    STATE(102), 1,
      sym__node_space,
    STATE(270), 1,
      sym_type,
    STATE(299), 1,
      sym__sign,
    STATE(402), 1,
      sym_arco_pure_math_children,
    STATE(467), 1,
      sym__integer,
    STATE(492), 1,
      sym__bare_identifier,
    STATE(497), 1,
      sym_string,
    STATE(507), 1,
      sym_boolean,
    STATE(518), 1,
      sym_node_field,
    STATE(587), 1,
      sym_identifier,
    ACTIONS(41), 2,
      anon_sym_PLUS,
      anon_sym_DASH,
    ACTIONS(49), 2,
      anon_sym_true,
      anon_sym_false,
    STATE(84), 2,
      sym__ws,
      aux_sym__node_space_repeat1,
    STATE(491), 2,
      sym__escaped_string,
      sym__raw_string,
    STATE(506), 2,
      sym_prop,
      sym_value,
    STATE(516), 2,
      sym__node_field_comment,
      sym__node_field,
    ACTIONS(53), 3,
      sym_multi_line_comment,
      sym__bom,
      sym__unicode_space,
    STATE(214), 3,
      sym__node_terminator,
      sym__newline,
      sym_single_line_comment,
    STATE(508), 3,
      sym_keyword,
      sym_number,
      sym_bare_identifier,
    STATE(501), 4,
      sym__decimal,
      sym__hex,
      sym__octal,
      sym__binary,
    ACTIONS(95), 10,
      sym__eof,
      sym__implicit_terminator,
      anon_sym_SEMI,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
  [2458] = 37,
    ACTIONS(11), 1,
      anon_sym_LPAREN,
    ACTIONS(13), 1,
      anon_sym_DQUOTE,
    ACTIONS(15), 1,
      aux_sym__raw_string_token1,
    ACTIONS(17), 1,
      aux_sym__raw_string_token3,
    ACTIONS(23), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(29), 1,
      sym__normal_bare_identifier,
    ACTIONS(37), 1,
      anon_sym_null,
    ACTIONS(39), 1,
      sym__digit,
    ACTIONS(43), 1,
      anon_sym_0x,
    ACTIONS(45), 1,
      anon_sym_0o,
    ACTIONS(47), 1,
      anon_sym_0b,
    ACTIONS(51), 1,
      anon_sym_BSLASH,
    ACTIONS(83), 1,
      anon_sym_SLASH_DASH,
    ACTIONS(85), 1,
      anon_sym_LBRACE,
    STATE(81), 1,
      aux_sym__node_field_comment_repeat1,
    STATE(93), 1,
      sym__escline,
    STATE(102), 1,
      sym__node_space,
    STATE(270), 1,
      sym_type,
    STATE(299), 1,
      sym__sign,
    STATE(421), 1,
      sym_arco_pure_math_children,
    STATE(467), 1,
      sym__integer,
    STATE(492), 1,
      sym__bare_identifier,
    STATE(497), 1,
      sym_string,
    STATE(507), 1,
      sym_boolean,
    STATE(518), 1,
      sym_node_field,
    STATE(587), 1,
      sym_identifier,
    ACTIONS(41), 2,
      anon_sym_PLUS,
      anon_sym_DASH,
    ACTIONS(49), 2,
      anon_sym_true,
      anon_sym_false,
    STATE(84), 2,
      sym__ws,
      aux_sym__node_space_repeat1,
    STATE(491), 2,
      sym__escaped_string,
      sym__raw_string,
    STATE(506), 2,
      sym_prop,
      sym_value,
    STATE(516), 2,
      sym__node_field_comment,
      sym__node_field,
    ACTIONS(53), 3,
      sym_multi_line_comment,
      sym__bom,
      sym__unicode_space,
    STATE(108), 3,
      sym__node_terminator,
      sym__newline,
      sym_single_line_comment,
    STATE(508), 3,
      sym_keyword,
      sym_number,
      sym_bare_identifier,
    STATE(501), 4,
      sym__decimal,
      sym__hex,
      sym__octal,
      sym__binary,
    ACTIONS(97), 10,
      sym__eof,
      sym__implicit_terminator,
      anon_sym_SEMI,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
  [2594] = 37,
    ACTIONS(11), 1,
      anon_sym_LPAREN,
    ACTIONS(13), 1,
      anon_sym_DQUOTE,
    ACTIONS(15), 1,
      aux_sym__raw_string_token1,
    ACTIONS(17), 1,
      aux_sym__raw_string_token3,
    ACTIONS(23), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(29), 1,
      sym__normal_bare_identifier,
    ACTIONS(37), 1,
      anon_sym_null,
    ACTIONS(39), 1,
      sym__digit,
    ACTIONS(43), 1,
      anon_sym_0x,
    ACTIONS(45), 1,
      anon_sym_0o,
    ACTIONS(47), 1,
      anon_sym_0b,
    ACTIONS(51), 1,
      anon_sym_BSLASH,
    ACTIONS(83), 1,
      anon_sym_SLASH_DASH,
    ACTIONS(85), 1,
      anon_sym_LBRACE,
    STATE(81), 1,
      aux_sym__node_field_comment_repeat1,
    STATE(93), 1,
      sym__escline,
    STATE(102), 1,
      sym__node_space,
    STATE(270), 1,
      sym_type,
    STATE(299), 1,
      sym__sign,
    STATE(386), 1,
      sym_arco_pure_math_children,
    STATE(467), 1,
      sym__integer,
    STATE(492), 1,
      sym__bare_identifier,
    STATE(497), 1,
      sym_string,
    STATE(507), 1,
      sym_boolean,
    STATE(518), 1,
      sym_node_field,
    STATE(587), 1,
      sym_identifier,
    ACTIONS(41), 2,
      anon_sym_PLUS,
      anon_sym_DASH,
    ACTIONS(49), 2,
      anon_sym_true,
      anon_sym_false,
    STATE(84), 2,
      sym__ws,
      aux_sym__node_space_repeat1,
    STATE(491), 2,
      sym__escaped_string,
      sym__raw_string,
    STATE(506), 2,
      sym_prop,
      sym_value,
    STATE(516), 2,
      sym__node_field_comment,
      sym__node_field,
    ACTIONS(53), 3,
      sym_multi_line_comment,
      sym__bom,
      sym__unicode_space,
    STATE(222), 3,
      sym__node_terminator,
      sym__newline,
      sym_single_line_comment,
    STATE(508), 3,
      sym_keyword,
      sym_number,
      sym_bare_identifier,
    STATE(501), 4,
      sym__decimal,
      sym__hex,
      sym__octal,
      sym__binary,
    ACTIONS(99), 10,
      sym__eof,
      sym__implicit_terminator,
      anon_sym_SEMI,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
  [2730] = 37,
    ACTIONS(11), 1,
      anon_sym_LPAREN,
    ACTIONS(13), 1,
      anon_sym_DQUOTE,
    ACTIONS(15), 1,
      aux_sym__raw_string_token1,
    ACTIONS(17), 1,
      aux_sym__raw_string_token3,
    ACTIONS(23), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(29), 1,
      sym__normal_bare_identifier,
    ACTIONS(37), 1,
      anon_sym_null,
    ACTIONS(39), 1,
      sym__digit,
    ACTIONS(43), 1,
      anon_sym_0x,
    ACTIONS(45), 1,
      anon_sym_0o,
    ACTIONS(47), 1,
      anon_sym_0b,
    ACTIONS(51), 1,
      anon_sym_BSLASH,
    ACTIONS(73), 1,
      anon_sym_SLASH_DASH,
    ACTIONS(75), 1,
      anon_sym_LBRACE,
    STATE(81), 1,
      aux_sym__node_field_comment_repeat1,
    STATE(93), 1,
      sym__escline,
    STATE(102), 1,
      sym__node_space,
    STATE(270), 1,
      sym_type,
    STATE(299), 1,
      sym__sign,
    STATE(415), 1,
      sym_node_children,
    STATE(467), 1,
      sym__integer,
    STATE(492), 1,
      sym__bare_identifier,
    STATE(497), 1,
      sym_string,
    STATE(507), 1,
      sym_boolean,
    STATE(518), 1,
      sym_node_field,
    STATE(587), 1,
      sym_identifier,
    ACTIONS(41), 2,
      anon_sym_PLUS,
      anon_sym_DASH,
    ACTIONS(49), 2,
      anon_sym_true,
      anon_sym_false,
    STATE(84), 2,
      sym__ws,
      aux_sym__node_space_repeat1,
    STATE(491), 2,
      sym__escaped_string,
      sym__raw_string,
    STATE(506), 2,
      sym_prop,
      sym_value,
    STATE(516), 2,
      sym__node_field_comment,
      sym__node_field,
    ACTIONS(53), 3,
      sym_multi_line_comment,
      sym__bom,
      sym__unicode_space,
    STATE(207), 3,
      sym__node_terminator,
      sym__newline,
      sym_single_line_comment,
    STATE(508), 3,
      sym_keyword,
      sym_number,
      sym_bare_identifier,
    STATE(501), 4,
      sym__decimal,
      sym__hex,
      sym__octal,
      sym__binary,
    ACTIONS(101), 10,
      sym__eof,
      sym__implicit_terminator,
      anon_sym_SEMI,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
  [2866] = 37,
    ACTIONS(11), 1,
      anon_sym_LPAREN,
    ACTIONS(13), 1,
      anon_sym_DQUOTE,
    ACTIONS(15), 1,
      aux_sym__raw_string_token1,
    ACTIONS(17), 1,
      aux_sym__raw_string_token3,
    ACTIONS(23), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(29), 1,
      sym__normal_bare_identifier,
    ACTIONS(37), 1,
      anon_sym_null,
    ACTIONS(39), 1,
      sym__digit,
    ACTIONS(43), 1,
      anon_sym_0x,
    ACTIONS(45), 1,
      anon_sym_0o,
    ACTIONS(47), 1,
      anon_sym_0b,
    ACTIONS(51), 1,
      anon_sym_BSLASH,
    ACTIONS(83), 1,
      anon_sym_SLASH_DASH,
    ACTIONS(85), 1,
      anon_sym_LBRACE,
    STATE(81), 1,
      aux_sym__node_field_comment_repeat1,
    STATE(93), 1,
      sym__escline,
    STATE(102), 1,
      sym__node_space,
    STATE(270), 1,
      sym_type,
    STATE(299), 1,
      sym__sign,
    STATE(445), 1,
      sym_arco_pure_math_children,
    STATE(467), 1,
      sym__integer,
    STATE(492), 1,
      sym__bare_identifier,
    STATE(497), 1,
      sym_string,
    STATE(507), 1,
      sym_boolean,
    STATE(518), 1,
      sym_node_field,
    STATE(587), 1,
      sym_identifier,
    ACTIONS(41), 2,
      anon_sym_PLUS,
      anon_sym_DASH,
    ACTIONS(49), 2,
      anon_sym_true,
      anon_sym_false,
    STATE(84), 2,
      sym__ws,
      aux_sym__node_space_repeat1,
    STATE(491), 2,
      sym__escaped_string,
      sym__raw_string,
    STATE(506), 2,
      sym_prop,
      sym_value,
    STATE(516), 2,
      sym__node_field_comment,
      sym__node_field,
    ACTIONS(53), 3,
      sym_multi_line_comment,
      sym__bom,
      sym__unicode_space,
    STATE(153), 3,
      sym__node_terminator,
      sym__newline,
      sym_single_line_comment,
    STATE(508), 3,
      sym_keyword,
      sym_number,
      sym_bare_identifier,
    STATE(501), 4,
      sym__decimal,
      sym__hex,
      sym__octal,
      sym__binary,
    ACTIONS(103), 10,
      sym__eof,
      sym__implicit_terminator,
      anon_sym_SEMI,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
  [3002] = 37,
    ACTIONS(11), 1,
      anon_sym_LPAREN,
    ACTIONS(13), 1,
      anon_sym_DQUOTE,
    ACTIONS(15), 1,
      aux_sym__raw_string_token1,
    ACTIONS(17), 1,
      aux_sym__raw_string_token3,
    ACTIONS(23), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(29), 1,
      sym__normal_bare_identifier,
    ACTIONS(37), 1,
      anon_sym_null,
    ACTIONS(39), 1,
      sym__digit,
    ACTIONS(43), 1,
      anon_sym_0x,
    ACTIONS(45), 1,
      anon_sym_0o,
    ACTIONS(47), 1,
      anon_sym_0b,
    ACTIONS(51), 1,
      anon_sym_BSLASH,
    ACTIONS(83), 1,
      anon_sym_SLASH_DASH,
    ACTIONS(85), 1,
      anon_sym_LBRACE,
    STATE(81), 1,
      aux_sym__node_field_comment_repeat1,
    STATE(93), 1,
      sym__escline,
    STATE(102), 1,
      sym__node_space,
    STATE(270), 1,
      sym_type,
    STATE(299), 1,
      sym__sign,
    STATE(313), 1,
      sym_arco_pure_math_children,
    STATE(467), 1,
      sym__integer,
    STATE(492), 1,
      sym__bare_identifier,
    STATE(497), 1,
      sym_string,
    STATE(507), 1,
      sym_boolean,
    STATE(518), 1,
      sym_node_field,
    STATE(587), 1,
      sym_identifier,
    ACTIONS(41), 2,
      anon_sym_PLUS,
      anon_sym_DASH,
    ACTIONS(49), 2,
      anon_sym_true,
      anon_sym_false,
    STATE(84), 2,
      sym__ws,
      aux_sym__node_space_repeat1,
    STATE(491), 2,
      sym__escaped_string,
      sym__raw_string,
    STATE(506), 2,
      sym_prop,
      sym_value,
    STATE(516), 2,
      sym__node_field_comment,
      sym__node_field,
    ACTIONS(53), 3,
      sym_multi_line_comment,
      sym__bom,
      sym__unicode_space,
    STATE(171), 3,
      sym__node_terminator,
      sym__newline,
      sym_single_line_comment,
    STATE(508), 3,
      sym_keyword,
      sym_number,
      sym_bare_identifier,
    STATE(501), 4,
      sym__decimal,
      sym__hex,
      sym__octal,
      sym__binary,
    ACTIONS(105), 10,
      sym__eof,
      sym__implicit_terminator,
      anon_sym_SEMI,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
  [3138] = 37,
    ACTIONS(11), 1,
      anon_sym_LPAREN,
    ACTIONS(13), 1,
      anon_sym_DQUOTE,
    ACTIONS(15), 1,
      aux_sym__raw_string_token1,
    ACTIONS(17), 1,
      aux_sym__raw_string_token3,
    ACTIONS(23), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(29), 1,
      sym__normal_bare_identifier,
    ACTIONS(37), 1,
      anon_sym_null,
    ACTIONS(39), 1,
      sym__digit,
    ACTIONS(43), 1,
      anon_sym_0x,
    ACTIONS(45), 1,
      anon_sym_0o,
    ACTIONS(47), 1,
      anon_sym_0b,
    ACTIONS(51), 1,
      anon_sym_BSLASH,
    ACTIONS(83), 1,
      anon_sym_SLASH_DASH,
    ACTIONS(85), 1,
      anon_sym_LBRACE,
    STATE(81), 1,
      aux_sym__node_field_comment_repeat1,
    STATE(93), 1,
      sym__escline,
    STATE(102), 1,
      sym__node_space,
    STATE(270), 1,
      sym_type,
    STATE(299), 1,
      sym__sign,
    STATE(347), 1,
      sym_arco_pure_math_children,
    STATE(467), 1,
      sym__integer,
    STATE(492), 1,
      sym__bare_identifier,
    STATE(497), 1,
      sym_string,
    STATE(507), 1,
      sym_boolean,
    STATE(518), 1,
      sym_node_field,
    STATE(587), 1,
      sym_identifier,
    ACTIONS(41), 2,
      anon_sym_PLUS,
      anon_sym_DASH,
    ACTIONS(49), 2,
      anon_sym_true,
      anon_sym_false,
    STATE(84), 2,
      sym__ws,
      aux_sym__node_space_repeat1,
    STATE(491), 2,
      sym__escaped_string,
      sym__raw_string,
    STATE(506), 2,
      sym_prop,
      sym_value,
    STATE(516), 2,
      sym__node_field_comment,
      sym__node_field,
    ACTIONS(53), 3,
      sym_multi_line_comment,
      sym__bom,
      sym__unicode_space,
    STATE(260), 3,
      sym__node_terminator,
      sym__newline,
      sym_single_line_comment,
    STATE(508), 3,
      sym_keyword,
      sym_number,
      sym_bare_identifier,
    STATE(501), 4,
      sym__decimal,
      sym__hex,
      sym__octal,
      sym__binary,
    ACTIONS(107), 10,
      sym__eof,
      sym__implicit_terminator,
      anon_sym_SEMI,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
  [3274] = 37,
    ACTIONS(11), 1,
      anon_sym_LPAREN,
    ACTIONS(13), 1,
      anon_sym_DQUOTE,
    ACTIONS(15), 1,
      aux_sym__raw_string_token1,
    ACTIONS(17), 1,
      aux_sym__raw_string_token3,
    ACTIONS(23), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(29), 1,
      sym__normal_bare_identifier,
    ACTIONS(37), 1,
      anon_sym_null,
    ACTIONS(39), 1,
      sym__digit,
    ACTIONS(43), 1,
      anon_sym_0x,
    ACTIONS(45), 1,
      anon_sym_0o,
    ACTIONS(47), 1,
      anon_sym_0b,
    ACTIONS(51), 1,
      anon_sym_BSLASH,
    ACTIONS(73), 1,
      anon_sym_SLASH_DASH,
    ACTIONS(75), 1,
      anon_sym_LBRACE,
    STATE(81), 1,
      aux_sym__node_field_comment_repeat1,
    STATE(93), 1,
      sym__escline,
    STATE(102), 1,
      sym__node_space,
    STATE(270), 1,
      sym_type,
    STATE(299), 1,
      sym__sign,
    STATE(336), 1,
      sym_node_children,
    STATE(467), 1,
      sym__integer,
    STATE(492), 1,
      sym__bare_identifier,
    STATE(497), 1,
      sym_string,
    STATE(507), 1,
      sym_boolean,
    STATE(518), 1,
      sym_node_field,
    STATE(587), 1,
      sym_identifier,
    ACTIONS(41), 2,
      anon_sym_PLUS,
      anon_sym_DASH,
    ACTIONS(49), 2,
      anon_sym_true,
      anon_sym_false,
    STATE(84), 2,
      sym__ws,
      aux_sym__node_space_repeat1,
    STATE(491), 2,
      sym__escaped_string,
      sym__raw_string,
    STATE(506), 2,
      sym_prop,
      sym_value,
    STATE(516), 2,
      sym__node_field_comment,
      sym__node_field,
    ACTIONS(53), 3,
      sym_multi_line_comment,
      sym__bom,
      sym__unicode_space,
    STATE(195), 3,
      sym__node_terminator,
      sym__newline,
      sym_single_line_comment,
    STATE(508), 3,
      sym_keyword,
      sym_number,
      sym_bare_identifier,
    STATE(501), 4,
      sym__decimal,
      sym__hex,
      sym__octal,
      sym__binary,
    ACTIONS(109), 10,
      sym__eof,
      sym__implicit_terminator,
      anon_sym_SEMI,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
  [3410] = 37,
    ACTIONS(11), 1,
      anon_sym_LPAREN,
    ACTIONS(13), 1,
      anon_sym_DQUOTE,
    ACTIONS(15), 1,
      aux_sym__raw_string_token1,
    ACTIONS(17), 1,
      aux_sym__raw_string_token3,
    ACTIONS(23), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(29), 1,
      sym__normal_bare_identifier,
    ACTIONS(37), 1,
      anon_sym_null,
    ACTIONS(39), 1,
      sym__digit,
    ACTIONS(43), 1,
      anon_sym_0x,
    ACTIONS(45), 1,
      anon_sym_0o,
    ACTIONS(47), 1,
      anon_sym_0b,
    ACTIONS(51), 1,
      anon_sym_BSLASH,
    ACTIONS(83), 1,
      anon_sym_SLASH_DASH,
    ACTIONS(85), 1,
      anon_sym_LBRACE,
    STATE(81), 1,
      aux_sym__node_field_comment_repeat1,
    STATE(93), 1,
      sym__escline,
    STATE(102), 1,
      sym__node_space,
    STATE(270), 1,
      sym_type,
    STATE(299), 1,
      sym__sign,
    STATE(307), 1,
      sym_arco_pure_math_children,
    STATE(467), 1,
      sym__integer,
    STATE(492), 1,
      sym__bare_identifier,
    STATE(497), 1,
      sym_string,
    STATE(507), 1,
      sym_boolean,
    STATE(518), 1,
      sym_node_field,
    STATE(587), 1,
      sym_identifier,
    ACTIONS(41), 2,
      anon_sym_PLUS,
      anon_sym_DASH,
    ACTIONS(49), 2,
      anon_sym_true,
      anon_sym_false,
    STATE(84), 2,
      sym__ws,
      aux_sym__node_space_repeat1,
    STATE(491), 2,
      sym__escaped_string,
      sym__raw_string,
    STATE(506), 2,
      sym_prop,
      sym_value,
    STATE(516), 2,
      sym__node_field_comment,
      sym__node_field,
    ACTIONS(53), 3,
      sym_multi_line_comment,
      sym__bom,
      sym__unicode_space,
    STATE(185), 3,
      sym__node_terminator,
      sym__newline,
      sym_single_line_comment,
    STATE(508), 3,
      sym_keyword,
      sym_number,
      sym_bare_identifier,
    STATE(501), 4,
      sym__decimal,
      sym__hex,
      sym__octal,
      sym__binary,
    ACTIONS(111), 10,
      sym__eof,
      sym__implicit_terminator,
      anon_sym_SEMI,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
  [3546] = 37,
    ACTIONS(11), 1,
      anon_sym_LPAREN,
    ACTIONS(13), 1,
      anon_sym_DQUOTE,
    ACTIONS(15), 1,
      aux_sym__raw_string_token1,
    ACTIONS(17), 1,
      aux_sym__raw_string_token3,
    ACTIONS(23), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(29), 1,
      sym__normal_bare_identifier,
    ACTIONS(37), 1,
      anon_sym_null,
    ACTIONS(39), 1,
      sym__digit,
    ACTIONS(43), 1,
      anon_sym_0x,
    ACTIONS(45), 1,
      anon_sym_0o,
    ACTIONS(47), 1,
      anon_sym_0b,
    ACTIONS(51), 1,
      anon_sym_BSLASH,
    ACTIONS(73), 1,
      anon_sym_SLASH_DASH,
    ACTIONS(75), 1,
      anon_sym_LBRACE,
    STATE(81), 1,
      aux_sym__node_field_comment_repeat1,
    STATE(93), 1,
      sym__escline,
    STATE(102), 1,
      sym__node_space,
    STATE(270), 1,
      sym_type,
    STATE(299), 1,
      sym__sign,
    STATE(304), 1,
      sym_node_children,
    STATE(467), 1,
      sym__integer,
    STATE(492), 1,
      sym__bare_identifier,
    STATE(497), 1,
      sym_string,
    STATE(507), 1,
      sym_boolean,
    STATE(518), 1,
      sym_node_field,
    STATE(587), 1,
      sym_identifier,
    ACTIONS(41), 2,
      anon_sym_PLUS,
      anon_sym_DASH,
    ACTIONS(49), 2,
      anon_sym_true,
      anon_sym_false,
    STATE(84), 2,
      sym__ws,
      aux_sym__node_space_repeat1,
    STATE(491), 2,
      sym__escaped_string,
      sym__raw_string,
    STATE(506), 2,
      sym_prop,
      sym_value,
    STATE(516), 2,
      sym__node_field_comment,
      sym__node_field,
    ACTIONS(53), 3,
      sym_multi_line_comment,
      sym__bom,
      sym__unicode_space,
    STATE(190), 3,
      sym__node_terminator,
      sym__newline,
      sym_single_line_comment,
    STATE(508), 3,
      sym_keyword,
      sym_number,
      sym_bare_identifier,
    STATE(501), 4,
      sym__decimal,
      sym__hex,
      sym__octal,
      sym__binary,
    ACTIONS(113), 10,
      sym__eof,
      sym__implicit_terminator,
      anon_sym_SEMI,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
  [3682] = 11,
    ACTIONS(3), 1,
      sym_multi_line_comment,
    ACTIONS(115), 1,
      sym__normal_bare_identifier,
    ACTIONS(119), 1,
      anon_sym_DQUOTE,
    ACTIONS(121), 1,
      aux_sym__raw_string_token1,
    ACTIONS(123), 1,
      aux_sym__raw_string_token3,
    STATE(583), 1,
      sym__sign,
    ACTIONS(125), 2,
      anon_sym_PLUS,
      anon_sym_DASH,
    STATE(590), 2,
      sym_identifier,
      sym_annotation_type,
    STATE(592), 2,
      sym__escaped_string,
      sym__raw_string,
    STATE(596), 2,
      sym__bare_identifier,
      sym_string,
    ACTIONS(117), 37,
      anon_sym_i8,
      anon_sym_i16,
      anon_sym_i32,
      anon_sym_i64,
      anon_sym_u8,
      anon_sym_u16,
      anon_sym_u32,
      anon_sym_u64,
      anon_sym_isize,
      anon_sym_usize,
      anon_sym_f32,
      anon_sym_f64,
      anon_sym_decimal64,
      anon_sym_decimal128,
      anon_sym_date_DASHtime,
      anon_sym_time,
      anon_sym_date,
      anon_sym_duration,
      anon_sym_decimal,
      anon_sym_currency,
      anon_sym_country_DASH2,
      anon_sym_country_DASH3,
      anon_sym_country_DASHsubdivision,
      anon_sym_email,
      anon_sym_idn_DASHemail,
      anon_sym_hostname,
      anon_sym_idn_DASHhostname,
      anon_sym_ipv4,
      anon_sym_ipv6,
      anon_sym_url,
      anon_sym_url_DASHreference,
      anon_sym_irl,
      anon_sym_iri_DASHreference,
      anon_sym_url_DASHtemplate,
      anon_sym_uuid,
      anon_sym_regex,
      anon_sym_base64,
  [3756] = 20,
    ACTIONS(129), 1,
      sym__normal_bare_identifier,
    ACTIONS(132), 1,
      anon_sym_SLASH_DASH,
    ACTIONS(135), 1,
      anon_sym_LPAREN,
    ACTIONS(138), 1,
      anon_sym_DQUOTE,
    ACTIONS(141), 1,
      aux_sym__raw_string_token1,
    ACTIONS(144), 1,
      aux_sym__raw_string_token3,
    ACTIONS(153), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(159), 1,
      anon_sym_constraint,
    STATE(294), 1,
      sym_identifier,
    STATE(461), 1,
      sym_type,
    STATE(493), 1,
      sym__sign,
    ACTIONS(127), 2,
      ts_builtin_sym_end,
      anon_sym_RBRACE,
    ACTIONS(147), 2,
      anon_sym_PLUS,
      anon_sym_DASH,
    STATE(30), 2,
      sym_node,
      aux_sym_document_repeat2,
    STATE(491), 2,
      sym__escaped_string,
      sym__raw_string,
    STATE(510), 2,
      sym__bare_identifier,
      sym_string,
    STATE(149), 3,
      sym_kdl_node,
      sym_arco_pure_math_node,
      sym_arco_constraint_node,
    STATE(69), 5,
      sym__linespace,
      sym__newline,
      sym__ws,
      sym_single_line_comment,
      aux_sym_document_repeat1,
    ACTIONS(156), 8,
      anon_sym_expression,
      anon_sym_minimize,
      anon_sym_maximize,
      anon_sym_expr,
      anon_sym_filter,
      anon_sym_if,
      anon_sym_lower,
      anon_sym_upper,
    ACTIONS(150), 10,
      sym_multi_line_comment,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
  [3844] = 20,
    ACTIONS(7), 1,
      sym__normal_bare_identifier,
    ACTIONS(9), 1,
      anon_sym_SLASH_DASH,
    ACTIONS(11), 1,
      anon_sym_LPAREN,
    ACTIONS(13), 1,
      anon_sym_DQUOTE,
    ACTIONS(15), 1,
      aux_sym__raw_string_token1,
    ACTIONS(17), 1,
      aux_sym__raw_string_token3,
    ACTIONS(23), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(27), 1,
      anon_sym_constraint,
    ACTIONS(162), 1,
      anon_sym_RBRACE,
    STATE(294), 1,
      sym_identifier,
    STATE(461), 1,
      sym_type,
    STATE(493), 1,
      sym__sign,
    ACTIONS(19), 2,
      anon_sym_PLUS,
      anon_sym_DASH,
    STATE(41), 2,
      sym_node,
      aux_sym_document_repeat2,
    STATE(491), 2,
      sym__escaped_string,
      sym__raw_string,
    STATE(510), 2,
      sym__bare_identifier,
      sym_string,
    STATE(149), 3,
      sym_kdl_node,
      sym_arco_pure_math_node,
      sym_arco_constraint_node,
    STATE(51), 5,
      sym__linespace,
      sym__newline,
      sym__ws,
      sym_single_line_comment,
      aux_sym_document_repeat1,
    ACTIONS(25), 8,
      anon_sym_expression,
      anon_sym_minimize,
      anon_sym_maximize,
      anon_sym_expr,
      anon_sym_filter,
      anon_sym_if,
      anon_sym_lower,
      anon_sym_upper,
    ACTIONS(164), 10,
      sym_multi_line_comment,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
  [3931] = 20,
    ACTIONS(7), 1,
      sym__normal_bare_identifier,
    ACTIONS(9), 1,
      anon_sym_SLASH_DASH,
    ACTIONS(11), 1,
      anon_sym_LPAREN,
    ACTIONS(13), 1,
      anon_sym_DQUOTE,
    ACTIONS(15), 1,
      aux_sym__raw_string_token1,
    ACTIONS(17), 1,
      aux_sym__raw_string_token3,
    ACTIONS(23), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(27), 1,
      anon_sym_constraint,
    ACTIONS(166), 1,
      ts_builtin_sym_end,
    STATE(294), 1,
      sym_identifier,
    STATE(461), 1,
      sym_type,
    STATE(493), 1,
      sym__sign,
    ACTIONS(19), 2,
      anon_sym_PLUS,
      anon_sym_DASH,
    STATE(30), 2,
      sym_node,
      aux_sym_document_repeat2,
    STATE(491), 2,
      sym__escaped_string,
      sym__raw_string,
    STATE(510), 2,
      sym__bare_identifier,
      sym_string,
    STATE(149), 3,
      sym_kdl_node,
      sym_arco_pure_math_node,
      sym_arco_constraint_node,
    STATE(66), 5,
      sym__linespace,
      sym__newline,
      sym__ws,
      sym_single_line_comment,
      aux_sym_document_repeat1,
    ACTIONS(25), 8,
      anon_sym_expression,
      anon_sym_minimize,
      anon_sym_maximize,
      anon_sym_expr,
      anon_sym_filter,
      anon_sym_if,
      anon_sym_lower,
      anon_sym_upper,
    ACTIONS(168), 10,
      sym_multi_line_comment,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
  [4018] = 20,
    ACTIONS(7), 1,
      sym__normal_bare_identifier,
    ACTIONS(9), 1,
      anon_sym_SLASH_DASH,
    ACTIONS(11), 1,
      anon_sym_LPAREN,
    ACTIONS(13), 1,
      anon_sym_DQUOTE,
    ACTIONS(15), 1,
      aux_sym__raw_string_token1,
    ACTIONS(17), 1,
      aux_sym__raw_string_token3,
    ACTIONS(23), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(27), 1,
      anon_sym_constraint,
    ACTIONS(170), 1,
      anon_sym_RBRACE,
    STATE(294), 1,
      sym_identifier,
    STATE(461), 1,
      sym_type,
    STATE(493), 1,
      sym__sign,
    ACTIONS(19), 2,
      anon_sym_PLUS,
      anon_sym_DASH,
    STATE(38), 2,
      sym_node,
      aux_sym_document_repeat2,
    STATE(491), 2,
      sym__escaped_string,
      sym__raw_string,
    STATE(510), 2,
      sym__bare_identifier,
      sym_string,
    STATE(149), 3,
      sym_kdl_node,
      sym_arco_pure_math_node,
      sym_arco_constraint_node,
    STATE(65), 5,
      sym__linespace,
      sym__newline,
      sym__ws,
      sym_single_line_comment,
      aux_sym_document_repeat1,
    ACTIONS(25), 8,
      anon_sym_expression,
      anon_sym_minimize,
      anon_sym_maximize,
      anon_sym_expr,
      anon_sym_filter,
      anon_sym_if,
      anon_sym_lower,
      anon_sym_upper,
    ACTIONS(172), 10,
      sym_multi_line_comment,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
  [4105] = 20,
    ACTIONS(7), 1,
      sym__normal_bare_identifier,
    ACTIONS(9), 1,
      anon_sym_SLASH_DASH,
    ACTIONS(11), 1,
      anon_sym_LPAREN,
    ACTIONS(13), 1,
      anon_sym_DQUOTE,
    ACTIONS(15), 1,
      aux_sym__raw_string_token1,
    ACTIONS(17), 1,
      aux_sym__raw_string_token3,
    ACTIONS(23), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(27), 1,
      anon_sym_constraint,
    ACTIONS(174), 1,
      ts_builtin_sym_end,
    STATE(294), 1,
      sym_identifier,
    STATE(461), 1,
      sym_type,
    STATE(493), 1,
      sym__sign,
    ACTIONS(19), 2,
      anon_sym_PLUS,
      anon_sym_DASH,
    STATE(46), 2,
      sym_node,
      aux_sym_document_repeat2,
    STATE(491), 2,
      sym__escaped_string,
      sym__raw_string,
    STATE(510), 2,
      sym__bare_identifier,
      sym_string,
    STATE(149), 3,
      sym_kdl_node,
      sym_arco_pure_math_node,
      sym_arco_constraint_node,
    STATE(49), 5,
      sym__linespace,
      sym__newline,
      sym__ws,
      sym_single_line_comment,
      aux_sym_document_repeat1,
    ACTIONS(25), 8,
      anon_sym_expression,
      anon_sym_minimize,
      anon_sym_maximize,
      anon_sym_expr,
      anon_sym_filter,
      anon_sym_if,
      anon_sym_lower,
      anon_sym_upper,
    ACTIONS(176), 10,
      sym_multi_line_comment,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
  [4192] = 20,
    ACTIONS(7), 1,
      sym__normal_bare_identifier,
    ACTIONS(9), 1,
      anon_sym_SLASH_DASH,
    ACTIONS(11), 1,
      anon_sym_LPAREN,
    ACTIONS(13), 1,
      anon_sym_DQUOTE,
    ACTIONS(15), 1,
      aux_sym__raw_string_token1,
    ACTIONS(17), 1,
      aux_sym__raw_string_token3,
    ACTIONS(23), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(27), 1,
      anon_sym_constraint,
    ACTIONS(178), 1,
      anon_sym_RBRACE,
    STATE(294), 1,
      sym_identifier,
    STATE(461), 1,
      sym_type,
    STATE(493), 1,
      sym__sign,
    ACTIONS(19), 2,
      anon_sym_PLUS,
      anon_sym_DASH,
    STATE(36), 2,
      sym_node,
      aux_sym_document_repeat2,
    STATE(491), 2,
      sym__escaped_string,
      sym__raw_string,
    STATE(510), 2,
      sym__bare_identifier,
      sym_string,
    STATE(149), 3,
      sym_kdl_node,
      sym_arco_pure_math_node,
      sym_arco_constraint_node,
    STATE(58), 5,
      sym__linespace,
      sym__newline,
      sym__ws,
      sym_single_line_comment,
      aux_sym_document_repeat1,
    ACTIONS(25), 8,
      anon_sym_expression,
      anon_sym_minimize,
      anon_sym_maximize,
      anon_sym_expr,
      anon_sym_filter,
      anon_sym_if,
      anon_sym_lower,
      anon_sym_upper,
    ACTIONS(180), 10,
      sym_multi_line_comment,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
  [4279] = 20,
    ACTIONS(7), 1,
      sym__normal_bare_identifier,
    ACTIONS(9), 1,
      anon_sym_SLASH_DASH,
    ACTIONS(11), 1,
      anon_sym_LPAREN,
    ACTIONS(13), 1,
      anon_sym_DQUOTE,
    ACTIONS(15), 1,
      aux_sym__raw_string_token1,
    ACTIONS(17), 1,
      aux_sym__raw_string_token3,
    ACTIONS(23), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(27), 1,
      anon_sym_constraint,
    ACTIONS(182), 1,
      anon_sym_RBRACE,
    STATE(294), 1,
      sym_identifier,
    STATE(461), 1,
      sym_type,
    STATE(493), 1,
      sym__sign,
    ACTIONS(19), 2,
      anon_sym_PLUS,
      anon_sym_DASH,
    STATE(30), 2,
      sym_node,
      aux_sym_document_repeat2,
    STATE(491), 2,
      sym__escaped_string,
      sym__raw_string,
    STATE(510), 2,
      sym__bare_identifier,
      sym_string,
    STATE(149), 3,
      sym_kdl_node,
      sym_arco_pure_math_node,
      sym_arco_constraint_node,
    STATE(57), 5,
      sym__linespace,
      sym__newline,
      sym__ws,
      sym_single_line_comment,
      aux_sym_document_repeat1,
    ACTIONS(25), 8,
      anon_sym_expression,
      anon_sym_minimize,
      anon_sym_maximize,
      anon_sym_expr,
      anon_sym_filter,
      anon_sym_if,
      anon_sym_lower,
      anon_sym_upper,
    ACTIONS(184), 10,
      sym_multi_line_comment,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
  [4366] = 20,
    ACTIONS(7), 1,
      sym__normal_bare_identifier,
    ACTIONS(9), 1,
      anon_sym_SLASH_DASH,
    ACTIONS(11), 1,
      anon_sym_LPAREN,
    ACTIONS(13), 1,
      anon_sym_DQUOTE,
    ACTIONS(15), 1,
      aux_sym__raw_string_token1,
    ACTIONS(17), 1,
      aux_sym__raw_string_token3,
    ACTIONS(23), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(27), 1,
      anon_sym_constraint,
    ACTIONS(186), 1,
      anon_sym_RBRACE,
    STATE(294), 1,
      sym_identifier,
    STATE(461), 1,
      sym_type,
    STATE(493), 1,
      sym__sign,
    ACTIONS(19), 2,
      anon_sym_PLUS,
      anon_sym_DASH,
    STATE(30), 2,
      sym_node,
      aux_sym_document_repeat2,
    STATE(491), 2,
      sym__escaped_string,
      sym__raw_string,
    STATE(510), 2,
      sym__bare_identifier,
      sym_string,
    STATE(149), 3,
      sym_kdl_node,
      sym_arco_pure_math_node,
      sym_arco_constraint_node,
    STATE(50), 5,
      sym__linespace,
      sym__newline,
      sym__ws,
      sym_single_line_comment,
      aux_sym_document_repeat1,
    ACTIONS(25), 8,
      anon_sym_expression,
      anon_sym_minimize,
      anon_sym_maximize,
      anon_sym_expr,
      anon_sym_filter,
      anon_sym_if,
      anon_sym_lower,
      anon_sym_upper,
    ACTIONS(188), 10,
      sym_multi_line_comment,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
  [4453] = 20,
    ACTIONS(7), 1,
      sym__normal_bare_identifier,
    ACTIONS(9), 1,
      anon_sym_SLASH_DASH,
    ACTIONS(11), 1,
      anon_sym_LPAREN,
    ACTIONS(13), 1,
      anon_sym_DQUOTE,
    ACTIONS(15), 1,
      aux_sym__raw_string_token1,
    ACTIONS(17), 1,
      aux_sym__raw_string_token3,
    ACTIONS(23), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(27), 1,
      anon_sym_constraint,
    ACTIONS(178), 1,
      anon_sym_RBRACE,
    STATE(294), 1,
      sym_identifier,
    STATE(461), 1,
      sym_type,
    STATE(493), 1,
      sym__sign,
    ACTIONS(19), 2,
      anon_sym_PLUS,
      anon_sym_DASH,
    STATE(30), 2,
      sym_node,
      aux_sym_document_repeat2,
    STATE(491), 2,
      sym__escaped_string,
      sym__raw_string,
    STATE(510), 2,
      sym__bare_identifier,
      sym_string,
    STATE(149), 3,
      sym_kdl_node,
      sym_arco_pure_math_node,
      sym_arco_constraint_node,
    STATE(58), 5,
      sym__linespace,
      sym__newline,
      sym__ws,
      sym_single_line_comment,
      aux_sym_document_repeat1,
    ACTIONS(25), 8,
      anon_sym_expression,
      anon_sym_minimize,
      anon_sym_maximize,
      anon_sym_expr,
      anon_sym_filter,
      anon_sym_if,
      anon_sym_lower,
      anon_sym_upper,
    ACTIONS(180), 10,
      sym_multi_line_comment,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
  [4540] = 20,
    ACTIONS(7), 1,
      sym__normal_bare_identifier,
    ACTIONS(9), 1,
      anon_sym_SLASH_DASH,
    ACTIONS(11), 1,
      anon_sym_LPAREN,
    ACTIONS(13), 1,
      anon_sym_DQUOTE,
    ACTIONS(15), 1,
      aux_sym__raw_string_token1,
    ACTIONS(17), 1,
      aux_sym__raw_string_token3,
    ACTIONS(23), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(27), 1,
      anon_sym_constraint,
    ACTIONS(190), 1,
      anon_sym_RBRACE,
    STATE(294), 1,
      sym_identifier,
    STATE(461), 1,
      sym_type,
    STATE(493), 1,
      sym__sign,
    ACTIONS(19), 2,
      anon_sym_PLUS,
      anon_sym_DASH,
    STATE(43), 2,
      sym_node,
      aux_sym_document_repeat2,
    STATE(491), 2,
      sym__escaped_string,
      sym__raw_string,
    STATE(510), 2,
      sym__bare_identifier,
      sym_string,
    STATE(149), 3,
      sym_kdl_node,
      sym_arco_pure_math_node,
      sym_arco_constraint_node,
    STATE(53), 5,
      sym__linespace,
      sym__newline,
      sym__ws,
      sym_single_line_comment,
      aux_sym_document_repeat1,
    ACTIONS(25), 8,
      anon_sym_expression,
      anon_sym_minimize,
      anon_sym_maximize,
      anon_sym_expr,
      anon_sym_filter,
      anon_sym_if,
      anon_sym_lower,
      anon_sym_upper,
    ACTIONS(192), 10,
      sym_multi_line_comment,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
  [4627] = 20,
    ACTIONS(7), 1,
      sym__normal_bare_identifier,
    ACTIONS(9), 1,
      anon_sym_SLASH_DASH,
    ACTIONS(11), 1,
      anon_sym_LPAREN,
    ACTIONS(13), 1,
      anon_sym_DQUOTE,
    ACTIONS(15), 1,
      aux_sym__raw_string_token1,
    ACTIONS(17), 1,
      aux_sym__raw_string_token3,
    ACTIONS(23), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(27), 1,
      anon_sym_constraint,
    ACTIONS(194), 1,
      ts_builtin_sym_end,
    STATE(294), 1,
      sym_identifier,
    STATE(461), 1,
      sym_type,
    STATE(493), 1,
      sym__sign,
    ACTIONS(19), 2,
      anon_sym_PLUS,
      anon_sym_DASH,
    STATE(32), 2,
      sym_node,
      aux_sym_document_repeat2,
    STATE(491), 2,
      sym__escaped_string,
      sym__raw_string,
    STATE(510), 2,
      sym__bare_identifier,
      sym_string,
    STATE(149), 3,
      sym_kdl_node,
      sym_arco_pure_math_node,
      sym_arco_constraint_node,
    STATE(63), 5,
      sym__linespace,
      sym__newline,
      sym__ws,
      sym_single_line_comment,
      aux_sym_document_repeat1,
    ACTIONS(25), 8,
      anon_sym_expression,
      anon_sym_minimize,
      anon_sym_maximize,
      anon_sym_expr,
      anon_sym_filter,
      anon_sym_if,
      anon_sym_lower,
      anon_sym_upper,
    ACTIONS(196), 10,
      sym_multi_line_comment,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
  [4714] = 20,
    ACTIONS(7), 1,
      sym__normal_bare_identifier,
    ACTIONS(9), 1,
      anon_sym_SLASH_DASH,
    ACTIONS(11), 1,
      anon_sym_LPAREN,
    ACTIONS(13), 1,
      anon_sym_DQUOTE,
    ACTIONS(15), 1,
      aux_sym__raw_string_token1,
    ACTIONS(17), 1,
      aux_sym__raw_string_token3,
    ACTIONS(23), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(27), 1,
      anon_sym_constraint,
    ACTIONS(170), 1,
      anon_sym_RBRACE,
    STATE(294), 1,
      sym_identifier,
    STATE(461), 1,
      sym_type,
    STATE(493), 1,
      sym__sign,
    ACTIONS(19), 2,
      anon_sym_PLUS,
      anon_sym_DASH,
    STATE(30), 2,
      sym_node,
      aux_sym_document_repeat2,
    STATE(491), 2,
      sym__escaped_string,
      sym__raw_string,
    STATE(510), 2,
      sym__bare_identifier,
      sym_string,
    STATE(149), 3,
      sym_kdl_node,
      sym_arco_pure_math_node,
      sym_arco_constraint_node,
    STATE(65), 5,
      sym__linespace,
      sym__newline,
      sym__ws,
      sym_single_line_comment,
      aux_sym_document_repeat1,
    ACTIONS(25), 8,
      anon_sym_expression,
      anon_sym_minimize,
      anon_sym_maximize,
      anon_sym_expr,
      anon_sym_filter,
      anon_sym_if,
      anon_sym_lower,
      anon_sym_upper,
    ACTIONS(172), 10,
      sym_multi_line_comment,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
  [4801] = 22,
    ACTIONS(7), 1,
      sym__normal_bare_identifier,
    ACTIONS(13), 1,
      anon_sym_DQUOTE,
    ACTIONS(15), 1,
      aux_sym__raw_string_token1,
    ACTIONS(17), 1,
      aux_sym__raw_string_token3,
    ACTIONS(27), 1,
      anon_sym_constraint,
    ACTIONS(198), 1,
      anon_sym_SLASH_DASH,
    ACTIONS(200), 1,
      anon_sym_RBRACE,
    ACTIONS(202), 1,
      anon_sym_LPAREN,
    ACTIONS(210), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(212), 1,
      sym_arco_constraint_math_text,
    STATE(33), 1,
      sym_node,
    STATE(294), 1,
      sym_identifier,
    STATE(461), 1,
      sym_type,
    STATE(493), 1,
      sym__sign,
    ACTIONS(204), 2,
      anon_sym_PLUS,
      anon_sym_DASH,
    ACTIONS(206), 2,
      sym_multi_line_comment,
      aux_sym__newline_token1,
    STATE(491), 2,
      sym__escaped_string,
      sym__raw_string,
    STATE(510), 2,
      sym__bare_identifier,
      sym_string,
    STATE(149), 3,
      sym_kdl_node,
      sym_arco_pure_math_node,
      sym_arco_constraint_node,
    STATE(68), 5,
      sym__linespace,
      sym__newline,
      sym__ws,
      sym_single_line_comment,
      aux_sym_document_repeat1,
    ACTIONS(25), 8,
      anon_sym_expression,
      anon_sym_minimize,
      anon_sym_maximize,
      anon_sym_expr,
      anon_sym_filter,
      anon_sym_if,
      anon_sym_lower,
      anon_sym_upper,
    ACTIONS(208), 8,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
  [4892] = 20,
    ACTIONS(7), 1,
      sym__normal_bare_identifier,
    ACTIONS(9), 1,
      anon_sym_SLASH_DASH,
    ACTIONS(11), 1,
      anon_sym_LPAREN,
    ACTIONS(13), 1,
      anon_sym_DQUOTE,
    ACTIONS(15), 1,
      aux_sym__raw_string_token1,
    ACTIONS(17), 1,
      aux_sym__raw_string_token3,
    ACTIONS(23), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(27), 1,
      anon_sym_constraint,
    ACTIONS(214), 1,
      anon_sym_RBRACE,
    STATE(294), 1,
      sym_identifier,
    STATE(461), 1,
      sym_type,
    STATE(493), 1,
      sym__sign,
    ACTIONS(19), 2,
      anon_sym_PLUS,
      anon_sym_DASH,
    STATE(30), 2,
      sym_node,
      aux_sym_document_repeat2,
    STATE(491), 2,
      sym__escaped_string,
      sym__raw_string,
    STATE(510), 2,
      sym__bare_identifier,
      sym_string,
    STATE(149), 3,
      sym_kdl_node,
      sym_arco_pure_math_node,
      sym_arco_constraint_node,
    STATE(67), 5,
      sym__linespace,
      sym__newline,
      sym__ws,
      sym_single_line_comment,
      aux_sym_document_repeat1,
    ACTIONS(25), 8,
      anon_sym_expression,
      anon_sym_minimize,
      anon_sym_maximize,
      anon_sym_expr,
      anon_sym_filter,
      anon_sym_if,
      anon_sym_lower,
      anon_sym_upper,
    ACTIONS(216), 10,
      sym_multi_line_comment,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
  [4979] = 22,
    ACTIONS(7), 1,
      sym__normal_bare_identifier,
    ACTIONS(13), 1,
      anon_sym_DQUOTE,
    ACTIONS(15), 1,
      aux_sym__raw_string_token1,
    ACTIONS(17), 1,
      aux_sym__raw_string_token3,
    ACTIONS(27), 1,
      anon_sym_constraint,
    ACTIONS(198), 1,
      anon_sym_SLASH_DASH,
    ACTIONS(202), 1,
      anon_sym_LPAREN,
    ACTIONS(210), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(218), 1,
      anon_sym_RBRACE,
    ACTIONS(224), 1,
      sym_arco_constraint_math_text,
    STATE(39), 1,
      sym_node,
    STATE(294), 1,
      sym_identifier,
    STATE(461), 1,
      sym_type,
    STATE(493), 1,
      sym__sign,
    ACTIONS(204), 2,
      anon_sym_PLUS,
      anon_sym_DASH,
    ACTIONS(220), 2,
      sym_multi_line_comment,
      aux_sym__newline_token1,
    STATE(491), 2,
      sym__escaped_string,
      sym__raw_string,
    STATE(510), 2,
      sym__bare_identifier,
      sym_string,
    STATE(149), 3,
      sym_kdl_node,
      sym_arco_pure_math_node,
      sym_arco_constraint_node,
    STATE(48), 5,
      sym__linespace,
      sym__newline,
      sym__ws,
      sym_single_line_comment,
      aux_sym_document_repeat1,
    ACTIONS(25), 8,
      anon_sym_expression,
      anon_sym_minimize,
      anon_sym_maximize,
      anon_sym_expr,
      anon_sym_filter,
      anon_sym_if,
      anon_sym_lower,
      anon_sym_upper,
    ACTIONS(222), 8,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
  [5070] = 22,
    ACTIONS(7), 1,
      sym__normal_bare_identifier,
    ACTIONS(13), 1,
      anon_sym_DQUOTE,
    ACTIONS(15), 1,
      aux_sym__raw_string_token1,
    ACTIONS(17), 1,
      aux_sym__raw_string_token3,
    ACTIONS(27), 1,
      anon_sym_constraint,
    ACTIONS(198), 1,
      anon_sym_SLASH_DASH,
    ACTIONS(202), 1,
      anon_sym_LPAREN,
    ACTIONS(210), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(226), 1,
      anon_sym_RBRACE,
    ACTIONS(232), 1,
      sym_arco_constraint_math_text,
    STATE(31), 1,
      sym_node,
    STATE(294), 1,
      sym_identifier,
    STATE(461), 1,
      sym_type,
    STATE(493), 1,
      sym__sign,
    ACTIONS(204), 2,
      anon_sym_PLUS,
      anon_sym_DASH,
    ACTIONS(228), 2,
      sym_multi_line_comment,
      aux_sym__newline_token1,
    STATE(491), 2,
      sym__escaped_string,
      sym__raw_string,
    STATE(510), 2,
      sym__bare_identifier,
      sym_string,
    STATE(149), 3,
      sym_kdl_node,
      sym_arco_pure_math_node,
      sym_arco_constraint_node,
    STATE(56), 5,
      sym__linespace,
      sym__newline,
      sym__ws,
      sym_single_line_comment,
      aux_sym_document_repeat1,
    ACTIONS(25), 8,
      anon_sym_expression,
      anon_sym_minimize,
      anon_sym_maximize,
      anon_sym_expr,
      anon_sym_filter,
      anon_sym_if,
      anon_sym_lower,
      anon_sym_upper,
    ACTIONS(230), 8,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
  [5161] = 20,
    ACTIONS(7), 1,
      sym__normal_bare_identifier,
    ACTIONS(9), 1,
      anon_sym_SLASH_DASH,
    ACTIONS(11), 1,
      anon_sym_LPAREN,
    ACTIONS(13), 1,
      anon_sym_DQUOTE,
    ACTIONS(15), 1,
      aux_sym__raw_string_token1,
    ACTIONS(17), 1,
      aux_sym__raw_string_token3,
    ACTIONS(23), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(27), 1,
      anon_sym_constraint,
    ACTIONS(194), 1,
      ts_builtin_sym_end,
    STATE(294), 1,
      sym_identifier,
    STATE(461), 1,
      sym_type,
    STATE(493), 1,
      sym__sign,
    ACTIONS(19), 2,
      anon_sym_PLUS,
      anon_sym_DASH,
    STATE(30), 2,
      sym_node,
      aux_sym_document_repeat2,
    STATE(491), 2,
      sym__escaped_string,
      sym__raw_string,
    STATE(510), 2,
      sym__bare_identifier,
      sym_string,
    STATE(149), 3,
      sym_kdl_node,
      sym_arco_pure_math_node,
      sym_arco_constraint_node,
    STATE(63), 5,
      sym__linespace,
      sym__newline,
      sym__ws,
      sym_single_line_comment,
      aux_sym_document_repeat1,
    ACTIONS(25), 8,
      anon_sym_expression,
      anon_sym_minimize,
      anon_sym_maximize,
      anon_sym_expr,
      anon_sym_filter,
      anon_sym_if,
      anon_sym_lower,
      anon_sym_upper,
    ACTIONS(196), 10,
      sym_multi_line_comment,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
  [5248] = 20,
    ACTIONS(7), 1,
      sym__normal_bare_identifier,
    ACTIONS(9), 1,
      anon_sym_SLASH_DASH,
    ACTIONS(11), 1,
      anon_sym_LPAREN,
    ACTIONS(13), 1,
      anon_sym_DQUOTE,
    ACTIONS(15), 1,
      aux_sym__raw_string_token1,
    ACTIONS(17), 1,
      aux_sym__raw_string_token3,
    ACTIONS(23), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(27), 1,
      anon_sym_constraint,
    ACTIONS(214), 1,
      anon_sym_RBRACE,
    STATE(294), 1,
      sym_identifier,
    STATE(461), 1,
      sym_type,
    STATE(493), 1,
      sym__sign,
    ACTIONS(19), 2,
      anon_sym_PLUS,
      anon_sym_DASH,
    STATE(37), 2,
      sym_node,
      aux_sym_document_repeat2,
    STATE(491), 2,
      sym__escaped_string,
      sym__raw_string,
    STATE(510), 2,
      sym__bare_identifier,
      sym_string,
    STATE(149), 3,
      sym_kdl_node,
      sym_arco_pure_math_node,
      sym_arco_constraint_node,
    STATE(67), 5,
      sym__linespace,
      sym__newline,
      sym__ws,
      sym_single_line_comment,
      aux_sym_document_repeat1,
    ACTIONS(25), 8,
      anon_sym_expression,
      anon_sym_minimize,
      anon_sym_maximize,
      anon_sym_expr,
      anon_sym_filter,
      anon_sym_if,
      anon_sym_lower,
      anon_sym_upper,
    ACTIONS(216), 10,
      sym_multi_line_comment,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
  [5335] = 20,
    ACTIONS(7), 1,
      sym__normal_bare_identifier,
    ACTIONS(9), 1,
      anon_sym_SLASH_DASH,
    ACTIONS(11), 1,
      anon_sym_LPAREN,
    ACTIONS(13), 1,
      anon_sym_DQUOTE,
    ACTIONS(15), 1,
      aux_sym__raw_string_token1,
    ACTIONS(17), 1,
      aux_sym__raw_string_token3,
    ACTIONS(23), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(27), 1,
      anon_sym_constraint,
    ACTIONS(234), 1,
      anon_sym_RBRACE,
    STATE(47), 1,
      sym_node,
    STATE(294), 1,
      sym_identifier,
    STATE(461), 1,
      sym_type,
    STATE(493), 1,
      sym__sign,
    ACTIONS(19), 2,
      anon_sym_PLUS,
      anon_sym_DASH,
    STATE(491), 2,
      sym__escaped_string,
      sym__raw_string,
    STATE(510), 2,
      sym__bare_identifier,
      sym_string,
    STATE(149), 3,
      sym_kdl_node,
      sym_arco_pure_math_node,
      sym_arco_constraint_node,
    STATE(87), 5,
      sym__linespace,
      sym__newline,
      sym__ws,
      sym_single_line_comment,
      aux_sym_document_repeat1,
    ACTIONS(25), 8,
      anon_sym_expression,
      anon_sym_minimize,
      anon_sym_maximize,
      anon_sym_expr,
      anon_sym_filter,
      anon_sym_if,
      anon_sym_lower,
      anon_sym_upper,
    ACTIONS(236), 10,
      sym_multi_line_comment,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
  [5421] = 20,
    ACTIONS(7), 1,
      sym__normal_bare_identifier,
    ACTIONS(9), 1,
      anon_sym_SLASH_DASH,
    ACTIONS(11), 1,
      anon_sym_LPAREN,
    ACTIONS(13), 1,
      anon_sym_DQUOTE,
    ACTIONS(15), 1,
      aux_sym__raw_string_token1,
    ACTIONS(17), 1,
      aux_sym__raw_string_token3,
    ACTIONS(23), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(27), 1,
      anon_sym_constraint,
    ACTIONS(194), 1,
      ts_builtin_sym_end,
    STATE(135), 1,
      sym_node,
    STATE(294), 1,
      sym_identifier,
    STATE(461), 1,
      sym_type,
    STATE(493), 1,
      sym__sign,
    ACTIONS(19), 2,
      anon_sym_PLUS,
      anon_sym_DASH,
    STATE(491), 2,
      sym__escaped_string,
      sym__raw_string,
    STATE(510), 2,
      sym__bare_identifier,
      sym_string,
    STATE(149), 3,
      sym_kdl_node,
      sym_arco_pure_math_node,
      sym_arco_constraint_node,
    STATE(82), 5,
      sym__linespace,
      sym__newline,
      sym__ws,
      sym_single_line_comment,
      aux_sym_document_repeat1,
    ACTIONS(25), 8,
      anon_sym_expression,
      anon_sym_minimize,
      anon_sym_maximize,
      anon_sym_expr,
      anon_sym_filter,
      anon_sym_if,
      anon_sym_lower,
      anon_sym_upper,
    ACTIONS(238), 10,
      sym_multi_line_comment,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
  [5507] = 20,
    ACTIONS(7), 1,
      sym__normal_bare_identifier,
    ACTIONS(9), 1,
      anon_sym_SLASH_DASH,
    ACTIONS(11), 1,
      anon_sym_LPAREN,
    ACTIONS(13), 1,
      anon_sym_DQUOTE,
    ACTIONS(15), 1,
      aux_sym__raw_string_token1,
    ACTIONS(17), 1,
      aux_sym__raw_string_token3,
    ACTIONS(23), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(27), 1,
      anon_sym_constraint,
    ACTIONS(240), 1,
      anon_sym_RBRACE,
    STATE(135), 1,
      sym_node,
    STATE(294), 1,
      sym_identifier,
    STATE(461), 1,
      sym_type,
    STATE(493), 1,
      sym__sign,
    ACTIONS(19), 2,
      anon_sym_PLUS,
      anon_sym_DASH,
    STATE(491), 2,
      sym__escaped_string,
      sym__raw_string,
    STATE(510), 2,
      sym__bare_identifier,
      sym_string,
    STATE(149), 3,
      sym_kdl_node,
      sym_arco_pure_math_node,
      sym_arco_constraint_node,
    STATE(82), 5,
      sym__linespace,
      sym__newline,
      sym__ws,
      sym_single_line_comment,
      aux_sym_document_repeat1,
    ACTIONS(25), 8,
      anon_sym_expression,
      anon_sym_minimize,
      anon_sym_maximize,
      anon_sym_expr,
      anon_sym_filter,
      anon_sym_if,
      anon_sym_lower,
      anon_sym_upper,
    ACTIONS(238), 10,
      sym_multi_line_comment,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
  [5593] = 20,
    ACTIONS(7), 1,
      sym__normal_bare_identifier,
    ACTIONS(9), 1,
      anon_sym_SLASH_DASH,
    ACTIONS(11), 1,
      anon_sym_LPAREN,
    ACTIONS(13), 1,
      anon_sym_DQUOTE,
    ACTIONS(15), 1,
      aux_sym__raw_string_token1,
    ACTIONS(17), 1,
      aux_sym__raw_string_token3,
    ACTIONS(23), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(27), 1,
      anon_sym_constraint,
    ACTIONS(170), 1,
      anon_sym_RBRACE,
    STATE(135), 1,
      sym_node,
    STATE(294), 1,
      sym_identifier,
    STATE(461), 1,
      sym_type,
    STATE(493), 1,
      sym__sign,
    ACTIONS(19), 2,
      anon_sym_PLUS,
      anon_sym_DASH,
    STATE(491), 2,
      sym__escaped_string,
      sym__raw_string,
    STATE(510), 2,
      sym__bare_identifier,
      sym_string,
    STATE(149), 3,
      sym_kdl_node,
      sym_arco_pure_math_node,
      sym_arco_constraint_node,
    STATE(82), 5,
      sym__linespace,
      sym__newline,
      sym__ws,
      sym_single_line_comment,
      aux_sym_document_repeat1,
    ACTIONS(25), 8,
      anon_sym_expression,
      anon_sym_minimize,
      anon_sym_maximize,
      anon_sym_expr,
      anon_sym_filter,
      anon_sym_if,
      anon_sym_lower,
      anon_sym_upper,
    ACTIONS(238), 10,
      sym_multi_line_comment,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
  [5679] = 20,
    ACTIONS(7), 1,
      sym__normal_bare_identifier,
    ACTIONS(9), 1,
      anon_sym_SLASH_DASH,
    ACTIONS(11), 1,
      anon_sym_LPAREN,
    ACTIONS(13), 1,
      anon_sym_DQUOTE,
    ACTIONS(15), 1,
      aux_sym__raw_string_token1,
    ACTIONS(17), 1,
      aux_sym__raw_string_token3,
    ACTIONS(23), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(27), 1,
      anon_sym_constraint,
    ACTIONS(242), 1,
      anon_sym_RBRACE,
    STATE(39), 1,
      sym_node,
    STATE(294), 1,
      sym_identifier,
    STATE(461), 1,
      sym_type,
    STATE(493), 1,
      sym__sign,
    ACTIONS(19), 2,
      anon_sym_PLUS,
      anon_sym_DASH,
    STATE(491), 2,
      sym__escaped_string,
      sym__raw_string,
    STATE(510), 2,
      sym__bare_identifier,
      sym_string,
    STATE(149), 3,
      sym_kdl_node,
      sym_arco_pure_math_node,
      sym_arco_constraint_node,
    STATE(61), 5,
      sym__linespace,
      sym__newline,
      sym__ws,
      sym_single_line_comment,
      aux_sym_document_repeat1,
    ACTIONS(25), 8,
      anon_sym_expression,
      anon_sym_minimize,
      anon_sym_maximize,
      anon_sym_expr,
      anon_sym_filter,
      anon_sym_if,
      anon_sym_lower,
      anon_sym_upper,
    ACTIONS(244), 10,
      sym_multi_line_comment,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
  [5765] = 20,
    ACTIONS(7), 1,
      sym__normal_bare_identifier,
    ACTIONS(9), 1,
      anon_sym_SLASH_DASH,
    ACTIONS(11), 1,
      anon_sym_LPAREN,
    ACTIONS(13), 1,
      anon_sym_DQUOTE,
    ACTIONS(15), 1,
      aux_sym__raw_string_token1,
    ACTIONS(17), 1,
      aux_sym__raw_string_token3,
    ACTIONS(23), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(27), 1,
      anon_sym_constraint,
    ACTIONS(214), 1,
      anon_sym_RBRACE,
    STATE(135), 1,
      sym_node,
    STATE(294), 1,
      sym_identifier,
    STATE(461), 1,
      sym_type,
    STATE(493), 1,
      sym__sign,
    ACTIONS(19), 2,
      anon_sym_PLUS,
      anon_sym_DASH,
    STATE(491), 2,
      sym__escaped_string,
      sym__raw_string,
    STATE(510), 2,
      sym__bare_identifier,
      sym_string,
    STATE(149), 3,
      sym_kdl_node,
      sym_arco_pure_math_node,
      sym_arco_constraint_node,
    STATE(82), 5,
      sym__linespace,
      sym__newline,
      sym__ws,
      sym_single_line_comment,
      aux_sym_document_repeat1,
    ACTIONS(25), 8,
      anon_sym_expression,
      anon_sym_minimize,
      anon_sym_maximize,
      anon_sym_expr,
      anon_sym_filter,
      anon_sym_if,
      anon_sym_lower,
      anon_sym_upper,
    ACTIONS(238), 10,
      sym_multi_line_comment,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
  [5851] = 20,
    ACTIONS(7), 1,
      sym__normal_bare_identifier,
    ACTIONS(9), 1,
      anon_sym_SLASH_DASH,
    ACTIONS(11), 1,
      anon_sym_LPAREN,
    ACTIONS(13), 1,
      anon_sym_DQUOTE,
    ACTIONS(15), 1,
      aux_sym__raw_string_token1,
    ACTIONS(17), 1,
      aux_sym__raw_string_token3,
    ACTIONS(23), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(27), 1,
      anon_sym_constraint,
    ACTIONS(162), 1,
      anon_sym_RBRACE,
    STATE(33), 1,
      sym_node,
    STATE(294), 1,
      sym_identifier,
    STATE(461), 1,
      sym_type,
    STATE(493), 1,
      sym__sign,
    ACTIONS(19), 2,
      anon_sym_PLUS,
      anon_sym_DASH,
    STATE(491), 2,
      sym__escaped_string,
      sym__raw_string,
    STATE(510), 2,
      sym__bare_identifier,
      sym_string,
    STATE(149), 3,
      sym_kdl_node,
      sym_arco_pure_math_node,
      sym_arco_constraint_node,
    STATE(88), 5,
      sym__linespace,
      sym__newline,
      sym__ws,
      sym_single_line_comment,
      aux_sym_document_repeat1,
    ACTIONS(25), 8,
      anon_sym_expression,
      anon_sym_minimize,
      anon_sym_maximize,
      anon_sym_expr,
      anon_sym_filter,
      anon_sym_if,
      anon_sym_lower,
      anon_sym_upper,
    ACTIONS(246), 10,
      sym_multi_line_comment,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
  [5937] = 20,
    ACTIONS(7), 1,
      sym__normal_bare_identifier,
    ACTIONS(9), 1,
      anon_sym_SLASH_DASH,
    ACTIONS(11), 1,
      anon_sym_LPAREN,
    ACTIONS(13), 1,
      anon_sym_DQUOTE,
    ACTIONS(15), 1,
      aux_sym__raw_string_token1,
    ACTIONS(17), 1,
      aux_sym__raw_string_token3,
    ACTIONS(23), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(27), 1,
      anon_sym_constraint,
    ACTIONS(162), 1,
      anon_sym_RBRACE,
    STATE(33), 1,
      sym_node,
    STATE(294), 1,
      sym_identifier,
    STATE(461), 1,
      sym_type,
    STATE(493), 1,
      sym__sign,
    ACTIONS(19), 2,
      anon_sym_PLUS,
      anon_sym_DASH,
    STATE(491), 2,
      sym__escaped_string,
      sym__raw_string,
    STATE(510), 2,
      sym__bare_identifier,
      sym_string,
    STATE(149), 3,
      sym_kdl_node,
      sym_arco_pure_math_node,
      sym_arco_constraint_node,
    STATE(59), 5,
      sym__linespace,
      sym__newline,
      sym__ws,
      sym_single_line_comment,
      aux_sym_document_repeat1,
    ACTIONS(25), 8,
      anon_sym_expression,
      anon_sym_minimize,
      anon_sym_maximize,
      anon_sym_expr,
      anon_sym_filter,
      anon_sym_if,
      anon_sym_lower,
      anon_sym_upper,
    ACTIONS(248), 10,
      sym_multi_line_comment,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
  [6023] = 20,
    ACTIONS(7), 1,
      sym__normal_bare_identifier,
    ACTIONS(9), 1,
      anon_sym_SLASH_DASH,
    ACTIONS(11), 1,
      anon_sym_LPAREN,
    ACTIONS(13), 1,
      anon_sym_DQUOTE,
    ACTIONS(15), 1,
      aux_sym__raw_string_token1,
    ACTIONS(17), 1,
      aux_sym__raw_string_token3,
    ACTIONS(23), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(27), 1,
      anon_sym_constraint,
    ACTIONS(200), 1,
      anon_sym_RBRACE,
    STATE(33), 1,
      sym_node,
    STATE(294), 1,
      sym_identifier,
    STATE(461), 1,
      sym_type,
    STATE(493), 1,
      sym__sign,
    ACTIONS(19), 2,
      anon_sym_PLUS,
      anon_sym_DASH,
    STATE(491), 2,
      sym__escaped_string,
      sym__raw_string,
    STATE(510), 2,
      sym__bare_identifier,
      sym_string,
    STATE(149), 3,
      sym_kdl_node,
      sym_arco_pure_math_node,
      sym_arco_constraint_node,
    STATE(88), 5,
      sym__linespace,
      sym__newline,
      sym__ws,
      sym_single_line_comment,
      aux_sym_document_repeat1,
    ACTIONS(25), 8,
      anon_sym_expression,
      anon_sym_minimize,
      anon_sym_maximize,
      anon_sym_expr,
      anon_sym_filter,
      anon_sym_if,
      anon_sym_lower,
      anon_sym_upper,
    ACTIONS(246), 10,
      sym_multi_line_comment,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
  [6109] = 20,
    ACTIONS(7), 1,
      sym__normal_bare_identifier,
    ACTIONS(9), 1,
      anon_sym_SLASH_DASH,
    ACTIONS(11), 1,
      anon_sym_LPAREN,
    ACTIONS(13), 1,
      anon_sym_DQUOTE,
    ACTIONS(15), 1,
      aux_sym__raw_string_token1,
    ACTIONS(17), 1,
      aux_sym__raw_string_token3,
    ACTIONS(23), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(27), 1,
      anon_sym_constraint,
    ACTIONS(250), 1,
      anon_sym_RBRACE,
    STATE(135), 1,
      sym_node,
    STATE(294), 1,
      sym_identifier,
    STATE(461), 1,
      sym_type,
    STATE(493), 1,
      sym__sign,
    ACTIONS(19), 2,
      anon_sym_PLUS,
      anon_sym_DASH,
    STATE(491), 2,
      sym__escaped_string,
      sym__raw_string,
    STATE(510), 2,
      sym__bare_identifier,
      sym_string,
    STATE(149), 3,
      sym_kdl_node,
      sym_arco_pure_math_node,
      sym_arco_constraint_node,
    STATE(82), 5,
      sym__linespace,
      sym__newline,
      sym__ws,
      sym_single_line_comment,
      aux_sym_document_repeat1,
    ACTIONS(25), 8,
      anon_sym_expression,
      anon_sym_minimize,
      anon_sym_maximize,
      anon_sym_expr,
      anon_sym_filter,
      anon_sym_if,
      anon_sym_lower,
      anon_sym_upper,
    ACTIONS(238), 10,
      sym_multi_line_comment,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
  [6195] = 20,
    ACTIONS(7), 1,
      sym__normal_bare_identifier,
    ACTIONS(9), 1,
      anon_sym_SLASH_DASH,
    ACTIONS(11), 1,
      anon_sym_LPAREN,
    ACTIONS(13), 1,
      anon_sym_DQUOTE,
    ACTIONS(15), 1,
      aux_sym__raw_string_token1,
    ACTIONS(17), 1,
      aux_sym__raw_string_token3,
    ACTIONS(23), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(27), 1,
      anon_sym_constraint,
    ACTIONS(182), 1,
      anon_sym_RBRACE,
    STATE(135), 1,
      sym_node,
    STATE(294), 1,
      sym_identifier,
    STATE(461), 1,
      sym_type,
    STATE(493), 1,
      sym__sign,
    ACTIONS(19), 2,
      anon_sym_PLUS,
      anon_sym_DASH,
    STATE(491), 2,
      sym__escaped_string,
      sym__raw_string,
    STATE(510), 2,
      sym__bare_identifier,
      sym_string,
    STATE(149), 3,
      sym_kdl_node,
      sym_arco_pure_math_node,
      sym_arco_constraint_node,
    STATE(82), 5,
      sym__linespace,
      sym__newline,
      sym__ws,
      sym_single_line_comment,
      aux_sym_document_repeat1,
    ACTIONS(25), 8,
      anon_sym_expression,
      anon_sym_minimize,
      anon_sym_maximize,
      anon_sym_expr,
      anon_sym_filter,
      anon_sym_if,
      anon_sym_lower,
      anon_sym_upper,
    ACTIONS(238), 10,
      sym_multi_line_comment,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
  [6281] = 20,
    ACTIONS(7), 1,
      sym__normal_bare_identifier,
    ACTIONS(9), 1,
      anon_sym_SLASH_DASH,
    ACTIONS(11), 1,
      anon_sym_LPAREN,
    ACTIONS(13), 1,
      anon_sym_DQUOTE,
    ACTIONS(15), 1,
      aux_sym__raw_string_token1,
    ACTIONS(17), 1,
      aux_sym__raw_string_token3,
    ACTIONS(23), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(27), 1,
      anon_sym_constraint,
    ACTIONS(170), 1,
      anon_sym_RBRACE,
    STATE(35), 1,
      sym_node,
    STATE(294), 1,
      sym_identifier,
    STATE(461), 1,
      sym_type,
    STATE(493), 1,
      sym__sign,
    ACTIONS(19), 2,
      anon_sym_PLUS,
      anon_sym_DASH,
    STATE(491), 2,
      sym__escaped_string,
      sym__raw_string,
    STATE(510), 2,
      sym__bare_identifier,
      sym_string,
    STATE(149), 3,
      sym_kdl_node,
      sym_arco_pure_math_node,
      sym_arco_constraint_node,
    STATE(83), 5,
      sym__linespace,
      sym__newline,
      sym__ws,
      sym_single_line_comment,
      aux_sym_document_repeat1,
    ACTIONS(25), 8,
      anon_sym_expression,
      anon_sym_minimize,
      anon_sym_maximize,
      anon_sym_expr,
      anon_sym_filter,
      anon_sym_if,
      anon_sym_lower,
      anon_sym_upper,
    ACTIONS(252), 10,
      sym_multi_line_comment,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
  [6367] = 32,
    ACTIONS(11), 1,
      anon_sym_LPAREN,
    ACTIONS(13), 1,
      anon_sym_DQUOTE,
    ACTIONS(15), 1,
      aux_sym__raw_string_token1,
    ACTIONS(17), 1,
      aux_sym__raw_string_token3,
    ACTIONS(29), 1,
      sym__normal_bare_identifier,
    ACTIONS(37), 1,
      anon_sym_null,
    ACTIONS(39), 1,
      sym__digit,
    ACTIONS(43), 1,
      anon_sym_0x,
    ACTIONS(45), 1,
      anon_sym_0o,
    ACTIONS(47), 1,
      anon_sym_0b,
    ACTIONS(254), 1,
      anon_sym_SLASH_DASH,
    ACTIONS(256), 1,
      anon_sym_BSLASH,
    STATE(86), 1,
      aux_sym__node_field_comment_repeat1,
    STATE(98), 1,
      sym__escline,
    STATE(267), 1,
      sym__node_space,
    STATE(270), 1,
      sym_type,
    STATE(299), 1,
      sym__sign,
    STATE(467), 1,
      sym__integer,
    STATE(492), 1,
      sym__bare_identifier,
    STATE(497), 1,
      sym_string,
    STATE(507), 1,
      sym_boolean,
    STATE(518), 1,
      sym_node_field,
    STATE(587), 1,
      sym_identifier,
    ACTIONS(41), 2,
      anon_sym_PLUS,
      anon_sym_DASH,
    ACTIONS(49), 2,
      anon_sym_true,
      anon_sym_false,
    STATE(95), 2,
      sym__ws,
      aux_sym__node_space_repeat1,
    STATE(491), 2,
      sym__escaped_string,
      sym__raw_string,
    STATE(506), 2,
      sym_prop,
      sym_value,
    STATE(516), 2,
      sym__node_field_comment,
      sym__node_field,
    ACTIONS(258), 3,
      sym_multi_line_comment,
      sym__bom,
      sym__unicode_space,
    STATE(508), 3,
      sym_keyword,
      sym_number,
      sym_bare_identifier,
    STATE(501), 4,
      sym__decimal,
      sym__hex,
      sym__octal,
      sym__binary,
  [6477] = 20,
    ACTIONS(7), 1,
      sym__normal_bare_identifier,
    ACTIONS(9), 1,
      anon_sym_SLASH_DASH,
    ACTIONS(11), 1,
      anon_sym_LPAREN,
    ACTIONS(13), 1,
      anon_sym_DQUOTE,
    ACTIONS(15), 1,
      aux_sym__raw_string_token1,
    ACTIONS(17), 1,
      aux_sym__raw_string_token3,
    ACTIONS(23), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(27), 1,
      anon_sym_constraint,
    ACTIONS(190), 1,
      anon_sym_RBRACE,
    STATE(47), 1,
      sym_node,
    STATE(294), 1,
      sym_identifier,
    STATE(461), 1,
      sym_type,
    STATE(493), 1,
      sym__sign,
    ACTIONS(19), 2,
      anon_sym_PLUS,
      anon_sym_DASH,
    STATE(491), 2,
      sym__escaped_string,
      sym__raw_string,
    STATE(510), 2,
      sym__bare_identifier,
      sym_string,
    STATE(149), 3,
      sym_kdl_node,
      sym_arco_pure_math_node,
      sym_arco_constraint_node,
    STATE(87), 5,
      sym__linespace,
      sym__newline,
      sym__ws,
      sym_single_line_comment,
      aux_sym_document_repeat1,
    ACTIONS(25), 8,
      anon_sym_expression,
      anon_sym_minimize,
      anon_sym_maximize,
      anon_sym_expr,
      anon_sym_filter,
      anon_sym_if,
      anon_sym_lower,
      anon_sym_upper,
    ACTIONS(236), 10,
      sym_multi_line_comment,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
  [6563] = 20,
    ACTIONS(7), 1,
      sym__normal_bare_identifier,
    ACTIONS(9), 1,
      anon_sym_SLASH_DASH,
    ACTIONS(11), 1,
      anon_sym_LPAREN,
    ACTIONS(13), 1,
      anon_sym_DQUOTE,
    ACTIONS(15), 1,
      aux_sym__raw_string_token1,
    ACTIONS(17), 1,
      aux_sym__raw_string_token3,
    ACTIONS(23), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(27), 1,
      anon_sym_constraint,
    ACTIONS(260), 1,
      anon_sym_RBRACE,
    STATE(31), 1,
      sym_node,
    STATE(294), 1,
      sym_identifier,
    STATE(461), 1,
      sym_type,
    STATE(493), 1,
      sym__sign,
    ACTIONS(19), 2,
      anon_sym_PLUS,
      anon_sym_DASH,
    STATE(491), 2,
      sym__escaped_string,
      sym__raw_string,
    STATE(510), 2,
      sym__bare_identifier,
      sym_string,
    STATE(149), 3,
      sym_kdl_node,
      sym_arco_pure_math_node,
      sym_arco_constraint_node,
    STATE(54), 5,
      sym__linespace,
      sym__newline,
      sym__ws,
      sym_single_line_comment,
      aux_sym_document_repeat1,
    ACTIONS(25), 8,
      anon_sym_expression,
      anon_sym_minimize,
      anon_sym_maximize,
      anon_sym_expr,
      anon_sym_filter,
      anon_sym_if,
      anon_sym_lower,
      anon_sym_upper,
    ACTIONS(262), 10,
      sym_multi_line_comment,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
  [6649] = 20,
    ACTIONS(7), 1,
      sym__normal_bare_identifier,
    ACTIONS(9), 1,
      anon_sym_SLASH_DASH,
    ACTIONS(11), 1,
      anon_sym_LPAREN,
    ACTIONS(13), 1,
      anon_sym_DQUOTE,
    ACTIONS(15), 1,
      aux_sym__raw_string_token1,
    ACTIONS(17), 1,
      aux_sym__raw_string_token3,
    ACTIONS(23), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(27), 1,
      anon_sym_constraint,
    ACTIONS(166), 1,
      ts_builtin_sym_end,
    STATE(135), 1,
      sym_node,
    STATE(294), 1,
      sym_identifier,
    STATE(461), 1,
      sym_type,
    STATE(493), 1,
      sym__sign,
    ACTIONS(19), 2,
      anon_sym_PLUS,
      anon_sym_DASH,
    STATE(491), 2,
      sym__escaped_string,
      sym__raw_string,
    STATE(510), 2,
      sym__bare_identifier,
      sym_string,
    STATE(149), 3,
      sym_kdl_node,
      sym_arco_pure_math_node,
      sym_arco_constraint_node,
    STATE(82), 5,
      sym__linespace,
      sym__newline,
      sym__ws,
      sym_single_line_comment,
      aux_sym_document_repeat1,
    ACTIONS(25), 8,
      anon_sym_expression,
      anon_sym_minimize,
      anon_sym_maximize,
      anon_sym_expr,
      anon_sym_filter,
      anon_sym_if,
      anon_sym_lower,
      anon_sym_upper,
    ACTIONS(238), 10,
      sym_multi_line_comment,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
  [6735] = 20,
    ACTIONS(7), 1,
      sym__normal_bare_identifier,
    ACTIONS(9), 1,
      anon_sym_SLASH_DASH,
    ACTIONS(11), 1,
      anon_sym_LPAREN,
    ACTIONS(13), 1,
      anon_sym_DQUOTE,
    ACTIONS(15), 1,
      aux_sym__raw_string_token1,
    ACTIONS(17), 1,
      aux_sym__raw_string_token3,
    ACTIONS(23), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(27), 1,
      anon_sym_constraint,
    ACTIONS(174), 1,
      ts_builtin_sym_end,
    STATE(40), 1,
      sym_node,
    STATE(294), 1,
      sym_identifier,
    STATE(461), 1,
      sym_type,
    STATE(493), 1,
      sym__sign,
    ACTIONS(19), 2,
      anon_sym_PLUS,
      anon_sym_DASH,
    STATE(491), 2,
      sym__escaped_string,
      sym__raw_string,
    STATE(510), 2,
      sym__bare_identifier,
      sym_string,
    STATE(149), 3,
      sym_kdl_node,
      sym_arco_pure_math_node,
      sym_arco_constraint_node,
    STATE(85), 5,
      sym__linespace,
      sym__newline,
      sym__ws,
      sym_single_line_comment,
      aux_sym_document_repeat1,
    ACTIONS(25), 8,
      anon_sym_expression,
      anon_sym_minimize,
      anon_sym_maximize,
      anon_sym_expr,
      anon_sym_filter,
      anon_sym_if,
      anon_sym_lower,
      anon_sym_upper,
    ACTIONS(264), 10,
      sym_multi_line_comment,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
  [6821] = 20,
    ACTIONS(7), 1,
      sym__normal_bare_identifier,
    ACTIONS(9), 1,
      anon_sym_SLASH_DASH,
    ACTIONS(11), 1,
      anon_sym_LPAREN,
    ACTIONS(13), 1,
      anon_sym_DQUOTE,
    ACTIONS(15), 1,
      aux_sym__raw_string_token1,
    ACTIONS(17), 1,
      aux_sym__raw_string_token3,
    ACTIONS(23), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(27), 1,
      anon_sym_constraint,
    ACTIONS(178), 1,
      anon_sym_RBRACE,
    STATE(135), 1,
      sym_node,
    STATE(294), 1,
      sym_identifier,
    STATE(461), 1,
      sym_type,
    STATE(493), 1,
      sym__sign,
    ACTIONS(19), 2,
      anon_sym_PLUS,
      anon_sym_DASH,
    STATE(491), 2,
      sym__escaped_string,
      sym__raw_string,
    STATE(510), 2,
      sym__bare_identifier,
      sym_string,
    STATE(149), 3,
      sym_kdl_node,
      sym_arco_pure_math_node,
      sym_arco_constraint_node,
    STATE(82), 5,
      sym__linespace,
      sym__newline,
      sym__ws,
      sym_single_line_comment,
      aux_sym_document_repeat1,
    ACTIONS(25), 8,
      anon_sym_expression,
      anon_sym_minimize,
      anon_sym_maximize,
      anon_sym_expr,
      anon_sym_filter,
      anon_sym_if,
      anon_sym_lower,
      anon_sym_upper,
    ACTIONS(238), 10,
      sym_multi_line_comment,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
  [6907] = 20,
    ACTIONS(7), 1,
      sym__normal_bare_identifier,
    ACTIONS(9), 1,
      anon_sym_SLASH_DASH,
    ACTIONS(11), 1,
      anon_sym_LPAREN,
    ACTIONS(13), 1,
      anon_sym_DQUOTE,
    ACTIONS(15), 1,
      aux_sym__raw_string_token1,
    ACTIONS(17), 1,
      aux_sym__raw_string_token3,
    ACTIONS(23), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(27), 1,
      anon_sym_constraint,
    ACTIONS(266), 1,
      ts_builtin_sym_end,
    STATE(135), 1,
      sym_node,
    STATE(294), 1,
      sym_identifier,
    STATE(461), 1,
      sym_type,
    STATE(493), 1,
      sym__sign,
    ACTIONS(19), 2,
      anon_sym_PLUS,
      anon_sym_DASH,
    STATE(491), 2,
      sym__escaped_string,
      sym__raw_string,
    STATE(510), 2,
      sym__bare_identifier,
      sym_string,
    STATE(149), 3,
      sym_kdl_node,
      sym_arco_pure_math_node,
      sym_arco_constraint_node,
    STATE(82), 5,
      sym__linespace,
      sym__newline,
      sym__ws,
      sym_single_line_comment,
      aux_sym_document_repeat1,
    ACTIONS(25), 8,
      anon_sym_expression,
      anon_sym_minimize,
      anon_sym_maximize,
      anon_sym_expr,
      anon_sym_filter,
      anon_sym_if,
      anon_sym_lower,
      anon_sym_upper,
    ACTIONS(238), 10,
      sym_multi_line_comment,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
  [6993] = 20,
    ACTIONS(7), 1,
      sym__normal_bare_identifier,
    ACTIONS(9), 1,
      anon_sym_SLASH_DASH,
    ACTIONS(11), 1,
      anon_sym_LPAREN,
    ACTIONS(13), 1,
      anon_sym_DQUOTE,
    ACTIONS(15), 1,
      aux_sym__raw_string_token1,
    ACTIONS(17), 1,
      aux_sym__raw_string_token3,
    ACTIONS(23), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(27), 1,
      anon_sym_constraint,
    ACTIONS(186), 1,
      anon_sym_RBRACE,
    STATE(135), 1,
      sym_node,
    STATE(294), 1,
      sym_identifier,
    STATE(461), 1,
      sym_type,
    STATE(493), 1,
      sym__sign,
    ACTIONS(19), 2,
      anon_sym_PLUS,
      anon_sym_DASH,
    STATE(491), 2,
      sym__escaped_string,
      sym__raw_string,
    STATE(510), 2,
      sym__bare_identifier,
      sym_string,
    STATE(149), 3,
      sym_kdl_node,
      sym_arco_pure_math_node,
      sym_arco_constraint_node,
    STATE(82), 5,
      sym__linespace,
      sym__newline,
      sym__ws,
      sym_single_line_comment,
      aux_sym_document_repeat1,
    ACTIONS(25), 8,
      anon_sym_expression,
      anon_sym_minimize,
      anon_sym_maximize,
      anon_sym_expr,
      anon_sym_filter,
      anon_sym_if,
      anon_sym_lower,
      anon_sym_upper,
    ACTIONS(238), 10,
      sym_multi_line_comment,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
  [7079] = 20,
    ACTIONS(7), 1,
      sym__normal_bare_identifier,
    ACTIONS(9), 1,
      anon_sym_SLASH_DASH,
    ACTIONS(11), 1,
      anon_sym_LPAREN,
    ACTIONS(13), 1,
      anon_sym_DQUOTE,
    ACTIONS(15), 1,
      aux_sym__raw_string_token1,
    ACTIONS(17), 1,
      aux_sym__raw_string_token3,
    ACTIONS(23), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(27), 1,
      anon_sym_constraint,
    ACTIONS(268), 1,
      anon_sym_RBRACE,
    STATE(35), 1,
      sym_node,
    STATE(294), 1,
      sym_identifier,
    STATE(461), 1,
      sym_type,
    STATE(493), 1,
      sym__sign,
    ACTIONS(19), 2,
      anon_sym_PLUS,
      anon_sym_DASH,
    STATE(491), 2,
      sym__escaped_string,
      sym__raw_string,
    STATE(510), 2,
      sym__bare_identifier,
      sym_string,
    STATE(149), 3,
      sym_kdl_node,
      sym_arco_pure_math_node,
      sym_arco_constraint_node,
    STATE(83), 5,
      sym__linespace,
      sym__newline,
      sym__ws,
      sym_single_line_comment,
      aux_sym_document_repeat1,
    ACTIONS(25), 8,
      anon_sym_expression,
      anon_sym_minimize,
      anon_sym_maximize,
      anon_sym_expr,
      anon_sym_filter,
      anon_sym_if,
      anon_sym_lower,
      anon_sym_upper,
    ACTIONS(252), 10,
      sym_multi_line_comment,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
  [7165] = 19,
    ACTIONS(7), 1,
      sym__normal_bare_identifier,
    ACTIONS(9), 1,
      anon_sym_SLASH_DASH,
    ACTIONS(11), 1,
      anon_sym_LPAREN,
    ACTIONS(13), 1,
      anon_sym_DQUOTE,
    ACTIONS(15), 1,
      aux_sym__raw_string_token1,
    ACTIONS(17), 1,
      aux_sym__raw_string_token3,
    ACTIONS(23), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(27), 1,
      anon_sym_constraint,
    STATE(135), 1,
      sym_node,
    STATE(294), 1,
      sym_identifier,
    STATE(461), 1,
      sym_type,
    STATE(493), 1,
      sym__sign,
    ACTIONS(19), 2,
      anon_sym_PLUS,
      anon_sym_DASH,
    STATE(491), 2,
      sym__escaped_string,
      sym__raw_string,
    STATE(510), 2,
      sym__bare_identifier,
      sym_string,
    STATE(149), 3,
      sym_kdl_node,
      sym_arco_pure_math_node,
      sym_arco_constraint_node,
    STATE(82), 5,
      sym__linespace,
      sym__newline,
      sym__ws,
      sym_single_line_comment,
      aux_sym_document_repeat1,
    ACTIONS(25), 8,
      anon_sym_expression,
      anon_sym_minimize,
      anon_sym_maximize,
      anon_sym_expr,
      anon_sym_filter,
      anon_sym_if,
      anon_sym_lower,
      anon_sym_upper,
    ACTIONS(238), 10,
      sym_multi_line_comment,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
  [7248] = 31,
    ACTIONS(11), 1,
      anon_sym_LPAREN,
    ACTIONS(13), 1,
      anon_sym_DQUOTE,
    ACTIONS(15), 1,
      aux_sym__raw_string_token1,
    ACTIONS(17), 1,
      aux_sym__raw_string_token3,
    ACTIONS(29), 1,
      sym__normal_bare_identifier,
    ACTIONS(37), 1,
      anon_sym_null,
    ACTIONS(39), 1,
      sym__digit,
    ACTIONS(43), 1,
      anon_sym_0x,
    ACTIONS(45), 1,
      anon_sym_0o,
    ACTIONS(47), 1,
      anon_sym_0b,
    ACTIONS(256), 1,
      anon_sym_BSLASH,
    ACTIONS(270), 1,
      anon_sym_LBRACE,
    STATE(73), 1,
      aux_sym__node_field_comment_repeat1,
    STATE(98), 1,
      sym__escline,
    STATE(267), 1,
      sym__node_space,
    STATE(270), 1,
      sym_type,
    STATE(299), 1,
      sym__sign,
    STATE(467), 1,
      sym__integer,
    STATE(492), 1,
      sym__bare_identifier,
    STATE(497), 1,
      sym_string,
    STATE(507), 1,
      sym_boolean,
    STATE(512), 1,
      sym__node_field,
    STATE(587), 1,
      sym_identifier,
    ACTIONS(41), 2,
      anon_sym_PLUS,
      anon_sym_DASH,
    ACTIONS(49), 2,
      anon_sym_true,
      anon_sym_false,
    STATE(95), 2,
      sym__ws,
      aux_sym__node_space_repeat1,
    STATE(491), 2,
      sym__escaped_string,
      sym__raw_string,
    STATE(506), 2,
      sym_prop,
      sym_value,
    ACTIONS(258), 3,
      sym_multi_line_comment,
      sym__bom,
      sym__unicode_space,
    STATE(508), 3,
      sym_keyword,
      sym_number,
      sym_bare_identifier,
    STATE(501), 4,
      sym__decimal,
      sym__hex,
      sym__octal,
      sym__binary,
  [7354] = 31,
    ACTIONS(11), 1,
      anon_sym_LPAREN,
    ACTIONS(13), 1,
      anon_sym_DQUOTE,
    ACTIONS(15), 1,
      aux_sym__raw_string_token1,
    ACTIONS(17), 1,
      aux_sym__raw_string_token3,
    ACTIONS(29), 1,
      sym__normal_bare_identifier,
    ACTIONS(37), 1,
      anon_sym_null,
    ACTIONS(39), 1,
      sym__digit,
    ACTIONS(43), 1,
      anon_sym_0x,
    ACTIONS(45), 1,
      anon_sym_0o,
    ACTIONS(47), 1,
      anon_sym_0b,
    ACTIONS(256), 1,
      anon_sym_BSLASH,
    ACTIONS(272), 1,
      anon_sym_LBRACE,
    STATE(75), 1,
      aux_sym__node_field_comment_repeat1,
    STATE(98), 1,
      sym__escline,
    STATE(267), 1,
      sym__node_space,
    STATE(270), 1,
      sym_type,
    STATE(299), 1,
      sym__sign,
    STATE(467), 1,
      sym__integer,
    STATE(492), 1,
      sym__bare_identifier,
    STATE(497), 1,
      sym_string,
    STATE(507), 1,
      sym_boolean,
    STATE(512), 1,
      sym__node_field,
    STATE(587), 1,
      sym_identifier,
    ACTIONS(41), 2,
      anon_sym_PLUS,
      anon_sym_DASH,
    ACTIONS(49), 2,
      anon_sym_true,
      anon_sym_false,
    STATE(95), 2,
      sym__ws,
      aux_sym__node_space_repeat1,
    STATE(491), 2,
      sym__escaped_string,
      sym__raw_string,
    STATE(506), 2,
      sym_prop,
      sym_value,
    ACTIONS(258), 3,
      sym_multi_line_comment,
      sym__bom,
      sym__unicode_space,
    STATE(508), 3,
      sym_keyword,
      sym_number,
      sym_bare_identifier,
    STATE(501), 4,
      sym__decimal,
      sym__hex,
      sym__octal,
      sym__binary,
  [7460] = 31,
    ACTIONS(11), 1,
      anon_sym_LPAREN,
    ACTIONS(13), 1,
      anon_sym_DQUOTE,
    ACTIONS(15), 1,
      aux_sym__raw_string_token1,
    ACTIONS(17), 1,
      aux_sym__raw_string_token3,
    ACTIONS(29), 1,
      sym__normal_bare_identifier,
    ACTIONS(37), 1,
      anon_sym_null,
    ACTIONS(39), 1,
      sym__digit,
    ACTIONS(43), 1,
      anon_sym_0x,
    ACTIONS(45), 1,
      anon_sym_0o,
    ACTIONS(47), 1,
      anon_sym_0b,
    ACTIONS(256), 1,
      anon_sym_BSLASH,
    ACTIONS(274), 1,
      anon_sym_LBRACE,
    STATE(74), 1,
      aux_sym__node_field_comment_repeat1,
    STATE(98), 1,
      sym__escline,
    STATE(267), 1,
      sym__node_space,
    STATE(270), 1,
      sym_type,
    STATE(299), 1,
      sym__sign,
    STATE(467), 1,
      sym__integer,
    STATE(492), 1,
      sym__bare_identifier,
    STATE(497), 1,
      sym_string,
    STATE(507), 1,
      sym_boolean,
    STATE(512), 1,
      sym__node_field,
    STATE(587), 1,
      sym_identifier,
    ACTIONS(41), 2,
      anon_sym_PLUS,
      anon_sym_DASH,
    ACTIONS(49), 2,
      anon_sym_true,
      anon_sym_false,
    STATE(95), 2,
      sym__ws,
      aux_sym__node_space_repeat1,
    STATE(491), 2,
      sym__escaped_string,
      sym__raw_string,
    STATE(506), 2,
      sym_prop,
      sym_value,
    ACTIONS(258), 3,
      sym_multi_line_comment,
      sym__bom,
      sym__unicode_space,
    STATE(508), 3,
      sym_keyword,
      sym_number,
      sym_bare_identifier,
    STATE(501), 4,
      sym__decimal,
      sym__hex,
      sym__octal,
      sym__binary,
  [7566] = 31,
    ACTIONS(11), 1,
      anon_sym_LPAREN,
    ACTIONS(13), 1,
      anon_sym_DQUOTE,
    ACTIONS(15), 1,
      aux_sym__raw_string_token1,
    ACTIONS(17), 1,
      aux_sym__raw_string_token3,
    ACTIONS(29), 1,
      sym__normal_bare_identifier,
    ACTIONS(37), 1,
      anon_sym_null,
    ACTIONS(39), 1,
      sym__digit,
    ACTIONS(43), 1,
      anon_sym_0x,
    ACTIONS(45), 1,
      anon_sym_0o,
    ACTIONS(47), 1,
      anon_sym_0b,
    ACTIONS(256), 1,
      anon_sym_BSLASH,
    ACTIONS(276), 1,
      anon_sym_LBRACE,
    STATE(86), 1,
      aux_sym__node_field_comment_repeat1,
    STATE(98), 1,
      sym__escline,
    STATE(267), 1,
      sym__node_space,
    STATE(270), 1,
      sym_type,
    STATE(299), 1,
      sym__sign,
    STATE(467), 1,
      sym__integer,
    STATE(492), 1,
      sym__bare_identifier,
    STATE(497), 1,
      sym_string,
    STATE(507), 1,
      sym_boolean,
    STATE(514), 1,
      sym__node_field,
    STATE(587), 1,
      sym_identifier,
    ACTIONS(41), 2,
      anon_sym_PLUS,
      anon_sym_DASH,
    ACTIONS(49), 2,
      anon_sym_true,
      anon_sym_false,
    STATE(95), 2,
      sym__ws,
      aux_sym__node_space_repeat1,
    STATE(491), 2,
      sym__escaped_string,
      sym__raw_string,
    STATE(506), 2,
      sym_prop,
      sym_value,
    ACTIONS(258), 3,
      sym_multi_line_comment,
      sym__bom,
      sym__unicode_space,
    STATE(508), 3,
      sym_keyword,
      sym_number,
      sym_bare_identifier,
    STATE(501), 4,
      sym__decimal,
      sym__hex,
      sym__octal,
      sym__binary,
  [7672] = 31,
    ACTIONS(11), 1,
      anon_sym_LPAREN,
    ACTIONS(13), 1,
      anon_sym_DQUOTE,
    ACTIONS(15), 1,
      aux_sym__raw_string_token1,
    ACTIONS(17), 1,
      aux_sym__raw_string_token3,
    ACTIONS(29), 1,
      sym__normal_bare_identifier,
    ACTIONS(37), 1,
      anon_sym_null,
    ACTIONS(39), 1,
      sym__digit,
    ACTIONS(43), 1,
      anon_sym_0x,
    ACTIONS(45), 1,
      anon_sym_0o,
    ACTIONS(47), 1,
      anon_sym_0b,
    ACTIONS(256), 1,
      anon_sym_BSLASH,
    ACTIONS(278), 1,
      anon_sym_LBRACE,
    STATE(86), 1,
      aux_sym__node_field_comment_repeat1,
    STATE(98), 1,
      sym__escline,
    STATE(267), 1,
      sym__node_space,
    STATE(270), 1,
      sym_type,
    STATE(299), 1,
      sym__sign,
    STATE(467), 1,
      sym__integer,
    STATE(492), 1,
      sym__bare_identifier,
    STATE(497), 1,
      sym_string,
    STATE(507), 1,
      sym_boolean,
    STATE(514), 1,
      sym__node_field,
    STATE(587), 1,
      sym_identifier,
    ACTIONS(41), 2,
      anon_sym_PLUS,
      anon_sym_DASH,
    ACTIONS(49), 2,
      anon_sym_true,
      anon_sym_false,
    STATE(95), 2,
      sym__ws,
      aux_sym__node_space_repeat1,
    STATE(491), 2,
      sym__escaped_string,
      sym__raw_string,
    STATE(506), 2,
      sym_prop,
      sym_value,
    ACTIONS(258), 3,
      sym_multi_line_comment,
      sym__bom,
      sym__unicode_space,
    STATE(508), 3,
      sym_keyword,
      sym_number,
      sym_bare_identifier,
    STATE(501), 4,
      sym__decimal,
      sym__hex,
      sym__octal,
      sym__binary,
  [7778] = 31,
    ACTIONS(11), 1,
      anon_sym_LPAREN,
    ACTIONS(13), 1,
      anon_sym_DQUOTE,
    ACTIONS(15), 1,
      aux_sym__raw_string_token1,
    ACTIONS(17), 1,
      aux_sym__raw_string_token3,
    ACTIONS(29), 1,
      sym__normal_bare_identifier,
    ACTIONS(37), 1,
      anon_sym_null,
    ACTIONS(39), 1,
      sym__digit,
    ACTIONS(43), 1,
      anon_sym_0x,
    ACTIONS(45), 1,
      anon_sym_0o,
    ACTIONS(47), 1,
      anon_sym_0b,
    ACTIONS(256), 1,
      anon_sym_BSLASH,
    ACTIONS(280), 1,
      anon_sym_LBRACE,
    STATE(86), 1,
      aux_sym__node_field_comment_repeat1,
    STATE(98), 1,
      sym__escline,
    STATE(267), 1,
      sym__node_space,
    STATE(270), 1,
      sym_type,
    STATE(299), 1,
      sym__sign,
    STATE(467), 1,
      sym__integer,
    STATE(492), 1,
      sym__bare_identifier,
    STATE(497), 1,
      sym_string,
    STATE(507), 1,
      sym_boolean,
    STATE(514), 1,
      sym__node_field,
    STATE(587), 1,
      sym_identifier,
    ACTIONS(41), 2,
      anon_sym_PLUS,
      anon_sym_DASH,
    ACTIONS(49), 2,
      anon_sym_true,
      anon_sym_false,
    STATE(95), 2,
      sym__ws,
      aux_sym__node_space_repeat1,
    STATE(491), 2,
      sym__escaped_string,
      sym__raw_string,
    STATE(506), 2,
      sym_prop,
      sym_value,
    ACTIONS(258), 3,
      sym_multi_line_comment,
      sym__bom,
      sym__unicode_space,
    STATE(508), 3,
      sym_keyword,
      sym_number,
      sym_bare_identifier,
    STATE(501), 4,
      sym__decimal,
      sym__hex,
      sym__octal,
      sym__binary,
  [7884] = 30,
    ACTIONS(11), 1,
      anon_sym_LPAREN,
    ACTIONS(13), 1,
      anon_sym_DQUOTE,
    ACTIONS(15), 1,
      aux_sym__raw_string_token1,
    ACTIONS(17), 1,
      aux_sym__raw_string_token3,
    ACTIONS(29), 1,
      sym__normal_bare_identifier,
    ACTIONS(37), 1,
      anon_sym_null,
    ACTIONS(39), 1,
      sym__digit,
    ACTIONS(43), 1,
      anon_sym_0x,
    ACTIONS(45), 1,
      anon_sym_0o,
    ACTIONS(47), 1,
      anon_sym_0b,
    ACTIONS(256), 1,
      anon_sym_BSLASH,
    STATE(86), 1,
      aux_sym__node_field_comment_repeat1,
    STATE(98), 1,
      sym__escline,
    STATE(267), 1,
      sym__node_space,
    STATE(270), 1,
      sym_type,
    STATE(299), 1,
      sym__sign,
    STATE(467), 1,
      sym__integer,
    STATE(492), 1,
      sym__bare_identifier,
    STATE(497), 1,
      sym_string,
    STATE(507), 1,
      sym_boolean,
    STATE(514), 1,
      sym__node_field,
    STATE(587), 1,
      sym_identifier,
    ACTIONS(41), 2,
      anon_sym_PLUS,
      anon_sym_DASH,
    ACTIONS(49), 2,
      anon_sym_true,
      anon_sym_false,
    STATE(95), 2,
      sym__ws,
      aux_sym__node_space_repeat1,
    STATE(491), 2,
      sym__escaped_string,
      sym__raw_string,
    STATE(506), 2,
      sym_prop,
      sym_value,
    ACTIONS(258), 3,
      sym_multi_line_comment,
      sym__bom,
      sym__unicode_space,
    STATE(508), 3,
      sym_keyword,
      sym_number,
      sym_bare_identifier,
    STATE(501), 4,
      sym__decimal,
      sym__hex,
      sym__octal,
      sym__binary,
  [7987] = 30,
    ACTIONS(11), 1,
      anon_sym_LPAREN,
    ACTIONS(13), 1,
      anon_sym_DQUOTE,
    ACTIONS(15), 1,
      aux_sym__raw_string_token1,
    ACTIONS(17), 1,
      aux_sym__raw_string_token3,
    ACTIONS(29), 1,
      sym__normal_bare_identifier,
    ACTIONS(37), 1,
      anon_sym_null,
    ACTIONS(39), 1,
      sym__digit,
    ACTIONS(43), 1,
      anon_sym_0x,
    ACTIONS(45), 1,
      anon_sym_0o,
    ACTIONS(47), 1,
      anon_sym_0b,
    ACTIONS(256), 1,
      anon_sym_BSLASH,
    STATE(76), 1,
      aux_sym__node_field_comment_repeat1,
    STATE(98), 1,
      sym__escline,
    STATE(267), 1,
      sym__node_space,
    STATE(270), 1,
      sym_type,
    STATE(299), 1,
      sym__sign,
    STATE(467), 1,
      sym__integer,
    STATE(492), 1,
      sym__bare_identifier,
    STATE(497), 1,
      sym_string,
    STATE(507), 1,
      sym_boolean,
    STATE(512), 1,
      sym__node_field,
    STATE(587), 1,
      sym_identifier,
    ACTIONS(41), 2,
      anon_sym_PLUS,
      anon_sym_DASH,
    ACTIONS(49), 2,
      anon_sym_true,
      anon_sym_false,
    STATE(95), 2,
      sym__ws,
      aux_sym__node_space_repeat1,
    STATE(491), 2,
      sym__escaped_string,
      sym__raw_string,
    STATE(506), 2,
      sym_prop,
      sym_value,
    ACTIONS(258), 3,
      sym_multi_line_comment,
      sym__bom,
      sym__unicode_space,
    STATE(508), 3,
      sym_keyword,
      sym_number,
      sym_bare_identifier,
    STATE(501), 4,
      sym__decimal,
      sym__hex,
      sym__octal,
      sym__binary,
  [8090] = 2,
    ACTIONS(284), 15,
      sym__normal_bare_identifier,
      anon_sym_null,
      aux_sym__raw_string_token1,
      sym__digit,
      anon_sym_true,
      anon_sym_false,
      anon_sym_expression,
      anon_sym_minimize,
      anon_sym_maximize,
      anon_sym_expr,
      anon_sym_filter,
      anon_sym_if,
      anon_sym_lower,
      anon_sym_upper,
      anon_sym_constraint,
    ACTIONS(282), 24,
      sym_multi_line_comment,
      ts_builtin_sym_end,
      anon_sym_SLASH_DASH,
      anon_sym_LBRACE,
      anon_sym_RBRACE,
      anon_sym_LPAREN,
      anon_sym_DQUOTE,
      aux_sym__raw_string_token3,
      anon_sym_PLUS,
      anon_sym_DASH,
      anon_sym_0x,
      anon_sym_0o,
      anon_sym_0b,
      anon_sym_BSLASH,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
      anon_sym_SLASH_SLASH,
  [8134] = 4,
    STATE(79), 2,
      sym__ws,
      aux_sym__node_space_repeat1,
    ACTIONS(290), 3,
      sym_multi_line_comment,
      sym__bom,
      sym__unicode_space,
    ACTIONS(286), 15,
      sym__normal_bare_identifier,
      anon_sym_null,
      aux_sym__raw_string_token1,
      sym__digit,
      anon_sym_true,
      anon_sym_false,
      anon_sym_expression,
      anon_sym_minimize,
      anon_sym_maximize,
      anon_sym_expr,
      anon_sym_filter,
      anon_sym_if,
      anon_sym_lower,
      anon_sym_upper,
      anon_sym_constraint,
    ACTIONS(288), 19,
      anon_sym_SLASH_DASH,
      anon_sym_LBRACE,
      anon_sym_LPAREN,
      anon_sym_DQUOTE,
      aux_sym__raw_string_token3,
      anon_sym_PLUS,
      anon_sym_DASH,
      anon_sym_0x,
      anon_sym_0o,
      anon_sym_0b,
      anon_sym_BSLASH,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      anon_sym_SLASH_SLASH,
  [8182] = 2,
    ACTIONS(295), 15,
      sym__normal_bare_identifier,
      anon_sym_null,
      aux_sym__raw_string_token1,
      sym__digit,
      anon_sym_true,
      anon_sym_false,
      anon_sym_expression,
      anon_sym_minimize,
      anon_sym_maximize,
      anon_sym_expr,
      anon_sym_filter,
      anon_sym_if,
      anon_sym_lower,
      anon_sym_upper,
      anon_sym_constraint,
    ACTIONS(293), 24,
      sym_multi_line_comment,
      ts_builtin_sym_end,
      anon_sym_SLASH_DASH,
      anon_sym_LBRACE,
      anon_sym_RBRACE,
      anon_sym_LPAREN,
      anon_sym_DQUOTE,
      aux_sym__raw_string_token3,
      anon_sym_PLUS,
      anon_sym_DASH,
      anon_sym_0x,
      anon_sym_0o,
      anon_sym_0b,
      anon_sym_BSLASH,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
      anon_sym_SLASH_SLASH,
  [8226] = 8,
    ACTIONS(301), 1,
      anon_sym_BSLASH,
    STATE(81), 1,
      aux_sym__node_field_comment_repeat1,
    STATE(93), 1,
      sym__escline,
    STATE(102), 1,
      sym__node_space,
    STATE(84), 2,
      sym__ws,
      aux_sym__node_space_repeat1,
    ACTIONS(304), 3,
      sym_multi_line_comment,
      sym__bom,
      sym__unicode_space,
    ACTIONS(297), 6,
      sym__normal_bare_identifier,
      anon_sym_null,
      aux_sym__raw_string_token1,
      sym__digit,
      anon_sym_true,
      anon_sym_false,
    ACTIONS(299), 21,
      sym__eof,
      sym__implicit_terminator,
      anon_sym_SLASH_DASH,
      anon_sym_LBRACE,
      anon_sym_SEMI,
      anon_sym_LPAREN,
      anon_sym_DQUOTE,
      aux_sym__raw_string_token3,
      anon_sym_PLUS,
      anon_sym_DASH,
      anon_sym_0x,
      anon_sym_0o,
      anon_sym_0b,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      anon_sym_SLASH_SLASH,
  [8279] = 5,
    ACTIONS(314), 1,
      anon_sym_SLASH_SLASH,
    STATE(82), 5,
      sym__linespace,
      sym__newline,
      sym__ws,
      sym_single_line_comment,
      aux_sym_document_repeat1,
    ACTIONS(307), 8,
      ts_builtin_sym_end,
      anon_sym_SLASH_DASH,
      anon_sym_RBRACE,
      anon_sym_LPAREN,
      anon_sym_DQUOTE,
      aux_sym__raw_string_token3,
      anon_sym_PLUS,
      anon_sym_DASH,
    ACTIONS(311), 10,
      sym_multi_line_comment,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
    ACTIONS(309), 11,
      sym__normal_bare_identifier,
      aux_sym__raw_string_token1,
      anon_sym_expression,
      anon_sym_minimize,
      anon_sym_maximize,
      anon_sym_expr,
      anon_sym_filter,
      anon_sym_if,
      anon_sym_lower,
      anon_sym_upper,
      anon_sym_constraint,
  [8325] = 6,
    ACTIONS(314), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(317), 1,
      anon_sym_RBRACE,
    STATE(82), 5,
      sym__linespace,
      sym__newline,
      sym__ws,
      sym_single_line_comment,
      aux_sym_document_repeat1,
    ACTIONS(307), 6,
      anon_sym_SLASH_DASH,
      anon_sym_LPAREN,
      anon_sym_DQUOTE,
      aux_sym__raw_string_token3,
      anon_sym_PLUS,
      anon_sym_DASH,
    ACTIONS(311), 10,
      sym_multi_line_comment,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
    ACTIONS(309), 11,
      sym__normal_bare_identifier,
      aux_sym__raw_string_token1,
      anon_sym_expression,
      anon_sym_minimize,
      anon_sym_maximize,
      anon_sym_expr,
      anon_sym_filter,
      anon_sym_if,
      anon_sym_lower,
      anon_sym_upper,
      anon_sym_constraint,
  [8372] = 6,
    ACTIONS(324), 1,
      anon_sym_BSLASH,
    STATE(91), 1,
      sym__escline,
    STATE(90), 2,
      sym__ws,
      aux_sym__node_space_repeat1,
    ACTIONS(327), 3,
      sym_multi_line_comment,
      sym__bom,
      sym__unicode_space,
    ACTIONS(320), 6,
      sym__normal_bare_identifier,
      anon_sym_null,
      aux_sym__raw_string_token1,
      sym__digit,
      anon_sym_true,
      anon_sym_false,
    ACTIONS(322), 21,
      sym__eof,
      sym__implicit_terminator,
      anon_sym_SLASH_DASH,
      anon_sym_LBRACE,
      anon_sym_SEMI,
      anon_sym_LPAREN,
      anon_sym_DQUOTE,
      aux_sym__raw_string_token3,
      anon_sym_PLUS,
      anon_sym_DASH,
      anon_sym_0x,
      anon_sym_0o,
      anon_sym_0b,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      anon_sym_SLASH_SLASH,
  [8419] = 6,
    ACTIONS(314), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(330), 1,
      ts_builtin_sym_end,
    STATE(82), 5,
      sym__linespace,
      sym__newline,
      sym__ws,
      sym_single_line_comment,
      aux_sym_document_repeat1,
    ACTIONS(307), 6,
      anon_sym_SLASH_DASH,
      anon_sym_LPAREN,
      anon_sym_DQUOTE,
      aux_sym__raw_string_token3,
      anon_sym_PLUS,
      anon_sym_DASH,
    ACTIONS(311), 10,
      sym_multi_line_comment,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
    ACTIONS(309), 11,
      sym__normal_bare_identifier,
      aux_sym__raw_string_token1,
      anon_sym_expression,
      anon_sym_minimize,
      anon_sym_maximize,
      anon_sym_expr,
      anon_sym_filter,
      anon_sym_if,
      anon_sym_lower,
      anon_sym_upper,
      anon_sym_constraint,
  [8466] = 8,
    ACTIONS(333), 1,
      anon_sym_BSLASH,
    STATE(86), 1,
      aux_sym__node_field_comment_repeat1,
    STATE(98), 1,
      sym__escline,
    STATE(267), 1,
      sym__node_space,
    STATE(95), 2,
      sym__ws,
      aux_sym__node_space_repeat1,
    ACTIONS(336), 3,
      sym_multi_line_comment,
      sym__bom,
      sym__unicode_space,
    ACTIONS(299), 10,
      anon_sym_SLASH_DASH,
      anon_sym_LBRACE,
      anon_sym_LPAREN,
      anon_sym_DQUOTE,
      aux_sym__raw_string_token3,
      anon_sym_PLUS,
      anon_sym_DASH,
      anon_sym_0x,
      anon_sym_0o,
      anon_sym_0b,
    ACTIONS(297), 15,
      sym__normal_bare_identifier,
      anon_sym_null,
      aux_sym__raw_string_token1,
      sym__digit,
      anon_sym_true,
      anon_sym_false,
      anon_sym_expression,
      anon_sym_minimize,
      anon_sym_maximize,
      anon_sym_expr,
      anon_sym_filter,
      anon_sym_if,
      anon_sym_lower,
      anon_sym_upper,
      anon_sym_constraint,
  [8517] = 6,
    ACTIONS(314), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(339), 1,
      anon_sym_RBRACE,
    STATE(82), 5,
      sym__linespace,
      sym__newline,
      sym__ws,
      sym_single_line_comment,
      aux_sym_document_repeat1,
    ACTIONS(307), 6,
      anon_sym_SLASH_DASH,
      anon_sym_LPAREN,
      anon_sym_DQUOTE,
      aux_sym__raw_string_token3,
      anon_sym_PLUS,
      anon_sym_DASH,
    ACTIONS(311), 10,
      sym_multi_line_comment,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
    ACTIONS(309), 11,
      sym__normal_bare_identifier,
      aux_sym__raw_string_token1,
      anon_sym_expression,
      anon_sym_minimize,
      anon_sym_maximize,
      anon_sym_expr,
      anon_sym_filter,
      anon_sym_if,
      anon_sym_lower,
      anon_sym_upper,
      anon_sym_constraint,
  [8564] = 6,
    ACTIONS(314), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(342), 1,
      anon_sym_RBRACE,
    STATE(82), 5,
      sym__linespace,
      sym__newline,
      sym__ws,
      sym_single_line_comment,
      aux_sym_document_repeat1,
    ACTIONS(307), 6,
      anon_sym_SLASH_DASH,
      anon_sym_LPAREN,
      anon_sym_DQUOTE,
      aux_sym__raw_string_token3,
      anon_sym_PLUS,
      anon_sym_DASH,
    ACTIONS(311), 10,
      sym_multi_line_comment,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
    ACTIONS(309), 11,
      sym__normal_bare_identifier,
      aux_sym__raw_string_token1,
      anon_sym_expression,
      anon_sym_minimize,
      anon_sym_maximize,
      anon_sym_expr,
      anon_sym_filter,
      anon_sym_if,
      anon_sym_lower,
      anon_sym_upper,
      anon_sym_constraint,
  [8611] = 4,
    STATE(90), 2,
      sym__ws,
      aux_sym__node_space_repeat1,
    ACTIONS(349), 3,
      sym_multi_line_comment,
      sym__bom,
      sym__unicode_space,
    ACTIONS(345), 6,
      sym__normal_bare_identifier,
      anon_sym_null,
      aux_sym__raw_string_token1,
      sym__digit,
      anon_sym_true,
      anon_sym_false,
    ACTIONS(347), 22,
      sym__eof,
      sym__implicit_terminator,
      anon_sym_SLASH_DASH,
      anon_sym_LBRACE,
      anon_sym_SEMI,
      anon_sym_LPAREN,
      anon_sym_DQUOTE,
      aux_sym__raw_string_token3,
      anon_sym_PLUS,
      anon_sym_DASH,
      anon_sym_0x,
      anon_sym_0o,
      anon_sym_0b,
      anon_sym_BSLASH,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      anon_sym_SLASH_SLASH,
  [8653] = 4,
    STATE(90), 2,
      sym__ws,
      aux_sym__node_space_repeat1,
    ACTIONS(352), 3,
      sym_multi_line_comment,
      sym__bom,
      sym__unicode_space,
    ACTIONS(286), 6,
      sym__normal_bare_identifier,
      anon_sym_null,
      aux_sym__raw_string_token1,
      sym__digit,
      anon_sym_true,
      anon_sym_false,
    ACTIONS(288), 22,
      sym__eof,
      sym__implicit_terminator,
      anon_sym_SLASH_DASH,
      anon_sym_LBRACE,
      anon_sym_SEMI,
      anon_sym_LPAREN,
      anon_sym_DQUOTE,
      aux_sym__raw_string_token3,
      anon_sym_PLUS,
      anon_sym_DASH,
      anon_sym_0x,
      anon_sym_0o,
      anon_sym_0b,
      anon_sym_BSLASH,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      anon_sym_SLASH_SLASH,
  [8695] = 4,
    STATE(89), 2,
      sym__ws,
      aux_sym__node_space_repeat1,
    ACTIONS(359), 3,
      sym_multi_line_comment,
      sym__bom,
      sym__unicode_space,
    ACTIONS(355), 6,
      sym__normal_bare_identifier,
      anon_sym_null,
      aux_sym__raw_string_token1,
      sym__digit,
      anon_sym_true,
      anon_sym_false,
    ACTIONS(357), 22,
      sym__eof,
      sym__implicit_terminator,
      anon_sym_SLASH_DASH,
      anon_sym_LBRACE,
      anon_sym_SEMI,
      anon_sym_LPAREN,
      anon_sym_DQUOTE,
      aux_sym__raw_string_token3,
      anon_sym_PLUS,
      anon_sym_DASH,
      anon_sym_0x,
      anon_sym_0o,
      anon_sym_0b,
      anon_sym_BSLASH,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      anon_sym_SLASH_SLASH,
  [8737] = 4,
    STATE(90), 2,
      sym__ws,
      aux_sym__node_space_repeat1,
    ACTIONS(362), 3,
      sym_multi_line_comment,
      sym__bom,
      sym__unicode_space,
    ACTIONS(355), 6,
      sym__normal_bare_identifier,
      anon_sym_null,
      aux_sym__raw_string_token1,
      sym__digit,
      anon_sym_true,
      anon_sym_false,
    ACTIONS(357), 22,
      sym__eof,
      sym__implicit_terminator,
      anon_sym_SLASH_DASH,
      anon_sym_LBRACE,
      anon_sym_SEMI,
      anon_sym_LPAREN,
      anon_sym_DQUOTE,
      aux_sym__raw_string_token3,
      anon_sym_PLUS,
      anon_sym_DASH,
      anon_sym_0x,
      anon_sym_0o,
      anon_sym_0b,
      anon_sym_BSLASH,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      anon_sym_SLASH_SLASH,
  [8779] = 4,
    STATE(92), 2,
      sym__ws,
      aux_sym__node_space_repeat1,
    ACTIONS(365), 3,
      sym_multi_line_comment,
      sym__bom,
      sym__unicode_space,
    ACTIONS(320), 6,
      sym__normal_bare_identifier,
      anon_sym_null,
      aux_sym__raw_string_token1,
      sym__digit,
      anon_sym_true,
      anon_sym_false,
    ACTIONS(322), 22,
      sym__eof,
      sym__implicit_terminator,
      anon_sym_SLASH_DASH,
      anon_sym_LBRACE,
      anon_sym_SEMI,
      anon_sym_LPAREN,
      anon_sym_DQUOTE,
      aux_sym__raw_string_token3,
      anon_sym_PLUS,
      anon_sym_DASH,
      anon_sym_0x,
      anon_sym_0o,
      anon_sym_0b,
      anon_sym_BSLASH,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      anon_sym_SLASH_SLASH,
  [8821] = 19,
    ACTIONS(7), 1,
      sym__normal_bare_identifier,
    ACTIONS(11), 1,
      anon_sym_LPAREN,
    ACTIONS(13), 1,
      anon_sym_DQUOTE,
    ACTIONS(15), 1,
      aux_sym__raw_string_token1,
    ACTIONS(17), 1,
      aux_sym__raw_string_token3,
    ACTIONS(256), 1,
      anon_sym_BSLASH,
    ACTIONS(370), 1,
      anon_sym_constraint,
    STATE(96), 1,
      aux_sym__node_field_comment_repeat1,
    STATE(98), 1,
      sym__escline,
    STATE(267), 1,
      sym__node_space,
    STATE(279), 1,
      sym_identifier,
    STATE(463), 1,
      sym_type,
    STATE(493), 1,
      sym__sign,
    ACTIONS(19), 2,
      anon_sym_PLUS,
      anon_sym_DASH,
    STATE(95), 2,
      sym__ws,
      aux_sym__node_space_repeat1,
    STATE(491), 2,
      sym__escaped_string,
      sym__raw_string,
    STATE(510), 2,
      sym__bare_identifier,
      sym_string,
    ACTIONS(258), 3,
      sym_multi_line_comment,
      sym__bom,
      sym__unicode_space,
    ACTIONS(368), 8,
      anon_sym_expression,
      anon_sym_minimize,
      anon_sym_maximize,
      anon_sym_expr,
      anon_sym_filter,
      anon_sym_if,
      anon_sym_lower,
      anon_sym_upper,
  [8892] = 6,
    ACTIONS(372), 1,
      anon_sym_BSLASH,
    STATE(97), 1,
      sym__escline,
    STATE(79), 2,
      sym__ws,
      aux_sym__node_space_repeat1,
    ACTIONS(375), 3,
      sym_multi_line_comment,
      sym__bom,
      sym__unicode_space,
    ACTIONS(322), 10,
      anon_sym_SLASH_DASH,
      anon_sym_LBRACE,
      anon_sym_LPAREN,
      anon_sym_DQUOTE,
      aux_sym__raw_string_token3,
      anon_sym_PLUS,
      anon_sym_DASH,
      anon_sym_0x,
      anon_sym_0o,
      anon_sym_0b,
    ACTIONS(320), 15,
      sym__normal_bare_identifier,
      anon_sym_null,
      aux_sym__raw_string_token1,
      sym__digit,
      anon_sym_true,
      anon_sym_false,
      anon_sym_expression,
      anon_sym_minimize,
      anon_sym_maximize,
      anon_sym_expr,
      anon_sym_filter,
      anon_sym_if,
      anon_sym_lower,
      anon_sym_upper,
      anon_sym_constraint,
  [8937] = 19,
    ACTIONS(7), 1,
      sym__normal_bare_identifier,
    ACTIONS(11), 1,
      anon_sym_LPAREN,
    ACTIONS(13), 1,
      anon_sym_DQUOTE,
    ACTIONS(15), 1,
      aux_sym__raw_string_token1,
    ACTIONS(17), 1,
      aux_sym__raw_string_token3,
    ACTIONS(256), 1,
      anon_sym_BSLASH,
    ACTIONS(380), 1,
      anon_sym_constraint,
    STATE(86), 1,
      aux_sym__node_field_comment_repeat1,
    STATE(98), 1,
      sym__escline,
    STATE(267), 1,
      sym__node_space,
    STATE(283), 1,
      sym_identifier,
    STATE(462), 1,
      sym_type,
    STATE(493), 1,
      sym__sign,
    ACTIONS(19), 2,
      anon_sym_PLUS,
      anon_sym_DASH,
    STATE(95), 2,
      sym__ws,
      aux_sym__node_space_repeat1,
    STATE(491), 2,
      sym__escaped_string,
      sym__raw_string,
    STATE(510), 2,
      sym__bare_identifier,
      sym_string,
    ACTIONS(258), 3,
      sym_multi_line_comment,
      sym__bom,
      sym__unicode_space,
    ACTIONS(378), 8,
      anon_sym_expression,
      anon_sym_minimize,
      anon_sym_maximize,
      anon_sym_expr,
      anon_sym_filter,
      anon_sym_if,
      anon_sym_lower,
      anon_sym_upper,
  [9008] = 4,
    STATE(103), 2,
      sym__ws,
      aux_sym__node_space_repeat1,
    ACTIONS(382), 3,
      sym_multi_line_comment,
      sym__bom,
      sym__unicode_space,
    ACTIONS(357), 11,
      anon_sym_SLASH_DASH,
      anon_sym_LBRACE,
      anon_sym_LPAREN,
      anon_sym_DQUOTE,
      aux_sym__raw_string_token3,
      anon_sym_PLUS,
      anon_sym_DASH,
      anon_sym_0x,
      anon_sym_0o,
      anon_sym_0b,
      anon_sym_BSLASH,
    ACTIONS(355), 15,
      sym__normal_bare_identifier,
      anon_sym_null,
      aux_sym__raw_string_token1,
      sym__digit,
      anon_sym_true,
      anon_sym_false,
      anon_sym_expression,
      anon_sym_minimize,
      anon_sym_maximize,
      anon_sym_expr,
      anon_sym_filter,
      anon_sym_if,
      anon_sym_lower,
      anon_sym_upper,
      anon_sym_constraint,
  [9048] = 4,
    STATE(106), 2,
      sym__ws,
      aux_sym__node_space_repeat1,
    ACTIONS(385), 3,
      sym_multi_line_comment,
      sym__bom,
      sym__unicode_space,
    ACTIONS(322), 11,
      anon_sym_SLASH_DASH,
      anon_sym_LBRACE,
      anon_sym_LPAREN,
      anon_sym_DQUOTE,
      aux_sym__raw_string_token3,
      anon_sym_PLUS,
      anon_sym_DASH,
      anon_sym_0x,
      anon_sym_0o,
      anon_sym_0b,
      anon_sym_BSLASH,
    ACTIONS(320), 15,
      sym__normal_bare_identifier,
      anon_sym_null,
      aux_sym__raw_string_token1,
      sym__digit,
      anon_sym_true,
      anon_sym_false,
      anon_sym_expression,
      anon_sym_minimize,
      anon_sym_maximize,
      anon_sym_expr,
      anon_sym_filter,
      anon_sym_if,
      anon_sym_lower,
      anon_sym_upper,
      anon_sym_constraint,
  [9088] = 2,
    ACTIONS(284), 6,
      sym__normal_bare_identifier,
      anon_sym_null,
      aux_sym__raw_string_token1,
      sym__digit,
      anon_sym_true,
      anon_sym_false,
    ACTIONS(282), 25,
      sym__eof,
      sym_multi_line_comment,
      sym__implicit_terminator,
      anon_sym_SLASH_DASH,
      anon_sym_LBRACE,
      anon_sym_SEMI,
      anon_sym_LPAREN,
      anon_sym_DQUOTE,
      aux_sym__raw_string_token3,
      anon_sym_PLUS,
      anon_sym_DASH,
      anon_sym_0x,
      anon_sym_0o,
      anon_sym_0b,
      anon_sym_BSLASH,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
      anon_sym_SLASH_SLASH,
  [9124] = 2,
    ACTIONS(388), 6,
      sym__normal_bare_identifier,
      anon_sym_null,
      aux_sym__raw_string_token1,
      sym__digit,
      anon_sym_true,
      anon_sym_false,
    ACTIONS(390), 25,
      sym__eof,
      sym_multi_line_comment,
      sym__implicit_terminator,
      anon_sym_SLASH_DASH,
      anon_sym_LBRACE,
      anon_sym_SEMI,
      anon_sym_LPAREN,
      anon_sym_DQUOTE,
      aux_sym__raw_string_token3,
      anon_sym_PLUS,
      anon_sym_DASH,
      anon_sym_0x,
      anon_sym_0o,
      anon_sym_0b,
      anon_sym_BSLASH,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
      anon_sym_SLASH_SLASH,
  [9160] = 22,
    ACTIONS(3), 1,
      sym_multi_line_comment,
    ACTIONS(11), 1,
      anon_sym_LPAREN,
    ACTIONS(13), 1,
      anon_sym_DQUOTE,
    ACTIONS(15), 1,
      aux_sym__raw_string_token1,
    ACTIONS(17), 1,
      aux_sym__raw_string_token3,
    ACTIONS(37), 1,
      anon_sym_null,
    ACTIONS(39), 1,
      sym__digit,
    ACTIONS(43), 1,
      anon_sym_0x,
    ACTIONS(45), 1,
      anon_sym_0o,
    ACTIONS(47), 1,
      anon_sym_0b,
    ACTIONS(392), 1,
      sym__normal_bare_identifier,
    STATE(270), 1,
      sym_type,
    STATE(299), 1,
      sym__sign,
    STATE(467), 1,
      sym__integer,
    STATE(499), 1,
      sym__bare_identifier,
    STATE(507), 1,
      sym_boolean,
    STATE(509), 1,
      sym_value,
    ACTIONS(41), 2,
      anon_sym_PLUS,
      anon_sym_DASH,
    ACTIONS(49), 2,
      anon_sym_true,
      anon_sym_false,
    STATE(491), 2,
      sym__escaped_string,
      sym__raw_string,
    STATE(501), 4,
      sym__decimal,
      sym__hex,
      sym__octal,
      sym__binary,
    STATE(508), 4,
      sym_keyword,
      sym_string,
      sym_number,
      sym_bare_identifier,
  [9236] = 2,
    ACTIONS(394), 6,
      sym__normal_bare_identifier,
      anon_sym_null,
      aux_sym__raw_string_token1,
      sym__digit,
      anon_sym_true,
      anon_sym_false,
    ACTIONS(396), 25,
      sym__eof,
      sym_multi_line_comment,
      sym__implicit_terminator,
      anon_sym_SLASH_DASH,
      anon_sym_LBRACE,
      anon_sym_SEMI,
      anon_sym_LPAREN,
      anon_sym_DQUOTE,
      aux_sym__raw_string_token3,
      anon_sym_PLUS,
      anon_sym_DASH,
      anon_sym_0x,
      anon_sym_0o,
      anon_sym_0b,
      anon_sym_BSLASH,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
      anon_sym_SLASH_SLASH,
  [9272] = 4,
    STATE(79), 2,
      sym__ws,
      aux_sym__node_space_repeat1,
    ACTIONS(398), 3,
      sym_multi_line_comment,
      sym__bom,
      sym__unicode_space,
    ACTIONS(347), 11,
      anon_sym_SLASH_DASH,
      anon_sym_LBRACE,
      anon_sym_LPAREN,
      anon_sym_DQUOTE,
      aux_sym__raw_string_token3,
      anon_sym_PLUS,
      anon_sym_DASH,
      anon_sym_0x,
      anon_sym_0o,
      anon_sym_0b,
      anon_sym_BSLASH,
    ACTIONS(345), 15,
      sym__normal_bare_identifier,
      anon_sym_null,
      aux_sym__raw_string_token1,
      sym__digit,
      anon_sym_true,
      anon_sym_false,
      anon_sym_expression,
      anon_sym_minimize,
      anon_sym_maximize,
      anon_sym_expr,
      anon_sym_filter,
      anon_sym_if,
      anon_sym_lower,
      anon_sym_upper,
      anon_sym_constraint,
  [9312] = 2,
    ACTIONS(295), 6,
      sym__normal_bare_identifier,
      anon_sym_null,
      aux_sym__raw_string_token1,
      sym__digit,
      anon_sym_true,
      anon_sym_false,
    ACTIONS(293), 25,
      sym__eof,
      sym_multi_line_comment,
      sym__implicit_terminator,
      anon_sym_SLASH_DASH,
      anon_sym_LBRACE,
      anon_sym_SEMI,
      anon_sym_LPAREN,
      anon_sym_DQUOTE,
      aux_sym__raw_string_token3,
      anon_sym_PLUS,
      anon_sym_DASH,
      anon_sym_0x,
      anon_sym_0o,
      anon_sym_0b,
      anon_sym_BSLASH,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
      anon_sym_SLASH_SLASH,
  [9348] = 2,
    ACTIONS(401), 6,
      sym__normal_bare_identifier,
      anon_sym_null,
      aux_sym__raw_string_token1,
      sym__digit,
      anon_sym_true,
      anon_sym_false,
    ACTIONS(403), 25,
      sym__eof,
      sym_multi_line_comment,
      sym__implicit_terminator,
      anon_sym_SLASH_DASH,
      anon_sym_LBRACE,
      anon_sym_SEMI,
      anon_sym_LPAREN,
      anon_sym_DQUOTE,
      aux_sym__raw_string_token3,
      anon_sym_PLUS,
      anon_sym_DASH,
      anon_sym_0x,
      anon_sym_0o,
      anon_sym_0b,
      anon_sym_BSLASH,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
      anon_sym_SLASH_SLASH,
  [9384] = 4,
    STATE(79), 2,
      sym__ws,
      aux_sym__node_space_repeat1,
    ACTIONS(405), 3,
      sym_multi_line_comment,
      sym__bom,
      sym__unicode_space,
    ACTIONS(357), 11,
      anon_sym_SLASH_DASH,
      anon_sym_LBRACE,
      anon_sym_LPAREN,
      anon_sym_DQUOTE,
      aux_sym__raw_string_token3,
      anon_sym_PLUS,
      anon_sym_DASH,
      anon_sym_0x,
      anon_sym_0o,
      anon_sym_0b,
      anon_sym_BSLASH,
    ACTIONS(355), 15,
      sym__normal_bare_identifier,
      anon_sym_null,
      aux_sym__raw_string_token1,
      sym__digit,
      anon_sym_true,
      anon_sym_false,
      anon_sym_expression,
      anon_sym_minimize,
      anon_sym_maximize,
      anon_sym_expr,
      anon_sym_filter,
      anon_sym_if,
      anon_sym_lower,
      anon_sym_upper,
      anon_sym_constraint,
  [9424] = 2,
    ACTIONS(410), 11,
      sym__normal_bare_identifier,
      aux_sym__raw_string_token1,
      anon_sym_expression,
      anon_sym_minimize,
      anon_sym_maximize,
      anon_sym_expr,
      anon_sym_filter,
      anon_sym_if,
      anon_sym_lower,
      anon_sym_upper,
      anon_sym_constraint,
    ACTIONS(408), 19,
      sym_multi_line_comment,
      ts_builtin_sym_end,
      anon_sym_SLASH_DASH,
      anon_sym_RBRACE,
      anon_sym_LPAREN,
      anon_sym_DQUOTE,
      aux_sym__raw_string_token3,
      anon_sym_PLUS,
      anon_sym_DASH,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
      anon_sym_SLASH_SLASH,
  [9459] = 2,
    ACTIONS(414), 11,
      sym__normal_bare_identifier,
      aux_sym__raw_string_token1,
      anon_sym_expression,
      anon_sym_minimize,
      anon_sym_maximize,
      anon_sym_expr,
      anon_sym_filter,
      anon_sym_if,
      anon_sym_lower,
      anon_sym_upper,
      anon_sym_constraint,
    ACTIONS(412), 19,
      sym_multi_line_comment,
      ts_builtin_sym_end,
      anon_sym_SLASH_DASH,
      anon_sym_RBRACE,
      anon_sym_LPAREN,
      anon_sym_DQUOTE,
      aux_sym__raw_string_token3,
      anon_sym_PLUS,
      anon_sym_DASH,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
      anon_sym_SLASH_SLASH,
  [9494] = 2,
    ACTIONS(418), 11,
      sym__normal_bare_identifier,
      aux_sym__raw_string_token1,
      anon_sym_expression,
      anon_sym_minimize,
      anon_sym_maximize,
      anon_sym_expr,
      anon_sym_filter,
      anon_sym_if,
      anon_sym_lower,
      anon_sym_upper,
      anon_sym_constraint,
    ACTIONS(416), 19,
      sym_multi_line_comment,
      ts_builtin_sym_end,
      anon_sym_SLASH_DASH,
      anon_sym_RBRACE,
      anon_sym_LPAREN,
      anon_sym_DQUOTE,
      aux_sym__raw_string_token3,
      anon_sym_PLUS,
      anon_sym_DASH,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
      anon_sym_SLASH_SLASH,
  [9529] = 2,
    ACTIONS(422), 11,
      sym__normal_bare_identifier,
      aux_sym__raw_string_token1,
      anon_sym_expression,
      anon_sym_minimize,
      anon_sym_maximize,
      anon_sym_expr,
      anon_sym_filter,
      anon_sym_if,
      anon_sym_lower,
      anon_sym_upper,
      anon_sym_constraint,
    ACTIONS(420), 19,
      sym_multi_line_comment,
      ts_builtin_sym_end,
      anon_sym_SLASH_DASH,
      anon_sym_RBRACE,
      anon_sym_LPAREN,
      anon_sym_DQUOTE,
      aux_sym__raw_string_token3,
      anon_sym_PLUS,
      anon_sym_DASH,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
      anon_sym_SLASH_SLASH,
  [9564] = 2,
    ACTIONS(426), 11,
      sym__normal_bare_identifier,
      aux_sym__raw_string_token1,
      anon_sym_expression,
      anon_sym_minimize,
      anon_sym_maximize,
      anon_sym_expr,
      anon_sym_filter,
      anon_sym_if,
      anon_sym_lower,
      anon_sym_upper,
      anon_sym_constraint,
    ACTIONS(424), 19,
      sym_multi_line_comment,
      ts_builtin_sym_end,
      anon_sym_SLASH_DASH,
      anon_sym_RBRACE,
      anon_sym_LPAREN,
      anon_sym_DQUOTE,
      aux_sym__raw_string_token3,
      anon_sym_PLUS,
      anon_sym_DASH,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
      anon_sym_SLASH_SLASH,
  [9599] = 2,
    ACTIONS(430), 11,
      sym__normal_bare_identifier,
      aux_sym__raw_string_token1,
      anon_sym_expression,
      anon_sym_minimize,
      anon_sym_maximize,
      anon_sym_expr,
      anon_sym_filter,
      anon_sym_if,
      anon_sym_lower,
      anon_sym_upper,
      anon_sym_constraint,
    ACTIONS(428), 19,
      sym_multi_line_comment,
      ts_builtin_sym_end,
      anon_sym_SLASH_DASH,
      anon_sym_RBRACE,
      anon_sym_LPAREN,
      anon_sym_DQUOTE,
      aux_sym__raw_string_token3,
      anon_sym_PLUS,
      anon_sym_DASH,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
      anon_sym_SLASH_SLASH,
  [9634] = 2,
    ACTIONS(434), 11,
      sym__normal_bare_identifier,
      aux_sym__raw_string_token1,
      anon_sym_expression,
      anon_sym_minimize,
      anon_sym_maximize,
      anon_sym_expr,
      anon_sym_filter,
      anon_sym_if,
      anon_sym_lower,
      anon_sym_upper,
      anon_sym_constraint,
    ACTIONS(432), 19,
      sym_multi_line_comment,
      ts_builtin_sym_end,
      anon_sym_SLASH_DASH,
      anon_sym_RBRACE,
      anon_sym_LPAREN,
      anon_sym_DQUOTE,
      aux_sym__raw_string_token3,
      anon_sym_PLUS,
      anon_sym_DASH,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
      anon_sym_SLASH_SLASH,
  [9669] = 2,
    ACTIONS(438), 11,
      sym__normal_bare_identifier,
      aux_sym__raw_string_token1,
      anon_sym_expression,
      anon_sym_minimize,
      anon_sym_maximize,
      anon_sym_expr,
      anon_sym_filter,
      anon_sym_if,
      anon_sym_lower,
      anon_sym_upper,
      anon_sym_constraint,
    ACTIONS(436), 19,
      sym_multi_line_comment,
      ts_builtin_sym_end,
      anon_sym_SLASH_DASH,
      anon_sym_RBRACE,
      anon_sym_LPAREN,
      anon_sym_DQUOTE,
      aux_sym__raw_string_token3,
      anon_sym_PLUS,
      anon_sym_DASH,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
      anon_sym_SLASH_SLASH,
  [9704] = 2,
    ACTIONS(442), 11,
      sym__normal_bare_identifier,
      aux_sym__raw_string_token1,
      anon_sym_expression,
      anon_sym_minimize,
      anon_sym_maximize,
      anon_sym_expr,
      anon_sym_filter,
      anon_sym_if,
      anon_sym_lower,
      anon_sym_upper,
      anon_sym_constraint,
    ACTIONS(440), 19,
      sym_multi_line_comment,
      ts_builtin_sym_end,
      anon_sym_SLASH_DASH,
      anon_sym_RBRACE,
      anon_sym_LPAREN,
      anon_sym_DQUOTE,
      aux_sym__raw_string_token3,
      anon_sym_PLUS,
      anon_sym_DASH,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
      anon_sym_SLASH_SLASH,
  [9739] = 2,
    ACTIONS(446), 11,
      sym__normal_bare_identifier,
      aux_sym__raw_string_token1,
      anon_sym_expression,
      anon_sym_minimize,
      anon_sym_maximize,
      anon_sym_expr,
      anon_sym_filter,
      anon_sym_if,
      anon_sym_lower,
      anon_sym_upper,
      anon_sym_constraint,
    ACTIONS(444), 19,
      sym_multi_line_comment,
      ts_builtin_sym_end,
      anon_sym_SLASH_DASH,
      anon_sym_RBRACE,
      anon_sym_LPAREN,
      anon_sym_DQUOTE,
      aux_sym__raw_string_token3,
      anon_sym_PLUS,
      anon_sym_DASH,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
      anon_sym_SLASH_SLASH,
  [9774] = 2,
    ACTIONS(450), 11,
      sym__normal_bare_identifier,
      aux_sym__raw_string_token1,
      anon_sym_expression,
      anon_sym_minimize,
      anon_sym_maximize,
      anon_sym_expr,
      anon_sym_filter,
      anon_sym_if,
      anon_sym_lower,
      anon_sym_upper,
      anon_sym_constraint,
    ACTIONS(448), 19,
      sym_multi_line_comment,
      ts_builtin_sym_end,
      anon_sym_SLASH_DASH,
      anon_sym_RBRACE,
      anon_sym_LPAREN,
      anon_sym_DQUOTE,
      aux_sym__raw_string_token3,
      anon_sym_PLUS,
      anon_sym_DASH,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
      anon_sym_SLASH_SLASH,
  [9809] = 2,
    ACTIONS(454), 11,
      sym__normal_bare_identifier,
      aux_sym__raw_string_token1,
      anon_sym_expression,
      anon_sym_minimize,
      anon_sym_maximize,
      anon_sym_expr,
      anon_sym_filter,
      anon_sym_if,
      anon_sym_lower,
      anon_sym_upper,
      anon_sym_constraint,
    ACTIONS(452), 19,
      sym_multi_line_comment,
      ts_builtin_sym_end,
      anon_sym_SLASH_DASH,
      anon_sym_RBRACE,
      anon_sym_LPAREN,
      anon_sym_DQUOTE,
      aux_sym__raw_string_token3,
      anon_sym_PLUS,
      anon_sym_DASH,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
      anon_sym_SLASH_SLASH,
  [9844] = 2,
    ACTIONS(458), 11,
      sym__normal_bare_identifier,
      aux_sym__raw_string_token1,
      anon_sym_expression,
      anon_sym_minimize,
      anon_sym_maximize,
      anon_sym_expr,
      anon_sym_filter,
      anon_sym_if,
      anon_sym_lower,
      anon_sym_upper,
      anon_sym_constraint,
    ACTIONS(456), 19,
      sym_multi_line_comment,
      ts_builtin_sym_end,
      anon_sym_SLASH_DASH,
      anon_sym_RBRACE,
      anon_sym_LPAREN,
      anon_sym_DQUOTE,
      aux_sym__raw_string_token3,
      anon_sym_PLUS,
      anon_sym_DASH,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
      anon_sym_SLASH_SLASH,
  [9879] = 2,
    ACTIONS(462), 11,
      sym__normal_bare_identifier,
      aux_sym__raw_string_token1,
      anon_sym_expression,
      anon_sym_minimize,
      anon_sym_maximize,
      anon_sym_expr,
      anon_sym_filter,
      anon_sym_if,
      anon_sym_lower,
      anon_sym_upper,
      anon_sym_constraint,
    ACTIONS(460), 19,
      sym_multi_line_comment,
      ts_builtin_sym_end,
      anon_sym_SLASH_DASH,
      anon_sym_RBRACE,
      anon_sym_LPAREN,
      anon_sym_DQUOTE,
      aux_sym__raw_string_token3,
      anon_sym_PLUS,
      anon_sym_DASH,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
      anon_sym_SLASH_SLASH,
  [9914] = 2,
    ACTIONS(466), 11,
      sym__normal_bare_identifier,
      aux_sym__raw_string_token1,
      anon_sym_expression,
      anon_sym_minimize,
      anon_sym_maximize,
      anon_sym_expr,
      anon_sym_filter,
      anon_sym_if,
      anon_sym_lower,
      anon_sym_upper,
      anon_sym_constraint,
    ACTIONS(464), 19,
      sym_multi_line_comment,
      ts_builtin_sym_end,
      anon_sym_SLASH_DASH,
      anon_sym_RBRACE,
      anon_sym_LPAREN,
      anon_sym_DQUOTE,
      aux_sym__raw_string_token3,
      anon_sym_PLUS,
      anon_sym_DASH,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
      anon_sym_SLASH_SLASH,
  [9949] = 2,
    ACTIONS(470), 11,
      sym__normal_bare_identifier,
      aux_sym__raw_string_token1,
      anon_sym_expression,
      anon_sym_minimize,
      anon_sym_maximize,
      anon_sym_expr,
      anon_sym_filter,
      anon_sym_if,
      anon_sym_lower,
      anon_sym_upper,
      anon_sym_constraint,
    ACTIONS(468), 19,
      sym_multi_line_comment,
      ts_builtin_sym_end,
      anon_sym_SLASH_DASH,
      anon_sym_RBRACE,
      anon_sym_LPAREN,
      anon_sym_DQUOTE,
      aux_sym__raw_string_token3,
      anon_sym_PLUS,
      anon_sym_DASH,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
      anon_sym_SLASH_SLASH,
  [9984] = 2,
    ACTIONS(474), 11,
      sym__normal_bare_identifier,
      aux_sym__raw_string_token1,
      anon_sym_expression,
      anon_sym_minimize,
      anon_sym_maximize,
      anon_sym_expr,
      anon_sym_filter,
      anon_sym_if,
      anon_sym_lower,
      anon_sym_upper,
      anon_sym_constraint,
    ACTIONS(472), 19,
      sym_multi_line_comment,
      ts_builtin_sym_end,
      anon_sym_SLASH_DASH,
      anon_sym_RBRACE,
      anon_sym_LPAREN,
      anon_sym_DQUOTE,
      aux_sym__raw_string_token3,
      anon_sym_PLUS,
      anon_sym_DASH,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
      anon_sym_SLASH_SLASH,
  [10019] = 2,
    ACTIONS(478), 11,
      sym__normal_bare_identifier,
      aux_sym__raw_string_token1,
      anon_sym_expression,
      anon_sym_minimize,
      anon_sym_maximize,
      anon_sym_expr,
      anon_sym_filter,
      anon_sym_if,
      anon_sym_lower,
      anon_sym_upper,
      anon_sym_constraint,
    ACTIONS(476), 19,
      sym_multi_line_comment,
      ts_builtin_sym_end,
      anon_sym_SLASH_DASH,
      anon_sym_RBRACE,
      anon_sym_LPAREN,
      anon_sym_DQUOTE,
      aux_sym__raw_string_token3,
      anon_sym_PLUS,
      anon_sym_DASH,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
      anon_sym_SLASH_SLASH,
  [10054] = 2,
    ACTIONS(482), 11,
      sym__normal_bare_identifier,
      aux_sym__raw_string_token1,
      anon_sym_expression,
      anon_sym_minimize,
      anon_sym_maximize,
      anon_sym_expr,
      anon_sym_filter,
      anon_sym_if,
      anon_sym_lower,
      anon_sym_upper,
      anon_sym_constraint,
    ACTIONS(480), 19,
      sym_multi_line_comment,
      ts_builtin_sym_end,
      anon_sym_SLASH_DASH,
      anon_sym_RBRACE,
      anon_sym_LPAREN,
      anon_sym_DQUOTE,
      aux_sym__raw_string_token3,
      anon_sym_PLUS,
      anon_sym_DASH,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
      anon_sym_SLASH_SLASH,
  [10089] = 2,
    ACTIONS(486), 11,
      sym__normal_bare_identifier,
      aux_sym__raw_string_token1,
      anon_sym_expression,
      anon_sym_minimize,
      anon_sym_maximize,
      anon_sym_expr,
      anon_sym_filter,
      anon_sym_if,
      anon_sym_lower,
      anon_sym_upper,
      anon_sym_constraint,
    ACTIONS(484), 19,
      sym_multi_line_comment,
      ts_builtin_sym_end,
      anon_sym_SLASH_DASH,
      anon_sym_RBRACE,
      anon_sym_LPAREN,
      anon_sym_DQUOTE,
      aux_sym__raw_string_token3,
      anon_sym_PLUS,
      anon_sym_DASH,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
      anon_sym_SLASH_SLASH,
  [10124] = 2,
    ACTIONS(490), 11,
      sym__normal_bare_identifier,
      aux_sym__raw_string_token1,
      anon_sym_expression,
      anon_sym_minimize,
      anon_sym_maximize,
      anon_sym_expr,
      anon_sym_filter,
      anon_sym_if,
      anon_sym_lower,
      anon_sym_upper,
      anon_sym_constraint,
    ACTIONS(488), 19,
      sym_multi_line_comment,
      ts_builtin_sym_end,
      anon_sym_SLASH_DASH,
      anon_sym_RBRACE,
      anon_sym_LPAREN,
      anon_sym_DQUOTE,
      aux_sym__raw_string_token3,
      anon_sym_PLUS,
      anon_sym_DASH,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
      anon_sym_SLASH_SLASH,
  [10159] = 2,
    ACTIONS(494), 11,
      sym__normal_bare_identifier,
      aux_sym__raw_string_token1,
      anon_sym_expression,
      anon_sym_minimize,
      anon_sym_maximize,
      anon_sym_expr,
      anon_sym_filter,
      anon_sym_if,
      anon_sym_lower,
      anon_sym_upper,
      anon_sym_constraint,
    ACTIONS(492), 19,
      sym_multi_line_comment,
      ts_builtin_sym_end,
      anon_sym_SLASH_DASH,
      anon_sym_RBRACE,
      anon_sym_LPAREN,
      anon_sym_DQUOTE,
      aux_sym__raw_string_token3,
      anon_sym_PLUS,
      anon_sym_DASH,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
      anon_sym_SLASH_SLASH,
  [10194] = 2,
    ACTIONS(498), 11,
      sym__normal_bare_identifier,
      aux_sym__raw_string_token1,
      anon_sym_expression,
      anon_sym_minimize,
      anon_sym_maximize,
      anon_sym_expr,
      anon_sym_filter,
      anon_sym_if,
      anon_sym_lower,
      anon_sym_upper,
      anon_sym_constraint,
    ACTIONS(496), 19,
      sym_multi_line_comment,
      ts_builtin_sym_end,
      anon_sym_SLASH_DASH,
      anon_sym_RBRACE,
      anon_sym_LPAREN,
      anon_sym_DQUOTE,
      aux_sym__raw_string_token3,
      anon_sym_PLUS,
      anon_sym_DASH,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
      anon_sym_SLASH_SLASH,
  [10229] = 2,
    ACTIONS(502), 11,
      sym__normal_bare_identifier,
      aux_sym__raw_string_token1,
      anon_sym_expression,
      anon_sym_minimize,
      anon_sym_maximize,
      anon_sym_expr,
      anon_sym_filter,
      anon_sym_if,
      anon_sym_lower,
      anon_sym_upper,
      anon_sym_constraint,
    ACTIONS(500), 19,
      sym_multi_line_comment,
      ts_builtin_sym_end,
      anon_sym_SLASH_DASH,
      anon_sym_RBRACE,
      anon_sym_LPAREN,
      anon_sym_DQUOTE,
      aux_sym__raw_string_token3,
      anon_sym_PLUS,
      anon_sym_DASH,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
      anon_sym_SLASH_SLASH,
  [10264] = 2,
    ACTIONS(506), 11,
      sym__normal_bare_identifier,
      aux_sym__raw_string_token1,
      anon_sym_expression,
      anon_sym_minimize,
      anon_sym_maximize,
      anon_sym_expr,
      anon_sym_filter,
      anon_sym_if,
      anon_sym_lower,
      anon_sym_upper,
      anon_sym_constraint,
    ACTIONS(504), 19,
      sym_multi_line_comment,
      ts_builtin_sym_end,
      anon_sym_SLASH_DASH,
      anon_sym_RBRACE,
      anon_sym_LPAREN,
      anon_sym_DQUOTE,
      aux_sym__raw_string_token3,
      anon_sym_PLUS,
      anon_sym_DASH,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
      anon_sym_SLASH_SLASH,
  [10299] = 2,
    ACTIONS(510), 11,
      sym__normal_bare_identifier,
      aux_sym__raw_string_token1,
      anon_sym_expression,
      anon_sym_minimize,
      anon_sym_maximize,
      anon_sym_expr,
      anon_sym_filter,
      anon_sym_if,
      anon_sym_lower,
      anon_sym_upper,
      anon_sym_constraint,
    ACTIONS(508), 19,
      sym_multi_line_comment,
      ts_builtin_sym_end,
      anon_sym_SLASH_DASH,
      anon_sym_RBRACE,
      anon_sym_LPAREN,
      anon_sym_DQUOTE,
      aux_sym__raw_string_token3,
      anon_sym_PLUS,
      anon_sym_DASH,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
      anon_sym_SLASH_SLASH,
  [10334] = 2,
    ACTIONS(514), 11,
      sym__normal_bare_identifier,
      aux_sym__raw_string_token1,
      anon_sym_expression,
      anon_sym_minimize,
      anon_sym_maximize,
      anon_sym_expr,
      anon_sym_filter,
      anon_sym_if,
      anon_sym_lower,
      anon_sym_upper,
      anon_sym_constraint,
    ACTIONS(512), 19,
      sym_multi_line_comment,
      ts_builtin_sym_end,
      anon_sym_SLASH_DASH,
      anon_sym_RBRACE,
      anon_sym_LPAREN,
      anon_sym_DQUOTE,
      aux_sym__raw_string_token3,
      anon_sym_PLUS,
      anon_sym_DASH,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
      anon_sym_SLASH_SLASH,
  [10369] = 2,
    ACTIONS(518), 11,
      sym__normal_bare_identifier,
      aux_sym__raw_string_token1,
      anon_sym_expression,
      anon_sym_minimize,
      anon_sym_maximize,
      anon_sym_expr,
      anon_sym_filter,
      anon_sym_if,
      anon_sym_lower,
      anon_sym_upper,
      anon_sym_constraint,
    ACTIONS(516), 19,
      sym_multi_line_comment,
      ts_builtin_sym_end,
      anon_sym_SLASH_DASH,
      anon_sym_RBRACE,
      anon_sym_LPAREN,
      anon_sym_DQUOTE,
      aux_sym__raw_string_token3,
      anon_sym_PLUS,
      anon_sym_DASH,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
      anon_sym_SLASH_SLASH,
  [10404] = 2,
    ACTIONS(520), 11,
      sym__normal_bare_identifier,
      aux_sym__raw_string_token1,
      anon_sym_expression,
      anon_sym_minimize,
      anon_sym_maximize,
      anon_sym_expr,
      anon_sym_filter,
      anon_sym_if,
      anon_sym_lower,
      anon_sym_upper,
      anon_sym_constraint,
    ACTIONS(127), 19,
      sym_multi_line_comment,
      ts_builtin_sym_end,
      anon_sym_SLASH_DASH,
      anon_sym_RBRACE,
      anon_sym_LPAREN,
      anon_sym_DQUOTE,
      aux_sym__raw_string_token3,
      anon_sym_PLUS,
      anon_sym_DASH,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
      anon_sym_SLASH_SLASH,
  [10439] = 2,
    ACTIONS(524), 11,
      sym__normal_bare_identifier,
      aux_sym__raw_string_token1,
      anon_sym_expression,
      anon_sym_minimize,
      anon_sym_maximize,
      anon_sym_expr,
      anon_sym_filter,
      anon_sym_if,
      anon_sym_lower,
      anon_sym_upper,
      anon_sym_constraint,
    ACTIONS(522), 19,
      sym_multi_line_comment,
      ts_builtin_sym_end,
      anon_sym_SLASH_DASH,
      anon_sym_RBRACE,
      anon_sym_LPAREN,
      anon_sym_DQUOTE,
      aux_sym__raw_string_token3,
      anon_sym_PLUS,
      anon_sym_DASH,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
      anon_sym_SLASH_SLASH,
  [10474] = 2,
    ACTIONS(528), 11,
      sym__normal_bare_identifier,
      aux_sym__raw_string_token1,
      anon_sym_expression,
      anon_sym_minimize,
      anon_sym_maximize,
      anon_sym_expr,
      anon_sym_filter,
      anon_sym_if,
      anon_sym_lower,
      anon_sym_upper,
      anon_sym_constraint,
    ACTIONS(526), 19,
      sym_multi_line_comment,
      ts_builtin_sym_end,
      anon_sym_SLASH_DASH,
      anon_sym_RBRACE,
      anon_sym_LPAREN,
      anon_sym_DQUOTE,
      aux_sym__raw_string_token3,
      anon_sym_PLUS,
      anon_sym_DASH,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
      anon_sym_SLASH_SLASH,
  [10509] = 2,
    ACTIONS(532), 11,
      sym__normal_bare_identifier,
      aux_sym__raw_string_token1,
      anon_sym_expression,
      anon_sym_minimize,
      anon_sym_maximize,
      anon_sym_expr,
      anon_sym_filter,
      anon_sym_if,
      anon_sym_lower,
      anon_sym_upper,
      anon_sym_constraint,
    ACTIONS(530), 19,
      sym_multi_line_comment,
      ts_builtin_sym_end,
      anon_sym_SLASH_DASH,
      anon_sym_RBRACE,
      anon_sym_LPAREN,
      anon_sym_DQUOTE,
      aux_sym__raw_string_token3,
      anon_sym_PLUS,
      anon_sym_DASH,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
      anon_sym_SLASH_SLASH,
  [10544] = 2,
    ACTIONS(536), 11,
      sym__normal_bare_identifier,
      aux_sym__raw_string_token1,
      anon_sym_expression,
      anon_sym_minimize,
      anon_sym_maximize,
      anon_sym_expr,
      anon_sym_filter,
      anon_sym_if,
      anon_sym_lower,
      anon_sym_upper,
      anon_sym_constraint,
    ACTIONS(534), 19,
      sym_multi_line_comment,
      ts_builtin_sym_end,
      anon_sym_SLASH_DASH,
      anon_sym_RBRACE,
      anon_sym_LPAREN,
      anon_sym_DQUOTE,
      aux_sym__raw_string_token3,
      anon_sym_PLUS,
      anon_sym_DASH,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
      anon_sym_SLASH_SLASH,
  [10579] = 2,
    ACTIONS(540), 11,
      sym__normal_bare_identifier,
      aux_sym__raw_string_token1,
      anon_sym_expression,
      anon_sym_minimize,
      anon_sym_maximize,
      anon_sym_expr,
      anon_sym_filter,
      anon_sym_if,
      anon_sym_lower,
      anon_sym_upper,
      anon_sym_constraint,
    ACTIONS(538), 19,
      sym_multi_line_comment,
      ts_builtin_sym_end,
      anon_sym_SLASH_DASH,
      anon_sym_RBRACE,
      anon_sym_LPAREN,
      anon_sym_DQUOTE,
      aux_sym__raw_string_token3,
      anon_sym_PLUS,
      anon_sym_DASH,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
      anon_sym_SLASH_SLASH,
  [10614] = 2,
    ACTIONS(544), 11,
      sym__normal_bare_identifier,
      aux_sym__raw_string_token1,
      anon_sym_expression,
      anon_sym_minimize,
      anon_sym_maximize,
      anon_sym_expr,
      anon_sym_filter,
      anon_sym_if,
      anon_sym_lower,
      anon_sym_upper,
      anon_sym_constraint,
    ACTIONS(542), 19,
      sym_multi_line_comment,
      ts_builtin_sym_end,
      anon_sym_SLASH_DASH,
      anon_sym_RBRACE,
      anon_sym_LPAREN,
      anon_sym_DQUOTE,
      aux_sym__raw_string_token3,
      anon_sym_PLUS,
      anon_sym_DASH,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
      anon_sym_SLASH_SLASH,
  [10649] = 2,
    ACTIONS(548), 11,
      sym__normal_bare_identifier,
      aux_sym__raw_string_token1,
      anon_sym_expression,
      anon_sym_minimize,
      anon_sym_maximize,
      anon_sym_expr,
      anon_sym_filter,
      anon_sym_if,
      anon_sym_lower,
      anon_sym_upper,
      anon_sym_constraint,
    ACTIONS(546), 19,
      sym_multi_line_comment,
      ts_builtin_sym_end,
      anon_sym_SLASH_DASH,
      anon_sym_RBRACE,
      anon_sym_LPAREN,
      anon_sym_DQUOTE,
      aux_sym__raw_string_token3,
      anon_sym_PLUS,
      anon_sym_DASH,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
      anon_sym_SLASH_SLASH,
  [10684] = 2,
    ACTIONS(552), 11,
      sym__normal_bare_identifier,
      aux_sym__raw_string_token1,
      anon_sym_expression,
      anon_sym_minimize,
      anon_sym_maximize,
      anon_sym_expr,
      anon_sym_filter,
      anon_sym_if,
      anon_sym_lower,
      anon_sym_upper,
      anon_sym_constraint,
    ACTIONS(550), 19,
      sym_multi_line_comment,
      ts_builtin_sym_end,
      anon_sym_SLASH_DASH,
      anon_sym_RBRACE,
      anon_sym_LPAREN,
      anon_sym_DQUOTE,
      aux_sym__raw_string_token3,
      anon_sym_PLUS,
      anon_sym_DASH,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
      anon_sym_SLASH_SLASH,
  [10719] = 2,
    ACTIONS(556), 11,
      sym__normal_bare_identifier,
      aux_sym__raw_string_token1,
      anon_sym_expression,
      anon_sym_minimize,
      anon_sym_maximize,
      anon_sym_expr,
      anon_sym_filter,
      anon_sym_if,
      anon_sym_lower,
      anon_sym_upper,
      anon_sym_constraint,
    ACTIONS(554), 19,
      sym_multi_line_comment,
      ts_builtin_sym_end,
      anon_sym_SLASH_DASH,
      anon_sym_RBRACE,
      anon_sym_LPAREN,
      anon_sym_DQUOTE,
      aux_sym__raw_string_token3,
      anon_sym_PLUS,
      anon_sym_DASH,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
      anon_sym_SLASH_SLASH,
  [10754] = 2,
    ACTIONS(560), 11,
      sym__normal_bare_identifier,
      aux_sym__raw_string_token1,
      anon_sym_expression,
      anon_sym_minimize,
      anon_sym_maximize,
      anon_sym_expr,
      anon_sym_filter,
      anon_sym_if,
      anon_sym_lower,
      anon_sym_upper,
      anon_sym_constraint,
    ACTIONS(558), 19,
      sym_multi_line_comment,
      ts_builtin_sym_end,
      anon_sym_SLASH_DASH,
      anon_sym_RBRACE,
      anon_sym_LPAREN,
      anon_sym_DQUOTE,
      aux_sym__raw_string_token3,
      anon_sym_PLUS,
      anon_sym_DASH,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
      anon_sym_SLASH_SLASH,
  [10789] = 2,
    ACTIONS(564), 11,
      sym__normal_bare_identifier,
      aux_sym__raw_string_token1,
      anon_sym_expression,
      anon_sym_minimize,
      anon_sym_maximize,
      anon_sym_expr,
      anon_sym_filter,
      anon_sym_if,
      anon_sym_lower,
      anon_sym_upper,
      anon_sym_constraint,
    ACTIONS(562), 19,
      sym_multi_line_comment,
      ts_builtin_sym_end,
      anon_sym_SLASH_DASH,
      anon_sym_RBRACE,
      anon_sym_LPAREN,
      anon_sym_DQUOTE,
      aux_sym__raw_string_token3,
      anon_sym_PLUS,
      anon_sym_DASH,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
      anon_sym_SLASH_SLASH,
  [10824] = 2,
    ACTIONS(568), 11,
      sym__normal_bare_identifier,
      aux_sym__raw_string_token1,
      anon_sym_expression,
      anon_sym_minimize,
      anon_sym_maximize,
      anon_sym_expr,
      anon_sym_filter,
      anon_sym_if,
      anon_sym_lower,
      anon_sym_upper,
      anon_sym_constraint,
    ACTIONS(566), 19,
      sym_multi_line_comment,
      ts_builtin_sym_end,
      anon_sym_SLASH_DASH,
      anon_sym_RBRACE,
      anon_sym_LPAREN,
      anon_sym_DQUOTE,
      aux_sym__raw_string_token3,
      anon_sym_PLUS,
      anon_sym_DASH,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
      anon_sym_SLASH_SLASH,
  [10859] = 2,
    ACTIONS(572), 11,
      sym__normal_bare_identifier,
      aux_sym__raw_string_token1,
      anon_sym_expression,
      anon_sym_minimize,
      anon_sym_maximize,
      anon_sym_expr,
      anon_sym_filter,
      anon_sym_if,
      anon_sym_lower,
      anon_sym_upper,
      anon_sym_constraint,
    ACTIONS(570), 19,
      sym_multi_line_comment,
      ts_builtin_sym_end,
      anon_sym_SLASH_DASH,
      anon_sym_RBRACE,
      anon_sym_LPAREN,
      anon_sym_DQUOTE,
      aux_sym__raw_string_token3,
      anon_sym_PLUS,
      anon_sym_DASH,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
      anon_sym_SLASH_SLASH,
  [10894] = 2,
    ACTIONS(576), 11,
      sym__normal_bare_identifier,
      aux_sym__raw_string_token1,
      anon_sym_expression,
      anon_sym_minimize,
      anon_sym_maximize,
      anon_sym_expr,
      anon_sym_filter,
      anon_sym_if,
      anon_sym_lower,
      anon_sym_upper,
      anon_sym_constraint,
    ACTIONS(574), 19,
      sym_multi_line_comment,
      ts_builtin_sym_end,
      anon_sym_SLASH_DASH,
      anon_sym_RBRACE,
      anon_sym_LPAREN,
      anon_sym_DQUOTE,
      aux_sym__raw_string_token3,
      anon_sym_PLUS,
      anon_sym_DASH,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
      anon_sym_SLASH_SLASH,
  [10929] = 2,
    ACTIONS(580), 11,
      sym__normal_bare_identifier,
      aux_sym__raw_string_token1,
      anon_sym_expression,
      anon_sym_minimize,
      anon_sym_maximize,
      anon_sym_expr,
      anon_sym_filter,
      anon_sym_if,
      anon_sym_lower,
      anon_sym_upper,
      anon_sym_constraint,
    ACTIONS(578), 19,
      sym_multi_line_comment,
      ts_builtin_sym_end,
      anon_sym_SLASH_DASH,
      anon_sym_RBRACE,
      anon_sym_LPAREN,
      anon_sym_DQUOTE,
      aux_sym__raw_string_token3,
      anon_sym_PLUS,
      anon_sym_DASH,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
      anon_sym_SLASH_SLASH,
  [10964] = 2,
    ACTIONS(584), 11,
      sym__normal_bare_identifier,
      aux_sym__raw_string_token1,
      anon_sym_expression,
      anon_sym_minimize,
      anon_sym_maximize,
      anon_sym_expr,
      anon_sym_filter,
      anon_sym_if,
      anon_sym_lower,
      anon_sym_upper,
      anon_sym_constraint,
    ACTIONS(582), 19,
      sym_multi_line_comment,
      ts_builtin_sym_end,
      anon_sym_SLASH_DASH,
      anon_sym_RBRACE,
      anon_sym_LPAREN,
      anon_sym_DQUOTE,
      aux_sym__raw_string_token3,
      anon_sym_PLUS,
      anon_sym_DASH,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
      anon_sym_SLASH_SLASH,
  [10999] = 2,
    ACTIONS(588), 11,
      sym__normal_bare_identifier,
      aux_sym__raw_string_token1,
      anon_sym_expression,
      anon_sym_minimize,
      anon_sym_maximize,
      anon_sym_expr,
      anon_sym_filter,
      anon_sym_if,
      anon_sym_lower,
      anon_sym_upper,
      anon_sym_constraint,
    ACTIONS(586), 19,
      sym_multi_line_comment,
      ts_builtin_sym_end,
      anon_sym_SLASH_DASH,
      anon_sym_RBRACE,
      anon_sym_LPAREN,
      anon_sym_DQUOTE,
      aux_sym__raw_string_token3,
      anon_sym_PLUS,
      anon_sym_DASH,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
      anon_sym_SLASH_SLASH,
  [11034] = 2,
    ACTIONS(592), 11,
      sym__normal_bare_identifier,
      aux_sym__raw_string_token1,
      anon_sym_expression,
      anon_sym_minimize,
      anon_sym_maximize,
      anon_sym_expr,
      anon_sym_filter,
      anon_sym_if,
      anon_sym_lower,
      anon_sym_upper,
      anon_sym_constraint,
    ACTIONS(590), 19,
      sym_multi_line_comment,
      ts_builtin_sym_end,
      anon_sym_SLASH_DASH,
      anon_sym_RBRACE,
      anon_sym_LPAREN,
      anon_sym_DQUOTE,
      aux_sym__raw_string_token3,
      anon_sym_PLUS,
      anon_sym_DASH,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
      anon_sym_SLASH_SLASH,
  [11069] = 2,
    ACTIONS(596), 11,
      sym__normal_bare_identifier,
      aux_sym__raw_string_token1,
      anon_sym_expression,
      anon_sym_minimize,
      anon_sym_maximize,
      anon_sym_expr,
      anon_sym_filter,
      anon_sym_if,
      anon_sym_lower,
      anon_sym_upper,
      anon_sym_constraint,
    ACTIONS(594), 19,
      sym_multi_line_comment,
      ts_builtin_sym_end,
      anon_sym_SLASH_DASH,
      anon_sym_RBRACE,
      anon_sym_LPAREN,
      anon_sym_DQUOTE,
      aux_sym__raw_string_token3,
      anon_sym_PLUS,
      anon_sym_DASH,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
      anon_sym_SLASH_SLASH,
  [11104] = 2,
    ACTIONS(600), 11,
      sym__normal_bare_identifier,
      aux_sym__raw_string_token1,
      anon_sym_expression,
      anon_sym_minimize,
      anon_sym_maximize,
      anon_sym_expr,
      anon_sym_filter,
      anon_sym_if,
      anon_sym_lower,
      anon_sym_upper,
      anon_sym_constraint,
    ACTIONS(598), 19,
      sym_multi_line_comment,
      ts_builtin_sym_end,
      anon_sym_SLASH_DASH,
      anon_sym_RBRACE,
      anon_sym_LPAREN,
      anon_sym_DQUOTE,
      aux_sym__raw_string_token3,
      anon_sym_PLUS,
      anon_sym_DASH,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
      anon_sym_SLASH_SLASH,
  [11139] = 2,
    ACTIONS(604), 11,
      sym__normal_bare_identifier,
      aux_sym__raw_string_token1,
      anon_sym_expression,
      anon_sym_minimize,
      anon_sym_maximize,
      anon_sym_expr,
      anon_sym_filter,
      anon_sym_if,
      anon_sym_lower,
      anon_sym_upper,
      anon_sym_constraint,
    ACTIONS(602), 19,
      sym_multi_line_comment,
      ts_builtin_sym_end,
      anon_sym_SLASH_DASH,
      anon_sym_RBRACE,
      anon_sym_LPAREN,
      anon_sym_DQUOTE,
      aux_sym__raw_string_token3,
      anon_sym_PLUS,
      anon_sym_DASH,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
      anon_sym_SLASH_SLASH,
  [11174] = 2,
    ACTIONS(608), 11,
      sym__normal_bare_identifier,
      aux_sym__raw_string_token1,
      anon_sym_expression,
      anon_sym_minimize,
      anon_sym_maximize,
      anon_sym_expr,
      anon_sym_filter,
      anon_sym_if,
      anon_sym_lower,
      anon_sym_upper,
      anon_sym_constraint,
    ACTIONS(606), 19,
      sym_multi_line_comment,
      ts_builtin_sym_end,
      anon_sym_SLASH_DASH,
      anon_sym_RBRACE,
      anon_sym_LPAREN,
      anon_sym_DQUOTE,
      aux_sym__raw_string_token3,
      anon_sym_PLUS,
      anon_sym_DASH,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
      anon_sym_SLASH_SLASH,
  [11209] = 2,
    ACTIONS(612), 11,
      sym__normal_bare_identifier,
      aux_sym__raw_string_token1,
      anon_sym_expression,
      anon_sym_minimize,
      anon_sym_maximize,
      anon_sym_expr,
      anon_sym_filter,
      anon_sym_if,
      anon_sym_lower,
      anon_sym_upper,
      anon_sym_constraint,
    ACTIONS(610), 19,
      sym_multi_line_comment,
      ts_builtin_sym_end,
      anon_sym_SLASH_DASH,
      anon_sym_RBRACE,
      anon_sym_LPAREN,
      anon_sym_DQUOTE,
      aux_sym__raw_string_token3,
      anon_sym_PLUS,
      anon_sym_DASH,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
      anon_sym_SLASH_SLASH,
  [11244] = 2,
    ACTIONS(616), 11,
      sym__normal_bare_identifier,
      aux_sym__raw_string_token1,
      anon_sym_expression,
      anon_sym_minimize,
      anon_sym_maximize,
      anon_sym_expr,
      anon_sym_filter,
      anon_sym_if,
      anon_sym_lower,
      anon_sym_upper,
      anon_sym_constraint,
    ACTIONS(614), 19,
      sym_multi_line_comment,
      ts_builtin_sym_end,
      anon_sym_SLASH_DASH,
      anon_sym_RBRACE,
      anon_sym_LPAREN,
      anon_sym_DQUOTE,
      aux_sym__raw_string_token3,
      anon_sym_PLUS,
      anon_sym_DASH,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
      anon_sym_SLASH_SLASH,
  [11279] = 2,
    ACTIONS(620), 11,
      sym__normal_bare_identifier,
      aux_sym__raw_string_token1,
      anon_sym_expression,
      anon_sym_minimize,
      anon_sym_maximize,
      anon_sym_expr,
      anon_sym_filter,
      anon_sym_if,
      anon_sym_lower,
      anon_sym_upper,
      anon_sym_constraint,
    ACTIONS(618), 19,
      sym_multi_line_comment,
      ts_builtin_sym_end,
      anon_sym_SLASH_DASH,
      anon_sym_RBRACE,
      anon_sym_LPAREN,
      anon_sym_DQUOTE,
      aux_sym__raw_string_token3,
      anon_sym_PLUS,
      anon_sym_DASH,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
      anon_sym_SLASH_SLASH,
  [11314] = 2,
    ACTIONS(624), 11,
      sym__normal_bare_identifier,
      aux_sym__raw_string_token1,
      anon_sym_expression,
      anon_sym_minimize,
      anon_sym_maximize,
      anon_sym_expr,
      anon_sym_filter,
      anon_sym_if,
      anon_sym_lower,
      anon_sym_upper,
      anon_sym_constraint,
    ACTIONS(622), 19,
      sym_multi_line_comment,
      ts_builtin_sym_end,
      anon_sym_SLASH_DASH,
      anon_sym_RBRACE,
      anon_sym_LPAREN,
      anon_sym_DQUOTE,
      aux_sym__raw_string_token3,
      anon_sym_PLUS,
      anon_sym_DASH,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
      anon_sym_SLASH_SLASH,
  [11349] = 2,
    ACTIONS(628), 11,
      sym__normal_bare_identifier,
      aux_sym__raw_string_token1,
      anon_sym_expression,
      anon_sym_minimize,
      anon_sym_maximize,
      anon_sym_expr,
      anon_sym_filter,
      anon_sym_if,
      anon_sym_lower,
      anon_sym_upper,
      anon_sym_constraint,
    ACTIONS(626), 19,
      sym_multi_line_comment,
      ts_builtin_sym_end,
      anon_sym_SLASH_DASH,
      anon_sym_RBRACE,
      anon_sym_LPAREN,
      anon_sym_DQUOTE,
      aux_sym__raw_string_token3,
      anon_sym_PLUS,
      anon_sym_DASH,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
      anon_sym_SLASH_SLASH,
  [11384] = 2,
    ACTIONS(632), 11,
      sym__normal_bare_identifier,
      aux_sym__raw_string_token1,
      anon_sym_expression,
      anon_sym_minimize,
      anon_sym_maximize,
      anon_sym_expr,
      anon_sym_filter,
      anon_sym_if,
      anon_sym_lower,
      anon_sym_upper,
      anon_sym_constraint,
    ACTIONS(630), 19,
      sym_multi_line_comment,
      ts_builtin_sym_end,
      anon_sym_SLASH_DASH,
      anon_sym_RBRACE,
      anon_sym_LPAREN,
      anon_sym_DQUOTE,
      aux_sym__raw_string_token3,
      anon_sym_PLUS,
      anon_sym_DASH,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
      anon_sym_SLASH_SLASH,
  [11419] = 2,
    ACTIONS(636), 11,
      sym__normal_bare_identifier,
      aux_sym__raw_string_token1,
      anon_sym_expression,
      anon_sym_minimize,
      anon_sym_maximize,
      anon_sym_expr,
      anon_sym_filter,
      anon_sym_if,
      anon_sym_lower,
      anon_sym_upper,
      anon_sym_constraint,
    ACTIONS(634), 19,
      sym_multi_line_comment,
      ts_builtin_sym_end,
      anon_sym_SLASH_DASH,
      anon_sym_RBRACE,
      anon_sym_LPAREN,
      anon_sym_DQUOTE,
      aux_sym__raw_string_token3,
      anon_sym_PLUS,
      anon_sym_DASH,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
      anon_sym_SLASH_SLASH,
  [11454] = 2,
    ACTIONS(640), 11,
      sym__normal_bare_identifier,
      aux_sym__raw_string_token1,
      anon_sym_expression,
      anon_sym_minimize,
      anon_sym_maximize,
      anon_sym_expr,
      anon_sym_filter,
      anon_sym_if,
      anon_sym_lower,
      anon_sym_upper,
      anon_sym_constraint,
    ACTIONS(638), 19,
      sym_multi_line_comment,
      ts_builtin_sym_end,
      anon_sym_SLASH_DASH,
      anon_sym_RBRACE,
      anon_sym_LPAREN,
      anon_sym_DQUOTE,
      aux_sym__raw_string_token3,
      anon_sym_PLUS,
      anon_sym_DASH,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
      anon_sym_SLASH_SLASH,
  [11489] = 2,
    ACTIONS(644), 11,
      sym__normal_bare_identifier,
      aux_sym__raw_string_token1,
      anon_sym_expression,
      anon_sym_minimize,
      anon_sym_maximize,
      anon_sym_expr,
      anon_sym_filter,
      anon_sym_if,
      anon_sym_lower,
      anon_sym_upper,
      anon_sym_constraint,
    ACTIONS(642), 19,
      sym_multi_line_comment,
      ts_builtin_sym_end,
      anon_sym_SLASH_DASH,
      anon_sym_RBRACE,
      anon_sym_LPAREN,
      anon_sym_DQUOTE,
      aux_sym__raw_string_token3,
      anon_sym_PLUS,
      anon_sym_DASH,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
      anon_sym_SLASH_SLASH,
  [11524] = 2,
    ACTIONS(648), 11,
      sym__normal_bare_identifier,
      aux_sym__raw_string_token1,
      anon_sym_expression,
      anon_sym_minimize,
      anon_sym_maximize,
      anon_sym_expr,
      anon_sym_filter,
      anon_sym_if,
      anon_sym_lower,
      anon_sym_upper,
      anon_sym_constraint,
    ACTIONS(646), 19,
      sym_multi_line_comment,
      ts_builtin_sym_end,
      anon_sym_SLASH_DASH,
      anon_sym_RBRACE,
      anon_sym_LPAREN,
      anon_sym_DQUOTE,
      aux_sym__raw_string_token3,
      anon_sym_PLUS,
      anon_sym_DASH,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
      anon_sym_SLASH_SLASH,
  [11559] = 2,
    ACTIONS(652), 11,
      sym__normal_bare_identifier,
      aux_sym__raw_string_token1,
      anon_sym_expression,
      anon_sym_minimize,
      anon_sym_maximize,
      anon_sym_expr,
      anon_sym_filter,
      anon_sym_if,
      anon_sym_lower,
      anon_sym_upper,
      anon_sym_constraint,
    ACTIONS(650), 19,
      sym_multi_line_comment,
      ts_builtin_sym_end,
      anon_sym_SLASH_DASH,
      anon_sym_RBRACE,
      anon_sym_LPAREN,
      anon_sym_DQUOTE,
      aux_sym__raw_string_token3,
      anon_sym_PLUS,
      anon_sym_DASH,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
      anon_sym_SLASH_SLASH,
  [11594] = 2,
    ACTIONS(656), 11,
      sym__normal_bare_identifier,
      aux_sym__raw_string_token1,
      anon_sym_expression,
      anon_sym_minimize,
      anon_sym_maximize,
      anon_sym_expr,
      anon_sym_filter,
      anon_sym_if,
      anon_sym_lower,
      anon_sym_upper,
      anon_sym_constraint,
    ACTIONS(654), 19,
      sym_multi_line_comment,
      ts_builtin_sym_end,
      anon_sym_SLASH_DASH,
      anon_sym_RBRACE,
      anon_sym_LPAREN,
      anon_sym_DQUOTE,
      aux_sym__raw_string_token3,
      anon_sym_PLUS,
      anon_sym_DASH,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
      anon_sym_SLASH_SLASH,
  [11629] = 2,
    ACTIONS(660), 11,
      sym__normal_bare_identifier,
      aux_sym__raw_string_token1,
      anon_sym_expression,
      anon_sym_minimize,
      anon_sym_maximize,
      anon_sym_expr,
      anon_sym_filter,
      anon_sym_if,
      anon_sym_lower,
      anon_sym_upper,
      anon_sym_constraint,
    ACTIONS(658), 19,
      sym_multi_line_comment,
      ts_builtin_sym_end,
      anon_sym_SLASH_DASH,
      anon_sym_RBRACE,
      anon_sym_LPAREN,
      anon_sym_DQUOTE,
      aux_sym__raw_string_token3,
      anon_sym_PLUS,
      anon_sym_DASH,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
      anon_sym_SLASH_SLASH,
  [11664] = 2,
    ACTIONS(664), 11,
      sym__normal_bare_identifier,
      aux_sym__raw_string_token1,
      anon_sym_expression,
      anon_sym_minimize,
      anon_sym_maximize,
      anon_sym_expr,
      anon_sym_filter,
      anon_sym_if,
      anon_sym_lower,
      anon_sym_upper,
      anon_sym_constraint,
    ACTIONS(662), 19,
      sym_multi_line_comment,
      ts_builtin_sym_end,
      anon_sym_SLASH_DASH,
      anon_sym_RBRACE,
      anon_sym_LPAREN,
      anon_sym_DQUOTE,
      aux_sym__raw_string_token3,
      anon_sym_PLUS,
      anon_sym_DASH,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
      anon_sym_SLASH_SLASH,
  [11699] = 2,
    ACTIONS(668), 11,
      sym__normal_bare_identifier,
      aux_sym__raw_string_token1,
      anon_sym_expression,
      anon_sym_minimize,
      anon_sym_maximize,
      anon_sym_expr,
      anon_sym_filter,
      anon_sym_if,
      anon_sym_lower,
      anon_sym_upper,
      anon_sym_constraint,
    ACTIONS(666), 19,
      sym_multi_line_comment,
      ts_builtin_sym_end,
      anon_sym_SLASH_DASH,
      anon_sym_RBRACE,
      anon_sym_LPAREN,
      anon_sym_DQUOTE,
      aux_sym__raw_string_token3,
      anon_sym_PLUS,
      anon_sym_DASH,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
      anon_sym_SLASH_SLASH,
  [11734] = 2,
    ACTIONS(672), 11,
      sym__normal_bare_identifier,
      aux_sym__raw_string_token1,
      anon_sym_expression,
      anon_sym_minimize,
      anon_sym_maximize,
      anon_sym_expr,
      anon_sym_filter,
      anon_sym_if,
      anon_sym_lower,
      anon_sym_upper,
      anon_sym_constraint,
    ACTIONS(670), 19,
      sym_multi_line_comment,
      ts_builtin_sym_end,
      anon_sym_SLASH_DASH,
      anon_sym_RBRACE,
      anon_sym_LPAREN,
      anon_sym_DQUOTE,
      aux_sym__raw_string_token3,
      anon_sym_PLUS,
      anon_sym_DASH,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
      anon_sym_SLASH_SLASH,
  [11769] = 2,
    ACTIONS(676), 11,
      sym__normal_bare_identifier,
      aux_sym__raw_string_token1,
      anon_sym_expression,
      anon_sym_minimize,
      anon_sym_maximize,
      anon_sym_expr,
      anon_sym_filter,
      anon_sym_if,
      anon_sym_lower,
      anon_sym_upper,
      anon_sym_constraint,
    ACTIONS(674), 19,
      sym_multi_line_comment,
      ts_builtin_sym_end,
      anon_sym_SLASH_DASH,
      anon_sym_RBRACE,
      anon_sym_LPAREN,
      anon_sym_DQUOTE,
      aux_sym__raw_string_token3,
      anon_sym_PLUS,
      anon_sym_DASH,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
      anon_sym_SLASH_SLASH,
  [11804] = 2,
    ACTIONS(680), 11,
      sym__normal_bare_identifier,
      aux_sym__raw_string_token1,
      anon_sym_expression,
      anon_sym_minimize,
      anon_sym_maximize,
      anon_sym_expr,
      anon_sym_filter,
      anon_sym_if,
      anon_sym_lower,
      anon_sym_upper,
      anon_sym_constraint,
    ACTIONS(678), 19,
      sym_multi_line_comment,
      ts_builtin_sym_end,
      anon_sym_SLASH_DASH,
      anon_sym_RBRACE,
      anon_sym_LPAREN,
      anon_sym_DQUOTE,
      aux_sym__raw_string_token3,
      anon_sym_PLUS,
      anon_sym_DASH,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
      anon_sym_SLASH_SLASH,
  [11839] = 2,
    ACTIONS(684), 11,
      sym__normal_bare_identifier,
      aux_sym__raw_string_token1,
      anon_sym_expression,
      anon_sym_minimize,
      anon_sym_maximize,
      anon_sym_expr,
      anon_sym_filter,
      anon_sym_if,
      anon_sym_lower,
      anon_sym_upper,
      anon_sym_constraint,
    ACTIONS(682), 19,
      sym_multi_line_comment,
      ts_builtin_sym_end,
      anon_sym_SLASH_DASH,
      anon_sym_RBRACE,
      anon_sym_LPAREN,
      anon_sym_DQUOTE,
      aux_sym__raw_string_token3,
      anon_sym_PLUS,
      anon_sym_DASH,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
      anon_sym_SLASH_SLASH,
  [11874] = 2,
    ACTIONS(688), 11,
      sym__normal_bare_identifier,
      aux_sym__raw_string_token1,
      anon_sym_expression,
      anon_sym_minimize,
      anon_sym_maximize,
      anon_sym_expr,
      anon_sym_filter,
      anon_sym_if,
      anon_sym_lower,
      anon_sym_upper,
      anon_sym_constraint,
    ACTIONS(686), 19,
      sym_multi_line_comment,
      ts_builtin_sym_end,
      anon_sym_SLASH_DASH,
      anon_sym_RBRACE,
      anon_sym_LPAREN,
      anon_sym_DQUOTE,
      aux_sym__raw_string_token3,
      anon_sym_PLUS,
      anon_sym_DASH,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
      anon_sym_SLASH_SLASH,
  [11909] = 2,
    ACTIONS(692), 11,
      sym__normal_bare_identifier,
      aux_sym__raw_string_token1,
      anon_sym_expression,
      anon_sym_minimize,
      anon_sym_maximize,
      anon_sym_expr,
      anon_sym_filter,
      anon_sym_if,
      anon_sym_lower,
      anon_sym_upper,
      anon_sym_constraint,
    ACTIONS(690), 19,
      sym_multi_line_comment,
      ts_builtin_sym_end,
      anon_sym_SLASH_DASH,
      anon_sym_RBRACE,
      anon_sym_LPAREN,
      anon_sym_DQUOTE,
      aux_sym__raw_string_token3,
      anon_sym_PLUS,
      anon_sym_DASH,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
      anon_sym_SLASH_SLASH,
  [11944] = 2,
    ACTIONS(696), 11,
      sym__normal_bare_identifier,
      aux_sym__raw_string_token1,
      anon_sym_expression,
      anon_sym_minimize,
      anon_sym_maximize,
      anon_sym_expr,
      anon_sym_filter,
      anon_sym_if,
      anon_sym_lower,
      anon_sym_upper,
      anon_sym_constraint,
    ACTIONS(694), 19,
      sym_multi_line_comment,
      ts_builtin_sym_end,
      anon_sym_SLASH_DASH,
      anon_sym_RBRACE,
      anon_sym_LPAREN,
      anon_sym_DQUOTE,
      aux_sym__raw_string_token3,
      anon_sym_PLUS,
      anon_sym_DASH,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
      anon_sym_SLASH_SLASH,
  [11979] = 2,
    ACTIONS(700), 11,
      sym__normal_bare_identifier,
      aux_sym__raw_string_token1,
      anon_sym_expression,
      anon_sym_minimize,
      anon_sym_maximize,
      anon_sym_expr,
      anon_sym_filter,
      anon_sym_if,
      anon_sym_lower,
      anon_sym_upper,
      anon_sym_constraint,
    ACTIONS(698), 19,
      sym_multi_line_comment,
      ts_builtin_sym_end,
      anon_sym_SLASH_DASH,
      anon_sym_RBRACE,
      anon_sym_LPAREN,
      anon_sym_DQUOTE,
      aux_sym__raw_string_token3,
      anon_sym_PLUS,
      anon_sym_DASH,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
      anon_sym_SLASH_SLASH,
  [12014] = 2,
    ACTIONS(704), 11,
      sym__normal_bare_identifier,
      aux_sym__raw_string_token1,
      anon_sym_expression,
      anon_sym_minimize,
      anon_sym_maximize,
      anon_sym_expr,
      anon_sym_filter,
      anon_sym_if,
      anon_sym_lower,
      anon_sym_upper,
      anon_sym_constraint,
    ACTIONS(702), 19,
      sym_multi_line_comment,
      ts_builtin_sym_end,
      anon_sym_SLASH_DASH,
      anon_sym_RBRACE,
      anon_sym_LPAREN,
      anon_sym_DQUOTE,
      aux_sym__raw_string_token3,
      anon_sym_PLUS,
      anon_sym_DASH,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
      anon_sym_SLASH_SLASH,
  [12049] = 2,
    ACTIONS(708), 11,
      sym__normal_bare_identifier,
      aux_sym__raw_string_token1,
      anon_sym_expression,
      anon_sym_minimize,
      anon_sym_maximize,
      anon_sym_expr,
      anon_sym_filter,
      anon_sym_if,
      anon_sym_lower,
      anon_sym_upper,
      anon_sym_constraint,
    ACTIONS(706), 19,
      sym_multi_line_comment,
      ts_builtin_sym_end,
      anon_sym_SLASH_DASH,
      anon_sym_RBRACE,
      anon_sym_LPAREN,
      anon_sym_DQUOTE,
      aux_sym__raw_string_token3,
      anon_sym_PLUS,
      anon_sym_DASH,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
      anon_sym_SLASH_SLASH,
  [12084] = 2,
    ACTIONS(712), 11,
      sym__normal_bare_identifier,
      aux_sym__raw_string_token1,
      anon_sym_expression,
      anon_sym_minimize,
      anon_sym_maximize,
      anon_sym_expr,
      anon_sym_filter,
      anon_sym_if,
      anon_sym_lower,
      anon_sym_upper,
      anon_sym_constraint,
    ACTIONS(710), 19,
      sym_multi_line_comment,
      ts_builtin_sym_end,
      anon_sym_SLASH_DASH,
      anon_sym_RBRACE,
      anon_sym_LPAREN,
      anon_sym_DQUOTE,
      aux_sym__raw_string_token3,
      anon_sym_PLUS,
      anon_sym_DASH,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
      anon_sym_SLASH_SLASH,
  [12119] = 2,
    ACTIONS(716), 11,
      sym__normal_bare_identifier,
      aux_sym__raw_string_token1,
      anon_sym_expression,
      anon_sym_minimize,
      anon_sym_maximize,
      anon_sym_expr,
      anon_sym_filter,
      anon_sym_if,
      anon_sym_lower,
      anon_sym_upper,
      anon_sym_constraint,
    ACTIONS(714), 19,
      sym_multi_line_comment,
      ts_builtin_sym_end,
      anon_sym_SLASH_DASH,
      anon_sym_RBRACE,
      anon_sym_LPAREN,
      anon_sym_DQUOTE,
      aux_sym__raw_string_token3,
      anon_sym_PLUS,
      anon_sym_DASH,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
      anon_sym_SLASH_SLASH,
  [12154] = 2,
    ACTIONS(720), 11,
      sym__normal_bare_identifier,
      aux_sym__raw_string_token1,
      anon_sym_expression,
      anon_sym_minimize,
      anon_sym_maximize,
      anon_sym_expr,
      anon_sym_filter,
      anon_sym_if,
      anon_sym_lower,
      anon_sym_upper,
      anon_sym_constraint,
    ACTIONS(718), 19,
      sym_multi_line_comment,
      ts_builtin_sym_end,
      anon_sym_SLASH_DASH,
      anon_sym_RBRACE,
      anon_sym_LPAREN,
      anon_sym_DQUOTE,
      aux_sym__raw_string_token3,
      anon_sym_PLUS,
      anon_sym_DASH,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
      anon_sym_SLASH_SLASH,
  [12189] = 2,
    ACTIONS(724), 11,
      sym__normal_bare_identifier,
      aux_sym__raw_string_token1,
      anon_sym_expression,
      anon_sym_minimize,
      anon_sym_maximize,
      anon_sym_expr,
      anon_sym_filter,
      anon_sym_if,
      anon_sym_lower,
      anon_sym_upper,
      anon_sym_constraint,
    ACTIONS(722), 19,
      sym_multi_line_comment,
      ts_builtin_sym_end,
      anon_sym_SLASH_DASH,
      anon_sym_RBRACE,
      anon_sym_LPAREN,
      anon_sym_DQUOTE,
      aux_sym__raw_string_token3,
      anon_sym_PLUS,
      anon_sym_DASH,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
      anon_sym_SLASH_SLASH,
  [12224] = 2,
    ACTIONS(728), 11,
      sym__normal_bare_identifier,
      aux_sym__raw_string_token1,
      anon_sym_expression,
      anon_sym_minimize,
      anon_sym_maximize,
      anon_sym_expr,
      anon_sym_filter,
      anon_sym_if,
      anon_sym_lower,
      anon_sym_upper,
      anon_sym_constraint,
    ACTIONS(726), 19,
      sym_multi_line_comment,
      ts_builtin_sym_end,
      anon_sym_SLASH_DASH,
      anon_sym_RBRACE,
      anon_sym_LPAREN,
      anon_sym_DQUOTE,
      aux_sym__raw_string_token3,
      anon_sym_PLUS,
      anon_sym_DASH,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
      anon_sym_SLASH_SLASH,
  [12259] = 2,
    ACTIONS(732), 11,
      sym__normal_bare_identifier,
      aux_sym__raw_string_token1,
      anon_sym_expression,
      anon_sym_minimize,
      anon_sym_maximize,
      anon_sym_expr,
      anon_sym_filter,
      anon_sym_if,
      anon_sym_lower,
      anon_sym_upper,
      anon_sym_constraint,
    ACTIONS(730), 19,
      sym_multi_line_comment,
      ts_builtin_sym_end,
      anon_sym_SLASH_DASH,
      anon_sym_RBRACE,
      anon_sym_LPAREN,
      anon_sym_DQUOTE,
      aux_sym__raw_string_token3,
      anon_sym_PLUS,
      anon_sym_DASH,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
      anon_sym_SLASH_SLASH,
  [12294] = 2,
    ACTIONS(736), 11,
      sym__normal_bare_identifier,
      aux_sym__raw_string_token1,
      anon_sym_expression,
      anon_sym_minimize,
      anon_sym_maximize,
      anon_sym_expr,
      anon_sym_filter,
      anon_sym_if,
      anon_sym_lower,
      anon_sym_upper,
      anon_sym_constraint,
    ACTIONS(734), 19,
      sym_multi_line_comment,
      ts_builtin_sym_end,
      anon_sym_SLASH_DASH,
      anon_sym_RBRACE,
      anon_sym_LPAREN,
      anon_sym_DQUOTE,
      aux_sym__raw_string_token3,
      anon_sym_PLUS,
      anon_sym_DASH,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
      anon_sym_SLASH_SLASH,
  [12329] = 2,
    ACTIONS(740), 11,
      sym__normal_bare_identifier,
      aux_sym__raw_string_token1,
      anon_sym_expression,
      anon_sym_minimize,
      anon_sym_maximize,
      anon_sym_expr,
      anon_sym_filter,
      anon_sym_if,
      anon_sym_lower,
      anon_sym_upper,
      anon_sym_constraint,
    ACTIONS(738), 19,
      sym_multi_line_comment,
      ts_builtin_sym_end,
      anon_sym_SLASH_DASH,
      anon_sym_RBRACE,
      anon_sym_LPAREN,
      anon_sym_DQUOTE,
      aux_sym__raw_string_token3,
      anon_sym_PLUS,
      anon_sym_DASH,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
      anon_sym_SLASH_SLASH,
  [12364] = 2,
    ACTIONS(744), 11,
      sym__normal_bare_identifier,
      aux_sym__raw_string_token1,
      anon_sym_expression,
      anon_sym_minimize,
      anon_sym_maximize,
      anon_sym_expr,
      anon_sym_filter,
      anon_sym_if,
      anon_sym_lower,
      anon_sym_upper,
      anon_sym_constraint,
    ACTIONS(742), 19,
      sym_multi_line_comment,
      ts_builtin_sym_end,
      anon_sym_SLASH_DASH,
      anon_sym_RBRACE,
      anon_sym_LPAREN,
      anon_sym_DQUOTE,
      aux_sym__raw_string_token3,
      anon_sym_PLUS,
      anon_sym_DASH,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
      anon_sym_SLASH_SLASH,
  [12399] = 2,
    ACTIONS(748), 11,
      sym__normal_bare_identifier,
      aux_sym__raw_string_token1,
      anon_sym_expression,
      anon_sym_minimize,
      anon_sym_maximize,
      anon_sym_expr,
      anon_sym_filter,
      anon_sym_if,
      anon_sym_lower,
      anon_sym_upper,
      anon_sym_constraint,
    ACTIONS(746), 19,
      sym_multi_line_comment,
      ts_builtin_sym_end,
      anon_sym_SLASH_DASH,
      anon_sym_RBRACE,
      anon_sym_LPAREN,
      anon_sym_DQUOTE,
      aux_sym__raw_string_token3,
      anon_sym_PLUS,
      anon_sym_DASH,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
      anon_sym_SLASH_SLASH,
  [12434] = 2,
    ACTIONS(752), 11,
      sym__normal_bare_identifier,
      aux_sym__raw_string_token1,
      anon_sym_expression,
      anon_sym_minimize,
      anon_sym_maximize,
      anon_sym_expr,
      anon_sym_filter,
      anon_sym_if,
      anon_sym_lower,
      anon_sym_upper,
      anon_sym_constraint,
    ACTIONS(750), 19,
      sym_multi_line_comment,
      ts_builtin_sym_end,
      anon_sym_SLASH_DASH,
      anon_sym_RBRACE,
      anon_sym_LPAREN,
      anon_sym_DQUOTE,
      aux_sym__raw_string_token3,
      anon_sym_PLUS,
      anon_sym_DASH,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
      anon_sym_SLASH_SLASH,
  [12469] = 2,
    ACTIONS(756), 11,
      sym__normal_bare_identifier,
      aux_sym__raw_string_token1,
      anon_sym_expression,
      anon_sym_minimize,
      anon_sym_maximize,
      anon_sym_expr,
      anon_sym_filter,
      anon_sym_if,
      anon_sym_lower,
      anon_sym_upper,
      anon_sym_constraint,
    ACTIONS(754), 19,
      sym_multi_line_comment,
      ts_builtin_sym_end,
      anon_sym_SLASH_DASH,
      anon_sym_RBRACE,
      anon_sym_LPAREN,
      anon_sym_DQUOTE,
      aux_sym__raw_string_token3,
      anon_sym_PLUS,
      anon_sym_DASH,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
      anon_sym_SLASH_SLASH,
  [12504] = 2,
    ACTIONS(760), 11,
      sym__normal_bare_identifier,
      aux_sym__raw_string_token1,
      anon_sym_expression,
      anon_sym_minimize,
      anon_sym_maximize,
      anon_sym_expr,
      anon_sym_filter,
      anon_sym_if,
      anon_sym_lower,
      anon_sym_upper,
      anon_sym_constraint,
    ACTIONS(758), 19,
      sym_multi_line_comment,
      ts_builtin_sym_end,
      anon_sym_SLASH_DASH,
      anon_sym_RBRACE,
      anon_sym_LPAREN,
      anon_sym_DQUOTE,
      aux_sym__raw_string_token3,
      anon_sym_PLUS,
      anon_sym_DASH,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
      anon_sym_SLASH_SLASH,
  [12539] = 2,
    ACTIONS(764), 11,
      sym__normal_bare_identifier,
      aux_sym__raw_string_token1,
      anon_sym_expression,
      anon_sym_minimize,
      anon_sym_maximize,
      anon_sym_expr,
      anon_sym_filter,
      anon_sym_if,
      anon_sym_lower,
      anon_sym_upper,
      anon_sym_constraint,
    ACTIONS(762), 19,
      sym_multi_line_comment,
      ts_builtin_sym_end,
      anon_sym_SLASH_DASH,
      anon_sym_RBRACE,
      anon_sym_LPAREN,
      anon_sym_DQUOTE,
      aux_sym__raw_string_token3,
      anon_sym_PLUS,
      anon_sym_DASH,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
      anon_sym_SLASH_SLASH,
  [12574] = 2,
    ACTIONS(768), 11,
      sym__normal_bare_identifier,
      aux_sym__raw_string_token1,
      anon_sym_expression,
      anon_sym_minimize,
      anon_sym_maximize,
      anon_sym_expr,
      anon_sym_filter,
      anon_sym_if,
      anon_sym_lower,
      anon_sym_upper,
      anon_sym_constraint,
    ACTIONS(766), 19,
      sym_multi_line_comment,
      ts_builtin_sym_end,
      anon_sym_SLASH_DASH,
      anon_sym_RBRACE,
      anon_sym_LPAREN,
      anon_sym_DQUOTE,
      aux_sym__raw_string_token3,
      anon_sym_PLUS,
      anon_sym_DASH,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
      anon_sym_SLASH_SLASH,
  [12609] = 2,
    ACTIONS(772), 11,
      sym__normal_bare_identifier,
      aux_sym__raw_string_token1,
      anon_sym_expression,
      anon_sym_minimize,
      anon_sym_maximize,
      anon_sym_expr,
      anon_sym_filter,
      anon_sym_if,
      anon_sym_lower,
      anon_sym_upper,
      anon_sym_constraint,
    ACTIONS(770), 19,
      sym_multi_line_comment,
      ts_builtin_sym_end,
      anon_sym_SLASH_DASH,
      anon_sym_RBRACE,
      anon_sym_LPAREN,
      anon_sym_DQUOTE,
      aux_sym__raw_string_token3,
      anon_sym_PLUS,
      anon_sym_DASH,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
      anon_sym_SLASH_SLASH,
  [12644] = 2,
    ACTIONS(776), 11,
      sym__normal_bare_identifier,
      aux_sym__raw_string_token1,
      anon_sym_expression,
      anon_sym_minimize,
      anon_sym_maximize,
      anon_sym_expr,
      anon_sym_filter,
      anon_sym_if,
      anon_sym_lower,
      anon_sym_upper,
      anon_sym_constraint,
    ACTIONS(774), 19,
      sym_multi_line_comment,
      ts_builtin_sym_end,
      anon_sym_SLASH_DASH,
      anon_sym_RBRACE,
      anon_sym_LPAREN,
      anon_sym_DQUOTE,
      aux_sym__raw_string_token3,
      anon_sym_PLUS,
      anon_sym_DASH,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
      anon_sym_SLASH_SLASH,
  [12679] = 2,
    ACTIONS(780), 11,
      sym__normal_bare_identifier,
      aux_sym__raw_string_token1,
      anon_sym_expression,
      anon_sym_minimize,
      anon_sym_maximize,
      anon_sym_expr,
      anon_sym_filter,
      anon_sym_if,
      anon_sym_lower,
      anon_sym_upper,
      anon_sym_constraint,
    ACTIONS(778), 19,
      sym_multi_line_comment,
      ts_builtin_sym_end,
      anon_sym_SLASH_DASH,
      anon_sym_RBRACE,
      anon_sym_LPAREN,
      anon_sym_DQUOTE,
      aux_sym__raw_string_token3,
      anon_sym_PLUS,
      anon_sym_DASH,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
      anon_sym_SLASH_SLASH,
  [12714] = 2,
    ACTIONS(784), 11,
      sym__normal_bare_identifier,
      aux_sym__raw_string_token1,
      anon_sym_expression,
      anon_sym_minimize,
      anon_sym_maximize,
      anon_sym_expr,
      anon_sym_filter,
      anon_sym_if,
      anon_sym_lower,
      anon_sym_upper,
      anon_sym_constraint,
    ACTIONS(782), 19,
      sym_multi_line_comment,
      ts_builtin_sym_end,
      anon_sym_SLASH_DASH,
      anon_sym_RBRACE,
      anon_sym_LPAREN,
      anon_sym_DQUOTE,
      aux_sym__raw_string_token3,
      anon_sym_PLUS,
      anon_sym_DASH,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
      anon_sym_SLASH_SLASH,
  [12749] = 2,
    ACTIONS(788), 11,
      sym__normal_bare_identifier,
      aux_sym__raw_string_token1,
      anon_sym_expression,
      anon_sym_minimize,
      anon_sym_maximize,
      anon_sym_expr,
      anon_sym_filter,
      anon_sym_if,
      anon_sym_lower,
      anon_sym_upper,
      anon_sym_constraint,
    ACTIONS(786), 19,
      sym_multi_line_comment,
      ts_builtin_sym_end,
      anon_sym_SLASH_DASH,
      anon_sym_RBRACE,
      anon_sym_LPAREN,
      anon_sym_DQUOTE,
      aux_sym__raw_string_token3,
      anon_sym_PLUS,
      anon_sym_DASH,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
      anon_sym_SLASH_SLASH,
  [12784] = 2,
    ACTIONS(792), 11,
      sym__normal_bare_identifier,
      aux_sym__raw_string_token1,
      anon_sym_expression,
      anon_sym_minimize,
      anon_sym_maximize,
      anon_sym_expr,
      anon_sym_filter,
      anon_sym_if,
      anon_sym_lower,
      anon_sym_upper,
      anon_sym_constraint,
    ACTIONS(790), 19,
      sym_multi_line_comment,
      ts_builtin_sym_end,
      anon_sym_SLASH_DASH,
      anon_sym_RBRACE,
      anon_sym_LPAREN,
      anon_sym_DQUOTE,
      aux_sym__raw_string_token3,
      anon_sym_PLUS,
      anon_sym_DASH,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
      anon_sym_SLASH_SLASH,
  [12819] = 2,
    ACTIONS(796), 11,
      sym__normal_bare_identifier,
      aux_sym__raw_string_token1,
      anon_sym_expression,
      anon_sym_minimize,
      anon_sym_maximize,
      anon_sym_expr,
      anon_sym_filter,
      anon_sym_if,
      anon_sym_lower,
      anon_sym_upper,
      anon_sym_constraint,
    ACTIONS(794), 19,
      sym_multi_line_comment,
      ts_builtin_sym_end,
      anon_sym_SLASH_DASH,
      anon_sym_RBRACE,
      anon_sym_LPAREN,
      anon_sym_DQUOTE,
      aux_sym__raw_string_token3,
      anon_sym_PLUS,
      anon_sym_DASH,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
      anon_sym_SLASH_SLASH,
  [12854] = 2,
    ACTIONS(800), 11,
      sym__normal_bare_identifier,
      aux_sym__raw_string_token1,
      anon_sym_expression,
      anon_sym_minimize,
      anon_sym_maximize,
      anon_sym_expr,
      anon_sym_filter,
      anon_sym_if,
      anon_sym_lower,
      anon_sym_upper,
      anon_sym_constraint,
    ACTIONS(798), 19,
      sym_multi_line_comment,
      ts_builtin_sym_end,
      anon_sym_SLASH_DASH,
      anon_sym_RBRACE,
      anon_sym_LPAREN,
      anon_sym_DQUOTE,
      aux_sym__raw_string_token3,
      anon_sym_PLUS,
      anon_sym_DASH,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
      anon_sym_SLASH_SLASH,
  [12889] = 2,
    ACTIONS(804), 11,
      sym__normal_bare_identifier,
      aux_sym__raw_string_token1,
      anon_sym_expression,
      anon_sym_minimize,
      anon_sym_maximize,
      anon_sym_expr,
      anon_sym_filter,
      anon_sym_if,
      anon_sym_lower,
      anon_sym_upper,
      anon_sym_constraint,
    ACTIONS(802), 19,
      sym_multi_line_comment,
      ts_builtin_sym_end,
      anon_sym_SLASH_DASH,
      anon_sym_RBRACE,
      anon_sym_LPAREN,
      anon_sym_DQUOTE,
      aux_sym__raw_string_token3,
      anon_sym_PLUS,
      anon_sym_DASH,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
      anon_sym_SLASH_SLASH,
  [12924] = 2,
    ACTIONS(808), 11,
      sym__normal_bare_identifier,
      aux_sym__raw_string_token1,
      anon_sym_expression,
      anon_sym_minimize,
      anon_sym_maximize,
      anon_sym_expr,
      anon_sym_filter,
      anon_sym_if,
      anon_sym_lower,
      anon_sym_upper,
      anon_sym_constraint,
    ACTIONS(806), 19,
      sym_multi_line_comment,
      ts_builtin_sym_end,
      anon_sym_SLASH_DASH,
      anon_sym_RBRACE,
      anon_sym_LPAREN,
      anon_sym_DQUOTE,
      aux_sym__raw_string_token3,
      anon_sym_PLUS,
      anon_sym_DASH,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
      anon_sym_SLASH_SLASH,
  [12959] = 2,
    ACTIONS(812), 11,
      sym__normal_bare_identifier,
      aux_sym__raw_string_token1,
      anon_sym_expression,
      anon_sym_minimize,
      anon_sym_maximize,
      anon_sym_expr,
      anon_sym_filter,
      anon_sym_if,
      anon_sym_lower,
      anon_sym_upper,
      anon_sym_constraint,
    ACTIONS(810), 19,
      sym_multi_line_comment,
      ts_builtin_sym_end,
      anon_sym_SLASH_DASH,
      anon_sym_RBRACE,
      anon_sym_LPAREN,
      anon_sym_DQUOTE,
      aux_sym__raw_string_token3,
      anon_sym_PLUS,
      anon_sym_DASH,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
      anon_sym_SLASH_SLASH,
  [12994] = 2,
    ACTIONS(816), 11,
      sym__normal_bare_identifier,
      aux_sym__raw_string_token1,
      anon_sym_expression,
      anon_sym_minimize,
      anon_sym_maximize,
      anon_sym_expr,
      anon_sym_filter,
      anon_sym_if,
      anon_sym_lower,
      anon_sym_upper,
      anon_sym_constraint,
    ACTIONS(814), 19,
      sym_multi_line_comment,
      ts_builtin_sym_end,
      anon_sym_SLASH_DASH,
      anon_sym_RBRACE,
      anon_sym_LPAREN,
      anon_sym_DQUOTE,
      aux_sym__raw_string_token3,
      anon_sym_PLUS,
      anon_sym_DASH,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
      anon_sym_SLASH_SLASH,
  [13029] = 2,
    ACTIONS(820), 11,
      sym__normal_bare_identifier,
      aux_sym__raw_string_token1,
      anon_sym_expression,
      anon_sym_minimize,
      anon_sym_maximize,
      anon_sym_expr,
      anon_sym_filter,
      anon_sym_if,
      anon_sym_lower,
      anon_sym_upper,
      anon_sym_constraint,
    ACTIONS(818), 19,
      sym_multi_line_comment,
      ts_builtin_sym_end,
      anon_sym_SLASH_DASH,
      anon_sym_RBRACE,
      anon_sym_LPAREN,
      anon_sym_DQUOTE,
      aux_sym__raw_string_token3,
      anon_sym_PLUS,
      anon_sym_DASH,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
      anon_sym_SLASH_SLASH,
  [13064] = 2,
    ACTIONS(824), 11,
      sym__normal_bare_identifier,
      aux_sym__raw_string_token1,
      anon_sym_expression,
      anon_sym_minimize,
      anon_sym_maximize,
      anon_sym_expr,
      anon_sym_filter,
      anon_sym_if,
      anon_sym_lower,
      anon_sym_upper,
      anon_sym_constraint,
    ACTIONS(822), 19,
      sym_multi_line_comment,
      ts_builtin_sym_end,
      anon_sym_SLASH_DASH,
      anon_sym_RBRACE,
      anon_sym_LPAREN,
      anon_sym_DQUOTE,
      aux_sym__raw_string_token3,
      anon_sym_PLUS,
      anon_sym_DASH,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
      anon_sym_SLASH_SLASH,
  [13099] = 2,
    ACTIONS(828), 11,
      sym__normal_bare_identifier,
      aux_sym__raw_string_token1,
      anon_sym_expression,
      anon_sym_minimize,
      anon_sym_maximize,
      anon_sym_expr,
      anon_sym_filter,
      anon_sym_if,
      anon_sym_lower,
      anon_sym_upper,
      anon_sym_constraint,
    ACTIONS(826), 19,
      sym_multi_line_comment,
      ts_builtin_sym_end,
      anon_sym_SLASH_DASH,
      anon_sym_RBRACE,
      anon_sym_LPAREN,
      anon_sym_DQUOTE,
      aux_sym__raw_string_token3,
      anon_sym_PLUS,
      anon_sym_DASH,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
      anon_sym_SLASH_SLASH,
  [13134] = 2,
    ACTIONS(832), 11,
      sym__normal_bare_identifier,
      aux_sym__raw_string_token1,
      anon_sym_expression,
      anon_sym_minimize,
      anon_sym_maximize,
      anon_sym_expr,
      anon_sym_filter,
      anon_sym_if,
      anon_sym_lower,
      anon_sym_upper,
      anon_sym_constraint,
    ACTIONS(830), 19,
      sym_multi_line_comment,
      ts_builtin_sym_end,
      anon_sym_SLASH_DASH,
      anon_sym_RBRACE,
      anon_sym_LPAREN,
      anon_sym_DQUOTE,
      aux_sym__raw_string_token3,
      anon_sym_PLUS,
      anon_sym_DASH,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
      anon_sym_SLASH_SLASH,
  [13169] = 2,
    ACTIONS(836), 11,
      sym__normal_bare_identifier,
      aux_sym__raw_string_token1,
      anon_sym_expression,
      anon_sym_minimize,
      anon_sym_maximize,
      anon_sym_expr,
      anon_sym_filter,
      anon_sym_if,
      anon_sym_lower,
      anon_sym_upper,
      anon_sym_constraint,
    ACTIONS(834), 19,
      sym_multi_line_comment,
      ts_builtin_sym_end,
      anon_sym_SLASH_DASH,
      anon_sym_RBRACE,
      anon_sym_LPAREN,
      anon_sym_DQUOTE,
      aux_sym__raw_string_token3,
      anon_sym_PLUS,
      anon_sym_DASH,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
      anon_sym_SLASH_SLASH,
  [13204] = 2,
    ACTIONS(840), 11,
      sym__normal_bare_identifier,
      aux_sym__raw_string_token1,
      anon_sym_expression,
      anon_sym_minimize,
      anon_sym_maximize,
      anon_sym_expr,
      anon_sym_filter,
      anon_sym_if,
      anon_sym_lower,
      anon_sym_upper,
      anon_sym_constraint,
    ACTIONS(838), 19,
      sym_multi_line_comment,
      ts_builtin_sym_end,
      anon_sym_SLASH_DASH,
      anon_sym_RBRACE,
      anon_sym_LPAREN,
      anon_sym_DQUOTE,
      aux_sym__raw_string_token3,
      anon_sym_PLUS,
      anon_sym_DASH,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
      anon_sym_SLASH_SLASH,
  [13239] = 2,
    ACTIONS(844), 11,
      sym__normal_bare_identifier,
      aux_sym__raw_string_token1,
      anon_sym_expression,
      anon_sym_minimize,
      anon_sym_maximize,
      anon_sym_expr,
      anon_sym_filter,
      anon_sym_if,
      anon_sym_lower,
      anon_sym_upper,
      anon_sym_constraint,
    ACTIONS(842), 19,
      sym_multi_line_comment,
      ts_builtin_sym_end,
      anon_sym_SLASH_DASH,
      anon_sym_RBRACE,
      anon_sym_LPAREN,
      anon_sym_DQUOTE,
      aux_sym__raw_string_token3,
      anon_sym_PLUS,
      anon_sym_DASH,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
      anon_sym_SLASH_SLASH,
  [13274] = 2,
    ACTIONS(848), 11,
      sym__normal_bare_identifier,
      aux_sym__raw_string_token1,
      anon_sym_expression,
      anon_sym_minimize,
      anon_sym_maximize,
      anon_sym_expr,
      anon_sym_filter,
      anon_sym_if,
      anon_sym_lower,
      anon_sym_upper,
      anon_sym_constraint,
    ACTIONS(846), 19,
      sym_multi_line_comment,
      ts_builtin_sym_end,
      anon_sym_SLASH_DASH,
      anon_sym_RBRACE,
      anon_sym_LPAREN,
      anon_sym_DQUOTE,
      aux_sym__raw_string_token3,
      anon_sym_PLUS,
      anon_sym_DASH,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
      anon_sym_SLASH_SLASH,
  [13309] = 2,
    ACTIONS(852), 11,
      sym__normal_bare_identifier,
      aux_sym__raw_string_token1,
      anon_sym_expression,
      anon_sym_minimize,
      anon_sym_maximize,
      anon_sym_expr,
      anon_sym_filter,
      anon_sym_if,
      anon_sym_lower,
      anon_sym_upper,
      anon_sym_constraint,
    ACTIONS(850), 19,
      sym_multi_line_comment,
      ts_builtin_sym_end,
      anon_sym_SLASH_DASH,
      anon_sym_RBRACE,
      anon_sym_LPAREN,
      anon_sym_DQUOTE,
      aux_sym__raw_string_token3,
      anon_sym_PLUS,
      anon_sym_DASH,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
      anon_sym_SLASH_SLASH,
  [13344] = 2,
    ACTIONS(856), 11,
      sym__normal_bare_identifier,
      aux_sym__raw_string_token1,
      anon_sym_expression,
      anon_sym_minimize,
      anon_sym_maximize,
      anon_sym_expr,
      anon_sym_filter,
      anon_sym_if,
      anon_sym_lower,
      anon_sym_upper,
      anon_sym_constraint,
    ACTIONS(854), 19,
      sym_multi_line_comment,
      ts_builtin_sym_end,
      anon_sym_SLASH_DASH,
      anon_sym_RBRACE,
      anon_sym_LPAREN,
      anon_sym_DQUOTE,
      aux_sym__raw_string_token3,
      anon_sym_PLUS,
      anon_sym_DASH,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
      anon_sym_SLASH_SLASH,
  [13379] = 2,
    ACTIONS(860), 11,
      sym__normal_bare_identifier,
      aux_sym__raw_string_token1,
      anon_sym_expression,
      anon_sym_minimize,
      anon_sym_maximize,
      anon_sym_expr,
      anon_sym_filter,
      anon_sym_if,
      anon_sym_lower,
      anon_sym_upper,
      anon_sym_constraint,
    ACTIONS(858), 19,
      sym_multi_line_comment,
      ts_builtin_sym_end,
      anon_sym_SLASH_DASH,
      anon_sym_RBRACE,
      anon_sym_LPAREN,
      anon_sym_DQUOTE,
      aux_sym__raw_string_token3,
      anon_sym_PLUS,
      anon_sym_DASH,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
      anon_sym_SLASH_SLASH,
  [13414] = 2,
    ACTIONS(864), 11,
      sym__normal_bare_identifier,
      aux_sym__raw_string_token1,
      anon_sym_expression,
      anon_sym_minimize,
      anon_sym_maximize,
      anon_sym_expr,
      anon_sym_filter,
      anon_sym_if,
      anon_sym_lower,
      anon_sym_upper,
      anon_sym_constraint,
    ACTIONS(862), 19,
      sym_multi_line_comment,
      ts_builtin_sym_end,
      anon_sym_SLASH_DASH,
      anon_sym_RBRACE,
      anon_sym_LPAREN,
      anon_sym_DQUOTE,
      aux_sym__raw_string_token3,
      anon_sym_PLUS,
      anon_sym_DASH,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
      anon_sym_SLASH_SLASH,
  [13449] = 2,
    ACTIONS(868), 11,
      sym__normal_bare_identifier,
      aux_sym__raw_string_token1,
      anon_sym_expression,
      anon_sym_minimize,
      anon_sym_maximize,
      anon_sym_expr,
      anon_sym_filter,
      anon_sym_if,
      anon_sym_lower,
      anon_sym_upper,
      anon_sym_constraint,
    ACTIONS(866), 19,
      sym_multi_line_comment,
      ts_builtin_sym_end,
      anon_sym_SLASH_DASH,
      anon_sym_RBRACE,
      anon_sym_LPAREN,
      anon_sym_DQUOTE,
      aux_sym__raw_string_token3,
      anon_sym_PLUS,
      anon_sym_DASH,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
      anon_sym_SLASH_SLASH,
  [13484] = 2,
    ACTIONS(872), 11,
      sym__normal_bare_identifier,
      aux_sym__raw_string_token1,
      anon_sym_expression,
      anon_sym_minimize,
      anon_sym_maximize,
      anon_sym_expr,
      anon_sym_filter,
      anon_sym_if,
      anon_sym_lower,
      anon_sym_upper,
      anon_sym_constraint,
    ACTIONS(870), 19,
      sym_multi_line_comment,
      ts_builtin_sym_end,
      anon_sym_SLASH_DASH,
      anon_sym_RBRACE,
      anon_sym_LPAREN,
      anon_sym_DQUOTE,
      aux_sym__raw_string_token3,
      anon_sym_PLUS,
      anon_sym_DASH,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
      anon_sym_SLASH_SLASH,
  [13519] = 2,
    ACTIONS(876), 11,
      sym__normal_bare_identifier,
      aux_sym__raw_string_token1,
      anon_sym_expression,
      anon_sym_minimize,
      anon_sym_maximize,
      anon_sym_expr,
      anon_sym_filter,
      anon_sym_if,
      anon_sym_lower,
      anon_sym_upper,
      anon_sym_constraint,
    ACTIONS(874), 19,
      sym_multi_line_comment,
      ts_builtin_sym_end,
      anon_sym_SLASH_DASH,
      anon_sym_RBRACE,
      anon_sym_LPAREN,
      anon_sym_DQUOTE,
      aux_sym__raw_string_token3,
      anon_sym_PLUS,
      anon_sym_DASH,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
      anon_sym_SLASH_SLASH,
  [13554] = 2,
    ACTIONS(880), 11,
      sym__normal_bare_identifier,
      aux_sym__raw_string_token1,
      anon_sym_expression,
      anon_sym_minimize,
      anon_sym_maximize,
      anon_sym_expr,
      anon_sym_filter,
      anon_sym_if,
      anon_sym_lower,
      anon_sym_upper,
      anon_sym_constraint,
    ACTIONS(878), 19,
      sym_multi_line_comment,
      ts_builtin_sym_end,
      anon_sym_SLASH_DASH,
      anon_sym_RBRACE,
      anon_sym_LPAREN,
      anon_sym_DQUOTE,
      aux_sym__raw_string_token3,
      anon_sym_PLUS,
      anon_sym_DASH,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
      anon_sym_SLASH_SLASH,
  [13589] = 2,
    ACTIONS(884), 11,
      sym__normal_bare_identifier,
      aux_sym__raw_string_token1,
      anon_sym_expression,
      anon_sym_minimize,
      anon_sym_maximize,
      anon_sym_expr,
      anon_sym_filter,
      anon_sym_if,
      anon_sym_lower,
      anon_sym_upper,
      anon_sym_constraint,
    ACTIONS(882), 19,
      sym_multi_line_comment,
      ts_builtin_sym_end,
      anon_sym_SLASH_DASH,
      anon_sym_RBRACE,
      anon_sym_LPAREN,
      anon_sym_DQUOTE,
      aux_sym__raw_string_token3,
      anon_sym_PLUS,
      anon_sym_DASH,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
      anon_sym_SLASH_SLASH,
  [13624] = 2,
    ACTIONS(888), 11,
      sym__normal_bare_identifier,
      aux_sym__raw_string_token1,
      anon_sym_expression,
      anon_sym_minimize,
      anon_sym_maximize,
      anon_sym_expr,
      anon_sym_filter,
      anon_sym_if,
      anon_sym_lower,
      anon_sym_upper,
      anon_sym_constraint,
    ACTIONS(886), 19,
      sym_multi_line_comment,
      ts_builtin_sym_end,
      anon_sym_SLASH_DASH,
      anon_sym_RBRACE,
      anon_sym_LPAREN,
      anon_sym_DQUOTE,
      aux_sym__raw_string_token3,
      anon_sym_PLUS,
      anon_sym_DASH,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
      anon_sym_SLASH_SLASH,
  [13659] = 2,
    ACTIONS(892), 11,
      sym__normal_bare_identifier,
      aux_sym__raw_string_token1,
      anon_sym_expression,
      anon_sym_minimize,
      anon_sym_maximize,
      anon_sym_expr,
      anon_sym_filter,
      anon_sym_if,
      anon_sym_lower,
      anon_sym_upper,
      anon_sym_constraint,
    ACTIONS(890), 19,
      sym_multi_line_comment,
      ts_builtin_sym_end,
      anon_sym_SLASH_DASH,
      anon_sym_RBRACE,
      anon_sym_LPAREN,
      anon_sym_DQUOTE,
      aux_sym__raw_string_token3,
      anon_sym_PLUS,
      anon_sym_DASH,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
      anon_sym_SLASH_SLASH,
  [13694] = 2,
    ACTIONS(896), 11,
      sym__normal_bare_identifier,
      aux_sym__raw_string_token1,
      anon_sym_expression,
      anon_sym_minimize,
      anon_sym_maximize,
      anon_sym_expr,
      anon_sym_filter,
      anon_sym_if,
      anon_sym_lower,
      anon_sym_upper,
      anon_sym_constraint,
    ACTIONS(894), 19,
      sym_multi_line_comment,
      ts_builtin_sym_end,
      anon_sym_SLASH_DASH,
      anon_sym_RBRACE,
      anon_sym_LPAREN,
      anon_sym_DQUOTE,
      aux_sym__raw_string_token3,
      anon_sym_PLUS,
      anon_sym_DASH,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
      anon_sym_SLASH_SLASH,
  [13729] = 2,
    ACTIONS(900), 11,
      sym__normal_bare_identifier,
      aux_sym__raw_string_token1,
      anon_sym_expression,
      anon_sym_minimize,
      anon_sym_maximize,
      anon_sym_expr,
      anon_sym_filter,
      anon_sym_if,
      anon_sym_lower,
      anon_sym_upper,
      anon_sym_constraint,
    ACTIONS(898), 19,
      sym_multi_line_comment,
      ts_builtin_sym_end,
      anon_sym_SLASH_DASH,
      anon_sym_RBRACE,
      anon_sym_LPAREN,
      anon_sym_DQUOTE,
      aux_sym__raw_string_token3,
      anon_sym_PLUS,
      anon_sym_DASH,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
      anon_sym_SLASH_SLASH,
  [13764] = 2,
    ACTIONS(904), 11,
      sym__normal_bare_identifier,
      aux_sym__raw_string_token1,
      anon_sym_expression,
      anon_sym_minimize,
      anon_sym_maximize,
      anon_sym_expr,
      anon_sym_filter,
      anon_sym_if,
      anon_sym_lower,
      anon_sym_upper,
      anon_sym_constraint,
    ACTIONS(902), 19,
      sym_multi_line_comment,
      ts_builtin_sym_end,
      anon_sym_SLASH_DASH,
      anon_sym_RBRACE,
      anon_sym_LPAREN,
      anon_sym_DQUOTE,
      aux_sym__raw_string_token3,
      anon_sym_PLUS,
      anon_sym_DASH,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
      anon_sym_SLASH_SLASH,
  [13799] = 2,
    ACTIONS(908), 11,
      sym__normal_bare_identifier,
      aux_sym__raw_string_token1,
      anon_sym_expression,
      anon_sym_minimize,
      anon_sym_maximize,
      anon_sym_expr,
      anon_sym_filter,
      anon_sym_if,
      anon_sym_lower,
      anon_sym_upper,
      anon_sym_constraint,
    ACTIONS(906), 19,
      sym_multi_line_comment,
      ts_builtin_sym_end,
      anon_sym_SLASH_DASH,
      anon_sym_RBRACE,
      anon_sym_LPAREN,
      anon_sym_DQUOTE,
      aux_sym__raw_string_token3,
      anon_sym_PLUS,
      anon_sym_DASH,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
      anon_sym_SLASH_SLASH,
  [13834] = 2,
    ACTIONS(912), 11,
      sym__normal_bare_identifier,
      aux_sym__raw_string_token1,
      anon_sym_expression,
      anon_sym_minimize,
      anon_sym_maximize,
      anon_sym_expr,
      anon_sym_filter,
      anon_sym_if,
      anon_sym_lower,
      anon_sym_upper,
      anon_sym_constraint,
    ACTIONS(910), 19,
      sym_multi_line_comment,
      ts_builtin_sym_end,
      anon_sym_SLASH_DASH,
      anon_sym_RBRACE,
      anon_sym_LPAREN,
      anon_sym_DQUOTE,
      aux_sym__raw_string_token3,
      anon_sym_PLUS,
      anon_sym_DASH,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
      anon_sym_SLASH_SLASH,
  [13869] = 2,
    ACTIONS(916), 11,
      sym__normal_bare_identifier,
      aux_sym__raw_string_token1,
      anon_sym_expression,
      anon_sym_minimize,
      anon_sym_maximize,
      anon_sym_expr,
      anon_sym_filter,
      anon_sym_if,
      anon_sym_lower,
      anon_sym_upper,
      anon_sym_constraint,
    ACTIONS(914), 19,
      sym_multi_line_comment,
      ts_builtin_sym_end,
      anon_sym_SLASH_DASH,
      anon_sym_RBRACE,
      anon_sym_LPAREN,
      anon_sym_DQUOTE,
      aux_sym__raw_string_token3,
      anon_sym_PLUS,
      anon_sym_DASH,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
      anon_sym_SLASH_SLASH,
  [13904] = 2,
    ACTIONS(920), 11,
      sym__normal_bare_identifier,
      aux_sym__raw_string_token1,
      anon_sym_expression,
      anon_sym_minimize,
      anon_sym_maximize,
      anon_sym_expr,
      anon_sym_filter,
      anon_sym_if,
      anon_sym_lower,
      anon_sym_upper,
      anon_sym_constraint,
    ACTIONS(918), 19,
      sym_multi_line_comment,
      ts_builtin_sym_end,
      anon_sym_SLASH_DASH,
      anon_sym_RBRACE,
      anon_sym_LPAREN,
      anon_sym_DQUOTE,
      aux_sym__raw_string_token3,
      anon_sym_PLUS,
      anon_sym_DASH,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
      anon_sym_SLASH_SLASH,
  [13939] = 2,
    ACTIONS(924), 11,
      sym__normal_bare_identifier,
      aux_sym__raw_string_token1,
      anon_sym_expression,
      anon_sym_minimize,
      anon_sym_maximize,
      anon_sym_expr,
      anon_sym_filter,
      anon_sym_if,
      anon_sym_lower,
      anon_sym_upper,
      anon_sym_constraint,
    ACTIONS(922), 19,
      sym_multi_line_comment,
      ts_builtin_sym_end,
      anon_sym_SLASH_DASH,
      anon_sym_RBRACE,
      anon_sym_LPAREN,
      anon_sym_DQUOTE,
      aux_sym__raw_string_token3,
      anon_sym_PLUS,
      anon_sym_DASH,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
      anon_sym_SLASH_SLASH,
  [13974] = 2,
    ACTIONS(928), 11,
      sym__normal_bare_identifier,
      aux_sym__raw_string_token1,
      anon_sym_expression,
      anon_sym_minimize,
      anon_sym_maximize,
      anon_sym_expr,
      anon_sym_filter,
      anon_sym_if,
      anon_sym_lower,
      anon_sym_upper,
      anon_sym_constraint,
    ACTIONS(926), 19,
      sym_multi_line_comment,
      ts_builtin_sym_end,
      anon_sym_SLASH_DASH,
      anon_sym_RBRACE,
      anon_sym_LPAREN,
      anon_sym_DQUOTE,
      aux_sym__raw_string_token3,
      anon_sym_PLUS,
      anon_sym_DASH,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
      anon_sym_SLASH_SLASH,
  [14009] = 2,
    ACTIONS(932), 11,
      sym__normal_bare_identifier,
      aux_sym__raw_string_token1,
      anon_sym_expression,
      anon_sym_minimize,
      anon_sym_maximize,
      anon_sym_expr,
      anon_sym_filter,
      anon_sym_if,
      anon_sym_lower,
      anon_sym_upper,
      anon_sym_constraint,
    ACTIONS(930), 19,
      sym_multi_line_comment,
      ts_builtin_sym_end,
      anon_sym_SLASH_DASH,
      anon_sym_RBRACE,
      anon_sym_LPAREN,
      anon_sym_DQUOTE,
      aux_sym__raw_string_token3,
      anon_sym_PLUS,
      anon_sym_DASH,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
      anon_sym_SLASH_SLASH,
  [14044] = 2,
    ACTIONS(936), 11,
      sym__normal_bare_identifier,
      aux_sym__raw_string_token1,
      anon_sym_expression,
      anon_sym_minimize,
      anon_sym_maximize,
      anon_sym_expr,
      anon_sym_filter,
      anon_sym_if,
      anon_sym_lower,
      anon_sym_upper,
      anon_sym_constraint,
    ACTIONS(934), 19,
      sym_multi_line_comment,
      ts_builtin_sym_end,
      anon_sym_SLASH_DASH,
      anon_sym_RBRACE,
      anon_sym_LPAREN,
      anon_sym_DQUOTE,
      aux_sym__raw_string_token3,
      anon_sym_PLUS,
      anon_sym_DASH,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
      anon_sym_SLASH_SLASH,
  [14079] = 2,
    ACTIONS(940), 11,
      sym__normal_bare_identifier,
      aux_sym__raw_string_token1,
      anon_sym_expression,
      anon_sym_minimize,
      anon_sym_maximize,
      anon_sym_expr,
      anon_sym_filter,
      anon_sym_if,
      anon_sym_lower,
      anon_sym_upper,
      anon_sym_constraint,
    ACTIONS(938), 19,
      sym_multi_line_comment,
      ts_builtin_sym_end,
      anon_sym_SLASH_DASH,
      anon_sym_RBRACE,
      anon_sym_LPAREN,
      anon_sym_DQUOTE,
      aux_sym__raw_string_token3,
      anon_sym_PLUS,
      anon_sym_DASH,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
      anon_sym_SLASH_SLASH,
  [14114] = 2,
    ACTIONS(944), 11,
      sym__normal_bare_identifier,
      aux_sym__raw_string_token1,
      anon_sym_expression,
      anon_sym_minimize,
      anon_sym_maximize,
      anon_sym_expr,
      anon_sym_filter,
      anon_sym_if,
      anon_sym_lower,
      anon_sym_upper,
      anon_sym_constraint,
    ACTIONS(942), 19,
      sym_multi_line_comment,
      ts_builtin_sym_end,
      anon_sym_SLASH_DASH,
      anon_sym_RBRACE,
      anon_sym_LPAREN,
      anon_sym_DQUOTE,
      aux_sym__raw_string_token3,
      anon_sym_PLUS,
      anon_sym_DASH,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
      anon_sym_SLASH_SLASH,
  [14149] = 2,
    ACTIONS(948), 11,
      sym__normal_bare_identifier,
      aux_sym__raw_string_token1,
      anon_sym_expression,
      anon_sym_minimize,
      anon_sym_maximize,
      anon_sym_expr,
      anon_sym_filter,
      anon_sym_if,
      anon_sym_lower,
      anon_sym_upper,
      anon_sym_constraint,
    ACTIONS(946), 19,
      sym_multi_line_comment,
      ts_builtin_sym_end,
      anon_sym_SLASH_DASH,
      anon_sym_RBRACE,
      anon_sym_LPAREN,
      anon_sym_DQUOTE,
      aux_sym__raw_string_token3,
      anon_sym_PLUS,
      anon_sym_DASH,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
      anon_sym_SLASH_SLASH,
  [14184] = 2,
    ACTIONS(952), 11,
      sym__normal_bare_identifier,
      aux_sym__raw_string_token1,
      anon_sym_expression,
      anon_sym_minimize,
      anon_sym_maximize,
      anon_sym_expr,
      anon_sym_filter,
      anon_sym_if,
      anon_sym_lower,
      anon_sym_upper,
      anon_sym_constraint,
    ACTIONS(950), 19,
      sym_multi_line_comment,
      ts_builtin_sym_end,
      anon_sym_SLASH_DASH,
      anon_sym_RBRACE,
      anon_sym_LPAREN,
      anon_sym_DQUOTE,
      aux_sym__raw_string_token3,
      anon_sym_PLUS,
      anon_sym_DASH,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
      anon_sym_SLASH_SLASH,
  [14219] = 2,
    ACTIONS(956), 11,
      sym__normal_bare_identifier,
      aux_sym__raw_string_token1,
      anon_sym_expression,
      anon_sym_minimize,
      anon_sym_maximize,
      anon_sym_expr,
      anon_sym_filter,
      anon_sym_if,
      anon_sym_lower,
      anon_sym_upper,
      anon_sym_constraint,
    ACTIONS(954), 19,
      sym_multi_line_comment,
      ts_builtin_sym_end,
      anon_sym_SLASH_DASH,
      anon_sym_RBRACE,
      anon_sym_LPAREN,
      anon_sym_DQUOTE,
      aux_sym__raw_string_token3,
      anon_sym_PLUS,
      anon_sym_DASH,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
      anon_sym_SLASH_SLASH,
  [14254] = 2,
    ACTIONS(960), 11,
      sym__normal_bare_identifier,
      aux_sym__raw_string_token1,
      anon_sym_expression,
      anon_sym_minimize,
      anon_sym_maximize,
      anon_sym_expr,
      anon_sym_filter,
      anon_sym_if,
      anon_sym_lower,
      anon_sym_upper,
      anon_sym_constraint,
    ACTIONS(958), 19,
      sym_multi_line_comment,
      ts_builtin_sym_end,
      anon_sym_SLASH_DASH,
      anon_sym_RBRACE,
      anon_sym_LPAREN,
      anon_sym_DQUOTE,
      aux_sym__raw_string_token3,
      anon_sym_PLUS,
      anon_sym_DASH,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
      anon_sym_SLASH_SLASH,
  [14289] = 2,
    ACTIONS(964), 11,
      sym__normal_bare_identifier,
      aux_sym__raw_string_token1,
      anon_sym_expression,
      anon_sym_minimize,
      anon_sym_maximize,
      anon_sym_expr,
      anon_sym_filter,
      anon_sym_if,
      anon_sym_lower,
      anon_sym_upper,
      anon_sym_constraint,
    ACTIONS(962), 19,
      sym_multi_line_comment,
      ts_builtin_sym_end,
      anon_sym_SLASH_DASH,
      anon_sym_RBRACE,
      anon_sym_LPAREN,
      anon_sym_DQUOTE,
      aux_sym__raw_string_token3,
      anon_sym_PLUS,
      anon_sym_DASH,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
      anon_sym_SLASH_SLASH,
  [14324] = 2,
    ACTIONS(968), 11,
      sym__normal_bare_identifier,
      aux_sym__raw_string_token1,
      anon_sym_expression,
      anon_sym_minimize,
      anon_sym_maximize,
      anon_sym_expr,
      anon_sym_filter,
      anon_sym_if,
      anon_sym_lower,
      anon_sym_upper,
      anon_sym_constraint,
    ACTIONS(966), 19,
      sym_multi_line_comment,
      ts_builtin_sym_end,
      anon_sym_SLASH_DASH,
      anon_sym_RBRACE,
      anon_sym_LPAREN,
      anon_sym_DQUOTE,
      aux_sym__raw_string_token3,
      anon_sym_PLUS,
      anon_sym_DASH,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
      anon_sym_SLASH_SLASH,
  [14359] = 2,
    ACTIONS(972), 11,
      sym__normal_bare_identifier,
      aux_sym__raw_string_token1,
      anon_sym_expression,
      anon_sym_minimize,
      anon_sym_maximize,
      anon_sym_expr,
      anon_sym_filter,
      anon_sym_if,
      anon_sym_lower,
      anon_sym_upper,
      anon_sym_constraint,
    ACTIONS(970), 19,
      sym_multi_line_comment,
      ts_builtin_sym_end,
      anon_sym_SLASH_DASH,
      anon_sym_RBRACE,
      anon_sym_LPAREN,
      anon_sym_DQUOTE,
      aux_sym__raw_string_token3,
      anon_sym_PLUS,
      anon_sym_DASH,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
      anon_sym_SLASH_SLASH,
  [14394] = 2,
    ACTIONS(976), 11,
      sym__normal_bare_identifier,
      aux_sym__raw_string_token1,
      anon_sym_expression,
      anon_sym_minimize,
      anon_sym_maximize,
      anon_sym_expr,
      anon_sym_filter,
      anon_sym_if,
      anon_sym_lower,
      anon_sym_upper,
      anon_sym_constraint,
    ACTIONS(974), 19,
      sym_multi_line_comment,
      ts_builtin_sym_end,
      anon_sym_SLASH_DASH,
      anon_sym_RBRACE,
      anon_sym_LPAREN,
      anon_sym_DQUOTE,
      aux_sym__raw_string_token3,
      anon_sym_PLUS,
      anon_sym_DASH,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
      anon_sym_SLASH_SLASH,
  [14429] = 2,
    ACTIONS(980), 11,
      sym__normal_bare_identifier,
      aux_sym__raw_string_token1,
      anon_sym_expression,
      anon_sym_minimize,
      anon_sym_maximize,
      anon_sym_expr,
      anon_sym_filter,
      anon_sym_if,
      anon_sym_lower,
      anon_sym_upper,
      anon_sym_constraint,
    ACTIONS(978), 19,
      sym_multi_line_comment,
      ts_builtin_sym_end,
      anon_sym_SLASH_DASH,
      anon_sym_RBRACE,
      anon_sym_LPAREN,
      anon_sym_DQUOTE,
      aux_sym__raw_string_token3,
      anon_sym_PLUS,
      anon_sym_DASH,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
      anon_sym_SLASH_SLASH,
  [14464] = 2,
    ACTIONS(984), 11,
      sym__normal_bare_identifier,
      aux_sym__raw_string_token1,
      anon_sym_expression,
      anon_sym_minimize,
      anon_sym_maximize,
      anon_sym_expr,
      anon_sym_filter,
      anon_sym_if,
      anon_sym_lower,
      anon_sym_upper,
      anon_sym_constraint,
    ACTIONS(982), 19,
      sym_multi_line_comment,
      ts_builtin_sym_end,
      anon_sym_SLASH_DASH,
      anon_sym_RBRACE,
      anon_sym_LPAREN,
      anon_sym_DQUOTE,
      aux_sym__raw_string_token3,
      anon_sym_PLUS,
      anon_sym_DASH,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
      anon_sym_SLASH_SLASH,
  [14499] = 2,
    ACTIONS(988), 11,
      sym__normal_bare_identifier,
      aux_sym__raw_string_token1,
      anon_sym_expression,
      anon_sym_minimize,
      anon_sym_maximize,
      anon_sym_expr,
      anon_sym_filter,
      anon_sym_if,
      anon_sym_lower,
      anon_sym_upper,
      anon_sym_constraint,
    ACTIONS(986), 19,
      sym_multi_line_comment,
      ts_builtin_sym_end,
      anon_sym_SLASH_DASH,
      anon_sym_RBRACE,
      anon_sym_LPAREN,
      anon_sym_DQUOTE,
      aux_sym__raw_string_token3,
      anon_sym_PLUS,
      anon_sym_DASH,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
      anon_sym_SLASH_SLASH,
  [14534] = 2,
    ACTIONS(992), 11,
      sym__normal_bare_identifier,
      aux_sym__raw_string_token1,
      anon_sym_expression,
      anon_sym_minimize,
      anon_sym_maximize,
      anon_sym_expr,
      anon_sym_filter,
      anon_sym_if,
      anon_sym_lower,
      anon_sym_upper,
      anon_sym_constraint,
    ACTIONS(990), 19,
      sym_multi_line_comment,
      ts_builtin_sym_end,
      anon_sym_SLASH_DASH,
      anon_sym_RBRACE,
      anon_sym_LPAREN,
      anon_sym_DQUOTE,
      aux_sym__raw_string_token3,
      anon_sym_PLUS,
      anon_sym_DASH,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
      anon_sym_SLASH_SLASH,
  [14569] = 2,
    ACTIONS(996), 11,
      sym__normal_bare_identifier,
      aux_sym__raw_string_token1,
      anon_sym_expression,
      anon_sym_minimize,
      anon_sym_maximize,
      anon_sym_expr,
      anon_sym_filter,
      anon_sym_if,
      anon_sym_lower,
      anon_sym_upper,
      anon_sym_constraint,
    ACTIONS(994), 19,
      sym_multi_line_comment,
      ts_builtin_sym_end,
      anon_sym_SLASH_DASH,
      anon_sym_RBRACE,
      anon_sym_LPAREN,
      anon_sym_DQUOTE,
      aux_sym__raw_string_token3,
      anon_sym_PLUS,
      anon_sym_DASH,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
      anon_sym_SLASH_SLASH,
  [14604] = 2,
    ACTIONS(1000), 11,
      sym__normal_bare_identifier,
      aux_sym__raw_string_token1,
      anon_sym_expression,
      anon_sym_minimize,
      anon_sym_maximize,
      anon_sym_expr,
      anon_sym_filter,
      anon_sym_if,
      anon_sym_lower,
      anon_sym_upper,
      anon_sym_constraint,
    ACTIONS(998), 19,
      sym_multi_line_comment,
      ts_builtin_sym_end,
      anon_sym_SLASH_DASH,
      anon_sym_RBRACE,
      anon_sym_LPAREN,
      anon_sym_DQUOTE,
      aux_sym__raw_string_token3,
      anon_sym_PLUS,
      anon_sym_DASH,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
      anon_sym_SLASH_SLASH,
  [14639] = 2,
    ACTIONS(1004), 11,
      sym__normal_bare_identifier,
      aux_sym__raw_string_token1,
      anon_sym_expression,
      anon_sym_minimize,
      anon_sym_maximize,
      anon_sym_expr,
      anon_sym_filter,
      anon_sym_if,
      anon_sym_lower,
      anon_sym_upper,
      anon_sym_constraint,
    ACTIONS(1002), 19,
      sym_multi_line_comment,
      ts_builtin_sym_end,
      anon_sym_SLASH_DASH,
      anon_sym_RBRACE,
      anon_sym_LPAREN,
      anon_sym_DQUOTE,
      aux_sym__raw_string_token3,
      anon_sym_PLUS,
      anon_sym_DASH,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
      anon_sym_SLASH_SLASH,
  [14674] = 2,
    ACTIONS(1008), 11,
      sym__normal_bare_identifier,
      aux_sym__raw_string_token1,
      anon_sym_expression,
      anon_sym_minimize,
      anon_sym_maximize,
      anon_sym_expr,
      anon_sym_filter,
      anon_sym_if,
      anon_sym_lower,
      anon_sym_upper,
      anon_sym_constraint,
    ACTIONS(1006), 19,
      sym_multi_line_comment,
      ts_builtin_sym_end,
      anon_sym_SLASH_DASH,
      anon_sym_RBRACE,
      anon_sym_LPAREN,
      anon_sym_DQUOTE,
      aux_sym__raw_string_token3,
      anon_sym_PLUS,
      anon_sym_DASH,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
      anon_sym_SLASH_SLASH,
  [14709] = 2,
    ACTIONS(1012), 11,
      sym__normal_bare_identifier,
      aux_sym__raw_string_token1,
      anon_sym_expression,
      anon_sym_minimize,
      anon_sym_maximize,
      anon_sym_expr,
      anon_sym_filter,
      anon_sym_if,
      anon_sym_lower,
      anon_sym_upper,
      anon_sym_constraint,
    ACTIONS(1010), 19,
      sym_multi_line_comment,
      ts_builtin_sym_end,
      anon_sym_SLASH_DASH,
      anon_sym_RBRACE,
      anon_sym_LPAREN,
      anon_sym_DQUOTE,
      aux_sym__raw_string_token3,
      anon_sym_PLUS,
      anon_sym_DASH,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
      anon_sym_SLASH_SLASH,
  [14744] = 2,
    ACTIONS(1016), 11,
      sym__normal_bare_identifier,
      aux_sym__raw_string_token1,
      anon_sym_expression,
      anon_sym_minimize,
      anon_sym_maximize,
      anon_sym_expr,
      anon_sym_filter,
      anon_sym_if,
      anon_sym_lower,
      anon_sym_upper,
      anon_sym_constraint,
    ACTIONS(1014), 19,
      sym_multi_line_comment,
      ts_builtin_sym_end,
      anon_sym_SLASH_DASH,
      anon_sym_RBRACE,
      anon_sym_LPAREN,
      anon_sym_DQUOTE,
      aux_sym__raw_string_token3,
      anon_sym_PLUS,
      anon_sym_DASH,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
      anon_sym_SLASH_SLASH,
  [14779] = 2,
    ACTIONS(1020), 11,
      sym__normal_bare_identifier,
      aux_sym__raw_string_token1,
      anon_sym_expression,
      anon_sym_minimize,
      anon_sym_maximize,
      anon_sym_expr,
      anon_sym_filter,
      anon_sym_if,
      anon_sym_lower,
      anon_sym_upper,
      anon_sym_constraint,
    ACTIONS(1018), 19,
      sym_multi_line_comment,
      ts_builtin_sym_end,
      anon_sym_SLASH_DASH,
      anon_sym_RBRACE,
      anon_sym_LPAREN,
      anon_sym_DQUOTE,
      aux_sym__raw_string_token3,
      anon_sym_PLUS,
      anon_sym_DASH,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
      anon_sym_SLASH_SLASH,
  [14814] = 2,
    ACTIONS(1024), 11,
      sym__normal_bare_identifier,
      aux_sym__raw_string_token1,
      anon_sym_expression,
      anon_sym_minimize,
      anon_sym_maximize,
      anon_sym_expr,
      anon_sym_filter,
      anon_sym_if,
      anon_sym_lower,
      anon_sym_upper,
      anon_sym_constraint,
    ACTIONS(1022), 19,
      sym_multi_line_comment,
      ts_builtin_sym_end,
      anon_sym_SLASH_DASH,
      anon_sym_RBRACE,
      anon_sym_LPAREN,
      anon_sym_DQUOTE,
      aux_sym__raw_string_token3,
      anon_sym_PLUS,
      anon_sym_DASH,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
      anon_sym_SLASH_SLASH,
  [14849] = 2,
    ACTIONS(1028), 11,
      sym__normal_bare_identifier,
      aux_sym__raw_string_token1,
      anon_sym_expression,
      anon_sym_minimize,
      anon_sym_maximize,
      anon_sym_expr,
      anon_sym_filter,
      anon_sym_if,
      anon_sym_lower,
      anon_sym_upper,
      anon_sym_constraint,
    ACTIONS(1026), 19,
      sym_multi_line_comment,
      ts_builtin_sym_end,
      anon_sym_SLASH_DASH,
      anon_sym_RBRACE,
      anon_sym_LPAREN,
      anon_sym_DQUOTE,
      aux_sym__raw_string_token3,
      anon_sym_PLUS,
      anon_sym_DASH,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
      anon_sym_SLASH_SLASH,
  [14884] = 2,
    ACTIONS(1032), 11,
      sym__normal_bare_identifier,
      aux_sym__raw_string_token1,
      anon_sym_expression,
      anon_sym_minimize,
      anon_sym_maximize,
      anon_sym_expr,
      anon_sym_filter,
      anon_sym_if,
      anon_sym_lower,
      anon_sym_upper,
      anon_sym_constraint,
    ACTIONS(1030), 19,
      sym_multi_line_comment,
      ts_builtin_sym_end,
      anon_sym_SLASH_DASH,
      anon_sym_RBRACE,
      anon_sym_LPAREN,
      anon_sym_DQUOTE,
      aux_sym__raw_string_token3,
      anon_sym_PLUS,
      anon_sym_DASH,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
      anon_sym_SLASH_SLASH,
  [14919] = 2,
    ACTIONS(1036), 11,
      sym__normal_bare_identifier,
      aux_sym__raw_string_token1,
      anon_sym_expression,
      anon_sym_minimize,
      anon_sym_maximize,
      anon_sym_expr,
      anon_sym_filter,
      anon_sym_if,
      anon_sym_lower,
      anon_sym_upper,
      anon_sym_constraint,
    ACTIONS(1034), 19,
      sym_multi_line_comment,
      ts_builtin_sym_end,
      anon_sym_SLASH_DASH,
      anon_sym_RBRACE,
      anon_sym_LPAREN,
      anon_sym_DQUOTE,
      aux_sym__raw_string_token3,
      anon_sym_PLUS,
      anon_sym_DASH,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
      anon_sym_SLASH_SLASH,
  [14954] = 2,
    ACTIONS(390), 14,
      sym_multi_line_comment,
      anon_sym_SLASH_DASH,
      anon_sym_LBRACE,
      anon_sym_LPAREN,
      anon_sym_DQUOTE,
      aux_sym__raw_string_token3,
      anon_sym_PLUS,
      anon_sym_DASH,
      anon_sym_0x,
      anon_sym_0o,
      anon_sym_0b,
      anon_sym_BSLASH,
      sym__bom,
      sym__unicode_space,
    ACTIONS(388), 15,
      sym__normal_bare_identifier,
      anon_sym_null,
      aux_sym__raw_string_token1,
      sym__digit,
      anon_sym_true,
      anon_sym_false,
      anon_sym_expression,
      anon_sym_minimize,
      anon_sym_maximize,
      anon_sym_expr,
      anon_sym_filter,
      anon_sym_if,
      anon_sym_lower,
      anon_sym_upper,
      anon_sym_constraint,
  [14988] = 2,
    ACTIONS(403), 14,
      sym_multi_line_comment,
      anon_sym_SLASH_DASH,
      anon_sym_LBRACE,
      anon_sym_LPAREN,
      anon_sym_DQUOTE,
      aux_sym__raw_string_token3,
      anon_sym_PLUS,
      anon_sym_DASH,
      anon_sym_0x,
      anon_sym_0o,
      anon_sym_0b,
      anon_sym_BSLASH,
      sym__bom,
      sym__unicode_space,
    ACTIONS(401), 15,
      sym__normal_bare_identifier,
      anon_sym_null,
      aux_sym__raw_string_token1,
      sym__digit,
      anon_sym_true,
      anon_sym_false,
      anon_sym_expression,
      anon_sym_minimize,
      anon_sym_maximize,
      anon_sym_expr,
      anon_sym_filter,
      anon_sym_if,
      anon_sym_lower,
      anon_sym_upper,
      anon_sym_constraint,
  [15022] = 2,
    ACTIONS(396), 14,
      sym_multi_line_comment,
      anon_sym_SLASH_DASH,
      anon_sym_LBRACE,
      anon_sym_LPAREN,
      anon_sym_DQUOTE,
      aux_sym__raw_string_token3,
      anon_sym_PLUS,
      anon_sym_DASH,
      anon_sym_0x,
      anon_sym_0o,
      anon_sym_0b,
      anon_sym_BSLASH,
      sym__bom,
      sym__unicode_space,
    ACTIONS(394), 15,
      sym__normal_bare_identifier,
      anon_sym_null,
      aux_sym__raw_string_token1,
      sym__digit,
      anon_sym_true,
      anon_sym_false,
      anon_sym_expression,
      anon_sym_minimize,
      anon_sym_maximize,
      anon_sym_expr,
      anon_sym_filter,
      anon_sym_if,
      anon_sym_lower,
      anon_sym_upper,
      anon_sym_constraint,
  [15056] = 13,
    ACTIONS(23), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(33), 1,
      anon_sym_LBRACE,
    ACTIONS(51), 1,
      anon_sym_BSLASH,
    ACTIONS(1038), 1,
      anon_sym_SLASH_DASH,
    STATE(10), 1,
      aux_sym__node_field_comment_repeat1,
    STATE(93), 1,
      sym__escline,
    STATE(102), 1,
      sym__node_space,
    STATE(278), 1,
      aux_sym_kdl_node_repeat1,
    STATE(84), 2,
      sym__ws,
      aux_sym__node_space_repeat1,
    STATE(453), 2,
      sym_node_children,
      sym_arco_constraint_math_children,
    ACTIONS(53), 3,
      sym_multi_line_comment,
      sym__bom,
      sym__unicode_space,
    STATE(182), 3,
      sym__node_terminator,
      sym__newline,
      sym_single_line_comment,
    ACTIONS(1040), 10,
      sym__eof,
      sym__implicit_terminator,
      anon_sym_SEMI,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
  [15111] = 13,
    ACTIONS(23), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(33), 1,
      anon_sym_LBRACE,
    ACTIONS(51), 1,
      anon_sym_BSLASH,
    ACTIONS(1038), 1,
      anon_sym_SLASH_DASH,
    STATE(5), 1,
      aux_sym__node_field_comment_repeat1,
    STATE(93), 1,
      sym__escline,
    STATE(102), 1,
      sym__node_space,
    STATE(416), 1,
      aux_sym_kdl_node_repeat1,
    STATE(84), 2,
      sym__ws,
      aux_sym__node_space_repeat1,
    STATE(437), 2,
      sym_node_children,
      sym_arco_constraint_math_children,
    ACTIONS(53), 3,
      sym_multi_line_comment,
      sym__bom,
      sym__unicode_space,
    STATE(134), 3,
      sym__node_terminator,
      sym__newline,
      sym_single_line_comment,
    ACTIONS(57), 10,
      sym__eof,
      sym__implicit_terminator,
      anon_sym_SEMI,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
  [15166] = 19,
    ACTIONS(3), 1,
      sym_multi_line_comment,
    ACTIONS(13), 1,
      anon_sym_DQUOTE,
    ACTIONS(15), 1,
      aux_sym__raw_string_token1,
    ACTIONS(17), 1,
      aux_sym__raw_string_token3,
    ACTIONS(37), 1,
      anon_sym_null,
    ACTIONS(39), 1,
      sym__digit,
    ACTIONS(43), 1,
      anon_sym_0x,
    ACTIONS(45), 1,
      anon_sym_0o,
    ACTIONS(47), 1,
      anon_sym_0b,
    ACTIONS(392), 1,
      sym__normal_bare_identifier,
    STATE(299), 1,
      sym__sign,
    STATE(467), 1,
      sym__integer,
    STATE(499), 1,
      sym__bare_identifier,
    STATE(507), 1,
      sym_boolean,
    ACTIONS(41), 2,
      anon_sym_PLUS,
      anon_sym_DASH,
    ACTIONS(49), 2,
      anon_sym_true,
      anon_sym_false,
    STATE(491), 2,
      sym__escaped_string,
      sym__raw_string,
    STATE(501), 4,
      sym__decimal,
      sym__hex,
      sym__octal,
      sym__binary,
    STATE(505), 4,
      sym_keyword,
      sym_string,
      sym_number,
      sym_bare_identifier,
  [15233] = 13,
    ACTIONS(23), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(33), 1,
      anon_sym_LBRACE,
    ACTIONS(51), 1,
      anon_sym_BSLASH,
    ACTIONS(1038), 1,
      anon_sym_SLASH_DASH,
    STATE(4), 1,
      aux_sym__node_field_comment_repeat1,
    STATE(93), 1,
      sym__escline,
    STATE(102), 1,
      sym__node_space,
    STATE(269), 1,
      aux_sym_kdl_node_repeat1,
    STATE(84), 2,
      sym__ws,
      aux_sym__node_space_repeat1,
    STATE(326), 2,
      sym_node_children,
      sym_arco_constraint_math_children,
    ACTIONS(53), 3,
      sym_multi_line_comment,
      sym__bom,
      sym__unicode_space,
    STATE(240), 3,
      sym__node_terminator,
      sym__newline,
      sym_single_line_comment,
    ACTIONS(1042), 10,
      sym__eof,
      sym__implicit_terminator,
      anon_sym_SEMI,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
  [15288] = 13,
    ACTIONS(23), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(33), 1,
      anon_sym_LBRACE,
    ACTIONS(51), 1,
      anon_sym_BSLASH,
    ACTIONS(1038), 1,
      anon_sym_SLASH_DASH,
    STATE(3), 1,
      aux_sym__node_field_comment_repeat1,
    STATE(93), 1,
      sym__escline,
    STATE(102), 1,
      sym__node_space,
    STATE(416), 1,
      aux_sym_kdl_node_repeat1,
    STATE(84), 2,
      sym__ws,
      aux_sym__node_space_repeat1,
    STATE(449), 2,
      sym_node_children,
      sym_arco_constraint_math_children,
    ACTIONS(53), 3,
      sym_multi_line_comment,
      sym__bom,
      sym__unicode_space,
    STATE(160), 3,
      sym__node_terminator,
      sym__newline,
      sym_single_line_comment,
    ACTIONS(71), 10,
      sym__eof,
      sym__implicit_terminator,
      anon_sym_SEMI,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
  [15343] = 13,
    ACTIONS(23), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(33), 1,
      anon_sym_LBRACE,
    ACTIONS(51), 1,
      anon_sym_BSLASH,
    ACTIONS(1038), 1,
      anon_sym_SLASH_DASH,
    STATE(9), 1,
      aux_sym__node_field_comment_repeat1,
    STATE(93), 1,
      sym__escline,
    STATE(102), 1,
      sym__node_space,
    STATE(416), 1,
      aux_sym_kdl_node_repeat1,
    STATE(84), 2,
      sym__ws,
      aux_sym__node_space_repeat1,
    STATE(426), 2,
      sym_node_children,
      sym_arco_constraint_math_children,
    ACTIONS(53), 3,
      sym_multi_line_comment,
      sym__bom,
      sym__unicode_space,
    STATE(201), 3,
      sym__node_terminator,
      sym__newline,
      sym_single_line_comment,
    ACTIONS(35), 10,
      sym__eof,
      sym__implicit_terminator,
      anon_sym_SEMI,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
  [15398] = 13,
    ACTIONS(23), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(33), 1,
      anon_sym_LBRACE,
    ACTIONS(51), 1,
      anon_sym_BSLASH,
    ACTIONS(1038), 1,
      anon_sym_SLASH_DASH,
    STATE(7), 1,
      aux_sym__node_field_comment_repeat1,
    STATE(93), 1,
      sym__escline,
    STATE(102), 1,
      sym__node_space,
    STATE(275), 1,
      aux_sym_kdl_node_repeat1,
    STATE(84), 2,
      sym__ws,
      aux_sym__node_space_repeat1,
    STATE(301), 2,
      sym_node_children,
      sym_arco_constraint_math_children,
    ACTIONS(53), 3,
      sym_multi_line_comment,
      sym__bom,
      sym__unicode_space,
    STATE(173), 3,
      sym__node_terminator,
      sym__newline,
      sym_single_line_comment,
    ACTIONS(1044), 10,
      sym__eof,
      sym__implicit_terminator,
      anon_sym_SEMI,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
  [15453] = 13,
    ACTIONS(23), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(33), 1,
      anon_sym_LBRACE,
    ACTIONS(51), 1,
      anon_sym_BSLASH,
    ACTIONS(1038), 1,
      anon_sym_SLASH_DASH,
    STATE(8), 1,
      aux_sym__node_field_comment_repeat1,
    STATE(93), 1,
      sym__escline,
    STATE(102), 1,
      sym__node_space,
    STATE(416), 1,
      aux_sym_kdl_node_repeat1,
    STATE(84), 2,
      sym__ws,
      aux_sym__node_space_repeat1,
    STATE(310), 2,
      sym_node_children,
      sym_arco_constraint_math_children,
    ACTIONS(53), 3,
      sym_multi_line_comment,
      sym__bom,
      sym__unicode_space,
    STATE(180), 3,
      sym__node_terminator,
      sym__newline,
      sym_single_line_comment,
    ACTIONS(63), 10,
      sym__eof,
      sym__implicit_terminator,
      anon_sym_SEMI,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
  [15508] = 13,
    ACTIONS(23), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(33), 1,
      anon_sym_LBRACE,
    ACTIONS(51), 1,
      anon_sym_BSLASH,
    ACTIONS(1038), 1,
      anon_sym_SLASH_DASH,
    STATE(2), 1,
      aux_sym__node_field_comment_repeat1,
    STATE(93), 1,
      sym__escline,
    STATE(102), 1,
      sym__node_space,
    STATE(273), 1,
      aux_sym_kdl_node_repeat1,
    STATE(84), 2,
      sym__ws,
      aux_sym__node_space_repeat1,
    STATE(321), 2,
      sym_node_children,
      sym_arco_constraint_math_children,
    ACTIONS(53), 3,
      sym_multi_line_comment,
      sym__bom,
      sym__unicode_space,
    STATE(174), 3,
      sym__node_terminator,
      sym__newline,
      sym_single_line_comment,
    ACTIONS(1046), 10,
      sym__eof,
      sym__implicit_terminator,
      anon_sym_SEMI,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
  [15563] = 13,
    ACTIONS(23), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(33), 1,
      anon_sym_LBRACE,
    ACTIONS(51), 1,
      anon_sym_BSLASH,
    ACTIONS(1038), 1,
      anon_sym_SLASH_DASH,
    STATE(11), 1,
      aux_sym__node_field_comment_repeat1,
    STATE(93), 1,
      sym__escline,
    STATE(102), 1,
      sym__node_space,
    STATE(272), 1,
      aux_sym_kdl_node_repeat1,
    STATE(84), 2,
      sym__ws,
      aux_sym__node_space_repeat1,
    STATE(334), 2,
      sym_node_children,
      sym_arco_constraint_math_children,
    ACTIONS(53), 3,
      sym_multi_line_comment,
      sym__bom,
      sym__unicode_space,
    STATE(203), 3,
      sym__node_terminator,
      sym__newline,
      sym_single_line_comment,
    ACTIONS(1048), 10,
      sym__eof,
      sym__implicit_terminator,
      anon_sym_SEMI,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
  [15618] = 13,
    ACTIONS(23), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(33), 1,
      anon_sym_LBRACE,
    ACTIONS(51), 1,
      anon_sym_BSLASH,
    ACTIONS(1038), 1,
      anon_sym_SLASH_DASH,
    STATE(6), 1,
      aux_sym__node_field_comment_repeat1,
    STATE(93), 1,
      sym__escline,
    STATE(102), 1,
      sym__node_space,
    STATE(416), 1,
      aux_sym_kdl_node_repeat1,
    STATE(84), 2,
      sym__ws,
      aux_sym__node_space_repeat1,
    STATE(316), 2,
      sym_node_children,
      sym_arco_constraint_math_children,
    ACTIONS(53), 3,
      sym_multi_line_comment,
      sym__bom,
      sym__unicode_space,
    STATE(120), 3,
      sym__node_terminator,
      sym__newline,
      sym_single_line_comment,
    ACTIONS(69), 10,
      sym__eof,
      sym__implicit_terminator,
      anon_sym_SEMI,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
  [15673] = 13,
    ACTIONS(23), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(51), 1,
      anon_sym_BSLASH,
    ACTIONS(75), 1,
      anon_sym_LBRACE,
    ACTIONS(1050), 1,
      anon_sym_SLASH_DASH,
    STATE(14), 1,
      aux_sym__node_field_comment_repeat1,
    STATE(93), 1,
      sym__escline,
    STATE(102), 1,
      sym__node_space,
    STATE(297), 1,
      aux_sym_kdl_node_repeat1,
    STATE(336), 1,
      sym_node_children,
    STATE(84), 2,
      sym__ws,
      aux_sym__node_space_repeat1,
    ACTIONS(53), 3,
      sym_multi_line_comment,
      sym__bom,
      sym__unicode_space,
    STATE(195), 3,
      sym__node_terminator,
      sym__newline,
      sym_single_line_comment,
    ACTIONS(109), 10,
      sym__eof,
      sym__implicit_terminator,
      anon_sym_SEMI,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
  [15727] = 13,
    ACTIONS(23), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(51), 1,
      anon_sym_BSLASH,
    ACTIONS(75), 1,
      anon_sym_LBRACE,
    ACTIONS(1050), 1,
      anon_sym_SLASH_DASH,
    STATE(28), 1,
      aux_sym__node_field_comment_repeat1,
    STATE(93), 1,
      sym__escline,
    STATE(102), 1,
      sym__node_space,
    STATE(289), 1,
      aux_sym_kdl_node_repeat1,
    STATE(458), 1,
      sym_node_children,
    STATE(84), 2,
      sym__ws,
      aux_sym__node_space_repeat1,
    ACTIONS(53), 3,
      sym_multi_line_comment,
      sym__bom,
      sym__unicode_space,
    STATE(166), 3,
      sym__node_terminator,
      sym__newline,
      sym_single_line_comment,
    ACTIONS(81), 10,
      sym__eof,
      sym__implicit_terminator,
      anon_sym_SEMI,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
  [15781] = 13,
    ACTIONS(23), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(51), 1,
      anon_sym_BSLASH,
    ACTIONS(85), 1,
      anon_sym_LBRACE,
    ACTIONS(1052), 1,
      anon_sym_SLASH_DASH,
    STATE(24), 1,
      aux_sym__node_field_comment_repeat1,
    STATE(93), 1,
      sym__escline,
    STATE(102), 1,
      sym__node_space,
    STATE(286), 1,
      aux_sym_kdl_node_repeat1,
    STATE(454), 1,
      sym_arco_pure_math_children,
    STATE(84), 2,
      sym__ws,
      aux_sym__node_space_repeat1,
    ACTIONS(53), 3,
      sym_multi_line_comment,
      sym__bom,
      sym__unicode_space,
    STATE(177), 3,
      sym__node_terminator,
      sym__newline,
      sym_single_line_comment,
    ACTIONS(1054), 10,
      sym__eof,
      sym__implicit_terminator,
      anon_sym_SEMI,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
  [15835] = 13,
    ACTIONS(23), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(51), 1,
      anon_sym_BSLASH,
    ACTIONS(85), 1,
      anon_sym_LBRACE,
    ACTIONS(1052), 1,
      anon_sym_SLASH_DASH,
    STATE(16), 1,
      aux_sym__node_field_comment_repeat1,
    STATE(93), 1,
      sym__escline,
    STATE(102), 1,
      sym__node_space,
    STATE(410), 1,
      sym_arco_pure_math_children,
    STATE(416), 1,
      aux_sym_kdl_node_repeat1,
    STATE(84), 2,
      sym__ws,
      aux_sym__node_space_repeat1,
    ACTIONS(53), 3,
      sym_multi_line_comment,
      sym__bom,
      sym__unicode_space,
    STATE(115), 3,
      sym__node_terminator,
      sym__newline,
      sym_single_line_comment,
    ACTIONS(87), 10,
      sym__eof,
      sym__implicit_terminator,
      anon_sym_SEMI,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
  [15889] = 13,
    ACTIONS(23), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(51), 1,
      anon_sym_BSLASH,
    ACTIONS(75), 1,
      anon_sym_LBRACE,
    ACTIONS(1050), 1,
      anon_sym_SLASH_DASH,
    STATE(13), 1,
      aux_sym__node_field_comment_repeat1,
    STATE(93), 1,
      sym__escline,
    STATE(102), 1,
      sym__node_space,
    STATE(287), 1,
      aux_sym_kdl_node_repeat1,
    STATE(451), 1,
      sym_node_children,
    STATE(84), 2,
      sym__ws,
      aux_sym__node_space_repeat1,
    ACTIONS(53), 3,
      sym_multi_line_comment,
      sym__bom,
      sym__unicode_space,
    STATE(186), 3,
      sym__node_terminator,
      sym__newline,
      sym_single_line_comment,
    ACTIONS(1056), 10,
      sym__eof,
      sym__implicit_terminator,
      anon_sym_SEMI,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
  [15943] = 13,
    ACTIONS(23), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(51), 1,
      anon_sym_BSLASH,
    ACTIONS(75), 1,
      anon_sym_LBRACE,
    ACTIONS(1050), 1,
      anon_sym_SLASH_DASH,
    STATE(14), 1,
      aux_sym__node_field_comment_repeat1,
    STATE(93), 1,
      sym__escline,
    STATE(102), 1,
      sym__node_space,
    STATE(336), 1,
      sym_node_children,
    STATE(416), 1,
      aux_sym_kdl_node_repeat1,
    STATE(84), 2,
      sym__ws,
      aux_sym__node_space_repeat1,
    ACTIONS(53), 3,
      sym_multi_line_comment,
      sym__bom,
      sym__unicode_space,
    STATE(195), 3,
      sym__node_terminator,
      sym__newline,
      sym_single_line_comment,
    ACTIONS(109), 10,
      sym__eof,
      sym__implicit_terminator,
      anon_sym_SEMI,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
  [15997] = 13,
    ACTIONS(23), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(51), 1,
      anon_sym_BSLASH,
    ACTIONS(85), 1,
      anon_sym_LBRACE,
    ACTIONS(1052), 1,
      anon_sym_SLASH_DASH,
    STATE(18), 1,
      aux_sym__node_field_comment_repeat1,
    STATE(93), 1,
      sym__escline,
    STATE(102), 1,
      sym__node_space,
    STATE(416), 1,
      aux_sym_kdl_node_repeat1,
    STATE(421), 1,
      sym_arco_pure_math_children,
    STATE(84), 2,
      sym__ws,
      aux_sym__node_space_repeat1,
    ACTIONS(53), 3,
      sym_multi_line_comment,
      sym__bom,
      sym__unicode_space,
    STATE(108), 3,
      sym__node_terminator,
      sym__newline,
      sym_single_line_comment,
    ACTIONS(97), 10,
      sym__eof,
      sym__implicit_terminator,
      anon_sym_SEMI,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
  [16051] = 13,
    ACTIONS(23), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(51), 1,
      anon_sym_BSLASH,
    ACTIONS(85), 1,
      anon_sym_LBRACE,
    ACTIONS(1052), 1,
      anon_sym_SLASH_DASH,
    STATE(19), 1,
      aux_sym__node_field_comment_repeat1,
    STATE(93), 1,
      sym__escline,
    STATE(102), 1,
      sym__node_space,
    STATE(313), 1,
      sym_arco_pure_math_children,
    STATE(416), 1,
      aux_sym_kdl_node_repeat1,
    STATE(84), 2,
      sym__ws,
      aux_sym__node_space_repeat1,
    ACTIONS(53), 3,
      sym_multi_line_comment,
      sym__bom,
      sym__unicode_space,
    STATE(171), 3,
      sym__node_terminator,
      sym__newline,
      sym_single_line_comment,
    ACTIONS(105), 10,
      sym__eof,
      sym__implicit_terminator,
      anon_sym_SEMI,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
  [16105] = 13,
    ACTIONS(23), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(51), 1,
      anon_sym_BSLASH,
    ACTIONS(75), 1,
      anon_sym_LBRACE,
    ACTIONS(1050), 1,
      anon_sym_SLASH_DASH,
    STATE(22), 1,
      aux_sym__node_field_comment_repeat1,
    STATE(93), 1,
      sym__escline,
    STATE(102), 1,
      sym__node_space,
    STATE(319), 1,
      sym_node_children,
    STATE(416), 1,
      aux_sym_kdl_node_repeat1,
    STATE(84), 2,
      sym__ws,
      aux_sym__node_space_repeat1,
    ACTIONS(53), 3,
      sym_multi_line_comment,
      sym__bom,
      sym__unicode_space,
    STATE(188), 3,
      sym__node_terminator,
      sym__newline,
      sym_single_line_comment,
    ACTIONS(79), 10,
      sym__eof,
      sym__implicit_terminator,
      anon_sym_SEMI,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
  [16159] = 13,
    ACTIONS(23), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(51), 1,
      anon_sym_BSLASH,
    ACTIONS(85), 1,
      anon_sym_LBRACE,
    ACTIONS(1052), 1,
      anon_sym_SLASH_DASH,
    STATE(23), 1,
      aux_sym__node_field_comment_repeat1,
    STATE(93), 1,
      sym__escline,
    STATE(102), 1,
      sym__node_space,
    STATE(298), 1,
      aux_sym_kdl_node_repeat1,
    STATE(331), 1,
      sym_arco_pure_math_children,
    STATE(84), 2,
      sym__ws,
      aux_sym__node_space_repeat1,
    ACTIONS(53), 3,
      sym_multi_line_comment,
      sym__bom,
      sym__unicode_space,
    STATE(225), 3,
      sym__node_terminator,
      sym__newline,
      sym_single_line_comment,
    ACTIONS(1058), 10,
      sym__eof,
      sym__implicit_terminator,
      anon_sym_SEMI,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
  [16213] = 13,
    ACTIONS(23), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(51), 1,
      anon_sym_BSLASH,
    ACTIONS(75), 1,
      anon_sym_LBRACE,
    ACTIONS(1050), 1,
      anon_sym_SLASH_DASH,
    STATE(17), 1,
      aux_sym__node_field_comment_repeat1,
    STATE(93), 1,
      sym__escline,
    STATE(102), 1,
      sym__node_space,
    STATE(304), 1,
      sym_node_children,
    STATE(416), 1,
      aux_sym_kdl_node_repeat1,
    STATE(84), 2,
      sym__ws,
      aux_sym__node_space_repeat1,
    ACTIONS(53), 3,
      sym_multi_line_comment,
      sym__bom,
      sym__unicode_space,
    STATE(190), 3,
      sym__node_terminator,
      sym__newline,
      sym_single_line_comment,
    ACTIONS(113), 10,
      sym__eof,
      sym__implicit_terminator,
      anon_sym_SEMI,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
  [16267] = 13,
    ACTIONS(23), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(51), 1,
      anon_sym_BSLASH,
    ACTIONS(85), 1,
      anon_sym_LBRACE,
    ACTIONS(1052), 1,
      anon_sym_SLASH_DASH,
    STATE(20), 1,
      aux_sym__node_field_comment_repeat1,
    STATE(93), 1,
      sym__escline,
    STATE(102), 1,
      sym__node_space,
    STATE(285), 1,
      aux_sym_kdl_node_repeat1,
    STATE(320), 1,
      sym_arco_pure_math_children,
    STATE(84), 2,
      sym__ws,
      aux_sym__node_space_repeat1,
    ACTIONS(53), 3,
      sym_multi_line_comment,
      sym__bom,
      sym__unicode_space,
    STATE(184), 3,
      sym__node_terminator,
      sym__newline,
      sym_single_line_comment,
    ACTIONS(1060), 10,
      sym__eof,
      sym__implicit_terminator,
      anon_sym_SEMI,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
  [16321] = 13,
    ACTIONS(23), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(51), 1,
      anon_sym_BSLASH,
    ACTIONS(85), 1,
      anon_sym_LBRACE,
    ACTIONS(1052), 1,
      anon_sym_SLASH_DASH,
    STATE(15), 1,
      aux_sym__node_field_comment_repeat1,
    STATE(93), 1,
      sym__escline,
    STATE(102), 1,
      sym__node_space,
    STATE(282), 1,
      aux_sym_kdl_node_repeat1,
    STATE(325), 1,
      sym_arco_pure_math_children,
    STATE(84), 2,
      sym__ws,
      aux_sym__node_space_repeat1,
    ACTIONS(53), 3,
      sym_multi_line_comment,
      sym__bom,
      sym__unicode_space,
    STATE(253), 3,
      sym__node_terminator,
      sym__newline,
      sym_single_line_comment,
    ACTIONS(1062), 10,
      sym__eof,
      sym__implicit_terminator,
      anon_sym_SEMI,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
  [16375] = 13,
    ACTIONS(23), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(51), 1,
      anon_sym_BSLASH,
    ACTIONS(85), 1,
      anon_sym_LBRACE,
    ACTIONS(1052), 1,
      anon_sym_SLASH_DASH,
    STATE(21), 1,
      aux_sym__node_field_comment_repeat1,
    STATE(93), 1,
      sym__escline,
    STATE(102), 1,
      sym__node_space,
    STATE(307), 1,
      sym_arco_pure_math_children,
    STATE(416), 1,
      aux_sym_kdl_node_repeat1,
    STATE(84), 2,
      sym__ws,
      aux_sym__node_space_repeat1,
    ACTIONS(53), 3,
      sym_multi_line_comment,
      sym__bom,
      sym__unicode_space,
    STATE(185), 3,
      sym__node_terminator,
      sym__newline,
      sym_single_line_comment,
    ACTIONS(111), 10,
      sym__eof,
      sym__implicit_terminator,
      anon_sym_SEMI,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
  [16429] = 13,
    ACTIONS(23), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(51), 1,
      anon_sym_BSLASH,
    ACTIONS(75), 1,
      anon_sym_LBRACE,
    ACTIONS(1050), 1,
      anon_sym_SLASH_DASH,
    STATE(22), 1,
      aux_sym__node_field_comment_repeat1,
    STATE(93), 1,
      sym__escline,
    STATE(102), 1,
      sym__node_space,
    STATE(296), 1,
      aux_sym_kdl_node_repeat1,
    STATE(319), 1,
      sym_node_children,
    STATE(84), 2,
      sym__ws,
      aux_sym__node_space_repeat1,
    ACTIONS(53), 3,
      sym_multi_line_comment,
      sym__bom,
      sym__unicode_space,
    STATE(188), 3,
      sym__node_terminator,
      sym__newline,
      sym_single_line_comment,
    ACTIONS(79), 10,
      sym__eof,
      sym__implicit_terminator,
      anon_sym_SEMI,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
  [16483] = 13,
    ACTIONS(23), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(51), 1,
      anon_sym_BSLASH,
    ACTIONS(75), 1,
      anon_sym_LBRACE,
    ACTIONS(1050), 1,
      anon_sym_SLASH_DASH,
    STATE(26), 1,
      aux_sym__node_field_comment_repeat1,
    STATE(93), 1,
      sym__escline,
    STATE(102), 1,
      sym__node_space,
    STATE(284), 1,
      aux_sym_kdl_node_repeat1,
    STATE(327), 1,
      sym_node_children,
    STATE(84), 2,
      sym__ws,
      aux_sym__node_space_repeat1,
    ACTIONS(53), 3,
      sym_multi_line_comment,
      sym__bom,
      sym__unicode_space,
    STATE(233), 3,
      sym__node_terminator,
      sym__newline,
      sym_single_line_comment,
    ACTIONS(1064), 10,
      sym__eof,
      sym__implicit_terminator,
      anon_sym_SEMI,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
  [16537] = 13,
    ACTIONS(23), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(51), 1,
      anon_sym_BSLASH,
    ACTIONS(85), 1,
      anon_sym_LBRACE,
    ACTIONS(1052), 1,
      anon_sym_SLASH_DASH,
    STATE(27), 1,
      aux_sym__node_field_comment_repeat1,
    STATE(93), 1,
      sym__escline,
    STATE(102), 1,
      sym__node_space,
    STATE(292), 1,
      aux_sym_kdl_node_repeat1,
    STATE(457), 1,
      sym_arco_pure_math_children,
    STATE(84), 2,
      sym__ws,
      aux_sym__node_space_repeat1,
    ACTIONS(53), 3,
      sym_multi_line_comment,
      sym__bom,
      sym__unicode_space,
    STATE(168), 3,
      sym__node_terminator,
      sym__newline,
      sym_single_line_comment,
    ACTIONS(1066), 10,
      sym__eof,
      sym__implicit_terminator,
      anon_sym_SEMI,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
  [16591] = 13,
    ACTIONS(23), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(51), 1,
      anon_sym_BSLASH,
    ACTIONS(75), 1,
      anon_sym_LBRACE,
    ACTIONS(1050), 1,
      anon_sym_SLASH_DASH,
    STATE(12), 1,
      aux_sym__node_field_comment_repeat1,
    STATE(93), 1,
      sym__escline,
    STATE(102), 1,
      sym__node_space,
    STATE(415), 1,
      sym_node_children,
    STATE(416), 1,
      aux_sym_kdl_node_repeat1,
    STATE(84), 2,
      sym__ws,
      aux_sym__node_space_repeat1,
    ACTIONS(53), 3,
      sym_multi_line_comment,
      sym__bom,
      sym__unicode_space,
    STATE(207), 3,
      sym__node_terminator,
      sym__newline,
      sym_single_line_comment,
    ACTIONS(101), 10,
      sym__eof,
      sym__implicit_terminator,
      anon_sym_SEMI,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
  [16645] = 13,
    ACTIONS(23), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(51), 1,
      anon_sym_BSLASH,
    ACTIONS(75), 1,
      anon_sym_LBRACE,
    ACTIONS(1050), 1,
      anon_sym_SLASH_DASH,
    STATE(28), 1,
      aux_sym__node_field_comment_repeat1,
    STATE(93), 1,
      sym__escline,
    STATE(102), 1,
      sym__node_space,
    STATE(416), 1,
      aux_sym_kdl_node_repeat1,
    STATE(458), 1,
      sym_node_children,
    STATE(84), 2,
      sym__ws,
      aux_sym__node_space_repeat1,
    ACTIONS(53), 3,
      sym_multi_line_comment,
      sym__bom,
      sym__unicode_space,
    STATE(166), 3,
      sym__node_terminator,
      sym__newline,
      sym_single_line_comment,
    ACTIONS(81), 10,
      sym__eof,
      sym__implicit_terminator,
      anon_sym_SEMI,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
  [16699] = 13,
    ACTIONS(23), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(51), 1,
      anon_sym_BSLASH,
    ACTIONS(85), 1,
      anon_sym_LBRACE,
    ACTIONS(1052), 1,
      anon_sym_SLASH_DASH,
    STATE(25), 1,
      aux_sym__node_field_comment_repeat1,
    STATE(93), 1,
      sym__escline,
    STATE(102), 1,
      sym__node_space,
    STATE(416), 1,
      aux_sym_kdl_node_repeat1,
    STATE(445), 1,
      sym_arco_pure_math_children,
    STATE(84), 2,
      sym__ws,
      aux_sym__node_space_repeat1,
    ACTIONS(53), 3,
      sym_multi_line_comment,
      sym__bom,
      sym__unicode_space,
    STATE(153), 3,
      sym__node_terminator,
      sym__newline,
      sym_single_line_comment,
    ACTIONS(103), 10,
      sym__eof,
      sym__implicit_terminator,
      anon_sym_SEMI,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
  [16753] = 7,
    ACTIONS(39), 1,
      sym__digit,
    ACTIONS(1070), 1,
      sym___identifier_char_no_digit,
    ACTIONS(1072), 1,
      anon_sym_0x,
    ACTIONS(1074), 1,
      anon_sym_0o,
    ACTIONS(1076), 1,
      anon_sym_0b,
    STATE(466), 1,
      sym__integer,
    ACTIONS(1068), 18,
      sym__eof,
      sym_multi_line_comment,
      sym__implicit_terminator,
      anon_sym_SLASH_DASH,
      anon_sym_LBRACE,
      anon_sym_SEMI,
      anon_sym_EQ,
      anon_sym_BSLASH,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
      anon_sym_SLASH_SLASH,
  [16792] = 9,
    ACTIONS(23), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(51), 1,
      anon_sym_BSLASH,
    STATE(81), 1,
      aux_sym__node_field_comment_repeat1,
    STATE(93), 1,
      sym__escline,
    STATE(102), 1,
      sym__node_space,
    STATE(84), 2,
      sym__ws,
      aux_sym__node_space_repeat1,
    ACTIONS(53), 3,
      sym_multi_line_comment,
      sym__bom,
      sym__unicode_space,
    STATE(255), 3,
      sym__node_terminator,
      sym__newline,
      sym_single_line_comment,
    ACTIONS(1078), 10,
      sym__eof,
      sym__implicit_terminator,
      anon_sym_SEMI,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
  [16834] = 9,
    ACTIONS(23), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(51), 1,
      anon_sym_BSLASH,
    STATE(93), 1,
      sym__escline,
    STATE(102), 1,
      sym__node_space,
    STATE(308), 1,
      aux_sym__node_field_comment_repeat1,
    STATE(309), 2,
      sym__ws,
      aux_sym__node_space_repeat1,
    ACTIONS(1082), 3,
      sym_multi_line_comment,
      sym__bom,
      sym__unicode_space,
    STATE(183), 3,
      sym__node_terminator,
      sym__newline,
      sym_single_line_comment,
    ACTIONS(1080), 10,
      sym__eof,
      sym__implicit_terminator,
      anon_sym_SEMI,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
  [16876] = 9,
    ACTIONS(23), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(51), 1,
      anon_sym_BSLASH,
    STATE(81), 1,
      aux_sym__node_field_comment_repeat1,
    STATE(93), 1,
      sym__escline,
    STATE(102), 1,
      sym__node_space,
    STATE(84), 2,
      sym__ws,
      aux_sym__node_space_repeat1,
    ACTIONS(53), 3,
      sym_multi_line_comment,
      sym__bom,
      sym__unicode_space,
    STATE(231), 3,
      sym__node_terminator,
      sym__newline,
      sym_single_line_comment,
    ACTIONS(1084), 10,
      sym__eof,
      sym__implicit_terminator,
      anon_sym_SEMI,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
  [16918] = 9,
    ACTIONS(23), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(51), 1,
      anon_sym_BSLASH,
    STATE(102), 1,
      sym__node_space,
    STATE(376), 1,
      aux_sym__node_field_comment_repeat1,
    STATE(513), 1,
      sym__escline,
    STATE(489), 2,
      sym__ws,
      aux_sym__node_space_repeat1,
    ACTIONS(1086), 3,
      sym_multi_line_comment,
      sym__bom,
      sym__unicode_space,
    STATE(231), 3,
      sym__node_terminator,
      sym__newline,
      sym_single_line_comment,
    ACTIONS(1084), 10,
      sym__eof,
      sym__implicit_terminator,
      anon_sym_SEMI,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
  [16960] = 9,
    ACTIONS(23), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(51), 1,
      anon_sym_BSLASH,
    STATE(93), 1,
      sym__escline,
    STATE(102), 1,
      sym__node_space,
    STATE(378), 1,
      aux_sym__node_field_comment_repeat1,
    STATE(379), 2,
      sym__ws,
      aux_sym__node_space_repeat1,
    ACTIONS(1090), 3,
      sym_multi_line_comment,
      sym__bom,
      sym__unicode_space,
    STATE(227), 3,
      sym__node_terminator,
      sym__newline,
      sym_single_line_comment,
    ACTIONS(1088), 10,
      sym__eof,
      sym__implicit_terminator,
      anon_sym_SEMI,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
  [17002] = 9,
    ACTIONS(23), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(51), 1,
      anon_sym_BSLASH,
    STATE(81), 1,
      aux_sym__node_field_comment_repeat1,
    STATE(93), 1,
      sym__escline,
    STATE(102), 1,
      sym__node_space,
    STATE(84), 2,
      sym__ws,
      aux_sym__node_space_repeat1,
    ACTIONS(53), 3,
      sym_multi_line_comment,
      sym__bom,
      sym__unicode_space,
    STATE(226), 3,
      sym__node_terminator,
      sym__newline,
      sym_single_line_comment,
    ACTIONS(1092), 10,
      sym__eof,
      sym__implicit_terminator,
      anon_sym_SEMI,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
  [17044] = 9,
    ACTIONS(23), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(51), 1,
      anon_sym_BSLASH,
    STATE(102), 1,
      sym__node_space,
    STATE(381), 1,
      aux_sym__node_field_comment_repeat1,
    STATE(513), 1,
      sym__escline,
    STATE(489), 2,
      sym__ws,
      aux_sym__node_space_repeat1,
    ACTIONS(1086), 3,
      sym_multi_line_comment,
      sym__bom,
      sym__unicode_space,
    STATE(226), 3,
      sym__node_terminator,
      sym__newline,
      sym_single_line_comment,
    ACTIONS(1092), 10,
      sym__eof,
      sym__implicit_terminator,
      anon_sym_SEMI,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
  [17086] = 9,
    ACTIONS(23), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(51), 1,
      anon_sym_BSLASH,
    STATE(93), 1,
      sym__escline,
    STATE(102), 1,
      sym__node_space,
    STATE(383), 1,
      aux_sym__node_field_comment_repeat1,
    STATE(384), 2,
      sym__ws,
      aux_sym__node_space_repeat1,
    ACTIONS(1096), 3,
      sym_multi_line_comment,
      sym__bom,
      sym__unicode_space,
    STATE(107), 3,
      sym__node_terminator,
      sym__newline,
      sym_single_line_comment,
    ACTIONS(1094), 10,
      sym__eof,
      sym__implicit_terminator,
      anon_sym_SEMI,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
  [17128] = 9,
    ACTIONS(23), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(51), 1,
      anon_sym_BSLASH,
    STATE(81), 1,
      aux_sym__node_field_comment_repeat1,
    STATE(93), 1,
      sym__escline,
    STATE(102), 1,
      sym__node_space,
    STATE(84), 2,
      sym__ws,
      aux_sym__node_space_repeat1,
    ACTIONS(53), 3,
      sym_multi_line_comment,
      sym__bom,
      sym__unicode_space,
    STATE(221), 3,
      sym__node_terminator,
      sym__newline,
      sym_single_line_comment,
    ACTIONS(1098), 10,
      sym__eof,
      sym__implicit_terminator,
      anon_sym_SEMI,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
  [17170] = 9,
    ACTIONS(23), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(51), 1,
      anon_sym_BSLASH,
    STATE(102), 1,
      sym__node_space,
    STATE(388), 1,
      aux_sym__node_field_comment_repeat1,
    STATE(513), 1,
      sym__escline,
    STATE(489), 2,
      sym__ws,
      aux_sym__node_space_repeat1,
    ACTIONS(1086), 3,
      sym_multi_line_comment,
      sym__bom,
      sym__unicode_space,
    STATE(221), 3,
      sym__node_terminator,
      sym__newline,
      sym_single_line_comment,
    ACTIONS(1098), 10,
      sym__eof,
      sym__implicit_terminator,
      anon_sym_SEMI,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
  [17212] = 9,
    ACTIONS(23), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(51), 1,
      anon_sym_BSLASH,
    STATE(93), 1,
      sym__escline,
    STATE(102), 1,
      sym__node_space,
    STATE(390), 1,
      aux_sym__node_field_comment_repeat1,
    STATE(391), 2,
      sym__ws,
      aux_sym__node_space_repeat1,
    ACTIONS(1102), 3,
      sym_multi_line_comment,
      sym__bom,
      sym__unicode_space,
    STATE(219), 3,
      sym__node_terminator,
      sym__newline,
      sym_single_line_comment,
    ACTIONS(1100), 10,
      sym__eof,
      sym__implicit_terminator,
      anon_sym_SEMI,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
  [17254] = 9,
    ACTIONS(23), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(51), 1,
      anon_sym_BSLASH,
    STATE(81), 1,
      aux_sym__node_field_comment_repeat1,
    STATE(93), 1,
      sym__escline,
    STATE(102), 1,
      sym__node_space,
    STATE(84), 2,
      sym__ws,
      aux_sym__node_space_repeat1,
    ACTIONS(53), 3,
      sym_multi_line_comment,
      sym__bom,
      sym__unicode_space,
    STATE(216), 3,
      sym__node_terminator,
      sym__newline,
      sym_single_line_comment,
    ACTIONS(1104), 10,
      sym__eof,
      sym__implicit_terminator,
      anon_sym_SEMI,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
  [17296] = 9,
    ACTIONS(23), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(51), 1,
      anon_sym_BSLASH,
    STATE(102), 1,
      sym__node_space,
    STATE(397), 1,
      aux_sym__node_field_comment_repeat1,
    STATE(513), 1,
      sym__escline,
    STATE(489), 2,
      sym__ws,
      aux_sym__node_space_repeat1,
    ACTIONS(1086), 3,
      sym_multi_line_comment,
      sym__bom,
      sym__unicode_space,
    STATE(216), 3,
      sym__node_terminator,
      sym__newline,
      sym_single_line_comment,
    ACTIONS(1104), 10,
      sym__eof,
      sym__implicit_terminator,
      anon_sym_SEMI,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
  [17338] = 9,
    ACTIONS(23), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(51), 1,
      anon_sym_BSLASH,
    STATE(93), 1,
      sym__escline,
    STATE(102), 1,
      sym__node_space,
    STATE(399), 1,
      aux_sym__node_field_comment_repeat1,
    STATE(400), 2,
      sym__ws,
      aux_sym__node_space_repeat1,
    ACTIONS(1108), 3,
      sym_multi_line_comment,
      sym__bom,
      sym__unicode_space,
    STATE(215), 3,
      sym__node_terminator,
      sym__newline,
      sym_single_line_comment,
    ACTIONS(1106), 10,
      sym__eof,
      sym__implicit_terminator,
      anon_sym_SEMI,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
  [17380] = 9,
    ACTIONS(23), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(51), 1,
      anon_sym_BSLASH,
    STATE(81), 1,
      aux_sym__node_field_comment_repeat1,
    STATE(93), 1,
      sym__escline,
    STATE(102), 1,
      sym__node_space,
    STATE(84), 2,
      sym__ws,
      aux_sym__node_space_repeat1,
    ACTIONS(53), 3,
      sym_multi_line_comment,
      sym__bom,
      sym__unicode_space,
    STATE(213), 3,
      sym__node_terminator,
      sym__newline,
      sym_single_line_comment,
    ACTIONS(1110), 10,
      sym__eof,
      sym__implicit_terminator,
      anon_sym_SEMI,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
  [17422] = 9,
    ACTIONS(23), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(51), 1,
      anon_sym_BSLASH,
    STATE(102), 1,
      sym__node_space,
    STATE(404), 1,
      aux_sym__node_field_comment_repeat1,
    STATE(513), 1,
      sym__escline,
    STATE(489), 2,
      sym__ws,
      aux_sym__node_space_repeat1,
    ACTIONS(1086), 3,
      sym_multi_line_comment,
      sym__bom,
      sym__unicode_space,
    STATE(213), 3,
      sym__node_terminator,
      sym__newline,
      sym_single_line_comment,
    ACTIONS(1110), 10,
      sym__eof,
      sym__implicit_terminator,
      anon_sym_SEMI,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
  [17464] = 9,
    ACTIONS(23), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(51), 1,
      anon_sym_BSLASH,
    STATE(93), 1,
      sym__escline,
    STATE(102), 1,
      sym__node_space,
    STATE(406), 1,
      aux_sym__node_field_comment_repeat1,
    STATE(407), 2,
      sym__ws,
      aux_sym__node_space_repeat1,
    ACTIONS(1114), 3,
      sym_multi_line_comment,
      sym__bom,
      sym__unicode_space,
    STATE(212), 3,
      sym__node_terminator,
      sym__newline,
      sym_single_line_comment,
    ACTIONS(1112), 10,
      sym__eof,
      sym__implicit_terminator,
      anon_sym_SEMI,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
  [17506] = 9,
    ACTIONS(23), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(51), 1,
      anon_sym_BSLASH,
    STATE(81), 1,
      aux_sym__node_field_comment_repeat1,
    STATE(93), 1,
      sym__escline,
    STATE(102), 1,
      sym__node_space,
    STATE(84), 2,
      sym__ws,
      aux_sym__node_space_repeat1,
    ACTIONS(53), 3,
      sym_multi_line_comment,
      sym__bom,
      sym__unicode_space,
    STATE(209), 3,
      sym__node_terminator,
      sym__newline,
      sym_single_line_comment,
    ACTIONS(1116), 10,
      sym__eof,
      sym__implicit_terminator,
      anon_sym_SEMI,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
  [17548] = 9,
    ACTIONS(23), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(51), 1,
      anon_sym_BSLASH,
    STATE(102), 1,
      sym__node_space,
    STATE(411), 1,
      aux_sym__node_field_comment_repeat1,
    STATE(513), 1,
      sym__escline,
    STATE(489), 2,
      sym__ws,
      aux_sym__node_space_repeat1,
    ACTIONS(1086), 3,
      sym_multi_line_comment,
      sym__bom,
      sym__unicode_space,
    STATE(209), 3,
      sym__node_terminator,
      sym__newline,
      sym_single_line_comment,
    ACTIONS(1116), 10,
      sym__eof,
      sym__implicit_terminator,
      anon_sym_SEMI,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
  [17590] = 9,
    ACTIONS(23), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(51), 1,
      anon_sym_BSLASH,
    STATE(93), 1,
      sym__escline,
    STATE(102), 1,
      sym__node_space,
    STATE(413), 1,
      aux_sym__node_field_comment_repeat1,
    STATE(414), 2,
      sym__ws,
      aux_sym__node_space_repeat1,
    ACTIONS(1120), 3,
      sym_multi_line_comment,
      sym__bom,
      sym__unicode_space,
    STATE(208), 3,
      sym__node_terminator,
      sym__newline,
      sym_single_line_comment,
    ACTIONS(1118), 10,
      sym__eof,
      sym__implicit_terminator,
      anon_sym_SEMI,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
  [17632] = 9,
    ACTIONS(23), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(51), 1,
      anon_sym_BSLASH,
    STATE(93), 1,
      sym__escline,
    STATE(102), 1,
      sym__node_space,
    STATE(418), 1,
      aux_sym__node_field_comment_repeat1,
    STATE(419), 2,
      sym__ws,
      aux_sym__node_space_repeat1,
    ACTIONS(1124), 3,
      sym_multi_line_comment,
      sym__bom,
      sym__unicode_space,
    STATE(206), 3,
      sym__node_terminator,
      sym__newline,
      sym_single_line_comment,
    ACTIONS(1122), 10,
      sym__eof,
      sym__implicit_terminator,
      anon_sym_SEMI,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
  [17674] = 9,
    ACTIONS(23), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(51), 1,
      anon_sym_BSLASH,
    STATE(93), 1,
      sym__escline,
    STATE(102), 1,
      sym__node_space,
    STATE(424), 1,
      aux_sym__node_field_comment_repeat1,
    STATE(425), 2,
      sym__ws,
      aux_sym__node_space_repeat1,
    ACTIONS(1128), 3,
      sym_multi_line_comment,
      sym__bom,
      sym__unicode_space,
    STATE(202), 3,
      sym__node_terminator,
      sym__newline,
      sym_single_line_comment,
    ACTIONS(1126), 10,
      sym__eof,
      sym__implicit_terminator,
      anon_sym_SEMI,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
  [17716] = 9,
    ACTIONS(23), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(51), 1,
      anon_sym_BSLASH,
    STATE(81), 1,
      aux_sym__node_field_comment_repeat1,
    STATE(93), 1,
      sym__escline,
    STATE(102), 1,
      sym__node_space,
    STATE(84), 2,
      sym__ws,
      aux_sym__node_space_repeat1,
    ACTIONS(53), 3,
      sym_multi_line_comment,
      sym__bom,
      sym__unicode_space,
    STATE(187), 3,
      sym__node_terminator,
      sym__newline,
      sym_single_line_comment,
    ACTIONS(1130), 10,
      sym__eof,
      sym__implicit_terminator,
      anon_sym_SEMI,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
  [17758] = 9,
    ACTIONS(23), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(51), 1,
      anon_sym_BSLASH,
    STATE(81), 1,
      aux_sym__node_field_comment_repeat1,
    STATE(93), 1,
      sym__escline,
    STATE(102), 1,
      sym__node_space,
    STATE(84), 2,
      sym__ws,
      aux_sym__node_space_repeat1,
    ACTIONS(53), 3,
      sym_multi_line_comment,
      sym__bom,
      sym__unicode_space,
    STATE(191), 3,
      sym__node_terminator,
      sym__newline,
      sym_single_line_comment,
    ACTIONS(1132), 10,
      sym__eof,
      sym__implicit_terminator,
      anon_sym_SEMI,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
  [17800] = 9,
    ACTIONS(23), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(51), 1,
      anon_sym_BSLASH,
    STATE(81), 1,
      aux_sym__node_field_comment_repeat1,
    STATE(93), 1,
      sym__escline,
    STATE(102), 1,
      sym__node_space,
    STATE(84), 2,
      sym__ws,
      aux_sym__node_space_repeat1,
    ACTIONS(53), 3,
      sym_multi_line_comment,
      sym__bom,
      sym__unicode_space,
    STATE(192), 3,
      sym__node_terminator,
      sym__newline,
      sym_single_line_comment,
    ACTIONS(1134), 10,
      sym__eof,
      sym__implicit_terminator,
      anon_sym_SEMI,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
  [17842] = 9,
    ACTIONS(23), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(51), 1,
      anon_sym_BSLASH,
    STATE(93), 1,
      sym__escline,
    STATE(102), 1,
      sym__node_space,
    STATE(361), 1,
      aux_sym__node_field_comment_repeat1,
    STATE(374), 2,
      sym__ws,
      aux_sym__node_space_repeat1,
    ACTIONS(1138), 3,
      sym_multi_line_comment,
      sym__bom,
      sym__unicode_space,
    STATE(138), 3,
      sym__node_terminator,
      sym__newline,
      sym_single_line_comment,
    ACTIONS(1136), 10,
      sym__eof,
      sym__implicit_terminator,
      anon_sym_SEMI,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
  [17884] = 9,
    ACTIONS(23), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(51), 1,
      anon_sym_BSLASH,
    STATE(93), 1,
      sym__escline,
    STATE(102), 1,
      sym__node_space,
    STATE(428), 1,
      aux_sym__node_field_comment_repeat1,
    STATE(431), 2,
      sym__ws,
      aux_sym__node_space_repeat1,
    ACTIONS(1142), 3,
      sym_multi_line_comment,
      sym__bom,
      sym__unicode_space,
    STATE(130), 3,
      sym__node_terminator,
      sym__newline,
      sym_single_line_comment,
    ACTIONS(1140), 10,
      sym__eof,
      sym__implicit_terminator,
      anon_sym_SEMI,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
  [17926] = 9,
    ACTIONS(23), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(51), 1,
      anon_sym_BSLASH,
    STATE(93), 1,
      sym__escline,
    STATE(102), 1,
      sym__node_space,
    STATE(354), 1,
      aux_sym__node_field_comment_repeat1,
    STATE(439), 2,
      sym__ws,
      aux_sym__node_space_repeat1,
    ACTIONS(1146), 3,
      sym_multi_line_comment,
      sym__bom,
      sym__unicode_space,
    STATE(143), 3,
      sym__node_terminator,
      sym__newline,
      sym_single_line_comment,
    ACTIONS(1144), 10,
      sym__eof,
      sym__implicit_terminator,
      anon_sym_SEMI,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
  [17968] = 3,
    STATE(328), 1,
      aux_sym__integer_repeat1,
    ACTIONS(1150), 2,
      anon_sym__,
      sym__digit,
    ACTIONS(1148), 20,
      sym__eof,
      sym_multi_line_comment,
      sym__implicit_terminator,
      anon_sym_SLASH_DASH,
      anon_sym_LBRACE,
      anon_sym_SEMI,
      anon_sym_DOT,
      anon_sym_e,
      anon_sym_E,
      anon_sym_BSLASH,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
      anon_sym_SLASH_SLASH,
  [17998] = 9,
    ACTIONS(23), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(51), 1,
      anon_sym_BSLASH,
    STATE(102), 1,
      sym__node_space,
    STATE(322), 1,
      aux_sym__node_field_comment_repeat1,
    STATE(513), 1,
      sym__escline,
    STATE(489), 2,
      sym__ws,
      aux_sym__node_space_repeat1,
    ACTIONS(1086), 3,
      sym_multi_line_comment,
      sym__bom,
      sym__unicode_space,
    STATE(230), 3,
      sym__node_terminator,
      sym__newline,
      sym_single_line_comment,
    ACTIONS(1153), 10,
      sym__eof,
      sym__implicit_terminator,
      anon_sym_SEMI,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
  [18040] = 9,
    ACTIONS(23), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(51), 1,
      anon_sym_BSLASH,
    STATE(81), 1,
      aux_sym__node_field_comment_repeat1,
    STATE(93), 1,
      sym__escline,
    STATE(102), 1,
      sym__node_space,
    STATE(84), 2,
      sym__ws,
      aux_sym__node_space_repeat1,
    ACTIONS(53), 3,
      sym_multi_line_comment,
      sym__bom,
      sym__unicode_space,
    STATE(230), 3,
      sym__node_terminator,
      sym__newline,
      sym_single_line_comment,
    ACTIONS(1153), 10,
      sym__eof,
      sym__implicit_terminator,
      anon_sym_SEMI,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
  [18082] = 9,
    ACTIONS(23), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(51), 1,
      anon_sym_BSLASH,
    STATE(93), 1,
      sym__escline,
    STATE(102), 1,
      sym__node_space,
    STATE(300), 1,
      aux_sym__node_field_comment_repeat1,
    STATE(442), 2,
      sym__ws,
      aux_sym__node_space_repeat1,
    ACTIONS(1157), 3,
      sym_multi_line_comment,
      sym__bom,
      sym__unicode_space,
    STATE(150), 3,
      sym__node_terminator,
      sym__newline,
      sym_single_line_comment,
    ACTIONS(1155), 10,
      sym__eof,
      sym__implicit_terminator,
      anon_sym_SEMI,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
  [18124] = 9,
    ACTIONS(23), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(51), 1,
      anon_sym_BSLASH,
    STATE(102), 1,
      sym__node_space,
    STATE(323), 1,
      aux_sym__node_field_comment_repeat1,
    STATE(513), 1,
      sym__escline,
    STATE(489), 2,
      sym__ws,
      aux_sym__node_space_repeat1,
    ACTIONS(1086), 3,
      sym_multi_line_comment,
      sym__bom,
      sym__unicode_space,
    STATE(237), 3,
      sym__node_terminator,
      sym__newline,
      sym_single_line_comment,
    ACTIONS(1159), 10,
      sym__eof,
      sym__implicit_terminator,
      anon_sym_SEMI,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
  [18166] = 9,
    ACTIONS(23), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(51), 1,
      anon_sym_BSLASH,
    STATE(81), 1,
      aux_sym__node_field_comment_repeat1,
    STATE(93), 1,
      sym__escline,
    STATE(102), 1,
      sym__node_space,
    STATE(84), 2,
      sym__ws,
      aux_sym__node_space_repeat1,
    ACTIONS(53), 3,
      sym_multi_line_comment,
      sym__bom,
      sym__unicode_space,
    STATE(237), 3,
      sym__node_terminator,
      sym__newline,
      sym_single_line_comment,
    ACTIONS(1159), 10,
      sym__eof,
      sym__implicit_terminator,
      anon_sym_SEMI,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
  [18208] = 9,
    ACTIONS(23), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(51), 1,
      anon_sym_BSLASH,
    STATE(93), 1,
      sym__escline,
    STATE(102), 1,
      sym__node_space,
    STATE(446), 1,
      aux_sym__node_field_comment_repeat1,
    STATE(448), 2,
      sym__ws,
      aux_sym__node_space_repeat1,
    ACTIONS(1163), 3,
      sym_multi_line_comment,
      sym__bom,
      sym__unicode_space,
    STATE(156), 3,
      sym__node_terminator,
      sym__newline,
      sym_single_line_comment,
    ACTIONS(1161), 10,
      sym__eof,
      sym__implicit_terminator,
      anon_sym_SEMI,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
  [18250] = 9,
    ACTIONS(23), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(51), 1,
      anon_sym_BSLASH,
    STATE(81), 1,
      aux_sym__node_field_comment_repeat1,
    STATE(93), 1,
      sym__escline,
    STATE(102), 1,
      sym__node_space,
    STATE(84), 2,
      sym__ws,
      aux_sym__node_space_repeat1,
    ACTIONS(53), 3,
      sym_multi_line_comment,
      sym__bom,
      sym__unicode_space,
    STATE(249), 3,
      sym__node_terminator,
      sym__newline,
      sym_single_line_comment,
    ACTIONS(1165), 10,
      sym__eof,
      sym__implicit_terminator,
      anon_sym_SEMI,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
  [18292] = 9,
    ACTIONS(23), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(51), 1,
      anon_sym_BSLASH,
    STATE(93), 1,
      sym__escline,
    STATE(102), 1,
      sym__node_space,
    STATE(452), 1,
      aux_sym__node_field_comment_repeat1,
    STATE(460), 2,
      sym__ws,
      aux_sym__node_space_repeat1,
    ACTIONS(1169), 3,
      sym_multi_line_comment,
      sym__bom,
      sym__unicode_space,
    STATE(162), 3,
      sym__node_terminator,
      sym__newline,
      sym_single_line_comment,
    ACTIONS(1167), 10,
      sym__eof,
      sym__implicit_terminator,
      anon_sym_SEMI,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
  [18334] = 9,
    ACTIONS(23), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(51), 1,
      anon_sym_BSLASH,
    STATE(102), 1,
      sym__node_space,
    STATE(324), 1,
      aux_sym__node_field_comment_repeat1,
    STATE(513), 1,
      sym__escline,
    STATE(489), 2,
      sym__ws,
      aux_sym__node_space_repeat1,
    ACTIONS(1086), 3,
      sym_multi_line_comment,
      sym__bom,
      sym__unicode_space,
    STATE(263), 3,
      sym__node_terminator,
      sym__newline,
      sym_single_line_comment,
    ACTIONS(1171), 10,
      sym__eof,
      sym__implicit_terminator,
      anon_sym_SEMI,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
  [18376] = 9,
    ACTIONS(23), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(51), 1,
      anon_sym_BSLASH,
    STATE(81), 1,
      aux_sym__node_field_comment_repeat1,
    STATE(93), 1,
      sym__escline,
    STATE(102), 1,
      sym__node_space,
    STATE(84), 2,
      sym__ws,
      aux_sym__node_space_repeat1,
    ACTIONS(53), 3,
      sym_multi_line_comment,
      sym__bom,
      sym__unicode_space,
    STATE(263), 3,
      sym__node_terminator,
      sym__newline,
      sym_single_line_comment,
    ACTIONS(1171), 10,
      sym__eof,
      sym__implicit_terminator,
      anon_sym_SEMI,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
  [18418] = 3,
    ACTIONS(3), 1,
      sym_multi_line_comment,
    ACTIONS(1175), 7,
      anon_sym_DQUOTE,
      aux_sym__raw_string_token3,
      anon_sym_PLUS,
      anon_sym_DASH,
      anon_sym_0x,
      anon_sym_0o,
      anon_sym_0b,
    ACTIONS(1173), 15,
      sym__normal_bare_identifier,
      anon_sym_null,
      aux_sym__raw_string_token1,
      sym__digit,
      anon_sym_true,
      anon_sym_false,
      anon_sym_expression,
      anon_sym_minimize,
      anon_sym_maximize,
      anon_sym_expr,
      anon_sym_filter,
      anon_sym_if,
      anon_sym_lower,
      anon_sym_upper,
      anon_sym_constraint,
  [18448] = 9,
    ACTIONS(23), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(51), 1,
      anon_sym_BSLASH,
    STATE(81), 1,
      aux_sym__node_field_comment_repeat1,
    STATE(93), 1,
      sym__escline,
    STATE(102), 1,
      sym__node_space,
    STATE(84), 2,
      sym__ws,
      aux_sym__node_space_repeat1,
    ACTIONS(53), 3,
      sym_multi_line_comment,
      sym__bom,
      sym__unicode_space,
    STATE(261), 3,
      sym__node_terminator,
      sym__newline,
      sym_single_line_comment,
    ACTIONS(1177), 10,
      sym__eof,
      sym__implicit_terminator,
      anon_sym_SEMI,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
  [18490] = 9,
    ACTIONS(23), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(51), 1,
      anon_sym_BSLASH,
    STATE(81), 1,
      aux_sym__node_field_comment_repeat1,
    STATE(93), 1,
      sym__escline,
    STATE(102), 1,
      sym__node_space,
    STATE(84), 2,
      sym__ws,
      aux_sym__node_space_repeat1,
    ACTIONS(53), 3,
      sym_multi_line_comment,
      sym__bom,
      sym__unicode_space,
    STATE(197), 3,
      sym__node_terminator,
      sym__newline,
      sym_single_line_comment,
    ACTIONS(1179), 10,
      sym__eof,
      sym__implicit_terminator,
      anon_sym_SEMI,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
  [18532] = 9,
    ACTIONS(23), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(51), 1,
      anon_sym_BSLASH,
    STATE(81), 1,
      aux_sym__node_field_comment_repeat1,
    STATE(93), 1,
      sym__escline,
    STATE(102), 1,
      sym__node_space,
    STATE(84), 2,
      sym__ws,
      aux_sym__node_space_repeat1,
    ACTIONS(53), 3,
      sym_multi_line_comment,
      sym__bom,
      sym__unicode_space,
    STATE(196), 3,
      sym__node_terminator,
      sym__newline,
      sym_single_line_comment,
    ACTIONS(1181), 10,
      sym__eof,
      sym__implicit_terminator,
      anon_sym_SEMI,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
  [18574] = 9,
    ACTIONS(23), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(51), 1,
      anon_sym_BSLASH,
    STATE(102), 1,
      sym__node_space,
    STATE(443), 1,
      aux_sym__node_field_comment_repeat1,
    STATE(513), 1,
      sym__escline,
    STATE(489), 2,
      sym__ws,
      aux_sym__node_space_repeat1,
    ACTIONS(1086), 3,
      sym_multi_line_comment,
      sym__bom,
      sym__unicode_space,
    STATE(196), 3,
      sym__node_terminator,
      sym__newline,
      sym_single_line_comment,
    ACTIONS(1181), 10,
      sym__eof,
      sym__implicit_terminator,
      anon_sym_SEMI,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
  [18616] = 9,
    ACTIONS(23), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(51), 1,
      anon_sym_BSLASH,
    STATE(81), 1,
      aux_sym__node_field_comment_repeat1,
    STATE(93), 1,
      sym__escline,
    STATE(102), 1,
      sym__node_space,
    STATE(84), 2,
      sym__ws,
      aux_sym__node_space_repeat1,
    ACTIONS(53), 3,
      sym_multi_line_comment,
      sym__bom,
      sym__unicode_space,
    STATE(254), 3,
      sym__node_terminator,
      sym__newline,
      sym_single_line_comment,
    ACTIONS(1183), 10,
      sym__eof,
      sym__implicit_terminator,
      anon_sym_SEMI,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
  [18658] = 9,
    ACTIONS(23), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(51), 1,
      anon_sym_BSLASH,
    STATE(81), 1,
      aux_sym__node_field_comment_repeat1,
    STATE(93), 1,
      sym__escline,
    STATE(102), 1,
      sym__node_space,
    STATE(84), 2,
      sym__ws,
      aux_sym__node_space_repeat1,
    ACTIONS(53), 3,
      sym_multi_line_comment,
      sym__bom,
      sym__unicode_space,
    STATE(252), 3,
      sym__node_terminator,
      sym__newline,
      sym_single_line_comment,
    ACTIONS(1185), 10,
      sym__eof,
      sym__implicit_terminator,
      anon_sym_SEMI,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
  [18700] = 9,
    ACTIONS(23), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(51), 1,
      anon_sym_BSLASH,
    STATE(81), 1,
      aux_sym__node_field_comment_repeat1,
    STATE(93), 1,
      sym__escline,
    STATE(102), 1,
      sym__node_space,
    STATE(84), 2,
      sym__ws,
      aux_sym__node_space_repeat1,
    ACTIONS(53), 3,
      sym_multi_line_comment,
      sym__bom,
      sym__unicode_space,
    STATE(248), 3,
      sym__node_terminator,
      sym__newline,
      sym_single_line_comment,
    ACTIONS(1187), 10,
      sym__eof,
      sym__implicit_terminator,
      anon_sym_SEMI,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
  [18742] = 9,
    ACTIONS(23), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(51), 1,
      anon_sym_BSLASH,
    STATE(93), 1,
      sym__escline,
    STATE(102), 1,
      sym__node_space,
    STATE(366), 1,
      aux_sym__node_field_comment_repeat1,
    STATE(367), 2,
      sym__ws,
      aux_sym__node_space_repeat1,
    ACTIONS(1191), 3,
      sym_multi_line_comment,
      sym__bom,
      sym__unicode_space,
    STATE(243), 3,
      sym__node_terminator,
      sym__newline,
      sym_single_line_comment,
    ACTIONS(1189), 10,
      sym__eof,
      sym__implicit_terminator,
      anon_sym_SEMI,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
  [18784] = 9,
    ACTIONS(23), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(51), 1,
      anon_sym_BSLASH,
    STATE(93), 1,
      sym__escline,
    STATE(102), 1,
      sym__node_space,
    STATE(372), 1,
      aux_sym__node_field_comment_repeat1,
    STATE(373), 2,
      sym__ws,
      aux_sym__node_space_repeat1,
    ACTIONS(1195), 3,
      sym_multi_line_comment,
      sym__bom,
      sym__unicode_space,
    STATE(235), 3,
      sym__node_terminator,
      sym__newline,
      sym_single_line_comment,
    ACTIONS(1193), 10,
      sym__eof,
      sym__implicit_terminator,
      anon_sym_SEMI,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
  [18826] = 9,
    ACTIONS(23), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(51), 1,
      anon_sym_BSLASH,
    STATE(81), 1,
      aux_sym__node_field_comment_repeat1,
    STATE(93), 1,
      sym__escline,
    STATE(102), 1,
      sym__node_space,
    STATE(84), 2,
      sym__ws,
      aux_sym__node_space_repeat1,
    ACTIONS(53), 3,
      sym_multi_line_comment,
      sym__bom,
      sym__unicode_space,
    STATE(246), 3,
      sym__node_terminator,
      sym__newline,
      sym_single_line_comment,
    ACTIONS(1197), 10,
      sym__eof,
      sym__implicit_terminator,
      anon_sym_SEMI,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
  [18868] = 9,
    ACTIONS(23), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(51), 1,
      anon_sym_BSLASH,
    STATE(102), 1,
      sym__node_space,
    STATE(370), 1,
      aux_sym__node_field_comment_repeat1,
    STATE(513), 1,
      sym__escline,
    STATE(489), 2,
      sym__ws,
      aux_sym__node_space_repeat1,
    ACTIONS(1086), 3,
      sym_multi_line_comment,
      sym__bom,
      sym__unicode_space,
    STATE(239), 3,
      sym__node_terminator,
      sym__newline,
      sym_single_line_comment,
    ACTIONS(1199), 10,
      sym__eof,
      sym__implicit_terminator,
      anon_sym_SEMI,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
  [18910] = 9,
    ACTIONS(23), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(51), 1,
      anon_sym_BSLASH,
    STATE(81), 1,
      aux_sym__node_field_comment_repeat1,
    STATE(93), 1,
      sym__escline,
    STATE(102), 1,
      sym__node_space,
    STATE(84), 2,
      sym__ws,
      aux_sym__node_space_repeat1,
    ACTIONS(53), 3,
      sym_multi_line_comment,
      sym__bom,
      sym__unicode_space,
    STATE(236), 3,
      sym__node_terminator,
      sym__newline,
      sym_single_line_comment,
    ACTIONS(1201), 10,
      sym__eof,
      sym__implicit_terminator,
      anon_sym_SEMI,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
  [18952] = 9,
    ACTIONS(23), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(51), 1,
      anon_sym_BSLASH,
    STATE(81), 1,
      aux_sym__node_field_comment_repeat1,
    STATE(93), 1,
      sym__escline,
    STATE(102), 1,
      sym__node_space,
    STATE(84), 2,
      sym__ws,
      aux_sym__node_space_repeat1,
    ACTIONS(53), 3,
      sym_multi_line_comment,
      sym__bom,
      sym__unicode_space,
    STATE(239), 3,
      sym__node_terminator,
      sym__newline,
      sym_single_line_comment,
    ACTIONS(1199), 10,
      sym__eof,
      sym__implicit_terminator,
      anon_sym_SEMI,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
  [18994] = 9,
    ACTIONS(23), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(51), 1,
      anon_sym_BSLASH,
    STATE(81), 1,
      aux_sym__node_field_comment_repeat1,
    STATE(93), 1,
      sym__escline,
    STATE(102), 1,
      sym__node_space,
    STATE(84), 2,
      sym__ws,
      aux_sym__node_space_repeat1,
    ACTIONS(53), 3,
      sym_multi_line_comment,
      sym__bom,
      sym__unicode_space,
    STATE(242), 3,
      sym__node_terminator,
      sym__newline,
      sym_single_line_comment,
    ACTIONS(1203), 10,
      sym__eof,
      sym__implicit_terminator,
      anon_sym_SEMI,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
  [19036] = 9,
    ACTIONS(23), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(51), 1,
      anon_sym_BSLASH,
    STATE(81), 1,
      aux_sym__node_field_comment_repeat1,
    STATE(93), 1,
      sym__escline,
    STATE(102), 1,
      sym__node_space,
    STATE(84), 2,
      sym__ws,
      aux_sym__node_space_repeat1,
    ACTIONS(53), 3,
      sym_multi_line_comment,
      sym__bom,
      sym__unicode_space,
    STATE(250), 3,
      sym__node_terminator,
      sym__newline,
      sym_single_line_comment,
    ACTIONS(1205), 10,
      sym__eof,
      sym__implicit_terminator,
      anon_sym_SEMI,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
  [19078] = 9,
    ACTIONS(23), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(51), 1,
      anon_sym_BSLASH,
    STATE(102), 1,
      sym__node_space,
    STATE(364), 1,
      aux_sym__node_field_comment_repeat1,
    STATE(513), 1,
      sym__escline,
    STATE(489), 2,
      sym__ws,
      aux_sym__node_space_repeat1,
    ACTIONS(1086), 3,
      sym_multi_line_comment,
      sym__bom,
      sym__unicode_space,
    STATE(244), 3,
      sym__node_terminator,
      sym__newline,
      sym_single_line_comment,
    ACTIONS(1207), 10,
      sym__eof,
      sym__implicit_terminator,
      anon_sym_SEMI,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
  [19120] = 9,
    ACTIONS(23), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(51), 1,
      anon_sym_BSLASH,
    STATE(81), 1,
      aux_sym__node_field_comment_repeat1,
    STATE(93), 1,
      sym__escline,
    STATE(102), 1,
      sym__node_space,
    STATE(84), 2,
      sym__ws,
      aux_sym__node_space_repeat1,
    ACTIONS(53), 3,
      sym_multi_line_comment,
      sym__bom,
      sym__unicode_space,
    STATE(172), 3,
      sym__node_terminator,
      sym__newline,
      sym_single_line_comment,
    ACTIONS(1209), 10,
      sym__eof,
      sym__implicit_terminator,
      anon_sym_SEMI,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
  [19162] = 9,
    ACTIONS(23), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(51), 1,
      anon_sym_BSLASH,
    STATE(81), 1,
      aux_sym__node_field_comment_repeat1,
    STATE(93), 1,
      sym__escline,
    STATE(102), 1,
      sym__node_space,
    STATE(84), 2,
      sym__ws,
      aux_sym__node_space_repeat1,
    ACTIONS(53), 3,
      sym_multi_line_comment,
      sym__bom,
      sym__unicode_space,
    STATE(244), 3,
      sym__node_terminator,
      sym__newline,
      sym_single_line_comment,
    ACTIONS(1207), 10,
      sym__eof,
      sym__implicit_terminator,
      anon_sym_SEMI,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
  [19204] = 9,
    ACTIONS(23), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(51), 1,
      anon_sym_BSLASH,
    STATE(81), 1,
      aux_sym__node_field_comment_repeat1,
    STATE(93), 1,
      sym__escline,
    STATE(102), 1,
      sym__node_space,
    STATE(84), 2,
      sym__ws,
      aux_sym__node_space_repeat1,
    ACTIONS(53), 3,
      sym_multi_line_comment,
      sym__bom,
      sym__unicode_space,
    STATE(169), 3,
      sym__node_terminator,
      sym__newline,
      sym_single_line_comment,
    ACTIONS(1211), 10,
      sym__eof,
      sym__implicit_terminator,
      anon_sym_SEMI,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
  [19246] = 9,
    ACTIONS(23), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(51), 1,
      anon_sym_BSLASH,
    STATE(102), 1,
      sym__node_space,
    STATE(456), 1,
      aux_sym__node_field_comment_repeat1,
    STATE(513), 1,
      sym__escline,
    STATE(489), 2,
      sym__ws,
      aux_sym__node_space_repeat1,
    ACTIONS(1086), 3,
      sym_multi_line_comment,
      sym__bom,
      sym__unicode_space,
    STATE(169), 3,
      sym__node_terminator,
      sym__newline,
      sym_single_line_comment,
    ACTIONS(1211), 10,
      sym__eof,
      sym__implicit_terminator,
      anon_sym_SEMI,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
  [19288] = 9,
    ACTIONS(23), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(51), 1,
      anon_sym_BSLASH,
    STATE(81), 1,
      aux_sym__node_field_comment_repeat1,
    STATE(93), 1,
      sym__escline,
    STATE(102), 1,
      sym__node_space,
    STATE(84), 2,
      sym__ws,
      aux_sym__node_space_repeat1,
    ACTIONS(53), 3,
      sym_multi_line_comment,
      sym__bom,
      sym__unicode_space,
    STATE(247), 3,
      sym__node_terminator,
      sym__newline,
      sym_single_line_comment,
    ACTIONS(1213), 10,
      sym__eof,
      sym__implicit_terminator,
      anon_sym_SEMI,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
  [19330] = 9,
    ACTIONS(23), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(51), 1,
      anon_sym_BSLASH,
    STATE(81), 1,
      aux_sym__node_field_comment_repeat1,
    STATE(93), 1,
      sym__escline,
    STATE(102), 1,
      sym__node_space,
    STATE(84), 2,
      sym__ws,
      aux_sym__node_space_repeat1,
    ACTIONS(53), 3,
      sym_multi_line_comment,
      sym__bom,
      sym__unicode_space,
    STATE(200), 3,
      sym__node_terminator,
      sym__newline,
      sym_single_line_comment,
    ACTIONS(1215), 10,
      sym__eof,
      sym__implicit_terminator,
      anon_sym_SEMI,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
  [19372] = 9,
    ACTIONS(23), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(51), 1,
      anon_sym_BSLASH,
    STATE(81), 1,
      aux_sym__node_field_comment_repeat1,
    STATE(93), 1,
      sym__escline,
    STATE(102), 1,
      sym__node_space,
    STATE(84), 2,
      sym__ws,
      aux_sym__node_space_repeat1,
    ACTIONS(53), 3,
      sym_multi_line_comment,
      sym__bom,
      sym__unicode_space,
    STATE(251), 3,
      sym__node_terminator,
      sym__newline,
      sym_single_line_comment,
    ACTIONS(1217), 10,
      sym__eof,
      sym__implicit_terminator,
      anon_sym_SEMI,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
  [19414] = 9,
    ACTIONS(23), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(51), 1,
      anon_sym_BSLASH,
    STATE(81), 1,
      aux_sym__node_field_comment_repeat1,
    STATE(93), 1,
      sym__escline,
    STATE(102), 1,
      sym__node_space,
    STATE(84), 2,
      sym__ws,
      aux_sym__node_space_repeat1,
    ACTIONS(53), 3,
      sym_multi_line_comment,
      sym__bom,
      sym__unicode_space,
    STATE(234), 3,
      sym__node_terminator,
      sym__newline,
      sym_single_line_comment,
    ACTIONS(1219), 10,
      sym__eof,
      sym__implicit_terminator,
      anon_sym_SEMI,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
  [19456] = 9,
    ACTIONS(23), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(51), 1,
      anon_sym_BSLASH,
    STATE(81), 1,
      aux_sym__node_field_comment_repeat1,
    STATE(93), 1,
      sym__escline,
    STATE(102), 1,
      sym__node_space,
    STATE(84), 2,
      sym__ws,
      aux_sym__node_space_repeat1,
    ACTIONS(53), 3,
      sym_multi_line_comment,
      sym__bom,
      sym__unicode_space,
    STATE(165), 3,
      sym__node_terminator,
      sym__newline,
      sym_single_line_comment,
    ACTIONS(1221), 10,
      sym__eof,
      sym__implicit_terminator,
      anon_sym_SEMI,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
  [19498] = 9,
    ACTIONS(23), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(51), 1,
      anon_sym_BSLASH,
    STATE(93), 1,
      sym__escline,
    STATE(102), 1,
      sym__node_space,
    STATE(358), 1,
      aux_sym__node_field_comment_repeat1,
    STATE(359), 2,
      sym__ws,
      aux_sym__node_space_repeat1,
    ACTIONS(1225), 3,
      sym_multi_line_comment,
      sym__bom,
      sym__unicode_space,
    STATE(256), 3,
      sym__node_terminator,
      sym__newline,
      sym_single_line_comment,
    ACTIONS(1223), 10,
      sym__eof,
      sym__implicit_terminator,
      anon_sym_SEMI,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
  [19540] = 9,
    ACTIONS(23), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(51), 1,
      anon_sym_BSLASH,
    STATE(81), 1,
      aux_sym__node_field_comment_repeat1,
    STATE(93), 1,
      sym__escline,
    STATE(102), 1,
      sym__node_space,
    STATE(84), 2,
      sym__ws,
      aux_sym__node_space_repeat1,
    ACTIONS(53), 3,
      sym_multi_line_comment,
      sym__bom,
      sym__unicode_space,
    STATE(164), 3,
      sym__node_terminator,
      sym__newline,
      sym_single_line_comment,
    ACTIONS(1227), 10,
      sym__eof,
      sym__implicit_terminator,
      anon_sym_SEMI,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
  [19582] = 9,
    ACTIONS(23), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(51), 1,
      anon_sym_BSLASH,
    STATE(102), 1,
      sym__node_space,
    STATE(459), 1,
      aux_sym__node_field_comment_repeat1,
    STATE(513), 1,
      sym__escline,
    STATE(489), 2,
      sym__ws,
      aux_sym__node_space_repeat1,
    ACTIONS(1086), 3,
      sym_multi_line_comment,
      sym__bom,
      sym__unicode_space,
    STATE(164), 3,
      sym__node_terminator,
      sym__newline,
      sym_single_line_comment,
    ACTIONS(1227), 10,
      sym__eof,
      sym__implicit_terminator,
      anon_sym_SEMI,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
  [19624] = 9,
    ACTIONS(23), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(51), 1,
      anon_sym_BSLASH,
    STATE(102), 1,
      sym__node_space,
    STATE(356), 1,
      aux_sym__node_field_comment_repeat1,
    STATE(513), 1,
      sym__escline,
    STATE(489), 2,
      sym__ws,
      aux_sym__node_space_repeat1,
    ACTIONS(1086), 3,
      sym_multi_line_comment,
      sym__bom,
      sym__unicode_space,
    STATE(258), 3,
      sym__node_terminator,
      sym__newline,
      sym_single_line_comment,
    ACTIONS(1229), 10,
      sym__eof,
      sym__implicit_terminator,
      anon_sym_SEMI,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
  [19666] = 9,
    ACTIONS(23), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(51), 1,
      anon_sym_BSLASH,
    STATE(81), 1,
      aux_sym__node_field_comment_repeat1,
    STATE(93), 1,
      sym__escline,
    STATE(102), 1,
      sym__node_space,
    STATE(84), 2,
      sym__ws,
      aux_sym__node_space_repeat1,
    ACTIONS(53), 3,
      sym_multi_line_comment,
      sym__bom,
      sym__unicode_space,
    STATE(258), 3,
      sym__node_terminator,
      sym__newline,
      sym_single_line_comment,
    ACTIONS(1229), 10,
      sym__eof,
      sym__implicit_terminator,
      anon_sym_SEMI,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
  [19708] = 9,
    ACTIONS(23), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(51), 1,
      anon_sym_BSLASH,
    STATE(81), 1,
      aux_sym__node_field_comment_repeat1,
    STATE(93), 1,
      sym__escline,
    STATE(102), 1,
      sym__node_space,
    STATE(84), 2,
      sym__ws,
      aux_sym__node_space_repeat1,
    ACTIONS(53), 3,
      sym_multi_line_comment,
      sym__bom,
      sym__unicode_space,
    STATE(163), 3,
      sym__node_terminator,
      sym__newline,
      sym_single_line_comment,
    ACTIONS(1231), 10,
      sym__eof,
      sym__implicit_terminator,
      anon_sym_SEMI,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
  [19750] = 9,
    ACTIONS(23), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(51), 1,
      anon_sym_BSLASH,
    STATE(81), 1,
      aux_sym__node_field_comment_repeat1,
    STATE(93), 1,
      sym__escline,
    STATE(102), 1,
      sym__node_space,
    STATE(84), 2,
      sym__ws,
      aux_sym__node_space_repeat1,
    ACTIONS(53), 3,
      sym_multi_line_comment,
      sym__bom,
      sym__unicode_space,
    STATE(259), 3,
      sym__node_terminator,
      sym__newline,
      sym_single_line_comment,
    ACTIONS(1233), 10,
      sym__eof,
      sym__implicit_terminator,
      anon_sym_SEMI,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
  [19792] = 9,
    ACTIONS(23), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(51), 1,
      anon_sym_BSLASH,
    STATE(81), 1,
      aux_sym__node_field_comment_repeat1,
    STATE(93), 1,
      sym__escline,
    STATE(102), 1,
      sym__node_space,
    STATE(84), 2,
      sym__ws,
      aux_sym__node_space_repeat1,
    ACTIONS(53), 3,
      sym_multi_line_comment,
      sym__bom,
      sym__unicode_space,
    STATE(159), 3,
      sym__node_terminator,
      sym__newline,
      sym_single_line_comment,
    ACTIONS(1235), 10,
      sym__eof,
      sym__implicit_terminator,
      anon_sym_SEMI,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
  [19834] = 9,
    ACTIONS(23), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(51), 1,
      anon_sym_BSLASH,
    STATE(102), 1,
      sym__node_space,
    STATE(450), 1,
      aux_sym__node_field_comment_repeat1,
    STATE(513), 1,
      sym__escline,
    STATE(489), 2,
      sym__ws,
      aux_sym__node_space_repeat1,
    ACTIONS(1086), 3,
      sym_multi_line_comment,
      sym__bom,
      sym__unicode_space,
    STATE(159), 3,
      sym__node_terminator,
      sym__newline,
      sym_single_line_comment,
    ACTIONS(1235), 10,
      sym__eof,
      sym__implicit_terminator,
      anon_sym_SEMI,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
  [19876] = 9,
    ACTIONS(23), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(51), 1,
      anon_sym_BSLASH,
    STATE(102), 1,
      sym__node_space,
    STATE(427), 1,
      aux_sym__node_field_comment_repeat1,
    STATE(513), 1,
      sym__escline,
    STATE(489), 2,
      sym__ws,
      aux_sym__node_space_repeat1,
    ACTIONS(1086), 3,
      sym_multi_line_comment,
      sym__bom,
      sym__unicode_space,
    STATE(200), 3,
      sym__node_terminator,
      sym__newline,
      sym_single_line_comment,
    ACTIONS(1215), 10,
      sym__eof,
      sym__implicit_terminator,
      anon_sym_SEMI,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
  [19918] = 3,
    STATE(420), 1,
      aux_sym__integer_repeat1,
    ACTIONS(1239), 2,
      anon_sym__,
      sym__digit,
    ACTIONS(1237), 20,
      sym__eof,
      sym_multi_line_comment,
      sym__implicit_terminator,
      anon_sym_SLASH_DASH,
      anon_sym_LBRACE,
      anon_sym_SEMI,
      anon_sym_DOT,
      anon_sym_e,
      anon_sym_E,
      anon_sym_BSLASH,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
      anon_sym_SLASH_SLASH,
  [19948] = 9,
    ACTIONS(23), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(51), 1,
      anon_sym_BSLASH,
    STATE(81), 1,
      aux_sym__node_field_comment_repeat1,
    STATE(93), 1,
      sym__escline,
    STATE(102), 1,
      sym__node_space,
    STATE(84), 2,
      sym__ws,
      aux_sym__node_space_repeat1,
    ACTIONS(53), 3,
      sym_multi_line_comment,
      sym__bom,
      sym__unicode_space,
    STATE(158), 3,
      sym__node_terminator,
      sym__newline,
      sym_single_line_comment,
    ACTIONS(1241), 10,
      sym__eof,
      sym__implicit_terminator,
      anon_sym_SEMI,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
  [19990] = 9,
    ACTIONS(23), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(51), 1,
      anon_sym_BSLASH,
    STATE(93), 1,
      sym__escline,
    STATE(102), 1,
      sym__node_space,
    STATE(330), 1,
      aux_sym__node_field_comment_repeat1,
    STATE(329), 2,
      sym__ws,
      aux_sym__node_space_repeat1,
    ACTIONS(1245), 3,
      sym_multi_line_comment,
      sym__bom,
      sym__unicode_space,
    STATE(228), 3,
      sym__node_terminator,
      sym__newline,
      sym_single_line_comment,
    ACTIONS(1243), 10,
      sym__eof,
      sym__implicit_terminator,
      anon_sym_SEMI,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
  [20032] = 9,
    ACTIONS(23), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(51), 1,
      anon_sym_BSLASH,
    STATE(81), 1,
      aux_sym__node_field_comment_repeat1,
    STATE(93), 1,
      sym__escline,
    STATE(102), 1,
      sym__node_space,
    STATE(84), 2,
      sym__ws,
      aux_sym__node_space_repeat1,
    ACTIONS(53), 3,
      sym_multi_line_comment,
      sym__bom,
      sym__unicode_space,
    STATE(157), 3,
      sym__node_terminator,
      sym__newline,
      sym_single_line_comment,
    ACTIONS(1247), 10,
      sym__eof,
      sym__implicit_terminator,
      anon_sym_SEMI,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
  [20074] = 9,
    ACTIONS(23), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(51), 1,
      anon_sym_BSLASH,
    STATE(102), 1,
      sym__node_space,
    STATE(447), 1,
      aux_sym__node_field_comment_repeat1,
    STATE(513), 1,
      sym__escline,
    STATE(489), 2,
      sym__ws,
      aux_sym__node_space_repeat1,
    ACTIONS(1086), 3,
      sym_multi_line_comment,
      sym__bom,
      sym__unicode_space,
    STATE(157), 3,
      sym__node_terminator,
      sym__newline,
      sym_single_line_comment,
    ACTIONS(1247), 10,
      sym__eof,
      sym__implicit_terminator,
      anon_sym_SEMI,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
  [20116] = 9,
    ACTIONS(23), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(51), 1,
      anon_sym_BSLASH,
    STATE(93), 1,
      sym__escline,
    STATE(102), 1,
      sym__node_space,
    STATE(333), 1,
      aux_sym__node_field_comment_repeat1,
    STATE(332), 2,
      sym__ws,
      aux_sym__node_space_repeat1,
    ACTIONS(1251), 3,
      sym_multi_line_comment,
      sym__bom,
      sym__unicode_space,
    STATE(224), 3,
      sym__node_terminator,
      sym__newline,
      sym_single_line_comment,
    ACTIONS(1249), 10,
      sym__eof,
      sym__implicit_terminator,
      anon_sym_SEMI,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
  [20158] = 9,
    ACTIONS(23), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(51), 1,
      anon_sym_BSLASH,
    STATE(81), 1,
      aux_sym__node_field_comment_repeat1,
    STATE(93), 1,
      sym__escline,
    STATE(102), 1,
      sym__node_space,
    STATE(84), 2,
      sym__ws,
      aux_sym__node_space_repeat1,
    ACTIONS(53), 3,
      sym_multi_line_comment,
      sym__bom,
      sym__unicode_space,
    STATE(154), 3,
      sym__node_terminator,
      sym__newline,
      sym_single_line_comment,
    ACTIONS(1253), 10,
      sym__eof,
      sym__implicit_terminator,
      anon_sym_SEMI,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
  [20200] = 9,
    ACTIONS(23), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(51), 1,
      anon_sym_BSLASH,
    STATE(102), 1,
      sym__node_space,
    STATE(335), 1,
      aux_sym__node_field_comment_repeat1,
    STATE(513), 1,
      sym__escline,
    STATE(489), 2,
      sym__ws,
      aux_sym__node_space_repeat1,
    ACTIONS(1086), 3,
      sym_multi_line_comment,
      sym__bom,
      sym__unicode_space,
    STATE(205), 3,
      sym__node_terminator,
      sym__newline,
      sym_single_line_comment,
    ACTIONS(1255), 10,
      sym__eof,
      sym__implicit_terminator,
      anon_sym_SEMI,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
  [20242] = 9,
    ACTIONS(23), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(51), 1,
      anon_sym_BSLASH,
    STATE(81), 1,
      aux_sym__node_field_comment_repeat1,
    STATE(93), 1,
      sym__escline,
    STATE(102), 1,
      sym__node_space,
    STATE(84), 2,
      sym__ws,
      aux_sym__node_space_repeat1,
    ACTIONS(53), 3,
      sym_multi_line_comment,
      sym__bom,
      sym__unicode_space,
    STATE(152), 3,
      sym__node_terminator,
      sym__newline,
      sym_single_line_comment,
    ACTIONS(1257), 10,
      sym__eof,
      sym__implicit_terminator,
      anon_sym_SEMI,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
  [20284] = 9,
    ACTIONS(23), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(51), 1,
      anon_sym_BSLASH,
    STATE(102), 1,
      sym__node_space,
    STATE(444), 1,
      aux_sym__node_field_comment_repeat1,
    STATE(513), 1,
      sym__escline,
    STATE(489), 2,
      sym__ws,
      aux_sym__node_space_repeat1,
    ACTIONS(1086), 3,
      sym_multi_line_comment,
      sym__bom,
      sym__unicode_space,
    STATE(152), 3,
      sym__node_terminator,
      sym__newline,
      sym_single_line_comment,
    ACTIONS(1257), 10,
      sym__eof,
      sym__implicit_terminator,
      anon_sym_SEMI,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
  [20326] = 9,
    ACTIONS(23), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(51), 1,
      anon_sym_BSLASH,
    STATE(93), 1,
      sym__escline,
    STATE(102), 1,
      sym__node_space,
    STATE(342), 1,
      aux_sym__node_field_comment_repeat1,
    STATE(343), 2,
      sym__ws,
      aux_sym__node_space_repeat1,
    ACTIONS(1261), 3,
      sym_multi_line_comment,
      sym__bom,
      sym__unicode_space,
    STATE(193), 3,
      sym__node_terminator,
      sym__newline,
      sym_single_line_comment,
    ACTIONS(1259), 10,
      sym__eof,
      sym__implicit_terminator,
      anon_sym_SEMI,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
  [20368] = 9,
    ACTIONS(23), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(51), 1,
      anon_sym_BSLASH,
    STATE(93), 1,
      sym__escline,
    STATE(102), 1,
      sym__node_space,
    STATE(441), 1,
      aux_sym__node_field_comment_repeat1,
    STATE(440), 2,
      sym__ws,
      aux_sym__node_space_repeat1,
    ACTIONS(1265), 3,
      sym_multi_line_comment,
      sym__bom,
      sym__unicode_space,
    STATE(147), 3,
      sym__node_terminator,
      sym__newline,
      sym_single_line_comment,
    ACTIONS(1263), 10,
      sym__eof,
      sym__implicit_terminator,
      anon_sym_SEMI,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
  [20410] = 9,
    ACTIONS(23), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(51), 1,
      anon_sym_BSLASH,
    STATE(102), 1,
      sym__node_space,
    STATE(341), 1,
      aux_sym__node_field_comment_repeat1,
    STATE(513), 1,
      sym__escline,
    STATE(489), 2,
      sym__ws,
      aux_sym__node_space_repeat1,
    ACTIONS(1086), 3,
      sym_multi_line_comment,
      sym__bom,
      sym__unicode_space,
    STATE(181), 3,
      sym__node_terminator,
      sym__newline,
      sym_single_line_comment,
    ACTIONS(1267), 10,
      sym__eof,
      sym__implicit_terminator,
      anon_sym_SEMI,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
  [20452] = 9,
    ACTIONS(23), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(51), 1,
      anon_sym_BSLASH,
    STATE(81), 1,
      aux_sym__node_field_comment_repeat1,
    STATE(93), 1,
      sym__escline,
    STATE(102), 1,
      sym__node_space,
    STATE(84), 2,
      sym__ws,
      aux_sym__node_space_repeat1,
    ACTIONS(53), 3,
      sym_multi_line_comment,
      sym__bom,
      sym__unicode_space,
    STATE(145), 3,
      sym__node_terminator,
      sym__newline,
      sym_single_line_comment,
    ACTIONS(1269), 10,
      sym__eof,
      sym__implicit_terminator,
      anon_sym_SEMI,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
  [20494] = 9,
    ACTIONS(23), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(51), 1,
      anon_sym_BSLASH,
    STATE(81), 1,
      aux_sym__node_field_comment_repeat1,
    STATE(93), 1,
      sym__escline,
    STATE(102), 1,
      sym__node_space,
    STATE(84), 2,
      sym__ws,
      aux_sym__node_space_repeat1,
    ACTIONS(53), 3,
      sym_multi_line_comment,
      sym__bom,
      sym__unicode_space,
    STATE(181), 3,
      sym__node_terminator,
      sym__newline,
      sym_single_line_comment,
    ACTIONS(1267), 10,
      sym__eof,
      sym__implicit_terminator,
      anon_sym_SEMI,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
  [20536] = 9,
    ACTIONS(23), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(51), 1,
      anon_sym_BSLASH,
    STATE(81), 1,
      aux_sym__node_field_comment_repeat1,
    STATE(93), 1,
      sym__escline,
    STATE(102), 1,
      sym__node_space,
    STATE(84), 2,
      sym__ws,
      aux_sym__node_space_repeat1,
    ACTIONS(53), 3,
      sym_multi_line_comment,
      sym__bom,
      sym__unicode_space,
    STATE(144), 3,
      sym__node_terminator,
      sym__newline,
      sym_single_line_comment,
    ACTIONS(1271), 10,
      sym__eof,
      sym__implicit_terminator,
      anon_sym_SEMI,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
  [20578] = 9,
    ACTIONS(23), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(51), 1,
      anon_sym_BSLASH,
    STATE(102), 1,
      sym__node_space,
    STATE(438), 1,
      aux_sym__node_field_comment_repeat1,
    STATE(513), 1,
      sym__escline,
    STATE(489), 2,
      sym__ws,
      aux_sym__node_space_repeat1,
    ACTIONS(1086), 3,
      sym_multi_line_comment,
      sym__bom,
      sym__unicode_space,
    STATE(144), 3,
      sym__node_terminator,
      sym__newline,
      sym_single_line_comment,
    ACTIONS(1271), 10,
      sym__eof,
      sym__implicit_terminator,
      anon_sym_SEMI,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
  [20620] = 9,
    ACTIONS(23), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(51), 1,
      anon_sym_BSLASH,
    STATE(93), 1,
      sym__escline,
    STATE(102), 1,
      sym__node_space,
    STATE(436), 1,
      aux_sym__node_field_comment_repeat1,
    STATE(435), 2,
      sym__ws,
      aux_sym__node_space_repeat1,
    ACTIONS(1275), 3,
      sym_multi_line_comment,
      sym__bom,
      sym__unicode_space,
    STATE(141), 3,
      sym__node_terminator,
      sym__newline,
      sym_single_line_comment,
    ACTIONS(1273), 10,
      sym__eof,
      sym__implicit_terminator,
      anon_sym_SEMI,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
  [20662] = 9,
    ACTIONS(23), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(51), 1,
      anon_sym_BSLASH,
    STATE(81), 1,
      aux_sym__node_field_comment_repeat1,
    STATE(93), 1,
      sym__escline,
    STATE(102), 1,
      sym__node_space,
    STATE(84), 2,
      sym__ws,
      aux_sym__node_space_repeat1,
    ACTIONS(53), 3,
      sym_multi_line_comment,
      sym__bom,
      sym__unicode_space,
    STATE(205), 3,
      sym__node_terminator,
      sym__newline,
      sym_single_line_comment,
    ACTIONS(1255), 10,
      sym__eof,
      sym__implicit_terminator,
      anon_sym_SEMI,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
  [20704] = 9,
    ACTIONS(23), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(51), 1,
      anon_sym_BSLASH,
    STATE(93), 1,
      sym__escline,
    STATE(102), 1,
      sym__node_space,
    STATE(434), 1,
      aux_sym__node_field_comment_repeat1,
    STATE(433), 2,
      sym__ws,
      aux_sym__node_space_repeat1,
    ACTIONS(1279), 3,
      sym_multi_line_comment,
      sym__bom,
      sym__unicode_space,
    STATE(140), 3,
      sym__node_terminator,
      sym__newline,
      sym_single_line_comment,
    ACTIONS(1277), 10,
      sym__eof,
      sym__implicit_terminator,
      anon_sym_SEMI,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
  [20746] = 9,
    ACTIONS(23), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(51), 1,
      anon_sym_BSLASH,
    STATE(81), 1,
      aux_sym__node_field_comment_repeat1,
    STATE(93), 1,
      sym__escline,
    STATE(102), 1,
      sym__node_space,
    STATE(84), 2,
      sym__ws,
      aux_sym__node_space_repeat1,
    ACTIONS(53), 3,
      sym_multi_line_comment,
      sym__bom,
      sym__unicode_space,
    STATE(198), 3,
      sym__node_terminator,
      sym__newline,
      sym_single_line_comment,
    ACTIONS(1281), 10,
      sym__eof,
      sym__implicit_terminator,
      anon_sym_SEMI,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
  [20788] = 9,
    ACTIONS(23), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(51), 1,
      anon_sym_BSLASH,
    STATE(93), 1,
      sym__escline,
    STATE(102), 1,
      sym__node_space,
    STATE(338), 1,
      aux_sym__node_field_comment_repeat1,
    STATE(337), 2,
      sym__ws,
      aux_sym__node_space_repeat1,
    ACTIONS(1285), 3,
      sym_multi_line_comment,
      sym__bom,
      sym__unicode_space,
    STATE(179), 3,
      sym__node_terminator,
      sym__newline,
      sym_single_line_comment,
    ACTIONS(1283), 10,
      sym__eof,
      sym__implicit_terminator,
      anon_sym_SEMI,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
  [20830] = 9,
    ACTIONS(23), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(51), 1,
      anon_sym_BSLASH,
    STATE(81), 1,
      aux_sym__node_field_comment_repeat1,
    STATE(93), 1,
      sym__escline,
    STATE(102), 1,
      sym__node_space,
    STATE(84), 2,
      sym__ws,
      aux_sym__node_space_repeat1,
    ACTIONS(53), 3,
      sym_multi_line_comment,
      sym__bom,
      sym__unicode_space,
    STATE(133), 3,
      sym__node_terminator,
      sym__newline,
      sym_single_line_comment,
    ACTIONS(1287), 10,
      sym__eof,
      sym__implicit_terminator,
      anon_sym_SEMI,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
  [20872] = 9,
    ACTIONS(23), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(51), 1,
      anon_sym_BSLASH,
    STATE(102), 1,
      sym__node_space,
    STATE(340), 1,
      aux_sym__node_field_comment_repeat1,
    STATE(513), 1,
      sym__escline,
    STATE(489), 2,
      sym__ws,
      aux_sym__node_space_repeat1,
    ACTIONS(1086), 3,
      sym_multi_line_comment,
      sym__bom,
      sym__unicode_space,
    STATE(178), 3,
      sym__node_terminator,
      sym__newline,
      sym_single_line_comment,
    ACTIONS(1289), 10,
      sym__eof,
      sym__implicit_terminator,
      anon_sym_SEMI,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
  [20914] = 9,
    ACTIONS(23), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(51), 1,
      anon_sym_BSLASH,
    STATE(81), 1,
      aux_sym__node_field_comment_repeat1,
    STATE(93), 1,
      sym__escline,
    STATE(102), 1,
      sym__node_space,
    STATE(84), 2,
      sym__ws,
      aux_sym__node_space_repeat1,
    ACTIONS(53), 3,
      sym_multi_line_comment,
      sym__bom,
      sym__unicode_space,
    STATE(132), 3,
      sym__node_terminator,
      sym__newline,
      sym_single_line_comment,
    ACTIONS(1291), 10,
      sym__eof,
      sym__implicit_terminator,
      anon_sym_SEMI,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
  [20956] = 9,
    ACTIONS(23), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(51), 1,
      anon_sym_BSLASH,
    STATE(102), 1,
      sym__node_space,
    STATE(432), 1,
      aux_sym__node_field_comment_repeat1,
    STATE(513), 1,
      sym__escline,
    STATE(489), 2,
      sym__ws,
      aux_sym__node_space_repeat1,
    ACTIONS(1086), 3,
      sym_multi_line_comment,
      sym__bom,
      sym__unicode_space,
    STATE(132), 3,
      sym__node_terminator,
      sym__newline,
      sym_single_line_comment,
    ACTIONS(1291), 10,
      sym__eof,
      sym__implicit_terminator,
      anon_sym_SEMI,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
  [20998] = 9,
    ACTIONS(23), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(51), 1,
      anon_sym_BSLASH,
    STATE(81), 1,
      aux_sym__node_field_comment_repeat1,
    STATE(93), 1,
      sym__escline,
    STATE(102), 1,
      sym__node_space,
    STATE(84), 2,
      sym__ws,
      aux_sym__node_space_repeat1,
    ACTIONS(53), 3,
      sym_multi_line_comment,
      sym__bom,
      sym__unicode_space,
    STATE(178), 3,
      sym__node_terminator,
      sym__newline,
      sym_single_line_comment,
    ACTIONS(1289), 10,
      sym__eof,
      sym__implicit_terminator,
      anon_sym_SEMI,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
  [21040] = 9,
    ACTIONS(23), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(51), 1,
      anon_sym_BSLASH,
    STATE(93), 1,
      sym__escline,
    STATE(102), 1,
      sym__node_space,
    STATE(430), 1,
      aux_sym__node_field_comment_repeat1,
    STATE(429), 2,
      sym__ws,
      aux_sym__node_space_repeat1,
    ACTIONS(1295), 3,
      sym_multi_line_comment,
      sym__bom,
      sym__unicode_space,
    STATE(131), 3,
      sym__node_terminator,
      sym__newline,
      sym_single_line_comment,
    ACTIONS(1293), 10,
      sym__eof,
      sym__implicit_terminator,
      anon_sym_SEMI,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
  [21082] = 9,
    ACTIONS(23), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(51), 1,
      anon_sym_BSLASH,
    STATE(81), 1,
      aux_sym__node_field_comment_repeat1,
    STATE(93), 1,
      sym__escline,
    STATE(102), 1,
      sym__node_space,
    STATE(84), 2,
      sym__ws,
      aux_sym__node_space_repeat1,
    ACTIONS(53), 3,
      sym_multi_line_comment,
      sym__bom,
      sym__unicode_space,
    STATE(136), 3,
      sym__node_terminator,
      sym__newline,
      sym_single_line_comment,
    ACTIONS(1297), 10,
      sym__eof,
      sym__implicit_terminator,
      anon_sym_SEMI,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
  [21124] = 9,
    ACTIONS(23), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(51), 1,
      anon_sym_BSLASH,
    STATE(81), 1,
      aux_sym__node_field_comment_repeat1,
    STATE(93), 1,
      sym__escline,
    STATE(102), 1,
      sym__node_space,
    STATE(84), 2,
      sym__ws,
      aux_sym__node_space_repeat1,
    ACTIONS(53), 3,
      sym_multi_line_comment,
      sym__bom,
      sym__unicode_space,
    STATE(129), 3,
      sym__node_terminator,
      sym__newline,
      sym_single_line_comment,
    ACTIONS(1299), 10,
      sym__eof,
      sym__implicit_terminator,
      anon_sym_SEMI,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
  [21166] = 9,
    ACTIONS(23), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(51), 1,
      anon_sym_BSLASH,
    STATE(102), 1,
      sym__node_space,
    STATE(344), 1,
      aux_sym__node_field_comment_repeat1,
    STATE(513), 1,
      sym__escline,
    STATE(489), 2,
      sym__ws,
      aux_sym__node_space_repeat1,
    ACTIONS(1086), 3,
      sym_multi_line_comment,
      sym__bom,
      sym__unicode_space,
    STATE(170), 3,
      sym__node_terminator,
      sym__newline,
      sym_single_line_comment,
    ACTIONS(1301), 10,
      sym__eof,
      sym__implicit_terminator,
      anon_sym_SEMI,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
  [21208] = 9,
    ACTIONS(23), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(51), 1,
      anon_sym_BSLASH,
    STATE(81), 1,
      aux_sym__node_field_comment_repeat1,
    STATE(93), 1,
      sym__escline,
    STATE(102), 1,
      sym__node_space,
    STATE(84), 2,
      sym__ws,
      aux_sym__node_space_repeat1,
    ACTIONS(53), 3,
      sym_multi_line_comment,
      sym__bom,
      sym__unicode_space,
    STATE(128), 3,
      sym__node_terminator,
      sym__newline,
      sym_single_line_comment,
    ACTIONS(1303), 10,
      sym__eof,
      sym__implicit_terminator,
      anon_sym_SEMI,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
  [21250] = 9,
    ACTIONS(23), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(51), 1,
      anon_sym_BSLASH,
    STATE(102), 1,
      sym__node_space,
    STATE(423), 1,
      aux_sym__node_field_comment_repeat1,
    STATE(513), 1,
      sym__escline,
    STATE(489), 2,
      sym__ws,
      aux_sym__node_space_repeat1,
    ACTIONS(1086), 3,
      sym_multi_line_comment,
      sym__bom,
      sym__unicode_space,
    STATE(128), 3,
      sym__node_terminator,
      sym__newline,
      sym_single_line_comment,
    ACTIONS(1303), 10,
      sym__eof,
      sym__implicit_terminator,
      anon_sym_SEMI,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
  [21292] = 9,
    ACTIONS(23), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(51), 1,
      anon_sym_BSLASH,
    STATE(93), 1,
      sym__escline,
    STATE(102), 1,
      sym__node_space,
    STATE(422), 1,
      aux_sym__node_field_comment_repeat1,
    STATE(417), 2,
      sym__ws,
      aux_sym__node_space_repeat1,
    ACTIONS(1307), 3,
      sym_multi_line_comment,
      sym__bom,
      sym__unicode_space,
    STATE(127), 3,
      sym__node_terminator,
      sym__newline,
      sym_single_line_comment,
    ACTIONS(1305), 10,
      sym__eof,
      sym__implicit_terminator,
      anon_sym_SEMI,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
  [21334] = 9,
    ACTIONS(23), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(51), 1,
      anon_sym_BSLASH,
    STATE(81), 1,
      aux_sym__node_field_comment_repeat1,
    STATE(93), 1,
      sym__escline,
    STATE(102), 1,
      sym__node_space,
    STATE(84), 2,
      sym__ws,
      aux_sym__node_space_repeat1,
    ACTIONS(53), 3,
      sym_multi_line_comment,
      sym__bom,
      sym__unicode_space,
    STATE(170), 3,
      sym__node_terminator,
      sym__newline,
      sym_single_line_comment,
    ACTIONS(1301), 10,
      sym__eof,
      sym__implicit_terminator,
      anon_sym_SEMI,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
  [21376] = 9,
    ACTIONS(23), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(51), 1,
      anon_sym_BSLASH,
    STATE(93), 1,
      sym__escline,
    STATE(102), 1,
      sym__node_space,
    STATE(389), 1,
      aux_sym__node_field_comment_repeat1,
    STATE(387), 2,
      sym__ws,
      aux_sym__node_space_repeat1,
    ACTIONS(1311), 3,
      sym_multi_line_comment,
      sym__bom,
      sym__unicode_space,
    STATE(220), 3,
      sym__node_terminator,
      sym__newline,
      sym_single_line_comment,
    ACTIONS(1309), 10,
      sym__eof,
      sym__implicit_terminator,
      anon_sym_SEMI,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
  [21418] = 9,
    ACTIONS(23), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(51), 1,
      anon_sym_BSLASH,
    STATE(81), 1,
      aux_sym__node_field_comment_repeat1,
    STATE(93), 1,
      sym__escline,
    STATE(102), 1,
      sym__node_space,
    STATE(84), 2,
      sym__ws,
      aux_sym__node_space_repeat1,
    ACTIONS(53), 3,
      sym_multi_line_comment,
      sym__bom,
      sym__unicode_space,
    STATE(126), 3,
      sym__node_terminator,
      sym__newline,
      sym_single_line_comment,
    ACTIONS(1313), 10,
      sym__eof,
      sym__implicit_terminator,
      anon_sym_SEMI,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
  [21460] = 9,
    ACTIONS(23), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(51), 1,
      anon_sym_BSLASH,
    STATE(81), 1,
      aux_sym__node_field_comment_repeat1,
    STATE(93), 1,
      sym__escline,
    STATE(102), 1,
      sym__node_space,
    STATE(84), 2,
      sym__ws,
      aux_sym__node_space_repeat1,
    ACTIONS(53), 3,
      sym_multi_line_comment,
      sym__bom,
      sym__unicode_space,
    STATE(167), 3,
      sym__node_terminator,
      sym__newline,
      sym_single_line_comment,
    ACTIONS(1315), 10,
      sym__eof,
      sym__implicit_terminator,
      anon_sym_SEMI,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
  [21502] = 9,
    ACTIONS(23), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(51), 1,
      anon_sym_BSLASH,
    STATE(81), 1,
      aux_sym__node_field_comment_repeat1,
    STATE(93), 1,
      sym__escline,
    STATE(102), 1,
      sym__node_space,
    STATE(84), 2,
      sym__ws,
      aux_sym__node_space_repeat1,
    ACTIONS(53), 3,
      sym_multi_line_comment,
      sym__bom,
      sym__unicode_space,
    STATE(125), 3,
      sym__node_terminator,
      sym__newline,
      sym_single_line_comment,
    ACTIONS(1317), 10,
      sym__eof,
      sym__implicit_terminator,
      anon_sym_SEMI,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
  [21544] = 9,
    ACTIONS(23), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(51), 1,
      anon_sym_BSLASH,
    STATE(102), 1,
      sym__node_space,
    STATE(412), 1,
      aux_sym__node_field_comment_repeat1,
    STATE(513), 1,
      sym__escline,
    STATE(489), 2,
      sym__ws,
      aux_sym__node_space_repeat1,
    ACTIONS(1086), 3,
      sym_multi_line_comment,
      sym__bom,
      sym__unicode_space,
    STATE(125), 3,
      sym__node_terminator,
      sym__newline,
      sym_single_line_comment,
    ACTIONS(1317), 10,
      sym__eof,
      sym__implicit_terminator,
      anon_sym_SEMI,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
  [21586] = 9,
    ACTIONS(23), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(51), 1,
      anon_sym_BSLASH,
    STATE(93), 1,
      sym__escline,
    STATE(102), 1,
      sym__node_space,
    STATE(409), 1,
      aux_sym__node_field_comment_repeat1,
    STATE(405), 2,
      sym__ws,
      aux_sym__node_space_repeat1,
    ACTIONS(1321), 3,
      sym_multi_line_comment,
      sym__bom,
      sym__unicode_space,
    STATE(123), 3,
      sym__node_terminator,
      sym__newline,
      sym_single_line_comment,
    ACTIONS(1319), 10,
      sym__eof,
      sym__implicit_terminator,
      anon_sym_SEMI,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
  [21628] = 8,
    ACTIONS(1325), 1,
      anon_sym_BSLASH,
    STATE(60), 1,
      aux_sym__node_field_comment_repeat1,
    STATE(98), 1,
      sym__escline,
    STATE(267), 1,
      sym__node_space,
    STATE(416), 1,
      aux_sym_kdl_node_repeat1,
    STATE(95), 2,
      sym__ws,
      aux_sym__node_space_repeat1,
    ACTIONS(1328), 3,
      sym_multi_line_comment,
      sym__bom,
      sym__unicode_space,
    ACTIONS(1323), 13,
      sym__eof,
      sym__implicit_terminator,
      anon_sym_SLASH_DASH,
      anon_sym_LBRACE,
      anon_sym_SEMI,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      anon_sym_SLASH_SLASH,
  [21668] = 9,
    ACTIONS(23), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(51), 1,
      anon_sym_BSLASH,
    STATE(102), 1,
      sym__node_space,
    STATE(345), 1,
      aux_sym__node_field_comment_repeat1,
    STATE(513), 1,
      sym__escline,
    STATE(489), 2,
      sym__ws,
      aux_sym__node_space_repeat1,
    ACTIONS(1086), 3,
      sym_multi_line_comment,
      sym__bom,
      sym__unicode_space,
    STATE(161), 3,
      sym__node_terminator,
      sym__newline,
      sym_single_line_comment,
    ACTIONS(1331), 10,
      sym__eof,
      sym__implicit_terminator,
      anon_sym_SEMI,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
  [21710] = 9,
    ACTIONS(23), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(51), 1,
      anon_sym_BSLASH,
    STATE(81), 1,
      aux_sym__node_field_comment_repeat1,
    STATE(93), 1,
      sym__escline,
    STATE(102), 1,
      sym__node_space,
    STATE(84), 2,
      sym__ws,
      aux_sym__node_space_repeat1,
    ACTIONS(53), 3,
      sym_multi_line_comment,
      sym__bom,
      sym__unicode_space,
    STATE(122), 3,
      sym__node_terminator,
      sym__newline,
      sym_single_line_comment,
    ACTIONS(1333), 10,
      sym__eof,
      sym__implicit_terminator,
      anon_sym_SEMI,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
  [21752] = 9,
    ACTIONS(23), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(51), 1,
      anon_sym_BSLASH,
    STATE(102), 1,
      sym__node_space,
    STATE(403), 1,
      aux_sym__node_field_comment_repeat1,
    STATE(513), 1,
      sym__escline,
    STATE(489), 2,
      sym__ws,
      aux_sym__node_space_repeat1,
    ACTIONS(1086), 3,
      sym_multi_line_comment,
      sym__bom,
      sym__unicode_space,
    STATE(122), 3,
      sym__node_terminator,
      sym__newline,
      sym_single_line_comment,
    ACTIONS(1333), 10,
      sym__eof,
      sym__implicit_terminator,
      anon_sym_SEMI,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
  [21794] = 3,
    STATE(328), 1,
      aux_sym__integer_repeat1,
    ACTIONS(1337), 2,
      anon_sym__,
      sym__digit,
    ACTIONS(1335), 20,
      sym__eof,
      sym_multi_line_comment,
      sym__implicit_terminator,
      anon_sym_SLASH_DASH,
      anon_sym_LBRACE,
      anon_sym_SEMI,
      anon_sym_DOT,
      anon_sym_e,
      anon_sym_E,
      anon_sym_BSLASH,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
      anon_sym_SLASH_SLASH,
  [21824] = 9,
    ACTIONS(23), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(51), 1,
      anon_sym_BSLASH,
    STATE(93), 1,
      sym__escline,
    STATE(102), 1,
      sym__node_space,
    STATE(401), 1,
      aux_sym__node_field_comment_repeat1,
    STATE(398), 2,
      sym__ws,
      aux_sym__node_space_repeat1,
    ACTIONS(1341), 3,
      sym_multi_line_comment,
      sym__bom,
      sym__unicode_space,
    STATE(121), 3,
      sym__node_terminator,
      sym__newline,
      sym_single_line_comment,
    ACTIONS(1339), 10,
      sym__eof,
      sym__implicit_terminator,
      anon_sym_SEMI,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
  [21866] = 9,
    ACTIONS(23), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(51), 1,
      anon_sym_BSLASH,
    STATE(81), 1,
      aux_sym__node_field_comment_repeat1,
    STATE(93), 1,
      sym__escline,
    STATE(102), 1,
      sym__node_space,
    STATE(84), 2,
      sym__ws,
      aux_sym__node_space_repeat1,
    ACTIONS(53), 3,
      sym_multi_line_comment,
      sym__bom,
      sym__unicode_space,
    STATE(161), 3,
      sym__node_terminator,
      sym__newline,
      sym_single_line_comment,
    ACTIONS(1331), 10,
      sym__eof,
      sym__implicit_terminator,
      anon_sym_SEMI,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
  [21908] = 9,
    ACTIONS(23), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(51), 1,
      anon_sym_BSLASH,
    STATE(81), 1,
      aux_sym__node_field_comment_repeat1,
    STATE(93), 1,
      sym__escline,
    STATE(102), 1,
      sym__node_space,
    STATE(84), 2,
      sym__ws,
      aux_sym__node_space_repeat1,
    ACTIONS(53), 3,
      sym_multi_line_comment,
      sym__bom,
      sym__unicode_space,
    STATE(155), 3,
      sym__node_terminator,
      sym__newline,
      sym_single_line_comment,
    ACTIONS(1343), 10,
      sym__eof,
      sym__implicit_terminator,
      anon_sym_SEMI,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
  [21950] = 9,
    ACTIONS(23), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(51), 1,
      anon_sym_BSLASH,
    STATE(81), 1,
      aux_sym__node_field_comment_repeat1,
    STATE(93), 1,
      sym__escline,
    STATE(102), 1,
      sym__node_space,
    STATE(84), 2,
      sym__ws,
      aux_sym__node_space_repeat1,
    ACTIONS(53), 3,
      sym_multi_line_comment,
      sym__bom,
      sym__unicode_space,
    STATE(118), 3,
      sym__node_terminator,
      sym__newline,
      sym_single_line_comment,
    ACTIONS(1345), 10,
      sym__eof,
      sym__implicit_terminator,
      anon_sym_SEMI,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
  [21992] = 9,
    ACTIONS(23), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(51), 1,
      anon_sym_BSLASH,
    STATE(102), 1,
      sym__node_space,
    STATE(395), 1,
      aux_sym__node_field_comment_repeat1,
    STATE(513), 1,
      sym__escline,
    STATE(489), 2,
      sym__ws,
      aux_sym__node_space_repeat1,
    ACTIONS(1086), 3,
      sym_multi_line_comment,
      sym__bom,
      sym__unicode_space,
    STATE(118), 3,
      sym__node_terminator,
      sym__newline,
      sym_single_line_comment,
    ACTIONS(1345), 10,
      sym__eof,
      sym__implicit_terminator,
      anon_sym_SEMI,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
  [22034] = 9,
    ACTIONS(23), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(51), 1,
      anon_sym_BSLASH,
    STATE(93), 1,
      sym__escline,
    STATE(102), 1,
      sym__node_space,
    STATE(393), 1,
      aux_sym__node_field_comment_repeat1,
    STATE(382), 2,
      sym__ws,
      aux_sym__node_space_repeat1,
    ACTIONS(1349), 3,
      sym_multi_line_comment,
      sym__bom,
      sym__unicode_space,
    STATE(117), 3,
      sym__node_terminator,
      sym__newline,
      sym_single_line_comment,
    ACTIONS(1347), 10,
      sym__eof,
      sym__implicit_terminator,
      anon_sym_SEMI,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
  [22076] = 9,
    ACTIONS(23), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(51), 1,
      anon_sym_BSLASH,
    STATE(81), 1,
      aux_sym__node_field_comment_repeat1,
    STATE(93), 1,
      sym__escline,
    STATE(102), 1,
      sym__node_space,
    STATE(84), 2,
      sym__ws,
      aux_sym__node_space_repeat1,
    ACTIONS(53), 3,
      sym_multi_line_comment,
      sym__bom,
      sym__unicode_space,
    STATE(175), 3,
      sym__node_terminator,
      sym__newline,
      sym_single_line_comment,
    ACTIONS(1351), 10,
      sym__eof,
      sym__implicit_terminator,
      anon_sym_SEMI,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
  [22118] = 9,
    ACTIONS(23), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(51), 1,
      anon_sym_BSLASH,
    STATE(81), 1,
      aux_sym__node_field_comment_repeat1,
    STATE(93), 1,
      sym__escline,
    STATE(102), 1,
      sym__node_space,
    STATE(84), 2,
      sym__ws,
      aux_sym__node_space_repeat1,
    ACTIONS(53), 3,
      sym_multi_line_comment,
      sym__bom,
      sym__unicode_space,
    STATE(238), 3,
      sym__node_terminator,
      sym__newline,
      sym_single_line_comment,
    ACTIONS(1353), 10,
      sym__eof,
      sym__implicit_terminator,
      anon_sym_SEMI,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
  [22160] = 9,
    ACTIONS(23), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(51), 1,
      anon_sym_BSLASH,
    STATE(102), 1,
      sym__node_space,
    STATE(346), 1,
      aux_sym__node_field_comment_repeat1,
    STATE(513), 1,
      sym__escline,
    STATE(489), 2,
      sym__ws,
      aux_sym__node_space_repeat1,
    ACTIONS(1086), 3,
      sym_multi_line_comment,
      sym__bom,
      sym__unicode_space,
    STATE(151), 3,
      sym__node_terminator,
      sym__newline,
      sym_single_line_comment,
    ACTIONS(1355), 10,
      sym__eof,
      sym__implicit_terminator,
      anon_sym_SEMI,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
  [22202] = 9,
    ACTIONS(23), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(51), 1,
      anon_sym_BSLASH,
    STATE(81), 1,
      aux_sym__node_field_comment_repeat1,
    STATE(93), 1,
      sym__escline,
    STATE(102), 1,
      sym__node_space,
    STATE(84), 2,
      sym__ws,
      aux_sym__node_space_repeat1,
    ACTIONS(53), 3,
      sym_multi_line_comment,
      sym__bom,
      sym__unicode_space,
    STATE(151), 3,
      sym__node_terminator,
      sym__newline,
      sym_single_line_comment,
    ACTIONS(1355), 10,
      sym__eof,
      sym__implicit_terminator,
      anon_sym_SEMI,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
  [22244] = 9,
    ACTIONS(23), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(51), 1,
      anon_sym_BSLASH,
    STATE(102), 1,
      sym__node_space,
    STATE(371), 1,
      aux_sym__node_field_comment_repeat1,
    STATE(513), 1,
      sym__escline,
    STATE(489), 2,
      sym__ws,
      aux_sym__node_space_repeat1,
    ACTIONS(1086), 3,
      sym_multi_line_comment,
      sym__bom,
      sym__unicode_space,
    STATE(238), 3,
      sym__node_terminator,
      sym__newline,
      sym_single_line_comment,
    ACTIONS(1353), 10,
      sym__eof,
      sym__implicit_terminator,
      anon_sym_SEMI,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
  [22286] = 9,
    ACTIONS(23), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(51), 1,
      anon_sym_BSLASH,
    STATE(81), 1,
      aux_sym__node_field_comment_repeat1,
    STATE(93), 1,
      sym__escline,
    STATE(102), 1,
      sym__node_space,
    STATE(84), 2,
      sym__ws,
      aux_sym__node_space_repeat1,
    ACTIONS(53), 3,
      sym_multi_line_comment,
      sym__bom,
      sym__unicode_space,
    STATE(148), 3,
      sym__node_terminator,
      sym__newline,
      sym_single_line_comment,
    ACTIONS(1357), 10,
      sym__eof,
      sym__implicit_terminator,
      anon_sym_SEMI,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
  [22328] = 9,
    ACTIONS(23), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(51), 1,
      anon_sym_BSLASH,
    STATE(102), 1,
      sym__node_space,
    STATE(349), 1,
      aux_sym__node_field_comment_repeat1,
    STATE(513), 1,
      sym__escline,
    STATE(489), 2,
      sym__ws,
      aux_sym__node_space_repeat1,
    ACTIONS(1086), 3,
      sym_multi_line_comment,
      sym__bom,
      sym__unicode_space,
    STATE(146), 3,
      sym__node_terminator,
      sym__newline,
      sym_single_line_comment,
    ACTIONS(1359), 10,
      sym__eof,
      sym__implicit_terminator,
      anon_sym_SEMI,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
  [22370] = 9,
    ACTIONS(23), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(51), 1,
      anon_sym_BSLASH,
    STATE(81), 1,
      aux_sym__node_field_comment_repeat1,
    STATE(93), 1,
      sym__escline,
    STATE(102), 1,
      sym__node_space,
    STATE(84), 2,
      sym__ws,
      aux_sym__node_space_repeat1,
    ACTIONS(53), 3,
      sym_multi_line_comment,
      sym__bom,
      sym__unicode_space,
    STATE(146), 3,
      sym__node_terminator,
      sym__newline,
      sym_single_line_comment,
    ACTIONS(1359), 10,
      sym__eof,
      sym__implicit_terminator,
      anon_sym_SEMI,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
  [22412] = 9,
    ACTIONS(23), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(51), 1,
      anon_sym_BSLASH,
    STATE(102), 1,
      sym__node_space,
    STATE(351), 1,
      aux_sym__node_field_comment_repeat1,
    STATE(513), 1,
      sym__escline,
    STATE(489), 2,
      sym__ws,
      aux_sym__node_space_repeat1,
    ACTIONS(1086), 3,
      sym_multi_line_comment,
      sym__bom,
      sym__unicode_space,
    STATE(142), 3,
      sym__node_terminator,
      sym__newline,
      sym_single_line_comment,
    ACTIONS(1361), 10,
      sym__eof,
      sym__implicit_terminator,
      anon_sym_SEMI,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
  [22454] = 9,
    ACTIONS(23), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(51), 1,
      anon_sym_BSLASH,
    STATE(81), 1,
      aux_sym__node_field_comment_repeat1,
    STATE(93), 1,
      sym__escline,
    STATE(102), 1,
      sym__node_space,
    STATE(84), 2,
      sym__ws,
      aux_sym__node_space_repeat1,
    ACTIONS(53), 3,
      sym_multi_line_comment,
      sym__bom,
      sym__unicode_space,
    STATE(142), 3,
      sym__node_terminator,
      sym__newline,
      sym_single_line_comment,
    ACTIONS(1361), 10,
      sym__eof,
      sym__implicit_terminator,
      anon_sym_SEMI,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
  [22496] = 9,
    ACTIONS(23), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(51), 1,
      anon_sym_BSLASH,
    STATE(93), 1,
      sym__escline,
    STATE(102), 1,
      sym__node_space,
    STATE(369), 1,
      aux_sym__node_field_comment_repeat1,
    STATE(368), 2,
      sym__ws,
      aux_sym__node_space_repeat1,
    ACTIONS(1365), 3,
      sym_multi_line_comment,
      sym__bom,
      sym__unicode_space,
    STATE(241), 3,
      sym__node_terminator,
      sym__newline,
      sym_single_line_comment,
    ACTIONS(1363), 10,
      sym__eof,
      sym__implicit_terminator,
      anon_sym_SEMI,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
  [22538] = 9,
    ACTIONS(23), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(51), 1,
      anon_sym_BSLASH,
    STATE(81), 1,
      aux_sym__node_field_comment_repeat1,
    STATE(93), 1,
      sym__escline,
    STATE(102), 1,
      sym__node_space,
    STATE(84), 2,
      sym__ws,
      aux_sym__node_space_repeat1,
    ACTIONS(53), 3,
      sym_multi_line_comment,
      sym__bom,
      sym__unicode_space,
    STATE(139), 3,
      sym__node_terminator,
      sym__newline,
      sym_single_line_comment,
    ACTIONS(1367), 10,
      sym__eof,
      sym__implicit_terminator,
      anon_sym_SEMI,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
  [22580] = 9,
    ACTIONS(23), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(51), 1,
      anon_sym_BSLASH,
    STATE(102), 1,
      sym__node_space,
    STATE(362), 1,
      aux_sym__node_field_comment_repeat1,
    STATE(513), 1,
      sym__escline,
    STATE(489), 2,
      sym__ws,
      aux_sym__node_space_repeat1,
    ACTIONS(1086), 3,
      sym_multi_line_comment,
      sym__bom,
      sym__unicode_space,
    STATE(250), 3,
      sym__node_terminator,
      sym__newline,
      sym_single_line_comment,
    ACTIONS(1205), 10,
      sym__eof,
      sym__implicit_terminator,
      anon_sym_SEMI,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
  [22622] = 9,
    ACTIONS(23), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(51), 1,
      anon_sym_BSLASH,
    STATE(102), 1,
      sym__node_space,
    STATE(363), 1,
      aux_sym__node_field_comment_repeat1,
    STATE(513), 1,
      sym__escline,
    STATE(489), 2,
      sym__ws,
      aux_sym__node_space_repeat1,
    ACTIONS(1086), 3,
      sym_multi_line_comment,
      sym__bom,
      sym__unicode_space,
    STATE(137), 3,
      sym__node_terminator,
      sym__newline,
      sym_single_line_comment,
    ACTIONS(1369), 10,
      sym__eof,
      sym__implicit_terminator,
      anon_sym_SEMI,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
  [22664] = 9,
    ACTIONS(23), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(51), 1,
      anon_sym_BSLASH,
    STATE(81), 1,
      aux_sym__node_field_comment_repeat1,
    STATE(93), 1,
      sym__escline,
    STATE(102), 1,
      sym__node_space,
    STATE(84), 2,
      sym__ws,
      aux_sym__node_space_repeat1,
    ACTIONS(53), 3,
      sym_multi_line_comment,
      sym__bom,
      sym__unicode_space,
    STATE(137), 3,
      sym__node_terminator,
      sym__newline,
      sym_single_line_comment,
    ACTIONS(1369), 10,
      sym__eof,
      sym__implicit_terminator,
      anon_sym_SEMI,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
  [22706] = 9,
    ACTIONS(23), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(51), 1,
      anon_sym_BSLASH,
    STATE(102), 1,
      sym__node_space,
    STATE(360), 1,
      aux_sym__node_field_comment_repeat1,
    STATE(513), 1,
      sym__escline,
    STATE(489), 2,
      sym__ws,
      aux_sym__node_space_repeat1,
    ACTIONS(1086), 3,
      sym_multi_line_comment,
      sym__bom,
      sym__unicode_space,
    STATE(255), 3,
      sym__node_terminator,
      sym__newline,
      sym_single_line_comment,
    ACTIONS(1078), 10,
      sym__eof,
      sym__implicit_terminator,
      anon_sym_SEMI,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
  [22748] = 9,
    ACTIONS(23), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(51), 1,
      anon_sym_BSLASH,
    STATE(81), 1,
      aux_sym__node_field_comment_repeat1,
    STATE(93), 1,
      sym__escline,
    STATE(102), 1,
      sym__node_space,
    STATE(84), 2,
      sym__ws,
      aux_sym__node_space_repeat1,
    ACTIONS(53), 3,
      sym_multi_line_comment,
      sym__bom,
      sym__unicode_space,
    STATE(113), 3,
      sym__node_terminator,
      sym__newline,
      sym_single_line_comment,
    ACTIONS(1371), 10,
      sym__eof,
      sym__implicit_terminator,
      anon_sym_SEMI,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
  [22790] = 9,
    ACTIONS(23), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(51), 1,
      anon_sym_BSLASH,
    STATE(81), 1,
      aux_sym__node_field_comment_repeat1,
    STATE(93), 1,
      sym__escline,
    STATE(102), 1,
      sym__node_space,
    STATE(84), 2,
      sym__ws,
      aux_sym__node_space_repeat1,
    ACTIONS(53), 3,
      sym_multi_line_comment,
      sym__bom,
      sym__unicode_space,
    STATE(204), 3,
      sym__node_terminator,
      sym__newline,
      sym_single_line_comment,
    ACTIONS(1373), 10,
      sym__eof,
      sym__implicit_terminator,
      anon_sym_SEMI,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
  [22832] = 9,
    ACTIONS(23), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(51), 1,
      anon_sym_BSLASH,
    STATE(93), 1,
      sym__escline,
    STATE(102), 1,
      sym__node_space,
    STATE(357), 1,
      aux_sym__node_field_comment_repeat1,
    STATE(355), 2,
      sym__ws,
      aux_sym__node_space_repeat1,
    ACTIONS(1377), 3,
      sym_multi_line_comment,
      sym__bom,
      sym__unicode_space,
    STATE(257), 3,
      sym__node_terminator,
      sym__newline,
      sym_single_line_comment,
    ACTIONS(1375), 10,
      sym__eof,
      sym__implicit_terminator,
      anon_sym_SEMI,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
  [22874] = 9,
    ACTIONS(23), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(51), 1,
      anon_sym_BSLASH,
    STATE(81), 1,
      aux_sym__node_field_comment_repeat1,
    STATE(93), 1,
      sym__escline,
    STATE(102), 1,
      sym__node_space,
    STATE(84), 2,
      sym__ws,
      aux_sym__node_space_repeat1,
    ACTIONS(53), 3,
      sym_multi_line_comment,
      sym__bom,
      sym__unicode_space,
    STATE(262), 3,
      sym__node_terminator,
      sym__newline,
      sym_single_line_comment,
    ACTIONS(1379), 10,
      sym__eof,
      sym__implicit_terminator,
      anon_sym_SEMI,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
  [22916] = 9,
    ACTIONS(23), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(51), 1,
      anon_sym_BSLASH,
    STATE(81), 1,
      aux_sym__node_field_comment_repeat1,
    STATE(93), 1,
      sym__escline,
    STATE(102), 1,
      sym__node_space,
    STATE(84), 2,
      sym__ws,
      aux_sym__node_space_repeat1,
    ACTIONS(53), 3,
      sym_multi_line_comment,
      sym__bom,
      sym__unicode_space,
    STATE(109), 3,
      sym__node_terminator,
      sym__newline,
      sym_single_line_comment,
    ACTIONS(1381), 10,
      sym__eof,
      sym__implicit_terminator,
      anon_sym_SEMI,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
  [22958] = 9,
    ACTIONS(23), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(51), 1,
      anon_sym_BSLASH,
    STATE(102), 1,
      sym__node_space,
    STATE(353), 1,
      aux_sym__node_field_comment_repeat1,
    STATE(513), 1,
      sym__escline,
    STATE(489), 2,
      sym__ws,
      aux_sym__node_space_repeat1,
    ACTIONS(1086), 3,
      sym_multi_line_comment,
      sym__bom,
      sym__unicode_space,
    STATE(262), 3,
      sym__node_terminator,
      sym__newline,
      sym_single_line_comment,
    ACTIONS(1379), 10,
      sym__eof,
      sym__implicit_terminator,
      anon_sym_SEMI,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
  [23000] = 9,
    ACTIONS(23), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(51), 1,
      anon_sym_BSLASH,
    STATE(93), 1,
      sym__escline,
    STATE(102), 1,
      sym__node_space,
    STATE(352), 1,
      aux_sym__node_field_comment_repeat1,
    STATE(350), 2,
      sym__ws,
      aux_sym__node_space_repeat1,
    ACTIONS(1385), 3,
      sym_multi_line_comment,
      sym__bom,
      sym__unicode_space,
    STATE(264), 3,
      sym__node_terminator,
      sym__newline,
      sym_single_line_comment,
    ACTIONS(1383), 10,
      sym__eof,
      sym__implicit_terminator,
      anon_sym_SEMI,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
  [23042] = 9,
    ACTIONS(23), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(51), 1,
      anon_sym_BSLASH,
    STATE(81), 1,
      aux_sym__node_field_comment_repeat1,
    STATE(93), 1,
      sym__escline,
    STATE(102), 1,
      sym__node_space,
    STATE(84), 2,
      sym__ws,
      aux_sym__node_space_repeat1,
    ACTIONS(53), 3,
      sym_multi_line_comment,
      sym__bom,
      sym__unicode_space,
    STATE(110), 3,
      sym__node_terminator,
      sym__newline,
      sym_single_line_comment,
    ACTIONS(1387), 10,
      sym__eof,
      sym__implicit_terminator,
      anon_sym_SEMI,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
  [23084] = 9,
    ACTIONS(23), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(51), 1,
      anon_sym_BSLASH,
    STATE(93), 1,
      sym__escline,
    STATE(102), 1,
      sym__node_space,
    STATE(317), 1,
      aux_sym__node_field_comment_repeat1,
    STATE(318), 2,
      sym__ws,
      aux_sym__node_space_repeat1,
    ACTIONS(1391), 3,
      sym_multi_line_comment,
      sym__bom,
      sym__unicode_space,
    STATE(199), 3,
      sym__node_terminator,
      sym__newline,
      sym_single_line_comment,
    ACTIONS(1389), 10,
      sym__eof,
      sym__implicit_terminator,
      anon_sym_SEMI,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
  [23126] = 9,
    ACTIONS(23), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(51), 1,
      anon_sym_BSLASH,
    STATE(81), 1,
      aux_sym__node_field_comment_repeat1,
    STATE(93), 1,
      sym__escline,
    STATE(102), 1,
      sym__node_space,
    STATE(84), 2,
      sym__ws,
      aux_sym__node_space_repeat1,
    ACTIONS(53), 3,
      sym_multi_line_comment,
      sym__bom,
      sym__unicode_space,
    STATE(210), 3,
      sym__node_terminator,
      sym__newline,
      sym_single_line_comment,
    ACTIONS(1393), 10,
      sym__eof,
      sym__implicit_terminator,
      anon_sym_SEMI,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
  [23168] = 9,
    ACTIONS(23), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(51), 1,
      anon_sym_BSLASH,
    STATE(93), 1,
      sym__escline,
    STATE(102), 1,
      sym__node_space,
    STATE(314), 1,
      aux_sym__node_field_comment_repeat1,
    STATE(315), 2,
      sym__ws,
      aux_sym__node_space_repeat1,
    ACTIONS(1397), 3,
      sym_multi_line_comment,
      sym__bom,
      sym__unicode_space,
    STATE(124), 3,
      sym__node_terminator,
      sym__newline,
      sym_single_line_comment,
    ACTIONS(1395), 10,
      sym__eof,
      sym__implicit_terminator,
      anon_sym_SEMI,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
  [23210] = 9,
    ACTIONS(23), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(51), 1,
      anon_sym_BSLASH,
    STATE(93), 1,
      sym__escline,
    STATE(102), 1,
      sym__node_space,
    STATE(311), 1,
      aux_sym__node_field_comment_repeat1,
    STATE(312), 2,
      sym__ws,
      aux_sym__node_space_repeat1,
    ACTIONS(1401), 3,
      sym_multi_line_comment,
      sym__bom,
      sym__unicode_space,
    STATE(176), 3,
      sym__node_terminator,
      sym__newline,
      sym_single_line_comment,
    ACTIONS(1399), 10,
      sym__eof,
      sym__implicit_terminator,
      anon_sym_SEMI,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
  [23252] = 9,
    ACTIONS(23), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(51), 1,
      anon_sym_BSLASH,
    STATE(81), 1,
      aux_sym__node_field_comment_repeat1,
    STATE(93), 1,
      sym__escline,
    STATE(102), 1,
      sym__node_space,
    STATE(84), 2,
      sym__ws,
      aux_sym__node_space_repeat1,
    ACTIONS(53), 3,
      sym_multi_line_comment,
      sym__bom,
      sym__unicode_space,
    STATE(232), 3,
      sym__node_terminator,
      sym__newline,
      sym_single_line_comment,
    ACTIONS(1403), 10,
      sym__eof,
      sym__implicit_terminator,
      anon_sym_SEMI,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
  [23294] = 9,
    ACTIONS(23), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(51), 1,
      anon_sym_BSLASH,
    STATE(81), 1,
      aux_sym__node_field_comment_repeat1,
    STATE(93), 1,
      sym__escline,
    STATE(102), 1,
      sym__node_space,
    STATE(84), 2,
      sym__ws,
      aux_sym__node_space_repeat1,
    ACTIONS(53), 3,
      sym_multi_line_comment,
      sym__bom,
      sym__unicode_space,
    STATE(112), 3,
      sym__node_terminator,
      sym__newline,
      sym_single_line_comment,
    ACTIONS(1405), 10,
      sym__eof,
      sym__implicit_terminator,
      anon_sym_SEMI,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
  [23336] = 9,
    ACTIONS(23), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(51), 1,
      anon_sym_BSLASH,
    STATE(93), 1,
      sym__escline,
    STATE(102), 1,
      sym__node_space,
    STATE(305), 1,
      aux_sym__node_field_comment_repeat1,
    STATE(306), 2,
      sym__ws,
      aux_sym__node_space_repeat1,
    ACTIONS(1409), 3,
      sym_multi_line_comment,
      sym__bom,
      sym__unicode_space,
    STATE(189), 3,
      sym__node_terminator,
      sym__newline,
      sym_single_line_comment,
    ACTIONS(1407), 10,
      sym__eof,
      sym__implicit_terminator,
      anon_sym_SEMI,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
  [23378] = 9,
    ACTIONS(23), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(51), 1,
      anon_sym_BSLASH,
    STATE(93), 1,
      sym__escline,
    STATE(102), 1,
      sym__node_space,
    STATE(302), 1,
      aux_sym__node_field_comment_repeat1,
    STATE(303), 2,
      sym__ws,
      aux_sym__node_space_repeat1,
    ACTIONS(1413), 3,
      sym_multi_line_comment,
      sym__bom,
      sym__unicode_space,
    STATE(194), 3,
      sym__node_terminator,
      sym__newline,
      sym_single_line_comment,
    ACTIONS(1411), 10,
      sym__eof,
      sym__implicit_terminator,
      anon_sym_SEMI,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
  [23420] = 9,
    ACTIONS(23), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(51), 1,
      anon_sym_BSLASH,
    STATE(81), 1,
      aux_sym__node_field_comment_repeat1,
    STATE(93), 1,
      sym__escline,
    STATE(102), 1,
      sym__node_space,
    STATE(84), 2,
      sym__ws,
      aux_sym__node_space_repeat1,
    ACTIONS(53), 3,
      sym_multi_line_comment,
      sym__bom,
      sym__unicode_space,
    STATE(111), 3,
      sym__node_terminator,
      sym__newline,
      sym_single_line_comment,
    ACTIONS(1415), 10,
      sym__eof,
      sym__implicit_terminator,
      anon_sym_SEMI,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
  [23462] = 9,
    ACTIONS(23), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(51), 1,
      anon_sym_BSLASH,
    STATE(102), 1,
      sym__node_space,
    STATE(455), 1,
      aux_sym__node_field_comment_repeat1,
    STATE(513), 1,
      sym__escline,
    STATE(489), 2,
      sym__ws,
      aux_sym__node_space_repeat1,
    ACTIONS(1086), 3,
      sym_multi_line_comment,
      sym__bom,
      sym__unicode_space,
    STATE(210), 3,
      sym__node_terminator,
      sym__newline,
      sym_single_line_comment,
    ACTIONS(1393), 10,
      sym__eof,
      sym__implicit_terminator,
      anon_sym_SEMI,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
  [23504] = 12,
    ACTIONS(3), 1,
      sym_multi_line_comment,
    ACTIONS(7), 1,
      sym__normal_bare_identifier,
    ACTIONS(13), 1,
      anon_sym_DQUOTE,
    ACTIONS(15), 1,
      aux_sym__raw_string_token1,
    ACTIONS(17), 1,
      aux_sym__raw_string_token3,
    ACTIONS(370), 1,
      anon_sym_constraint,
    STATE(279), 1,
      sym_identifier,
    STATE(493), 1,
      sym__sign,
    ACTIONS(19), 2,
      anon_sym_PLUS,
      anon_sym_DASH,
    STATE(491), 2,
      sym__escaped_string,
      sym__raw_string,
    STATE(510), 2,
      sym__bare_identifier,
      sym_string,
    ACTIONS(368), 8,
      anon_sym_expression,
      anon_sym_minimize,
      anon_sym_maximize,
      anon_sym_expr,
      anon_sym_filter,
      anon_sym_if,
      anon_sym_lower,
      anon_sym_upper,
  [23551] = 12,
    ACTIONS(3), 1,
      sym_multi_line_comment,
    ACTIONS(7), 1,
      sym__normal_bare_identifier,
    ACTIONS(13), 1,
      anon_sym_DQUOTE,
    ACTIONS(15), 1,
      aux_sym__raw_string_token1,
    ACTIONS(17), 1,
      aux_sym__raw_string_token3,
    ACTIONS(1419), 1,
      anon_sym_constraint,
    STATE(293), 1,
      sym_identifier,
    STATE(493), 1,
      sym__sign,
    ACTIONS(19), 2,
      anon_sym_PLUS,
      anon_sym_DASH,
    STATE(491), 2,
      sym__escaped_string,
      sym__raw_string,
    STATE(510), 2,
      sym__bare_identifier,
      sym_string,
    ACTIONS(1417), 8,
      anon_sym_expression,
      anon_sym_minimize,
      anon_sym_maximize,
      anon_sym_expr,
      anon_sym_filter,
      anon_sym_if,
      anon_sym_lower,
      anon_sym_upper,
  [23598] = 12,
    ACTIONS(3), 1,
      sym_multi_line_comment,
    ACTIONS(7), 1,
      sym__normal_bare_identifier,
    ACTIONS(13), 1,
      anon_sym_DQUOTE,
    ACTIONS(15), 1,
      aux_sym__raw_string_token1,
    ACTIONS(17), 1,
      aux_sym__raw_string_token3,
    ACTIONS(1423), 1,
      anon_sym_constraint,
    STATE(280), 1,
      sym_identifier,
    STATE(493), 1,
      sym__sign,
    ACTIONS(19), 2,
      anon_sym_PLUS,
      anon_sym_DASH,
    STATE(491), 2,
      sym__escaped_string,
      sym__raw_string,
    STATE(510), 2,
      sym__bare_identifier,
      sym_string,
    ACTIONS(1421), 8,
      anon_sym_expression,
      anon_sym_minimize,
      anon_sym_maximize,
      anon_sym_expr,
      anon_sym_filter,
      anon_sym_if,
      anon_sym_lower,
      anon_sym_upper,
  [23645] = 3,
    STATE(470), 1,
      aux_sym__binary_repeat1,
    ACTIONS(1427), 3,
      anon_sym__,
      anon_sym_0,
      anon_sym_1,
    ACTIONS(1425), 17,
      sym__eof,
      sym_multi_line_comment,
      sym__implicit_terminator,
      anon_sym_SLASH_DASH,
      anon_sym_LBRACE,
      anon_sym_SEMI,
      anon_sym_BSLASH,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
      anon_sym_SLASH_SLASH,
  [23673] = 3,
    STATE(464), 1,
      aux_sym__binary_repeat1,
    ACTIONS(1431), 3,
      anon_sym__,
      anon_sym_0,
      anon_sym_1,
    ACTIONS(1429), 17,
      sym__eof,
      sym_multi_line_comment,
      sym__implicit_terminator,
      anon_sym_SLASH_DASH,
      anon_sym_LBRACE,
      anon_sym_SEMI,
      anon_sym_BSLASH,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
      anon_sym_SLASH_SLASH,
  [23701] = 4,
    ACTIONS(1435), 1,
      anon_sym_DOT,
    STATE(511), 1,
      sym__exponent,
    ACTIONS(1437), 2,
      anon_sym_e,
      anon_sym_E,
    ACTIONS(1433), 17,
      sym__eof,
      sym_multi_line_comment,
      sym__implicit_terminator,
      anon_sym_SLASH_DASH,
      anon_sym_LBRACE,
      anon_sym_SEMI,
      anon_sym_BSLASH,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
      anon_sym_SLASH_SLASH,
  [23731] = 4,
    ACTIONS(1441), 1,
      anon_sym_DOT,
    STATE(502), 1,
      sym__exponent,
    ACTIONS(1437), 2,
      anon_sym_e,
      anon_sym_E,
    ACTIONS(1439), 17,
      sym__eof,
      sym_multi_line_comment,
      sym__implicit_terminator,
      anon_sym_SLASH_DASH,
      anon_sym_LBRACE,
      anon_sym_SEMI,
      anon_sym_BSLASH,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
      anon_sym_SLASH_SLASH,
  [23761] = 3,
    STATE(469), 1,
      aux_sym__binary_repeat1,
    ACTIONS(1445), 3,
      anon_sym__,
      anon_sym_0,
      anon_sym_1,
    ACTIONS(1443), 17,
      sym__eof,
      sym_multi_line_comment,
      sym__implicit_terminator,
      anon_sym_SLASH_DASH,
      anon_sym_LBRACE,
      anon_sym_SEMI,
      anon_sym_BSLASH,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
      anon_sym_SLASH_SLASH,
  [23789] = 3,
    STATE(470), 1,
      aux_sym__binary_repeat1,
    ACTIONS(1427), 3,
      anon_sym__,
      anon_sym_0,
      anon_sym_1,
    ACTIONS(1429), 17,
      sym__eof,
      sym_multi_line_comment,
      sym__implicit_terminator,
      anon_sym_SLASH_DASH,
      anon_sym_LBRACE,
      anon_sym_SEMI,
      anon_sym_BSLASH,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
      anon_sym_SLASH_SLASH,
  [23817] = 3,
    STATE(470), 1,
      aux_sym__binary_repeat1,
    ACTIONS(1449), 3,
      anon_sym__,
      anon_sym_0,
      anon_sym_1,
    ACTIONS(1447), 17,
      sym__eof,
      sym_multi_line_comment,
      sym__implicit_terminator,
      anon_sym_SLASH_DASH,
      anon_sym_LBRACE,
      anon_sym_SEMI,
      anon_sym_BSLASH,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
      anon_sym_SLASH_SLASH,
  [23845] = 3,
    STATE(475), 1,
      aux_sym__octal_repeat1,
    ACTIONS(1454), 2,
      anon_sym__,
      aux_sym__octal_token1,
    ACTIONS(1452), 17,
      sym__eof,
      sym_multi_line_comment,
      sym__implicit_terminator,
      anon_sym_SLASH_DASH,
      anon_sym_LBRACE,
      anon_sym_SEMI,
      anon_sym_BSLASH,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
      anon_sym_SLASH_SLASH,
  [23872] = 3,
    STATE(480), 1,
      aux_sym__octal_repeat1,
    ACTIONS(1456), 2,
      anon_sym__,
      aux_sym__octal_token1,
    ACTIONS(1452), 17,
      sym__eof,
      sym_multi_line_comment,
      sym__implicit_terminator,
      anon_sym_SLASH_DASH,
      anon_sym_LBRACE,
      anon_sym_SEMI,
      anon_sym_BSLASH,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
      anon_sym_SLASH_SLASH,
  [23899] = 3,
    STATE(515), 1,
      sym__exponent,
    ACTIONS(1437), 2,
      anon_sym_e,
      anon_sym_E,
    ACTIONS(1458), 17,
      sym__eof,
      sym_multi_line_comment,
      sym__implicit_terminator,
      anon_sym_SLASH_DASH,
      anon_sym_LBRACE,
      anon_sym_SEMI,
      anon_sym_BSLASH,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
      anon_sym_SLASH_SLASH,
  [23926] = 3,
    ACTIONS(1462), 1,
      sym__identifier_char,
    STATE(477), 1,
      aux_sym__bare_identifier_repeat1,
    ACTIONS(1460), 18,
      sym__eof,
      sym_multi_line_comment,
      sym__implicit_terminator,
      anon_sym_SLASH_DASH,
      anon_sym_LBRACE,
      anon_sym_SEMI,
      anon_sym_EQ,
      anon_sym_BSLASH,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
      anon_sym_SLASH_SLASH,
  [23953] = 3,
    STATE(480), 1,
      aux_sym__octal_repeat1,
    ACTIONS(1456), 2,
      anon_sym__,
      aux_sym__octal_token1,
    ACTIONS(1464), 17,
      sym__eof,
      sym_multi_line_comment,
      sym__implicit_terminator,
      anon_sym_SLASH_DASH,
      anon_sym_LBRACE,
      anon_sym_SEMI,
      anon_sym_BSLASH,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
      anon_sym_SLASH_SLASH,
  [23980] = 3,
    STATE(483), 1,
      aux_sym__hex_repeat1,
    ACTIONS(1468), 2,
      sym__hex_digit,
      anon_sym__,
    ACTIONS(1466), 17,
      sym__eof,
      sym_multi_line_comment,
      sym__implicit_terminator,
      anon_sym_SLASH_DASH,
      anon_sym_LBRACE,
      anon_sym_SEMI,
      anon_sym_BSLASH,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
      anon_sym_SLASH_SLASH,
  [24007] = 3,
    ACTIONS(1472), 1,
      sym__identifier_char,
    STATE(477), 1,
      aux_sym__bare_identifier_repeat1,
    ACTIONS(1470), 18,
      sym__eof,
      sym_multi_line_comment,
      sym__implicit_terminator,
      anon_sym_SLASH_DASH,
      anon_sym_LBRACE,
      anon_sym_SEMI,
      anon_sym_EQ,
      anon_sym_BSLASH,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
      anon_sym_SLASH_SLASH,
  [24034] = 3,
    STATE(476), 1,
      aux_sym__hex_repeat1,
    ACTIONS(1477), 2,
      sym__hex_digit,
      anon_sym__,
    ACTIONS(1475), 17,
      sym__eof,
      sym_multi_line_comment,
      sym__implicit_terminator,
      anon_sym_SLASH_DASH,
      anon_sym_LBRACE,
      anon_sym_SEMI,
      anon_sym_BSLASH,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
      anon_sym_SLASH_SLASH,
  [24061] = 3,
    STATE(522), 1,
      sym__exponent,
    ACTIONS(1437), 2,
      anon_sym_e,
      anon_sym_E,
    ACTIONS(1479), 17,
      sym__eof,
      sym_multi_line_comment,
      sym__implicit_terminator,
      anon_sym_SLASH_DASH,
      anon_sym_LBRACE,
      anon_sym_SEMI,
      anon_sym_BSLASH,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
      anon_sym_SLASH_SLASH,
  [24088] = 3,
    STATE(480), 1,
      aux_sym__octal_repeat1,
    ACTIONS(1483), 2,
      anon_sym__,
      aux_sym__octal_token1,
    ACTIONS(1481), 17,
      sym__eof,
      sym_multi_line_comment,
      sym__implicit_terminator,
      anon_sym_SLASH_DASH,
      anon_sym_LBRACE,
      anon_sym_SEMI,
      anon_sym_BSLASH,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
      anon_sym_SLASH_SLASH,
  [24115] = 3,
    STATE(472), 1,
      aux_sym__octal_repeat1,
    ACTIONS(1488), 2,
      anon_sym__,
      aux_sym__octal_token1,
    ACTIONS(1486), 17,
      sym__eof,
      sym_multi_line_comment,
      sym__implicit_terminator,
      anon_sym_SLASH_DASH,
      anon_sym_LBRACE,
      anon_sym_SEMI,
      anon_sym_BSLASH,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
      anon_sym_SLASH_SLASH,
  [24142] = 3,
    ACTIONS(1492), 1,
      anon_sym_POUND,
    STATE(487), 1,
      aux_sym__raw_string_repeat1,
    ACTIONS(1490), 18,
      sym__eof,
      sym_multi_line_comment,
      sym__implicit_terminator,
      anon_sym_SLASH_DASH,
      anon_sym_LBRACE,
      anon_sym_SEMI,
      anon_sym_EQ,
      anon_sym_BSLASH,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
      anon_sym_SLASH_SLASH,
  [24169] = 3,
    STATE(483), 1,
      aux_sym__hex_repeat1,
    ACTIONS(1496), 2,
      sym__hex_digit,
      anon_sym__,
    ACTIONS(1494), 17,
      sym__eof,
      sym_multi_line_comment,
      sym__implicit_terminator,
      anon_sym_SLASH_DASH,
      anon_sym_LBRACE,
      anon_sym_SEMI,
      anon_sym_BSLASH,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
      anon_sym_SLASH_SLASH,
  [24196] = 3,
    STATE(486), 1,
      aux_sym__hex_repeat1,
    ACTIONS(1501), 2,
      sym__hex_digit,
      anon_sym__,
    ACTIONS(1499), 17,
      sym__eof,
      sym_multi_line_comment,
      sym__implicit_terminator,
      anon_sym_SLASH_DASH,
      anon_sym_LBRACE,
      anon_sym_SEMI,
      anon_sym_BSLASH,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
      anon_sym_SLASH_SLASH,
  [24223] = 3,
    ACTIONS(1505), 1,
      sym__identifier_char,
    STATE(474), 1,
      aux_sym__bare_identifier_repeat1,
    ACTIONS(1503), 18,
      sym__eof,
      sym_multi_line_comment,
      sym__implicit_terminator,
      anon_sym_SLASH_DASH,
      anon_sym_LBRACE,
      anon_sym_SEMI,
      anon_sym_EQ,
      anon_sym_BSLASH,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
      anon_sym_SLASH_SLASH,
  [24250] = 3,
    STATE(483), 1,
      aux_sym__hex_repeat1,
    ACTIONS(1468), 2,
      sym__hex_digit,
      anon_sym__,
    ACTIONS(1475), 17,
      sym__eof,
      sym_multi_line_comment,
      sym__implicit_terminator,
      anon_sym_SLASH_DASH,
      anon_sym_LBRACE,
      anon_sym_SEMI,
      anon_sym_BSLASH,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
      anon_sym_SLASH_SLASH,
  [24277] = 3,
    ACTIONS(1509), 1,
      anon_sym_POUND,
    STATE(487), 1,
      aux_sym__raw_string_repeat1,
    ACTIONS(1507), 18,
      sym__eof,
      sym_multi_line_comment,
      sym__implicit_terminator,
      anon_sym_SLASH_DASH,
      anon_sym_LBRACE,
      anon_sym_SEMI,
      anon_sym_EQ,
      anon_sym_BSLASH,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
      anon_sym_SLASH_SLASH,
  [24304] = 6,
    ACTIONS(210), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(1512), 1,
      anon_sym_RBRACE,
    ACTIONS(1516), 1,
      sym_arco_math_text,
    ACTIONS(1518), 1,
      sym_multi_line_comment,
    STATE(504), 5,
      sym__linespace,
      sym__newline,
      sym__ws,
      sym_single_line_comment,
      aux_sym_document_repeat1,
    ACTIONS(1514), 9,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
  [24335] = 5,
    ACTIONS(1523), 1,
      anon_sym_BSLASH,
    STATE(91), 1,
      sym__escline,
    STATE(90), 2,
      sym__ws,
      aux_sym__node_space_repeat1,
    ACTIONS(1527), 3,
      sym_multi_line_comment,
      sym__bom,
      sym__unicode_space,
    ACTIONS(1520), 11,
      sym__eof,
      sym__implicit_terminator,
      anon_sym_SEMI,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      anon_sym_SLASH_SLASH,
  [24364] = 6,
    ACTIONS(210), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(1531), 1,
      anon_sym_RBRACE,
    ACTIONS(1535), 1,
      sym_arco_math_text,
    ACTIONS(1537), 1,
      sym_multi_line_comment,
    STATE(521), 5,
      sym__linespace,
      sym__newline,
      sym__ws,
      sym_single_line_comment,
      aux_sym_document_repeat1,
    ACTIONS(1533), 9,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
  [24395] = 1,
    ACTIONS(1539), 18,
      sym__eof,
      sym_multi_line_comment,
      sym__implicit_terminator,
      anon_sym_SLASH_DASH,
      anon_sym_LBRACE,
      anon_sym_SEMI,
      anon_sym_EQ,
      anon_sym_BSLASH,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
      anon_sym_SLASH_SLASH,
  [24416] = 2,
    ACTIONS(1543), 1,
      anon_sym_EQ,
    ACTIONS(1541), 17,
      sym__eof,
      sym_multi_line_comment,
      sym__implicit_terminator,
      anon_sym_SLASH_DASH,
      anon_sym_LBRACE,
      anon_sym_SEMI,
      anon_sym_BSLASH,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
      anon_sym_SLASH_SLASH,
  [24439] = 2,
    ACTIONS(1070), 1,
      sym___identifier_char_no_digit,
    ACTIONS(1068), 17,
      sym__eof,
      sym_multi_line_comment,
      sym__implicit_terminator,
      anon_sym_SLASH_DASH,
      anon_sym_LBRACE,
      anon_sym_SEMI,
      anon_sym_BSLASH,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
      anon_sym_SLASH_SLASH,
  [24462] = 1,
    ACTIONS(1545), 18,
      sym__eof,
      sym_multi_line_comment,
      sym__implicit_terminator,
      anon_sym_SLASH_DASH,
      anon_sym_LBRACE,
      anon_sym_SEMI,
      anon_sym_EQ,
      anon_sym_BSLASH,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
      anon_sym_SLASH_SLASH,
  [24483] = 6,
    ACTIONS(210), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(1547), 1,
      anon_sym_RBRACE,
    ACTIONS(1551), 1,
      sym_arco_math_text,
    ACTIONS(1553), 1,
      sym_multi_line_comment,
    STATE(520), 5,
      sym__linespace,
      sym__newline,
      sym__ws,
      sym_single_line_comment,
      aux_sym_document_repeat1,
    ACTIONS(1549), 9,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
  [24514] = 1,
    ACTIONS(1555), 18,
      sym__eof,
      sym_multi_line_comment,
      sym__implicit_terminator,
      anon_sym_SLASH_DASH,
      anon_sym_LBRACE,
      anon_sym_SEMI,
      anon_sym_EQ,
      anon_sym_BSLASH,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
      anon_sym_SLASH_SLASH,
  [24535] = 2,
    ACTIONS(1543), 1,
      anon_sym_EQ,
    ACTIONS(1557), 17,
      sym__eof,
      sym_multi_line_comment,
      sym__implicit_terminator,
      anon_sym_SLASH_DASH,
      anon_sym_LBRACE,
      anon_sym_SEMI,
      anon_sym_BSLASH,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
      anon_sym_SLASH_SLASH,
  [24558] = 1,
    ACTIONS(1490), 18,
      sym__eof,
      sym_multi_line_comment,
      sym__implicit_terminator,
      anon_sym_SLASH_DASH,
      anon_sym_LBRACE,
      anon_sym_SEMI,
      anon_sym_EQ,
      anon_sym_BSLASH,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
      anon_sym_SLASH_SLASH,
  [24579] = 1,
    ACTIONS(1541), 17,
      sym__eof,
      sym_multi_line_comment,
      sym__implicit_terminator,
      anon_sym_SLASH_DASH,
      anon_sym_LBRACE,
      anon_sym_SEMI,
      anon_sym_BSLASH,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
      anon_sym_SLASH_SLASH,
  [24599] = 1,
    ACTIONS(1559), 17,
      sym__eof,
      sym_multi_line_comment,
      sym__implicit_terminator,
      anon_sym_SLASH_DASH,
      anon_sym_LBRACE,
      anon_sym_SEMI,
      anon_sym_BSLASH,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
      anon_sym_SLASH_SLASH,
  [24619] = 1,
    ACTIONS(1561), 17,
      sym__eof,
      sym_multi_line_comment,
      sym__implicit_terminator,
      anon_sym_SLASH_DASH,
      anon_sym_LBRACE,
      anon_sym_SEMI,
      anon_sym_BSLASH,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
      anon_sym_SLASH_SLASH,
  [24639] = 1,
    ACTIONS(1433), 17,
      sym__eof,
      sym_multi_line_comment,
      sym__implicit_terminator,
      anon_sym_SLASH_DASH,
      anon_sym_LBRACE,
      anon_sym_SEMI,
      anon_sym_BSLASH,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
      anon_sym_SLASH_SLASH,
  [24659] = 1,
    ACTIONS(1563), 17,
      sym__eof,
      sym_multi_line_comment,
      sym__implicit_terminator,
      anon_sym_SLASH_DASH,
      anon_sym_LBRACE,
      anon_sym_SEMI,
      anon_sym_BSLASH,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
      anon_sym_SLASH_SLASH,
  [24679] = 4,
    ACTIONS(23), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(1547), 1,
      anon_sym_RBRACE,
    STATE(82), 5,
      sym__linespace,
      sym__newline,
      sym__ws,
      sym_single_line_comment,
      aux_sym_document_repeat1,
    ACTIONS(238), 10,
      sym_multi_line_comment,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
  [24705] = 1,
    ACTIONS(1565), 17,
      sym__eof,
      sym_multi_line_comment,
      sym__implicit_terminator,
      anon_sym_SLASH_DASH,
      anon_sym_LBRACE,
      anon_sym_SEMI,
      anon_sym_BSLASH,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
      anon_sym_SLASH_SLASH,
  [24725] = 1,
    ACTIONS(1567), 17,
      sym__eof,
      sym_multi_line_comment,
      sym__implicit_terminator,
      anon_sym_SLASH_DASH,
      anon_sym_LBRACE,
      anon_sym_SEMI,
      anon_sym_BSLASH,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
      anon_sym_SLASH_SLASH,
  [24745] = 1,
    ACTIONS(1569), 17,
      sym__eof,
      sym_multi_line_comment,
      sym__implicit_terminator,
      anon_sym_SLASH_DASH,
      anon_sym_LBRACE,
      anon_sym_SEMI,
      anon_sym_BSLASH,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
      anon_sym_SLASH_SLASH,
  [24765] = 1,
    ACTIONS(1557), 17,
      sym__eof,
      sym_multi_line_comment,
      sym__implicit_terminator,
      anon_sym_SLASH_DASH,
      anon_sym_LBRACE,
      anon_sym_SEMI,
      anon_sym_BSLASH,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
      anon_sym_SLASH_SLASH,
  [24785] = 1,
    ACTIONS(1571), 17,
      sym__eof,
      sym_multi_line_comment,
      sym__implicit_terminator,
      anon_sym_SLASH_DASH,
      anon_sym_LBRACE,
      anon_sym_SEMI,
      anon_sym_BSLASH,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
      anon_sym_SLASH_SLASH,
  [24805] = 1,
    ACTIONS(1543), 17,
      sym__eof,
      sym_multi_line_comment,
      sym__implicit_terminator,
      anon_sym_SLASH_DASH,
      anon_sym_LBRACE,
      anon_sym_SEMI,
      anon_sym_BSLASH,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
      anon_sym_SLASH_SLASH,
  [24825] = 1,
    ACTIONS(1573), 17,
      sym__eof,
      sym_multi_line_comment,
      sym__implicit_terminator,
      anon_sym_SLASH_DASH,
      anon_sym_LBRACE,
      anon_sym_SEMI,
      anon_sym_BSLASH,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
      anon_sym_SLASH_SLASH,
  [24845] = 1,
    ACTIONS(1575), 17,
      sym__eof,
      sym_multi_line_comment,
      sym__implicit_terminator,
      anon_sym_SLASH_DASH,
      anon_sym_LBRACE,
      anon_sym_SEMI,
      anon_sym_BSLASH,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
      anon_sym_SLASH_SLASH,
  [24865] = 3,
    STATE(517), 2,
      sym__ws,
      aux_sym__node_space_repeat1,
    ACTIONS(1580), 3,
      sym_multi_line_comment,
      sym__bom,
      sym__unicode_space,
    ACTIONS(1577), 12,
      sym__eof,
      sym__implicit_terminator,
      anon_sym_SEMI,
      anon_sym_BSLASH,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      anon_sym_SLASH_SLASH,
  [24889] = 1,
    ACTIONS(1584), 17,
      sym__eof,
      sym_multi_line_comment,
      sym__implicit_terminator,
      anon_sym_SLASH_DASH,
      anon_sym_LBRACE,
      anon_sym_SEMI,
      anon_sym_BSLASH,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
      anon_sym_SLASH_SLASH,
  [24909] = 1,
    ACTIONS(1586), 17,
      sym__eof,
      sym_multi_line_comment,
      sym__implicit_terminator,
      anon_sym_SLASH_DASH,
      anon_sym_LBRACE,
      anon_sym_SEMI,
      anon_sym_BSLASH,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
      anon_sym_SLASH_SLASH,
  [24929] = 1,
    ACTIONS(1588), 17,
      sym__eof,
      sym_multi_line_comment,
      sym__implicit_terminator,
      anon_sym_SLASH_DASH,
      anon_sym_LBRACE,
      anon_sym_SEMI,
      anon_sym_BSLASH,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
      anon_sym_SLASH_SLASH,
  [24949] = 3,
    STATE(90), 2,
      sym__ws,
      aux_sym__node_space_repeat1,
    ACTIONS(1593), 3,
      sym_multi_line_comment,
      sym__bom,
      sym__unicode_space,
    ACTIONS(1590), 12,
      sym__eof,
      sym__implicit_terminator,
      anon_sym_SEMI,
      anon_sym_BSLASH,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      anon_sym_SLASH_SLASH,
  [24973] = 1,
    ACTIONS(1323), 17,
      sym__eof,
      sym_multi_line_comment,
      sym__implicit_terminator,
      anon_sym_SLASH_DASH,
      anon_sym_LBRACE,
      anon_sym_SEMI,
      anon_sym_BSLASH,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
      anon_sym_SLASH_SLASH,
  [24993] = 1,
    ACTIONS(1597), 17,
      sym__eof,
      sym_multi_line_comment,
      sym__implicit_terminator,
      anon_sym_SLASH_DASH,
      anon_sym_LBRACE,
      anon_sym_SEMI,
      anon_sym_BSLASH,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
      anon_sym_SLASH_SLASH,
  [25013] = 4,
    ACTIONS(23), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(1599), 1,
      anon_sym_RBRACE,
    STATE(82), 5,
      sym__linespace,
      sym__newline,
      sym__ws,
      sym_single_line_comment,
      aux_sym_document_repeat1,
    ACTIONS(238), 10,
      sym_multi_line_comment,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
  [25039] = 4,
    ACTIONS(23), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(1601), 1,
      anon_sym_RBRACE,
    STATE(82), 5,
      sym__linespace,
      sym__newline,
      sym__ws,
      sym_single_line_comment,
      aux_sym_document_repeat1,
    ACTIONS(238), 10,
      sym_multi_line_comment,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
  [25065] = 1,
    ACTIONS(1603), 17,
      sym__eof,
      sym_multi_line_comment,
      sym__implicit_terminator,
      anon_sym_SLASH_DASH,
      anon_sym_LBRACE,
      anon_sym_SEMI,
      anon_sym_BSLASH,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
      anon_sym_SLASH_SLASH,
  [25085] = 1,
    ACTIONS(1605), 15,
      sym__eof,
      sym_multi_line_comment,
      sym__implicit_terminator,
      anon_sym_SEMI,
      anon_sym_BSLASH,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
      anon_sym_SLASH_SLASH,
  [25103] = 1,
    ACTIONS(1607), 15,
      sym__eof,
      sym_multi_line_comment,
      sym__implicit_terminator,
      anon_sym_SEMI,
      anon_sym_BSLASH,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
      anon_sym_SLASH_SLASH,
  [25121] = 1,
    ACTIONS(1609), 15,
      sym__eof,
      sym_multi_line_comment,
      sym__implicit_terminator,
      anon_sym_SEMI,
      anon_sym_BSLASH,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
      anon_sym_SLASH_SLASH,
  [25139] = 1,
    ACTIONS(1611), 15,
      sym__eof,
      sym_multi_line_comment,
      sym__implicit_terminator,
      anon_sym_SEMI,
      anon_sym_BSLASH,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
      anon_sym_SLASH_SLASH,
  [25157] = 1,
    ACTIONS(1613), 15,
      sym__eof,
      sym_multi_line_comment,
      sym__implicit_terminator,
      anon_sym_SEMI,
      anon_sym_BSLASH,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
      anon_sym_SLASH_SLASH,
  [25175] = 1,
    ACTIONS(1615), 15,
      sym__eof,
      sym_multi_line_comment,
      sym__implicit_terminator,
      anon_sym_SEMI,
      anon_sym_BSLASH,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
      anon_sym_SLASH_SLASH,
  [25193] = 5,
    ACTIONS(23), 1,
      anon_sym_SLASH_SLASH,
    STATE(79), 2,
      sym__ws,
      aux_sym__node_space_repeat1,
    STATE(266), 2,
      sym__newline,
      sym_single_line_comment,
    ACTIONS(1619), 3,
      sym_multi_line_comment,
      sym__bom,
      sym__unicode_space,
    ACTIONS(1617), 7,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
  [25219] = 1,
    ACTIONS(1621), 15,
      sym__eof,
      sym_multi_line_comment,
      sym__implicit_terminator,
      anon_sym_SEMI,
      anon_sym_BSLASH,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
      anon_sym_SLASH_SLASH,
  [25237] = 1,
    ACTIONS(1623), 15,
      sym__eof,
      sym_multi_line_comment,
      sym__implicit_terminator,
      anon_sym_SEMI,
      anon_sym_BSLASH,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
      anon_sym_SLASH_SLASH,
  [25255] = 1,
    ACTIONS(1625), 15,
      sym__eof,
      sym_multi_line_comment,
      sym__implicit_terminator,
      anon_sym_SEMI,
      anon_sym_BSLASH,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
      anon_sym_SLASH_SLASH,
  [25273] = 1,
    ACTIONS(1627), 15,
      sym__eof,
      sym_multi_line_comment,
      sym__implicit_terminator,
      anon_sym_SEMI,
      anon_sym_BSLASH,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
      anon_sym_SLASH_SLASH,
  [25291] = 5,
    ACTIONS(1631), 1,
      anon_sym_SLASH_SLASH,
    STATE(79), 2,
      sym__ws,
      aux_sym__node_space_repeat1,
    STATE(105), 2,
      sym__newline,
      sym_single_line_comment,
    ACTIONS(1619), 3,
      sym_multi_line_comment,
      sym__bom,
      sym__unicode_space,
    ACTIONS(1629), 7,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
  [25317] = 1,
    ACTIONS(1633), 15,
      sym__eof,
      sym_multi_line_comment,
      sym__implicit_terminator,
      anon_sym_SEMI,
      anon_sym_BSLASH,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
      anon_sym_SLASH_SLASH,
  [25335] = 1,
    ACTIONS(1635), 15,
      sym__eof,
      sym_multi_line_comment,
      sym__implicit_terminator,
      anon_sym_SEMI,
      anon_sym_BSLASH,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
      anon_sym_SLASH_SLASH,
  [25353] = 1,
    ACTIONS(1637), 15,
      sym__eof,
      sym_multi_line_comment,
      sym__implicit_terminator,
      anon_sym_SEMI,
      anon_sym_BSLASH,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
      anon_sym_SLASH_SLASH,
  [25371] = 1,
    ACTIONS(1639), 15,
      sym__eof,
      sym_multi_line_comment,
      sym__implicit_terminator,
      anon_sym_SEMI,
      anon_sym_BSLASH,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
      anon_sym_SLASH_SLASH,
  [25389] = 1,
    ACTIONS(1641), 15,
      sym__eof,
      sym_multi_line_comment,
      sym__implicit_terminator,
      anon_sym_SEMI,
      anon_sym_BSLASH,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
      anon_sym_SLASH_SLASH,
  [25407] = 1,
    ACTIONS(1643), 15,
      sym__eof,
      sym_multi_line_comment,
      sym__implicit_terminator,
      anon_sym_SEMI,
      anon_sym_BSLASH,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
      anon_sym_SLASH_SLASH,
  [25425] = 1,
    ACTIONS(1645), 15,
      sym__eof,
      sym_multi_line_comment,
      sym__implicit_terminator,
      anon_sym_SEMI,
      anon_sym_BSLASH,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
      anon_sym_SLASH_SLASH,
  [25443] = 1,
    ACTIONS(1647), 15,
      sym__eof,
      sym_multi_line_comment,
      sym__implicit_terminator,
      anon_sym_SEMI,
      anon_sym_BSLASH,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
      anon_sym_SLASH_SLASH,
  [25461] = 5,
    ACTIONS(23), 1,
      anon_sym_SLASH_SLASH,
    STATE(265), 2,
      sym__newline,
      sym_single_line_comment,
    STATE(529), 2,
      sym__ws,
      aux_sym__node_space_repeat1,
    ACTIONS(1651), 3,
      sym_multi_line_comment,
      sym__bom,
      sym__unicode_space,
    ACTIONS(1649), 7,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
  [25487] = 1,
    ACTIONS(1653), 15,
      sym__eof,
      sym_multi_line_comment,
      sym__implicit_terminator,
      anon_sym_SEMI,
      anon_sym_BSLASH,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
      anon_sym_SLASH_SLASH,
  [25505] = 1,
    ACTIONS(1655), 15,
      sym__eof,
      sym_multi_line_comment,
      sym__implicit_terminator,
      anon_sym_SEMI,
      anon_sym_BSLASH,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
      anon_sym_SLASH_SLASH,
  [25523] = 5,
    ACTIONS(1631), 1,
      anon_sym_SLASH_SLASH,
    STATE(100), 2,
      sym__newline,
      sym_single_line_comment,
    STATE(534), 2,
      sym__ws,
      aux_sym__node_space_repeat1,
    ACTIONS(1659), 3,
      sym_multi_line_comment,
      sym__bom,
      sym__unicode_space,
    ACTIONS(1657), 7,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
  [25549] = 1,
    ACTIONS(1661), 15,
      sym__eof,
      sym_multi_line_comment,
      sym__implicit_terminator,
      anon_sym_SEMI,
      anon_sym_BSLASH,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
      anon_sym_SLASH_SLASH,
  [25567] = 1,
    ACTIONS(1663), 15,
      sym__eof,
      sym_multi_line_comment,
      sym__implicit_terminator,
      anon_sym_SEMI,
      anon_sym_BSLASH,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
      anon_sym_SLASH_SLASH,
  [25585] = 1,
    ACTIONS(1665), 15,
      sym__eof,
      sym_multi_line_comment,
      sym__implicit_terminator,
      anon_sym_SEMI,
      anon_sym_BSLASH,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
      anon_sym_SLASH_SLASH,
  [25603] = 1,
    ACTIONS(1667), 15,
      sym__eof,
      sym_multi_line_comment,
      sym__implicit_terminator,
      anon_sym_SEMI,
      anon_sym_BSLASH,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
      anon_sym_SLASH_SLASH,
  [25621] = 1,
    ACTIONS(1669), 15,
      sym__eof,
      sym_multi_line_comment,
      sym__implicit_terminator,
      anon_sym_SEMI,
      anon_sym_BSLASH,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
      anon_sym_SLASH_SLASH,
  [25639] = 1,
    ACTIONS(1671), 15,
      sym__eof,
      sym_multi_line_comment,
      sym__implicit_terminator,
      anon_sym_SEMI,
      anon_sym_BSLASH,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
      anon_sym_SLASH_SLASH,
  [25657] = 1,
    ACTIONS(1673), 15,
      sym__eof,
      sym_multi_line_comment,
      sym__implicit_terminator,
      anon_sym_SEMI,
      anon_sym_BSLASH,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
      sym__bom,
      sym__unicode_space,
      anon_sym_SLASH_SLASH,
  [25675] = 5,
    ACTIONS(3), 1,
      sym_multi_line_comment,
    ACTIONS(1677), 1,
      aux_sym_single_line_comment_token1,
    STATE(80), 1,
      sym__newline,
    STATE(558), 1,
      aux_sym_single_line_comment_repeat1,
    ACTIONS(1675), 8,
      sym__eof,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
  [25698] = 5,
    ACTIONS(3), 1,
      sym_multi_line_comment,
    ACTIONS(1681), 1,
      aux_sym_single_line_comment_token1,
    STATE(99), 1,
      sym__newline,
    STATE(556), 1,
      aux_sym_single_line_comment_repeat1,
    ACTIONS(1679), 8,
      sym__eof,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
  [25721] = 5,
    ACTIONS(3), 1,
      sym_multi_line_comment,
    ACTIONS(1677), 1,
      aux_sym_single_line_comment_token1,
    STATE(104), 1,
      sym__newline,
    STATE(558), 1,
      aux_sym_single_line_comment_repeat1,
    ACTIONS(1683), 8,
      sym__eof,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
  [25744] = 5,
    ACTIONS(3), 1,
      sym_multi_line_comment,
    ACTIONS(1687), 1,
      aux_sym_single_line_comment_token1,
    STATE(78), 1,
      sym__newline,
    STATE(554), 1,
      aux_sym_single_line_comment_repeat1,
    ACTIONS(1685), 8,
      sym__eof,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
  [25767] = 4,
    ACTIONS(3), 1,
      sym_multi_line_comment,
    ACTIONS(1691), 1,
      aux_sym_single_line_comment_token1,
    STATE(558), 1,
      aux_sym_single_line_comment_repeat1,
    ACTIONS(1689), 8,
      sym__eof,
      aux_sym__newline_token1,
      aux_sym__newline_token2,
      aux_sym__newline_token3,
      aux_sym__newline_token4,
      aux_sym__newline_token5,
      aux_sym__newline_token6,
      aux_sym__newline_token7,
  [25787] = 7,
    ACTIONS(256), 1,
      anon_sym_BSLASH,
    ACTIONS(270), 1,
      anon_sym_LBRACE,
    STATE(98), 1,
      sym__escline,
    STATE(267), 1,
      sym__node_space,
    STATE(564), 1,
      aux_sym__node_field_comment_repeat1,
    STATE(95), 2,
      sym__ws,
      aux_sym__node_space_repeat1,
    ACTIONS(258), 3,
      sym_multi_line_comment,
      sym__bom,
      sym__unicode_space,
  [25812] = 7,
    ACTIONS(256), 1,
      anon_sym_BSLASH,
    ACTIONS(280), 1,
      anon_sym_LBRACE,
    STATE(86), 1,
      aux_sym__node_field_comment_repeat1,
    STATE(98), 1,
      sym__escline,
    STATE(267), 1,
      sym__node_space,
    STATE(95), 2,
      sym__ws,
      aux_sym__node_space_repeat1,
    ACTIONS(258), 3,
      sym_multi_line_comment,
      sym__bom,
      sym__unicode_space,
  [25837] = 7,
    ACTIONS(256), 1,
      anon_sym_BSLASH,
    ACTIONS(278), 1,
      anon_sym_LBRACE,
    STATE(86), 1,
      aux_sym__node_field_comment_repeat1,
    STATE(98), 1,
      sym__escline,
    STATE(267), 1,
      sym__node_space,
    STATE(95), 2,
      sym__ws,
      aux_sym__node_space_repeat1,
    ACTIONS(258), 3,
      sym_multi_line_comment,
      sym__bom,
      sym__unicode_space,
  [25862] = 7,
    ACTIONS(256), 1,
      anon_sym_BSLASH,
    ACTIONS(272), 1,
      anon_sym_LBRACE,
    STATE(98), 1,
      sym__escline,
    STATE(267), 1,
      sym__node_space,
    STATE(560), 1,
      aux_sym__node_field_comment_repeat1,
    STATE(95), 2,
      sym__ws,
      aux_sym__node_space_repeat1,
    ACTIONS(258), 3,
      sym_multi_line_comment,
      sym__bom,
      sym__unicode_space,
  [25887] = 7,
    ACTIONS(256), 1,
      anon_sym_BSLASH,
    ACTIONS(274), 1,
      anon_sym_LBRACE,
    STATE(98), 1,
      sym__escline,
    STATE(267), 1,
      sym__node_space,
    STATE(561), 1,
      aux_sym__node_field_comment_repeat1,
    STATE(95), 2,
      sym__ws,
      aux_sym__node_space_repeat1,
    ACTIONS(258), 3,
      sym_multi_line_comment,
      sym__bom,
      sym__unicode_space,
  [25912] = 7,
    ACTIONS(256), 1,
      anon_sym_BSLASH,
    ACTIONS(276), 1,
      anon_sym_LBRACE,
    STATE(86), 1,
      aux_sym__node_field_comment_repeat1,
    STATE(98), 1,
      sym__escline,
    STATE(267), 1,
      sym__node_space,
    STATE(95), 2,
      sym__ws,
      aux_sym__node_space_repeat1,
    ACTIONS(258), 3,
      sym_multi_line_comment,
      sym__bom,
      sym__unicode_space,
  [25937] = 5,
    ACTIONS(3), 1,
      sym_multi_line_comment,
    ACTIONS(1694), 1,
      sym__digit,
    STATE(500), 1,
      sym__integer,
    STATE(580), 1,
      sym__sign,
    ACTIONS(1696), 2,
      anon_sym_PLUS,
      anon_sym_DASH,
  [25954] = 5,
    ACTIONS(3), 1,
      sym_multi_line_comment,
    ACTIONS(1698), 1,
      anon_sym_DQUOTE,
    ACTIONS(1700), 1,
      aux_sym__escaped_string_token1,
    ACTIONS(1702), 1,
      sym_escape,
    STATE(569), 1,
      aux_sym__escaped_string_repeat1,
  [25970] = 5,
    ACTIONS(3), 1,
      sym_multi_line_comment,
    ACTIONS(1700), 1,
      aux_sym__escaped_string_token1,
    ACTIONS(1702), 1,
      sym_escape,
    ACTIONS(1704), 1,
      anon_sym_DQUOTE,
    STATE(566), 1,
      aux_sym__escaped_string_repeat1,
  [25986] = 5,
    ACTIONS(3), 1,
      sym_multi_line_comment,
    ACTIONS(1700), 1,
      aux_sym__escaped_string_token1,
    ACTIONS(1702), 1,
      sym_escape,
    ACTIONS(1706), 1,
      anon_sym_DQUOTE,
    STATE(569), 1,
      aux_sym__escaped_string_repeat1,
  [26002] = 5,
    ACTIONS(3), 1,
      sym_multi_line_comment,
    ACTIONS(1708), 1,
      anon_sym_DQUOTE,
    ACTIONS(1710), 1,
      aux_sym__escaped_string_token1,
    ACTIONS(1713), 1,
      sym_escape,
    STATE(569), 1,
      aux_sym__escaped_string_repeat1,
  [26018] = 5,
    ACTIONS(3), 1,
      sym_multi_line_comment,
    ACTIONS(1700), 1,
      aux_sym__escaped_string_token1,
    ACTIONS(1702), 1,
      sym_escape,
    ACTIONS(1716), 1,
      anon_sym_DQUOTE,
    STATE(568), 1,
      aux_sym__escaped_string_repeat1,
  [26034] = 4,
    ACTIONS(3), 1,
      sym_multi_line_comment,
    ACTIONS(1470), 1,
      anon_sym_RPAREN,
    ACTIONS(1718), 1,
      sym__identifier_char,
    STATE(571), 1,
      aux_sym__bare_identifier_repeat1,
  [26047] = 4,
    ACTIONS(3), 1,
      sym_multi_line_comment,
    ACTIONS(1507), 1,
      anon_sym_RPAREN,
    ACTIONS(1721), 1,
      anon_sym_POUND,
    STATE(572), 1,
      aux_sym__raw_string_repeat1,
  [26060] = 4,
    ACTIONS(3), 1,
      sym_multi_line_comment,
    ACTIONS(1460), 1,
      anon_sym_RPAREN,
    ACTIONS(1724), 1,
      sym__identifier_char,
    STATE(571), 1,
      aux_sym__bare_identifier_repeat1,
  [26073] = 3,
    ACTIONS(3), 1,
      sym_multi_line_comment,
    ACTIONS(1728), 1,
      aux_sym__escaped_string_token1,
    ACTIONS(1726), 2,
      anon_sym_DQUOTE,
      sym_escape,
  [26084] = 4,
    ACTIONS(3), 1,
      sym_multi_line_comment,
    ACTIONS(1490), 1,
      anon_sym_RPAREN,
    ACTIONS(1730), 1,
      anon_sym_POUND,
    STATE(572), 1,
      aux_sym__raw_string_repeat1,
  [26097] = 4,
    ACTIONS(3), 1,
      sym_multi_line_comment,
    ACTIONS(1503), 1,
      anon_sym_RPAREN,
    ACTIONS(1732), 1,
      sym__identifier_char,
    STATE(573), 1,
      aux_sym__bare_identifier_repeat1,
  [26110] = 3,
    ACTIONS(3), 1,
      sym_multi_line_comment,
    ACTIONS(1694), 1,
      sym__digit,
    STATE(479), 1,
      sym__integer,
  [26120] = 3,
    ACTIONS(3), 1,
      sym_multi_line_comment,
    ACTIONS(1734), 1,
      anon_sym_POUND,
    STATE(482), 1,
      aux_sym__raw_string_repeat1,
  [26130] = 3,
    ACTIONS(3), 1,
      sym_multi_line_comment,
    ACTIONS(1694), 1,
      sym__digit,
    STATE(473), 1,
      sym__integer,
  [26140] = 3,
    ACTIONS(3), 1,
      sym_multi_line_comment,
    ACTIONS(1694), 1,
      sym__digit,
    STATE(503), 1,
      sym__integer,
  [26150] = 3,
    ACTIONS(3), 1,
      sym_multi_line_comment,
    ACTIONS(1736), 1,
      anon_sym_POUND,
    STATE(575), 1,
      aux_sym__raw_string_repeat1,
  [26160] = 2,
    ACTIONS(3), 1,
      sym_multi_line_comment,
    ACTIONS(1738), 2,
      anon_sym_0,
      anon_sym_1,
  [26168] = 3,
    ACTIONS(3), 1,
      sym_multi_line_comment,
    ACTIONS(1068), 1,
      anon_sym_RPAREN,
    ACTIONS(1740), 1,
      sym___identifier_char_no_digit,
  [26178] = 2,
    ACTIONS(3), 1,
      sym_multi_line_comment,
    ACTIONS(1742), 2,
      anon_sym_0,
      anon_sym_1,
  [26186] = 2,
    ACTIONS(3), 1,
      sym_multi_line_comment,
    ACTIONS(1744), 1,
      anon_sym_RBRACE,
  [26193] = 2,
    ACTIONS(3), 1,
      sym_multi_line_comment,
    ACTIONS(1555), 1,
      anon_sym_RPAREN,
  [26200] = 2,
    ACTIONS(3), 1,
      sym_multi_line_comment,
    ACTIONS(1746), 1,
      anon_sym_EQ,
  [26207] = 2,
    ACTIONS(3), 1,
      sym_multi_line_comment,
    ACTIONS(1748), 1,
      anon_sym_RBRACE,
  [26214] = 2,
    ACTIONS(3), 1,
      sym_multi_line_comment,
    ACTIONS(1750), 1,
      anon_sym_RPAREN,
  [26221] = 2,
    ACTIONS(3), 1,
      sym_multi_line_comment,
    ACTIONS(1752), 1,
      anon_sym_RPAREN,
  [26228] = 2,
    ACTIONS(3), 1,
      sym_multi_line_comment,
    ACTIONS(1754), 1,
      anon_sym_RBRACE,
  [26235] = 2,
    ACTIONS(3), 1,
      sym_multi_line_comment,
    ACTIONS(1539), 1,
      anon_sym_RPAREN,
  [26242] = 2,
    ACTIONS(3), 1,
      sym_multi_line_comment,
    ACTIONS(1756), 1,
      anon_sym_RBRACE,
  [26249] = 2,
    ACTIONS(3), 1,
      sym_multi_line_comment,
    ACTIONS(1758), 1,
      ts_builtin_sym_end,
  [26256] = 2,
    ACTIONS(3), 1,
      sym_multi_line_comment,
    ACTIONS(1545), 1,
      anon_sym_RPAREN,
  [26263] = 2,
    ACTIONS(3), 1,
      sym_multi_line_comment,
    ACTIONS(1543), 1,
      anon_sym_RPAREN,
  [26270] = 2,
    ACTIONS(3), 1,
      sym_multi_line_comment,
    ACTIONS(1490), 1,
      anon_sym_RPAREN,
  [26277] = 2,
    ACTIONS(3), 1,
      sym_multi_line_comment,
    ACTIONS(1760), 1,
      anon_sym_RBRACE,
  [26284] = 2,
    ACTIONS(3), 1,
      sym_multi_line_comment,
    ACTIONS(1762), 1,
      aux_sym__octal_token1,
  [26291] = 2,
    ACTIONS(3), 1,
      sym_multi_line_comment,
    ACTIONS(1764), 1,
      aux_sym__raw_string_token4,
  [26298] = 2,
    ACTIONS(3), 1,
      sym_multi_line_comment,
    ACTIONS(1766), 1,
      aux_sym__raw_string_token2,
  [26305] = 2,
    ACTIONS(3), 1,
      sym_multi_line_comment,
    ACTIONS(1768), 1,
      sym__hex_digit,
  [26312] = 2,
    ACTIONS(3), 1,
      sym_multi_line_comment,
    ACTIONS(1770), 1,
      anon_sym_DQUOTE,
  [26319] = 2,
    ACTIONS(3), 1,
      sym_multi_line_comment,
    ACTIONS(1772), 1,
      aux_sym__octal_token1,
  [26326] = 2,
    ACTIONS(3), 1,
      sym_multi_line_comment,
    ACTIONS(1774), 1,
      anon_sym_RBRACE,
  [26333] = 2,
    ACTIONS(3), 1,
      sym_multi_line_comment,
    ACTIONS(1776), 1,
      sym__hex_digit,
  [26340] = 2,
    ACTIONS(3), 1,
      sym_multi_line_comment,
    ACTIONS(1778), 1,
      anon_sym_DQUOTE,
  [26347] = 2,
    ACTIONS(3), 1,
      sym_multi_line_comment,
    ACTIONS(1780), 1,
      aux_sym__raw_string_token2,
  [26354] = 2,
    ACTIONS(3), 1,
      sym_multi_line_comment,
    ACTIONS(1782), 1,
      aux_sym__raw_string_token4,
};

static const uint32_t ts_small_parse_table_map[] = {
  [SMALL_STATE(2)] = 0,
  [SMALL_STATE(3)] = 137,
  [SMALL_STATE(4)] = 274,
  [SMALL_STATE(5)] = 411,
  [SMALL_STATE(6)] = 548,
  [SMALL_STATE(7)] = 685,
  [SMALL_STATE(8)] = 822,
  [SMALL_STATE(9)] = 959,
  [SMALL_STATE(10)] = 1096,
  [SMALL_STATE(11)] = 1233,
  [SMALL_STATE(12)] = 1370,
  [SMALL_STATE(13)] = 1506,
  [SMALL_STATE(14)] = 1642,
  [SMALL_STATE(15)] = 1778,
  [SMALL_STATE(16)] = 1914,
  [SMALL_STATE(17)] = 2050,
  [SMALL_STATE(18)] = 2186,
  [SMALL_STATE(19)] = 2322,
  [SMALL_STATE(20)] = 2458,
  [SMALL_STATE(21)] = 2594,
  [SMALL_STATE(22)] = 2730,
  [SMALL_STATE(23)] = 2866,
  [SMALL_STATE(24)] = 3002,
  [SMALL_STATE(25)] = 3138,
  [SMALL_STATE(26)] = 3274,
  [SMALL_STATE(27)] = 3410,
  [SMALL_STATE(28)] = 3546,
  [SMALL_STATE(29)] = 3682,
  [SMALL_STATE(30)] = 3756,
  [SMALL_STATE(31)] = 3844,
  [SMALL_STATE(32)] = 3931,
  [SMALL_STATE(33)] = 4018,
  [SMALL_STATE(34)] = 4105,
  [SMALL_STATE(35)] = 4192,
  [SMALL_STATE(36)] = 4279,
  [SMALL_STATE(37)] = 4366,
  [SMALL_STATE(38)] = 4453,
  [SMALL_STATE(39)] = 4540,
  [SMALL_STATE(40)] = 4627,
  [SMALL_STATE(41)] = 4714,
  [SMALL_STATE(42)] = 4801,
  [SMALL_STATE(43)] = 4892,
  [SMALL_STATE(44)] = 4979,
  [SMALL_STATE(45)] = 5070,
  [SMALL_STATE(46)] = 5161,
  [SMALL_STATE(47)] = 5248,
  [SMALL_STATE(48)] = 5335,
  [SMALL_STATE(49)] = 5421,
  [SMALL_STATE(50)] = 5507,
  [SMALL_STATE(51)] = 5593,
  [SMALL_STATE(52)] = 5679,
  [SMALL_STATE(53)] = 5765,
  [SMALL_STATE(54)] = 5851,
  [SMALL_STATE(55)] = 5937,
  [SMALL_STATE(56)] = 6023,
  [SMALL_STATE(57)] = 6109,
  [SMALL_STATE(58)] = 6195,
  [SMALL_STATE(59)] = 6281,
  [SMALL_STATE(60)] = 6367,
  [SMALL_STATE(61)] = 6477,
  [SMALL_STATE(62)] = 6563,
  [SMALL_STATE(63)] = 6649,
  [SMALL_STATE(64)] = 6735,
  [SMALL_STATE(65)] = 6821,
  [SMALL_STATE(66)] = 6907,
  [SMALL_STATE(67)] = 6993,
  [SMALL_STATE(68)] = 7079,
  [SMALL_STATE(69)] = 7165,
  [SMALL_STATE(70)] = 7248,
  [SMALL_STATE(71)] = 7354,
  [SMALL_STATE(72)] = 7460,
  [SMALL_STATE(73)] = 7566,
  [SMALL_STATE(74)] = 7672,
  [SMALL_STATE(75)] = 7778,
  [SMALL_STATE(76)] = 7884,
  [SMALL_STATE(77)] = 7987,
  [SMALL_STATE(78)] = 8090,
  [SMALL_STATE(79)] = 8134,
  [SMALL_STATE(80)] = 8182,
  [SMALL_STATE(81)] = 8226,
  [SMALL_STATE(82)] = 8279,
  [SMALL_STATE(83)] = 8325,
  [SMALL_STATE(84)] = 8372,
  [SMALL_STATE(85)] = 8419,
  [SMALL_STATE(86)] = 8466,
  [SMALL_STATE(87)] = 8517,
  [SMALL_STATE(88)] = 8564,
  [SMALL_STATE(89)] = 8611,
  [SMALL_STATE(90)] = 8653,
  [SMALL_STATE(91)] = 8695,
  [SMALL_STATE(92)] = 8737,
  [SMALL_STATE(93)] = 8779,
  [SMALL_STATE(94)] = 8821,
  [SMALL_STATE(95)] = 8892,
  [SMALL_STATE(96)] = 8937,
  [SMALL_STATE(97)] = 9008,
  [SMALL_STATE(98)] = 9048,
  [SMALL_STATE(99)] = 9088,
  [SMALL_STATE(100)] = 9124,
  [SMALL_STATE(101)] = 9160,
  [SMALL_STATE(102)] = 9236,
  [SMALL_STATE(103)] = 9272,
  [SMALL_STATE(104)] = 9312,
  [SMALL_STATE(105)] = 9348,
  [SMALL_STATE(106)] = 9384,
  [SMALL_STATE(107)] = 9424,
  [SMALL_STATE(108)] = 9459,
  [SMALL_STATE(109)] = 9494,
  [SMALL_STATE(110)] = 9529,
  [SMALL_STATE(111)] = 9564,
  [SMALL_STATE(112)] = 9599,
  [SMALL_STATE(113)] = 9634,
  [SMALL_STATE(114)] = 9669,
  [SMALL_STATE(115)] = 9704,
  [SMALL_STATE(116)] = 9739,
  [SMALL_STATE(117)] = 9774,
  [SMALL_STATE(118)] = 9809,
  [SMALL_STATE(119)] = 9844,
  [SMALL_STATE(120)] = 9879,
  [SMALL_STATE(121)] = 9914,
  [SMALL_STATE(122)] = 9949,
  [SMALL_STATE(123)] = 9984,
  [SMALL_STATE(124)] = 10019,
  [SMALL_STATE(125)] = 10054,
  [SMALL_STATE(126)] = 10089,
  [SMALL_STATE(127)] = 10124,
  [SMALL_STATE(128)] = 10159,
  [SMALL_STATE(129)] = 10194,
  [SMALL_STATE(130)] = 10229,
  [SMALL_STATE(131)] = 10264,
  [SMALL_STATE(132)] = 10299,
  [SMALL_STATE(133)] = 10334,
  [SMALL_STATE(134)] = 10369,
  [SMALL_STATE(135)] = 10404,
  [SMALL_STATE(136)] = 10439,
  [SMALL_STATE(137)] = 10474,
  [SMALL_STATE(138)] = 10509,
  [SMALL_STATE(139)] = 10544,
  [SMALL_STATE(140)] = 10579,
  [SMALL_STATE(141)] = 10614,
  [SMALL_STATE(142)] = 10649,
  [SMALL_STATE(143)] = 10684,
  [SMALL_STATE(144)] = 10719,
  [SMALL_STATE(145)] = 10754,
  [SMALL_STATE(146)] = 10789,
  [SMALL_STATE(147)] = 10824,
  [SMALL_STATE(148)] = 10859,
  [SMALL_STATE(149)] = 10894,
  [SMALL_STATE(150)] = 10929,
  [SMALL_STATE(151)] = 10964,
  [SMALL_STATE(152)] = 10999,
  [SMALL_STATE(153)] = 11034,
  [SMALL_STATE(154)] = 11069,
  [SMALL_STATE(155)] = 11104,
  [SMALL_STATE(156)] = 11139,
  [SMALL_STATE(157)] = 11174,
  [SMALL_STATE(158)] = 11209,
  [SMALL_STATE(159)] = 11244,
  [SMALL_STATE(160)] = 11279,
  [SMALL_STATE(161)] = 11314,
  [SMALL_STATE(162)] = 11349,
  [SMALL_STATE(163)] = 11384,
  [SMALL_STATE(164)] = 11419,
  [SMALL_STATE(165)] = 11454,
  [SMALL_STATE(166)] = 11489,
  [SMALL_STATE(167)] = 11524,
  [SMALL_STATE(168)] = 11559,
  [SMALL_STATE(169)] = 11594,
  [SMALL_STATE(170)] = 11629,
  [SMALL_STATE(171)] = 11664,
  [SMALL_STATE(172)] = 11699,
  [SMALL_STATE(173)] = 11734,
  [SMALL_STATE(174)] = 11769,
  [SMALL_STATE(175)] = 11804,
  [SMALL_STATE(176)] = 11839,
  [SMALL_STATE(177)] = 11874,
  [SMALL_STATE(178)] = 11909,
  [SMALL_STATE(179)] = 11944,
  [SMALL_STATE(180)] = 11979,
  [SMALL_STATE(181)] = 12014,
  [SMALL_STATE(182)] = 12049,
  [SMALL_STATE(183)] = 12084,
  [SMALL_STATE(184)] = 12119,
  [SMALL_STATE(185)] = 12154,
  [SMALL_STATE(186)] = 12189,
  [SMALL_STATE(187)] = 12224,
  [SMALL_STATE(188)] = 12259,
  [SMALL_STATE(189)] = 12294,
  [SMALL_STATE(190)] = 12329,
  [SMALL_STATE(191)] = 12364,
  [SMALL_STATE(192)] = 12399,
  [SMALL_STATE(193)] = 12434,
  [SMALL_STATE(194)] = 12469,
  [SMALL_STATE(195)] = 12504,
  [SMALL_STATE(196)] = 12539,
  [SMALL_STATE(197)] = 12574,
  [SMALL_STATE(198)] = 12609,
  [SMALL_STATE(199)] = 12644,
  [SMALL_STATE(200)] = 12679,
  [SMALL_STATE(201)] = 12714,
  [SMALL_STATE(202)] = 12749,
  [SMALL_STATE(203)] = 12784,
  [SMALL_STATE(204)] = 12819,
  [SMALL_STATE(205)] = 12854,
  [SMALL_STATE(206)] = 12889,
  [SMALL_STATE(207)] = 12924,
  [SMALL_STATE(208)] = 12959,
  [SMALL_STATE(209)] = 12994,
  [SMALL_STATE(210)] = 13029,
  [SMALL_STATE(211)] = 13064,
  [SMALL_STATE(212)] = 13099,
  [SMALL_STATE(213)] = 13134,
  [SMALL_STATE(214)] = 13169,
  [SMALL_STATE(215)] = 13204,
  [SMALL_STATE(216)] = 13239,
  [SMALL_STATE(217)] = 13274,
  [SMALL_STATE(218)] = 13309,
  [SMALL_STATE(219)] = 13344,
  [SMALL_STATE(220)] = 13379,
  [SMALL_STATE(221)] = 13414,
  [SMALL_STATE(222)] = 13449,
  [SMALL_STATE(223)] = 13484,
  [SMALL_STATE(224)] = 13519,
  [SMALL_STATE(225)] = 13554,
  [SMALL_STATE(226)] = 13589,
  [SMALL_STATE(227)] = 13624,
  [SMALL_STATE(228)] = 13659,
  [SMALL_STATE(229)] = 13694,
  [SMALL_STATE(230)] = 13729,
  [SMALL_STATE(231)] = 13764,
  [SMALL_STATE(232)] = 13799,
  [SMALL_STATE(233)] = 13834,
  [SMALL_STATE(234)] = 13869,
  [SMALL_STATE(235)] = 13904,
  [SMALL_STATE(236)] = 13939,
  [SMALL_STATE(237)] = 13974,
  [SMALL_STATE(238)] = 14009,
  [SMALL_STATE(239)] = 14044,
  [SMALL_STATE(240)] = 14079,
  [SMALL_STATE(241)] = 14114,
  [SMALL_STATE(242)] = 14149,
  [SMALL_STATE(243)] = 14184,
  [SMALL_STATE(244)] = 14219,
  [SMALL_STATE(245)] = 14254,
  [SMALL_STATE(246)] = 14289,
  [SMALL_STATE(247)] = 14324,
  [SMALL_STATE(248)] = 14359,
  [SMALL_STATE(249)] = 14394,
  [SMALL_STATE(250)] = 14429,
  [SMALL_STATE(251)] = 14464,
  [SMALL_STATE(252)] = 14499,
  [SMALL_STATE(253)] = 14534,
  [SMALL_STATE(254)] = 14569,
  [SMALL_STATE(255)] = 14604,
  [SMALL_STATE(256)] = 14639,
  [SMALL_STATE(257)] = 14674,
  [SMALL_STATE(258)] = 14709,
  [SMALL_STATE(259)] = 14744,
  [SMALL_STATE(260)] = 14779,
  [SMALL_STATE(261)] = 14814,
  [SMALL_STATE(262)] = 14849,
  [SMALL_STATE(263)] = 14884,
  [SMALL_STATE(264)] = 14919,
  [SMALL_STATE(265)] = 14954,
  [SMALL_STATE(266)] = 14988,
  [SMALL_STATE(267)] = 15022,
  [SMALL_STATE(268)] = 15056,
  [SMALL_STATE(269)] = 15111,
  [SMALL_STATE(270)] = 15166,
  [SMALL_STATE(271)] = 15233,
  [SMALL_STATE(272)] = 15288,
  [SMALL_STATE(273)] = 15343,
  [SMALL_STATE(274)] = 15398,
  [SMALL_STATE(275)] = 15453,
  [SMALL_STATE(276)] = 15508,
  [SMALL_STATE(277)] = 15563,
  [SMALL_STATE(278)] = 15618,
  [SMALL_STATE(279)] = 15673,
  [SMALL_STATE(280)] = 15727,
  [SMALL_STATE(281)] = 15781,
  [SMALL_STATE(282)] = 15835,
  [SMALL_STATE(283)] = 15889,
  [SMALL_STATE(284)] = 15943,
  [SMALL_STATE(285)] = 15997,
  [SMALL_STATE(286)] = 16051,
  [SMALL_STATE(287)] = 16105,
  [SMALL_STATE(288)] = 16159,
  [SMALL_STATE(289)] = 16213,
  [SMALL_STATE(290)] = 16267,
  [SMALL_STATE(291)] = 16321,
  [SMALL_STATE(292)] = 16375,
  [SMALL_STATE(293)] = 16429,
  [SMALL_STATE(294)] = 16483,
  [SMALL_STATE(295)] = 16537,
  [SMALL_STATE(296)] = 16591,
  [SMALL_STATE(297)] = 16645,
  [SMALL_STATE(298)] = 16699,
  [SMALL_STATE(299)] = 16753,
  [SMALL_STATE(300)] = 16792,
  [SMALL_STATE(301)] = 16834,
  [SMALL_STATE(302)] = 16876,
  [SMALL_STATE(303)] = 16918,
  [SMALL_STATE(304)] = 16960,
  [SMALL_STATE(305)] = 17002,
  [SMALL_STATE(306)] = 17044,
  [SMALL_STATE(307)] = 17086,
  [SMALL_STATE(308)] = 17128,
  [SMALL_STATE(309)] = 17170,
  [SMALL_STATE(310)] = 17212,
  [SMALL_STATE(311)] = 17254,
  [SMALL_STATE(312)] = 17296,
  [SMALL_STATE(313)] = 17338,
  [SMALL_STATE(314)] = 17380,
  [SMALL_STATE(315)] = 17422,
  [SMALL_STATE(316)] = 17464,
  [SMALL_STATE(317)] = 17506,
  [SMALL_STATE(318)] = 17548,
  [SMALL_STATE(319)] = 17590,
  [SMALL_STATE(320)] = 17632,
  [SMALL_STATE(321)] = 17674,
  [SMALL_STATE(322)] = 17716,
  [SMALL_STATE(323)] = 17758,
  [SMALL_STATE(324)] = 17800,
  [SMALL_STATE(325)] = 17842,
  [SMALL_STATE(326)] = 17884,
  [SMALL_STATE(327)] = 17926,
  [SMALL_STATE(328)] = 17968,
  [SMALL_STATE(329)] = 17998,
  [SMALL_STATE(330)] = 18040,
  [SMALL_STATE(331)] = 18082,
  [SMALL_STATE(332)] = 18124,
  [SMALL_STATE(333)] = 18166,
  [SMALL_STATE(334)] = 18208,
  [SMALL_STATE(335)] = 18250,
  [SMALL_STATE(336)] = 18292,
  [SMALL_STATE(337)] = 18334,
  [SMALL_STATE(338)] = 18376,
  [SMALL_STATE(339)] = 18418,
  [SMALL_STATE(340)] = 18448,
  [SMALL_STATE(341)] = 18490,
  [SMALL_STATE(342)] = 18532,
  [SMALL_STATE(343)] = 18574,
  [SMALL_STATE(344)] = 18616,
  [SMALL_STATE(345)] = 18658,
  [SMALL_STATE(346)] = 18700,
  [SMALL_STATE(347)] = 18742,
  [SMALL_STATE(348)] = 18784,
  [SMALL_STATE(349)] = 18826,
  [SMALL_STATE(350)] = 18868,
  [SMALL_STATE(351)] = 18910,
  [SMALL_STATE(352)] = 18952,
  [SMALL_STATE(353)] = 18994,
  [SMALL_STATE(354)] = 19036,
  [SMALL_STATE(355)] = 19078,
  [SMALL_STATE(356)] = 19120,
  [SMALL_STATE(357)] = 19162,
  [SMALL_STATE(358)] = 19204,
  [SMALL_STATE(359)] = 19246,
  [SMALL_STATE(360)] = 19288,
  [SMALL_STATE(361)] = 19330,
  [SMALL_STATE(362)] = 19372,
  [SMALL_STATE(363)] = 19414,
  [SMALL_STATE(364)] = 19456,
  [SMALL_STATE(365)] = 19498,
  [SMALL_STATE(366)] = 19540,
  [SMALL_STATE(367)] = 19582,
  [SMALL_STATE(368)] = 19624,
  [SMALL_STATE(369)] = 19666,
  [SMALL_STATE(370)] = 19708,
  [SMALL_STATE(371)] = 19750,
  [SMALL_STATE(372)] = 19792,
  [SMALL_STATE(373)] = 19834,
  [SMALL_STATE(374)] = 19876,
  [SMALL_STATE(375)] = 19918,
  [SMALL_STATE(376)] = 19948,
  [SMALL_STATE(377)] = 19990,
  [SMALL_STATE(378)] = 20032,
  [SMALL_STATE(379)] = 20074,
  [SMALL_STATE(380)] = 20116,
  [SMALL_STATE(381)] = 20158,
  [SMALL_STATE(382)] = 20200,
  [SMALL_STATE(383)] = 20242,
  [SMALL_STATE(384)] = 20284,
  [SMALL_STATE(385)] = 20326,
  [SMALL_STATE(386)] = 20368,
  [SMALL_STATE(387)] = 20410,
  [SMALL_STATE(388)] = 20452,
  [SMALL_STATE(389)] = 20494,
  [SMALL_STATE(390)] = 20536,
  [SMALL_STATE(391)] = 20578,
  [SMALL_STATE(392)] = 20620,
  [SMALL_STATE(393)] = 20662,
  [SMALL_STATE(394)] = 20704,
  [SMALL_STATE(395)] = 20746,
  [SMALL_STATE(396)] = 20788,
  [SMALL_STATE(397)] = 20830,
  [SMALL_STATE(398)] = 20872,
  [SMALL_STATE(399)] = 20914,
  [SMALL_STATE(400)] = 20956,
  [SMALL_STATE(401)] = 20998,
  [SMALL_STATE(402)] = 21040,
  [SMALL_STATE(403)] = 21082,
  [SMALL_STATE(404)] = 21124,
  [SMALL_STATE(405)] = 21166,
  [SMALL_STATE(406)] = 21208,
  [SMALL_STATE(407)] = 21250,
  [SMALL_STATE(408)] = 21292,
  [SMALL_STATE(409)] = 21334,
  [SMALL_STATE(410)] = 21376,
  [SMALL_STATE(411)] = 21418,
  [SMALL_STATE(412)] = 21460,
  [SMALL_STATE(413)] = 21502,
  [SMALL_STATE(414)] = 21544,
  [SMALL_STATE(415)] = 21586,
  [SMALL_STATE(416)] = 21628,
  [SMALL_STATE(417)] = 21668,
  [SMALL_STATE(418)] = 21710,
  [SMALL_STATE(419)] = 21752,
  [SMALL_STATE(420)] = 21794,
  [SMALL_STATE(421)] = 21824,
  [SMALL_STATE(422)] = 21866,
  [SMALL_STATE(423)] = 21908,
  [SMALL_STATE(424)] = 21950,
  [SMALL_STATE(425)] = 21992,
  [SMALL_STATE(426)] = 22034,
  [SMALL_STATE(427)] = 22076,
  [SMALL_STATE(428)] = 22118,
  [SMALL_STATE(429)] = 22160,
  [SMALL_STATE(430)] = 22202,
  [SMALL_STATE(431)] = 22244,
  [SMALL_STATE(432)] = 22286,
  [SMALL_STATE(433)] = 22328,
  [SMALL_STATE(434)] = 22370,
  [SMALL_STATE(435)] = 22412,
  [SMALL_STATE(436)] = 22454,
  [SMALL_STATE(437)] = 22496,
  [SMALL_STATE(438)] = 22538,
  [SMALL_STATE(439)] = 22580,
  [SMALL_STATE(440)] = 22622,
  [SMALL_STATE(441)] = 22664,
  [SMALL_STATE(442)] = 22706,
  [SMALL_STATE(443)] = 22748,
  [SMALL_STATE(444)] = 22790,
  [SMALL_STATE(445)] = 22832,
  [SMALL_STATE(446)] = 22874,
  [SMALL_STATE(447)] = 22916,
  [SMALL_STATE(448)] = 22958,
  [SMALL_STATE(449)] = 23000,
  [SMALL_STATE(450)] = 23042,
  [SMALL_STATE(451)] = 23084,
  [SMALL_STATE(452)] = 23126,
  [SMALL_STATE(453)] = 23168,
  [SMALL_STATE(454)] = 23210,
  [SMALL_STATE(455)] = 23252,
  [SMALL_STATE(456)] = 23294,
  [SMALL_STATE(457)] = 23336,
  [SMALL_STATE(458)] = 23378,
  [SMALL_STATE(459)] = 23420,
  [SMALL_STATE(460)] = 23462,
  [SMALL_STATE(461)] = 23504,
  [SMALL_STATE(462)] = 23551,
  [SMALL_STATE(463)] = 23598,
  [SMALL_STATE(464)] = 23645,
  [SMALL_STATE(465)] = 23673,
  [SMALL_STATE(466)] = 23701,
  [SMALL_STATE(467)] = 23731,
  [SMALL_STATE(468)] = 23761,
  [SMALL_STATE(469)] = 23789,
  [SMALL_STATE(470)] = 23817,
  [SMALL_STATE(471)] = 23845,
  [SMALL_STATE(472)] = 23872,
  [SMALL_STATE(473)] = 23899,
  [SMALL_STATE(474)] = 23926,
  [SMALL_STATE(475)] = 23953,
  [SMALL_STATE(476)] = 23980,
  [SMALL_STATE(477)] = 24007,
  [SMALL_STATE(478)] = 24034,
  [SMALL_STATE(479)] = 24061,
  [SMALL_STATE(480)] = 24088,
  [SMALL_STATE(481)] = 24115,
  [SMALL_STATE(482)] = 24142,
  [SMALL_STATE(483)] = 24169,
  [SMALL_STATE(484)] = 24196,
  [SMALL_STATE(485)] = 24223,
  [SMALL_STATE(486)] = 24250,
  [SMALL_STATE(487)] = 24277,
  [SMALL_STATE(488)] = 24304,
  [SMALL_STATE(489)] = 24335,
  [SMALL_STATE(490)] = 24364,
  [SMALL_STATE(491)] = 24395,
  [SMALL_STATE(492)] = 24416,
  [SMALL_STATE(493)] = 24439,
  [SMALL_STATE(494)] = 24462,
  [SMALL_STATE(495)] = 24483,
  [SMALL_STATE(496)] = 24514,
  [SMALL_STATE(497)] = 24535,
  [SMALL_STATE(498)] = 24558,
  [SMALL_STATE(499)] = 24579,
  [SMALL_STATE(500)] = 24599,
  [SMALL_STATE(501)] = 24619,
  [SMALL_STATE(502)] = 24639,
  [SMALL_STATE(503)] = 24659,
  [SMALL_STATE(504)] = 24679,
  [SMALL_STATE(505)] = 24705,
  [SMALL_STATE(506)] = 24725,
  [SMALL_STATE(507)] = 24745,
  [SMALL_STATE(508)] = 24765,
  [SMALL_STATE(509)] = 24785,
  [SMALL_STATE(510)] = 24805,
  [SMALL_STATE(511)] = 24825,
  [SMALL_STATE(512)] = 24845,
  [SMALL_STATE(513)] = 24865,
  [SMALL_STATE(514)] = 24889,
  [SMALL_STATE(515)] = 24909,
  [SMALL_STATE(516)] = 24929,
  [SMALL_STATE(517)] = 24949,
  [SMALL_STATE(518)] = 24973,
  [SMALL_STATE(519)] = 24993,
  [SMALL_STATE(520)] = 25013,
  [SMALL_STATE(521)] = 25039,
  [SMALL_STATE(522)] = 25065,
  [SMALL_STATE(523)] = 25085,
  [SMALL_STATE(524)] = 25103,
  [SMALL_STATE(525)] = 25121,
  [SMALL_STATE(526)] = 25139,
  [SMALL_STATE(527)] = 25157,
  [SMALL_STATE(528)] = 25175,
  [SMALL_STATE(529)] = 25193,
  [SMALL_STATE(530)] = 25219,
  [SMALL_STATE(531)] = 25237,
  [SMALL_STATE(532)] = 25255,
  [SMALL_STATE(533)] = 25273,
  [SMALL_STATE(534)] = 25291,
  [SMALL_STATE(535)] = 25317,
  [SMALL_STATE(536)] = 25335,
  [SMALL_STATE(537)] = 25353,
  [SMALL_STATE(538)] = 25371,
  [SMALL_STATE(539)] = 25389,
  [SMALL_STATE(540)] = 25407,
  [SMALL_STATE(541)] = 25425,
  [SMALL_STATE(542)] = 25443,
  [SMALL_STATE(543)] = 25461,
  [SMALL_STATE(544)] = 25487,
  [SMALL_STATE(545)] = 25505,
  [SMALL_STATE(546)] = 25523,
  [SMALL_STATE(547)] = 25549,
  [SMALL_STATE(548)] = 25567,
  [SMALL_STATE(549)] = 25585,
  [SMALL_STATE(550)] = 25603,
  [SMALL_STATE(551)] = 25621,
  [SMALL_STATE(552)] = 25639,
  [SMALL_STATE(553)] = 25657,
  [SMALL_STATE(554)] = 25675,
  [SMALL_STATE(555)] = 25698,
  [SMALL_STATE(556)] = 25721,
  [SMALL_STATE(557)] = 25744,
  [SMALL_STATE(558)] = 25767,
  [SMALL_STATE(559)] = 25787,
  [SMALL_STATE(560)] = 25812,
  [SMALL_STATE(561)] = 25837,
  [SMALL_STATE(562)] = 25862,
  [SMALL_STATE(563)] = 25887,
  [SMALL_STATE(564)] = 25912,
  [SMALL_STATE(565)] = 25937,
  [SMALL_STATE(566)] = 25954,
  [SMALL_STATE(567)] = 25970,
  [SMALL_STATE(568)] = 25986,
  [SMALL_STATE(569)] = 26002,
  [SMALL_STATE(570)] = 26018,
  [SMALL_STATE(571)] = 26034,
  [SMALL_STATE(572)] = 26047,
  [SMALL_STATE(573)] = 26060,
  [SMALL_STATE(574)] = 26073,
  [SMALL_STATE(575)] = 26084,
  [SMALL_STATE(576)] = 26097,
  [SMALL_STATE(577)] = 26110,
  [SMALL_STATE(578)] = 26120,
  [SMALL_STATE(579)] = 26130,
  [SMALL_STATE(580)] = 26140,
  [SMALL_STATE(581)] = 26150,
  [SMALL_STATE(582)] = 26160,
  [SMALL_STATE(583)] = 26168,
  [SMALL_STATE(584)] = 26178,
  [SMALL_STATE(585)] = 26186,
  [SMALL_STATE(586)] = 26193,
  [SMALL_STATE(587)] = 26200,
  [SMALL_STATE(588)] = 26207,
  [SMALL_STATE(589)] = 26214,
  [SMALL_STATE(590)] = 26221,
  [SMALL_STATE(591)] = 26228,
  [SMALL_STATE(592)] = 26235,
  [SMALL_STATE(593)] = 26242,
  [SMALL_STATE(594)] = 26249,
  [SMALL_STATE(595)] = 26256,
  [SMALL_STATE(596)] = 26263,
  [SMALL_STATE(597)] = 26270,
  [SMALL_STATE(598)] = 26277,
  [SMALL_STATE(599)] = 26284,
  [SMALL_STATE(600)] = 26291,
  [SMALL_STATE(601)] = 26298,
  [SMALL_STATE(602)] = 26305,
  [SMALL_STATE(603)] = 26312,
  [SMALL_STATE(604)] = 26319,
  [SMALL_STATE(605)] = 26326,
  [SMALL_STATE(606)] = 26333,
  [SMALL_STATE(607)] = 26340,
  [SMALL_STATE(608)] = 26347,
  [SMALL_STATE(609)] = 26354,
};

static const TSParseActionEntry ts_parse_actions[] = {
  [0] = {.entry = {.count = 0, .reusable = false}},
  [1] = {.entry = {.count = 1, .reusable = false}}, RECOVER(),
  [3] = {.entry = {.count = 1, .reusable = true}}, SHIFT_EXTRA(),
  [5] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_document, 0),
  [7] = {.entry = {.count = 1, .reusable = false}}, SHIFT(510),
  [9] = {.entry = {.count = 1, .reusable = true}}, SHIFT(94),
  [11] = {.entry = {.count = 1, .reusable = true}}, SHIFT(29),
  [13] = {.entry = {.count = 1, .reusable = true}}, SHIFT(570),
  [15] = {.entry = {.count = 1, .reusable = false}}, SHIFT(601),
  [17] = {.entry = {.count = 1, .reusable = true}}, SHIFT(600),
  [19] = {.entry = {.count = 1, .reusable = true}}, SHIFT(493),
  [21] = {.entry = {.count = 1, .reusable = true}}, SHIFT(64),
  [23] = {.entry = {.count = 1, .reusable = true}}, SHIFT(557),
  [25] = {.entry = {.count = 1, .reusable = false}}, SHIFT(291),
  [27] = {.entry = {.count = 1, .reusable = false}}, SHIFT(271),
  [29] = {.entry = {.count = 1, .reusable = false}}, SHIFT(492),
  [31] = {.entry = {.count = 1, .reusable = true}}, SHIFT(72),
  [33] = {.entry = {.count = 1, .reusable = true}}, SHIFT(44),
  [35] = {.entry = {.count = 1, .reusable = true}}, SHIFT(201),
  [37] = {.entry = {.count = 1, .reusable = false}}, SHIFT(507),
  [39] = {.entry = {.count = 1, .reusable = false}}, SHIFT(375),
  [41] = {.entry = {.count = 1, .reusable = true}}, SHIFT(299),
  [43] = {.entry = {.count = 1, .reusable = true}}, SHIFT(606),
  [45] = {.entry = {.count = 1, .reusable = true}}, SHIFT(604),
  [47] = {.entry = {.count = 1, .reusable = true}}, SHIFT(582),
  [49] = {.entry = {.count = 1, .reusable = false}}, SHIFT(519),
  [51] = {.entry = {.count = 1, .reusable = true}}, SHIFT(546),
  [53] = {.entry = {.count = 1, .reusable = true}}, SHIFT(84),
  [55] = {.entry = {.count = 1, .reusable = true}}, SHIFT(229),
  [57] = {.entry = {.count = 1, .reusable = true}}, SHIFT(134),
  [59] = {.entry = {.count = 1, .reusable = true}}, SHIFT(245),
  [61] = {.entry = {.count = 1, .reusable = true}}, SHIFT(211),
  [63] = {.entry = {.count = 1, .reusable = true}}, SHIFT(180),
  [65] = {.entry = {.count = 1, .reusable = true}}, SHIFT(218),
  [67] = {.entry = {.count = 1, .reusable = true}}, SHIFT(116),
  [69] = {.entry = {.count = 1, .reusable = true}}, SHIFT(120),
  [71] = {.entry = {.count = 1, .reusable = true}}, SHIFT(160),
  [73] = {.entry = {.count = 1, .reusable = true}}, SHIFT(71),
  [75] = {.entry = {.count = 1, .reusable = true}}, SHIFT(52),
  [77] = {.entry = {.count = 1, .reusable = true}}, SHIFT(114),
  [79] = {.entry = {.count = 1, .reusable = true}}, SHIFT(188),
  [81] = {.entry = {.count = 1, .reusable = true}}, SHIFT(166),
  [83] = {.entry = {.count = 1, .reusable = true}}, SHIFT(70),
  [85] = {.entry = {.count = 1, .reusable = true}}, SHIFT(490),
  [87] = {.entry = {.count = 1, .reusable = true}}, SHIFT(115),
  [89] = {.entry = {.count = 1, .reusable = true}}, SHIFT(223),
  [91] = {.entry = {.count = 1, .reusable = true}}, SHIFT(217),
  [93] = {.entry = {.count = 1, .reusable = true}}, SHIFT(119),
  [95] = {.entry = {.count = 1, .reusable = true}}, SHIFT(214),
  [97] = {.entry = {.count = 1, .reusable = true}}, SHIFT(108),
  [99] = {.entry = {.count = 1, .reusable = true}}, SHIFT(222),
  [101] = {.entry = {.count = 1, .reusable = true}}, SHIFT(207),
  [103] = {.entry = {.count = 1, .reusable = true}}, SHIFT(153),
  [105] = {.entry = {.count = 1, .reusable = true}}, SHIFT(171),
  [107] = {.entry = {.count = 1, .reusable = true}}, SHIFT(260),
  [109] = {.entry = {.count = 1, .reusable = true}}, SHIFT(195),
  [111] = {.entry = {.count = 1, .reusable = true}}, SHIFT(185),
  [113] = {.entry = {.count = 1, .reusable = true}}, SHIFT(190),
  [115] = {.entry = {.count = 1, .reusable = false}}, SHIFT(596),
  [117] = {.entry = {.count = 1, .reusable = false}}, SHIFT(589),
  [119] = {.entry = {.count = 1, .reusable = true}}, SHIFT(567),
  [121] = {.entry = {.count = 1, .reusable = false}}, SHIFT(608),
  [123] = {.entry = {.count = 1, .reusable = true}}, SHIFT(609),
  [125] = {.entry = {.count = 1, .reusable = true}}, SHIFT(583),
  [127] = {.entry = {.count = 1, .reusable = true}}, REDUCE(aux_sym_document_repeat2, 2),
  [129] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_document_repeat2, 2), SHIFT_REPEAT(510),
  [132] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_document_repeat2, 2), SHIFT_REPEAT(94),
  [135] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_document_repeat2, 2), SHIFT_REPEAT(29),
  [138] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_document_repeat2, 2), SHIFT_REPEAT(570),
  [141] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_document_repeat2, 2), SHIFT_REPEAT(601),
  [144] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_document_repeat2, 2), SHIFT_REPEAT(600),
  [147] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_document_repeat2, 2), SHIFT_REPEAT(493),
  [150] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_document_repeat2, 2), SHIFT_REPEAT(69),
  [153] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_document_repeat2, 2), SHIFT_REPEAT(557),
  [156] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_document_repeat2, 2), SHIFT_REPEAT(291),
  [159] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_document_repeat2, 2), SHIFT_REPEAT(271),
  [162] = {.entry = {.count = 1, .reusable = true}}, SHIFT(538),
  [164] = {.entry = {.count = 1, .reusable = true}}, SHIFT(51),
  [166] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_document, 3),
  [168] = {.entry = {.count = 1, .reusable = true}}, SHIFT(66),
  [170] = {.entry = {.count = 1, .reusable = true}}, SHIFT(536),
  [172] = {.entry = {.count = 1, .reusable = true}}, SHIFT(65),
  [174] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_document, 1),
  [176] = {.entry = {.count = 1, .reusable = true}}, SHIFT(49),
  [178] = {.entry = {.count = 1, .reusable = true}}, SHIFT(524),
  [180] = {.entry = {.count = 1, .reusable = true}}, SHIFT(58),
  [182] = {.entry = {.count = 1, .reusable = true}}, SHIFT(537),
  [184] = {.entry = {.count = 1, .reusable = true}}, SHIFT(57),
  [186] = {.entry = {.count = 1, .reusable = true}}, SHIFT(535),
  [188] = {.entry = {.count = 1, .reusable = true}}, SHIFT(50),
  [190] = {.entry = {.count = 1, .reusable = true}}, SHIFT(525),
  [192] = {.entry = {.count = 1, .reusable = true}}, SHIFT(53),
  [194] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_document, 2),
  [196] = {.entry = {.count = 1, .reusable = true}}, SHIFT(63),
  [198] = {.entry = {.count = 1, .reusable = false}}, SHIFT(94),
  [200] = {.entry = {.count = 1, .reusable = true}}, SHIFT(523),
  [202] = {.entry = {.count = 1, .reusable = false}}, SHIFT(29),
  [204] = {.entry = {.count = 1, .reusable = false}}, SHIFT(493),
  [206] = {.entry = {.count = 1, .reusable = true}}, SHIFT(68),
  [208] = {.entry = {.count = 1, .reusable = false}}, SHIFT(68),
  [210] = {.entry = {.count = 1, .reusable = false}}, SHIFT(557),
  [212] = {.entry = {.count = 1, .reusable = true}}, SHIFT(593),
  [214] = {.entry = {.count = 1, .reusable = true}}, SHIFT(530),
  [216] = {.entry = {.count = 1, .reusable = true}}, SHIFT(67),
  [218] = {.entry = {.count = 1, .reusable = true}}, SHIFT(548),
  [220] = {.entry = {.count = 1, .reusable = true}}, SHIFT(48),
  [222] = {.entry = {.count = 1, .reusable = false}}, SHIFT(48),
  [224] = {.entry = {.count = 1, .reusable = true}}, SHIFT(588),
  [226] = {.entry = {.count = 1, .reusable = true}}, SHIFT(547),
  [228] = {.entry = {.count = 1, .reusable = true}}, SHIFT(56),
  [230] = {.entry = {.count = 1, .reusable = false}}, SHIFT(56),
  [232] = {.entry = {.count = 1, .reusable = true}}, SHIFT(585),
  [234] = {.entry = {.count = 1, .reusable = true}}, SHIFT(526),
  [236] = {.entry = {.count = 1, .reusable = true}}, SHIFT(87),
  [238] = {.entry = {.count = 1, .reusable = true}}, SHIFT(82),
  [240] = {.entry = {.count = 1, .reusable = true}}, SHIFT(539),
  [242] = {.entry = {.count = 1, .reusable = true}}, SHIFT(549),
  [244] = {.entry = {.count = 1, .reusable = true}}, SHIFT(61),
  [246] = {.entry = {.count = 1, .reusable = true}}, SHIFT(88),
  [248] = {.entry = {.count = 1, .reusable = true}}, SHIFT(59),
  [250] = {.entry = {.count = 1, .reusable = true}}, SHIFT(550),
  [252] = {.entry = {.count = 1, .reusable = true}}, SHIFT(83),
  [254] = {.entry = {.count = 1, .reusable = true}}, SHIFT(77),
  [256] = {.entry = {.count = 1, .reusable = true}}, SHIFT(543),
  [258] = {.entry = {.count = 1, .reusable = true}}, SHIFT(95),
  [260] = {.entry = {.count = 1, .reusable = true}}, SHIFT(527),
  [262] = {.entry = {.count = 1, .reusable = true}}, SHIFT(54),
  [264] = {.entry = {.count = 1, .reusable = true}}, SHIFT(85),
  [266] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_document, 4),
  [268] = {.entry = {.count = 1, .reusable = true}}, SHIFT(528),
  [270] = {.entry = {.count = 1, .reusable = true}}, SHIFT(488),
  [272] = {.entry = {.count = 1, .reusable = true}}, SHIFT(62),
  [274] = {.entry = {.count = 1, .reusable = true}}, SHIFT(45),
  [276] = {.entry = {.count = 1, .reusable = true}}, SHIFT(495),
  [278] = {.entry = {.count = 1, .reusable = true}}, SHIFT(42),
  [280] = {.entry = {.count = 1, .reusable = true}}, SHIFT(55),
  [282] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_single_line_comment, 2),
  [284] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_single_line_comment, 2),
  [286] = {.entry = {.count = 1, .reusable = false}}, REDUCE(aux_sym__node_space_repeat1, 2),
  [288] = {.entry = {.count = 1, .reusable = true}}, REDUCE(aux_sym__node_space_repeat1, 2),
  [290] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym__node_space_repeat1, 2), SHIFT_REPEAT(79),
  [293] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_single_line_comment, 3),
  [295] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_single_line_comment, 3),
  [297] = {.entry = {.count = 1, .reusable = false}}, REDUCE(aux_sym__node_field_comment_repeat1, 2),
  [299] = {.entry = {.count = 1, .reusable = true}}, REDUCE(aux_sym__node_field_comment_repeat1, 2),
  [301] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym__node_field_comment_repeat1, 2), SHIFT_REPEAT(546),
  [304] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym__node_field_comment_repeat1, 2), SHIFT_REPEAT(84),
  [307] = {.entry = {.count = 1, .reusable = true}}, REDUCE(aux_sym_document_repeat1, 2),
  [309] = {.entry = {.count = 1, .reusable = false}}, REDUCE(aux_sym_document_repeat1, 2),
  [311] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_document_repeat1, 2), SHIFT_REPEAT(82),
  [314] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_document_repeat1, 2), SHIFT_REPEAT(557),
  [317] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_document_repeat1, 2), SHIFT(524),
  [320] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym__node_space, 1),
  [322] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym__node_space, 1),
  [324] = {.entry = {.count = 2, .reusable = true}}, REDUCE(sym__node_space, 1), SHIFT(546),
  [327] = {.entry = {.count = 2, .reusable = true}}, REDUCE(sym__node_space, 1), SHIFT(90),
  [330] = {.entry = {.count = 2, .reusable = true}}, REDUCE(sym_document, 2), REDUCE(aux_sym_document_repeat1, 2),
  [333] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym__node_field_comment_repeat1, 2), SHIFT_REPEAT(543),
  [336] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym__node_field_comment_repeat1, 2), SHIFT_REPEAT(95),
  [339] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_document_repeat1, 2), SHIFT(530),
  [342] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_document_repeat1, 2), SHIFT(536),
  [345] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym__node_space, 3),
  [347] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym__node_space, 3),
  [349] = {.entry = {.count = 2, .reusable = true}}, REDUCE(sym__node_space, 3), SHIFT(90),
  [352] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym__node_space_repeat1, 2), SHIFT_REPEAT(90),
  [355] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym__node_space, 2),
  [357] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym__node_space, 2),
  [359] = {.entry = {.count = 2, .reusable = true}}, REDUCE(sym__node_space, 2), SHIFT(89),
  [362] = {.entry = {.count = 2, .reusable = true}}, REDUCE(sym__node_space, 2), SHIFT(90),
  [365] = {.entry = {.count = 2, .reusable = true}}, REDUCE(sym__node_space, 1), SHIFT(92),
  [368] = {.entry = {.count = 1, .reusable = false}}, SHIFT(288),
  [370] = {.entry = {.count = 1, .reusable = false}}, SHIFT(277),
  [372] = {.entry = {.count = 2, .reusable = true}}, REDUCE(sym__node_space, 1), SHIFT(543),
  [375] = {.entry = {.count = 2, .reusable = true}}, REDUCE(sym__node_space, 1), SHIFT(79),
  [378] = {.entry = {.count = 1, .reusable = false}}, SHIFT(281),
  [380] = {.entry = {.count = 1, .reusable = false}}, SHIFT(268),
  [382] = {.entry = {.count = 2, .reusable = true}}, REDUCE(sym__node_space, 2), SHIFT(103),
  [385] = {.entry = {.count = 2, .reusable = true}}, REDUCE(sym__node_space, 1), SHIFT(106),
  [388] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym__escline, 2),
  [390] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym__escline, 2),
  [392] = {.entry = {.count = 1, .reusable = false}}, SHIFT(499),
  [394] = {.entry = {.count = 1, .reusable = false}}, REDUCE(aux_sym__node_field_comment_repeat1, 1),
  [396] = {.entry = {.count = 1, .reusable = true}}, REDUCE(aux_sym__node_field_comment_repeat1, 1),
  [398] = {.entry = {.count = 2, .reusable = true}}, REDUCE(sym__node_space, 3), SHIFT(79),
  [401] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym__escline, 3),
  [403] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym__escline, 3),
  [405] = {.entry = {.count = 2, .reusable = true}}, REDUCE(sym__node_space, 2), SHIFT(79),
  [408] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_arco_pure_math_node, 6, .production_id = 27),
  [410] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_arco_pure_math_node, 6, .production_id = 27),
  [412] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_arco_pure_math_node, 6, .production_id = 20),
  [414] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_arco_pure_math_node, 6, .production_id = 20),
  [416] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_kdl_node, 8, .production_id = 26),
  [418] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_kdl_node, 8, .production_id = 26),
  [420] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_arco_constraint_node, 8, .production_id = 25),
  [422] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_arco_constraint_node, 8, .production_id = 25),
  [424] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_arco_pure_math_node, 8, .production_id = 25),
  [426] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_arco_pure_math_node, 8, .production_id = 25),
  [428] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_arco_constraint_node, 7, .production_id = 24),
  [430] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_arco_constraint_node, 7, .production_id = 24),
  [432] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_arco_pure_math_node, 7, .production_id = 24),
  [434] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_arco_pure_math_node, 7, .production_id = 24),
  [436] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_kdl_node, 7, .production_id = 10),
  [438] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_kdl_node, 7, .production_id = 10),
  [440] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_arco_pure_math_node, 3, .production_id = 1),
  [442] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_arco_pure_math_node, 3, .production_id = 1),
  [444] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_arco_constraint_node, 7, .production_id = 20),
  [446] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_arco_constraint_node, 7, .production_id = 20),
  [448] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_arco_constraint_node, 7, .production_id = 37),
  [450] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_arco_constraint_node, 7, .production_id = 37),
  [452] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_arco_constraint_node, 7, .production_id = 30),
  [454] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_arco_constraint_node, 7, .production_id = 30),
  [456] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_arco_pure_math_node, 7, .production_id = 20),
  [458] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_arco_pure_math_node, 7, .production_id = 20),
  [460] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_arco_constraint_node, 5, .production_id = 9),
  [462] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_arco_constraint_node, 5, .production_id = 9),
  [464] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_arco_pure_math_node, 7, .production_id = 37),
  [466] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_arco_pure_math_node, 7, .production_id = 37),
  [468] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_arco_pure_math_node, 7, .production_id = 30),
  [470] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_arco_pure_math_node, 7, .production_id = 30),
  [472] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_kdl_node, 7, .production_id = 36),
  [474] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_kdl_node, 7, .production_id = 36),
  [476] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_arco_constraint_node, 5, .production_id = 18),
  [478] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_arco_constraint_node, 5, .production_id = 18),
  [480] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_kdl_node, 7, .production_id = 29),
  [482] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_kdl_node, 7, .production_id = 29),
  [484] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_kdl_node, 7, .production_id = 19),
  [486] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_kdl_node, 7, .production_id = 19),
  [488] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_arco_constraint_node, 7, .production_id = 35),
  [490] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_arco_constraint_node, 7, .production_id = 35),
  [492] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_arco_constraint_node, 7, .production_id = 28),
  [494] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_arco_constraint_node, 7, .production_id = 28),
  [496] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_arco_constraint_node, 7, .production_id = 18),
  [498] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_arco_constraint_node, 7, .production_id = 18),
  [500] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_arco_constraint_node, 3, .production_id = 4),
  [502] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_arco_constraint_node, 3, .production_id = 4),
  [504] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_arco_pure_math_node, 7, .production_id = 35),
  [506] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_arco_pure_math_node, 7, .production_id = 35),
  [508] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_arco_pure_math_node, 7, .production_id = 28),
  [510] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_arco_pure_math_node, 7, .production_id = 28),
  [512] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_arco_pure_math_node, 7, .production_id = 18),
  [514] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_arco_pure_math_node, 7, .production_id = 18),
  [516] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_arco_constraint_node, 3, .production_id = 1),
  [518] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_arco_constraint_node, 3, .production_id = 1),
  [520] = {.entry = {.count = 1, .reusable = false}}, REDUCE(aux_sym_document_repeat2, 2),
  [522] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_arco_pure_math_node, 8, .production_id = 30),
  [524] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_arco_pure_math_node, 8, .production_id = 30),
  [526] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_arco_pure_math_node, 8, .production_id = 33),
  [528] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_arco_pure_math_node, 8, .production_id = 33),
  [530] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_arco_pure_math_node, 3, .production_id = 4),
  [532] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_arco_pure_math_node, 3, .production_id = 4),
  [534] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_arco_constraint_node, 8, .production_id = 27),
  [536] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_arco_constraint_node, 8, .production_id = 27),
  [538] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_kdl_node, 7, .production_id = 34),
  [540] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_kdl_node, 7, .production_id = 34),
  [542] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_arco_constraint_node, 7, .production_id = 33),
  [544] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_arco_constraint_node, 7, .production_id = 33),
  [546] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_arco_constraint_node, 8, .production_id = 33),
  [548] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_arco_constraint_node, 8, .production_id = 33),
  [550] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_kdl_node, 3, .production_id = 5),
  [552] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_kdl_node, 3, .production_id = 5),
  [554] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_arco_constraint_node, 7, .production_id = 27),
  [556] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_arco_constraint_node, 7, .production_id = 27),
  [558] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_arco_constraint_node, 7, .production_id = 17),
  [560] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_arco_constraint_node, 7, .production_id = 17),
  [562] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_kdl_node, 8, .production_id = 34),
  [564] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_kdl_node, 8, .production_id = 34),
  [566] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_arco_pure_math_node, 7, .production_id = 33),
  [568] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_arco_pure_math_node, 7, .production_id = 33),
  [570] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_arco_pure_math_node, 8, .production_id = 28),
  [572] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_arco_pure_math_node, 8, .production_id = 28),
  [574] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_node, 1),
  [576] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_node, 1),
  [578] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_arco_pure_math_node, 4, .production_id = 6),
  [580] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_arco_pure_math_node, 4, .production_id = 6),
  [582] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_arco_pure_math_node, 8, .production_id = 35),
  [584] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_arco_pure_math_node, 8, .production_id = 35),
  [586] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_arco_pure_math_node, 7, .production_id = 27),
  [588] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_arco_pure_math_node, 7, .production_id = 27),
  [590] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_arco_pure_math_node, 4, .production_id = 2),
  [592] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_arco_pure_math_node, 4, .production_id = 2),
  [594] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_arco_pure_math_node, 7, .production_id = 17),
  [596] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_arco_pure_math_node, 7, .production_id = 17),
  [598] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_arco_constraint_node, 8, .production_id = 28),
  [600] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_arco_constraint_node, 8, .production_id = 28),
  [602] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_arco_constraint_node, 4, .production_id = 6),
  [604] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_arco_constraint_node, 4, .production_id = 6),
  [606] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_kdl_node, 7, .production_id = 26),
  [608] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_kdl_node, 7, .production_id = 26),
  [610] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_kdl_node, 7, .production_id = 16),
  [612] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_kdl_node, 7, .production_id = 16),
  [614] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_arco_constraint_node, 7, .production_id = 25),
  [616] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_arco_constraint_node, 7, .production_id = 25),
  [618] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_arco_constraint_node, 4, .production_id = 2),
  [620] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_arco_constraint_node, 4, .production_id = 2),
  [622] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_arco_constraint_node, 8, .production_id = 35),
  [624] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_arco_constraint_node, 8, .production_id = 35),
  [626] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_kdl_node, 4, .production_id = 7),
  [628] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_kdl_node, 4, .production_id = 7),
  [630] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_arco_constraint_node, 7, .production_id = 15),
  [632] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_arco_constraint_node, 7, .production_id = 15),
  [634] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_arco_pure_math_node, 7, .production_id = 25),
  [636] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_arco_pure_math_node, 7, .production_id = 25),
  [638] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_arco_pure_math_node, 7, .production_id = 15),
  [640] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_arco_pure_math_node, 7, .production_id = 15),
  [642] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_kdl_node, 4),
  [644] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_kdl_node, 4),
  [646] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_kdl_node, 8, .production_id = 29),
  [648] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_kdl_node, 8, .production_id = 29),
  [650] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_arco_pure_math_node, 4, .production_id = 8),
  [652] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_arco_pure_math_node, 4, .production_id = 8),
  [654] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_arco_constraint_node, 6, .production_id = 24),
  [656] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_arco_constraint_node, 6, .production_id = 24),
  [658] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_kdl_node, 8, .production_id = 36),
  [660] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_kdl_node, 8, .production_id = 36),
  [662] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_arco_pure_math_node, 5, .production_id = 9),
  [664] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_arco_pure_math_node, 5, .production_id = 9),
  [666] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_arco_constraint_node, 6, .production_id = 14),
  [668] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_arco_constraint_node, 6, .production_id = 14),
  [670] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_arco_constraint_node, 4, .production_id = 8),
  [672] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_arco_constraint_node, 4, .production_id = 8),
  [674] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_arco_constraint_node, 5, .production_id = 20),
  [676] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_arco_constraint_node, 5, .production_id = 20),
  [678] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_arco_pure_math_node, 5, .production_id = 4),
  [680] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_arco_pure_math_node, 5, .production_id = 4),
  [682] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_arco_pure_math_node, 5, .production_id = 18),
  [684] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_arco_pure_math_node, 5, .production_id = 18),
  [686] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_arco_pure_math_node, 4, .production_id = 9),
  [688] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_arco_pure_math_node, 4, .production_id = 9),
  [690] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_arco_pure_math_node, 8, .production_id = 37),
  [692] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_arco_pure_math_node, 8, .production_id = 37),
  [694] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_arco_pure_math_node, 8, .production_id = 38),
  [696] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_arco_pure_math_node, 8, .production_id = 38),
  [698] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_arco_constraint_node, 5, .production_id = 8),
  [700] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_arco_constraint_node, 5, .production_id = 8),
  [702] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_arco_pure_math_node, 5, .production_id = 14),
  [704] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_arco_pure_math_node, 5, .production_id = 14),
  [706] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_arco_constraint_node, 4, .production_id = 9),
  [708] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_arco_constraint_node, 4, .production_id = 9),
  [710] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_arco_constraint_node, 5, .production_id = 17),
  [712] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_arco_constraint_node, 5, .production_id = 17),
  [714] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_arco_pure_math_node, 5, .production_id = 20),
  [716] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_arco_pure_math_node, 5, .production_id = 20),
  [718] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_arco_pure_math_node, 5, .production_id = 8),
  [720] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_arco_pure_math_node, 5, .production_id = 8),
  [722] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_kdl_node, 4, .production_id = 10),
  [724] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_kdl_node, 4, .production_id = 10),
  [726] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_kdl_node, 10, .production_id = 39),
  [728] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_kdl_node, 10, .production_id = 39),
  [730] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_kdl_node, 5, .production_id = 10),
  [732] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_kdl_node, 5, .production_id = 10),
  [734] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_arco_pure_math_node, 5, .production_id = 17),
  [736] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_arco_pure_math_node, 5, .production_id = 17),
  [738] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_kdl_node, 5),
  [740] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_kdl_node, 5),
  [742] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_arco_constraint_node, 10, .production_id = 38),
  [744] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_arco_constraint_node, 10, .production_id = 38),
  [746] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_arco_pure_math_node, 10, .production_id = 38),
  [748] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_arco_pure_math_node, 10, .production_id = 38),
  [750] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_arco_pure_math_node, 5, .production_id = 24),
  [752] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_arco_pure_math_node, 5, .production_id = 24),
  [754] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_kdl_node, 5, .production_id = 16),
  [756] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_kdl_node, 5, .production_id = 16),
  [758] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_kdl_node, 3),
  [760] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_kdl_node, 3),
  [762] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_arco_pure_math_node, 6, .production_id = 24),
  [764] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_arco_pure_math_node, 6, .production_id = 24),
  [766] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_arco_pure_math_node, 6, .production_id = 14),
  [768] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_arco_pure_math_node, 6, .production_id = 14),
  [770] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_arco_constraint_node, 8, .production_id = 30),
  [772] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_arco_constraint_node, 8, .production_id = 30),
  [774] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_kdl_node, 5, .production_id = 19),
  [776] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_kdl_node, 5, .production_id = 19),
  [778] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_arco_pure_math_node, 4, .production_id = 4),
  [780] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_arco_pure_math_node, 4, .production_id = 4),
  [782] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_arco_constraint_node, 6, .production_id = 20),
  [784] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_arco_constraint_node, 6, .production_id = 20),
  [786] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_arco_constraint_node, 6, .production_id = 30),
  [788] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_arco_constraint_node, 6, .production_id = 30),
  [790] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_arco_constraint_node, 3, .production_id = 2),
  [792] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_arco_constraint_node, 3, .production_id = 2),
  [794] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_arco_pure_math_node, 8, .production_id = 27),
  [796] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_arco_pure_math_node, 8, .production_id = 27),
  [798] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_arco_constraint_node, 8, .production_id = 37),
  [800] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_arco_constraint_node, 8, .production_id = 37),
  [802] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_arco_pure_math_node, 6, .production_id = 30),
  [804] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_arco_pure_math_node, 6, .production_id = 30),
  [806] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_kdl_node, 6, .production_id = 10),
  [808] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_kdl_node, 6, .production_id = 10),
  [810] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_kdl_node, 6, .production_id = 29),
  [812] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_kdl_node, 6, .production_id = 29),
  [814] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_kdl_node, 6, .production_id = 19),
  [816] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_kdl_node, 6, .production_id = 19),
  [818] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_kdl_node, 5, .production_id = 7),
  [820] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_kdl_node, 5, .production_id = 7),
  [822] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_arco_constraint_node, 6, .production_id = 9),
  [824] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_arco_constraint_node, 6, .production_id = 9),
  [826] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_arco_constraint_node, 6, .production_id = 28),
  [828] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_arco_constraint_node, 6, .production_id = 28),
  [830] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_arco_constraint_node, 6, .production_id = 18),
  [832] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_arco_constraint_node, 6, .production_id = 18),
  [834] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_arco_pure_math_node, 6, .production_id = 9),
  [836] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_arco_pure_math_node, 6, .production_id = 9),
  [838] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_arco_pure_math_node, 6, .production_id = 28),
  [840] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_arco_pure_math_node, 6, .production_id = 28),
  [842] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_arco_pure_math_node, 6, .production_id = 18),
  [844] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_arco_pure_math_node, 6, .production_id = 18),
  [846] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_kdl_node, 6),
  [848] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_kdl_node, 6),
  [850] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_arco_constraint_node, 6, .production_id = 8),
  [852] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_arco_constraint_node, 6, .production_id = 8),
  [854] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_arco_constraint_node, 6, .production_id = 27),
  [856] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_arco_constraint_node, 6, .production_id = 27),
  [858] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_arco_pure_math_node, 4, .production_id = 14),
  [860] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_arco_pure_math_node, 4, .production_id = 14),
  [862] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_arco_constraint_node, 6, .production_id = 17),
  [864] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_arco_constraint_node, 6, .production_id = 17),
  [866] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_arco_pure_math_node, 6, .production_id = 8),
  [868] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_arco_pure_math_node, 6, .production_id = 8),
  [870] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_arco_pure_math_node, 4, .production_id = 1),
  [872] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_arco_pure_math_node, 4, .production_id = 1),
  [874] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_arco_constraint_node, 8, .production_id = 38),
  [876] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_arco_constraint_node, 8, .production_id = 38),
  [878] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_arco_pure_math_node, 3, .production_id = 2),
  [880] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_arco_pure_math_node, 3, .production_id = 2),
  [882] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_arco_pure_math_node, 6, .production_id = 17),
  [884] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_arco_pure_math_node, 6, .production_id = 17),
  [886] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_kdl_node, 6, .production_id = 26),
  [888] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_kdl_node, 6, .production_id = 26),
  [890] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_kdl_node, 8, .production_id = 39),
  [892] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_kdl_node, 8, .production_id = 39),
  [894] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_arco_constraint_node, 5, .production_id = 2),
  [896] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_arco_constraint_node, 5, .production_id = 2),
  [898] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_kdl_node, 9, .production_id = 39),
  [900] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_kdl_node, 9, .production_id = 39),
  [902] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_kdl_node, 6, .production_id = 16),
  [904] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_kdl_node, 6, .production_id = 16),
  [906] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_kdl_node, 6, .production_id = 7),
  [908] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_kdl_node, 6, .production_id = 7),
  [910] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_kdl_node, 2),
  [912] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_kdl_node, 2),
  [914] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_arco_pure_math_node, 9, .production_id = 33),
  [916] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_arco_pure_math_node, 9, .production_id = 33),
  [918] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_arco_constraint_node, 6, .production_id = 25),
  [920] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_arco_constraint_node, 6, .production_id = 25),
  [922] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_arco_constraint_node, 9, .production_id = 33),
  [924] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_arco_constraint_node, 9, .production_id = 33),
  [926] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_arco_constraint_node, 9, .production_id = 38),
  [928] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_arco_constraint_node, 9, .production_id = 38),
  [930] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_arco_constraint_node, 4, .production_id = 4),
  [932] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_arco_constraint_node, 4, .production_id = 4),
  [934] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_arco_constraint_node, 6, .production_id = 15),
  [936] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_arco_constraint_node, 6, .production_id = 15),
  [938] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_arco_constraint_node, 2, .production_id = 1),
  [940] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_arco_constraint_node, 2, .production_id = 1),
  [942] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_arco_constraint_node, 4, .production_id = 14),
  [944] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_arco_constraint_node, 4, .production_id = 14),
  [946] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_arco_constraint_node, 6, .production_id = 6),
  [948] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_arco_constraint_node, 6, .production_id = 6),
  [950] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_arco_pure_math_node, 6, .production_id = 25),
  [952] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_arco_pure_math_node, 6, .production_id = 25),
  [954] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_arco_pure_math_node, 6, .production_id = 15),
  [956] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_arco_pure_math_node, 6, .production_id = 15),
  [958] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_arco_constraint_node, 4, .production_id = 1),
  [960] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_arco_constraint_node, 4, .production_id = 1),
  [962] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_kdl_node, 9, .production_id = 34),
  [964] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_kdl_node, 9, .production_id = 34),
  [966] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_arco_pure_math_node, 6, .production_id = 6),
  [968] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_arco_pure_math_node, 6, .production_id = 6),
  [970] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_arco_pure_math_node, 9, .production_id = 35),
  [972] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_arco_pure_math_node, 9, .production_id = 35),
  [974] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_arco_constraint_node, 9, .production_id = 37),
  [976] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_arco_constraint_node, 9, .production_id = 37),
  [978] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_kdl_node, 4, .production_id = 5),
  [980] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_kdl_node, 4, .production_id = 5),
  [982] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_kdl_node, 5, .production_id = 5),
  [984] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_kdl_node, 5, .production_id = 5),
  [986] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_arco_constraint_node, 9, .production_id = 35),
  [988] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_arco_constraint_node, 9, .production_id = 35),
  [990] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_arco_pure_math_node, 2, .production_id = 1),
  [992] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_arco_pure_math_node, 2, .production_id = 1),
  [994] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_kdl_node, 9, .production_id = 36),
  [996] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_kdl_node, 9, .production_id = 36),
  [998] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_arco_pure_math_node, 5, .production_id = 6),
  [1000] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_arco_pure_math_node, 5, .production_id = 6),
  [1002] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_arco_constraint_node, 5, .production_id = 24),
  [1004] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_arco_constraint_node, 5, .production_id = 24),
  [1006] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_arco_pure_math_node, 5, .production_id = 15),
  [1008] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_arco_pure_math_node, 5, .production_id = 15),
  [1010] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_arco_constraint_node, 5, .production_id = 14),
  [1012] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_arco_constraint_node, 5, .production_id = 14),
  [1014] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_arco_constraint_node, 5, .production_id = 4),
  [1016] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_arco_constraint_node, 5, .production_id = 4),
  [1018] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_arco_pure_math_node, 5, .production_id = 2),
  [1020] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_arco_pure_math_node, 5, .production_id = 2),
  [1022] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_arco_pure_math_node, 9, .production_id = 37),
  [1024] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_arco_pure_math_node, 9, .production_id = 37),
  [1026] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_arco_constraint_node, 5, .production_id = 6),
  [1028] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_arco_constraint_node, 5, .production_id = 6),
  [1030] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_arco_pure_math_node, 9, .production_id = 38),
  [1032] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_arco_pure_math_node, 9, .production_id = 38),
  [1034] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_arco_constraint_node, 5, .production_id = 15),
  [1036] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_arco_constraint_node, 5, .production_id = 15),
  [1038] = {.entry = {.count = 1, .reusable = true}}, SHIFT(563),
  [1040] = {.entry = {.count = 1, .reusable = true}}, SHIFT(182),
  [1042] = {.entry = {.count = 1, .reusable = true}}, SHIFT(240),
  [1044] = {.entry = {.count = 1, .reusable = true}}, SHIFT(173),
  [1046] = {.entry = {.count = 1, .reusable = true}}, SHIFT(174),
  [1048] = {.entry = {.count = 1, .reusable = true}}, SHIFT(203),
  [1050] = {.entry = {.count = 1, .reusable = true}}, SHIFT(562),
  [1052] = {.entry = {.count = 1, .reusable = true}}, SHIFT(559),
  [1054] = {.entry = {.count = 1, .reusable = true}}, SHIFT(177),
  [1056] = {.entry = {.count = 1, .reusable = true}}, SHIFT(186),
  [1058] = {.entry = {.count = 1, .reusable = true}}, SHIFT(225),
  [1060] = {.entry = {.count = 1, .reusable = true}}, SHIFT(184),
  [1062] = {.entry = {.count = 1, .reusable = true}}, SHIFT(253),
  [1064] = {.entry = {.count = 1, .reusable = true}}, SHIFT(233),
  [1066] = {.entry = {.count = 1, .reusable = true}}, SHIFT(168),
  [1068] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym__bare_identifier, 1),
  [1070] = {.entry = {.count = 1, .reusable = true}}, SHIFT(485),
  [1072] = {.entry = {.count = 1, .reusable = true}}, SHIFT(602),
  [1074] = {.entry = {.count = 1, .reusable = true}}, SHIFT(599),
  [1076] = {.entry = {.count = 1, .reusable = true}}, SHIFT(584),
  [1078] = {.entry = {.count = 1, .reusable = true}}, SHIFT(255),
  [1080] = {.entry = {.count = 1, .reusable = true}}, SHIFT(183),
  [1082] = {.entry = {.count = 1, .reusable = true}}, SHIFT(309),
  [1084] = {.entry = {.count = 1, .reusable = true}}, SHIFT(231),
  [1086] = {.entry = {.count = 1, .reusable = true}}, SHIFT(489),
  [1088] = {.entry = {.count = 1, .reusable = true}}, SHIFT(227),
  [1090] = {.entry = {.count = 1, .reusable = true}}, SHIFT(379),
  [1092] = {.entry = {.count = 1, .reusable = true}}, SHIFT(226),
  [1094] = {.entry = {.count = 1, .reusable = true}}, SHIFT(107),
  [1096] = {.entry = {.count = 1, .reusable = true}}, SHIFT(384),
  [1098] = {.entry = {.count = 1, .reusable = true}}, SHIFT(221),
  [1100] = {.entry = {.count = 1, .reusable = true}}, SHIFT(219),
  [1102] = {.entry = {.count = 1, .reusable = true}}, SHIFT(391),
  [1104] = {.entry = {.count = 1, .reusable = true}}, SHIFT(216),
  [1106] = {.entry = {.count = 1, .reusable = true}}, SHIFT(215),
  [1108] = {.entry = {.count = 1, .reusable = true}}, SHIFT(400),
  [1110] = {.entry = {.count = 1, .reusable = true}}, SHIFT(213),
  [1112] = {.entry = {.count = 1, .reusable = true}}, SHIFT(212),
  [1114] = {.entry = {.count = 1, .reusable = true}}, SHIFT(407),
  [1116] = {.entry = {.count = 1, .reusable = true}}, SHIFT(209),
  [1118] = {.entry = {.count = 1, .reusable = true}}, SHIFT(208),
  [1120] = {.entry = {.count = 1, .reusable = true}}, SHIFT(414),
  [1122] = {.entry = {.count = 1, .reusable = true}}, SHIFT(206),
  [1124] = {.entry = {.count = 1, .reusable = true}}, SHIFT(419),
  [1126] = {.entry = {.count = 1, .reusable = true}}, SHIFT(202),
  [1128] = {.entry = {.count = 1, .reusable = true}}, SHIFT(425),
  [1130] = {.entry = {.count = 1, .reusable = true}}, SHIFT(187),
  [1132] = {.entry = {.count = 1, .reusable = true}}, SHIFT(191),
  [1134] = {.entry = {.count = 1, .reusable = true}}, SHIFT(192),
  [1136] = {.entry = {.count = 1, .reusable = true}}, SHIFT(138),
  [1138] = {.entry = {.count = 1, .reusable = true}}, SHIFT(374),
  [1140] = {.entry = {.count = 1, .reusable = true}}, SHIFT(130),
  [1142] = {.entry = {.count = 1, .reusable = true}}, SHIFT(431),
  [1144] = {.entry = {.count = 1, .reusable = true}}, SHIFT(143),
  [1146] = {.entry = {.count = 1, .reusable = true}}, SHIFT(439),
  [1148] = {.entry = {.count = 1, .reusable = true}}, REDUCE(aux_sym__integer_repeat1, 2),
  [1150] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym__integer_repeat1, 2), SHIFT_REPEAT(328),
  [1153] = {.entry = {.count = 1, .reusable = true}}, SHIFT(230),
  [1155] = {.entry = {.count = 1, .reusable = true}}, SHIFT(150),
  [1157] = {.entry = {.count = 1, .reusable = true}}, SHIFT(442),
  [1159] = {.entry = {.count = 1, .reusable = true}}, SHIFT(237),
  [1161] = {.entry = {.count = 1, .reusable = true}}, SHIFT(156),
  [1163] = {.entry = {.count = 1, .reusable = true}}, SHIFT(448),
  [1165] = {.entry = {.count = 1, .reusable = true}}, SHIFT(249),
  [1167] = {.entry = {.count = 1, .reusable = true}}, SHIFT(162),
  [1169] = {.entry = {.count = 1, .reusable = true}}, SHIFT(460),
  [1171] = {.entry = {.count = 1, .reusable = true}}, SHIFT(263),
  [1173] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_type, 3),
  [1175] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_type, 3),
  [1177] = {.entry = {.count = 1, .reusable = true}}, SHIFT(261),
  [1179] = {.entry = {.count = 1, .reusable = true}}, SHIFT(197),
  [1181] = {.entry = {.count = 1, .reusable = true}}, SHIFT(196),
  [1183] = {.entry = {.count = 1, .reusable = true}}, SHIFT(254),
  [1185] = {.entry = {.count = 1, .reusable = true}}, SHIFT(252),
  [1187] = {.entry = {.count = 1, .reusable = true}}, SHIFT(248),
  [1189] = {.entry = {.count = 1, .reusable = true}}, SHIFT(243),
  [1191] = {.entry = {.count = 1, .reusable = true}}, SHIFT(367),
  [1193] = {.entry = {.count = 1, .reusable = true}}, SHIFT(235),
  [1195] = {.entry = {.count = 1, .reusable = true}}, SHIFT(373),
  [1197] = {.entry = {.count = 1, .reusable = true}}, SHIFT(246),
  [1199] = {.entry = {.count = 1, .reusable = true}}, SHIFT(239),
  [1201] = {.entry = {.count = 1, .reusable = true}}, SHIFT(236),
  [1203] = {.entry = {.count = 1, .reusable = true}}, SHIFT(242),
  [1205] = {.entry = {.count = 1, .reusable = true}}, SHIFT(250),
  [1207] = {.entry = {.count = 1, .reusable = true}}, SHIFT(244),
  [1209] = {.entry = {.count = 1, .reusable = true}}, SHIFT(172),
  [1211] = {.entry = {.count = 1, .reusable = true}}, SHIFT(169),
  [1213] = {.entry = {.count = 1, .reusable = true}}, SHIFT(247),
  [1215] = {.entry = {.count = 1, .reusable = true}}, SHIFT(200),
  [1217] = {.entry = {.count = 1, .reusable = true}}, SHIFT(251),
  [1219] = {.entry = {.count = 1, .reusable = true}}, SHIFT(234),
  [1221] = {.entry = {.count = 1, .reusable = true}}, SHIFT(165),
  [1223] = {.entry = {.count = 1, .reusable = true}}, SHIFT(256),
  [1225] = {.entry = {.count = 1, .reusable = true}}, SHIFT(359),
  [1227] = {.entry = {.count = 1, .reusable = true}}, SHIFT(164),
  [1229] = {.entry = {.count = 1, .reusable = true}}, SHIFT(258),
  [1231] = {.entry = {.count = 1, .reusable = true}}, SHIFT(163),
  [1233] = {.entry = {.count = 1, .reusable = true}}, SHIFT(259),
  [1235] = {.entry = {.count = 1, .reusable = true}}, SHIFT(159),
  [1237] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym__integer, 1),
  [1239] = {.entry = {.count = 1, .reusable = true}}, SHIFT(420),
  [1241] = {.entry = {.count = 1, .reusable = true}}, SHIFT(158),
  [1243] = {.entry = {.count = 1, .reusable = true}}, SHIFT(228),
  [1245] = {.entry = {.count = 1, .reusable = true}}, SHIFT(329),
  [1247] = {.entry = {.count = 1, .reusable = true}}, SHIFT(157),
  [1249] = {.entry = {.count = 1, .reusable = true}}, SHIFT(224),
  [1251] = {.entry = {.count = 1, .reusable = true}}, SHIFT(332),
  [1253] = {.entry = {.count = 1, .reusable = true}}, SHIFT(154),
  [1255] = {.entry = {.count = 1, .reusable = true}}, SHIFT(205),
  [1257] = {.entry = {.count = 1, .reusable = true}}, SHIFT(152),
  [1259] = {.entry = {.count = 1, .reusable = true}}, SHIFT(193),
  [1261] = {.entry = {.count = 1, .reusable = true}}, SHIFT(343),
  [1263] = {.entry = {.count = 1, .reusable = true}}, SHIFT(147),
  [1265] = {.entry = {.count = 1, .reusable = true}}, SHIFT(440),
  [1267] = {.entry = {.count = 1, .reusable = true}}, SHIFT(181),
  [1269] = {.entry = {.count = 1, .reusable = true}}, SHIFT(145),
  [1271] = {.entry = {.count = 1, .reusable = true}}, SHIFT(144),
  [1273] = {.entry = {.count = 1, .reusable = true}}, SHIFT(141),
  [1275] = {.entry = {.count = 1, .reusable = true}}, SHIFT(435),
  [1277] = {.entry = {.count = 1, .reusable = true}}, SHIFT(140),
  [1279] = {.entry = {.count = 1, .reusable = true}}, SHIFT(433),
  [1281] = {.entry = {.count = 1, .reusable = true}}, SHIFT(198),
  [1283] = {.entry = {.count = 1, .reusable = true}}, SHIFT(179),
  [1285] = {.entry = {.count = 1, .reusable = true}}, SHIFT(337),
  [1287] = {.entry = {.count = 1, .reusable = true}}, SHIFT(133),
  [1289] = {.entry = {.count = 1, .reusable = true}}, SHIFT(178),
  [1291] = {.entry = {.count = 1, .reusable = true}}, SHIFT(132),
  [1293] = {.entry = {.count = 1, .reusable = true}}, SHIFT(131),
  [1295] = {.entry = {.count = 1, .reusable = true}}, SHIFT(429),
  [1297] = {.entry = {.count = 1, .reusable = true}}, SHIFT(136),
  [1299] = {.entry = {.count = 1, .reusable = true}}, SHIFT(129),
  [1301] = {.entry = {.count = 1, .reusable = true}}, SHIFT(170),
  [1303] = {.entry = {.count = 1, .reusable = true}}, SHIFT(128),
  [1305] = {.entry = {.count = 1, .reusable = true}}, SHIFT(127),
  [1307] = {.entry = {.count = 1, .reusable = true}}, SHIFT(417),
  [1309] = {.entry = {.count = 1, .reusable = true}}, SHIFT(220),
  [1311] = {.entry = {.count = 1, .reusable = true}}, SHIFT(387),
  [1313] = {.entry = {.count = 1, .reusable = true}}, SHIFT(126),
  [1315] = {.entry = {.count = 1, .reusable = true}}, SHIFT(167),
  [1317] = {.entry = {.count = 1, .reusable = true}}, SHIFT(125),
  [1319] = {.entry = {.count = 1, .reusable = true}}, SHIFT(123),
  [1321] = {.entry = {.count = 1, .reusable = true}}, SHIFT(405),
  [1323] = {.entry = {.count = 1, .reusable = true}}, REDUCE(aux_sym_kdl_node_repeat1, 2),
  [1325] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_kdl_node_repeat1, 2), SHIFT_REPEAT(543),
  [1328] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_kdl_node_repeat1, 2), SHIFT_REPEAT(95),
  [1331] = {.entry = {.count = 1, .reusable = true}}, SHIFT(161),
  [1333] = {.entry = {.count = 1, .reusable = true}}, SHIFT(122),
  [1335] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym__integer, 2),
  [1337] = {.entry = {.count = 1, .reusable = true}}, SHIFT(328),
  [1339] = {.entry = {.count = 1, .reusable = true}}, SHIFT(121),
  [1341] = {.entry = {.count = 1, .reusable = true}}, SHIFT(398),
  [1343] = {.entry = {.count = 1, .reusable = true}}, SHIFT(155),
  [1345] = {.entry = {.count = 1, .reusable = true}}, SHIFT(118),
  [1347] = {.entry = {.count = 1, .reusable = true}}, SHIFT(117),
  [1349] = {.entry = {.count = 1, .reusable = true}}, SHIFT(382),
  [1351] = {.entry = {.count = 1, .reusable = true}}, SHIFT(175),
  [1353] = {.entry = {.count = 1, .reusable = true}}, SHIFT(238),
  [1355] = {.entry = {.count = 1, .reusable = true}}, SHIFT(151),
  [1357] = {.entry = {.count = 1, .reusable = true}}, SHIFT(148),
  [1359] = {.entry = {.count = 1, .reusable = true}}, SHIFT(146),
  [1361] = {.entry = {.count = 1, .reusable = true}}, SHIFT(142),
  [1363] = {.entry = {.count = 1, .reusable = true}}, SHIFT(241),
  [1365] = {.entry = {.count = 1, .reusable = true}}, SHIFT(368),
  [1367] = {.entry = {.count = 1, .reusable = true}}, SHIFT(139),
  [1369] = {.entry = {.count = 1, .reusable = true}}, SHIFT(137),
  [1371] = {.entry = {.count = 1, .reusable = true}}, SHIFT(113),
  [1373] = {.entry = {.count = 1, .reusable = true}}, SHIFT(204),
  [1375] = {.entry = {.count = 1, .reusable = true}}, SHIFT(257),
  [1377] = {.entry = {.count = 1, .reusable = true}}, SHIFT(355),
  [1379] = {.entry = {.count = 1, .reusable = true}}, SHIFT(262),
  [1381] = {.entry = {.count = 1, .reusable = true}}, SHIFT(109),
  [1383] = {.entry = {.count = 1, .reusable = true}}, SHIFT(264),
  [1385] = {.entry = {.count = 1, .reusable = true}}, SHIFT(350),
  [1387] = {.entry = {.count = 1, .reusable = true}}, SHIFT(110),
  [1389] = {.entry = {.count = 1, .reusable = true}}, SHIFT(199),
  [1391] = {.entry = {.count = 1, .reusable = true}}, SHIFT(318),
  [1393] = {.entry = {.count = 1, .reusable = true}}, SHIFT(210),
  [1395] = {.entry = {.count = 1, .reusable = true}}, SHIFT(124),
  [1397] = {.entry = {.count = 1, .reusable = true}}, SHIFT(315),
  [1399] = {.entry = {.count = 1, .reusable = true}}, SHIFT(176),
  [1401] = {.entry = {.count = 1, .reusable = true}}, SHIFT(312),
  [1403] = {.entry = {.count = 1, .reusable = true}}, SHIFT(232),
  [1405] = {.entry = {.count = 1, .reusable = true}}, SHIFT(112),
  [1407] = {.entry = {.count = 1, .reusable = true}}, SHIFT(189),
  [1409] = {.entry = {.count = 1, .reusable = true}}, SHIFT(306),
  [1411] = {.entry = {.count = 1, .reusable = true}}, SHIFT(194),
  [1413] = {.entry = {.count = 1, .reusable = true}}, SHIFT(303),
  [1415] = {.entry = {.count = 1, .reusable = true}}, SHIFT(111),
  [1417] = {.entry = {.count = 1, .reusable = false}}, SHIFT(290),
  [1419] = {.entry = {.count = 1, .reusable = false}}, SHIFT(276),
  [1421] = {.entry = {.count = 1, .reusable = false}}, SHIFT(295),
  [1423] = {.entry = {.count = 1, .reusable = false}}, SHIFT(274),
  [1425] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym__binary, 4),
  [1427] = {.entry = {.count = 1, .reusable = true}}, SHIFT(470),
  [1429] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym__binary, 3),
  [1431] = {.entry = {.count = 1, .reusable = true}}, SHIFT(464),
  [1433] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym__decimal, 2),
  [1435] = {.entry = {.count = 1, .reusable = true}}, SHIFT(579),
  [1437] = {.entry = {.count = 1, .reusable = true}}, SHIFT(565),
  [1439] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym__decimal, 1),
  [1441] = {.entry = {.count = 1, .reusable = true}}, SHIFT(577),
  [1443] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym__binary, 2),
  [1445] = {.entry = {.count = 1, .reusable = true}}, SHIFT(469),
  [1447] = {.entry = {.count = 1, .reusable = true}}, REDUCE(aux_sym__binary_repeat1, 2),
  [1449] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym__binary_repeat1, 2), SHIFT_REPEAT(470),
  [1452] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym__octal, 3),
  [1454] = {.entry = {.count = 1, .reusable = true}}, SHIFT(475),
  [1456] = {.entry = {.count = 1, .reusable = true}}, SHIFT(480),
  [1458] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym__decimal, 4, .production_id = 32),
  [1460] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym__bare_identifier, 3),
  [1462] = {.entry = {.count = 1, .reusable = true}}, SHIFT(477),
  [1464] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym__octal, 4),
  [1466] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym__hex, 4),
  [1468] = {.entry = {.count = 1, .reusable = true}}, SHIFT(483),
  [1470] = {.entry = {.count = 1, .reusable = true}}, REDUCE(aux_sym__bare_identifier_repeat1, 2),
  [1472] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym__bare_identifier_repeat1, 2), SHIFT_REPEAT(477),
  [1475] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym__hex, 3),
  [1477] = {.entry = {.count = 1, .reusable = true}}, SHIFT(476),
  [1479] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym__decimal, 3, .production_id = 23),
  [1481] = {.entry = {.count = 1, .reusable = true}}, REDUCE(aux_sym__octal_repeat1, 2),
  [1483] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym__octal_repeat1, 2), SHIFT_REPEAT(480),
  [1486] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym__octal, 2),
  [1488] = {.entry = {.count = 1, .reusable = true}}, SHIFT(472),
  [1490] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym__raw_string, 3),
  [1492] = {.entry = {.count = 1, .reusable = true}}, SHIFT(487),
  [1494] = {.entry = {.count = 1, .reusable = true}}, REDUCE(aux_sym__hex_repeat1, 2),
  [1496] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym__hex_repeat1, 2), SHIFT_REPEAT(483),
  [1499] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym__hex, 2),
  [1501] = {.entry = {.count = 1, .reusable = true}}, SHIFT(486),
  [1503] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym__bare_identifier, 2),
  [1505] = {.entry = {.count = 1, .reusable = true}}, SHIFT(474),
  [1507] = {.entry = {.count = 1, .reusable = true}}, REDUCE(aux_sym__raw_string_repeat1, 2),
  [1509] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym__raw_string_repeat1, 2), SHIFT_REPEAT(487),
  [1512] = {.entry = {.count = 1, .reusable = true}}, SHIFT(532),
  [1514] = {.entry = {.count = 1, .reusable = false}}, SHIFT(504),
  [1516] = {.entry = {.count = 1, .reusable = true}}, SHIFT(591),
  [1518] = {.entry = {.count = 1, .reusable = true}}, SHIFT(504),
  [1520] = {.entry = {.count = 2, .reusable = true}}, REDUCE(sym__node_space, 1), REDUCE(aux_sym__node_space_repeat1, 2),
  [1523] = {.entry = {.count = 3, .reusable = true}}, REDUCE(sym__node_space, 1), REDUCE(aux_sym__node_space_repeat1, 2), SHIFT(546),
  [1527] = {.entry = {.count = 3, .reusable = true}}, REDUCE(sym__node_space, 1), REDUCE(aux_sym__node_space_repeat1, 2), SHIFT(90),
  [1531] = {.entry = {.count = 1, .reusable = true}}, SHIFT(533),
  [1533] = {.entry = {.count = 1, .reusable = false}}, SHIFT(521),
  [1535] = {.entry = {.count = 1, .reusable = true}}, SHIFT(598),
  [1537] = {.entry = {.count = 1, .reusable = true}}, SHIFT(521),
  [1539] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_string, 1),
  [1541] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_bare_identifier, 1),
  [1543] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_identifier, 1),
  [1545] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym__escaped_string, 3, .production_id = 3),
  [1547] = {.entry = {.count = 1, .reusable = true}}, SHIFT(552),
  [1549] = {.entry = {.count = 1, .reusable = false}}, SHIFT(520),
  [1551] = {.entry = {.count = 1, .reusable = true}}, SHIFT(605),
  [1553] = {.entry = {.count = 1, .reusable = true}}, SHIFT(520),
  [1555] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym__escaped_string, 2),
  [1557] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_value, 1),
  [1559] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym__exponent, 2),
  [1561] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_number, 1),
  [1563] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym__exponent, 3),
  [1565] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_value, 2),
  [1567] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym__node_field, 1),
  [1569] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_keyword, 1),
  [1571] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_prop, 3),
  [1573] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym__decimal, 3),
  [1575] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym__node_field_comment, 2, .production_id = 13),
  [1577] = {.entry = {.count = 2, .reusable = true}}, REDUCE(sym__node_space, 1), REDUCE(sym__node_space, 2),
  [1580] = {.entry = {.count = 3, .reusable = true}}, REDUCE(sym__node_space, 1), REDUCE(sym__node_space, 2), SHIFT(517),
  [1584] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym__node_field_comment, 3, .production_id = 22),
  [1586] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym__decimal, 5, .production_id = 32),
  [1588] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_node_field, 1),
  [1590] = {.entry = {.count = 2, .reusable = true}}, REDUCE(sym__node_space, 2), REDUCE(sym__node_space, 3),
  [1593] = {.entry = {.count = 3, .reusable = true}}, REDUCE(sym__node_space, 2), REDUCE(sym__node_space, 3), SHIFT(90),
  [1597] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_boolean, 1),
  [1599] = {.entry = {.count = 1, .reusable = true}}, SHIFT(542),
  [1601] = {.entry = {.count = 1, .reusable = true}}, SHIFT(540),
  [1603] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym__decimal, 4, .production_id = 23),
  [1605] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_arco_constraint_math_children, 4, .production_id = 11),
  [1607] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_node_children, 6, .production_id = 11),
  [1609] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_node_children, 3),
  [1611] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_arco_constraint_math_children, 3),
  [1613] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_node_children, 3, .production_id = 11),
  [1615] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_arco_constraint_math_children, 5, .production_id = 11),
  [1617] = {.entry = {.count = 1, .reusable = true}}, SHIFT(266),
  [1619] = {.entry = {.count = 1, .reusable = true}}, SHIFT(79),
  [1621] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_node_children, 4),
  [1623] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_arco_constraint_math_children, 5, .production_id = 31),
  [1625] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_arco_pure_math_children, 3, .production_id = 11),
  [1627] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_arco_pure_math_children, 2),
  [1629] = {.entry = {.count = 1, .reusable = true}}, SHIFT(105),
  [1631] = {.entry = {.count = 1, .reusable = true}}, SHIFT(555),
  [1633] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_node_children, 5),
  [1635] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_node_children, 5, .production_id = 11),
  [1637] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_node_children, 7, .production_id = 11),
  [1639] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_node_children, 4, .production_id = 11),
  [1641] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_node_children, 6),
  [1643] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_arco_pure_math_children, 3),
  [1645] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_arco_constraint_math_children, 4, .production_id = 21),
  [1647] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_arco_pure_math_children, 5, .production_id = 11),
  [1649] = {.entry = {.count = 1, .reusable = true}}, SHIFT(265),
  [1651] = {.entry = {.count = 1, .reusable = true}}, SHIFT(529),
  [1653] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_arco_constraint_math_children, 3, .production_id = 12),
  [1655] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_arco_pure_math_children, 3, .production_id = 12),
  [1657] = {.entry = {.count = 1, .reusable = true}}, SHIFT(100),
  [1659] = {.entry = {.count = 1, .reusable = true}}, SHIFT(534),
  [1661] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_arco_constraint_math_children, 3, .production_id = 11),
  [1663] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_arco_constraint_math_children, 2),
  [1665] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_node_children, 2),
  [1667] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_node_children, 8, .production_id = 11),
  [1669] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_arco_pure_math_children, 4, .production_id = 21),
  [1671] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_arco_pure_math_children, 4, .production_id = 11),
  [1673] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_arco_pure_math_children, 5, .production_id = 31),
  [1675] = {.entry = {.count = 1, .reusable = true}}, SHIFT(80),
  [1677] = {.entry = {.count = 1, .reusable = true}}, SHIFT(558),
  [1679] = {.entry = {.count = 1, .reusable = true}}, SHIFT(99),
  [1681] = {.entry = {.count = 1, .reusable = true}}, SHIFT(556),
  [1683] = {.entry = {.count = 1, .reusable = true}}, SHIFT(104),
  [1685] = {.entry = {.count = 1, .reusable = true}}, SHIFT(78),
  [1687] = {.entry = {.count = 1, .reusable = true}}, SHIFT(554),
  [1689] = {.entry = {.count = 1, .reusable = true}}, REDUCE(aux_sym_single_line_comment_repeat1, 2),
  [1691] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_single_line_comment_repeat1, 2), SHIFT_REPEAT(558),
  [1694] = {.entry = {.count = 1, .reusable = true}}, SHIFT(375),
  [1696] = {.entry = {.count = 1, .reusable = true}}, SHIFT(580),
  [1698] = {.entry = {.count = 1, .reusable = true}}, SHIFT(595),
  [1700] = {.entry = {.count = 1, .reusable = false}}, SHIFT(574),
  [1702] = {.entry = {.count = 1, .reusable = true}}, SHIFT(574),
  [1704] = {.entry = {.count = 1, .reusable = true}}, SHIFT(586),
  [1706] = {.entry = {.count = 1, .reusable = true}}, SHIFT(494),
  [1708] = {.entry = {.count = 1, .reusable = true}}, REDUCE(aux_sym__escaped_string_repeat1, 2),
  [1710] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym__escaped_string_repeat1, 2), SHIFT_REPEAT(574),
  [1713] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym__escaped_string_repeat1, 2), SHIFT_REPEAT(574),
  [1716] = {.entry = {.count = 1, .reusable = true}}, SHIFT(496),
  [1718] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym__bare_identifier_repeat1, 2), SHIFT_REPEAT(571),
  [1721] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym__raw_string_repeat1, 2), SHIFT_REPEAT(572),
  [1724] = {.entry = {.count = 1, .reusable = true}}, SHIFT(571),
  [1726] = {.entry = {.count = 1, .reusable = true}}, REDUCE(aux_sym__escaped_string_repeat1, 1),
  [1728] = {.entry = {.count = 1, .reusable = false}}, REDUCE(aux_sym__escaped_string_repeat1, 1),
  [1730] = {.entry = {.count = 1, .reusable = true}}, SHIFT(572),
  [1732] = {.entry = {.count = 1, .reusable = true}}, SHIFT(573),
  [1734] = {.entry = {.count = 1, .reusable = true}}, SHIFT(482),
  [1736] = {.entry = {.count = 1, .reusable = true}}, SHIFT(575),
  [1738] = {.entry = {.count = 1, .reusable = true}}, SHIFT(468),
  [1740] = {.entry = {.count = 1, .reusable = true}}, SHIFT(576),
  [1742] = {.entry = {.count = 1, .reusable = true}}, SHIFT(465),
  [1744] = {.entry = {.count = 1, .reusable = true}}, SHIFT(541),
  [1746] = {.entry = {.count = 1, .reusable = true}}, SHIFT(101),
  [1748] = {.entry = {.count = 1, .reusable = true}}, SHIFT(544),
  [1750] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_annotation_type, 1),
  [1752] = {.entry = {.count = 1, .reusable = true}}, SHIFT(339),
  [1754] = {.entry = {.count = 1, .reusable = true}}, SHIFT(551),
  [1756] = {.entry = {.count = 1, .reusable = true}}, SHIFT(531),
  [1758] = {.entry = {.count = 1, .reusable = true}},  ACCEPT_INPUT(),
  [1760] = {.entry = {.count = 1, .reusable = true}}, SHIFT(545),
  [1762] = {.entry = {.count = 1, .reusable = true}}, SHIFT(471),
  [1764] = {.entry = {.count = 1, .reusable = true}}, SHIFT(603),
  [1766] = {.entry = {.count = 1, .reusable = true}}, SHIFT(578),
  [1768] = {.entry = {.count = 1, .reusable = true}}, SHIFT(478),
  [1770] = {.entry = {.count = 1, .reusable = true}}, SHIFT(498),
  [1772] = {.entry = {.count = 1, .reusable = true}}, SHIFT(481),
  [1774] = {.entry = {.count = 1, .reusable = true}}, SHIFT(553),
  [1776] = {.entry = {.count = 1, .reusable = true}}, SHIFT(484),
  [1778] = {.entry = {.count = 1, .reusable = true}}, SHIFT(597),
  [1780] = {.entry = {.count = 1, .reusable = true}}, SHIFT(581),
  [1782] = {.entry = {.count = 1, .reusable = true}}, SHIFT(607),
};

enum ts_external_scanner_symbol_identifiers {
  ts_external_token__eof = 0,
  ts_external_token_multi_line_comment = 1,
  ts_external_token__implicit_terminator = 2,
};

static const TSSymbol ts_external_scanner_symbol_map[EXTERNAL_TOKEN_COUNT] = {
  [ts_external_token__eof] = sym__eof,
  [ts_external_token_multi_line_comment] = sym_multi_line_comment,
  [ts_external_token__implicit_terminator] = sym__implicit_terminator,
};

static const bool ts_external_scanner_states[4][EXTERNAL_TOKEN_COUNT] = {
  [1] = {
    [ts_external_token__eof] = true,
    [ts_external_token_multi_line_comment] = true,
    [ts_external_token__implicit_terminator] = true,
  },
  [2] = {
    [ts_external_token_multi_line_comment] = true,
  },
  [3] = {
    [ts_external_token__eof] = true,
    [ts_external_token_multi_line_comment] = true,
  },
};

#ifdef __cplusplus
extern "C" {
#endif
void *tree_sitter_arco_kdl_external_scanner_create(void);
void tree_sitter_arco_kdl_external_scanner_destroy(void *);
bool tree_sitter_arco_kdl_external_scanner_scan(void *, TSLexer *, const bool *);
unsigned tree_sitter_arco_kdl_external_scanner_serialize(void *, char *);
void tree_sitter_arco_kdl_external_scanner_deserialize(void *, const char *, unsigned);

#ifdef _WIN32
#define extern __declspec(dllexport)
#endif

extern const TSLanguage *tree_sitter_arco_kdl(void) {
  static const TSLanguage language = {
    .version = LANGUAGE_VERSION,
    .symbol_count = SYMBOL_COUNT,
    .alias_count = ALIAS_COUNT,
    .token_count = TOKEN_COUNT,
    .external_token_count = EXTERNAL_TOKEN_COUNT,
    .state_count = STATE_COUNT,
    .large_state_count = LARGE_STATE_COUNT,
    .production_id_count = PRODUCTION_ID_COUNT,
    .field_count = FIELD_COUNT,
    .max_alias_sequence_length = MAX_ALIAS_SEQUENCE_LENGTH,
    .parse_table = &ts_parse_table[0][0],
    .small_parse_table = ts_small_parse_table,
    .small_parse_table_map = ts_small_parse_table_map,
    .parse_actions = ts_parse_actions,
    .symbol_names = ts_symbol_names,
    .field_names = ts_field_names,
    .field_map_slices = ts_field_map_slices,
    .field_map_entries = ts_field_map_entries,
    .symbol_metadata = ts_symbol_metadata,
    .public_symbol_map = ts_symbol_map,
    .alias_map = ts_non_terminal_alias_map,
    .alias_sequences = &ts_alias_sequences[0][0],
    .lex_modes = ts_lex_modes,
    .lex_fn = ts_lex,
    .keyword_lex_fn = ts_lex_keywords,
    .keyword_capture_token = sym__normal_bare_identifier,
    .external_scanner = {
      &ts_external_scanner_states[0][0],
      ts_external_scanner_symbol_map,
      tree_sitter_arco_kdl_external_scanner_create,
      tree_sitter_arco_kdl_external_scanner_destroy,
      tree_sitter_arco_kdl_external_scanner_scan,
      tree_sitter_arco_kdl_external_scanner_serialize,
      tree_sitter_arco_kdl_external_scanner_deserialize,
    },
    .primary_state_ids = ts_primary_state_ids,
  };
  return &language;
}
#ifdef __cplusplus
}
#endif
