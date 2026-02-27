use crate::tokenizer::Token;

fn is_clause_keyword(kw: &str) -> bool {
    matches!(
        kw,
        "SELECT" | "FROM" | "WHERE" | "HAVING" | "LIMIT" | "SET" | "VALUES"
            | "UNION" | "EXCEPT" | "INTERSECT" | "JOIN" | "ON"
    )
}

fn is_compound_first(kw: &str) -> bool {
    matches!(kw, "GROUP" | "ORDER" | "INSERT" | "DELETE")
}

fn is_compound_second(kw: &str) -> bool {
    matches!(kw, "BY" | "INTO" | "FROM")
}

fn is_statement_starter(kw: &str) -> bool {
    matches!(kw, "SELECT" | "INSERT" | "UPDATE" | "DELETE" | "CREATE" | "DROP" | "ALTER" | "WITH")
}

fn is_join_modifier(kw: &str) -> bool {
    matches!(kw, "LEFT" | "RIGHT" | "INNER" | "OUTER" | "CROSS" | "FULL" | "NATURAL")
}

fn indent_str(level: usize) -> String {
    "  ".repeat(level)
}

fn filter_tokens(tokens: &[Token]) -> Vec<&Token> {
    tokens.iter().filter(|t| !matches!(t, Token::Whitespace(_))).collect()
}

fn next_significant_token(tokens: &[&Token], from: usize) -> Option<usize> {
    for j in (from + 1)..tokens.len() {
        if !matches!(tokens[j], Token::Comment(_)) {
            return Some(j);
        }
    }
    None
}

fn paren_contains_subquery(tokens: &[&Token], start: usize) -> bool {
    if let Some(j) = next_significant_token(tokens, start) {
        if let Token::Keyword(kw) = tokens[j] {
            return is_statement_starter(&kw.to_uppercase());
        }
    }
    false
}

