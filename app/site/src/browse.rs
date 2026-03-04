use leptos::prelude::*;
use wasm_bindgen_futures::spawn_local;

use crate::arena;

#[derive(Clone, PartialEq)]
enum SortColumn {
    Number,
    Name,
    Category,
    Phase,
}

#[derive(Clone, PartialEq)]
enum SortDir {
    Asc,
    Desc,
}

#[component]
pub fn ItemsBrowse() -> impl IntoView {
    let search_text = RwSignal::new(String::new());
    let items = RwSignal::new(Vec::<arena::ItemRow>::new());
    let loading = RwSignal::new(false);
    let error = RwSignal::new(Option::<String>::None);
    let sort_col = RwSignal::new(SortColumn::Number);
    let sort_dir = RwSignal::new(SortDir::Asc);
    let selected_guid = RwSignal::new(Option::<String>::None);

    let do_search = move || {
        let query = search_text.get();
        loading.set(true);
        error.set(None);

        let name_filter = if query.trim().is_empty() {
            Some("*".to_string())
        } else {
            Some(format!("*{}*", query.trim()))
        };

        spawn_local(async move {
            match arena::search_items(None, name_filter, None, None, None, None, Some(50)).await {
                Ok(json) => {
                    let parsed: Result<arena::ListResponse<arena::ItemRow>, _> =
                        serde_json::from_str(&json);
                    match parsed {
                        Ok(list) => items.set(list.results.unwrap_or_default()),
                        Err(parse_error) => error.set(Some(format!("Parse error: {parse_error}"))),
                    }
                }
                Err(message) => error.set(Some(message)),
            }
            loading.set(false);
        });
    };

    let on_search_keydown = move |event: web_sys::KeyboardEvent| {
        if event.key() == "Enter" {
            do_search();
        }
    };

    let toggle_sort = move |col: SortColumn| {
        if sort_col.get() == col {
            sort_dir.update(|dir| {
                *dir = if *dir == SortDir::Asc {
                    SortDir::Desc
                } else {
                    SortDir::Asc
                };
            });
        } else {
            sort_col.set(col);
            sort_dir.set(SortDir::Asc);
        }
    };

    let sorted_items = move || {
        let mut rows = items.get();
        let col = sort_col.get();
        let dir = sort_dir.get();
        rows.sort_by(|left, right| {
            let cmp = match col {
                SortColumn::Number => left
                    .number
                    .as_deref()
                    .unwrap_or("")
                    .cmp(right.number.as_deref().unwrap_or("")),
                SortColumn::Name => left
                    .name
                    .as_deref()
                    .unwrap_or("")
                    .cmp(right.name.as_deref().unwrap_or("")),
                SortColumn::Category => {
                    let left_cat = left
                        .category
                        .as_ref()
                        .and_then(|category| category.name.as_deref())
                        .unwrap_or("");
                    let right_cat = right
                        .category
                        .as_ref()
                        .and_then(|category| category.name.as_deref())
                        .unwrap_or("");
                    left_cat.cmp(right_cat)
                }
                SortColumn::Phase => {
                    let left_phase = left
                        .lifecycle_phase
                        .as_ref()
                        .and_then(|phase| phase.name.as_deref())
                        .unwrap_or("");
                    let right_phase = right
                        .lifecycle_phase
                        .as_ref()
                        .and_then(|phase| phase.name.as_deref())
                        .unwrap_or("");
                    left_phase.cmp(right_phase)
                }
            };
            if dir == SortDir::Desc {
                cmp.reverse()
            } else {
                cmp
            }
        });
        rows
    };

    let sort_indicator = move |col: SortColumn| {
        if sort_col.get() == col {
            if sort_dir.get() == SortDir::Asc {
                " ▲"
            } else {
                " ▼"
            }
        } else {
            ""
        }
    };

    let th_class = "px-3 py-2 text-left text-xs font-medium text-[#8b949e] cursor-pointer hover:text-[#c9d1d9] select-none";

    view! {
        <div class="flex-1 flex flex-col min-h-0 p-4">
            <div class="flex gap-2 mb-4">
                <input
                    type="text"
                    class="flex-1 bg-[#0d1117] text-[#c9d1d9] border border-[#30363d] rounded-lg px-3 py-2 text-sm font-mono focus:outline-none focus:border-[#58a6ff] placeholder-[#484f58]"
                    placeholder="Search items by name..."
                    prop:value=move || search_text.get()
                    on:input=move |event| {
                        use web_sys::wasm_bindgen::JsCast;
                        let target = event.target().unwrap();
                        let input: web_sys::HtmlInputElement = target.unchecked_into();
                        search_text.set(input.value());
                    }
                    on:keydown=on_search_keydown
                />
                <button
                    class="px-4 py-2 bg-[#238636] text-white text-sm rounded-lg hover:bg-[#2ea043] disabled:opacity-40 cursor-pointer"
                    on:click=move |_| do_search()
                    disabled=move || loading.get()
                >
                    {move || if loading.get() { "Searching..." } else { "Search" }}
                </button>
            </div>

            {move || error.get().map(|message| view! {
                <div class="mb-4 px-3 py-2 bg-[#490202] border border-[#da3633] rounded-lg text-xs text-[#f85149]">
                    {message}
                </div>
            })}

            <div class="flex-1 overflow-auto border border-[#30363d] rounded-lg">
                <table class="w-full">
                    <thead class="bg-[#161b22] sticky top-0">
                        <tr>
                            <th class={th_class} on:click=move |_| toggle_sort(SortColumn::Number)>{move || format!("Number{}", sort_indicator(SortColumn::Number))}</th>
                            <th class={th_class} on:click=move |_| toggle_sort(SortColumn::Name)>{move || format!("Name{}", sort_indicator(SortColumn::Name))}</th>
                            <th class={th_class} on:click=move |_| toggle_sort(SortColumn::Category)>{move || format!("Category{}", sort_indicator(SortColumn::Category))}</th>
                            <th class={th_class} on:click=move |_| toggle_sort(SortColumn::Phase)>{move || format!("Phase{}", sort_indicator(SortColumn::Phase))}</th>
                        </tr>
                    </thead>
                    <tbody>
                        {move || sorted_items().into_iter().map(|item| {
                            let guid = item.guid.clone().unwrap_or_default();
                            let guid_click = guid.clone();
                            let is_selected = move || selected_guid.get().as_deref() == Some(&guid);
                            view! {
                                <tr
                                    class=move || if is_selected() {
                                        "border-b border-[#30363d] bg-[#1f6feb33] cursor-pointer"
                                    } else {
                                        "border-b border-[#30363d] hover:bg-[#161b22] cursor-pointer"
                                    }
                                    on:click=move |_| selected_guid.set(Some(guid_click.clone()))
                                >
                                    <td class="px-3 py-2 text-sm text-[#58a6ff]">{item.number.unwrap_or_default()}</td>
                                    <td class="px-3 py-2 text-sm">{item.name.unwrap_or_default()}</td>
                                    <td class="px-3 py-2 text-sm text-[#8b949e]">{item.category.and_then(|category| category.name).unwrap_or_default()}</td>
                                    <td class="px-3 py-2 text-sm text-[#8b949e]">{item.lifecycle_phase.and_then(|phase| phase.name).unwrap_or_default()}</td>
                                </tr>
                            }
                        }).collect_view()}
                    </tbody>
                </table>
            </div>

            {move || selected_guid.get().map(|guid| view! {
                <ItemDetail guid=guid on_close=move || selected_guid.set(None) />
            })}
        </div>
    }
}

