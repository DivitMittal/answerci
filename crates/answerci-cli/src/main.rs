use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use answerci_capture::CaptureStore;
use answerci_core::{PromptSuite, Run, Scores};
use answerci_extract::extract_observation;
use answerci_github::scan_local_repo;
use answerci_providers::{ManualImportProvider, OpenAiProvider, Provider};
use answerci_report::generate_markdown_report;
use answerci_score::{compute_scores, regression_delta};
use anyhow::{anyhow, Context, Result};
use chrono::Utc;
use clap::{Parser, Subcommand, ValueEnum};

#[derive(Debug, Parser)]
#[command(name = "answerci")]
#[command(about = "Visibility regression testing for AI answer engines")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    Init {
        #[arg(default_value = ".")]
        path: PathBuf,
    },
    Suite {
        #[command(subcommand)]
        command: SuiteCommands,
    },
    ScanGithub {
        repo: String,
        #[arg(long, default_value = ".")]
        root: PathBuf,
        #[arg(long)]
        output: Option<PathBuf>,
    },
    Run {
        #[arg(long)]
        suite: PathBuf,
        #[arg(long)]
        providers: String,
        #[arg(long, default_value = ".answerci/runs.db")]
        store: PathBuf,
        #[arg(long)]
        run_id: Option<String>,
        #[arg(long)]
        export_jsonl: Option<PathBuf>,
    },
    Diff {
        #[arg(long)]
        suite: PathBuf,
        #[arg(long)]
        baseline: String,
        #[arg(long)]
        candidate: String,
        #[arg(long, default_value = ".answerci/runs.db")]
        store: PathBuf,
    },
    Report {
        #[arg(long, value_enum, default_value_t = ReportFormat::Markdown)]
        format: ReportFormat,
        #[arg(long)]
        suite: PathBuf,
        #[arg(long)]
        run: Option<String>,
        #[arg(long)]
        baseline: Option<String>,
        #[arg(long)]
        candidate: Option<String>,
        #[arg(long, default_value = ".answerci/runs.db")]
        store: PathBuf,
        #[arg(long)]
        output: Option<PathBuf>,
    },
    Ci {
        #[arg(long)]
        fail_on: FailOn,
        #[arg(long)]
        suite: PathBuf,
        #[arg(long)]
        baseline: String,
        #[arg(long)]
        candidate: String,
        #[arg(long, default_value = ".answerci/runs.db")]
        store: PathBuf,
    },
}

#[derive(Debug, Subcommand)]
enum SuiteCommands {
    New {
        name: String,
        #[arg(long)]
        output: Option<PathBuf>,
    },
}

#[derive(Debug, Clone, ValueEnum)]
enum ReportFormat {
    Markdown,
}

#[derive(Debug, Clone, ValueEnum)]
enum FailOn {
    CitationRegression,
}

fn main() -> ExitCode {
    match run_cli() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("error: {error:#}");
            ExitCode::from(1)
        }
    }
}

fn run_cli() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Init { path } => init_project(&path),
        Commands::Suite { command } => match command {
            SuiteCommands::New { name, output } => suite_new(&name, output),
        },
        Commands::ScanGithub { repo, root, output } => scan_github(&repo, &root, output),
        Commands::Run {
            suite,
            providers,
            store,
            run_id,
            export_jsonl,
        } => run_suite(&suite, &providers, &store, run_id, export_jsonl),
        Commands::Diff {
            suite,
            baseline,
            candidate,
            store,
        } => diff_runs(&suite, &store, &baseline, &candidate),
        Commands::Report {
            format,
            suite,
            run,
            baseline,
            candidate,
            store,
            output,
        } => report(format, &suite, &store, run, baseline, candidate, output),
        Commands::Ci {
            fail_on,
            suite,
            baseline,
            candidate,
            store,
        } => ci(fail_on, &suite, &store, &baseline, &candidate),
    }
}

fn init_project(path: &Path) -> Result<()> {
    fs::create_dir_all(path.join("prompt-suites"))?;
    fs::create_dir_all(path.join("manual-imports"))?;
    fs::create_dir_all(path.join("reports"))?;
    println!("initialized AnswerCI project at {}", path.display());
    Ok(())
}

fn suite_new(name: &str, output: Option<PathBuf>) -> Result<()> {
    let suite = PromptSuite {
        name: name.to_string(),
        target: answerci_core::Entity {
            name: name.to_string(),
            aliases: vec![],
            domains: vec![],
            repository: None,
        },
        competitors: vec![],
        prompts: vec![answerci_core::Prompt {
            id: "overview".to_string(),
            text: format!("What is {name}, and when should someone use it?"),
            intent: Some("overview".to_string()),
            expected_entities: vec![name.to_string()],
        }],
    };
    write_yaml_or_stdout(&suite, output.as_deref())
}

fn scan_github(repo: &str, root: &Path, output: Option<PathBuf>) -> Result<()> {
    let suite = scan_local_repo(repo, root)?;
    write_yaml_or_stdout(&suite, output.as_deref())
}