pub fn beautify(tokens: &[Token]) -> String {
    let filtered = filter_tokens(tokens);
    if filtered.is_empty() {
        return String::new();
    }

    let mut out = String::new();
    let mut base_indent: usize = 0;
    let mut line_started = false;
    let mut in_clause_content = false;
    let mut need_blank_line = false;
    let mut i = 0;

    // Paren stack: (saved_base_indent, saved_in_clause_content, is_subquery)
    let mut paren_stack: Vec<(usize, bool, bool)> = Vec::new();
    // Track inline paren depth (non-subquery parens where commas don't cause newlines)
    let mut inline_paren_depth: usize = 0;

    // Track what the last emitted non-whitespace token type was for space decisions
    let mut last_was_keyword = false;

    while i < filtered.len() {
        let token = filtered[i];

        match token {
            Token::Comment(c) => {
                if line_started {
                    out.push(' ');
                } else {
                    if need_blank_line {
                        out.push('\n');
                        need_blank_line = false;
                    }
                    out.push_str(&indent_str(base_indent));
                }
                out.push_str(c);
                out.push('\n');
                line_started = false;
                last_was_keyword = false;
                i += 1;
            }
            Token::Keyword(kw) => {
                let upper = kw.to_uppercase();

                // Inside inline parens, keywords are just inline
                if inline_paren_depth > 0 {
                    if line_started {
                        out.push(' ');
                    }
                    out.push_str(&upper);
                    line_started = true;
                    last_was_keyword = true;
                    i += 1;
                    continue;
                }

                // Check for join modifier + JOIN compound
                if is_join_modifier(&upper) {
                    if let Some(j) = next_significant_token(&filtered, i) {
                        if let Token::Keyword(next_kw) = filtered[j] {
                            if next_kw.to_uppercase() == "JOIN" {
                                emit_clause_line(&mut out, &mut line_started, &mut need_blank_line, &mut in_clause_content, base_indent, &format!("{} {}", upper, next_kw.to_uppercase()));
                                in_clause_content = true;
                                last_was_keyword = true;
                                i = j + 1;
                                continue;
                            }
                        }
                    }
                    // Not followed by JOIN, treat as regular keyword
                    emit_inline_keyword(&mut out, &mut line_started, &mut last_was_keyword, base_indent, in_clause_content, &upper);
                    i += 1;
                    continue;
                }

                // Check for compound keywords: GROUP BY, ORDER BY, INSERT INTO, DELETE FROM
                if is_compound_first(&upper) {
                    if let Some(j) = next_significant_token(&filtered, i) {
                        if let Token::Keyword(next_kw) = filtered[j] {
                            let next_upper = next_kw.to_uppercase();
                            if is_compound_second(&next_upper) {
                                emit_clause_line(&mut out, &mut line_started, &mut need_blank_line, &mut in_clause_content, base_indent, &format!("{} {}", upper, next_upper));
                                in_clause_content = true;
                                last_was_keyword = true;
                                i = j + 1;
                                continue;
                            }
                        }
                    }
                }

                // Major clause keywords
                if is_clause_keyword(&upper) {
                    emit_clause_line(&mut out, &mut line_started, &mut need_blank_line, &mut in_clause_content, base_indent, &upper);
                    in_clause_content = true;
                    last_was_keyword = true;
                    i += 1;
                } else if upper == "AND" || upper == "OR" {
                    // AND/OR on new line at content indent (base_indent + 1)
                    if line_started {
                        out.push('\n');
                    }
                    out.push_str(&indent_str(base_indent + 1));
                    out.push_str(&upper);
                    line_started = true;
                    last_was_keyword = true;
                    i += 1;
                } else {
                    // Regular keyword (AS, NOT, IN, IS, NULL, COUNT, etc.)
                    emit_inline_keyword(&mut out, &mut line_started, &mut last_was_keyword, base_indent, in_clause_content, &upper);
                    i += 1;
                }
            }
            Token::Comma => {
                if inline_paren_depth > 0 {
                    // Inside inline parens: comma + space, no newline
                    out.push(',');
                    // Space will be added by next token
                    line_started = true;
                } else {
                    // Trailing comma style: comma at end, newline, next item indented
                    out.push(',');
                    out.push('\n');
                    line_started = false;
                }
                last_was_keyword = false;
                i += 1;
            }
            Token::Semicolon => {
                out.push(';');
                out.push('\n');
                line_started = false;
                in_clause_content = false;
                need_blank_line = true;
                last_was_keyword = false;
                i += 1;
            }
            Token::OpenParen => {
                let is_subquery = paren_contains_subquery(&filtered, i);
                if is_subquery {
                    let paren_indent = if in_clause_content { base_indent + 1 } else { base_indent };
                    if !line_started {
                        out.push_str(&indent_str(paren_indent));
                    } else {
                        out.push(' ');
                    }
                    out.push('(');
                    out.push('\n');
                    paren_stack.push((base_indent, in_clause_content, true));
                    base_indent = paren_indent + 1;
                    in_clause_content = false;
                    line_started = false;
                } else {
                    if inline_paren_depth == 0 {
                        paren_stack.push((base_indent, in_clause_content, false));
                    }
                    inline_paren_depth += 1;
                    // Space before ( depends on context
                    if !line_started {
                        out.push_str(&indent_str(if in_clause_content { base_indent + 1 } else { base_indent }));
                    } else if !last_was_keyword {
                        out.push(' ');
                    }
                    out.push('(');
                    line_started = true;
                }
                last_was_keyword = false;
                i += 1;
            }
            Token::CloseParen => {
                if inline_paren_depth > 0 {
                    out.push(')');
                    inline_paren_depth -= 1;
                    if inline_paren_depth == 0 {
                        paren_stack.pop();
                    }
                    line_started = true;
                } else if let Some((saved_base, saved_in_clause, true)) = paren_stack.pop() {
                    // Closing a subquery paren
                    if line_started {
                        out.push('\n');
                    }
                    // Close paren at the indent level of the content of the outer clause
                    out.push_str(&indent_str(saved_base + 1));
                    out.push(')');
                    base_indent = saved_base;
                    in_clause_content = saved_in_clause;
                    line_started = true;
                } else {
                    out.push(')');
                    line_started = true;
                }
                last_was_keyword = false;
                i += 1;
            }
            Token::Operator(op) => {
                if op == "." {
                    out.push('.');
                } else {
                    if inline_paren_depth > 0 {
                        if !out.ends_with('(') {
                            out.push(' ');
                        }
                    } else if line_started {
                        out.push(' ');
                    } else if in_clause_content {
                        out.push_str(&indent_str(base_indent + 1));
                    } else {
                        out.push_str(&indent_str(base_indent));
                    }
                    out.push_str(op);
                }
                line_started = true;
                last_was_keyword = false;
                i += 1;
            }
            _ => {
                // Identifier, StringLiteral, NumberLiteral, Other
                let text = match token {
                    Token::Identifier(s) | Token::StringLiteral(s) | Token::NumberLiteral(s) | Token::Other(s) => s.as_str(),
                    _ => unreachable!(),
                };

                if inline_paren_depth > 0 {
                    if line_started && !out.ends_with('(') && !out.ends_with('.') {
                        out.push(' ');
                    }
                    out.push_str(text);
                    line_started = true;
                } else if !line_started {
                    // Start of new line: indent at content level
                    if in_clause_content {
                        out.push_str(&indent_str(base_indent + 1));
                    } else {
                        out.push_str(&indent_str(base_indent));
                    }
                    out.push_str(text);
                    line_started = true;
                } else {
                    if !out.ends_with('.') {
                        out.push(' ');
                    }
                    out.push_str(text);
                }
                last_was_keyword = false;
                i += 1;
            }
        }
    }

    out
}

