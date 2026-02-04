use anyhow::Result;
use rusqlite::{Connection, params};
use std::path::Path;
use tracing::info;

use crate::modules::types::{Endpoint, Finding, HttpRequest, HttpResponse};

pub struct Storage {
    conn: Connection,
}

impl Storage {
    pub fn new<P: AsRef<Path>>(db_path: P) -> Result<Self> {
        let conn = Connection::open(db_path)?;

        let storage = Self { conn };
        storage.initialize_schema()?;

        Ok(storage)
    }

    fn initialize_schema(&self) -> Result<()> {
        // Endpoints table
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS endpoints (
                id TEXT PRIMARY KEY,
                url TEXT NOT NULL UNIQUE,
                method TEXT NOT NULL,
                discovered_at TEXT NOT NULL,
                source TEXT NOT NULL,
                parameters TEXT
            )",
            [],
        )?;

        // HTTP Requests table
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS requests (
                id TEXT PRIMARY KEY,
                url TEXT NOT NULL,
                method TEXT NOT NULL,
                headers TEXT,
                body TEXT,
                timestamp TEXT NOT NULL
            )",
            [],
        )?;

        // HTTP Responses table
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS responses (
                id TEXT PRIMARY KEY,
                request_id TEXT NOT NULL,
                status_code INTEGER NOT NULL,
                headers TEXT,
                body TEXT NOT NULL,
                duration_ms INTEGER NOT NULL,
                timestamp TEXT NOT NULL,
                FOREIGN KEY (request_id) REFERENCES requests(id)
            )",
            [],
        )?;

        // Findings table
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS findings (
                id TEXT PRIMARY KEY,
                endpoint_id TEXT NOT NULL,
                severity TEXT NOT NULL,
                title TEXT NOT NULL,
                description TEXT NOT NULL,
                evidence TEXT NOT NULL,
                discovered_at TEXT NOT NULL,
                FOREIGN KEY (endpoint_id) REFERENCES endpoints(id)
            )",
            [],
        )?;

        // Indexes for better query performance
        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_endpoints_url ON endpoints(url)",
            [],
        )?;

        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_responses_request_id ON responses(request_id)",
            [],
        )?;

        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_findings_endpoint_id ON findings(endpoint_id)",
            [],
        )?;

        info!("Database schema initialized");
        Ok(())
    }

    pub fn save_endpoint(&self, endpoint: &Endpoint) -> Result<()> {
        let parameters_json = serde_json::to_string(&endpoint.parameters)?;
        let source_json = serde_json::to_string(&endpoint.source)?;

        self.conn.execute(
            "INSERT OR IGNORE INTO endpoints (id, url, method, discovered_at, source, parameters)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                endpoint.id.to_string(),
                endpoint.url,
                endpoint.method,
                endpoint.discovered_at.to_rfc3339(),
                source_json,
                parameters_json,
            ],
        )?;

        Ok(())
    }

    pub fn save_request_response(
        &self,
        request: &HttpRequest,
        response: &HttpResponse,
    ) -> Result<()> {
        let request_headers = serde_json::to_string(&request.headers)?;
        let response_headers = serde_json::to_string(&response.headers)?;

        self.conn.execute(
            "INSERT INTO requests (id, url, method, headers, body, timestamp)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                request.id.to_string(),
                request.url,
                request.method,
                request_headers,
                request.body,
                request.timestamp.to_rfc3339(),
            ],
        )?;

        self.conn.execute(
            "INSERT INTO responses (id, request_id, status_code, headers, body, duration_ms, timestamp)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                response.id.to_string(),
                response.request_id.to_string(),
                response.status_code,
                response_headers,
                response.body,
                response.duration_ms as i64,
                response.timestamp.to_rfc3339(),
            ],
        )?;

        Ok(())
    }

    pub fn save_finding(&self, finding: &Finding) -> Result<()> {
        let evidence_json = serde_json::to_string(&finding.evidence)?;
        let severity_json = serde_json::to_string(&finding.severity)?;

        self.conn.execute(
            "INSERT INTO findings (id, endpoint_id, severity, title, description, evidence, discovered_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                finding.id.to_string(),
                finding.endpoint_id.to_string(),
                severity_json,
                finding.title,
                finding.description,
                evidence_json,
                finding.discovered_at.to_rfc3339(),
            ],
        )?;

        Ok(())
    }

    #[allow(dead_code)]
    pub fn get_all_endpoints(&self) -> Result<Vec<Endpoint>> {
        let mut stmt = self
            .conn
            .prepare("SELECT id, url, method, discovered_at, source, parameters FROM endpoints")?;

        let endpoints = stmt.query_map([], |row| {
            let id_str: String = row.get(0)?;
            let parameters_json: String = row.get(5)?;
            let source_json: String = row.get(4)?;

            Ok(Endpoint {
                id: uuid::Uuid::parse_str(&id_str).unwrap(),
                url: row.get(1)?,
                method: row.get(2)?,
                discovered_at: chrono::DateTime::parse_from_rfc3339(&row.get::<_, String>(3)?)
                    .unwrap()
                    .with_timezone(&chrono::Utc),
                source: serde_json::from_str(&source_json).unwrap(),
                parameters: serde_json::from_str(&parameters_json).unwrap(),
            })
        })?;

        let mut result = Vec::new();
        for endpoint in endpoints {
            result.push(endpoint?);
        }

        Ok(result)
    }

    pub fn get_all_findings(&self) -> Result<Vec<Finding>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, endpoint_id, severity, title, description, evidence, discovered_at FROM findings"
        )?;

        let findings = stmt.query_map([], |row| {
            let id_str: String = row.get(0)?;
            let endpoint_id_str: String = row.get(1)?;
            let severity_json: String = row.get(2)?;
            let evidence_json: String = row.get(5)?;

            Ok(Finding {
                id: uuid::Uuid::parse_str(&id_str).unwrap(),
                endpoint_id: uuid::Uuid::parse_str(&endpoint_id_str).unwrap(),
                severity: serde_json::from_str(&severity_json).unwrap(),
                title: row.get(3)?,
                description: row.get(4)?,
                evidence: serde_json::from_str(&evidence_json).unwrap(),
                discovered_at: chrono::DateTime::parse_from_rfc3339(&row.get::<_, String>(6)?)
                    .unwrap()
                    .with_timezone(&chrono::Utc),
            })
        })?;

        let mut result = Vec::new();
        for finding in findings {
            result.push(finding?);
        }

        Ok(result)
    }

    pub fn count_endpoints(&self) -> Result<usize> {
        let count: i64 = self
            .conn
            .query_row("SELECT COUNT(*) FROM endpoints", [], |row| row.get(0))?;
        Ok(count as usize)
    }

    #[allow(dead_code)]
    pub fn count_findings(&self) -> Result<usize> {
        let count: i64 = self
            .conn
            .query_row("SELECT COUNT(*) FROM findings", [], |row| row.get(0))?;
        Ok(count as usize)
    }

    pub fn export(&self, format: &str, output_path: &Path) -> Result<()> {
        match format {
            "json" => self.export_json(output_path),
            "csv" => self.export_csv(output_path),
            "markdown" => self.export_markdown(output_path),
            _ => Err(anyhow::anyhow!("Unsupported export format: {}", format)),
        }
    }

    fn export_json(&self, output_path: &Path) -> Result<()> {
        let findings = self.get_all_findings()?;
        let json = serde_json::to_string_pretty(&findings)?;
        std::fs::write(output_path, json)?;
        info!("Exported {} findings to {:?}", findings.len(), output_path);
        Ok(())
    }

    fn export_csv(&self, output_path: &Path) -> Result<()> {
        let findings = self.get_all_findings()?;
        let mut csv = String::from("ID,Endpoint ID,Severity,Title,Description,Discovered At\n");

        for finding in findings {
            csv.push_str(&format!(
                "{},{},{:?},{},{},{}\n",
                finding.id,
                finding.endpoint_id,
                finding.severity,
                finding.title.replace(',', ";"),
                finding.description.replace(',', ";"),
                finding.discovered_at.to_rfc3339(),
            ));
        }

        std::fs::write(output_path, csv)?;
        Ok(())
    }

    fn export_markdown(&self, output_path: &Path) -> Result<()> {
        let findings = self.get_all_findings()?;
        let mut md = String::from("# Bug Bounty Findings\n\n");

        for finding in findings {
            md.push_str(&format!("## {}\n\n", finding.title));
            md.push_str(&format!("**Severity:** {:?}\n\n", finding.severity));
            md.push_str(&format!("**Description:** {}\n\n", finding.description));
            md.push_str(&format!(
                "**Discovered:** {}\n\n",
                finding.discovered_at.to_rfc3339()
            ));
            md.push_str("---\n\n");
        }

        std::fs::write(output_path, md)?;
        Ok(())
    }
}
