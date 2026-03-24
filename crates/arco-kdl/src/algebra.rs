use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::error::Error;
use std::fmt::{self, Display, Formatter};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Expr {
    Number(String),
    String(String),
    Boolean(bool),
    Identifier(String),
    Indexed {
        target: String,
        indices: Vec<Expr>,
    },
    Unary {
        op: UnaryOp,
        expr: Box<Expr>,
    },
    Binary {
        op: BinaryOp,
        left: Box<Expr>,
        right: Box<Expr>,
    },
    Comparison {
        op: ComparisonOp,
        left: Box<Expr>,
        right: Box<Expr>,
    },
    FunctionCall {
        name: String,
        args: Vec<Expr>,
    },
    Reduction(ReductionExpr),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReductionExpr {
    pub op: ReductionOp,
    pub body: Box<Expr>,
    pub bindings: Vec<Binding>,
    pub filters: Vec<Expr>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Binding {
    pub pattern: BindingPattern,
    pub domain: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum BindingPattern {
    Name(String),
    Tuple(Vec<String>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReductionOp {
    Sum,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UnaryOp {
    Negate,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BinaryOp {
    Add,
    Subtract,
    Multiply,
    Divide,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ComparisonOp {
    Equal,
    DoubleEqual,
    LessEqual,
    GreaterEqual,
    Less,
    Greater,
    NotEqual,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConstraintBody {
    Comparison {
        op: ComparisonOp,
        left: Expr,
        right: Expr,
    },
    Range {
        lower: Expr,
        lower_op: ComparisonOp,
        middle: Expr,
        upper_op: ComparisonOp,
        upper: Expr,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseError {
    position: usize,
    message: String,
}

impl ParseError {
    fn new(position: usize, message: impl Into<String>) -> Self {
        Self {
            position,
            message: message.into(),
        }
    }

    pub fn position(&self) -> usize {
        self.position
    }
}

impl Display for ParseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{} at byte {}", self.message, self.position)
    }
}

impl Error for ParseError {}

#[derive(Debug, Clone, PartialEq, Eq)]
enum TokenKind {
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
struct Token {
    kind: TokenKind,
    position: usize,
}

pub fn parse_value_formula(text: &str) -> Result<Expr, ParseError> {
    parse_formula(text, |parser| parser.parse_value_expr())
}

pub fn parse_constraint_formula(text: &str) -> Result<ConstraintBody, ParseError> {
    parse_formula(text, |parser| {
        let left = parser.parse_arithmetic_expr()?;
        let op = parser.parse_comparison_operator()?;
        let middle = parser.parse_arithmetic_expr()?;
        Ok(if let Some(next_op) = parser.maybe_comparison_operator() {
            let upper = parser.parse_arithmetic_expr()?;
            ConstraintBody::Range {
                lower: left,
                lower_op: op,
                middle,
                upper_op: next_op,
                upper,
            }
        } else {
            ConstraintBody::Comparison {
                op,
                left,
                right: middle,
            }
        })
    })
}

fn parse_formula<T>(
    text: &str,
    parse: impl FnOnce(&mut Parser<'_>) -> Result<T, ParseError>,
) -> Result<T, ParseError> {
    let tokens = tokenize(text)?;
    let mut parser = Parser::new(&tokens);
    let constraint = parse(&mut parser)?;
    parser.expect_end()?;
    Ok(constraint)
}

pub fn collect_named_expression_dependencies(expr: &Expr) -> BTreeSet<String> {
    let mut names = BTreeSet::new();
    collect_named_dependencies(expr, &mut BTreeSet::new(), &mut names);
    names
}

pub fn constraint_mentions_previous_time(constraint: &ConstraintBody) -> bool {
    match constraint {
        ConstraintBody::Comparison { left, right, .. } => {
            expr_mentions_previous_time(left) || expr_mentions_previous_time(right)
        }
        ConstraintBody::Range {
            lower,
            middle,
            upper,
            ..
        } => {
            expr_mentions_previous_time(lower)
                || expr_mentions_previous_time(middle)
                || expr_mentions_previous_time(upper)
        }
    }
}

impl Display for Expr {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        format_expr(self, f, 0)
    }
}

impl Display for ConstraintBody {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Comparison { op, left, right } => write!(f, "{left} {op} {right}"),
            Self::Range {
                lower,
                lower_op,
                middle,
                upper_op,
                upper,
            } => write!(f, "{lower} {lower_op} {middle} {upper_op} {upper}"),
        }
    }
}

impl Display for UnaryOp {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Negate => f.write_str("-"),
        }
    }
}

impl Display for BinaryOp {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Add => "+",
            Self::Subtract => "-",
            Self::Multiply => "*",
            Self::Divide => "/",
        })
    }
}

impl Display for ComparisonOp {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Equal => "=",
            Self::DoubleEqual => "==",
            Self::LessEqual => "<=",
            Self::GreaterEqual => ">=",
            Self::Less => "<",
            Self::Greater => ">",
            Self::NotEqual => "!=",
        })
    }
}

