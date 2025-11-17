use dashmap::DashMap;
use std::sync::Arc;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer};

use crate::lexer::Lexer;
use crate::parser::Parser;
use crate::linter::Linter;
use crate::types::checker::TypeChecker;

use super::completion::CompletionProvider;
use super::diagnostics::DiagnosticsProvider;
use super::hover::HoverProvider;
use super::navigation::NavigationProvider;
use super::refactor::RefactorProvider;

/// Document state stored in memory
#[derive(Debug, Clone)]
pub struct DocumentState {
    pub uri: Url,
    pub text: String,
    pub version: i32,
}

/// Main LSP backend for Bulu language
pub struct BuluLanguageServer {
    client: Client,
    documents: Arc<DashMap<String, DocumentState>>,
    completion_provider: CompletionProvider,
    diagnostics_provider: DiagnosticsProvider,
    hover_provider: HoverProvider,
    navigation_provider: NavigationProvider,
    refactor_provider: RefactorProvider,
}

impl BuluLanguageServer {
    pub fn new(client: Client) -> Self {
        let documents = Arc::new(DashMap::new());
        
        Self {
            client,
            documents: documents.clone(),
            completion_provider: CompletionProvider::new(documents.clone()),
            diagnostics_provider: DiagnosticsProvider::new(documents.clone()),
            hover_provider: HoverProvider::new(documents.clone()),
            navigation_provider: NavigationProvider::new(documents.clone()),
            refactor_provider: RefactorProvider::new(documents.clone()),
        }
    }

    /// Parse document and return diagnostics
    async fn analyze_document(&self, uri: &Url, text: &str) -> Vec<Diagnostic> {
        self.diagnostics_provider.analyze(uri, text).await
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for BuluLanguageServer {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL,
                )),
                completion_provider: Some(CompletionOptions {
                    resolve_provider: Some(true),
                    trigger_characters: Some(vec![
                        ".".to_string(),
                        ":".to_string(),
                        "<".to_string(),
                    ]),
                    ..Default::default()
                }),
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                definition_provider: Some(OneOf::Left(true)),
                references_provider: Some(OneOf::Left(true)),
                rename_provider: Some(OneOf::Left(true)),
                code_action_provider: Some(CodeActionProviderCapability::Simple(true)),
                document_symbol_provider: Some(OneOf::Left(true)),
                workspace_symbol_provider: Some(OneOf::Left(true)),
                signature_help_provider: Some(SignatureHelpOptions {
                    trigger_characters: Some(vec!["(".to_string(), ",".to_string()]),
                    retrigger_characters: None,
                    work_done_progress_options: WorkDoneProgressOptions::default(),
                }),
                ..Default::default()
            },
            server_info: Some(ServerInfo {
                name: "Bulu Language Server".to_string(),
                version: Some(env!("CARGO_PKG_VERSION").to_string()),
            }),
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        self.client
            .log_message(MessageType::INFO, "Bulu Language Server initialized")
            .await;
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let uri = params.text_document.uri.to_string();
        let text = params.text_document.text.clone();
        let version = params.text_document.version;

        // Store document
        self.documents.insert(
            uri.clone(),
            DocumentState {
                uri: params.text_document.uri.clone(),
                text: text.clone(),
                version,
            },
        );

        // Analyze and send diagnostics
        let diagnostics = self.analyze_document(&params.text_document.uri, &text).await;
        self.client
            .publish_diagnostics(params.text_document.uri, diagnostics, Some(version))
            .await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let uri = params.text_document.uri.to_string();
        
        if let Some(change) = params.content_changes.first() {
            let text = change.text.clone();
            let version = params.text_document.version;

            // Update document
            self.documents.insert(
                uri.clone(),
                DocumentState {
                    uri: params.text_document.uri.clone(),
                    text: text.clone(),
                    version,
                },
            );

            // Analyze and send diagnostics
            let diagnostics = self.analyze_document(&params.text_document.uri, &text).await;
            self.client
                .publish_diagnostics(params.text_document.uri, diagnostics, Some(version))
                .await;
        }
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        let uri = params.text_document.uri.to_string();
        self.documents.remove(&uri);
    }

    async fn completion(&self, params: CompletionParams) -> Result<Option<CompletionResponse>> {
        self.completion_provider.provide_completion(params).await
    }

    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        self.hover_provider.provide_hover(params).await
    }

    async fn goto_definition(
        &self,
        params: GotoDefinitionParams,
    ) -> Result<Option<GotoDefinitionResponse>> {
        self.navigation_provider.goto_definition(params).await
    }

    async fn references(&self, params: ReferenceParams) -> Result<Option<Vec<Location>>> {
        self.navigation_provider.find_references(params).await
    }

    async fn rename(&self, params: RenameParams) -> Result<Option<WorkspaceEdit>> {
        self.refactor_provider.rename(params).await
    }

    async fn code_action(&self, params: CodeActionParams) -> Result<Option<CodeActionResponse>> {
        self.refactor_provider.code_actions(params).await
    }

    async fn signature_help(&self, params: SignatureHelpParams) -> Result<Option<SignatureHelp>> {
        self.hover_provider.signature_help(params).await
    }

    async fn document_symbol(
        &self,
        params: DocumentSymbolParams,
    ) -> Result<Option<DocumentSymbolResponse>> {
        self.navigation_provider.document_symbols(params).await
    }
}
