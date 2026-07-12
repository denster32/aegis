use crate::token::TokenSource;
use crate::types::*;
use futures::StreamExt;
use reqwest::header::{AUTHORIZATION, CONTENT_TYPE};
use std::sync::Arc;
use std::time::Duration;
use tracing::{debug, warn};

const DEFAULT_BASE: &str = "https://api.x.ai/v1";

#[derive(Clone)]
pub struct ResponsesClient {
    http: reqwest::Client,
    tokens: Arc<dyn TokenSource>,
    base_url: String,
    max_retries: u32,
}

impl ResponsesClient {
    pub fn new(tokens: Arc<dyn TokenSource>) -> anyhow::Result<Self> {
        let http = reqwest::Client::builder()
            .timeout(Duration::from_secs(3600))
            .connect_timeout(Duration::from_secs(30))
            .pool_max_idle_per_host(4)
            .build()?;
        Ok(Self {
            http,
            tokens,
            base_url: DEFAULT_BASE.into(),
            max_retries: 5,
        })
    }

    pub fn with_base_url(mut self, base: impl Into<String>) -> Self {
        self.base_url = base.into();
        self
    }

    pub fn with_max_retries(mut self, n: u32) -> Self {
        self.max_retries = n;
        self
    }

    async fn auth_header(&self) -> Result<String, XaiError> {
        let t = self
            .tokens
            .bearer_token()
            .await
            .map_err(|e| XaiError::Auth(e.to_string()))?;
        Ok(format!("Bearer {t}"))
    }

    /// Non-streaming create.
    pub async fn create(&self, req: CreateResponseRequest) -> Result<ResponseObject, XaiError> {
        let mut attempt = 0u32;
        let mut refreshed = false;
        loop {
            attempt += 1;
            let url = format!("{}/responses", self.base_url.trim_end_matches('/'));
            debug!(model = %req.model, attempt, "POST /responses");

            let auth = self.auth_header().await?;
            let resp = self
                .http
                .post(&url)
                .header(AUTHORIZATION, auth)
                .header(CONTENT_TYPE, "application/json")
                .json(&req)
                .send()
                .await?;

            let status = resp.status();
            if status.as_u16() == 429 {
                let retry_ms = resp
                    .headers()
                    .get("retry-after")
                    .and_then(|v| v.to_str().ok())
                    .and_then(|s| s.parse::<u64>().ok())
                    .map(|s| s * 1000)
                    .or(Some(1000 * attempt as u64));
                if attempt <= self.max_retries {
                    let wait = retry_ms.unwrap_or(1000);
                    warn!(wait_ms = wait, "rate limited, retrying");
                    tokio::time::sleep(Duration::from_millis(wait)).await;
                    continue;
                }
                return Err(XaiError::RateLimited {
                    retry_after_ms: retry_ms,
                });
            }

            if status.as_u16() == 401 || status.as_u16() == 403 {
                let body = resp.text().await.unwrap_or_default();
                // One forced refresh then retry
                if !refreshed && status.as_u16() == 401 {
                    refreshed = true;
                    if self.tokens.on_unauthorized().await.is_ok() {
                        warn!("401 — refreshed credentials, retrying");
                        continue;
                    }
                }
                return Err(XaiError::Auth(body));
            }

            if !status.is_success() {
                let body = resp.text().await.unwrap_or_default();
                if status.is_server_error() && attempt <= self.max_retries {
                    warn!(%status, attempt, "server error, retrying");
                    tokio::time::sleep(Duration::from_millis(500 * attempt as u64)).await;
                    continue;
                }
                return Err(XaiError::Http {
                    status: status.as_u16(),
                    body,
                });
            }

            let obj: ResponseObject = resp.json().await?;
            return Ok(obj);
        }
    }

