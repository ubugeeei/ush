//! End-to-end LSP wire test for `textDocument/documentHighlight`
//! and `textDocument/documentSymbol`.
//!
//! Drives the real `ush_lsp` binary over stdio, opens a tiny `.ush`
//! document, asks both methods, and asserts the responses contain
//! the expected shapes.

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

fn recv_with_id(
    reader: &mut BufReader<ChildStdout>,
    deadline: Instant,
    wanted_id: u64,
) -> serde_json::Value {
    loop {
        let response = recv(reader, deadline);
        if response.get("id").and_then(|v| v.as_u64()) == Some(wanted_id) {
            return response;
        }
    }
}

fn recv(reader: &mut BufReader<ChildStdout>, deadline: Instant) -> serde_json::Value {
    let mut content_length: Option<usize> = None;
    loop {
        if Instant::now() > deadline {
            panic!("timed out waiting for an LSP header");
        }
        let mut line = String::new();
        reader.read_line(&mut line).expect("read header line");
        if line == "\r\n" {
            break;
        }
        if let Some(rest) = line.strip_prefix("Content-Length:") {
            content_length = rest.trim().parse().ok();
        }
    }
    let len = content_length.expect("Content-Length header");
    let mut buf = vec![0u8; len];
    reader.read_exact(&mut buf).expect("read body");
    serde_json::from_slice(&buf).expect("parse JSON-RPC body")
}

fn kill_child(child: &mut Child) {
    let _ = child.kill();
    let _ = child.wait();
}

fn shake_hands(stdin: &mut ChildStdin, reader: &mut BufReader<ChildStdout>, deadline: Instant) {
    send(
        stdin,
        r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"processId":null,"rootUri":null,"capabilities":{}}}"#,
    );
    let init = recv_with_id(reader, deadline, 1);
    assert!(init.get("result").is_some(), "initialize: {init}");

    send(
        stdin,
        r#"{"jsonrpc":"2.0","method":"initialized","params":{}}"#,
    );
}

fn open_document(stdin: &mut ChildStdin, uri: &str, text: &str) {
    let payload = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "textDocument/didOpen",
        "params": {
            "textDocument": {
                "uri": uri,
                "languageId": "ush",
                "version": 1,
                "text": text,
            }
        }
    });
    send(stdin, &payload.to_string());
}

#[test]
fn document_highlight_returns_every_occurrence_of_a_variable() {
    let mut child = ush_lsp()
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .expect("spawn ush_lsp");
    let mut stdin = child.stdin.take().expect("stdin");
    let mut reader = BufReader::new(child.stdout.take().expect("stdout"));
    let deadline = Instant::now() + Duration::from_secs(5);

    shake_hands(&mut stdin, &mut reader, deadline);

    let uri = "file:///tmp/highlight.ush";
    let text = "let value = 1\nprint value\nprint value\n";
    open_document(&mut stdin, uri, text);

    let request = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 2,
        "method": "textDocument/documentHighlight",
        "params": {
            "textDocument": { "uri": uri },
            "position": { "line": 0, "character": 4 }
        }
    });
    send(&mut stdin, &request.to_string());
    let response = recv_with_id(&mut reader, deadline, 2);
    let highlights = response["result"]
        .as_array()
        .expect("documentHighlight result is an array");
    assert_eq!(
        highlights.len(),
        3,
        "expected 3 occurrences of `value`, got: {response}",
    );
    for highlight in highlights {
        assert!(highlight.get("range").is_some());
        assert!(highlight.get("kind").is_some());
    }

    send(&mut stdin, r#"{"jsonrpc":"2.0","method":"exit"}"#);
    drop(stdin);
    let _ = child.wait();
    kill_child(&mut child);
}

#[test]
fn document_symbol_lists_top_level_declarations() {
    let mut child = ush_lsp()
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .expect("spawn ush_lsp");
    let mut stdin = child.stdin.take().expect("stdin");
    let mut reader = BufReader::new(child.stdout.take().expect("stdout"));
    let deadline = Instant::now() + Duration::from_secs(5);

    shake_hands(&mut stdin, &mut reader, deadline);

    let uri = "file:///tmp/symbols.ush";
    let text = "fn greet(name: String) -> String {\n  return \"hi\"\n}\n\nenum Colour { Red, Blue }\nlet value = 1\n";
    open_document(&mut stdin, uri, text);

    let request = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 2,
        "method": "textDocument/documentSymbol",
        "params": { "textDocument": { "uri": uri } }
    });
    send(&mut stdin, &request.to_string());
    let response = recv_with_id(&mut reader, deadline, 2);
    let symbols = response["result"]
        .as_array()
        .expect("documentSymbol result is an array");

    let names: Vec<&str> = symbols
        .iter()
        .map(|s| s["name"].as_str().expect("symbol name"))
        .collect();
    assert!(names.contains(&"greet"), "names = {names:?}");
    assert!(names.contains(&"Colour"), "names = {names:?}");
    assert!(names.contains(&"value"), "names = {names:?}");

    send(&mut stdin, r#"{"jsonrpc":"2.0","method":"exit"}"#);
    drop(stdin);
    let _ = child.wait();
    kill_child(&mut child);
}
