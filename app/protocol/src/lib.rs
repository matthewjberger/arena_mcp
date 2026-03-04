#![no_std]
extern crate alloc;

use alloc::string::String;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ArenaMethod {
    SearchItems {
        number: Option<String>,
        name: Option<String>,
        description: Option<String>,
        category_guid: Option<String>,
        lifecycle_phase_guid: Option<String>,
        offset: Option<i64>,
        limit: Option<i64>,
    },
    GetItem {
        guid: String,
    },
    GetBom {
        guid: String,
    },
    GetWhereUsed {
        guid: String,
    },
    SearchChanges {
        number: Option<String>,
        title: Option<String>,
        lifecycle_status: Option<String>,
        implementation_status: Option<String>,
        offset: Option<i64>,
        limit: Option<i64>,
    },
    GetChange {
        guid: String,
    },
    GetChangeAffectedItems {
        guid: String,
    },
    GetItemFiles {
        guid: String,
    },
    GetItemRevisions {
        guid: String,
    },
    GetLifecyclePhases,
    GetItemCategories,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct RateLimitInfo {
    pub requests_remaining: u32,
    pub request_limit: u32,
    pub reset_time: Option<String>,
}

#[derive(Clone, Serialize, Deserialize)]
pub enum ArenaResult {
    Success { json: String },
    Error { message: String },
}

#[derive(Clone, Serialize, Deserialize)]
pub enum FrontendCommand {
    Ready,
    SendPrompt {
        prompt: String,
        session_id: Option<String>,
        model: Option<String>,
    },
    CancelRequest,
    Login {
        email: String,
        password: String,
        workspace_id: Option<String>,
        base_url: Option<String>,
    },
    Logout,
    ArenaRequest {
        request_id: u32,
        method: ArenaMethod,
    },
    DownloadFile {
        request_id: u32,
        item_guid: String,
        file_guid: String,
        file_name: String,
    },
    SetWriteMode {
        enabled: bool,
    },
    ReadLogs,
    ResetLogs,
    OpenLogFile,
}

#[derive(Clone, Serialize, Deserialize)]
pub enum BackendEvent {
    Connected,
    StreamingStarted {
        session_id: String,
    },
    TextDelta {
        text: String,
    },
    ThinkingDelta {
        text: String,
    },
    ToolUseStarted {
        tool_name: String,
        tool_id: String,
    },
    ToolUseInputDelta {
        tool_id: String,
        partial_json: String,
    },
    ToolUseFinished {
        tool_id: String,
    },
    TurnComplete {
        session_id: String,
    },
    RequestComplete {
        session_id: String,
        total_cost_usd: Option<f64>,
        num_turns: u32,
    },
    Error {
        message: String,
    },
    StatusUpdate {
        status: AgentStatus,
    },
    LoginSuccess {
        workspace_name: Option<String>,
    },
    LoginFailure {
        message: String,
    },
    ArenaResponse {
        request_id: u32,
        result: ArenaResult,
    },
    FileContent {
        request_id: u32,
        file_name: String,
        data_base64: String,
        mime_type: String,
    },
    FileError {
        request_id: u32,
        message: String,
    },
    WriteModeChanged {
        enabled: bool,
    },
    LogContent {
        text: String,
        append: bool,
    },
    RateLimitUpdate {
        info: RateLimitInfo,
    },
}

#[derive(Clone, Serialize, Deserialize)]
pub enum AgentStatus {
    Idle,
    Thinking,
    Streaming,
    UsingTool { tool_name: String },
}
