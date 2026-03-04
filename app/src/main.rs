#![windows_subsystem = "windows"]

use std::sync::mpsc;

use arena_app_protocol::{AgentStatus, ArenaMethod, ArenaResult, BackendEvent, FrontendCommand};
use base64::Engine;
use include_dir::{Dir, include_dir};
use nightshade::claude::{ClaudeConfig, CliCommand, CliEvent, McpConfig, spawn_cli_worker};
use nightshade::prelude::tracing;
use nightshade::prelude::*;
use nightshade::webview::{WebviewContext, serve_embedded_dir};

static DIST: Dir = include_dir!("$CARGO_MANIFEST_DIR/site/dist");

const WRITE_TOOLS: &[&str] = &[
    "mcp__arena_mcp__create_item",
    "mcp__arena_mcp__update_item",
    "mcp__arena_mcp__delete_item",
    "mcp__arena_mcp__create_bom_line",
    "mcp__arena_mcp__update_bom_line",
    "mcp__arena_mcp__delete_bom_line",
    "mcp__arena_mcp__create_change",
    "mcp__arena_mcp__update_change",
    "mcp__arena_mcp__change_change_status",
    "mcp__arena_mcp__add_change_affected_item",
    "mcp__arena_mcp__remove_change_affected_item",
    "mcp__arena_mcp__create_request",
    "mcp__arena_mcp__update_request",
    "mcp__arena_mcp__change_request_status",
    "mcp__arena_mcp__create_supplier",
    "mcp__arena_mcp__update_supplier",
    "mcp__arena_mcp__create_ticket",
    "mcp__arena_mcp__change_quality_status",
    "mcp__arena_mcp__item_lifecycle_phase_change",
];

struct Credentials {
    email: String,
    password: String,
    workspace_id: Option<String>,
    base_url: String,
}

struct FileDownloadRequest {
    request_id: u32,
    item_guid: String,
    file_guid: String,
    file_name: String,
}

struct FileDownloadResponse {
    request_id: u32,
    file_name: String,
    data: Vec<u8>,
    mime_type: String,
}

struct FileDownloadError {
    request_id: u32,
    message: String,
}

enum FileResult {
    Success(FileDownloadResponse),
    Error(FileDownloadError),
}

enum LoginResult {
    Success {
        email: String,
        password: String,
        workspace_id: Option<String>,
        base_url: String,
        session_token: String,
    },
    Failure {
        message: String,
    },
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    if std::env::args().any(|arg| arg == "--mcp") {
        tokio::runtime::Runtime::new()?.block_on(arena::serve())?;
        return Ok(());
    }

    let (cli_cmd_tx, cli_cmd_rx) = mpsc::channel::<CliCommand>();
    let (cli_event_tx, cli_event_rx) = mpsc::channel::<CliEvent>();
    let (arena_result_tx, arena_result_rx) = mpsc::channel::<(u32, ArenaResult)>();
    let (file_result_tx, file_result_rx) = mpsc::channel::<FileResult>();
    let (login_result_tx, login_result_rx) = mpsc::channel::<LoginResult>();

    let port = serve_embedded_dir(&DIST);

    let log_config = nightshade::prelude::LogConfig {
        default_filter: "info,wgpu_hal=error,wgpu_core=error,naga=error".to_string(),
        ..Default::default()
    };
    let log_file_path =
        std::path::PathBuf::from(&log_config.directory).join(log_file_name("Arena PLM", &log_config));

    launch(ArenaApp {
        port,
        ctx: WebviewContext::default(),
        connected: false,
        cli_cmd_tx,
        cli_cmd_rx: Some(cli_cmd_rx),
        cli_event_tx,
        cli_event_rx,
        arena_tx: None,
        arena_result_tx,
        arena_result_rx,
        file_tx: None,
        file_result_tx,
        file_result_rx,
        login_result_tx,
        login_result_rx,
        credentials: None,
        write_mode: false,
        log_file_path,
    })?;

    Ok(())
}

