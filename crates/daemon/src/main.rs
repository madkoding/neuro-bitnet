//! Neuro BitNet Daemon
//!
//! Background service for BitNet inference with HTTP API

use clap::Parser;
use std::path::PathBuf;
use tokio::signal;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use neuro_daemon::{DaemonConfig, DaemonServer};

#[derive(Parser, Debug)]
#[command(name = "neuro-daemon")]
#[command(author, version, about = "Neuro BitNet inference daemon")]
struct Args {
    /// Host to bind to
    #[arg(short = 'H', long, default_value = "127.0.0.1")]
    host: String,

    /// Port to listen on
    #[arg(short, long, default_value = "11435")]
    port: u16,

    /// Path to the model
    #[arg(short, long, env = "NEURO_MODEL_PATH")]
    model: Option<PathBuf>,

    /// Auto-translate non-English queries to English
    #[arg(short = 't', long, default_value = "true")]
    auto_translate: bool,

    /// Maximum tokens to generate
    #[arg(long, default_value = "512")]
    max_tokens: u32,

    /// Temperature for generation
    #[arg(long, default_value = "0.7")]
    temperature: f32,

    /// Run in foreground (don't daemonize)
    #[arg(short, long)]
    foreground: bool,

    /// PID file path
    #[arg(long)]
    pid_file: Option<PathBuf>,

    /// Log file path
    #[arg(long)]
    log_file: Option<PathBuf>,
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
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "neuro_daemon=info,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let args = Args::parse();

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

    let config = DaemonConfig {
        host: args.host,
        port: args.port,
        model_path: model_path.to_string_lossy().to_string(),
        auto_translate: args.auto_translate,
        max_tokens: args.max_tokens,
        temperature: args.temperature,
    };

    if args.foreground {
        // Run in foreground
        let server = DaemonServer::new(config);
        server.run_with_shutdown(shutdown_signal()).await?;
    } else {
        // Daemonize
        #[cfg(unix)]
        {
            use daemonize::Daemonize;

            let mut daemon = Daemonize::new();

            if let Some(pid_file) = args.pid_file {
                daemon = daemon.pid_file(pid_file);
            }

            // Note: For proper daemonization with logging, you'd need more setup
            daemon.start()?;

            let server = DaemonServer::new(config);
            server.run_with_shutdown(shutdown_signal()).await?;
        }

        #[cfg(not(unix))]
        {
            // On non-Unix, just run in foreground
            let server = DaemonServer::new(config);
            server.run_with_shutdown(shutdown_signal()).await?;
        }
    }

    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("Failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("Failed to install SIGTERM handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {
            tracing::info!("Received Ctrl+C, shutting down...");
        }
        _ = terminate => {
            tracing::info!("Received SIGTERM, shutting down...");
        }
    }
}
