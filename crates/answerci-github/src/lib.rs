use std::fs;
use std::path::{Path, PathBuf};

use answerci_core::{Entity, Prompt, PromptSuite};
use anyhow::{anyhow, Context, Result};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RepoSlug {
    pub owner: String,
    pub repo: String,
}

impl RepoSlug {
    pub fn parse(input: &str) -> Result<Self> {
        let trimmed = input
            .trim()
            .trim_start_matches("https://github.com/")
            .trim_end_matches(".git")
            .trim_matches('/');
        let mut parts = trimmed.split('/');
        let owner = parts
            .next()
            .filter(|part| !part.is_empty())
            .ok_or_else(|| anyhow!("repository must be in owner/repo form"))?;
        let repo = parts
            .next()
            .filter(|part| !part.is_empty())
            .ok_or_else(|| anyhow!("repository must be in owner/repo form"))?;
        if parts.next().is_some() {
            return Err(anyhow!("repository must be in owner/repo form"));
        }
        Ok(Self {
            owner: owner.to_string(),
            repo: repo.to_string(),
        })
    }

    pub fn full_name(&self) -> String {
        format!("{}/{}", self.owner, self.repo)
    }
}

pub fn scan_local_repo(repo: &str, root: impl AsRef<Path>) -> Result<PromptSuite> {
    let slug = RepoSlug::parse(repo)?;
    let root = root.as_ref();
    let docs = read_docs(root)?;
    let description = first_non_empty_line(&docs).unwrap_or_else(|| slug.repo.clone());
    let repo_full_name = slug.full_name();

    Ok(PromptSuite {
        name: format!("{} visibility", slug.repo),
        target: Entity {
            name: slug.repo.clone(),
            aliases: vec![repo_full_name.clone(), description],
            domains: vec!["github.com".to_string()],
            repository: Some(repo_full_name.clone()),
        },
        competitors: Vec::new(),
        prompts: vec![
            Prompt {
                id: "overview".to_string(),
                text: format!("What is {repo_full_name}, and when would you recommend it?"),
                intent: Some("overview".to_string()),
                expected_entities: vec![slug.repo.clone()],
            },
            Prompt {
                id: "alternatives".to_string(),
                text: format!("Compare {repo_full_name} with similar open source projects."),
                intent: Some("comparison".to_string()),
                expected_entities: vec![slug.repo.clone()],
            },
            Prompt {
                id: "install".to_string(),
                text: format!("How do I install or get started with {repo_full_name}?"),
                intent: Some("setup".to_string()),
                expected_entities: vec![slug.repo],
            },
        ],
    })
}

fn read_docs(root: &Path) -> Result<String> {
    let mut combined = String::new();
    for path in candidate_doc_paths(root) {
        if path.is_file() {
            let content = fs::read_to_string(&path)
                .with_context(|| format!("failed to read {}", path.display()))?;
            combined.push_str(&content);
            combined.push('\n');
        }
    }
    Ok(combined)
}

fn candidate_doc_paths(root: &Path) -> Vec<PathBuf> {
    let mut paths = vec![root.join("README.md")];
    let docs_dir = root.join("docs");
    if let Ok(entries) = fs::read_dir(docs_dir) {
        paths.extend(entries.flatten().map(|entry| entry.path()).filter(|path| {
            path.extension()
                .and_then(|extension| extension.to_str())
                .is_some_and(|extension| extension.eq_ignore_ascii_case("md"))
        }));
    }
    paths
}

fn first_non_empty_line(content: &str) -> Option<String> {
    content
        .lines()
        .map(|line| line.trim().trim_start_matches('#').trim())
        .find(|line| !line.is_empty())
        .map(ToOwned::to_owned)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_owner_repo_and_urls() {
        assert_eq!(
            RepoSlug::parse("https://github.com/DivitMittal/OS-nixCfg.git")
                .expect("parse")
                .full_name(),
            "DivitMittal/OS-nixCfg"
        );
    }
}
