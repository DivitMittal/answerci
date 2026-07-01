use answerci_core::{Observation, Scores, ScoresDelta};
use chrono::Utc;

pub fn generate_markdown_report(
    title: &str,
    baseline_label: Option<&str>,
    candidate_label: &str,
    baseline_scores: Option<Scores>,
    candidate_scores: Scores,
    delta: Option<ScoresDelta>,
    observations: &[Observation],
) -> String {
    let mut report = String::new();
    report.push_str(&format!("# {title}\n\n"));
    report.push_str(&format!("Generated: {}\n\n", Utc::now().to_rfc3339()));
    report.push_str("## Summary\n\n");
    report.push_str("| Metric | Baseline | Candidate | Delta |\n");
    report.push_str("| --- | ---: | ---: | ---: |\n");

    for (name, baseline, candidate, delta_value) in
        metric_rows(baseline_scores, candidate_scores, delta)
    {
        report.push_str(&format!(
            "| {name} | {} | {} | {} |\n",
            format_optional_rate(baseline),
            format_rate(candidate),
            format_optional_delta(delta_value)
        ));
    }

    if let Some(label) = baseline_label {
        report.push_str(&format!("\nBaseline run: `{label}`  \n"));
    }
    report.push_str(&format!("Candidate run: `{candidate_label}`\n\n"));

    report.push_str("## Regressions\n\n");
    let regression_lines = regression_lines(delta);
    if regression_lines.is_empty() {
        report.push_str("No metric regressions detected.\n\n");
    } else {
        for line in regression_lines {
            report.push_str(&format!("- {line}\n"));
        }
        report.push('\n');
    }

    report.push_str("## Observations\n\n");
    report.push_str("| Prompt | Mentions | Citations | Unsupported claims |\n");
    report.push_str("| --- | --- | --- | ---: |\n");
    for observation in observations {
        let mentions = if observation.mentions.is_empty() {
            "—".to_string()
        } else {
            observation
                .mentions
                .iter()
                .map(|mention| mention.entity.clone())
                .collect::<Vec<_>>()
                .join(", ")
        };
        let citations = if observation.citations.is_empty() {
            "—".to_string()
        } else {
            observation
                .citations
                .iter()
                .map(|citation| {
                    citation
                        .matched_entity
                        .clone()
                        .unwrap_or_else(|| "unmatched".to_string())
                })
                .collect::<Vec<_>>()
                .join(", ")
        };
        report.push_str(&format!(
            "| `{}` | {} | {} | {} |\n",
            observation.prompt_id,
            mentions,
            citations,
            observation.unsupported_claims.len()
        ));
    }

    report
}

type MetricRow = (&'static str, Option<f64>, f64, Option<f64>);

fn metric_rows(
    baseline: Option<Scores>,
    candidate: Scores,
    delta: Option<ScoresDelta>,
) -> Vec<MetricRow> {
    vec![
        (
            "visibility_rate",
            baseline.map(|scores| scores.visibility_rate),
            candidate.visibility_rate,
            delta.map(|value| value.visibility_rate),
        ),
        (
            "citation_rate",
            baseline.map(|scores| scores.citation_rate),
            candidate.citation_rate,
            delta.map(|value| value.citation_rate),
        ),
        (
            "rank_weighted_visibility",
            baseline.map(|scores| scores.rank_weighted_visibility),
            candidate.rank_weighted_visibility,
            delta.map(|value| value.rank_weighted_visibility),
        ),
        (
            "competitor_share_of_voice",
            baseline.map(|scores| scores.competitor_share_of_voice),
            candidate.competitor_share_of_voice,
            delta.map(|value| value.competitor_share_of_voice),
        ),
        (
            "hallucination_rate",
            baseline.map(|scores| scores.hallucination_rate),
            candidate.hallucination_rate,
            delta.map(|value| value.hallucination_rate),
        ),
    ]
}

fn regression_lines(delta: Option<ScoresDelta>) -> Vec<String> {
    let Some(delta) = delta else {
        return Vec::new();
    };
    let mut lines = Vec::new();
    if delta.visibility_rate < 0.0 {
        lines.push(format!(
            "Visibility rate declined by {}.",
            format_rate(delta.visibility_rate.abs())
        ));
    }
    if delta.citation_rate < 0.0 {
        lines.push(format!(
            "Citation rate declined by {}.",
            format_rate(delta.citation_rate.abs())
        ));
    }
    if delta.rank_weighted_visibility < 0.0 {
        lines.push(format!(
            "Rank-weighted visibility declined by {}.",
            format_rate(delta.rank_weighted_visibility.abs())
        ));
    }
    if delta.competitor_share_of_voice > 0.0 {
        lines.push(format!(
            "Competitor share of voice increased by {}.",
            format_rate(delta.competitor_share_of_voice)
        ));
    }
    if delta.hallucination_rate > 0.0 {
        lines.push(format!(
            "Hallucination rate increased by {}.",
            format_rate(delta.hallucination_rate)
        ));
    }
    lines
}

fn format_optional_rate(value: Option<f64>) -> String {
    value.map(format_rate).unwrap_or_else(|| "—".to_string())
}

fn format_optional_delta(value: Option<f64>) -> String {
    value
        .map(|number| format!("{number:+.1}%", number = number * 100.0))
        .unwrap_or_else(|| "—".to_string())
}

fn format_rate(value: f64) -> String {
    format!("{:.1}%", value * 100.0)
}
