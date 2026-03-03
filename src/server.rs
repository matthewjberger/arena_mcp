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
}

#[tool_handler]
impl ServerHandler for ArenaServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::V_2024_11_05,
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            server_info: Implementation::from_build_env(),
            instructions: Some(
                "Arena PLM MCP server. Query items, BOMs, changes, suppliers, and other PLM data from Arena Solutions. Requires ARENA_EMAIL and ARENA_PASSWORD environment variables.".to_string(),
            ),
        }
    }
}

pub async fn serve() -> anyhow::Result<()> {
    let client = ArenaClient::from_env()?;
    let server = ArenaServer::new(client);
    let transport = stdio();
    server.serve(transport).await?.waiting().await?;
    Ok(())
}