struct ArenaApp {
    port: u16,
    ctx: WebviewContext<FrontendCommand, BackendEvent>,
    connected: bool,
    cli_cmd_tx: mpsc::Sender<CliCommand>,
    cli_cmd_rx: Option<mpsc::Receiver<CliCommand>>,
    cli_event_tx: mpsc::Sender<CliEvent>,
    cli_event_rx: mpsc::Receiver<CliEvent>,
    arena_tx: Option<mpsc::Sender<(u32, ArenaMethod)>>,
    arena_result_tx: mpsc::Sender<(u32, ArenaResult)>,
    arena_result_rx: mpsc::Receiver<(u32, ArenaResult)>,
    file_tx: Option<mpsc::Sender<FileDownloadRequest>>,
    file_result_tx: mpsc::Sender<FileResult>,
    file_result_rx: mpsc::Receiver<FileResult>,
    login_result_tx: mpsc::Sender<LoginResult>,
    login_result_rx: mpsc::Receiver<LoginResult>,
    credentials: Option<Credentials>,
    write_mode: bool,
    log_file_path: std::path::PathBuf,
}

impl State for ArenaApp {
    fn title(&self) -> &str {
        "Arena PLM"
    }

    fn log_config(&self) -> nightshade::prelude::LogConfig {
        nightshade::prelude::LogConfig {
            default_filter: "info,wgpu_hal=error,wgpu_core=error,naga=error".to_string(),
            ..Default::default()
        }
    }

    fn initialize(&mut self, world: &mut World) {
        world.resources.user_interface.enabled = true;
        tracing::info!("arena app initialized");
    }

    fn ui(&mut self, world: &mut World, ctx: &egui::Context) {
        self.process_frontend_commands();
        self.forward_login_results();
        self.forward_cli_events();
        self.forward_arena_results();
        self.forward_file_results();

        egui::CentralPanel::default()
            .frame(egui::Frame::NONE)
            .show(ctx, |ui| {
                if let Some(handle) = &world.resources.window.handle {
                    if self.ctx.ensure_webview(
                        handle.clone(),
                        self.port,
                        ui.available_rect_before_wrap(),
                    ) {
                        tracing::info!("webview created");
                    }
                    handle.request_redraw();
                }
            });
    }
}

impl ArenaApp {
    fn process_frontend_commands(&mut self) {
        let commands: Vec<FrontendCommand> = self.ctx.drain_messages().collect();
        for command in commands {
            match command {
                FrontendCommand::Ready => {
                    if !self.connected {
                        tracing::info!("frontend connected");
                        self.ctx.send(BackendEvent::Connected);
                        self.connected = true;
                    }
                }
                FrontendCommand::Login {
                    email,
                    password,
                    workspace_id,
                    base_url,
                } => {
                    tracing::info!(email = %email, workspace_id = ?workspace_id, "login requested");
                    self.handle_login(email, password, workspace_id, base_url);
                }
                FrontendCommand::Logout => {
                    tracing::info!("logout requested");
                    self.handle_logout();
                }
                FrontendCommand::SendPrompt {
                    prompt,
                    session_id,
                    model,
                } => {
                    tracing::info!(
                        session_id = ?session_id,
                        model = ?model,
                        prompt_len = prompt.len(),
                        "sending prompt to claude cli"
                    );
                    self.ctx.send(BackendEvent::StatusUpdate {
                        status: AgentStatus::Thinking,
                    });
                    let _ = self.cli_cmd_tx.send(CliCommand::StartQuery {
                        prompt,
                        session_id,
                        model,
                    });
                }
                FrontendCommand::CancelRequest => {
                    tracing::info!("cancel request");
                    let _ = self.cli_cmd_tx.send(CliCommand::Cancel);
                    self.ctx.send(BackendEvent::StatusUpdate {
                        status: AgentStatus::Idle,
                    });
                }
                FrontendCommand::ArenaRequest { request_id, method } => {
                    tracing::debug!(request_id, ?method, "arena api request");
                    let sent = self
                        .arena_tx
                        .as_ref()
                        .map(|tx| tx.send((request_id, method)).is_ok())
                        .unwrap_or(false);
                    if !sent {
                        tracing::warn!(request_id, "arena request failed: not connected");
                        self.ctx.send(BackendEvent::ArenaResponse {
                            request_id,
                            result: ArenaResult::Error {
                                message: "Not connected to Arena".into(),
                            },
                        });
                    }
                }
                FrontendCommand::DownloadFile {
                    request_id,
                    item_guid,
                    file_guid,
                    file_name,
                } => {
                    tracing::info!(request_id, %file_name, "file download requested");
                    let sent = self
                        .file_tx
                        .as_ref()
                        .map(|tx| {
                            tx.send(FileDownloadRequest {
                                request_id,
                                item_guid,
                                file_guid,
                                file_name,
                            })
                            .is_ok()
                        })
                        .unwrap_or(false);
                    if !sent {
                        tracing::warn!(request_id, "file download failed: not connected");
                        self.ctx.send(BackendEvent::FileError {
                            request_id,
                            message: "Not connected to Arena".into(),
                        });
                    }
                }
                FrontendCommand::SetWriteMode { enabled } => {
                    tracing::info!(enabled, "write mode changed");
                    self.write_mode = enabled;
                    self.respawn_cli_worker();
                    self.ctx.send(BackendEvent::WriteModeChanged { enabled });
                }
                FrontendCommand::ReadLogs => {
                    let text = std::fs::read_to_string(&self.log_file_path)
                        .unwrap_or_else(|error| format!("Failed to read log file: {error}"));
                    self.ctx.send(BackendEvent::LogContent { text });
                }
                FrontendCommand::OpenLogFile => {
                    let path = rfd::FileDialog::new()
                        .set_title("Open Log File")
                        .add_filter("Log files", &["log", "txt"])
                        .pick_file();
                    if let Some(path) = path {
                        let text = std::fs::read_to_string(&path)
                            .unwrap_or_else(|error| format!("Failed to read file: {error}"));
                        self.ctx.send(BackendEvent::LogContent { text });
                    }
                }
            }
        }
    }