fn run_suite(
    suite_path: &Path,
    providers: &str,
    store_path: &Path,
    run_id: Option<String>,
    export_jsonl: Option<PathBuf>,
) -> Result<()> {
    let suite = read_suite(suite_path)?;
    let store = CaptureStore::open(store_path)?;
    let run_id = run_id.unwrap_or_else(|| format!("run-{}", Utc::now().format("%Y%m%d%H%M%S")));

    for provider_spec in providers
        .split(',')
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        let mut provider = build_provider(provider_spec)?;
        let run = Run {
            id: run_id.clone(),
            suite_name: suite.name.clone(),
            provider: provider.name().to_string(),
            started_at: Utc::now(),
            completed_at: None,
        };
        store.insert_run(&run)?;
        for prompt in &suite.prompts {
            let raw = provider.capture(prompt).with_context(|| {
                format!("provider {} failed prompt {}", provider.name(), prompt.id)
            })?;
            let observation = extract_observation(&run_id, prompt, &raw, &suite)?;
            store.insert_raw_response(&run_id, &raw)?;
            store.insert_observation(&observation)?;
        }
        store.complete_run(&run_id, Utc::now())?;
    }

    if let Some(path) = export_jsonl {
        store.export_jsonl(&run_id, path)?;
    }
    println!("RUN_ID={run_id}");
    Ok(())
}

fn build_provider(spec: &str) -> Result<Box<dyn Provider>> {
    if let Some(path) = spec.strip_prefix("manual:") {
        return Ok(Box::new(ManualImportProvider::from_jsonl(path)?));
    }
    match spec {
        "openai" => Ok(Box::new(OpenAiProvider::new())),
        other => Err(anyhow!(
            "unknown provider '{other}'. Use manual:<jsonl-path> or openai"
        )),
    }
}

fn diff_runs(suite_path: &Path, store_path: &Path, baseline: &str, candidate: &str) -> Result<()> {
    let suite = read_suite(suite_path)?;
    let store = CaptureStore::open(store_path)?;
    let baseline_scores = compute_scores(&store.load_observations(baseline)?, &suite);
    let candidate_scores = compute_scores(&store.load_observations(candidate)?, &suite);
    let delta = regression_delta(baseline_scores, candidate_scores);
    println!("{}", serde_json::to_string_pretty(&delta)?);
    Ok(())
}

fn report(
    _format: ReportFormat,
    suite_path: &Path,
    store_path: &Path,
    run: Option<String>,
    baseline: Option<String>,
    candidate: Option<String>,
    output: Option<PathBuf>,
) -> Result<()> {
    let suite = read_suite(suite_path)?;
    let store = CaptureStore::open(store_path)?;
    let candidate_id = candidate
        .or(run)
        .ok_or_else(|| anyhow!("report requires --run or --candidate"))?;
    let candidate_observations = store.load_observations(&candidate_id)?;
    let candidate_scores = compute_scores(&candidate_observations, &suite);

    let (baseline_scores, delta) = if let Some(baseline_id) = &baseline {
        let scores = compute_scores(&store.load_observations(baseline_id)?, &suite);
        (
            Some(scores),
            Some(regression_delta(scores, candidate_scores)),
        )
    } else {
        (None, None)
    };

    let markdown = generate_markdown_report(
        &format!("{} Report", suite.name),
        baseline.as_deref(),
        &candidate_id,
        baseline_scores,
        candidate_scores,
        delta,
        &candidate_observations,
    );
    if let Some(path) = output {
        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() {
                fs::create_dir_all(parent)?;
            }
        }
        fs::write(path, markdown)?;
    } else {
        println!("{markdown}");
    }
    Ok(())
}

fn ci(
    fail_on: FailOn,
    suite_path: &Path,
    store_path: &Path,
    baseline: &str,
    candidate: &str,
) -> Result<()> {
    let suite = read_suite(suite_path)?;
    let store = CaptureStore::open(store_path)?;
    let baseline_scores: Scores = compute_scores(&store.load_observations(baseline)?, &suite);
    let candidate_scores: Scores = compute_scores(&store.load_observations(candidate)?, &suite);
    let delta = regression_delta(baseline_scores, candidate_scores);
    match fail_on {
        FailOn::CitationRegression if delta.citation_rate < 0.0 => Err(anyhow!(
            "citation regression detected: {:+.1}%",
            delta.citation_rate * 100.0
        )),
        FailOn::CitationRegression => {
            println!("no citation regression detected");
            Ok(())
        }
    }
}

fn read_suite(path: &Path) -> Result<PromptSuite> {
    let content = fs::read_to_string(path)
        .with_context(|| format!("failed to read suite {}", path.display()))?;
    serde_yaml::from_str(&content)
        .with_context(|| format!("failed to parse suite {}", path.display()))
}

fn write_yaml_or_stdout(suite: &PromptSuite, output: Option<&Path>) -> Result<()> {
    let yaml = serde_yaml::to_string(suite)?;
    if let Some(path) = output {
        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() {
                fs::create_dir_all(parent)?;
            }
        }
        fs::write(path, yaml)?;
    } else {
        println!("{yaml}");
    }
    Ok(())
}
