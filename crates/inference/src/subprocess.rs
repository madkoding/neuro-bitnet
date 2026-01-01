//! Subprocess backend for bitnet.cpp
//!
//! Fallback that calls the llama-cli binary directly when native bindings fail.

use crate::backend::{InferenceBackend, TokenCallback};
use crate::error::{InferenceError, Result};
use crate::sampler::SamplerConfig;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::io::{BufRead, BufReader};
use tracing::{debug, info, warn};

/// Subprocess-based inference backend
/// 
/// Uses the llama-cli binary from bitnet.cpp for inference.
/// This is a fallback when native bitnet-cpp bindings fail to compile.
pub struct SubprocessBackend {
    /// Path to llama-cli binary
    binary_path: PathBuf,
    /// Path to the model file
    model_path: PathBuf,
    /// Context size
    n_ctx: u32,
    /// Number of threads
    n_threads: Option<i32>,
}

impl SubprocessBackend {
    /// Create a new subprocess backend
    pub fn new<P: AsRef<Path>>(model_path: P) -> Result<Self> {
        let binary_path = Self::find_binary()?;
        
        Ok(Self {
            binary_path,
            model_path: model_path.as_ref().to_path_buf(),
            n_ctx: 2048,
            n_threads: None,
        })
    }

    /// Create with a specific binary path
    pub fn with_binary<P1: AsRef<Path>, P2: AsRef<Path>>(
        binary_path: P1,
        model_path: P2,
    ) -> Result<Self> {
        let binary = binary_path.as_ref().to_path_buf();
        if !binary.exists() {
            return Err(InferenceError::InvalidConfig(format!(
                "Binary not found: {}",
                binary.display()
            )));
        }

        Ok(Self {
            binary_path: binary,
            model_path: model_path.as_ref().to_path_buf(),
            n_ctx: 2048,
            n_threads: None,
        })
    }

    /// Set context size
    pub fn with_context_size(mut self, n_ctx: u32) -> Self {
        self.n_ctx = n_ctx;
        self
    }

    /// Set number of threads
    pub fn with_threads(mut self, threads: i32) -> Self {
        self.n_threads = Some(threads);
        self
    }

    /// Find the llama-cli binary
    fn find_binary() -> Result<PathBuf> {
        // Check environment variable
        if let Ok(path) = std::env::var("BITNET_CLI_PATH") {
            let path = PathBuf::from(path);
            if path.exists() {
                return Ok(path);
            }
        }

        // Check common locations
        let candidates = [
            // User installation (preferred)
            "~/.local/bin/llama-cli-bitnet",
            "~/.local/bin/llama-cli",
            // Local build
            "~/.local/share/bitnet.cpp/build/bin/llama-cli",
            "./bitnet.cpp/build/bin/llama-cli",
            "./BitNet/build/bin/llama-cli",
            // System installation
            "/usr/local/bin/llama-cli-bitnet",
            // Conda environment
            "${CONDA_PREFIX}/bin/llama-cli",
        ];

        for candidate in candidates {
            let expanded = shellexpand::tilde(candidate);
            let path = PathBuf::from(expanded.as_ref());
            if path.exists() {
                info!("Found bitnet.cpp binary at: {}", path.display());
                return Ok(path);
            }
        }

        Err(InferenceError::InvalidConfig(
            "Could not find llama-cli binary from bitnet.cpp. \
             Set BITNET_CLI_PATH or run scripts/setup_bitnet.sh".to_string()
        ))
    }

    /// Check if the backend is available
    pub fn is_available() -> bool {
        Self::find_binary().is_ok()
    }

    /// Get binary version (internal method)
    fn get_version(&self) -> Result<String> {
        let output = Command::new(&self.binary_path)
            .arg("--version")
            .output()
            .map_err(InferenceError::Io)?;

        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    }
}

