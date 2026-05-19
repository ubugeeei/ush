//! End-to-end LSP wire test for the new
//! `textDocument/definition`, `textDocument/references`, and
//! `textDocument/rename` handlers.

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
fn definition_references_and_rename_round_trip() {
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

    let uri = "file:///tmp/refs.ush";
    let text = "let value = 1\nprint value\nprint value\n";
    open_document(&mut stdin, uri, text);

    // definition
    let def_request = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 2,
        "method": "textDocument/definition",
        "params": {
            "textDocument": { "uri": uri },
            "position": { "line": 1, "character": 6 }
        }
    });
    send(&mut stdin, &def_request.to_string());
    let def = recv_with_id(&mut reader, deadline, 2);
    let location = def["result"].clone();
    assert_eq!(location["range"]["start"]["line"], 0, "{def}");

    // references
    let refs_request = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 3,
        "method": "textDocument/references",
        "params": {
            "textDocument": { "uri": uri },
            "position": { "line": 0, "character": 5 },
            "context": { "includeDeclaration": true }
        }
    });
    send(&mut stdin, &refs_request.to_string());
    let refs = recv_with_id(&mut reader, deadline, 3);
    assert_eq!(
        refs["result"].as_array().map(Vec::len).unwrap_or(0),
        3,
        "expected 3 occurrences of `value`, got: {refs}"
    );

    // rename
    let rename_request = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 4,
        "method": "textDocument/rename",
        "params": {
            "textDocument": { "uri": uri },
            "position": { "line": 0, "character": 5 },
            "newName": "answer"
        }
    });
    send(&mut stdin, &rename_request.to_string());
    let rename = recv_with_id(&mut reader, deadline, 4);
    let edits = rename["result"]["changes"][uri]
        .as_array()
        .expect("changes for uri");
    assert_eq!(edits.len(), 3, "{rename}");
    for edit in edits {
        assert_eq!(edit["newText"], "answer");
    }

    // rename with an invalid identifier is rejected
    let bad_rename = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 5,
        "method": "textDocument/rename",
        "params": {
            "textDocument": { "uri": uri },
            "position": { "line": 0, "character": 5 },
            "newName": "1 invalid"
        }
    });
    send(&mut stdin, &bad_rename.to_string());
    let response = recv_with_id(&mut reader, deadline, 5);
    assert!(response.get("error").is_some(), "{response}");

    send(&mut stdin, r#"{"jsonrpc":"2.0","method":"exit"}"#);
    drop(stdin);
    let _ = child.wait();
    kill_child(&mut child);
}
