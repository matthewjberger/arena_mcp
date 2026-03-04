use arena_app_protocol::FrontendCommand;
use leptos::prelude::*;
use web_sys::wasm_bindgen::JsCast;

use crate::state::LoginState;

#[component]
pub fn LoginScreen() -> impl IntoView {
    let login_state = use_context::<LoginState>().unwrap();

    let email = RwSignal::new(String::new());
    let password = RwSignal::new(String::new());
    let workspace_id = RwSignal::new(String::new());
    let base_url = RwSignal::new(String::new());

    let can_submit = move || {
        !email.get().trim().is_empty()
            && !password.get().trim().is_empty()
            && !login_state.loading.get()
    };

    let on_submit = move |_| {
        if !can_submit() {
            return;
        }
        login_state.loading.set(true);
        login_state.error.set(None);

        let ws = workspace_id.get();
        let bu = base_url.get();
        nightshade::webview::send(&FrontendCommand::Login {
            email: email.get(),
            password: password.get(),
            workspace_id: if ws.trim().is_empty() { None } else { Some(ws) },
            base_url: if bu.trim().is_empty() { None } else { Some(bu) },
        });
    };

    let on_keydown = move |event: web_sys::KeyboardEvent| {
        if event.key() == "Enter" && can_submit() {
            event.prevent_default();
            login_state.loading.set(true);
            login_state.error.set(None);

            let ws = workspace_id.get();
            let bu = base_url.get();
            nightshade::webview::send(&FrontendCommand::Login {
                email: email.get(),
                password: password.get(),
                workspace_id: if ws.trim().is_empty() { None } else { Some(ws) },
                base_url: if bu.trim().is_empty() { None } else { Some(bu) },
            });
        }
    };

    let input_class = "w-full bg-[#0d1117] text-[#c9d1d9] border border-[#30363d] rounded-lg px-3 py-2 text-sm font-mono focus:outline-none focus:border-[#58a6ff] placeholder-[#484f58]";

    view! {
        <div class="h-screen flex items-center justify-center bg-[#0d1117]" on:keydown=on_keydown>
            <div class="w-96 bg-[#161b22] border border-[#30363d] rounded-xl p-8">
                <div class="text-center mb-6">
                    <h1 class="text-xl font-bold text-[#c9d1d9] tracking-wide">"ARENA PLM"</h1>
                    <p class="text-xs text-[#8b949e] mt-1">"Sign in to your workspace"</p>
                </div>

                {move || login_state.error.get().map(|error| view! {
                    <div class="mb-4 px-3 py-2 bg-[#490202] border border-[#da3633] rounded-lg text-xs text-[#f85149]">
                        {error}
                    </div>
                })}

                <div class="space-y-4">
                    <div>
                        <label class="block text-xs text-[#8b949e] mb-1">"Email"</label>
                        <input
                            type="email"
                            class={input_class}
                            placeholder="user@company.com"
                            prop:value=move || email.get()
                            on:input=move |event| {
                                let target = event.target().unwrap();
                                let input: web_sys::HtmlInputElement = target.unchecked_into();
                                email.set(input.value());
                            }
                        />
                    </div>
                    <div>
                        <label class="block text-xs text-[#8b949e] mb-1">"Password"</label>
                        <input
                            type="password"
                            class={input_class}
                            placeholder="Password"
                            prop:value=move || password.get()
                            on:input=move |event| {
                                let target = event.target().unwrap();
                                let input: web_sys::HtmlInputElement = target.unchecked_into();
                                password.set(input.value());
                            }
                        />
                    </div>
                    <div>
                        <label class="block text-xs text-[#8b949e] mb-1">"Workspace ID " <span class="text-[#484f58]">"(optional)"</span></label>
                        <input
                            type="text"
                            class={input_class}
                            placeholder="Numeric workspace ID"
                            prop:value=move || workspace_id.get()
                            on:input=move |event| {
                                let target = event.target().unwrap();
                                let input: web_sys::HtmlInputElement = target.unchecked_into();
                                workspace_id.set(input.value());
                            }
                        />
                    </div>
                    <div>
                        <label class="block text-xs text-[#8b949e] mb-1">"Base URL " <span class="text-[#484f58]">"(optional)"</span></label>
                        <input
                            type="text"
                            class={input_class}
                            placeholder="https://api.arenasolutions.com/v1"
                            prop:value=move || base_url.get()
                            on:input=move |event| {
                                let target = event.target().unwrap();
                                let input: web_sys::HtmlInputElement = target.unchecked_into();
                                base_url.set(input.value());
                            }
                        />
                    </div>
                    <button
                        class="w-full px-4 py-2.5 bg-[#238636] text-white text-sm rounded-lg hover:bg-[#2ea043] disabled:opacity-40 disabled:cursor-not-allowed cursor-pointer font-medium"
                        on:click=on_submit
                        disabled=move || !can_submit()
                    >
                        {move || if login_state.loading.get() { "Signing in..." } else { "Sign In" }}
                    </button>
                </div>
            </div>
        </div>
    }
}
