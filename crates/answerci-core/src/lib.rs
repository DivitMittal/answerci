use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Entity {
    pub name: String,
    #[serde(default)]
    pub aliases: Vec<String>,
    #[serde(default)]
    pub domains: Vec<String>,
    #[serde(default)]
    pub repository: Option<String>,
}

impl Entity {
    pub fn all_aliases(&self) -> Vec<String> {
        let mut aliases = vec![self.name.clone()];
        aliases.extend(self.aliases.iter().cloned());
        if let Some(repository) = &self.repository {
            aliases.push(repository.clone());
            if let Some(repo_name) = repository.split('/').next_back() {
                aliases.push(repo_name.to_string());
            }
        }
        aliases.sort_by_key(|alias| alias.to_lowercase());
        aliases.dedup_by(|left, right| left.eq_ignore_ascii_case(right));
        aliases
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Prompt {
    pub id: String,
    pub text: String,
    #[serde(default)]
    pub intent: Option<String>,
    #[serde(default)]
    pub expected_entities: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PromptSuite {
    pub name: String,
    pub target: Entity,
    #[serde(default)]
    pub competitors: Vec<Entity>,
    pub prompts: Vec<Prompt>,
}

impl PromptSuite {
    pub fn entities(&self) -> Vec<&Entity> {
        let mut entities = vec![&self.target];
        entities.extend(self.competitors.iter());
        entities
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RawResponse {
    pub id: String,
    pub prompt_id: String,
    pub provider: String,
    pub captured_at: DateTime<Utc>,
    pub content: String,
    #[serde(default)]
    pub metadata: serde_json::Value,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Mention {
    pub entity: String,
    pub first_position: usize,
    pub matched_alias: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Citation {
    pub url: String,
    #[serde(default)]
    pub matched_entity: Option<String>,
    pub position: usize,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Observation {
    pub id: String,
    pub run_id: String,
    pub prompt_id: String,
    pub provider: String,
    pub captured_at: DateTime<Utc>,
    pub raw_response_id: String,
    pub response_text: String,
    #[serde(default)]
    pub mentions: Vec<Mention>,
    #[serde(default)]
    pub citations: Vec<Citation>,
    #[serde(default)]
    pub unsupported_claims: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Run {
    pub id: String,
    pub suite_name: String,
    pub provider: String,
    pub started_at: DateTime<Utc>,
    #[serde(default)]
    pub completed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Scores {
    pub visibility_rate: f64,
    pub citation_rate: f64,
    pub rank_weighted_visibility: f64,
    pub competitor_share_of_voice: f64,
    pub hallucination_rate: f64,
}

impl Default for Scores {
    fn default() -> Self {
        Self {
            visibility_rate: 0.0,
            citation_rate: 0.0,
            rank_weighted_visibility: 0.0,
            competitor_share_of_voice: 0.0,
            hallucination_rate: 0.0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ScoresDelta {
    pub visibility_rate: f64,
    pub citation_rate: f64,
    pub rank_weighted_visibility: f64,
    pub competitor_share_of_voice: f64,
    pub hallucination_rate: f64,
}
