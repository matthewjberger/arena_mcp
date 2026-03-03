use rmcp::schemars::{self, JsonSchema};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workspace_id: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LoginResponse {
    pub arena_session_id: String,
    pub workspace_id: Option<i64>,
    pub workspace_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArenaListResponse<T> {
    pub count: Option<i64>,
    pub results: Vec<T>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GuidRef {
    pub guid: Option<String>,
    pub name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UrlRef {
    pub api: Option<String>,
    pub app: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserRef {
    pub guid: Option<String>,
    pub email: Option<String>,
    pub full_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Item {
    pub guid: Option<String>,
    pub name: Option<String>,
    pub number: Option<String>,
    pub revision_number: Option<String>,
    pub description: Option<String>,
    pub assembly_type: Option<String>,
    pub category: Option<GuidRef>,
    pub lifecycle_phase: Option<GuidRef>,
    pub owner: Option<UserRef>,
    pub creator: Option<UserRef>,
    pub creation_date_time: Option<String>,
    pub modified_date_time: Option<String>,
    pub effective_date_time: Option<String>,
    pub modified_bom: Option<bool>,
    pub modified_files: Option<bool>,
    pub modified_specs: Option<bool>,
    pub modified_sourcing: Option<bool>,
    pub uom: Option<String>,
    pub procurement_type: Option<String>,
    pub url: Option<UrlRef>,
    pub additional_attributes: Option<Vec<serde_json::Value>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BomLine {
    pub guid: Option<String>,
    pub line_number: Option<i64>,
    pub quantity: Option<f64>,
    pub ref_des: Option<String>,
    pub notes: Option<String>,
    pub item: Option<BomItemRef>,
    pub additional_attributes: Option<Vec<serde_json::Value>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BomItemRef {
    pub guid: Option<String>,
    pub name: Option<String>,
    pub number: Option<String>,
    pub revision_number: Option<String>,
    pub revision_status: Option<String>,
    pub url: Option<UrlRef>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WhereUsedEntry {
    pub guid: Option<String>,
    pub item: Option<BomItemRef>,
    pub line_number: Option<i64>,
    pub quantity: Option<f64>,
    pub ref_des: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Change {
    pub guid: Option<String>,
    pub number: Option<String>,
    pub title: Option<String>,
    pub description: Option<String>,
    pub category: Option<GuidRef>,
    pub creator: Option<UserRef>,
    pub owner: Option<UserRef>,
    pub lifecycle_status: Option<LifecycleStatus>,
    pub implementation_status: Option<String>,
    pub effective_date_time: Option<String>,
    pub creation_date_time: Option<String>,
    pub modified_date_time: Option<String>,
    pub effectivity_type: Option<String>,
    pub url: Option<UrlRef>,
    pub additional_attributes: Option<Vec<serde_json::Value>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LifecycleStatus {
    #[serde(rename = "type")]
    pub status_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChangeAffectedItem {
    pub guid: Option<String>,
    pub item: Option<BomItemRef>,
    pub disposition_attribute: Option<serde_json::Value>,
    pub new_lifecycle_phase: Option<GuidRef>,
    pub new_revision_number: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ItemRevision {
    pub guid: Option<String>,
    pub number: Option<String>,
    pub status: Option<i64>,
    pub lifecycle_phase: Option<GuidRef>,
    pub change: Option<RevisionChangeRef>,
    pub creation_date_time: Option<String>,
    pub effective_date_time: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RevisionChangeRef {
    pub guid: Option<String>,
    pub number: Option<String>,
    pub creation_date_time: Option<String>,
    pub effective_date_time: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ItemFile {
    pub guid: Option<String>,
    pub file: Option<FileRef>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileRef {
    pub guid: Option<String>,
    pub name: Option<String>,
    pub format: Option<String>,
    pub size: Option<i64>,
    pub creation_date_time: Option<String>,
    pub author: Option<UserRef>,
    pub category: Option<GuidRef>,
    pub mime_type: Option<String>,
    pub is_latest_edition: Option<bool>,
    pub is_primary: Option<bool>,
    pub is_checked_out: Option<bool>,
    pub is_locked: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LifecyclePhase {
    pub guid: Option<String>,
    pub name: Option<String>,
    pub short_name: Option<String>,
    pub stage: Option<String>,
    pub active: Option<bool>,
    pub used: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct SearchItemsParams {
    #[schemars(description = "Filter by item number (supports trailing * wildcard)")]
    pub number: Option<String>,
    #[schemars(description = "Filter by item name (supports trailing * wildcard)")]
    pub name: Option<String>,
    #[schemars(description = "Filter by description (supports trailing * wildcard)")]
    pub description: Option<String>,
    #[schemars(description = "Filter by category GUID")]
    pub category_guid: Option<String>,
    #[schemars(description = "Filter by lifecycle phase GUID")]
    pub lifecycle_phase_guid: Option<String>,
    #[schemars(description = "Result offset for pagination (default 0)")]
    pub offset: Option<i64>,
    #[schemars(description = "Max results to return (default 20, max 400)")]
    pub limit: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct GetItemParams {
    #[schemars(description = "The GUID of the item to retrieve")]
    pub guid: String,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct GetBomParams {
    #[schemars(description = "The GUID of the item whose BOM to retrieve")]
    pub guid: String,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct GetWhereUsedParams {
    #[schemars(description = "The GUID of the item to find parent assemblies for")]
    pub guid: String,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct SearchChangesParams {
    #[schemars(description = "Filter by change number")]
    pub number: Option<String>,
    #[schemars(description = "Filter by change title")]
    pub title: Option<String>,
    #[schemars(description = "Filter by lifecycle status type (OPEN_AND_UNLOCKED, EFFECTIVE, APPROVED, etc.)")]
    pub lifecycle_status: Option<String>,
    #[schemars(description = "Filter by implementation status (NOT_STARTED, IN_PROGRESS, NEEDS_ATTENTION, DONE)")]
    pub implementation_status: Option<String>,
    #[schemars(description = "Result offset for pagination (default 0)")]
    pub offset: Option<i64>,
    #[schemars(description = "Max results to return (default 20, max 400)")]
    pub limit: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct GetChangeParams {
    #[schemars(description = "The GUID of the change to retrieve")]
    pub guid: String,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct GetChangeAffectedItemsParams {
    #[schemars(description = "The GUID of the change whose affected items to retrieve")]
    pub guid: String,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct GetItemRevisionsParams {
    #[schemars(description = "The GUID of the item whose revisions to retrieve")]
    pub guid: String,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct GetItemFilesParams {
    #[schemars(description = "The GUID of the item whose files to retrieve")]
    pub guid: String,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct SearchSuppliersParams {
    #[schemars(description = "Filter by supplier name (supports trailing * wildcard)")]
    pub name: Option<String>,
    #[schemars(description = "Result offset for pagination (default 0)")]
    pub offset: Option<i64>,
    #[schemars(description = "Max results to return (default 20, max 400)")]
    pub limit: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct SearchQualityProcessesParams {
    #[schemars(description = "Filter by number")]
    pub number: Option<String>,
    #[schemars(description = "Filter by name")]
    pub name: Option<String>,
    #[schemars(description = "Filter by status (OPEN, IN_PROGRESS, COMPLETED)")]
    pub status: Option<String>,
    #[schemars(description = "Result offset for pagination (default 0)")]
    pub offset: Option<i64>,
    #[schemars(description = "Max results to return (default 20, max 400)")]
    pub limit: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct SearchRequestsParams {
    #[schemars(description = "Filter by request number")]
    pub number: Option<String>,
    #[schemars(description = "Filter by title")]
    pub title: Option<String>,
    #[schemars(description = "Filter by lifecycle status (UNSUBMITTED, SUBMITTED, DEFERRED, PROMOTED, CLOSED)")]
    pub lifecycle_status: Option<String>,
    #[schemars(description = "Result offset for pagination (default 0)")]
    pub offset: Option<i64>,
    #[schemars(description = "Max results to return (default 20, max 400)")]
    pub limit: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct EmptyParams {}