    fn handle_login(
        &mut self,
        email: String,
        password: String,
        workspace_id: Option<String>,
        base_url: Option<String>,
    ) {
        let base_url = base_url
            .filter(|url| !url.is_empty())
            .unwrap_or_else(|| "https://api.arenasolutions.com/v1".to_string());

        if !base_url.starts_with("https://") {
            tracing::error!(%base_url, "login rejected: base URL must use HTTPS");
            self.ctx.send(BackendEvent::LoginFailure {
                message: "Base URL must use HTTPS".to_string(),
            });
            return;
        }

        tracing::info!(%base_url, "spawning login thread");

        let login_result_tx = self.login_result_tx.clone();
        let workspace_id_clone = workspace_id.clone();
        let email_clone = email.clone();
        let password_clone = password.clone();
        let base_url_clone = base_url.clone();

        std::thread::spawn(move || {
            let workspace_id_i64 = workspace_id_clone
                .as_ref()
                .and_then(|workspace| workspace.parse::<i64>().ok());

            let mut body = serde_json::json!({
                "email": email_clone,
                "password": password_clone,
            });
            if let Some(workspace) = workspace_id_i64 {
                body["workspaceId"] = serde_json::json!(workspace);
            }

            let client = match reqwest::blocking::Client::builder()
                .timeout(std::time::Duration::from_secs(30))
                .build()
            {
                Ok(client) => client,
                Err(error) => {
                    tracing::error!(%error, "failed to build HTTP client");
                    let _ = login_result_tx.send(LoginResult::Failure {
                        message: format!("HTTP client error: {error}"),
                    });
                    return;
                }
            };

            let login_url = format!("{base_url_clone}/login");
            tracing::debug!(%login_url, "posting login request");
            let response = match client.post(&login_url).json(&body).send() {
                Ok(response) => response,
                Err(error) => {
                    tracing::error!(%error, "login connection failed");
                    let _ = login_result_tx.send(LoginResult::Failure {
                        message: format!("Connection failed: {error}"),
                    });
                    return;
                }
            };

            if !response.status().is_success() {
                let status = response.status();
                let text = response.text().unwrap_or_default();
                tracing::error!(%status, "login failed");
                let _ = login_result_tx.send(LoginResult::Failure {
                    message: format!("Login failed ({status}): {text}"),
                });
                return;
            }

            let json: serde_json::Value = match response.json() {
                Ok(json) => json,
                Err(error) => {
                    tracing::error!(%error, "invalid login response");
                    let _ = login_result_tx.send(LoginResult::Failure {
                        message: format!("Invalid login response: {error}"),
                    });
                    return;
                }
            };

            let session_token = match json.get("arenaSessionId").and_then(|value| value.as_str()) {
                Some(token) => token.to_string(),
                None => {
                    tracing::error!("no session ID in login response");
                    let _ = login_result_tx.send(LoginResult::Failure {
                        message: "No session ID in login response".to_string(),
                    });
                    return;
                }
            };

            tracing::info!("login successful, session token acquired");
            let _ = login_result_tx.send(LoginResult::Success {
                email: email_clone,
                password: password_clone,
                workspace_id: workspace_id_clone,
                base_url: base_url_clone,
                session_token,
            });
        });
    }

