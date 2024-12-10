use esdt_safe_interactor::esdt_safe_cli;
use multiversx_sc_snippets::imports::*;

#[tokio::main]
async fn main() {
    esdt_safe_cli().await;
}
