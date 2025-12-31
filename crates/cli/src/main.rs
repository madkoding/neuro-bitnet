//! neuro-bitnet CLI entry point

use neuro_cli::cli::{Cli, Commands};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse_args();

    match cli.command {
        Commands::Serve {
            host,
            port,
            storage,
            model,
        } => {
            neuro_cli::commands::serve(host, port, storage, model, cli.verbose).await?;
        }
        Commands::Index {
            paths,
            recursive,
            include,
            exclude,
            max_size,
            storage,
            model,
            progress,
        } => {
            neuro_cli::commands::index(
                paths,
                recursive,
                include,
                exclude,
                max_size,
                storage,
                model,
                progress,
                cli.verbose,
            )
            .await?;
        }
        Commands::Query {
            query,
            top_k,
            storage,
            model,
            format,
            web,
        } => {
            neuro_cli::commands::query(query, top_k, storage, model, format, web, cli.verbose)
                .await?;
        }
        Commands::Stats { storage } => {
            neuro_cli::commands::stats(storage, cli.verbose).await?;
        }
        Commands::Embed {
            text,
            model,
            format,
        } => {
            neuro_cli::commands::embed(text, model, format, cli.verbose)?;
        }
        Commands::Classify { query, format } => {
            neuro_cli::commands::classify(query, format, cli.verbose)?;
        }
        Commands::Search {
            query,
            count,
            format,
        } => {
            neuro_cli::commands::search(query, count, format, cli.verbose).await?;
        }
        Commands::Ask {
            question,
            model_path,
            model,
            llm_url,
            max_tokens,
            temperature,
            ctx_size,
            threads,
            storage,
            web,
            format,
            timing,
            stream,
            yes,
            force_download,
        } => {
            neuro_cli::commands::ask(
                question,
                model_path,
                model,
                llm_url,
                max_tokens,
                temperature,
                ctx_size,
                threads,
                storage,
                web,
                format,
                timing,
                stream,
                yes,
                force_download,
                cli.verbose,
            )
            .await?;
        }
        Commands::Model { action } => {
            neuro_cli::commands::model(action, cli.verbose).await?;
        }
    }

    Ok(())
}
