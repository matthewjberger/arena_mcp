use std::cell::Cell;

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

#[derive(Clone)]
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
}

#[derive(Clone)]
struct SearchableBlock {
    block: LogBlock,
    search_text: String,
}

impl SearchableBlock {
    fn new(block: LogBlock) -> Self {
        let (first, second) = block.text_content();
        let search_text = format!("{first} {second}").to_lowercase();
        Self { block, search_text }
    }

    fn matches_search(&self, query_lower: &str) -> bool {
        query_lower.is_empty() || self.search_text.contains(query_lower)
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

fn parse_from(lines: &[&str], start: usize) -> (Vec<SearchableBlock>, usize) {
    let mut blocks = Vec::new();
    let mut index = start;
    let mut last_header_line = start;

    while index < lines.len() {
        let line = lines[index];

        if !is_log_header(line) {
            index += 1;
            continue;
        }

        last_header_line = index;
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

            let block = if tag.contains("PROMPT") {
                LogBlock::Prompt { meta, body }
            } else if tag.contains("RESPONSE") {
                LogBlock::Response { meta, body }
            } else if tag.contains("THINKING") {
                LogBlock::Thinking { meta, body }
            } else if tag.contains("TOOL CALL") {
                LogBlock::ToolCall { meta, body }
            } else if tag.contains("TOOL RESULT") {
                LogBlock::ToolResult { meta, body }
            } else if tag.contains("ARENA REQUEST") {
                LogBlock::ArenaRequest { meta, body }
            } else if tag.contains("ARENA RESPONSE") {
                LogBlock::ArenaResponse { meta, body }
            } else {
                LogBlock::Info {
                    time: time.to_string(),
                    message: format!("{tag}\n{body}"),
                }
            };
            blocks.push(SearchableBlock::new(block));
            continue;
        }

        let block = match level {
            "ERROR" => LogBlock::Error {
                time: time.to_string(),
                message: message.to_string(),
            },
            "WARN" => LogBlock::Warn {
                time: time.to_string(),
                message: message.to_string(),
            },
            _ => LogBlock::Info {
                time: time.to_string(),
                message: message.to_string(),
            },
        };
        blocks.push(SearchableBlock::new(block));
        index += 1;
    }

    (blocks, last_header_line)
}

fn render_block(block: &LogBlock) -> leptos::tachys::view::any_view::AnyView {
    match block.clone() {
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

    let blocks_signal = RwSignal::new(Vec::<SearchableBlock>::new());
    let parser_resume_line = Cell::new(0usize);
    let parser_complete_count = Cell::new(0usize);
    let parser_content_len = Cell::new(0usize);

    Effect::new(move |_| {
        let content = app.log_content.get();
        let current_len = content.len();
        let previous_len = parser_content_len.get();

        if current_len == previous_len {
            return;
        }

        let lines: Vec<&str> = content.lines().collect();

        if current_len < previous_len || previous_len == 0 {
            let (new_blocks, last_header) = parse_from(&lines, 0);
            let complete = new_blocks.len().saturating_sub(1);
            blocks_signal.set(new_blocks);
            parser_resume_line.set(last_header);
            parser_complete_count.set(complete);
        } else {
            let resume = parser_resume_line.get();
            let truncate_to = parser_complete_count.get();
            let (new_blocks, last_header) = parse_from(&lines, resume);
            blocks_signal.update(|existing| {
                existing.truncate(truncate_to);
                existing.extend(new_blocks);
            });
            let new_complete =
                blocks_signal.with_untracked(|blocks| blocks.len().saturating_sub(1));
            parser_resume_line.set(last_header);
            parser_complete_count.set(new_complete);
        }

        parser_content_len.set(current_len);
    });

    let on_open_file = move |_| {
        app.log_content.set(String::new());
        nightshade::webview::send(&FrontendCommand::OpenLogFile);
    };

    Effect::new(move |prev: Option<Option<IntervalGuard>>| {
        drop(prev);
        let on_logs_tab = app.view.get() == View::Logs;
        let enabled = auto_refresh.get() && on_logs_tab;
        if enabled {
            app.log_content.set(String::new());
            nightshade::webview::send(&FrontendCommand::ResetLogs);
            nightshade::webview::send(&FrontendCommand::ReadLogs);
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
                    class=move || format!(
                        "px-3 py-1 text-xs border border-[#30363d] rounded {} {}",
                        if auto_refresh.get() {
                            "bg-[#21262d44] text-[#484f58] cursor-default"
                        } else {
                            "bg-[#21262d] text-[#c9d1d9] hover:bg-[#30363d] cursor-pointer"
                        },
                        ""
                    )
                    disabled=move || auto_refresh.get()
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
                        blocks_signal.with(|all_blocks| {
                            if all_blocks.is_empty() {
                                view! { <div class="px-3 py-2 text-xs text-[#484f58]">"Loading..."</div> }.into_any()
                            } else {
                                let query_lower = search_text.get().to_lowercase();
                                let filter = active_filter.get();
                                all_blocks
                                    .iter()
                                    .filter(|searchable_block| {
                                        let kind_ok = filter
                                            .is_none_or(|kind| searchable_block.block.kind() == kind);
                                        kind_ok && searchable_block.matches_search(&query_lower)
                                    })
                                    .map(|searchable_block| render_block(&searchable_block.block))
                                    .collect_view()
                                    .into_any()
                            }
                        })
                    }}
                </div>
            </div>
        </div>
    }
}
