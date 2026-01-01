//! Neuro MCP - Model Context Protocol Server
//!
//! Provides MCP interface for IDE integration (VS Code, etc.)

use clap::Parser;
use std::path::PathBuf;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use neuro_mcp::McpServer;

#[derive(Parser, Debug)]
#[command(name = "neuro-mcp")]
#[command(author, version, about = "MCP server for BitNet inference")]
struct Args {
    /// Path to the model
    #[arg(short, long, env = "NEURO_MODEL_PATH")]
    model: Option<PathBuf>,

    /// Enable debug logging (writes to stderr)
    #[arg(short, long)]
    debug: bool,
}

fn find_model() -> Option<PathBuf> {
    let candidates = [
        // Environment variable
        std::env::var("NEURO_MODEL_PATH").ok().map(PathBuf::from),
        // Common locations
        Some(PathBuf::from("/opt/bitnet/models/BitNet-b1.58-2B-4T-gguf/ggml-model-i2_s.gguf")),
        Some(dirs::home_dir()?.join(".local/share/bitnet/models/ggml-model-i2_s.gguf")),
        Some(PathBuf::from("./models/ggml-model-i2_s.gguf")),
    ];

    for candidate in candidates.into_iter().flatten() {
        if candidate.exists() {
            return Some(candidate);
        }
    }
    None
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    // Initialize tracing (to stderr, since stdout is used for MCP)
    if args.debug {
        tracing_subscriber::registry()
            .with(
                tracing_subscriber::EnvFilter::try_from_default_env()
                    .unwrap_or_else(|_| "neuro_mcp=debug".into()),
            )
            .with(tracing_subscriber::fmt::layer().with_writer(std::io::stderr))
            .init();
    }

    // Find model path
    let model_path = args
        .model
        .or_else(find_model)
        .ok_or_else(|| anyhow::anyhow!(
            "No model found. Set NEURO_MODEL_PATH or use --model"
        ))?;

    if !model_path.exists() {
        anyhow::bail!("Model not found: {}", model_path.display());
    }

    let server = McpServer::new(model_path.to_string_lossy().to_string());
    server.run().await?;

    Ok(())
}
