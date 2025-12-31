//! CLI argument parsing

use clap::{Parser, Subcommand};
use std::path::PathBuf;

/// neuro-bitnet - A Rust-based RAG system
#[derive(Parser, Debug)]
#[command(name = "neuro")]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
pub struct Cli {
    /// Enable verbose output
    #[arg(short, long, global = true)]
    pub verbose: bool,

    /// Configuration file path
    #[arg(short, long, global = true)]
    pub config: Option<PathBuf>,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Start the HTTP API server
    Serve {
        /// Host address to bind to
        #[arg(short = 'H', long, default_value = "0.0.0.0")]
        host: String,

        /// Port to listen on
        #[arg(short, long, default_value = "8080")]
        port: u16,

        /// Storage directory for persistence
        #[arg(short, long)]
        storage: Option<PathBuf>,

        /// Embedding model to use
        #[arg(short, long, default_value = "minilm")]
        model: String,
    },

    /// Index files or directories
    Index {
        /// Path(s) to index
        #[arg(required = true)]
        paths: Vec<PathBuf>,

        /// Index recursively
        #[arg(short, long)]
        recursive: bool,

        /// File patterns to include (glob)
        #[arg(short, long)]
        include: Option<Vec<String>>,

        /// File patterns to exclude (glob)
        #[arg(short, long)]
        exclude: Option<Vec<String>>,

        /// Maximum file size in KB
        #[arg(long, default_value = "1024")]
        max_size: usize,

        /// Storage directory for persistence
        #[arg(short, long)]
        storage: Option<PathBuf>,

        /// Embedding model to use
        #[arg(short, long, default_value = "minilm")]
        model: String,

        /// Show progress bar
        #[arg(long, default_value = "true")]
        progress: bool,
    },

    /// Execute a query against the RAG system
    Query {
        /// The query to execute
        query: String,

        /// Number of results to return
        #[arg(short, long, default_value = "5")]
        top_k: usize,

        /// Storage directory
        #[arg(short, long)]
        storage: Option<PathBuf>,

        /// Embedding model to use
        #[arg(short, long, default_value = "minilm")]
        model: String,

        /// Output format (text, json)
        #[arg(short, long, default_value = "text")]
        format: String,

        /// Include web search if needed
        #[arg(short, long)]
        web: bool,
    },

    /// Show storage statistics
    Stats {
        /// Storage directory
        #[arg(short, long)]
        storage: Option<PathBuf>,
    },

    /// Generate embeddings for text
    Embed {
        /// Text to embed
        text: String,

        /// Embedding model to use
        #[arg(short, long, default_value = "minilm")]
        model: String,

        /// Output format (text, json)
        #[arg(short, long, default_value = "text")]
        format: String,
    },

    /// Classify a query
    Classify {
        /// The query to classify
        query: String,

        /// Output format (text, json)
        #[arg(short, long, default_value = "text")]
        format: String,
    },

    /// Search the web
    Search {
        /// Search query
        query: String,

        /// Number of results
        #[arg(short, long, default_value = "5")]
        count: usize,

        /// Output format (text, json)
        #[arg(short, long, default_value = "text")]
        format: String,
    },

    /// Ask a question to the LLM (local BitNet inference or remote server)
    Ask {
        /// The question to ask
        question: String,

        /// Path to local GGUF model file (enables local inference)
        #[arg(short = 'm', long)]
        model_path: Option<PathBuf>,

        /// BitNet model to use (2b, large, 3b, 8b) - auto-downloads if needed
        #[arg(long, default_value = "2b")]
        model: String,

        /// LLM server URL (used if no local model specified)
        #[arg(short, long, default_value = "http://localhost:11435")]
        llm_url: String,

        /// Maximum tokens to generate
        #[arg(long, default_value = "512")]
        max_tokens: u32,

        /// Temperature (0.0 = deterministic, 1.0 = creative)
        #[arg(short, long, default_value = "0.7")]
        temperature: f32,

        /// Context size for local model
        #[arg(long, default_value = "2048")]
        ctx_size: u32,

        /// Number of CPU threads (default: auto-detect)
        #[arg(long)]
        threads: Option<i32>,

        /// Use RAG context from storage
        #[arg(short, long)]
        storage: Option<PathBuf>,

        /// Include web search context
        #[arg(short, long)]
        web: bool,

        /// Output format (text, json)
        #[arg(short, long, default_value = "text")]
        format: String,

        /// Show timing information
        #[arg(long)]
        timing: bool,

        /// Stream output (local inference only)
        #[arg(long)]
        stream: bool,

        /// Skip download confirmation (auto-download model)
        #[arg(short = 'y', long)]
        yes: bool,

        /// Force download even if model exists
        #[arg(long)]
        force_download: bool,
    },

    /// Manage BitNet models (list, download, remove)
    Model {
        #[command(subcommand)]
        action: ModelAction,
    },
}

/// Model management subcommands
#[derive(Subcommand, Debug)]
pub enum ModelAction {
    /// List available and downloaded models
    List,

    /// Download a model
    Download {
        /// Model to download (2b, large, 3b, 8b)
        #[arg(default_value = "2b")]
        model: String,

        /// Force re-download
        #[arg(short, long)]
        force: bool,
    },

    /// Remove a downloaded model
    Remove {
        /// Model to remove (2b, large, 3b, 8b)
        model: String,
    },

    /// Show model cache info
    Info,
}

impl Cli {
    /// Parse command line arguments
    pub fn parse_args() -> Self {
        Self::parse()
    }
}
