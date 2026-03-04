use std::time::Duration;

use anyhow::{Context, Result, anyhow};
use reqwest::Client;
use tokio::sync::Mutex;

use crate::types::*;

struct ArenaCredentials {
    email: String,
    password: String,
    workspace_id: Option<i64>,
    base_url: String,
}

pub(crate) struct ArenaClient {
    http: Client,
    credentials: Mutex<Option<ArenaCredentials>>,
    session: Mutex<Option<String>>,
}

impl ArenaClient {
    pub(crate) fn new() -> Result<Self> {
        let credentials = Self::credentials_from_env();
        Ok(Self {
            http: Client::builder().timeout(Duration::from_secs(30)).build()?,
            credentials: Mutex::new(credentials),
            session: Mutex::new(None),
        })
    }

    fn credentials_from_env() -> Option<ArenaCredentials> {
        let email = std::env::var("ARENA_EMAIL").ok()?;
        let password = std::env::var("ARENA_PASSWORD").ok()?;
        let workspace_id = std::env::var("ARENA_WORKSPACE_ID")
            .ok()
            .filter(|value| !value.is_empty())
            .and_then(|value| value.parse::<i64>().ok());
        let base_url = std::env::var("ARENA_BASE_URL")
            .unwrap_or_else(|_| "https://api.arenasolutions.com/v1".to_string());
        Some(ArenaCredentials {
            email,
            password,
            workspace_id,
            base_url,
        })
    }

