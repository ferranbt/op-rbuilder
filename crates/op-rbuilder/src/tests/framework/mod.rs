mod apis;
//mod blocks;
mod chain;
//mod harness;
mod instance;
mod op;
mod service;
mod txs;
mod utils;

pub use apis::*;
//pub use blocks::*;
pub use chain::*;

//pub use harness::*;
pub use instance::*;
pub use op::*;
pub use service::*;
pub use txs::*;
pub use utils::*;

const BUILDER_PRIVATE_KEY: &str =
    "0x59c6995e998f97a5a0044966f0945389dc9e86dae88c7a8412f4603b6b78690d";

const FUNDED_PRIVATE_KEYS: &[&str] =
    &["0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80"];

pub const DEFAULT_JWT_TOKEN: &str =
    "688f5d737bad920bdfb2fc2f488d6b6209eebda1dae949a8de91398d932c517a";

pub const ONE_ETH: u128 = 1_000_000_000_000_000_000;

/// This gets invoked before any tests, when the cargo test framework loads the test library.
/// It injects itself into
#[ctor::ctor]
fn init_tests_logging() {
    if let Ok(v) = std::env::var("TEST_TRACE") {
        let level = match v.as_str() {
            "false" | "off" => return,
            "true" | "debug" | "on" => tracing::Level::DEBUG,
            "trace" => tracing::Level::TRACE,
            "info" => tracing::Level::INFO,
            "warn" => tracing::Level::WARN,
            "error" => tracing::Level::ERROR,
            _ => return,
        };

        tracing_subscriber::fmt()
            .with_max_level(level)
            .with_test_writer()
            .init();
    }
}