#[component]
fn ItemDetail(guid: String, on_close: impl Fn() + 'static) -> impl IntoView {
    let item_json = RwSignal::new(Option::<String>::None);
    let files = RwSignal::new(Vec::<arena::FileRow>::new());
    let revisions = RwSignal::new(Vec::<arena::RevisionRow>::new());
    let loading = RwSignal::new(true);

    let guid_for_detail = guid.clone();
    let guid_clone = guid;
    spawn_local(async move {
        if let Ok(json) = arena::get_item(guid_clone.clone()).await {
            item_json.set(Some(json));
        }
        if let Ok(json) = arena::get_item_files(guid_clone.clone()).await
            && let Ok(list) = serde_json::from_str::<arena::ListResponse<arena::FileRow>>(&json)
        {
            files.set(list.results.unwrap_or_default());
        }
        if let Ok(json) = arena::get_item_revisions(guid_clone).await
            && let Ok(list) = serde_json::from_str::<arena::ListResponse<arena::RevisionRow>>(&json)
        {
            revisions.set(list.results.unwrap_or_default());
        }
        loading.set(false);
    });

    view! {
        <div class="mt-4 border border-[#30363d] rounded-lg bg-[#161b22] p-4">
            <div class="flex justify-between items-center mb-3">
                <h3 class="text-sm font-bold text-[#c9d1d9]">"Item Detail"</h3>
                <button
                    class="text-xs text-[#8b949e] hover:text-[#c9d1d9] cursor-pointer"
                    on:click=move |_| on_close()
                >
                    "Close"
                </button>
            </div>

            {move || if loading.get() {
                view! { <div class="text-xs text-[#8b949e] animate-pulse">"Loading..."</div> }.into_any()
            } else {
                let guid_for_files = guid_for_detail.clone();
                view! {
                    <div>
                        {move || item_json.get().map(|json| view! {
                            <details class="mb-3">
                                <summary class="text-xs text-[#58a6ff] cursor-pointer">"Raw JSON"</summary>
                                <pre class="mt-1 text-xs text-[#8b949e] overflow-auto max-h-48 whitespace-pre-wrap">{json}</pre>
                            </details>
                        })}

                        {
                            let guid_for_files = guid_for_files.clone();
                            move || {
                                let file_list = files.get();
                                if file_list.is_empty() {
                                    None
                                } else {
                                    let guid_for_files = guid_for_files.clone();
                                    Some(view! {
                                        <div class="mb-3">
                                            <h4 class="text-xs font-medium text-[#8b949e] mb-1">"Files"</h4>
                                            {file_list.into_iter().map(|file| {
                                                let file_guid = file.guid.clone().unwrap_or_default();
                                                let file_title = file.title.clone().unwrap_or_else(|| "untitled".to_string());
                                                let item_guid = guid_for_files.clone();
                                                let download_name = file_title.clone();
                                                view! {
                                                    <div class="flex items-center gap-2 py-1">
                                                        <span class="text-xs text-[#c9d1d9]">{file_title}</span>
                                                        <span class="text-xs text-[#484f58]">{file.format.unwrap_or_default()}</span>
                                                        <button
                                                            class="text-xs text-[#58a6ff] hover:underline cursor-pointer ml-auto"
                                                            on:click=move |_| {
                                                                let item = item_guid.clone();
                                                                let file = file_guid.clone();
                                                                let name = download_name.clone();
                                                                spawn_local(async move {
                                                                    if let Ok(data) = arena::download_file(item, file, name).await {
                                                                        arena::trigger_browser_download(&data.file_name, &data.data_base64, &data.mime_type);
                                                                    }
                                                                });
                                                            }
                                                        >
                                                            "Download"
                                                        </button>
                                                    </div>
                                                }
                                            }).collect_view()}
                                        </div>
                                    })
                                }
                            }
                        }

                        {move || {
                            let rev_list = revisions.get();
                            if rev_list.is_empty() {
                                None
                            } else {
                                Some(view! {
                                    <div>
                                        <h4 class="text-xs font-medium text-[#8b949e] mb-1">"Revisions"</h4>
                                        {rev_list.into_iter().map(|rev| {
                                            view! {
                                                <div class="flex gap-4 py-1 text-xs">
                                                    <span class="text-[#c9d1d9]">{rev.revision_number.unwrap_or_default()}</span>
                                                    <span class="text-[#8b949e]">{rev.lifecycle_phase.and_then(|phase| phase.name).unwrap_or_default()}</span>
                                                    <span class="text-[#484f58]">{rev.effective_datetime.unwrap_or_default()}</span>
                                                </div>
                                            }
                                        }).collect_view()}
                                    </div>
                                })
                            }
                        }}
                    </div>
                }.into_any()
            }}
        </div>
    }
}

