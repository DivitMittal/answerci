use std::fs::{self, File};
use std::io::Write;
use std::path::Path;

use answerci_core::{Observation, RawResponse, Run};
use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use rusqlite::{params, Connection};

pub struct CaptureStore {
    conn: Connection,
}

impl CaptureStore {
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let path_ref = path.as_ref();
        if let Some(parent) = path_ref.parent() {
            if !parent.as_os_str().is_empty() {
                fs::create_dir_all(parent).with_context(|| {
                    format!("failed to create store directory {}", parent.display())
                })?;
            }
        }
        let conn = Connection::open(path_ref)
            .with_context(|| format!("failed to open SQLite store {}", path_ref.display()))?;
        let store = Self { conn };
        store.init()?;
        Ok(store)
    }

    pub fn init(&self) -> Result<()> {
        self.conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS runs (
                id TEXT PRIMARY KEY,
                suite_name TEXT NOT NULL,
                provider TEXT NOT NULL,
                started_at TEXT NOT NULL,
                completed_at TEXT
            );
            CREATE TABLE IF NOT EXISTS raw_responses (
                id TEXT PRIMARY KEY,
                run_id TEXT NOT NULL,
                prompt_id TEXT NOT NULL,
                provider TEXT NOT NULL,
                captured_at TEXT NOT NULL,
                content TEXT NOT NULL,
                metadata TEXT NOT NULL,
                FOREIGN KEY(run_id) REFERENCES runs(id)
            );
            CREATE TABLE IF NOT EXISTS observations (
                id TEXT PRIMARY KEY,
                run_id TEXT NOT NULL,
                prompt_id TEXT NOT NULL,
                provider TEXT NOT NULL,
                captured_at TEXT NOT NULL,
                raw_response_id TEXT NOT NULL,
                response_text TEXT NOT NULL,
                mentions TEXT NOT NULL,
                citations TEXT NOT NULL,
                unsupported_claims TEXT NOT NULL,
                FOREIGN KEY(run_id) REFERENCES runs(id),
                FOREIGN KEY(raw_response_id) REFERENCES raw_responses(id)
            );",
        )?;
        Ok(())
    }

    pub fn insert_run(&self, run: &Run) -> Result<()> {
        self.conn.execute(
            "INSERT OR REPLACE INTO runs (id, suite_name, provider, started_at, completed_at)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                run.id,
                run.suite_name,
                run.provider,
                run.started_at.to_rfc3339(),
                run.completed_at.map(|value| value.to_rfc3339())
            ],
        )?;
        Ok(())
    }

    pub fn complete_run(&self, run_id: &str, completed_at: DateTime<Utc>) -> Result<()> {
        self.conn.execute(
            "UPDATE runs SET completed_at = ?1 WHERE id = ?2",
            params![completed_at.to_rfc3339(), run_id],
        )?;
        Ok(())
    }

    pub fn insert_raw_response(&self, run_id: &str, raw: &RawResponse) -> Result<()> {
        self.conn.execute(
            "INSERT OR REPLACE INTO raw_responses
             (id, run_id, prompt_id, provider, captured_at, content, metadata)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                raw.id,
                run_id,
                raw.prompt_id,
                raw.provider,
                raw.captured_at.to_rfc3339(),
                raw.content,
                serde_json::to_string(&raw.metadata)?
            ],
        )?;
        Ok(())
    }

    pub fn insert_observation(&self, observation: &Observation) -> Result<()> {
        self.conn.execute(
            "INSERT OR REPLACE INTO observations
             (id, run_id, prompt_id, provider, captured_at, raw_response_id, response_text,
              mentions, citations, unsupported_claims)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            params![
                observation.id,
                observation.run_id,
                observation.prompt_id,
                observation.provider,
                observation.captured_at.to_rfc3339(),
                observation.raw_response_id,
                observation.response_text,
                serde_json::to_string(&observation.mentions)?,
                serde_json::to_string(&observation.citations)?,
                serde_json::to_string(&observation.unsupported_claims)?
            ],
        )?;
        Ok(())
    }

    pub fn load_observations(&self, run_id: &str) -> Result<Vec<Observation>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, run_id, prompt_id, provider, captured_at, raw_response_id, response_text,
                    mentions, citations, unsupported_claims
             FROM observations
             WHERE run_id = ?1
             ORDER BY prompt_id",
        )?;
        let rows = stmt.query_map(params![run_id], |row| {
            let captured_at: String = row.get(4)?;
            let mentions_json: String = row.get(7)?;
            let citations_json: String = row.get(8)?;
            let unsupported_json: String = row.get(9)?;
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                captured_at,
                row.get::<_, String>(5)?,
                row.get::<_, String>(6)?,
                mentions_json,
                citations_json,
                unsupported_json,
            ))
        })?;

        let mut observations = Vec::new();
        for row in rows {
            let (
                id,
                run_id,
                prompt_id,
                provider,
                captured_at,
                raw_response_id,
                response_text,
                mentions_json,
                citations_json,
                unsupported_json,
            ) = row?;
            observations.push(Observation {
                id,
                run_id,
                prompt_id,
                provider,
                captured_at: DateTime::parse_from_rfc3339(&captured_at)
                    .with_context(|| format!("invalid captured_at timestamp {captured_at}"))?
                    .with_timezone(&Utc),
                raw_response_id,
                response_text,
                mentions: serde_json::from_str(&mentions_json)?,
                citations: serde_json::from_str(&citations_json)?,
                unsupported_claims: serde_json::from_str(&unsupported_json)?,
            });
        }
        Ok(observations)
    }

    pub fn export_jsonl(&self, run_id: &str, path: impl AsRef<Path>) -> Result<()> {
        let path_ref = path.as_ref();
        if let Some(parent) = path_ref.parent() {
            if !parent.as_os_str().is_empty() {
                fs::create_dir_all(parent).with_context(|| {
                    format!("failed to create export directory {}", parent.display())
                })?;
            }
        }
        let mut file = File::create(path_ref)
            .with_context(|| format!("failed to create export {}", path_ref.display()))?;
        for observation in self.load_observations(run_id)? {
            let line = serde_json::to_string(&observation)?;
            writeln!(file, "{line}")?;
        }
        Ok(())
    }
}
