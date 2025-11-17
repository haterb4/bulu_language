use tower_lsp::{LspService, Server};
use super::backend::BuluLanguageServer;

/// Run the LSP server
pub async fn run_lsp_server() -> anyhow::Result<()> {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::new(|client| BuluLanguageServer::new(client));
    
    Server::new(stdin, stdout, socket).serve(service).await;

    Ok(())
}