    /// Stream with live text deltas via callback; returns final ResponseObject.
    pub async fn create_stream_with_callback<F>(
        &self,
        mut req: CreateResponseRequest,
        mut on_delta: F,
    ) -> Result<ResponseObject, XaiError>
    where
        F: FnMut(StreamEvent) + Send,
    {
        req.stream = Some(true);
        let url = format!("{}/responses", self.base_url.trim_end_matches('/'));
        let auth = self.auth_header().await?;

        let resp = self
            .http
            .post(&url)
            .header(AUTHORIZATION, auth)
            .header(CONTENT_TYPE, "application/json")
            .json(&req)
            .send()
            .await?;

        let status = resp.status();
        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            if status.as_u16() == 400 || status.as_u16() == 404 || status.as_u16() == 422 {
                warn!("stream unsupported ({status}), falling back to non-stream");
                req.stream = Some(false);
                let obj = self.create(req).await?;
                let text = obj.output_text();
                if !text.is_empty() {
                    on_delta(StreamEvent::TextDelta(text));
                }
                on_delta(StreamEvent::Done(obj.clone()));
                return Ok(obj);
            }
            return Err(XaiError::Http {
                status: status.as_u16(),
                body,
            });
        }

        let mut buf = String::new();
        let mut completed: Option<ResponseObject> = None;
        let mut stream = resp.bytes_stream();

        while let Some(chunk) = stream.next().await {
            let chunk = chunk.map_err(|e| XaiError::Other(e.to_string()))?;
            buf.push_str(&String::from_utf8_lossy(&chunk));

            // Process complete SSE lines
            while let Some(pos) = buf.find('\n') {
                let line = buf[..pos].trim_end_matches('\r').to_string();
                buf.drain(..=pos);

                if line.is_empty() || line.starts_with(':') {
                    continue;
                }
                let data = if let Some(d) = line.strip_prefix("data: ") {
                    d
                } else if let Some(d) = line.strip_prefix("data:") {
                    d.trim()
                } else {
                    continue;
                };
                if data == "[DONE]" {
                    continue;
                }
                if let Ok(v) = serde_json::from_str::<serde_json::Value>(data) {
                    let etype = v.get("type").and_then(|t| t.as_str()).unwrap_or("");

                    // Prefer explicit delta event types only (avoid double-printing)
                    if etype.contains("output_text.delta")
                        || etype == "response.output_text.delta"
                        || etype.ends_with("text.delta")
                    {
                        if let Some(delta) = v
                            .pointer("/delta")
                            .and_then(|x| x.as_str())
                            .or_else(|| v.get("delta").and_then(|x| x.as_str()))
                        {
                            if !delta.is_empty() {
                                on_delta(StreamEvent::TextDelta(delta.to_string()));
                            }
                        }
                    }

                    // Completed full response
                    if v.get("output").is_some() && v.get("id").is_some() && etype.is_empty() {
                        if let Ok(obj) = serde_json::from_value::<ResponseObject>(v.clone()) {
                            completed = Some(obj);
                        }
                    }
                    if etype == "response.completed" {
                        if let Some(resp_v) = v.get("response") {
                            if let Ok(obj) =
                                serde_json::from_value::<ResponseObject>(resp_v.clone())
                            {
                                completed = Some(obj);
                            }
                        }
                    }
                    // Some servers send the final object as type response.done
                    if etype == "response.done" || etype == "response" {
                        let src = v.get("response").cloned().unwrap_or(v.clone());
                        if src.get("output").is_some() && src.get("id").is_some() {
                            if let Ok(obj) = serde_json::from_value::<ResponseObject>(src) {
                                completed = Some(obj);
                            }
                        }
                    }
                }
            }
        }

        if let Some(obj) = completed {
            on_delta(StreamEvent::Done(obj.clone()));
            return Ok(obj);
        }

        warn!("stream ended without completed object; non-stream fallback");
        req.stream = Some(false);
        let obj = self.create(req).await?;
        on_delta(StreamEvent::Done(obj.clone()));
        Ok(obj)
    }

    pub async fn create_stream(
        &self,
        req: CreateResponseRequest,
    ) -> Result<ResponseObject, XaiError> {
        self.create_stream_with_callback(req, |_| {}).await
    }
}

#[derive(Debug, Clone)]
pub enum StreamEvent {
    TextDelta(String),
    ToolCall {
        call_id: String,
        name: String,
        arguments: String,
    },
    Done(ResponseObject),
    Error(String),
}
