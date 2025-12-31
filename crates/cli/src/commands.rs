//! Command implementations

use colored::Colorize;
use indicatif::{ProgressBar, ProgressStyle};
use std::path::PathBuf;
use walkdir::WalkDir;

use neuro_classifier::Classifier;
use neuro_core::QueryResult;
use neuro_embeddings::{Embedder, EmbeddingModel, FastEmbedder};
use neuro_search::{WebSearcher, WikipediaSearcher};
use neuro_server::{Server, ServerConfig};
use neuro_storage::{FileStorage, MemoryStorage, Storage};

// ============================================================================
// Serve command
// ============================================================================

pub async fn serve(
    host: String,
    port: u16,
    storage: Option<PathBuf>,
    model: String,
    verbose: bool,
) -> anyhow::Result<()> {
    init_tracing(verbose);

    let config = ServerConfig {
        host,
        port,
        storage_path: storage,
        embedding_model: model,
        ..ServerConfig::default()
    };

    println!(
        "{} Starting server on {}:{}",
        "▶".green().bold(),
        config.host,
        config.port
    );

    let server = Server::new(config).await?;

    // Setup graceful shutdown
    let shutdown = async {
        tokio::signal::ctrl_c()
            .await
            .expect("Failed to install Ctrl+C handler");
        println!("\n{} Shutting down...", "⏹".yellow().bold());
    };

    server.run_with_shutdown(shutdown).await?;
    Ok(())
}

// ============================================================================
// Index command
// ============================================================================

pub async fn index(
    paths: Vec<PathBuf>,
    recursive: bool,
    include: Option<Vec<String>>,
    exclude: Option<Vec<String>>,
    max_size: usize,
    storage_path: Option<PathBuf>,
    model: String,
    show_progress: bool,
    verbose: bool,
) -> anyhow::Result<()> {
    init_tracing(verbose);

    println!("{} Initializing embedder...", "⚙".cyan().bold());
    let embedding_model: EmbeddingModel = model.parse().unwrap_or(EmbeddingModel::AllMiniLmL6V2);
    let embedder = FastEmbedder::new(embedding_model)?;

    // Initialize storage
    let mut storage: Box<dyn Storage> = if let Some(path) = storage_path {
        println!(
            "{} Using file storage: {}",
            "📁".cyan().bold(),
            path.display()
        );
        Box::new(FileStorage::new(&path).await?)
    } else {
        println!(
            "{} Using in-memory storage (data will not persist)",
            "⚠".yellow().bold()
        );
        Box::new(MemoryStorage::new())
    };

    // Collect files
    println!("{} Collecting files...", "🔍".cyan().bold());
    let mut files: Vec<PathBuf> = Vec::new();

    for path in paths {
        if path.is_file() {
            if should_include_file(&path, &include, &exclude, max_size) {
                files.push(path);
            }
        } else if path.is_dir() {
            let walker = if recursive {
                WalkDir::new(&path)
            } else {
                WalkDir::new(&path).max_depth(1)
            };

            for entry in walker.into_iter().filter_map(|e| e.ok()) {
                let path = entry.path().to_path_buf();
                if path.is_file() && should_include_file(&path, &include, &exclude, max_size) {
                    files.push(path);
                }
            }
        }
    }

    if files.is_empty() {
        println!("{} No files found to index", "⚠".yellow().bold());
        return Ok(());
    }

    println!(
        "{} Found {} files to index",
        "📝".cyan().bold(),
        files.len()
    );

    // Index files
    let progress = if show_progress {
        let pb = ProgressBar::new(files.len() as u64);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta})")?
                .progress_chars("#>-"),
        );
        Some(pb)
    } else {
        None
    };

    let mut indexed = 0;
    let mut errors = 0;

    for file in files {
        if let Some(ref pb) = progress {
            pb.set_message(format!("{}", file.display()));
        }

        match std::fs::read_to_string(&file) {
            Ok(content) => {
                if content.trim().is_empty() {
                    if let Some(ref pb) = progress {
                        pb.inc(1);
                    }
                    continue;
                }

                match embedder.embed_single(&content) {
                    Ok(embedding) => {
                        let mut doc = neuro_core::Document::new(&content)
                            .with_embedding(embedding)
                            .with_source(neuro_core::DocumentSource::File)
                            .with_metadata(
                                "file_path",
                                serde_json::Value::String(file.display().to_string()),
                            );

                        if let Some(name) = file.file_name() {
                            doc = doc.with_metadata(
                                "file_name",
                                serde_json::Value::String(name.to_string_lossy().to_string()),
                            );
                        }

                        match storage.add(doc).await {
                            Ok(_) => indexed += 1,
                            Err(e) => {
                                errors += 1;
                                if verbose {
                                    eprintln!(
                                        "{} Failed to store {}: {}",
                                        "✗".red().bold(),
                                        file.display(),
                                        e
                                    );
                                }
                            }
                        }
                    }
                    Err(e) => {
                        errors += 1;
                        if verbose {
                            eprintln!(
                                "{} Failed to embed {}: {}",
                                "✗".red().bold(),
                                file.display(),
                                e
                            );
                        }
                    }
                }
            }
            Err(e) => {
                errors += 1;
                if verbose {
                    eprintln!(
                        "{} Failed to read {}: {}",
                        "✗".red().bold(),
                        file.display(),
                        e
                    );
                }
            }
        }

        if let Some(ref pb) = progress {
            pb.inc(1);
        }
    }

    if let Some(pb) = progress {
        pb.finish_with_message("Done");
    }

    println!(
        "\n{} Indexed {} files ({} errors)",
        "✓".green().bold(),
        indexed,
        errors
    );

    let stats = storage.stats().await;
    println!(
        "{} Storage: {} documents, {}KB",
        "📊".cyan().bold(),
        stats.document_count,
        stats.total_content_bytes / 1024
    );

    Ok(())
}

