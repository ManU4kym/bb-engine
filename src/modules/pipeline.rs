use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::Path;
use tokio::fs;
use tracing::info;

use crate::config::Config;
use crate::modules::{discovery::EndpointDiscovery, fuzzer::Fuzzer, pattern::PatternMatcher};

#[derive(Debug, Serialize, Deserialize)]
pub struct WorkflowConfig {
    pub name: String,
    pub description: String,
    pub stages: Vec<Stage>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Stage {
    pub name: String,
    pub stage_type: StageType,
    pub config: StageConfig,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum StageType {
    Discovery,
    Fuzzing,
    PatternMatching,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StageConfig {
    // Discovery settings
    pub wordlist: Option<String>,
    pub max_depth: Option<usize>,

    // Fuzzing settings
    pub fuzzing_mode: Option<String>,
    pub payload_file: Option<String>,

    // Pattern matching settings
    pub pattern_file: Option<String>,
}

pub struct Pipeline {
    config: Config,
}

impl Pipeline {
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    pub async fn run(&self, target: &str, workflow_path: &Path) -> Result<()> {
        info!("Loading workflow from {:?}", workflow_path);

        let workflow = self.load_workflow(workflow_path).await?;
        info!("Starting workflow: {}", workflow.name);
        info!("Description: {}", workflow.description);

        for (idx, stage) in workflow.stages.iter().enumerate() {
            info!("========================================");
            info!(
                "Stage {}/{}: {}",
                idx + 1,
                workflow.stages.len(),
                stage.name
            );
            info!("========================================");

            self.execute_stage(target, stage).await?;
        }

        info!("Workflow completed successfully");
        Ok(())
    }

    async fn load_workflow(&self, path: &Path) -> Result<WorkflowConfig> {
        let content = fs::read_to_string(path).await?;
        let workflow: WorkflowConfig = serde_yaml::from_str(&content)?;
        Ok(workflow)
    }

    async fn execute_stage(&self, target: &str, stage: &Stage) -> Result<()> {
        match stage.stage_type {
            StageType::Discovery => {
                let mut discovery = EndpointDiscovery::new(self.config.clone());

                let wordlist = stage.config.wordlist.as_ref().map(|s| Path::new(s));
                let max_depth = stage.config.max_depth.unwrap_or(3);

                discovery.discover(target, wordlist, max_depth).await?;
            }

            StageType::Fuzzing => {
                let fuzzer = Fuzzer::new(self.config.clone());

                let mode = stage
                    .config
                    .fuzzing_mode
                    .as_ref()
                    .ok_or_else(|| anyhow::anyhow!("Fuzzing mode not specified"))?;

                let payload_file = stage.config.payload_file.as_ref().map(|s| Path::new(s));

                fuzzer.fuzz(target, mode, payload_file).await?;
            }

            StageType::PatternMatching => {
                let matcher = PatternMatcher::new(self.config.clone());

                let pattern_file = stage
                    .config
                    .pattern_file
                    .as_ref()
                    .ok_or_else(|| anyhow::anyhow!("Pattern file not specified"))?;

                let db_path = Path::new(&self.config.storage.db_path);
                matcher
                    .match_patterns(db_path, Path::new(pattern_file))
                    .await?;
            }
        }

        Ok(())
    }

    #[allow(dead_code)]
    pub fn create_example_workflow() -> WorkflowConfig {
        WorkflowConfig {
            name: "Full Bug Bounty Scan".to_string(),
            description: "Complete automated bug bounty workflow with discovery, fuzzing, and pattern matching".to_string(),
            stages: vec![
                Stage {
                    name: "Endpoint Discovery".to_string(),
                    stage_type: StageType::Discovery,
                    config: StageConfig {
                        wordlist: Some("wordlist.txt".to_string()),
                        max_depth: Some(3),
                        fuzzing_mode: None,
                        payload_file: None,
                        pattern_file: None,
                    },
                },
                Stage {
                    name: "SQL Injection Fuzzing".to_string(),
                    stage_type: StageType::Fuzzing,
                    config: StageConfig {
                        wordlist: None,
                        max_depth: None,
                        fuzzing_mode: Some("sqli".to_string()),
                        payload_file: None,
                        pattern_file: None,
                    },
                },
                Stage {
                    name: "XSS Fuzzing".to_string(),
                    stage_type: StageType::Fuzzing,
                    config: StageConfig {
                        wordlist: None,
                        max_depth: None,
                        fuzzing_mode: Some("xss".to_string()),
                        payload_file: None,
                        pattern_file: None,
                    },
                },
                Stage {
                    name: "Pattern Matching for Secrets".to_string(),
                    stage_type: StageType::PatternMatching,
                    config: StageConfig {
                        wordlist: None,
                        max_depth: None,
                        fuzzing_mode: None,
                        payload_file: None,
                        pattern_file: Some("patterns.yaml".to_string()),
                    },
                },
            ],
        }
    }
}
