use sqlfmt::tokenizer::tokenize;
use sqlfmt::formatter::beautify;

#[test]
fn beautify_simple_select() {
    let tokens = tokenize("SELECT id, name FROM users WHERE id = 1;");
    let expected = "\
SELECT
  id,
  name
FROM
  users
WHERE
  id = 1;
";
    assert_eq!(beautify(&tokens), expected);
}

#[test]
fn beautify_join() {
    let tokens = tokenize("SELECT a.id FROM a JOIN b ON a.id = b.id;");
    let expected = "\
SELECT
  a.id
FROM
  a
JOIN
  b
ON
  a.id = b.id;
";
    assert_eq!(beautify(&tokens), expected);
}

#[test]
fn beautify_and_or() {
    let tokens = tokenize("SELECT 1 FROM t WHERE a = 1 AND b = 2 OR c = 3;");
    let expected = "\
SELECT
  1
FROM
  t
WHERE
  a = 1
  AND b = 2
  OR c = 3;
";
    assert_eq!(beautify(&tokens), expected);
}

#[test]
fn beautify_multiple_statements() {
    let tokens = tokenize("SELECT 1; SELECT 2;");
    let expected = "\
SELECT
  1;

SELECT
  2;
";
    assert_eq!(beautify(&tokens), expected);
}

#[test]
fn beautify_subquery() {
    let tokens = tokenize("SELECT * FROM (SELECT id FROM t);");
    let expected = "\
SELECT
  *
FROM
  (
    SELECT
      id
    FROM
      t
  );
";
    assert_eq!(beautify(&tokens), expected);
}

#[test]
fn beautify_preserves_comments() {
    let tokens = tokenize("-- header\nSELECT 1;");
    let expected = "\
-- header
SELECT
  1;
";
    assert_eq!(beautify(&tokens), expected);
}

#[test]
fn beautify_keywords_uppercased() {
    let tokens = tokenize("select id from users where id = 1;");
    let expected = "\
SELECT
  id
FROM
  users
WHERE
  id = 1;
";
    assert_eq!(beautify(&tokens), expected);
}

#[test]
fn beautify_insert() {
    let tokens = tokenize("INSERT INTO users (id, name) VALUES (1, 'Alice');");
    let expected = "\
INSERT INTO
  users (id, name)
VALUES
  (1, 'Alice');
";
    assert_eq!(beautify(&tokens), expected);
}

#[test]
fn beautify_group_by_order_by() {
    let tokens = tokenize("SELECT country, COUNT(*) FROM users GROUP BY country ORDER BY country;");
    let expected = "\
SELECT
  country,
  COUNT(*)
FROM
  users
GROUP BY
  country
ORDER BY
  country;
";
    assert_eq!(beautify(&tokens), expected);
}

#[test]
fn beautify_dot_qualified_keyword_column() {
    let tokens = tokenize("SELECT t.count, t.key FROM t;");
    let expected = "\
SELECT
  t.COUNT,
  t.KEY
FROM
  t;
";
    assert_eq!(beautify(&tokens), expected);
}
