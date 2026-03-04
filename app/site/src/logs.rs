use arena_app_protocol::FrontendCommand;
use leptos::prelude::*;
use wasm_bindgen::prelude::*;

use crate::state::{AppState, View};

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
