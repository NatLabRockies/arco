use kdl::{KdlDocument, KdlEntry, KdlNode, KdlValue};

pub(super) fn normalize_surface_syntax(text: &str) -> String {
    let mut normalized = String::with_capacity(text.len());
    let bytes = text.as_bytes();
    let mut index = 0;
    while index < bytes.len() {
        let rewritten = rewrite_math_block_at(text, index);
        if let Some((replacement, end)) = rewritten {
            normalized.push_str(&replacement);
            index = end;
            continue;
        }
        normalized.push(bytes[index] as char);
        index += 1;
    }
    normalized
}

pub(super) fn format_surface_document(document: &KdlDocument) -> String {
    let mut rendered = String::new();
    for node in document.nodes() {
        render_node(node, 0, &mut rendered);
    }
    rendered
}

fn render_node(node: &KdlNode, indent: usize, output: &mut String) {
    if let Some(expression) =
        expression_property(node).filter(|_| promotes_expression_property(node.name().value()))
    {
        let algebra = if node.children().is_some() {
            AlgebraBlock::Named("expression", expression)
        } else {
            AlgebraBlock::Bare(expression)
        };
        render_block_node(
            node,
            indent,
            Some("expression"),
            None,
            Some(algebra),
            output,
        );
        return;
    }

    if node.name().value() == "expression" {
        if let Some((formula_index, formula)) = formula_child(node) {
            let algebra = if node
                .children()
                .is_some_and(|children| children.nodes().len() == 1)
            {
                AlgebraBlock::Bare(formula)
            } else {
                AlgebraBlock::Named("expression", formula)
            };
            render_block_node(
                node,
                indent,
                None,
                Some(formula_index),
                Some(algebra),
                output,
            );
            return;
        }
    }

    render_block_node(node, indent, None, None, None, output);
}

#[derive(Clone, Copy)]
enum AlgebraBlock<'a> {
    Bare(&'a str),
    Named(&'static str, &'a str),
}

fn render_block_node(
    node: &KdlNode,
    indent: usize,
    excluded_property: Option<&str>,
    skipped_child: Option<usize>,
    algebra: Option<AlgebraBlock<'_>>,
    output: &mut String,
) {
    render_header(node, indent, excluded_property, output);
    if node.children().is_none() && algebra.is_none() {
        output.push('\n');
        return;
    }
    output.push_str(" {\n");

    if let Some(children) = node.children() {
        for (index, child) in children.nodes().iter().enumerate() {
            if Some(index) != skipped_child {
                render_node(child, indent + 2, output);
            }
        }
    }
    if let Some(algebra) = algebra {
        render_algebra_block(algebra, indent + 2, output);
    }

    push_indent(indent, output);
    output.push_str("}\n");
}

fn render_algebra_block(algebra: AlgebraBlock<'_>, indent: usize, output: &mut String) {
    match algebra {
        AlgebraBlock::Bare(expression) => render_algebra(expression, indent, output),
        AlgebraBlock::Named(name, expression) => {
            push_indent(indent, output);
            output.push_str(name);
            output.push_str(" {\n");
            render_algebra(expression, indent + 2, output);
            push_indent(indent, output);
            output.push_str("}\n");
        }
    }
}

fn render_header(
    node: &KdlNode,
    indent: usize,
    excluded_property: Option<&str>,
    output: &mut String,
) {
    render_leading_comments(node, indent, output);
    push_indent(indent, output);
    output.push_str(node.name().value());
    for entry in node.entries() {
        if excluded_property.is_some() && entry.name().map(|name| name.value()) == excluded_property
        {
            continue;
        }
        output.push(' ');
        output.push_str(&render_entry(entry));
    }
}

fn render_leading_comments(node: &KdlNode, indent: usize, output: &mut String) {
    let Some(format) = node.format() else {
        return;
    };

    let mut rendered_comment = false;
    let mut pending_blank_line = false;
    for comment in leading_comments(&format.leading) {
        match comment {
            LeadingComment::BlankLine if rendered_comment => {
                pending_blank_line = true;
            }
            LeadingComment::BlankLine => {}
            LeadingComment::Comment(span) => {
                if pending_blank_line {
                    output.push('\n');
                    pending_blank_line = false;
                }
                render_comment_span(span, indent, output);
                rendered_comment = true;
            }
        }
    }
}

enum LeadingComment<'a> {
    BlankLine,
    Comment(&'a str),
}

