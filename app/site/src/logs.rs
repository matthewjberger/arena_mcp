use arena_app_protocol::FrontendCommand;
use leptos::prelude::*;
use wasm_bindgen::prelude::*;

use crate::state::{AppState, View};

enum LogBlock {
    Prompt { meta: String, body: String },
    Response { meta: String, body: String },
    Thinking { meta: String, body: String },
    ToolCall { meta: String },
    ToolResult { meta: String, body: String },
    ArenaRequest { meta: String, body: String },
    ArenaResponse { meta: String, body: String },
    Info { time: String, message: String },
    Warn { time: String, message: String },
    Error { time: String, message: String },
}

fn strip_prefix(line: &str) -> (&str, &str, &str) {
    let rest = line;
    let time = if rest.len() >= 27 && rest.as_bytes()[4] == b'-' {
        &rest[11..19]
    } else {
        ""
    };
    let level = if rest.contains(" INFO ") {
        "INFO"
    } else if rest.contains(" WARN ") {
        "WARN"
    } else if rest.contains(" ERROR ") {
        "ERROR"
    } else {
        ""
    };
    let message = rest
        .find("arena_app: ")
        .map(|index| &rest[index + 11..])
        .or_else(|| rest.find("arena_app:").map(|index| &rest[index + 10..]))
        .unwrap_or(rest);
    (time, level, message.trim())
}

fn parse_blocks(raw: &str) -> Vec<LogBlock> {
    let mut blocks = Vec::new();
    let lines: Vec<&str> = raw.lines().collect();
    let mut index = 0;

    while index < lines.len() {
        let line = lines[index];
        let (time, level, message) = strip_prefix(line);

        if message.starts_with("\u{2501}\u{2501}\u{2501}") {
            let tag = message;
            let mut body_lines = Vec::new();
            index += 1;
            while index < lines.len() {
                let next = lines[index];
                let is_new_header = next.len() >= 27
                    && next.as_bytes().get(4) == Some(&b'-')
                    && next.as_bytes().get(10) == Some(&b'T');
                if is_new_header {
                    break;
                }
                body_lines.push(next);
                index += 1;
            }
            let body = body_lines.join("\n");

            if tag.contains("PROMPT") {
                let meta = tag
                    .trim_start_matches('\u{2501}')
                    .trim_end_matches('\u{2501}')
                    .trim_start_matches(' ')
                    .trim_end_matches(' ')
                    .to_string();
                blocks.push(LogBlock::Prompt { meta, body });
            } else if tag.contains("RESPONSE") {
                let meta = tag
                    .trim_start_matches('\u{2501}')
                    .trim_end_matches('\u{2501}')
                    .trim()
                    .to_string();
                blocks.push(LogBlock::Response { meta, body });
            } else if tag.contains("THINKING") {
                let meta = tag
                    .trim_start_matches('\u{2501}')
                    .trim_end_matches('\u{2501}')
                    .trim()
                    .to_string();
                blocks.push(LogBlock::Thinking { meta, body });
            } else if tag.contains("TOOL CALL") {
                let meta = tag
                    .trim_start_matches('\u{2501}')
                    .trim_end_matches('\u{2501}')
                    .trim()
                    .to_string();
                blocks.push(LogBlock::ToolCall { meta });
            } else if tag.contains("TOOL RESULT") {
                let meta = tag
                    .trim_start_matches('\u{2501}')
                    .trim_end_matches('\u{2501}')
                    .trim()
                    .to_string();
                blocks.push(LogBlock::ToolResult { meta, body });
            } else if tag.contains("ARENA REQUEST") {
                let meta = tag
                    .trim_start_matches('\u{2501}')
                    .trim_end_matches('\u{2501}')
                    .trim()
                    .to_string();
                blocks.push(LogBlock::ArenaRequest { meta, body });
            } else if tag.contains("ARENA RESPONSE") {
                let meta = tag
                    .trim_start_matches('\u{2501}')
                    .trim_end_matches('\u{2501}')
                    .trim()
                    .to_string();
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
        LogBlock::ToolCall { meta } => view! {
            <div class="my-1 px-3 py-1.5 rounded border border-[#8b5cf644] bg-[#8b5cf611] text-[10px] font-bold text-[#a78bfa] tracking-wide">
                {meta}
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

#[component]
pub fn LogsView() -> impl IntoView {
    let app = use_context::<AppState>().unwrap();
    let auto_refresh = RwSignal::new(true);

    let on_refresh = move |_| {
        nightshade::webview::send(&FrontendCommand::ReadLogs);
    };

    let on_open_file = move |_| {
        nightshade::webview::send(&FrontendCommand::OpenLogFile);
    };

    Effect::new(move |prev: Option<Option<i32>>| {
        if let Some(Some(handle)) = prev {
            web_sys::window()
                .unwrap()
                .clear_interval_with_handle(handle);
        }
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
            callback.forget();
            Some(handle)
        } else {
            None
        }
    });

    view! {
        <div class="flex-1 flex flex-col min-h-0 p-4">
            <div class="flex items-center gap-3 mb-3">
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
            <div class="flex-1 min-h-0 overflow-auto bg-[#0d1117] border border-[#30363d] rounded-lg flex flex-col-reverse">
                <div class="p-2 font-mono">
                    {move || {
                        let content = app.log_content.get();
                        if content.is_empty() {
                            view! { <div class="px-3 py-2 text-xs text-[#484f58]">"Loading..."</div> }.into_any()
                        } else {
                            let blocks = parse_blocks(&content);
                            blocks.into_iter().map(render_block).collect_view().into_any()
                        }
                    }}
                </div>
            </div>
        </div>
    }
}