    fn forward_login_results(&mut self) {
        let results: Vec<LoginResult> = self.login_result_rx.try_iter().collect();
        for result in results {
            match result {
                LoginResult::Success {
                    email,
                    password,
                    workspace_id,
                    base_url,
                    session_token,
                } => {
                    tracing::info!(%email, %base_url, "login succeeded, spawning workers");
                    let credentials = Credentials {
                        email,
                        password,
                        workspace_id,
                        base_url,
                    };
                    self.spawn_arena_worker(&credentials, session_token);
                    self.credentials = Some(credentials);
                    self.spawn_cli_worker();
                    self.ctx.send(BackendEvent::LoginSuccess {
                        workspace_name: None,
                    });
                    self.ctx.send(BackendEvent::StatusUpdate {
                        status: AgentStatus::Idle,
                    });
                }
                LoginResult::Failure { message } => {
                    tracing::warn!(%message, "login failed");
                    self.ctx.send(BackendEvent::LoginFailure { message });
                }
            }
        }
    }

    fn handle_logout(&mut self) {
        tracing::info!("logging out, dropping arena and cli workers");
        self.arena_tx = None;
        self.file_tx = None;
        self.credentials = None;
        self.write_mode = false;

        let (new_cmd_tx, new_cmd_rx) = mpsc::channel::<CliCommand>();
        self.cli_cmd_tx = new_cmd_tx;
        self.cli_cmd_rx = Some(new_cmd_rx);
    }