#[component]
pub fn BomTree() -> impl IntoView {
    let search_text = RwSignal::new(String::new());
    let root_items = RwSignal::new(Vec::<arena::ItemRow>::new());
    let selected_item = RwSignal::new(Option::<(String, String)>::None);
    let bom_lines = RwSignal::new(Vec::<BomNodeData>::new());
    let loading_search = RwSignal::new(false);
    let loading_bom = RwSignal::new(false);

    let do_search = move || {
        let query = search_text.get();
        if query.trim().is_empty() {
            return;
        }
        loading_search.set(true);
        let name_filter = Some(format!("*{}*", query.trim()));
        spawn_local(async move {
            if let Ok(json) =
                arena::search_items(None, name_filter, None, None, None, None, Some(20)).await
                && let Ok(list) = serde_json::from_str::<arena::ListResponse<arena::ItemRow>>(&json)
            {
                root_items.set(list.results.unwrap_or_default());
            }
            loading_search.set(false);
        });
    };

    let select_item = move |guid: String, label: String| {
        selected_item.set(Some((guid.clone(), label)));
        loading_bom.set(true);
        bom_lines.set(Vec::new());
        spawn_local(async move {
            if let Ok(json) = arena::get_bom(guid).await
                && let Ok(list) =
                    serde_json::from_str::<arena::ListResponse<arena::BomLineRow>>(&json)
            {
                let nodes: Vec<BomNodeData> = list
                    .results
                    .unwrap_or_default()
                    .into_iter()
                    .map(bom_line_to_node)
                    .collect();
                bom_lines.set(nodes);
            }
            loading_bom.set(false);
        });
    };

    view! {
        <div class="flex-1 flex flex-col min-h-0 p-4">
            <div class="flex gap-2 mb-4">
                <input
                    type="text"
                    class="flex-1 bg-[#0d1117] text-[#c9d1d9] border border-[#30363d] rounded-lg px-3 py-2 text-sm font-mono focus:outline-none focus:border-[#58a6ff] placeholder-[#484f58]"
                    placeholder="Search for an item to view its BOM..."
                    prop:value=move || search_text.get()
                    on:input=move |event| {
                        use web_sys::wasm_bindgen::JsCast;
                        let target = event.target().unwrap();
                        let input: web_sys::HtmlInputElement = target.unchecked_into();
                        search_text.set(input.value());
                    }
                    on:keydown=move |event: web_sys::KeyboardEvent| {
                        if event.key() == "Enter" { do_search(); }
                    }
                />
                <button
                    class="px-4 py-2 bg-[#238636] text-white text-sm rounded-lg hover:bg-[#2ea043] disabled:opacity-40 cursor-pointer"
                    on:click=move |_| do_search()
                    disabled=move || loading_search.get()
                >
                    "Search"
                </button>
            </div>

            {move || {
                let items = root_items.get();
                if items.is_empty() {
                    None
                } else {
                    Some(view! {
                        <div class="mb-4 border border-[#30363d] rounded-lg max-h-32 overflow-auto">
                            {items.into_iter().map(|item| {
                                let guid = item.guid.clone().unwrap_or_default();
                                let number = item.number.clone().unwrap_or_default();
                                let name = item.name.clone().unwrap_or_default();
                                let label = format!("{number} - {name}");
                                let select_guid = guid.clone();
                                let select_label = label.clone();
                                view! {
                                    <div
                                        class="px-3 py-1.5 text-sm hover:bg-[#161b22] cursor-pointer border-b border-[#30363d] last:border-b-0"
                                        on:click=move |_| select_item(select_guid.clone(), select_label.clone())
                                    >
                                        <span class="text-[#58a6ff]">{number}</span>
                                        " - "
                                        <span class="text-[#c9d1d9]">{name}</span>
                                    </div>
                                }
                            }).collect_view()}
                        </div>
                    })
                }
            }}

            {move || selected_item.get().map(|(_, label)| view! {
                <div class="mb-2 text-sm font-medium text-[#c9d1d9]">{format!("BOM: {label}")}</div>
            })}

            {move || if loading_bom.get() {
                Some(view! { <div class="text-xs text-[#8b949e] animate-pulse">"Loading BOM..."</div> })
            } else {
                None
            }}

            <div class="flex-1 overflow-auto">
                {move || bom_lines.get().into_iter().map(|node| {
                    bom_node_view(node, 0)
                }).collect_view()}
            </div>
        </div>
    }
}

