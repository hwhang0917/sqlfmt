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

// Keywords allowed between CREATE/ALTER and TABLE (e.g., CREATE OR REPLACE TABLE)
fn is_ddl_modifier(kw: &str) -> bool {
    matches!(kw, "OR" | "REPLACE" | "TEMPORARY" | "TEMP" | "UNIQUE" | "IF" | "NOT" | "EXISTS")
}

fn indent_str(level: usize) -> String {
    "  ".repeat(level)
}

fn filter_tokens(tokens: &[Token]) -> Vec<&Token> {
    tokens.iter().filter(|t| !matches!(t, Token::Whitespace(_))).collect()
}

fn next_significant_token(tokens: &[&Token], from: usize) -> Option<usize> {
    for (j, tok) in tokens.iter().enumerate().skip(from + 1) {
        if !matches!(tok, Token::Comment(_)) {
            return Some(j);
        }
    }
    None
}

fn ends_in_word_like(s: &str) -> bool {
    s.chars().last().is_some_and(|c| c.is_alphanumeric() || c == '_' || c == '`' || c == '"' || c == ']')
}

fn paren_contains_subquery(tokens: &[&Token], start: usize) -> bool {
    if let Some(j) = next_significant_token(tokens, start) {
        if let Token::Keyword(kw) = tokens[j] {
            return is_statement_starter(&kw.to_uppercase());
        }
    }
    false
}

#[derive(Clone, Copy, PartialEq, Debug)]
enum ParenMode {
    Inline,
    Subquery,
    DefList,
}

struct ParenCtx {
    saved_base_indent: usize,
    saved_in_clause_content: bool,
    mode: ParenMode,
}

