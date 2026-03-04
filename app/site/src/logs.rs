use arena_app_protocol::FrontendCommand;
use leptos::prelude::*;
use wasm_bindgen::prelude::*;

use crate::state::{AppState, View};

#[derive(Clone, Copy, PartialEq)]
enum BlockKind {
    Prompt,
    Response,
    Thinking,
    Tool,
    Arena,
    Info,
    Warn,
    Error,
}

enum LogBlock {
    Prompt { meta: String, body: String },
    Response { meta: String, body: String },
    Thinking { meta: String, body: String },
    ToolCall { meta: String, body: String },
    ToolResult { meta: String, body: String },
    ArenaRequest { meta: String, body: String },
    ArenaResponse { meta: String, body: String },
    Info { time: String, message: String },
    Warn { time: String, message: String },
    Error { time: String, message: String },
}

impl LogBlock {
    fn kind(&self) -> BlockKind {
        match self {
            Self::Prompt { .. } => BlockKind::Prompt,
            Self::Response { .. } => BlockKind::Response,
            Self::Thinking { .. } => BlockKind::Thinking,
            Self::ToolCall { .. } | Self::ToolResult { .. } => BlockKind::Tool,
            Self::ArenaRequest { .. } | Self::ArenaResponse { .. } => BlockKind::Arena,
            Self::Info { .. } => BlockKind::Info,
            Self::Warn { .. } => BlockKind::Warn,
            Self::Error { .. } => BlockKind::Error,
        }
    }

    fn text_content(&self) -> (&str, &str) {
        match self {
            Self::Prompt { meta, body }
            | Self::Response { meta, body }
            | Self::Thinking { meta, body }
            | Self::ToolCall { meta, body }
            | Self::ToolResult { meta, body }
            | Self::ArenaRequest { meta, body }
            | Self::ArenaResponse { meta, body } => (meta.as_str(), body.as_str()),
            Self::Info { time, message }
            | Self::Warn { time, message }
            | Self::Error { time, message } => (time.as_str(), message.as_str()),
        }
    }

    fn matches_search(&self, query: &str) -> bool {
        if query.is_empty() {
            return true;
        }
        let query_lower = query.to_lowercase();
        let (first, second) = self.text_content();
        first.to_lowercase().contains(&query_lower)
            || second.to_lowercase().contains(&query_lower)
    }
}

fn is_log_header(line: &str) -> bool {
    let bytes = line.as_bytes();
    bytes.len() >= 24
        && bytes[0].is_ascii_digit()
        && bytes[1].is_ascii_digit()
        && bytes[2].is_ascii_digit()
        && bytes[3].is_ascii_digit()
        && bytes[4] == b'-'
        && bytes[5].is_ascii_digit()
        && bytes[6].is_ascii_digit()
        && bytes[7] == b'-'
        && bytes[8].is_ascii_digit()
        && bytes[9].is_ascii_digit()
        && bytes[10] == b'T'
        && bytes[11].is_ascii_digit()
        && bytes[12].is_ascii_digit()
        && bytes[13] == b':'
        && bytes[14].is_ascii_digit()
        && bytes[15].is_ascii_digit()
        && bytes[16] == b':'
}

fn parse_header(line: &str) -> (&str, &str, &str) {
    let time = &line[11..19];

    let after_timestamp = line
        .as_bytes()
        .iter()
        .position(|&byte| byte == b'Z' || byte == b'+')
        .map(|position| &line[position + 1..])
        .unwrap_or(&line[20..]);

    let trimmed = after_timestamp.trim_start();
    let (level, rest) = if let Some(stripped) = trimmed.strip_prefix("INFO ") {
        ("INFO", stripped)
    } else if let Some(stripped) = trimmed.strip_prefix("WARN ") {
        ("WARN", stripped)
    } else if let Some(stripped) = trimmed.strip_prefix("ERROR ") {
        ("ERROR", stripped)
    } else {
        ("INFO", trimmed)
    };

    let message = rest
        .find("arena_app: ")
        .map(|index| &rest[index + 11..])
        .or_else(|| rest.find("arena_app:").map(|index| &rest[index + 10..]))
        .unwrap_or(rest);

    (time, level, message.trim())
}

