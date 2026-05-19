//! Smoke test for the LSP wire protocol.
//!
//! This stands up the real `ush_lsp` binary over a pipe pair and
//! drives a minimum LSP handshake:
//!
//!   client → server: initialize
//!   server → client: InitializeResult
//!   client → server: initialized (notification)
//!   client → server: shutdown
//!   server → client: null result
//!   client → server: exit (notification)
//!
//! We do not assert the full schema of the InitializeResult, just
//! that it parses as JSON and carries a result body. The point of
//! this test is to catch the wide class of regressions where the
//! server panics, deadlocks, or stops responding to JSON-RPC at all.

use std::io::{BufRead, BufReader, Read, Write};
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};
use std::time::{Duration, Instant};

fn ush_lsp() -> Command {
    Command::new(env!("CARGO_BIN_EXE_ush_lsp"))
}

fn send(stdin: &mut ChildStdin, body: &str) {
    let header = format!("Content-Length: {}\r\n\r\n", body.len());
    stdin.write_all(header.as_bytes()).expect("write header");
    stdin.write_all(body.as_bytes()).expect("write body");
    stdin.flush().expect("flush");
}

fn recv(stdout: &mut BufReader<ChildStdout>, deadline: Instant) -> serde_json::Value {
    let mut content_length: Option<usize> = None;
    loop {
        if Instant::now() > deadline {
            panic!("timed out waiting for an LSP header");
        }
        let mut line = String::new();
        stdout.read_line(&mut line).expect("read header line");
        if line == "\r\n" {
            break;
        }
        if let Some(rest) = line.strip_prefix("Content-Length:") {
            content_length = rest.trim().parse().ok();
        }
    }
    let len = content_length.expect("Content-Length header");
    let mut buf = vec![0u8; len];
    stdout.read_exact(&mut buf).expect("read body");
    serde_json::from_slice(&buf).expect("parse JSON-RPC body")
}

fn kill_child(child: &mut Child) {
    let _ = child.kill();
    let _ = child.wait();
}

#[test]
fn responds_to_initialize_handshake() {
    let mut child = ush_lsp()
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .expect("spawn ush_lsp");

    let mut stdin = child.stdin.take().expect("stdin");
    let stdout = child.stdout.take().expect("stdout");
    let mut reader = BufReader::new(stdout);
    let deadline = Instant::now() + Duration::from_secs(5);

    let init = r#"{
      "jsonrpc":"2.0",
      "id":1,
      "method":"initialize",
      "params":{
        "processId": null,
        "rootUri": null,
        "capabilities": {}
      }
    }"#;
    send(&mut stdin, init);

    let response = recv(&mut reader, deadline);
    assert_eq!(response["jsonrpc"], "2.0", "got: {response}");
    assert_eq!(response["id"], 1, "got: {response}");
    assert!(
        response.get("result").is_some(),
        "expected an `initialize` result, got: {response}"
    );

    let initialized = r#"{"jsonrpc":"2.0","method":"initialized","params":{}}"#;
    send(&mut stdin, initialized);

    let shutdown = r#"{"jsonrpc":"2.0","id":2,"method":"shutdown"}"#;
    send(&mut stdin, shutdown);
    let shutdown_response = recv(&mut reader, deadline);
    assert_eq!(shutdown_response["id"], 2);

    let exit = r#"{"jsonrpc":"2.0","method":"exit"}"#;
    send(&mut stdin, exit);
    drop(stdin);

    // The server should exit cleanly after `exit`.
    let status = match child.wait() {
        Ok(status) => status,
        Err(err) => {
            kill_child(&mut child);
            panic!("wait failed: {err}");
        }
    };
    assert!(
        status.code().is_some(),
        "ush_lsp must exit with a real status code after `exit`, got {status:?}",
    );
}
