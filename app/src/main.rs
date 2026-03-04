#![windows_subsystem = "windows"]

use std::sync::mpsc;

use arena_app_protocol::{AgentStatus, ArenaMethod, ArenaResult, BackendEvent, FrontendCommand};
use base64::Engine;
use include_dir::{Dir, include_dir};
use nightshade::claude::{ClaudeConfig, CliCommand, CliEvent, McpConfig, spawn_cli_worker};
use nightshade::prelude::*;
use nightshade::webview::{WebviewContext, serve_embedded_dir};

static DIST: Dir = include_dir!("$CARGO_MANIFEST_DIR/site/dist");

const WRITE_TOOLS: &[&str] = &[
    "mcp__arena__create_item",
    "mcp__arena__update_item",
    "mcp__arena__delete_item",
    "mcp__arena__create_bom_line",
    "mcp__arena__update_bom_line",
    "mcp__arena__delete_bom_line",
    "mcp__arena__create_change",
    "mcp__arena__update_change",
    "mcp__arena__change_change_status",
    "mcp__arena__add_change_affected_item",
    "mcp__arena__remove_change_affected_item",
    "mcp__arena__create_request",
    "mcp__arena__update_request",
    "mcp__arena__change_request_status",
    "mcp__arena__create_supplier",
    "mcp__arena__update_supplier",
    "mcp__arena__create_ticket",
    "mcp__arena__change_quality_status",
    "mcp__arena__item_lifecycle_phase_change",
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
    },
    Failure {
        message: String,
    },
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (cli_cmd_tx, cli_cmd_rx) = mpsc::channel::<CliCommand>();
    let (cli_event_tx, cli_event_rx) = mpsc::channel::<CliEvent>();
    let (arena_result_tx, arena_result_rx) = mpsc::channel::<(u32, ArenaResult)>();
    let (file_result_tx, file_result_rx) = mpsc::channel::<FileResult>();
    let (login_result_tx, login_result_rx) = mpsc::channel::<LoginResult>();

    let port = serve_embedded_dir(&DIST);

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
}

impl State for ArenaApp {
    fn title(&self) -> &str {
        "Arena PLM"
    }

    fn initialize(&mut self, world: &mut World) {
        world.resources.user_interface.enabled = true;
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
                    self.ctx.ensure_webview(
                        handle.clone(),
                        self.port,
                        ui.available_rect_before_wrap(),
                    );
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
                    self.handle_login(email, password, workspace_id, base_url);
                }
                FrontendCommand::Logout => {
                    self.handle_logout();
                }
                FrontendCommand::SendPrompt {
                    prompt,
                    session_id,
                    model,
                } => {
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
                    let _ = self.cli_cmd_tx.send(CliCommand::Cancel);
                    self.ctx.send(BackendEvent::StatusUpdate {
                        status: AgentStatus::Idle,
                    });
                }
                FrontendCommand::ArenaRequest { request_id, method } => {
                    if let Some(arena_tx) = &self.arena_tx {
                        let _ = arena_tx.send((request_id, method));
                    } else {
                        self.ctx.send(BackendEvent::ArenaResponse {
                            request_id,
                            result: ArenaResult::Error {
                                message: "Not logged in".into(),
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
                    if let Some(file_tx) = &self.file_tx {
                        let _ = file_tx.send(FileDownloadRequest {
                            request_id,
                            item_guid,
                            file_guid,
                            file_name,
                        });
                    } else {
                        self.ctx.send(BackendEvent::FileError {
                            request_id,
                            message: "Not logged in".into(),
                        });
                    }
                }
                FrontendCommand::SetWriteMode { enabled } => {
                    self.write_mode = enabled;
                    self.respawn_cli_worker();
                    self.ctx.send(BackendEvent::WriteModeChanged { enabled });
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
            self.ctx.send(BackendEvent::LoginFailure {
                message: "Base URL must use HTTPS".to_string(),
            });
            return;
        }

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
                    let _ = login_result_tx.send(LoginResult::Failure {
                        message: format!("HTTP client error: {error}"),
                    });
                    return;
                }
            };

            let login_url = format!("{base_url_clone}/login");
            let response = match client.post(&login_url).json(&body).send() {
                Ok(response) => response,
                Err(error) => {
                    let _ = login_result_tx.send(LoginResult::Failure {
                        message: format!("Connection failed: {error}"),
                    });
                    return;
                }
            };

            if !response.status().is_success() {
                let status = response.status();
                let text = response.text().unwrap_or_default();
                let _ = login_result_tx.send(LoginResult::Failure {
                    message: format!("Login failed ({status}): {text}"),
                });
                return;
            }

            let _ = login_result_tx.send(LoginResult::Success {
                email: email_clone,
                password: password_clone,
                workspace_id: workspace_id_clone,
                base_url: base_url_clone,
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
                } => {
                    let credentials = Credentials {
                        email,
                        password,
                        workspace_id,
                        base_url,
                    };
                    self.spawn_arena_worker(&credentials);
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
                    self.ctx.send(BackendEvent::LoginFailure { message });
                }
            }
        }
    }

