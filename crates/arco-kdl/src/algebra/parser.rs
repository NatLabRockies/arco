use crate::algebra::error::ParseError;
use crate::algebra::tokenizer::{Token, TokenKind, is_builtin_function, tokenize};
use crate::algebra::types::{
    BinaryOp, Binding, BindingPattern, ComparisonOp, ConstraintBody, Expr, ReductionExpr,
    ReductionOp, UnaryOp,
};
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
                self.index += 2;
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
                domain: self.parse_domain_reference()?,
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

    fn parse_domain_reference(&mut self) -> Result<String, ParseError> {
        let base = self.parse_identifier_name()?;
        if !self.matches(|kind| matches!(kind, TokenKind::LBracket)) {
            return Ok(base);
        }

        let mut selectors = Vec::new();
        loop {
            let key = self.parse_identifier_name()?;
            self.expect_token(
                |kind| matches!(kind, TokenKind::Equal),
                "expected `=` in inline selector",
            )?;
            let value = self.parse_selector_value()?;
            selectors.push(format!("{key}={value}"));

            if self.matches(|kind| matches!(kind, TokenKind::RBracket)) {
                break;
            }

            if self.matches(|kind| matches!(kind, TokenKind::Comma)) {
                continue;
            }

            let Some(token) = self.tokens.get(self.index) else {
                return Err(ParseError::new(0, "expected `]` to close inline selector"));
            };
            if !matches!(token.kind, TokenKind::Identifier(_)) {
                return Err(ParseError::new(
                    token.position,
                    "expected another `key=value` selector pair or `]`",
                ));
            }
        }

        Ok(format!("{base}[{}]", selectors.join(" ")))
    }

    fn parse_selector_value(&mut self) -> Result<String, ParseError> {
        let token = self
            .tokens
            .get(self.index)
            .ok_or_else(|| ParseError::new(0, "unexpected end of input"))?;
        let value = match &token.kind {
            TokenKind::Identifier(value) => value.clone(),
            TokenKind::Number(value) => value.clone(),
            TokenKind::String(value) => {
                let escaped = value.replace('"', "\\\"");
                format!("\"{escaped}\"")
            }
            _ => {
                return Err(ParseError::new(token.position, "expected selector value"));
            }
        };
        self.index += 1;
        Ok(value)
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