fn should_include_file(
    path: &PathBuf,
    include: &Option<Vec<String>>,
    exclude: &Option<Vec<String>>,
    max_size_kb: usize,
) -> bool {
    // Check file size
    if let Ok(metadata) = std::fs::metadata(path) {
        if metadata.len() > (max_size_kb * 1024) as u64 {
            return false;
        }
    }

    let file_name = path
        .file_name()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_default();

    // Check excludes
    if let Some(exclude_patterns) = exclude {
        for pattern in exclude_patterns {
            if file_name.contains(pattern) || file_name.ends_with(pattern) {
                return false;
            }
        }
    }

    // Check includes
    if let Some(include_patterns) = include {
        for pattern in include_patterns {
            if file_name.contains(pattern) || file_name.ends_with(pattern) {
                return true;
            }
        }
        return false;
    }

    true
}

// ============================================================================
// Query command
// ============================================================================

pub async fn query(
    query_text: String,
    top_k: usize,
    storage_path: Option<PathBuf>,
    model: String,
    format: String,
    web_search: bool,
    verbose: bool,
) -> anyhow::Result<()> {
    init_tracing(verbose);

    println!("{} Loading model...", "⚙".cyan().bold());
    let embedding_model: EmbeddingModel = model.parse().unwrap_or(EmbeddingModel::AllMiniLmL6V2);
    let embedder = FastEmbedder::new(embedding_model)?;
    let classifier = Classifier::new();

    // Initialize storage
    let storage: Box<dyn Storage> = if let Some(path) = storage_path {
        Box::new(FileStorage::new(&path).await?)
    } else {
        Box::new(MemoryStorage::new())
    };

    // Classify
    println!("{} Classifying query...", "🔍".cyan().bold());
    let classification = classifier.classify(&query_text);

    // Embed and search
    println!("{} Searching...", "🔍".cyan().bold());
    let embedding = embedder.embed_single(&query_text)?;
    let search_results = storage.search(&embedding, top_k).await?;

    // Build result
    let mut result = QueryResult::new(&query_text, classification);
    result = result.with_search_results(search_results);
    result.build_context(10000);

    // Web search if requested
    if web_search {
        println!("{} Searching web...", "🌐".cyan().bold());
        let searcher = WikipediaSearcher::new();
        if let Ok(web_results) = searcher.search(&query_text, 3).await {
            let mut context = result.context.clone();
            for web_result in web_results {
                if !context.is_empty() {
                    context.push_str("\n\n---\n\n");
                }
                context.push_str(&web_result.to_rag_context());
            }
            result = result.with_context(context).with_web_search();
        }
    }

    // Output
    match format.as_str() {
        "json" => {
            println!("{}", serde_json::to_string_pretty(&result)?);
        }
        _ => {
            println!("\n{}", "═".repeat(60).blue());
            println!(
                "{} {}",
                "Category:".bold(),
                format!("{:?}", result.classification.category).yellow()
            );
            println!(
                "{} {}",
                "Confidence:".bold(),
                format!("{:.2}", result.classification.confidence).green()
            );
            println!(
                "{} {:?}",
                "Strategy:".bold(),
                result.classification.strategy
            );
            println!("{}", "═".repeat(60).blue());

            if result.search_results.is_empty() {
                println!("\n{} No relevant documents found", "⚠".yellow().bold());
            } else {
                println!(
                    "\n{} {} relevant results:",
                    "📄".cyan().bold(),
                    result.search_results.len()
                );
                for (i, sr) in result.search_results.iter().enumerate() {
                    println!(
                        "\n{}. [Score: {:.4}]",
                        (i + 1).to_string().bold(),
                        sr.score
                    );
                    let preview: String = sr.document.content.chars().take(200).collect();
                    println!("   {}", preview.dimmed());
                }
            }

            if result.used_web_search {
                println!("\n{} Web search was used", "🌐".green().bold());
            }
        }
    }

    Ok(())
}

