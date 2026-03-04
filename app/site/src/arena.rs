use std::cell::RefCell;
use std::collections::HashMap;

use arena_app_protocol::{ArenaMethod, ArenaResult, FrontendCommand};
use futures_channel::oneshot;
use serde::Deserialize;
use wasm_bindgen::JsCast;

thread_local! {
    static NEXT_REQUEST_ID: RefCell<u32> = const { RefCell::new(1) };
    static PENDING_ARENA: RefCell<HashMap<u32, oneshot::Sender<Result<String, String>>>> = RefCell::new(HashMap::new());
    static PENDING_FILES: RefCell<HashMap<u32, oneshot::Sender<Result<FileData, String>>>> = RefCell::new(HashMap::new());
}

pub struct FileData {
    pub file_name: String,
    pub data_base64: String,
    pub mime_type: String,
}

fn next_id() -> u32 {
    NEXT_REQUEST_ID.with(|cell| {
        let mut id = cell.borrow_mut();
        let current = *id;
        *id = id.wrapping_add(1);
        current
    })
}

pub async fn arena_request(method: ArenaMethod) -> Result<String, String> {
    let request_id = next_id();
    let (sender, receiver) = oneshot::channel();

    PENDING_ARENA.with(|cell| {
        cell.borrow_mut().insert(request_id, sender);
    });

    nightshade::webview::send(&FrontendCommand::ArenaRequest { request_id, method });

    receiver
        .await
        .map_err(|_| "Request cancelled".to_string())?
}

pub fn resolve_arena(request_id: u32, result: ArenaResult) {
    let sender = PENDING_ARENA.with(|cell| cell.borrow_mut().remove(&request_id));
    if let Some(sender) = sender {
        let value = match result {
            ArenaResult::Success { json } => Ok(json),
            ArenaResult::Error { message } => Err(message),
        };
        let _ = sender.send(value);
    }
}

pub async fn download_file(
    item_guid: String,
    file_guid: String,
    file_name: String,
) -> Result<FileData, String> {
    let request_id = next_id();
    let (sender, receiver) = oneshot::channel();

    PENDING_FILES.with(|cell| {
        cell.borrow_mut().insert(request_id, sender);
    });

    nightshade::webview::send(&FrontendCommand::DownloadFile {
        request_id,
        item_guid,
        file_guid,
        file_name,
    });

    receiver
        .await
        .map_err(|_| "Request cancelled".to_string())?
}

pub fn resolve_file_success(
    request_id: u32,
    file_name: String,
    data_base64: String,
    mime_type: String,
) {
    let sender = PENDING_FILES.with(|cell| cell.borrow_mut().remove(&request_id));
    if let Some(sender) = sender {
        let _ = sender.send(Ok(FileData {
            file_name,
            data_base64,
            mime_type,
        }));
    }
}

pub fn resolve_file_error(request_id: u32, message: String) {
    let sender = PENDING_FILES.with(|cell| cell.borrow_mut().remove(&request_id));
    if let Some(sender) = sender {
        let _ = sender.send(Err(message));
    }
}

pub async fn search_items(
    number: Option<String>,
    name: Option<String>,
    description: Option<String>,
    category_guid: Option<String>,
    lifecycle_phase_guid: Option<String>,
    offset: Option<i64>,
    limit: Option<i64>,
) -> Result<String, String> {
    arena_request(ArenaMethod::SearchItems {
        number,
        name,
        description,
        category_guid,
        lifecycle_phase_guid,
        offset,
        limit,
    })
    .await
}

pub async fn get_item(guid: String) -> Result<String, String> {
    arena_request(ArenaMethod::GetItem { guid }).await
}

pub async fn get_bom(guid: String) -> Result<String, String> {
    arena_request(ArenaMethod::GetBom { guid }).await
}