fn leading_comments(text: &str) -> Vec<LeadingComment<'_>> {
    let mut comments = Vec::new();
    let mut index = 0usize;
    let mut newlines_since_comment = 0usize;

    while index < text.len() {
        let Some(remaining) = text.get(index..) else {
            break;
        };

        if remaining.starts_with("//") {
            if newlines_since_comment > 1 {
                comments.push(LeadingComment::BlankLine);
            }
            let end = line_comment_end(text, index);
            if let Some(span) = text.get(index..end) {
                comments.push(LeadingComment::Comment(span));
            }
            index = end;
            newlines_since_comment = 0;
            continue;
        }

        if remaining.starts_with("/*") {
            if newlines_since_comment > 1 {
                comments.push(LeadingComment::BlankLine);
            }
            let end = block_comment_end(text, index);
            if let Some(span) = text.get(index..end) {
                comments.push(LeadingComment::Comment(span));
            }
            index = end;
            newlines_since_comment = 0;
            continue;
        }

        let Some(character) = remaining.chars().next() else {
            break;
        };
        if character == '\n' {
            newlines_since_comment += 1;
        } else if !character.is_whitespace() {
            newlines_since_comment = 0;
        }
        index += character.len_utf8();
    }

    comments
}

fn line_comment_end(text: &str, start: usize) -> usize {
    text.get(start..)
        .and_then(|remaining| remaining.find('\n'))
        .map_or(text.len(), |offset| start + offset)
}

fn block_comment_end(text: &str, start: usize) -> usize {
    let search_start = start + "/*".len();
    text.get(search_start..)
        .and_then(|remaining| remaining.find("*/"))
        .map_or(text.len(), |offset| search_start + offset + "*/".len())
}

fn render_comment_span(span: &str, indent: usize, output: &mut String) {
    let lines = span.lines().collect::<Vec<_>>();
    let common_indent = common_following_line_indent(&lines);
    for (index, line) in lines.into_iter().enumerate() {
        push_indent(indent, output);
        let rendered = if index == 0 {
            line
        } else {
            strip_leading_whitespace(line, common_indent)
        };
        output.push_str(rendered.trim_end());
        output.push('\n');
    }
}

fn common_following_line_indent(lines: &[&str]) -> usize {
    lines
        .iter()
        .skip(1)
        .filter(|line| !line.trim().is_empty())
        .map(|line| leading_whitespace_width(line))
        .min()
        .unwrap_or(0)
}

fn leading_whitespace_width(line: &str) -> usize {
    line.chars()
        .take_while(|character| character.is_whitespace() && *character != '\n')
        .map(char::len_utf8)
        .sum()
}

fn strip_leading_whitespace(line: &str, width: usize) -> &str {
    let mut remaining = width;
    for (index, character) in line.char_indices() {
        if remaining == 0 || !character.is_whitespace() || character == '\n' {
            return line.get(index..).unwrap_or("");
        }
        remaining = remaining.saturating_sub(character.len_utf8());
    }
    if remaining == 0 { line } else { "" }
}

fn render_entry(entry: &KdlEntry) -> String {
    let mut rendered = String::new();
    if let Some(name) = entry.name() {
        rendered.push_str(name.value());
        rendered.push('=');
    }
    if let Some(ty) = entry.ty() {
        rendered.push('(');
        rendered.push_str(ty.value());
        rendered.push(')');
    }
    let value = entry.value().to_string();
    if value.is_empty() {
        if let Some(format) = entry.format() {
            rendered.push_str(format.value_repr.trim());
        }
    } else {
        rendered.push_str(&value);
    }
    rendered
}

fn render_algebra(expression: &str, indent: usize, output: &mut String) {
    for line in format_algebra_lines(expression) {
        push_indent(indent, output);
        output.push_str(&line);
        output.push('\n');
    }
}

fn format_algebra_lines(expression: &str) -> Vec<String> {
    let normalized = expression.split_whitespace().collect::<Vec<_>>().join(" ");
    let splits = split_top_level_operators(&normalized);
    if splits.len() <= 1 {
        return vec![normalized];
    }

    splits
        .into_iter()
        .enumerate()
        .map(|(index, line)| {
            if index == 0 {
                line
            } else {
                format!("  {line}")
            }
        })
        .collect()
}

fn split_top_level_operators(expression: &str) -> Vec<String> {
    let mut lines = Vec::new();
    let mut segment_start = 0usize;
    let mut pending_operator: Option<&str> = None;
    let mut state = OperatorScanState::default();
    let mut skip_until = 0usize;

    for (index, character) in expression.char_indices() {
        if index < skip_until {
            continue;
        }
        if state.advance(character) {
            continue;
        }
        if state.is_nested() {
            continue;
        }

        let operator = top_level_operator_at(expression, index);
        let Some(operator) = operator else {
            continue;
        };

        let segment = expression[segment_start..index].trim();
        if !segment.is_empty() {
            lines.push(format_segment(pending_operator, segment));
        }
        pending_operator = Some(operator);
        segment_start = index + operator.len();
        skip_until = segment_start;
    }

    let segment = expression[segment_start..].trim();
    if !segment.is_empty() {
        lines.push(format_segment(pending_operator, segment));
    }

    lines
}