// ============================================================================
// Stats command
// ============================================================================

pub async fn stats(storage_path: Option<PathBuf>, verbose: bool) -> anyhow::Result<()> {
    init_tracing(verbose);

    let storage: Box<dyn Storage> = if let Some(path) = storage_path {
        Box::new(FileStorage::new(&path).await?)
    } else {
        println!(
            "{} No storage path provided, using empty memory storage",
            "⚠".yellow().bold()
        );
        Box::new(MemoryStorage::new())
    };

    let stats = storage.stats().await;

    println!("\n{}", "═".repeat(40).blue());
    println!("{}", "   Storage Statistics".bold());
    println!("{}", "═".repeat(40).blue());
    println!(
        "{} {}",
        "Documents:".bold(),
        stats.document_count.to_string().green()
    );
    println!(
        "{} {} bytes ({} KB)",
        "Storage size:".bold(),
        stats.total_content_bytes,
        stats.total_content_bytes / 1024
    );
    if let Some(dim) = stats.embedding_dimension {
        println!(
            "{} {}",
            "Embedding dimension:".bold(),
            dim.to_string().cyan()
        );
    }
    println!("{}", "═".repeat(40).blue());

    Ok(())
}

// ============================================================================
// Embed command
// ============================================================================

pub fn embed(text: String, model: String, format: String, verbose: bool) -> anyhow::Result<()> {
    init_tracing(verbose);

    let embedding_model: EmbeddingModel = model.parse().unwrap_or(EmbeddingModel::AllMiniLmL6V2);
    let embedder = FastEmbedder::new(embedding_model)?;

    let embedding = embedder.embed_single(&text)?;

    match format.as_str() {
        "json" => {
            let output = serde_json::json!({
                "text": text,
                "model": embedder.model(),
                "dimension": embedding.len(),
                "embedding": embedding
            });
            println!("{}", serde_json::to_string_pretty(&output)?);
        }
        _ => {
            println!(
                "{} Generated embedding with {} dimensions",
                "✓".green().bold(),
                embedding.len()
            );
            println!("{} Model: {}", "ℹ".cyan().bold(), embedder.model());
            println!(
                "{} First 5 values: {:?}...",
                "📊".cyan().bold(),
                &embedding[..5.min(embedding.len())]
            );
        }
    }

    Ok(())
}

