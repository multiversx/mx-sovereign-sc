use multiversx_sc_snippets::imports::*;
use enshrine_esdt_safe_interactor::enshrine_esdt_safe_cli;

#[tokio::main]
async fn main() {
    enshrine_esdt_safe_cli().await;
}  