fn top_level_operator_at(expression: &str, index: usize) -> Option<&'static str> {
    let remaining = expression.get(index..)?;
    for operator in ["<=", ">=", "==", "="] {
        if remaining.starts_with(operator) {
            return Some(operator);
        }
    }

    let operator = remaining.chars().next()?;
    if matches!(operator, '+' | '-') && has_space_around(expression, index) {
        return Some(if operator == '+' { "+" } else { "-" });
    }

    None
}

fn has_space_around(expression: &str, index: usize) -> bool {
    let Some(previous) = expression[..index].chars().next_back() else {
        return false;
    };
    let Some(next) = expression[index + 1..].chars().next() else {
        return false;
    };
    previous.is_ascii_whitespace() && next.is_ascii_whitespace()
}

fn format_segment(operator: Option<&str>, segment: &str) -> String {
    if let Some(operator) = operator {
        format!("{operator} {segment}")
    } else {
        segment.to_string()
    }
}

#[derive(Default)]
struct OperatorScanState {
    nesting_depth: usize,
    in_string: bool,
    escaped: bool,
}

impl OperatorScanState {
    fn advance(&mut self, character: char) -> bool {
        if self.in_string {
            if self.escaped {
                self.escaped = false;
            } else if character == '\\' {
                self.escaped = true;
            } else if character == '"' {
                self.in_string = false;
            }
            return true;
        }

        match character {
            '"' => {
                self.in_string = true;
                true
            }
            '(' | '[' | '{' => {
                self.nesting_depth += 1;
                true
            }
            ')' | ']' | '}' => {
                self.nesting_depth = self.nesting_depth.saturating_sub(1);
                true
            }
            _ => false,
        }
    }

    fn is_nested(&self) -> bool {
        self.nesting_depth > 0
    }
}

fn expression_property(node: &KdlNode) -> Option<&str> {
    node.get("expression").and_then(KdlValue::as_string)
}

fn formula_child(node: &KdlNode) -> Option<(usize, &str)> {
    node.children()?
        .nodes()
        .iter()
        .enumerate()
        .find_map(|(index, child)| {
            if child.name().value() == "formula" {
                child
                    .get(0)
                    .and_then(KdlValue::as_string)
                    .map(|value| (index, value))
            } else {
                None
            }
        })
}

fn promotes_expression_property(name: &str) -> bool {
    matches!(
        name,
        "constraint" | "expression" | "filter" | "if" | "lower" | "maximize" | "minimize" | "upper"
    )
}

fn push_indent(indent: usize, output: &mut String) {
    for _ in 0..indent {
        output.push(' ');
    }
}

fn rewrite_math_block_at(text: &str, start: usize) -> Option<(String, usize)> {
    let byte = *text.as_bytes().get(start)?;
    match byte {
        b'c' => rewrite_math_block(text, start, "constraint"),
        b'e' => rewrite_math_block(text, start, "expression"),
        b'f' => rewrite_math_block(text, start, "filter"),
        b'i' => rewrite_math_block(text, start, "if"),
        b'l' => rewrite_math_block(text, start, "lower"),
        b'm' => rewrite_math_block(text, start, "minimize")
            .or_else(|| rewrite_math_block(text, start, "maximize")),
        b'u' => rewrite_math_block(text, start, "upper"),
        _ => None,
    }
}

fn rewrite_math_block(text: &str, start: usize, keyword: &str) -> Option<(String, usize)> {
    if !matches_keyword_at(text, start, keyword) {
        return None;
    }

    let opening_brace = find_opening_brace(text, start + keyword.len())?;

    if matches!(keyword, "constraint" | "expression")
        && body_starts_with_generation_keyword(text, opening_brace)
    {
        return None;
    }

    let closing_index = find_matching_brace(text.as_bytes(), opening_brace)?;

    let header = text[start..opening_brace].trim_end();
    let body = normalize_math_body(&text[opening_brace + 1..closing_index]);
    let encoded_body = encode_kdl_string(&body);
    let replacement = rewrite_math_replacement(keyword, header, &encoded_body)?;

    Some((replacement, closing_index + 1))
}