// ============================================================================
// Classify command
// ============================================================================

pub fn classify(query: String, format: String, verbose: bool) -> anyhow::Result<()> {
    init_tracing(verbose);

    let classifier = Classifier::new();
    let result = classifier.classify(&query);

    match format.as_str() {
        "json" => {
            println!("{}", serde_json::to_string_pretty(&result)?);
        }
        _ => {
            println!("\n{}", "═".repeat(40).blue());
            println!("{} {}", "Query:".bold(), query.italic());
            println!("{}", "═".repeat(40).blue());
            println!(
                "{} {}",
                "Category:".bold(),
                format!("{:?}", result.category).yellow()
            );
            println!(
                "{} {:.2}",
                "Confidence:".bold(),
                result.confidence.to_string().green()
            );
            println!("{} {:?}", "Strategy:".bold(), result.strategy);
            println!("{}", "═".repeat(40).blue());
        }
    }

    Ok(())
}

// ============================================================================
// Search command
// ============================================================================

pub async fn search(query: String, count: usize, format: String, verbose: bool) -> anyhow::Result<()> {
    init_tracing(verbose);

    println!("{} Searching Wikipedia...", "🌐".cyan().bold());
    let searcher = WikipediaSearcher::new();
    let results = searcher.search(&query, count).await?;

    match format.as_str() {
        "json" => {
            println!("{}", serde_json::to_string_pretty(&results)?);
        }
        _ => {
            if results.is_empty() {
                println!("{} No results found", "⚠".yellow().bold());
            } else {
                println!(
                    "\n{} Found {} results:",
                    "✓".green().bold(),
                    results.len()
                );
                for (i, result) in results.iter().enumerate() {
                    println!("\n{}", "─".repeat(50).blue());
                    println!("{}. {}", (i + 1).to_string().bold(), result.title.yellow());
                    println!("   {}", result.url.dimmed());
                    let snippet: String = result.snippet.chars().take(200).collect();
                    println!("   {}", snippet);
                }
            }
        }
    }

    Ok(())
}

// ============================================================================
// Ask command (LLM integration)
// ============================================================================