fn format_expr(expr: &Expr, f: &mut Formatter<'_>, parent_precedence: u8) -> fmt::Result {
    let precedence = expr_precedence(expr);
    let needs_parens = precedence < parent_precedence;
    if needs_parens {
        f.write_str("(")?;
    }

    match expr {
        Expr::Number(value) | Expr::Identifier(value) => f.write_str(value)?,
        Expr::String(value) => write!(f, "\"{}\"", value.replace('"', "\\\""))?,
        Expr::Boolean(value) => f.write_str(if *value { "true" } else { "false" })?,
        Expr::Indexed { target, indices } => {
            write!(f, "{target}[")?;
            for (index, value) in indices.iter().enumerate() {
                if index > 0 {
                    f.write_str(",")?;
                }
                format_expr(value, f, 0)?;
            }
            f.write_str("]")?;
        }
        Expr::Unary { op, expr } => {
            write!(f, "{op}")?;
            format_expr(expr, f, precedence)?;
        }
        Expr::Binary { op, left, right } => {
            format_expr(left, f, precedence)?;
            write!(f, " {op} ")?;
            format_expr(right, f, precedence + 1)?;
        }
        Expr::Comparison { op, left, right } => {
            format_expr(left, f, precedence)?;
            write!(f, " {op} ")?;
            format_expr(right, f, precedence + 1)?;
        }
        Expr::FunctionCall { name, args } => {
            write!(f, "{name}(")?;
            for (index, arg) in args.iter().enumerate() {
                if index > 0 {
                    f.write_str(", ")?;
                }
                format_expr(arg, f, 0)?;
            }
            f.write_str(")")?;
        }
        Expr::Reduction(reduction) => {
            write!(f, "sum(")?;
            format_expr(&reduction.body, f, 0)?;
            for binding in &reduction.bindings {
                write!(f, " for ")?;
                match &binding.pattern {
                    BindingPattern::Name(name) => f.write_str(name)?,
                    BindingPattern::Tuple(names) => {
                        f.write_str("(")?;
                        for (index, name) in names.iter().enumerate() {
                            if index > 0 {
                                f.write_str(", ")?;
                            }
                            f.write_str(name)?;
                        }
                        f.write_str(")")?;
                    }
                }
                write!(f, " in {}", binding.domain)?;
            }
            for filter in &reduction.filters {
                write!(f, " if ")?;
                format_expr(filter, f, 0)?;
            }
            f.write_str(")")?;
        }
    }

    if needs_parens {
        f.write_str(")")?;
    }
    Ok(())
}

fn expr_precedence(expr: &Expr) -> u8 {
    match expr {
        Expr::Comparison { .. } => 1,
        Expr::Binary {
            op: BinaryOp::Add | BinaryOp::Subtract,
            ..
        } => 2,
        Expr::Binary {
            op: BinaryOp::Multiply | BinaryOp::Divide,
            ..
        } => 3,
        Expr::Unary { .. } => 4,
        Expr::Number(_)
        | Expr::String(_)
        | Expr::Boolean(_)
        | Expr::Identifier(_)
        | Expr::Indexed { .. }
        | Expr::FunctionCall { .. }
        | Expr::Reduction(_) => 5,
    }
}

