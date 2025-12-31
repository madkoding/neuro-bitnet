//! Model cache and download management
//!
//! Handles model storage in `NEURO_BITNET_MODELS_DIR` or `~/.cache/neuro-bitnet/models/`

use crate::error::{InferenceError, Result};
use crate::models::BitNetModel;
use std::path::{Path, PathBuf};
use tracing::{info, warn, debug};

/// Model cache manager
pub struct ModelCache {
    cache_dir: PathBuf,
}

impl ModelCache {
    /// Create a new model cache
    /// 
    /// Uses `NEURO_BITNET_MODELS_DIR` env var if set, otherwise `~/.cache/neuro-bitnet/models/`
    pub fn new() -> Result<Self> {
        let cache_dir = Self::resolve_cache_dir()?;
        Ok(Self { cache_dir })
    }

    /// Create cache with a specific directory
    pub fn with_dir(cache_dir: PathBuf) -> Self {
        Self { cache_dir }
    }

    /// Resolve the cache directory path
    fn resolve_cache_dir() -> Result<PathBuf> {
        // Check environment variable first
        if let Ok(dir) = std::env::var("NEURO_BITNET_MODELS_DIR") {
            let path = PathBuf::from(dir);
            return Ok(path);
        }

        // Fall back to ~/.cache/neuro-bitnet/models/
        let cache_base = dirs::cache_dir()
            .ok_or_else(|| InferenceError::InvalidConfig(
                "Could not determine cache directory".to_string()
            ))?;

        Ok(cache_base.join("neuro-bitnet").join("models"))
    }

    /// Get the cache directory path
    pub fn cache_dir(&self) -> &Path {
        &self.cache_dir
    }

    /// Ensure cache directory exists
    pub fn ensure_dir(&self) -> Result<()> {
        if !self.cache_dir.exists() {
            std::fs::create_dir_all(&self.cache_dir)?;
            info!("Created model cache directory: {}", self.cache_dir.display());
        }
        Ok(())
    }

    /// Get the path where a model should be stored
    pub fn model_path(&self, model: BitNetModel) -> PathBuf {
        self.cache_dir.join(model.id()).join(model.filename())
    }

    /// Check if a model is already downloaded
    pub fn is_downloaded(&self, model: BitNetModel) -> bool {
        let path = self.model_path(model);
        if !path.exists() {
            return false;
        }

        // Verify file size is reasonable (at least 100MB for any model)
        if let Ok(metadata) = std::fs::metadata(&path) {
            metadata.len() > 100_000_000
        } else {
            false
        }
    }

    /// List all downloaded models
    pub fn list_downloaded(&self) -> Vec<(BitNetModel, PathBuf)> {
        BitNetModel::all()
            .iter()
            .filter_map(|&model| {
                if self.is_downloaded(model) {
                    Some((model, self.model_path(model)))
                } else {
                    None
                }
            })
            .collect()
    }

    /// Get model path, returning error if not downloaded
    pub fn get_model(&self, model: BitNetModel) -> Result<PathBuf> {
        let path = self.model_path(model);
        if self.is_downloaded(model) {
            Ok(path)
        } else {
            Err(InferenceError::ModelLoad {
                path: path.display().to_string(),
                message: format!(
                    "Model {} not found. Run with --download or use: neuro model download {}",
                    model.name(),
                    model.id()
                ),
            })
        }
    }

