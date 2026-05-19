use anyhow::Result;
use lsp_server::{Connection, Message, Notification, Request, Response};
use lsp_types::{
    CompletionOptions, CompletionParams, DidChangeTextDocumentParams, DidOpenTextDocumentParams,
    DidSaveTextDocumentParams, DocumentFormattingParams, DocumentHighlightParams,
    DocumentSymbolParams, DocumentSymbolResponse, FoldingRangeParams,
    FoldingRangeProviderCapability, GotoDefinitionParams, GotoDefinitionResponse, HoverParams,
    HoverProviderCapability, Location, OneOf, PublishDiagnosticsParams, ReferenceParams,
    RenameOptions, RenameParams, SemanticTokenType, SemanticTokensFullOptions,
    SemanticTokensLegend, SemanticTokensOptions, SemanticTokensParams,
    SemanticTokensServerCapabilities, ServerCapabilities, TextDocumentSyncCapability,
    TextDocumentSyncKind, WorkDoneProgressOptions,
    notification::{
        DidChangeTextDocument, DidOpenTextDocument, DidSaveTextDocument, Notification as _,
    },
    request::{
        Completion, DocumentHighlightRequest, DocumentSymbolRequest, FoldingRangeRequest,
        Formatting, GotoDefinition, HoverRequest, PrepareRenameRequest, References, Rename,
        Request as _, SemanticTokensFullRequest,
    },
};
use ush_tooling::{
    check_source, completions, definition as ush_definition, document_highlights, document_symbols,
    folding_ranges, format_source, hover as ush_hover, prepare_rename as ush_prepare_rename,
    references as ush_refs, rename_locations, semantic_token_legend, semantic_tokens,
};

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
        document_highlight_provider: Some(OneOf::Left(true)),
        document_symbol_provider: Some(OneOf::Left(true)),
        folding_range_provider: Some(FoldingRangeProviderCapability::Simple(true)),
        hover_provider: Some(HoverProviderCapability::Simple(true)),
        definition_provider: Some(OneOf::Left(true)),
        references_provider: Some(OneOf::Left(true)),
        rename_provider: Some(OneOf::Right(RenameOptions {
            prepare_provider: Some(true),
            work_done_progress_options: WorkDoneProgressOptions::default(),
        })),
        completion_provider: Some(CompletionOptions {
            resolve_provider: Some(false),
            trigger_characters: None,
            ..CompletionOptions::default()
        }),
        semantic_tokens_provider: Some(SemanticTokensServerCapabilities::SemanticTokensOptions(
            SemanticTokensOptions {
                legend: SemanticTokensLegend {
                    token_types: semantic_token_legend()
                        .iter()
                        .map(|name| SemanticTokenType::new(name))
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
        DocumentHighlightRequest::METHOD => highlight(connection, docs, request),
        DocumentSymbolRequest::METHOD => symbols(connection, docs, request),
        FoldingRangeRequest::METHOD => folding(connection, docs, request),
        Completion::METHOD => completion(connection, docs, request),
        HoverRequest::METHOD => hover(connection, docs, request),
        GotoDefinition::METHOD => definition(connection, docs, request),
        References::METHOD => references(connection, docs, request),
        Rename::METHOD => rename(connection, docs, request),
        PrepareRenameRequest::METHOD => prepare_rename(connection, docs, request),
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

fn highlight(connection: &Connection, docs: &mut DocumentStore, request: Request) -> Result<()> {
    let params: DocumentHighlightParams = serde_json::from_value(request.params)?;
    let source = docs.read(&params.text_document_position_params.text_document.uri)?;
    let position = params.text_document_position_params.position;
    let highlights = convert::document_highlights(&document_highlights(
        &source,
        position.line,
        position.character,
    ));
    respond_ok(connection, request.id, serde_json::to_value(highlights)?)
}

fn symbols(connection: &Connection, docs: &mut DocumentStore, request: Request) -> Result<()> {
    let params: DocumentSymbolParams = serde_json::from_value(request.params)?;
    let source = docs.read(&params.text_document.uri)?;
    let response =
        DocumentSymbolResponse::Nested(convert::document_symbols(&document_symbols(&source)));
    respond_ok(connection, request.id, serde_json::to_value(response)?)
}

fn folding(connection: &Connection, docs: &mut DocumentStore, request: Request) -> Result<()> {
    let params: FoldingRangeParams = serde_json::from_value(request.params)?;
    let source = docs.read(&params.text_document.uri)?;
    let ranges = convert::folding_ranges(&folding_ranges(&source));
    respond_ok(connection, request.id, serde_json::to_value(ranges)?)
}

fn completion(connection: &Connection, docs: &mut DocumentStore, request: Request) -> Result<()> {
    let params: CompletionParams = serde_json::from_value(request.params)?;
    let source = docs.read(&params.text_document_position.text_document.uri)?;
    let items = convert::completion_items(&completions(&source));
    respond_ok(connection, request.id, serde_json::to_value(items)?)
}

fn hover(connection: &Connection, docs: &mut DocumentStore, request: Request) -> Result<()> {
    let params: HoverParams = serde_json::from_value(request.params)?;
    let source = docs.read(&params.text_document_position_params.text_document.uri)?;
    let position = params.text_document_position_params.position;
    let result = ush_hover(&source, position.line, position.character).map(convert::hover);
    respond_ok(connection, request.id, serde_json::to_value(result)?)
}

fn definition(connection: &Connection, docs: &mut DocumentStore, request: Request) -> Result<()> {
    let params: GotoDefinitionParams = serde_json::from_value(request.params)?;
    let uri = params
        .text_document_position_params
        .text_document
        .uri
        .clone();
    let source = docs.read(&uri)?;
    let position = params.text_document_position_params.position;
    let result = ush_definition(&source, position.line, position.character).map(|reference| {
        GotoDefinitionResponse::Scalar(Location {
            uri: uri.clone(),
            range: convert::range_of_reference(&reference),
        })
    });
    respond_ok(connection, request.id, serde_json::to_value(result)?)
}

fn references(connection: &Connection, docs: &mut DocumentStore, request: Request) -> Result<()> {
    let params: ReferenceParams = serde_json::from_value(request.params)?;
    let uri = params.text_document_position.text_document.uri.clone();
    let source = docs.read(&uri)?;
    let position = params.text_document_position.position;
    let locations: Vec<Location> = ush_refs(&source, position.line, position.character)
        .into_iter()
        .map(|reference| Location {
            uri: uri.clone(),
            range: convert::range_of_reference(&reference),
        })
        .collect();
    respond_ok(connection, request.id, serde_json::to_value(locations)?)
}

fn prepare_rename(
    connection: &Connection,
    docs: &mut DocumentStore,
    request: Request,
) -> Result<()> {
    let params: lsp_types::TextDocumentPositionParams = serde_json::from_value(request.params)?;
    let source = docs.read(&params.text_document.uri)?;
    let result = ush_prepare_rename(&source, params.position.line, params.position.character)
        .map(|reference| convert::range_of_reference(&reference));
    respond_ok(connection, request.id, serde_json::to_value(result)?)
}

fn rename(connection: &Connection, docs: &mut DocumentStore, request: Request) -> Result<()> {
    let params: RenameParams = serde_json::from_value(request.params)?;
    let uri = params.text_document_position.text_document.uri.clone();
    let source = docs.read(&uri)?;
    let position = params.text_document_position.position;
    match rename_locations(&source, position.line, position.character, &params.new_name) {
        Ok(locations) => {
            let edit = convert::rename_workspace_edit(&uri, &locations, &params.new_name);
            respond_ok(connection, request.id, serde_json::to_value(edit)?)
        }
        Err(_) => {
            connection.sender.send(Message::Response(Response::new_err(
                request.id,
                -32602,
                format!("`{}` is not a valid .ush identifier", params.new_name),
            )))?;
            Ok(())
        }
    }
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