fn innermost_mode(stack: &[ParenCtx]) -> Option<ParenMode> {
    stack.last().map(|c| c.mode)
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
    let mut paren_stack: Vec<ParenCtx> = Vec::new();
    let mut last_was_keyword = false;

    // DDL context: mark the next top-level `(` after CREATE/ALTER TABLE as a
    // column definition list (one item per line).
    let mut saw_create_alter = false;
    let mut expect_def_list_paren = false;

    // Unary sign detection: when `-` or `+` appears without a value on its
    // left, attach the next value directly (DEFAULT -1, not DEFAULT - 1).
    let mut prev_was_value = false;
    let mut attach_next = false;

    let mut i = 0;
    while i < filtered.len() {
        let token = filtered[i];
        let in_inline = innermost_mode(&paren_stack) == Some(ParenMode::Inline);

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
                attach_next = false;
                prev_was_value = false;
                i += 1;
            }
            Token::Keyword(kw) => {
                let upper = kw.to_uppercase();
                attach_next = false;

                // After a dot (e.g., t.count), emit as-is without spacing.
                // The qualified name is value-like for unary/binary detection.
                if out.ends_with('.') {
                    out.push_str(&upper);
                    line_started = true;
                    last_was_keyword = false;
                    prev_was_value = true;
                    i += 1;
                    continue;
                }

                // Value-like keywords (NULL, TRUE, FALSE, etc.) are treated as
                // values so a following `-` is binary.
                prev_was_value = matches!(upper.as_str(), "TRUE" | "FALSE" | "NULL" | "UNBOUNDED");

                // Inside inline parens, keywords are just inline
                if in_inline {
                    if line_started {
                        out.push(' ');
                    }
                    out.push_str(&upper);
                    line_started = true;
                    last_was_keyword = true;
                    i += 1;
                    continue;
                }

                // DDL state tracking (only at top level outside any paren)
                if paren_stack.is_empty() {
                    if matches!(upper.as_str(), "CREATE" | "ALTER") {
                        saw_create_alter = true;
                    } else if upper == "TABLE" && saw_create_alter {
                        expect_def_list_paren = true;
                        saw_create_alter = false;
                    } else if !is_ddl_modifier(&upper) {
                        saw_create_alter = false;
                    }
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
                    if line_started {
                        out.push('\n');
                    }
                    out.push_str(&indent_str(base_indent + 1));
                    out.push_str(&upper);
                    line_started = true;
                    last_was_keyword = true;
                    i += 1;
                } else {
                    emit_inline_keyword(&mut out, &mut line_started, &mut last_was_keyword, base_indent, in_clause_content, &upper);
                    i += 1;
                }
            }
            Token::Comma => {
                if in_inline {
                    out.push(',');
                    line_started = true;
                } else {
                    // Clause-level or DefList: comma at end, newline, next item re-indents
                    out.push(',');
                    out.push('\n');
                    line_started = false;
                }
                last_was_keyword = false;
                prev_was_value = false;
                attach_next = false;
                i += 1;
            }
            Token::Semicolon => {
                out.push(';');
                out.push('\n');
                line_started = false;
                in_clause_content = false;
                need_blank_line = true;
                last_was_keyword = false;
                saw_create_alter = false;
                expect_def_list_paren = false;
                prev_was_value = false;
                attach_next = false;
                i += 1;
            }
            Token::OpenParen => {
                let mode = if paren_contains_subquery(&filtered, i) {
                    ParenMode::Subquery
                } else if expect_def_list_paren && paren_stack.is_empty() {
                    ParenMode::DefList
                } else {
                    ParenMode::Inline
                };
                expect_def_list_paren = false;

                match mode {
                    ParenMode::Subquery => {
                        let paren_indent = if in_clause_content { base_indent + 1 } else { base_indent };
                        if !line_started {
                            out.push_str(&indent_str(paren_indent));
                        } else {
                            out.push(' ');
                        }
                        out.push('(');
                        out.push('\n');
                        paren_stack.push(ParenCtx {
                            saved_base_indent: base_indent,
                            saved_in_clause_content: in_clause_content,
                            mode,
                        });
                        base_indent = paren_indent + 1;
                        in_clause_content = false;
                        line_started = false;
                    }
                    ParenMode::DefList => {
                        if !line_started {
                            out.push_str(&indent_str(base_indent));
                        } else {
                            out.push(' ');
                        }
                        out.push('(');
                        out.push('\n');
                        paren_stack.push(ParenCtx {
                            saved_base_indent: base_indent,
                            saved_in_clause_content: in_clause_content,
                            mode,
                        });
                        base_indent += 1;
                        in_clause_content = false;
                        line_started = false;
                    }
                    ParenMode::Inline => {
                        if !line_started {
                            out.push_str(&indent_str(if in_clause_content { base_indent + 1 } else { base_indent }));
                        } else if !last_was_keyword && !ends_in_word_like(&out) {
                            out.push(' ');
                        }
                        out.push('(');
                        paren_stack.push(ParenCtx {
                            saved_base_indent: base_indent,
                            saved_in_clause_content: in_clause_content,
                            mode,
                        });
                        line_started = true;
                    }
                }
                last_was_keyword = false;
                prev_was_value = false;
                attach_next = false;
                i += 1;
            }
            Token::CloseParen => {
                match paren_stack.pop() {
                    Some(ctx) => match ctx.mode {
                        ParenMode::Inline => {
                            out.push(')');
                            line_started = true;
                        }
                        ParenMode::Subquery => {
                            if line_started {
                                out.push('\n');
                            }
                            out.push_str(&indent_str(ctx.saved_base_indent + 1));
                            out.push(')');
                            base_indent = ctx.saved_base_indent;
                            in_clause_content = ctx.saved_in_clause_content;
                            line_started = true;
                        }
                        ParenMode::DefList => {
                            if line_started {
                                out.push('\n');
                            }
                            out.push_str(&indent_str(ctx.saved_base_indent));
                            out.push(')');
                            base_indent = ctx.saved_base_indent;
                            in_clause_content = ctx.saved_in_clause_content;
                            line_started = true;
                        }
                    },
                    None => {
                        out.push(')');
                        line_started = true;
                    }
                }
                last_was_keyword = false;
                prev_was_value = true;
                attach_next = false;
                i += 1;
            }
            Token::Operator(op) => {
                if op == "." {
                    out.push('.');
                    line_started = true;
                    last_was_keyword = false;
                    attach_next = false;
                    // prev_was_value stays as-is so the next identifier after
                    // `.` is emitted without a leading space (handled by the
                    // `out.ends_with('.')` check).
                    i += 1;
                    continue;
                }

                let is_unary = matches!(op.as_str(), "-" | "+") && !prev_was_value;

                if in_inline {
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
                line_started = true;
                last_was_keyword = false;
                attach_next = is_unary;
                prev_was_value = false;
                i += 1;
            }
            _ => {
                let text = match token {
                    Token::Identifier(s) | Token::StringLiteral(s) | Token::NumberLiteral(s) | Token::Other(s) => s.as_str(),
                    _ => unreachable!(),
                };

                if attach_next {
                    out.push_str(text);
                    line_started = true;
                } else if in_inline {
                    if line_started && !out.ends_with('(') && !out.ends_with('.') {
                        out.push(' ');
                    }
                    out.push_str(text);
                    line_started = true;
                } else if !line_started {
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
                prev_was_value = true;
                attach_next = false;
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
    Dot,
    Comma,
    Semicolon,
    OpenParen,
    CloseParen,
}

fn needs_space(prev: PrevToken, token: &Token) -> bool {
    if prev == PrevToken::Dot {
        return false;
    }
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
        Token::Operator(op) if op == "." => false,
        Token::Operator(_) => prev == PrevToken::Keyword,
        _ => false,
    }
}

pub struct Palette {
    pub keyword: &'static str,
    pub identifier: &'static str,
    pub string: &'static str,
    pub number: &'static str,
    pub operator: &'static str,
    pub comment: &'static str,
    pub punct: &'static str,
    pub reset: &'static str,
}

impl Palette {
    pub const fn ansi() -> Self {
        Self {
            keyword: "\x1b[1;36m",
            identifier: "",
            string: "\x1b[32m",
            number: "\x1b[33m",
            operator: "",
            comment: "\x1b[2m",
            punct: "",
            reset: "\x1b[0m",
        }
    }

    pub const fn none() -> Self {
        Self {
            keyword: "",
            identifier: "",
            string: "",
            number: "",
            operator: "",
            comment: "",
            punct: "",
            reset: "",
        }
    }
}

pub fn colorize(formatted: &str, palette: &Palette) -> String {
    let tokens = crate::tokenizer::tokenize(formatted);
    let mut out = String::with_capacity(formatted.len());
    for token in tokens {
        match token {
            Token::Keyword(kw) => {
                out.push_str(palette.keyword);
                out.push_str(&kw);
                out.push_str(palette.reset);
            }
            Token::Identifier(id) => {
                out.push_str(palette.identifier);
                out.push_str(&id);
                if !palette.identifier.is_empty() {
                    out.push_str(palette.reset);
                }
            }
            Token::StringLiteral(s) => {
                out.push_str(palette.string);
                out.push_str(&s);
                out.push_str(palette.reset);
            }
            Token::NumberLiteral(n) => {
                out.push_str(palette.number);
                out.push_str(&n);
                out.push_str(palette.reset);
            }
            Token::Operator(op) => {
                out.push_str(palette.operator);
                out.push_str(&op);
                if !palette.operator.is_empty() {
                    out.push_str(palette.reset);
                }
            }
            Token::Comment(c) => {
                out.push_str(palette.comment);
                out.push_str(&c);
                out.push_str(palette.reset);
            }
            Token::Comma => {
                out.push_str(palette.punct);
                out.push(',');
                if !palette.punct.is_empty() {
                    out.push_str(palette.reset);
                }
            }
            Token::Semicolon => {
                out.push_str(palette.punct);
                out.push(';');
                if !palette.punct.is_empty() {
                    out.push_str(palette.reset);
                }
            }
            Token::OpenParen => {
                out.push_str(palette.punct);
                out.push('(');
                if !palette.punct.is_empty() {
                    out.push_str(palette.reset);
                }
            }
            Token::CloseParen => {
                out.push_str(palette.punct);
                out.push(')');
                if !palette.punct.is_empty() {
                    out.push_str(palette.reset);
                }
            }
            Token::Whitespace(ws) => {
                out.push_str(&ws);
            }
            Token::Other(o) => {
                out.push_str(&o);
            }
        }
    }
    out
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
                prev = if op == "." { PrevToken::Dot } else { PrevToken::Operator };
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