fn find_opening_brace(text: &str, mut index: usize) -> Option<usize> {
    let bytes = text.as_bytes();
    let mut in_string = false;
    let mut escaped = false;

    while index < bytes.len() {
        let byte = bytes[index];
        if in_string {
            if escaped {
                escaped = false;
            } else if byte == b'\\' {
                escaped = true;
            } else if byte == b'"' {
                in_string = false;
            }
            index += 1;
            continue;
        }

        match byte {
            b'"' => in_string = true,
            b'{' => return Some(index),
            b'\n' => return None,
            _ => {}
        }
        index += 1;
    }

    None
}

fn find_matching_brace(bytes: &[u8], opening_brace: usize) -> Option<usize> {
    let mut index = opening_brace + 1;
    let mut brace_depth = 1usize;
    let mut in_string = false;
    let mut escaped = false;

    while index < bytes.len() {
        let byte = bytes[index];
        if in_string {
            if escaped {
                escaped = false;
            } else if byte == b'\\' {
                escaped = true;
            } else if byte == b'"' {
                in_string = false;
            }
            index += 1;
            continue;
        }

        match byte {
            b'"' => in_string = true,
            b'{' => brace_depth += 1,
            b'}' => {
                brace_depth -= 1;
                if brace_depth == 0 {
                    return Some(index);
                }
            }
            _ => {}
        }
        index += 1;
    }

    None
}

fn rewrite_math_replacement(keyword: &str, header: &str, encoded_body: &str) -> Option<String> {
    match keyword {
        "constraint" => Some(format!("{header} expression={encoded_body}")),
        "expression" => Some(format!("{header} {{ formula {encoded_body} }}")),
        "minimize" | "maximize" | "lower" | "upper" | "if" | "filter" => {
            Some(format!("{header} expression={encoded_body}"))
        }
        _ => None,
    }
}

fn body_starts_with_generation_keyword(text: &str, opening_brace: usize) -> bool {
    let trimmed = text[opening_brace + 1..].trim_start();
    for keyword in ["index", "if", "slack", "expression", "formula"] {
        let Some(rest) = trimmed.strip_prefix(keyword) else {
            continue;
        };
        if rest.starts_with([' ', '\t', '{']) {
            return true;
        }
    }
    false
}

fn matches_keyword_at(text: &str, start: usize, keyword: &str) -> bool {
    let bytes = text.as_bytes();
    let end = start + keyword.len();

    if end > bytes.len() || &bytes[start..end] != keyword.as_bytes() {
        return false;
    }

    let previous_ok = start == 0 || is_keyword_boundary(bytes[start - 1] as char);
    let next_ok = end >= bytes.len() || is_keyword_boundary(bytes[end] as char);
    previous_ok && next_ok
}

fn is_keyword_boundary(character: char) -> bool {
    character.is_ascii_whitespace() || matches!(character, '{' | '}')
}

fn normalize_math_body(body: &str) -> String {
    body.lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join(" ")
}

fn encode_kdl_string(value: &str) -> String {
    let mut encoded = String::with_capacity(value.len() + 2);
    encoded.push('"');
    for character in value.chars() {
        match character {
            '\\' => encoded.push_str("\\\\"),
            '"' => encoded.push_str("\\\""),
            '\n' => encoded.push_str("\\n"),
            '\r' => encoded.push_str("\\r"),
            '\t' => encoded.push_str("\\t"),
            _ => encoded.push(character),
        }
    }
    encoded.push('"');
    encoded
}

#[cfg(test)]
mod tests {
    use super::{format_algebra_lines, normalize_surface_syntax};

    #[test]
    fn rewrites_inline_math_bodies_to_expression_properties() {
        let input = r#"
constraint "c1" {
  x[a] <= 1
}
"#;
        let normalized = normalize_surface_syntax(input);
        assert!(normalized.contains("constraint \"c1\" expression=\"x[a] <= 1\""));
    }

    #[test]
    fn preserves_generated_constraint_blocks() {
        let input = r#"
constraint "ramp" {
  index g
  expression {
    p[g] <= 1
  }
}
"#;
        let normalized = normalize_surface_syntax(input);
        assert!(normalized.contains("index g"));
        assert!(normalized.contains("expression {"));
    }

    #[test]
    fn splits_top_level_algebra_operators() {
        let lines = format_algebra_lines(
            "sum(pg[g] for g in generators if connected[b,g] > 0) - pd[b] / 100 = sum(incidence[l,b] * flow[l] for l in lines)",
        );
        assert_eq!(
            lines,
            vec![
                "sum(pg[g] for g in generators if connected[b,g] > 0)",
                "  - pd[b] / 100",
                "  = sum(incidence[l,b] * flow[l] for l in lines)",
            ]
        );
    }
}
