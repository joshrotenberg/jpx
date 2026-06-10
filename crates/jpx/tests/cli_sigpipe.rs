//! Regression test for broken-pipe handling (issue #188).
//!
//! `jpx ... | head` must terminate cleanly via SIGPIPE rather than panicking
//! on EPIPE. This is Unix-specific behaviour.
#![cfg(unix)]

use std::io::{Read, Write};
use std::os::unix::process::ExitStatusExt;
use std::process::{Command, Stdio};

/// SIGPIPE signal number (13 on Linux and macOS).
const SIGPIPE: i32 = 13;

#[test]
fn broken_pipe_terminates_via_sigpipe_not_panic() {
    // A JSON array far larger than a pipe buffer (~64 KiB), so the process is
    // guaranteed to still be writing when we close the read end.
    let big = format!(
        "[{}]",
        (0..200_000)
            .map(|i| i.to_string())
            .collect::<Vec<_>>()
            .join(",")
    );
    let mut input = tempfile::NamedTempFile::new().unwrap();
    input.write_all(big.as_bytes()).unwrap();

    let mut child = Command::new(env!("CARGO_BIN_EXE_jpx"))
        .arg("@")
        .arg(input.path())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();

    // Read a little, then close the read end of the pipe.
    {
        let mut out = child.stdout.take().unwrap();
        let mut buf = [0u8; 16];
        let _ = out.read(&mut buf);
    } // `out` dropped here -> the downstream reader has "gone away"

    let status = child.wait().unwrap();

    // Killed by SIGPIPE -> signal() is Some(13); a regression would instead
    // panic on EPIPE (exit code 101) with no terminating signal.
    assert_eq!(
        status.signal(),
        Some(SIGPIPE),
        "expected termination by SIGPIPE, got {status:?}"
    );
}