fn emit_clause_line(out: &mut String, line_started: &mut bool, need_blank_line: &mut bool, in_clause_content: &mut bool, base_indent: usize, clause_text: &str) {
    if *need_blank_line {
        out.push('\n');
        *need_blank_line = false;
    }
    if *line_started {
        out.push('\n');
    }
    out.push_str(&indent_str(base_indent));
    out.push_str(clause_text);
    out.push('\n');
    *line_started = false;
    *in_clause_content = false;
}

fn emit_inline_keyword(out: &mut String, line_started: &mut bool, last_was_keyword: &mut bool, base_indent: usize, in_clause_content: bool, upper: &str) {
    if !*line_started {
        if in_clause_content {
            out.push_str(&indent_str(base_indent + 1));
        } else {
            out.push_str(&indent_str(base_indent));
        }
    } else {
        out.push(' ');
    }
    out.push_str(upper);
    *line_started = true;
    *last_was_keyword = true;
}

#[derive(Clone, Copy, PartialEq)]
enum PrevToken {
    None,
    Keyword,
    Word,
    Operator,
    Comma,
    Semicolon,
    OpenParen,
    CloseParen,
}

fn needs_space(prev: PrevToken, token: &Token) -> bool {
    match token {
        Token::Keyword(_) => matches!(
            prev,
            PrevToken::Keyword | PrevToken::Word | PrevToken::Operator | PrevToken::CloseParen
        ),
        Token::Identifier(_) | Token::StringLiteral(_) | Token::NumberLiteral(_) | Token::Other(_) => {
            matches!(
                prev,
                PrevToken::Keyword | PrevToken::Word | PrevToken::CloseParen
            )
        }
        Token::Operator(_) => prev == PrevToken::Keyword,
        _ => false,
    }
}

pub fn minify(tokens: &[Token]) -> String {
    let mut out = String::new();
    let mut prev = PrevToken::None;

    for token in tokens {
        match token {
            Token::Whitespace(_) | Token::Comment(_) => continue,
            _ => {}
        }

        if needs_space(prev, token) {
            out.push(' ');
        }

        match token {
            Token::Keyword(kw) => {
                out.push_str(&kw.to_uppercase());
                prev = PrevToken::Keyword;
            }
            Token::Identifier(id) => {
                out.push_str(id);
                prev = PrevToken::Word;
            }
            Token::StringLiteral(s) => {
                out.push_str(s);
                prev = PrevToken::Word;
            }
            Token::NumberLiteral(n) => {
                out.push_str(n);
                prev = PrevToken::Word;
            }
            Token::Operator(op) => {
                out.push_str(op);
                prev = PrevToken::Operator;
            }
            Token::Comma => {
                out.push(',');
                prev = PrevToken::Comma;
            }
            Token::Semicolon => {
                out.push(';');
                prev = PrevToken::Semicolon;
            }
            Token::OpenParen => {
                out.push('(');
                prev = PrevToken::OpenParen;
            }
            Token::CloseParen => {
                out.push(')');
                prev = PrevToken::CloseParen;
            }
            Token::Other(o) => {
                out.push_str(o);
                prev = PrevToken::Word;
            }
            Token::Whitespace(_) | Token::Comment(_) => unreachable!(),
        }
    }

    out.trim_end().to_string()
}
