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
    // ...at ANY length, including below the checksum floor: the format
    // impossibility wins over length arithmetic
    let (_, err, code) = run(&["-f", "bech32", "-l", "5"]);
    assert_eq!(code, 1, "stderr: {err}");
    assert!(err.contains("no bare form"), "stderr: {err}");
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
fn checksummed_floor_verifies_and_shorter_is_plain_but_permissive() {
    // the shortest checksummed bare token verifies
    let (tok, err, code) = run(&["-l", "7", "-n"]);
    assert_eq!(code, 0, "stderr: {err}");
    assert_eq!(tok.chars().count(), 7, "output: {tok}");
    let (_, verr, vcode) = run_stdin(&["--verify"], format!("{tok}\n").as_bytes());
    assert_eq!(vcode, 0, "stderr: {verr}");
    assert!(verr.contains("1 of 1"), "stderr: {verr}");

    // permissive default: -l 6 PRODUCES output (checksum-less) instead of
    // erroring — it simply does not verify
    let (tok6, err, code) = run(&["-l", "6", "-n"]);
    assert_eq!(code, 0, "stderr: {err}");
    assert_eq!(tok6.chars().count(), 6, "output: {tok6}");
    let (_, verr, vcode) = run_stdin(&["--verify"], format!("{tok6}\n").as_bytes());
    assert_eq!(vcode, 1, "plain output must NOT verify: {verr}");
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
fn standard_flag_forces_standard_form_even_with_empty_prefix() {
    // an explicitly empty -P must behave as absent: --standard implies -r
    // and never silently falls back to bare output
    let (out, err, code) = run(&["--standard", "-P", "", "-l", "40"]);
    assert_eq!(code, 0, "stderr: {err}");
    let out = out.trim();
    assert!(out.starts_with("r1"), "standard must not be bare: {out}");
    assert_eq!(out.chars().count(), 40, "output: {out}");
    let (_, verr, vcode) = run_stdin(&["--verify"], format!("{out}\n").as_bytes());
    assert_eq!(vcode, 0, "stderr: {verr}");

    // without the flag an empty -P keeps the documented bare default
    let (out, err, code) = run(&["-P", "", "-l", "12"]);
    assert_eq!(code, 0, "stderr: {err}");
    assert!(!out.contains('1'), "empty prefix stays bare: {out}");
}

#[test]
fn min_bits_is_enforced_posix_style() {
    // violation: stderr message, non-zero exit, EMPTY stdout (pipe-safe)
    let (out, err, code) = run(&["--min-bits", "128", "-l", "20"]);
    assert_eq!(code, 1, "stderr: {err}");
    assert!(out.is_empty(), "stdout must stay clean, got: {out}");
    assert!(err.contains(">= 128 bits"), "stderr: {err}");

    // bare bech32m: 5 bits x (l - 6) >= 128 -> l >= 32 (5 x 26 = 130)
    let (_, err, code) = run(&["--min-bits", "128", "-l", "31"]);
    assert_eq!(code, 1, "stderr: {err}");
    let (out, err, code) = run(&["--min-bits", "128", "-l", "32"]);
    assert_eq!(code, 0, "stderr: {err}");
    assert_eq!(out.trim().chars().count(), 32);

    // below the checksum floor the plain path prints 6 x 5 = 30 bits of
    // payload (regression pin: a payload-0 branch would report 0 bits)
    let (_, err, code) = run(&["--min-bits", "30", "-l", "6"]);
    assert_eq!(code, 0, "stderr: {err}");
    let (_, _, code) = run(&["--min-bits", "31", "-l", "6"]);
    assert_eq!(code, 1);

    // per-format accounting: hex is exactly 4 bits/char
    let (_, _, code) = run(&["--min-bits", "128", "-f", "hex", "-l", "31"]);
    assert_eq!(code, 1);
    let (_, err, code) = run(&["--min-bits", "128", "-f", "hex", "-l", "32"]);
    assert_eq!(code, 0, "stderr: {err}");
}

#[test]
fn checksum_flag_forbids_unverifiable_output() {
    // below the floor the permissive default would emit plain — the strict
    // flag turns that into an error with clean stdout
    let (out, err, code) = run(&["--checksum", "-l", "6"]);
    assert_eq!(code, 1, "stderr: {err}");
    assert!(out.is_empty(), "stdout must stay clean, got: {out}");
    assert!(
        err.contains("cannot carry the 6-char checksum (minimum 7)"),
        "stderr: {err}"
    );

    // at the floor it passes and the output verifies
    let (tok, err, code) = run(&["--checksum", "-l", "7", "-n"]);
    assert_eq!(code, 0, "stderr: {err}");
    let (_, verr, vcode) = run_stdin(&["--verify"], format!("{tok}\n").as_bytes());
    assert_eq!(vcode, 0, "stderr: {verr}");

    // formats without a checksum cannot honor the flag
    let (_, err, code) = run(&["--checksum", "-f", "hex", "-l", "8"]);
    assert_eq!(code, 1, "stderr: {err}");
    assert!(err.contains("has no checksum"), "stderr: {err}");

    // truncating base58check voids its check — rejected, not silently done
    let (_, err, code) = run(&["--checksum", "-f", "base58check", "-l", "20"]);
    assert_eq!(code, 1, "stderr: {err}");
    assert!(err.contains("voids the check"), "stderr: {err}");

    // base58check WITHOUT -l honors it
    let (_, err, code) = run(&["--checksum", "-f", "base58check"]);
    assert_eq!(code, 0, "stderr: {err}");
}

#[test]
fn standard_flag_rejects_non_bech_formats() {
    let (out, err, code) = run(&["--standard", "-f", "hex"]);
    assert_eq!(code, 1, "stderr: {err}");
    assert!(out.is_empty(), "stdout must stay clean, got: {out}");
    assert!(err.contains("bech32(m) only"), "stderr: {err}");
}

#[test]
fn default_is_permissive_across_lengths() {
    // no plain length may ever break a pipeline
    for l in ["1", "2", "3", "6"] {
        let (out, err, code) = run(&["-l", l, "-n"]);
        assert_eq!(code, 0, "-l {l}: stderr: {err}");
        assert_eq!(out.chars().count(), l.parse::<usize>().unwrap(), "-l {l}");
    }
    // ...only the hard BIP 90-char cap and zero-length remain errors
    let (_, err, code) = run(&["-l", "91"]);
    assert_eq!(code, 1, "stderr: {err}");
    assert!(err.contains("90-character"), "stderr: {err}");
}
