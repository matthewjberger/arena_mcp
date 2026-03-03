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
use tokio::sync::RwLock;

use crate::client::ArenaClient;
use crate::types::*;

#[derive(Clone)]
pub struct ArenaServer {
    client: Arc<RwLock<ArenaClient>>,
    tool_router: ToolRouter<Self>,
}

impl ArenaServer {
    fn new(client: ArenaClient) -> Self {
        Self {
            client: Arc::new(RwLock::new(client)),
            tool_router: Self::tool_router(),
        }
    }
}

fn to_json<T: Serialize>(value: &T) -> String {
    serde_json::to_string_pretty(value).unwrap_or_else(|_| "{}".to_string())
}

fn err_json(message: &str) -> String {
    serde_json::json!({"error": message}).to_string()
}

#[tool_router]
impl ArenaServer {
    #[tool(
        description = "Search for items in Arena PLM. Supports filtering by name, number, description, category, and lifecycle phase. Use trailing * wildcard for partial matches."
    )]
    async fn search_items(
        &self,
        params: Parameters<SearchItemsParams>,
    ) -> Result<String, McpError> {
        let client = self.client.read().await;
        match client.search_items(&params.0).await {
            Ok(result) => Ok(to_json(&result)),
            Err(error) => Ok(err_json(&error.to_string())),
        }
    }

    #[tool(description = "Get a single item by GUID with full detail including custom attributes.")]
    async fn get_item(&self, params: Parameters<GetItemParams>) -> Result<String, McpError> {
        let client = self.client.read().await;
        match client.get_item(&params.0.guid).await {
            Ok(result) => Ok(to_json(&result)),
            Err(error) => Ok(err_json(&error.to_string())),
        }
    }

    #[tool(description = "Create a new item in Arena PLM.")]
    async fn create_item(
        &self,
        params: Parameters<CreateItemParams>,
    ) -> Result<String, McpError> {
        let client = self.client.read().await;
        match client.create_item(&params.0).await {
            Ok(result) => Ok(to_json(&result)),
            Err(error) => Ok(err_json(&error.to_string())),
        }
    }

    #[tool(description = "Update an existing item in Arena PLM.")]
    async fn update_item(
        &self,
        params: Parameters<UpdateItemParams>,
    ) -> Result<String, McpError> {
        let client = self.client.read().await;
        match client.update_item(&params.0).await {
            Ok(result) => Ok(to_json(&result)),
            Err(error) => Ok(err_json(&error.to_string())),
        }
    }

    #[tool(description = "Delete an item from Arena PLM.")]
    async fn delete_item(
        &self,
        params: Parameters<DeleteItemParams>,
    ) -> Result<String, McpError> {
        let client = self.client.read().await;
        match client.delete_item(&params.0.guid).await {
            Ok(result) => Ok(to_json(&result)),
            Err(error) => Ok(err_json(&error.to_string())),
        }
    }

    #[tool(description = "Get sourcing entries for an item. Returns approved suppliers and manufacturer parts.")]
    async fn get_item_sourcing(
        &self,
        params: Parameters<GetItemSourcingParams>,
    ) -> Result<String, McpError> {
        let client = self.client.read().await;
        match client.get_item_sourcing(&params.0.guid).await {
            Ok(result) => Ok(to_json(&result)),
            Err(error) => Ok(err_json(&error.to_string())),
        }
    }

    #[tool(description = "Get compliance requirements for an item.")]
    async fn get_item_compliance(
        &self,
        params: Parameters<GetItemComplianceParams>,
    ) -> Result<String, McpError> {
        let client = self.client.read().await;
        match client.get_item_compliance(&params.0.guid).await {
            Ok(result) => Ok(to_json(&result)),
            Err(error) => Ok(err_json(&error.to_string())),
        }
    }

    #[tool(description = "Get references for an item.")]
    async fn get_item_references(
        &self,
        params: Parameters<GetItemReferencesParams>,
    ) -> Result<String, McpError> {
        let client = self.client.read().await;
        match client.get_item_references(&params.0.guid).await {
            Ok(result) => Ok(to_json(&result)),
            Err(error) => Ok(err_json(&error.to_string())),
        }
    }

    #[tool(description = "Get quality processes associated with an item.")]
    async fn get_item_quality(
        &self,
        params: Parameters<GetItemQualityParams>,
    ) -> Result<String, McpError> {
        let client = self.client.read().await;
        match client.get_item_quality(&params.0.guid).await {
            Ok(result) => Ok(to_json(&result)),
            Err(error) => Ok(err_json(&error.to_string())),
        }
    }

    #[tool(description = "Change an item's lifecycle phase (e.g. from Design to Production).")]
    async fn item_lifecycle_phase_change(
        &self,
        params: Parameters<ItemLifecyclePhaseChangeParams>,
    ) -> Result<String, McpError> {
        let client = self.client.read().await;
        match client.item_lifecycle_phase_change(&params.0).await {
            Ok(result) => Ok(to_json(&result)),
            Err(error) => Ok(err_json(&error.to_string())),
        }
    }

    #[tool(
        description = "Get the Bill of Materials (BOM) for an item. Returns child components with quantities, reference designators, and line numbers."
    )]
    async fn get_bom(&self, params: Parameters<GetBomParams>) -> Result<String, McpError> {
        let client = self.client.read().await;
        match client.get_bom(&params.0.guid).await {
            Ok(result) => Ok(to_json(&result)),
            Err(error) => Ok(err_json(&error.to_string())),
        }
    }

    #[tool(
        description = "Get where-used information for an item. Returns parent assemblies that contain this item."
    )]
    async fn get_where_used(
        &self,
        params: Parameters<GetWhereUsedParams>,
    ) -> Result<String, McpError> {
        let client = self.client.read().await;
        match client.get_where_used(&params.0.guid).await {
            Ok(result) => Ok(to_json(&result)),
            Err(error) => Ok(err_json(&error.to_string())),
        }
    }

    #[tool(description = "Add a line to an item's Bill of Materials.")]
    async fn create_bom_line(
        &self,
        params: Parameters<CreateBomLineParams>,
    ) -> Result<String, McpError> {
        let client = self.client.read().await;
        match client.create_bom_line(&params.0).await {
            Ok(result) => Ok(to_json(&result)),
            Err(error) => Ok(err_json(&error.to_string())),
        }
    }

    #[tool(description = "Update an existing BOM line (quantity, ref des, notes).")]
    async fn update_bom_line(
        &self,
        params: Parameters<UpdateBomLineParams>,
    ) -> Result<String, McpError> {
        let client = self.client.read().await;
        match client.update_bom_line(&params.0).await {
            Ok(result) => Ok(to_json(&result)),
            Err(error) => Ok(err_json(&error.to_string())),
        }
    }

    #[tool(description = "Remove a line from an item's Bill of Materials.")]
    async fn delete_bom_line(
        &self,
        params: Parameters<DeleteBomLineParams>,
    ) -> Result<String, McpError> {
        let client = self.client.read().await;
        match client.delete_bom_line(&params.0).await {
            Ok(result) => Ok(to_json(&result)),
            Err(error) => Ok(err_json(&error.to_string())),
        }
    }

    #[tool(
        description = "Search for changes (ECOs, DCOs, etc.) in Arena PLM. Filter by number, title, lifecycle status, and implementation status."
    )]
    async fn search_changes(
        &self,
        params: Parameters<SearchChangesParams>,
    ) -> Result<String, McpError> {
        let client = self.client.read().await;
        match client.search_changes(&params.0).await {
            Ok(result) => Ok(to_json(&result)),
            Err(error) => Ok(err_json(&error.to_string())),
        }
    }

    #[tool(description = "Get a single change order by GUID with full detail.")]
    async fn get_change(
        &self,
        params: Parameters<GetChangeParams>,
    ) -> Result<String, McpError> {
        let client = self.client.read().await;
        match client.get_change(&params.0.guid).await {
            Ok(result) => Ok(to_json(&result)),
            Err(error) => Ok(err_json(&error.to_string())),
        }
    }

    #[tool(
        description = "Get the items affected by a change order. Returns items with disposition and revision information."
    )]
    async fn get_change_affected_items(
        &self,
        params: Parameters<GetChangeAffectedItemsParams>,
    ) -> Result<String, McpError> {
        let client = self.client.read().await;
        match client.get_change_affected_items(&params.0.guid).await {
            Ok(result) => Ok(to_json(&result)),
            Err(error) => Ok(err_json(&error.to_string())),
        }
    }

    #[tool(description = "Create a new change order in Arena PLM.")]
    async fn create_change(
        &self,
        params: Parameters<CreateChangeParams>,
    ) -> Result<String, McpError> {
        let client = self.client.read().await;
        match client.create_change(&params.0).await {
            Ok(result) => Ok(to_json(&result)),
            Err(error) => Ok(err_json(&error.to_string())),
        }
    }

    #[tool(description = "Update an existing change order in Arena PLM.")]
    async fn update_change(
        &self,
        params: Parameters<UpdateChangeParams>,
    ) -> Result<String, McpError> {
        let client = self.client.read().await;
        match client.update_change(&params.0).await {
            Ok(result) => Ok(to_json(&result)),
            Err(error) => Ok(err_json(&error.to_string())),
        }
    }

    #[tool(description = "Change the lifecycle status of a change order (SUBMITTED, APPROVED, EFFECTIVE, COMPLETED, CANCELED, REOPENED, WITHDRAWN).")]
    async fn change_change_status(
        &self,
        params: Parameters<ChangeChangeStatusParams>,
    ) -> Result<String, McpError> {
        let client = self.client.read().await;
        match client.change_change_status(&params.0).await {
            Ok(result) => Ok(to_json(&result)),
            Err(error) => Ok(err_json(&error.to_string())),
        }
    }

    #[tool(description = "Add an item to a change order's affected items list.")]
    async fn add_change_affected_item(
        &self,
        params: Parameters<AddChangeAffectedItemParams>,
    ) -> Result<String, McpError> {
        let client = self.client.read().await;
        match client.add_change_affected_item(&params.0).await {
            Ok(result) => Ok(to_json(&result)),
            Err(error) => Ok(err_json(&error.to_string())),
        }
    }

    #[tool(description = "Remove an item from a change order's affected items list.")]
    async fn remove_change_affected_item(
        &self,
        params: Parameters<RemoveChangeAffectedItemParams>,
    ) -> Result<String, McpError> {
        let client = self.client.read().await;
        match client.remove_change_affected_item(&params.0).await {
            Ok(result) => Ok(to_json(&result)),
            Err(error) => Ok(err_json(&error.to_string())),
        }
    }

    #[tool(description = "Get files associated with a change order.")]
    async fn get_change_files(
        &self,
        params: Parameters<GetChangeFilesParams>,
    ) -> Result<String, McpError> {
        let client = self.client.read().await;
        match client.get_change_files(&params.0.guid).await {
            Ok(result) => Ok(to_json(&result)),
            Err(error) => Ok(err_json(&error.to_string())),
        }
    }

    #[tool(description = "Get implementation statuses for a change order's affected items.")]
    async fn get_change_implementation_statuses(
        &self,
        params: Parameters<GetChangeImplementationStatusesParams>,
    ) -> Result<String, McpError> {
        let client = self.client.read().await;
        match client
            .get_change_implementation_statuses(&params.0.guid)
            .await
        {
            Ok(result) => Ok(to_json(&result)),
            Err(error) => Ok(err_json(&error.to_string())),
        }
    }

    #[tool(
        description = "Get revision history for an item. Returns all revisions with status and associated changes."
    )]
    async fn get_item_revisions(
        &self,
        params: Parameters<GetItemRevisionsParams>,
    ) -> Result<String, McpError> {
        let client = self.client.read().await;
        match client.get_item_revisions(&params.0.guid).await {
            Ok(result) => Ok(to_json(&result)),
            Err(error) => Ok(err_json(&error.to_string())),
        }
    }

    #[tool(
        description = "Get files associated with an item. Returns file metadata including name, format, size, and author."
    )]
    async fn get_item_files(
        &self,
        params: Parameters<GetItemFilesParams>,
    ) -> Result<String, McpError> {
        let client = self.client.read().await;
        match client.get_item_files(&params.0.guid).await {
            Ok(result) => Ok(to_json(&result)),
            Err(error) => Ok(err_json(&error.to_string())),
        }
    }

    #[tool(description = "Get the content of a file associated with an item. Returns text content or a size summary for binary files.")]
    async fn get_item_file_content(
        &self,
        params: Parameters<GetItemFileContentParams>,
    ) -> Result<String, McpError> {
        let client = self.client.read().await;
        match client
            .get_item_file_content(&params.0.item_guid, &params.0.file_guid)
            .await
        {
            Ok(result) => Ok(result),
            Err(error) => Ok(err_json(&error.to_string())),
        }
    }

    #[tool(description = "Search for files in Arena PLM. Filter by name and category.")]
    async fn search_files(
        &self,
        params: Parameters<SearchFilesParams>,
    ) -> Result<String, McpError> {
        let client = self.client.read().await;
        match client.search_files(&params.0).await {
            Ok(result) => Ok(to_json(&result)),
            Err(error) => Ok(err_json(&error.to_string())),
        }
    }

    #[tool(description = "Get a single file by GUID with full metadata.")]
    async fn get_file(&self, params: Parameters<GetFileParams>) -> Result<String, McpError> {
        let client = self.client.read().await;
        match client.get_file(&params.0.guid).await {
            Ok(result) => Ok(to_json(&result)),
            Err(error) => Ok(err_json(&error.to_string())),
        }
    }

    #[tool(
        description = "Search for requests (change requests) in Arena PLM. Filter by number, title, and lifecycle status."
    )]
    async fn search_requests(
        &self,
        params: Parameters<SearchRequestsParams>,
    ) -> Result<String, McpError> {
        let client = self.client.read().await;
        match client.search_requests(&params.0).await {
            Ok(result) => Ok(to_json(&result)),
            Err(error) => Ok(err_json(&error.to_string())),
        }
    }

    #[tool(description = "Get a single request by GUID with full detail.")]
    async fn get_request(
        &self,
        params: Parameters<GetRequestParams>,
    ) -> Result<String, McpError> {
        let client = self.client.read().await;
        match client.get_request(&params.0.guid).await {
            Ok(result) => Ok(to_json(&result)),
            Err(error) => Ok(err_json(&error.to_string())),
        }
    }

    #[tool(description = "Create a new change request in Arena PLM.")]
    async fn create_request(
        &self,
        params: Parameters<CreateRequestParams>,
    ) -> Result<String, McpError> {
        let client = self.client.read().await;
        match client.create_request(&params.0).await {
            Ok(result) => Ok(to_json(&result)),
            Err(error) => Ok(err_json(&error.to_string())),
        }
    }

    #[tool(description = "Update an existing change request in Arena PLM.")]
    async fn update_request(
        &self,
        params: Parameters<UpdateRequestParams>,
    ) -> Result<String, McpError> {
        let client = self.client.read().await;
        match client.update_request(&params.0).await {
            Ok(result) => Ok(to_json(&result)),
            Err(error) => Ok(err_json(&error.to_string())),
        }
    }

    #[tool(description = "Change the lifecycle status of a request (SUBMITTED, DEFERRED, PROMOTED, CLOSED, UNSUBMITTED).")]
    async fn change_request_status(
        &self,
        params: Parameters<ChangeRequestStatusParams>,
    ) -> Result<String, McpError> {
        let client = self.client.read().await;
        match client.change_request_status(&params.0).await {
            Ok(result) => Ok(to_json(&result)),
            Err(error) => Ok(err_json(&error.to_string())),
        }
    }

    #[tool(description = "Get items associated with a request.")]
    async fn get_request_items(
        &self,
        params: Parameters<GetRequestItemsParams>,
    ) -> Result<String, McpError> {
        let client = self.client.read().await;
        match client.get_request_items(&params.0.guid).await {
            Ok(result) => Ok(to_json(&result)),
            Err(error) => Ok(err_json(&error.to_string())),
        }
    }

    #[tool(description = "Search for suppliers in Arena PLM. Filter by name with wildcard support.")]
    async fn search_suppliers(
        &self,
        params: Parameters<SearchSuppliersParams>,
    ) -> Result<String, McpError> {
        let client = self.client.read().await;
        match client.search_suppliers(&params.0).await {
            Ok(result) => Ok(to_json(&result)),
            Err(error) => Ok(err_json(&error.to_string())),
        }
    }

    #[tool(description = "Get a single supplier by GUID with full detail.")]
    async fn get_supplier(
        &self,
        params: Parameters<GetSupplierParams>,
    ) -> Result<String, McpError> {
        let client = self.client.read().await;
        match client.get_supplier(&params.0.guid).await {
            Ok(result) => Ok(to_json(&result)),
            Err(error) => Ok(err_json(&error.to_string())),
        }
    }

    #[tool(description = "Create a new supplier in Arena PLM.")]
    async fn create_supplier(
        &self,
        params: Parameters<CreateSupplierParams>,
    ) -> Result<String, McpError> {
        let client = self.client.read().await;
        match client.create_supplier(&params.0).await {
            Ok(result) => Ok(to_json(&result)),
            Err(error) => Ok(err_json(&error.to_string())),
        }
    }

    #[tool(description = "Update an existing supplier in Arena PLM.")]
    async fn update_supplier(
        &self,
        params: Parameters<UpdateSupplierParams>,
    ) -> Result<String, McpError> {
        let client = self.client.read().await;
        match client.update_supplier(&params.0).await {
            Ok(result) => Ok(to_json(&result)),
            Err(error) => Ok(err_json(&error.to_string())),
        }
    }

    #[tool(description = "Search for supplier items. Filter by supplier GUID.")]
    async fn search_supplier_items(
        &self,
        params: Parameters<SearchSupplierItemsParams>,
    ) -> Result<String, McpError> {
        let client = self.client.read().await;
        match client.search_supplier_items(&params.0).await {
            Ok(result) => Ok(to_json(&result)),
            Err(error) => Ok(err_json(&error.to_string())),
        }
    }

    #[tool(
        description = "Search for quality processes in Arena PLM. Filter by number, name, and status."
    )]
    async fn search_quality_processes(
        &self,
        params: Parameters<SearchQualityProcessesParams>,
    ) -> Result<String, McpError> {
        let client = self.client.read().await;
        match client.search_quality_processes(&params.0).await {
            Ok(result) => Ok(to_json(&result)),
            Err(error) => Ok(err_json(&error.to_string())),
        }
    }

    #[tool(description = "Get a single quality process by GUID with full detail.")]
    async fn get_quality_process(
        &self,
        params: Parameters<GetQualityProcessParams>,
    ) -> Result<String, McpError> {
        let client = self.client.read().await;
        match client.get_quality_process(&params.0.guid).await {
            Ok(result) => Ok(to_json(&result)),
            Err(error) => Ok(err_json(&error.to_string())),
        }
    }

    #[tool(description = "Get the steps for a quality process.")]
    async fn get_quality_process_steps(
        &self,
        params: Parameters<GetQualityProcessStepsParams>,
    ) -> Result<String, McpError> {
        let client = self.client.read().await;
        match client.get_quality_process_steps(&params.0.guid).await {
            Ok(result) => Ok(to_json(&result)),
            Err(error) => Ok(err_json(&error.to_string())),
        }
    }

    #[tool(description = "Change the status of a quality process (COMPLETE or REOPEN).")]
    async fn change_quality_status(
        &self,
        params: Parameters<ChangeQualityStatusParams>,
    ) -> Result<String, McpError> {
        let client = self.client.read().await;
        match client.change_quality_status(&params.0).await {
            Ok(result) => Ok(to_json(&result)),
            Err(error) => Ok(err_json(&error.to_string())),
        }
    }

    #[tool(description = "Search for tickets in Arena PLM. Filter by number and title.")]
    async fn search_tickets(
        &self,
        params: Parameters<SearchTicketsParams>,
    ) -> Result<String, McpError> {
        let client = self.client.read().await;
        match client.search_tickets(&params.0).await {
            Ok(result) => Ok(to_json(&result)),
            Err(error) => Ok(err_json(&error.to_string())),
        }
    }

    #[tool(description = "Get a single ticket by GUID with full detail.")]
    async fn get_ticket(
        &self,
        params: Parameters<GetTicketParams>,
    ) -> Result<String, McpError> {
        let client = self.client.read().await;
        match client.get_ticket(&params.0.guid).await {
            Ok(result) => Ok(to_json(&result)),
            Err(error) => Ok(err_json(&error.to_string())),
        }
    }

    #[tool(description = "Create a new ticket in Arena PLM from a template.")]
    async fn create_ticket(
        &self,
        params: Parameters<CreateTicketParams>,
    ) -> Result<String, McpError> {
        let client = self.client.read().await;
        match client.create_ticket(&params.0).await {
            Ok(result) => Ok(to_json(&result)),
            Err(error) => Ok(err_json(&error.to_string())),
        }
    }

    #[tool(description = "Search for training plans in Arena PLM. Filter by number and name.")]
    async fn search_training_plans(
        &self,
        params: Parameters<SearchTrainingPlansParams>,
    ) -> Result<String, McpError> {
        let client = self.client.read().await;
        match client.search_training_plans(&params.0).await {
            Ok(result) => Ok(to_json(&result)),
            Err(error) => Ok(err_json(&error.to_string())),
        }
    }

    #[tool(description = "Get a single training plan by GUID with full detail.")]
    async fn get_training_plan(
        &self,
        params: Parameters<GetTrainingPlanParams>,
    ) -> Result<String, McpError> {
        let client = self.client.read().await;
        match client.get_training_plan(&params.0.guid).await {
            Ok(result) => Ok(to_json(&result)),
            Err(error) => Ok(err_json(&error.to_string())),
        }
    }

    #[tool(description = "Get training records for a training plan.")]
    async fn get_training_plan_records(
        &self,
        params: Parameters<GetTrainingPlanRecordsParams>,
    ) -> Result<String, McpError> {
        let client = self.client.read().await;
        match client.get_training_plan_records(&params.0.guid).await {
            Ok(result) => Ok(to_json(&result)),
            Err(error) => Ok(err_json(&error.to_string())),
        }
    }

    #[tool(description = "Get all lifecycle phases configured in the Arena workspace.")]
    async fn get_lifecycle_phases(
        &self,
        _params: Parameters<EmptyParams>,
    ) -> Result<String, McpError> {
        let client = self.client.read().await;
        match client.get_lifecycle_phases().await {
            Ok(result) => Ok(to_json(&result)),
            Err(error) => Ok(err_json(&error.to_string())),
        }
    }

    #[tool(description = "Get all item categories configured in the Arena workspace.")]
    async fn get_item_categories(
        &self,
        _params: Parameters<EmptyParams>,
    ) -> Result<String, McpError> {
        let client = self.client.read().await;
        match client.get_item_categories().await {
            Ok(result) => Ok(to_json(&result)),
            Err(error) => Ok(err_json(&error.to_string())),
        }
    }

    #[tool(description = "Get all change categories configured in the Arena workspace.")]
    async fn get_change_categories(
        &self,
        _params: Parameters<EmptyParams>,
    ) -> Result<String, McpError> {
        let client = self.client.read().await;
        match client.get_change_categories().await {
            Ok(result) => Ok(to_json(&result)),
            Err(error) => Ok(err_json(&error.to_string())),
        }
    }

    #[tool(description = "Get item number formats configured in the Arena workspace.")]
    async fn get_item_number_formats(
        &self,
        _params: Parameters<EmptyParams>,
    ) -> Result<String, McpError> {
        let client = self.client.read().await;
        match client.get_item_number_formats().await {
            Ok(result) => Ok(to_json(&result)),
            Err(error) => Ok(err_json(&error.to_string())),
        }
    }
}

#[tool_handler]
impl ServerHandler for ArenaServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::V_2024_11_05,
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            server_info: Implementation::from_build_env(),
            instructions: Some(
                "Arena PLM MCP server. Query and modify items, BOMs, changes, suppliers, quality processes, requests, tickets, training plans, and settings in Arena Solutions. Requires ARENA_EMAIL and ARENA_PASSWORD environment variables.".to_string(),
            ),
        }
    }
}

pub async fn serve() -> anyhow::Result<()> {
    let client = ArenaClient::from_env()?;
    let server = ArenaServer::new(client);
    let logout_client = Arc::clone(&server.client);
    let transport = stdio();
    server.serve(transport).await?.waiting().await?;
    let client = logout_client.read().await;
    client.logout().await;
    Ok(())
}
