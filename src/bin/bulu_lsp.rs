use bulu::lsp::run_lsp_server;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Run the LSP server
    run_lsp_server().await
}
