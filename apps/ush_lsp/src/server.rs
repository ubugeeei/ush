use anyhow::Result;
use lsp_server::{Connection, Message, Notification, Request, Response};
use lsp_types::{
    DidChangeTextDocumentParams, DidOpenTextDocumentParams, DidSaveTextDocumentParams,
    DocumentFormattingParams, OneOf, PublishDiagnosticsParams, SemanticTokenType,
    SemanticTokensFullOptions, SemanticTokensLegend, SemanticTokensOptions, SemanticTokensParams,
    SemanticTokensServerCapabilities, ServerCapabilities, TextDocumentSyncCapability,
    TextDocumentSyncKind,
    notification::{
        DidChangeTextDocument, DidOpenTextDocument, DidSaveTextDocument, Notification as _,
    },
    request::{Formatting, Request as _, SemanticTokensFullRequest},
};
use ush_tooling::{check_source, format_source, semantic_token_legend, semantic_tokens};

use crate::{convert, document::DocumentStore};

pub fn run(connection: Connection) -> Result<()> {
    connection.initialize(serde_json::to_value(capabilities())?)?;
    let mut docs = DocumentStore::default();

    for message in &connection.receiver {
        match message {
            Message::Request(request) => {
                if connection.handle_shutdown(&request)? {
                    return Ok(());
                }
                handle_request(&connection, &mut docs, request.clone())?;
            }
            Message::Notification(notification) => {
                handle_notification(&connection, &mut docs, notification.clone())?;
            }
            Message::Response(_) => {}
        }
    }

    Ok(())
}

fn capabilities() -> ServerCapabilities {
    ServerCapabilities {
        text_document_sync: Some(TextDocumentSyncCapability::Kind(TextDocumentSyncKind::FULL)),
        document_formatting_provider: Some(OneOf::Left(true)),
        semantic_tokens_provider: Some(SemanticTokensServerCapabilities::SemanticTokensOptions(
            SemanticTokensOptions {
                legend: SemanticTokensLegend {
                    token_types: semantic_token_legend()
                        .iter()
                        .map(|name| SemanticTokenType::new(*name))
                        .collect(),
                    token_modifiers: Vec::new(),
                },
                full: Some(SemanticTokensFullOptions::Bool(true)),
                range: None,
                ..SemanticTokensOptions::default()
            },
        )),
        ..ServerCapabilities::default()
    }
}

fn handle_request(
    connection: &Connection,
    docs: &mut DocumentStore,
    request: Request,
) -> Result<()> {
    match request.method.as_str() {
        Formatting::METHOD => formatting(connection, docs, request),
        SemanticTokensFullRequest::METHOD => semantic_full(connection, docs, request),
        _ => {
            connection.sender.send(Message::Response(Response::new_err(
                request.id,
                -32601,
                "method not supported".to_string(),
            )))?;
            Ok(())
        }
    }
}

fn handle_notification(
    connection: &Connection,
    docs: &mut DocumentStore,
    notification: Notification,
) -> Result<()> {
    match notification.method.as_str() {
        DidOpenTextDocument::METHOD => {
            let params: DidOpenTextDocumentParams = serde_json::from_value(notification.params)?;
            let uri = params.text_document.uri;
            let text = params.text_document.text;
            docs.open(uri.clone(), text.clone());
            publish_diagnostics(connection, &uri, &text)
        }
        DidChangeTextDocument::METHOD => {
            let params: DidChangeTextDocumentParams = serde_json::from_value(notification.params)?;
            let uri = params.text_document.uri;
            if let Some(change) = params.content_changes.into_iter().last() {
                docs.update(&uri, change.text.clone());
                return publish_diagnostics(connection, &uri, &change.text);
            }
            Ok(())
        }
        DidSaveTextDocument::METHOD => {
            let params: DidSaveTextDocumentParams = serde_json::from_value(notification.params)?;
            let uri = params.text_document.uri;
            let text = docs.read(&uri)?;
            publish_diagnostics(connection, &uri, &text)
        }
        _ => Ok(()),
    }
}

fn formatting(connection: &Connection, docs: &mut DocumentStore, request: Request) -> Result<()> {
    let params: DocumentFormattingParams = serde_json::from_value(request.params)?;
    let source = docs.read(&params.text_document.uri)?;
    let formatted = format_source(&source);
    let edits = if source == formatted {
        Vec::new()
    } else {
        vec![convert::full_document_edit(&source, &formatted)]
    };
    respond_ok(connection, request.id, serde_json::to_value(edits)?)
}

fn semantic_full(
    connection: &Connection,
    docs: &mut DocumentStore,
    request: Request,
) -> Result<()> {
    let params: SemanticTokensParams = serde_json::from_value(request.params)?;
    let source = docs.read(&params.text_document.uri)?;
    let tokens = convert::semantic_tokens(&semantic_tokens(&source));
    respond_ok(connection, request.id, serde_json::to_value(tokens)?)
}

fn publish_diagnostics(connection: &Connection, uri: &lsp_types::Uri, source: &str) -> Result<()> {
    let params = PublishDiagnosticsParams {
        uri: uri.clone(),
        diagnostics: convert::diagnostics(source, &check_source(source)),
        version: None,
    };
    connection
        .sender
        .send(Message::Notification(Notification::new(
            "textDocument/publishDiagnostics".to_string(),
            params,
        )))?;
    Ok(())
}

fn respond_ok(
    connection: &Connection,
    id: lsp_server::RequestId,
    value: serde_json::Value,
) -> Result<()> {
    connection
        .sender
        .send(Message::Response(Response::new_ok(id, value)))?;
    Ok(())
}
