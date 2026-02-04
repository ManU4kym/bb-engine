use anyhow::Result;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::path::Path;
use tokio::fs;
use tracing::info;

use crate::config::Config;
use crate::modules::storage::Storage;
use crate::modules::types::{Evidence, Finding, Severity};

#[derive(Debug, Serialize, Deserialize)]
pub struct PatternConfig {
    pub patterns: Vec<Pattern>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pattern {
    pub name: String,
    pub description: String,
    pub regex: String,
    pub severity: Severity,
    pub match_type: MatchType,
    pub context_lines: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MatchType {
    /// Match in response body
    Body,
    /// Match in response headers
    Headers,
    /// Match in both body and headers
    Both,
    /// Match in request
    Request,
}

pub struct PatternMatcher {
    #[allow(dead_code)]
    config: Config,
}

impl PatternMatcher {
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    pub async fn match_patterns(&self, db_path: &Path, patterns_path: &Path) -> Result<()> {
        info!("Loading patterns from {:?}", patterns_path);

        let patterns = self.load_patterns(patterns_path).await?;
        info!("Loaded {} patterns", patterns.len());

        let storage = Storage::new(db_path)?;

        // Get all responses from database
        let responses = self.get_all_responses_from_db(&storage)?;
        info!("Analyzing {} responses", responses.len());

        let mut findings_count = 0;

        for response in responses {
            for pattern in &patterns {
                if let Some(finding) = self.match_pattern(&pattern, &response, &storage)? {
                    storage.save_finding(&finding)?;
                    findings_count += 1;
                    info!("MATCH: {} - {}", pattern.name, finding.title);
                }
            }
        }

        info!(
            "Pattern matching complete. Found {} matches",
            findings_count
        );
        Ok(())
    }

    async fn load_patterns(&self, path: &Path) -> Result<Vec<Pattern>> {
        let content = fs::read_to_string(path).await?;
        let config: PatternConfig = serde_yaml::from_str(&content)?;
        Ok(config.patterns)
    }

    fn get_all_responses_from_db(&self, _storage: &Storage) -> Result<Vec<ResponseWithRequest>> {
        // This would need to be implemented in the Storage module
        // For now, we'll return a placeholder
        Ok(Vec::new())
    }

    fn match_pattern(
        &self,
        pattern: &Pattern,
        response: &ResponseWithRequest,
        _storage: &Storage,
    ) -> Result<Option<Finding>> {
        let regex = Regex::new(&pattern.regex)?;

        let text_to_match = match pattern.match_type {
            MatchType::Body => &response.response_body,
            MatchType::Headers => &response.response_headers,
            MatchType::Both => {
                &format!("{}\n{}", response.response_headers, response.response_body)
            }
            MatchType::Request => &response.request_text,
        };

        if let Some(captures) = regex.captures(text_to_match) {
            let matched_text = captures.get(0).map(|m| m.as_str()).unwrap_or("");

            // Extract context if specified
            let context = if let Some(lines) = pattern.context_lines {
                self.extract_context(text_to_match, matched_text, lines)
            } else {
                matched_text.to_string()
            };

            let description = format!(
                "{}\n\nMatched pattern: {}\nContext:\n{}",
                pattern.description, pattern.regex, context
            );

            // Create finding with evidence
            let evidence = Evidence {
                request: response.request.clone(),
                response: response.response.clone(),
                matched_pattern: Some(pattern.regex.clone()),
                notes: vec![
                    format!("Pattern: {}", pattern.name),
                    format!("Matched text: {}", matched_text),
                ],
            };

            let finding = Finding::new(
                response.endpoint_id,
                pattern.severity.clone(),
                pattern.name.clone(),
                description,
                evidence,
            );

            return Ok(Some(finding));
        }

        Ok(None)
    }

    fn extract_context(&self, text: &str, matched: &str, lines: usize) -> String {
        let lines_vec: Vec<&str> = text.lines().collect();

        // Find the line containing the match
        if let Some(match_line_idx) = lines_vec.iter().position(|line| line.contains(matched)) {
            let start = match_line_idx.saturating_sub(lines);
            let end = (match_line_idx + lines + 1).min(lines_vec.len());

            lines_vec[start..end].join("\n")
        } else {
            matched.to_string()
        }
    }

    #[allow(dead_code)]
    pub fn create_example_pattern_file() -> PatternConfig {
        PatternConfig {
            patterns: vec![
                Pattern {
                    name: "AWS Access Key".to_string(),
                    description: "Potential AWS access key found in response".to_string(),
                    regex: r"AKIA[0-9A-Z]{16}".to_string(),
                    severity: Severity::Critical,
                    match_type: MatchType::Body,
                    context_lines: Some(2),
                },
                Pattern {
                    name: "Private Key".to_string(),
                    description: "Private key found in response".to_string(),
                    regex: r"-----BEGIN (RSA |EC |DSA |OPENSSH )?PRIVATE KEY-----".to_string(),
                    severity: Severity::Critical,
                    match_type: MatchType::Body,
                    context_lines: Some(5),
                },
                Pattern {
                    name: "JWT Token".to_string(),
                    description: "JWT token found in response".to_string(),
                    regex: r"eyJ[A-Za-z0-9_-]*\.eyJ[A-Za-z0-9_-]*\.[A-Za-z0-9_-]*".to_string(),
                    severity: Severity::Medium,
                    match_type: MatchType::Both,
                    context_lines: None,
                },
                Pattern {
                    name: "SQL Error".to_string(),
                    description: "SQL error message indicating potential SQL injection".to_string(),
                    regex: r"(SQL syntax.*?MySQL|Warning.*?mysql_|MySQLSyntaxErrorException|PostgreSQL.*?ERROR|PG::SyntaxError)".to_string(),
                    severity: Severity::High,
                    match_type: MatchType::Body,
                    context_lines: Some(3),
                },
                Pattern {
                    name: "Stack Trace".to_string(),
                    description: "Stack trace found in response".to_string(),
                    regex: r"(at\s+[\w\.\$]+\([^\)]+\)|Traceback \(most recent call last\)|Exception in thread)".to_string(),
                    severity: Severity::Medium,
                    match_type: MatchType::Body,
                    context_lines: Some(5),
                },
                Pattern {
                    name: "Email Address".to_string(),
                    description: "Email address found in response".to_string(),
                    regex: r"[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}".to_string(),
                    severity: Severity::Low,
                    match_type: MatchType::Body,
                    context_lines: None,
                },
                Pattern {
                    name: "API Key Generic".to_string(),
                    description: "Generic API key pattern found".to_string(),
                    regex: r#"(?i)(api[_-]?key|apikey|access[_-]?token|auth[_-]?token)['":=\s]+([a-zA-Z0-9_\-]{20,})"#.to_string(),
                    severity: Severity::High,
                    match_type: MatchType::Body,
                    context_lines: Some(1),
                },
                Pattern {
                    name: "Admin Panel".to_string(),
                    description: "Potential admin panel or sensitive endpoint".to_string(),
                    regex: r"(?i)(admin|administrator|dashboard|panel|console)".to_string(),
                    severity: Severity::Info,
                    match_type: MatchType::Request,
                    context_lines: None,
                },
            ],
        }
    }
}

// Helper struct to combine request and response data
#[derive(Debug, Clone)]
pub struct ResponseWithRequest {
    pub endpoint_id: uuid::Uuid,
    pub request: crate::modules::types::HttpRequest,
    pub response: crate::modules::types::HttpResponse,
    pub request_text: String,
    pub response_body: String,
    pub response_headers: String,
}