fn tokenize(text: &str) -> Result<Vec<Token>, ParseError> {
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

fn is_builtin_function(name: &str) -> bool {
    matches!(name, "sqrt" | "pow" | "exp" | "ln" | "abs")
}

struct Parser<'a> {
    tokens: &'a [Token],
    index: usize,
}

impl<'a> Parser<'a> {
    fn new(tokens: &'a [Token]) -> Self {
        Self { tokens, index: 0 }
    }

    fn expect_end(&self) -> Result<(), ParseError> {
        if let Some(token) = self.tokens.get(self.index) {
            Err(ParseError::new(token.position, "unexpected trailing input"))
        } else {
            Ok(())
        }
    }

    fn parse_value_expr(&mut self) -> Result<Expr, ParseError> {
        let left = self.parse_arithmetic_expr()?;
        if let Some(op) = self.maybe_comparison_operator() {
            let right = self.parse_arithmetic_expr()?;
            Ok(Expr::Comparison {
                op,
                left: Box::new(left),
                right: Box::new(right),
            })
        } else {
            Ok(left)
        }
    }

    fn parse_arithmetic_expr(&mut self) -> Result<Expr, ParseError> {
        self.parse_binary_expr(Parser::parse_term_expr, |kind| match kind {
            TokenKind::Plus => Some(BinaryOp::Add),
            TokenKind::Minus => Some(BinaryOp::Subtract),
            _ => None,
        })
    }

    fn parse_term_expr(&mut self) -> Result<Expr, ParseError> {
        self.parse_binary_expr(Parser::parse_unary_expr, |kind| match kind {
            TokenKind::Star => Some(BinaryOp::Multiply),
            TokenKind::Slash => Some(BinaryOp::Divide),
            _ => None,
        })
    }

    fn parse_unary_expr(&mut self) -> Result<Expr, ParseError> {
        if self.matches(|kind| matches!(kind, TokenKind::Minus)) {
            return Ok(Expr::Unary {
                op: UnaryOp::Negate,
                expr: Box::new(self.parse_unary_expr()?),
            });
        }
        self.parse_primary_expr()
    }

    fn parse_primary_expr(&mut self) -> Result<Expr, ParseError> {
        let token = self
            .tokens
            .get(self.index)
            .ok_or_else(|| ParseError::new(0, "unexpected end of input"))?;

        match &token.kind {
            TokenKind::Number(value) => {
                self.index += 1;
                Ok(Expr::Number(value.clone()))
            }
            TokenKind::String(value) => {
                self.index += 1;
                Ok(Expr::String(value.clone()))
            }
            TokenKind::Identifier(name) if name == "sum" => self.parse_sum_expr(),
            TokenKind::Identifier(name) if name == "true" || name == "false" => {
                self.index += 1;
                Ok(Expr::Boolean(name == "true"))
            }
            TokenKind::Identifier(name)
                if is_builtin_function(name)
                    && self
                        .tokens
                        .get(self.index + 1)
                        .is_some_and(|t| matches!(t.kind, TokenKind::LParen)) =>
            {
                let name = name.clone();
                self.index += 2; // consume identifier and LParen
                let mut args = vec![self.parse_value_expr()?];
                while self.matches(|kind| matches!(kind, TokenKind::Comma)) {
                    args.push(self.parse_value_expr()?);
                }
                self.expect_token(
                    |kind| matches!(kind, TokenKind::RParen),
                    "expected `)` to close function call",
                )?;
                Ok(Expr::FunctionCall { name, args })
            }
            TokenKind::Identifier(name) => {
                self.index += 1;
                if self.matches(|kind| matches!(kind, TokenKind::LBracket)) {
                    let mut indices = Vec::new();
                    if !self.matches(|kind| matches!(kind, TokenKind::RBracket)) {
                        loop {
                            indices.push(self.parse_value_expr()?);
                            if self.matches(|kind| matches!(kind, TokenKind::RBracket)) {
                                break;
                            }
                            self.expect_token(
                                |kind| matches!(kind, TokenKind::Comma),
                                "expected `,` or `]` in index list",
                            )?;
                        }
                    }
                    Ok(Expr::Indexed {
                        target: name.clone(),
                        indices,
                    })
                } else {
                    Ok(Expr::Identifier(name.clone()))
                }
            }
            TokenKind::LParen => {
                self.index += 1;
                let expression = self.parse_value_expr()?;
                self.expect_token(|kind| matches!(kind, TokenKind::RParen), "expected `)`")?;
                Ok(expression)
            }
            _ => Err(ParseError::new(
                token.position,
                "expected a value expression",
            )),
        }
    }

