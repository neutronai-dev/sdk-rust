use std::time::Duration;

use reqwest::{Client as HttpClient, Method};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use serde_json::{Map, Value, json};
use thiserror::Error;

pub mod mcp;

#[derive(Debug, Error)]
pub enum NeutronAIError {
    #[error("nucleus_id is required")]
    MissingNucleus,
    #[error("nucleusId and tenantId must match when both are provided")]
    MismatchedNucleus,
    #[error("base_url is required")]
    MissingBaseUrl,
    #[error("Neutron AI request failed: {status}")]
    Api { status: u16, body: Value },
    #[error(transparent)]
    Http(#[from] reqwest::Error),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
}

#[derive(Clone, Debug)]
pub struct NeutronAIClient {
    base_url: String,
    token: Option<String>,
    nucleus_id: Option<String>,
    tenant_id: Option<String>,
    http: HttpClient,
}

#[derive(Clone, Debug)]
pub struct ClientOptions {
    pub base_url: String,
    pub token: Option<String>,
    pub nucleus_id: Option<String>,
    pub tenant_id: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct HealthResponse {
    pub ok: bool,
    pub service: String,
    pub version: String,
}

impl NeutronAIClient {
    pub fn new(options: ClientOptions) -> Result<Self, NeutronAIError> {
        let base_url = options.base_url.trim().trim_end_matches('/').to_string();
        if base_url.is_empty() {
            return Err(NeutronAIError::MissingBaseUrl);
        }
        if options.nucleus_id.is_some()
            && options.tenant_id.is_some()
            && options.nucleus_id != options.tenant_id
        {
            return Err(NeutronAIError::MismatchedNucleus);
        }

        let http = HttpClient::builder()
            .timeout(Duration::from_secs(30))
            .user_agent("neutron-ai-rust/0.3.0")
            .build()?;
        Ok(Self {
            base_url,
            token: options.token,
            nucleus_id: options.nucleus_id,
            tenant_id: options.tenant_id,
            http,
        })
    }

    async fn request<T: DeserializeOwned>(
        &self,
        method: Method,
        path: &str,
        body: Option<Value>,
    ) -> Result<T, NeutronAIError> {
        let mut request = self
            .http
            .request(method, format!("{}{}", self.base_url, path));
        if let Some(token) = &self.token {
            request = request.bearer_auth(token);
        }
        if let Some(payload) = body {
            request = request.json(&payload);
        }

        let response = request.send().await?;
        let status = response.status();
        let raw_body = response.text().await?;
        let parsed_body =
            serde_json::from_str::<Value>(&raw_body).unwrap_or_else(|_| json!({ "raw": raw_body }));
        if !status.is_success() {
            return Err(NeutronAIError::Api {
                status: status.as_u16(),
                body: parsed_body,
            });
        }
        Ok(serde_json::from_value(parsed_body)?)
    }

    fn with_nucleus(&self, body: Value) -> Result<Value, NeutronAIError> {
        let mut object = match body {
            Value::Object(object) => object,
            _ => Map::new(),
        };
        let body_nucleus_id = object
            .get("nucleusId")
            .or_else(|| object.get("nucleus_id"))
            .and_then(Value::as_str)
            .map(ToOwned::to_owned);
        let body_tenant_id = object
            .get("tenantId")
            .or_else(|| object.get("tenant_id"))
            .and_then(Value::as_str)
            .map(ToOwned::to_owned);
        if body_nucleus_id.is_some()
            && body_tenant_id.is_some()
            && body_nucleus_id != body_tenant_id
        {
            return Err(NeutronAIError::MismatchedNucleus);
        }

        let nucleus_id = body_nucleus_id
            .or(body_tenant_id)
            .or_else(|| self.nucleus_id.clone())
            .or_else(|| self.tenant_id.clone())
            .filter(|value| !value.trim().is_empty())
            .ok_or(NeutronAIError::MissingNucleus)?;
        object.remove("nucleus_id");
        object.remove("tenant_id");
        object.insert("nucleusId".to_string(), Value::String(nucleus_id.clone()));
        object.insert("tenantId".to_string(), Value::String(nucleus_id));
        Ok(Value::Object(object))
    }

    fn configured_nucleus_id(&self) -> Result<&str, NeutronAIError> {
        self.nucleus_id
            .as_deref()
            .or(self.tenant_id.as_deref())
            .filter(|value| !value.trim().is_empty())
            .ok_or(NeutronAIError::MissingNucleus)
    }

    pub async fn health(&self) -> Result<HealthResponse, NeutronAIError> {
        self.request(Method::GET, "/health", None).await
    }

    pub async fn remember(&self, body: Value) -> Result<Value, NeutronAIError> {
        let body = self.with_nucleus(body)?;
        self.request(Method::POST, "/v1/memory/remember", Some(body))
            .await
    }

    pub async fn recall(&self, body: Value) -> Result<Value, NeutronAIError> {
        let body = self.with_nucleus(body)?;
        self.request(Method::POST, "/v1/memory/recall", Some(body))
            .await
    }

