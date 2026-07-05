# Metrics

Let `N` be the number of observations in a run, `T` the target entity, and `C` the set of competitor entities.

| Metric | Formula | Meaning |
| --- | --- | --- |
| `visibility_rate` | `count(observations mentioning T) / N` | How often the target appears at all. |
| `citation_rate` | `count(observations citing a URL matched to T) / N` | How often the target is cited with a matching URL. |
| `rank_weighted_visibility` | `sum(1 / mention_rank(T)) / N`, with `0` when absent | Rewards earlier target mentions. First mention is `1.0`, second is `0.5`, third is `0.333...`. |
| `competitor_share_of_voice` | `competitor_mentions / (target_mentions + competitor_mentions)` | Share of entity mentions captured by competitors. Lower is better for the target. |
| `hallucination_rate` | `count(observations with unsupported_claims) / N` | Placeholder quality signal for likely unsupported claims. |
| `regression_delta` | `candidate_metric - baseline_metric` | Signed change between two runs. Negative is bad for visibility/citation/rank; positive is bad for competitor share and hallucination. |

The MVP uses deterministic text heuristics. Strong v1 should add provider-specific metadata, richer claim checking, and confidence intervals across repeated captures.