fn extract_meta(tag: &str) -> String {
    tag.trim_start_matches('\u{2501}')
        .trim_end_matches('\u{2501}')
        .trim()
        .to_string()
}

fn parse_blocks(raw: &str) -> Vec<LogBlock> {
    let mut blocks = Vec::new();
    let lines: Vec<&str> = raw.lines().collect();
    let mut index = 0;

    while index < lines.len() {
        let line = lines[index];

        if !is_log_header(line) {
            index += 1;
            continue;
        }

        let (time, level, message) = parse_header(line);

        if message.starts_with("\u{2501}\u{2501}\u{2501}") {
            let tag = message;
            let mut body_lines = Vec::new();
            index += 1;
            while index < lines.len() {
                if is_log_header(lines[index]) {
                    break;
                }
                body_lines.push(lines[index]);
                index += 1;
            }
            let body = body_lines.join("\n");
            let meta = extract_meta(tag);

            if tag.contains("PROMPT") {
                blocks.push(LogBlock::Prompt { meta, body });
            } else if tag.contains("RESPONSE") {
                blocks.push(LogBlock::Response { meta, body });
            } else if tag.contains("THINKING") {
                blocks.push(LogBlock::Thinking { meta, body });
            } else if tag.contains("TOOL CALL") {
                blocks.push(LogBlock::ToolCall { meta, body });
            } else if tag.contains("TOOL RESULT") {
                blocks.push(LogBlock::ToolResult { meta, body });
            } else if tag.contains("ARENA REQUEST") {
                blocks.push(LogBlock::ArenaRequest { meta, body });
            } else if tag.contains("ARENA RESPONSE") {
                blocks.push(LogBlock::ArenaResponse { meta, body });
            } else {
                blocks.push(LogBlock::Info {
                    time: time.to_string(),
                    message: format!("{tag}\n{body}"),
                });
            }
            continue;
        }

        match level {
            "ERROR" => blocks.push(LogBlock::Error {
                time: time.to_string(),
                message: message.to_string(),
            }),
            "WARN" => blocks.push(LogBlock::Warn {
                time: time.to_string(),
                message: message.to_string(),
            }),
            _ => blocks.push(LogBlock::Info {
                time: time.to_string(),
                message: message.to_string(),
            }),
        }
        index += 1;
    }

    blocks
}