    fn parse_sum_expr(&mut self) -> Result<Expr, ParseError> {
        self.index += 1;
        self.expect_token(
            |kind| matches!(kind, TokenKind::LParen),
            "expected `(` after `sum`",
        )?;
        let body = self.parse_value_expr()?;

        let mut bindings = Vec::new();
        while self.matches(|kind| matches!(kind, TokenKind::KeywordFor)) {
            let pattern = if self.matches(|kind| matches!(kind, TokenKind::LParen)) {
                let mut members = Vec::new();
                loop {
                    members.push(self.parse_identifier_name()?);
                    if self.matches(|kind| matches!(kind, TokenKind::RParen)) {
                        break;
                    }
                    self.expect_token(
                        |kind| matches!(kind, TokenKind::Comma),
                        "expected `,` or `)` in tuple binding",
                    )?;
                }
                BindingPattern::Tuple(members)
            } else {
                BindingPattern::Name(self.parse_identifier_name()?)
            };
            self.expect_token(
                |kind| matches!(kind, TokenKind::KeywordIn),
                "expected `in` in reduction binding",
            )?;
            bindings.push(Binding {
                pattern,
                domain: self.parse_identifier_name()?,
            });
        }

        if bindings.is_empty() {
            let position = self
                .tokens
                .get(self.index)
                .map_or(0, |token| token.position);
            return Err(ParseError::new(
                position,
                "expected at least one `for` binding in reduction",
            ));
        }

        let mut filters = Vec::new();
        while self.matches(|kind| matches!(kind, TokenKind::KeywordIf)) {
            filters.push(self.parse_value_expr()?);
        }

        self.expect_token(
            |kind| matches!(kind, TokenKind::RParen),
            "expected `)` to close reduction",
        )?;
        Ok(Expr::Reduction(ReductionExpr {
            op: ReductionOp::Sum,
            body: Box::new(body),
            bindings,
            filters,
        }))
    }

    fn parse_identifier_name(&mut self) -> Result<String, ParseError> {
        let token = self
            .tokens
            .get(self.index)
            .ok_or_else(|| ParseError::new(0, "unexpected end of input"))?;
        if let TokenKind::Identifier(name) = &token.kind {
            self.index += 1;
            Ok(name.clone())
        } else {
            Err(ParseError::new(token.position, "expected an identifier"))
        }
    }

    fn parse_comparison_operator(&mut self) -> Result<ComparisonOp, ParseError> {
        self.maybe_comparison_operator().ok_or_else(|| {
            ParseError::new(self.current_position(), "expected a comparison operator")
        })
    }

    fn maybe_comparison_operator(&mut self) -> Option<ComparisonOp> {
        let token = self.tokens.get(self.index)?;
        let op = match token.kind {
            TokenKind::Equal => ComparisonOp::Equal,
            TokenKind::DoubleEqual => ComparisonOp::DoubleEqual,
            TokenKind::NotEqual => ComparisonOp::NotEqual,
            TokenKind::Less => ComparisonOp::Less,
            TokenKind::LessEqual => ComparisonOp::LessEqual,
            TokenKind::Greater => ComparisonOp::Greater,
            TokenKind::GreaterEqual => ComparisonOp::GreaterEqual,
            _ => return None,
        };
        self.index += 1;
        Some(op)
    }

    fn matches(&mut self, predicate: impl FnOnce(&TokenKind) -> bool) -> bool {
        if let Some(token) = self.tokens.get(self.index) {
            if predicate(&token.kind) {
                self.index += 1;
                return true;
            }
        }
        false
    }

