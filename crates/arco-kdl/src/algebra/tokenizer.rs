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
    KeywordAnd,
    KeywordOr,
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

        let token = match character {
            '(' => Token {
                kind: TokenKind::LParen,
                position: index,
            },
            ')' => Token {
                kind: TokenKind::RParen,
                position: index,
            },
            '[' => Token {
                kind: TokenKind::LBracket,
                position: index,
            },
            ']' => Token {
                kind: TokenKind::RBracket,
                position: index,
            },
            ',' => Token {
                kind: TokenKind::Comma,
                position: index,
            },
            '+' => Token {
                kind: TokenKind::Plus,
                position: index,
            },
            '-' => Token {
                kind: TokenKind::Minus,
                position: index,
            },
            '*' => Token {
                kind: TokenKind::Star,
                position: index,
            },
            '/' => Token {
                kind: TokenKind::Slash,
                position: index,
            },
            '=' => {
                if bytes.get(index + 1) == Some(&b'=') {
                    index += 1;
                    Token {
                        kind: TokenKind::DoubleEqual,
                        position: index - 1,
                    }
                } else {
                    Token {
                        kind: TokenKind::Equal,
                        position: index,
                    }
                }
            }
            '!' => {
                if bytes.get(index + 1) != Some(&b'=') {
                    return Err(ParseError::new(index, "unexpected `!`"));
                }
                index += 1;
                Token {
                    kind: TokenKind::NotEqual,
                    position: index - 1,
                }
            }
            '&' => {
                if bytes.get(index + 1) != Some(&b'&') {
                    return Err(ParseError::new(index, "unexpected `&`"));
                }
                index += 1;
                Token {
                    kind: TokenKind::KeywordAnd,
                    position: index - 1,
                }
            }
            '|' => {
                if bytes.get(index + 1) != Some(&b'|') {
                    return Err(ParseError::new(index, "unexpected `|`"));
                }
                index += 1;
                Token {
                    kind: TokenKind::KeywordOr,
                    position: index - 1,
                }
            }
            '<' => {
                if bytes.get(index + 1) == Some(&b'=') {
                    index += 1;
                    Token {
                        kind: TokenKind::LessEqual,
                        position: index - 1,
                    }
                } else {
                    Token {
                        kind: TokenKind::Less,
                        position: index,
                    }
                }
            }
            '>' => {
                if bytes.get(index + 1) == Some(&b'=') {
                    index += 1;
                    Token {
                        kind: TokenKind::GreaterEqual,
                        position: index - 1,
                    }
                } else {
                    Token {
                        kind: TokenKind::Greater,
                        position: index,
                    }
                }
            }
            '"' => {
                let start = index;
                index += 1;
                let mut value = String::new();
                let mut escaped = false;
                while index < bytes.len() {
                    let current = bytes[index] as char;
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
                        break;
                    } else {
                        value.push(current);
                    }
                    index += 1;
                }
                if index >= bytes.len() || bytes[index] as char != '"' {
                    return Err(ParseError::new(start, "unterminated string literal"));
                }
                Token {
                    kind: TokenKind::String(value),
                    position: start,
                }
            }
            character if character.is_ascii_digit() => {
                let start = index;
                while index + 1 < bytes.len() {
                    let next = bytes[index + 1] as char;
                    if next.is_ascii_digit() || next == '.' {
                        index += 1;
                    } else {
                        break;
                    }
                }
                Token {
                    kind: TokenKind::Number(text[start..=index].to_string()),
                    position: start,
                }
            }
            character if is_identifier_start(character) => {
                let start = index;
                while index + 1 < bytes.len() && is_identifier_continue(bytes[index + 1] as char) {
                    index += 1;
                }
                let identifier = &text[start..=index];
                let kind = match identifier {
                    "for" => TokenKind::KeywordFor,
                    "if" => TokenKind::KeywordIf,
                    "in" => TokenKind::KeywordIn,
                    "and" => TokenKind::KeywordAnd,
                    "or" => TokenKind::KeywordOr,
                    _ => TokenKind::Identifier(identifier.to_string()),
                };
                Token {
                    kind,
                    position: start,
                }
            }
            other => {
                return Err(ParseError::new(
                    index,
                    format!("unexpected character `{other}`"),
                ));
            }
        };

        tokens.push(token);
        index += 1;
    }

    Ok(tokens)
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
