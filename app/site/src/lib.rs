mod arena;
mod browse;
mod chat;
mod login;
mod logs;
mod settings;
mod state;

use arena_app_protocol::{AgentStatus, BackendEvent, FrontendCommand};
use leptos::prelude::*;

use crate::browse::{BomTree, ChangesBrowse, ItemsBrowse};
use crate::chat::ChatView;
use crate::login::LoginScreen;
use crate::logs::LogsView;
use crate::settings::SettingsView;
use crate::state::{
    AppState, ChatState, LoginState, MessageRole, StatusDisplay, ToolUseBlock, View,
};

#[component]
pub fn App() -> impl IntoView {
    let app = AppState::new();
    let chat = ChatState::new();
    let login_state = LoginState::new();

    provide_context(app.clone());
    provide_context(chat.clone());
    provide_context(login_state.clone());

    Effect::new({
        let app = app.clone();
        let chat = chat.clone();
        let login_state = login_state.clone();
        move |_| {
            let app = app.clone();
            let chat = chat.clone();
            let login_state = login_state.clone();
            nightshade::webview::connect(FrontendCommand::Ready, move |event| match event {
                BackendEvent::Connected => {
                    app.status.set(StatusDisplay::Idle);
                }
                BackendEvent::LoginSuccess { .. } => {
                    app.logged_in.set(true);
                    app.status.set(StatusDisplay::Idle);
                    login_state.loading.set(false);
                    login_state.error.set(None);
                }
                BackendEvent::LoginFailure { message } => {
                    login_state.loading.set(false);
                    login_state.error.set(Some(message));
                }
                BackendEvent::StreamingStarted { session_id: sid } => {
                    app.session_id.set(Some(sid));
                    chat.streaming_text.set(String::new());
                    chat.thinking_text.set(String::new());
                    chat.active_tools.set(Vec::new());
                }
                BackendEvent::TextDelta { text } => {
                    chat.streaming_text
                        .update(|current| current.push_str(&text));
                }
                BackendEvent::ThinkingDelta { text } => {
                    chat.thinking_text.update(|current| current.push_str(&text));
                }
                BackendEvent::ToolUseStarted { tool_name, tool_id } => {
                    chat.active_tools.update(|tools| {
                        tools.push(ToolUseBlock {
                            tool_name,
                            tool_id,
                            input_json: String::new(),
                            finished: false,
                        });
                    });
                }
                BackendEvent::ToolUseInputDelta {
                    tool_id,
                    partial_json,
                } => {
                    chat.active_tools.update(|tools| {
                        if let Some(tool) = tools
                            .iter_mut()
                            .rev()
                            .find(|tool| tool.tool_id == tool_id || tool_id.is_empty())
                        {
                            tool.input_json.push_str(&partial_json);
                        }
                    });
                }
                BackendEvent::ToolUseFinished { tool_id } => {
                    chat.active_tools.update(|tools| {
                        if let Some(tool) = tools
                            .iter_mut()
                            .rev()
                            .find(|tool| tool.tool_id == tool_id || tool_id.is_empty())
                        {
                            tool.finished = true;
                        }
                    });
                }
                BackendEvent::TurnComplete { .. } => {}
                BackendEvent::RequestComplete { .. } => {
                    chat.finalize();
                }
                BackendEvent::Error { message } => {
                    chat.finalize();
                    chat.messages.update(|messages| {
                        messages.push(state::ChatMessage {
                            role: MessageRole::Assistant,
                            content: format!("Error: {message}"),
                            thinking: String::new(),
                            tool_uses: Vec::new(),
                        });
                    });
                }
                BackendEvent::StatusUpdate {
                    status: agent_status,
                } => {
                    app.status.set(match agent_status {
                        AgentStatus::Idle => StatusDisplay::Idle,
                        AgentStatus::Thinking => StatusDisplay::Thinking,
                        AgentStatus::Streaming => StatusDisplay::Streaming,
                        AgentStatus::UsingTool { tool_name } => {
                            StatusDisplay::UsingTool { tool_name }
                        }
                    });
                }
                BackendEvent::ArenaResponse { request_id, result } => {
                    arena::resolve_arena(request_id, result);
                }
                BackendEvent::FileContent {
                    request_id,
                    file_name,
                    data_base64,
                    mime_type,
                } => {
                    arena::resolve_file_success(request_id, file_name, data_base64, mime_type);
                }
                BackendEvent::FileError {
                    request_id,
                    message,
                } => {
                    arena::resolve_file_error(request_id, message);
                }
                BackendEvent::WriteModeChanged { enabled } => {
                    app.write_mode.set(enabled);
                }
                BackendEvent::LogContent { text, append } => {
                    if append {
                        app.log_content.update(|content| content.push_str(&text));
                    } else {
                        app.log_content.set(text);
                    }
                }
                BackendEvent::RateLimitUpdate { info } => {
                    app.rate_limit.set(Some(info));
                }
            });
        }
    });

    view! {
        {move || {
            if app.logged_in.get() {
                view! { <MainApp /> }.into_any()
            } else {
                view! { <LoginScreen /> }.into_any()
            }
        }}
    }
}

