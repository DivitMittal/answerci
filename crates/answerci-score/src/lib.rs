use answerci_core::{Observation, PromptSuite, Scores, ScoresDelta};

pub fn compute_scores(observations: &[Observation], suite: &PromptSuite) -> Scores {
    if observations.is_empty() {
        return Scores::default();
    }

    let total = observations.len() as f64;
    let target_name = &suite.target.name;
    let competitor_names: Vec<&str> = suite
        .competitors
        .iter()
        .map(|entity| entity.name.as_str())
        .collect();

    let target_visible = observations
        .iter()
        .filter(|observation| mentions_entity(observation, target_name))
        .count() as f64;
    let target_cited = observations
        .iter()
        .filter(|observation| cites_entity(observation, target_name))
        .count() as f64;
    let rank_weighted_visibility = observations
        .iter()
        .map(|observation| rank_weight_for_entity(observation, target_name))
        .sum::<f64>()
        / total;

    let target_mentions = observations
        .iter()
        .flat_map(|observation| observation.mentions.iter())
        .filter(|mention| mention.entity == *target_name)
        .count() as f64;
    let competitor_mentions = observations
        .iter()
        .flat_map(|observation| observation.mentions.iter())
        .filter(|mention| competitor_names.contains(&mention.entity.as_str()))
        .count() as f64;
    let all_voice_mentions = target_mentions + competitor_mentions;
    let competitor_share_of_voice = if all_voice_mentions == 0.0 {
        0.0
    } else {
        competitor_mentions / all_voice_mentions
    };
    let hallucinated = observations
        .iter()
        .filter(|observation| !observation.unsupported_claims.is_empty())
        .count() as f64;

    Scores {
        visibility_rate: target_visible / total,
        citation_rate: target_cited / total,
        rank_weighted_visibility,
        competitor_share_of_voice,
        hallucination_rate: hallucinated / total,
    }
}

pub fn regression_delta(baseline: Scores, candidate: Scores) -> ScoresDelta {
    ScoresDelta {
        visibility_rate: candidate.visibility_rate - baseline.visibility_rate,
        citation_rate: candidate.citation_rate - baseline.citation_rate,
        rank_weighted_visibility: candidate.rank_weighted_visibility
            - baseline.rank_weighted_visibility,
        competitor_share_of_voice: candidate.competitor_share_of_voice
            - baseline.competitor_share_of_voice,
        hallucination_rate: candidate.hallucination_rate - baseline.hallucination_rate,
    }
}

fn mentions_entity(observation: &Observation, entity: &str) -> bool {
    observation
        .mentions
        .iter()
        .any(|mention| mention.entity == entity)
}

fn cites_entity(observation: &Observation, entity: &str) -> bool {
    observation
        .citations
        .iter()
        .any(|citation| citation.matched_entity.as_deref() == Some(entity))
}

fn rank_weight_for_entity(observation: &Observation, entity: &str) -> f64 {
    let mut mentions = observation.mentions.clone();
    mentions.sort_by_key(|mention| mention.first_position);
    mentions
        .iter()
        .position(|mention| mention.entity == entity)
        .map(|index| 1.0 / (index as f64 + 1.0))
        .unwrap_or(0.0)
}

#[cfg(test)]
mod tests {
    use answerci_core::{Citation, Entity, Mention, Observation, PromptSuite};
    use chrono::Utc;

    use super::*;

    fn observation(
        id: &str,
        mentions: Vec<(&str, usize)>,
        citations: Vec<Option<&str>>,
        unsupported_claims: Vec<&str>,
    ) -> Observation {
        Observation {
            id: id.to_string(),
            run_id: "run".to_string(),
            prompt_id: id.to_string(),
            provider: "mock".to_string(),
            captured_at: Utc::now(),
            raw_response_id: format!("raw-{id}"),
            response_text: String::new(),
            mentions: mentions
                .into_iter()
                .map(|(entity, first_position)| Mention {
                    entity: entity.to_string(),
                    first_position,
                    matched_alias: entity.to_string(),
                })
                .collect(),
            citations: citations
                .into_iter()
                .enumerate()
                .map(|(index, matched_entity)| Citation {
                    url: format!("https://example.com/{index}"),
                    matched_entity: matched_entity.map(ToOwned::to_owned),
                    position: index,
                })
                .collect(),
            unsupported_claims: unsupported_claims
                .into_iter()
                .map(ToOwned::to_owned)
                .collect(),
        }
    }

    fn suite() -> PromptSuite {
        PromptSuite {
            name: "fixture".to_string(),
            target: Entity {
                name: "Target".to_string(),
                aliases: vec![],
                domains: vec![],
                repository: None,
            },
            competitors: vec![Entity {
                name: "Competitor".to_string(),
                aliases: vec![],
                domains: vec![],
                repository: None,
            }],
            prompts: vec![],
        }
    }

    #[test]
    fn computes_visibility_citation_voice_and_hallucination() {
        let observations = vec![
            observation(
                "one",
                vec![("Target", 10), ("Competitor", 50)],
                vec![Some("Target")],
                vec![],
            ),
            observation(
                "two",
                vec![("Competitor", 5), ("Target", 25)],
                vec![],
                vec!["bad"],
            ),
            observation(
                "three",
                vec![("Competitor", 5)],
                vec![Some("Competitor")],
                vec![],
            ),
        ];
        let scores = compute_scores(&observations, &suite());
        assert!((scores.visibility_rate - (2.0 / 3.0)).abs() < f64::EPSILON);
        assert!((scores.citation_rate - (1.0 / 3.0)).abs() < f64::EPSILON);
        assert!((scores.rank_weighted_visibility - 0.5).abs() < f64::EPSILON);
        assert!((scores.competitor_share_of_voice - 0.6).abs() < f64::EPSILON);
        assert!((scores.hallucination_rate - (1.0 / 3.0)).abs() < f64::EPSILON);
    }
}