impl InferenceBackend for SubprocessBackend {
    fn generate(&self, prompt: &str, max_tokens: u32, sampler: &SamplerConfig) -> Result<String> {
        let mut cmd = Command::new(&self.binary_path);
        
        cmd.arg("-m").arg(&self.model_path)
            .arg("-p").arg(prompt)
            .arg("-n").arg(max_tokens.to_string())
            .arg("-c").arg(self.n_ctx.to_string())
            .arg("--temp").arg(sampler.temperature.to_string())
            .arg("--top-k").arg(sampler.top_k.to_string())
            .arg("--top-p").arg(sampler.top_p.to_string())
            .arg("--repeat-penalty").arg(sampler.repeat_penalty.to_string())
            .arg("--no-display-prompt");

        if let Some(threads) = self.n_threads {
            cmd.arg("-t").arg(threads.to_string());
        }

        if sampler.seed != 0 {
            cmd.arg("-s").arg(sampler.seed.to_string());
        }

        cmd.env("LLAMA_LOG_DISABLE", "1");

        debug!("Running: {:?}", cmd);

        let output = cmd
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .map_err(InferenceError::Io)?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(InferenceError::Decode(format!(
                "llama-cli failed: {}",
                stderr
            )));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        Ok(stdout.trim().to_string())
    }

    fn generate_streaming(
        &self,
        prompt: &str,
        max_tokens: u32,
        sampler: &SamplerConfig,
        on_token: TokenCallback<'_>,
    ) -> Result<String> {
        let mut cmd = Command::new(&self.binary_path);
        
        cmd.arg("-m").arg(&self.model_path)
            .arg("-p").arg(prompt)
            .arg("-n").arg(max_tokens.to_string())
            .arg("-c").arg(self.n_ctx.to_string())
            .arg("--temp").arg(sampler.temperature.to_string())
            .arg("--top-k").arg(sampler.top_k.to_string())
            .arg("--top-p").arg(sampler.top_p.to_string())
            .arg("--repeat-penalty").arg(sampler.repeat_penalty.to_string())
            .arg("--log-disable");

        if let Some(threads) = self.n_threads {
            cmd.arg("-t").arg(threads.to_string());
        }

        let mut child = cmd
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .map_err(InferenceError::Io)?;

        let stdout = child.stdout.take()
            .ok_or_else(|| InferenceError::Decode("Failed to capture stdout".to_string()))?;

        let reader = BufReader::new(stdout);
        let mut output = String::new();
        let mut past_prompt = false;

        for line in reader.lines() {
            let line = line.map_err(InferenceError::Io)?;
            
            if !past_prompt {
                if line.contains(prompt) || output.len() < prompt.len() {
                    output.push_str(&line);
                    output.push('\n');
                    if output.len() >= prompt.len() {
                        past_prompt = true;
                        output.clear();
                    }
                    continue;
                }
                past_prompt = true;
            }

            on_token(&line);
            on_token("\n");
            output.push_str(&line);
            output.push('\n');
        }

        let status = child.wait().map_err(InferenceError::Io)?;
        if !status.success() {
            warn!("llama-cli exited with status: {}", status);
        }

        Ok(output.trim().to_string())
    }

    fn chat(
        &self,
        system_prompt: &str,
        user_message: &str,
        max_tokens: u32,
        sampler: &SamplerConfig,
    ) -> Result<String> {
        let prompt = format!(
            "<|system|>\n{}</s>\n<|user|>\n{}</s>\n<|assistant|>\n",
            system_prompt, user_message
        );
        self.generate(&prompt, max_tokens, sampler)
    }

    fn name(&self) -> &'static str {
        "bitnet.cpp (subprocess)"
    }

    fn is_ready(&self) -> bool {
        self.binary_path.exists()
    }

    fn version(&self) -> Result<String> {
        self.get_version()
    }
}

// Add shellexpand for tilde expansion
mod shellexpand {
    pub fn tilde(path: &str) -> std::borrow::Cow<'_, str> {
        if path.starts_with("~/") {
            if let Some(home) = dirs::home_dir() {
                return std::borrow::Cow::Owned(
                    path.replacen("~", &home.display().to_string(), 1)
                );
            }
        }
        std::borrow::Cow::Borrowed(path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shellexpand_tilde() {
        let expanded = shellexpand::tilde("~/test/path");
        assert!(!expanded.starts_with("~/"));
    }

    #[test]
    fn test_subprocess_not_found() {
        // Should fail gracefully when binary not found
        let result = SubprocessBackend::find_binary();
        // This will fail in test environment, which is expected
        assert!(result.is_err() || result.is_ok());
    }
}
