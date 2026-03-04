use arena_app_protocol::FrontendCommand;
use leptos::prelude::*;

use crate::state::AppState;

#[component]
pub fn LogsView() -> impl IntoView {
    let app = use_context::<AppState>().unwrap();

    let on_refresh = move |_| {
        nightshade::webview::send(&FrontendCommand::ReadLogs);
    };

    let on_open_file = move |_| {
        nightshade::webview::send(&FrontendCommand::OpenLogFile);
    };

    Effect::new(move |_| {
        nightshade::webview::send(&FrontendCommand::ReadLogs);
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
            </div>
            <div class="flex-1 min-h-0 overflow-auto bg-[#0d1117] border border-[#30363d] rounded-lg">
                <pre class="p-3 text-xs text-[#8b949e] whitespace-pre-wrap break-words m-0">
                    {move || {
                        let content = app.log_content.get();
                        if content.is_empty() {
                            "Loading...".to_string()
                        } else {
                            content
                        }
                    }}
                </pre>
            </div>
        </div>
    }
}
