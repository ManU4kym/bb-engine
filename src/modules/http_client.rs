use anyhow::{Context, Result};
use governor::{
    Quota, RateLimiter, clock::DefaultClock, state::InMemoryState, state::direct::NotKeyed,
};
use reqwest::{Client, Method};
use std::num::NonZeroU32;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tracing::{debug, warn};

use crate::config::Config;
use crate::modules::types::{HttpRequest, HttpResponse};

pub struct HttpClient {
    client: Client,
    rate_limiter: Arc<RateLimiter<NotKeyed, InMemoryState, DefaultClock>>,
    #[allow(dead_code)]
    config: Config,
}

impl HttpClient {
    pub fn new(config: Config) -> Result<Self> {
        let mut headers = reqwest::header::HeaderMap::new();

        // Add custom headers from config
        for (key, value) in &config.http.headers {
            headers.insert(
                reqwest::header::HeaderName::from_bytes(key.as_bytes())?,
                reqwest::header::HeaderValue::from_str(value)?,
            );
        }

        let client = Client::builder()
            .timeout(Duration::from_secs(config.http.timeout))
            .user_agent(&config.http.user_agent)
            .default_headers(headers)
            .redirect(if config.http.follow_redirects {
                reqwest::redirect::Policy::limited(config.http.max_redirects)
            } else {
                reqwest::redirect::Policy::none()
            })
            .build()?;

        // Create rate limiter
        let quota = Quota::per_second(NonZeroU32::new(config.http.rate_limit).unwrap());
        let rate_limiter = Arc::new(RateLimiter::direct(quota));

        Ok(Self {
            client,
            rate_limiter,
            config,
        })
    }

    pub async fn send_request(
        &self,
        method: Method,
        url: &str,
        headers: Option<Vec<(String, String)>>,
        body: Option<String>,
    ) -> Result<(HttpRequest, HttpResponse)> {
        // Wait for rate limiter
        self.rate_limiter.until_ready().await;

        let start = Instant::now();
        let request_id = uuid::Uuid::new_v4();

        // Build request
        let mut request_builder = self.client.request(method.clone(), url);

        // Add custom headers
        if let Some(headers) = &headers {
            for (key, value) in headers {
                request_builder = request_builder.header(key, value);
            }
        }

        // Add body if present
        if let Some(body) = &body {
            request_builder = request_builder.body(body.clone());
        }

        debug!("Sending {} request to {}", method, url);

        // Send request
        let response = request_builder
            .send()
            .await
            .context("Failed to send HTTP request")?;

        let duration = start.elapsed();

        // Build HttpRequest record
        let http_request = HttpRequest {
            id: request_id,
            url: url.to_string(),
            method: method.to_string(),
            headers: headers.unwrap_or_default(),
            body: body.clone(),
            timestamp: chrono::Utc::now(),
        };

        // Extract response details
        let status = response.status().as_u16();
        let response_headers: Vec<(String, String)> = response
            .headers()
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or("").to_string()))
            .collect();

        let response_body = response.text().await.unwrap_or_else(|e| {
            warn!("Failed to read response body: {}", e);
            String::new()
        });

        // Build HttpResponse record
        let http_response = HttpResponse {
            id: uuid::Uuid::new_v4(),
            request_id,
            status_code: status,
            headers: response_headers,
            body: response_body,
            duration_ms: duration.as_millis() as u64,
            timestamp: chrono::Utc::now(),
        };

        debug!(
            "Received response: {} ({}ms)",
            http_response.status_code, http_response.duration_ms
        );

        Ok((http_request, http_response))
    }

    pub async fn get(&self, url: &str) -> Result<(HttpRequest, HttpResponse)> {
        self.send_request(Method::GET, url, None, None).await
    }

    #[allow(dead_code)]
    pub async fn post(
        &self,
        url: &str,
        headers: Option<Vec<(String, String)>>,
        body: Option<String>,
    ) -> Result<(HttpRequest, HttpResponse)> {
        self.send_request(Method::POST, url, headers, body).await
    }
}