    fn spawn_arena_worker(&mut self, credentials: &Credentials, initial_token: String) {
        tracing::info!("spawning arena worker thread");
        let (arena_req_tx, arena_req_rx) = mpsc::channel::<(u32, ArenaMethod)>();
        let (file_req_tx, file_req_rx) = mpsc::channel::<FileDownloadRequest>();

        let arena_result_tx = self.arena_result_tx.clone();
        let file_result_tx = self.file_result_tx.clone();
        let base_url = credentials.base_url.clone();
        let email = credentials.email.clone();
        let password = credentials.password.clone();
        let workspace_id = credentials
            .workspace_id
            .as_ref()
            .and_then(|workspace| workspace.parse::<i64>().ok());

        std::thread::spawn(move || {
            let client = match reqwest::blocking::Client::builder()
                .timeout(std::time::Duration::from_secs(30))
                .build()
            {
                Ok(client) => client,
                Err(error) => {
                    tracing::error!(%error, "arena worker: failed to build HTTP client");
                    return;
                }
            };

            let mut session_token: Option<String> = Some(initial_token);
            tracing::info!("arena worker started with initial session token");

            let do_login = |client: &reqwest::blocking::Client| -> Result<String, String> {
                tracing::info!("arena worker: re-authenticating");
                let mut body = serde_json::json!({
                    "email": email,
                    "password": password,
                });
                if let Some(workspace) = workspace_id {
                    body["workspaceId"] = serde_json::json!(workspace);
                }
                let response = client
                    .post(format!("{base_url}/login"))
                    .json(&body)
                    .send()
                    .map_err(|error| error.to_string())?;
                if !response.status().is_success() {
                    let status = response.status();
                    tracing::error!(%status, "arena worker: re-auth failed");
                    return Err(format!("Login failed: {}", response.status()));
                }
                let json: serde_json::Value = response.json().map_err(|error| error.to_string())?;
                json.get("arenaSessionId")
                    .and_then(|value| value.as_str())
                    .map(String::from)
                    .ok_or_else(|| "No session ID in response".to_string())
            };

            let ensure_session = |session_token: &mut Option<String>,
                                  client: &reqwest::blocking::Client|
             -> Result<String, String> {
                if let Some(token) = session_token.as_ref() {
                    return Ok(token.clone());
                }
                let token = do_login(client)?;
                *session_token = Some(token.clone());
                Ok(token)
            };

            let arena_get = |session_token: &mut Option<String>,
                             client: &reqwest::blocking::Client,
                             path: &str,
                             query: &[(String, String)]|
             -> Result<serde_json::Value, String> {
                let token = ensure_session(session_token, client)?;
                let url = format!("{base_url}{path}");
                tracing::debug!(%path, "arena worker: GET");
                let response = client
                    .get(&url)
                    .header("arena_session_id", &token)
                    .header("content-type", "application/json")
                    .query(query)
                    .send()
                    .map_err(|error| error.to_string())?;

                if response.status().as_u16() == 401 {
                    tracing::warn!(%path, "arena worker: 401, re-authenticating");
                    *session_token = None;
                    let new_token = ensure_session(session_token, client)?;
                    let retry = client
                        .get(&url)
                        .header("arena_session_id", &new_token)
                        .header("content-type", "application/json")
                        .query(query)
                        .send()
                        .map_err(|error| error.to_string())?;
                    return retry.json().map_err(|error| error.to_string());
                }

                if !response.status().is_success() {
                    let status = response.status();
                    let text = response.text().unwrap_or_default();
                    tracing::error!(%path, %status, "arena worker: API error");
                    return Err(format!("Arena API error ({status}): {text}"));
                }
                response.json().map_err(|error| error.to_string())
            };

            let arena_get_bytes = |session_token: &mut Option<String>,
                                   client: &reqwest::blocking::Client,
                                   path: &str|
             -> Result<(Vec<u8>, String), String> {
                let token = ensure_session(session_token, client)?;
                let url = format!("{base_url}{path}");
                tracing::debug!(%path, "arena worker: GET bytes");
                let response = client
                    .get(&url)
                    .header("arena_session_id", &token)
                    .send()
                    .map_err(|error| error.to_string())?;

                if response.status().as_u16() == 401 {
                    tracing::warn!(%path, "arena worker: 401 on file download, re-authenticating");
                    *session_token = None;
                    let new_token = ensure_session(session_token, client)?;
                    let retry = client
                        .get(&url)
                        .header("arena_session_id", &new_token)
                        .send()
                        .map_err(|error| error.to_string())?;
                    let mime = retry
                        .headers()
                        .get("content-type")
                        .and_then(|value| value.to_str().ok())
                        .unwrap_or("application/octet-stream")
                        .to_string();
                    let bytes = retry.bytes().map_err(|error| error.to_string())?.to_vec();
                    return Ok((bytes, mime));
                }

                if !response.status().is_success() {
                    let status = response.status();
                    let text = response.text().unwrap_or_default();
                    tracing::error!(%path, %status, "arena worker: file download error");
                    return Err(format!("Arena API error ({status}): {text}"));
                }
                let mime = response
                    .headers()
                    .get("content-type")
                    .and_then(|value| value.to_str().ok())
                    .unwrap_or("application/octet-stream")
                    .to_string();
                let bytes = response
                    .bytes()
                    .map_err(|error| error.to_string())?
                    .to_vec();
                Ok((bytes, mime))
            };

            let handle_method = |session_token: &mut Option<String>,
                                 client: &reqwest::blocking::Client,
                                 method: ArenaMethod|
             -> Result<serde_json::Value, String> {
                match method {
                    ArenaMethod::SearchItems {
                        number,
                        name,
                        description,
                        category_guid,
                        lifecycle_phase_guid,
                        offset,
                        limit,
                    } => {
                        let mut query = Vec::new();
                        if let Some(value) = number {
                            query.push(("number".to_string(), value));
                        }
                        if let Some(value) = name {
                            query.push(("name".to_string(), value));
                        }
                        if let Some(value) = description {
                            query.push(("description".to_string(), value));
                        }
                        if let Some(value) = category_guid {
                            query.push(("category.guid".to_string(), value));
                        }
                        if let Some(value) = lifecycle_phase_guid {
                            query.push(("lifecyclePhase.guid".to_string(), value));
                        }
                        if let Some(value) = offset {
                            query.push(("offset".to_string(), value.to_string()));
                        }
                        if let Some(value) = limit {
                            query.push(("limit".to_string(), value.to_string()));
                        }
                        arena_get(session_token, client, "/items", &query)
                    }
                    ArenaMethod::GetItem { guid } => {
                        let query = vec![("responseview".to_string(), "full".to_string())];
                        arena_get(session_token, client, &format!("/items/{guid}"), &query)
                    }
                    ArenaMethod::GetBom { guid } => {
                        arena_get(session_token, client, &format!("/items/{guid}/bom"), &[])
                    }
                    ArenaMethod::GetWhereUsed { guid } => arena_get(
                        session_token,
                        client,
                        &format!("/items/{guid}/whereused"),
                        &[],
                    ),
                    ArenaMethod::SearchChanges {
                        number,
                        title,
                        lifecycle_status,
                        implementation_status,
                        offset,
                        limit,
                    } => {
                        let mut query = Vec::new();
                        if let Some(value) = number {
                            query.push(("number".to_string(), value));
                        }
                        if let Some(value) = title {
                            query.push(("title".to_string(), value));
                        }
                        if let Some(value) = lifecycle_status {
                            query.push(("lifecycleStatus.type".to_string(), value));
                        }
                        if let Some(value) = implementation_status {
                            query.push(("implementationStatus".to_string(), value));
                        }
                        if let Some(value) = offset {
                            query.push(("offset".to_string(), value.to_string()));
                        }
                        if let Some(value) = limit {
                            query.push(("limit".to_string(), value.to_string()));
                        }
                        arena_get(session_token, client, "/changes", &query)
                    }
                    ArenaMethod::GetChange { guid } => {
                        arena_get(session_token, client, &format!("/changes/{guid}"), &[])
                    }
                    ArenaMethod::GetChangeAffectedItems { guid } => arena_get(
                        session_token,
                        client,
                        &format!("/changes/{guid}/items"),
                        &[],
                    ),
                    ArenaMethod::GetItemFiles { guid } => {
                        arena_get(session_token, client, &format!("/items/{guid}/files"), &[])
                    }
                    ArenaMethod::GetItemRevisions { guid } => arena_get(
                        session_token,
                        client,
                        &format!("/items/{guid}/revisions"),
                        &[],
                    ),
                    ArenaMethod::GetLifecyclePhases => arena_get(
                        session_token,
                        client,
                        "/settings/items/lifecyclephases",
                        &[],
                    ),
                    ArenaMethod::GetItemCategories => {
                        arena_get(session_token, client, "/settings/items/categories", &[])
                    }
                }
            };

            loop {
                enum WorkItem {
                    Arena(u32, ArenaMethod),
                    File(FileDownloadRequest),
                }

                let work = crossbeam_channel_select(&arena_req_rx, &file_req_rx);

                match work {
                    None => {
                        tracing::info!("arena worker: all channels disconnected, shutting down");
                        break;
                    }
                    Some(WorkItem::Arena(request_id, method)) => {
                        tracing::debug!(request_id, "arena worker: handling request");
                        let result = match handle_method(&mut session_token, &client, method) {
                            Ok(json) => {
                                tracing::debug!(request_id, "arena worker: request succeeded");
                                ArenaResult::Success {
                                    json: json.to_string(),
                                }
                            }
                            Err(message) => {
                                tracing::warn!(request_id, %message, "arena worker: request failed");
                                ArenaResult::Error { message }
                            }
                        };
                        if arena_result_tx.send((request_id, result)).is_err() {
                            tracing::warn!("arena worker: result channel closed");
                            break;
                        }
                    }
                    Some(WorkItem::File(request)) => {
                        tracing::info!(
                            request_id = request.request_id,
                            file_name = %request.file_name,
                            "arena worker: downloading file"
                        );
                        let path = format!(
                            "/items/{}/files/{}/content",
                            request.item_guid, request.file_guid
                        );
                        match arena_get_bytes(&mut session_token, &client, &path) {
                            Ok((data, mime_type)) => {
                                tracing::info!(
                                    request_id = request.request_id,
                                    size = data.len(),
                                    %mime_type,
                                    "arena worker: file downloaded"
                                );
                                if file_result_tx
                                    .send(FileResult::Success(FileDownloadResponse {
                                        request_id: request.request_id,
                                        file_name: request.file_name,
                                        data,
                                        mime_type,
                                    }))
                                    .is_err()
                                {
                                    tracing::warn!("arena worker: file result channel closed");
                                    break;
                                }
                            }
                            Err(message) => {
                                tracing::error!(
                                    request_id = request.request_id,
                                    %message,
                                    "arena worker: file download failed"
                                );
                                if file_result_tx
                                    .send(FileResult::Error(FileDownloadError {
                                        request_id: request.request_id,
                                        message,
                                    }))
                                    .is_err()
                                {
                                    tracing::warn!("arena worker: file result channel closed");
                                    break;
                                }
                            }
                        }
                    }
                }

                fn crossbeam_channel_select(
                    arena_rx: &mpsc::Receiver<(u32, ArenaMethod)>,
                    file_rx: &mpsc::Receiver<FileDownloadRequest>,
                ) -> Option<WorkItem> {
                    loop {
                        match arena_rx.try_recv() {
                            Ok((id, method)) => return Some(WorkItem::Arena(id, method)),
                            Err(mpsc::TryRecvError::Disconnected) => {}
                            Err(mpsc::TryRecvError::Empty) => {}
                        }
                        match file_rx.try_recv() {
                            Ok(request) => return Some(WorkItem::File(request)),
                            Err(mpsc::TryRecvError::Disconnected) => {}
                            Err(mpsc::TryRecvError::Empty) => {}
                        }
                        match arena_rx.recv_timeout(std::time::Duration::from_millis(50)) {
                            Ok((id, method)) => return Some(WorkItem::Arena(id, method)),
                            Err(mpsc::RecvTimeoutError::Timeout) => {}
                            Err(mpsc::RecvTimeoutError::Disconnected) => match file_rx.try_recv() {
                                Ok(request) => return Some(WorkItem::File(request)),
                                Err(_) => return None,
                            },
                        }
                    }
                }
            }
        });

        self.arena_tx = Some(arena_req_tx);
        self.file_tx = Some(file_req_tx);
    }