pub async fn ask(
    question: String,
    model_path: Option<PathBuf>,
    model_name: String,
    llm_url: String,
    max_tokens: u32,
    temperature: f32,
    ctx_size: u32,
    threads: Option<i32>,
    storage_path: Option<PathBuf>,
    use_web: bool,
    format: String,
    show_timing: bool,
    stream: bool,
    auto_yes: bool,
    force_download: bool,
    verbose: bool,
) -> anyhow::Result<()> {
    use neuro_storage::Storage;
    use neuro_inference::{BitNetModel, ModelCache, DownloadOptions, get_or_download};
    use std::time::Instant;

    init_tracing(verbose);

    let total_start = Instant::now();

    // Step 1: Classify the query
    println!("{} Classifying query...", "🔍".cyan().bold());
    let classify_start = Instant::now();
    let classifier = Classifier::new();
    let classification = classifier.classify(&question);
    let classify_time = classify_start.elapsed();

    if verbose {
        println!(
            "  {} Category: {:?}, Confidence: {:.2}",
            "→".dimmed(),
            classification.category,
            classification.confidence
        );
    }

    // Step 2: Gather context
    let mut context_parts: Vec<String> = Vec::new();
    let mut context_time = std::time::Duration::ZERO;

    // From storage (RAG)
    if let Some(path) = &storage_path {
        println!("{} Loading context from storage...", "📁".cyan().bold());
        let ctx_start = Instant::now();
        
        let embedding_model = neuro_embeddings::EmbeddingModel::AllMiniLmL6V2;
        let embedder = FastEmbedder::new(embedding_model)?;
        let storage = FileStorage::new(path).await?;
        
        let query_embedding = embedder.embed_single(&question)?;
        let results = storage.search(&query_embedding, 3).await?;
        
        for result in results {
            if result.score > 0.5 {
                context_parts.push(format!("[Score: {:.2}] {}", result.score, result.document.content));
            }
        }
        context_time += ctx_start.elapsed();
    }

    // From web search
    if use_web {
        println!("{} Searching the web...", "🌐".cyan().bold());
        let web_start = Instant::now();
        
        let searcher = WikipediaSearcher::new();
        if let Ok(results) = searcher.search(&question, 3).await {
            for result in results {
                context_parts.push(format!("[{}] {}", result.title, result.snippet));
            }
        }
        context_time += web_start.elapsed();
    }

    let context = if context_parts.is_empty() {
        String::new()
    } else {
        context_parts.join("\n\n")
    };

    // Step 3: Resolve model path
    let resolved_model_path = if let Some(path) = model_path {
        // User specified explicit path
        path
    } else {
        // Use model name to get/download from cache
        let bitnet_model = BitNetModel::from_str(&model_name)
            .ok_or_else(|| anyhow::anyhow!(
                "Unknown model '{}'. Available: 2b, large, 3b, 8b", model_name
            ))?;

        let cache = ModelCache::new()?;
        
        let download_opts = DownloadOptions {
            yes: auto_yes,
            verify: true,
            force: force_download,
        };

        // Check if model exists, download if needed
        if !cache.is_downloaded(bitnet_model) {
            println!(
                "{} Model {} not found locally",
                "📦".yellow().bold(),
                bitnet_model.name()
            );
            
            get_or_download(&cache, bitnet_model, &download_opts).await?
        } else {
            cache.model_path(bitnet_model)
        }
    };

    // Step 4: Generate response - local or remote
    let (answer, llm_time) = if resolved_model_path.exists() {
        // Local inference with BitNet
        ask_local(
            &question,
            &context,
            resolved_model_path,
            max_tokens,
            temperature,
            ctx_size,
            threads,
            stream,
            verbose,
        ).await?
    } else {
        // Remote server
        ask_remote(&question, &context, &llm_url, max_tokens, temperature).await?
    };

    let total_time = total_start.elapsed();

    // Output results
    match format.as_str() {
        "json" => {
            let output = serde_json::json!({
                "question": question,
                "answer": answer,
                "category": format!("{:?}", classification.category),
                "confidence": classification.confidence,
                "context_used": !context_parts.is_empty(),
                "timing": {
                    "classification_ms": classify_time.as_millis(),
                    "context_ms": context_time.as_millis(),
                    "llm_ms": llm_time.as_millis(),
                    "total_ms": total_time.as_millis(),
                }
            });
            println!("{}", serde_json::to_string_pretty(&output)?);
        }
        _ => {
            if !stream {
                println!("\n{}", "═".repeat(60).blue());
                println!("{} {}", "Question:".bold(), question.yellow());
                println!("{}", "═".repeat(60).blue());
                println!("\n{}\n", answer.green());
                println!("{}", "═".repeat(60).blue());
            }
            
            if show_timing {
                println!(
                    "{} Classification: {}ms | Context: {}ms | LLM: {}ms | Total: {}ms",
                    "⏱".dimmed(),
                    classify_time.as_millis(),
                    context_time.as_millis(),
                    llm_time.as_millis(),
                    total_time.as_millis()
                );
            }
        }
    }

    Ok(())
}

