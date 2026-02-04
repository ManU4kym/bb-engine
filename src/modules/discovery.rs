use anyhow::{Context, Result};
use scraper::{Html, Selector};
use std::collections::HashSet;
use std::path::Path;
use tokio::fs;
use tracing::{info, warn};
use url::Url;

use crate::config::Config;
use crate::modules::http_client::HttpClient;
use crate::modules::storage::Storage;
use crate::modules::types::{Endpoint, EndpointSource};

pub struct EndpointDiscovery {
    config: Config,
    http_client: HttpClient,
    #[allow(dead_code)]
    discovered: HashSet<String>,
}

impl EndpointDiscovery {
    pub fn new(config: Config) -> Self {
        let http_client = HttpClient::new(config.clone()).expect("Failed to create HTTP client");

        Self {
            config,
            http_client,
            discovered: HashSet::new(),
        }
    }

    pub async fn discover(
        &mut self,
        target: &str,
        wordlist: Option<&Path>,
        max_depth: usize,
    ) -> Result<()> {
        info!("Starting endpoint discovery for: {}", target);

        let base_url = Url::parse(target).context("Invalid target URL")?;

        // Initialize storage
        let storage = Storage::new(&self.config.storage.db_path)?;

        // 1. Crawl the website
        info!("Crawling website (max depth: {})", max_depth);
        let crawled = self.crawl(&base_url, max_depth).await?;
        info!("Found {} endpoints via crawling", crawled.len());

        for endpoint in crawled {
            storage.save_endpoint(&endpoint)?;
        }

        // 2. Brute force with wordlist if provided
        if let Some(wordlist_path) = wordlist {
            info!("Brute forcing with wordlist: {:?}", wordlist_path);
            let bruted = self.brute_force(&base_url, wordlist_path).await?;
            info!("Found {} endpoints via brute force", bruted.len());

            for endpoint in bruted {
                storage.save_endpoint(&endpoint)?;
            }
        }

        // 3. Analyze JavaScript files for API endpoints
        info!("Analyzing JavaScript files for API endpoints");
        let js_endpoints = self.analyze_js_files(&base_url).await?;
        info!("Found {} endpoints via JS analysis", js_endpoints.len());

        for endpoint in js_endpoints {
            storage.save_endpoint(&endpoint)?;
        }

        let total = storage.count_endpoints()?;
        info!("Total unique endpoints discovered: {}", total);

        Ok(())
    }

    async fn crawl(&mut self, base_url: &Url, max_depth: usize) -> Result<Vec<Endpoint>> {
        let mut endpoints = Vec::new();
        let mut to_visit = vec![(base_url.clone(), 0)];
        let mut visited = HashSet::new();

        while let Some((url, depth)) = to_visit.pop() {
            if depth > max_depth || visited.contains(url.as_str()) {
                continue;
            }

            visited.insert(url.to_string());

            // Fetch the page
            match self.http_client.get(url.as_str()).await {
                Ok((_, resp)) => {
                    // Only process successful responses
                    if !self
                        .config
                        .discovery
                        .success_codes
                        .contains(&resp.status_code)
                    {
                        continue;
                    }

                    // Save as endpoint
                    let endpoint =
                        Endpoint::new(url.to_string(), "GET".to_string(), EndpointSource::Crawled);
                    endpoints.push(endpoint);

                    // Parse HTML and extract links
                    if resp.body.contains("<html") {
                        let links = self.extract_links(&resp.body, &url)?;

                        for link in links {
                            if link.domain() == base_url.domain() {
                                to_visit.push((link, depth + 1));
                            }
                        }
                    }
                }
                Err(e) => {
                    warn!("Failed to fetch {}: {}", url, e);
                }
            }
        }

        Ok(endpoints)
    }

    fn extract_links(&self, html: &str, base_url: &Url) -> Result<Vec<Url>> {
        let mut links = Vec::new();
        let document = Html::parse_document(html);

        // Extract from <a> tags
        if let Ok(selector) = Selector::parse("a[href]") {
            for element in document.select(&selector) {
                if let Some(href) = element.value().attr("href") {
                    if let Ok(absolute_url) = base_url.join(href) {
                        links.push(absolute_url);
                    }
                }
            }
        }

        // Extract from <form> tags
        if let Ok(selector) = Selector::parse("form[action]") {
            for element in document.select(&selector) {
                if let Some(action) = element.value().attr("action") {
                    if let Ok(absolute_url) = base_url.join(action) {
                        links.push(absolute_url);
                    }
                }
            }
        }

        Ok(links)
    }

    async fn brute_force(&mut self, base_url: &Url, wordlist_path: &Path) -> Result<Vec<Endpoint>> {
        let mut endpoints = Vec::new();
        let wordlist = fs::read_to_string(wordlist_path).await?;

        for word in wordlist.lines() {
            let word = word.trim();
            if word.is_empty() || word.starts_with('#') {
                continue;
            }

            // Try different extensions
            for ext in &self.config.discovery.extensions {
                let path = format!("{}.{}", word, ext);
                let url = base_url.join(&path)?;

                match self.http_client.get(url.as_str()).await {
                    Ok((_, resp)) => {
                        if self
                            .config
                            .discovery
                            .success_codes
                            .contains(&resp.status_code)
                        {
                            info!("Found: {} ({})", url, resp.status_code);
                            let endpoint = Endpoint::new(
                                url.to_string(),
                                "GET".to_string(),
                                EndpointSource::Wordlist,
                            );
                            endpoints.push(endpoint);
                        }
                    }
                    Err(_) => continue,
                }
            }

            // Try without extension
            let url = base_url.join(word)?;
            match self.http_client.get(url.as_str()).await {
                Ok((_, resp)) => {
                    if self
                        .config
                        .discovery
                        .success_codes
                        .contains(&resp.status_code)
                    {
                        info!("Found: {} ({})", url, resp.status_code);
                        let endpoint = Endpoint::new(
                            url.to_string(),
                            "GET".to_string(),
                            EndpointSource::Wordlist,
                        );
                        endpoints.push(endpoint);
                    }
                }
                Err(_) => continue,
            }
        }

        Ok(endpoints)
    }

    async fn analyze_js_files(&mut self, _base_url: &Url) -> Result<Vec<Endpoint>> {
        let endpoints = Vec::new();

        // This is a simplified version - in production you'd want more sophisticated JS parsing
        // For now, we'll use regex to find potential API endpoints

        let _api_patterns = vec![
            regex::Regex::new(r#"['\`](/api/[^'\'\`\s]+)['\`]"#)?,
            regex::Regex::new(r#"['\`](/v\d+/[^'\'\`\s]+)['\`]"#)?,
            regex::Regex::new(r#"fetch\(['\`]([^'\'\`]+)['\`]\)"#)?,
            regex::Regex::new(r#"axios\.(get|post|put|delete)\(['\`]([^'\'\`]+)['\`]"#)?,
        ];

        // In a real implementation, you'd:
        // 1. Find all .js files from crawling
        // 2. Download and parse each one
        // 3. Extract API endpoints using regex or AST parsing
        // 4. Validate endpoints by making requests

        // Placeholder for now
        info!("JS analysis implementation pending - add sophisticated parsing here");

        Ok(endpoints)
    }
}
