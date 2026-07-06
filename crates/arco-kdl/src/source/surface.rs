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
    if promotes_expression_property(node.name().value()) {
        if let Some(expression) = expression_property(node) {
            render_expression_property_node(node, expression, indent, output);
            return;
        }
    }

    if node.name().value() == "expression" {
        if let Some((formula_index, formula)) = formula_child(node) {
            render_expression_formula_node(node, formula_index, formula, indent, output);
            return;
        }
    }

    render_generic_node(node, indent, output);
}

fn render_expression_property_node(
    node: &KdlNode,
    expression: &str,
    indent: usize,
    output: &mut String,
) {
    render_header(node, indent, Some("expression"), output);
    output.push_str(" {\n");

    if let Some(children) = node.children() {
        for child in children.nodes() {
            render_node(child, indent + 2, output);
        }
        render_named_algebra_block("expression", expression, indent + 2, output);
    } else {
        render_algebra(expression, indent + 2, output);
    }

    push_indent(indent, output);
    output.push_str("}\n");
}

fn render_expression_formula_node(
    node: &KdlNode,
    formula_index: usize,
    formula: &str,
    indent: usize,
    output: &mut String,
) {
    render_header(node, indent, None, output);
    output.push_str(" {\n");

    let Some(children) = node.children() else {
        render_algebra(formula, indent + 2, output);
        push_indent(indent, output);
        output.push_str("}\n");
        return;
    };

    if children.nodes().len() == 1 {
        render_algebra(formula, indent + 2, output);
    } else {
        for (index, child) in children.nodes().iter().enumerate() {
            if index != formula_index {
                render_node(child, indent + 2, output);
            }
        }
        render_named_algebra_block("expression", formula, indent + 2, output);
    }

    push_indent(indent, output);
    output.push_str("}\n");
}

fn render_named_algebra_block(name: &str, expression: &str, indent: usize, output: &mut String) {
    push_indent(indent, output);
    output.push_str(name);
    output.push_str(" {\n");
    render_algebra(expression, indent + 2, output);
    push_indent(indent, output);
    output.push_str("}\n");
}

fn render_generic_node(node: &KdlNode, indent: usize, output: &mut String) {
    render_header(node, indent, None, output);
    if let Some(children) = node.children() {
        output.push_str(" {\n");
        for child in children.nodes() {
            render_node(child, indent + 2, output);
        }
        push_indent(indent, output);
        output.push_str("}\n");
    } else {
        output.push('\n');
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

    let mut saw_comment = false;
    for line in format.leading.lines() {
        let trimmed = line.trim();
        if is_comment_line(trimmed) {
            push_indent(indent, output);
            output.push_str(trimmed);
            output.push('\n');
            saw_comment = true;
        } else if saw_comment && trimmed.is_empty() {
            output.push('\n');
        }
    }
}

fn is_comment_line(line: &str) -> bool {
    line.starts_with("//")
        || line.starts_with("/*")
        || line.starts_with('*')
        || line.starts_with("*/")
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
    if remaining.starts_with("<=") {
        return Some("<=");
    }
    if remaining.starts_with(">=") {
        return Some(">=");
    }
    if remaining.starts_with("==") {
        return Some("==");
    }
    if remaining.starts_with('=') {
        return Some("=");
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
    paren_depth: usize,
    bracket_depth: usize,
    brace_depth: usize,
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
            '(' => {
                self.paren_depth += 1;
                true
            }
            ')' => {
                self.paren_depth = self.paren_depth.saturating_sub(1);
                true
            }
            '[' => {
                self.bracket_depth += 1;
                true
            }
            ']' => {
                self.bracket_depth = self.bracket_depth.saturating_sub(1);
                true
            }
            '{' => {
                self.brace_depth += 1;
                true
            }
            '}' => {
                self.brace_depth = self.brace_depth.saturating_sub(1);
                true
            }
            _ => false,
        }
    }

    fn is_nested(&self) -> bool {
        self.paren_depth > 0 || self.bracket_depth > 0 || self.brace_depth > 0
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