/// Ask using local model inference
async fn ask_local(
    question: &str,
    context: &str,
    model_path: PathBuf,
    max_tokens: u32,
    temperature: f32,
    ctx_size: u32,
    threads: Option<i32>,
    stream: bool,
    verbose: bool,
) -> anyhow::Result<(String, std::time::Duration)> {
    use neuro_inference::{InferenceConfig, InferenceModel, GenerateOptions, SamplerConfig};
    use std::time::Instant;

    println!(
        "{} Loading BitNet model: {}",
        "🤖".cyan().bold(),
        model_path.display()
    );

    let llm_start = Instant::now();

    // Build config
    let mut config = InferenceConfig::new(&model_path)
        .with_context_size(ctx_size);

    if let Some(t) = threads {
        config = config.with_threads(t);
    }

    // Load model (this is blocking, so we spawn a blocking task)
    let model = tokio::task::spawn_blocking(move || InferenceModel::load(config))
        .await??;

    if verbose {
        println!(
            "  {} Using backend: {}",
            "→".dimmed(),
            model.backend_name()
        );
    }

    // Build prompt with context
    let prompt = if context.is_empty() {
        format!(
            "You are a helpful assistant. Answer the following question concisely and accurately.\n\n\
             Question: {}\n\n\
             Answer:",
            question
        )
    } else {
        format!(
            "You are a helpful assistant. Use the following context to answer the question.\n\n\
             Context:\n{}\n\n\
             Question: {}\n\n\
             Answer:",
            context, question
        )
    };

    println!("{} Generating response...", "✨".cyan().bold());

    // Generate options
    let options = GenerateOptions::new(max_tokens)
        .with_sampler(SamplerConfig::default().with_temperature(temperature))
        .with_stop_sequence("\n\nQuestion:")
        .with_stop_sequence("\n\nUser:");

    let options_with_stream = GenerateOptions {
        stream,
        ..options
    };

    // Generate response (blocking)
    let answer = tokio::task::spawn_blocking(move || model.generate(&prompt, &options_with_stream))
        .await??;

    let llm_time = llm_start.elapsed();

    Ok((answer.trim().to_string(), llm_time))
}

/// Ask using remote LLM server
async fn ask_remote(
    question: &str,
    context: &str,
    llm_url: &str,
    max_tokens: u32,
    temperature: f32,
) -> anyhow::Result<(String, std::time::Duration)> {
    use neuro_llm::{LlmClient, LlmConfig};
    use std::time::Instant;

    println!("{} Connecting to LLM at {}...", "🤖".cyan().bold(), llm_url);
    let llm_start = Instant::now();

    let config = LlmConfig {
        base_url: llm_url.to_string(),
        model: "bitnet".to_string(),
        timeout_secs: 120,
        max_tokens,
        temperature,
    };
    let client = LlmClient::with_config(config);

    // Check if server is available
    if !client.health_check().await.unwrap_or(false) {
        println!(
            "\n{} LLM server not available at {}",
            "✗".red().bold(),
            llm_url
        );
        println!("\n{}", "Options:".yellow());
        println!("  1. Use local model: neuro ask \"question\" --model-path /path/to/model.gguf");
        println!("  2. Start BitNet server: docker run -p 11435:11435 madkoding/neuro-bitnet:bitnet-2b");
        return Err(anyhow::anyhow!("LLM server not available"));
    }

    println!("{} Generating response...", "✨".cyan().bold());

    let context_opt = if context.is_empty() {
        None
    } else {
        Some(context)
    };

    let answer = client
        .ask_with_context(question, context_opt.unwrap_or(""), None)
        .await?;

    let llm_time = llm_start.elapsed();

    Ok((answer, llm_time))
}

// ============================================================================
// Helpers
// ============================================================================

fn init_tracing(verbose: bool) {
    use tracing_subscriber::EnvFilter;

    let filter = if verbose {
        EnvFilter::new("debug")
    } else {
        EnvFilter::new("info")
    };

    let _ = tracing_subscriber::fmt().with_env_filter(filter).try_init();
}

// ============================================================================
// Model command
// ============================================================================

use crate::cli::ModelAction;