#[derive(Clone)]
struct BomNodeData {
    guid: String,
    number: String,
    name: String,
    quantity: Option<f64>,
    ref_des: Option<String>,
    children: RwSignal<Option<Vec<BomNodeData>>>,
    expanded: RwSignal<bool>,
    loading: RwSignal<bool>,
}

fn bom_line_to_node(line: arena::BomLineRow) -> BomNodeData {
    BomNodeData {
        guid: line
            .item
            .as_ref()
            .and_then(|item| item.guid.clone())
            .unwrap_or_default(),
        number: line
            .item
            .as_ref()
            .and_then(|item| item.number.clone())
            .unwrap_or_default(),
        name: line
            .item
            .as_ref()
            .and_then(|item| item.name.clone())
            .unwrap_or_default(),
        quantity: line.quantity,
        ref_des: line.ref_des,
        children: RwSignal::new(None),
        expanded: RwSignal::new(false),
        loading: RwSignal::new(false),
    }
}

fn bom_node_view(node: BomNodeData, depth: usize) -> AnyView {
    let toggle_expand = move || {
        if node.expanded.get() {
            node.expanded.set(false);
            return;
        }
        if node.children.get().is_some() {
            node.expanded.set(true);
            return;
        }
        node.loading.set(true);
        let guid = node.guid.clone();
        let children_signal = node.children;
        let expanded_signal = node.expanded;
        let loading_signal = node.loading;
        spawn_local(async move {
            if let Ok(json) = arena::get_bom(guid).await
                && let Ok(list) =
                    serde_json::from_str::<arena::ListResponse<arena::BomLineRow>>(&json)
            {
                let nodes: Vec<BomNodeData> = list
                    .results
                    .unwrap_or_default()
                    .into_iter()
                    .map(bom_line_to_node)
                    .collect();
                children_signal.set(Some(nodes));
            } else {
                children_signal.set(Some(Vec::new()));
            }
            expanded_signal.set(true);
            loading_signal.set(false);
        });
    };

    let indent = depth * 20;
    let number = node.number.clone();
    let name = node.name.clone();
    let qty = node.quantity;
    let ref_des = node.ref_des.clone();
    let children_signal = node.children;
    let expanded_signal = node.expanded;
    let loading_signal = node.loading;
    let child_depth = depth + 1;

    view! {
        <div>
            <div
                class="flex items-center gap-2 py-1 px-2 hover:bg-[#161b22] cursor-pointer text-sm"
                style=format!("padding-left: {}px", indent + 8)
                on:click=move |_| toggle_expand()
            >
                <span class="w-4 text-center text-[#484f58]">
                    {move || if loading_signal.get() {
                        "...".to_string()
                    } else if expanded_signal.get() {
                        "▼".to_string()
                    } else {
                        "▶".to_string()
                    }}
                </span>
                <span class="text-[#58a6ff]">{number}</span>
                <span class="text-[#c9d1d9]">{name}</span>
                {qty.map(|quantity| view! { <span class="text-[#8b949e] text-xs">{format!("qty: {quantity}")}</span> })}
                {ref_des.map(|rd| view! { <span class="text-[#484f58] text-xs">{format!("ref: {rd}")}</span> })}
            </div>
            {move || {
                if expanded_signal.get() {
                    children_signal.get().map(|children| {
                        view! {
                            <div>
                                {children.into_iter().map(|child| {
                                    bom_node_view(child, child_depth)
                                }).collect_view()}
                            </div>
                        }
                    })
                } else {
                    None
                }
            }}
        </div>
    }.into_any()
}

