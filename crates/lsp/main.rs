use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};

#[derive(Debug)]
struct LoiServer {
    client: Client,
}

#[tower_lsp::async_trait]
impl LanguageServer for LoiServer {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult, ResponseError> {
        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL,
                )),
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                completion_provider: Some(CompletionOptions::default()),
                ..Default::default()
            },
            ..Default::default()
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        self.client
            .log_message(MessageType::INFO, "Loi LSP ready")
            .await;
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let text = params.text_document.text;

        let diagnostics = vec![];

        self.client
            .publish_diagnostics(params.text_document.uri, diagnostics, None)
            .await;
    }

    async fn hover(&self, _: HoverParams) -> Result<Option<Hover>, ResponseError> {
        Ok(Some(Hover {
            contents: HoverContents::Scalar(MarkedString::String("Loi hover".into())),
            range: None,
        }))
    }

    async fn completion(
        &self,
        _: CompletionParams,
    ) -> Result<Option<CompletionResponse>, ResponseError> {
        Ok(Some(CompletionResponse::Array(vec![
            CompletionItem::new_simple("print".into(), "Built-in print".into()),
        ])))
    }

    async fn shutdown(&self) -> Result<(), ResponseError> {
        Ok(())
    }
}

#[tokio::main]
async fn main() {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::new(|client| LoiServer { client });

    Server::new(stdin, stdout, socket).serve(service).await;
}
