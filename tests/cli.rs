//! CLI-level regression tests: these lock the exit-code and stderr
//! contracts of review findings F1-F6 (previously verified only by hand)
//! so they cannot silently return, plus the strict/empty-prefix contract.

use std::io::Write;
use std::process::{Command, Stdio};

fn run(args: &[&str]) -> (String, String, i32) {
    run_stdin(args, b"")
}

fn run_stdin(args: &[&str], input: &[u8]) -> (String, String, i32) {
    let mut child = Command::new(env!("CARGO_BIN_EXE_azar"))
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn azar");
    child
        .stdin
        .take()
        .expect("piped stdin")
        .write_all(input)
        .expect("write stdin");
    let out = child.wait_with_output().expect("wait azar");
    (
        String::from_utf8_lossy(&out.stdout).into_owned(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
        out.status.code().unwrap_or(-1),
    )
}

#[test]
fn f1_multibyte_verify_is_rejected_not_panicking() {
    // regression F1: non-ASCII input must be rejected (exit 1), never panic
    let (out, err, code) = run_stdin(&["--verify"], "ñññññññ\n".as_bytes());
    assert_eq!(code, 1, "stderr: {err}");
    assert!(err.contains("0 of 1"), "stderr: {err}");
    assert!(out.is_empty(), "verify keeps stdout clean");
}

#[test]
fn f2_literal_prefix_length_underflow_is_clean_error() {
    // regression F2: -l shorter than the literal prefix underflows
    let (_, err, code) = run(&["-f", "hex", "-P", "ab", "-l", "1"]);
    assert_eq!(code, 1, "stderr: {err}");
    assert!(err.contains("below the minimum 3"), "stderr: {err}");
}

#[test]
fn f3_empty_verify_stdin_is_clean_error() {
    // regression F3: empty stdin fails fast with a message, exit 1
    let (out, err, code) = run(&["--verify"]);
    assert_eq!(code, 1, "stderr: {err}");
    assert!(err.contains("empty input"), "stderr: {err}");
    assert!(out.is_empty(), "verify keeps stdout clean");
}

#[test]
fn f4_classic_format_has_no_bare_form() {
    // regression F4: bech32 classic refuses bare output at any length...
    let (_, err, code) = run(&["-f", "bech32", "-l", "12"]);
    assert_eq!(code, 1, "stderr: {err}");
    assert!(err.contains("no bare form"), "stderr: {err}");
    // ...and requests below the structural floor report the floor first
    let (_, err, code) = run(&["-f", "bech32", "-l", "5"]);
    assert_eq!(code, 1, "stderr: {err}");
    assert!(err.contains("below the minimum 7"), "stderr: {err}");
}

#[test]
fn f6_zero_length_requests_are_clean_errors() {
    // regression F6: -l 0 / -b 0 / -B 0 must never emit zero-entropy output
    for args in [["-l", "0"], ["-b", "0"], ["-B", "0"]] {
        let (out, err, code) = run(&args);
        assert_eq!(code, 1, "args {args:?}, stderr: {err}");
        assert!(err.contains("zero-length"), "args {args:?}, stderr: {err}");
        assert!(out.is_empty(), "args {args:?} must print nothing");
    }
}

#[test]
fn generator_verifier_contract_holds_at_the_floor() {
    // the shortest generatable bare token verifies; shorter is refused
    let (tok, err, code) = run(&["-l", "7", "-n"]);
    assert_eq!(code, 0, "stderr: {err}");
    assert_eq!(tok.chars().count(), 7, "output: {tok}");
    let (_, verr, vcode) = run_stdin(&["--verify"], format!("{tok}\n").as_bytes());
    assert_eq!(vcode, 0, "stderr: {verr}");
    assert!(verr.contains("1 of 1"), "stderr: {verr}");

    let (_, err, code) = run(&["-l", "6"]);
    assert_eq!(code, 1, "stderr: {err}");
    assert!(err.contains("below the minimum 7"), "stderr: {err}");
}

#[test]
fn usage_errors_exit_2() {
    // README contract: runtime errors exit 1, clap usage errors exit 2
    let (_, err, code) = run(&["--bogus"]);
    assert_eq!(code, 2, "stderr: {err}");
    let (_, err, code) = run(&["-R", "1"]);
    assert_eq!(code, 2, "stderr: {err}");
}

#[test]
fn strict_with_empty_prefix_still_emits_standard_form() {
    // an explicitly empty -P must behave as absent: --strict implies -P r
    // and never silently falls back to bare output
    let (out, err, code) = run(&["-s", "-P", "", "-l", "40"]);
    assert_eq!(code, 0, "stderr: {err}");
    let out = out.trim();
    assert!(out.starts_with("r1"), "strict must not be bare: {out}");
    assert_eq!(out.chars().count(), 40, "output: {out}");
    let (_, verr, vcode) = run_stdin(&["--verify"], format!("{out}\n").as_bytes());
    assert_eq!(vcode, 0, "stderr: {verr}");

    // without --strict an empty -P keeps the documented bare default
    let (out, err, code) = run(&["-P", "", "-l", "12"]);
    assert_eq!(code, 0, "stderr: {err}");
    assert!(!out.contains('1'), "empty prefix stays bare: {out}");
}
