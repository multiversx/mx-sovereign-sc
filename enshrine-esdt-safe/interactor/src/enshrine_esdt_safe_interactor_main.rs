use enshrine_esdt_safe_interactor::enshrine_esdt_safe_cli;
use multiversx_sc_snippets::imports::*;

#[tokio::main]
async fn main() {
    enshrine_esdt_safe_cli().await;
}