pub async fn model(action: ModelAction, verbose: bool) -> anyhow::Result<()> {
    use neuro_inference::{BitNetModel, ModelCache, DownloadOptions, download_model};

    init_tracing(verbose);

    let cache = ModelCache::new()?;

    match action {
        ModelAction::List => {
            println!("\n{}", "═".repeat(70).blue());
            println!("{}", "   BitNet Models".bold());
            println!("{}", "═".repeat(70).blue());

            for model in BitNetModel::all() {
                let is_downloaded = cache.is_downloaded(*model);
                let status = if is_downloaded {
                    "✅ Downloaded".green().to_string()
                } else {
                    "⬜ Not downloaded".dimmed().to_string()
                };

                let default_marker = if *model == BitNetModel::default() {
                    " (default)".yellow().to_string()
                } else {
                    String::new()
                };

                println!(
                    "\n{}{} - {}",
                    model.name().bold(),
                    default_marker,
                    status
                );
                println!("   {} {}", "ID:".dimmed(), model.id());
                println!("   {} {}", "Size:".dimmed(), model.size_human());
                println!("   {} {}", "Params:".dimmed(), format!("{}B", model.params_billions()));
                println!("   {} {}", "Description:".dimmed(), model.description());
                
                if is_downloaded {
                    println!("   {} {}", "Path:".dimmed(), cache.model_path(*model).display());
                }
            }

            println!("\n{}", "═".repeat(70).blue());
            
            // Cache summary
            let downloaded = cache.list_downloaded();
            let total_size = cache.total_size();
            println!(
                "\n{} {} models downloaded, {:.2} GB total",
                "📊".cyan().bold(),
                downloaded.len(),
                total_size as f64 / 1_073_741_824.0
            );
            println!(
                "{} Cache: {}",
                "📁".cyan().bold(),
                cache.cache_dir().display()
            );
        }

        ModelAction::Download { model: model_name, force } => {
            let bitnet_model = BitNetModel::from_str(&model_name)
                .ok_or_else(|| anyhow::anyhow!(
                    "Unknown model '{}'. Available: 2b, large, 3b, 8b", model_name
                ))?;

            println!(
                "\n{} Downloading {}...",
                "📥".cyan().bold(),
                bitnet_model.name()
            );

            let opts = DownloadOptions {
                yes: true, // No confirmation for explicit download
                verify: true,
                force,
            };

            let path = download_model(&cache, bitnet_model, &opts).await?;
            
            println!(
                "\n{} Model downloaded to: {}",
                "✓".green().bold(),
                path.display()
            );
        }

        ModelAction::Remove { model: model_name } => {
            let bitnet_model = BitNetModel::from_str(&model_name)
                .ok_or_else(|| anyhow::anyhow!(
                    "Unknown model '{}'. Available: 2b, large, 3b, 8b", model_name
                ))?;

            if cache.delete_model(bitnet_model)? {
                println!(
                    "{} Removed model: {}",
                    "✓".green().bold(),
                    bitnet_model.name()
                );
            } else {
                println!(
                    "{} Model {} not found in cache",
                    "⚠".yellow().bold(),
                    bitnet_model.name()
                );
            }
        }

        ModelAction::Info => {
            let cache_dir = cache.cache_dir();
            let downloaded = cache.list_downloaded();
            let total_size = cache.total_size();

            println!("\n{}", "═".repeat(50).blue());
            println!("{}", "   Model Cache Info".bold());
            println!("{}", "═".repeat(50).blue());
            println!(
                "{} {}",
                "Cache directory:".bold(),
                cache_dir.display().to_string().cyan()
            );
            println!(
                "{} {}",
                "Downloaded models:".bold(),
                downloaded.len().to_string().green()
            );
            println!(
                "{} {:.2} GB",
                "Total size:".bold(),
                total_size as f64 / 1_073_741_824.0
            );
            
            if let Ok(env_var) = std::env::var("NEURO_BITNET_MODELS_DIR") {
                println!(
                    "{} {} (from NEURO_BITNET_MODELS_DIR)",
                    "Custom path:".bold(),
                    env_var.cyan()
                );
            }
            
            println!("{}", "═".repeat(50).blue());
        }
    }

    Ok(())
}
