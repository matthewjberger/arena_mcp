use arena_app_protocol::FrontendCommand;
use leptos::prelude::*;
use web_sys::wasm_bindgen::JsCast;

use crate::state::{AppState, ChatState, MessageRole, StatusDisplay};

#[component]
pub fn ChatView() -> impl IntoView {
    let app = use_context::<AppState>().unwrap();
    let chat = use_context::<ChatState>().unwrap();

    let is_busy = move || {
        !matches!(
            app.status.get(),
            StatusDisplay::Idle | StatusDisplay::Disconnected
        )
    };
    let can_send = move || !chat.input_text.get().trim().is_empty() && !is_busy();

    let send_prompt = move || {
        let text = chat.input_text.get_untracked();
        if text.trim().is_empty() {
            return;
        }
        chat.messages.update(|messages| {
            messages.push(crate::state::ChatMessage {
                role: MessageRole::User,
                content: text.clone(),
                thinking: String::new(),
                tool_uses: Vec::new(),
            });
        });
        nightshade::webview::send(&FrontendCommand::SendPrompt {
            prompt: text,
            session_id: app.session_id.get_untracked(),
            model: None,
        });
        chat.input_text.set(String::new());
    };

    let on_keydown = move |event: web_sys::KeyboardEvent| {
        if event.key() == "Enter" && event.ctrl_key() && can_send() {
            event.prevent_default();
            send_prompt();
        }
    };

    let on_send = move |_| {
        if can_send() {
            send_prompt();
        }
    };

    let on_cancel = move |_| {
        nightshade::webview::send(&FrontendCommand::CancelRequest);
    };

    view! {
        <div class="flex-1 flex flex-col min-h-0">
            <div class="flex-1 overflow-y-auto px-4 py-4" id="chat-scroll">
                {move || {
                    let msgs = chat.messages.get();
                    let is_thinking = matches!(app.status.get(), StatusDisplay::Thinking);
                    if msgs.is_empty() && chat.streaming_text.get().is_empty() && chat.thinking_text.get().is_empty() && !is_thinking {
                        view! {
                            <div class="flex flex-col items-center justify-center h-full text-[#484f58] text-sm gap-2">
                                <div>"Ask about items, BOMs, changes, or suppliers in Arena PLM"</div>
                                <div class="text-xs">"Claude can search items, view BOMs, look up ECOs, check where-used, and more"</div>
                            </div>
                        }.into_any()
                    } else {
                        view! {
                            <div>
                                {msgs.into_iter().map(|message| {
                                    let is_user = matches!(message.role, MessageRole::User);
                                    let container_class = if is_user { "flex justify-end mb-3" } else { "flex justify-start mb-3" };
                                    let bubble_class = if is_user {
                                        "max-w-[80%] px-4 py-2.5 rounded-lg bg-[#1f6feb] text-white"
                                    } else {
                                        "max-w-[80%] px-4 py-2.5 rounded-lg bg-[#161b22] text-[#c9d1d9] border border-[#30363d]"
                                    };
                                    let thinking = message.thinking.clone();
                                    let has_thinking = !thinking.is_empty();
                                    let tool_uses = message.tool_uses.clone();

                                    view! {
                                        <div class={container_class}>
                                            <div class={bubble_class}>
                                                {if has_thinking && !is_user {
                                                    Some(view! {
                                                        <details class="mb-2">
                                                            <summary class="text-xs text-yellow-500 cursor-pointer hover:text-yellow-400">"Thinking"</summary>
                                                            <pre class="whitespace-pre-wrap break-words font-mono text-xs leading-relaxed mt-1 text-[#8b949e] pl-3 border-l-2 border-[#30363d]">{thinking}</pre>
                                                        </details>
                                                    })
                                                } else {
                                                    None
                                                }}
                                                <pre class="whitespace-pre-wrap break-words font-mono text-sm leading-relaxed m-0">{message.content}</pre>
                                                {if !tool_uses.is_empty() {
                                                    Some(view! {
                                                        <div class="mt-2">
                                                            {tool_uses.into_iter().map(|tool| {
                                                                view! {
                                                                    <div class="my-1 border border-[#30363d] rounded-md overflow-hidden">
                                                                        <details>
                                                                            <summary class="flex items-center gap-2 px-3 py-1.5 bg-[#161b22] text-xs cursor-pointer hover:bg-[#1c2129]">
                                                                                <span class="text-purple-400 font-medium">{tool.tool_name}</span>
                                                                                {if tool.finished {
                                                                                    view! { <span class="text-green-500 ml-auto">"done"</span> }.into_any()
                                                                                } else {
                                                                                    view! { <span class="text-yellow-500 ml-auto animate-pulse">"running..."</span> }.into_any()
                                                                                }}
                                                                            </summary>
                                                                            {if !tool.input_json.is_empty() {
                                                                                Some(view! {
                                                                                    <pre class="px-3 py-2 text-xs text-[#8b949e] bg-[#0d1117] overflow-x-auto whitespace-pre-wrap break-all">{tool.input_json}</pre>
                                                                                })
                                                                            } else {
                                                                                None
                                                                            }}
                                                                        </details>
                                                                    </div>
                                                                }
                                                            }).collect_view()}
                                                        </div>
                                                    })
                                                } else {
                                                    None
                                                }}
                                            </div>
                                        </div>
                                    }
                                }).collect_view()}

                                // Streaming bubble
                                {move || {
                                    let thinking = chat.thinking_text.get();
                                    let text = chat.streaming_text.get();
                                    let tools = chat.active_tools.get();
                                    let current_status = app.status.get();
                                    let is_thinking = matches!(current_status, StatusDisplay::Thinking);
                                    let is_active = !text.is_empty() || !tools.is_empty() || !thinking.is_empty() || is_thinking;

                                    if is_active {
                                        Some(view! {
                                            <div class="flex justify-start mb-3">
                                                <div class="max-w-[80%] px-4 py-2.5 rounded-lg bg-[#161b22] text-[#c9d1d9] border border-[#30363d]">
                                                    {if !thinking.is_empty() {
                                                        view! {
                                                            <div class="mb-3 pb-3 border-b border-[#30363d]">
                                                                <span class="text-yellow-500 text-xs">"Thinking"</span>
                                                                <pre class="whitespace-pre-wrap break-words font-mono text-xs leading-relaxed mt-1 text-[#8b949e]">{thinking}</pre>
                                                            </div>
                                                        }.into_any()
                                                    } else if is_thinking && text.is_empty() {
                                                        view! {
                                                            <div class="mb-3 pb-3 border-b border-[#30363d]">
                                                                <span class="text-yellow-500 text-xs animate-pulse">"Thinking..."</span>
                                                            </div>
                                                        }.into_any()
                                                    } else {
                                                        view! { <div></div> }.into_any()
                                                    }}
                                                    {if !text.is_empty() {
                                                        Some(view! {
                                                            <pre class="whitespace-pre-wrap break-words font-mono text-sm leading-relaxed m-0">{text}</pre>
                                                        })
                                                    } else {
                                                        None
                                                    }}
                                                    {if !tools.is_empty() {
                                                        Some(view! {
                                                            <div class="mt-2">
                                                                {tools.into_iter().map(|tool| {
                                                                    view! {
                                                                        <div class="my-1 border border-[#30363d] rounded-md overflow-hidden">
                                                                            <div class="flex items-center gap-2 px-3 py-1.5 bg-[#161b22] text-xs">
                                                                                <span class="text-purple-400 font-medium">{tool.tool_name}</span>
                                                                                {if tool.finished {
                                                                                    view! { <span class="text-green-500 ml-auto">"done"</span> }.into_any()
                                                                                } else {
                                                                                    view! { <span class="text-yellow-500 ml-auto animate-pulse">"running..."</span> }.into_any()
                                                                                }}
                                                                            </div>
                                                                        </div>
                                                                    }
                                                                }).collect_view()}
                                                            </div>
                                                        })
                                                    } else {
                                                        None
                                                    }}
                                                    <span class="inline-block w-2 h-4 bg-[#c9d1d9] animate-pulse ml-0.5"></span>
                                                </div>
                                            </div>
                                        })
                                    } else {
                                        None
                                    }
                                }}
                            </div>
                        }.into_any()
                    }
                }}
            </div>

            <div class="px-4 py-3 bg-[#161b22] border-t border-[#30363d]">
                <div class="flex gap-2">
                    <textarea
                        class="flex-1 bg-[#0d1117] text-[#c9d1d9] border border-[#30363d] rounded-lg px-3 py-2 text-sm font-mono resize-none focus:outline-none focus:border-[#58a6ff] placeholder-[#484f58]"
                        placeholder="Ask about Arena PLM data... (Ctrl+Enter to send)"
                        rows="3"
                        prop:value=move || chat.input_text.get()
                        on:input=move |event| {
                            let target = event.target().unwrap();
                            let textarea: web_sys::HtmlTextAreaElement = target.unchecked_into();
                            chat.input_text.set(textarea.value());
                        }
                        on:keydown=on_keydown
                    />
                    <div class="flex flex-col gap-1">
                        <button
                            class="px-4 py-2 bg-[#238636] text-white text-sm rounded-lg hover:bg-[#2ea043] disabled:opacity-40 disabled:cursor-not-allowed cursor-pointer"
                            on:click=on_send
                            disabled=move || !can_send()
                        >
                            "Send"
                        </button>
                        <button
                            class="px-4 py-2 bg-[#da3633] text-white text-sm rounded-lg hover:bg-[#f85149] disabled:opacity-40 disabled:cursor-not-allowed cursor-pointer"
                            on:click=on_cancel
                            disabled=move || !is_busy()
                        >
                            "Cancel"
                        </button>
                    </div>
                </div>
            </div>
        </div>
    }
}