fn render_block(block: LogBlock) -> leptos::tachys::view::any_view::AnyView {
    match block {
        LogBlock::Prompt { meta, body } => view! {
            <div class="my-2 rounded border border-[#1f6feb44] bg-[#1f6feb11]">
                <div class="px-3 py-1.5 text-[10px] font-bold text-[#58a6ff] border-b border-[#1f6feb44] tracking-wide">
                    {meta}
                </div>
                <pre class="px-3 py-2 text-xs text-[#c9d1d9] whitespace-pre-wrap break-words m-0">{body}</pre>
            </div>
        }
        .into_any(),
        LogBlock::Response { meta, body } => view! {
            <div class="my-2 rounded border border-[#23863644] bg-[#23863611]">
                <div class="px-3 py-1.5 text-[10px] font-bold text-[#3fb950] border-b border-[#23863644] tracking-wide">
                    {meta}
                </div>
                <pre class="px-3 py-2 text-xs text-[#c9d1d9] whitespace-pre-wrap break-words m-0">{body}</pre>
            </div>
        }
        .into_any(),
        LogBlock::Thinking { meta, body } => view! {
            <div class="my-2 rounded border border-[#d2992244] bg-[#d2992211]">
                <div class="px-3 py-1.5 text-[10px] font-bold text-[#d29922] border-b border-[#d2992244] tracking-wide">
                    {meta}
                </div>
                <pre class="px-3 py-2 text-xs text-[#b0a080] whitespace-pre-wrap break-words m-0">{body}</pre>
            </div>
        }
        .into_any(),
        LogBlock::ToolCall { meta, body } => view! {
            <div class="my-1 rounded border border-[#8b5cf644] bg-[#8b5cf611]">
                <div class="px-3 py-1.5 text-[10px] font-bold text-[#a78bfa] border-b border-[#8b5cf644] tracking-wide">
                    {meta}
                </div>
                <pre class="px-3 py-2 text-xs text-[#8b949e] whitespace-pre-wrap break-words m-0">{body}</pre>
            </div>
        }
        .into_any(),
        LogBlock::ToolResult { meta, body } => view! {
            <div class="my-1 rounded border border-[#8b5cf644] bg-[#8b5cf608]">
                <div class="px-3 py-1.5 text-[10px] font-bold text-[#a78bfa] border-b border-[#8b5cf644] tracking-wide">
                    {meta}
                </div>
                <pre class="px-3 py-2 text-xs text-[#8b949e] whitespace-pre-wrap break-words m-0">{body}</pre>
            </div>
        }
        .into_any(),
        LogBlock::ArenaRequest { meta, body } => view! {
            <div class="my-1 rounded border border-[#f7830044] bg-[#f7830011]">
                <div class="px-3 py-1.5 text-[10px] font-bold text-[#f0883e] border-b border-[#f7830044] tracking-wide">
                    {meta}
                </div>
                <pre class="px-3 py-2 text-xs text-[#8b949e] whitespace-pre-wrap break-words m-0">{body}</pre>
            </div>
        }
        .into_any(),
        LogBlock::ArenaResponse { meta, body } => view! {
            <div class="my-1 rounded border border-[#f7830044] bg-[#f7830008]">
                <div class="px-3 py-1.5 text-[10px] font-bold text-[#f0883e] border-b border-[#f7830044] tracking-wide">
                    {meta}
                </div>
                <pre class="px-3 py-2 text-xs text-[#8b949e] whitespace-pre-wrap break-words m-0">{body}</pre>
            </div>
        }
        .into_any(),
        LogBlock::Warn { time, message } => view! {
            <div class="px-3 py-1 text-xs">
                <span class="text-[#484f58] mr-2">{time}</span>
                <span class="text-[#d29922]">{message}</span>
            </div>
        }
        .into_any(),
        LogBlock::Error { time, message } => view! {
            <div class="px-3 py-1 text-xs">
                <span class="text-[#484f58] mr-2">{time}</span>
                <span class="text-[#f85149]">{message}</span>
            </div>
        }
        .into_any(),
        LogBlock::Info { time, message } => view! {
            <div class="px-3 py-0.5 text-xs">
                <span class="text-[#484f58] mr-2">{time}</span>
                <span class="text-[#6e7681]">{message}</span>
            </div>
        }
        .into_any(),
    }
}

struct IntervalGuard {
    handle: i32,
    _closure: Closure<dyn Fn()>,
}

impl Drop for IntervalGuard {
    fn drop(&mut self) {
        if let Some(window) = web_sys::window() {
            window.clear_interval_with_handle(self.handle);
        }
    }
}

fn filter_button(
    label: &'static str,
    kind: BlockKind,
    active_filter: RwSignal<Option<BlockKind>>,
) -> impl IntoView {
    let is_active = move || active_filter.get() == Some(kind);
    view! {
        <button
            class=move || format!(
                "px-2 py-0.5 text-[10px] rounded border cursor-pointer transition-colors {}",
                if is_active() {
                    "bg-[#1f6feb33] text-[#58a6ff] border-[#1f6feb]"
                } else {
                    "bg-transparent text-[#8b949e] border-[#30363d] hover:text-[#c9d1d9]"
                }
            )
            on:click=move |_| {
                if active_filter.get() == Some(kind) {
                    active_filter.set(None);
                } else {
                    active_filter.set(Some(kind));
                }
            }
        >
            {label}
        </button>
    }
}

