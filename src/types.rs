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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Supplier {
    pub guid: Option<String>,
    pub name: Option<String>,
    pub description: Option<String>,
    pub phone: Option<String>,
    pub fax: Option<String>,
    pub web: Option<String>,
    pub address_one: Option<String>,
    pub address_two: Option<String>,
    pub city: Option<String>,
    pub state: Option<String>,
    pub country: Option<String>,
    pub postal_code: Option<String>,
    pub creator: Option<UserRef>,
    pub owner: Option<UserRef>,
    pub creation_date_time: Option<String>,
    pub modified_date_time: Option<String>,
    pub url: Option<UrlRef>,
    pub additional_attributes: Option<Vec<serde_json::Value>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QualityProcess {
    pub guid: Option<String>,
    pub number: Option<String>,
    pub name: Option<String>,
    pub description: Option<String>,
    pub status: Option<String>,
    pub owner: Option<UserRef>,
    pub creator: Option<UserRef>,
    pub creation_date_time: Option<String>,
    pub modified_date_time: Option<String>,
    pub url: Option<UrlRef>,
    pub additional_attributes: Option<Vec<serde_json::Value>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Request {
    pub guid: Option<String>,
    pub number: Option<String>,
    pub title: Option<String>,
    pub description: Option<String>,
    pub category: Option<GuidRef>,
    pub creator: Option<UserRef>,
    pub owner: Option<UserRef>,
    pub lifecycle_status: Option<LifecycleStatus>,
    pub creation_date_time: Option<String>,
    pub modified_date_time: Option<String>,
    pub url: Option<UrlRef>,
    pub additional_attributes: Option<Vec<serde_json::Value>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ItemCategory {
    pub guid: Option<String>,
    pub name: Option<String>,
    pub path: Option<String>,
    pub active: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChangeCategory {
    pub guid: Option<String>,
    pub name: Option<String>,
    pub path: Option<String>,
    pub active: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NumberFormat {
    pub guid: Option<String>,
    pub prefix: Option<String>,
    pub suffix: Option<String>,
    pub length: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Ticket {
    pub guid: Option<String>,
    pub number: Option<String>,
    pub title: Option<String>,
    pub description: Option<String>,
    pub template: Option<GuidRef>,
    pub creator: Option<UserRef>,
    pub owner: Option<UserRef>,
    pub status: Option<String>,
    pub creation_date_time: Option<String>,
    pub modified_date_time: Option<String>,
    pub url: Option<UrlRef>,
    pub additional_attributes: Option<Vec<serde_json::Value>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TrainingPlan {
    pub guid: Option<String>,
    pub number: Option<String>,
    pub name: Option<String>,
    pub description: Option<String>,
    pub status: Option<String>,
    pub creator: Option<UserRef>,
    pub owner: Option<UserRef>,
    pub creation_date_time: Option<String>,
    pub modified_date_time: Option<String>,
    pub url: Option<UrlRef>,
    pub additional_attributes: Option<Vec<serde_json::Value>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TrainingRecord {
    pub guid: Option<String>,
    pub user: Option<UserRef>,
    pub status: Option<String>,
    pub completion_date_time: Option<String>,
    pub due_date_time: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SupplierItem {
    pub guid: Option<String>,
    pub supplier: Option<GuidRef>,
    pub item: Option<BomItemRef>,
    pub supplier_item_number: Option<String>,
    pub description: Option<String>,
    pub additional_attributes: Option<Vec<serde_json::Value>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QualityStep {
    pub guid: Option<String>,
    pub name: Option<String>,
    pub order: Option<i64>,
    pub status: Option<String>,
    pub owner: Option<UserRef>,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ItemSourcingEntry {
    pub guid: Option<String>,
    pub supplier: Option<GuidRef>,
    pub supplier_item_number: Option<String>,
    pub description: Option<String>,
    #[serde(rename = "type")]
    pub sourcing_type: Option<String>,
    pub additional_attributes: Option<Vec<serde_json::Value>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ComplianceRequirement {
    pub guid: Option<String>,
    pub name: Option<String>,
    pub description: Option<String>,
    pub status: Option<String>,
    pub evidence: Option<String>,
    pub additional_attributes: Option<Vec<serde_json::Value>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ItemReference {
    pub guid: Option<String>,
    pub referenced_object: Option<serde_json::Value>,
    #[serde(rename = "type")]
    pub reference_type: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileEntry {
    pub guid: Option<String>,
    pub name: Option<String>,
    pub format: Option<String>,
    pub size: Option<i64>,
    pub author: Option<UserRef>,
    pub category: Option<GuidRef>,
    pub mime_type: Option<String>,
    pub creation_date_time: Option<String>,
    pub modified_date_time: Option<String>,
    pub description: Option<String>,
    pub url: Option<UrlRef>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImplementationStatus {
    pub guid: Option<String>,
    pub item: Option<BomItemRef>,
    pub implementation_status: Option<String>,
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
pub struct CreateItemParams {
    #[schemars(description = "Item name (required)")]
    pub name: String,
    #[schemars(description = "Category GUID (required)")]
    pub category_guid: String,
    #[schemars(description = "Item number (auto-generated if omitted)")]
    pub number: Option<String>,
    #[schemars(description = "Item description")]
    pub description: Option<String>,
    #[schemars(description = "JSON array string of additional attribute objects")]
    pub additional_attributes_json: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct UpdateItemParams {
    #[schemars(description = "The GUID of the item to update")]
    pub guid: String,
    #[schemars(description = "New item name")]
    pub name: Option<String>,
    #[schemars(description = "New item number")]
    pub number: Option<String>,
    #[schemars(description = "New item description")]
    pub description: Option<String>,
    #[schemars(description = "JSON array string of additional attribute objects")]
    pub additional_attributes_json: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct DeleteItemParams {
    #[schemars(description = "The GUID of the item to delete")]
    pub guid: String,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct GetItemSourcingParams {
    #[schemars(description = "The GUID of the item")]
    pub guid: String,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct GetItemComplianceParams {
    #[schemars(description = "The GUID of the item")]
    pub guid: String,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct GetItemReferencesParams {
    #[schemars(description = "The GUID of the item")]
    pub guid: String,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct GetItemQualityParams {
    #[schemars(description = "The GUID of the item")]
    pub guid: String,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct ItemLifecyclePhaseChangeParams {
    #[schemars(description = "The GUID of the item")]
    pub item_guid: String,
    #[schemars(description = "The GUID of the target lifecycle phase")]
    pub lifecycle_phase_guid: String,
    #[schemars(description = "Optional comment for the phase change")]
    pub comment: Option<String>,
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
pub struct CreateBomLineParams {
    #[schemars(description = "The GUID of the parent item")]
    pub item_guid: String,
    #[schemars(description = "The GUID of the child item to add")]
    pub child_item_guid: String,
    #[schemars(description = "Quantity")]
    pub quantity: Option<f64>,
    #[schemars(description = "Reference designator")]
    pub ref_des: Option<String>,
    #[schemars(description = "Notes")]
    pub notes: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct UpdateBomLineParams {
    #[schemars(description = "The GUID of the parent item")]
    pub item_guid: String,
    #[schemars(description = "The GUID of the BOM line to update")]
    pub line_guid: String,
    #[schemars(description = "New quantity")]
    pub quantity: Option<f64>,
    #[schemars(description = "New reference designator")]
    pub ref_des: Option<String>,
    #[schemars(description = "New notes")]
    pub notes: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct DeleteBomLineParams {
    #[schemars(description = "The GUID of the parent item")]
    pub item_guid: String,
    #[schemars(description = "The GUID of the BOM line to delete")]
    pub line_guid: String,
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
pub struct CreateChangeParams {
    #[schemars(description = "Change title")]
    pub title: Option<String>,
    #[schemars(description = "Category GUID")]
    pub category_guid: Option<String>,
    #[schemars(description = "Change description")]
    pub description: Option<String>,
    #[schemars(description = "JSON array string of additional attribute objects")]
    pub additional_attributes_json: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct UpdateChangeParams {
    #[schemars(description = "The GUID of the change to update")]
    pub guid: String,
    #[schemars(description = "New title")]
    pub title: Option<String>,
    #[schemars(description = "New description")]
    pub description: Option<String>,
    #[schemars(description = "JSON array string of additional attribute objects")]
    pub additional_attributes_json: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct ChangeChangeStatusParams {
    #[schemars(description = "The GUID of the change")]
    pub change_guid: String,
    #[schemars(description = "Target status: SUBMITTED, APPROVED, EFFECTIVE, COMPLETED, CANCELED, REOPENED, or WITHDRAWN")]
    pub status: String,
    #[schemars(description = "Comment for the status change")]
    pub comment: Option<String>,
    #[schemars(description = "Force the status change even if validations fail")]
    pub force: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct AddChangeAffectedItemParams {
    #[schemars(description = "The GUID of the change")]
    pub change_guid: String,
    #[schemars(description = "The GUID of the item to add")]
    pub item_guid: String,
    #[schemars(description = "GUID of the new lifecycle phase for the item")]
    pub new_lifecycle_phase_guid: Option<String>,
    #[schemars(description = "New revision number for the item")]
    pub new_revision_number: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct RemoveChangeAffectedItemParams {
    #[schemars(description = "The GUID of the change")]
    pub change_guid: String,
    #[schemars(description = "The GUID of the affected item entry to remove")]
    pub affected_item_guid: String,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct GetChangeFilesParams {
    #[schemars(description = "The GUID of the change")]
    pub guid: String,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct GetChangeImplementationStatusesParams {
    #[schemars(description = "The GUID of the change")]
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
pub struct GetItemFileContentParams {
    #[schemars(description = "The GUID of the item")]
    pub item_guid: String,
    #[schemars(description = "The GUID of the file")]
    pub file_guid: String,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct SearchFilesParams {
    #[schemars(description = "Filter by file name (supports trailing * wildcard)")]
    pub name: Option<String>,
    #[schemars(description = "Filter by category GUID")]
    pub category_guid: Option<String>,
    #[schemars(description = "Result offset for pagination (default 0)")]
    pub offset: Option<i64>,
    #[schemars(description = "Max results to return (default 20, max 400)")]
    pub limit: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct GetFileParams {
    #[schemars(description = "The GUID of the file to retrieve")]
    pub guid: String,
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
pub struct GetRequestParams {
    #[schemars(description = "The GUID of the request to retrieve")]
    pub guid: String,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct CreateRequestParams {
    #[schemars(description = "Request title")]
    pub title: Option<String>,
    #[schemars(description = "Category GUID")]
    pub category_guid: Option<String>,
    #[schemars(description = "Request description")]
    pub description: Option<String>,
    #[schemars(description = "JSON array string of additional attribute objects")]
    pub additional_attributes_json: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct UpdateRequestParams {
    #[schemars(description = "The GUID of the request to update")]
    pub guid: String,
    #[schemars(description = "New title")]
    pub title: Option<String>,
    #[schemars(description = "New description")]
    pub description: Option<String>,
    #[schemars(description = "JSON array string of additional attribute objects")]
    pub additional_attributes_json: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct ChangeRequestStatusParams {
    #[schemars(description = "The GUID of the request")]
    pub request_guid: String,
    #[schemars(description = "Target status: SUBMITTED, DEFERRED, PROMOTED, CLOSED, or UNSUBMITTED")]
    pub status: String,
    #[schemars(description = "Comment for the status change")]
    pub comment: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct GetRequestItemsParams {
    #[schemars(description = "The GUID of the request")]
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
pub struct GetSupplierParams {
    #[schemars(description = "The GUID of the supplier to retrieve")]
    pub guid: String,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct CreateSupplierParams {
    #[schemars(description = "Supplier name (required)")]
    pub name: String,
    #[schemars(description = "Supplier description")]
    pub description: Option<String>,
    #[schemars(description = "JSON array string of additional attribute objects")]
    pub additional_attributes_json: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct UpdateSupplierParams {
    #[schemars(description = "The GUID of the supplier to update")]
    pub guid: String,
    #[schemars(description = "New supplier name")]
    pub name: Option<String>,
    #[schemars(description = "New supplier description")]
    pub description: Option<String>,
    #[schemars(description = "JSON array string of additional attribute objects")]
    pub additional_attributes_json: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct SearchSupplierItemsParams {
    #[schemars(description = "Filter by supplier GUID")]
    pub supplier_guid: Option<String>,
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
pub struct GetQualityProcessParams {
    #[schemars(description = "The GUID of the quality process to retrieve")]
    pub guid: String,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct GetQualityProcessStepsParams {
    #[schemars(description = "The GUID of the quality process")]
    pub guid: String,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct ChangeQualityStatusParams {
    #[schemars(description = "The GUID of the quality process")]
    pub quality_guid: String,
    #[schemars(description = "Target status: COMPLETE or REOPEN")]
    pub status: String,
    #[schemars(description = "Comment for the status change")]
    pub comment: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct SearchTicketsParams {
    #[schemars(description = "Filter by ticket number")]
    pub number: Option<String>,
    #[schemars(description = "Filter by title")]
    pub title: Option<String>,
    #[schemars(description = "Result offset for pagination (default 0)")]
    pub offset: Option<i64>,
    #[schemars(description = "Max results to return (default 20, max 400)")]
    pub limit: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct GetTicketParams {
    #[schemars(description = "The GUID of the ticket to retrieve")]
    pub guid: String,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct CreateTicketParams {
    #[schemars(description = "Template GUID (required)")]
    pub template_guid: String,
    #[schemars(description = "Ticket title")]
    pub title: Option<String>,
    #[schemars(description = "Ticket description")]
    pub description: Option<String>,
    #[schemars(description = "JSON array string of additional attribute objects")]
    pub additional_attributes_json: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct SearchTrainingPlansParams {
    #[schemars(description = "Filter by number")]
    pub number: Option<String>,
    #[schemars(description = "Filter by name")]
    pub name: Option<String>,
    #[schemars(description = "Result offset for pagination (default 0)")]
    pub offset: Option<i64>,
    #[schemars(description = "Max results to return (default 20, max 400)")]
    pub limit: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct GetTrainingPlanParams {
    #[schemars(description = "The GUID of the training plan to retrieve")]
    pub guid: String,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct GetTrainingPlanRecordsParams {
    #[schemars(description = "The GUID of the training plan")]
    pub guid: String,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct EmptyParams {}