    /// Delete a downloaded model
    pub fn delete_model(&self, model: BitNetModel) -> Result<bool> {
        let model_dir = self.cache_dir.join(model.id());
        if model_dir.exists() {
            std::fs::remove_dir_all(&model_dir)?;
            info!("Deleted model: {}", model.name());
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Get total size of cached models
    pub fn total_size(&self) -> u64 {
        self.list_downloaded()
            .iter()
            .filter_map(|(_, path)| std::fs::metadata(path).ok())
            .map(|m| m.len())
            .sum()
    }
}

impl Default for ModelCache {
    fn default() -> Self {
        Self::new().expect("Failed to initialize model cache")
    }
}

/// Download a model with progress reporting
#[cfg(feature = "download")]
pub mod download {
    use super::*;
    use futures_util::StreamExt;
    use indicatif::{ProgressBar, ProgressStyle};
    use sha2::{Sha256, Digest};
    use std::io::Write;
    use tokio::io::AsyncWriteExt;

    /// Download options
    #[derive(Debug, Clone)]
    pub struct DownloadOptions {
        /// Skip confirmation prompt
        pub yes: bool,
        /// Verify SHA256 checksum if available
        pub verify: bool,
        /// Force re-download even if exists
        pub force: bool,
    }

    impl Default for DownloadOptions {
        fn default() -> Self {
            Self {
                yes: false,
                verify: true,
                force: false,
            }
        }
    }

    /// Download a model to cache
    pub async fn download_model(
        cache: &ModelCache,
        model: BitNetModel,
        options: &DownloadOptions,
    ) -> Result<PathBuf> {
        let target_path = cache.model_path(model);
        
        // Check if already exists
        if !options.force && cache.is_downloaded(model) {
            info!("Model {} already downloaded at {}", model.name(), target_path.display());
            return Ok(target_path);
        }

        // Create directory
        if let Some(parent) = target_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let url = model.download_url();
        let expected_size = model.size_bytes();

        info!("Downloading {} ({})...", model.name(), model.size_human());
        info!("URL: {}", url);

        // Create HTTP client
        let client = reqwest::Client::new();
        let response = client
            .get(url)
            .send()
            .await
            .map_err(|e| InferenceError::ModelLoad {
                path: url.to_string(),
                message: format!("HTTP request failed: {}", e),
            })?;

        if !response.status().is_success() {
            return Err(InferenceError::ModelLoad {
                path: url.to_string(),
                message: format!("HTTP error: {}", response.status()),
            });
        }

        let total_size = response.content_length().unwrap_or(expected_size);

        // Create progress bar
        let pb = ProgressBar::new(total_size);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({eta})")
                .unwrap()
                .progress_chars("#>-"),
        );

        // Download with progress
        let temp_path = target_path.with_extension("download");
        let mut file = tokio::fs::File::create(&temp_path)
            .await
            .map_err(|e| InferenceError::Io(e))?;

        let mut hasher = Sha256::new();
        let mut downloaded: u64 = 0;
        let mut stream = response.bytes_stream();

        while let Some(chunk) = stream.next().await {
            let chunk = chunk.map_err(|e| InferenceError::ModelLoad {
                path: url.to_string(),
                message: format!("Download error: {}", e),
            })?;

            file.write_all(&chunk)
                .await
                .map_err(|e| InferenceError::Io(e))?;

            if options.verify {
                hasher.update(&chunk);
            }

            downloaded += chunk.len() as u64;
            pb.set_position(downloaded);
        }

        pb.finish_with_message("Download complete");

        // Flush and close file
        file.flush().await.map_err(|e| InferenceError::Io(e))?;
        drop(file);

        // Verify checksum if available
        if options.verify {
            if let Some(expected_hash) = model.sha256() {
                let actual_hash = format!("{:x}", hasher.finalize());
                if actual_hash != expected_hash {
                    // Clean up failed download
                    let _ = std::fs::remove_file(&temp_path);
                    return Err(InferenceError::ModelLoad {
                        path: target_path.display().to_string(),
                        message: format!(
                            "Checksum mismatch: expected {}, got {}",
                            expected_hash, actual_hash
                        ),
                    });
                }
                info!("Checksum verified: {}", actual_hash);
            } else {
                debug!("No checksum available for verification");
            }
        }

        // Move temp file to final location
        std::fs::rename(&temp_path, &target_path)?;

        info!("Model saved to: {}", target_path.display());
        Ok(target_path)
    }

    /// Interactive download prompt
    pub fn confirm_download(model: BitNetModel) -> bool {
        println!("\n📦 Model: {}", model.name());
        println!("   Size: {}", model.size_human());
        println!("   Description: {}", model.description());
        println!("   Repository: {}", model.hf_repo());
        println!();
        
        print!("Download this model? [y/N] ");
        std::io::stdout().flush().ok();

        let mut input = String::new();
        if std::io::stdin().read_line(&mut input).is_ok() {
            let input = input.trim().to_lowercase();
            input == "y" || input == "yes"
        } else {
            false
        }
    }

    /// Get model, downloading if necessary
    pub async fn get_or_download(
        cache: &ModelCache,
        model: BitNetModel,
        options: &DownloadOptions,
    ) -> Result<PathBuf> {
        if cache.is_downloaded(model) {
            return Ok(cache.model_path(model));
        }

        // Ask for confirmation unless --yes
        if !options.yes && !confirm_download(model) {
            return Err(InferenceError::ModelLoad {
                path: cache.model_path(model).display().to_string(),
                message: "Download cancelled by user".to_string(),
            });
        }

        download_model(cache, model, options).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_dir_resolution() {
        let cache = ModelCache::new().unwrap();
        assert!(cache.cache_dir().to_string_lossy().contains("neuro-bitnet"));
    }

    #[test]
    fn test_model_path() {
        let cache = ModelCache::with_dir(PathBuf::from("/tmp/models"));
        let path = cache.model_path(BitNetModel::B1_58_2B_4T);
        assert_eq!(
            path,
            PathBuf::from("/tmp/models/bitnet-b1.58-2b-4t/ggml-model-i2_s.gguf")
        );
    }
}