#[component]
pub fn LogsView() -> impl IntoView {
    let app = use_context::<AppState>().unwrap();
    let auto_refresh = RwSignal::new(true);
    let search_text = RwSignal::new(String::new());
    let active_filter = RwSignal::<Option<BlockKind>>::new(None);

    let on_refresh = move |_| {
        app.log_content.set(String::new());
        nightshade::webview::send(&FrontendCommand::ResetLogs);
        nightshade::webview::send(&FrontendCommand::ReadLogs);
    };

    let on_open_file = move |_| {
        nightshade::webview::send(&FrontendCommand::OpenLogFile);
    };

    Effect::new(move |prev: Option<Option<IntervalGuard>>| {
        drop(prev);
        let on_logs_tab = app.view.get() == View::Logs;
        let enabled = auto_refresh.get() && on_logs_tab;
        if on_logs_tab {
            nightshade::webview::send(&FrontendCommand::ReadLogs);
        }
        if enabled {
            let callback = Closure::wrap(Box::new(move || {
                nightshade::webview::send(&FrontendCommand::ReadLogs);
            }) as Box<dyn Fn()>);
            let handle = web_sys::window()
                .unwrap()
                .set_interval_with_callback_and_timeout_and_arguments_0(
                    callback.as_ref().unchecked_ref(),
                    2000,
                )
                .unwrap();
            Some(IntervalGuard {
                handle,
                _closure: callback,
            })
        } else {
            None
        }
    });

    view! {
        <div class="flex-1 flex flex-col min-h-0 p-4">
            <div class="flex items-center gap-3 mb-2">
                <h2 class="text-lg font-bold text-[#c9d1d9]">"Logs"</h2>
                <button
                    class="px-3 py-1 text-xs bg-[#21262d] text-[#c9d1d9] border border-[#30363d] rounded hover:bg-[#30363d] cursor-pointer"
                    on:click=on_refresh
                >
                    "Refresh"
                </button>
                <button
                    class="px-3 py-1 text-xs bg-[#21262d] text-[#c9d1d9] border border-[#30363d] rounded hover:bg-[#30363d] cursor-pointer"
                    on:click=on_open_file
                >
                    "Open File..."
                </button>
                <label class="flex items-center gap-1.5 text-xs text-[#8b949e] cursor-pointer select-none">
                    <input
                        type="checkbox"
                        class="cursor-pointer"
                        prop:checked=move || auto_refresh.get()
                        on:change=move |_| auto_refresh.update(|value| *value = !*value)
                    />
                    "Auto-refresh"
                </label>
            </div>
            <div class="flex items-center gap-2 mb-3">
                <input
                    type="text"
                    placeholder="Search logs..."
                    class="px-2 py-1 text-xs bg-[#0d1117] text-[#c9d1d9] border border-[#30363d] rounded w-48 placeholder-[#484f58] outline-none focus:border-[#1f6feb]"
                    prop:value=move || search_text.get()
                    on:input=move |event| {
                        search_text.set(event_target_value(&event));
                    }
                />
                {filter_button("Prompts", BlockKind::Prompt, active_filter)}
                {filter_button("Responses", BlockKind::Response, active_filter)}
                {filter_button("Thinking", BlockKind::Thinking, active_filter)}
                {filter_button("Tools", BlockKind::Tool, active_filter)}
                {filter_button("Arena", BlockKind::Arena, active_filter)}
                {filter_button("Errors", BlockKind::Error, active_filter)}
                {filter_button("Warnings", BlockKind::Warn, active_filter)}
                {filter_button("Info", BlockKind::Info, active_filter)}
            </div>
            <div class="flex-1 min-h-0 overflow-auto bg-[#0d1117] border border-[#30363d] rounded-lg flex flex-col-reverse">
                <div class="p-2 font-mono">
                    {move || {
                        let content = app.log_content.get();
                        if content.is_empty() {
                            view! { <div class="px-3 py-2 text-xs text-[#484f58]">"Loading..."</div> }.into_any()
                        } else {
                            let query = search_text.get();
                            let filter = active_filter.get();
                            let blocks = parse_blocks(&content);
                            blocks
                                .into_iter()
                                .filter(|block| {
                                    let kind_ok = filter.is_none_or(|kind| block.kind() == kind);
                                    let search_ok = block.matches_search(&query);
                                    kind_ok && search_ok
                                })
                                .map(render_block)
                                .collect_view()
                                .into_any()
                        }
                    }}
                </div>
            </div>
        </div>
    }
}