    fn handle_logout(&mut self) {
        self.arena_tx = None;
        self.file_tx = None;
        self.credentials = None;
        self.write_mode = false;

        let (new_cmd_tx, new_cmd_rx) = mpsc::channel::<CliCommand>();
        self.cli_cmd_tx = new_cmd_tx;
        self.cli_cmd_rx = Some(new_cmd_rx);
    }

    fn spawn_arena_worker(&mut self, credentials: &Credentials) {
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
                Err(_) => return,
            };

            let mut session_token: Option<String> = None;

            let do_login = |client: &reqwest::blocking::Client| -> Result<String, String> {
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
                let response = client
                    .get(&url)
                    .header("arena_session_id", &token)
                    .header("content-type", "application/json")
                    .query(query)
                    .send()
                    .map_err(|error| error.to_string())?;

                if response.status().as_u16() == 401 {
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
                let response = client
                    .get(&url)
                    .header("arena_session_id", &token)
                    .send()
                    .map_err(|error| error.to_string())?;

                if response.status().as_u16() == 401 {
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
                    None => break,
                    Some(WorkItem::Arena(request_id, method)) => {
                        let result = match handle_method(&mut session_token, &client, method) {
                            Ok(json) => ArenaResult::Success {
                                json: json.to_string(),
                            },
                            Err(message) => ArenaResult::Error { message },
                        };
                        if arena_result_tx.send((request_id, result)).is_err() {
                            break;
                        }
                    }
                    Some(WorkItem::File(request)) => {
                        let path = format!(
                            "/items/{}/files/{}/content",
                            request.item_guid, request.file_guid
                        );
                        match arena_get_bytes(&mut session_token, &client, &path) {
                            Ok((data, mime_type)) => {
                                if file_result_tx
                                    .send(FileResult::Success(FileDownloadResponse {
                                        request_id: request.request_id,
                                        file_name: request.file_name,
                                        data,
                                        mime_type,
                                    }))
                                    .is_err()
                                {
                                    break;
                                }
                            }
                            Err(message) => {
                                if file_result_tx
                                    .send(FileResult::Error(FileDownloadError {
                                        request_id: request.request_id,
                                        message,
                                    }))
                                    .is_err()
                                {
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
            return;
        };
        let Some(cli_cmd_rx) = self.cli_cmd_rx.take() else {
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

        let config = ClaudeConfig {
            system_prompt: Some(
                "You have access to Arena PLM tools via the arena MCP server. You can search items, view BOMs, look up changes/ECOs, check where-used references, view revisions, find suppliers, and explore quality processes. Use these tools to help answer questions about hardware parts, assemblies, and change orders in Arena.".to_string(),
            ),
            mcp_config: McpConfig::Custom(serde_json::json!({
                "mcpServers": {
                    "arena": {
                        "command": "arena",
                        "args": []
                    }
                }
            }).to_string()),
            disallowed_tools: disallowed,
            env,
            ..Default::default()
        };

        spawn_cli_worker(cli_cmd_rx, self.cli_event_tx.clone(), config);
    }

    fn respawn_cli_worker(&mut self) {
        let (new_cmd_tx, new_cmd_rx) = mpsc::channel::<CliCommand>();
        self.cli_cmd_tx = new_cmd_tx;
        self.cli_cmd_rx = Some(new_cmd_rx);
        self.spawn_cli_worker();
    }

    fn forward_cli_events(&mut self) {
        for event in self.cli_event_rx.try_iter() {
            match event {
                CliEvent::SessionStarted { session_id } => {
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
                    self.ctx.send(BackendEvent::ToolUseFinished { tool_id });
                    self.ctx.send(BackendEvent::StatusUpdate {
                        status: AgentStatus::Streaming,
                    });
                }
                CliEvent::TurnComplete { session_id } => {
                    self.ctx.send(BackendEvent::TurnComplete { session_id });
                }
                CliEvent::Complete {
                    session_id,
                    total_cost_usd,
                    num_turns,
                } => {
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
            self.ctx
                .send(BackendEvent::ArenaResponse { request_id, result });
        }
    }

    fn forward_file_results(&mut self) {
        for result in self.file_result_rx.try_iter() {
            match result {
                FileResult::Success(response) => {
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
                    self.ctx.send(BackendEvent::FileError {
                        request_id: error.request_id,
                        message: error.message,
                    });
                }
            }
        }
    }
}