#[component]
pub fn ChangesBrowse() -> impl IntoView {
    let search_text = RwSignal::new(String::new());
    let changes = RwSignal::new(Vec::<arena::ChangeRow>::new());
    let loading = RwSignal::new(false);
    let error = RwSignal::new(Option::<String>::None);
    let selected_guid = RwSignal::new(Option::<String>::None);
    let affected_items_json = RwSignal::new(Option::<String>::None);

    let do_search = move || {
        let query = search_text.get();
        loading.set(true);
        error.set(None);

        let title_filter = if query.trim().is_empty() {
            None
        } else {
            Some(format!("*{}*", query.trim()))
        };

        spawn_local(async move {
            match arena::search_changes(None, title_filter, None, None, None, Some(50)).await {
                Ok(json) => {
                    let parsed: Result<arena::ListResponse<arena::ChangeRow>, _> =
                        serde_json::from_str(&json);
                    match parsed {
                        Ok(list) => changes.set(list.results.unwrap_or_default()),
                        Err(parse_error) => error.set(Some(format!("Parse error: {parse_error}"))),
                    }
                }
                Err(message) => error.set(Some(message)),
            }
            loading.set(false);
        });
    };

    let on_row_click = move |guid: String| {
        selected_guid.set(Some(guid.clone()));
        affected_items_json.set(None);
        spawn_local(async move {
            if let Ok(json) = arena::get_change_affected_items(guid).await {
                affected_items_json.set(Some(json));
            }
        });
    };

    view! {
        <div class="flex-1 flex flex-col min-h-0 p-4">
            <div class="flex gap-2 mb-4">
                <input
                    type="text"
                    class="flex-1 bg-[#0d1117] text-[#c9d1d9] border border-[#30363d] rounded-lg px-3 py-2 text-sm font-mono focus:outline-none focus:border-[#58a6ff] placeholder-[#484f58]"
                    placeholder="Search changes by title..."
                    prop:value=move || search_text.get()
                    on:input=move |event| {
                        use web_sys::wasm_bindgen::JsCast;
                        let target = event.target().unwrap();
                        let input: web_sys::HtmlInputElement = target.unchecked_into();
                        search_text.set(input.value());
                    }
                    on:keydown=move |event: web_sys::KeyboardEvent| {
                        if event.key() == "Enter" { do_search(); }
                    }
                />
                <button
                    class="px-4 py-2 bg-[#238636] text-white text-sm rounded-lg hover:bg-[#2ea043] disabled:opacity-40 cursor-pointer"
                    on:click=move |_| do_search()
                    disabled=move || loading.get()
                >
                    {move || if loading.get() { "Searching..." } else { "Search" }}
                </button>
            </div>

            {move || error.get().map(|message| view! {
                <div class="mb-4 px-3 py-2 bg-[#490202] border border-[#da3633] rounded-lg text-xs text-[#f85149]">
                    {message}
                </div>
            })}

            <div class="flex-1 overflow-auto border border-[#30363d] rounded-lg">
                <table class="w-full">
                    <thead class="bg-[#161b22] sticky top-0">
                        <tr>
                            <th class="px-3 py-2 text-left text-xs font-medium text-[#8b949e]">"Number"</th>
                            <th class="px-3 py-2 text-left text-xs font-medium text-[#8b949e]">"Title"</th>
                            <th class="px-3 py-2 text-left text-xs font-medium text-[#8b949e]">"Status"</th>
                            <th class="px-3 py-2 text-left text-xs font-medium text-[#8b949e]">"Implementation"</th>
                        </tr>
                    </thead>
                    <tbody>
                        {move || changes.get().into_iter().map(|change| {
                            let guid = change.guid.clone().unwrap_or_default();
                            let guid_click = guid.clone();
                            let is_selected = move || selected_guid.get().as_deref() == Some(&guid);
                            view! {
                                <tr
                                    class=move || if is_selected() {
                                        "border-b border-[#30363d] bg-[#1f6feb33] cursor-pointer"
                                    } else {
                                        "border-b border-[#30363d] hover:bg-[#161b22] cursor-pointer"
                                    }
                                    on:click=move |_| on_row_click(guid_click.clone())
                                >
                                    <td class="px-3 py-2 text-sm text-[#58a6ff]">{change.number.unwrap_or_default()}</td>
                                    <td class="px-3 py-2 text-sm">{change.title.unwrap_or_default()}</td>
                                    <td class="px-3 py-2 text-sm text-[#8b949e]">{change.lifecycle_status.and_then(|status| status.status_type).unwrap_or_default()}</td>
                                    <td class="px-3 py-2 text-sm text-[#8b949e]">{change.implementation_status.unwrap_or_default()}</td>
                                </tr>
                            }
                        }).collect_view()}
                    </tbody>
                </table>
            </div>

            {move || affected_items_json.get().map(|json| view! {
                <div class="mt-4 border border-[#30363d] rounded-lg bg-[#161b22] p-4">
                    <h3 class="text-sm font-bold text-[#c9d1d9] mb-2">"Affected Items"</h3>
                    <pre class="text-xs text-[#8b949e] overflow-auto max-h-48 whitespace-pre-wrap">{json}</pre>
                </div>
            })}
        </div>
    }
}
