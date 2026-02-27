#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Keyword(String),
    Identifier(String),
    StringLiteral(String),
    NumberLiteral(String),
    Operator(String),
    Comma,
    Semicolon,
    OpenParen,
    CloseParen,
    Comment(String),
    Whitespace(String),
    Other(String),
}

const KEYWORDS: &[&str] = &[
    "SELECT", "FROM", "WHERE", "JOIN", "LEFT", "RIGHT", "INNER", "OUTER",
    "CROSS", "FULL", "ON", "AND", "OR", "NOT", "IN", "IS", "NULL", "AS",
    "GROUP", "BY", "ORDER", "HAVING", "LIMIT", "OFFSET", "INSERT", "INTO",
    "VALUES", "UPDATE", "SET", "DELETE", "CREATE", "TABLE", "DROP", "ALTER",
    "INDEX", "VIEW", "UNION", "ALL", "DISTINCT", "EXCEPT", "INTERSECT",
    "EXISTS", "BETWEEN", "LIKE", "CASE", "WHEN", "THEN", "ELSE", "END",
    "ASC", "DESC", "TRUE", "FALSE", "CAST", "WITH", "RECURSIVE", "PRIMARY",
    "KEY", "FOREIGN", "REFERENCES", "CONSTRAINT", "DEFAULT", "CHECK",
    "UNIQUE", "IF", "REPLACE", "TEMPORARY", "TEMP", "RETURNING",
    "NATURAL", "USING", "FETCH", "NEXT", "ROWS", "ONLY", "FIRST",
    "NULLS", "LAST", "COUNT", "SUM", "AVG", "MIN", "MAX", "COALESCE",
    "OVER", "PARTITION", "ROW_NUMBER", "RANK", "DENSE_RANK", "LAG", "LEAD",
    "WINDOW", "RANGE", "UNBOUNDED", "PRECEDING", "FOLLOWING", "CURRENT",
    "ROW", "GRANT", "REVOKE", "ROLLBACK", "COMMIT", "BEGIN", "TRANSACTION",
    "SAVEPOINT", "RELEASE", "TRIGGER", "EXECUTE", "PROCEDURE", "FUNCTION",
    "DECLARE", "CURSOR", "OPEN", "CLOSE",
];

fn is_keyword(word: &str) -> bool {
    KEYWORDS.contains(&word.to_uppercase().as_str())
}

pub fn tokenize(input: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    let chars: Vec<char> = input.chars().collect();
    let len = chars.len();
    let mut i = 0;

    while i < len {
        let ch = chars[i];

        // Whitespace
        if ch.is_ascii_whitespace() {
            let start = i;
            while i < len && chars[i].is_ascii_whitespace() {
                i += 1;
            }
            tokens.push(Token::Whitespace(chars[start..i].iter().collect()));
            continue;
        }

        // Line comment
        if ch == '-' && i + 1 < len && chars[i + 1] == '-' {
            let start = i;
            while i < len && chars[i] != '\n' {
                i += 1;
            }
            tokens.push(Token::Comment(chars[start..i].iter().collect()));
            continue;
        }

        // Block comment
        if ch == '/' && i + 1 < len && chars[i + 1] == '*' {
            let start = i;
            i += 2;
            while i + 1 < len && !(chars[i] == '*' && chars[i + 1] == '/') {
                i += 1;
            }
            if i + 1 < len {
                i += 2; // skip */
            }
            tokens.push(Token::Comment(chars[start..i].iter().collect()));
            continue;
        }

        // String literal
        if ch == '\'' {
            let start = i;
            i += 1;
            while i < len {
                if chars[i] == '\'' {
                    if i + 1 < len && chars[i + 1] == '\'' {
                        i += 2; // escaped quote
                    } else {
                        i += 1;
                        break;
                    }
                } else {
                    i += 1;
                }
            }
            tokens.push(Token::StringLiteral(chars[start..i].iter().collect()));
            continue;
        }

        // Quoted identifier
        if ch == '"' {
            let start = i;
            i += 1;
            while i < len && chars[i] != '"' {
                i += 1;
            }
            if i < len {
                i += 1;
            }
            tokens.push(Token::Identifier(chars[start..i].iter().collect()));
            continue;
        }

        // Number
        if ch.is_ascii_digit() || (ch == '.' && i + 1 < len && chars[i + 1].is_ascii_digit()) {
            let start = i;
            while i < len && (chars[i].is_ascii_digit() || chars[i] == '.') {
                i += 1;
            }
            tokens.push(Token::NumberLiteral(chars[start..i].iter().collect()));
            continue;
        }

        // Punctuation
        match ch {
            '(' => { tokens.push(Token::OpenParen); i += 1; continue; }
            ')' => { tokens.push(Token::CloseParen); i += 1; continue; }
            ',' => { tokens.push(Token::Comma); i += 1; continue; }
            ';' => { tokens.push(Token::Semicolon); i += 1; continue; }
            _ => {}
        }

        // Multi-char operators
        if i + 1 < len {
            let two: String = chars[i..i+2].iter().collect();
            if matches!(two.as_str(), "<>" | "<=" | ">=" | "!=" | "||") {
                tokens.push(Token::Operator(two));
                i += 2;
                continue;
            }
        }

        // Single-char operators
        if matches!(ch, '=' | '<' | '>' | '+' | '-' | '*' | '/' | '%' | '&' | '|' | '^' | '~') {
            tokens.push(Token::Operator(ch.to_string()));
            i += 1;
            continue;
        }

        // Word (keyword or identifier)
        if ch.is_alphanumeric() || ch == '_' {
            let start = i;
            while i < len && (chars[i].is_alphanumeric() || chars[i] == '_') {
                i += 1;
            }
            let word: String = chars[start..i].iter().collect();
            if is_keyword(&word) {
                tokens.push(Token::Keyword(word));
            } else {
                tokens.push(Token::Identifier(word));
            }
            continue;
        }

        // Dot
        if ch == '.' {
            tokens.push(Token::Operator(".".to_string()));
            i += 1;
            continue;
        }

        // Other
        tokens.push(Token::Other(ch.to_string()));
        i += 1;
    }

    tokens
}
