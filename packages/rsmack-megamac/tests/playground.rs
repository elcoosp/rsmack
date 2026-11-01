//! Integration tests driving the `playground` binary end to end.

use std::process::Command;

/// Run the playground binary with the given arguments.
fn run(args: &[&str]) -> (String, String, std::process::ExitStatus) {
    let output = Command::new(env!("CARGO_BIN_EXE_playground"))
        .args(args)
        .output()
        .expect("playground binary runs");
    (
        String::from_utf8_lossy(&output.stdout).to_string(),
        String::from_utf8_lossy(&output.stderr).to_string(),
        output.status,
    )
}

#[test]
fn prints_the_expansion_of_the_self_documented_macro() {
    let (stdout, stderr, status) = run(&["--kind", "Func", "--name", "megamac"]);
    assert!(status.success(), "stderr: {stderr}");
    assert!(stdout.contains("# [proc_macro]"));
    assert!(stdout.contains("pub fn megamac"));
    assert!(stdout.contains("func_entrypoint"));
}

#[test]
fn accepts_src_dir_impls_args_and_doc_options() {
    let src_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let (stdout, stderr, status) = run(&[
        "--kind",
        "Func",
        "--name",
        "megamac",
        "--src-dir",
        src_dir.to_str().expect("utf-8 src dir"),
        "--impls",
        "impls",
        "--args",
        "Args",
        "--doc",
        "inline",
    ]);
    assert!(status.success(), "stderr: {stderr}");
    assert!(stdout.contains("# [proc_macro]"));
}

#[test]
fn missing_required_options_print_usage_and_fail() {
    let (stdout, stderr, status) = run(&["--kind", "Func"]);
    assert!(!status.success());
    assert_eq!(stdout, "");
    assert!(stderr.contains("missing required option `--name`"));
    assert!(stderr.contains("usage: playground"));
}

#[test]
fn unexpected_arguments_are_rejected() {
    let (_, stderr, status) = run(&["positional", "value"]);
    assert!(!status.success());
    assert!(stderr.contains("unexpected argument `positional`"));
}

#[test]
fn unknown_kind_fails_with_the_macro_error() {
    let (_, stderr, status) = run(&["--kind", "Nope", "--name", "demo"]);
    assert!(!status.success());
    assert!(stderr.contains("Unsupported MacroKind `Nope`"));
}

#[test]
fn missing_impl_file_fails_with_the_macro_error() {
    let (_, stderr, status) = run(&["--kind", "Func", "--name", "does_not_exist"]);
    assert!(!status.success());
    assert!(stderr.contains("Failed to get macro_impl_src of does_not_exist"));
}

#[test]
fn invalid_ident_fails_cleanly() {
    let (_, stderr, status) = run(&["--kind", "Func", "--name", "not an ident!"]);
    assert!(!status.success());
    assert!(stderr.contains("is not a valid Rust identifier"));
}
