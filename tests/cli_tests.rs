use std::process::Command;

fn sqlfmt() -> Command {
    Command::new(env!("CARGO_BIN_EXE_sqlfmt"))
}

#[test]
fn cli_string_arg_beautify() {
    let output = sqlfmt()
        .arg("SELECT * FROM users;")
        .output()
        .expect("failed to run sqlfmt");
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("SELECT"));
    assert!(stdout.contains("FROM"));
}

#[test]
fn cli_string_arg_minify() {
    let output = sqlfmt()
        .args(["-m", "SELECT   *   FROM   users  ;"])
        .output()
        .expect("failed to run sqlfmt");
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert_eq!(stdout.trim(), "SELECT * FROM users;");
}

#[test]
fn cli_stdin_beautify() {
    use std::io::Write;
    use std::process::Stdio;

    let mut child = sqlfmt()
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("failed to run sqlfmt");

    child.stdin.take().unwrap().write_all(b"SELECT 1;").unwrap();
    let output = child.wait_with_output().unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("SELECT"));
}

#[test]
fn cli_stdin_minify() {
    use std::io::Write;
    use std::process::Stdio;

    let mut child = sqlfmt()
        .arg("-m")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("failed to run sqlfmt");

    child.stdin.take().unwrap().write_all(b"SELECT   1  ;").unwrap();
    let output = child.wait_with_output().unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert_eq!(stdout.trim(), "SELECT 1;");
}

#[test]
fn cli_help_flag() {
    let output = sqlfmt()
        .arg("--help")
        .output()
        .expect("failed to run sqlfmt");
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("sqlfmt"));
    assert!(stdout.contains("--minify"));
}

#[test]
fn cli_version_flag() {
    let output = sqlfmt()
        .arg("--version")
        .output()
        .expect("failed to run sqlfmt");
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("sqlfmt"));
    assert!(stdout.contains("0.1.0"));
}

#[test]
fn cli_version_short_flag() {
    let output = sqlfmt()
        .arg("-V")
        .output()
        .expect("failed to run sqlfmt");
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("sqlfmt"));
}

#[test]
fn cli_empty_input() {
    use std::io::Write;
    use std::process::Stdio;

    let mut child = sqlfmt()
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("failed to run sqlfmt");

    child.stdin.take().unwrap().write_all(b"").unwrap();
    let output = child.wait_with_output().unwrap();
    assert!(output.status.success());
    assert!(output.stdout.is_empty());
}
