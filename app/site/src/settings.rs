use arena_app_protocol::FrontendCommand;
use leptos::prelude::*;

use crate::state::{AppState, SavedSearch};

#[component]
pub fn SettingsView() -> impl IntoView {
    let app = use_context::<AppState>().unwrap();
    let saved_searches = RwSignal::new(load_saved_searches());

    let on_write_toggle = move |_| {
        let new_value = !app.write_mode.get();
        nightshade::webview::send(&FrontendCommand::SetWriteMode { enabled: new_value });
    };

    let on_logout = move |_| {
        nightshade::webview::send(&FrontendCommand::Logout);
        app.logged_in.set(false);
        app.rate_limit.set(None);
    };

    let delete_search = move |index: usize| {
        saved_searches.update(|searches| {
            if index < searches.len() {
                searches.remove(index);
            }
        });
        persist_saved_searches(&saved_searches.get_untracked());
    };

    view! {
        <div class="flex-1 p-6 overflow-auto">
            <h2 class="text-lg font-bold text-[#c9d1d9] mb-6">"Settings"</h2>

            <div class="space-y-6 max-w-lg">
                <div class="bg-[#161b22] border border-[#30363d] rounded-lg p-4">
                    <div class="flex items-center justify-between">
                        <div>
                            <h3 class="text-sm font-medium text-[#c9d1d9]">"Write Mode"</h3>
                            <p class="text-xs text-[#8b949e] mt-1">"Enable write operations (create, update, delete) via Claude"</p>
                        </div>
                        <button
                            class=move || format!(
                                "w-12 h-6 rounded-full transition-colors duration-200 cursor-pointer {}",
                                if app.write_mode.get() { "bg-[#238636]" } else { "bg-[#30363d]" }
                            )
                            on:click=on_write_toggle
                        >
                            <div class=move || format!(
                                "w-5 h-5 bg-white rounded-full shadow transition-transform duration-200 {}",
                                if app.write_mode.get() { "translate-x-6" } else { "translate-x-0.5" }
                            )></div>
                        </button>
                    </div>
                    {move || if app.write_mode.get() {
                        Some(view! {
                            <div class="mt-3 px-3 py-2 bg-[#4903023d] border border-[#da363366] rounded text-xs text-[#f85149]">
                                "Write mode is ON. Claude can create, update, and delete Arena data."
                            </div>
                        })
                    } else {
                        None
                    }}
                </div>

                <div class="bg-[#161b22] border border-[#30363d] rounded-lg p-4">
                    <h3 class="text-sm font-medium text-[#c9d1d9] mb-3">"Saved Searches"</h3>
                    {move || {
                        let searches = saved_searches.get();
                        if searches.is_empty() {
                            view! {
                                <div class="text-xs text-[#484f58]">"No saved searches yet"</div>
                            }.into_any()
                        } else {
                            view! {
                                <div class="space-y-2">
                                    {searches.into_iter().enumerate().map(|(index, search)| {
                                        view! {
                                            <div class="flex items-center justify-between py-1.5 px-2 bg-[#0d1117] rounded">
                                                <div>
                                                    <span class="text-sm text-[#c9d1d9]">{search.label}</span>
                                                    <span class="text-xs text-[#484f58] ml-2">{search.view}</span>
                                                </div>
                                                <button
                                                    class="text-xs text-[#da3633] hover:text-[#f85149] cursor-pointer"
                                                    on:click=move |_| delete_search(index)
                                                >
                                                    "Delete"
                                                </button>
                                            </div>
                                        }
                                    }).collect_view()}
                                </div>
                            }.into_any()
                        }
                    }}
                </div>

                <div class="bg-[#161b22] border border-[#30363d] rounded-lg p-4">
                    <h3 class="text-sm font-medium text-[#c9d1d9] mb-3">"Account"</h3>
                    <button
                        class="px-4 py-2 bg-[#da3633] text-white text-sm rounded-lg hover:bg-[#f85149] cursor-pointer"
                        on:click=on_logout
                    >
                        "Sign Out"
                    </button>
                </div>
            </div>
        </div>
    }
}

fn load_saved_searches() -> Vec<SavedSearch> {
    let window = web_sys::window().unwrap();
    let storage = window.local_storage().ok().flatten();
    storage
        .and_then(|storage| storage.get_item("arena_saved_searches").ok().flatten())
        .and_then(|json| serde_json::from_str(&json).ok())
        .unwrap_or_default()
}

fn persist_saved_searches(searches: &[SavedSearch]) {
    let window = web_sys::window().unwrap();
    if let Some(storage) = window.local_storage().ok().flatten()
        && let Ok(json) = serde_json::to_string(searches)
    {
        let _ = storage.set_item("arena_saved_searches", &json);
    }
}
