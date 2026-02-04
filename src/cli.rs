use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

use crate::config::Config;
use crate::modules::{
    discovery::EndpointDiscovery, fuzzer::Fuzzer, pattern::PatternMatcher, pipeline::Pipeline,
    storage::Storage,
};

#[derive(Parser)]
#[command(name = "bb-engine")]
#[command(about = "Bug Bounty Automation Engine", long_about = None)]
pub struct Cli {
    /// Configuration file path
    #[arg(short, long, value_name = "FILE")]
    config: Option<PathBuf>,

    /// Verbosity level
    #[arg(short, long, action = clap::ArgAction::Count)]
    verbose: u8,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Discover endpoints from a target
    Discover {
        /// Target URL
        #[arg(short, long)]
        target: String,

        /// Wordlist path for brute forcing
        #[arg(short, long)]
        wordlist: Option<PathBuf>,

        /// Maximum crawl depth
        #[arg(short, long, default_value = "3")]
        depth: usize,
    },

    /// Run the full pipeline on a target
    Pipeline {
        /// Target URL
        #[arg(short, long)]
        target: String,

        /// Workflow configuration file
        #[arg(short, long)]
        workflow: PathBuf,
    },

    /// Fuzz endpoints with payloads
    Fuzz {
        /// Target URL or file with URLs
        #[arg(short, long)]
        target: String,

        /// Fuzzing mode (sqli, xss, idor, etc.)
        #[arg(short, long)]
        mode: String,

        /// Custom payload file
        #[arg(short, long)]
        payloads: Option<PathBuf>,
    },

    /// Match patterns in saved responses
    Match {
        /// Database file with responses
        #[arg(short, long)]
        database: PathBuf,

        /// Pattern configuration file
        #[arg(short, long)]
        patterns: PathBuf,
    },

    /// Export findings to various formats
    Export {
        /// Database file with findings
        #[arg(short, long)]
        database: PathBuf,

        /// Output format (json, csv, markdown)
        #[arg(short, long, default_value = "json")]
        format: String,

        /// Output file path
        #[arg(short, long)]
        output: PathBuf,
    },
}

impl Cli {
    pub async fn execute(&self) -> Result<()> {
        // Load configuration
        let config = if let Some(config_path) = &self.config {
            Config::from_file(config_path)?
        } else {
            Config::default()
        };

        match &self.command {
            Commands::Discover {
                target,
                wordlist,
                depth,
            } => {
                let mut discovery = EndpointDiscovery::new(config.clone());
                discovery
                    .discover(target, wordlist.as_deref(), *depth)
                    .await?;
            }

            Commands::Pipeline { target, workflow } => {
                let pipeline = Pipeline::new(config.clone());
                pipeline.run(target, workflow).await?;
            }

            Commands::Fuzz {
                target,
                mode,
                payloads,
            } => {
                let fuzzer = Fuzzer::new(config.clone());
                fuzzer.fuzz(target, mode, payloads.as_deref()).await?;
            }

            Commands::Match { database, patterns } => {
                let matcher = PatternMatcher::new(config.clone());
                matcher.match_patterns(database, patterns).await?;
            }

            Commands::Export {
                database,
                format,
                output,
            } => {
                let storage = Storage::new(database)?;
                storage.export(format, output)?;
            }
        }

        Ok(())
    }
}
