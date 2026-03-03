use anyhow::{Result, anyhow};
use reqwest::Client;
use tokio::sync::Mutex;

use crate::types::*;

pub(crate) struct ArenaClient {
    http: Client,
    base_url: String,
    email: String,
    password: String,
    workspace_id: Option<i64>,
    session: Mutex<Option<String>>,
}

impl ArenaClient {
    pub(crate) fn from_env() -> Result<Self> {
        let email = std::env::var("ARENA_EMAIL")
            .map_err(|_| anyhow!("ARENA_EMAIL environment variable not set"))?;
        let password = std::env::var("ARENA_PASSWORD")
            .map_err(|_| anyhow!("ARENA_PASSWORD environment variable not set"))?;
        let workspace_id = std::env::var("ARENA_WORKSPACE_ID")
            .ok()
            .and_then(|value| value.parse().ok());
        let base_url = std::env::var("ARENA_BASE_URL")
            .unwrap_or_else(|_| "https://api.arenasolutions.com/v1".to_string());

        Ok(Self {
            http: Client::new(),
            base_url,
            email,
            password,
            workspace_id,
            session: Mutex::new(None),
        })
    }

    async fn login(&self) -> Result<String> {
        let url = format!("{}/login", self.base_url);
        let body = LoginRequest {
            email: self.email.clone(),
            password: self.password.clone(),
            workspace_id: self.workspace_id,
        };
        let response = self
            .http
            .post(&url)
            .json(&body)
            .send()
            .await?;
        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(anyhow!("Login failed ({}): {}", status, text));
        }
        let login: LoginResponse = response.json().await?;
        Ok(login.arena_session_id)
    }

    async fn ensure_session(&self) -> Result<String> {
        let mut session = self.session.lock().await;
        if let Some(token) = session.as_ref() {
            return Ok(token.clone());
        }
        let token = self.login().await?;
        *session = Some(token.clone());
        Ok(token)
    }

    async fn invalidate_session(&self, stale_token: &str) {
        let mut session = self.session.lock().await;
        if session.as_deref() == Some(stale_token) {
            *session = None;
        }
    }

    async fn get(&self, path: &str, query: &[(String, String)]) -> Result<serde_json::Value> {
        let url = format!("{}{}", self.base_url, path);
        let token = self.ensure_session().await?;

        let response = self
            .http
            .get(&url)
            .header("arena_session_id", &token)
            .header("content-type", "application/json")
            .query(query)
            .send()
            .await?;

        if response.status().as_u16() == 401 {
            self.invalidate_session(&token).await;
            let new_token = self.ensure_session().await?;
            let retry_response = self
                .http
                .get(&url)
                .header("arena_session_id", &new_token)
                .header("content-type", "application/json")
                .query(query)
                .send()
                .await?;
            if !retry_response.status().is_success() {
                let status = retry_response.status();
                let text = retry_response.text().await.unwrap_or_default();
                return Err(anyhow!("Arena API error ({}): {}", status, text));
            }
            return Ok(retry_response.json().await?);
        }

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(anyhow!("Arena API error ({}): {}", status, text));
        }

        Ok(response.json().await?)
    }

    pub(crate) async fn search_items(
        &self,
        params: &SearchItemsParams,
    ) -> Result<ArenaListResponse<Item>> {
        let mut query = Vec::new();
        if let Some(number) = &params.number {
            query.push(("number".to_string(), number.clone()));
        }
        if let Some(name) = &params.name {
            query.push(("name".to_string(), name.clone()));
        }
        if let Some(description) = &params.description {
            query.push(("description".to_string(), description.clone()));
        }
        if let Some(category_guid) = &params.category_guid {
            query.push(("category.guid".to_string(), category_guid.clone()));
        }
        if let Some(lifecycle_phase_guid) = &params.lifecycle_phase_guid {
            query.push((
                "lifecyclePhase.guid".to_string(),
                lifecycle_phase_guid.clone(),
            ));
        }
        if let Some(offset) = params.offset {
            query.push(("offset".to_string(), offset.to_string()));
        }
        if let Some(limit) = params.limit {
            query.push(("limit".to_string(), limit.to_string()));
        }
        let value = self.get("/items", &query).await?;
        Ok(serde_json::from_value(value)?)
    }

    pub(crate) async fn get_item(&self, guid: &str) -> Result<Item> {
        let path = format!("/items/{guid}");
        let query = vec![("responseview".to_string(), "full".to_string())];
        let value = self.get(&path, &query).await?;
        Ok(serde_json::from_value(value)?)
    }

    pub(crate) async fn get_bom(&self, guid: &str) -> Result<ArenaListResponse<BomLine>> {
        let path = format!("/items/{guid}/bom");
        let value = self.get(&path, &[]).await?;
        Ok(serde_json::from_value(value)?)
    }

    pub(crate) async fn get_where_used(&self, guid: &str) -> Result<ArenaListResponse<WhereUsedEntry>> {
        let path = format!("/items/{guid}/whereused");
        let value = self.get(&path, &[]).await?;
        Ok(serde_json::from_value(value)?)
    }

    pub(crate) async fn search_changes(
        &self,
        params: &SearchChangesParams,
    ) -> Result<ArenaListResponse<Change>> {
        let mut query = Vec::new();
        if let Some(number) = &params.number {
            query.push(("number".to_string(), number.clone()));
        }
        if let Some(title) = &params.title {
            query.push(("title".to_string(), title.clone()));
        }
        if let Some(lifecycle_status) = &params.lifecycle_status {
            query.push((
                "lifecycleStatus.type".to_string(),
                lifecycle_status.clone(),
            ));
        }
        if let Some(implementation_status) = &params.implementation_status {
            query.push((
                "implementationStatus".to_string(),
                implementation_status.clone(),
            ));
        }
        if let Some(offset) = params.offset {
            query.push(("offset".to_string(), offset.to_string()));
        }
        if let Some(limit) = params.limit {
            query.push(("limit".to_string(), limit.to_string()));
        }
        let value = self.get("/changes", &query).await?;
        Ok(serde_json::from_value(value)?)
    }

    pub(crate) async fn get_change(&self, guid: &str) -> Result<Change> {
        let path = format!("/changes/{guid}");
        let value = self.get(&path, &[]).await?;
        Ok(serde_json::from_value(value)?)
    }

    pub(crate) async fn get_change_affected_items(
        &self,
        guid: &str,
    ) -> Result<ArenaListResponse<ChangeAffectedItem>> {
        let path = format!("/changes/{guid}/items");
        let value = self.get(&path, &[]).await?;
        Ok(serde_json::from_value(value)?)
    }

    pub(crate) async fn get_item_revisions(
        &self,
        guid: &str,
    ) -> Result<ArenaListResponse<ItemRevision>> {
        let path = format!("/items/{guid}/revisions");
        let value = self.get(&path, &[]).await?;
        Ok(serde_json::from_value(value)?)
    }

    pub(crate) async fn get_item_files(&self, guid: &str) -> Result<ArenaListResponse<ItemFile>> {
        let path = format!("/items/{guid}/files");
        let value = self.get(&path, &[]).await?;
        Ok(serde_json::from_value(value)?)
    }

    pub(crate) async fn search_suppliers(
        &self,
        params: &SearchSuppliersParams,
    ) -> Result<serde_json::Value> {
        let mut query = Vec::new();
        if let Some(name) = &params.name {
            query.push(("name".to_string(), name.clone()));
        }
        if let Some(offset) = params.offset {
            query.push(("offset".to_string(), offset.to_string()));
        }
        if let Some(limit) = params.limit {
            query.push(("limit".to_string(), limit.to_string()));
        }
        self.get("/suppliers", &query).await
    }

    pub(crate) async fn search_quality_processes(
        &self,
        params: &SearchQualityProcessesParams,
    ) -> Result<serde_json::Value> {
        let mut query = Vec::new();
        if let Some(number) = &params.number {
            query.push(("number".to_string(), number.clone()));
        }
        if let Some(name) = &params.name {
            query.push(("name".to_string(), name.clone()));
        }
        if let Some(status) = &params.status {
            query.push(("status".to_string(), status.clone()));
        }
        if let Some(offset) = params.offset {
            query.push(("offset".to_string(), offset.to_string()));
        }
        if let Some(limit) = params.limit {
            query.push(("limit".to_string(), limit.to_string()));
        }
        self.get("/qualityprocesses", &query).await
    }

    pub(crate) async fn search_requests(
        &self,
        params: &SearchRequestsParams,
    ) -> Result<serde_json::Value> {
        let mut query = Vec::new();
        if let Some(number) = &params.number {
            query.push(("number".to_string(), number.clone()));
        }
        if let Some(title) = &params.title {
            query.push(("title".to_string(), title.clone()));
        }
        if let Some(lifecycle_status) = &params.lifecycle_status {
            query.push((
                "lifecycleStatus.type".to_string(),
                lifecycle_status.clone(),
            ));
        }
        if let Some(offset) = params.offset {
            query.push(("offset".to_string(), offset.to_string()));
        }
        if let Some(limit) = params.limit {
            query.push(("limit".to_string(), limit.to_string()));
        }
        self.get("/requests", &query).await
    }

    pub(crate) async fn get_lifecycle_phases(
        &self,
    ) -> Result<ArenaListResponse<LifecyclePhase>> {
        let value = self
            .get("/settings/items/lifecyclephases", &[])
            .await?;
        Ok(serde_json::from_value(value)?)
    }

    pub(crate) async fn get_item_categories(&self) -> Result<serde_json::Value> {
        self.get("/settings/items/categories", &[]).await
    }
}