    pub async fn agent_context(&self, body: Value) -> Result<Value, NeutronAIError> {
        let body = self.with_nucleus(body)?;
        self.request(Method::POST, "/v1/memory/agent-context", Some(body))
            .await
    }

    pub async fn reflect(&self, body: Value) -> Result<Value, NeutronAIError> {
        let body = self.with_nucleus(body)?;
        self.request(Method::POST, "/v1/memory/reflect", Some(body))
            .await
    }

    pub async fn learn_skill(&self, body: Value) -> Result<Value, NeutronAIError> {
        let body = self.with_nucleus(body)?;
        self.request(Method::POST, "/v1/memory/skills/learn", Some(body))
            .await
    }

    pub async fn upsert_knowledge(&self, body: Value) -> Result<Value, NeutronAIError> {
        let body = self.with_nucleus(body)?;
        self.request(Method::POST, "/v1/memory/kb/upsert", Some(body))
            .await
    }

    pub async fn forget(&self, body: Value) -> Result<Value, NeutronAIError> {
        let body = self.with_nucleus(body)?;
        self.request(Method::POST, "/v1/memory/forget", Some(body))
            .await
    }

    pub async fn compact(&self, body: Value) -> Result<Value, NeutronAIError> {
        let body = self.with_nucleus(body)?;
        self.request(Method::POST, "/v1/memory/compact", Some(body))
            .await
    }

    pub async fn audit(
        &self,
        scope_id: &str,
        limit: Option<u32>,
        since: Option<i64>,
        until: Option<i64>,
    ) -> Result<Value, NeutronAIError> {
        let nucleus_id = percent_encode(self.configured_nucleus_id()?);
        let scope_id = percent_encode(scope_id);
        let mut path = format!(
            "/v1/memory/audit?nucleusId={nucleus_id}&scopeId={scope_id}&limit={}",
            limit.unwrap_or(50)
        );
        if let Some(since) = since {
            path.push_str(&format!("&since={since}"));
        }
        if let Some(until) = until {
            path.push_str(&format!("&until={until}"));
        }
        self.request(Method::GET, &path, None).await
    }

    pub async fn cache_stats(&self) -> Result<Value, NeutronAIError> {
        self.cache_stats_for_agent(None).await
    }

    pub async fn cache_stats_for_agent(
        &self,
        agent_id: Option<&str>,
    ) -> Result<Value, NeutronAIError> {
        let nucleus_id = percent_encode(self.configured_nucleus_id()?);
        let mut path = format!("/v1/memory/cache/stats?nucleusId={nucleus_id}");
        if let Some(agent_id) = agent_id {
            path.push_str("&agentId=");
            path.push_str(&percent_encode(agent_id));
        }
        self.request(Method::GET, &path, None).await
    }

    pub async fn invalidate_cache(&self, body: Value) -> Result<Value, NeutronAIError> {
        let body = self.with_nucleus(body)?;
        self.request(Method::POST, "/v1/memory/cache/invalidate", Some(body))
            .await
    }

    pub async fn warm_cache(&self, body: Value) -> Result<Value, NeutronAIError> {
        let body = self.with_nucleus(body)?;
        self.request(Method::POST, "/v1/memory/cache/warm", Some(body))
            .await
    }

    pub async fn list_mcp_tools(&self) -> Result<Value, NeutronAIError> {
        self.request(Method::GET, "/mcp/tools", None).await
    }
}

fn percent_encode(value: &str) -> String {
    let mut encoded = String::with_capacity(value.len());
    for byte in value.bytes() {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'.' | b'_' | b'~') {
            encoded.push(char::from(byte));
        } else {
            encoded.push_str(&format!("%{byte:02X}"));
        }
    }
    encoded
}

#[cfg(test)]
mod tests {
    use super::{ClientOptions, NeutronAIClient, NeutronAIError, percent_encode};
    use serde_json::json;

    #[test]
    fn injects_nucleus_and_legacy_alias() {
        let client = NeutronAIClient::new(ClientOptions {
            base_url: "https://api.example.test".to_string(),
            token: None,
            nucleus_id: Some("nucleus_demo".to_string()),
            tenant_id: None,
        })
        .expect("client should be valid");

        let body = client
            .with_nucleus(json!({ "scopeId": "user:derek" }))
            .expect("body should be scoped");
        assert_eq!(body["nucleusId"], "nucleus_demo");
        assert_eq!(body["tenantId"], "nucleus_demo");
    }

    #[test]
    fn rejects_mismatched_nucleus_aliases() {
        let result = NeutronAIClient::new(ClientOptions {
            base_url: "https://api.example.test".to_string(),
            token: None,
            nucleus_id: Some("nucleus_a".to_string()),
            tenant_id: Some("nucleus_b".to_string()),
        });
        assert!(matches!(result, Err(NeutronAIError::MismatchedNucleus)));
    }

    #[test]
    fn encodes_query_values() {
        assert_eq!(
            percent_encode("team:release notes"),
            "team%3Arelease%20notes"
        );
    }
}
