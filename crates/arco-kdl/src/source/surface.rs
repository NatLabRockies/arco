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

    let bytes = text.as_bytes();
    let mut index = start + keyword.len();
    let mut in_string = false;
    let mut escaped = false;
    let mut opening_brace = None;

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
            b'{' => {
                opening_brace = Some(index);
                break;
            }
            b'\n' => return None,
            _ => {}
        }
        index += 1;
    }

    let opening_brace = opening_brace?;

    if keyword == "constraint" && body_starts_with_generation_keyword(text, opening_brace) {
        return None;
    }

    let mut closing_index = opening_brace + 1;
    let mut brace_depth = 1usize;
    in_string = false;
    escaped = false;

    while closing_index < bytes.len() {
        let byte = bytes[closing_index];
        if in_string {
            if escaped {
                escaped = false;
            } else if byte == b'\\' {
                escaped = true;
            } else if byte == b'"' {
                in_string = false;
            }
            closing_index += 1;
            continue;
        }

        match byte {
            b'"' => in_string = true,
            b'{' => brace_depth += 1,
            b'}' => {
                brace_depth -= 1;
                if brace_depth == 0 {
                    break;
                }
            }
            _ => {}
        }
        closing_index += 1;
    }

    if brace_depth != 0 {
        return None;
    }

    let header = text[start..opening_brace].trim_end();
    let body = normalize_math_body(&text[opening_brace + 1..closing_index]);
    let encoded_body = encode_kdl_string(&body);

    let replacement = match keyword {
        "constraint" => format!("{header} expression={encoded_body}"),
        "expression" => format!("{header} {{ formula {encoded_body} }}"),
        "minimize" | "maximize" => format!("{header} expression={encoded_body}"),
        "lower" | "upper" | "if" | "filter" => {
            format!("{header} expression={encoded_body}")
        }
        _ => return None,
    };

    Some((replacement, closing_index + 1))
}

fn body_starts_with_generation_keyword(text: &str, opening_brace: usize) -> bool {
    let trimmed = text[opening_brace + 1..].trim_start();
    for keyword in ["index", "if", "slack", "expression"] {
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
    use super::normalize_surface_syntax;

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
}
