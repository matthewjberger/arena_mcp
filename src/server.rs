use rmcp::{
    ErrorData as McpError, ServerHandler,
    handler::server::{tool::ToolRouter, wrapper::Parameters},
    model::*,
    service::ServiceExt,
    tool, tool_handler, tool_router,
    transport::stdio,
};
use serde::Serialize;
use std::sync::Arc;

use crate::client::ArenaClient;
use crate::types::*;

#[derive(Clone)]
pub struct ArenaServer {
    client: Arc<ArenaClient>,
    tool_router: ToolRouter<Self>,
}

impl ArenaServer {
    fn new(client: ArenaClient) -> Self {
        Self {
            client: Arc::new(client),
            tool_router: Self::tool_router(),
        }
    }
}

fn to_json<T: Serialize>(value: &T) -> Result<String, McpError> {
    serde_json::to_string_pretty(value).map_err(|error| McpError {
        code: ErrorCode(-32603),
        message: format!("serialization failed: {error}").into(),
        data: None,
    })
}

fn to_mcp_error(error: anyhow::Error) -> McpError {
    McpError {
        code: ErrorCode(-32603),
        message: error.to_string().into(),
        data: None,
    }
}

#[tool_router]
impl ArenaServer {
    #[tool(
        description = "Log in to Arena PLM. Must be called before using any other tools, unless credentials were provided via environment variables. Ask the user for their email and password if not provided."
    )]
    async fn login(&self, params: Parameters<LoginParams>) -> Result<String, McpError> {
        let result = self
            .client
            .authenticate(
                &params.0.email,
                &params.0.password,
                params.0.workspace_id,
                params.0.base_url.as_deref(),
            )
            .await
            .map_err(to_mcp_error)?;
        to_json(&result)
    }

    #[tool(
        description = "Search for items in Arena PLM. Supports filtering by name, number, description, category, and lifecycle phase. Use trailing * wildcard for partial matches."
    )]
    async fn search_items(
        &self,
        params: Parameters<SearchItemsParams>,
    ) -> Result<String, McpError> {
        let result = self
            .client
            .search_items(&params.0)
            .await
            .map_err(to_mcp_error)?;
        to_json(&result)
    }

    #[tool(description = "Get a single item by GUID with full detail including custom attributes.")]
    async fn get_item(&self, params: Parameters<GetItemParams>) -> Result<String, McpError> {
        let result = self
            .client
            .get_item(&params.0.guid)
            .await
            .map_err(to_mcp_error)?;
        to_json(&result)
    }

    #[tool(description = "Create a new item in Arena PLM.")]
    async fn create_item(&self, params: Parameters<CreateItemParams>) -> Result<String, McpError> {
        let result = self
            .client
            .create_item(&params.0)
            .await
            .map_err(to_mcp_error)?;
        to_json(&result)
    }

    #[tool(description = "Update an existing item in Arena PLM.")]
    async fn update_item(&self, params: Parameters<UpdateItemParams>) -> Result<String, McpError> {
        let result = self
            .client
            .update_item(&params.0)
            .await
            .map_err(to_mcp_error)?;
        to_json(&result)
    }

    #[tool(description = "Delete an item from Arena PLM.")]
    async fn delete_item(&self, params: Parameters<DeleteItemParams>) -> Result<String, McpError> {
        let result = self
            .client
            .delete_item(&params.0.guid)
            .await
            .map_err(to_mcp_error)?;
        to_json(&result)
    }

    #[tool(
        description = "Get sourcing entries for an item. Returns approved suppliers and manufacturer parts."
    )]
    async fn get_item_sourcing(
        &self,
        params: Parameters<GetItemSourcingParams>,
    ) -> Result<String, McpError> {
        let result = self
            .client
            .get_item_sourcing(&params.0.guid)
            .await
            .map_err(to_mcp_error)?;
        to_json(&result)
    }

    #[tool(description = "Get compliance requirements for an item.")]
    async fn get_item_compliance(
        &self,
        params: Parameters<GetItemComplianceParams>,
    ) -> Result<String, McpError> {
        let result = self
            .client
            .get_item_compliance(&params.0.guid)
            .await
            .map_err(to_mcp_error)?;
        to_json(&result)
    }

    #[tool(description = "Get references for an item.")]
    async fn get_item_references(
        &self,
        params: Parameters<GetItemReferencesParams>,
    ) -> Result<String, McpError> {
        let result = self
            .client
            .get_item_references(&params.0.guid)
            .await
            .map_err(to_mcp_error)?;
        to_json(&result)
    }

    #[tool(description = "Get quality processes associated with an item.")]
    async fn get_item_quality(
        &self,
        params: Parameters<GetItemQualityParams>,
    ) -> Result<String, McpError> {
        let result = self
            .client
            .get_item_quality(&params.0.guid)
            .await
            .map_err(to_mcp_error)?;
        to_json(&result)
    }

    #[tool(description = "Change an item's lifecycle phase (e.g. from Design to Production).")]
    async fn item_lifecycle_phase_change(
        &self,
        params: Parameters<ItemLifecyclePhaseChangeParams>,
    ) -> Result<String, McpError> {
        let result = self
            .client
            .item_lifecycle_phase_change(&params.0)
            .await
            .map_err(to_mcp_error)?;
        to_json(&result)
    }

    #[tool(
        description = "Get the Bill of Materials (BOM) for an item. Returns child components with quantities, reference designators, and line numbers."
    )]
    async fn get_bom(&self, params: Parameters<GetBomParams>) -> Result<String, McpError> {
        let result = self
            .client
            .get_bom(&params.0.guid)
            .await
            .map_err(to_mcp_error)?;
        to_json(&result)
    }

    #[tool(
        description = "Get where-used information for an item. Returns parent assemblies that contain this item."
    )]
    async fn get_where_used(
        &self,
        params: Parameters<GetWhereUsedParams>,
    ) -> Result<String, McpError> {
        let result = self
            .client
            .get_where_used(&params.0.guid)
            .await
            .map_err(to_mcp_error)?;
        to_json(&result)
    }

    #[tool(description = "Add a line to an item's Bill of Materials.")]
    async fn create_bom_line(
        &self,
        params: Parameters<CreateBomLineParams>,
    ) -> Result<String, McpError> {
        let result = self
            .client
            .create_bom_line(&params.0)
            .await
            .map_err(to_mcp_error)?;
        to_json(&result)
    }

    #[tool(description = "Update an existing BOM line (quantity, ref des, notes).")]
    async fn update_bom_line(
        &self,
        params: Parameters<UpdateBomLineParams>,
    ) -> Result<String, McpError> {
        let result = self
            .client
            .update_bom_line(&params.0)
            .await
            .map_err(to_mcp_error)?;
        to_json(&result)
    }

    #[tool(description = "Remove a line from an item's Bill of Materials.")]
    async fn delete_bom_line(
        &self,
        params: Parameters<DeleteBomLineParams>,
    ) -> Result<String, McpError> {
        let result = self
            .client
            .delete_bom_line(&params.0)
            .await
            .map_err(to_mcp_error)?;
        to_json(&result)
    }

    #[tool(
        description = "Search for changes (ECOs, DCOs, etc.) in Arena PLM. Filter by number, title, lifecycle status, and implementation status."
    )]
    async fn search_changes(
        &self,
        params: Parameters<SearchChangesParams>,
    ) -> Result<String, McpError> {
        let result = self
            .client
            .search_changes(&params.0)
            .await
            .map_err(to_mcp_error)?;
        to_json(&result)
    }

    #[tool(description = "Get a single change order by GUID with full detail.")]
    async fn get_change(&self, params: Parameters<GetChangeParams>) -> Result<String, McpError> {
        let result = self
            .client
            .get_change(&params.0.guid)
            .await
            .map_err(to_mcp_error)?;
        to_json(&result)
    }

    #[tool(
        description = "Get the items affected by a change order. Returns items with disposition and revision information."
    )]
    async fn get_change_affected_items(
        &self,
        params: Parameters<GetChangeAffectedItemsParams>,
    ) -> Result<String, McpError> {
        let result = self
            .client
            .get_change_affected_items(&params.0.guid)
            .await
            .map_err(to_mcp_error)?;
        to_json(&result)
    }

    #[tool(description = "Create a new change order in Arena PLM.")]
    async fn create_change(
        &self,
        params: Parameters<CreateChangeParams>,
    ) -> Result<String, McpError> {
        let result = self
            .client
            .create_change(&params.0)
            .await
            .map_err(to_mcp_error)?;
        to_json(&result)
    }

    #[tool(description = "Update an existing change order in Arena PLM.")]
    async fn update_change(
        &self,
        params: Parameters<UpdateChangeParams>,
    ) -> Result<String, McpError> {
        let result = self
            .client
            .update_change(&params.0)
            .await
            .map_err(to_mcp_error)?;
        to_json(&result)
    }

    #[tool(
        description = "Change the lifecycle status of a change order (SUBMITTED, APPROVED, EFFECTIVE, COMPLETED, CANCELED, REOPENED, WITHDRAWN)."
    )]
    async fn change_change_status(
        &self,
        params: Parameters<ChangeChangeStatusParams>,
    ) -> Result<String, McpError> {
        let result = self
            .client
            .change_change_status(&params.0)
            .await
            .map_err(to_mcp_error)?;
        to_json(&result)
    }

    #[tool(description = "Add an item to a change order's affected items list.")]
    async fn add_change_affected_item(
        &self,
        params: Parameters<AddChangeAffectedItemParams>,
    ) -> Result<String, McpError> {
        let result = self
            .client
            .add_change_affected_item(&params.0)
            .await
            .map_err(to_mcp_error)?;
        to_json(&result)
    }

    #[tool(description = "Remove an item from a change order's affected items list.")]
    async fn remove_change_affected_item(
        &self,
        params: Parameters<RemoveChangeAffectedItemParams>,
    ) -> Result<String, McpError> {
        let result = self
            .client
            .remove_change_affected_item(&params.0)
            .await
            .map_err(to_mcp_error)?;
        to_json(&result)
    }

    #[tool(description = "Get files associated with a change order.")]
    async fn get_change_files(
        &self,
        params: Parameters<GetChangeFilesParams>,
    ) -> Result<String, McpError> {
        let result = self
            .client
            .get_change_files(&params.0.guid)
            .await
            .map_err(to_mcp_error)?;
        to_json(&result)
    }

    #[tool(description = "Get implementation statuses for a change order's affected items.")]
    async fn get_change_implementation_statuses(
        &self,
        params: Parameters<GetChangeImplementationStatusesParams>,
    ) -> Result<String, McpError> {
        let result = self
            .client
            .get_change_implementation_statuses(&params.0.guid)
            .await
            .map_err(to_mcp_error)?;
        to_json(&result)
    }

    #[tool(
        description = "Get revision history for an item. Returns all revisions with status and associated changes."
    )]
    async fn get_item_revisions(
        &self,
        params: Parameters<GetItemRevisionsParams>,
    ) -> Result<String, McpError> {
        let result = self
            .client
            .get_item_revisions(&params.0.guid)
            .await
            .map_err(to_mcp_error)?;
        to_json(&result)
    }

    #[tool(
        description = "Get files associated with an item. Returns file metadata including name, format, size, and author."
    )]
    async fn get_item_files(
        &self,
        params: Parameters<GetItemFilesParams>,
    ) -> Result<String, McpError> {
        let result = self
            .client
            .get_item_files(&params.0.guid)
            .await
            .map_err(to_mcp_error)?;
        to_json(&result)
    }

    #[tool(
        description = "Get the content of a file associated with an item. Returns JSON with content_type, encoding (text or base64), data, and size_bytes. Binary files (CAD, PDF, images) are base64-encoded."
    )]
    async fn get_item_file_content(
        &self,
        params: Parameters<GetItemFileContentParams>,
    ) -> Result<String, McpError> {
        let result = self
            .client
            .get_item_file_content(&params.0.item_guid, &params.0.file_guid)
            .await
            .map_err(to_mcp_error)?;
        to_json(&result)
    }

    #[tool(description = "Search for files in Arena PLM. Filter by name and category.")]
    async fn search_files(
        &self,
        params: Parameters<SearchFilesParams>,
    ) -> Result<String, McpError> {
        let result = self
            .client
            .search_files(&params.0)
            .await
            .map_err(to_mcp_error)?;
        to_json(&result)
    }

    #[tool(description = "Get a single file by GUID with full metadata.")]
    async fn get_file(&self, params: Parameters<GetFileParams>) -> Result<String, McpError> {
        let result = self
            .client
            .get_file(&params.0.guid)
            .await
            .map_err(to_mcp_error)?;
        to_json(&result)
    }

    #[tool(
        description = "Search for requests (change requests) in Arena PLM. Filter by number, title, and lifecycle status."
    )]
    async fn search_requests(
        &self,
        params: Parameters<SearchRequestsParams>,
    ) -> Result<String, McpError> {
        let result = self
            .client
            .search_requests(&params.0)
            .await
            .map_err(to_mcp_error)?;
        to_json(&result)
    }

    #[tool(description = "Get a single request by GUID with full detail.")]
    async fn get_request(&self, params: Parameters<GetRequestParams>) -> Result<String, McpError> {
        let result = self
            .client
            .get_request(&params.0.guid)
            .await
            .map_err(to_mcp_error)?;
        to_json(&result)
    }

    #[tool(description = "Create a new change request in Arena PLM.")]
    async fn create_request(
        &self,
        params: Parameters<CreateRequestParams>,
    ) -> Result<String, McpError> {
        let result = self
            .client
            .create_request(&params.0)
            .await
            .map_err(to_mcp_error)?;
        to_json(&result)
    }

    #[tool(description = "Update an existing change request in Arena PLM.")]
    async fn update_request(
        &self,
        params: Parameters<UpdateRequestParams>,
    ) -> Result<String, McpError> {
        let result = self
            .client
            .update_request(&params.0)
            .await
            .map_err(to_mcp_error)?;
        to_json(&result)
    }

    #[tool(
        description = "Change the lifecycle status of a request (SUBMITTED, DEFERRED, PROMOTED, CLOSED, UNSUBMITTED)."
    )]
    async fn change_request_status(
        &self,
        params: Parameters<ChangeRequestStatusParams>,
    ) -> Result<String, McpError> {
        let result = self
            .client
            .change_request_status(&params.0)
            .await
            .map_err(to_mcp_error)?;
        to_json(&result)
    }

    #[tool(description = "Get items associated with a request.")]
    async fn get_request_items(
        &self,
        params: Parameters<GetRequestItemsParams>,
    ) -> Result<String, McpError> {
        let result = self
            .client
            .get_request_items(&params.0.guid)
            .await
            .map_err(to_mcp_error)?;
        to_json(&result)
    }

    #[tool(
        description = "Search for suppliers in Arena PLM. Filter by name with wildcard support."
    )]
    async fn search_suppliers(
        &self,
        params: Parameters<SearchSuppliersParams>,
    ) -> Result<String, McpError> {
        let result = self
            .client
            .search_suppliers(&params.0)
            .await
            .map_err(to_mcp_error)?;
        to_json(&result)
    }

    #[tool(description = "Get a single supplier by GUID with full detail.")]
    async fn get_supplier(
        &self,
        params: Parameters<GetSupplierParams>,
    ) -> Result<String, McpError> {
        let result = self
            .client
            .get_supplier(&params.0.guid)
            .await
            .map_err(to_mcp_error)?;
        to_json(&result)
    }

    #[tool(description = "Create a new supplier in Arena PLM.")]
    async fn create_supplier(
        &self,
        params: Parameters<CreateSupplierParams>,
    ) -> Result<String, McpError> {
        let result = self
            .client
            .create_supplier(&params.0)
            .await
            .map_err(to_mcp_error)?;
        to_json(&result)
    }

    #[tool(description = "Update an existing supplier in Arena PLM.")]
    async fn update_supplier(
        &self,
        params: Parameters<UpdateSupplierParams>,
    ) -> Result<String, McpError> {
        let result = self
            .client
            .update_supplier(&params.0)
            .await
            .map_err(to_mcp_error)?;
        to_json(&result)
    }

    #[tool(description = "Search for supplier items. Filter by supplier GUID.")]
    async fn search_supplier_items(
        &self,
        params: Parameters<SearchSupplierItemsParams>,
    ) -> Result<String, McpError> {
        let result = self
            .client
            .search_supplier_items(&params.0)
            .await
            .map_err(to_mcp_error)?;
        to_json(&result)
    }

    #[tool(
        description = "Search for quality processes in Arena PLM. Filter by number, name, and status."
    )]
    async fn search_quality_processes(
        &self,
        params: Parameters<SearchQualityProcessesParams>,
    ) -> Result<String, McpError> {
        let result = self
            .client
            .search_quality_processes(&params.0)
            .await
            .map_err(to_mcp_error)?;
        to_json(&result)
    }

    #[tool(description = "Get a single quality process by GUID with full detail.")]
    async fn get_quality_process(
        &self,
        params: Parameters<GetQualityProcessParams>,
    ) -> Result<String, McpError> {
        let result = self
            .client
            .get_quality_process(&params.0.guid)
            .await
            .map_err(to_mcp_error)?;
        to_json(&result)
    }

    #[tool(description = "Get the steps for a quality process.")]
    async fn get_quality_process_steps(
        &self,
        params: Parameters<GetQualityProcessStepsParams>,
    ) -> Result<String, McpError> {
        let result = self
            .client
            .get_quality_process_steps(&params.0.guid)
            .await
            .map_err(to_mcp_error)?;
        to_json(&result)
    }

    #[tool(description = "Change the status of a quality process (COMPLETE or REOPEN).")]
    async fn change_quality_status(
        &self,
        params: Parameters<ChangeQualityStatusParams>,
    ) -> Result<String, McpError> {
        let result = self
            .client
            .change_quality_status(&params.0)
            .await
            .map_err(to_mcp_error)?;
        to_json(&result)
    }

    #[tool(description = "Search for tickets in Arena PLM. Filter by number and title.")]
    async fn search_tickets(
        &self,
        params: Parameters<SearchTicketsParams>,
    ) -> Result<String, McpError> {
        let result = self
            .client
            .search_tickets(&params.0)
            .await
            .map_err(to_mcp_error)?;
        to_json(&result)
    }

    #[tool(description = "Get a single ticket by GUID with full detail.")]
    async fn get_ticket(&self, params: Parameters<GetTicketParams>) -> Result<String, McpError> {
        let result = self
            .client
            .get_ticket(&params.0.guid)
            .await
            .map_err(to_mcp_error)?;
        to_json(&result)
    }

    #[tool(description = "Create a new ticket in Arena PLM from a template.")]
    async fn create_ticket(
        &self,
        params: Parameters<CreateTicketParams>,
    ) -> Result<String, McpError> {
        let result = self
            .client
            .create_ticket(&params.0)
            .await
            .map_err(to_mcp_error)?;
        to_json(&result)
    }

    #[tool(description = "Search for training plans in Arena PLM. Filter by number and name.")]
    async fn search_training_plans(
        &self,
        params: Parameters<SearchTrainingPlansParams>,
    ) -> Result<String, McpError> {
        let result = self
            .client
            .search_training_plans(&params.0)
            .await
            .map_err(to_mcp_error)?;
        to_json(&result)
    }

    #[tool(description = "Get a single training plan by GUID with full detail.")]
    async fn get_training_plan(
        &self,
        params: Parameters<GetTrainingPlanParams>,
    ) -> Result<String, McpError> {
        let result = self
            .client
            .get_training_plan(&params.0.guid)
            .await
            .map_err(to_mcp_error)?;
        to_json(&result)
    }

    #[tool(description = "Get training records for a training plan.")]
    async fn get_training_plan_records(
        &self,
        params: Parameters<GetTrainingPlanRecordsParams>,
    ) -> Result<String, McpError> {
        let result = self
            .client
            .get_training_plan_records(&params.0.guid)
            .await
            .map_err(to_mcp_error)?;
        to_json(&result)
    }

    #[tool(description = "Get all lifecycle phases configured in the Arena workspace.")]
    async fn get_lifecycle_phases(
        &self,
        _params: Parameters<EmptyParams>,
    ) -> Result<String, McpError> {
        let result = self
            .client
            .get_lifecycle_phases()
            .await
            .map_err(to_mcp_error)?;
        to_json(&result)
    }

    #[tool(description = "Get all item categories configured in the Arena workspace.")]
    async fn get_item_categories(
        &self,
        _params: Parameters<EmptyParams>,
    ) -> Result<String, McpError> {
        let result = self
            .client
            .get_item_categories()
            .await
            .map_err(to_mcp_error)?;
        to_json(&result)
    }

    #[tool(description = "Get all change categories configured in the Arena workspace.")]
    async fn get_change_categories(
        &self,
        _params: Parameters<EmptyParams>,
    ) -> Result<String, McpError> {
        let result = self
            .client
            .get_change_categories()
            .await
            .map_err(to_mcp_error)?;
        to_json(&result)
    }

    #[tool(description = "Get item number formats configured in the Arena workspace.")]
    async fn get_item_number_formats(
        &self,
        _params: Parameters<EmptyParams>,
    ) -> Result<String, McpError> {
        let result = self
            .client
            .get_item_number_formats()
            .await
            .map_err(to_mcp_error)?;
        to_json(&result)
    }

    #[tool(
        description = "Get substitute parts for a BOM line. Returns alternate components that can replace this line item."
    )]
    async fn get_bom_substitutes(
        &self,
        params: Parameters<GetBomSubstitutesParams>,
    ) -> Result<String, McpError> {
        let result = self
            .client
            .get_bom_substitutes(&params.0.item_guid, &params.0.bom_line_guid)
            .await
            .map_err(to_mcp_error)?;
        to_json(&result)
    }

    #[tool(
        description = "Get the history/audit trail for an item. Shows all modifications over time."
    )]
    async fn get_item_history(
        &self,
        params: Parameters<GetItemParams>,
    ) -> Result<String, McpError> {
        let result = self
            .client
            .get_item_history(&params.0.guid)
            .await
            .map_err(to_mcp_error)?;
        to_json(&result)
    }

    #[tool(description = "Get pending/future changes (ECOs) that will affect an item.")]
    async fn get_item_future_changes(
        &self,
        params: Parameters<GetItemParams>,
    ) -> Result<String, McpError> {
        let result = self
            .client
            .get_item_future_changes(&params.0.guid)
            .await
            .map_err(to_mcp_error)?;
        to_json(&result)
    }

    #[tool(
        description = "Get the thumbnail image for an item. Returns JSON with content_type, encoding (text or base64), data, and size_bytes."
    )]
    async fn get_item_thumbnail(
        &self,
        params: Parameters<GetItemParams>,
    ) -> Result<String, McpError> {
        let result = self
            .client
            .get_item_thumbnail(&params.0.guid)
            .await
            .map_err(to_mcp_error)?;
        to_json(&result)
    }

    #[tool(description = "Get the history/audit trail for a change order.")]
    async fn get_change_history(
        &self,
        params: Parameters<GetChangeParams>,
    ) -> Result<String, McpError> {
        let result = self
            .client
            .get_change_history(&params.0.guid)
            .await
            .map_err(to_mcp_error)?;
        to_json(&result)
    }

    #[tool(
        description = "Get alerts (errors/warnings) for a change order. Shows validation issues before submission."
    )]
    async fn get_change_alerts(
        &self,
        params: Parameters<GetChangeParams>,
    ) -> Result<String, McpError> {
        let result = self
            .client
            .get_change_alerts(&params.0.guid)
            .await
            .map_err(to_mcp_error)?;
        to_json(&result)
    }

    #[tool(description = "Get implementation tasks for a change order.")]
    async fn get_change_implementation_tasks(
        &self,
        params: Parameters<GetChangeParams>,
    ) -> Result<String, McpError> {
        let result = self
            .client
            .get_change_implementation_tasks(&params.0.guid)
            .await
            .map_err(to_mcp_error)?;
        to_json(&result)
    }

    #[tool(description = "Get a single implementation task for a change order.")]
    async fn get_change_implementation_task(
        &self,
        params: Parameters<GetChangeImplementationTaskParams>,
    ) -> Result<String, McpError> {
        let result = self
            .client
            .get_change_implementation_task(&params.0.change_guid, &params.0.task_guid)
            .await
            .map_err(to_mcp_error)?;
        to_json(&result)
    }

    #[tool(description = "Get notes on an implementation task for a change order.")]
    async fn get_implementation_task_notes(
        &self,
        params: Parameters<GetImplementationTaskNotesParams>,
    ) -> Result<String, McpError> {
        let result = self
            .client
            .get_implementation_task_notes(&params.0.change_guid, &params.0.task_guid)
            .await
            .map_err(to_mcp_error)?;
        to_json(&result)
    }

    #[tool(
        description = "Download file content by file GUID. Returns JSON with content_type, encoding (text or base64), data, and size_bytes. Binary files are base64-encoded."
    )]
    async fn get_file_content(
        &self,
        params: Parameters<GetFileParams>,
    ) -> Result<String, McpError> {
        let result = self
            .client
            .get_file_content(&params.0.guid)
            .await
            .map_err(to_mcp_error)?;
        to_json(&result)
    }

    #[tool(description = "Get files attached to a request.")]
    async fn get_request_files(
        &self,
        params: Parameters<GetRequestParams>,
    ) -> Result<String, McpError> {
        let result = self
            .client
            .get_request_files(&params.0.guid)
            .await
            .map_err(to_mcp_error)?;
        to_json(&result)
    }

    #[tool(description = "Get changes linked to a request.")]
    async fn get_request_changes(
        &self,
        params: Parameters<GetRequestParams>,
    ) -> Result<String, McpError> {
        let result = self
            .client
            .get_request_changes(&params.0.guid)
            .await
            .map_err(to_mcp_error)?;
        to_json(&result)
    }

    #[tool(description = "Get full detail for a supplier item by GUID.")]
    async fn get_supplier_item(
        &self,
        params: Parameters<GetSupplierItemParams>,
    ) -> Result<String, McpError> {
        let result = self
            .client
            .get_supplier_item(&params.0.guid)
            .await
            .map_err(to_mcp_error)?;
        to_json(&result)
    }

    #[tool(description = "Get addresses for a supplier.")]
    async fn get_supplier_addresses(
        &self,
        params: Parameters<GetSupplierParams>,
    ) -> Result<String, McpError> {
        let result = self
            .client
            .get_supplier_addresses(&params.0.guid)
            .await
            .map_err(to_mcp_error)?;
        to_json(&result)
    }

    #[tool(description = "Get phone numbers for a supplier.")]
    async fn get_supplier_phone_numbers(
        &self,
        params: Parameters<GetSupplierParams>,
    ) -> Result<String, McpError> {
        let result = self
            .client
            .get_supplier_phone_numbers(&params.0.guid)
            .await
            .map_err(to_mcp_error)?;
        to_json(&result)
    }

    #[tool(description = "Get files associated with a supplier.")]
    async fn get_supplier_files(
        &self,
        params: Parameters<GetSupplierParams>,
    ) -> Result<String, McpError> {
        let result = self
            .client
            .get_supplier_files(&params.0.guid)
            .await
            .map_err(to_mcp_error)?;
        to_json(&result)
    }

    #[tool(description = "Get the approval status for a supplier.")]
    async fn get_supplier_approval_status(
        &self,
        params: Parameters<GetSupplierParams>,
    ) -> Result<String, McpError> {
        let result = self
            .client
            .get_supplier_approval_status(&params.0.guid)
            .await
            .map_err(to_mcp_error)?;
        to_json(&result)
    }

    #[tool(description = "Get files associated with a supplier item.")]
    async fn get_supplier_item_files(
        &self,
        params: Parameters<GetSupplierItemParams>,
    ) -> Result<String, McpError> {
        let result = self
            .client
            .get_supplier_item_files(&params.0.guid)
            .await
            .map_err(to_mcp_error)?;
        to_json(&result)
    }

    #[tool(description = "Get compliance requirements for a supplier item.")]
    async fn get_supplier_item_compliance(
        &self,
        params: Parameters<GetSupplierItemParams>,
    ) -> Result<String, McpError> {
        let result = self
            .client
            .get_supplier_item_compliance(&params.0.guid)
            .await
            .map_err(to_mcp_error)?;
        to_json(&result)
    }

    #[tool(description = "Get sourcing relationships for a supplier item.")]
    async fn get_supplier_item_sourcing(
        &self,
        params: Parameters<GetSupplierItemParams>,
    ) -> Result<String, McpError> {
        let result = self
            .client
            .get_supplier_item_sourcing(&params.0.guid)
            .await
            .map_err(to_mcp_error)?;
        to_json(&result)
    }

    #[tool(description = "Get a single step from a quality process.")]
    async fn get_quality_process_step(
        &self,
        params: Parameters<GetQualityProcessStepParams>,
    ) -> Result<String, McpError> {
        let result = self
            .client
            .get_quality_process_step(&params.0.process_guid, &params.0.step_guid)
            .await
            .map_err(to_mcp_error)?;
        to_json(&result)
    }

    #[tool(description = "Get sign-off decisions for a quality process step.")]
    async fn get_quality_step_decisions(
        &self,
        params: Parameters<GetQualityStepDecisionsParams>,
    ) -> Result<String, McpError> {
        let result = self
            .client
            .get_quality_step_decisions(&params.0.process_guid, &params.0.step_guid)
            .await
            .map_err(to_mcp_error)?;
        to_json(&result)
    }

    #[tool(description = "Get objects affected by a quality process step.")]
    async fn get_quality_step_affected_objects(
        &self,
        params: Parameters<GetQualityStepAffectedObjectsParams>,
    ) -> Result<String, McpError> {
        let result = self
            .client
            .get_quality_step_affected_objects(&params.0.process_guid, &params.0.step_guid)
            .await
            .map_err(to_mcp_error)?;
        to_json(&result)
    }

    #[tool(description = "Get items linked to a ticket.")]
    async fn get_ticket_items(
        &self,
        params: Parameters<GetTicketParams>,
    ) -> Result<String, McpError> {
        let result = self
            .client
            .get_ticket_items(&params.0.guid)
            .await
            .map_err(to_mcp_error)?;
        to_json(&result)
    }

    #[tool(description = "Get changes linked to a ticket.")]
    async fn get_ticket_changes(
        &self,
        params: Parameters<GetTicketParams>,
    ) -> Result<String, McpError> {
        let result = self
            .client
            .get_ticket_changes(&params.0.guid)
            .await
            .map_err(to_mcp_error)?;
        to_json(&result)
    }

    #[tool(description = "Get files attached to a ticket.")]
    async fn get_ticket_files(
        &self,
        params: Parameters<GetTicketParams>,
    ) -> Result<String, McpError> {
        let result = self
            .client
            .get_ticket_files(&params.0.guid)
            .await
            .map_err(to_mcp_error)?;
        to_json(&result)
    }

    #[tool(description = "Get users assigned to a training plan.")]
    async fn get_training_plan_users(
        &self,
        params: Parameters<GetTrainingPlanParams>,
    ) -> Result<String, McpError> {
        let result = self
            .client
            .get_training_plan_users(&params.0.guid)
            .await
            .map_err(to_mcp_error)?;
        to_json(&result)
    }

    #[tool(description = "Get items linked to a training plan.")]
    async fn get_training_plan_items(
        &self,
        params: Parameters<GetTrainingPlanParams>,
    ) -> Result<String, McpError> {
        let result = self
            .client
            .get_training_plan_items(&params.0.guid)
            .await
            .map_err(to_mcp_error)?;
        to_json(&result)
    }

    #[tool(description = "Get files attached to a training plan.")]
    async fn get_training_plan_files(
        &self,
        params: Parameters<GetTrainingPlanParams>,
    ) -> Result<String, McpError> {
        let result = self
            .client
            .get_training_plan_files(&params.0.guid)
            .await
            .map_err(to_mcp_error)?;
        to_json(&result)
    }

    #[tool(description = "Get file categories configured in the Arena workspace.")]
    async fn get_file_categories(
        &self,
        _params: Parameters<EmptyParams>,
    ) -> Result<String, McpError> {
        let result = self
            .client
            .get_file_categories()
            .await
            .map_err(to_mcp_error)?;
        to_json(&result)
    }

    #[tool(description = "Get attributes defined for a change category.")]
    async fn get_change_category_attributes(
        &self,
        params: Parameters<GetChangeCategorySettingsParams>,
    ) -> Result<String, McpError> {
        let result = self
            .client
            .get_change_category_attributes(&params.0.guid)
            .await
            .map_err(to_mcp_error)?;
        to_json(&result)
    }

    #[tool(description = "Get routing configurations for a change category.")]
    async fn get_change_category_routings(
        &self,
        params: Parameters<GetChangeCategorySettingsParams>,
    ) -> Result<String, McpError> {
        let result = self
            .client
            .get_change_category_routings(&params.0.guid)
            .await
            .map_err(to_mcp_error)?;
        to_json(&result)
    }

    #[tool(description = "Get request categories configured in the Arena workspace.")]
    async fn get_request_categories(
        &self,
        _params: Parameters<EmptyParams>,
    ) -> Result<String, McpError> {
        let result = self
            .client
            .get_request_categories()
            .await
            .map_err(to_mcp_error)?;
        to_json(&result)
    }

    #[tool(description = "Get item spec attribute definitions configured in the Arena workspace.")]
    async fn get_item_attributes(
        &self,
        _params: Parameters<EmptyParams>,
    ) -> Result<String, McpError> {
        let result = self
            .client
            .get_item_attributes()
            .await
            .map_err(to_mcp_error)?;
        to_json(&result)
    }

    #[tool(description = "Get BOM attribute definitions configured in the Arena workspace.")]
    async fn get_bom_attributes(
        &self,
        _params: Parameters<EmptyParams>,
    ) -> Result<String, McpError> {
        let result = self
            .client
            .get_bom_attributes()
            .await
            .map_err(to_mcp_error)?;
        to_json(&result)
    }

    #[tool(description = "Get workspace users.")]
    async fn get_users(&self, _params: Parameters<EmptyParams>) -> Result<String, McpError> {
        let result = self.client.get_users().await.map_err(to_mcp_error)?;
        to_json(&result)
    }

    #[tool(description = "Get a single workspace user by GUID.")]
    async fn get_user(&self, params: Parameters<GetUserParams>) -> Result<String, McpError> {
        let result = self
            .client
            .get_user(&params.0.guid)
            .await
            .map_err(to_mcp_error)?;
        to_json(&result)
    }

    #[tool(description = "Get user groups configured in the Arena workspace.")]
    async fn get_user_groups(&self, _params: Parameters<EmptyParams>) -> Result<String, McpError> {
        let result = self.client.get_user_groups().await.map_err(to_mcp_error)?;
        to_json(&result)
    }

    #[tool(description = "Get API usage logs for the Arena workspace.")]
    async fn get_api_usage(&self, _params: Parameters<EmptyParams>) -> Result<String, McpError> {
        let result = self.client.get_api_usage().await.map_err(to_mcp_error)?;
        to_json(&result)
    }
}

#[tool_handler]
impl ServerHandler for ArenaServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::V_2024_11_05,
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            server_info: Implementation {
                name: "arena".to_string(),
                version: env!("CARGO_PKG_VERSION").to_string(),
                title: None,
                website_url: None,
                icons: None,
            },
            instructions: Some(
                "Arena PLM MCP server. Query and modify items, BOMs, changes, suppliers, quality processes, requests, tickets, training plans, and settings in Arena Solutions. Call the login tool first with the user's email and password to authenticate. If ARENA_EMAIL and ARENA_PASSWORD environment variables are set, login happens automatically.".to_string(),
            ),
        }
    }
}

pub async fn serve() -> anyhow::Result<()> {
    let client = ArenaClient::new()?;
    let server = ArenaServer::new(client);
    let logout_client = Arc::clone(&server.client);
    let transport = stdio();
    server.serve(transport).await?.waiting().await?;
    logout_client.logout().await;
    Ok(())
}
