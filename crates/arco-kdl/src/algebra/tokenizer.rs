use crate::algebra::error::ParseError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum TokenKind {
    Identifier(String),
    Number(String),
    String(String),
    LParen,
    RParen,
    LBracket,
    RBracket,
    Comma,
    Plus,
    Minus,
    Star,
    Slash,
    Equal,
    DoubleEqual,
    NotEqual,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
    KeywordFor,
    KeywordIf,
    KeywordIn,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct Token {
    pub(super) kind: TokenKind,
    pub(super) position: usize,
}

pub(super) fn tokenize(text: &str) -> Result<Vec<Token>, ParseError> {
    let bytes = text.as_bytes();
    let mut tokens = Vec::new();
    let mut index = 0usize;

    while index < bytes.len() {
        let character = bytes[index] as char;
        if character.is_ascii_whitespace() {
            index += 1;
            continue;
        }

        let token = if let Some(kind) = single_char_token_kind(character) {
            Token {
                kind,
                position: index,
            }
        } else if let Some(token) = parse_comparison_token(bytes, &mut index, character)? {
            token
        } else if character == '"' {
            parse_string_token(bytes, &mut index)?
        } else if character.is_ascii_digit() {
            parse_number_token(text, bytes, &mut index)
        } else if is_identifier_start(character) {
            parse_identifier_token(text, bytes, &mut index)
        } else {
            return Err(ParseError::new(
                index,
                format!("unexpected character `{character}`"),
            ));
        };

        tokens.push(token);
        index += 1;
    }

    Ok(tokens)
}

fn single_char_token_kind(character: char) -> Option<TokenKind> {
    match character {
        '(' => Some(TokenKind::LParen),
        ')' => Some(TokenKind::RParen),
        '[' => Some(TokenKind::LBracket),
        ']' => Some(TokenKind::RBracket),
        ',' => Some(TokenKind::Comma),
        '+' => Some(TokenKind::Plus),
        '-' => Some(TokenKind::Minus),
        '*' => Some(TokenKind::Star),
        '/' => Some(TokenKind::Slash),
        _ => None,
    }
}

fn parse_comparison_token(
    bytes: &[u8],
    index: &mut usize,
    character: char,
) -> Result<Option<Token>, ParseError> {
    let position = *index;
    let token = match character {
        '=' => {
            if bytes.get(position + 1) == Some(&b'=') {
                *index += 1;
                TokenKind::DoubleEqual
            } else {
                TokenKind::Equal
            }
        }
        '!' => {
            if bytes.get(position + 1) != Some(&b'=') {
                return Err(ParseError::new(position, "unexpected `!`"));
            }
            *index += 1;
            TokenKind::NotEqual
        }
        '<' => {
            if bytes.get(position + 1) == Some(&b'=') {
                *index += 1;
                TokenKind::LessEqual
            } else {
                TokenKind::Less
            }
        }
        '>' => {
            if bytes.get(position + 1) == Some(&b'=') {
                *index += 1;
                TokenKind::GreaterEqual
            } else {
                TokenKind::Greater
            }
        }
        _ => return Ok(None),
    };

    Ok(Some(Token {
        kind: token,
        position,
    }))
}

fn parse_string_token(bytes: &[u8], index: &mut usize) -> Result<Token, ParseError> {
    let start = *index;
    *index += 1;
    let mut value = String::new();
    let mut escaped = false;

    while *index < bytes.len() {
        let current = bytes[*index] as char;
        if escaped {
            value.push(match current {
                'n' => '\n',
                'r' => '\r',
                't' => '\t',
                '\\' => '\\',
                '"' => '"',
                other => other,
            });
            escaped = false;
        } else if current == '\\' {
            escaped = true;
        } else if current == '"' {
            return Ok(Token {
                kind: TokenKind::String(value),
                position: start,
            });
        } else {
            value.push(current);
        }
        *index += 1;
    }

    Err(ParseError::new(start, "unterminated string literal"))
}

fn parse_number_token(text: &str, bytes: &[u8], index: &mut usize) -> Token {
    let start = *index;
    while *index + 1 < bytes.len() {
        let next = bytes[*index + 1] as char;
        if next.is_ascii_digit() || next == '.' {
            *index += 1;
        } else {
            break;
        }
    }
    Token {
        kind: TokenKind::Number(text[start..=*index].to_string()),
        position: start,
    }
}

fn parse_identifier_token(text: &str, bytes: &[u8], index: &mut usize) -> Token {
    let start = *index;
    while *index + 1 < bytes.len() && is_identifier_continue(bytes[*index + 1] as char) {
        *index += 1;
    }

    let identifier = &text[start..=*index];
    let kind = match identifier {
        "for" => TokenKind::KeywordFor,
        "if" => TokenKind::KeywordIf,
        "in" => TokenKind::KeywordIn,
        _ => TokenKind::Identifier(identifier.to_string()),
    };

    Token {
        kind,
        position: start,
    }
}

pub(super) fn is_builtin_function(name: &str) -> bool {
    matches!(name, "sqrt" | "pow" | "exp" | "ln" | "abs")
}

fn is_identifier_start(character: char) -> bool {
    character.is_ascii_alphabetic() || character == '_'
}

fn is_identifier_continue(character: char) -> bool {
    character.is_ascii_alphanumeric() || character == '_'
}
