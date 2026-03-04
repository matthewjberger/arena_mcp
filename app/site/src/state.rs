use leptos::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq)]
pub enum View {
    Chat,
    Items,
    Bom,
    Changes,
    Settings,
}

#[derive(Clone)]
pub struct AppState {
    pub view: RwSignal<View>,
    pub status: RwSignal<StatusDisplay>,
    pub logged_in: RwSignal<bool>,
    pub write_mode: RwSignal<bool>,
    pub session_id: RwSignal<Option<String>>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            view: RwSignal::new(View::Chat),
            status: RwSignal::new(StatusDisplay::Disconnected),
            logged_in: RwSignal::new(false),
            write_mode: RwSignal::new(false),
            session_id: RwSignal::new(None),
        }
    }
}

#[derive(Clone)]
pub enum StatusDisplay {
    Disconnected,
    Idle,
    Thinking,
    Streaming,
    UsingTool { tool_name: String },
}

impl StatusDisplay {
    pub fn label(&self) -> &str {
        match self {
            Self::Disconnected => "Disconnected",
            Self::Idle => "Ready",
            Self::Thinking => "Thinking...",
            Self::Streaming => "Streaming...",
            Self::UsingTool { .. } => "Using tool...",
        }
    }

    pub fn dot_class(&self) -> &str {
        match self {
            Self::Disconnected => "bg-red-500",
            Self::Idle => "bg-green-500",
            Self::Thinking => "bg-yellow-500",
            Self::Streaming => "bg-blue-500",
            Self::UsingTool { .. } => "bg-purple-500",
        }
    }
}

#[derive(Clone)]
pub struct ChatMessage {
    pub role: MessageRole,
    pub content: String,
    pub thinking: String,
    pub tool_uses: Vec<ToolUseBlock>,
}

#[derive(Clone)]
pub enum MessageRole {
    User,
    Assistant,
}

#[derive(Clone)]
pub struct ToolUseBlock {
    pub tool_name: String,
    pub tool_id: String,
    pub input_json: String,
    pub finished: bool,
}

#[derive(Clone)]
pub struct ChatState {
    pub messages: RwSignal<Vec<ChatMessage>>,
    pub streaming_text: RwSignal<String>,
    pub thinking_text: RwSignal<String>,
    pub active_tools: RwSignal<Vec<ToolUseBlock>>,
    pub input_text: RwSignal<String>,
}

impl ChatState {
    pub fn new() -> Self {
        Self {
            messages: RwSignal::new(Vec::new()),
            streaming_text: RwSignal::new(String::new()),
            thinking_text: RwSignal::new(String::new()),
            active_tools: RwSignal::new(Vec::new()),
            input_text: RwSignal::new(String::new()),
        }
    }

    pub fn finalize(&self) {
        let text = self.streaming_text.get_untracked();
        let thinking = self.thinking_text.get_untracked();
        let tools = self.active_tools.get_untracked();
        if !text.is_empty() || !thinking.is_empty() || !tools.is_empty() {
            self.messages.update(|messages| {
                messages.push(ChatMessage {
                    role: MessageRole::Assistant,
                    content: text,
                    thinking,
                    tool_uses: tools,
                });
            });
        }
        self.streaming_text.set(String::new());
        self.thinking_text.set(String::new());
        self.active_tools.set(Vec::new());
    }
}

#[derive(Clone)]
pub struct LoginState {
    pub error: RwSignal<Option<String>>,
    pub loading: RwSignal<bool>,
}

impl LoginState {
    pub fn new() -> Self {
        Self {
            error: RwSignal::new(None),
            loading: RwSignal::new(false),
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct SavedSearch {
    pub label: String,
    pub view: String,
    pub query: String,
}