    fn spawn_cli_worker(&mut self) {
        let Some(credentials) = &self.credentials else {
            tracing::warn!("spawn_cli_worker called without credentials");
            return;
        };
        let Some(cli_cmd_rx) = self.cli_cmd_rx.take() else {
            tracing::warn!("spawn_cli_worker called but command receiver already taken");
            return;
        };

        let mut env = vec![
            ("ARENA_EMAIL".to_string(), credentials.email.clone()),
            ("ARENA_PASSWORD".to_string(), credentials.password.clone()),
            ("ARENA_BASE_URL".to_string(), credentials.base_url.clone()),
        ];
        if let Some(workspace_id) = &credentials.workspace_id {
            env.push(("ARENA_WORKSPACE_ID".to_string(), workspace_id.clone()));
        }

        let disallowed = if self.write_mode {
            None
        } else {
            Some(WRITE_TOOLS.iter().map(|tool| (*tool).to_string()).collect())
        };

        tracing::info!(
            write_mode = self.write_mode,
            disallowed_count = disallowed
                .as_ref()
                .map_or(0, |tools: &Vec<String>| tools.len()),
            "spawning claude cli worker"
        );

        let config = ClaudeConfig {
            system_prompt: Some(
                "You have access to Arena PLM tools via the arena MCP server. You can search items, view BOMs, look up changes/ECOs, check where-used references, view revisions, find suppliers, and explore quality processes. Use these tools to help answer questions about hardware parts, assemblies, and change orders in Arena.".to_string(),
            ),
            mcp_config: McpConfig::Custom(serde_json::json!({
                "mcpServers": {
                    "arena_mcp": {
                        "command": std::env::current_exe()
                            .unwrap_or_default()
                            .to_string_lossy()
                            .to_string(),
                        "args": ["--mcp"]
                    }
                }
            }).to_string()),
            allowed_tools: Some(vec!["mcp__arena_mcp__*".to_string()]),
            disallowed_tools: disallowed,
            env,
            ..Default::default()
        };

        spawn_cli_worker(cli_cmd_rx, self.cli_event_tx.clone(), config);
    }

