use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

use answerci_core::{Prompt, RawResponse};
use anyhow::{anyhow, Context, Result};
use chrono::Utc;
use serde::Deserialize;
use thiserror::Error;

pub trait Provider {
    fn name(&self) -> &str;
    fn capture(&mut self, prompt: &Prompt) -> Result<RawResponse>;
}

#[derive(Debug, Error)]
pub enum ProviderError {
    #[error("provider did not contain a response for prompt '{0}'")]
    MissingPrompt(String),
    #[error("OpenAI capture requires an API key and live network support; use manual imports for CI fixtures")]
    OpenAiUnavailable,
}

#[derive(Debug, Deserialize)]
struct ManualImportRecord {
    prompt_id: String,
    content: String,
    #[serde(default = "default_provider")]
    provider: String,
    #[serde(default)]
    id: Option<String>,
    #[serde(default)]
    metadata: serde_json::Value,
}

fn default_provider() -> String {
    "manual".to_string()
}

pub struct ManualImportProvider {
    provider_name: String,
    responses: HashMap<String, RawResponse>,
}

impl ManualImportProvider {
    pub fn from_jsonl(path: impl AsRef<Path>) -> Result<Self> {
        let path_ref = path.as_ref();
        let file = File::open(path_ref)
            .with_context(|| format!("failed to open manual import {}", path_ref.display()))?;
        let reader = BufReader::new(file);
        let mut provider_name = "manual".to_string();
        let mut responses = HashMap::new();

        for (line_index, line) in reader.lines().enumerate() {
            let line = line.with_context(|| {
                format!(
                    "failed to read line {} from {}",
                    line_index + 1,
                    path_ref.display()
                )
            })?;
            if line.trim().is_empty() {
                continue;
            }
            let record: ManualImportRecord = serde_json::from_str(&line).with_context(|| {
                format!(
                    "failed to parse JSONL line {} from {}",
                    line_index + 1,
                    path_ref.display()
                )
            })?;
            provider_name = record.provider.clone();
            let raw = RawResponse {
                id: record
                    .id
                    .unwrap_or_else(|| format!("{}:{}", record.provider, record.prompt_id)),
                prompt_id: record.prompt_id.clone(),
                provider: record.provider,
                captured_at: Utc::now(),
                content: record.content,
                metadata: record.metadata,
            };
            responses.insert(record.prompt_id, raw);
        }

        if responses.is_empty() {
            return Err(anyhow!(
                "manual import {} contained no responses",
                path_ref.display()
            ));
        }

        Ok(Self {
            provider_name,
            responses,
        })
    }
}

impl Provider for ManualImportProvider {
    fn name(&self) -> &str {
        &self.provider_name
    }

    fn capture(&mut self, prompt: &Prompt) -> Result<RawResponse> {
        self.responses
            .get(&prompt.id)
            .cloned()
            .ok_or_else(|| ProviderError::MissingPrompt(prompt.id.clone()).into())
    }
}

pub struct MockProvider {
    provider_name: String,
    responses: HashMap<String, String>,
}

impl MockProvider {
    pub fn new(responses: HashMap<String, String>) -> Self {
        Self {
            provider_name: "mock".to_string(),
            responses,
        }
    }
}

impl Provider for MockProvider {
    fn name(&self) -> &str {
        &self.provider_name
    }

    fn capture(&mut self, prompt: &Prompt) -> Result<RawResponse> {
        let content = self
            .responses
            .get(&prompt.id)
            .cloned()
            .ok_or_else(|| ProviderError::MissingPrompt(prompt.id.clone()))?;
        Ok(RawResponse {
            id: format!("mock:{}", prompt.id),
            prompt_id: prompt.id.clone(),
            provider: self.provider_name.clone(),
            captured_at: Utc::now(),
            content,
            metadata: serde_json::Value::Null,
        })
    }
}

pub struct OpenAiProvider;

impl OpenAiProvider {
    pub fn new() -> Self {
        Self
    }
}

impl Default for OpenAiProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl Provider for OpenAiProvider {
    fn name(&self) -> &str {
        "openai"
    }

    fn capture(&mut self, _prompt: &Prompt) -> Result<RawResponse> {
        Err(ProviderError::OpenAiUnavailable.into())
    }
}