    pub(crate) async fn authenticate(
        &self,
        email: &str,
        password: &str,
        workspace_id: Option<i64>,
        base_url: Option<&str>,
    ) -> Result<LoginResponse> {
        let base_url = base_url.unwrap_or("https://api.arenasolutions.com/v1");
        let url = format!("{base_url}/login");
        let body = LoginRequest {
            email: email.to_string(),
            password: password.to_string(),
            workspace_id,
        };
        let response = self.http.post(&url).json(&body).send().await?;
        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(anyhow!("Login failed ({}): {}", status, text));
        }
        let login: LoginResponse = response.json().await?;
        *self.session.lock().await = Some(login.arena_session_id.clone());
        *self.credentials.lock().await = Some(ArenaCredentials {
            email: email.to_string(),
            password: password.to_string(),
            workspace_id,
            base_url: base_url.to_string(),
        });
        Ok(login)
    }

    async fn do_login(&self) -> Result<String> {
        let (url, body) = {
            let guard = self.credentials.lock().await;
            let credentials = guard
                .as_ref()
                .ok_or_else(|| anyhow!("Not logged in. Use the login tool first with your Arena email, password, and optional workspace ID."))?;
            (
                format!("{}/login", credentials.base_url),
                LoginRequest {
                    email: credentials.email.clone(),
                    password: credentials.password.clone(),
                    workspace_id: credentials.workspace_id,
                },
            )
        };
        let response = self.http.post(&url).json(&body).send().await?;
        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(anyhow!("Login failed ({}): {}", status, text));
        }
        let login: LoginResponse = response.json().await?;
        Ok(login.arena_session_id)
    }

    pub(crate) async fn logout(&self) {
        let base_url = {
            let credentials = self.credentials.lock().await;
            credentials.as_ref().map(|cred| cred.base_url.clone())
        };
        let token = {
            let session = self.session.lock().await;
            session.clone()
        };
        if let (Some(base_url), Some(token)) = (base_url, token) {
            let url = format!("{base_url}/login");
            let _ = self
                .http
                .put(&url)
                .header("arena_session_id", &token)
                .send()
                .await;
        }
        *self.session.lock().await = None;
        *self.credentials.lock().await = None;
    }

    async fn ensure_session(&self) -> Result<String> {
        let session = self.session.lock().await;
        if let Some(token) = session.as_ref() {
            return Ok(token.clone());
        }
        drop(session);
        let token = self.do_login().await?;
        let mut session = self.session.lock().await;
        *session = Some(token.clone());
        Ok(token)
    }

    async fn invalidate_session(&self, stale_token: &str) {
        let mut session = self.session.lock().await;
        if session.as_deref() == Some(stale_token) {
            *session = None;
        }
    }

    fn base_url(&self, credentials: &Option<ArenaCredentials>) -> Result<String> {
        credentials
            .as_ref()
            .map(|cred| cred.base_url.clone())
            .ok_or_else(|| anyhow!("Not logged in. Use the login tool first."))
    }

    async fn send_with_auth(
        &self,
        build_request: impl Fn(&str) -> reqwest::RequestBuilder,
    ) -> Result<reqwest::Response> {
        let token = self.ensure_session().await?;
        let response = build_request(&token).send().await?;

        if response.status().as_u16() == 401 {
            self.invalidate_session(&token).await;
            let new_token = self.ensure_session().await?;
            let retry_response = build_request(&new_token).send().await?;
            if !retry_response.status().is_success() {
                let status = retry_response.status();
                let text = retry_response.text().await.unwrap_or_default();
                return Err(anyhow!("Arena API error ({}): {}", status, text));
            }
            return Ok(retry_response);
        }

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(anyhow!("Arena API error ({}): {}", status, text));
        }

        Ok(response)
    }

    async fn get(&self, path: &str, query: &[(String, String)]) -> Result<serde_json::Value> {
        let base_url = self.base_url(&*self.credentials.lock().await)?;
        let url = format!("{base_url}{path}");
        let response = self
            .send_with_auth(|token| {
                self.http
                    .get(&url)
                    .header("arena_session_id", token)
                    .query(query)
            })
            .await?;
        Ok(response.json().await?)
    }

    async fn post(&self, path: &str, body: &serde_json::Value) -> Result<serde_json::Value> {
        let base_url = self.base_url(&*self.credentials.lock().await)?;
        let url = format!("{base_url}{path}");
        let response = self
            .send_with_auth(|token| {
                self.http
                    .post(&url)
                    .header("arena_session_id", token)
                    .header("content-type", "application/json")
                    .json(body)
            })
            .await?;
        Ok(response.json().await?)
    }

    async fn put(&self, path: &str, body: &serde_json::Value) -> Result<serde_json::Value> {
        let base_url = self.base_url(&*self.credentials.lock().await)?;
        let url = format!("{base_url}{path}");
        let response = self
            .send_with_auth(|token| {
                self.http
                    .put(&url)
                    .header("arena_session_id", token)
                    .header("content-type", "application/json")
                    .json(body)
            })
            .await?;
        Ok(response.json().await?)
    }

    async fn delete(&self, path: &str) -> Result<serde_json::Value> {
        let base_url = self.base_url(&*self.credentials.lock().await)?;
        let url = format!("{base_url}{path}");
        let response = self
            .send_with_auth(|token| self.http.delete(&url).header("arena_session_id", token))
            .await?;
        let status = response.status();
        let bytes = response.bytes().await?;
        if status.as_u16() == 204 || bytes.is_empty() {
            return Ok(serde_json::Value::Null);
        }
        serde_json::from_slice(&bytes).context("failed to deserialize Arena API response")
    }

    async fn get_bytes(&self, path: &str) -> Result<(Vec<u8>, String)> {
        let base_url = self.base_url(&*self.credentials.lock().await)?;
        let url = format!("{base_url}{path}");
        let response = self
            .send_with_auth(|token| self.http.get(&url).header("arena_session_id", token))
            .await?;
        let content_type = response
            .headers()
            .get("content-type")
            .and_then(|value| value.to_str().ok())
            .unwrap_or("application/octet-stream")
            .to_string();
        let bytes = response.bytes().await?.to_vec();
        Ok((bytes, content_type))
    }

    fn parse_additional_attributes(
        json: &Option<String>,
    ) -> Result<Option<Vec<serde_json::Value>>> {
        match json {
            Some(text) => {
                let parsed: Vec<serde_json::Value> = serde_json::from_str(text)
                    .context("additional_attributes_json must be a valid JSON array")?;
                Ok(Some(parsed))
            }
            None => Ok(None),
        }
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
        if let Some(procurement_type) = &params.procurement_type {
            query.push(("procurementType".to_string(), procurement_type.clone()));
        }
        if let Some(offset) = params.offset {
            query.push(("offset".to_string(), offset.to_string()));
        }
        if let Some(limit) = params.limit {
            query.push(("limit".to_string(), limit.to_string()));
        }
        let value = self.get("/items", &query).await?;
        serde_json::from_value(value).context("failed to deserialize Arena API response")
    }

    pub(crate) async fn get_item(&self, guid: &str) -> Result<Item> {
        let path = format!("/items/{guid}");
        let query = vec![("responseview".to_string(), "full".to_string())];
        let value = self.get(&path, &query).await?;
        serde_json::from_value(value).context("failed to deserialize Arena API response")
    }

    pub(crate) async fn create_item(&self, params: &CreateItemParams) -> Result<Item> {
        let additional_attributes =
            Self::parse_additional_attributes(&params.additional_attributes_json)?;
        let mut body = serde_json::json!({
            "name": params.name,
            "category": { "guid": params.category_guid },
        });
        if let Some(number) = &params.number {
            body["number"] = serde_json::json!(number);
        }
        if let Some(description) = &params.description {
            body["description"] = serde_json::json!(description);
        }
        if let Some(attrs) = additional_attributes {
            body["additionalAttributes"] = serde_json::json!(attrs);
        }
        let value = self.post("/items", &body).await?;
        serde_json::from_value(value).context("failed to deserialize Arena API response")
    }

    pub(crate) async fn update_item(&self, params: &UpdateItemParams) -> Result<Item> {
        let additional_attributes =
            Self::parse_additional_attributes(&params.additional_attributes_json)?;
        let mut body = serde_json::json!({});
        if let Some(name) = &params.name {
            body["name"] = serde_json::json!(name);
        }
        if let Some(number) = &params.number {
            body["number"] = serde_json::json!(number);
        }
        if let Some(description) = &params.description {
            body["description"] = serde_json::json!(description);
        }
        if let Some(attrs) = additional_attributes {
            body["additionalAttributes"] = serde_json::json!(attrs);
        }
        let path = format!("/items/{}", params.guid);
        let value = self.put(&path, &body).await?;
        serde_json::from_value(value).context("failed to deserialize Arena API response")
    }

    pub(crate) async fn delete_item(&self, guid: &str) -> Result<serde_json::Value> {
        let path = format!("/items/{guid}");
        self.delete(&path).await
    }

    pub(crate) async fn get_item_sourcing(
        &self,
        guid: &str,
    ) -> Result<ArenaListResponse<ItemSourcingEntry>> {
        let path = format!("/items/{guid}/sourcing");
        let value = self.get(&path, &[]).await?;
        serde_json::from_value(value).context("failed to deserialize Arena API response")
    }

    pub(crate) async fn get_item_compliance(
        &self,
        guid: &str,
    ) -> Result<ArenaListResponse<ComplianceRequirement>> {
        let path = format!("/items/{guid}/compliance");
        let value = self.get(&path, &[]).await?;
        serde_json::from_value(value).context("failed to deserialize Arena API response")
    }

    pub(crate) async fn get_item_references(
        &self,
        guid: &str,
    ) -> Result<ArenaListResponse<ItemReference>> {
        let path = format!("/items/{guid}/references");
        let value = self.get(&path, &[]).await?;
        serde_json::from_value(value).context("failed to deserialize Arena API response")
    }

    pub(crate) async fn get_item_quality(
        &self,
        guid: &str,
    ) -> Result<ArenaListResponse<QualityProcess>> {
        let path = format!("/items/{guid}/quality");
        let value = self.get(&path, &[]).await?;
        serde_json::from_value(value).context("failed to deserialize Arena API response")
    }

    pub(crate) async fn item_lifecycle_phase_change(
        &self,
        params: &ItemLifecyclePhaseChangeParams,
    ) -> Result<serde_json::Value> {
        let mut body = serde_json::json!({
            "item": { "guid": params.item_guid },
            "lifecyclePhase": { "guid": params.lifecycle_phase_guid },
        });
        if let Some(comment) = &params.comment {
            body["comment"] = serde_json::json!(comment);
        }
        self.post("/items/lifecyclephasechanges", &body).await
    }

    pub(crate) async fn get_bom(&self, guid: &str) -> Result<ArenaListResponse<BomLine>> {
        let path = format!("/items/{guid}/bom");
        let value = self.get(&path, &[]).await?;
        serde_json::from_value(value).context("failed to deserialize Arena API response")
    }

    pub(crate) async fn get_where_used(
        &self,
        guid: &str,
    ) -> Result<ArenaListResponse<WhereUsedEntry>> {
        let path = format!("/items/{guid}/whereused");
        let value = self.get(&path, &[]).await?;
        serde_json::from_value(value).context("failed to deserialize Arena API response")
    }

    pub(crate) async fn create_bom_line(&self, params: &CreateBomLineParams) -> Result<BomLine> {
        let mut body = serde_json::json!({
            "item": { "guid": params.child_item_guid },
        });
        if let Some(quantity) = params.quantity {
            body["quantity"] = serde_json::json!(quantity);
        }
        if let Some(ref_des) = &params.ref_des {
            body["refDes"] = serde_json::json!(ref_des);
        }
        if let Some(notes) = &params.notes {
            body["notes"] = serde_json::json!(notes);
        }
        let path = format!("/items/{}/bom", params.item_guid);
        let value = self.post(&path, &body).await?;
        serde_json::from_value(value).context("failed to deserialize Arena API response")
    }

    pub(crate) async fn update_bom_line(&self, params: &UpdateBomLineParams) -> Result<BomLine> {
        let mut body = serde_json::json!({});
        if let Some(quantity) = params.quantity {
            body["quantity"] = serde_json::json!(quantity);
        }
        if let Some(ref_des) = &params.ref_des {
            body["refDes"] = serde_json::json!(ref_des);
        }
        if let Some(notes) = &params.notes {
            body["notes"] = serde_json::json!(notes);
        }
        let path = format!("/items/{}/bom/{}", params.item_guid, params.line_guid);
        let value = self.put(&path, &body).await?;
        serde_json::from_value(value).context("failed to deserialize Arena API response")
    }

    pub(crate) async fn delete_bom_line(
        &self,
        params: &DeleteBomLineParams,
    ) -> Result<serde_json::Value> {
        let path = format!("/items/{}/bom/{}", params.item_guid, params.line_guid);
        self.delete(&path).await
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
            query.push(("lifecycleStatus.type".to_string(), lifecycle_status.clone()));
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
        serde_json::from_value(value).context("failed to deserialize Arena API response")
    }

    pub(crate) async fn get_change(&self, guid: &str) -> Result<Change> {
        let path = format!("/changes/{guid}");
        let value = self.get(&path, &[]).await?;
        serde_json::from_value(value).context("failed to deserialize Arena API response")
    }

    pub(crate) async fn get_change_affected_items(
        &self,
        guid: &str,
    ) -> Result<ArenaListResponse<ChangeAffectedItem>> {
        let path = format!("/changes/{guid}/items");
        let value = self.get(&path, &[]).await?;
        serde_json::from_value(value).context("failed to deserialize Arena API response")
    }

    pub(crate) async fn create_change(&self, params: &CreateChangeParams) -> Result<Change> {
        let additional_attributes =
            Self::parse_additional_attributes(&params.additional_attributes_json)?;
        let mut body = serde_json::json!({});
        if let Some(title) = &params.title {
            body["title"] = serde_json::json!(title);
        }
        if let Some(category_guid) = &params.category_guid {
            body["category"] = serde_json::json!({ "guid": category_guid });
        }
        if let Some(description) = &params.description {
            body["description"] = serde_json::json!(description);
        }
        if let Some(attrs) = additional_attributes {
            body["additionalAttributes"] = serde_json::json!(attrs);
        }
        let value = self.post("/changes", &body).await?;
        serde_json::from_value(value).context("failed to deserialize Arena API response")
    }

    pub(crate) async fn update_change(&self, params: &UpdateChangeParams) -> Result<Change> {
        let additional_attributes =
            Self::parse_additional_attributes(&params.additional_attributes_json)?;
        let mut body = serde_json::json!({});
        if let Some(title) = &params.title {
            body["title"] = serde_json::json!(title);
        }
        if let Some(description) = &params.description {
            body["description"] = serde_json::json!(description);
        }
        if let Some(attrs) = additional_attributes {
            body["additionalAttributes"] = serde_json::json!(attrs);
        }
        let path = format!("/changes/{}", params.guid);
        let value = self.put(&path, &body).await?;
        serde_json::from_value(value).context("failed to deserialize Arena API response")
    }

    pub(crate) async fn change_change_status(
        &self,
        params: &ChangeChangeStatusParams,
    ) -> Result<serde_json::Value> {
        let mut body = serde_json::json!({
            "change": { "guid": params.change_guid },
            "lifecycleStatus": { "type": params.status },
        });
        if let Some(comment) = &params.comment {
            body["comment"] = serde_json::json!(comment);
        }
        if let Some(force) = params.force {
            body["force"] = serde_json::json!(force);
        }
        self.post("/changes/statuschanges", &body).await
    }

    pub(crate) async fn add_change_affected_item(
        &self,
        params: &AddChangeAffectedItemParams,
    ) -> Result<ChangeAffectedItem> {
        let mut body = serde_json::json!({
            "item": { "guid": params.item_guid },
        });
        if let Some(guid) = &params.new_lifecycle_phase_guid {
            body["newLifecyclePhase"] = serde_json::json!({ "guid": guid });
        }
        if let Some(revision) = &params.new_revision_number {
            body["newRevisionNumber"] = serde_json::json!(revision);
        }
        let path = format!("/changes/{}/items", params.change_guid);
        let value = self.post(&path, &body).await?;
        serde_json::from_value(value).context("failed to deserialize Arena API response")
    }

    pub(crate) async fn remove_change_affected_item(
        &self,
        params: &RemoveChangeAffectedItemParams,
    ) -> Result<serde_json::Value> {
        let path = format!(
            "/changes/{}/items/{}",
            params.change_guid, params.affected_item_guid
        );
        self.delete(&path).await
    }

    pub(crate) async fn get_change_files(&self, guid: &str) -> Result<ArenaListResponse<ItemFile>> {
        let path = format!("/changes/{guid}/files");
        let value = self.get(&path, &[]).await?;
        serde_json::from_value(value).context("failed to deserialize Arena API response")
    }

    pub(crate) async fn get_change_implementation_statuses(
        &self,
        guid: &str,
    ) -> Result<ArenaListResponse<ImplementationStatus>> {
        let path = format!("/changes/{guid}/implementationstatuses");
        let value = self.get(&path, &[]).await?;
        serde_json::from_value(value).context("failed to deserialize Arena API response")
    }

    pub(crate) async fn get_item_revisions(
        &self,
        guid: &str,
    ) -> Result<ArenaListResponse<ItemRevision>> {
        let path = format!("/items/{guid}/revisions");
        let value = self.get(&path, &[]).await?;
        serde_json::from_value(value).context("failed to deserialize Arena API response")
    }

    pub(crate) async fn get_item_files(&self, guid: &str) -> Result<ArenaListResponse<ItemFile>> {
        let path = format!("/items/{guid}/files");
        let value = self.get(&path, &[]).await?;
        serde_json::from_value(value).context("failed to deserialize Arena API response")
    }

    pub(crate) async fn get_item_file_content(
        &self,
        item_guid: &str,
        file_guid: &str,
    ) -> Result<FileContent> {
        let path = format!("/items/{item_guid}/files/{file_guid}/content");
        let (bytes, content_type) = self.get_bytes(&path).await?;
        if content_type.starts_with("text/")
            || content_type.contains("json")
            || content_type.contains("xml")
        {
            Ok(FileContent {
                content_type,
                encoding: "text".to_string(),
                data: String::from_utf8_lossy(&bytes).into_owned(),
                size_bytes: bytes.len(),
            })
        } else {
            use base64::Engine;
            Ok(FileContent {
                content_type,
                encoding: "base64".to_string(),
                data: base64::engine::general_purpose::STANDARD.encode(&bytes),
                size_bytes: bytes.len(),
            })
        }
    }

    pub(crate) async fn search_files(
        &self,
        params: &SearchFilesParams,
    ) -> Result<ArenaListResponse<FileEntry>> {
        let mut query = Vec::new();
        if let Some(name) = &params.name {
            query.push(("name".to_string(), name.clone()));
        }
        if let Some(category_guid) = &params.category_guid {
            query.push(("category.guid".to_string(), category_guid.clone()));
        }
        if let Some(offset) = params.offset {
            query.push(("offset".to_string(), offset.to_string()));
        }
        if let Some(limit) = params.limit {
            query.push(("limit".to_string(), limit.to_string()));
        }
        let value = self.get("/files", &query).await?;
        serde_json::from_value(value).context("failed to deserialize Arena API response")
    }

    pub(crate) async fn get_file(&self, guid: &str) -> Result<FileEntry> {
        let path = format!("/files/{guid}");
        let value = self.get(&path, &[]).await?;
        serde_json::from_value(value).context("failed to deserialize Arena API response")
    }

    pub(crate) async fn search_requests(
        &self,
        params: &SearchRequestsParams,
    ) -> Result<ArenaListResponse<Request>> {
        let mut query = Vec::new();
        if let Some(number) = &params.number {
            query.push(("number".to_string(), number.clone()));
        }
        if let Some(title) = &params.title {
            query.push(("title".to_string(), title.clone()));
        }
        if let Some(lifecycle_status) = &params.lifecycle_status {
            query.push(("lifecycleStatus.type".to_string(), lifecycle_status.clone()));
        }
        if let Some(offset) = params.offset {
            query.push(("offset".to_string(), offset.to_string()));
        }
        if let Some(limit) = params.limit {
            query.push(("limit".to_string(), limit.to_string()));
        }
        let value = self.get("/requests", &query).await?;
        serde_json::from_value(value).context("failed to deserialize Arena API response")
    }

    pub(crate) async fn get_request(&self, guid: &str) -> Result<Request> {
        let path = format!("/requests/{guid}");
        let value = self.get(&path, &[]).await?;
        serde_json::from_value(value).context("failed to deserialize Arena API response")
    }

    pub(crate) async fn create_request(&self, params: &CreateRequestParams) -> Result<Request> {
        let additional_attributes =
            Self::parse_additional_attributes(&params.additional_attributes_json)?;
        let mut body = serde_json::json!({});
        if let Some(title) = &params.title {
            body["title"] = serde_json::json!(title);
        }
        if let Some(category_guid) = &params.category_guid {
            body["category"] = serde_json::json!({ "guid": category_guid });
        }
        if let Some(description) = &params.description {
            body["description"] = serde_json::json!(description);
        }
        if let Some(attrs) = additional_attributes {
            body["additionalAttributes"] = serde_json::json!(attrs);
        }
        let value = self.post("/requests", &body).await?;
        serde_json::from_value(value).context("failed to deserialize Arena API response")
    }

    pub(crate) async fn update_request(&self, params: &UpdateRequestParams) -> Result<Request> {
        let additional_attributes =
            Self::parse_additional_attributes(&params.additional_attributes_json)?;
        let mut body = serde_json::json!({});
        if let Some(title) = &params.title {
            body["title"] = serde_json::json!(title);
        }
        if let Some(description) = &params.description {
            body["description"] = serde_json::json!(description);
        }
        if let Some(attrs) = additional_attributes {
            body["additionalAttributes"] = serde_json::json!(attrs);
        }
        let path = format!("/requests/{}", params.guid);
        let value = self.put(&path, &body).await?;
        serde_json::from_value(value).context("failed to deserialize Arena API response")
    }

    pub(crate) async fn change_request_status(
        &self,
        params: &ChangeRequestStatusParams,
    ) -> Result<serde_json::Value> {
        let mut body = serde_json::json!({
            "request": { "guid": params.request_guid },
            "lifecycleStatus": { "type": params.status },
        });
        if let Some(comment) = &params.comment {
            body["comment"] = serde_json::json!(comment);
        }
        self.post("/requests/statuschanges", &body).await
    }

    pub(crate) async fn get_request_items(
        &self,
        guid: &str,
    ) -> Result<ArenaListResponse<RequestItem>> {
        let path = format!("/requests/{guid}/items");
        let value = self.get(&path, &[]).await?;
        serde_json::from_value(value).context("failed to deserialize Arena API response")
    }

    pub(crate) async fn search_suppliers(
        &self,
        params: &SearchSuppliersParams,
    ) -> Result<ArenaListResponse<Supplier>> {
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
        let value = self.get("/suppliers", &query).await?;
        serde_json::from_value(value).context("failed to deserialize Arena API response")
    }

    pub(crate) async fn get_supplier(&self, guid: &str) -> Result<Supplier> {
        let path = format!("/suppliers/{guid}");
        let value = self.get(&path, &[]).await?;
        serde_json::from_value(value).context("failed to deserialize Arena API response")
    }

    pub(crate) async fn create_supplier(&self, params: &CreateSupplierParams) -> Result<Supplier> {
        let additional_attributes =
            Self::parse_additional_attributes(&params.additional_attributes_json)?;
        let mut body = serde_json::json!({
            "name": params.name,
        });
        if let Some(description) = &params.description {
            body["description"] = serde_json::json!(description);
        }
        if let Some(attrs) = additional_attributes {
            body["additionalAttributes"] = serde_json::json!(attrs);
        }
        let value = self.post("/suppliers", &body).await?;
        serde_json::from_value(value).context("failed to deserialize Arena API response")
    }

    pub(crate) async fn update_supplier(&self, params: &UpdateSupplierParams) -> Result<Supplier> {
        let additional_attributes =
            Self::parse_additional_attributes(&params.additional_attributes_json)?;
        let mut body = serde_json::json!({});
        if let Some(name) = &params.name {
            body["name"] = serde_json::json!(name);
        }
        if let Some(description) = &params.description {
            body["description"] = serde_json::json!(description);
        }
        if let Some(attrs) = additional_attributes {
            body["additionalAttributes"] = serde_json::json!(attrs);
        }
        let path = format!("/suppliers/{}", params.guid);
        let value = self.put(&path, &body).await?;
        serde_json::from_value(value).context("failed to deserialize Arena API response")
    }

    pub(crate) async fn search_supplier_items(
        &self,
        params: &SearchSupplierItemsParams,
    ) -> Result<ArenaListResponse<SupplierItem>> {
        let mut query = Vec::new();
        if let Some(supplier_guid) = &params.supplier_guid {
            query.push(("supplier.guid".to_string(), supplier_guid.clone()));
        }
        if let Some(offset) = params.offset {
            query.push(("offset".to_string(), offset.to_string()));
        }
        if let Some(limit) = params.limit {
            query.push(("limit".to_string(), limit.to_string()));
        }
        let value = self.get("/supplieritems", &query).await?;
        serde_json::from_value(value).context("failed to deserialize Arena API response")
    }

    pub(crate) async fn search_quality_processes(
        &self,
        params: &SearchQualityProcessesParams,
    ) -> Result<ArenaListResponse<QualityProcess>> {
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
        let value = self.get("/qualityprocesses", &query).await?;
        serde_json::from_value(value).context("failed to deserialize Arena API response")
    }

    pub(crate) async fn get_quality_process(&self, guid: &str) -> Result<QualityProcess> {
        let path = format!("/qualityprocesses/{guid}");
        let value = self.get(&path, &[]).await?;
        serde_json::from_value(value).context("failed to deserialize Arena API response")
    }

    pub(crate) async fn get_quality_process_steps(
        &self,
        guid: &str,
    ) -> Result<ArenaListResponse<QualityStep>> {
        let path = format!("/qualityprocesses/{guid}/steps");
        let value = self.get(&path, &[]).await?;
        serde_json::from_value(value).context("failed to deserialize Arena API response")
    }

    pub(crate) async fn change_quality_status(
        &self,
        params: &ChangeQualityStatusParams,
    ) -> Result<serde_json::Value> {
        let mut body = serde_json::json!({
            "qualityProcess": { "guid": params.quality_guid },
            "status": params.status,
        });
        if let Some(comment) = &params.comment {
            body["comment"] = serde_json::json!(comment);
        }
        self.post("/qualityprocesses/statuschanges", &body).await
    }

    pub(crate) async fn search_tickets(
        &self,
        params: &SearchTicketsParams,
    ) -> Result<ArenaListResponse<Ticket>> {
        let mut query = Vec::new();
        if let Some(number) = &params.number {
            query.push(("number".to_string(), number.clone()));
        }
        if let Some(title) = &params.title {
            query.push(("title".to_string(), title.clone()));
        }
        if let Some(offset) = params.offset {
            query.push(("offset".to_string(), offset.to_string()));
        }
        if let Some(limit) = params.limit {
            query.push(("limit".to_string(), limit.to_string()));
        }
        let value = self.get("/tickets", &query).await?;
        serde_json::from_value(value).context("failed to deserialize Arena API response")
    }

    pub(crate) async fn get_ticket(&self, guid: &str) -> Result<Ticket> {
        let path = format!("/tickets/{guid}");
        let value = self.get(&path, &[]).await?;
        serde_json::from_value(value).context("failed to deserialize Arena API response")
    }

    pub(crate) async fn create_ticket(&self, params: &CreateTicketParams) -> Result<Ticket> {
        let additional_attributes =
            Self::parse_additional_attributes(&params.additional_attributes_json)?;
        let mut body = serde_json::json!({
            "template": { "guid": params.template_guid },
        });
        if let Some(title) = &params.title {
            body["title"] = serde_json::json!(title);
        }
        if let Some(description) = &params.description {
            body["description"] = serde_json::json!(description);
        }
        if let Some(attrs) = additional_attributes {
            body["additionalAttributes"] = serde_json::json!(attrs);
        }
        let value = self.post("/tickets", &body).await?;
        serde_json::from_value(value).context("failed to deserialize Arena API response")
    }

    pub(crate) async fn search_training_plans(
        &self,
        params: &SearchTrainingPlansParams,
    ) -> Result<ArenaListResponse<TrainingPlan>> {
        let mut query = Vec::new();
        if let Some(number) = &params.number {
            query.push(("number".to_string(), number.clone()));
        }
        if let Some(name) = &params.name {
            query.push(("name".to_string(), name.clone()));
        }
        if let Some(offset) = params.offset {
            query.push(("offset".to_string(), offset.to_string()));
        }
        if let Some(limit) = params.limit {
            query.push(("limit".to_string(), limit.to_string()));
        }
        let value = self.get("/trainingplans", &query).await?;
        serde_json::from_value(value).context("failed to deserialize Arena API response")
    }

    pub(crate) async fn get_training_plan(&self, guid: &str) -> Result<TrainingPlan> {
        let path = format!("/trainingplans/{guid}");
        let value = self.get(&path, &[]).await?;
        serde_json::from_value(value).context("failed to deserialize Arena API response")
    }

    pub(crate) async fn get_training_plan_records(
        &self,
        guid: &str,
    ) -> Result<ArenaListResponse<TrainingRecord>> {
        let path = format!("/trainingplans/{guid}/records");
        let value = self.get(&path, &[]).await?;
        serde_json::from_value(value).context("failed to deserialize Arena API response")
    }

    pub(crate) async fn get_lifecycle_phases(&self) -> Result<ArenaListResponse<LifecyclePhase>> {
        let value = self.get("/settings/items/lifecyclephases", &[]).await?;
        serde_json::from_value(value).context("failed to deserialize Arena API response")
    }

    pub(crate) async fn get_item_categories(&self) -> Result<ArenaListResponse<ItemCategory>> {
        let value = self.get("/settings/items/categories", &[]).await?;
        serde_json::from_value(value).context("failed to deserialize Arena API response")
    }

    pub(crate) async fn get_change_categories(&self) -> Result<ArenaListResponse<ChangeCategory>> {
        let value = self.get("/settings/changes/categories", &[]).await?;
        serde_json::from_value(value).context("failed to deserialize Arena API response")
    }

    pub(crate) async fn get_item_number_formats(&self) -> Result<ArenaListResponse<NumberFormat>> {
        let value = self.get("/settings/items/numberformats", &[]).await?;
        serde_json::from_value(value).context("failed to deserialize Arena API response")
    }

    pub(crate) async fn get_bom_substitutes(
        &self,
        item_guid: &str,
        bom_line_guid: &str,
    ) -> Result<serde_json::Value> {
        let path = format!("/items/{item_guid}/bom/{bom_line_guid}/substitutes");
        self.get(&path, &[]).await
    }

    pub(crate) async fn get_item_history(&self, guid: &str) -> Result<serde_json::Value> {
        let path = format!("/items/{guid}/history");
        self.get(&path, &[]).await
    }

    pub(crate) async fn get_item_future_changes(&self, guid: &str) -> Result<serde_json::Value> {
        let path = format!("/items/{guid}/future-changes");
        self.get(&path, &[]).await
    }

    pub(crate) async fn get_item_thumbnail(&self, guid: &str) -> Result<FileContent> {
        let path = format!("/items/{guid}/thumbnail");
        let (bytes, content_type) = self.get_bytes(&path).await?;
        if content_type.starts_with("text/")
            || content_type.contains("json")
            || content_type.contains("xml")
        {
            Ok(FileContent {
                content_type,
                encoding: "text".to_string(),
                data: String::from_utf8_lossy(&bytes).into_owned(),
                size_bytes: bytes.len(),
            })
        } else {
            use base64::Engine;
            Ok(FileContent {
                content_type,
                encoding: "base64".to_string(),
                data: base64::engine::general_purpose::STANDARD.encode(&bytes),
                size_bytes: bytes.len(),
            })
        }
    }

    pub(crate) async fn get_change_history(&self, guid: &str) -> Result<serde_json::Value> {
        let path = format!("/changes/{guid}/history");
        self.get(&path, &[]).await
    }

    pub(crate) async fn get_change_alerts(&self, guid: &str) -> Result<serde_json::Value> {
        let path = format!("/changes/{guid}/alerts");
        self.get(&path, &[]).await
    }

    pub(crate) async fn get_change_implementation_tasks(
        &self,
        guid: &str,
    ) -> Result<serde_json::Value> {
        let path = format!("/changes/{guid}/implementation-tasks");
        self.get(&path, &[]).await
    }

    pub(crate) async fn get_change_implementation_task(
        &self,
        change_guid: &str,
        task_guid: &str,
    ) -> Result<serde_json::Value> {
        let path = format!("/changes/{change_guid}/implementation-tasks/{task_guid}");
        self.get(&path, &[]).await
    }

    pub(crate) async fn get_implementation_task_notes(
        &self,
        change_guid: &str,
        task_guid: &str,
    ) -> Result<serde_json::Value> {
        let path = format!("/changes/{change_guid}/implementation-tasks/{task_guid}/notes");
        self.get(&path, &[]).await
    }

    pub(crate) async fn get_file_content(&self, guid: &str) -> Result<FileContent> {
        let path = format!("/files/{guid}/content");
        let (bytes, content_type) = self.get_bytes(&path).await?;
        if content_type.starts_with("text/")
            || content_type.contains("json")
            || content_type.contains("xml")
        {
            Ok(FileContent {
                content_type,
                encoding: "text".to_string(),
                data: String::from_utf8_lossy(&bytes).into_owned(),
                size_bytes: bytes.len(),
            })
        } else {
            use base64::Engine;
            Ok(FileContent {
                content_type,
                encoding: "base64".to_string(),
                data: base64::engine::general_purpose::STANDARD.encode(&bytes),
                size_bytes: bytes.len(),
            })
        }
    }

    pub(crate) async fn get_request_files(&self, guid: &str) -> Result<serde_json::Value> {
        let path = format!("/requests/{guid}/files");
        self.get(&path, &[]).await
    }

    pub(crate) async fn get_request_changes(&self, guid: &str) -> Result<serde_json::Value> {
        let path = format!("/requests/{guid}/changes");
        self.get(&path, &[]).await
    }

    pub(crate) async fn get_supplier_item(&self, guid: &str) -> Result<serde_json::Value> {
        let path = format!("/supplieritems/{guid}");
        self.get(&path, &[]).await
    }

    pub(crate) async fn get_supplier_addresses(&self, guid: &str) -> Result<serde_json::Value> {
        let path = format!("/suppliers/{guid}/addresses");
        self.get(&path, &[]).await
    }

    pub(crate) async fn get_supplier_phone_numbers(&self, guid: &str) -> Result<serde_json::Value> {
        let path = format!("/suppliers/{guid}/phone-numbers");
        self.get(&path, &[]).await
    }

    pub(crate) async fn get_supplier_files(&self, guid: &str) -> Result<serde_json::Value> {
        let path = format!("/suppliers/{guid}/files");
        self.get(&path, &[]).await
    }

    pub(crate) async fn get_supplier_approval_status(
        &self,
        guid: &str,
    ) -> Result<serde_json::Value> {
        let path = format!("/suppliers/{guid}/approval-status");
        self.get(&path, &[]).await
    }

    pub(crate) async fn get_supplier_item_files(&self, guid: &str) -> Result<serde_json::Value> {
        let path = format!("/supplieritems/{guid}/files");
        self.get(&path, &[]).await
    }

    pub(crate) async fn get_supplier_item_compliance(
        &self,
        guid: &str,
    ) -> Result<serde_json::Value> {
        let path = format!("/supplieritems/{guid}/compliance");
        self.get(&path, &[]).await
    }

    pub(crate) async fn get_supplier_item_sourcing(&self, guid: &str) -> Result<serde_json::Value> {
        let path = format!("/supplieritems/{guid}/sourcing");
        self.get(&path, &[]).await
    }

    pub(crate) async fn get_quality_process_step(
        &self,
        process_guid: &str,
        step_guid: &str,
    ) -> Result<serde_json::Value> {
        let path = format!("/qualityprocesses/{process_guid}/steps/{step_guid}");
        self.get(&path, &[]).await
    }

    pub(crate) async fn get_quality_step_decisions(
        &self,
        process_guid: &str,
        step_guid: &str,
    ) -> Result<serde_json::Value> {
        let path = format!("/qualityprocesses/{process_guid}/steps/{step_guid}/decisions");
        self.get(&path, &[]).await
    }

    pub(crate) async fn get_quality_step_affected_objects(
        &self,
        process_guid: &str,
        step_guid: &str,
    ) -> Result<serde_json::Value> {
        let path = format!("/qualityprocesses/{process_guid}/steps/{step_guid}/objects");
        self.get(&path, &[]).await
    }

    pub(crate) async fn get_ticket_items(&self, guid: &str) -> Result<serde_json::Value> {
        let path = format!("/tickets/{guid}/items");
        self.get(&path, &[]).await
    }

    pub(crate) async fn get_ticket_changes(&self, guid: &str) -> Result<serde_json::Value> {
        let path = format!("/tickets/{guid}/changes");
        self.get(&path, &[]).await
    }

    pub(crate) async fn get_ticket_files(&self, guid: &str) -> Result<serde_json::Value> {
        let path = format!("/tickets/{guid}/files");
        self.get(&path, &[]).await
    }

    pub(crate) async fn get_training_plan_users(&self, guid: &str) -> Result<serde_json::Value> {
        let path = format!("/trainingplans/{guid}/users");
        self.get(&path, &[]).await
    }

    pub(crate) async fn get_training_plan_items(&self, guid: &str) -> Result<serde_json::Value> {
        let path = format!("/trainingplans/{guid}/items");
        self.get(&path, &[]).await
    }

    pub(crate) async fn get_training_plan_files(&self, guid: &str) -> Result<serde_json::Value> {
        let path = format!("/trainingplans/{guid}/files");
        self.get(&path, &[]).await
    }

    pub(crate) async fn get_file_categories(&self) -> Result<serde_json::Value> {
        self.get("/settings/files/categories", &[]).await
    }

    pub(crate) async fn get_change_category_attributes(
        &self,
        guid: &str,
    ) -> Result<serde_json::Value> {
        let path = format!("/settings/changes/categories/{guid}/attributes");
        self.get(&path, &[]).await
    }

    pub(crate) async fn get_change_category_routings(
        &self,
        guid: &str,
    ) -> Result<serde_json::Value> {
        let path = format!("/settings/changes/categories/{guid}/routings");
        self.get(&path, &[]).await
    }

    pub(crate) async fn get_request_categories(&self) -> Result<serde_json::Value> {
        self.get("/settings/requests/categories", &[]).await
    }

    pub(crate) async fn get_item_attributes(&self) -> Result<serde_json::Value> {
        self.get("/settings/items/attributes", &[]).await
    }

    pub(crate) async fn get_bom_attributes(&self) -> Result<serde_json::Value> {
        self.get("/settings/bom/attributes", &[]).await
    }

    pub(crate) async fn get_users(&self) -> Result<serde_json::Value> {
        self.get("/settings/users", &[]).await
    }

    pub(crate) async fn get_user(&self, guid: &str) -> Result<serde_json::Value> {
        let path = format!("/settings/users/{guid}");
        self.get(&path, &[]).await
    }

    pub(crate) async fn get_user_groups(&self) -> Result<serde_json::Value> {
        self.get("/settings/usergroups", &[]).await
    }

    pub(crate) async fn get_api_usage(&self) -> Result<serde_json::Value> {
        self.get("/settings/recentactivities/apiusages", &[]).await
    }
}