    fn respawn_cli_worker(&mut self) {
        tracing::info!("respawning claude cli worker");
        let (new_cmd_tx, new_cmd_rx) = mpsc::channel::<CliCommand>();
        self.cli_cmd_tx = new_cmd_tx;
        self.cli_cmd_rx = Some(new_cmd_rx);
        self.spawn_cli_worker();
    }

    fn forward_cli_events(&mut self) {
        for event in self.cli_event_rx.try_iter() {
            match event {
                CliEvent::SessionStarted { session_id } => {
                    tracing::info!(%session_id, "cli session started");
                    self.ctx.send(BackendEvent::StreamingStarted { session_id });
                    self.ctx.send(BackendEvent::StatusUpdate {
                        status: AgentStatus::Streaming,
                    });
                }
                CliEvent::TextDelta { text } => {
                    self.ctx.send(BackendEvent::TextDelta { text });
                }
                CliEvent::ThinkingDelta { text } => {
                    self.ctx.send(BackendEvent::ThinkingDelta { text });
                }
                CliEvent::ToolUseStarted { tool_name, tool_id } => {
                    tracing::info!(%tool_name, %tool_id, "tool use started");
                    self.ctx.send(BackendEvent::StatusUpdate {
                        status: AgentStatus::UsingTool {
                            tool_name: tool_name.clone(),
                        },
                    });
                    self.ctx
                        .send(BackendEvent::ToolUseStarted { tool_name, tool_id });
                }
                CliEvent::ToolUseInputDelta {
                    tool_id,
                    partial_json,
                } => {
                    self.ctx.send(BackendEvent::ToolUseInputDelta {
                        tool_id,
                        partial_json,
                    });
                }
                CliEvent::ToolUseFinished { tool_id } => {
                    tracing::debug!(%tool_id, "tool use finished");
                    self.ctx.send(BackendEvent::ToolUseFinished { tool_id });
                    self.ctx.send(BackendEvent::StatusUpdate {
                        status: AgentStatus::Streaming,
                    });
                }
                CliEvent::TurnComplete { session_id } => {
                    tracing::debug!(%session_id, "turn complete");
                    self.ctx.send(BackendEvent::TurnComplete { session_id });
                }
                CliEvent::Complete {
                    session_id,
                    total_cost_usd,
                    num_turns,
                } => {
                    tracing::info!(
                        %session_id,
                        total_cost_usd,
                        num_turns,
                        "request complete"
                    );
                    self.ctx.send(BackendEvent::RequestComplete {
                        session_id,
                        total_cost_usd,
                        num_turns,
                    });
                    self.ctx.send(BackendEvent::StatusUpdate {
                        status: AgentStatus::Idle,
                    });
                }
                CliEvent::Error { message } => {
                    tracing::error!(%message, "cli error");
                    self.ctx.send(BackendEvent::Error { message });
                    self.ctx.send(BackendEvent::StatusUpdate {
                        status: AgentStatus::Idle,
                    });
                }
            }
        }
    }