#[component]
fn MainApp() -> impl IntoView {
    let app = use_context::<AppState>().unwrap();

    let sidebar_btn = |label: &'static str, icon: &'static str, target: View| {
        let target_clone = target.clone();
        let is_active = move || app.view.get() == target_clone;
        view! {
            <button
                class=move || format!(
                    "w-full text-left px-3 py-2 rounded-lg text-sm cursor-pointer transition-colors {}",
                    if is_active() { "bg-[#1f6feb33] text-[#58a6ff]" } else { "text-[#8b949e] hover:text-[#c9d1d9] hover:bg-[#161b22]" }
                )
                on:click=move |_| app.view.set(target.clone())
            >
                <span class="mr-2">{icon}</span>
                {label}
            </button>
        }
    };

    view! {
        <div class="h-screen flex bg-[#0d1117] text-[#c9d1d9] font-mono">
            // Sidebar
            <div class="w-48 flex flex-col bg-[#161b22] border-r border-[#30363d]">
                <div class="px-3 py-3 border-b border-[#30363d]">
                    <div class="flex items-center gap-2">
                        <span class="text-sm font-bold tracking-wide">"ARENA PLM"</span>
                        <div class={move || format!("w-2 h-2 rounded-full {}", app.status.get().dot_class())}></div>
                    </div>
                    <div class="text-xs text-[#8b949e] mt-0.5">
                        {move || app.status.get().label().to_string()}
                        {move || {
                            if let StatusDisplay::UsingTool { tool_name } = app.status.get() {
                                format!(" ({tool_name})")
                            } else {
                                String::new()
                            }
                        }}
                    </div>
                </div>
                {move || {
                    app.rate_limit.get().map(|info| {
                        let remaining = info.requests_remaining;
                        let total = info.request_limit;
                        let fraction = if total > 0 { remaining as f64 / total as f64 } else { 0.0 };
                        let percent = ((fraction * 100.0) as u32).min(100);
                        let bar_color = if fraction > 0.5 {
                            "bg-green-500"
                        } else if fraction > 0.1 {
                            "bg-yellow-500"
                        } else {
                            "bg-red-500"
                        };
                        let remaining_text = format_with_commas(remaining);
                        let total_text = format_with_commas(total);
                        view! {
                            <div class="px-3 py-2 border-b border-[#30363d]">
                                <div class="w-full h-1.5 bg-[#21262d] rounded-full overflow-hidden mb-1">
                                    <div
                                        class=format!("h-full rounded-full {bar_color}")
                                        style=format!("width: {}%", percent)
                                    ></div>
                                </div>
                                <div class="text-xs text-[#8b949e]">
                                    {format!("{remaining_text} / {total_text} remaining")}
                                </div>
                                {info.reset_time.map(|reset| {
                                    view! {
                                        <div class="text-xs text-[#484f58]">
                                            {format!("Resets: {reset}")}
                                        </div>
                                    }
                                })}
                            </div>
                        }
                    })
                }}
                <div class="flex-1 p-2 space-y-1">
                    {sidebar_btn("Chat", "💬", View::Chat)}
                    {sidebar_btn("Items", "📦", View::Items)}
                    {sidebar_btn("BOMs", "🌳", View::Bom)}
                    {sidebar_btn("Changes", "📋", View::Changes)}
                    {sidebar_btn("Settings", "⚙️", View::Settings)}
                    {sidebar_btn("Logs", "📄", View::Logs)}
                </div>
                <div class="px-3 py-2 border-t border-[#30363d] text-xs text-[#484f58]">
                    {move || app.session_id.get().map(|id| {
                        if id.len() > 16 { format!("{}...", &id[..16]) } else { id }
                    }).unwrap_or_default()}
                </div>
            </div>

            // Content
            <div class="flex-1 flex flex-col min-h-0">
                {move || match app.view.get() {
                    View::Chat => view! { <ChatView /> }.into_any(),
                    View::Items => view! { <ItemsBrowse /> }.into_any(),
                    View::Bom => view! { <BomTree /> }.into_any(),
                    View::Changes => view! { <ChangesBrowse /> }.into_any(),
                    View::Settings => view! { <SettingsView /> }.into_any(),
                    View::Logs => view! { <LogsView /> }.into_any(),
                }}
            </div>
        </div>
    }
}

fn format_with_commas(value: u32) -> String {
    let string = value.to_string();
    let bytes = string.as_bytes();
    let mut result = String::with_capacity(string.len() + bytes.len() / 3);
    for (index, &byte) in bytes.iter().enumerate() {
        if index > 0 && (bytes.len() - index).is_multiple_of(3) {
            result.push(',');
        }
        result.push(byte as char);
    }
    result
}