pub async fn search_changes(
    number: Option<String>,
    title: Option<String>,
    lifecycle_status: Option<String>,
    implementation_status: Option<String>,
    offset: Option<i64>,
    limit: Option<i64>,
) -> Result<String, String> {
    arena_request(ArenaMethod::SearchChanges {
        number,
        title,
        lifecycle_status,
        implementation_status,
        offset,
        limit,
    })
    .await
}

pub async fn get_change_affected_items(guid: String) -> Result<String, String> {
    arena_request(ArenaMethod::GetChangeAffectedItems { guid }).await
}

pub async fn get_item_files(guid: String) -> Result<String, String> {
    arena_request(ArenaMethod::GetItemFiles { guid }).await
}

pub async fn get_item_revisions(guid: String) -> Result<String, String> {
    arena_request(ArenaMethod::GetItemRevisions { guid }).await
}

#[derive(Clone, Default, Deserialize)]
pub struct GuidRef {
    pub name: Option<String>,
}

#[derive(Clone, Deserialize)]
pub struct ItemRow {
    pub guid: Option<String>,
    pub number: Option<String>,
    pub name: Option<String>,
    pub category: Option<GuidRef>,
    #[serde(rename = "lifecyclePhase")]
    pub lifecycle_phase: Option<GuidRef>,
}

#[derive(Clone, Deserialize)]
pub struct BomLineRow {
    pub item: Option<BomItemRef>,
    pub quantity: Option<f64>,
    #[serde(rename = "refDes")]
    pub ref_des: Option<String>,
}

#[derive(Clone, Deserialize)]
pub struct BomItemRef {
    pub guid: Option<String>,
    pub number: Option<String>,
    pub name: Option<String>,
}

#[derive(Clone, Deserialize)]
pub struct ChangeRow {
    pub guid: Option<String>,
    pub number: Option<String>,
    pub title: Option<String>,
    #[serde(rename = "lifecycleStatus")]
    pub lifecycle_status: Option<LifecycleStatusRef>,
    #[serde(rename = "implementationStatus")]
    pub implementation_status: Option<String>,
}

#[derive(Clone, Deserialize)]
pub struct LifecycleStatusRef {
    #[serde(rename = "type")]
    pub status_type: Option<String>,
}

#[derive(Clone, Deserialize)]
pub struct FileRow {
    pub guid: Option<String>,
    pub title: Option<String>,
    pub format: Option<String>,
}

#[derive(Clone, Deserialize)]
pub struct RevisionRow {
    #[serde(rename = "revisionNumber")]
    pub revision_number: Option<String>,
    #[serde(rename = "lifecyclePhase")]
    pub lifecycle_phase: Option<GuidRef>,
    #[serde(rename = "effectiveDateTime")]
    pub effective_datetime: Option<String>,
}

#[derive(Clone, Deserialize)]
pub struct ListResponse<T> {
    pub results: Option<Vec<T>>,
}

pub fn trigger_browser_download(file_name: &str, data_base64: &str, mime_type: &str) {
    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();

    let binary = base64_decode(data_base64);
    let uint8 = js_sys::Uint8Array::new_with_length(binary.len() as u32);
    uint8.copy_from(&binary);

    let array = js_sys::Array::new();
    array.push(&uint8.buffer());

    let options = web_sys::BlobPropertyBag::new();
    options.set_type(mime_type);

    let blob = web_sys::Blob::new_with_u8_array_sequence_and_options(&array, &options).unwrap();
    let url = web_sys::Url::create_object_url_with_blob(&blob).unwrap();

    let anchor: web_sys::HtmlAnchorElement =
        document.create_element("a").unwrap().dyn_into().unwrap();
    anchor.set_href(&url);
    anchor.set_download(file_name);
    anchor.click();

    let _ = web_sys::Url::revoke_object_url(&url);
}

fn base64_decode(input: &str) -> Vec<u8> {
    let window = web_sys::window().unwrap();
    let decoded = window.atob(input).unwrap_or_default();
    decoded.bytes().collect()
}