    fn forward_arena_results(&mut self) {
        for (request_id, result) in self.arena_result_rx.try_iter() {
            match &result {
                ArenaResult::Success { json } => {
                    tracing::debug!(
                        request_id,
                        response_len = json.len(),
                        "arena response forwarded"
                    );
                }
                ArenaResult::Error { message } => {
                    tracing::warn!(request_id, %message, "arena error forwarded");
                }
            }
            self.ctx
                .send(BackendEvent::ArenaResponse { request_id, result });
        }
    }

    fn forward_file_results(&mut self) {
        for result in self.file_result_rx.try_iter() {
            match result {
                FileResult::Success(response) => {
                    tracing::info!(
                        request_id = response.request_id,
                        file_name = %response.file_name,
                        size = response.data.len(),
                        "file content forwarded to frontend"
                    );
                    let data_base64 =
                        base64::engine::general_purpose::STANDARD.encode(&response.data);
                    self.ctx.send(BackendEvent::FileContent {
                        request_id: response.request_id,
                        file_name: response.file_name,
                        data_base64,
                        mime_type: response.mime_type,
                    });
                }
                FileResult::Error(error) => {
                    tracing::error!(
                        request_id = error.request_id,
                        message = %error.message,
                        "file download error forwarded to frontend"
                    );
                    self.ctx.send(BackendEvent::FileError {
                        request_id: error.request_id,
                        message: error.message,
                    });
                }
            }
        }
    }
}
