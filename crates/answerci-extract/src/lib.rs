use answerci_core::{Citation, Entity, Mention, Observation, Prompt, PromptSuite, RawResponse};
use anyhow::Result;
use regex::Regex;
use url::Url;

pub fn extract_observation(
    run_id: &str,
    prompt: &Prompt,
    raw: &RawResponse,
    suite: &PromptSuite,
) -> Result<Observation> {
    let mentions = extract_mentions(&raw.content, suite)?;
    let citations = extract_citations(&raw.content, suite)?;
    let unsupported_claims = extract_unsupported_claims(&raw.content);

    Ok(Observation {
        id: format!("{}:{}", run_id, prompt.id),
        run_id: run_id.to_string(),
        prompt_id: prompt.id.clone(),
        provider: raw.provider.clone(),
        captured_at: raw.captured_at,
        raw_response_id: raw.id.clone(),
        response_text: raw.content.clone(),
        mentions,
        citations,
        unsupported_claims,
    })
}

pub fn extract_mentions(content: &str, suite: &PromptSuite) -> Result<Vec<Mention>> {
    let mut mentions = Vec::new();
    for entity in suite.entities() {
        if let Some((matched_alias, first_position)) = first_alias_match(content, entity)? {
            mentions.push(Mention {
                entity: entity.name.clone(),
                first_position,
                matched_alias,
            });
        }
    }
    mentions.sort_by_key(|mention| mention.first_position);
    Ok(mentions)
}

fn first_alias_match(content: &str, entity: &Entity) -> Result<Option<(String, usize)>> {
    let mut best: Option<(String, usize)> = None;
    for alias in entity.all_aliases() {
        let pattern = format!(r"(?i)\b{}\b", regex::escape(&alias));
        let regex = Regex::new(&pattern)?;
        if let Some(found) = regex.find(content) {
            let candidate = (alias, found.start());
            if best
                .as_ref()
                .map(|(_, position)| candidate.1 < *position)
                .unwrap_or(true)
            {
                best = Some(candidate);
            }
        }
    }
    Ok(best)
}

pub fn extract_citations(content: &str, suite: &PromptSuite) -> Result<Vec<Citation>> {
    let url_regex = Regex::new(r#"https?://[^\s)\]>"']+"#)?;
    let mut citations = Vec::new();
    for found in url_regex.find_iter(content) {
        let url = found.as_str().trim_end_matches(['.', ',', ';', ':']);
        citations.push(Citation {
            url: url.to_string(),
            matched_entity: match_entity_for_url(url, suite),
            position: found.start(),
        });
    }
    Ok(citations)
}

fn match_entity_for_url(raw_url: &str, suite: &PromptSuite) -> Option<String> {
    let parsed = Url::parse(raw_url).ok()?;
    let host = parsed.host_str()?.trim_start_matches("www.").to_lowercase();
    let path = parsed.path().trim_matches('/').to_lowercase();

    for entity in suite.entities() {
        let domain_matches = entity.domains.iter().any(|domain| {
            let normalized = domain.trim_start_matches("www.").to_lowercase();
            host == normalized || host.ends_with(&format!(".{normalized}"))
        });
        let repo_matches = entity.repository.as_ref().is_some_and(|repository| {
            host == "github.com" && path.starts_with(&repository.to_lowercase())
        });
        if domain_matches || repo_matches {
            return Some(entity.name.clone());
        }
    }
    None
}

pub fn extract_unsupported_claims(content: &str) -> Vec<String> {
    content
        .split(['.', '!', '?'])
        .map(str::trim)
        .filter(|sentence| {
            let lower = sentence.to_lowercase();
            lower.contains("guarantees")
                || lower.contains("officially endorsed")
                || lower.contains("without any evidence")
        })
        .filter(|sentence| !sentence.is_empty())
        .map(ToOwned::to_owned)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn suite() -> PromptSuite {
        PromptSuite {
            name: "test".to_string(),
            target: Entity {
                name: "AnswerCI".to_string(),
                aliases: vec!["answer ci".to_string()],
                domains: vec!["answerci.dev".to_string()],
                repository: Some("DivitMittal/answerci".to_string()),
            },
            competitors: vec![Entity {
                name: "OtherTool".to_string(),
                aliases: vec![],
                domains: vec!["other.example".to_string()],
                repository: None,
            }],
            prompts: vec![],
        }
    }

    #[test]
    fn extracts_case_insensitive_mentions_and_citations() {
        let content = "Try otherTool, then answer ci. See https://github.com/DivitMittal/answerci.";
        let mentions = extract_mentions(content, &suite()).expect("mentions extract");
        let citations = extract_citations(content, &suite()).expect("citations extract");
        assert_eq!(mentions[0].entity, "OtherTool");
        assert_eq!(mentions[1].entity, "AnswerCI");
        assert_eq!(citations[0].matched_entity.as_deref(), Some("AnswerCI"));
    }
}