    fn expect_token(
        &mut self,
        predicate: impl FnOnce(&TokenKind) -> bool,
        message: &'static str,
    ) -> Result<(), ParseError> {
        let token = self
            .tokens
            .get(self.index)
            .ok_or_else(|| ParseError::new(0, message))?;
        if predicate(&token.kind) {
            self.index += 1;
            Ok(())
        } else {
            Err(ParseError::new(token.position, message))
        }
    }

    fn parse_binary_expr(
        &mut self,
        parse_operand: impl Fn(&mut Self) -> Result<Expr, ParseError>,
        operator_for: impl Fn(&TokenKind) -> Option<BinaryOp>,
    ) -> Result<Expr, ParseError> {
        let mut expression = parse_operand(self)?;
        while let Some(token) = self.tokens.get(self.index) {
            let Some(op) = operator_for(&token.kind) else {
                break;
            };
            self.index += 1;
            let right = parse_operand(self)?;
            expression = Expr::Binary {
                op,
                left: Box::new(expression),
                right: Box::new(right),
            };
        }
        Ok(expression)
    }

    fn current_position(&self) -> usize {
        self.tokens
            .get(self.index)
            .map_or(0, |token| token.position)
    }
}

fn is_identifier_start(character: char) -> bool {
    character.is_ascii_alphabetic() || character == '_'
}

fn is_identifier_continue(character: char) -> bool {
    character.is_ascii_alphanumeric() || character == '_'
}

fn collect_named_dependencies(
    expr: &Expr,
    bound: &mut BTreeSet<String>,
    names: &mut BTreeSet<String>,
) {
    match expr {
        Expr::Identifier(name) => {
            if !bound.contains(name) {
                names.insert(name.clone());
            }
        }
        Expr::Indexed { indices, .. } => {
            for index in indices {
                collect_named_dependencies(index, bound, names);
            }
        }
        Expr::Unary { expr, .. } => collect_named_dependencies(expr, bound, names),
        Expr::Binary { left, right, .. } | Expr::Comparison { left, right, .. } => {
            collect_named_dependencies(left, bound, names);
            collect_named_dependencies(right, bound, names);
        }
        Expr::Reduction(reduction) => {
            let mut local_bound = bound.clone();
            for binding in &reduction.bindings {
                match &binding.pattern {
                    BindingPattern::Name(name) => {
                        local_bound.insert(name.clone());
                    }
                    BindingPattern::Tuple(names) => {
                        local_bound.extend(names.iter().cloned());
                    }
                }
            }
            collect_named_dependencies(&reduction.body, &mut local_bound, names);
            for filter in &reduction.filters {
                collect_named_dependencies(filter, &mut local_bound, names);
            }
        }
        Expr::FunctionCall { args, .. } => {
            for arg in args {
                collect_named_dependencies(arg, bound, names);
            }
        }
        Expr::Number(_) | Expr::String(_) | Expr::Boolean(_) => {}
    }
}

fn expr_mentions_previous_time(expr: &Expr) -> bool {
    match expr {
        Expr::Indexed { indices, .. } => indices.iter().any(index_mentions_previous_time),
        Expr::Unary { expr, .. } => expr_mentions_previous_time(expr),
        Expr::Binary { left, right, .. } | Expr::Comparison { left, right, .. } => {
            expr_mentions_previous_time(left) || expr_mentions_previous_time(right)
        }
        Expr::FunctionCall { args, .. } => args.iter().any(expr_mentions_previous_time),
        Expr::Reduction(reduction) => {
            expr_mentions_previous_time(&reduction.body)
                || reduction.filters.iter().any(expr_mentions_previous_time)
        }
        Expr::Number(_) | Expr::String(_) | Expr::Boolean(_) | Expr::Identifier(_) => false,
    }
}

fn index_mentions_previous_time(expr: &Expr) -> bool {
    matches!(
        expr,
        Expr::Binary {
            op: BinaryOp::Subtract,
            left,
            right,
        } if matches!(left.as_ref(), Expr::Identifier(name) if name == "t")
            && matches!(right.as_ref(), Expr::Number(value) if value == "1")
    ) || expr_mentions_previous_time(expr)
}
