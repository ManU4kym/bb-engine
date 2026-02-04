use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub http: HttpConfig,
    pub discovery: DiscoveryConfig,
    pub fuzzing: FuzzingConfig,
    pub storage: StorageConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpConfig {
    /// Maximum concurrent requests
    pub max_concurrent: usize,

    /// Request timeout in seconds
    pub timeout: u64,

    /// Requests per second limit
    pub rate_limit: u32,

    /// Custom headers
    pub headers: Vec<(String, String)>,

    /// User agent
    pub user_agent: String,

    /// Follow redirects
    pub follow_redirects: bool,

    /// Maximum redirects to follow
    pub max_redirects: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveryConfig {
    /// Extensions to look for
    pub extensions: Vec<String>,

    /// Directories to ignore
    pub ignore_dirs: Vec<String>,

    /// Status codes to consider success
    pub success_codes: Vec<u16>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FuzzingConfig {
    /// Generate payloads or use file
    pub auto_generate: bool,

    /// Number of payloads per parameter
    pub payloads_per_param: usize,

    /// Delay between requests in milliseconds
    pub delay_ms: u64,

    /// Similarity threshold for duplicate detection (0.0 - 1.0)
    pub similarity_threshold: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    /// Database path
    pub db_path: String,

    /// Auto-save interval in seconds
    pub auto_save_interval: u64,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            http: HttpConfig {
                max_concurrent: 10,
                timeout: 30,
                rate_limit: 10,
                headers: vec![],
                user_agent: "bb-engine/0.1.0".to_string(),
                follow_redirects: true,
                max_redirects: 5,
            },
            discovery: DiscoveryConfig {
                extensions: vec![
                    "php".to_string(),
                    "asp".to_string(),
                    "aspx".to_string(),
                    "jsp".to_string(),
                    "json".to_string(),
                    "xml".to_string(),
                ],
                ignore_dirs: vec![
                    "node_modules".to_string(),
                    ".git".to_string(),
                    "vendor".to_string(),
                ],
                success_codes: vec![200, 201, 202, 204, 301, 302, 307, 308],
            },
            fuzzing: FuzzingConfig {
                auto_generate: true,
                payloads_per_param: 50,
                delay_ms: 100,
                similarity_threshold: 0.85,
            },
            storage: StorageConfig {
                db_path: "bb-engine.db".to_string(),
                auto_save_interval: 60,
            },
        }
    }
}

impl Config {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let contents = fs::read_to_string(path)?;
        let config: Config = serde_yaml::from_str(&contents)?;
        Ok(config)
    }

    #[allow(dead_code)]
    pub fn to_file<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let yaml = serde_yaml::to_string(self)?;
        fs::write(path, yaml)?;
        Ok(())
    }
}
